# Plan: one way to animate windows

Revised 2026-11-10 (fourth version; the third is `git show
229fc21:STAND_INS_PLAN.md`, the second `git show
ec8e0fe:STAND_INS_PLAN.md`). Written so a session that has not seen the
earlier ones can carry it out one task at a time. Read `CLAUDE.md`,
`TODO.md` and `ANIMATION_REVIEW.md` first.

Nothing in this plan has been seen running. It is a reading of the code
as of `15deb4d`. Task 3 is where it proves itself on screen; if its live
check fails, stop there.

## What changed from the third version, and why

The owner's answers (2026-11-10) and a closer read of the overview:

- **Both layouts, every task.** The owner uses the scrolling and the
  tiling layouts about equally. The third version's gate (task 3) left
  scrolling columns out, so it would have changed nothing in half their
  use. Each task now has a step for each layout.
- **Close before restore.** Close is what the owner asked about.
- **A closing window stays below its neighbours** (owner's decision).
  Task 4 says what that means on peeks and what it costs.
- **Pinning is one of four reasons the overview is smoother**, and the
  third version only took that one. See the next section: the frame's
  place in the tick (task 7) and curves that turn round instead of
  restarting (task 2a, `Transit::reversed`) are new.
- A fade-style close left its border and backdrop at full strength until
  the end (`15deb4d` fixes it on the old path). Task 2b's copies must
  fade the same way; task 4 checks it.

## Why the overview is smoother, and which task takes each lever

Read from the code (`native_overview.rs`, `overview_layout.rs`,
`manager.rs`, `main.rs`), not measured. Largest first.

| | Overview | A real window's move, open or close today | Taken by |
|---|---|---|---|
| A. What moves | One window per monitor that never moves; a frame only updates thumbnails in it | Three top-level windows per animating window (stand-in, ring, backdrop) moved by `SetWindowPos`, plus thumbnail size and opacity by another route; all must land in one composition | Tasks 3 to 6 |
| B. When in the frame | Its own thread wakes on the vblank and does nothing but spring steps and thumbnail updates: done well inside a millisecond or two | The WM thread first works off queued events for up to 4 ms (`CATCH_UP_BUDGET`), then runs a whole `platform_sync`. DWM has no transaction for thumbnail updates, so a frame's updates that straddle a composition show half applied | Task 7 |
| C. Curves | Springs stepped by `dt`: a retarget keeps the speed, a reversal turns round | A fixed-length curve that restarts from rest at each retarget (columns excepted, `retargeted`); a close during an open jumps | Task 2a, tasks 4 and 6 |
| D. Hand-over | None: nothing is cloaked, uncloaked or waited for | A flush before the first frame, a swap at the end, an app relaying out in between | Not removable. Tasks 3 to 6 reuse the scroll's, the one tuned live; entry and exit never resize the app |

What this plan does not take, on purpose: one surface per monitor
instead of one per window (A, second half), and a thread of its own for
stand-in frames (B, in full). Both are under "Later"; task 7's
measurement says whether they are worth it.

## What changed from the second version, and why (still holds)

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

Owner's call, 2026-11-10: they cannot run live checks for now, so tasks
3b to 6 are built one after another without waiting for 3a's gate. All
of it stays behind the switch, and every live check still has to be run
before a default is flipped. A flaw found later in 3a may mean rework in
what was built on it. Tasks 8 and 9 still wait: for the owner's decision, and for
seen-working code.

Owner's answers, 2026-11-11:

- No `perf.log` can be made for now. Task 7 is built anyway, behind its
  own word, and measured later.
- A restore plays minimize's curve time-reversed (`ANIMATION_REVIEW.md`
  15): task 5.
- Floating windows get the open animation (finding 18): task 6b.
- "Just make the new thing the default": task 7b. Read as: peeks are the
  default for every kind built, and `window_close` is enabled by
  default. The old paths stay in the code behind an opt-out word.
- Stretch looked wrong because "it zooms a bit as it ends": task 8 (b).
- A stripped laptop build is wanted once 4 to 7b are in:
  `tryout/stand-ins`, then `test/stand-ins` cut from it (`CLAUDE.md`
  "Cutting a test build").

- [ ] 0. Baseline recordings (desktop, by hand)
- [x] 1. Split the per-window stand-in out of the scroll view (move only)
- [x] 2. Entry track: the maths, the drawing, the switch
- [ ] 3. Open on peeks: tiling layouts (the gate), then scrolling columns
  (3a and 3b built, not checked live)
- [ ] 4. Close and minimize on peeks, both layouts (built, not checked
  live)
- [ ] 5. Restore on peeks (built, not checked live)
- [ ] 6. Moves on peeks (built, not checked live)
- [ ] 6b. Floating windows open on peeks (built, not checked live)
- [ ] 7. The frame's place in the tick: build behind `peeksfirst`,
  measured later (built, not checked live or measured)
- [ ] 7b. Peeks by default, `window_close` on by default (built, not
  checked live)
- [ ] 8. Resize: `stretch` on peeks (owner's choice, 2026-11-11; first
  built as wait, then stretch, rebuilt instant after the owner's live
  run; not checked live)
- [ ] 9. Remove what is no longer reached
- [ ] 10. `fill` on peeks, for mouse drags first, then keyboard resizes
  (planned and reviewed 2026-11-12: 10a answered by reading; the owner's
  pre-check in 10a still to run; 10b, 10c, 10e-1 and 10e-2 built, not
  checked live; 10d is the owner's call; 10e-3 done on the owner's
  say-so, 2026-11-12; 10f ports under way behind `morepeeks`)

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
  /// How far in the window was when this transit began: 0 for an open,
  /// 1 for a close, anything between for one turned round mid-way.
  pub from: f32,
  /// Runs 0 to 1 over the transition's duration and easing; only its
  /// progress is read, never its rects.
  pub clock: WindowAnimationState,
}

impl Transit {
  /// How far in the window is at `now`: 0 away, 1 at rest.
  /// In: `from + (1 - from) * p`. Out: `from * (1 - p)`.
  pub fn amount_at(&self, now: Instant) -> f32;
  pub fn is_complete(&self) -> bool;
  /// The transit the other way, from where this one is at `now`, over
  /// `params`' duration scaled by how far it has left to go (at least
  /// one frame). Its clock is unstarted: the caller starts it at `now`.
  pub fn reversed(
    &self,
    params: &WindowTransitionParams,
    now: Instant,
  ) -> Self;
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
  2/3)`), 200 ms, clock started at `t0`: `In` (`from` 0) at `t0 + 50 ms`
  is `0.25`, `Out` (`from` 1) is `0.75` (tolerance `1e-3`).
- `Transit::reversed`: that `In` reversed at `t0 + 50 ms` with the same
  params is `Out`, `from` `0.25`, lasting 50 ms; started at `t0 + 50 ms`
  its amount is `0.25` there, `0.125` at `t0 + 75 ms`, `0` at
  `t0 + 100 ms`. Reversed again at once it is `In` from the same amount:
  no step either way.

Why `from` and not a second curve: a reversal must start at the amount
on screen or it jumps (`ANIMATION_REVIEW.md` 9 and 14), and the overview
never jumps for exactly this reason (lever C). A spring easing keeps its
speed through `WindowAnimationState::retargeted` already; use that for
the clock when the easing is a spring, and say so in the commit.

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
   3 rule). The copies must fade with the window: `15deb4d` fixed the
   old path, where a fading close kept its border and backdrop at full
   strength to the end.
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

### Task 3: open on peeks

As built in task 2 (`30e2937`, `ca2df53`), where it differs from the
text of task 2 above:

- Build a transit with `Transit::new(direction, params, from)`, never by
  hand: its clock's rects carry the amount (`AMOUNT_SCALE`).
- `Transit::is_complete` reads the wall clock, as
  `WindowAnimationState::is_complete` does.
- `WindowPeek` remembers its fade (`frac`); `place` resets it to 1.
  `WindowPeek::carry_ring` still covers the ring at full opacity, so a
  ring carried mid-fade would show at full strength until the next
  `place_scaled`: look at `carry_ring` (`peek_view.rs`) for entries with
  a transit, in 3a for an open and in task 4 for a close.
- `transit` is `#[allow(dead_code)]` in `animation/mod.rs` until used:
  remove the allow in 3a.
- The start-up warning in `main.rs` says `GLAZEWM_ISOLATE` "leaves
  parts of moving windows out"; reword it in 3a so `peeks` alone reads
  right.

This is the task that shows whether the plan is right. Two steps, two
commits: 3a tiling layouts (the gate), 3b scrolling columns. Do not
start 3b before 3a's live check has passed.

