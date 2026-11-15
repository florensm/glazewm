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
- Step 6b: floating opens (focused or shown on top) go on surfaces under
  `onesurface` without `morepeeks` too, as they are `peeks` cases today.
  Kept unless the owner wants them behind `morepeeks`.
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

## Resting columns under `slidepeeks`: as built

Branch `os/10-rest-slide` (from `tryout/one-surface`, `2ada205`). Open
item 1 of "State and next steps". Not seen running.

### Today's order, found by reading (own peeks)

Each step keeps this order; only where things sit changes.

| step | where, when | what, and z-order |
|---|---|---|
| `stand_by` | `sync_scroll_view`, start of `platform_sync`, before `upsert` and before the pre-pass | an incoming column at rest (`slides_in`: switch incoming and animated, or `awaits_peek_hand_back`): peek made (or kept) just below its cloaked window, window and copies at warm-up opacity. The pre-pass then makes its surrogate just below the same window, so between the two: surrogate over peek. Under `onesurface` the surrogate is on the switch surface (below the lowest switch tile): over or under the peek |
| `keep_switching_out` | same call, after `take_departed`, before the upserts | an outgoing column at rest: its entry leaves `entries`, kept at full, just below its cloaked window, in `switching_out` |
| `take_over_from_peek` | `redraw_containers`' `Apply` arm, for an outgoing window `is_switching_out` (instead of `take_over_outgoing`) | surrogate (made in the pre-pass, warmed at opacity 1, flushed) to full (`apply_effect_opacity`), then the kept peek dropped, back to back: a frame between shows both |
| `end_stand_by` | after the `Apply` loop (`peek_hand_backs`: displayed, hidden by the layout, incoming of `pending_ws_cleanup`); in `sync_scroll_view` once it no longer slides in; in `settle_switch_peeks` | window to its opacity first, ring copy covered, backdrop copy to full; then `track_overlays` once (ring parked concealed, carried) |
| `hand_back_to_peek` | right after, same ids | surrogate thumbnail and backdrop copy hidden (`HandBack::ToPeek`), ring copy kept until `WsCleanupStep::Drop`, over the peek's identical one |
| `settle_switch_peeks` | after `redraw_containers` | stood-by peeks no switch owes a hand-over drawn again (`end_stand_by`, `track_overlays`); `switching_out` cleared, so none is kept past the sync |

### Hosted

| step | own window | hosted (on its column surface) |
|---|---|---|
| `stand_by` | its window's thumbnail and copies at warm-up | the same, on the tenant's thumbnails only; the surface stays shown and where it is (just above the column's cloaked window, by `admit` and `settle_resting_surfaces`), other tenants untouched |
| `keep_switching_out` | kept window | kept tenant at full, surface where it was; it counts as the surface's resting tenant until dropped (`rests_on_surface`, `settle_resting_surfaces`) |
| `take_over_from_peek` | surrogate full, then the window destroyed | surrogate full, then the tenant's thumbnails unregistered; the surface hides with its last drawing tenant |
| `end_stand_by` | thumbnail and copies to full | the same, on the tenant; surface unmoved |
| `hand_back_to_peek` | acts on the surrogate | the same |
| `settle_switch_peeks` | as above | as above; a kept tenant goes with `switching_out` |

Code: `keeps_rest_hosted` lost its `slide_peeks` exclusion (one resting
column per surface, under `slidepeeks` or not); `rests_on_surface` and
`settle_resting_surfaces` also read `switching_out`. `hosts_column` had
no `slidepeeks` term. `wm-platform` is unchanged: `WindowPeek::stand_by`
and `end_stand_by` only set opacities, hosted or not (a hosted peek's
warm-up already keeps its backdrop copy faint, `warms_backdrop`).

### Z-order through a switch, same monitor

The column surface rests just above its tenant's cloaked window. The
switch surface goes just below each switch tile's window unless already
below it (`admit`, shown), so below the column's window too: **the
column surface is above the switch surface whenever both hold that
column**. The same holds for a column whose switch surrogate is its own
window (a companion): just below the column's window, so below the
column surface.

- Column workspace switching in. Start: the stood-by tenant draws at
  warm-up over the sliding surrogates; faint, so its height does not
  show. End: `end_stand_by` brings the peek to full over its surrogate,
  both at rest, cut at the same working area, then `hand_back_to_peek`
  hides the surrogate: while both draw they draw the same pixels (an
  opaque window looks the same; a semi-transparent one a frame darker,
  as with own windows today). The ring copies (surrogate's under the
  peek's) are identical until `Drop`. Other incoming windows between the
  two surfaces are uncloaked in the same pass; at rest they do not reach
  into the cut column.
- Column workspace switching out. Start: the kept tenant draws at full,
  above the switch surface; its surrogate warms at opacity 1 under it
  (flushed), then `take_over_from_peek` brings the surrogate to full and
  only then drops the tenant: the peek draws until the surrogate has
  taken over, and the two are identical while both draw.
- Own windows, for comparison: the surrogate was over the peek, here
  under it. Same pixels while both draw; neither order shows a blank.

### Edge cases

- **An incoming resting column while an outgoing one is kept on the same
  surface** (two monitors, a cut column on both workspaces): the
  incoming one takes a peek of its own for its life (until it departs).
  Hosted, its `admit` (just above its cloaked window) and
  `settle_resting_surfaces` would move the surface the kept one draws on
  at full, for the rest of the sync, which could put it over a floating
  window that covered it, or under a shown one. So a switch from a
  hosted column alternates: the next switch back finds the surface empty
  and hosts the other one. Own windows draw as today.
- **Mix of hosted and own resting peeks** in one switch (the second
  resting column of a surface, the case above, a stack's or a live
  column): each step goes by window id, so each meets its own kind; the
  two never share a place (different edges, or the case above).
- **Hiding and showing a hosted resting tenant** (stand-by, end of it):
  its thumbnails only; the surface's shown count is unchanged (a
  stood-by tenant is shown, at warm-up), other tenants untouched.
- **Height while one stands by and others move** (the strip scrolls
  mid-slide): movers set the surface's height as in step 5 (admitted
  above cloaked joiners, `stay_over_window` on landing); the stood-by
  one is faint meanwhile. A stood-by column that moves ends its stand-by
  first (`waits_on_switch`), as for own peeks. Once all rest, settled
  over the one resting tenant.
- **Second switch mid-slide, or back**: a stood-by hosted tenant
  departs and is kept (still faint), `hand_over_cut_short_slide` brings
  its new outgoing surrogate up, `take_over_from_peek` drops it; the
  column coming back in meets the case above (own peek) if it rests on
  the same surface.
- **Ready store**: a resting tenant is in use, never ready; a kept one
  is dropped, never released to the store; an incoming stood-by column's
  window is cloaked, so its ready peek is taken and dropped
  (`take_ready`, not reused).
- **Purge points**: a stood-by tenant keeps its surface in use, so
  reload, pause and display change keep it listed (`is_stale_surface`);
  `switching_out` is empty between syncs. Display change mid-switch: the
  tenant is replaced at its next upsert (clip changed), the new one stood
  by too (`standing_by` is by id), the old one in `replaced` until the
  flush. Pause mid-switch with a stood-by hosted column: the switch plays
  on and hands back as for an own peek. Shutdown: the switch's
  surrogates go first, the tenant with `PeekView`, before its surface.
- **`noslidepeeks`**: `switching_out` is never filled, so admission and
  settling are step 5's exactly. Without `onesurface` no surface exists:
  today's behaviour.

### Still on a window of its own, and why

- A second resting column on one surface (one height at rest), as in
  step 5.
- An incoming resting column whose surface holds an outgoing one kept
  for its surrogate (above).
- As before: live peeks, held columns, stacks, and every column of a
  workspace with one of those.

### Windows functions called by the new code

No new call. Newly reached for resting columns under `slidepeeks`, all
already called by `wm-platform`: `GetWindow` and `SetWindowPos`
(`SurfaceWindow::admit`, `PeekSurface::rest_above` through
`window_class::place_above`, `companion::is_above`), `ShowWindow`
(`note_drawing`), `DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`,
`DwmUnregisterThumbnail`, `CreateWindowExW` (a column surface),
`DestroyWindow`, `DwmGetWindowAttribute` (`is_cloaked`).

### To check on screen

`GLAZEWM_ISOLATE=onesurface`, then `onesurface,readypeeks`. Two
monitors side by side, scrolling layout, `workspace_switch` slide.

- Left monitor, workspace 2 with a column cut at its right edge,
  workspace 1 tiled. 1→2: the column slides in with its neighbours, cut
  at the edge, nothing of it on the right monitor; at the end no flash,
  no double ring, no faint frame. One surrogate-class window for the
  columns (working area) shows the column at rest, none of its own.
- 2→1: the column slides out with the rest from the first frame (no
  blank, no doubled frame at the start); the column surface hidden after.
- Workspaces 1 and 2 both with a cut column on the left monitor: 1→2,
  2→1, 1→2. Each switch slides both columns; every other switch the
  incoming one is on a peek of its own (an extra surrogate-class window
  just below its window): no difference on screen.
- 1→2 then back mid-slide, 1→2 then 2→3 mid-slide, fast presses: no
  column left faint, blank or doubled.
- Semi-transparent window and one with a copyable backdrop as the cut
  column: at most one frame darker at the hand-over, as with `default`.
- A floating window over the cut column on workspace 2: 1→2, it stays
  over the column once landed; 2→1, it never shows under the column.
- Three monitors, columns cut at both edges of the middle one: one on
  the surface, the other on its own window; switches slide both.
- `fade`, `zoom`; `workspace_switch.enabled: false`; overview pick: the
  column is shown at rest at once, as today.
- Focus a neighbour mid-slide so the strip scrolls: the column ends
  where the layout puts it.
- Pause mid-switch, then unpause; reload mid-switch; unplug a monitor
  mid-switch: nothing left faint, the next switch draws.
- With `readypeeks`: resize a column, switch away and back within a few
  seconds: no stale frame.
- `onesurface,noslidepeeks`: as step 5 (the column shows at rest at
  once during a switch).

## Step 6a: as built

Branch `os/6a-held` (from `tryout/one-surface`, `aee1185`). Grab resizes
only (`hold_on_peek`, held columns); 6b (floating <-> tiling, fullscreen,
top band) and 6c (tiles crossing monitors) not started. Not seen running.

### Today's order, found by reading (own peeks)

