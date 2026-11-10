# Plan: one way to animate windows

Revised 2026-11-10 (third version; the second is `git show
ec8e0fe:STAND_INS_PLAN.md`). Written so a session that has not seen the
earlier ones can carry it out one task at a time. Read `CLAUDE.md`,
`TODO.md` and `ANIMATION_REVIEW.md` first.

Nothing in this plan has been seen running. It is a reading of the code
as of `88e5183`. Task 3 is where it proves itself on screen; if its live
check fails, stop there.

## What changed from the second version, and why

The second version's diagnosis holds: three stand-in systems, fixes landing
three times, move/resize on the oldest drawing. Its order does not serve
the goal (smoother open and close first):

- Its steps 1 and 2 merge what the switch and the scroll share. That is
  upkeep: nothing on screen changes.
- The change you would see, moving the `ResizeSession` animations onto
  pinned stand-ins, was its step 3, "only if local fixes won't do", and
  was described as porting everything `ResizeSession` does. Most of that
  is size handling (gap fill, grow-only registration, mid-motion resize),
  the hardest part.

What makes a shorter road possible:

1. Open, close, minimize and restore **never change the window's size**.
   They are a move, a scale or a fade of a window at its resting size:
   exactly what a pinned thumbnail does well. None of `ResizeSession`'s
   size handling is needed for them.
2. The scroll view already has a per-window stand-in with every hand-over
   solved and tuned live: warm-up, alpha take-over, ring and backdrop
   copies, companion wait, hand-back (`Entry`, `Role`, `Handover` in
   `animation/scroll_view.rs`). It already opens a new column this way.
   The switch's hand-over is per switch (`WsCleanupStep`), so the scroll's
   is the one to build on.
3. `platform_sync` already treats a window a peek draws as frozen, keeps
   its cloaked real window at its target (`follow_retargeted_column`), and
   hands it back (`prepare_hand_back`, `release`). A window that enters
   through a peek gets all of that with no new code there.

So: make the scroll's per-window stand-in usable for any tiled window,
give it an entry track (how far the window has zoomed, slid or faded in),
and move animations onto it one kind at a time, most seen first. Each
kind stays on the old path until you have seen the new one.

## Words used here

- **Real window**: the app's own window. Cloaked while a stand-in draws it.
- **Stand-in**: a click-through window of ours showing a DWM thumbnail of
  the real window (`NativeSurrogate` underneath).
- **Peek**: `WindowPeek`, the scroll's stand-in. **Pinned**: spans its
  monitor's working area and never moves; only its thumbnails do.
  **Live**: moved with its window every frame; the fallback when a
  backdrop can't be copied (rounded corners, or parallax != 1).
- **Overlays**: the WM's border ring and backdrop windows behind a real
  window. **Copies**: thumbnails of them drawn inside a pinned stand-in
  (`OverlayCopies`), so they move in the same DWM updates as the window.
- **Take-over / hand-back**: swapping real window and stand-in at the
  start / end without a frame showing both or neither.
- **Rect track**: where a window's tile is going (retargetable).
  **Entry track**: how far it has entered or left, 0 (away) to 1 (in).
- **Old path**: `ResizeSession` (`wm-platform/src/resize_session.rs`).

## Rules for whoever carries this out

1. One task per session unless one is clearly done early. Tasks are in
   order; do not skip ahead.
2. Before a task: `git log --oneline -15`, read every file the task names,
   and find each named function with `grep -n`. Line numbers here drift;
   names are what to trust. If a name is gone or does something else than
   described, stop and report instead of guessing.
3. A task marked **move only** changes no behaviour: no renamed logic, no
   "while I am here" fixes. Its diff must read as cut and paste plus
   `use` lines and visibility (`pub(super)`).
4. New behaviour goes behind the switch from task 2 until its live check
   has passed. Never flip a default in the task that adds the code.