**3a. Tiling layouts.**

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
   (not a restore: task 5), and the switch is on, and the window is not a
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
5. A retarget while entering (another window opens and the first one's
   tile changes). Nothing does this for a tiling-layout window today:
   `upsert`'s only caller is `sync_scroll_view`, and `enter` runs once
   (found when 3a was first attempted). Without it the peek keeps
   drawing the old tile while `follow_retargeted_column` moves the real
   window to the new one, and the hand-back shows it misplaced.

   So: in `platform_sync`'s `Frozen` branch, beside
   `follow_retargeted_column`, when the switch is on, the entry has a
   transit, is not a column, and `owned_target` differs from the new
   rect, call a new `PeekView::retarget_entering(id, target, duration_ms,
   easing, now)`. It retargets the motion only
   (`WindowAnimationState::retargeted`, clock started at `now`) and
   leaves `transit` untouched. Timing: what `start_animation_if_needed`
   would pick for that window (`window_move`, or `window_resize` when
   the tile's size changes or `cycle_has_resize`), so it stays in step
   with its neighbours, as the old path's `update_target` does. Not the
   scroll's timing. Update the peek's `rest` for the new tile's size
   through whatever `upsert` uses for a retargeted column.

   This is `ANIMATION_REVIEW.md` finding 9; add a unit test that a
   retargeted entry's `amount_at` is unchanged.
6. `take_departed` (scroll sync) removes entries of windows that are not
   scrolling columns. Entries with a `transit` must survive it: give
   `Entry` an `is_column: bool`, set by `sync_scroll_view`'s `upsert`
   calls, and only take departed column entries.

7. Parity with `start_open_animation`, so the switch changes how an
   open is drawn and nothing else:
   - `enter` returns `false` for a transition with nothing to show (a
     stationary style that is not a zoom, with `away_opacity >= 1`);
   - a stationary, non-zoom style (a plain fade) holds its transit at
     the start for `OPEN_PAINT_GRACE` (`clock.start_delay`), as the old
     path does. Slides and zooms get none (`ANIMATION_REVIEW.md` 12
     stays open for both paths).
8. The ring copy must fade with the window. `track_overlays` runs after
   `tick`, and `carry_ring` calls `WindowPeek::carry_ring`, which covers
   the ring at full opacity (`OverlayCopies::cover_ring`): on a fading
   entry that would undo `place_scaled` every pass. Make
   `WindowPeek::carry_ring` (and `show_backdrop`, if it does the same)
   apply the peek's `frac` after covering. At `frac` 1 the result must
   be what it is now. This is in `wm-platform`, outside the `Handover`
   steps rule 5 protects; do not change those.
9. `Entry` holds a `WindowPeek`, a real window, so the unit tests below
   are on pure functions: factor "is this moving entry complete",
   "which entries does `take_departed` take" and the retarget into
   functions over `motion`, `transit` and `is_column`, and test those.
10. Strays (found while building 3a): nothing else removes an entering
    entry whose window goes away, since `take_departed` takes columns
    only. `drop_strays` (`peek_view.rs`) runs on every `platform_sync`,
    right after the scroll view's sync and outside its condition, and
    costs nothing while no non-column entry exists. It takes non-column
    entries whose window is unmanaged or no longer shown (workspace not
    displayed, hidden in its layout, minimized, or playing a close or
    minimize animation); `is_stray` decides. The peek is dropped at once,
    with no hand-back, and a still-managed window is queued for redraw,
    as `sync_scroll_view` does for departed columns: the ordinary path
    then hides it, and uncloaks it once it is shown again.
    `unmanage_window` also drops the entry directly
    (`forget_entering`), as an unfocused window's removal may queue no
    sync.

    `time_to_land` reads the later of the motion's and the transit's
    remaining time.

Do not: touch `start_open_animation` or `ResizeSession`; touch scrolling
columns in this step; handle `Live` peeks.

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

Keep `perf.log` (`GLAZEWM_PERF=1`) of one run with the switch on and one
with it off: `tick ms/frame` during an open beside three windows. Task 7
needs both numbers.

If any of these is worse than with the switch off and the cause is not a
plain bug in the new code, stop: report which, with a screenshot. Do not
tune hand-over timings to make it pass.

**3b. Scrolling columns.** A new column already opens on a peek
(`opening_rect`), widening in place on `animations.scroll`'s timing;
`window_open` is ignored there (`ANIMATION_REVIEW.md` 17). With the
switch on, give that entry `transit: Some(Transit { In, .. })` from
`window_open` as well: the column's rect track stays the scroll's, the
entry track is `window_open`'s style, opacity and timing. A column is
complete when both are (3a step 1 already says so).

One look decision this adds, for the owner at the live check: a
`slide_*` style on a column that is also widening. If it reads badly,
columns take `zoom` and the fade only; do not invent a third behaviour.

Commit: `feat(animations): let a new column enter by window_open behind
the switch`.

Live check, scrolling workspace, switch on: a new column with each
`window_open` style at 2000 ms and 150 ms; one opened while the strip is
still scrolling; one opened off the right edge. Same five points as 3a.

As built: `sync_scroll_view` hands `upsert` a `Transit::opening` for a
window in `new_ids` (no target position yet, the test an open uses in
`platform_sync`); `upsert` keeps it only on a peek it creates moving and
pinned. A retargeted column keeps an unfinished transit and drops it
when put at rest (`kept_transit`), and `upsert` draws through
`Entry::draw`, which is `place` while there is no transit.

### Task 4: close and minimize on peeks

Different in kind: nothing is handed back, and a closing window is out
of the tree, so `track_overlays`' `state.window_by_id(id)` finds nothing.

Decided by the owner (2026-11-10): **a closing window is drawn below
its neighbours**, as `start_away_animation` does today (`place_at_top:
false`; the neighbours' stand-ins go to `HWND_TOP`). On peeks: each peek
is a top-level window of its own, so put a leaving peek directly below
its real window when it is created and never raise it (no
`begin_hand_back`). Every other peek and stand-in is created above it.

What that costs, so nobody "fixes" it: in a tiled layout the neighbours
grow over the closing window, so what shows of a close is the part they
have not reached yet. With the sample config's curves the neighbours
start fast (`cubic_bezier(0.2, 0, 0, 1)`) while the close eases in
(`ease_in_cubic`, 12% done at half time): next to a neighbour a close is
all but covered, and plays in full only for a workspace's last window.
That is a config matter (`window_close.easing`, `style: "zoom"`), noted
in `TODO.md`; this task does not change defaults or the order.

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
7. A close or minimize of a window whose entry is still playing (task
   3): do not start over from the layout rect (`ANIMATION_REVIEW.md` 14).
   Its entry becomes `Role::Leaving` where it is, with
   `transit.reversed(..)` for the close's params.
8. Scrolling layout: a closing column gets the same leaving entry, with
   `is_column: false` so `take_departed` leaves it alone; the strip
   closes the gap by the scroll as now. Check `column_moves` does not
   read the leaving window (it is out of the tree).

Unit tests: a leaving entry is reported once and dropped; the four
queries above answer `true` while it plays; an entering entry closed
mid-way leaves from its current amount.

Live check, in both layouts, `window_close` on at 3000 ms and at 150 ms,
styles `none` (fade), `zoom` and one slide: `TODO.md` 32, 34 and 35
again with the switch on, plus

- the last window of a workspace, and one with a neighbour: both play;
  the second is covered as the neighbour arrives, never before;
- border and backdrop fade or shrink with the window, never linger;
- no frame at the start with neither window nor stand-in (finding 11);
- close a window still opening: it turns round where it is;
- a refused close ("Save changes?" in Notepad): the window comes back.

As built, for task 5:

- `animate_away` calls `peek_view::leave_on_peek`; `PeekView::leave`
  returns `Leave::Leaving`, `Skipped` (nothing to show, or no
  thumbnail: closed or minimized at once) or `Unsupported` (a `Live`
  peek, or an entry in a role other than `Moving`, e.g. a partly
  visible column at rest: the old path, as before).
- A leaving entry is `Role::Leaving(AwayTransition)`, `is_column:
  false`, its motion held where it was (`held_motion`), and keeps a
  `Departing` (native window, effect set, frames read before it went,
  `awaits_cloak`). A minimized window's frames are its taskbar
  button's, so they are never re-read.
- Take-over: a close's sync never reaches `commit_pending_cloaks` for
  it (out of the tree), so the first tick takes over and cloaks it. A
  minimize can't wait (the caller minimizes it next), so
  `leave_on_peek` flushes once (`DwmFlush`) and takes over before it
  returns; the window is minimized, never cloaked, as on the old path.
- `remove_animation` drops a leaving entry (`forget_leaving`), as it
  dropped an away surrogate: the restore of a still-playing minimize
  (`platform_sync`, `9a23bf4`) therefore still ends it. Task 5 turns it
  round there instead: the entry's transit is `Out`, so
  `leaving_transit`'s counterpart is `transit.reversed(..)` started at
  the frame time, the role back to `Moving { ends_shown: true }`,
  `departing` cleared and `is_column` as it was.
- `sync_overlays` keeps a leaving window's overlays (the peek copies
  them) until it has left; `column_moves` skips a leaving window, which
  a minimizing column still is in the tree.

### Task 5: restore on peeks

As task 3a, for the restore branch of `entry_transition`, with these
differences:

- The window may be on screen (the OS restored it) or still natively
  minimized (`toggle-minimized`; `ANIMATION_REVIEW.md` finding 10). For
  the second: cloak it, then restore it natively at its target
  (`NativeWindow::restore(Some(rect))`), then `enter`. For the first:
  `upsert` already warms the peek up under a window still shown and takes
  over at the cloak; confirm `commit_pending_cloaks` is reached for it
  (`covers_shown_window`).
- A minimize still playing is turned round, not ended: its leaving
  entry (task 4) becomes a moving one with `transit.reversed(..)`, so
  the window grows back from where it had shrunk to. `9a23bf4` ends it
  and starts again on the old path; with the switch off that stays.
- Scrolling layout: a restored column re-enters as a new column does
  (3b).
- Curve (owner, 2026-11-11): a restore plays `window_minimize`'s easing
  time-reversed, so an ease-in minimize comes back as an ease-out
  (`ANIMATION_REVIEW.md` 15). Add the reversal to `EasingFunction`
  (`wm-common`): a cubic bezier `(x1, y1, x2, y2)` becomes
  `(1 - x2, 1 - y2, 1 - x1, 1 - y1)`; named curves map through their
  bezier or to their named opposite; a spring is its own reverse. Use
  it in `WindowTransitionParams::from_minimize`'s restore use on BOTH
  paths (this one is not behind the switch: the config's doc already
  says "plays in reverse"). Unit-test the mapping, and that reversing
  twice gives the curve back.

Live check, in both layouts: `toggle-minimized` twice two seconds apart,
and a taskbar restore, each `window_minimize` style; then both within
the animation: the window turns round where it was, with no jump.

As built (`9699b12` curve, `9e19c06` restore), for task 6:

- `WindowTransitionParams::from_restore` is the restore's params on
  both paths; `EasingFunction::time_reversed` leaves springs alone.
- `restore_on_peek` (`peek_view.rs`) is called from the restore branch
  for a tiled, non-column window; floating windows keep the old path.
  It turns a minimize still on its peek round (`PeekView::turn_round`),
  else enters as an open does, after the same checks as `enter`, so
  nothing is cloaked or restored for a window that falls back.
  `restore_cloaked` cloaks and then restores (`restore(Some(rect))`) a
  window still natively minimized, before any peek goes up.
- A turned-round entry is an ordinary entering one: role `Moving {
  ends_shown: true }`, `departing` dropped, motion held where it was;
  in a tiling layout the `Frozen` pass retargets it to its tile
  (`retarget_entering`, `movement_timing`). Task 6's pure moves must
  leave such entries (`is_entering`) to that retarget, as for an open.
- `platform_sync` no longer ends a minimize on its peek at the start of
  a restore's redraw (`leaves_on_peek`); it ends it after the entry
  transition, before `start_open_animation`, if it wasn't turned round.
- Scrolling layout: `sync_scroll_view` takes windows flagged
  `is_window_restore` as it takes `new_ids` (`opening_ids`: opening
  rect, the strip moves, a transit by the restore's params), and turns
  round one still minimizing, which then moves with the strip. A
  turned-round column whose rect didn't change keeps its entry through
  the `owned_target` check, untouched.
- A peek goes up just below its real window, which minimized is at the
  bottom of the z-order, and nothing restacks a peek before
  `begin_hand_back`. So a restore's entering or turned-round peek is
  raised to `HWND_TOP` (`PeekView::raise_restored`,
  `WindowPeek::raise_to_top`), as the old path's `place_at_top` does;
  a leaving one is not. Opens are left where they are: a new window is
  normally at the top already.

### Task 6: moves on peeks

A move that stays on screen uses `ResizeSession` today; the scroll's
strip already moves on peeks. In `sync_scroll_view`, `strip_moves` is
what decides. For tiling layouts there is no such sync.

Do, behind the switch: a pure move (same size before and after, window
shown before and after) of a tiled window becomes a `Role::Moving` entry
from its previous target. Not through `upsert`, whose only caller is
`sync_scroll_view` (scrolling workspaces only): where `platform_sync`
would call `start_animation_if_needed`, as 3a's `enter_on_peek` sits
beside `start_open_animation`. Resizes stay on the old path. In the
scrolling layout, widen `strip_moves` to any column move with the
switch on.

A move retargeted mid-way (a second swap, a held key) keeps its speed
(`retargeted`): lever C. For a column `upsert` does it; for a tiled
window the `Frozen` pass's retarget, as 3a's `retarget_entering` does
for an entering one. Check that holds for both, with a unit test.

Watch for: a relayout that moves one window and resizes its neighbour.
`cycle_has_resize` makes both share `window_resize`'s timing today; keep
that, by giving the peek's motion the same duration and easing.

Live check: swap two equal windows; the same swap pressed twice within
the animation (no stop and restart in between); `TODO.md` 15 and 16.

As built, for tasks 6b and 7:

- `platform_sync` calls `peek_view::move_on_peek` in place of
  `start_animation_if_needed` for a shown, tiled (`WindowState::Tiling`,
  not a column) window with no state change, whose move keeps its size
  past `window_move.threshold_px` (`is_pure_move`), with no old-path
  stand-in on it (`has_other_stand_in`), from inside its monitor, and
  not `Live`. Timing: `movement_timing`, so `window_resize`'s when the
  cycle resizes. `PeekView::move_tile` makes the entry (`is_column:
  false`, no transit), or takes on again one still handing back.
- Take-over: the `Frozen` branch's `TakeOver` step, into
  `commit_pending_cloaks`' one flush. Around it, `cover_moved_overlays`
  (before) and `hand_over_moved_overlays` (after) hand the ring and
  backdrop over as `sync_scroll_view`'s flush does. Then the peeks go
  to `HWND_TOP` (`raise_moving`, formerly `raise_restored`), never
  before their windows are cloaked.
- Retarget: `is_entering` and `retarget_entering` became `follows_tile`
  and `retarget_tile`, covering any non-column `Moving` entry, at
  `frame_now` (the pass's instant, which old-path retargets use) rather
  than `frame_time`. A retarget that resizes stays on the peek
  (`place`, as a column), as 3a does for an entering window.
- 6b: this path keeps floating windows out (the `Tiling` test) and a
  floating-to-tiling drop (`is_state_change`). 6b's open goes through
  `enter_on_peek`, not here, but its entry is a non-column `Moving` one,
  so `follows_tile` holds and the `Frozen` pass retargets it whenever
  its rect changes mid-entry: check that is wanted for a floating rect.
- 7: moving entries are ticked by `PeekView::tick` like the others; the
  only new work outside the tick is in the relayout pass.
- Scrolling: with the switch on, a peek drawing one column no longer
  makes the whole strip move (`moves_on_peek`): pure moves and columns
  already drawn go on peeks, resizes on screen stay on the old path.

### Task 6b: floating windows open on peeks

Owner, 2026-11-11: floating windows animate open like tiled ones
(`ANIMATION_REVIEW.md` 18). Today `platform_sync` gives a new floating
window no `entry_transition`. With peeks on, let it take the 3a path
(`enter_on_peek`) at its floating rect; with peeks off, nothing changes
(the old path stays without it). Check first what excludes a floating
window from `entry_transition` and why, that the Frozen branch and the
hand-back treat a non-tiling window as 3a's three statements say, and
that a floating window partly off its monitor is clipped as slides are.
Fullscreen and maximized windows stay without. Live check: a floating
dialog and a floating app window, each `window_open` style; one opened
while a tiled window is still entering.

As built, for tasks 7 and 7b:

- `entry_transition` gives a new floating window `window_open` when
  `opens_floating_on_peek` (`platform_sync.rs`): the switch on, and not
  natively maximized (`reposition_window` would restore it first).
  Fullscreen ones were already out (`is_fullscreen`). The exclusion was
  a design choice (`ANIMATION_REVIEW.md` 18), not a fix: with the switch
  off it stands. A floating open that `enter_on_peek` turns down (a
  `Live` peek, nothing to show) opens at once: never
  `start_open_animation`. Floating restores keep the old path.
- The rest is 3a's path unchanged: a new window is cloaked from
  `manage_window` on, so the `Frozen` branch's step is `Keep` and
  `follow_retargeted_column` moves it to its floating rect; landed, it
  is handed back by the `Apply` branch as a tile is. Clip and vsync
  waiter are its workspace's monitor's (`window.monitor()`), cut at
  that monitor's edge as slides are.
- Z-order: its peek goes up just below its real window, and
  `windows_to_bring_to_front` then restacks the other floating windows
  directly below that window (asynchronously), over the peek. So a
  focused one's peek is raised after the pass (`peek_raises`, beside
  task 6's `raise_moving`); `begin_hand_back` later puts it directly
  above its window, already on top. An unfocused one stays where its
  window is.
- Retarget: `follows_tile` holds for it, so the `Frozen` pass retargets
  it, but a floating window that stays floating jumps there
  (`follow_duration` 0), as the old path's floating moves are not
  animated; its transit plays on. An app moving its own cloaked window
  mid-entry (a dialog centring itself) queues a redraw
  (`handle_window_moved_or_resized`), so the peek follows instead of
  landing where it was.
- 7b: nothing here reads the switch but `opens_floating_on_peek`,
  `enters_on_peek`'s and the `Frozen` branch's tests, and the app-move
  redraw: flipping `peeks` turns floating opens on with the rest.

### Task 7: the frame's place in the tick

Owner, 2026-11-11: no `perf.log` can be made for now; build step 2
behind `peeksfirst` and the `perf::Stage` of step 1, and measure later.
That is the go-ahead rule 9 asks for, for this change only. `peeksfirst`
stays off by default until the numbers exist (task 7b does not flip it).

Why: lever B. A peek frame is cheap (track maths and thumbnail updates,
no `platform_sync`, no `SetWindowPos` for a pinned peek), but it is
drawn where the old frames were: after `catch_up_before_frame`
(`main.rs`, up to 4 ms of queued events) and inside `update_internal`,
ahead of a `platform_sync` only by position. DWM applies each thumbnail
update at its next composition with no way to group them, so the later
and longer the frame's updates run, the likelier a composition falls
between a window and its border. The overview avoids that by doing
nothing else on its thread.

Do:

1. From task 3's two `perf.log` runs, and one more of a scroll: per
   frame, the time from the tick's wake to the last peek update, and
   its spread, on the fast monitor. Add a `perf::Stage` for the peek
   tick if none covers it (`PeekView::tick` plus its `track_overlays`
   and batch commit). Report the numbers to the owner.
2. Only if they say go, and only if the wake-to-last-update time is
   over about a quarter of that monitor's frame or varies by more than
   that: draw the peeks' frame before the catch-up. On an animation
   tick with `peeks.is_active()`, run the peek tick first, then
   `catch_up_before_frame`, then the rest of `update_internal`, which
   must not tick the peeks a second time that frame. Landed windows are
   still queued for the same pass's redraw. Behind a second
   `GLAZEWM_ISOLATE` word, `peeksfirst`.

Do not: move peeks to another thread here; touch `CATCH_UP_BUDGET` or
the timer thread.

Live check: 3a's and task 6's checks with `peeks,peeksfirst` against
`peeks`, on the fast monitor, with keys held so the queue is busy.

As built (step 1's stage and step 2 together, on the owner's go-ahead;
no numbers yet), for task 7b:

- `Isolation::peeks_first`, word `peeksfirst`, off by default. 7b must
  leave it so: `nopeeks` turns `peeks` off and nothing else.
- A frame with it on and `peeks.is_active()` at the top of the main
  loop: the loop skips its catch-up; the tick's branch drains stale
  ticks, then `AnimationManager::draw_peeks_first` (last frame time,
  `perf::begin_frame`, `begin_frame_times`: one `begin_redraw_pass`,
  `frame_now` and `clock_origin`; `PeekView::tick`; `finish_leaving` of
  what left; `track_overlays` and its batch commit), then
  `catch_up_before_frame` as before, then `finish_frame` (the rest of the
  old `update_internal`), which takes the pass up again at the same
  instants (`resume_pass`), starts animation clocks at the same
  `clock_origin`, queues what landed and does not tick the peeks again.
  With it off, `update_internal` is `finish_frame(None)`: the same steps
  in the same order as before.
- Carried across the catch-up: the frame's instants and `Landed` minus
  `left` (shown, hidden, released ids), queued for redraw by
  `queue_landed` after it, each looked up again with `window_by_id`, so
  a window closed or unmanaged meanwhile is skipped. A landed window an
  event's sync redrew already is handed back there; the later redraw
  finds nothing to do. Peeks created during the catch-up get their first
  tick next frame: ticked now, `note_composited` would claim a
  composition that hadn't happened.
- `perf::Stage::PeekFrame` (`peek_frame`), on both orders, frames with
  peeks only: from the timer thread's wake (`FrameClock::last_wake`, if
  since the last frame began, else now) to the peeks' commit
  (`perf::scope_since`). Inert without `GLAZEWM_PERF`. With
  `peeksfirst`, the catch-up runs inside the perf frame: its stages
  count there, outside `tick`.
- Not moved: tab bars of moving stacks (`sync_tab_bars`) still go in the
  rest, after the catch-up.

### Task 7b: peeks by default

Owner, 2026-11-11: "just make the new thing the default", without
waiting for the live checks. Two commits:

1. `Isolation::peeks` is on unless `GLAZEWM_ISOLATE` has the new word
   `nopeeks` (`peeks` stays accepted and means nothing more). Module
   doc, tests and `TODO.md` "Live-check switch" say so, and that
   `nopeeks` is how to compare with, and fall back to, the old paths.
   `Live` peeks and everything not moved over keep the old path as now.
2. `WindowCloseConfig`'s default `enabled` becomes `true` (the sample
   config already has it on). `ANIMATION_REVIEW.md` 16 lists what that
   exposes (a refused close re-opens the window); note it in `TODO.md`.

Nothing is deleted: task 9 still waits for seen-working code.

As built: `Isolation::default()` has `peeks: true`; `nopeeks` clears it.
Code still reads `isolation::current().peeks`, so nothing else moved.
`peeksfirst` is untouched and off. Every task above says "behind the
switch": read that as "unless `nopeeks`" from here on. No live check of
tasks 3 to 7 had been run when the default changed.

### Task 8: resize, `stretch` on peeks

Decided by the owner, 2026-11-11, after the first live run (opens,
closes, minimize, restore, turn-rounds and moves on peeks all worked;
opens and closes looked about as before): build option (b) below. With
`window_resize.style: "stretch"` and peeks on, a resize of a tiled
window or a column is drawn by a pinned peek that scales the window's
content into the animated rect; the app is resized once, at the start,
behind the cloak. `style: "fill"` (the default) keeps `ResizeSession`
unchanged, and so does `stretch` with `nopeeks`.

**Corrected 2026-11-11, after the first attempt stopped.** The
sampling rule suggested further down (sample the window's current size
each frame) is what the old path already does, and it is the cause of
the end-zoom, not its cure: old `stretch` registers its thumbnail with
no source rect, so the scale is always animated size / actual size, and
it steps by target / source on the frame the app's resized content
replaces the old. Apps take one frame to about 300 ms; with a 150 ms
fast-start curve that lands near the end, where the old content is
blown up most. No rule that fills the rect avoids the step.

Owner's choice: **wait, then stretch.** The app is resized first, behind
the cloak, with the peek held at the source rect showing the window 1:1;
the motion starts when the window reports its target size (existing
`GetWindowRect` read) or a cap runs out. From then the picture is the
new content, scaled animated / target, ending exactly 1:1. One visible
swap remains, at the start: the old layout replaced by the new one
squeezed into the old rect. Costs: the relayout starts late by the
slowest resizing window (one clock for all its windows, so they stay in
step), bounded by the cap; this is a timing change the owner asked for.
A window that never reaches its target (minimum size, fixed aspect) is
drawn as what it really is at the cap: scale by its real size, so the
hand-back does not snap.

The rest of this section is as first written; read it with the above.

What must not show, in the owner's words about today's `stretch` on the
old path: "a zoom effect when it kind of ended". So: no change of scale
in the last frames or at the hand-back. The picture's scale must follow
the animated rect and the window's real size at every frame, and at the
last frame be exactly 1:1 with the real window at its target.

The hard part, to settle by reading before writing anything: the app
resizes asynchronously, so for some frames the real window is still its
old size, then its new one, and its content may repaint a frame or more
later. A thumbnail that samples a fixed source rect then shows a cut or
padded window, or a jump in scale when the source is switched. Find what
`WorkspaceSurrogate::show_frame`, `WindowPeek::place_scaled`
(`scaled_rects`) and the old path's `stretch` (`NativeSurrogate`,
`covered_extent`, `update_thumbnail_dims`, `sync_registration`,
`pre_commit`) each sample and when, find where the old path's end-zoom
comes from, and choose a sampling rule that cannot jump: e.g. sample
the window's actual current size each frame (as `sync_registration`
polls it), so the destination scale changes smoothly whatever the app
does. Say in the commit which rule was chosen and why.

