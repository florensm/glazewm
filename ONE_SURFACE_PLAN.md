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
