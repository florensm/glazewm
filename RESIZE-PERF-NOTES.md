# GlazeWM resize-animation performance — investigation handoff

> ## WORK IN `feat/dcomp`. DO NOT EDIT `personal/main`.
>
> `personal/main` is a merge of several branches (force-manage, move-cursor,
> and others). This issue is being chased in `feat/dcomp`, which is smaller
> and easier to reason about. **Anything in `personal/main` that is not also
> in `feat/dcomp` is out of scope — don't read it, don't fix it.**
>
> - **Worktree to use:** `C:\Users\FMN\glazewm\.claude\worktrees\border-overlay`
>   (already checked out on `feat/dcomp`)
> - **Binary it builds:** `<that worktree>\target\release\glazewm.exe`
> - `C:\Users\FMN\glazewm` is the **`personal/main`** worktree — building there
>   gives you a `personal/main` binary. That is exactly how the previous
>   session drifted, so run `git rev-parse --abbrev-ref HEAD` before you edit
>   or build.

**Branch:** `feat/dcomp`, base `651143d5`. All the perf work below is on it.
**Machine:** laptop, single 3440×1440 ultrawide @ 175 Hz (frame budget **5.7 ms**).
**Symptom:** multi-window resize animations lag, worst when one window goes
full monitor width; disabling the resize animation feels instant.

---

## 0. Update — session 2 (2026-09-02)

Two of §6's ranked ideas rested on wrong premises, and the third turned out
to be a real defect. Read this before acting on §6.

1. **§6.1's premise is false.** `rd_apply` was assumed to be a *synchronous*
   cross-process `SetWindowPos` blocking on the target app's message pump.
   The profiler now records that flag per call: across every burst of the
   fullwidth benchmark, **zero** repositions were synchronous. `has_surrogate`
   is already `false` by the time the `Apply` arm runs, because the resize
   session is torn down first. Staggering "the synchronous commits" would
   stagger nothing.
2. **§6.3 already exists** as `animations.window_move.threshold_px` /
   `window_resize.threshold_px` (default 10, Manhattan distance over
   x+y+w+h, `manager.rs:should_start_new_animation`). It does nothing for the
   fullwidth case, where every window moves hundreds of pixels.
3. **§6.4 is real, was measured, and is now fixed behind a config flag.**
   Window events waited a median ~210 ms and up to ~577 ms for the main
   loop. See §9.

What `rd_apply` actually is: the **animation-completion landing**, roughly
once per window per animation, not a per-frame cost. Half its entries take
the `already_positioned` shortcut and never reach `reposition_window` at all.
Its breakdown now adds up (children 141.9 ms against a 142.1 ms parent, so
nothing is hiding):

```
  rd_apply     142.1ms / 16 calls
    rp_query     0.0ms   <-- is_minimized/is_maximized/restore: free
    rp_swp      13.5ms   <-- the SetWindowPos itself
    rp_visible  38.4ms   <-- set_cloaked inside reposition_window
    uncloak     90.0ms   <-- set_cloaked AGAIN, in the Apply arm
```

Both large items are `DwmSetWindowAttribute(DWMWA_CLOAK)`, and for windows
that go through `reposition_window` they were **the same call made twice**.
Deduplicated — see §10. That reinforces §1 from a new angle: what remains in
`rd_apply` is one unavoidable DWM round-trip per window.

---

## 1. The one-paragraph conclusion

**GlazeWM is not CPU-bound in its own code — it is bound by DWM (the Windows
compositor).** During an animation the WM thread is ~100 % busy, but most of
that "busy" time is `EndDeferWindowPos` *blocking* while DWM absorbs the
surface changes. Proof: that same call costs **0.004 ms** in an isolated
microbenchmark and **~23 ms** inside the running WM. So the levers that work
are *how many surfaces are animated and how big they are*; micro-optimising
our own API usage does essentially nothing.

---

## 2. How to measure (this is the important part)

A frame profiler is built in, opt-in, and costs nothing when off.