Rest as in task 6, which this extends (a resize is a move whose size
changes): same call site and qualifying rules apart from the size test,
same take-over, timing (`movement_timing` with `is_resize`), retarget
that keeps its speed, z-order, strays, landing hooks. A relayout mixes
windows that move and windows that resize: with `stretch` all are on
peeks, on one clock. Scrolling columns: a column whose width changes
follows the same rule through `upsert`.

Not in this task: `fill` on peeks (option (c) below), floating
resizes, mouse drags, `Live` peeks.

Live check: `window_resize: { style: "stretch", duration_ms: 2000 }`
and at 150; grow and shrink a tile by keyboard, open a window beside
two others, close one of three, a held resize key; Notepad with wrapped
text, a browser, Explorer, an Electron app. Look first for any change
of scale as it ends; then borders in step, no cut or padded content
mid-way, no blank frame. Compare with `nopeeks` and with `fill`.

The options as they were put:

- (a) Leave resize on `ResizeSession`. Smallest risk; two systems remain,
  but only one kind of animation uses the old one.
- (b) `window_resize.style: "stretch"` on peeks: the content scales to
  the animated rect and the app resizes on the first frame. `place_scaled`
  already draws this. How Hyprland looks; text is blurred mid-resize.
  The owner (2026-11-10): the idea "could be nice", but today's
  `stretch` on the old path "looks awful and acts weird" (weird
  zooming): "a zoom effect when it kind of ended". That fits the old
  path: the picture is scaled to the last frame, then swapped for the
  real window at its true size. So (b) is open, judged only on a
  peek-drawn build beside the old one, and its live check lists first:
  no change of scale in the last frames or at the hand-back.
