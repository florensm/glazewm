# Open, close, minimize and restore: animation review

2026-11-10, on `claude/glazewm-window-animations-zbw9df` (from
`tryout/smooth-shades` at `ec8e0fe`). Done in a cloud session: nothing
here was seen running. Every fix was built, linted and unit-tested for the
Windows target (see "Checks run"). `TODO.md` "Not checked live yet" 29-36
has the live checks.

Ranked by how much each changes what you see. "Stand-in" as in
`STAND_INS_PLAN.md`; open, close, minimize and restore all use
`ResizeSession`.

## Read this first: two paths open a window

- **Tiling layouts, stack tabs**: `window_open` through
  `start_open_animation` (`manager.rs:3331`). Findings 1-3 and 9 are about
  this path.
- **Scrolling layout** (your config): a new column is taken by the scroll
  view (`scroll_view.rs:596`, `opening_rect` `:1151`). It widens in place
  on `animations.scroll`'s duration and curve; `window_open`'s style,
  duration and easing are not used. That path already moves the new window
  to its column at once (`follow_retargeted_column`), so findings 1-2 do
  not apply to it. See 17.

A close always uses `window_close` (off by default); the neighbours close
the gap by `window_resize`, or by the scroll.

## Summary

| # | Finding | Status |
|---|---|---|
| 1 | A new window stayed at its app's size until after its open animation | fixed `4f37740`, `b5f3592` |
| 2 | A `fade` open never showed its stand-in | fixed `0ad7cf1` |
| 3 | A zoom ended a few percent short of full size | fixed `983ef0d` |
| 4 | Zoom, close and minimize frames were timed by the wall clock | fixed `7074e09` |
| 5 | A restore during a minimize left the window in the wrong state | fixed `9a23bf4` |
| 6 | A close animation played on around a window already gone | fixed `9dfab1f` |
| 7 | A close with no stand-in hid the window for the whole duration | fixed `5f39127` |
| 8 | A slide restore showed the window twice for a frame | fixed `356f882` |
| 9 | An open retargeted mid-way loses its zoom or fade | proposed (restructure) |
| 10 | A restore by command animates a window still minimized | on peeks: built `9e19c06`, not checked live; old path open |
| 11 | A close cloaks before its stand-in is composited | proposed (live A/B) |
| 12 | Slide and zoom opens have no paint grace | proposed (live A/B) |
| 13 | See-through windows are handed back without an alpha swap | proposed (`STAND_INS_PLAN.md` step 3) |
| 14 | A close during the window's own animation jumps | proposed |
| 15 | Curves: restore plays minimize's ease-in forwards | fixed `9699b12` |
| 16 | `window_close` on by default | on, at the owner's word (`STAND_INS_PLAN.md` 7b) |
| 17-21 | Smaller items | proposed |
| 22 | A fading close kept its border and backdrop to the end | fixed `15deb4d` |
| 23 | A close beside neighbours is covered by them | by design; config |

## Fixed

### 1. A new window stayed at its app's size until after its open animation

- **Where**: `platform_sync.rs:1596` (`FrozenStep`, `:555`),
  `preposition_cloaked` `:665`; cause at `manage_window.rs:102`.
- **Sequence**: `manage_window` cloaks every new window (`CloakGuard`)
  before its first sync. The first `Frozen` redraw set a window up (show
  the stand-in, flush, cloak, move to target) only `if !is_cloaked()`, so
  for an open the whole block was skipped, the move with it. The stand-in's
  thumbnail is registered at the tile's size (`ResizeSession::begin` gets
  `start_rect`, which has the target's size) while the window keeps the
  size its app chose. So the animation drew that size's content cut to the
  tile, or past its edge where the tile is larger. At the end `pre_commit`
  (`resize_session.rs:1113`) found the window off target, moved it, and
  waited for it to land (polled every other tick, up to 500 ms): the content
  reflowed in one step at rest, and the uncloak, border settle and focus
  came that much later.
