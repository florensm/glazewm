# Plan: columns drawn by a resting peek slide with the workspace switch

Written 2026-11-13 from a reading of `1d65077` (`tryout/start-latency`).
Nothing here has been seen running: the container is Linux. Read
`CLAUDE.md` and `STAND_INS_PLAN.md` ("Words used here", "Rules", rule 5
especially) first. Behind `GLAZEWM_ISOLATE=slidepeeks`, off by default.

## The bug

Scrolling layout, `animations.workspace_switch` on. Switching to a
workspace with a column partly past the monitor edge, the fully visible
windows slide in; the partly visible column is at rest, solid, for the
whole slide. Only on an edge with another monitor beyond it (owner,
2026-11-13): left monitor, the column cut at its right edge; right
monitor, the one cut at its left edge. On an outer edge it slides.

## What happens today

- **Which columns are hidden.** `layout/scrolling.rs`:
  `shows_real_window` is `is_in_view || !overflows_onto_other_monitor`.
  A column past an outer edge keeps its real window shown (it runs off
  screen) and so takes part in the switch like any window. One whose
  off-screen part would cover another monitor is cloaked
  (`real_window_rect` keeps the cloaked window on its own monitor, same
  size) and its on-screen part is drawn by a resting peek. Confirmed:
  this is exactly the owner's "only next to another monitor".
- **Who slides.** `commands/workspace/focus_workspace.rs`:
  `is_switch_animated` = not minimized and not `layout::is_hidden_in_layout`
  (inactive stack tab or `!shows_real_window`). Only those get
  `pending_sync.setup_workspace_switch_incoming/_outgoing`, so only
  those get a `WorkspaceSurrogate` in the pre-pass.
- **Resting peek.** `animation/scroll_view.rs: sync_scroll_view` runs at
  the start of `platform_sync`, before `redraw_containers`, with the new
  workspace already displayed. For the hidden column `from == target`
  (its `last_rects` were dropped by `take_departed` when its workspace
  was hidden), so it takes the `else if is_on_screen(&target) { Role::Peek }`
  branch: `PeekView::upsert` → `new_entry` (`WindowPeek::new`, pinned to
  the working area, copies of the backdrop styles) → `Entry::draw` →
  `WindowPeek::place` → `show_pinned`: shown at once, at rest, at full.
  `peek_view.rs: track_overlays` then carries its ring and backdrop
  (`carry_ring`, `carry_backdrop`, `Handover`). That is the bug, incoming.
- **Outgoing, same cause.** In that same `sync_scroll_view` the old
  workspace is no longer displayed, so `take_departed` takes the
  column's `Role::Peek` entry and it is dropped at the end of the call,
  before the pre-pass shows any outgoing surrogate: the column vanishes
  at the start instead of sliding out. Not reported, confirmed by
  reading.
- **Inactive stack tabs** show nothing (`scroll_view.rs: tiling_windows`
  leaves them out; no peek). **Columns entirely off screen** have no peek
  (`is_on_screen` false → `entries.remove`). Neither must slide.
- **Pre-pass** (`platform_sync.rs: redraw_containers`, under
  `Stage::RedrawPrep`): per switch window a `WorkspaceSurrogate::new`
  (incoming at `to_rect + total_border_delta`, the layout rect;
  outgoing at `window_target_positions`, also the layout rect),
  `carry_backdrop` (copy) or `Live` mode (uncopyable backdrop);
  `park_switch_borders` (ring parked at `rest_frame`, concealed for
  incoming, shown for outgoing, copy carried); outgoing `show_initial`
  (warm-up opacity), `cover_ring` for a shown parked ring; incoming
  `show_incoming`/`show_initial` for non-slide styles; one `dwm_flush`;
  parked rings `conceal`ed; `hand_over_cut_short_slide`;
  `AnimationManager::start_workspace_switch`.
- **Thumbnail source.** `WorkspaceSurrogate` registers a DWM thumbnail of
  the real window, cloaked on its hidden workspace for incoming windows.
  The hidden column's real window is cloaked too, at the same size, so it
  is sourced the same way; where the cloaked window sits doesn't matter
  to a thumbnail. The resting peek already does exactly this.
- **Clipping.** `WorkspaceSurrogate::show_frame` cuts the frame at
  `panel ∩ monitor`, `panel` being the working area moved and scaled with
  the workspace (`slide_axis`, `update_zoom`; `update_fade` stays at
  rest). Only what the window shows at rest slides: the part past the
  edge stays outside the panel at every frame, for both edges and both
  directions. Pinned, the surrogate window spans the working area and
  can't draw outside it. Ring copy: `OverlayCopies::place(frame, clip)`,
  same cut. `Live`: the surrogate is the visible rect, the backdrop
  overlay `defer_cut_rect` to it, the ring `upsert_pinned_border_overlay`
  pinned to `border_viewport`, cut at the panel. So a column reaching
  past the edge is cut there for free, ring and backdrop included, as
  long as nothing shows the real ring at the layout rect.