- (c) Port the `fill` style (grow-only registration, gap fill, edge
  colour, mid-motion hand-off). The hard part the second version
  described; only if (b) is not acceptable.

Owner's verdicts on the first build (wait, then stretch), live,
2026-11-11: the stretch itself "looks pretty nice", with no zoom and no
snap. Rejected: the wait ("it should be a bit instant", "a bit laggy")
and the wallpaper showing for a moment at the key press, sometimes on
the rightmost window. One window alone resized well. `fill` judged "more
solid", "a bit smoother" for now, with a thin unfilled line (fixed
since, `TODO.md` entry 50).

Where that wallpaper came from, read from the code: a held peek sampled
its window at its old size while the app was resized behind the cloak. A
window that shrank was oversampled, a transparent hole, from the moment
its resize landed until a tick had read its new rect and the next had
drawn with it: one to two frames, on every shrink. And a growing window
whose content had not arrived when it moved (the cap run out, or any key
press after the first, which never waited) was drawn at animated /
target of its old size: short of its rect, its backdrop copy with it,
the wallpaper showing in the rest, at its far edge. The rightmost window
has nothing beside it to hide that.

As rebuilt (not seen running), for task 9 and a later `fill` port:

- Only with `style: "stretch"`, `window_resize.enabled` and peeks on
  (`stretches_on_peeks`); `fill` and `nopeeks` take exactly the paths
  they took before.
- No wait: `Entry::held`, `PeekView::hold` and `STRETCH_WAIT_CAP` are
  gone. The take-over's flush resizes every cloaked window of the
  relayout at once (`preposition_cloaked`), a retarget resizes it again,
  and the motion starts on the next tick, as a pure move's does.
- Drawing rule (`Stretch`, `WindowPeek::place_stretched`,
  `stretched_rects`): the picture always covers the animated rect, cut
  at it, and samples no more of the window than it is sure to have (its
  read size, capped per axis at the target's until it has been seen at
  the target's size). Scale per axis = animated / target x cover, cover
  >= 1: what it takes to reach across the rect with what is sampled.
  The ring and backdrop copies are placed at the animated rect.
  - Shrinking axis: only the target's length can be sampled safely, so
    cover is 1 from the first frame: the part that stays, enlarged by
    old / new as the motion starts, gliding to 1:1; the new content
    arrives at the same scale. One step, at the start, as large as the
    swap the first build made there.
  - Growing axis: the old content is stretched across the rect (cover =
    target / old). Once the window is seen at its target's size, the
    cover eases to 1 over `STRETCH_SETTLE` (80 ms) instead of stepping:
    the new content keeps the old one's scale, cut at the rect's edge,
    then comes to fit. The step was the old path's zoom.
  - Lands 1:1: at the target, a window of its size has cover 1 and is
    sampled whole. A landing waits for the cover to be 1 (at most 80 ms
    more, only when the content arrived in the last frames).
- Why this rule: filling the rect with all of the old content
  (animated / actual) oversamples a window the moment it shrinks, and
  steps when content arrives; animated / target alone leaves a growing
  window short of its rect, wallpaper beside it. Covering with what can
  be sampled, and easing where easing can still cover, is what is left
  with no wait.
- Weak points: a growing window whose app is slower than the animation
  lands stretched and is handed back so (the old zoom, for slow apps
  only); a window that refuses its target's size is handed back with a
  jump (the first build drew it as it was); the ring copy's thickness
  follows animated / actual, so steps as a growing window's content
  arrives (TODO "Still open" 5); the shrink's enlarged first frame.
  Sending a shrinking window its size a few frames in, once the rect is
  within a few percent of the target, would make that step as small:
  not built, it is a wait of another kind.
- The window's rect is read as its peek is made (`Stretch::at`) and by
  `carry_ring` each tick (`GetWindowRect`, existing calls), so a draw
  uses a read up to a tick old: the cap is what keeps that safe.
- Mixed relayouts: `stretches_relayout` keeps the whole relayout off
  stretch (resizes on `ResizeSession`, pure moves on peeks, as task 6)
  when any of its tiles couldn't go on a peek (`Live`, another
  stand-in, floating state change, off its monitor) or is at its new
  size already, which an interactive resize's drag end leaves it at.
  Columns: `sync_scroll_view` does the same per strip (`can_stretch`); a
  new column still widens from nothing (`opening_rect`), never scaled.
