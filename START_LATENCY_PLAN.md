# Plan: peek path that starts as fast as the old engine

Written 2026-11-12 from a reading of `cd04e3c` (`tryout/start-latency`,
= `tryout/stand-ins` `65a60de` plus the bench results) and
`perf_bench/results/2026-11-12/`. Nothing here has been seen running:
the container is Linux, so every claim about time is read off last
night's `perf.log` excerpts or the code. Read `CLAUDE.md`,
`STAND_INS_PLAN.md` ("Rules for whoever carries this out", rule 5
especially) first.

## Where the time goes (from `perf-excerpts.txt`, one `resize` burst)

"Outside frames" is the work between the command and the first tick, so
roughly what the IPC answer waits for.

| stage (outside frames) | `main` | `nopeeks` | `default` |
|---|---|---|---|
| `platform_sync` | 74 ms | 101 ms | 124 ms |
| `redraw_loop` | 11.6 | 26.6 | 33.3 |
| of which `anim_step` | 3.0 | 4.1 | **25.6** (7 peeks created) |
| of which `rd_frozen` (take-over) | 4.2 | 11.6 | 0 |
| `cloak` (flush, cloak, preposition) | 23.0 | 28.2 | 29.0 |
| `session_overlays` | **38.8** | **51.3** | 0 |
| in `redraw`, in no stage | ~0 | ~0 | **~56** |

- **Peek path**: 7 new peeks cost 25 ms. Another ~56 ms of `redraw` has
  no stage: it can only be `cover_moved_overlays`,
  `hand_over_moved_overlays`, `raise_moving`/`restack_live_overlays` and
  the `peek_raises` loop, all after `redraw_loop`. Unknown which.
- **Old path, both builds**: `session_overlays` is the largest single
  piece, and 12 ms larger on the tryout build. It is ~0.03 ms per frame
  later, so the cost is in its first pass only. Its sub-stages
  (`ovl_region`, `ovl_visual`, `batch_commit`) don't add up to it.
- **The stall** is the hand-over, confirmed by the code: a `fill` resize
  defers the new size of every window that shrinks on an axis until its
  peek lands (`resizes_at_start`, `Stretch::deferred`). At landing,
  `redraw_containers`' `Apply` arm sees `peeks.owns_window`, so
  `has_surrogate` is true and `reposition_window` resizes each of them
  with a synchronous `SetWindowPos`, one app after another, in one frame
  (`Notepad [sync]` 106 ms, `explorer` 15, `Everything` 10...). The old
  engine posts the size at the start and confirms it in `pre_commit`.

## Words (all off by default, `GLAZEWM_ISOLATE`)

| word | workstream | change |
|---|---|---|
| `asynchandback` | D | A peek's window is moved asynchronously as it is handed back |
| `asynclanding` | D | A deferred window is sent its size as its motion ends; its peek lands once it has it |
| `earlylanding` | D | As `asynclanding`, sent as the peek comes within a few px of its target |
| `keeppeeks` | C | A released peek is kept, hidden, for its window's next motion |
| B's | B | Named by B, one per fix it can point at code for |

Instrumentation (A) needs no word: it is inert without `GLAZEWM_PERF`.

## Order and file ownership

1. **A first, alone.** It adds every word above to `isolation.rs`
   (field, parse arm, doc line, test), so C and D never edit that file.
