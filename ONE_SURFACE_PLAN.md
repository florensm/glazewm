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
each new before old goes), or hand it to a peek of its own. Done in step
2 (B.2).

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

## Step 2: as built

Branch `os/2-ready`. Not seen running.

### Ready peeks (`readypeeks`)

- When a hosted tile's entry is released (`tick`, `landed.released`),
  its `WindowPeek` goes to `PeekView.ready` (by window, apart from
  `entries`, so no query sees it) instead of being dropped:
  `WindowPeek::set_ready` hides it (every thumbnail held, so the surface
  counts one fewer drawing tenant), gives back any alpha a handover set
  aside (as its drop would), resets its copies (`OverlayCopies::set_aside`)
  and moves it on its surface from the tenants to a `ready` count
  (`SurfaceWindow::retire`). The surface hides, and takes new tenants
  again (`raised` reset), exactly as if the peek had gone.
- Kept only for a tile's peek with no trail (`keeps_ready`). Lifetime
  `READY_LIFETIME` (5 s), cap `MAX_READY` (16, oldest out).
- `move_tile` takes the window's ready peek (always out of the store)
  and uses it only if the window handle and clip are the same, it is not
  expired (`takes_ready`), the motion `host`s and `tenancy` admits it,
  on the very surface it is on. Otherwise it is dropped there and a peek
  is built as in step 1. `reuse_entry` is `new_entry` on it:
  `WindowPeek::reclaim`, then `carry_styles`, `warm_up` and the same new
  `Entry`, all inside `peek_new`.

Per part, what it was built from and when it is kept (the pure checks
have unit tests):

| part | built from | kept when | else |
|---|---|---|---|
| window thumbnail | source window, its companion (`Companion::find`) | `keeps_window_thumbnail`: registered, and `Companion::find` now gives the same companion (or none, as before) | registered anew (on top) |
| backdrop crops, vignette | copyable styles, `BackdropSource` per style on `monitor_bounds(viewport)`, vignette at the window's frame | `keeps_backdrop`: same set of styles, every source on the same monitor; sources refreshed for a changed wallpaper (`BackdropSource::refresh`, as `acquire` does), vignette moved to the new frame (`move_backdrop`) | dropped; `carry_styles` registers them anew (on top, then the window over them) |
| ring cells | the ring overlay's window, the cut (`RingPart::all(corner_reach)`) | `keeps_ring`, when `carry_ring` next carries it: same overlay window, same parts; rects, rest, corner reach and opacity are set anew from what `carry_ring` is given, as a registration would draw them | registered anew by `carry_ring` |
| fill strips | `FillSource` of one edge color | `keeps_fill`, when `carry_fill` next carries it: same color | registered anew by `carry_fill` (then the ring raised) |