| what | where, when |
|---|---|
| start | `grab::step` → `apply_resize` → `place_moved_windows`: a window `holds_on_peek` wants and no peek holds yet is refused by `is_placeable` and queued for redraw. The redraw (`held_from`, ahead of the animation gate) calls `hold_on_peek` → `move_tile` (duration 0, `Fill`, then `hold(id, true)`); pushed to `peek_moves`, frozen; the pass's `cover_moved_overlays`, `commit_pending_cloaks` (one flush, `take_over`, cloak), `hand_over_moved_overlays` take over as for a move |
| columns | `sync_scroll_view` asks `holds_on_peek` per column; `upsert(held)`: `Role::Moving`, duration 0, filled, no transit; a resting one becomes held with no take-over |
| z-order | a new held peek goes up just below its window (warmed). The raise loop skips held ids, so it stays there for the whole grab, under any floating window over its tile. Held tiles stack among themselves as their windows do (focus order); a tile already landing on its own peek is moved on in place, where `begin_hand_back` put it (just above its window). Columns: just below their windows, never raised |
| each step | `place_moved_windows`: the placer resizes the cloaked window; `retarget_tile` (duration 0) moves the target. Drawn on the next tick (`tick` → `Entry::draw` → `place_filled`): thumbnails only, the peek's window never moves (pinned) |
| neighbours | every tile and column the grab changes is held if `can_hold`; one that can't (stack, live, off its monitor, another stand-in) is redrawn with animations suppressed: it snaps. A tile still moving on a peek is retargeted on its timing (frozen), and held at the first step after it landed |
| end | `finish`: placer's last rects, `release_holds` (`Hold::Released`, 150 ms or its size read), redraw of every window. `tick`: `may_land` → `begin_hand_back` (own: just above its window; a column's hosted surface: `stay_over_window`) → `Apply` (`prepare_hand_back`, synchronous `SetWindowPos`, uncloak, `release`) → `Releasing` → `hand_back` → released (ready store under `readypeeks`) |
| guards | `window_at` (resting `Role::Peek` rects) and `moving_rects` (`visible_rect`) read the same for a hosted peek; the grab owns the mouse meanwhile. `is_grabbable` refuses a window a peek owns, so no grab starts on a moving one |
| live peeks | `can_hold` refuses them: they snap, own (unchanged) |
| mid-grab | switch: `sync_listener` ends the grab when its workspace is no longer displayed, `drop_strays` takes the peeks. Pause: listener unhooked, `end_early`. Reload, display change: the grab goes on; a held peek keeps its clip until it lands (`retarget_tile` doesn't repin) |

### Hosted

| step | own window | hosted |
|---|---|---|
| `move_tile` from `hold_on_peek` | peek made just below its window | registered on the monitor's tile surface (`tenancy`, `Stacking::Raised` rank); the surface admitted by the shown window (`SurfaceWindow::admit`): not raised, it goes just below the lowest held window; raised, never down. A landing hosted tile on the same surface is moved on in place; a ready one there is reclaimed |
| `hold(id, true)` | `Hold::Held` | the same, and `Tenancy::never_raised` |
| take-over (`cover_moved_overlays`, flush, `take_over`, cloak, `hand_over_moved_overlays`) | as is | as is; `restack_joining_overlays` puts shown overlays of tenants on an unraised surface behind it (step 3) |
| raise loop | skipped (held) | skipped: the surface stays where admission put it |
| each step's `retarget_tile` | target moved | the same, lane anew, `restack_crossings` (pure unless out of order) |
| tick draw | thumbnails | thumbnails on the surface |
| `begin_hand_back` | peek just above its window | nothing (`raise_above` is for own windows), then `stay_over_window` (new for held tiles, as columns since step 5): the surface just above the landing window unless above it already, never down after |
| `Apply`, `hand_back`, release | as is | as is; tenant's thumbnails go, or kept ready |
| column `upsert(held)` | just below its window | on the column surface as any moving column (step 5): admitted, moved on in place if hosted there, else replaced via `replaced`/`drop_replaced` |

### Stacking

- Held tiles among themselves: ranks are given in the redraw's order,
  the same order their own peeks went up in. Tiles don't overlap; only
  rings meeting in a gap can, and a late registration there (ring cells
  in the cover pass, fill strips on the first ticks) restacks the upper
  tile once (`out_of_order`, step 3), not per frame: per step only lanes
  are recomputed. No re-registration per frame.
- Against other windows: one surface height, below the lowest held
  window, so floating windows over the tiles stay over them, as today.
- A surface already at the top of its band (`PeekSurface::is_topped`,
  new: set by `raise_to_top`/`raise_to_top_of_band`, cleared when empty
  or put at rest) is above every floating window until its last tenant
  goes, which a grab's tenants would keep it at: a tile held then keeps
  a peek of its own (`hosts_hold`, tested). That is a grab started while
  a neighbour still lands from a command.
- A tile entering (open, restore) beside a held one keeps a peek of its
  own (`hosts_transit`'s `entering`): its raise would top the surface,
  putting the held tiles over floating windows. As today, its own raised
  peek goes over the held ones. A leaving tile is never raised, so stays
  hosted, ranked below.
- Columns: column surfaces are not raised (a restored column aside), so
  no such check; a held column draws at its surface's height, step 5's
  rules for moving columns.

### Exclusions lifted

- `hosts_column` lost `held`; `sync_scroll_view`'s `beside_own_peek` no
  longer counts held columns. Held columns are hosted, so nothing of the
  workspace is on an own peek because of them. A column whose entry was
  own already (a second resting column) stays own for its life, as in
  step 5, beside hosted ones.
- Tiles had no such exclusion: `has_own_peek_neighbour` counts stacks and
  live peeks only.

### Edge cases

- Grab start on a tile moving hosted: not held until it has landed
  (unchanged); then moved on in place on its surface. Landing on another
  clip (display change): a new peek, the old one dropped at once, as for
  every tile move since step 3. On a topped surface: an own peek
  replaces the hosted landing one at once, as every hold did before this
  step.
- Ready store: a held tile's peek released after landing shown is kept
  ready (hosted, `keeps_ready`), its fill and ring cells with it; a hold
  takes it only on its surface, otherwise it goes (`take_ready`). Purge
  points unchanged: a held tenant keeps its surface in use, so reload,
  pause and display change keep it (`is_stale_surface`).
- Several held tiles land apart: each lifts the surface just above its
  window if needed (`keep_above`), only ever up.
- Window closed mid-grab: `drop_strays`, its thumbnails go; the surface
  hides with its last drawing tenant.
- Without `onesurface`: `hosts_hold` false, empty `monitor_clips`, as the
  call had; no surface exists, `never_raised` never read. Today's
  behaviour.

### Known differences on screen

- A window between two tiled windows in the z-order (an unmanaged one)
  covers a held tile whose window is above it: the surface is below the
  lowest held window. Its own peek was just below its own window.
- A neighbour's real ring above the surface may cover a held tile's ring
  copy where the two meet in a gap.
- A column restored mid-grab tops its column surface (`raise_moving`),
  putting held columns there over floating windows until it empties.

### Still on a window of its own, and why

- Held tiles while their surface is topped (above); tiles entering beside
  a held tile (above).
- Live peeks, stacks: not held at all (`can_hold`).
- 6b and 6c: floating <-> tiling, fullscreen toggles, tiles crossing
  monitors.

### Windows functions called by the new code

None new. Newly reached for held peeks, all already called by
`wm-platform`: `SetWindowPos` and `GetWindow` (`SurfaceWindow::admit`,
`keep_above` through `window_class::place_above` and
`companion::is_above`), `ShowWindow` (`note_drawing`),
`CreateWindowExW` and `DestroyWindow` (a surface), `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail`,
`DwmGetWindowAttribute` (`is_cloaked`). `is_topped` reads a flag.

### To check on screen

`GLAZEWM_ISOLATE=onesurface`, then `onesurface,readypeeks`, with
`grab_and_move.draw: "peeks"`, a border and a gap wide enough to see.

- Two, then three tiles: drag an edge slowly, fast, back and forth. One
  surrogate-class window for the monitor, no peek of its own per tile;
  rings and gaps solid; no blank, backdrop-only or ring-less frame at the
  start; the surface hidden after the release.
- A floating window over a tile being resized: stays over it all the
  drag, and at the release no flash of the window over its peek.
- Release: nothing moves, blinks or changes size; Escape mid-drag lands
  back where it started.
- Resize by keyboard, then grab a neighbour's edge within the animation:
  the held tiles on peeks of their own (an extra surrogate-class window
  each), floating windows still over them.
- Close a neighbour mid-drag (keybinding): it closes under the held
  tiles; open a window without focus mid-drag: it enters on its own
  peek over the held tiles.
- Scrolling layout: drag a column's edge so neighbours slide, past the
  screen's edge, a column partly off screen: one column surface, no peek
  of its own per column; columns cut at the edge stay cut.
- Two monitors: grab on each in turn, each surface on its own monitor.
- With `readypeeks`: resize by grab, then by keyboard within 5 s: no
  stale frame of the grab's last size.

## Step 6b: as built

Branch `os/6b-band` (from `tryout/one-surface`, `5934e41`). Floating <->
tiling, WM fullscreen toggles, and floating opens (a transit kept off
surfaces only for its band, step 4). Tiles crossing monitors (6c) are
untouched. Not seen running.

### Today's order, found by reading (own peeks)

All pinned, clipped to the monitor rect (`tile_clips`,
`fullscreen_on_peek`, `enter_on_peek`), created just below the real
window (in its band), warmed, taken over at the pass's flush. Then the
raise loop (`redraw_containers`): `raise_moving` for `peek_moves` not in
`band_moves`, the pass closed, then `peek_raises` in loop order, each
setting the cloaked window's band first (`set_z_order`), then
`raise_to_top_of_band` on its peek. Landing: `begin_hand_back`, peek just
above its window.

| case | `raise_moving` | `peek_raises` | window's band before / after |
|---|---|---|---|
| tiling -> floating, shown on top | no (`band_moves`) | `TopMost` | normal / topmost, set in the raise loop |
| tiling -> floating, focused | no | `Top` | normal / normal, not moved |
| tiling -> floating, unfocused | no | none: stays just below its window | normal / normal |
| floating -> tiling | no | `Top`, window set `Normal` first | topmost or normal / normal |
| fullscreen on or off, not on top | yes | none | normal / normal |
| fullscreen on or off, on top at either end | yes | `TopMost`, window set `TopMost` | either / topmost until the hand-back's `reposition_window` |
| floating open, shown on top | no (entering) | `TopMost` | topmost |
| floating open, focused | no | `Top` | normal |
| floating open, unfocused | no | none | normal |
| floating close, minimize | never raised | none | its own |

Neighbours: the tiles a crossing or toggle moves are raised by
`raise_moving` in the same pass, so a band raise ends over them. A
window never changes band mid-motion: its band is set once, in the
raise loop of the pass that cloaks it; a second toggle mid-motion
retargets (`follows_tile`, frozen), no new raise.

### Surface decision

Per monitor rect, a second `PeekSurface` in `band_surfaces`
(`Layer::TopBand`), next to the tiles' one (`surfaces`,
`Layer::Tiles`). Which one a peek takes follows its raise
(`host_for`, tested):

- `TopMost` raise: the band surface, raised to the top of the
  always-on-top band as the own peek was.
- `Top` raise: the tiles' surface, ranked over the pass's raised tiles
  (`Stacking::OverPass`, rank pass + 2: as first put on top in the next
  pass), the surface raised to the top as the own peek was.
- No band raise, a tile (fullscreen toggle not on top): the tiles'
  surface, ranked as a move.
- No band raise, not a tile (unfocused floating crossing or open,
  floating close or minimize): own peek. It stays just below its own
  window, under floating windows above it; a surface raised for other
  tenants can't.

Clip: the monitor rect, not the working area. A fullscreen window
covers the taskbar, and a floating one may overlap it; today's own peeks
of these windows span the monitor rect already, topmost for the
on-top ones, so Explorer's fullscreen-app rule sees nothing new (still
unchecked on screen, as for the tiles' surface).

One surface for the whole motion: the band is fixed at the pass's raise
loop, before anything after the take-over is composited, and kept to the
landing. Between the flush and the raise the own peek sat just below its
window in the window's old band; hosted, the surface is where admission
put it (a shown window: just below it, or stays if raised), and the
cloaked window shows nothing either way; the raise follows in the same
synchronous loop. A landing peek moved on by a motion on another surface
(e.g. floating on top -> tiling while landing) is replaced: the new one
drawn, the old one kept drawing in `replaced_tiles` until the next tick
(`tick` clears it after `note_composited`), so a frame may show both,
never neither. This also covers step 3's clip change, which dropped the
old one at once.

### Hosted

| step | own window | hosted |
|---|---|---|
| `move_on_peek` (crossing), `fullscreen_on_peek`, `enter_on_peek` (floating) | peek made just below its window | `host_for` picks the surface; `tenancy` registers on it (`SurfaceWindow::admit`: shown window, unraised surface goes just below it, raised stays; cloaked, just above, held) |
| take-over (cover, flush, `take_over`, cloak, hand-over) | as is | as is; `joining_tenants` now walks band surfaces too |
| `raise_moving` (fullscreen) | peek to top of its band | surface to top of its band (`HWND_TOP`), as before |
| `set_z_order` of the window (`peek_raises`) | as is | as is, unchanged order |
| `raise_in_band` | peek to the band's top | surface to the band's top (`SurfaceWindow::raise_to_top_of_band`), each raise in turn, as a later window's band change can put it above the surface |
| retarget, ticks | thumbnails | thumbnails on the surface, lanes anew, `restack_crossings` |
| `begin_hand_back` | peek just above its window | nothing, then `stay_over_window` (`Tenancy::in_band`): the surface just above the window unless above it already, never down |
| `Apply`, `hand_back`, release | as is | as is; ready store as any hosted peek |

No hand-over step reordered or dropped (rule 5): the raise loop's order,
`set_z_order` before the raise, the flush and the hand-back's z-order rule
are as they were.

### Stacking

- Band surface vs tiles' surface: topmost over non-topmost, as `TopMost`
  own peeks were over `Top` ones.
- Inside each, registration order; ranks: a pass's band-raised tenants
  over its raised tiles, in loop order (the order of `peek_raises`), under
  the next pass's. Late parts (ring copies in the cover pass, fills)
  restack the upper tenant once (`out_of_order`, grouped per surface);
  nothing re-registers per frame.
- Beside a stack or live tile: only `Host::Tiles` transits keep their own
  peek; a band-raised one ends over every peek its pass raised anyway.
- Beside a held tile: a `Top` crossing or entering on the tiles' surface
  keeps its own peek (its raise would top the surface over the held
  tiles); the band surface is free to take one.

### Ownership and purge

`band_surfaces` declared after `surfaces`, before `switch_surfaces`;
entries, `replaced`, `replaced_tiles` and `ready` before both, so
tenants go first. Same rule as integration's (`is_stale_surface`): a
surface goes only unused, ready peeks first. Display change
(`retain_surfaces`), reload, pause, shutdown (`drop_unused_surfaces`) and
a new surface of that layer prune the band list too. A replaced peek is
a tenant until the next tick, so it keeps its surface listed. Ready
peeks: a band tenant's is kept on the band surface; a later motion takes
it only on that surface (`surface.hwnd() == peek.hwnd()`), else it goes.

### Edge cases

- Toggle twice mid-motion: retargeted on the same surface (no new raise).
- Toggle again while landing: in place if the same layer's surface at the
  same clip draws it (`surface_at`), else replaced as above.
- Floating open, closed mid-open: turns round in place on its surface,
  rank kept (step 4's rule).
- Floating open retargeted (app centring itself): jumps, lane anew.
- Into a stack, live, onto another monitor (`Trail`): own, as before.
- Surface creation fails: own peek, as before.
- Two monitors: one band surface per monitor rect.
- Without `onesurface`: `host_for` never reached, no band surface, the
  raises computed from the same values in the same pass (now once,
  before the call, `platform_sync.rs` `state_change_raise`,
  `fullscreen_raise`, `floating_raise`): today's behaviour. Without
  `morepeeks`: crossings and toggles stay off peeks as before; floating
  opens are under `peeks`, as before.

### Still on a window of its own, and why

- Unfocused floating windows not shown on top (crossing to floating, or
  opening): their peek stays below their own window, under floating
  windows above it.
- Floating and fullscreen windows closing or minimizing: never raised,
  same reason.
- Workspace switches' floating, fullscreen and topmost windows
  (`hosts_switch_window`, unchanged): they overlap one another, and a
  switch cloaks and uncloaks them a frame apart; on one surface below
  them (the band's would have to be, since incoming windows are uncloaked
  over their surrogates until `Drop`) the upper of two overlapping ones
  would show under the lower's surrogate for that frame. Tiles don't
  overlap, so they don't meet this.
- Stacks, live peeks, trails (6c).

### Windows functions called by the new code

None new. Newly reached on a band surface, all already called by
`wm-platform`: `SetWindowPos` (with `HWND_TOPMOST`, `HWND_NOTOPMOST`,
`HWND_TOP`, through `window_class::raise_to_top_of_band`, and in
`admit`), `GetWindowLongPtrW` (`is_topmost`), `GetWindow`
(`companion::is_above`, `place_above`), `CreateWindowExW` and
`DestroyWindow` (the surface), `ShowWindow`, `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail`,
`DwmGetWindowAttribute` (`is_cloaked`).

### To check on screen

`GLAZEWM_ISOLATE=onesurface,morepeeks`, then
`onesurface,morepeeks,readypeeks`; `window_resize`, `window_move`,
`window_fullscreen` at `duration_ms: 2000`, then default; a border and a
backdrop; three tiles; a floating window with `shown_on_top: true`.

- `toggle-floating` on the focused tile, `floating.shown_on_top` off:
  one surrogate-class window per monitor in a window list (no extra one),
  the window over the tiles closing the gap all the way.
- The same with `shown_on_top` on: a second surrogate-class window,
  topmost; the window over the other floating window shown on top from
  the first frame; once landed, in that band.
- `toggle-floating` back (from on top and not): sinks into its tile over
  the tiles, once landed under the floating windows shown on top; no
  blink at the hand-back.
- Each twice within the animation, and once more just as it lands: turns
  round; at the landing no frame without the window.
- Unfocused crossing (by IPC on another window): its own peek, as with
  `morepeeks` alone.
- Fullscreen on and off, `shown_on_top` off then on: on the tiles'
  surface, then on the band one; ring gone going in, back coming out; an
  app bar set to hide for fullscreen apps: as with `morepeeks` alone.
- Open a floating dialog, focused, then one shown on top: no own peek;
  over the tiles, resp. over the other window shown on top.
- Reload and pause mid-motion: it lands; band surface hidden after.
- Two monitors: each its own band surface.
- With `readypeeks`: fullscreen on and off within 5 s: no stale frame.

## Step 6c: as built

Branch `os/6c-cross` (from `tryout/one-surface`, `a78d5b2`). Tiles moving
onto another monitor under `onesurface,morepeeks`. All in
`peek_view.rs`. Not seen running.

### Today's order, found by reading (two own peeks, `Trail`)

| what | where, when |
|---|---|
| admission | `move_on_peek` → `tile_clips`: with `morepeeks`, a tile whose `from` centre is on another monitor than its new one gets `(clip, Some(source))` (`move_clips`), unless a peek draws it already (one landing), it is in a stack, or its peek is live; those and any crossing without `morepeeks` return `None`: not on peeks at all, the engine draws them |
| build | `move_tile` makes the own peek on the target monitor (just below the window, warmed); `add_trail` a second own peek cut at the source monitor's rect, same motion and role, warmed, drawn at once. If the trail fails, the entry goes and the move stays off peeks |
| take-over | `take_over`: the trail's `show_window` first, then the own peek's `take_over` (thumbnail full, alpha hidden), then the cloak, in the pass's one flush |
| raise | `raise_moving`: own peek to the top, then the trail. A band move (`band_moves`) skips that; `raise_in_band` raises the own peek only, the trail stays where it went up |
| ticks | every peek of the entry: `restore_alpha`, `show_window`, `refresh_companion`, drawn (place, scaled, stretched, filled), ring and backdrop carried, fill carried, on each, cut at its clip |
| hand-back | only the own peek: at `may_land` the trail is dropped (`entry.trail = None`, its `HiddenAlpha` given back on drop), then `begin_hand_back` on the own peek. Dropped in the tick that last drew it; what it drew at the landing is only what reaches past the tile (a ring at a monitor edge with no gap), which the real ring shows only from the same tick's `track_overlays` |
| retarget back | `retarget_tile`, `lands_on_trail`: the two peeks and clips swap; the trail's becomes the own one and hands back, the old own one (holding the hidden alpha) is the trail |
| retarget elsewhere | onto a third monitor, or wholly onto one: nothing changes; still cut at the two clips, the third shows nothing until the hand-back |
| z-order | each own peek spans its monitor's rect, so the two never overlap; each at the top of its monitor's windows after the raise |
| DPI | physical pixels throughout; each peek's backdrop copy cut from its own monitor's backdrop (`monitor_bounds(viewport)`), the ring copy one thumbnail of the one ring window at its physical size on both; `platform_sync` sends a window landing on another DPI its rect twice (`has_trail`) |
| ready store | the own peek is never hosted, so never kept; `keeps_ready`'s `has_trail` is moot at release (dropped at landing) |
| switch, grab, display change | a switch drops the entry (`drop_strays`), both peeks with it; `is_grabbable` refuses a window a peek owns; a removed monitor leaves its peek spanning a rect off screen to the landing |

### Hosted

| step | own windows | hosted |
|---|---|---|
| `move_on_peek` | `hosts_on_surface(.., has_trail)` kept crossings off surfaces | `has_trail` gone; `move_host`: `host_for` as any move, then `crossing_host` (tested): none if a grab holds a tile on either monitor's tiles' surface (`holds_on_surface`), whose raise would top it |
| ready (`readypeeks`) | none | `take_ready_at` takes the tile's ready peek for the trail only if made for its window at the trail's clip, before `move_tile`, which takes or drops what is left as for any move. So the own part reuses one at its clip, the trail one at its clip (a tile moving back onto the monitor it just landed on), never one elsewhere |
| `move_tile` | own peek | own part on the target monitor's surface of the host's layer, ranked as any move |
| `add_trail` | own trail | with the own part hosted: the trail on the source monitor's surface of the same layer (`surface_index`, made if missing, pruning unused off `surface_clips`), reused or new, sharing the entry's `Tenancy`; then `restack_crossings`. No surface: a trail of its own, as before |
| take-over | as is | as is; `joining_tenants` gives a tile joining two surfaces the lower one (`lowest`, tested), so its overlays go under both |
| `raise_moving` | both to the top | each part's surface to the top, in the same order, each noting the others covered (`note_raised`), so a later raise of the first in the pass isn't skipped |
| `raise_in_band` | own peek only | own part's surface, and the hosted trail's, to the top of the band; a trail of its own as before |
| ticks, retargets | as is | as is; `restack_crossings` per surface (`by_surface`, tested), each part by the tile's rank and lane |
| retarget back | swap | swap; parts keep their surfaces, the tenancy is the entry's |
| landing | trail dropped | hosted trail (`keeps_trail_drawing`, tested): alpha given back as the drop did, then kept in `replaced_tiles` to the next tick; own trail dropped as before |
| hand-back, release | own peek | own part only; kept ready as any hosted peek, the trail never |

No hand-over step reordered or dropped (rule 5): the take-over's order,
the raise loop, `begin_hand_back`, `Apply`, the alpha's return at the
landing are where they were.

### Two tenancies for one window

- `entries` stays keyed by window. A part is `(window, Part::Own |
  Part::Trail)`; `Entry::hosted_parts` gives each with its surface.
  The own part is hosted while the entry has a tenancy; the trail only
  then, and only if made on a surface.
- One `Tenancy`: rank and lane are the tile's, given once per command.
  The lane is in screen coordinates; on the other surface it reaches
  past the clip, so `meets` may restack where nothing can overlap
  (conservative, one registration at most).
- Restacks per surface: `restack_crossings` groups parts by surface, in
  rank order, and registers anew the part out of order
  (`Entry::part_mut`).
- Every place that asked "is this entry on that surface" now asks
  `Entry::is_on`: `settle_resting_surfaces` (a crossing trail on a
  surface shared with a resting column keeps it from going down),
  `holds_on_surface`.

### Stacking

- On each monitor, the part ranks among that surface's tenants as the
  tile: over the tiles its pass raised before it, under later ones.
- Surfaces: one per monitor rect and layer, so the two parts are on two
  windows that never overlap; their relative z-order doesn't show.
- Band-raised crossings (floating <-> tiling ending on another monitor):
  both parts on the band layer's surfaces, both raised in the band.
  Today the own trail is never raised for a band move; hosted, the
  source part follows its surface, as the own part does (judgement:
  the departing part under floating windows on one monitor and over
  them on the other was not a choice anyone made).

### Edge cases

- Landing: the trail outlives it by one tick, so a ring sliver it draws
  past the monitor edge is still there when the real ring, revealed in
  the landing tick's `track_overlays`, is composited: a frame may show
  both, never neither. Its alpha goes back at the landing, as the drop
  did, so `prepare_hand_back` (under `latealpha` after a swap) sets
  aside the real alpha, not a 0.
- `PeekView::is_active` now counts `replaced_tiles`, so a tick comes to
  drop one even when its entry is gone (a switch right after the
  landing); also covers 6b's replaced landing peeks.
- Retarget back onto the source monitor: the parts swap, each on its
  surface; the new own part hands back there.
- Retarget onto a third monitor, or wholly onto one: as today (cut at
  the two clips; nothing drawn on a third). A third part is not built.
- Switch mid-crossing: `drop_strays` drops the entry, both tenancies;
  each surface hides with its last drawing tenant.
- Grab: none starts on a moving tile. One on either monitor while a part
  is there: its surface is topped, so held tiles keep their own peeks
  (`hosts_hold`). A crossing onto or off a monitor whose tiles' surface
  holds a grab's tile: own, as today (`crossing_host`).
- Display change, source monitor removed: the trail's surface stays
  listed while in use, draws off screen, goes unused a tick after the
  landing and is pruned at the next purge or new surface. Target
  removed: the tile is retargeted onto a remaining monitor; onto the
  source one, the parts swap and it hands back there.
- Two DPIs: as today; each surface spans its monitor rect in physical
  pixels, each part's backdrop copy cut from its own monitor.
- Surface creation fails: for the own part, both own as today; for the
  trail only, the trail is its own window beside a hosted own part
  (each raise and restack looks at the part).
- Without `onesurface`: `move_host` gives `None` (no surface, no ready
  peek), `add_trail` makes the own trail as before, `raise_moving` and
  `raise_in_band` make the same calls in the same order,
  `keeps_trail_drawing(false)` drops it at the landing. Without
  `morepeeks`: no crossing reaches peeks, as before.

### Still on a window of its own, and why

- Live peeks and stacks: no crossing on peeks at all (`tile_clips`): a
  live peek's real backdrop follows one window, a stack's tab bar is cut
  at one monitor.
- A tile still landing from an earlier motion: `tile_clips` doesn't
  cross it (one peek draws it already); the engine draws it.
- A crossing beside a grab's held tile on either monitor (above).
- A trail whose surface couldn't be made.

### Windows functions called by the new code

None new; the new code in `wm` calls no Windows function directly.
Newly reached for a crossing's two parts, all already called by
`wm-platform`: `CreateWindowExW`, `DestroyWindow`,
`DwmSetWindowAttribute` (a surface on the source monitor),
`ShowWindow` (`note_drawing`), `SetWindowPos` (`admit`, `raise_to_top`,
`raise_to_top_of_band`), `GetWindowLongPtrW` (`is_topmost`),
`GetWindow` (`companion::is_above`: `admit`, `is_below` in
`joining_tenants`), `DwmRegisterThumbnail`,
`DwmUpdateThumbnailProperties`, `DwmUnregisterThumbnail`,
`DwmGetWindowAttribute` (`is_cloaked`), `MonitorFromPoint` and
`GetMonitorInfoW` (`monitor_bounds`, as the own trail),
`GetLayeredWindowAttributes` and `SetLayeredWindowAttributes`
(`restore_alpha`, as the drop did).

### To check on screen

`GLAZEWM_ISOLATE=onesurface,morepeeks`, then
`onesurface,morepeeks,readypeeks`; two monitors, ideally of different
DPIs; `window_move` at `duration_ms: 2000`, then default; a border and a
backdrop; a gap of 0 at the shared edge once.

- `move --direction` a tile onto the other monitor: one surrogate-class
  window per monitor (no extra one per tile), the tile cut cleanly at
  the edge, its backdrop on each side its own monitor's; no blank or
  backdrop-only frame at the start; the source monitor's surface hidden
  a tick after the landing.
- The same with tiles moving on both monitors in that command (the
  neighbours closing and opening the gap): the crossing tile over them
  on both sides; a debug line `Stacking ... over a tile it may cross`
  at most once per surface.
- Gap 0 at the edge: at the landing, no frame without the ring's sliver
  on the source monitor.
- Move it back before it lands: it turns round and hands back on the
  source monitor, no blink.
- On different DPIs: the window at its tile's size during the motion
  (the rect sent twice), ring and backdrop the same on both sides.
- `toggle-floating` on a floating window lying across onto another
  monitor, focused, then shown on top: both parts over the floating
  windows (the band surfaces with `shown_on_top`).
- Grab an edge on the target monitor, and with the grab held move a
  window onto it by keyboard: the crossing on its own peeks (two
  surrogate-class windows of its own), floating windows still over the
  held tiles.
- A switch on the source monitor mid-crossing: nothing left on either
  surface.
- Unplug the source monitor mid-crossing: it lands; no stray window.
- With `readypeeks`: move a tile across and back within 5 s: the move
  back draws at once (its trail reuses the peek kept where it landed).

## Stacks: as built

Branch `os/11-stacks` (from `tryout/one-surface`, `6798117`). Open item
1. Not seen running.

### Today's order, found by reading (own peeks)

| what | where, when |
|---|---|
| the bar | `NativeStackTabBar`: a layered popup on a thread of its own (`glazewm-tab-bars`), the WM only posts to it. `sync_tab_bars` (end of every `platform_sync`, and every tick while peeks are active) posts `shown_tab_frame`: the stack's rect moved along with its active window's stand-in (`PeekView::stand_in`: `moved_along(stack, motion target, entry.rect)`), cut at the stand-in's clip, anchored behind the stand-in's hwnd. The bar thread repaints only when the picture changes (`draws_like`: a width change relayouts the tabs), else re-shows the same picture elsewhere; `restack_behind` puts it just behind the anchor and the WM's overlays of it |
| z-order | just behind its window's own peek, so under that peek's window, ring and backdrop copies, over everything below the peek; the peek is raised in the pass, so the bar with it (a tick later, as posted) |
| during a motion | a window that follows the peek, a post (one thread hop) behind the peek's thumbnails; a resize repaints the tabs at each frame's width on the bar thread. Never hidden |
| landing | `stands_in` leaves out `Releasing`, so from the uncloak on the bar goes behind the real window again, at the stack's rect |
| tab switch | no motion: the new active tab is shown and the old one hidden in one pass (shown first, `windows_to_update`'s order); the bar repaints its active tab, anchored behind the new active window. Mid-motion, the moving entry is the old tab's: the bar follows the new active tab (no stand-in), at the stack's rect |
| tab opened or closed mid-motion | the bar repaints with the tab added or gone; an opening window is a transit (`animated_with_tab_bar`: the bar doesn't go along with an open), drawn at the stack's rect behind the entering peek; a closing tab leaves the tree, the next one is active, the bar stays at the stack's rect |
| scrolling layout | a stacked column's peek (moving or resting) is own; its bar follows it, cut at the working area; a column with no peek off the monitor beside it shows no bar (`shown_tab_frame`) |
| workspace switch | every bar of a stack in the switch is hidden for the slide (`is_in_workspace_switch`); `hosts_switch_window` never looked at stacks: their windows were hosted already |
| grab | `can_hold` refuses stacks: they snap |
| crossing monitors | `tile_clips` keeps stacks off peeks: the engine draws them |

### Hosted

The stack's active window is a tenant like any tile; its tab bar is a
thumbnail of the bar's own window on the same surface, the window's
other copies around it (`OverlayCopies::bar`, `BarCopy`).

| step | own peek | hosted |
|---|---|---|
| `move_on_peek`, `fullscreen_on_peek` | own peek | `hosts_on_surface` no longer asks `is_stacked`: on the surface `host_for` picks, ranked as any move |
| `enter_on_peek`, `leave_on_peek`, `restore_on_peek` (`hosts_transit`) | own peek | hosted. No copy: the bar doesn't go along with a transit; it stays a window at the stack's rect, behind the surface (its stand-in's anchor), where no tile of that transit's pass crosses it |
| scroll view `upsert` (`hosts_column`) | own peek | hosted, moving, held or resting, with the copy |
| tenancy (`Reach`) | none | the lane takes in the bar (`Reach::bar`, the bar's `TabBarPlace` at the motion's target), so `meets` sees a tile crossing it |
| take-over pass (`cover_moved_overlays` → `track_overlays` → `carry_tab_bar`) | bar behind the peek | `tab_bar_place` (the bar's outer rect at the stack's layout rect, as offsets from the window's target rect); `WindowPeek::carry_tab_bar` registers the copy, then the window's thumbnail anew over it and the fill and ring over that (each new one up before its old one goes); drawn 1:1 over the still-shown bar (`Handover::Covering`) |
| each tick (`Entry::draw` → `place*` → `OverlayCopies::place`/`place_backdrop`) | bar a post behind | the copy at `TabBarPlace::rect(window rect)`, cut at the clip, in the same thumbnail updates as the window (`bar_rects`) |
| next tick (`note_composited`) | | `Covered`, then `Parked` in `track_overlays`; `sync_tab_bars` → `conceal_copied_bars` cloaks the bar (`NativeStackTabBar::set_cloaked`, on the WM thread). DWM still composes it, the bar thread still paints it at each frame's width, and the copy shows it |
| landing (`HandingBack`, `Releasing`) | bar back behind the window from the uncloak | `Revealed`: uncloaked in the same tick's `sync_tab_bars`, under the copy; the copy goes with the peek two ticks later at least, so a frame may show both, never neither |
| moved on in place while landing | | `Revealed` → `Covering` → `Parked` again |
| `register_anew` (restacks) | | the copy goes with the backdrop group (`backdrop_held_anew`, `put_backdrop_anew`): backdrop, bar, window, fill, ring; `stacking` counts it |
| ready store (`set_aside`) | | the copy is dropped; carried anew by the next motion |
| entry dropped (`drop_strays`, forget, leave) | | the copy goes with the peek; the next `sync_tab_bars` (end of that same `platform_sync`) uncloaks the bar |

No hand-over step is reordered or dropped (rule 5): the bar's steps are
new and run inside `track_overlays` and `sync_tab_bars`, where the ring's
and the bar's already ran; `draw`, `take_over`, `prepare_hand_back`,
`hand_back`, the flush and the raise loop are unchanged.

### Stacking

- Within the tile: backdrop, bar, window, fill, ring. The bar is under the
  window, as its strip reaches under the window's rounded corners. Today
  the bar is behind the backdrop overlay; over it the copy shows the bar
  in the corner notches, which is what the strip is for (only a
  semi-transparent window shows the difference, in that strip).
- Among tenants: the copy is the tile's, registered late (cover pass, or
  a bar shown mid-motion), so `restack_crossings`' `out_of_order` restacks
  as for a ring copy: a stacked tile ranked over a crossing one draws its
  bar over it, which a bar window behind the surface could not.

### Edge cases

- Tab switch mid-motion: the moving entry's window is no longer the
  active tab, so `carry_tab_bar` no longer wants the bar: a parked one is
  revealed (`conceal_copied_bars` looks at the active tab only, so it is
  uncloaked that sync) and its copy dropped a tick later; one not yet
  parked is dropped at once. The bar follows the new active tab.
- Tab opened mid-motion into the moving stack: the bar repaints (one more
  tab) and the copy shows the new picture; the entering window is a
  transit, its own entry carries no copy. Active tab closed mid-motion:
  the entry turns `Leaving` (or the window leaves the stack): revealed
  and dropped as above.
- Stack turning fullscreen: `shows_tab_bar` is false, no place, no copy;
  out of fullscreen the copy is carried once the bar shows.
- Bar hidden mid-motion (a switch, a column out of view): `is_shown`
  false, so the copy is let go as above; a hidden window shows nothing in
  a thumbnail anyway.
- Bar window recreated (stack recreated) mid-motion: the copy is
  registered on the new source as it is carried.
- Resize: the copy samples the bar's window 1:1 at the new size; the bar
  thread repaints at that width a post later, so for a frame the copy
  shows the old picture uncut (growing: a sliver past its right end is
  transparent; shrinking: the last tab cut), where today the bar's window
  itself lands a post late. Never stretched.
- Columns: resting stacked columns are hosted too (one per surface, step
  5's rule), their bar copied and cloaked while at rest; cut at the
  working area as the bar's window is (`bar_rects`).
- Several stacks moving: one copy each; cloaks are per bar, decided by
  each stack's active tab alone.
- Without `onesurface`: no hosted peek, no copy, `conceals_tab_bar` always
  false, `set_cloaked(false)` a no-op: today's behaviour.

### Exclusions lifted and kept

| exclusion | now | why |
|---|---|---|
| `hosts_on_surface`'s `is_stacked` (moves, fullscreen, transits) | lifted | the copy draws the bar with its window; a transit's bar stays put |
| `hosts_column`'s `is_stacked` | lifted | as for tiles |
| `has_own_peek_neighbour` counting stacks (transits beside a stack) | lifted: live tiles only | stacks no longer keep own peeks |
| scroll view's `beside_own_peek` counting stacks | lifted: live columns only | as above |
| workspace switches | not applicable | stacked tiles were hosted already; bars hide for the slide |
| `tile_clips` (crossings) | kept | the bar's window is cut at the own part's clip (`shown_tab_frame`), so a trail's copy would have nothing past the monitor edge to draw. A cloaked bar no longer forces this in principle: drawn uncut while copied, with a copy on each part, it could cross. A step of its own, with the DPI of the two halves to check |
| `can_hold` (grabs) | kept | not in this step; the copy would carry a held stack the same way, with `place_filled` |

### Still on a window of its own, and why

- The tab bar itself: a window always, cloaked while a hosted peek's copy
  shows it. Transits (open into, close out of, minimize, restore a stack)
  draw it as a window behind the surface, at rest.
- A stack crossing monitors (engine), a stack under a grab (snaps).
- A stack whose peek is live, or whose surface couldn't be made, or a
  held one on a topped surface: its own peek, bar behind it, as before.

### Windows functions called by the new code

None new. `DwmSetWindowAttribute` with `DWMWA_CLOAK`
(`NativeStackTabBar::set_cloaked`), already called by `wm-platform`
(`overview_thumbnails::create_picture_window`, same attribute, on a
window of our own). Newly reached for a stack's tile, all already called:
`DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`,
`DwmUnregisterThumbnail` (the copy, the window anew), and what any hosted
tile reaches (`SetWindowPos`, `GetWindow`, `GetWindowLongPtrW`,
`ShowWindow`, `CreateWindowExW`, `DestroyWindow`,
`DwmGetWindowAttribute`). The bar thread's calls are unchanged.

### To check on screen

`GLAZEWM_ISOLATE=onesurface`, then `onesurface,readypeeks`; a stack of
three tabs beside two tiles, with a border, a backdrop and rounded
corners; `window_move`/`window_resize` at `duration_ms: 2000`, then
default.

- Move the stack (`move --direction`) and resize it: one surrogate-class
  window per monitor, no peek of its own; the bar glued to the window
  every frame (today it trails a frame); no frame without the bar at the
  start or end; ring around window and bar.
- Swap the stack with a tile, both ways (focused one on top): the bar of
  the upper one over the lower one all the way; at most one `Stacking
  ... over a tile it may cross` per swap.
- Resize: tabs never stretched; note any sliver at the bar's end.
- Switch tabs mid-move and right after it lands; open a window into the
  stack mid-move; close the active tab mid-move: the bar ends with the
  right tabs at the stack's rect, never missing (a window list or Spy++:
  `GlazeWM_TabBar` not left cloaked).
- Open, close, minimize and restore a tile beside the stack: no own peek
  (before: every one on the workspace had one).
- Scrolling layout: a stacked column scrolled half off the edge, at rest
  and moving; a floating window over it at rest stays over it.
- Toggle fullscreen on the stack (with `morepeeks`): bar gone going in,
  back coming out.
- Workspace switch mid-move: no bar left cloaked on either workspace.
- Clicks and hover on the bar after each landing: tooltips and drag still
  work (a cloaked bar takes no input while it moves).
- With `readypeeks`: move the stack twice within 5 s: the second carries
  the copy anew, no frame without the bar.

## State and next steps (2026-11-15)

For whoever picks this up in a new session. `tryout/one-surface` holds
steps 1 to 5, the workspace switches, the integration pass, resting
columns under `slidepeeks` steps 6a (grab resizes), 6b (top band), 6c (tiles crossing
monitors) and stacks, merged;
`cargo check
--workspace --tests` and `cargo clippy --workspace --all-targets -- -D
warnings` for `x86_64-pc-windows-msvc` are clean. Tests only type-check
on Linux. Everything is behind `onesurface` (and `readypeeks`), off by
default.

Seen running by the owner (desktop, `onesurface,readypeeks`): steps 1
and 2 only, a few resizes: "nothing weird, no difference noticed". Not
seen running: steps 3, 4, 5, the workspace switches, and how they work
together ("Integration: as built" has the checklist). Nothing has been
benched.

Seen by the owner on 2026-11-15 (an earlier build of this branch, before
the integration pass): swaps, resizes and workspace switches "pretty
alright" and smooth, but **the backdrop flickers away for a moment now
and then**, seen on a workspace switch: the window's opacity looked
set while its backdrop did not follow it (`restore_alpha`, the hosted
surrogate's backdrop copies vs the semi-transparent window's alpha
take-over are the first suspects). Not investigated yet, on the owner's request: everything is
ported first (items below), then this flicker, then the bench.

### What draws on a surface, and what still has a window of its own

| kind | on a surface | still its own window |
|---|---|---|
| tile move, resize | yes | stacks; live peeks |
| tile open, close, minimize, restore | yes | floating and fullscreen windows; any tile on a workspace with a stack or a live tile; a tile closed while handing back |
| scrolling column, moving | yes, held by a grab too | live; stacks; every column of a workspace with a stack or a live column |
| scrolling column, at rest | one per surface, `slidepeeks` or not | a second resting column on a surface; an incoming one while a peek kept for an outgoing surrogate rests there (that switch only) |
| workspace switch | tiles whose backdrop can be copied | floating, fullscreen, always-on-top, live, windows with a companion |
| grab resize | yes (6a) | live; a hold starting while its surface is topped (a neighbour still landing from a command); a tile opening or restoring beside a held one |
| floating <-> tiling, fullscreen toggle, floating open | yes (6b): on top -> the monitor's band surface; focused -> the tiles' surface | unfocused floating windows not on top; floating closes and minimizes; live |
| tile crossing monitors | yes (6c), with `morepeeks`: each monitor's surface draws its part | live; stacks; a grab holding a tile on either monitor's surface |

### Open, in the order to take them

1. **Removal of the old code** (in progress; merged: R0, R1, R2; `GLAZEWM_ISOLATE` is gone) (owner's decision, 2026-11-15, right
   after porting, before the bench): the old engine (`ResizeSession`,
   `nopeeks`) and every own-window peek path a surface replaced, so
   `onesurface` is the only engine, on by default with no word. See
   "Removal plan" (end of this file).
2. **Rounded backdrops on the surface** (owner, 2026-11-15; after the
   removal, before the flicker and the bench). A backdrop with rounded
   corners (the window's corner style) or `parallax` != 1 is not
   copyable (`BackdropOverlayParams::is_copyable`), so the window is live:
   a stand-in of its own, and with `has_own_peek_neighbour` /
   `beside_own_peek` much of its workspace too. The owner uses rounded
   corners with a custom border, so as built one surface mostly does not
   apply to their setup. Documented thumbnails only:
   - **A (try first): rounded from rectangles.** Keep today's pinned
     copy of the monitor backdrop, cut as a middle rect plus a few bands
     per corner, each narrower toward the corner, so their ends follow
     the corner's arc; the border ring's copy over the edge hides the
     steps where the ring is at least a step wide. Same look as at rest,
     no jump at landing. Cost: about 2k+1 backdrop registrations per
     tile instead of 1 (kept by `readypeeks`), more per-frame updates.
     Pick k from the radius and border width; without a border, say what
     shows.
   - **B (fallback): a thumbnail of the window's own backdrop overlay**,
     rounded already. The blur then moves (and stretches) with the
     window during the motion, and jumps to the new spot's at landing.
   Built behind a temporary switch (an environment variable read once
   at start, as `GLAZEWM_ISOLATE` was; it is gone after R1) so the owner
   can compare A, B and the live stand-in on screen; the switch goes
   once chosen. `parallax` != 1
   stays live either way.
3. **Parallax backdrops on the surface, then no live peeks** (owner,
   2026-11-15; after item 2). A pinned backdrop copy is cut from one
   blurred image of the whole monitor; parallax only changes where it is
   cut (window offset within the monitor times `parallax`), so shift the
   copy's `rcSource` by that, clamped at the monitor's edges as the real
   overlay is (check its code for what it does there). Not checked yet:
   read the overlay first and stop if it can't match. If items 2 and 3
   both hold, every backdrop is copyable: delete live peeks (`Live` modes
   of `WindowPeek` and `WorkspaceSurrogate`, `is_live`, `allows_motion`,
   the live arms of the admission rules) and the rules that keep tiles,
   transits and columns beside an own peek on their own
   (`has_own_peek_neighbour`, `beside_own_peek`), which only count live
   peeks since stacks. Own windows stay for the other fallbacks
   (unfocused floating not on top, floating closes, switch floating,
   fullscreen and topmost, a topped surface, surface failure).
4. **Step 8**: duplicated code, bench modes (owner, 2026-11-15: drop the
   isolate words and modes that are not compared, so the bench runs only
   what matters; and have the bench script also build and run a second
   branch, `origin/main` unless the owner names another, from a separate
   checkout, the same scenarios on each), the measuring section.
5. **The backdrop flicker** on workspace switches (above).
6. **Bench**.

### Owner's decisions still open

- Integration changed a rule: reload and pause now drop only unused
  surfaces (and every ready peek); one in use stays listed until a later
  purge or a new surface prunes it. Two cases left as they are: a moving
  tile retargeted after a clip change keeps its old surface until it
  lands; with `noslidepeeks` an incoming resting peek shows at once
  during the slide, as an own peek does today.
- Resting columns under `slidepeeks`: with a cut column on both
  workspaces of a monitor, an incoming one gets a window of its own on
  every other switch (a kept outgoing peek holds the surface's one place
  at rest). Hosting both would let the incoming one move the surface
  under the kept one for a sync. Kept as built unless the owner objects.
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
`os/3-tiles`, `os/4-transits`, `os/5-scroll`, `os/7-switch`, `os/9-integrate`, `os/10-rest-slide`, `os/6a-held`, `os/6b-band`, `os/6c-cross`, `os/11-stacks`, `os/r0-move`, `os/r1-words`, `os/r2-live`.

Laptop test build of this state (before the removal): `test/one-surface`.
Run it with `GLAZEWM_ISOLATE=onesurface,readypeeks,morepeeks`.

## Removal plan (2026-11-15)

Owner's decision: after porting (stacks last), remove the old engine
and every isolate word, so the surfaces are the only way windows
animate, with no switch to turn them off. Then step 8, the backdrop
flicker, and the bench against `origin/main` (a separate checkout:
there is no engine left on this branch to compare against).

### Decisions (owner, 2026-11-15)

| question | decision |
|---|---|
| what goes | `ResizeSession` and everything only it uses; every `GLAZEWM_ISOLATE` word |
| `onesurface`, `morepeeks`, `readypeeks`, `peeks`, `asynchandback`, `slidepeeks` | always on; word and every `false` branch deleted |
| `peeksfirst`, `latealpha`, `earlyfill`, `keepfills`, `nobackdrop`, `noring`, `live`, `keepoverlays` | removed with what only they reach (`peeksfirst`: the catch-up order of today's default stays) |
| a live window (uncopyable backdrop) opening, closing, minimizing | ported first: on an own live peek, as its moves already are |
| other rows the engine still serves | ported where an existing peek path fits, else snap (no animation); each row listed in its commit |
| own windows kept by design | live peeks; stacks if the stacks step leaves any; every "still on a window of its own" fallback in the "as built" sections; surface creation failure |

### What still reaches the engine today (from the inventory)

Entry points: `start_animation_if_needed` (`manager.rs`, from
`platform_sync.rs` redraw), `start_open_animation` (`platform_sync.rs`
open/restore), `start_away_animation` (`close_window.rs`).

| row | today | after |
|---|---|---|
| tile crossing monitors, floating <-> tiling, fullscreen toggle, live move/resize | engine unless `morepeeks` | peeks (`morepeeks` folded on) |
| live window open, close, minimize | engine, even with `morepeeks` | R2: own live peek |
| restore of a floating window | engine (`enters_on_peek` needs open or tiling) | R3: port onto 6b's floating-open path |
| mixed relayout (one window can't go on a peek) | engine for all (`resizes_relayout` returns `None`) | R3: per window; those that can't snap |
| column on the per-window path (`Motion::Column`) | engine | R3: port if `scroll_view`'s peek path fits, else snap |
| close/minimize while handing back, releasing, or of a resting column (`leave` → `Unsupported`) | engine | R3: snap (rare) unless `leave` takes it with a small change |
| peek refused otherwise (`has_other_stand_in`, no monitor, `new_entry` error, still landing) | engine | R3: snap |
| no thumbnail (elevated window) | snaps already | unchanged |

### Phases, one agent each, in order (they share files)

Branches `os/r<n>-<name>` from `tryout/one-surface`. Each phase
compiles, passes both checks, and adds its "as built" notes to this
section (a short subsection per phase, not a new top section).

- **R0, move only.** `compute_border_inset` and the edge-colour sampler
  (`EdgeColorCache`, `sample_edge_color*`, `dominant_color`, their tests)
  out of `resize_session.rs` into their own `wm-platform` module; callers
  in `window_peek.rs`, `workspace_surrogate.rs`, `companion_cover.rs`,
  `manager.rs`. No behaviour change.
- **R1, fold the words.** `isolation.rs` loses every word: fold each
  keeper to true and delete its `false` branches and parameters (the
  `one_surface`/`more_peeks`/`peeks`/... arguments of `hosts_*`,
  `host_for`, `move_host`, `allows_motion`, `surface_monitor_clips`,
  `hands_back_in_step`, ...), delete the experiments and what only they
  reach (`draw_peeks_first`/`PeeksDrawn`, `late_alpha`, `early_fill`,
  `keep_fills`, `no_backdrop`, `no_ring`, `live`). Then delete
  `isolation.rs`, the `GLAZEWM_ISOLATE` read and its warning in
  `main.rs`. The engine stays reachable for the rows above. Tests: rewrite
  those passing `false` for a folded word; drop tests of removed words.
- **R2, live transits.** Open, close and minimize of a live window on an
  own live peek, built from what `enter_on_peek`/`leave_on_peek` and
  live moves already do. Reorders no hand-over step; the backdrop
  overlay follows the moving window as for a live move.
- **R3, the other rows.** One commit per row in the table: port or
  snap. After it, nothing calls `start_animation_if_needed`,
  `start_open_animation` or `start_away_animation`.
- **R4, delete the engine.** `resize_session.rs`; in `manager.rs`
  every engine-only constant, field, function, tick section and test
  (the inventory's list: sessions, warm surrogates, handoff budget,
  away windows, session fade tail, ...); in `platform_sync.rs`
  `FrozenStep::Adopt` and the `Some(true)` arm, `mark_session_cloaked`,
  `has_slide_in`, session overlay tracking and its batch block,
  `tracks_at_take_over`, `session_overlay_placement`; session arms of
  `hands_back_to_companion`, `cover_companion_hand_back`,
  `has_active_surrogate`, `stands_in_for`, `owns_window_position`;
  `tab_bars.rs` and `wm_state.rs` session calls; engine perf stages.
  `perf_bench` columns that read removed stages go in step 8.
- **R5, `NativeSurrogate` leftovers.** Methods only the engine called
  (`revive`, `update`, `update_thumbnail_dims`, `content_size`,
  `defer_reposition`, `set_rest_opacity`, fill colour/opacity/covered,
  `reregister_thumbnail`), the `stretch` field and `create`'s `stretch`
  parameter. Then a sweep for own-window code left unreachable by R1
  (keep everything live peeks and the kept fallbacks use).
- **R6, docs.** `STAND_INS_PLAN.md` 10f marked done; references to
  removed words in `TODO.md`, `START_LATENCY_PLAN.md`,
  `ANIMATION_REVIEW.md`, `SWITCH_PEEKS_PLAN.md` noted as historical
  (not rewritten); `CLAUDE.md` lines that describe `GLAZEWM_ISOLATE`
  modes proposed to the reviewer, not edited by the agent.

### Rules for the removal agents

- **Hand-over order is untouched.** Where a step has an engine arm and a
  peek arm, delete the engine arm only and keep the call where it is:
  `hands_back_to_companion` (keep `companion_covers`, the ws-cleanup arm
  and `peeks.is_handing_back`), `cover_companion_hand_back` (keep its
  position after focus), `FrozenStep` (keep `TakeOver`/`Keep` and
  TakeOver before `pending_cloaks`), `commit_pending_cloaks`' single
  flush and `preposition_cloaked`, the pass order `cover_moved_overlays`
  → `commit_pending_cloaks` → `hand_over_moved_overlays` → raise loop,
  `apply_outgoing_surrogate_opacities` and the transparency flush,
  `HandoffWait` and the companion waits, every `WindowPeek` and
  `WorkspaceSurrogate` hand-over step. Say in the commit which arm went.
- **Delete, don't disable.** No `#[allow(dead_code)]`, no commented-out
  code, no flags kept "just in case".
- **No new Windows API.** Removal only takes calls away; R2 and R3 may
  only call what `wm-platform` already calls. Each report lists the
  Windows functions no longer called.
- Each phase: `cargo fmt --all`, `cargo check --workspace --tests` and
  `cargo clippy --workspace --all-targets -- -D warnings` for
  `x86_64-pc-windows-msvc`, clean.
- Not part of the removal: the test-build strip list (`CLAUDE.md`) and
  the bench scripts, which step 8 reworks to compare against
  `origin/main`.

#### R0: as built

Branch `os/r0-move`. Move only; not seen running.

- New module `wm-platform/src/window_edges.rs`: `compute_border_inset`,
  `EdgeColorCache`, `sample_edge_color_async`, `sample_edge_color`,
  `dominant_color`, `EDGE_SAMPLE_INSET`, `EDGE_SAMPLES`,
  `EDGE_COLOR_BUCKET_SHIFT`, `sampleable_rect`, `window_rect`, and the
  three `dominant_color` tests (with their `colorref`/`rgb` helpers).
- `window_rect` is also used by `ResizeSession` (its commit polls), so it
  moved too, now `pub(crate)`; `resize_session.rs` imports it and
  `compute_border_inset` from `window_edges`. R4 drops that import with
  the file.
- `window_peek.rs`, `workspace_surrogate.rs`, `companion_cover.rs` import
  `compute_border_inset` from `window_edges`. `lib.rs` re-exports
  `sample_edge_color_async` and `EdgeColorCache` from `window_edges`;
  `manager.rs` imports them from the crate root, unchanged.
- Left in `resize_session.rs` for R4: `overlay_factor`,
  `zoom_destination` and their tests, and the `fade_fraction` tests
  (`fade_fraction` lives in `native_surrogate.rs`; move its tests there
  if it outlives the engine).
- Windows functions added: none. Removed: none (the GDI calls of
  `sample_edge_color` moved with it).

#### R1: as built

Branch `os/r1-words`, nine commits, one per word or group. Not seen
running.

- Folded on (word and every `false` branch gone): `onesurface`,
  `readypeeks` (`hosts_on_surface`, `hosts_column`, `hosts_hold`,
  `keeps_ready`, both `surface_monitor_clips`, `hosts_switch_window`
  lose their word parameter; `hosts_transit`'s early `None` goes),
  `morepeeks` (`changes_state_on_peek`, `fullscreen_on_peeks`,
  `hides_ring_for_fullscreen`, `tile_clips`; `can_move_on_peek` and
  `allows_motion` were then always `true` and go), `peeks`/`nopeeks`
  (every `isolation::current().peeks` test, `close_window.rs`'s
  `leave_on_peek` and `grab.rs`' `nopeeks` log line included),
  `asynchandback` (`hands_back_in_step` goes: `has_surrogate` in the
  `Apply` arm keeps its session and incoming-switch terms),
  `slidepeeks` (`settle_switch_peeks` always runs where it did;
  `slides_with_switch` loses its parameter), `keepoverlays`
  (`tracks_at_take_over` takes `has_placement` only, its call kept in
  the `TakeOver` engine arm, before `pending_cloaks`).
- Removed, their default kept: `peeksfirst` (`PeeksDrawn`,
  `draw_peeks_first`, `resume_pass`, `Wm::draw_peeks_first`,
  `Wm::finish_animations`; `finish_frame` folded into
  `update_internal`; the catch-up stays before every tick), `latealpha`
  (the landing-hidden `restore_alpha` it added goes: the same tick's
  `Moving` loop restored the alpha already), `earlyfill`
  (`carries_fill_early`, `PeekView::carry_fill_early`), `keepfills`
  (`MAX_IDLE_FILL_SOURCES`, `max_idle_fill_sources`), `nobackdrop` and
  `noring` (`left_out`), `live` (out of `is_copyable`).
- Then `isolation.rs`, `pub mod isolation` and `main.rs`' warning.
  `GLAZEWM_ISOLATE` is read nowhere.
- Tests rewritten without the `false` word cases: `hosts_on_surface`,
  `hosts_column`, `hosts_hold`, `keeps_ready`, `hosts_switch_window`,
  `slides_with_switch`, `tracks_at_take_over`. Dropped: `isolation.rs`'
  tests, `allows_motion`'s, `hands_back_in_step`'s,
  `carries_fill_early`'s, `a_frame_drawn_first_resumes_at_its_instants`,
  `fill_sources_are_kept_per_color_only_under_keepfills`. Two renamed
  (`pure_column_moves_go_on_peeks`,
  `without_a_threshold_only_the_strip_moves_columns`).
- Engine rows now on peeks (`morepeeks`): tile crossing monitors,
  floating <-> tiling, fullscreen toggle, live moves and resizes. The
  other rows still reach `start_animation_if_needed`,
  `start_open_animation` and `start_away_animation` (R2, R3).
- Left for later: `WindowPeek::carries_ring` has no caller in `wm` any
  more (R5 sweep); `tracks_at_take_over` (R4); `perf_bench`'s doc
  comment naming `peeksfirst` and the bench scripts' isolate modes
  (step 8).
- For "Rounded backdrops on the surface": `BackdropOverlayParams::is_copyable`
  is still the one place deciding pinned vs live, read through
  `peek_view::is_live` (and `WindowPeek::is_live`,
  `WorkspaceSurrogate::is_live`). What keeps a live window off a
  surface today, each a distinct branch R1 left as it was: the `live`
  argument of `hosts_on_surface`, `hosts_column` and
  `hosts_switch_window`; `beside_own_peek` (tiles:
  `has_own_peek_neighbour`; columns: any live column of the workspace);
  `tile_clips` (a live tile crossing monitors gets no trail: its real
  backdrop overlay follows one peek); the switch's `copied_backdrop`
  filter; `track_overlays`' live arm (real backdrop moved along, no
  copy).
- Windows functions added: none. No longer called: the `earlyfill`
  `DwmRegisterThumbnail`s from the command (off by default), and the
  environment read of `GLAZEWM_ISOLATE` (`std::env::var`).

#### R2: as built

Branch `os/r2-live`. Not seen running.

A live window (`is_live`: a backdrop `is_copyable` refuses) now opens,
restores, closes and minimizes on a live peek of its own, by the paths
step 4 built for own pinned peeks, with the live branches the moves
already had. It reaches `start_open_animation` or
`start_away_animation` only when the peek is refused for another reason
(R3).

- **Admission**: `PeekView::can_enter` (open, restore) and
  `PeekView::leave` (close, minimize) no longer refuse `is_live`.
  `hosts_transit` still returns `None` for one (`hosts_on_surface`'s
  `live` argument), so `new_entry` makes `WindowPeek::new(.., live)`,
  `Live` mode, as for a live move. `has_own_peek_neighbour` keeps the
  other transits and moves of its workspace on own peeks, as before.
- **Motion**: `Entry::draw` calls `place_scaled` for a transit, which
  now draws a live peek too: moved over the drawn frame's visible part
  (`live_origin`, shared with `frame_origin`, which `place_stretched`
  and `place_filled` use), the thumbnail scaled and faded as on a pinned
  peek (`scaled_rects` from that origin; tested against a pinned one).
- **Overlays**: `track_overlays`' live arm, as for a live move: the real
  backdrop and ring follow the peek's drawn frame (backdrop by
  `upsert_clipped_overlay`, ring pinned and slid), behind the peek; now
  at the peek's fade (`tracked_blends`: both ends of each blend scaled
  by `WindowPeek::opacity_fraction` through
  `OverlayParams::with_opacity_scaled`), as a pinned peek fades its
  copies. A zoom draws them at full opacity, sized with the frame (the
  engine kept them at the target rect, faded by zoom progress).
- **Open**: window cloaked since `manage_window`; live peek up just
  below it, never raised (a floating one: raised in its band by the
  `peek_raises` loop); `Frozen` `Keep`; ticks zoom, slide or fade it;
  lands, `begin_hand_back` (`raise_above(source)`), overlays behind the
  window (`overlay_anchor`'s live `HandingBack` arm), uncloak
  (`Normal`), hand back. All as for an own pinned open or a live move.
- **Restore**: `restore_cloaked`, enter as an open, `raise_moving` in
  the loop, as before, then `restack_live_overlays`, as after the
  moves' raise loop. A minimize still playing turns round
  (`turn_round`), then the same two.
- **Close, minimize**: peek warmed up just below the still-shown window
  (`warm_up`), overlays moved behind it by `track_overlays`; a close's
  first tick takes over and cloaks, a minimize's `dwm_flush` and
  `take_over_leaving` run before the caller minimizes; ticks shrink,
  slide or fade it with its overlays; `finish_leaving` drops the
  overlays and sends `WM_CLOSE`. A live tile moving on its peek turns
  `Leaving` in place, as a pinned one does.
- **Band raises**: `restack_live_overlays` now also runs after each
  `raise_in_band` in the `peek_raises` loop (a no-op for pinned peeks):
  a live floating open, and a live floating <-> tiling crossing (R1),
  left their overlays under the raised peek otherwise.
- **Order**: no hand-over step moved, simplified or dropped; the two
  new `restack_live_overlays` calls go right after the raises they
  follow. No engine arm deleted (R4).
- **For items 2 and 3**: the live branch is keyed on `is_live` alone,
  in the places it was (`hosts_transit`/`hosts_on_surface`,
  `new_entry`'s mode, `track_overlays`' and `overlay_anchor`'s live
  arms), plus `place_scaled`'s `Live` arm and `tracked_blends`' fade. A
  window made copyable takes its transits onto a surface with no change
  here; deleting `PeekMode::Live` deletes those arms whole.

Still reaching the engine after R2: restore of a floating window, mixed
relayouts, `Motion::Column`, `Leave::Unsupported` (handing back,
releasing, a resting column; no monitor, no frame, a peek that couldn't
be made), other refused peeks (R3). Left as is: a live column's open in
`upsert` still has no transit (columns are R3's).

To check on screen (rounded corners with a backdrop, or `parallax` != 1):

- Open a tile with each `window_open` style (zoom, slide, fade): one
  peek window, no engine surrogate; blur and ring follow the zooming or
  sliding frame and fade with a fade; no frame without blur at landing.
- Close and minimize a tile: shrinks under its neighbours, blur and ring
  with it; no frame showing both the window and its peek, or neither.
- `toggle-minimized` twice within the animation: turns round, blur and
  ring stay with it, over its neighbours.
- A floating window with a rounded backdrop: opens raised in its band,
  its blur right under it, not left below other windows; then close it.
- Close during an open; close while neighbours still move.

Windows functions added: none (`live_origin` calls `move_to`, already
used by live peeks: `SetWindowPos`, or the batch's `DeferWindowPos`).
No longer called: none; these transits no longer make an engine
session (its surrogate, per-frame updates and overlay tracking, a zoom
open's early `set_cloaked`), but each function stays called elsewhere.

#### R3: as built

Branch `os/r3-rows`, six commits, one per row, then one deleting the
entry points left uncalled. Not seen running.

Nothing calls `start_animation_if_needed`, `start_open_animation` or
`start_away_animation` any more; they are deleted (see the last row).
R1 and R2 left no other caller: the three call sites were
`redraw_containers` (redraw, open/restore) and `animate_away`.

| row | ported to / snapped | why |
|---|---|---|
| restore of a floating window | ported: `restore_on_peek` takes the pass's `floating_raise` and enters as a floating open (6b): band surface, tiles' surface ranked over the pass, or an own peek just below its window; raised by the `peek_raises` loop, not `raise_moving`; a floating minimize still playing turns round the same way | 6b's open path fits as is |
| peek refused otherwise (moves, resizes, crossings, fullscreen toggles, opens, restores) | snapped: the redraw's `Apply` arm at `real_rect`, as with the animation off | no peek path takes them: another stand-in draws the window, no monitor or clip, a peek that couldn't be made, `move_tile` refusing an entry in another role |
| mixed relayout | per window: `resizes_relayout` goes, the relayout's resizes go on peeks whenever `window_resize` is on (`resizes_on_peeks`); `move_on_peek` refuses, per tile, one at its new size already (`is_resized_already`), which snaps with the refused ones | the all-or-nothing rule only kept neighbours in one style beside the engine |
| `Motion::Column` | ported: `sync_scroll_view` decides `resizes` per column (`can_stretch`, opening, same size), not per workspace. Snapped: what it still leaves to the redraw (a resize with `window_resize` off, one at its size already, one another stand-in draws) and a column opening or restored with `scroll` off | the scroll view's path takes any column it can stretch; the rest have no peek path (`enter_on_peek` excludes columns, whose entries the scroll view owns) |
| close/minimize `leave` -> `Unsupported` | snapped: `animate_away` returns `false`, so a close sends `WM_CLOSE` at once and a minimize minimizes at once, as with the animation off; covers the leave's own refusals too (no monitor, no frame, no peek), which share `Unsupported` | a handing-back or releasing peek has already put its window and overlays back, and a resting column's peek is the scroll view's: turning either into a leaving one is not a small change in `leave` |
| no thumbnail (elevated) | unchanged (snaps) | |
| entry points left uncalled | deleted with what only they reached: `should_start_new_animation`, `reclaim_surrogate`, `compute_transition_start_rect`, `HANDOFF_LEAD_MAX_MS`, `start_animation` (one engine test now inserts into `animations`) | `dead_code` under `-D warnings`; nothing else of the engine touched |

How a snap ends right: every snap is the redraw's `Apply` arm, the one an
animation turned off takes. `reposition_window` restores a still
minimized window and places it; `set_cloaked(false)` uncloaks it (an
open's `manage_window` cloak too); a peek still handing it back is
readied and released (`prepare_hand_back`, `release`) as at any
landing, `hand_back` giving back any alpha set aside; overlays are
placed at rest by the pass's `sync_overlays`, no session holding them.
No cloak, alpha or stand-in is put up for a snap. A close or minimize that snaps is never cloaked
for it (`close_window`'s engine cloak went: an animated close is always on
a peek, which cloaks on its first tick).

Hand-over order: no step moved, simplified or dropped. Engine arms gone:
the three calls, the effect opacity, corner style and overlay params
computed only for them, `close_window`'s cloak for an engine surrogate.
`Frozen` (`TakeOver`/`Keep`/`Adopt`), `pending_cloaks`, the flush, the
cover and hand-over passes and the raise loop are as they were.

For items 2 and 3: the live path is untouched. `is_live` (through
`is_copyable`) still alone decides live vs pinned, also for a floating
restore (`hosts_transit` -> `new_entry`), and `has_own_peek_neighbour`,
`beside_own_peek` and the live arms stay as they were.

On screen, against R2:

- A floating window coming back from minimized enters as it opens, by
  `window_minimize` reversed: focused, over the tiles; shown on top, in
  the top band; unfocused, just below its own window. The engine put its
  surrogate on top of everything. A natively maximized one snaps.
- `toggle-minimized` twice on a floating window: turns round on its
  peek (the minimize's peek was dropped and the engine restarted it).
- A relayout where one tile is at its new size already (an interactive
  resize's drag end): its neighbours resize on peeks; that one stays
  put. Before, every resize of the relayout was on the engine.
- Columns: one that can't stretch no longer keeps the others of its
  workspace off theirs; a column resize with `window_resize` off, and a
  column opening or restored with `scroll` off, snap (the engine played
  them on `scroll`'s timing, or by `window_open`).
- Close or minimize of a window still landing (handing back,
  releasing), or of a half-visible resting column: no animation.
- Any refused peek (rare: another stand-in, no monitor, a failed
  surface or peek window): no animation where the engine played one.

To check on screen:

- Minimize and restore a floating window, focused, unfocused (by IPC),
  and with `shown_on_top`; `toggle-minimized` twice within the
  animation.
- Drag a tile's edge and let go: no jump of the dragged tile, the
  others resize on peeks.
- Scrolling layout: resize a column with `window_resize` off (snaps),
  and a few columns at once with one dragged.
- Close a tile right as it lands from a move, and a half-visible
  column: it goes at once, nothing left cloaked or blank.
- With `scroll` off, open a column: it shows at once.

What R4 can now delete as well (never reached since this phase):
`has_slide_in` (always false), the `Frozen` arm's session reads
(`session_cloaked`, `reveal_session`), `FrozenStep::Adopt`,
`was_pre_committed_at`, the always-`None` `anim_opacity` and its
`set_transparency` in `Apply`, `away_windows`/`AwayWindow` and the
`has_away_animation` family (always false), `zoom_progress` and its
test, `animations` and `get_animation` (only removed, never filled),
`pending_surrogate_updates`, `slide_in_monitor_rects`, warm surrogates
(`stash_warm_surrogate` is all that is left of them), and in
`wm-platform` `ResizeSession::begin`, `begin_reusing_surrogate`,
`SessionOptions` and `DxgiVsyncWaiter::window_monitor`, which `wm` no
longer calls. `Stage::AnimStep` now times only the redraw's choice of
how a window moves (its doc says so).

Windows functions added: none. No longer called: none from the source
(each stays called elsewhere: `ResizeSession`'s `CreateWindowExW`,
`DwmRegisterThumbnail` and per-frame `DwmUpdateThumbnailProperties` /
`SetWindowPos` by peeks, `MonitorFromWindow` by `display.rs`); at run
time the engine's session path (its surrogate, the zoom open's early
`set_cloaked`, per-frame updates and overlay tracking) is no longer
reached.
