# Plan: one surface per monitor, a trial

Written 2026-11-14 from a reading of `ffb2d88` (`tryout/start-latency`).
Nothing here has been seen running. Read `CLAUDE.md`,
`STAND_INS_PLAN.md` ("Words used here", "Rules for whoever carries this
out", "Later") and `START_LATENCY_PLAN.md` ("First draw") first.

## What is tried, and why

Today a tile that moves or resizes gets a pinned `WindowPeek`: a window
of its own spanning its monitor, holding thumbnails of the real window
and of its ring, backdrop and fill (`OverlayCopies`). Six moving tiles
are six monitor-sized windows.

The trial draws those same thumbnails on **one window per monitor**, a
`PeekSurface`, with documented DWM thumbnails only
(`DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`), as the
overview does. No new Windows API: a build with a private `dwmapi`
export looked up by ordinal (`tryout/scene-spike`) was quarantined on
the laptop even with benchmarks stripped.

Questions the bench has to answer, against `default` on the same branch:

1. Start: `input->first frame`, `command`, `peek_new`, `sg_window`,
   `peek_raise`, `peek_draw (worst frame)`.
2. Smoothness: frame p90, interval p90/p99 unchanged or better.
3. GPU: `perf_bench --dwm` (`dwm gpu busy p90 ms`, missed vblanks): one
   layer to blend instead of one per tile.

Expected from the 2026-11-13 numbers: the windows themselves are ~5 ms
of a 6-tile start (`sg_window` 3.6, `peek_raise` 1.6). The larger gain
is step 3 below: registrations kept between motions (`peek_new` 5.7 ms,
17.7 with backdrops; the ring's 8 registrations per tile, 7.5 ms in
`peek_cover`; and what `earlyfill` targets in the first draw).

## Scope

Only what `move_on_peek` puts on a pinned peek today: a tile of a tiling
layout that moves or resizes, by command. Everything else keeps a window
of its own, unchanged: scrolling columns, entering and leaving windows,
peeks a grab holds, `morepeeks` cases (floating, fullscreen, trails onto
another monitor, live peeks), workspace-switch surrogates.

Words, off by default:

| word | what |
|---|---|
| `onesurface` | Tile moves and resizes draw on their monitor's `PeekSurface`. |
| `readypeeks` | With `onesurface`: a landed tile's thumbnails stay registered, hidden, for its next motion. |

With neither set, behaviour is exactly today's.

## Design

### `PeekSurface` (`wm-platform`)

A click-through, never-activated window of the surrogate class (so
`is_surrogate` and everything that skips our windows still does),
spanning a clip rect, with no thumbnail of its own. Created on first
use per clip, kept until its clip goes (display change), the config
reloads, the WM pauses or exits.

- Shown while any tenant draws; hidden when none does.
- Z-order is the surface's: raised to the top of the non-topmost windows
  when a motion starts on it, as `raise_moving` raises each peek today.
  It stays there until its last tenant has handed back.
- Tenants stack in registration order: DWM draws a destination's
  thumbnails in the order they were registered, and that is the only
  order there is.

### Hosted peeks

`NativeSurrogate` and `WindowPeek` gain a hosted form that draws on a
`PeekSurface` instead of creating a window. Everything that places
thumbnails stays as it is: a pinned peek already places them in its
viewport's coordinates, and `OverlayCopies` already takes its
destination as a parameter.

What changes meaning for a hosted peek:

| today, on its own window | hosted |
|---|---|
| `create_popup`, `style_like_window` | none: registers on the surface |
| `set_visible` | the tenant's thumbnails; the surface follows its tenants |
| `reposition`, `move_to`, `repin` | none: the surface spans the clip |
| `raise_to_top`, `raise_to_top_of_band` | the surface's, once per motion |
| `begin_hand_back`'s `raise_above(source)` | none: the surface is above every tile already |
| `hwnd()` as overlay anchor | the surface's |
| `Drop` | unregisters its thumbnails; the window stays |

No existing hand-over step is reordered or dropped (rule 5): each keeps
its place and either acts on the surface or does nothing.

### Known differences on screen, accepted for the trial

- A floating, non-topmost window over a landing tile comes back above it
  when the tile's peek has handed back (a tick or two later), not at
  `begin_hand_back`.
- Two tiles that cross mid-motion stack in registration order, not in
  the z-order their peeks had. Their order today is the redraw loop's
  too, so it should match; check a swap.

### `readypeeks`

A landed hosted peek is kept with its registrations (window, ring
cells, backdrop, fill), hidden, for a few seconds, instead of dropped:
what `keeppeeks` did (`04826e6`, removed: it cleared the copies and
saved 2.3 ms of 17.7) without rebuilding anything.

- Reused only when nothing it was built from changed: window handle,
  clip, liveness, ring geometry and style, backdrop style, edge color.
  What changed is rebuilt, the rest kept.
- Order: kept registrations keep their old order. Tiles that only
  resize never overlap, so it doesn't matter; tiles whose paths cross
  are registered anew, in order, as today.
- Dropped on unmanage, leave, stray, config reload, display change,
  pause, exit, after its lifetime, and past a cap.

## Steps

One agent per step, in order: each builds on the last.

1. **Surface and hosted peeks** (`onesurface`). `wm-platform`:
   `PeekSurface`, the hosted form of `NativeSurrogate` and `WindowPeek`.
   `wm`: `PeekView` owns the surfaces; `move_tile` builds hosted peeks
   under the word for the scope above; raise, hide and purge points;
   overlay anchor. A stage `surface` for creating, showing and raising
   it. Unit tests for the pure decisions.
2. **Kept registrations** (`readypeeks`).
3. **Bench**: modes and columns in `isolate_modes.ps1`/`perf_bench`,
   and a measuring section here.

Checks per step: `cargo check --workspace --tests` and `cargo clippy
--workspace --all-targets -- -D warnings` for
`x86_64-pc-windows-msvc`. No new Windows API calls; say so in each
report.

## To measure (desktop)

`resize` and `move`, load 0 and CPU 8, backdrops on and off:
`-Modes nopeeks,default,onesurface,onesurface+readypeeks`, then the same
with `earlyfill+latealpha+keepfills` added to the last two. One
`--dwm` run (elevated) of `default` against `onesurface` for GPU.

## Step 1: as built

Branch `os/1-surface`. Not seen running.

### Representation

- `PeekSurface` (`wm-platform/src/peek_surface.rs`): a handle on an
  `Rc<SurfaceWindow>`, the window plus counts of tenants and of tenants
  drawing, and whether it was raised since it was last empty. Every
  hosted peek holds the `Rc`, so the window is destroyed only once the
  surface and all its peeks are gone; nothing is `Send` here already
  (`OverlayCopies` holds `Rc`s).
- `NativeSurrogate` owns a `Host`: `Own(hwnd)` as before, or
  `Shared(Rc<SurfaceWindow>)` (`create_hosted`). `WindowPeek::hosted` is
  always pinned, its viewport the surface's clip. Placement code is
  untouched: every thumbnail already goes to `surrogate.hwnd()` in
  viewport coordinates.
- Per-tenant visibility is `Thumbnail::set_held`: held thumbnails send
  `fVisible = false` whatever their look says, and thumbnails registered
  while their peek is hidden are registered held. `WindowPeek::set_shown`
  holds the window's thumbnail and every copy (`OverlayCopies::set_held`).

### Hosted meaning of each window operation

| `NativeSurrogate` / `WindowPeek` | hosted |
|---|---|
| `create` (`create_popup`, `style_like_window`, `SetWindowPos`) | `create_hosted`: registers held on the surface, which goes below the tile's window unless it is already (`SurfaceWindow::admit`); no `sg_window`, `sg_thumb` as before |
| `set_visible` | holds or lets go the thumbnail; the surface's drawing count shows it at 1 and hides it at 0 |
| `reposition`, `defer_reposition`, `move_to`, `repin` | none |
| `show_clipped` (live only) | draws at its place on the surface |
| `raise_to_top`, `raise_to_top_of_band` | the surface's (marks it raised) |
| `raise_above` (`begin_hand_back`), `keep_above` | none |
| `revive` | error: never kept warm |
| `set_fill_color` (gap fill) | none: one composition tree per window |
| `hwnd()` (overlay anchor, `peek_hwnd`, `stand_in`) | the surface's |
| `Drop` | thumbnails go, tenant count down, window stays |

### Who goes on a surface

`hosts_on_surface`: `onesurface`, and not a state change, a stack's
window (its tab bar is a window behind the stand-in), a trail, or a live
peek. `hold_on_peek` and `fullscreen_on_peek` never host. Then
`PeekView::tenancy`, for a new entry only: the clip's surface (made on
first use) takes tenants, i.e. was not raised since it was last empty,
and every tenant on it is on the same timing, not started yet, and
`lanes_stay_apart` from the new one (visible frame plus ring reach, at
both ends; a changing gap only without overshoot). Otherwise a window of
its own, as today.