- **Evidence**: `FrozenStep::of(Some(false), || true)` was the skipped
  case; `handoff_done` starts `true` for a session whose source and target
  sizes match (`resize_session.rs:387`), so `maybe_handoff` never moved the
  window either. Nothing else positions a new window before `pre_commit`.
- **Fix**: the first frame is decided by `FrozenStep`. A session's window
  found cloaked is moved to its target at once (asynchronously, no flush:
  it is not on screen), so its app lays out at the tile's size while the
  stand-in plays, and `pre_commit` normally finds it there.
  `b5f3592`: the move is made once (`ResizeSession::adopt`) and the session
  stays unmarked, so an opening window is still checked every pass and
  taken over again if the shell uncloaks it (it does when a window is
  activated), as before this pass.
- **Tests**: `platform_sync::tests::a_window_cloaked_before_its_session_*`,
  `a_window_still_shown_is_taken_over`, `a_window_set_up_already_*`.
- **Risk**: medium. The app now resizes during the animation, so a slow one
  shows its old layout for the first frames instead of all of them (see
  12). Contradicts `STAND_INS_PLAN.md`'s table, which lists open's
  take-over as "stand-in shown, flush, cloak": that never ran for an open.

### 2. A `fade` open never showed its stand-in

- **Where**: `platform_sync.rs:1651`, `manager.rs:3020` (`reveal_session`).
- **Sequence**: a stand-in is created hidden. Slides show theirs from
  `show_clipped`, zooms from `update_zoom_fade`; `none`/`fade` relied on
  `session.show()` in the block finding 1 describes, which an open never
  reached. So with `style: "fade"` (or `"none"`) and `opacity_from < 1`
  nothing was drawn for the duration plus `OPEN_PAINT_GRACE`, then the
  window popped in at full opacity.
- **Evidence**: the only `set_visible(true)` callers are those three.
- **Fix**: the stand-in of a window found cloaked is shown on that first
  redraw, at the opacity the animation starts from.
- **Risk**: low. If you use `fade`, this is the largest visible change here.

### 3. A zoom ended a few percent short of full size

- **Where**: `resize_session.rs:1182` (`finish_commit`),
  `zoom_destination` `:1272`.
- **Sequence**: a zoom moves only its thumbnail's destination rect. The
  last frame is drawn a tick before the animation completes (completion is
  judged at the wall clock, `state.rs:324`). `finish_commit` squares the
  stand-in up with the window for the other styles, but left a zoom's
  thumbnail where that frame put it, and its overlays scaled by the same
  progress. The window was then uncloaked under a smaller copy of itself,
  which faded out over 100 ms (opaque windows), or went at once (see-through
  windows): a jump to full size.
- **Evidence**: default curve `cubic-bezier(0.16, 1, 0.3, 1)`, 150 ms,
  60 Hz: the frame before completion is about 3% short, 38 px across a
  1280 px window. Also `cx ± round(w/2)` ended 1 px too wide for an odd
  width.
- **Fix**: `finish_commit` draws a zoom at progress 1; the rect is inset
  from the edges, exact at 1.
- **Tests**: `resize_session::tests::a_full_zoom_covers_exactly_the_stand_in`
  and two more.
- **Risk**: low. Applies to a `zoom` open and to the default restore.

### 4. Zoom, close and minimize frames were timed by the wall clock

- **Where**: `manager.rs:766` (`zoom_progress`), `:1503`.
- **Sequence**: `SMOOTHNESS_REVIEW.md` finding 1 put frames on the vblank
  estimate. Two readers were missed: `zoom_progress` (`eased_progress()`,
  i.e. `Instant::now()`), used by every zoom, and the loop driving close
  and minimize stand-ins (`current_state()`), for all styles. Those frames
  stepped by when the WM thread reached each tick, and ran half a frame
  behind neighbours resizing beside them on the frame clock.
- **Fix**: both evaluate at the pass's `frame_now`.
- **Test**: `manager::tests::zooms_are_evaluated_at_the_frame_time`.
- **Risk**: low to medium, as for the original change: judged by eye.

