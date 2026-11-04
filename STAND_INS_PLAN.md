# Plan: one way to animate windows

For a later session on `tryout/todo-cleanup`. Read `CLAUDE.md` and
`TODO.md` first. Steps each end in a working, live-checked WM. Do one
step per session unless it is clearly done early.

Revised 2026-11-04, after step 1's check: share what the stand-ins do
alike (one core, one hand-over), not force what they draw differently
into one type. Move/resize join only if local fixes for their bugs don't
hold (step 3).

## Goal

What the switch and the scroll do alike is one piece of code: one core
stand-in (window, warm-up, alpha take-over and hand-back, companion wait,
ring and backdrop copies, `Live` choice), one overlay hand-over, one
hand-back. Bugs there then get fixed once, for both, and can't be fixed
in one and missed in the other. What they draw differently (the switch's
zoom, fade and sliding panel, the scroll's own-scale columns) stays with
each animation.

## Words used here

- **Real window**: the app's own window. Cloaked (hidden by the shell)
  while a stand-in draws it.
- **Stand-in**: a click-through window of ours showing a DWM thumbnail of
  the real window (`NativeSurrogate` underneath).
- **Overlays**: the WM's border ring (`NativeBorderOverlay`) and backdrop
  (`NativeBackdropOverlay`) windows that sit behind a real window.
- **Copies**: thumbnails of those overlays drawn inside a stand-in
  (`OverlayCopies`), so they move in the same DWM update as the window.
- **Pinned** stand-in: spans its monitor (or work area) and never moves;
  only its thumbnails move. **Moving** ("live") stand-in: a window resized
  and moved with `SetWindowPos` every frame.
- **Take-over / hand-back**: swapping between the real window and its
  stand-in at the start / end, without a frame showing both or neither.
- **Companion**: a window another app draws over the real one (shades'
  recolored copy); the stand-in shows it, and waits for it at hand-back
  (`HandoffWait`).
- **Flush**: `wm_platform::dwm_flush()`, which waits for DWM's next frame.
  Used as a barrier: put the stand-in up, flush, then cloak.

## What we have now

Three separate stand-in systems, grown one after another, plus two
animations that work differently by nature.

| Animations | Type | Drawing | Overlays | Take-over | Hand-back |
|---|---|---|---|---|---|
| Move, resize, open, close, minimize | `ResizeSession` (`wm-platform/src/resize_session.rs`) | Moving stand-in at `HWND_TOP`, `SetWindowPos` every frame (batched) | Real overlays moved alongside in the same `DeferWindowPos` batch | Stand-in shown, flush, cloak (`commit_pending_cloaks`) | Uncloak, then a 100 ms fade-out of the stand-in (`SESSION_FADE_OUT`); for a transparent window the stand-in drops to 0 at once instead |
| Workspace switch | `WorkspaceSurrogate` (`workspace_surrogate.rs`), driven in `animation/manager.rs` | Pinned (`PinnedViewport`), or moving (`Live`) when the backdrop can't be copied | Copies (ring, backdrop); the real ring parked (`park_switch_borders`), the real backdrop hidden on the first tick | Warm up at opacity 1, flush, `take_over` (alpha swap via `HiddenAlpha`), cloak | `WsCleanupStep`: uncloak (alpha prepared), wait for the companion, swap (`hide_thumbnail`, from below the window), show overlays under copies, re-anchor, drop |
| Scroll | `WindowPeek` (`window_peek.rs`), driven in `animation/scroll_view.rs` | Pinned, or moving (`Live`) as above | Copies, with a per-window `Handover` state machine (`Covering`, `Covered`, `Parked`, `Revealed`) | `warm_up`, flush (`flush_overlays`), `take_over` at the cloak | Raised above the window at landing (`cover_window`), `prepare_hand_back`, uncloak, `hand_back` once the companion is back (`Role::Releasing`) |
| Iris switch | `NativeIrisOverlay` | A snapshot of the monitor with a growing hole | none | none | none |
| Overview | `native_overview*` | Its own composition scene | its own | none | none |

Shared already: `NativeSurrogate` (window, thumbnail, gap fill,
companion, `HandoffWait`), `Thumbnail` (`dwm_thumbnail.rs`),
`OverlayCopies` (switch and scroll), `HiddenAlpha` (switch and scroll),
`SurrogateBatch`, and the rect maths (`placed_rects`, `clip_rects`,
`map_rect`).

Only move/resize can do (all in `ResizeSession`):

- Size changes: a growing window hasn't drawn its new size yet, so the
  thumbnail registration only grows once the real resize lands
  (`update_thumbnail_dims`, `sync_registration`), the rest is painted by
  the gap fill (`SurrogateFill`, edge colour sampled from the screen), or
  the whole window is stretched (`stretch`). `maybe_handoff` resizes the
  real window before the end so its reflow happens mid-motion.
  `pre_commit` waits (bounded) for the real window to reach its target.
- Retargeting with a direction change (`update_target`).
- Warm reuse: stand-ins kept 8 s after a session (`warm_surrogates`,
  `NativeSurrogate::revive`).
- Open/close/minimize: zoom (`update_zoom_fade`) and slides; close and
  minimize have no hand-back (`away_windows`).

## What we want to improve

Each problem below exists because the systems are separate:

1. **Fixes land three times.** The shades flash was fixed in all three
   (`d5316f10`, `4a4fe227`, `2e795fb6`); the shadow copies that drew
   stripes lived in two (`e1f259b0`); the double flush at scroll start came
   from two systems meeting (`c12b2737`).
2. **Move/resize use the oldest drawing.** Their overlays are separate
   windows moved alongside, which can land a frame apart from the stand-in.
   They have no alpha swap, so a transparent window is handled by zeroing
   the stand-in after the uncloak, and their companion wait is skipped for
   transparent windows (`TODO.md`, still open).
3. **Three hand-back strategies** (fade, zero-then-uncloak, alpha swap
   with steps) and **two overlay hand-overs** (scroll's `Handover`,
   switch's park plus `WsCleanupStep`) to keep correct.
4. **Flush barriers in several places** (`sync_scroll_view`,
   `commit_pending_cloaks`, switch start, session end), which is how
   two ended up back to back.
5. **Two fallbacks per system**: pinned and `Live` in both the switch and
   the scroll, chosen separately.

The revised steps cover 1, 3 and 5 for the switch and the scroll; 2 is
step 3's question. 4 is no longer a step: `c12b2737` removed the doubled
pair; revisit if another shows in `perf.log`.

## Target design

One core stand-in type in `wm-platform` (not named `StandIn`: `wm`'s
`animation/stand_in.rs` has one, imported next to `WindowPeek` in
`scroll_view.rs`), which `WorkspaceSurrogate` and `WindowPeek` are built
on:

- Pinned to an area (monitor, work area or viewport); `Live` only as the
  fallback for backdrops that can't be copied, chosen in one place.
- Draws the window's visible frame (past its invisible resize borders)
  plus ring and backdrop copies (`OverlayCopies`), carrying both focus
  styles up front (`carry_styles` / `show_style`).
- One lifecycle: `warm_up` → `take_over` (at the cloak) → placed each
  frame → `prepare_hand_back` → `hand_back` (once the companion is back)
  → drop.

One overlay hand-over in `wm` (from `scroll_view.rs`'s `Handover`): covered
by the copy, real one hidden a composited frame later, shown again under
the copy where the window lands. The switch's ring parking and the
overlay steps of `WsCleanupStep` become this.

`WorkspaceSurrogate` and `WindowPeek` keep only their drawing: the
switch's zoom, fade and sliding panel, the peek's own-scale columns and
the clip it follows. Whether to fold them into one type is decided after
step 2, when that is all that's left in them.

Out of scope: iris switch and overview (different in kind), shadow copies
(removed on purpose; `TODO.md` says not to bring them back without a live
A/B), app-specific fixes. Move/resize, open, close and minimize
(`ResizeSession`) only if step 3 finds local fixes won't do.

## Steps

Each step: small commits (one concern each), `cargo clippy` and tests for
the Windows target (see `TODO.md` "Checks"), push to
`tryout/todo-cleanup`, then a live check on the desktop before the next
step. The cloud session can't run the WM, so say plainly in commits and
`TODO.md` what wasn't seen live. Don't re-cut `test/todo-cleanup` without
a native build.

### Before each step: confirm it is needed

No code changes until this is done and written down:

1. Read the code the step touches, and `git log` since this plan was
   last updated. Check this plan's description of it still matches (types,
   functions, what each system does). If it drifted, fix the plan first.
2. Confirm the problem the step removes still exists in the code, with
   file and line: the duplicated logic, the special case, the extra
   flush. Name each one.
3. Check nothing smaller fixes it: a shared helper or a local fix that
   gets the same result with less risk beats a merge.
4. Weigh it: what gets simpler or better for the user, against what could
   break live. Say which recordings from step 0 would show a regression.
5. Write the findings under the step in this plan: confirmed needed (with
   the evidence), what will change, how it will be checked.

If the step isn't needed, isn't worth it, or a smaller change does the
job: change no code. Write why in this plan and `TODO.md`, and stop with a
recommendation (skip it, shrink it, or do the smaller change instead).

### Step 0: baseline

Not done yet. Before changing anything, record on the desktop from a
build of `ab822ab` or later docs-only commits, with a long `duration_ms`
(e.g. 3000) so one screenshot catches a mid-animation frame:

- Scroll: from rest, quick repeated presses, a column resized while
  scrolling.
- Workspace switch: both ways, two in quick succession, one cut short
  by the next.
- Move, a resize that grows and one that shrinks, open, close, minimize.

Include a semi-transparent window with a backdrop, and one themed by
shades. Also run once with `GLAZEWM_ISOLATE=live` (the `Live` fallback,
which square corners never pick otherwise), and once with
`GLAZEWM_PERF=1`, keeping `perf.log`. These are the A/B references for
every later step.

### Step 1: one core for the switch and the scroll

- 1a: one core type in `wm-platform` both use for what they do alike:
  creating the pinned or `Live` window, warm-up, the alpha take-over and
  hand-back, the companion wait, ring and backdrop copies (the peek's
  `carry_styles`, which the switch calls with its one style), and frame
  sampling with the size as a parameter. `WorkspaceSurrogate` and
  `WindowPeek` keep their drawing and the differences listed below.
  Moved code, no new options; behaviour stays the same.
- 1b: one function choosing `Live` from a window's backdrop styles, for
  both callers (`platform_sync.rs` switch start, `scroll_view.rs`).

Done when: both animations look identical to step 0's recordings, and
the duplicates below exist once.

#### Step 1 findings (2026-11-03): shrink it, no code changed

Decided 2026-11-04: shrunk to 1a and 1b above. The full merge first
planned here is dropped for now.

Checked against `3039190` (no code commits since this plan). Step 0's
recordings don't exist yet, so no step has an A/B reference.

Confirmed:

- Same code in both types: `WARM_UP_OPACITY`
  (`workspace_surrogate.rs:14`, `window_peek.rs:15`); the pinned/`Live`
  enum (`:45`, `:18`) and the `NativeSurrogate::create` call for each
  (`:247-296`, `:84-115`); `take_over`, `prepare_hand_back`,
  `restore_alpha` (`:632-652`, `:276-301`); `awaits_companion` (`:610`,
  `:444`); `carry_ring` (`:344`, `:239`); `carries_ring`,
  `carries_backdrop`, `hwnd`, `is_live`. About 80 lines of code, thin
  wrappers over the already shared `NativeSurrogate`, `OverlayCopies` and
  `HiddenAlpha`. The pinned frame sampling (`frame_rects` `:24`,
  `show_pinned` `:386`) differs only in the size passed to
  `placed_rects`.
- `Live` is chosen twice by the same rule: `platform_sync.rs:790-794`,
  `scroll_view.rs:464-467`.
- Fixes made in both types: `8a1fc9c` (frame sampling), `7e57367` and
  `e1f259b` (shadow copies), `2e795fb` (companion wait). Not ported:
  `5d43d2b`'s two-style backdrop carry (`TODO.md` "Still open" 4). The
  other repeated fixes (`d5316f10`, `d6ba457`, `0c93737`, `599f0aa`,
  `e6241cb`) were in the callers' hand-over and hand-back: step 2.

Differences beyond the three the merge would have kept as options (the
switch's zoom and fade, its panel and unclipped rect, the peek's
own-scale drawing), which a merge that keeps behaviour identical must
carry too:

1. Hand-back: the switch hides the thumbnail but keeps the backdrop copy
   (`hide_thumbnail`, `workspace_surrogate.rs:602`), from below its
   window (`:294`), as the real backdrop only shows a step later
   (`WsCleanupStep::ShowOverlays`). The peek is raised above its window
   (`window_peek.rs:436`), so draws it at 0 and drops the backdrop copy
   (`hand_back`, `:308`, `73f415e`).
2. Copies' opacity: faded with the window in the switch
   (`set_opacity_frac`, `:440`), held at full in the peek (`:412-413`).
3. Backdrop styles: one in the switch (`carry_backdrop`, `:379`), both
   focus styles in the peek (`carry_styles`, `:165`).
4. Pinned area: the monitor, fixed, in the switch; follows the clip in
   the peek (`:362-367`).
5. `Live` cut: at the sliding panel in the switch (`:513-525`), only by
   the window's bounds in the peek (`NativeSurrogate::show_clipped`).

So callers can't "change names only": their calls differ (`show_initial`
/ `warm_up`, `hide_thumbnail` / `hand_back`, `carry_backdrop` /
`carry_styles`).

Weighed: nothing the user would see changes. 2 to 4 can become
parameters and 5 an option, like the scale; 1 is hand-over behaviour
step 2 makes one, so a merge now adds a hand-back flag that step 2
removes. Over 1a, the merge also rewrites both drawing paths (the
part tuned live, from `8a1fc9c` on) for one type fewer, unchecked live.
Regressions would show in: a switch both ways (fade and
zoom too if used), two quick switches, a switch cut short, a scroll from
rest, quick repeated scrolls, the semi-transparent window with a
backdrop, the shades window, and a `Live` one (`GLAZEWM_ISOLATE=live`;
square corners never pick it otherwise).

### Step 2: one overlay hand-over and one hand-back

Move scroll's `Handover` and its role transitions into a module both
the switch and the scroll use, on step 1's core. The switch's
`park_switch_borders`, its first-tick backdrop hide and the overlay steps
of `WsCleanupStep` (`ShowOverlays`, `Reanchor`) become that hand-over;
its hand-back uses the same `prepare_hand_back` / companion wait /
`hand_back` sequence as the scroll. Behaviour changes a little here: it
removes difference 1 above (the switch's stand-in below its window at
hand-back, the peek's above), so which way wins is part of the check.

Done when: switch and scroll pass step 0's checks, including two quick
switches in a row and a switch cut short, and shades shows no flash.
Then decide, from what is left in `WorkspaceSurrogate` and `WindowPeek`
(motion and drawing only), whether folding them into one type is worth
it.

### Step 3: move and resize, only if local fixes won't do

Decide after step 2. Their open bugs: shades can flash on transparent
windows (`TODO.md` "Still open" 1: the companion wait is skipped when
`effect_opacity < 255`), and the ring copy thins during a column resize
("Still open" 5). First check, as in "Before each step", whether a local
fix does it, e.g. giving `ResizeSession` the `HiddenAlpha` swap the core
has, so its transparent-window special cases in `manager.rs` can go.

Only if not: move move/resize onto the core. What that needs, from the
first version of this plan: `ResizeSession`'s size handling as an option
(grow-only registration, gap fill and edge colour, stretch, the
mid-motion real resize, the bounded `pre_commit` wait, retargeting with a
direction change), unit-tested as `native_surrogate.rs` and
`overlay_copies.rs` test theirs; move and resize switched one at a time,
the old path kept behind a temporary switch until each is checked live;
warm reuse (8 s, one per window) kept; then open, close and minimize;
then the old paths deleted. Check: moves and resizes (grow, shrink,
mixed, retargeted) and all open/close/minimize styles match step 0,
transparent and shades windows included, and `perf.log` shows no slower
frames.

### Side fixes, any time

Independent of the steps, each small:

- `TODO.md` "Still open" 4: the switch's backdrop blends between focus
  styles again. Nearly free after step 1: the switch carries both styles
  as the peek does and crossfades them.
- `TODO.md` "Still open" 3: a peek reused for several moves keeps its
  companion-wait start time, e.g. reset it when a new move starts.

## Risks

- Most current behaviour was tuned against live recordings (companion
  timing, z-order on hand-back, retargeted columns). Keep each of those
  fixes' reasoning (commit messages from `599f0aa1` onward) and port it,
  don't re-derive it.
- `manager.rs` is ~4k lines; move code out of it in step 2 rather than
  growing it.
- Endpoint safety (`CLAUDE.md`): live checks stay manual and low-volume,
  no scripted input bursts.