Why the lane test: DWM stacks one window's thumbnails in registration
order only, and a tenant registers parts late (ring copy in
`track_overlays`, fill and companion on ticks), over other tenants'
windows. Today's peeks stack whole, last raised on top (`raise_moving`
in `peek_moves` order, the redraw loop's, which is also registration
order). Tenants that never overlap make the order moot; of two swapped
tiles, the second goes on a peek of its own, raised over the surface
after it, as its peek is today.

Why "not raised": a joining tile's real window still shows, and its
opaque backdrop copy must stay under it until it is cloaked. The
surface goes below each joiner's window; `cover_moved_overlays` then
puts the joiners' real overlays behind it (`restack_joining_overlays`),
or a higher joiner's backdrop would cover its copy between its cloak and
the raise.

### Z-order through a motion

1. Join: surface below the lowest joining window, joiners' overlays
   below it.
2. Flush, take-over, cloak, hand-over: unchanged.
3. `raise_moving`: the surface to the top of the non-topmost windows,
   at each tenant's call, as each peek is raised today.
4. `begin_hand_back`: no raise. The surface is above the landing window
   already; the uncloak keeps the window's place (`Normal`), and focus is
   deferred while the peek owns the window. A `SetForegroundWindow` on a
   `Releasing` tile raises its window over the surface as it does over an
   own peek today.