- **Per frame** (`manager.rs: finish_frame`, `workspace_switch` block):
  incoming first; outgoing `restore_alpha`, real backdrop hidden under
  its copy; style blend and effect opacity lerped; `slide_axis` /
  `update_fade` / `update_zoom`; `Live` backdrop and pinned ring
  overlays. At `ws_done`: outgoing hidden (thumbnail, ring, backdrop, real
  ring), `stash_for_ws_cleanup`, every id queued for redraw, focus
  re-queued, focus effects applied.
- **End.** The next `platform_sync` redraws them: an incoming window is
  `Apply`'d only `if is_visible` → `prepare_ws_hand_back`, sync
  reposition, uncloak, `hide_pending_ws_cleanup_surrogate` (`HandBack`).
  Then `advance_ws_cleanup` over ticks (`WsCleanupStep`): `AwaitUncloak`,
  `ShowOverlays` (`final_overlay_rects`: the real window's `frame()`),
  `Reanchor`, `Drop`.
- **Interruption.** `start_workspace_switch`: a playing switch's windows
  not in the new one go to `interrupted_ws_windows` and
  `stash_for_ws_cleanup`; redrawn next tick and handed back as at the
  end. `hand_over_cut_short_slide` hides the old surrogate of a window in
  both (outgoing: new one brought up first).
- **Peek-aware places during a switch**: `has_active_surrogate`,
  `has_live_ws_surrogate` / `overlays_tracked_elsewhere` (overlays left to
  the switch), `stands_in_for`, `is_frozen_by_ws_animation` (incoming
  `Frozen`, `FrozenStep::Keep` when cloaked), `has_other_stand_in`,
  `tab_bars` (`is_in_workspace_switch` hides the tab bar),
  `motion_clicks` (`switching_rects`), `PeekView::window_at`.

## Design: the column joins the switch (shape A)

The column gets a `WorkspaceSurrogate` like its neighbours; its resting
peek stands by, too faint to see, under it during the slide and takes
over at the end in the hand-back step. Outgoing, its peek is kept until
its surrogate has taken over.

Why A over B (the peek slides itself): the switch's motion is tuned code
that A reuses whole: every style (slide, drift, soft zoom, fade, zoom,
cut), the panel cut, `opacity_*`, focus style blend, `Live` backdrops,
the ring copy, interruption and the cleanup steps. B would rebuild all of
it for peeks: a second slide engine, a cut separate from the pinned
viewport (`WindowPeek::place` repins, i.e. moves the peek window, when
the clip changes), and no answer for `Live` peeks, which can't draw
scaled. B saves the two swaps below; each is the codebase's existing
thumbnail-to-thumbnail idiom (`hand_over_cut_short_slide`: new one up,
old one hidden, back to back).

### Steps (all gated on `slide_peeks`)

1. `focus_workspace: is_switch_animated` → `slides_with_switch`: also a
   column hidden only by `shows_real_window`, with part of it on its
   monitor's working area. Inactive tabs and off-screen columns stay out.
2. `sync_scroll_view`: a `Role::Peek` column that slides in
   (`pending_sync` incoming and the switch animates, or the manager still
   owes it a hand-over, `awaits_peek_hand_back`) is put on stand-by
   before `upsert` draws it (`PeekView::stand_by`, `WindowPeek::stand_by`:
   window and copies at warm-up opacity, kept there by `show_pinned`).
   `track_overlays` and `window_at` skip it: its ring and backdrop belong
   to the switch meanwhile. Safe: the surrogate draws it; the peek is only
   being warmed.
3. Same call, outgoing: a departed `Role::Peek` column flagged outgoing
   (switch animates) is kept, at rest, in `PeekView::switching_out`
   instead of dropped. Nothing ticks or tracks it.
4. `park_switch_borders`: the ring of such an outgoing column is parked
   concealed, as an incoming one's: shown at the layout rect it would
   reach onto the neighbouring monitor. Its copy goes full with the
   take-over.
5. `Apply` arm, outgoing: for a window with a kept peek,
   `take_over_from_peek` (surrogate to full: window, ring, backdrop
   copies) then the peek is dropped, back to back. No `HiddenAlpha`: the
   real window is cloaked already.
6. `Apply` arm, end of slide: a window not shown (`!is_visible`) that is
   an incoming `Pending` entry of `pending_ws_cleanup` is collected.
   After the loop: `stand_in` its peek (window, then ring and backdrop to
   full), `track_overlays` once (ring parked concealed at the real frame
   and carried into the peek, backdrop shown in its style), then
   `hand_back_to_peek`: surrogate thumbnail and backdrop copy hidden,
   `HandBack::ToPeek`. Its ring copy stays until `WsCleanupStep::Drop`,
   over the peek's identical new one (two rings in one place read as
   one). Never uncloaked: it isn't visible, so the uncloak branch is
   never taken.