- A retarget glides on (`retargeted_motion`); a stretch headed for
  another size keeps the scale it was drawn at (`Stretch::retarget`).
- Task 9: `ResizeSession`'s `stretch` (`SessionOptions::stretch`,
  `NativeSurrogate`'s `stretch` field, `stretched_destination`) is still
  reached with `nopeeks` and by mixed relayouts; nothing is unreachable
  yet.

### Task 9: remove what is no longer reached

Only for kinds whose live check passed and whose default was flipped (a
one-line change per kind, its own commit, after the owner says so).
Then delete the old branch for that kind: `start_open_animation`,
`start_away_animation`, `slide_in_monitor_rects`, `ResizeSession::zoom`
and `update_zoom_fade`, `away_windows`, as each loses its last caller.
`grep` for callers before deleting; one commit per kind.

### Task 10: `fill` on peeks, for mouse drags first

Asked for by the owner, 2026-11-12. Reviewed and rewritten the same day
against the code at `f0465543` (read only, nothing run): 10a is
answered below, and 10b to 10f are cut into steps of one commit each.

With `general.grab_and_move` (the AltSnap replacement,
`docs/grab-and-move.md`), a mouse resize moves the real windows live,
and their border rings follow. Even with the rings placed in the same
step as the window (`c70a4555`), a window shows a frame or more off its
ring, and a growing neighbour or a shrinking window covers a ring for a
frame: each app applies its size on its own thread, at its own time.
Hyprland avoids this by drawing window and border itself in one frame
from the window's real buffer; on Windows only DWM draws a real window.
The nearest equivalent is to draw the window and its ring ourselves,
from one thread, in one burst of thumbnail updates: peeks.

Goal: during a grab resize, the windows it changes are drawn by pinned
peeks placed at the layout's rects, so edges, gaps and rings agree on
(nearly, see "Limits") every frame. The real windows are still resized
every step, cloaked, and their thumbnails are live, so content keeps
updating; where an app's window has not reached its size, the uncovered
strip is the window's edge colour (`fill`, which the owner prefers to
`stretch`), and where it is too large it is cut at the tile. Then the
same drawing replaces `ResizeSession` for keyboard `fill` resizes, which
lets the old engine go.

Behind `grab_and_move.draw: "live" | "peeks"` (default `live`, today's
behaviour) until the owner has seen it. A config option, not a
`GLAZEWM_ISOLATE` word, so the owner can switch by reloading the config
mid-session.

Windows API for the whole task: **none new.** Every step below uses
calls the WM already makes (`DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail`,
`SetWindowPos`, `DeferWindowPos`, `DwmSetWindowAttribute` with
`DWMWA_CLOAK`, `DwmFlush`, `GetWindowRect`, `CreateWindowExW`,
`SetLayeredWindowAttributes`, and the `Windows.UI.Composition` calls
`SurrogateFill` makes). If a step turns out to need another, rule 9
holds: stop, say which and why, its own commit. What does change is how
often existing calls run (other apps' windows cloaked on every drag);
`CLAUDE.md` lists cloaking among what keeps the WM near Defender's
threshold. That verdict is on the binary, so no change is expected, but
it is not known: the first laptop build with 10c is the test.

#### Limits, stated before anything is built

- **"One DWM update" is not a guarantee.** DWM has no transaction for
  thumbnail updates (this plan, lever B and task 7): a frame is a run
  of `DwmUpdateThumbnailProperties` calls, each applied at the next
  composition. A composition that falls inside the run shows it half
  applied. What peeks buy is that the run is ours, on one thread, well
  under a millisecond, instead of one app thread per window taking
  milliseconds to hundreds of them. Edges and rings will be exact on
  nearly every frame, not on every frame. Steps 10b and 10c keep the
  run short and in an order whose halves look least wrong.
- **The fill covers a late window rect, not late pixels.** The strip is
  sized from the window's rect (`GetWindowRect`). An app whose window
  has its new size but has not painted it shows whatever DWM holds for
  it, as it does in a drag by its own border today.
- **Content is live only if the app paints while cloaked.** Resting
  peeks of the scroll view rely on that already, but nothing in the
  repo says every app does (a browser may stop painting a window it
  takes for hidden; not verified). 10a ends with a check of this that
  needs no new code. If it fails for the owner's apps, stop: the task
  is not worth building.
- **A take-over and a hand-back on every drag.** One blocked
  composition (`DwmFlush`, up to a frame) on the first step that
  changes anything, and the hand-over at release. A window's DWM shadow
  is not in its thumbnail, so it goes as the drag starts and comes back
  as it ends, as it does for a move on a peek.
- **Not covered at first**: floating windows, stacks (their tab bar is
  a window of its own, placed by `sync_tab_bars`), `Live` peeks
  (rounded corners, parallax != 1), windows DWM gives no thumbnail of.
  They keep the live path, which therefore stays in the code for good.

#### 10a. Design check: answered 2026-11-12

Read from the code, nothing run. "Inferred" marks what the code implies
but no comment or test states.

1. **The fill: a DWM thumbnail of a small window of ours, not a
   composition visual.**
   - A visual on the peek fails twice. `SurrogateFill`'s doc
     (`composition.rs`): it is drawn "underneath the thumbnail (DWM
     draws thumbnails above a non-topmost `DesktopWindowTarget`)". A
     pinned peek's thumbnails include its backdrop crops, which are
     opaque, so the fill would sit under them, unseen. And today's fill
     is in step only because it is sized relative to its surrogate
     window ("a per-frame resize of the surrogate needs no composition
     write at all"), which moves by `SetWindowPos`. A pinned peek never
     moves, so the strip would need `SetOffset`/`SetSize` every frame,
     and those reach DWM with the compositor's own commit:
     `overlay_copies.rs` says a window or visual tree of its own
     reaches DWM "on schedules of their own", which is why the ring and
     backdrop are copies at all.
   - A thumbnail goes in the same run as the window's, and stacks by
     registration order (`Thumbnail::register`: "Thumbnails stack in
     the order they are registered"), so it can sit above the backdrop
     crops and below the window. Its source is built from what exists:
     an `OverlayWindow` hidden by its alpha, "which thumbnails of it
     leave out" (as `BackdropSource::create`), with a `SurrogateFill`
     rooted on it and `set_covered((0, 0))`, which paints the whole
     window. A few pixels square: one colour stretches without harm.
     Shared per colour, as `BackdropSource` shares per style
     (`SOURCES`). The colour is written through composition once, when
     it is sampled, never per frame.
   - Cost: one small hidden window per colour in use, and two
     thumbnails per peek (the uncovered part is an L: a strip right of
     the content and one under it, as `SurrogateFill`'s two sprites).
     `FILL_UNDERLAP_PX` and `fade_fraction` (`native_surrogate.rs`)
     carry over: the strips start 2 px under the content's trailing
     edges, and are drawn at the thumbnail's fade, not its alpha.
   - The colour is `AnimationManager::cached_edge_color`, sampled from
     the screen (`sample_edge_color`: `GetDC(0)` and a blit), so only
     while the window is uncloaked and never during a drag. A window
     with no cached colour gets no strip: its backdrop copy, or the
     desktop, shows there. `queue_first_edge_sample` only queues one
     with `window_resize` on and not `stretch`: 10c widens that test.
   - Not chosen, noted for later: a second thumbnail of the window
     itself, a column a few pixels inside its edge stretched over the
     strip. No source window and no screen read, but it smears whatever
     is at the edge (a scroll bar), and is not the `fill` the owner
     has seen.
2. **The ring in slices: supported as the code stands; 8 thumbnails per
   ring instead of 1.**
   - `Thumbnail::show(source, dest)` and `Thumbnail::place` take a
     source rect per thumbnail (`DWM_TNP_RECTSOURCE`), so a slice is a
     `Thumbnail` of the ring's window with its own source. Two
     thumbnails of one source on one destination already coexist:
     `OverlayCopies::raise_ring` registers the new one "before the old
     one goes".
   - Slices: four corners drawn 1:1, four edges stretched along their
     length only. A corner slice reaches from the ring window's outer
     edge to a little inside the ring's inner edge:
     `BorderOverlayParams::outset()` (width plus glow reach) plus the
     rim hairline. Rounded rings never get here: a rounded backdrop
     makes the peek `Live` (`is_copyable`, `is_live`), which carries no
     ring copy (`WindowPeek::carry_ring`). Inferred: a square ring with
     a rounded-corner *border* and no backdrop is possible in config;
     the corner slice must then also take in `corner_radius`.
   - The ring's stroke is a vertical gradient over the top
     `HIGHLIGHT_FALLOFF` (0.35) of its height (`BorderVisual`). The
     side slices stretch it with the height, as the real ring does at
     the new size; only the part inside the corner slices keeps its
     old length. Inferred invisible; look for it in 10b's check.
   - The slices need a source that does not change during the drag.
     Today `carry_ring` (`peek_view.rs`) parks the ring at the real
     window's own frame on every tick (`overlay.park(&ring_frame, ..)`),
     which resizes the ring's window and its visual by their own
     routes. For a peek whose window is being resized the ring stays
     parked where it was taken over, and only the slices' destinations
     change. `Handover`'s steps and their order are untouched: the ring
     is still shown again at the landed frame under the copy
     (`Handover::Revealed`).
   - Cost: up to 8 updates per ring per frame, but `Thumbnail::set`
     sends only what changed; dragging one edge changes five slices.
     With the window, two fill strips, a backdrop crop and a vignette,
     about ten calls per changed window per frame. Not measured:
     `perf::Stage::PeekFrame` is where it shows.
   - A whole ring is the one-slice case of the same type, so
     `WorkspaceSurrogate` and peeks that never change size keep one
     thumbnail and one code path draws both.
3. **The backdrop covers any frame; one flaw in how the frame is
   mapped.** `CropCopy::place` cuts `window_frame ∩ clip` out of a
   source spanning the monitor, so any frame on the monitor is covered.
   But `BackdropCopy::place_current` gets that frame from
   `map_rect(frame_rest, window_rest, placement.frame)`, which scales
   the invisible-border inset with the window: at 1.5 times the width a
   7 px inset becomes 10 px, and the backdrop ends 3 px short of the
   frame on each side. During a drag `carry_ring` re-reads the rest
   rects every tick (`WindowPeek::follow_window`), so the error stays
   under a pixel; for a keyboard resize (10e) the window may stay at
   its old size for the whole motion, and a see-through window would
   show it. 10b fixes it by placing the copies at the frame minus the
   inset, not scaled.
4. **Sampling.** Rule, per axis: source length = the smallest of (the
   window's size as last read, the frame, every size sent to the window
   and not yet read back); destination = that length from the frame's
   top-left, 1:1; fill = the rest of the frame.
   - The read: `carry_ring` (`peek_view.rs`) already calls
     `frame_with_shadows()` (`GetWindowRect`) for every entry on every
     tick and stores it for `Stretch` (`stretch.actual`). Same read, no
     new call. It is up to a tick old, hence the third term.
   - "Sent and not yet read back": for a drag, the rect the window's
     placer thread holds or is applying (`WindowPlacer`'s `SlotState`:
     `rect`, `is_placing`), exposed by a read-only method; for keyboard
     resizes, the target, as `Stretch::reached` does today.
   - Why the third term: a drag reverses. Window at 1000 px, 950 sent,
     then 980 sent before 950 landed. A tick reads 1000 and has a frame
     of 980; 950 lands before DWM composes; 980 is oversampled. An
     oversampled source "renders as a transparent hole"
     (`NativeSurrogate`'s doc; whether all of it or only the excess is
     not stated anywhere: inferred the excess).
5. **Grab integration: take-over through the redraw, by the path a
   move on a peek takes; no new role.**
   - Where: a step is `grab::step` → `apply_resize` →
     `place_moved_windows`, which for each window whose target changed
     either hands it to the placer and places its overlays
     (`defer_window_frame`), or, if `is_placeable` says no, queues it
     for `platform_sync`'s redraw. The end is `grab::finish`
     (`placer.finish()`, then a redraw of every window the resize can
     have moved); Escape is `finish` with `GrabEnd::Cancelled`, which
     first steps back to a delta of 0.
   - Role: `Role::Moving { ends_shown: true }` with a hold flag on the
     `Entry`, not a new `Role`. Every existing test then holds
     unchanged: `owns_window`, `is_moving` (frozen in the redraw),
     `follows_tile`, `retarget_tile`, `track_overlays`, strays, and the
     landing in `PeekView::tick`, which only gains "and not held". No
     `Role` transition is added, so rule 5 is not touched.
   - Take-over, in the scroll view's order (`sync_scroll_view`,
     `flush_overlays`) as task 6 already runs it for a tile: on the
     first step that changes a window not yet on a peek, the step
     queues it for redraw instead of placing it. The redraw puts up a
     warmed-up peek at the window's rect (`PeekView::move_tile` →
     `new_entry` → `WindowPeek::warm_up`), lists it in `peek_moves`,
     and from there nothing is new: `cover_moved_overlays`, then
     `commit_pending_cloaks` (one `DwmFlush` for the pass,
     `PeekView::take_over`, cloak, `preposition_cloaked` at the new
     rect), then `hand_over_moved_overlays`. From the next step on the
     window goes to the placer, cloaked, and its peek is retargeted.
   - The redraw must do this ahead of its animation gate:
     `apply_resize` calls `suppress_animations()`, and
     `should_use_animations` is then false unless the window is already
     frozen. Same place and shape as `enters_on_peek`
     (`is_frozen_by_ws_animation |= peeks.is_moving(..)`).
   - `is_placeable` and `is_grabbable` both refuse a window a peek
     owns. `is_placeable` must let through one this grab holds;
     `is_grabbable` stays as it is (no new grab starts on a moving
     peek).
   - Latency: the first changing step is shown one composition late
     (the flush) plus a tick. Every later step is drawn on the next
     frame tick, not on the mouse event: at most a frame behind the
     cursor, the same for every window.
   - Drawn on the tick, not in the step. A step only retargets
     (`retarget_tile`, duration 0: it jumps, as `follow_duration` does
     for a floating window). `PeekView::tick` draws, on vblank ticks,
     so the run of updates sits where task 7 put it.
   - Not raised: `raise_moving` puts a moving peek on top, over
     floating windows it passes. A held peek stays where it went up,
     directly below its window, for the whole drag.
   - A window closing or hiding mid-drag: `drop_strays` → `is_stray`
     takes a non-column `Moving` entry no longer shown, and queues its
     window for redraw. The grabbed window going ends the grab
     (`sync_listener`). A workspace switch does not end it today unless
     focus moves to another window (`note_focused`): 10c adds "its
     workspace is no longer displayed" to `sync_listener`'s `is_lost`.
6. **End of the drag: the existing hand-back, with one wait; no
   fade-off first.** `finish` clears the hold. The next tick finds the
   entry landed (duration 0) and takes the normal road:
   `begin_hand_back`, `Landed::shown`, the redraw's `Apply` branch
   (`prepare_hand_back`, a synchronous `SetWindowPos` because
   `peeks.owns_window`, guarded by `allows_sync_reposition` for a hung
   app, uncloak, `release`), `Role::Releasing`, `hand_back`. The one
   addition: a held entry lands only once its window is read at its
   target's size, or 150 ms after the release, so the swap is not from
   a filled strip to a window still short. Same hook as
   `Stretch::is_settled`. The fade-off (end of this task) stays an A/B
   for later: it is a second kind of hand-back, and this one is the one
   tuned.

**Owner's check before any code (five minutes, today's build).** In a
scrolling layout, scroll until a browser column playing a video, then
an Electron app with something moving, is cut by the monitor's edge
and at rest. That column is a resting peek: a thumbnail of a cloaked
window. Does it keep playing, at full rate, for a minute? If it
freezes or stutters, a drag on peeks will too, for that app.