### 5. A restore during a minimize left the window in the wrong state

- **Where**: `platform_sync.rs:1313`.
- **Sequence**: `platform_sync` skips a window whose minimize animation
  plays. A restore inside that time (150 ms by default) was skipped too,
  and its restore flag is cleared with the sync; when the animation ended
  nothing redrew the window. After `toggle-minimized` twice it stayed
  natively minimized while holding a slot in the layout (an empty tile).
  After a taskbar click it was shown where it used to be, over neighbours
  that had filled its place, with no border or backdrop (the minimize's
  teardown removes them). Either lasted until something else redrew it.
- **Fix**: a window flagged as restoring ends its minimize animation and is
  handled as any other restore.
- **Risk**: low to medium. The minimize stops where it was and the restore
  starts from nothing: right end state, not a smooth reversal (see 9).
  No unit test: the decision is two flags inside the redraw loop.

### 6. A close animation played on around a window already gone

- **Where**: `wm.rs:472`.
- **Sequence**: the window is destroyed while its close animation plays (its
  app closes it, or crashes). The thumbnail has no source and draws
  nothing; the border and backdrop tracking the stand-in play on to the end
  around an empty rect, then `WM_CLOSE` goes to a dead handle (a warning in
  the log). `ClosingWindows` already reports it closed within 50 ms.
- **Fix**: the animation and its overlays are dropped on that report.
- **Test**: `closing_windows::tests::window_destroyed_mid_animation_is_closed`.
- **Risk**: low.

### 7. A close with no stand-in hid the window for the whole duration

- **Where**: `manager.rs:3543`.
- **Sequence**: `ResizeSession::begin` returns `Ok` without a thumbnail
  when DWM refuses one (an elevated window). The close started anyway: the
  window was cloaked and detached, nothing was drawn, and `WM_CLOSE` was
  sent a full duration later.
- **Fix**: no stand-in, no animation: closed or minimized at once.
- **Risk**: low.

### 8. A slide restore showed the window twice for a frame

- **Where**: `platform_sync.rs:1636`, `manager.rs:3020`.
- **Sequence**: restoring a window minimized with a `slide_*`
  `window_minimize` style is the one slide starting on a window still on
  screen. Its first frame (`update_clipped`) clips or hides the stand-in;
  `session.show()` then showed it again at the unclipped start rect, a
  window's width beside its place, for the frame the flush before the
  cloak composites.
- **Fix**: stand-ins that slide or zoom in are left to their own frames.
- **Risk**: low. Only that config.

## Proposed

### 9. An open retargeted mid-way loses its zoom or fade

- **Where**: `manager.rs:2560` (`new_movement`), `:2577`
  (`update_target`), `resize_session.rs:700` (`update_zoom_fade`).
- **Sequence**: window A is opening; window B opens (or anything else
  relays A out) before A lands. `start_animation_if_needed` replaces A's
  animation with a plain movement from its current rect to the new target.
  That drops the opacity track, so a fade snaps to full opacity. For a
  zoom, `session.zoom` stays set and `zoom_progress` now reads the new
  movement's progress: the zoom **restarts from no size**, and since a zoom
  never moves its stand-in window, it plays at the old place until
  `finish_commit` snaps it over. A slide carries on correctly.
- **Evidence**: `new_movement` sets `start_opacity`/`target_opacity` to
  `None`; `update_zoom_fade` only sets thumbnail rects.