5. Do not change, "simplify" or reorder any of these. Each was tuned
   against recordings (commit messages from `599f0aa` on explain them):
   - the order inside `WindowPeek::draw` (backdrop copy first out, last
     in), `take_over`, `prepare_hand_back`, `hand_back`;
   - `Role` transitions and `Handover` steps in `carry_ring` and
     `carry_backdrop`;
   - the z-order rule for a window being handed back
     (`platform_sync.rs`, "keeps its place");
   - `FrozenStep`, `commit_pending_cloaks`' single flush;
   - `HandoffWait` and the companion (shades) waits.
6. One commit per task step, prefixed like the history (`refactor(...)`,
   `feat(...)`, `fix(animations): ...`). Say in each message that it was
   not seen running.
7. Checks before every push (`TODO.md` "Checks" has the set-up; the test
   executables run with `wine`, one thread):
   - `cargo clippy --workspace --all-targets --target
     x86_64-pc-windows-gnu -- -D warnings` clean;
   - `wm` and `wm-common` tests all pass; `wm-platform` fails only the 8
     desktop tests `TODO.md` lists;
   - `cargo fmt --all`, then `git checkout` any file you did not edit
     (`workspace_surrogate.rs` has older formatting drift: leave it).
8. After each task: add its live check to `TODO.md` "Not checked live
   yet", written like the entries there (app, config, what to do, what
   should and should not show), and tick the task off below.
9. `CLAUDE.md` limits hold: no new dependency, no scheduling change, a new
   Windows API only in its own commit that names it, no benchmark outside
   `examples/` and the strip list.
10. Stop and report, changing nothing further, when: a check fails and
    the cause is not in your diff; a task needs a decision this plan does
    not make; or a "move only" task cannot be done without changing logic.

## Tasks

- [ ] 0. Baseline recordings (desktop, by hand)
- [ ] 1. Split the per-window stand-in out of the scroll view (move only)
- [ ] 2. Entry track: the maths, the drawing, the switch
- [ ] 3. Open on peeks, in tiling layouts
- [ ] 4. Restore on peeks
- [ ] 5. Close and minimize on peeks
- [ ] 6. Moves on peeks
- [ ] 7. Resize: decide
- [ ] 8. Remove what is no longer reached

### Task 0: baseline (desktop, by hand)

Not done yet. From a build of `88e5183`, with a long `duration_ms` (e.g.
3000) so one screenshot catches a mid-animation frame, record:

- On a tiling-layout workspace: open, close (`window_close` on),
  minimize, restore, a move, a resize that grows and one that shrinks.
- On the scrolling workspace: a new column, a scroll from rest, quick
  repeated presses, a column resized.
- A workspace switch both ways.

Include a see-through window with a backdrop and one themed by shades.
Once with `GLAZEWM_PERF=1`, keeping `perf.log`. These are the A/B
references for every task below. Tasks 1 and 2 can be done before the
recordings exist; task 3's live check cannot.

### Task 1: split the per-window stand-in out of the scroll view

**Move only.**

Goal: `animation/scroll_view.rs` (about 1200 lines) holds two things: a
generic per-window stand-in driver, and the scrolling layout's decision of
which columns move where. Separate them so later tasks can add entries
that are not columns.

Do:

1. Create `packages/wm/src/animation/peek_view.rs`. Move into it, as they
   are: `Role`, `Handover`, `Entry` (with `Entry::place`), `Landed`, the
   struct now called `ScrollView` renamed to `PeekView`, every method in
   its `impl` (`owns_window` through `upsert`), `track_overlays`,
   `carry_ring`, `carry_backdrop`, `peek_styles`.
2. Leave in `scroll_view.rs`: the module doc, `sync_scroll_view`,
   `drive_moving`, `flush_overlays`, `report_stuck_windows`,
   `column_moves`, `opening_rect`, `displayed_scrolling_workspaces`,
   `tiling_windows`, `target_rect`.
3. `last_rects` and `reported_stuck` belong to the scrolling layout, but
   `take_departed` prunes `last_rects` too. Keep both on `PeekView` for
   this task (moving state is a logic change); give them and `entries`
   `pub(super)` visibility.