#### 10b. `fill` drawing on `WindowPeek`

Three commits. No grab code yet.

**10b-1. Ring copy in slices.** In `overlay_copies.rs`: the ring copy
holds a list of slices, each a `Thumbnail` of the ring's window with a
fixed source and a rule for its destination (corner: 1:1 at its corner
of the frame; edge: stretched between two corners). `carry_ring` takes
whether to slice; whole is one slice. `raise_ring`, `set_ring_opacity`,
`cover_ring`, `hide_ring` loop over the slices; `raise_ring` still
registers the new ones before dropping the old. A pure function gives
`(source, dest)` per slice from the ring's rest, the window's rest, the
frame, the clip and the origin, cut at the clip as `copy_rects` is;
unit tests: at rest the slices tile the whole ring exactly; at another
size corners keep their size and the edges meet them; cut at the clip.
Sliced for peeks that stretch (`Entry::stretch`); `carry_ring`
(`peek_view.rs`) then parks the ring once and leaves it parked at that
rect while its window's size changes (10a.2).
- Reuses: `Thumbnail`, `placed_rects`, `OverlayCopy`.
- Deletes: nothing yet. Closes `TODO.md` "Still open" 5 (the ring
  copy's thickness stepping under `stretch`).
- Rule 5: `Handover` steps and their order unchanged; only what a ring
  copy is made of, and that a resizing window's ring is not re-parked.
  Say so in the commit.
- Windows API: `DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`,
  `DwmUnregisterThumbnail`. None new.
- Live check: `window_resize: { style: "stretch", duration_ms: 2000 }`,
  a border with a glow. Grow and shrink a tile by keyboard: the ring
  keeps its thickness and its corners their shape all the way, and
  nothing changes at the hand-back. Then a pure move, a workspace
  switch and a scroll: as before.

**10b-2. The fill's source and strips.** A `FillSource` beside
`BackdropSource` (`backdrop_source.rs`): an `OverlayWindow` of
`OverlayKind::BackdropSource` (no new window class), alpha 0, a few
pixels square at the bottom of the z-order, a `SurrogateFill` on it
painting it whole; `acquire(colour)` shares one per colour and keeps a
couple idle, as `BackdropSource::acquire`. In `OverlayCopies`: two
strip thumbnails of it, registered after the backdrop crops and before
the window's thumbnail is put back on top (`carry_backdrop`'s order
note and `NativeSurrogate::raise_thumbnail`), placed by a pure function
from the frame, the content's covered extent, the clip and the origin
(`fill_start`'s underlap; nothing when the content covers the frame).
Unit tests on that function.
- Reuses: `OverlayWindow`, `SurrogateFill`, `Thumbnail`, `fill_start`,
  `FILL_UNDERLAP_PX`, `fade_fraction`.
- Deletes: nothing yet; in 10f `SurrogateFill`'s two-sprite layout and
  `NativeSurrogate`'s `fill`, `covered`, `set_fill_color`,
  `set_covered` go, leaving `SurrogateFill` one sprite.
- Windows API: `CreateWindowExW`, `SetLayeredWindowAttributes`,
  `SetWindowPos` (through `OverlayWindow`), `CreateDesktopWindowTarget`
  and a colour brush (through `SurrogateFill`), the thumbnail calls.
  None new.
- Live check: with 10b-3.

Built 2026-11-12, not checked live: 10b-1 in `0f19042`; 10b-2 and 10b-3
in one commit, as 10b-2 alone has no caller and fails `-D warnings`. Three
departures from the text above, both in the commit message:

- The fill strips stack above the window's own thumbnail, not below:
  putting them below means registering the window's thumbnail anew,
  and `NativeSurrogate::raise_thumbnail` unregisters it first, which
  can leave a frame with no window. Above it they cover only where the
  content does not reach, plus the 2 px underlap, which is the hidden
  border's see-through rows anyway. The ring is raised over them.
- The fill is shown with the backdrop copy's step in
  `WindowPeek::apply_backdrop` (drawn only while the window is), so
  `WindowPeek::draw`'s order is untouched.
- `fillpeeks` also queues a first edge sample for windows as they open
  or are shown (`queue_first_edge_sample`), as `stretch` never samples.

**10b-3. `WindowPeek::place_filled` and the sampling rule.**
`place_filled(sampled, rect, clip, opacity, batch)`: content 1:1 from
the frame's top-left, `sampled` long per axis (10a.4, computed by the
caller), cut at the frame and the clip; strips over the rest; ring
slices and backdrop at the frame, the backdrop at the frame minus the
inset instead of `map_rect` (10a.3). A pure `filled_rects` beside
`stretched_rects`, with tests: at its size 1:1 and no strip; short of
the frame, a strip of the difference less the underlap; larger than the
frame, cut and never sampled past the frame; a size in flight smaller
than both caps the source; cut at the clip. The order of the calls in a
frame is fixed and said in a comment: backdrop, strips, content, ring
slices, so a composition mid-run shows an edge a frame late rather than
a hole. To see it: a `GLAZEWM_ISOLATE` word, `fillpeeks`, off by
default, that makes task 8's stretch path draw with `place_filled`
(`Entry::draw_stretched`), the strip in the cached edge colour. Listed
in `TODO.md` "Live-check switch"; 10e turns it into the real path.
- Reuses: `placed_rects`, `to_logical`, `ScaledRects`, `Stretch`'s
  `actual` read, `cached_edge_color`.