### Reuse and replacement

- A landing hosted peek is reused for a new motion that `host`s on the
  same clip; otherwise a new peek replaces it once drawn
  (`entries.insert`). A column taking over a hosted entry (`upsert`)
  gets a new peek of its own the same way.
- Interruptions drop the entry (strays, unmanage, `forget_*`): its
  thumbnails go, and the surface hides with its last drawing tenant.

### Purge points

Display change: surfaces whose clip is no monitor's. Config reload,
pause, shutdown (`drain_all_sessions`): all. A surface still drawing
tiles stays alive in them until they land. `PeekView.surfaces` is its
last field, so entries drop first.

### Not kept

A later command retargeting a hosted tile (`retarget_tile`), or reusing
a landed one, onto a path across another tenant: their parts then stack
in registration order. `note_crossing` logs it at debug. Options:
re-register the tenant that must be on top (backdrop, window, fill, ring,
each new before old goes), or hand it to a peek of its own.

### To check on screen (`GLAZEWM_ISOLATE=onesurface`)

- Resize two to six tiles: no blank or backdrop-only frame at the start
  or end, the surface hidden once they have landed.
- Swap two tiles: one on the surface, one on its own peek over it, as
  `default` stacks them.
- Retarget mid-motion (resize, then swap before it lands): the debug
  line, and whether a ring shows over the other tile.
- Two monitors: a resize on each, nothing drawn across the edge.
- A floating window over tiles: it goes under the moving tiles while
  they move, and comes back over a landed tile when its peek goes.
- Interrupted by a workspace switch: no tile left on the old monitor's
  surface, no stray frame.

## Full translation (owner's decision, 2026-11-14)

After steps 1 and 2, every stand-in the WM draws with documented
thumbnails moves onto the per-monitor surfaces, still behind
`onesurface` (off by default) until the owner has seen each kind and the
bench has compared them. Steps 1 and 2 have not been seen running when
this is written: a desktop check of `onesurface` between steps is worth
more than any reading, and each step below starts from what that check
found.

What stays on a window of its own, on purpose:

- **Live peeks** (a backdrop that can't be copied: rounded corners or
  parallax != 1). The real backdrop overlay has to follow the window,
  and it can only follow a window that moves. A surface never moves.
- **The old engine** (`ResizeSession`, `nopeeks`): the fallback and the
  bench's reference.

One surface is one height in the window stack, so a monitor gets up to
two: one at the top of the ordinary windows, one in the always-on-top
band for windows shown there (floating and fullscreen windows shown on
top). Inside a surface, order is registration order.

Steps, one agent each, in order. Each keeps the tuned hand-over steps in
place (rule 5) and adds no Windows API.

3. **Every tile motion.** Remove step 1's narrow admission: tiles whose
   paths cross share the surface, restacked by registering the upper
   one's parts anew, in order, each before the old one goes (step 2's
   B.2 mechanism); a motion that starts while earlier tenants land
   joins them; tiles in a stack, with their tab bar's stand-in.
4. **Entering and leaving.** Open, close, minimize and restore of tiles
   (`enter_on_peek`, `leave_on_peek`, `restore_on_peek`, `turn_round`):
   transits (zoom, slide, fade) on the surface. A leaving window stays
   below its neighbours: registered first.
5. **Scrolling layouts.** Moving columns and resting peeks
   (`sync_scroll_view`, `Role::Peek`, `Role::Moving`), the strip, opening
   columns, `stand_by` under a workspace switch (`slidepeeks`). A
   resting peek keeps its surface shown at rest: check what that costs
   idle (it must cost nothing per frame) and that clicks still reach the
   windows under it (`window_at`, the click guard).
6. **Held peeks and the top band.** Grab resizes (`hold_on_peek`), and
   the `morepeeks` cases that are not live: floating <-> tiling,
   fullscreen toggles (the second, always-on-top surface), tiles
   crossing monitors (each monitor's surface draws its part, replacing
   `Trail`).
7. **Workspace switches.** `WorkspaceSurrogate`'s windows become tenants
   of the surfaces: incoming and outgoing windows of a switch, its ring
   and backdrop copies, `pending_ws_cleanup`, a cut-short slide. The
   surface of a switch slides nothing itself: each tenant's thumbnail
   does, cut at the working area as moved with its workspace.
8. **Clean-up and bench.** What steps 3 to 7 left duplicated between
   `WindowPeek`, `WorkspaceSurrogate` and the hosted forms; bench modes;
   the measuring section.

Each step's report says which kinds still fall back to their own
window, and why.