2. **B, C, D in parallel**, each branched from A's result.
3. Merge order: A, D, C, B. Expected overlaps: `peek_view.rs`
   (`PeekView::tick`: D in the `Role::Moving` arm, C where released
   entries are removed) and `platform_sync.rs` (D in the `Apply` arm; B,
   if at all, in the old path's take-over and overlay loop).

| | owns | may touch (scopes only for A) |
|---|---|---|
| A | `wm-platform/src/perf.rs`, `isolation.rs`, `perf_bench/main.rs`, `perf_bench/isolate_modes.ps1` | `platform_sync.rs`, `peek_view.rs`, `manager.rs`, `overlay_window.rs` |
| B | what its findings name, outside C's and D's functions | `isolation.rs` (its own words) |
| C | `wm-platform/src/window_peek.rs`; in `peek_view.rs`: `move_tile`, `new_entry`, the removal of released entries | |
| D | in `peek_view.rs`: `Entry::may_land`, `Role::Moving` arm of `tick`, `Landed`; `manager.rs` `queue_landed`; `platform_sync.rs` `Apply` arm | |

## A. Instrumentation

- **Input→first frame on the peek path.** Only
  `start_animation_if_needed` calls `perf::note_animation_start`. Call
  it where a peek motion starts too (`move_on_peek`, `fullscreen_on_peek`,
  `enter_on_peek`, `leave_on_peek`, `restore_on_peek`, `hold_on_peek`:
  each once it has returned `true`), with `is_resize` as the old path
  computes it. Check `perf.rs` pairs the input with the first frame that
  follows (`record_animation_start`, `roll_up_frame`): a peek frame must
  count.
- **Stages that split the pre-answer time.** New `Stage`s, in report
  order under their parents:
  - `peek_new`: `new_entry` (the `WindowPeek` and its warm-up).
  - `peek_cover`, `peek_handover`, `peek_raise`: the three blocks after
    `redraw_loop` named above.
  - `take_over`, `preposition`: inside `commit_pending_cloaks`' loop
    (`peeks.take_over` + `set_cloaked`, `preposition_cloaked`).
  - `ovl_place`: `OverlayWindow::place` (immediate `SetWindowPos`).
  - `ovl_blend`: `Overlay::apply_blend` of both kinds (the backdrop's
    includes `sync_backdrop` and `set_bake_blend`).
  - `sync_tail`: `platform_sync` from after `redraw_containers` to its
    end (focus, overlay sync, tab bars, overview, guards, particles).
  - `command`: the WM-side handling of one IPC message
    (`handle_ipc_message`), so the answer time can be read from
    `perf.log` and compared with `perf_bench`'s client-side figure.
  Keep `Stage::ALL`, `COUNT`, `label`, `depth`, `is_cross_cutting` and
  the tests in step.
- **`perf_bench`**: parse `interval` p99 and max (the report prints
  them), add columns `interval p99 ms`, `interval max ms`; extend the
  parse test.
- **`isolate_modes.ps1`**: `-TileOrder`, ported from
  `results/2026-11-12/bench-run.ps1` (`Get-TileOrder`, `Set-TileOrder`,
  applied after each restart's `restore-layout`, order logged). No new
  script, so CLAUDE.md's strip list is unchanged.
- Risk: a scope in a hot loop. All are per pass or per window, none per
  pixel; `perf::scope` is inert when disabled.
- Check: `cargo check`/`clippy` for the Windows target; `cargo test -p
  wm-platform` on Linux if it builds there (perf tests are portable).
- Measure: nothing to A/B. A `resize` burst's report should now show
  `to first frame` in `default`, and the `~56 ms` split over the new
  stages.

## B. Why `nopeeks` answers slower than `main`

Ranked by the one burst per build there is; A's stages confirm or kill
each.

1. **First-pass overlay tracking** (`session_overlays` +12.5 ms,
   take-over `rd_frozen` +7.4 ms). Both call `track_overlays` →
   `Overlay::apply_blend` + `defer_rect`. Suspects, in
   `native_border_overlay.rs` (859 lines changed) and
   `native_backdrop_overlay.rs`: more knobs set per blend (`set_glow`
   bands, highlight, rim), `sync_backdrop`/`set_bake_blend` per call,
   `set_bounds`/parking state making `defer_track` fall back to the
   immediate `track`, `hide_untracked`.
2. **Cloaking** (`cloak` +5 ms; per window 1.0–2.3 ms vs 0.5–0.7).
   `set_view_cloaked`, `preposition_cloaked`'s `frame_with_shadows` +
   `discard_bits_if_resized`, `peeks.take_over`/`is_composited` lookups.
3. **More passes** between command and first tick (10 vs 8
   `platform_sync`, 8 vs 7 `rd_apply`): something queues an extra
   redraw, e.g. focus fades, covered tiles, tab bars.
4. **`switch`** (+17–26 ms, no excerpt): `WorkspaceSurrogate` set-up and
   its hand-over (`take_over_outgoing`, flushes) in `manager.rs` /
   `workspace_surrogate.rs`.
5. **Pass tail** (`platform_sync` − `redraw`: 3.2 vs 1.0 ms):
   `sync_covered_tiles`, `sync_scroll_view`, `sync_tab_bars`,
   `sync_overview`, guards, particles.

Approach: diff each named function against `origin/main` (`git diff
origin/main..HEAD -- <file>`, or `git show origin/main:<file>`), find
work that is new, repeated or no longer skipped on an unchanged
overlay/window, and fix it. A fix that only skips redundant work with
identical results needs no word; one that changes what reaches the
screen or its order goes behind a new word. No fix without code to
point at: report the rest as hypotheses.

- Risk: overlays are tuned against recordings (rule 5). No reordering.
- Measure: `nopeeks` vs `main`, all scenarios, with and without B's
  words. Expect command p50 down by up to the deltas above (`resize`
  ≤ ~25 ms), smoothness columns unchanged.

## C. Peek-path work before the answer

What can be removed rather than moved: moving work from the pass into
the first tick delays the first moved frame by the same amount, since
the tick draws only after it. The flush, cloak and overlay hand-over
around it are ordered against recordings and stay.

- **`keeppeeks`.** `PeekView::tick` drops a released entry's
  `WindowPeek`; the next motion of the same window builds a new one
  (window, thumbnail, copies: 3.6 ms each, 25 ms for 7). With the word,
  a released tile's peek is hidden and kept in a spare map (per window
  id; a few seconds' lifetime, a cap on their number; dropped when the
  window is unmanaged, its monitor or its liveness (`is_live`) differs,
  or the config reloads). `move_tile` takes the spare instead of
  `WindowPeek::new`, reset to the new target, clip, corner style and
  styles. Needs a reset on `WindowPeek` that leaves it exactly as `new`
  would; if any state can't be reset, that peek is not kept.
- Risks: a stale thumbnail or copy shown for a frame; a hidden peek
  holding a DWM thumbnail of a closed window (must be dropped on
  unmanage).
- Measure: `default` vs `default+keeppeeks`, `resize`, `move`, `float`
  (`morepeeks`). Expect `peek_new`/`anim_step` near 0 from the second
  burst on, command p50 −15–25 ms on `resize`, others unchanged.

## D. The hand-over stall

Three approaches, one word each. All leave pure moves and grown windows
as they are: those are already at their rect when the peek lands.

| word | what | cost |
|---|---|---|
| `asynchandback` | `Apply` arm: `has_surrogate` is false for a window its peek hands back, so its `SetWindowPos` is posted. The peek still covers it for the one or two ticks of `Role::Releasing`. | A slow app is shown at its old size, cut or short of its tile, once the peek goes. Cheapest; no wait. |
| `asynclanding` | When a deferred entry's motion is done, clear `Stretch::deferred` and queue its window for redraw; the `Frozen` arm's `follow_retargeted_column` then posts the size. `may_land` already holds such an entry (`Hold::Released`, `FILL_LANDING_WAIT`) until `Stretch::reached`. At hand-back it is `already_at_rect`: no `SetWindowPos`. | The peek stays up to 500 ms longer on a slow app (drawn at its tile, filled where the window is short). After the wait: today's synchronous resize. |
| `earlylanding` | The same, sent when the drawn rect is within `EARLY_LANDING_GAP_PX` of the target's size on both axes, so most apps have resized by the time the motion ends. | Until the motion ends the window may be smaller than its tile by up to that gap: a strip of its edge color. |

- `asynclanding`/`earlylanding` send once per motion; a retarget
  re-arms (`Stretch::retarget`, `restretched`). A held entry (`Hold::Held`)
  is left alone.
- Risks: `landing_frame` reads `deferred` to place ring and backdrop;
  with it cleared they follow the real frame, which is right once
  `reached`, and one wait late otherwise. `asynchandback` with a
  demoted app (`allows_sync_reposition`) is today's behaviour already.
- Measure: `resize` (and `float` under `morepeeks`), `default` vs each
  word, and `asynclanding+asynchandback`. Expect the longest gap from
  74–95 ms to ≤ ~20 ms, `rd_apply by process` without `[sync]` rows of
  tens of ms, frame p90 and interval p90 unchanged or better, command
  p50 unchanged (the stall is after the answer). Watch a `resize` of
  Notepad and Explorer for a window shown short of its tile.

## Checks before the push

`cargo check --workspace --target x86_64-pc-windows-msvc` (passes on
`cd04e3c` here, 48 s), `cargo clippy` for that target, `cargo test` for
what builds on Linux.

## B: findings

Read off the code and the `nopeeks`/`main` `resize` excerpts; not seen
running. On the command's own pass `platform_sync` is +18 ms (86.8 vs
68.8): `redraw_loop` +8.8, `session_overlays` +12.5, `cloak` −2.2.