- Deletes: nothing yet; in 10f `place_stretched`, `stretched_rects`,
  `covering` and `STRETCH_SETTLE` go if the owner drops `stretch`
  (their call: it is the only other style).
- Rule 5: `WindowPeek::draw`, `take_over`, `prepare_hand_back`,
  `hand_back` untouched; `place_filled` is a sibling of
  `place_stretched`.
- Windows API: `DwmUpdateThumbnailProperties`. None new.
- Live check: `GLAZEWM_ISOLATE=fillpeeks`, `window_resize: { style:
  "stretch", duration_ms: 2000 }`, two tiles, a window resized by
  keyboard once before (so its colour is cached). Shrink it: its app
  shrinks at once behind the cloak, so for two seconds the strip fills
  the rest of the tile. The strip's far edge, the ring and the gap move
  as one; its colour matches the window's edge; no line between content
  and strip; a see-through window's strip is not darker than the
  window. Grow it: content cut at the tile, no strip once the app has
  grown, no hole at any point.

#### 10c. Grab resizes on peeks

Owner, 2026-11-12: keep `fill`, and keep `stretch` for now, so 10f
leaves `place_stretched`, `stretched_rects`, `covering` and
`STRETCH_SETTLE` in place.

Built 2026-11-12, not checked live: 10c-1 to 10c-4 in one commit, as
none of them is reachable without the next. As written, plus:
`can_hold` (`peek_view.rs`) is `stretches_relayout`'s per-tile test
minus its animation checks, and also leaves out columns until 10c-5;
`WindowPlacer::unlanded` keeps the rect being applied
(`SlotState::applying`, which replaces `is_placing`); a held peek's
redraw retargets it with duration 0. No unit test reaches a held peek
(`WindowPeek` needs a desktop): `Hold`'s landing rule, `unlanded`, and
`holds_on_peek` refusing mock windows and `draw: "live"` are tested.

10c-5 built the same day, not checked live, as written. Steps place a
held column's real window at `real_window_rect` (on its monitor, cloaked)
through the placer, and retarget its peek (`retarget_tile` now also takes
a held column); a column not yet held is refused by the placer and held
by `sync_scroll_view` (`upsert`'s `held`: duration 0, no transit,
filled), whose own flush takes over from it.

Behind `draw: "peeks"`. Tiled windows of split and dwindle layouts
first, scrolling columns last. Five commits.

**10c-1. The option and the hold.** `GrabAndMoveConfig::draw`
(`"live"` default, `"peeks"`), docs in `docs/grab-and-move.md` and the
sample config. `Entry::held`, set and cleared by
`PeekView::hold(id, bool)`; `tick` lands a `Moving` entry only if not
held, and a just-released one only once its window is read at its
target's size or 150 ms on (10a.6). Unit tests beside
`landing_needs_both_tracks`. Nothing sets the flag yet.
- Reuses: `Role::Moving`, `has_landed`, the `Stretch::is_settled` hook.
- Windows API: none.
- Live check: none possible (no caller); `draw: "peeks"` must change
  nothing yet. Checked with 10c-2.

**10c-2. Take-over on the first step.** `grab::holds_on_peek(window,
state, config)`: `draw` is `peeks`, a resize grab is under way on the
window's workspace, and the window qualifies as `stretches_relayout`
asks of each tile (tiling, shown, on its monitor, not `Live`, no other
stand-in), and is not in a stack. `is_placeable` refuses such a window
until a peek holds it, so `place_moved_windows` queues it for redraw.
In `redraw_containers`, beside `enters_on_peek`: such a window goes up
by `move_tile` (duration 0, sliced ring, `fill` drawing, held), is
frozen from then on and pushed to `peek_moves`; it is not passed to
`raise_moving`. The rest is the existing take-over (10a.5).
`queue_first_edge_sample` also queues when `draw` is `peeks`.
- Reuses: `move_tile`, `new_entry`, `warm_up`, `cover_moved_overlays`,
  `commit_pending_cloaks`, `hand_over_moved_overlays`,
  `preposition_cloaked`.
- Rule 5: none changed. `commit_pending_cloaks`' single flush and
  `FrozenStep` are used as they are.
- Windows API: `DwmFlush`, `DwmSetWindowAttribute(DWMWA_CLOAK)`,
  `SetWindowPos` (async, existing flags), thumbnail calls. None new.
- Live check: after 10c-3 (a held window that is never stepped or
  released is not a state to test).

**10c-3. Steps.** `WindowPlacer::unlanded(window) -> Option<Rect>`:
the smaller, per axis, of the rect waiting and the rect being applied.
In `place_moved_windows`, a held window is placed by the placer as now
(same flags: no frame change, no discarded bits), its peek retargeted
(`retarget_tile`, duration 0), and `defer_window_frame` skipped: its
ring and backdrop are copies. `carry_ring`'s per-tick read feeds
`place_filled` with 10a.4's rule.
- Reuses: `WindowPlacer`, `retarget_tile`, `carry_ring`'s read,
  `place_filled`.
- Windows API: `SetWindowPos` on the placer's threads (existing, same
  flags), `GetWindowRect`, thumbnail calls. None new. No new thread:
  the placer's per-window threads exist.
- Live check: with 10c-4.

**10c-4. The end, Escape, strays.** `finish` clears every hold of the
grab before its redraws; a cancelled grab steps back first, as now.
`sync_listener` also ends a grab whose workspace is no longer
displayed. A unit test in `grab.rs` beside
`the_end_of_a_resize_syncs_every_window`: after `finish` no entry is
held.
- Reuses: `finish`, `end_early`, `drop_strays`, the whole hand-back.
- Rule 5: none changed.
- Windows API: none new.
- **Live check, round 1** (split layout, `draw: "peeks"`, a border and
  a gap wide enough to see):
  - Two tiles, then three with a nested split. Drag an edge slowly,
    then fast, then back and forth quickly. Gap and rings solid: no
    window over its own or its neighbour's ring, at any speed.
  - A slow app (browser, Electron, Outlook) beside a quick one
    (Notepad). Content keeps updating during the drag; edge colour
    where the slow one lags when growing; never a hole to the desktop.
  - The start: the first movement shows within a frame or two, with no
    blink, no blank window, no ring drawn twice. The end: release, and
    nothing moves, blinks or changes size; the shadow's return is the
    only change.
  - Press and release without moving: nothing is cloaked (no blink at
    all).
  - Escape mid-drag; the grabbed window closed mid-drag (by the WM's
    close keybinding); a neighbour closed mid-drag; a workspace switch
    mid-drag; lock the screen (Win+L) mid-drag and come back. After
    each: every window visible, at its tile, clickable, and no "stuck"
    report in the WM's log (`report_stuck_windows`).
  - Compare with `draw: "live"` on the same windows.

**10c-5. Scrolling columns.** `sync_scroll_view` asks
`grab::holds_on_peek` for the columns of the grab's workspace and
passes the hold to `upsert`, which gives such a column `Role::Moving`
with duration 0, held, sliced, `fill`; a resting peek (`Role::Peek`,
already cloaked) becomes held with no take-over. Columns leaving the
view keep `ends_shown: false` and are dropped as now.
- Reuses: `sync_scroll_view`, `upsert`, `column_moves`, `take_departed`.
- Rule 5: none changed.
- Windows API: none new.
- **Live check, round 2** (scrolling layout): drag a column's right
  edge so its neighbours slide; past the screen's edge so the view
  scrolls (`docs/grab-and-move.md`, "Past the screen's edge"); a row
  boundary inside a column; a column partly off screen at the start.
  Same things to look for as round 1, plus: columns cut at the
  monitor's edge stay cut, on two monitors nothing shows on the other.

#### 10d. Grab moves on peeks (optional, owner's call after 10c)

