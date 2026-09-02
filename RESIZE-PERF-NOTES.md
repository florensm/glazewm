# GlazeWM resize-animation performance — investigation handoff

Branch: `personal/main` (all work also cherry-picked to `feat/dcomp`).
Machine: laptop, single 3440×1440 ultrawide @ 175 Hz (frame budget **5.7 ms**).
Symptom being chased: multi-window resize animations lag, worst when one
window goes full monitor width; disabling the resize animation feels instant.

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
$env:GLAZEWM_PERF = "1"
& C:\Users\FMN\glazewm\target\release\glazewm.exe start --config C:\Users\FMN\.glzr\glazewm\config-dcomp.yaml
# reports land in ~/.glzr/glazewm/perf.log, one per animation burst
```

Repeatable benchmark (medians over many bursts, so noise doesn't fool you):

```powershell
& C:\Users\FMN\.glzr\glazewm\perf-bench.ps1 `
    -Exe C:\Users\FMN\glazewm\target\release\glazewm.exe `
    -Config C:\Users\FMN\.glzr\glazewm\config-dcomp.yaml `
    -Mode fullwidth -Bursts 6 -TargetProcess explorer -Label "control"
```

`-Mode resize` = grow/shrink one window's width. `-Mode fullwidth` = move a
window down so it spans the full monitor width (the worst case).

### Measurement traps that already bit us — read this

1. **Pin the target window** (`-TargetProcess`). The script originally picked
   the *narrowest* window, which was a different app each run. Making a heavy
   browser full-width costs far more than Explorer. This swamped real effects.
2. **The machine drifts hugely.** The identical config measured 30 ms, 87 ms,
   192 ms and 209 ms per frame at different points in one evening. **Trust
   ratios from back-to-back runs, never absolute milliseconds across time.**
   Always re-run the control immediately before/after a candidate.
3. **Don't start a second WM instance** while one is running — it fails on
   the single-instance lock and pops a modal error dialog.
4. Run-to-run variance is ~15 %, so anything under ~20 % is noise.
5. `glazewm.exe query` from PowerShell takes 1.7–10 s (process spawn +
   Defender scanning a fresh binary). **Useless as a latency probe.**

---

## 3. What the logs actually say

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

### 6.1 Stagger the real-window repositions ← **best next idea, yours**

`rd_apply` is **20.8 ms/frame from ~1.6 calls (~13 ms each)**. In
`platform_sync.rs`, the `AnimationPositionResult::Apply` arm *deliberately
omits* `SWP_ASYNCWINDOWPOS` when a surrogate is active:

```
// Only omit `SWP_ASYNCWINDOWPOS` when a surrogate is active for this
// window — adjacent windows must stay in lock-step with the overlay.
```

That makes it a **synchronous cross-process `SetWindowPos`**, which blocks the
WM thread on the *target app's* message pump. Slow apps (Outlook was measured
at 284 ms historically) stall everything.

Your proposal — only commit the focused/active window immediately, let the
others keep showing their surrogate for a few more frames — is sound, and
there is already a precedent in the codebase: `MAX_HANDOFFS_PER_TICK = 3`
caps real-window handoffs per tick for exactly this reason. The surrogate is
a live thumbnail already sitting at the final position, so a window uncloaking
one or two frames late should be invisible.

**Concretely:** cap or stagger the synchronous `Apply`-path commits the same
way handoffs are capped, prioritising the focused window. Watch for edge
desync between adjacent windows (that's what the comment is defending
against) — verify visually, not just by numbers.

### 6.2 Shorten `window_resize.duration_ms`

Doesn't reduce per-frame cost, but makes the rough patch briefer. Cheap to
try (config only, currently 350 ms).

### 6.3 Reduce animated surface area

Since we're DWM-bound: don't animate windows whose rect barely changes, or
skip the surrogate for windows below some area threshold and just snap them.

### 6.4 Investigate the `biased` select for input latency (unproven)

`main.rs` uses `tokio::select!` with `biased` and the animation tick branch
**above** mouse/keybinding/IPC. A tick arrives every 5.7 ms and takes ~50 ms,
so that branch is arguably always ready → input may be starved for the whole
animation. **This is a hypothesis, not measured** — the CLI-based latency
probe failed (see §2.5). Would need in-process instrumentation to confirm.

---

## 7. Logging we could still add

- **Wait-time for non-animation events.** Timestamp events when the listener
  pushes them, measure the delta when the main loop pops them. This is the
  only way to prove/disprove §6.4 — the "it feels delayed" symptom.
- **Split `rd_apply` per window** (with process name), to see *which* app's
  message pump is blocking us. `process_name_for_warning()` already exists in
  `resize_session.rs`.
- **A DWM-pressure signal.** Time `EndDeferWindowPos` against the number and
  total pixel area of windows in the batch, to confirm the area hypothesis
  directly.
- **Per-window surrogate area** in the report, to correlate cost with size.

---

## 8. Files and where things are

**Code (committed on `personal/main`, cherry-picked to `feat/dcomp`):**
- `packages/wm-platform/src/perf.rs` — the profiler (`GLAZEWM_PERF=1`)
- `packages/wm-platform/examples/surrogate_cost.rs` — the microbenchmark that
  killed the thumbnail-rewrite idea. Run:
  `cargo run -p wm-platform --release --example surrogate_cost`
- Fixes in `native_blur_overlay.rs`, `native_border_overlay.rs`,
  `platform_sync.rs`, `animation/manager.rs`

**Configs (`~/.glzr/glazewm/`):**
- `config-dcomp.yaml` — your original, untouched
- `config-dcomp-perf.yaml` — **the tuned one** (focused-only border)
- `config-dcomp-noborder.yaml` — all borders off, for measuring the ceiling

**Scripts (`~/.glzr/glazewm/`):**
- `perf-bench.ps1` — the repeatable benchmark

**Logs — yes, still present in `~/.glzr/glazewm/`:**
- `perf.log` (64 KB, current) — plus `perf-baseline.log`,
  `perf-before-regionfix.log`, `perf-regionfix-only.log`,
  `perf-startup.log`, `perf-startup-firstrun.log`
- ⚠️ `errors.log` is **18 MB** of WARN spam that has since been demoted to
  `debug!`. Safe to delete.

**Pre-existing unrelated test failure:** `user_config::tests::legacy_style_keys_parse`
(`direction: 'slide_top'` parses as `SlideRight`). Came in with the
force-manage merge, untouched by this work.