7. `final_overlay_rects` leaves `ToPeek` entries out: the real ring and
   backdrop at the cloaked window's `frame()` would be in the wrong
   place. The peek's `track_overlays` owns them from step 6 on.
8. After `redraw_containers` (`peek_view::settle_switch_stand_ins`): a
   peek on stand-by whose window the manager no longer owes a hand-over
   (no surrogate, switch not played, suppressed, cut, cleared) stands
   in; kept outgoing peeks left are dropped. Self-healing: no peek stays
   faint past the sync that decides it.

### Edge cases

- **Second switch mid-slide** (to a third workspace): the column is
  outgoing now; its stand-by peek departs and is kept (step 3), faint;
  `hand_over_cut_short_slide` brings the new outgoing surrogate up at
  rest as for its neighbours; step 5 drops the faint peek.
- **Switch back mid-slide**: an outgoing column's peek was dropped at the
  first switch's start (step 5); its new incoming peek stands by (step
  2); the old surrogate is hidden by `hand_over_cut_short_slide`; the end
  is step 6.
- **Switch on the other monitor cutting this one short**:
  `interrupted_ws_windows` redraws it next tick; step 6 hands it to its
  peek at rest, as neighbours jump to rest.
- **Focused column**: a hidden column is rarely focused (focus scrolls it
  into view). If it is (wider than the monitor), it is cloaked at rest
  today too; `sync_focus` defers during the slide as for any incoming
  window. Nothing new.
- **Scroll during the slide** (`sync_scroll_offsets`): becomes fully
  visible → it shows its real window: not standing by any more, the
  normal hand-back uncloaks it at the end (it is visible then). Fully
  off screen → `entries.remove`; step 6 just hides the surrogate. A
  column that moves on a peek mid-slide stands in first (step 2) and
  then moves as its neighbours' moving peeks do today (both their
  surrogates and peeks show: pre-existing, not changed here).
- **Two monitors**: the surrogate is pinned to its working area and cut
  at the panel; the outgoing real ring is never shown (step 4), the
  incoming one is parked concealed; the peek draws only inside its clip.
- **Other styles**: fade, zoom and cut show incoming surrogates at rest
  from the start; the stand-by peek is under them, faint. `opacity_*` is
  the surrogate's. `starts_on_screen` styles are unchanged.
- **Suppressed or disabled switch**: step 2 doesn't stand by (it checks
  `enabled` and `animates`), step 3 doesn't keep; step 8 catches the
  rest (e.g. `direction == 0`, a failed `WorkspaceSurrogate::new`).

### Risks

- Steps 5 and 6 swap two thumbnails back to back in one tick. A
  composition landing between them shows both for a frame (never
  neither): identical pixels for an opaque window, a frame darker for a
  semi-transparent one. Same as `hand_over_cut_short_slide`.
- The stand-by peek's ring copy is registered only at step 6; the
  surrogate's ring copy covers it until `Drop`, three ticks later.
- Stand-by backdrop copy warmed at opacity 1, as `warm_up` warms windows.
- New Windows API calls: none.

## Owner checks (two monitors side by side, scrolling layout, slide)

Without the word: unchanged, the bug as reported. With
`GLAZEWM_ISOLATE=slidepeeks`:

1. Left monitor, a column cut at its right edge on workspace 2. 1→2 and
   3→2: the column slides in with its neighbours, cut at the edge; its
   ring and backdrop with it; nothing of it on the right monitor at any
   frame; at the end it sits as before, no flash, no double ring.
2. Same on the right monitor with a column cut at its left edge.
3. Outgoing: from 2 to 1 and to 3: the column slides out with the rest,
   instead of vanishing first.
4. Interrupted: 1→2 then 2→3 mid-slide; 1→2 then back to 1 mid-slide;
   fast repeated presses. No column left faint, blank, or doubled.
5. A window with transparency and one with a backdrop (copyable and
   rounded, i.e. `Live`) as the cut column.
6. `fade` and `zoom` styles; `workspace_switch.enabled: false`; overview
   pick (suppressed): the column is shown at rest at once, as today.
7. A stack as the cut column: its tab bar as its neighbours'. Inactive
   tabs never appear.
8. Focus a neighbour mid-slide so the strip scrolls: the column ends
   where the layout puts it, real window shown if fully on screen.