Validity of the whole ready peek: same window id (the key) and native
handle, same clip and surface window, not expired, and the motion still
`hosts_on_surface` and passes `tenancy` (so it is never live, a column,
a trail, a state change or a stack's window).

### Per-field state after `reclaim`

`WindowPeek` (as `hosted`):

| field | after `reclaim` |
|---|---|
| `copies` | see `OverlayCopies` below |
| `surrogate` | see `NativeSurrogate` below |
| `mode` | kept: `Pinned` at the surface's clip, which is the same |
| `source` | the window's handle (the same, by validity) |
| `border_inset` | `compute_border_inset` again |
| `rest` | the target rect |
| `drawn` | `Shown`; `warm_up` follows as for a new one |
| `shows_backdrop` | false |
| `opacity`, `frac` | 255, 1.0 |
| `hidden_alpha` | empty (restored at `set_ready`) |
| `frame_rect`, `visible_rect` | `None` |
| `stands_by` | false |

`NativeSurrogate` (hosted, as `create_hosted`), `reclaim`:

| field | after `reclaim` |
|---|---|
| `host` | kept: the same surface's window |
| `thumbnail` | kept with `create_hosted`'s look set on it (content rects of the target, 255, visible, held), or registered anew held |
| `companion` | `Companion::find` again |
| `companion_wait` | default |
| `companion_checked_at` | now |
| `content_size` | target's size |
| `border_inset` | none |
| `is_visible` | false (hidden since `set_ready`) |
| `last_opacity`, `rest_opacity` | 255 |
| `last_rect` | `None` |
| `covered` | from the look, as `create_hosted` |
| `fill`, `fill_color` | `None` (never set on a hosted one) |
| `stretch` | false |
| `window_size` | the clip's size |
| `is_ready` | false; the surface takes it back as a tenant and goes below the window unless it is already (`SurfaceWindow::rejoin` → `admit`) |

`OverlayCopies` (as `OverlayCopies::held()` followed by `carry_styles`):

| field | after `set_aside` / `reclaim` |
|---|---|
| `held` | true |
| `placement` | `None` |
| `ring` | `None`: the ring is in `spare_ring` until `carry_ring` takes it back (so nothing of it is placed or drawn before, as with no ring at all) |
| `fill` | `None`: in `spare_fill` until `carry_fill` takes it back; hidden, not shown, opacity 0, as a just registered one |
| `backdrop` | kept or dropped (above); kept: no style, `t` 1, not shown, opacity 0, never placed, thumbnails hidden, rest and frame at the target, as `carry_backdrop` leaves a new one |
| `spare_ring`, `spare_fill` | dropped once anything they must stack above is registered anew (window thumbnail or backdrop at `reclaim` or in `carry_styles`, a companion adopted, `register_anew`) |

The new `Entry` is new: `Handover::None` for ring and backdrop,
`composited` false, no transit, stretch, hold or trail. So the cover pass
covers the real ring and backdrop as for a new peek, `carry_ring`'s
`None` branch parks the overlay and carries the ring (taking the spare
back instead of registering 8 cells), `Covering` → `Covered` → `Parked`
→ `Revealed` run as before; `carry_backdrop` sees `carries_backdrop()`
and walks the same steps. No step is skipped or reordered.

### Stacking

Within a tile, registrations stack backdrop, window, fill, ring.
Kept parts keep their order. A part registered anew goes on top of
everything on the surface, so every part that must sit above it is
registered after it: the backdrop's `carry_styles` raises the window,
`carry_fill` raises the ring, and the spare fill and ring are dropped
(to be registered on top when carried) whenever the window thumbnail or
backdrop is registered anew. Stated on `WindowPeek::reclaim` and
`OverlayCopies::spare_ring`.

### Purge points

Ready peeks go: on `forget_entering` (unmanage), `leave`, a column
`upsert`, `enter`, a `move_tile` that doesn't take them, `drop_strays`
(windows no longer shown, and expired ones; free while none is kept),
past the lifetime and cap, and always before their surface:
`retain_surfaces` (display change) drops those of a dropped clip first,
`drop_surfaces` (reload, pause, shutdown) all of them first, and
`ready` is declared before `surfaces`.

### Stages

`peek_new` wraps `reuse_entry`. Kept parts cost no registration: in
`sg_thumb` only `Companion::find` and one property update; in
`peek_styles` nothing for kept styles; in `peek_ring` and `ring_raise`
no 8-cell registration (the ring isn't registered anew when the fill is
taken back). A part rebuilt shows in the stage that builds it, as for a
new peek.

### B.1: one raise per pass

`platform_sync`'s raise loop opens a raise pass (`set_raise_pass`). A
surface raised in the pass is marked on top; its next raise in the pass
is skipped while nothing may have gone above it. That ends with: the
pass, any `raise_moving` of something other than that surface (an own
peek, a trail, another surface: `note_raised`), `raise_in_band`, the
surface going below a joiner (`admit`), shown or hidden
(`note_drawing`), or raised into a band. Outside a pass every raise is
made, as in step 1.

### B.2: a retarget across another tenant

Checked against own-window peeks: a retarget (`retarget_tile`) raises
nothing; they keep the order of the commands that put them up, the
later on top. A landed one reused for a new command is raised again by
it. So each tenant gets a rank as `move_tile` puts or reuses it (the
raise order), not as it is retargeted: the one on top is the later
ranked, not necessarily the one retargeted.

On a retarget or landed reuse whose lane crosses others on the same
surface (`crossing_group`, transitive), every tile of the group but the
lowest-ranked is registered anew in rank order (`register_anew`): all
its parts go up held in order, then each part is let go before its old
thumbnails go, top first, so no composition shows the tile without a
part. The group is kept: at the end of every `tick` and
`track_overlays`, a member whose registrations changed (late fill,
companion, ring cut) has every member ranked above it registered anew
again (`restacked`).

Chosen over moving the tile to a peek of its own: that would need a new
window mid-motion, a ring and backdrop handover onto it with the real
ones already parked, and a composited frame before the old peek may go,
which a retarget pass doesn't flush for; and it would have to move the
higher-ranked tile, not the retargeted one. Re-registration uses only
thumbnails of windows already composited.

### B.3: surfaces off the monitors

Making a surface (`tenancy`) first drops every surface with no tenant
and no ready peek whose clip is none of the monitors' (`is_stale_surface`).
`move_on_peek` reads the monitors' rects only when no surface spans the
clip yet.

### Windows functions called by the new code

All already called by `wm-platform`: `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail` (thumbnails);
`SetWindowPos` (`admit` on rejoin, fewer surface raises); `ShowWindow`
(the surface hiding as a peek is set ready, as on its drop);
`EnumWindows` through `Companion::find`; `GetWindowRect` and
`DwmGetWindowAttribute` through `compute_border_inset`;
`MonitorFromPoint`, `GetMonitorInfoW` through `monitor_bounds`; the
vignette's move and the wallpaper poll through existing helpers;
`GetLayeredWindowAttributes`/`SetLayeredWindowAttributes` through
`HiddenAlpha::restore`.

### To check on screen (`GLAZEWM_ISOLATE=onesurface,readypeeks`)

- Two resizes in a row on the same tiles: the second starts with no
  blank, backdrop-only or ring-less frame; `peek_new`, `peek_ring`,
  `ring_raise` well under the first's.
- A resize, a focus change, a resize: the ring copy in the new focus
  color from the first frame, no second ring line.
- A resize, a config reload (ring width or backdrop changed), a resize:
  the new ring width and backdrop, nothing of the old.
- A window closed between two motions of its neighbours: no frame of it.
- A swap retargeted across another tile mid-resize: the tile commanded
  later stays whole over the other (no ring or fill of the lower one over
  the upper one's window), and a semi-transparent window doesn't flash
  denser.
- A display change while tiles move, then a resize: the old clip's
  surface goes once a surface is made for the new one (one window of the
  surrogate class per monitor in a window list).

## Step 3: as built

Branch `os/3-tiles`. Not seen running.

Every tile move or resize `move_on_peek` hosts (`hosts_on_surface`) now
goes on its monitor's surface: `PeekView::tenancy` falls back to a
window of its own only if no surface can be made.

### Today's order, and how it is kept

Own-window peeks: each goes up just below its window, then the raise
loop raises them in `peek_moves` order, the redraw loop's
(`windows_to_update` by focus order, walked in reverse): of the tiles one
command moves, the most recently focused ends on top. A later command's
go over an earlier one's; a retarget raises nothing (step 2).

On a surface, `Tenancy::rank` is that order (given as `move_tile` puts or
reuses a tile), and new tiles register in it too. What breaks it: parts
registered late (ring copy in the cover pass, fill and companion on
ticks, ring cuts) and parts kept (ready peeks, a landing peek reused).

- Every `Thumbnail` gets a process-wide sequence number as it is
  registered (`Thumbnail::order`); `WindowPeek::stacking` is a peek's
  first and last, spares included (held, they keep their place). So the
  order on a surface is known exactly, not guessed from changed handles.
- `out_of_order` (tested): in rank order, a tile with a thumbnail under
  the last of a lower-ranked tile it may meet is registered anew whole
  (`register_anew`, step 2's B.2, unchanged); one restacked then counts
  as newest, so the higher ones meeting it follow.
- May meet (`Entry::meets`, `lanes_meet`, tested): on one clock
  (`WindowAnimationState::shares_clock`: same start instant or both yet
  to start, same delay, duration, easing) unless `lanes_stay_apart`;
  otherwise their lanes' hulls overlap, or either easing overshoots
  (then always). A landed tile (`HandingBack`, `Releasing`) only where it
  landed.
- Checked at the end of `move_tile` (both paths), in `retarget_tile`
  (lane now from the retargeted motion's start, so the one-clock test
  holds), and at the end of `tick` and `track_overlays`, as in step 2.
  Replaces step 2's handle records, `crossing_group` and `restacked`.
- Cost: nothing for tiles in order or apart (neighbours resized on one
  clock). A swap: one restack of the upper tile, after the cover pass
  registered the lower one's ring copy over its window. Shown in a new
  cross-cutting stage `restack`, with `perf_bench` columns `restack ms
  (worst frame)` and `restack ms (outside frames)`.

### Fallbacks removed

| step 1 kept a tile off a surface when | which protected | now |
|---|---|---|
| its lane may meet a tenant's | stacking against late registrations | the rank-order restack above |
| a tenant was started or on other timing | only the lane test's premise, one eased progress | `shares_clock` picks that test, the hull test otherwise |
| the surface was raised since last empty | a joiner's still-shown window against the surface's place | below |

A surface raised over its tenants can't go below a joiner's window (it
would take tiles drawn over other windows under them), and above it the
joiner's opaque backdrop copy would hide that window until its cloak.

- `SurfaceWindow::admit` (`goes_below`, tested): not raised, as before.
  Raised: never down; up to just below a joiner's window above it, so no
  window between covers the joiner's copies once it is cloaked and before
  the raise, which puts the surface there anyway.
- Over a joiner's window below it, the joiner draws that window as it
  is: its thumbnail at `WARM_UP_OPACITY` as today, and now its backdrop
  copy too (`WindowPeek::warms_backdrop`, hosted peeks only), at full
  again in `draw` at `take_over`, after the window's thumbnail (backdrop
  last in, as `draw` has it). Not held instead: a held thumbnail is not
  composited, and the warm-up is there so the first composition after
  the cloak has content. The ring copy is drawn at full over the real
  ring as today, which must be composited before the real one hides; the
  fill shows only once `Shown`.
- `restack_joining_overlays`: on a raised surface, joiners not yet
  composited whose window is above it (after `admit`, only the highest)
  get their shown overlays put behind it, as on one not raised.

So from admission to the raise: warmed (window over its faint copies, or
its faint copies over it), flush, `take_over` (thumbnail full, then
backdrop full, then the window's alpha hidden, then the cloak), hand-over,
raise. No frame shows neither (the thumbnail is full before anything of
the window goes), none the opaque backdrop copy over the shown window.
Hand-over steps are untouched (rule 5).

`readypeeks`: a reused ready peek keeps its old orders, so it is
restacked at `move_tile` if a lower-ranked tile it may meet registered
anything after it, like any tenant.

### Still on a window of its own

- **A stack's window.** Its tab bar is a real window (`NativeStackTabBar`)
  moved each frame by `sync_tab_bars` behind its stand-in (`stand_in`'s
  anchor). Behind a surface it would be under every tile there, so a
  stacked tile ranked on top would lose its bar under a lower one
  crossing it, and lanes leave the bar out. Missing for a contained
  change: a copy of the bar on the surface (a thumbnail of its window in
  `OverlayCopies`, between window and ring, with a cover, park and reveal
  hand-over like the ring's), lanes and ring reach taking in the bar, and
  a resize that draws the tabs at the new width (a thumbnail stretches
  their text: cells like the ring's, or the bar painted at the target).
- State changes, trails, live peeks: unchanged (later steps; live by
  design). `hold_on_peek`, `fullscreen_on_peek` never host.
- No surface could be made.

### Known differences on screen

- An own-window peek (stack, live, trail, state change) raised between
  two tenants of one surface ends above or below all of them: the surface
  is at its last raised tenant's height.
- A joiner below a raised surface: its ring copy (and faint thumbnail
  and backdrop) are drawn over windows between it and the surface, and
  over its window's shadow, from its first draw to the raise of the same
  pass, which puts them there anyway.

### Windows functions called by the new code

All already called by `wm-platform`: `SetWindowPos` (`admit` moving a
raised surface up), `GetWindow` (through `companion::is_above`, in
`PeekSurface::is_below` and `admit`), `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail` (restacks,
backdrop opacity).

### To check on screen (`onesurface`, then `onesurface,readypeeks`)

- Swap two tiles: both on one surface (one window of the surrogate class
  in a window list), the focused one whole over the other all the way (no
  ring or backdrop of the lower over the upper's window); a debug line
  `Stacking ... over a tile it may cross` once per swap.
- Move a tile across two others: the moved one, if focused, over both;
  no ring cut through it.
- Resize, then resize again before it lands: no blank, backdrop-only or
  ring-less frame; neighbours not restacked (no debug line) unless
  their rings meet in the gap.
- A resize while the previous one is still landing: the new motion's
  first frames show the tile, not its backdrop, and no frame without it;
  a floating window between (focused after the first resize) not drawn
  over the moving tile once it is cloaked.
- A stacked tile: still its own peek with its bar, as `default`.
- Two monitors: a swap on each, each surface on its own monitor, nothing
  across the edge.
- With `readypeeks`: swap, then swap back within 5 s: the reused peeks
  stack as the first time; count the debug lines (a reused peek is
  restacked only if a tile ranked below registered over it).
  `restack` in the profiler stays near zero for resizes.

## Step 4: as built

Branch `os/4-transits`. Not seen running.

A tile of a tiling layout that opens, closes, minimizes or is restored
on a pinned peek now draws on its monitor's surface when
`hosts_transit` says so (`hosts_on_surface`, tested). Hand-over steps are
untouched: each runs where it ran, on the surface or on nothing, as in
step 1's table.

### Today's order, found by reading

| kind | steps | z-position of its own peek |
|---|---|---|
| open (`enter_on_peek`) | window cloaked since `manage_window`; peek up, drawn at the transit's start; `Frozen` step `Keep`, window moved behind its cloak; ticks draw `place_scaled`; ring made parked and concealed (`conceals_new_ring`); lands, `begin_hand_back`, uncloak (`Normal`), hand back | just below its new window, normally at the top; never raised (only a floating one, `peek_raises`). The pass's moved neighbours are raised after the loop, so they end over it; earlier passes' peeks are under it |
| restore (`restore_on_peek`) | `restore_cloaked`, then as an open, or `turn_round` of a minimize still playing | raised to the top as it is drawn, inside the loop: under the pass's raises too |
| close / minimize (`leave_on_peek`) | peek up warmed under the still-shown window, ring and backdrop handed over (`track_overlays`); a close's first tick takes over and cloaks it, a minimize's flush and take-over run before the caller minimizes; never handed back; dropped when its transit ends, then `WM_CLOSE` | just below its real window, never raised: under every peek raised before, and under everything raised later |
| close during an open, close of a moving tile | the same entry turns `Leaving` where it is | kept |

### Hosted

- **Ranks** (`Rank`, tested): (raise pass, raised, order). Moves:
  raised in their pass. Opens and restores (and a minimize turned
  round): put on top in their pass, under its raises. A fresh leaving
  tile: below every other (pass 0); several leaving ones stack in the
  order they started. A tile turning round in place keeps its rank;
  `turn_round` re-ranks as a restore. `set_raise_pass(false)` counts the
  passes. Out-of-order registrations are restacked by step 3's
  `out_of_order` and `register_anew`, so a close's neighbours are
  registered anew over it if they were registered first (a close while
  they move), and an open's neighbours moved earlier in the pass over
  it (`enter` restacks at once, `track_overlays` again after the ring
  copies).
- **Lanes** (`Entry::lane`): a transit's tile is drawn within its
  motion's rect for a zoom or fade, and within the rect plus where it
  slides from for a slide (`transit_reach`, tested); a slide is never on
  one clock with another tile. A held motion (start = target) never
  overshoots, whatever its easing.
- **Admission** (`admission`, tested): a shown window (a closing or
  minimizing one, a restore the OS showed) as before; a cloaked one (an
  open, a restore behind `restore_cloaked`) puts the surface just above
  it, unless the surface is raised and above it already, and holds it
  there (`hold_place`: never down from then on). Just above, not below:
  the window shows nothing, and the hand-back (no raise for a hosted
  peek) needs the surface over the window it uncloaks.
- **Leaving**: warmed up as a step 3 joiner is (thumbnail and backdrop
  copy at `WARM_UP_OPACITY`, ring copy at full over the real ring), so a
  surface above the still-shown window draws nothing of it over that
  window until `take_over` (thumbnail full, backdrop full, alpha hidden,
  cloak, in that order). Then `hold_place`: nothing raises it, and a
  later joiner must not take it down. When its transit ends its entry
  drops, its thumbnails go, and the surface hides with its last drawing
  tenant (`note_drawing`).
- **Fades**: `place_scaled`'s `frac` reaches every copy of a hosted peek
  as of its own: ring and backdrop opacities in `place_scaled`, the ring
  in `carry_ring`, the backdrop at full times `frac` at `take_over` after
  its warm-up, kept by restacks (`held_anew` copies each look).
- **Ready peeks**: a leaving window's is dropped (`leave`); an entering
  one has none (`enter` drops any, a restore's included: its window was
  minimized since); one that entered and landed is kept as any tile's.

### Still on a window of its own, and why

- Floating windows (`opens_floating_on_peek`: their peek is raised in
  its band, `peek_raises`), fullscreen ones, and any window not tiled:
  the top band is step 6's.
- A stack's window (step 3), a live peek, a scrolling column (step 5).
- A tile whose workspace has a stack or a live tile (`beside_own_peek`):
  that one's own peek and the surface stack by their last raise in the
  pass, so the surface could end over it, and the entering or leaving
  tile with it, where it must be under.
- A close or minimize of a tile handing back (`HandingBack`,
  `Releasing`): the old path's surrogate, as on own peeks
  (`Leave::Unsupported`).

### Edge cases

- Closed while neighbours move on the surface: admitted under the
  raised surface (stays), drawn faint until the tick's take-over,
  neighbours restacked over it.
- Several closing at once: each below the others already leaving.
- Close during an open: turns round in place, rank kept. An open right
  after a close in the same slot: ranks over the leaving tile.
- Minimize then restore quickly: `turn_round` re-ranks it and restacks
  it over the tiles it may meet, as its own peek's raise puts it.
- Last window of a workspace: the surface hides as its transit ends.
- Workspace switch mid-transit: entering tiles go with `drop_strays`,
  leaving ones play on (not strays), as own peeks; `forget_all_leaving`
  at shutdown drops them before the surfaces.
- Two monitors: one surface per monitor clip, as for moves.

### Known differences on screen

- As for moves: once a neighbour's raise puts the surface on top, an
  entering or leaving tile is drawn over windows that were above its own
  peek (a floating window over a closing tile), and its ring copy over
  windows between before its take-over.
- Opens cost a restack of the neighbours moved before them in the pass
  that meet them, twice when their ring copies register after the
  open's (`restack` stage). An open ranked on top instead would cost
  none but draw over its neighbours where today they cover it.

### Windows functions called by the new code

All already called by `wm-platform`: `SetWindowPos`, `GetWindow`,
`GetWindowLongPtrW` (`window_class::place_above`, `companion::is_above`),
`DwmGetWindowAttribute` (`is_cloaked`), `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail`, `ShowWindow`.

### To check on screen (`GLAZEWM_ISOLATE=onesurface`)

- Open a window among two tiles with each `window_open` style used
  (the bench config's zoom, a fade, a slide): one surrogate-class window
  per monitor in a window list, no peek of its own; neighbours cover the
  opening tile where they still overlap it, as with `default`; border
  and backdrop zoom and fade with it; no frame without it at the end.
- Close a tile beside two others: it shrinks under its neighbours, never
  over them; no frame where its window shows over its opaque copy, or
  neither shows.
- Close during an open: it turns round where it is.
- Minimize and restore with `toggle-minimized`, then both within the
  animation: it turns round with no jump, over its neighbours.
- Close a tile while its neighbours still move from a resize: they stay
  over it; one `Stacking ... over a tile it may cross` debug line each.
- Close the last window of a workspace: it plays in full, and the
  surface is hidden once it has gone.
- Two monitors: an open on each, each on its own monitor's surface.
- A stack on the workspace: opens and closes there keep their own
  peeks, as `default`.

## Step 5: as built

Branch `os/5-scroll`. Not seen running.

### Today's z-order and clip for column peeks, found by reading

- Clip: `displayed_scrolling_workspaces` gives the monitor's working
  area, and a pinned column peek spans it (`WindowPeek::new`); tiles'
  surfaces span the monitor rect (`tile_clips`). Not the same, so
  columns get a surface of their own per working area (shared with tiles
  only where the two are equal, e.g. an auto-hidden taskbar). A resting
  surface then spans exactly what a resting peek spans: nothing new for
  Explorer's fullscreen detection.
- A clip change (column on another monitor's workspace, a working area
  change) `repin`s an own peek; a hosted one is replaced (below).
- Z-order: every column peek goes up just below its real window and is
  never raised, but for a restored column (`raise_moving`) and at
  `begin_hand_back` (directly above its window). Columns are not in the
  raise loop's `peek_moves`. A resting peek stays just below its cloaked
  window, which `windows_to_bring_to_front` leaves where it is
  (`is_hidden_in_layout`), so a floating window can cover it while the
  shown tiles go over that floating window.
- Clicks: a surface is click-through like a peek. A press on a resting
  peek is focused by `focus_clicked_peek` through `window_at` (rects of
  `Role::Peek` entries), moving ones through `moving_rects` and the
  motion guard: both read `visible_rect`, the same for a hosted peek.
- Idle: `is_active` is false with only `Role::Peek` entries, so no tick
  runs for a resting surface; it stays shown with no per-frame work.

### Admission (`hosts_column`, `keeps_rest_hosted`, tested)

A column `upsert` puts on a pinned peek is hosted on the surface of its
working area unless: its peek is live; a grab holds it; it is a stack's
window; or its workspace has a column on a peek of its own (stacked,
live or held), which stacks by where that peek went. And, by role: one
that rests (`Role::Peek`, or moving to end hidden) only if no other
tenant of that surface does, and never under `slidepeeks`. Own entries
stay own for their life; a hosted one no longer admitted (held,
switch-bound, a second resting one, another clip, its surface dropped)
is replaced by a new peek (own or on the right surface), its motion,
transit and stretch carried over, and the old one is kept in
`PeekView.replaced` until `sync_scroll_view`'s flush has composited the
new one (`drop_replaced`): a frame may show both, never neither. A
column taking over a tile's hosted entry goes the same way (it was
dropped at once before).

### Moving columns

- Joining with its window shown: `admit` as a tile, warmed, then the
  scroll view's flush, `take_over`, cloak, as before. Overlays of joiners
  above the surface go behind it (`restack_joining_overlays`, now also in
  `flush_overlays`). Cloaked (opening, restored behind `restore_cloaked`,
  resting turning to moving is moved on in place): just above the window
  and `hold_place`, as step 4.
- No raise: columns were never raised. Restored ones are, as before
  (`raise_moving`, the surface).
- Landing shown: after `begin_hand_back`, `stay_over_window` puts the
  surface just above the landing window unless it is above it already
  (`SurfaceWindow::keep_above`), so the uncloak never shows the window
  over its peek ("keeps its place" untouched).
- Ranks: given as the entry is made; a focused or opening column
  `Raised`, others `InPass`, in tree order: a focused column's own peek
  was over its neighbours' (a swap by `move`), and an opening one's just
  below its new window, on top. Lanes follow each upsert; columns of one
  scroll on one clock keep their gap (`lanes_stay_apart`), so no
  restack. A retarget keeps the rank, as it raised nothing.
- Retarget before landing: moved on in place (`retargeted`), lane anew.
  Columns started on different clocks (some retargeted, some new) are
  checked by hull and restacked by rank if out of order.

### Resting columns

- At most one per surface. Admitted just above its cloaked window.
- `settle_resting_surfaces` (end of `tick` and of `sync_scroll_view`):
  once every tenant of a surface rests (`rest_anchor`), it goes just above
  that tenant's cloaked window (`PeekSurface::rest_above`), once until
  something else moves it. Just above a window that shows nothing is
  where its own peek, just below it, draws: what covers it at rest
  (a floating window over the half-visible column) and what it covers are
  as today.
- Resting, then moving (focus scrolls it in): moved on in place, the
  surface goes up as needed, handed back above its window, then down
  again for any resting tenant left.

### Ready store (`readypeeks`)

A column's hosted peek released after landing shown is kept ready as a
tile's (`keeps_ready` no longer excludes columns), with the same
validity: window, clip, surface, not expired, admitted. Taken only for a
column whose window shows (a ready peek rejoins its surface as for a
shown window); otherwise dropped. A resting peek is in use, never ready.

### Edge cases

- Departed columns (`take_departed`): hosted ones drop after the flush,
  as own ones.
- A stack's active tab: stacks stay own.
- Display change: `retain_surfaces` keeps working areas too; a tenant on
  a dropped surface is replaced at its next upsert (a moving one with an
  unchanged target only once it lands). Reload, pause, shutdown: surfaces
  go from `surfaces`; their windows go with the last peek holding them,
  so tenants always before the surface. A resting tenant left on a
  dropped surface (pause) draws on until its next upsert.

### Still on a window of its own, and why

- Live peeks: they move with their windows.
- Held columns (grabs) and every column of a workspace with one: step 6.
- Stacks and their workspaces: the tab bar (step 3).
- Resting columns under `slidepeeks`: a workspace switch stands one by
  or keeps it for its surrogate (`stand_by`, `keep_switching_out`),
  which needs a window of its own until step 7.
- A second resting column on one surface (a monitor with neighbours on
  both sides): one height at rest. Options: an own window (as built), or
  a second surface per monitor for it.

### Known differences on screen

- While anything moves on a column surface, a resting tenant on it is
  drawn at the surface's height (below the lowest joiner, or above a
  landing window): a floating window over it may be covered until the
  motion ends.
- As for tiles, joiners' ring copies are drawn over windows between them
  and the surface until cloaked.

### Windows functions called by the new code

All already called by `wm-platform`: `SetWindowPos`, `GetWindow`,
`GetWindowLongPtrW` (`window_class::place_above`, `companion::is_above`),
`DwmGetWindowAttribute` (`is_cloaked`), `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail`, `ShowWindow`,
`CreateWindowExW` (a surface for a working area), `DwmFlush` (the scroll
view's flush).

### To check on screen (`GLAZEWM_ISOLATE=onesurface`)

- Scroll the strip: one surrogate-class window for the monitor's working
  area, no peek of its own per column; no blank, backdrop-only or
  ring-less frame at start or end; the surface hidden once landed.
- Scroll again before it lands: it glides on, no jump, no double ring.
- Open and close a column mid-strip: the new one grows between its
  neighbours, over them where rings meet.
- Two monitors, a column half off each inner edge, at rest, a floating
  window over each: the floating window stays over the column, as with
  `default`; the surface stays shown, and the profiler shows no ticks.
- Click the half-visible column: it is focused and scrolls in, handed
  back with no flash of the window over its peek.
- Resize a column (`stretch` and `fill`).
- Switch workspaces to and from a scrolling one, with and without
  `slidepeeks`: no column left faint, blank or doubled.
- Three monitors, columns cut at both edges of the middle one: one on
  the surface, one on a peek of its own.

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

## Workspace switches: as built

Branch `os/7-switch` (from `3c3c08c`, before step 4). Not seen running.

### A switch surrogate today

- One `WorkspaceSurrogate` window per switching window (`NativeSurrogate`,
  click-through popup of the surrogate class). `PinnedViewport`: spans the
  monitor's **working area**, never moves; per frame only its thumbnail's
  `rcSource`/`rcDestination` (and its ring/backdrop copies) change, cut at
  `panel ∩ monitor` (the working area moved and scaled with its
  workspace) and by the window's bounds at the working area (f8a226b: a
  monitor-sized window reads to Explorer as a fullscreen app, and app bars
  set to make way for one hide). `Live` (uncopyable backdrop): sized to its
  visible rect, moved every frame by a batched `SetWindowPos`.
- Z-order: created just below its own real window (`insert_after = hwnd`),
  so surrogates stack as their windows do, topmost ones in the band,
  floating over tiling, other workspaces' (cloaked) windows irrelevant.
  Outgoing: `show_initial` warms it at opacity 1 under the still-shown
  window, the flush composites it, `take_over` brings it to full and hides
  a semi-transparent window's alpha, then the cloak. Incoming: cloaked
  windows; shown at rest at start opacity (fade, zoom), at warm-up (cut),
  or on the first frame (slide); at the end uncloaked *over* their
  surrogates (`prepare_hand_back`, `hide_thumbnail`; the backdrop copy
  stays under the window until `WsCleanupStep::Drop`). A companion's
  (shades) surrogate is put above its window (`cover_window`).
- Tile peeks at the start: outgoing tiles' moving or landing peeks are
  dropped by `drop_strays` before the pre-pass (no hand-back; their
  windows are taken over by the switch in the same sync). Incoming
  windows have none; resting peeks of hidden columns stand by
  (`slidepeeks`), own windows just below their cloaked windows.

### Hosted form (`WorkspaceSurrogate::hosted`)

Only `PinnedViewport`. Registers on a surface through
`NativeSurrogate::create_hosted` (held, tenant count, `admit`), its
copies `OverlayCopies::held()`. Placement is unchanged: thumbnails are
placed from the surface's top-left (`origin`), which is the working
area's.

| operation, own window | hosted |
|---|---|
| `new` (`create_popup`, `style_like_window`, `SetWindowPos` below `hwnd`) | `hosted`: thumbnail held on the surface, which goes below `hwnd` unless already (`admit`); `None` for a topmost window or one with a companion |
| `set_visible` (show_frame, clear_frame) | `set_shown`: window thumbnail and every copy held or let go; the surface shows with its first drawing tenant, hides with its last |
| `reposition`/`defer_reposition` | never reached (`Live` only) |
| window bounds cutting at the working area | `cut`: clip also ∩ viewport, for the window and its copies |
| `cover_window` / `keep_above` | none (never needed: no companion) |
| `adopt_companion`, `refresh_companion` | none: a hosted one draws the window as it is |
| `raise_thumbnail` (backdrop carried) | re-registers on the surface, held while hidden |
| `hide_thumbnail` | thumbnail hidden, tenant still drawing (ring, backdrop copies) |
| `hwnd()` (overlay anchors: `final_overlay_rects`, pinned-ring fallback) | the surface's |
| `Drop` | copies and thumbnail unregistered, tenant count down; the window stays |

### Ownership and purge

`PeekView.switch_surfaces`, one per working area (`switch_surface`),
apart from the tiles' `surfaces`, which span the monitor (`tile_clips`)
and which a tile motion raises to the top. Two reasons: the Explorer
fullscreen rule above, and the switch surface must stay below every
incoming window from its uncloak to `Drop` (backdrop copies under their
real windows). Tenants hold the surface's window (`Rc`), so it outlives
them whatever drops first. Purged with the tiles': `drop_surfaces`
(reload, pause, shutdown), display change (`retain_switch_surfaces`, by
working area), and unused ones off every working area as one is made
(`is_stale_surface`).

### Who goes on it

`hosts_switch_window`: `onesurface`, a tile (`WindowState::Tiling`) whose
backdrop is copyable (not `Live`), and `hosted` accepts it (not topmost,
no companion at the start). Everything else keeps its own window:
floating and fullscreen windows (they overlap other switch windows; a
surface below all switch tiles would put a floating window under a tile
for the gap between their cloaks, or uncloaks, a frame now and then),
`Live`, topmost-band windows (step 6's top surface), companions.

### Z-order and stacking

- The surface is never raised: each tenant's `admit` puts it below its
  window, so it ends below the lowest switch tile. Outgoing tiles warm
  under their still-shown windows, then are cloaked over it; incoming are
  uncloaked over it; own-window surrogates (floating) sit just below
  their windows, above it.
- Among tenants: registration order. Each tile's rings are carried in
  `park_switch_borders`, after every window's thumbnail. So right after
  park, before anything is shown, `restack_switch_tenants` registers anew
  (`register_anew`, held) every hosted surrogate drawn under a lower one
  it may meet, lower and higher by the real windows' z-order
  (`stacking_ranks`, one `GetWindow` walk), as step 3's `out_of_order`.
  May meet (`switch_tenants_meet`, tested): same side, where their rest
  frames with rings overlap; across sides by `Crossing`: unzoomed styles
  (drift, fade, zoom, cut) a fixed offset apart, zoomed-down slides never
  (panels a monitor apart), soft zoom anywhere. A plain slide restacks
  nothing and walks nothing; a fade restacks the upper side's tiles that
  overlap the other side (stage `restack`).
- Not restacked: registrations after the start (none for hosted ones:
  no companion, the ring's source and cut stay), and a cut-short
  switch's tenants against the new one's (new ones register later, so
  over them, as `hand_over_cut_short_slide` needs).

### Hand-over steps

Untouched and in place: `show_initial`/`cover_ring`, the single flush,
`conceal`, `hand_over_cut_short_slide` (new one full before the old one
hides, both on one surface: registered later, so over it),
`take_over_outgoing`/`take_over_from_peek`, per-frame `restore_alpha`,
`hide_thumbnail`/`hide_ring` at `ws_done`, `stash_for_ws_cleanup`,
`prepare_ws_hand_back`, `hide_pending_ws_cleanup_surrogate`,
`hand_back_to_peek`, `WsCleanupStep` (overlays shown behind the surface,
then behind their windows, then `Drop`). The restack is a new step
between park and show, on held thumbnails only.

### Edge cases

- Second switch mid-slide, switch back mid-slide: both switches' tenants
  on one surface; `hand_over_cut_short_slide` unchanged.
- The other monitor: its own working area, its own surface; a playing
  switch finished by `start_workspace_switch` hands back on its surface.
- No windows on a side: nothing changes; the surface shows with the first
  shown tenant.
- A window closing or opening during the slide: as today; a closed
  window's thumbnail draws nothing until `Drop`.
- Fullscreen: own window. Semi-transparent: alpha take-over unchanged
  (z-order independent). Suppressed: no pre-pass, no surface.
- Two DPIs: one surface per working area, physical pixels throughout.
- Resting peeks (`slidepeeks`): own windows just below their columns'
  windows, so above or below the surface; faint either way, and at the
  end peek and surrogate are identical while both draw.

### Known differences on screen

- An own-window surrogate (floating, `Live`, topmost) and hosted tiles
  stack by the surface's height, not the real z-order: a floating window
  below a tile it overlaps ends above it; a `Live` tile above the other
  side's tiles in a crossfade.
- Unmanaged windows between switch tiles in the z-order end above all
  hosted tiles.
- A companion first seen after the start is not adopted by a hosted
  surrogate: the window slides uncolored and its companion comes back on
  its own after the uncloak.
- Tiles' surfaces still span the monitor: a tile motion may hide app bars
  that make way for fullscreen apps (not seen; option: tile surfaces on
  the working area too, one surface per monitor again).

### Windows functions called by the new code

All already called by `wm-platform`: `GetWindow` (`z_depths`, `admit`),
`GetWindowLongPtrW` (`is_topmost`), `EnumWindows`, `GetPropW` through
`Companion::find`, `SetWindowPos`, `ShowWindow`, `CreateWindowExW` (a
surface), `DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`,
`DwmUnregisterThumbnail`, `GetWindowRect`/`DwmGetWindowAttribute`
through `compute_border_inset`.

### To check on screen (`GLAZEWM_ISOLATE=onesurface`)

- Switch 1→2 and back, each style used (bench: slide): no blank,
  doubled or ring-less frame at start or end; one extra window of the
  surrogate class per monitor, spanning the working area, hidden after.
- A second switch mid-slide, and back mid-slide: nothing left behind.
- A workspace with a floating window over tiles: it stays over them all
  the way, on its own surrogate.
- Fade: which workspace draws on top matches `default`; `restack` shows in the profiler
  at the start, never for a slide.
- A scrolling column half off the inner edge of a monitor
  (`slidepeeks`): slides with its neighbours, cut at the edge, no flash
  at the hand-over to its peek.
- An app bar set to hide for fullscreen apps: stays through switches.
- Two monitors: a switch on each, nothing across the edge; then a
  display change and a switch.
- A shades-recolored window: still on its own surrogate, colors kept.

## Integration: as built

Branch `os/9-integrate` (from `tryout/one-surface`, `0df1473`). A
reading-and-fixing pass over steps 3, 4, 5 and the workspace switches
together. Not seen running.

### Three surfaces per monitor (reviewer's decision)

| surface | spans | height |
|---|---|---|
| tiles' (`surfaces`, `tile_clips`) | monitor rect | raised to the top of the ordinary windows for a motion |
| columns' (`surfaces`, `displayed_scrolling_workspaces`) | working area | just above its resting tenant's cloaked window at rest (`rest_above`), above a landing window (`keep_above`) |
| switch's (`switch_surfaces`) | working area | below every incoming window until `WsCleanupStep::Drop` (`admit`, never raised) |

Why apart:

- Columns' and switch's are not merged: their heights conflict. A column
  landing mid-switch lifts its surface above the landing window
  (`stay_over_window`); with switch tenants on it, they would go over
  the uncloaked incoming windows they must stay under, backdrop copies
  included, until `Drop`. A resting column's surface sits just above a
  cloaked window wherever that is in the z-order, which is not "below
  every incoming window" either.
- Tiles' keep the monitor rect: an own pinned peek of a tile spans the
  same rect today, so nothing new for Explorer's fullscreen rule (still
  unchecked on screen, see "Known differences" of the switches).
- A column surface equal to a tile surface (working area == monitor
  rect, e.g. an auto-hidden taskbar) is one surface, found by clip.
  Checked: a tile's raise lifts a resting column on it too (step 5's
  known difference: "drawn at the surface's height" while anything moves
  on it); `rest_anchor` gives no anchor while any tile tenant is there,
  `Releasing` ones included, so `settle_resting_surfaces` never takes the
  surface down under a landing tile; `raise_to_top` clears `rests_above`,
  so the next settle puts it back; a resting tenant holds the surface's
  place (`hold_place`), so a tile joining never moves it down
  (`admission`), and `joining_tenants` leaves the resting one out. One
  gap fixed (below): a tile dropped outside a tick left the surface
  raised.

### Purge points

One rule now, `is_stale_surface`: a surface goes only once unused (no
tenant, no ready peek) and off the clips kept; ready peeks always first.
So tenants always go before their surface, structurally, and no surface
in use is ever off its list (before: dropped from the lists while still
drawing, held alive by `Rc`, but then out of reach of raise passes,
restacks and settling, moved on in place by a landing tile's next
motion, and a second surface could be made at the same clip).

| point | order |
|---|---|
| display change (`handle_display_settings_changed`) | ready peeks cut at no monitor rect or working area; tile/column surfaces unused off those (`retain_surfaces`, clips from `surface_clips`); switch surfaces unused off every working area (`retain_switch_surfaces`). A tenant on a kept surface whose clip changed is replaced at its next motion or upsert (clip compare in `move_tile`, `moves_on_in_place`); a moving tile retargeted keeps it to its landing; the old surface goes, unused, as one is made or at the next purge |
| reload (`reload_config`), pause (`toggle_pause`) | `drop_unused_surfaces`: every ready peek, then every unused tile, column and switch surface. Moving and leaving tenants play on (ticks run while paused) and keep theirs; a playing switch keeps its own to its end, cut short or not; a resting column (`noslidepeeks`) keeps drawing on its surface, which stays listed, so the redraw after an unpause moves it on in place |
| shutdown (`drain_all_sessions`) | the switch and `pending_ws_cleanup` (their tenants) first, leaving peeks, then `drop_unused_surfaces`; entries left drop with `PeekView`, declared before `ready` and the surface lists |
| `drop_replaced` | after `sync_scroll_view`'s flush, the same sync as the `upsert` that filled it, so `replaced` is empty between syncs and at every purge |
| `switching_out`, `standing_by` | own peeks only (`slidepeeks` hosts no resting column, `keeps_rest_hosted`); `switching_out` cleared by `settle_switch_peeks` the same sync |
| ready store | dropped before surfaces everywhere above; strays (`drop_stray_ready`) at every sync before the switch pre-pass; per window on unmanage, leave, enter, a column upsert, a `move_tile` that doesn't take it |

`surface_clips` (monitor rects and working areas) is now also what a
surface made for a tile prunes by: it used monitor rects only, so it
dropped unused column surfaces (a `CreateWindowExW` again at the next
scroll). Without `onesurface` no surface or ready peek exists, so none
of this runs.

### Cross-interactions

- Switch starting with tiles on a surface: `drop_strays` drops the
  outgoing workspace's moving, landing and entering tiles' peeks before
  the pre-pass, as for own peeks; the surface hides with its last drawing
  tenant (`note_drawing`), `release` lets it go down again. Fine as is.
  Ready peeks of switched-out windows go in the same sync
  (`drop_stray_ready`); switched-in windows have none left (dropped when
  they went out, or expired). Fine as is.
- The same, with a resting column (`noslidepeeks`) of the incoming
  workspace on a surface shared with an outgoing tile: the column is
  upserted (and admitted, cloaked, where the tile's raise put the
  surface) before `drop_strays` drops the tile, and nothing settled the
  surface after: the column stayed over every window until the next tick
  or scroll. **Fixed**: `take_strays`, `forget_entering` and
  `forget_leaving` settle resting surfaces when they drop an entry.
- Resting columns at a switch, `slidepeeks` (default): never hosted
  (`keeps_rest_hosted`), so `stand_by`, `keep_switching_out`,
  `take_over_from_peek` and `hand_back_to_peek` only meet own peeks. A
  hosted column turning to rest is replaced by an own peek; `stand_by`
  runs before `upsert`, so the old hosted one and the new own one are
  both faint until the flush. Fine as is (open item 2).
- Resting columns at a switch, `noslidepeeks`: an outgoing hosted one
  departs (`take_departed`) and drops after the scroll view's flush, as
  an own one does; an incoming one rests at once, as an own one does
  with `noslidepeeks`, just above its cloaked window instead of just
  below it (step 5). Fine as is.
- Moving columns at a switch: an outgoing one departs (kept only at
  rest) and drops after the flush; the switch's surrogate takes the
  window as for an own peek. Fine as is.
- Tile or column motion while a switch's surface is up, same monitor:
  the switch's windows have another stand-in (`has_other_stand_in`, by
  `has_active_surrogate`, which includes `pending_ws_cleanup`), so their
  moves stay with the switch. A window opening meanwhile enters on the
  tiles' surface, raised to the top, over the switch's surface, as its
  own peek would be raised. A column landing lifts the columns' surface,
  never the switch's. Other monitor: other clips, other surfaces. Fine
  as is.
- Cut-short switches, switch back mid-slide: both switches' tenants on
  one switch surface, since a purge no longer drops it while in use
  (before: a reload or pause mid-switch, then a second switch, put the
  new tenants on a second surface, ordered against the first by where
  each was admitted rather than by registration). Fixed by the purge
  rule.
- Overlay anchors: `WorkspaceSurrogate::hwnd()` gives the switch surface,
  `PeekView::peek_hwnd`/`stand_in` the tile or column surface; a window
  never has both a hosted peek and a hosted switch surrogate drawing it.
  Fine as is.
- Shared helpers, checked to agree: `admit`'s `is_shown` (below);
  `is_unused` counts tenants and ready peeks, and switch surfaces have no
  ready peeks; tenant counts go up in `create_hosted`/`rejoin`, down in
  `Drop`/`retire`, the same for both kinds; `set_raise_pass` and
  `note_raised` reach `surfaces` only, and switch surfaces are never
  raised (`WorkspaceSurrogate` calls no `raise_*`, `hold_place` or
  `lift_surface_above` on its inner surrogate), and their `admit` only
  takes them down to below a window, never over a raised tile surface;
  `restack_crossings` groups entries by surface, `restack_switch_tenants`
  switch tenants only; `is_stale_surface` now prunes both lists, by
  `surface_clips` and by working areas.

### `1baeb71`: kept

`WorkspaceSurrogate::hosted` admits as below a shown window
(`create_hosted(..., true)`). Since step 4, `is_shown: false` means an
entering window: `admit` holds the surface's place (`raised`) and puts it
just above the cloaked window, where the hand-back needs it over the
window once uncloaked. A switch needs the opposite: its incoming windows
are uncloaked *over* their surrogates (`prepare_hand_back`,
`hide_thumbnail`), backdrop copies under them until `Drop`. With `true`,
an unraised surface goes just below each tenant's window unless already
below it, so it ends below the lowest switch tile, cloaked or shown, as
"Workspace switches: as built" intends, and `raised` is never set on it.

### Still on a window of its own

Unchanged by this pass: see the table in "State and next steps".

### Windows functions called by the new code

All already called by `wm-platform`: `GetWindow` and `SetWindowPos`
(`PeekSurface::rest_above`, through `window_class::place_above`, now
also reached as a tile drops off a surface), `DestroyWindow` (a surface
dropped, as before). No other call is new or newly reached.

### To check on screen

`GLAZEWM_ISOLATE=onesurface`, then `onesurface,readypeeks`. Watch the
surrogate-class windows (one per monitor rect, one per working area for
columns, one per working area for switches): each hidden when nothing
moves.

- Step 3: resize and swap two to six tiles; retarget mid-motion; a
  floating window over moving tiles goes under them and comes back over.
- Step 4: open, close, minimize and restore a tile among moving
  neighbours: no blank or ring-less frame; a closing tile under them.
- Step 5: scroll the strip, scroll again before it lands, open and close
  a column mid-strip; with `noslidepeeks`, a column resting half off the
  edge with a floating window over it.
- Switches: 1→2 and back, each style; a second switch mid-slide, and
  back mid-slide.
- Tiles moving, then a switch (press a resize and the switch back to
  back): nothing left on the old workspace's surface, no stray frame.
- A switch, then a resize on the incoming workspace during the slide
  and during the teardown: the move stays with the switch's stand-in
  (`has_other_stand_in`), no peek of its own or hosted one appears; the
  switch's tiles stay under their uncloaked windows.
- A new window opening on the incoming workspace mid-switch: drawn over
  the sliding tiles, then handed back; the switch surface hidden after.
- Scrolling workspace, `noslidepeeks`, taskbar auto-hidden (one surface
  for tiles and columns): a tiling workspace with a tile moving, switch
  to the scrolling one with a column resting half off the edge and a
  floating window over it: the floating window stays over the column
  once the switch is done.
- The same with the taskbar shown (separate surfaces): the column lands
  mid-switch and the switch's tiles stay under their windows.
- Reload the config mid-slide, then switch again mid-slide: one switch
  surface per monitor (no second one), no tile left faint or doubled.
- Pause mid-resize, unpause: tiles land; a resting column (with
  `noslidepeeks`) stays put, no flash.
- Display change (unplug a monitor, or change the taskbar's edge) mid
  motion and mid switch: what draws finishes on its old surface; the
  next motion and switch draw on the new ones; old ones hidden.
- `readypeeks`: resize the same tile twice within a few seconds, switch
  away and back, resize again: no stale frame of its old position.

## State and next steps (2026-11-15)

For whoever picks this up in a new session. `tryout/one-surface` holds
steps 1 to 5 and the workspace switches, merged; `cargo check
--workspace --tests` and `cargo clippy --workspace --all-targets -- -D
warnings` for `x86_64-pc-windows-msvc` are clean. Tests only type-check
on Linux. Everything is behind `onesurface` (and `readypeeks`), off by
default.

Seen running by the owner (desktop, `onesurface,readypeeks`): steps 1
and 2 only, a few resizes: "nothing weird, no difference noticed". Not
seen running: steps 3, 4, 5, the workspace switches, and how they work
together. Nothing has been benched.

### What draws on a surface, and what still has a window of its own

| kind | on a surface | still its own window |
|---|---|---|
| tile move, resize | yes | stacks; live peeks |
| tile open, close, minimize, restore | yes | floating and fullscreen windows; any tile on a workspace with a stack or a live tile; a tile closed while handing back |
| scrolling column, moving | yes | live; held by a grab; stacks; every column of a workspace with a stack, a live or a held column |
| scrolling column, at rest | one per surface, and only with `noslidepeeks` | all of them by default (`slidepeeks` is on); a second resting column on a surface |
| workspace switch | tiles whose backdrop can be copied | floating, fullscreen, always-on-top, live, windows with a companion |
| grab resize, floating <-> tiling, fullscreen toggle, tile crossing monitors | no | all (step 6) |

### Open, in the order to take them

1. **Integration, before anything new.** The three steps after step 3
   were built side by side and merged without having run together:
   - A monitor can now have three surfaces: the tiles' (monitor rect),
     the columns' (working area) and the switch's (working area, kept
     below its windows). Decide whether the columns' and the switch's
     can be one, and whether the tiles' should span the working area
     too (a monitor-sized window may make Explorer hide app bars; own
     pinned peeks of tiles span the same rect today, so this is not
     new, but it is unchecked).
   - Purge points: `drop_surfaces`, `retain_surfaces`,
     `retain_switch_surfaces`, `drop_replaced`, the ready store. One
     pass over them for order (tenants before surfaces) and for a
     display change, reload, pause and shutdown.
   - A switch while tiles or columns are on a surface; a tile motion
     while a switch's surface is up.
   - The merge fix `1baeb71`: a hosted switch surrogate is admitted as
     below a shown window (`is_shown: true`), so its surface stays
     below incoming windows. Check that against the switch step's
     intent in "Workspace switches: as built".
2. **Resting columns under `slidepeeks`** (the default). Step 5 hosts
   none, so a switch never meets a hosted resting peek. Join the two:
   `stand_by`, `end_stand_by`, `hand_back_to_peek`,
   `take_over_from_peek`, `settle_switch_peeks`, `keep_switching_out`
   for a hosted peek.
3. **Step 6**: grab resizes, floating and fullscreen (a top-band
   surface), tiles crossing monitors.
4. **Stacks**: a copy of the tab bar on the surface (step 3's notes say
   what it needs). Until then one stack keeps its whole workspace's
   opens, closes and columns on their own windows.
5. **Step 8**: duplicated code, bench modes, the measuring section.

### Owner's decisions still open

- An opening tile is drawn under its moving neighbours, as today, at
  the cost of restacking them on every open (step 4). The other way
  (opening tile on top) costs nothing and looks different.
- Whether to bench now (tile moves and resizes: `default` against
  `onesurface` and `onesurface+readypeeks`, see "To measure") before
  building steps 6 and on.

### How the work is run

Fable plans and reviews; Opus agents write the code, one step each, in
a worktree, on a branch `os/<n>-<name>` cut from `tryout/one-surface`;
each appends its "as built" section here; the reviewer merges, runs the
checks and pushes `tryout/one-surface`. Agents that share files run one
after another. Branches merged so far: `os/1-surface`, `os/2-ready`,
`os/3-tiles`, `os/4-transits`, `os/5-scroll`, `os/7-switch`.