4. In `AnimationManager` rename the field `scroll_view` to `peeks`, and
   update every user (`manager.rs`, `platform_sync.rs`, `tab_bars.rs`,
   `events/handle_mouse_move.rs`). `grep -rn "scroll_view" packages/wm/src`
   afterwards should list only the module itself, `mod.rs`, and calls to
   the functions that stayed in it.
5. Move the tests with the code they test.

Do not: change any function body beyond paths and names; merge or split
functions; touch `wm-platform`.

Done when: checks pass, test count for `wm` is unchanged (273), and `git
diff --stat` shows the two files' line counts adding up to about the old
one.

Commit: `refactor(animations): split the per-window stand-in out of the
scroll view`.

Live check (add to `TODO.md`): scrolling works as before: a scroll from
rest, quick repeated presses, a new column, a column leaving. Nothing
should differ from the baseline.

### Task 2: entry track

Three steps, three commits. No animation uses any of it yet.

**2a. The maths** (`packages/wm/src/animation/transit.rs`, new).

```rs
/// Which way a window's entry track runs.
pub enum Direction { In, Out }

/// A window entering or leaving: zoomed, slid or faded by how far its
/// entry track has run.
pub struct Transit {
  pub direction: Direction,
  pub style: WindowTransitionStyle,
  /// Opacity at the away end, as a fraction of the window's own.
  pub away_opacity: f32,
  /// Runs 0 to 1 over the transition's duration and easing; only its
  /// progress is read, never its rects.
  pub clock: WindowAnimationState,
}

impl Transit {
  /// How far in the window is at `now`: 0 away, 1 at rest.
  pub fn amount_at(&self, now: Instant) -> f32;   // In: p, Out: 1 - p
  pub fn is_complete(&self) -> bool;
}

/// Where a window resting at `rest` is drawn when `amount` in, and at
/// what fraction of its opacity. `None` while it has no size.
pub fn transit_frame(
  rest: &Rect,
  style: &WindowTransitionStyle,
  away_opacity: f32,
  amount: f32,
) -> Option<(Rect, f32)>;
```

`transit_frame`, with `w`, `h` the size of `rest` and `a` = `amount`
clamped to 0..=1:

| Style | Frame |
|---|---|
| `None` | `rest` |
| `Zoom` | `rest` inset by `round(w * (1 - a) / 2)` left and right, `round(h * (1 - a) / 2)` top and bottom; `None` if that leaves no width or height |
| `SlideRight` | `rest` moved by `+round(w * (1 - a))` in x |
| `SlideLeft` | `-round(w * (1 - a))` in x |
| `SlideTop` | `-round(h * (1 - a))` in y |
| `SlideBottom` | `+round(h * (1 - a))` in y |

Opacity fraction: `away_opacity + (1 - away_opacity) * a`, clamped.

These match today's `compute_transition_start_rect` (`manager.rs`) and
`zoom_destination` (`resize_session.rs`); do not change those here.

Tests to write first, with `rest = Rect::from_ltrb(100, 50, 900, 650)`:

- `Zoom`, `a = 1.0` gives `rest`; `a = 0.5` gives `(300, 200, 700, 500)`;
  `a = 0.0` gives `None`; a rest of `801 x 601` at `a = 1.0` gives itself.
- `SlideRight`, `a = 0.25` gives `(700, 50, 1500, 650)`; `SlideTop`,
  `a = 0.5` gives `(100, -250, 900, 350)`; every slide at `a = 1.0` gives
  `rest`.
- `None` gives `rest` at any `a`.
- Opacity: `away 0.0, a 0.5` gives `0.5`; `away 1.0` gives `1.0` at any
  `a`; `away 0.4, a 0.0` gives `0.4`.
- `Transit::amount_at`: with a linear easing (`CubicBezier(1/3, 1/3, 2/3,
  2/3)`), 200 ms, clock started at `t0`: `In` at `t0 + 50 ms` is `0.25`,
  `Out` is `0.75` (tolerance `1e-3`).