The picture translated, the real window moved cloaked by the mover
thread. One commit. A move never changes size, so no fill and no
slices: `place` as a tile move, held. Worth it only if the owner sees
the ring trail a moved window today (`follow_move` already places it
ahead of the window's own report).
- Windows API: none new.
- Live check: drag a floating and a tiled window by the modifier,
  across monitors of different DPI (the app rescales as it crosses: the
  one case where a move changes size, which must fall back to live).

#### 10e. Keyboard `fill` resizes on peeks

More than "the same drawing": the old engine also decides *when* the
app is resized, and waits for it at the end. Three commits.

**10e-1. When the app is resized.** `ResizeSession::needs_preposition`
today: a window growing on both axes is resized at the start (content
arrives under the cut: a curtain); one that shrinks on either axis
stays at its old size until `pre_commit` (a wipe, no strip), with the
strip on any grown axis. On peeks: `preposition_cloaked` for a held or
moving peek with `fill` follows the same test; the size a shrinking
window is owed is sent as it lands, which the redraw's `Apply` branch
already does synchronously for a window a peek owns. `fillpeeks` then
means: `fill` relayouts go on peeks (`stretches_relayout`'s test,
renamed for both styles).
- Reuses: `move_on_peek`, `stretches_relayout`, `retarget_tile`,
  `place_filled`, `follow_retargeted_column`.
- Deletes: nothing yet.
- Windows API: none new.
- Live check: `GLAZEWM_ISOLATE=fillpeeks`, `style: "fill"`, 2000 ms and
  150 ms: grow, shrink, a mixed resize (one axis each way), a held
  resize key, open a window beside two, close one of three. Against
  the same without the word.

**10e-2. The landing wait.** `pre_commit` waits up to
`COMMIT_CONFIRM_MAX_WAIT` (500 ms) for the window to report its size
before the stand-in goes. On peeks: 10c-1's wait, with that cap for
keyboard resizes. Mid-motion retargets: `retargeted_motion`, as
`stretch`.
- Live check: as 10e-1, with a slow app; look at the last frames and
  the hand-back only.

Built 2026-11-12, not checked live: 10e-1 and 10e-2 in one commit, as
the landing wait is the same `Hold` change. How it went in:

- `stretch: bool` through `move_tile`, `upsert`, `retarget_tile` and
  `move_on_peek` became `Option<PeekResize>` (`Stretch` or `Fill`), and
  `stretches_relayout`/`stretches_on_peeks` became `resizes_relayout`/
  `resizes_on_peeks`. A `Stretch` notes `filled`, and `deferred` for a
  filled window that shrinks on either axis (`resizes_at_start`):
  `preposition_cloaked` and `follow_retargeted_column` skip it, so it is
  sent its size only by the hand-back's `Apply`. A grab's hold clears
  `deferred`.
- A filled window sent its size waits up to `FILL_LANDING_WAIT` (500 ms)
  once its motion is done, through `Hold::Released`, which now carries
  its wait; a deferred one doesn't, as nothing was sent before it lands.
  A new motion resets the wait unless a grab holds it.
- As a `fill` session does, the edge colour is sampled again once a
  filled peek lands (`Landed::filled`).
- `fillpeeks` no longer draws `stretch` filled; entry 67 is rewritten
  for `fill`.

**10e-3. Default.** After the owner's A/B and say-so: `fill` on peeks
unless `nopeeks`; `fillpeeks` goes. Its own one-line commit (rule 4).

#### 10f. Delete the old engine

Only once 10e-3 has been lived with, and only kind by kind (task 9's
rule). `ResizeSession` is still reached, after 10e, by more than this
task first listed. `grep -n "SessionOptions {" manager.rs` finds its
three makers: `start_animation_if_needed`, `start_open_animation`,
`start_away_animation`. What reaches them then:

| Still on `ResizeSession` | Way out |
|---|---|
| Opens, closes, minimizes with `nopeeks` | Task 9: drop `nopeeks` |
| A WM fullscreen toggle (`Motion::Fullscreen`) | Port (a resize on a peek) or unanimated: owner |
| Floating ↔ tiling, floating moves by command | Port or unanimated: owner |
| A tile moving to another monitor (`move_on_peek` refuses) | Two peeks, or unanimated: owner |
| `Live` peeks: rounded backdrop, parallax != 1 | Unanimated, or keep the engine: owner |
| Windows with no thumbnail (elevated) | Already snap (`move_tile`) |
| Mixed relayouts (`stretches_relayout` false) | Gone once every row above is |

`Live` row: moves and resizes built behind `morepeeks`, not checked live
(`TODO.md` 73). A live peek moves and resizes tiles and columns as a
pinned one does: `place_stretched` and `place_filled` move its own
window over the frame's visible part (`frame_origin`), and its real
backdrop and ring follow as for a column (`track_overlays`); a filled
one has no strip, its backdrop shows there. Raised, its overlays are
restacked behind it (`restack_live_overlays`); landed, they go behind
the window (`overlay_anchor`). Opens, closes, minimizes and grabs stay
on the engine: the real overlays can't fade or zoom with a transit.

Owner's decision, 2026-11-12: port every row to peeks (fullscreen
toggle, floating <-> tiling and floating moves by command, a tile moving
to another monitor, `Live` peeks), so the engine can go later; but keep
`ResizeSession` in the code for now. Each port is its own step behind
`nopeeks` as the way back, with its own live check, and the engine's
deletion (steps 1 to 4 below) waits for the owner's say-so.

Built 2026-11-12, not checked live (`TODO.md` 71): floating <-> tiling,
behind `morepeeks`. A state change between floating and tiling
(`changes_state_on_peek`: not from fullscreen or minimized, not while
the OS drags it) takes task 6's path (`move_on_peek`), drawn in
`window_resize`'s style (`state_change_resize`), and counts in
`resizes_relayout`. Its peek goes into the band its window lands in
(`state_change_peek_raise`, through 6b's `peek_raises`): a floating one
as one opening, a tile to the top of the other windows, its cloaked
window first out of the always-on-top band. Floating moves by command
are not in it: the old path never animated them (no state change,
`should_use_animations`), so they are not on `ResizeSession`, and
animating them needs a way to tell a command's move from an app's or a
drag's: the owner's call.

Fullscreen toggle row: built behind `morepeeks`, not checked live
(`TODO.md` 70). A tile toggled in or out of fullscreen by the WM
(`Motion::Fullscreen`) goes on a pinned peek through `move_tile`, on
`window_fullscreen`'s timing and in its own `style` (`stretch` by
default), as the old path draws it; taken over from as a move on a peek
(`peek_moves`). A peek shown on top at either end plays in the
always-on-top band (`fullscreen_peek_raise`); its ring copy is left off
as `border_hidden_for_fullscreen` says. Floating windows and a window
driving itself fullscreen keep the old path. Windows API: none new.

Built 2026-11-12, not checked live (`TODO.md` 72): the monitor row,
behind `morepeeks`. A tile moving onto another monitor is drawn by two
pinned peeks (`Trail`): its own on the target monitor, which takes over
and hands back as before, and a trail on the monitor it comes from, which
only draws and goes as it lands. Not one peek spanning both: a backdrop
copy is cut from one monitor's source, and a union could reach a third.
Left on the engine: stacks, and a tile still landing from an earlier
move. A move retargeted onto a monitor neither peek is on is still cut at
its first monitor's edge, as without the word.

So 10f starts with a decision per row, recorded here, then:

1. Move out of `resize_session.rs` what others use: `compute_border_inset`
   (`window_peek.rs`, `workspace_surrogate.rs`, `companion_cover.rs`),
   the edge colour sampler
   (`sample_edge_color_async`, `sample_edge_color`, `dominant_color`,
   `EdgeColorCache`). Move only.
2. One commit per row: its last caller gone, then the branch.
3. `ResizeSession` and `SessionOptions`; `AnimationManager`'s
   `resize_sessions` and what only it feeds (`FrozenStep::Adopt`,
   `was_pre_committed_at`, `reveal_session`: check each with `grep`);
   `NativeSurrogate`'s session-only parts (`stretch`, `update`,
   `update_thumbnail_dims`, `reregister_thumbnail`, `show_clipped` if
   `Live` goes, the fill fields). `NativeSurrogate` itself stays: a
   peek and a `WorkspaceSurrogate` are built on it.
4. `nopeeks`, `fillpeeks`, and `TODO.md`'s entries for them.

`FrozenStep` is rule 5 code: step 3 removes a variant nothing reaches,
and must say so. If any row is kept on the engine, the engine stays and
the end state is two systems with one of them rarely used: say that
plainly rather than half-delete it.

- Windows API: none new; calls only go.
- Live check after each commit: the kind it touched, once, and one
  drag, one keyboard resize, one open and close.

#### Size of the task

About 12 commits to the end of 10e (3 + 5 + 3, and the default), one
more for 10d if wanted, and 5 to 8 for 10f depending on the table.
Owner's rounds: the pre-check, 10b (one sitting), 10c twice, 10e
twice, 10f once per row; six at best, and task 8 took three builds for
one round, so plan for about ten.

#### Risks

- The pre-check failing (apps that stop painting while cloaked): ends
  the task.
- A composition inside the run of updates (see "Limits"). If round 1
  shows it, turn on `peeksfirst` (task 7, built, unmeasured) before
  changing anything else.
- The take-over: a frame of latency, and the first drag on a window
  whose colour was never sampled has no strip.
- The WM thread now ticks every frame for the whole drag
  (`PeekView::is_active`), also while the mouse is still.
- Eight slices per ring: more DWM calls per frame; unmeasured.
- Rule 5's hand-over reused for a window whose size changed while it
  was held. Tasks 6 and 8 did the same for moves and `stretch`; neither
  has passed a live check yet, and this task stands on both.
- A crash mid-drag leaves windows cloaked until `wm-watcher` cleans up,
  as for any animation, but a drag lasts longer than one.
- The live path stays for floating windows, stacks and `Live` peeks, so
  the mouse feature ends with two ways to draw, not one.

The fade-off idea (2026-11-12, not built): at a hand-back, uncloak the
real window under its peek and fade the peek out over ~60 ms, as the
scene spike's `LIFT` did, so the shadow and any last difference blend
in. A see-through window shows doubled for those frames. Behind a
switch word, an owner A/B.

## Later

What is left of the overview's lead after task 7, each only if the
numbers or the owner's eye ask for it:

- **One surface per monitor.** A pinned peek is a monitor-sized window
  per moving window; the overview and the switch draw every window in
  one. Folding peeks into one surface per monitor removes the z-order
  between stand-ins (thumbnail order replaces it; task 4's "below
  neighbours" becomes draw order) and is the merge described below.
- **A thread for stand-in frames.** A peek frame needs only its tracks,
  so it could run as the overview's does, with the WM thread sending
  tracks and receiving "landed". Hand-overs stay on the WM thread. A
  large change to ownership (`PeekView` is reached from
  `platform_sync`); not before task 7 shows the WM thread is the limit.

Unchanged from the second version:

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
- Each pinned peek is a monitor-sized top-level window. They have no
  redirection bitmap, so no pixels of their own, but eight windows
  relaying out are eight such surfaces for DWM to walk. Task 3's
  `perf.log` (DWM side: the bench's `--dwm`, desktop only) is where that
  would show.
- `Live` peeks (rounded corners, parallax != 1) stay on the old path
  throughout. Square corners never pick `Live`, so the owner's config is
  covered; say so in `TODO.md` rather than leave it implied.
- `manager.rs` is about 4200 lines. Put new code in `peek_view.rs` and
  `transit.rs`, not there.
- Endpoint safety (`CLAUDE.md`): live checks stay manual and low-volume.