1. **First-pass overlay tracking: confirmed, fixed behind
   `keepoverlays`.** `platform_sync.rs: redraw_containers` (`Frozen` arm,
   `FrozenStep::TakeOver`) calls `track_overlays` with
   `session_overlay_placement`, which is `None` for a new session:
   `ResizeSession::current_rect` is only set by `queue_surrogate_updates`
   later in the pass. `track_overlay`'s `None` arm then hides both
   overlays (`ShowWindow` ×2 per window), and the session loop finds them
   hidden, so `NativeBorderOverlay::place_ring`/`NativeBackdropOverlay::place`
   take the reveal path (`SWP_SHOWWINDOW`, ring rebuilt first). `main`
   skipped the upsert without a rect (lost in `3448d80`, a refactor), so
   its loop only moved shown overlays behind the surrogate. With the word
   the take-over skips tracking without a placement (`tracks_at_take_over`).
   Changes the flushed frame (overlays kept behind the real window, as on
   `main`, instead of hidden), hence the word. Expected: `rd_frozen`
   back to ~0.6 ms/window (−7 ms on 7 windows) and `session_overlays`
   toward `main`'s 38.8 (−12 ms); also `move`/`float` (same take-over).
   Measure: `nopeeks` vs `nopeeks,keepoverlays` vs `main`, `resize` and
   `move`; command p50, `rd_frozen`, `session_overlays`, A's `ovl_place`;
   watch the ring/backdrop at a resize's first frame for a blink (should
   be gone, not added).