Commit: `feat(animations): the entry track's frame maths`.

**2b. The drawing** (`packages/wm-platform/src/window_peek.rs`).

A pinned peek draws its window at its own size today (`show_pinned`
passes the frame's size to `placed_rects`). Add scaled drawing and a
fade for the copies:

1. Read `placed_rects` and its three tests in `native_surrogate.rs`, and
   `WorkspaceSurrogate::show_frame` and `set_opacity_frac`
   (`workspace_surrogate.rs`), which already draw a window scaled and
   faded in a pinned stand-in. Port their approach; do not call into
   `WorkspaceSurrogate`.
2. Add `WindowPeek::place_scaled(&mut self, rest: &Rect, frame: &Rect,
   clip: &Rect, opacity: u8, frac: f32, batch: &mut SurrogateBatch)`:
   `rest` is the window's rect at rest (what the thumbnail samples),
   `frame` where it is drawn this frame (any size), `frac` the entry
   track's opacity fraction. Pinned mode only: it samples `rest`'s size
   into `frame` cut at `clip`, draws the window at
   `scale_alpha(window_opacity, frac)` and the copies at
   `scale_alpha(u8::MAX, frac)` (`OverlayCopies::set_opacity`), and places
   the copies at `frame`. In `Live` mode it logs at debug and calls
   `place(frame, ...)`: callers never pick a transit for a live peek (task
   3 rule).
3. `place` must keep behaving exactly as now. Do not route it through
   `place_scaled`.
4. Unit-test what is pure: if you add a helper computing the rects, test
   that `frame == rest` gives the same rects `show_pinned` computes today,
   and that a half-size centred `frame` gives a destination half the size.

Commit: `feat(peek): draw a window scaled and faded`.

**2c. The switch** (`packages/wm-platform/src/isolation.rs`).

Add a word `peeks` to `GLAZEWM_ISOLATE` setting `Isolation::peeks`,
documented in the module doc and `TODO.md` ("Live-check switch") as:
"draw open, close, minimize, restore and moves with pinned peeks instead
of `ResizeSession`, for the kinds already moved over". Extend the two
tests there. Unset, nothing changes.

Commit: `feat(isolation): a switch for animations drawn by peeks`.

### Task 3: open on peeks, in tiling layouts

This is the task that shows whether the plan is right.

Today: `platform_sync.rs` picks `entry_transition` and calls
`start_open_animation` (`manager.rs`), which begins a `ResizeSession`.

Do, all behind `isolation::current().peeks`:

1. `Entry` (`peek_view.rs`) gets `transit: Option<Transit>`. In
   `PeekView::tick`, for `Role::Moving`: start the transit's clock with
   the motion's (`start_clock(clock_origin)`); if `transit` is `Some`,
   compute `transit_frame(&rect, ...)` from the motion's current rect and
   call `place_scaled`, hiding the peek for `None`; else `place` as now. A
   moving entry is complete when its motion **and** its transit are.
2. Add `PeekView::enter(window, target, clip, params:
   &WindowTransitionParams, config, is_focused, now, batch)`: as `upsert`
   with `from == target`, `Role::Moving { ends_shown: true }`, the motion
   lasting the transition's duration, and `transit: Some(Transit { In,
   .. })`. Returns `false`, creating nothing, when the peek would be
   `Live` (same test `upsert` uses: a backdrop style that is not
   `is_copyable`).
3. In `platform_sync.rs`, where `entry_transition` is `Some` for an open
   (not a restore: task 4), and the switch is on, and the window is not a
   scrolling column: call `enter`. If it returns `true`, do not call
   `start_open_animation`. The window is then `peeks.is_moving`, so the
   existing code freezes it, moves its cloaked real window to its tile
   (`follow_retargeted_column`), and hands it back when the entry lands.
   Read that path once end to end before writing anything, and confirm
   each of those three statements in the code.

   One trap: `is_frozen_by_ws_animation` is computed a few lines
   *above* `entry_transition`, so it is still `false` after `enter`.
   Left so, the window takes the non-animated path and is uncloaked at
   once. Recompute it after a successful `enter`.
4. Install the monitor's vsync waiter and start the timer as
   `drive_moving` does (`scroll_view.rs`).
5. A retarget while entering (another window opens): `upsert` already
   retargets the motion of an entry that is moving. Make sure it keeps
   `transit` untouched. This is `ANIMATION_REVIEW.md` finding 9; add a
   unit test that a retargeted entry's `amount_at` is unchanged.
6. `take_departed` (scroll sync) removes entries of windows that are not
   scrolling columns. Entries with a `transit` must survive it: give
   `Entry` an `is_column: bool`, set by `sync_scroll_view`'s `upsert`
   calls, and only take departed column entries.

Do not: touch `start_open_animation` or `ResizeSession`; apply this to
scrolling columns (a new column keeps widening in place; whether it
should also zoom or fade is the owner's call, `ANIMATION_REVIEW.md` 17);
handle `Live` peeks.

Unit tests: completion needs both tracks; the retarget test above;
`take_departed` leaves a non-column entry.

Commit: `feat(animations): open windows through pinned peeks behind the
switch`.

Live check (the gate for everything after it), on a tiling-layout
workspace, `GLAZEWM_ISOLATE=peeks`, each `window_open` style at
`duration_ms: 2000` and at 150, against the baseline and against the
same build with the switch off:

- the window enters with its border and backdrop moving with it, never a
  frame apart;
- a see-through window and a shades-themed one show no flash, white frame
  or lighter look as they land;
- nothing shows twice or not at all at the start or the end;
- opening a second window mid-way does not restart or misplace the first;
- neighbours making room stay in step.

If any of these is worse than with the switch off and the cause is not a
plain bug in the new code, stop: report which, with a screenshot. Do not
tune hand-over timings to make it pass.

### Task 4: restore on peeks

As task 3, for the restore branch of `entry_transition`, with these
differences:

- The window may be on screen (the OS restored it) or still natively
  minimized (`toggle-minimized`; `ANIMATION_REVIEW.md` finding 10). For
  the second: cloak it, then restore it natively at its target
  (`NativeWindow::restore(Some(rect))`), then `enter`. For the first:
  `upsert` already warms the peek up under a window still shown and takes
  over at the cloak; confirm `commit_pending_cloaks` is reached for it
  (`covers_shown_window`).
- A minimize still playing is ended first (`9a23bf4` does this for the
  old path; keep that order).

Live check: `toggle-minimized` twice two seconds apart, and a taskbar
restore, each `window_minimize` style; then both within the animation.

### Task 5: close and minimize on peeks

Different in kind: nothing is handed back, and a closing window is out
of the tree, so `track_overlays`' `state.window_by_id(id)` finds nothing.

Do, behind the switch:

1. Add `Role::Leaving`: drawn like `Moving`, never handed back. When its
   transit completes, `tick` reports it in a new `Landed::left` list and
   drops the entry.
2. `Entry` needs what `track_overlays`, `carry_ring` and
   `carry_backdrop` read from the container: keep the `NativeWindow` and
   the effect set on the entry for a leaving window, and branch on that
   where they call `window_by_id`. Change those three functions as little
   as possible; no change for other roles.
3. `animate_away` (`close_window.rs`): with the switch on and a pinned
   peek possible, add a leaving entry instead of calling
   `start_away_animation`. The take-over is the scroll's: `warm_up`,
   one composited frame, `take_over`, then cloak. That removes
   `ANIMATION_REVIEW.md` finding 11. Use the pass's shared flush
   (`commit_pending_cloaks`) if the close's sync reaches it for this
   window; if it does not, cloak on the tick after the peek was shown.
   Decide which by reading, and say which in the commit.
4. `update_internal` (`manager.rs`): for each id in `Landed::left`, do
   what the "away_done" block does now: remove the overlays, send
   `WM_CLOSE` and `mark_close_sent` for a close; nothing more for a
   minimize.
5. `has_close_animation`, `has_minimize_animation`, `has_active_surrogate`
   and `stands_in_for` must answer for leaving entries too: each has
   callers that depend on it (`handle_window_hidden`, `platform_sync`).
6. `9dfab1f` (window gone mid-close) and `5f39127` (no thumbnail): keep
   both behaviours on the new path.

Unit tests: a leaving entry is reported once and dropped; the four
queries above answer `true` while it plays.

Live check: `TODO.md` 32, 34 and 35 again with the switch on, plus a
refused close ("Save changes?" in Notepad): the window comes back.

### Task 6: moves on peeks

A move that stays on screen uses `ResizeSession` today; the scroll's
strip already moves on peeks. In `sync_scroll_view`, `strip_moves` is
what decides. For tiling layouts there is no such sync.

Do, behind the switch: a pure move (same size before and after, window
shown before and after) of a tiled window becomes a `Role::Moving` entry
(`upsert` with `from` = previous target). Resizes stay on the old path.
In the scrolling layout, widen `strip_moves` to any column move with the
switch on.

Watch for: a relayout that moves one window and resizes its neighbour.
`cycle_has_resize` makes both share `window_resize`'s timing today; keep
that, by giving the peek's motion the same duration and easing.

Live check: swap two equal windows; `TODO.md` 15 and 16.

### Task 7: resize, a decision

Do not start without the owner's answer. The options, to put to them
with recordings from tasks 3 to 6:

- (a) Leave resize on `ResizeSession`. Smallest risk; two systems remain,
  but only one kind of animation uses the old one.
- (b) `window_resize.style: "stretch"` on peeks: the content scales to
  the animated rect and the app resizes on the first frame. `place_scaled`
  already draws this. Simple, and how Hyprland looks; text is blurred
  mid-resize.
- (c) Port the `fill` style (grow-only registration, gap fill, edge
  colour, mid-motion hand-off). The hard part the second version
  described; only if (b) is not acceptable.

### Task 8: remove what is no longer reached

Only for kinds whose live check passed and whose default was flipped (a
one-line change per kind, its own commit, after the owner says so).
Then delete the old branch for that kind: `start_open_animation`,
`start_away_animation`, `slide_in_monitor_rects`, `ResizeSession::zoom`
and `update_zoom_fade`, `away_windows`, as each loses its last caller.
`grep` for callers before deleting; one commit per kind.

## Later, unchanged from the second version

Sharing a core between the switch (`WorkspaceSurrogate`) and the scroll
(`WindowPeek`), and one overlay hand-over for both. Still worth doing for
upkeep, after task 6, when the peek's side of it has settled. Its
findings stand (`git show ec8e0fe:STAND_INS_PLAN.md`, "Step 1 findings"):
about 80 shared lines of wrappers, `Live` chosen twice by the same rule,
and five differences a merge must carry (hand-back from below or above
the window; copies faded or held at full; one backdrop style or both;
fixed or clip-following pinned area; where a `Live` stand-in is cut).
Task 2b narrows the second of those.

Side fixes, any time: `TODO.md` "Still open" 4 (the switch's backdrop
blends between focus styles).

## Risks

- The hand-over code being reused was tuned against recordings for
  columns that slide. A window that scales or fades is new to it: the
  copies' opacity and a frame that is not the window's size are the two
  places to look first if something flashes.
- `Live` peeks (rounded corners, parallax != 1) stay on the old path
  throughout. Square corners never pick `Live`, so the owner's config is
  covered; say so in `TODO.md` rather than leave it implied.
- `manager.rs` is about 4200 lines. Put new code in `peek_view.rs` and
  `transit.rs`, not there.
- Endpoint safety (`CLAUDE.md`): live checks stay manual and low-volume.
