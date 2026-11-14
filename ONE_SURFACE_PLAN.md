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