2. **Cloaking: refuted.** The per-window `cloak` rows in the timeline
   are the take-over scope (`Stage::Cloak` inside `rd_frozen`), i.e.
   finding 1. `commit_pending_cloaks` itself is 16.6 vs 18.8 ms;
   `set_view_cloaked` matches `main`. `preposition_cloaked` adds
   `frame_with_shadows` per growing window: small, and feeds
   `discard_bits_if_resized` (on screen), left alone.
3. **More passes: refuted for the answer.** The slowest-input timeline
   has one `platform_sync` between command and first tick in both
   builds (`WindowManager::process_commands` runs one). The extra two
   passes and the extra `rd_apply`/`uncloak` (3.4 ms) fall elsewhere in
   the burst. Open what queues them; measure with A's `command` stage and
   per-pass logs.
4. **`switch`: open.** The switch set-up in `redraw_containers`
   (`WorkspaceSurrogate::new` + `carry_backdrop`, `park_switch_borders`,
   `cover_ring`, `conceal` after the flush) is new per window and all of
   it reaches the screen in an order rule 5 protects. Nothing redundant
   found by reading. Measure: a `switch` excerpt with scopes around
   `park_switch_borders` and the surrogate loop.
5. **Pass tail: refuted.** On the command's pass `platform_sync` −
   `redraw` is 0.11 ms (`main` 0.06); the 3.2 ms total is the other
   passes (3).

Minor, open: `session_begin` 0.51 vs 0.37 ms per call
(`ResizeSession::begin_impl`; unread, possibly `NativeSurrogate`'s companion lookup).

## Result (2026-11-12, not seen running)

Merged into `tryout/start-latency`: A, B, C, D. `cargo check --workspace
--tests` and `cargo clippy --workspace --all-targets -- -D warnings`
for `x86_64-pc-windows-msvc` are clean. No test ran: `wm-platform`
doesn't build on Linux. No new Windows API calls.

| word | where | notes |
|---|---|---|
| `keepoverlays` | `redraw_containers`, `Frozen` take-over | Old path (and `morepeeks` leftovers). See "B: findings". |
| `keeppeeks` | `PeekView::keep_spare`/`take_spare`, `WindowPeek::set_aside`/`reset` | Copies are rebuilt, so the saving is the window and first thumbnail only. |
| `asynchandback` | `hands_back_in_step` | Posts the hand-back's move. |
| `asynclanding` | `Entry::sends_size_early`, `post_deferred_size` | Posted directly, not through a redraw pass: that would settle overlays mid-motion. |
| `earlylanding` | same, `EARLY_LANDING_GAP_PX` = 12 | Takes precedence over `asynclanding`. |

Differences from the plan above:

- `hold_on_peek` and scrolling columns don't note an animation start: no
  input starts them.
- `asynclanding`/`earlylanding` leave out entries that land hidden: they
  are posted already.
- `perf_bench` also prints the WM-side `command` time and the
  `peek_*`/`session_overlays` time outside frames.

To measure (desktop, `-TileOrder` as on 2026-11-12):

```
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Rounds 4 -Scenarios move,resize -Modes nopeeks,nopeeks+keepoverlays,default,keeppeeks,asynchandback,asynclanding,earlylanding,earlylanding+asynchandback+keeppeeks -TileOrder <names>
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Rounds 4 -Scenarios float,resize -Modes morepeeks,morepeeks+keeppeeks,morepeeks+earlylanding+asynchandback+keeppeeks -TileOrder <names>
```

`-Launch` (default `notepad,explorer,wt,chrome`) starts those once per run
and closes them after; `minimize` is no longer in the default scenarios.

Read first: `input->first frame ms` (now recorded on peeks), `command ms
(WM side)`, the `peek_*` columns (where the unscoped ~56 ms went), then
`interval p99`/`max` for the stall.