```powershell
# ALWAYS the dcomp worktree's binary, never C:\Users\FMN\glazewm\target\...
$D = "C:\Users\FMN\glazewm\.claude\worktrees\border-overlay\target\release\glazewm.exe"
$env:GLAZEWM_PERF = "1"
& $D start --config C:\Users\FMN\.glzr\glazewm\config-dcomp.yaml
# reports land in ~/.glzr/glazewm/perf.log, one per animation burst
```

Repeatable benchmark (medians over many bursts, so noise doesn't fool you):

```powershell
& C:\Users\FMN\.glzr\glazewm\perf-bench.ps1 `
    -Exe $D `
    -Config C:\Users\FMN\.glzr\glazewm\config-dcomp.yaml `
    -Mode fullwidth -Bursts 6 -TargetProcess msedge -Label "control"
```

`-Mode resize` = grow/shrink one window's width. `-Mode fullwidth` = move a
window down so it spans the full monitor width (the worst case).

### Measurement traps that already bit us — read this

1. **Pin the target window** (`-TargetProcess`). The script originally picked
   the *narrowest* window, which was a different app each run. Making a heavy
   browser full-width costs far more than a light app. This swamped real
   effects. The script now aborts if the named process isn't tiled — pick any
   app that will still be open across all your runs and use it throughout.
2. **The machine drifts hugely.** The identical config measured 30 ms, 87 ms,
   192 ms and 209 ms per frame at different points in one evening. **Trust
   ratios from back-to-back runs, never absolute milliseconds across time.**
   Always re-run the control immediately before/after a candidate.
3. **Don't start a second WM instance** while one is running — it fails on
   the single-instance lock and pops a modal error dialog.
4. Run-to-run variance is ~15 %, so anything under ~20 % is noise.
5. `glazewm.exe query` from PowerShell takes 1.7–10 s (process spawn +
   Defender scanning a fresh binary). **Useless as a latency probe.**
6. Today the same benchmark reported **~12 ms/frame and ~95 frames per
   burst**, against 57 ms and 6 frames in session 1 — a ~5× swing in the
   machine's state, not a code change. Trap 2 is not a footnote; it is the
   main hazard.

### What the report contains now

Beyond the stage tree and the cross-cutting section:

- **`rd_apply by process`** — every real-window reposition, keyed by the
  owning process and by whether the call was `[sync]` or `[async]`, averaged
  **per call** (a reposition is per-window, so a per-frame average would be
  meaningless).
- **`event queue wait`** — per event kind: how long each event sat in its
  channel before the main loop serviced it. Enqueue timestamps are paired
  FIFO with dequeues; if that pairing ever slips, the row is marked
  **`SUSPECT`** rather than quietly printing a skewed number.
- **`rp_query` / `rp_swp` / `rp_visible` / `uncloak`** — `rd_apply` split
  into the actual Win32/DWM calls underneath it.

Same arithmetic discipline as §5 applies: check the children sum to their
parent before believing any of it.

---

## 3. What the logs actually say

> **Provenance:** the detailed stage tables below were captured on a
> `personal/main` build (the previous session's drift). The headline finding
> was afterwards **re-confirmed on a `feat/dcomp` build**, back to back, 9
> windows, `-Mode fullwidth`:
>
> | dcomp build | `tick` ms/frame | frames/burst |
> |---|---|---|
> | borders on all windows | 64.1 | 11 |
> | border on focused window only | **37.6** | **19** |
>
> Same shape, same ~1.7–2× ratio. The stage *proportions* below hold on dcomp;
> re-measure absolutes on dcomp before quoting any specific number.

Corrected report, 8 animating windows, one driven to full width. Indented
stages nest inside their parent; the cross-cutting section is counted inside
the tree above, not in addition to it.

```
tick                  57.22 ms/frame     <-- vs a 5.7 ms budget
  platform_sync       49.51
    redraw            48.74
      redraw_prep      0.34
      redraw_loop     22.10
        anim_step      0.02   <-- our animation math is FREE
        rd_frozen      0.00
        rd_apply      20.82   <-- ~13 ms per call, only ~1.6 calls/frame
      surrogate_flush 21.10
      session_overlays 3.92
  cleanup              5.05
-- called from several parents, already counted above --
  batch_commit        22.95   <-- EndDeferWindowPos, ~40 % of the tick
  dwm_flush            3.45
  pre_commit           2.57
  ovl_region           1.47
```

**Validation:** `tick` totalled 572.2 ms against an independently measured
572.6 ms of wall clock — the thread is ~99.9 % saturated, and every child fits
inside its parent. (Earlier reports did *not* validate — see §5.)

### The cost model

- **Everything expensive is a Win32/DWM call.** Our logic is ~0.02 ms/frame.
- Cost scales with **number of windows moved per frame**, ~1.6 ms each *in situ*
  (but 0.0003 ms in isolation — the difference is DWM back-pressure).
- A border overlay is a *second* window per managed window, so enabling it
  roughly doubles the moves. That is why it costs ~40–47 % of the tick.
- Full-width on a 3440 px panel is worst because surface area is largest.

---

## 4. What we tried

### Landed (committed, measured)

| # | Change | Effect |
|---|---|---|
| 1 | `WS_EX_TRANSPARENT` on blur/border overlays | **Fixed the loading cursor.** Overlay windows live on a thread with no message pump, so Windows saw them as hung; hit-tests during a resize landed on them. |
| 2 | One `DwmFlush` per relayout instead of per window | Removed N−1 full composition stalls at animation start |
| 3 | `SetWindowRgn(..., bRedraw=FALSE)` on the composition path | **Biggest single win, ~2×.** The forced repaint bought nothing (no redirection surface) and ran per window per frame |
| 4 | Defer a session's first fade step instead of `DwmFlush` | `tick` 32.1 → 23.7 ms, flush calls 6 → 4 |
| 5 | Full profiler + correctness fixes | The reason any of this is knowable |

### Config finding (no code change)

| config | tick ms/frame | frames/burst |
|---|---|---|
| borders on all windows | 191.9 / 209.4 | 6 |
| **border on focused window only** | **95.7** | 10 |
| focused-only + square corners | 94.7 | 9 |
| no borders at all (ceiling) | 87.3 | 10 |

→ **`config-dcomp-perf.yaml`** = your config with `other_windows.border`
disabled. 2.1× faster, captures ~95 % of the ceiling, keeps the border that
carries information. Square corners add nothing.

### Tried and rejected (all reverted — don't redo these)

| Idea | Why rejected |
|---|---|
| **Tick backpressure** (only tick as fast as the WM can service) | Measured **worse**: 53 → 82 ms/frame. Fewer, larger deltas are harder for DWM than more, smaller ones. Pipelining beats pacing. |
| **Merge the 3 `DeferWindowPos` transactions/frame into 1** | No change (92.4 → 95.5, noise). Cost is per-window, not per-transaction. |
| **One host window + N thumbnails instead of N surrogates** | Microbenchmarked: **~200× worse.** `DwmUpdateThumbnailProperties` ≈ 0.05 ms each; moving 12 windows ≈ 0.004 ms total. |
| **Nine-grid hollow border** (drop the GDI region) | `ovl_region` is only ~12 % of the tick; not worth square inner corners. |

---

## 5. Instrumentation bugs we found (so you trust the numbers)

The profiler was initially **wrong** in two ways, both caught by arithmetic:

1. `platform_sync` (572.6) + `cleanup` (172.2) exceeded `tick` (671.7).
   `update_internal` runs a *second* nested `platform_sync` in the cleanup
   block and the `Cleanup` scope wrapped it → double-counted. Fixed.
2. `batch_commit` showed 313.8 ms / 12 calls under a "parent"
   (`surrogate_flush`) of 283.5 ms / 6 calls, because `SurrogateBatch::commit`
   runs from **five** call sites. The report indented it as a child anyway.
   Now split into a nesting tree + a cross-cutting section.

**Lesson for the next session: always check that children ≤ parent and that
`tick` ≈ the wall clock in the report header.** If they don't close, the
instrumentation is lying.

---

## 6. Ideas worth trying next (ranked)

> Superseded in places by §0. 6.1 and 6.3 are struck through with the reason;
> 6.4 is done. What is actually worth doing next is §11.

### ~~6.1 Stagger the real-window repositions~~ — premise disproved

The idea was that the `Apply` arm's `SetWindowPos` is synchronous (it omits
`SWP_ASYNCWINDOWPOS` when `has_surrogate`) and blocks on the target app's
message pump, so the fix was to cap those commits per tick the way
`MAX_HANDOFFS_PER_TICK = 3` caps handoffs.

Measured: **every `rd_apply` call is async.** The resize session is already
gone when the arm runs, so `has_surrogate` is `false` and the synchronous
branch is never taken. There is nothing to stagger.

Worse, staggering would now be actively harmful: with the session torn down
there is no surrogate left standing in for the window, so deferring its
`Apply` would defer its *uncloak* — the window would be invisible for the
deferred frames rather than covered by a thumbnail. Do not implement this as
written.

### 6.2 Shorten `window_resize.duration_ms`

Doesn't reduce per-frame cost, only shortens the rough patch. Dismissed as
not worth it.

### ~~6.3 Reduce animated surface area~~ — already exists

The "don't animate windows whose rect barely changes" half is
`animations.window_move.threshold_px` / `window_resize.threshold_px`
(default 10, Manhattan distance over x+y+w+h, applied in
`AnimationManager::should_start_new_animation`). It cannot help the
fullwidth case, where every window moves hundreds of pixels.

The "skip the surrogate below an area threshold" half is unimplemented, but
would also miss: in the benchmark all eight windows are 421px × full height,
so no useful threshold excludes any of them.

### 6.4 The `biased` select starves input — **confirmed, fixed behind a flag**

Was a hypothesis; is now measured. See §9.

---

## 7. Logging we could still add

- ~~Wait-time for non-animation events.~~ **Done** — "event queue wait" in
  the report; see §9.
- ~~Split `rd_apply` per window (with process name).~~ **Done** — "rd_apply
  by process" in the report, split by sync/async; see §0 and §10.
- **A DWM-pressure signal.** Time `EndDeferWindowPos` against the number and
  total pixel area of windows in the batch, to confirm the area hypothesis
  directly.
- **Per-window surrogate area** in the report, to correlate cost with size.

---

## 8. Files and where things are

**Code — all on `feat/dcomp` (base `651143d5`), 18 commits.** It also happens
to exist on `personal/main`; ignore that copy.
- `packages/wm-platform/src/perf.rs` — the profiler (`GLAZEWM_PERF=1`),
  including the `rd_apply` per-process breakdown and the event-queue wait
- `packages/wm/src/main.rs` — `drain_platform_events` (§9)
- `packages/wm-platform/examples/surrogate_cost.rs` — the microbenchmark that
  killed the thumbnail-rewrite idea. Run:
  `cargo run -p wm-platform --release --example surrogate_cost`
  (run from the `border-overlay` worktree)
- Fixes in `native_blur_overlay.rs`, `native_border_overlay.rs`,
  `platform_sync.rs`, `animation/manager.rs`

**Configs (`~/.glzr/glazewm/`):**
- `config-dcomp.yaml` — your original, untouched
- `config-dcomp-perf.yaml` — **the tuned one** (focused-only border)
- `config-dcomp-noborder.yaml` — all borders off, for measuring the ceiling
- `config-dcomp-prioritize.yaml` — `config-dcomp.yaml` +
  `prioritize_events_over_animation: true`, for A/B'ing §9

**Scripts (`~/.glzr/glazewm/`):**
- `perf-bench.ps1` — the repeatable benchmark. It aborts with
  `TARGET PROCESS 'explorer' NOT TILED` when no File Explorer window is
  open — open one first, since FPilot is the usual file manager here and
  is a much heavier, less comparable target.

**Logs — yes, still present in `~/.glzr/glazewm/`:**
- `perf.log` (64 KB, current) — plus `perf-baseline.log`,
  `perf-before-regionfix.log`, `perf-regionfix-only.log`,
  `perf-startup.log`, `perf-startup-firstrun.log`
- ⚠️ `errors.log` is **18 MB** of WARN spam that has since been demoted to
  `debug!`. Safe to delete.

**Pre-existing unrelated test failure:** `user_config::tests::legacy_style_keys_parse`
(`direction: 'slide_top'` parses as `SlideRight`). Came in with the
force-manage merge, untouched by this work.

---

## 9. Event-queue starvation — measured and fixed (opt-in)

`main.rs`' `select!` is `biased` with the animation tick above every event
branch. A tick is ready again the instant the previous frame finishes, so for
the whole of an animation the event branches are **never reached**. The
comment defending that ordering ("so that window/input events never delay
mid-animation frames") was paying for something that costs nothing.

`general.prioritize_events_over_animation` (default **off**) drains queued
platform events before each frame, capped at `MAX_PRIORITY_EVENTS_PER_FRAME
= 8` so an event storm cannot starve the animation the other way, and checks
keybindings first (the select checks them last).

Fullwidth benchmark, 8 tiled windows, controls run either side:

| run | tick ms/frame | event wait mean | event wait worst |
|---|---|---|---|
| control | 12.87 | ~207-282 ms | ~470-695 ms |
| **candidate** | **11.66** | **~42-50 ms** | **~112-131 ms** |
| control (after) | 11.34 | ~186-200 ms | ~450-481 ms |

**~4× less waiting at no frame cost** — the candidate's tick sits between its
two controls, i.e. inside noise. Handling events as they arrive costs no more
than handling them in a burst at the end.

Caveat on what was measured: the benchmark drives the WM over IPC, so it
generates **window** events, not keypresses. Keybindings share the same
select and sit *below* the window branch, so they can only have been starved
at least as badly — but that specific number is inferred, not measured. To
measure it directly, press a keybinding during an animation with
`GLAZEWM_PERF=1` and read the `keybinding` row.

`~/.glzr/glazewm/config-dcomp-prioritize.yaml` is `config-dcomp.yaml` with
the flag on, for A/B runs.

---

## 10. The duplicate uncloak (landed, unconditional)

`reposition_window` applies the window's cloak state itself under
`HideMethod::Cloak`; the `Apply` arm then uncloaked the same visible window
again immediately afterwards. The arm's own comment already noted that method
"already calls `set_cloaked` internally" — the call ran anyway.
`reposition_window` now returns a `CloakState` and the arm only uncloaks when
it is still owed (the `already_positioned` path, where `reposition_window`
never ran).

Per `Apply` entry, back-to-back:

| stage | before | after |
|---|---|---|
| `rd_apply` | 7.10 ms | 6.54 ms (**-8%**) |
| `uncloak` | 4.78 ms | 3.83 ms (-20%) |
| `rp_visible` | 3.30 ms | 3.86 ms (**+17%**) |

Note the third row: the surviving cloak call gets *dearer*. DWM does the work
once either way and the second call was only absorbing some of the
back-pressure — another instance of §1. Net ~0.2 ms of a ~12 ms tick: real,
strictly less work for identical behaviour, but not readable in the tick
median above noise. Do not expect this alone to change how it feels.

---

## 11. What is actually worth doing next (re-ranked)

Current shape of a ~12 ms tick on the fullwidth benchmark:

```
  tick               12.09
    platform_sync     9.46
      redraw_loop     2.48   (rd_apply 2.14)
      surrogate_flush 2.70
      session_overlays 3.85  <-- now the largest single child
    cleanup           1.73
  batch_commit        5.69   (cross-cutting, inside the above)
```

1. **`session_overlays` (3.85 ms/frame).** The largest remaining item, and
   the one §3's config finding already points at — borders on all windows
   cost 2.1× versus focused-only. Worth attacking in code now that
   `rd_apply` is understood: it tracks every live session's blur/border
   overlay onto its surrogate, every frame.
2. **The keybinding latency number.** Cheap: one keypress during an
   animation with `GLAZEWM_PERF=1` closes the last gap in §9.
3. **A DWM-pressure signal** (§7), to test the surface-area hypothesis that
   §1 rests on, rather than continuing to assume it.
4. **Do not** revisit §6.1 as written, and do not expect anything from §6.2
   or §6.3 (see §0).

---