- **Why not done**: the right fix is one driver with two tracks: where the
  window is (retargetable) and how far it has entered or left (zoom scale,
  slide offset, fade; never retargeted, reversible). That is lead 9's
  answer too: the per-style branches (`zoom`, `slide_in_monitor_rects`,
  the fade's `session.update`) are where the paths drifted, and findings
  1-3 and 8 all sat in them. It rewrites how every style is drawn, so it
  needs your recordings first (`STAND_INS_PLAN.md` step 0). With it, 5 can
  reverse a minimize from where it is.
- **Live check**: `window_open` `style: "zoom"`, `duration_ms: 2000`; open
  two windows a second apart. The first should keep growing, not restart.

### 10. A restore by command animates a window still minimized

- **Where**: `platform_sync.rs:1463`; the native restore is only in
  `reposition_window` (`:2169`), on the final `Apply`.
- **Sequence**: `toggle-minimized` (or `set-tiling`) on a minimized window
  flags the restore and starts its animation, but nothing has restored the
  native window. The stand-in samples a minimized window, and `pre_commit`
  compares the target with a minimized window's rect, so I expect it to
  wait out its 500 ms cap before the window is restored and shown. A
  taskbar click is unaffected: the OS restores first.
- **Not verified**: what DWM draws for a minimized source, and whether the
  move lands on one. Hence proposed, not fixed.
- **Live check**: `toggle-minimized` twice, a second apart, `duration_ms:
  1000`. If the window comes back late, without a visible zoom, and
  `errors.log` has "did not land within 500ms", this is real.
- **Fix sketch**: cloak, restore natively at the target, then
  `FrozenStep::Adopt`, as for an open. No new API.

### 11. A close cloaks before its stand-in is composited

- **Where**: `close_window.rs:30`, against `commit_pending_cloaks`
  (`platform_sync.rs:630`).
- **Sequence**: a move or resize shows its stand-in, waits one composition
  (`DwmFlush`), then cloaks, "otherwise the cloak can land on a frame where
  the surrogate's thumbnail is not yet populated". A close shows the
  stand-in and cloaks in the same breath; a minimize likewise. If that
  comment holds here, the window blinks out for a frame as a close starts.
- **Options**: (a) hand the cloak to the following sync's shared flush
  (the neighbours' sessions pay for it anyway); (b) cloak on the second
  tick. (b) shows a see-through window twice for a frame.
- **Live check**: `window_close` on, `duration_ms: 3000`, record a close
  at 60 fps and step through the first frames.

### 12. Slide and zoom opens have no paint grace

- **Where**: `manager.rs:3383`, `OPEN_PAINT_GRACE` `:68`.
- **Evidence**: the 30 ms hold applies to `fade` only, although the
  constant's own comment argues it for all styles. A slide's or zoom's
  first frames can draw a window its app has not painted, and since 1, one
  not yet laid out at its tile's size. Electron and browsers take longer
  than 30 ms for either.
- **Options**: hold every style at progress 0 for a configurable grace
  (delays motion, against "starts the frame after"); or leave it. A fixed
  number is a guess per app; needs your A/B with Vesktop and a browser at
  `duration_ms: 1000`.

### 13. See-through windows are handed back without an alpha swap

- **Where**: `manager.rs:2044`.
- **Evidence**: for a window with transparency the stand-in is set to 0 in
  the tick that uncloaks the window, with nothing tying the two to one
  frame. The switch and the scroll swap by the window's alpha
  (`HiddenAlpha`). I agree with `STAND_INS_PLAN.md` step 3: give
  `ResizeSession` that swap. Your config (80%) is the affected one.

### 14. A close during the window's own animation jumps

- **Where**: `close_window.rs:74`.
- **Evidence**: the closing stand-in starts at the window's layout rect,
  not where its open, move or resize was drawing it, and its thumbnail is
  registered at that rect's size whatever size the window has reached.
  Closing two windows in quick succession hits it: the second is mid-resize
  from the first.
- **Fix sketch**: start from `animation_stand_in`'s rect with a stretched
  thumbnail. Small once 9 exists.

### 15. Curves: restore plays minimize's ease-in forwards

- **Where**: `parsed_config.rs:1784` (`from_minimize`).
- **Evidence**: defaults are open 150 ms `(0.16, 1, 0.3, 1)` (ease-out),
  close and minimize 150 ms `(0.32, 0, 0.67, 0)` (ease-in), resize 150 ms
  `(0.42, 0, 0.58, 1)`. Durations match. A restore reuses minimize's
  ease-in as it is, so the window accelerates into place and stops dead,
  while the config's doc says it is "played in reverse". Hyprland eases
  both directions out.
- **Proposal**: time-reverse the curve for a restore. A visible change, so
  your call. Neighbours also ease in-out while an open eases out; they
  share a clock (see "Ruled out") but not a curve unless you set
  `window_resize.easing` to match.

### 16. `window_close` on by default: not yet

A refused close is inherent to animating first: the window is gone for the
animation, then `WM_CLOSE` is refused and it is managed again
(`wm.rs:501`), re-opening with the open animation. A prompt in its own
dialog is caught within 50 ms; one drawn inside the window leaves it
invisible for 2 s (`CLOSE_CONFIRM_TIMEOUT`). `WM_CLOSE` can't go first: a
thumbnail dies with its window. Add 11 and 14. Turn it on after their live
checks, not before.

### Smaller items

17. **`window_open` has no effect in a scrolling layout** (see top). Either
    document it or let the column's stand-in carry the configured entry
    (with 9).
18. **Floating windows pop in** (`platform_sync.rs:1470`): no open
    animation for them. Design choice; Hyprland animates them.
19. **Hand-back starts a tick late** (`SMOOTHNESS_REVIEW.md` 14) and the
    **first frame is on the wall clock** (its 9, 10): unchanged, and I
    agree with both. They cost an open one frame at each end.
20. **`ResizeSession::begin` can't fail** but returns `Result`; three
    callers carry dead `Err` arms, one of which uncloaks.
21. **Shadows** arrive at the uncloak, not with the stand-in. Known; no
    copies without a live A/B (`TODO.md`).

### 22. A fading close kept its border and backdrop to the end

- **Where**: `resize_session.rs` `overlay_params`; read by the close
  loop in `manager.rs` (`update_internal`) and by `platform_sync`.
- **Sequence**: only a zoom scaled the overlays' opacity. A `style:
  "none"` close with `opacity_to: 0` (the sample config's) faded the
  thumbnail alone; ring and backdrop, separate windows, stayed at full
  strength and went in one frame at the end. With a backdrop that reads
  as a window that lingers, then pops. A fading open likewise.
- **Fix**: a session records its stand-in's fade and scales both
  overlays by it (by a zoom's progress when lower). `TODO.md` 38.
- **Risk**: low. Sessions without a fade pass their resting opacity or
  more, which scales nothing.

### 23. A close beside neighbours is covered by them

- **Where**: `manager.rs` `start_away_animation` (`place_at_top:
  false`), against `place_at_top: true` for every move, resize and open.
- **Sequence**: the neighbours' stand-ins are above the closing one and
  grow into its place from the first frame, on a curve that starts fast,
  while the default close eases in. So beside a neighbour little of a
  close is seen; a workspace's last window plays in full.
- **Status**: the owner keeps this order (2026-11-10). What shows is
  then a matter of `window_close`'s style and easing. `TODO.md` 39.

## Why the overview looks smoother than real windows

`STAND_INS_PLAN.md` now carries this as a table with the task that takes
each lever; its B and D below are the plan's D and B.

Added after a second look, 2026-11-10. Read from the code, not measured.
The overview differs from a move, resize, open or close in four ways,
largest first. Each is a lever for real windows.

- **A. What moves.** The overview, like the switch and the scroll, has one
  window per monitor that never moves and only moves thumbnails inside it
  (`native_overview.rs:1`). A move, resize, open, close or minimize moves a
  top-level stand-in window every frame with `SetWindowPos`, with its
  border and backdrop as two more windows in the same batch, while its
  thumbnail size and opacity go to DWM by another route
  (`resize_session.rs` `defer_update`: "applied immediately"). Three
  surfaces per window over two routes have to land in one composition.
  Lever: pinned stand-ins for these too, `STAND_INS_PLAN.md` step 3. The
  largest change, and the one I expect most from.
- **B. Nothing is handed over.** The overview never cloaks, uncloaks or
  waits for an app. A real window pays a `DwmFlush` before its first frame
  (0 to 1 frame), has its app reflow mid-animation, and ends with a swap
  (findings 3, 13, 19). Levers: `window_resize: { style: "stretch" }`
  scales the content instead of revealing it and resizes the app on the
  first frame, which is how Hyprland does it: a config change you can try
  today. Then finding 13 and `SMOOTHNESS_REVIEW.md` 9.
- **C. Curves.** The overview runs on springs that keep their speed when
  retargeted (`overview_layout.rs` `Spring`). Windows run a fixed-length
  bezier that restarts from rest at every retarget, except scrolling
  columns. Levers: `easing: "spring"` is accepted by `window_move`,
  `window_resize` and `window_open` already; and `retargeted` could cover
  every move, not only columns (`SMOOTHNESS_REVIEW.md` 12, a few lines,
  a feel A/B while holding a resize key).
- **D. Whose thread.** The overview has a thread of its own: a frame is
  spring steps and thumbnail updates. A real-window frame runs on the WM
  thread, after up to 4 ms of queued events (`main.rs:437`,
  `CATCH_UP_BUDGET`), and includes a whole `platform_sync`: overlays of
  every window, tab bars, focus. The 4 ms is fixed: a quarter of a 60 Hz
  frame, but 70% of a 175 Hz one, so on the fast monitor a busy queue can
  cost frames. Lever: a budget that is a fraction of the pacing monitor's
  period. Measure first: `tick ms/frame` in `perf.log` (`GLAZEWM_PERF=1`)
  during a resize on that monitor.

## Ruled out

- Neighbours on another clock: started in the same pass, clocks set on
  the same tick (`manager.rs:1397`). Only curve, and `fade`'s 30 ms, differ.
- Wallpaper between an opening window and the neighbour it splits, with
  default curves: the open leads the resize over the same travel.
- A window left cloaked by a failed manage: `CloakGuard`.
- WM exit mid-animation: sessions committed, closing and held windows
  uncloaked (`wm_state.rs` `Drop`).
- A workspace switch, the overview or a display change mid-open: the
  non-animated path drops the session and uncloaks in the same pass.
- A stalled close animation: `WM_CLOSE` after `ANIMATION_GRACE`.
- A slide not landing on its target: `finish_commit` moves it there.

## Visible changes

- Tiling layouts: an opening window shows its content at the tile's size
  during the animation and is handed back sooner (1).
- `fade`/`none` opens fade in instead of popping in after a pause (2).
- Zooms end at full size (3); zoom, close and minimize motion is timed
  half a frame differently (4).
- A restore during a minimize plays the restore (5).
- A window destroyed mid-close loses its border and backdrop at once (6).
- Elevated windows close and minimize without the animation's delay (7).
- A restore eases out: minimize's curve played time-reversed (15).

Config keys, defaults and durations are unchanged.

## New Windows APIs

None. No new dependency, crate feature or scheduling change. No benchmark
or script added, so `CLAUDE.md`'s strip list is unchanged.

## Checks run

Off Windows, for `x86_64-pc-windows-gnu`:

- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- Tests under Wine, one thread: `wm` 273 passed; `wm-common` 23 passed;
  `wm-platform` 311 passed, 8 failed. The same 8 fail on `ec8e0fe` (they
  need a real desktop, as `TODO.md` "Checks" says).

## Docs

- [`DwmRegisterThumbnail`](https://learn.microsoft.com/windows/win32/api/dwmapi/nf-dwmapi-dwmregisterthumbnail)
  and [`DWM_THUMBNAIL_PROPERTIES`](https://learn.microsoft.com/windows/win32/api/dwmapi/ns-dwmapi-dwm_thumbnail_properties)
- [`SetWindowPos`](https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setwindowpos)
- [`DWMWA_CLOAKED`](https://learn.microsoft.com/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)
- [Hyprland: animations](https://wiki.hypr.land/Configuring/Animations/)
