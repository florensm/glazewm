# Zoom levels: design

A proposal, not built. The overview gets three levels along one continuous
zoom, which the touchpad's pinch follows with the fingers:

| Level | Shows |
| --- | --- |
| 0 | The desktop: the overview closed. |
| 1 | The exposé: the shown workspace's windows spread out (today's overview). |
| 2 | Every workspace side by side, each in miniature with its windows. |

Pinching in goes 1 → 2, pinching out 2 → 1 → 0 (closing). Lifting the fingers
settles on the nearest level, carrying the fingers' speed, so a flick goes on
to the next one. The mouse wheel with `Ctrl` steps one level per notch.

Liked in `experiments3.html` #1; ADR-0001 holds (thumbnails only, no capture).

## Input: no new Windows API

A precision touchpad sends a pinch to the window under the cursor as
`WM_MOUSEWHEEL` with `MK_CONTROL` in `wparam`, in small deltas (often well
under `WHEEL_DELTA`, 120) as the fingers move. The overview's own window
already receives `WM_MOUSEWHEEL` (`native/mod.rs`, `on_wheel`), so this is
only a branch on a flag it already reads. A two-finger scroll stays as it is
(the stack's glide, the exposé stepping workspaces).

Not used, on purpose:

- Raw finger contacts (Raw Input, HID): new Windows APIs, the kind
  `CLAUDE.md` flags for Defender on the work laptop.
- Pinching on the desktop to open: it would mean taking `Ctrl`+wheel in the
  WM's global mouse hook, which breaks zooming in every app. Opening by
  gesture is left to Windows' own touchpad settings (a three- or four-finger
  swipe mapped to the overview's keybinding).

**Step 0, before building:** check that the work laptop's touchpad driver
sends a pinch as `Ctrl`+wheel with fine deltas. Log each `WM_MOUSEWHEEL` with
`MK_CONTROL` and its delta while the overview is open (debug log), pinch,
read the log. If it sends nothing, or only whole notches, this design falls
back to `Ctrl`+wheel stepping levels, still useful with a mouse.

## Model

One number, `zoom`, from 0 to 2, owned by the overview thread:

- **Following:** each pinch delta moves `zoom` by `delta / PINCH_SPAN`
  (about 400 delta units per level, a config option to tune), and records
  the speed. `zoom` follows directly, with no spring, so it sticks to the
  fingers.
- **Settling:** no delta for about 120 ms means the fingers lifted. A spring
  (the zoom's own easing) takes `zoom` from where it is, with that speed, to
  the nearest level, or the next one past it if the speed carries it more
  than halfway. That is the same "keep velocity on retarget" the springs
  already do.
- **Landing on 0** closes the overview as `Escape` does (a cancel, picking
  nothing); landing on 1 or 2 keeps it open there.
- A `Ctrl`+wheel notch sets the target one level along and settles at once.

This is a small pure type (`ZoomLevels` in `layout.rs`), unit tested: delta
in, value and target out, settle on lift, flick past halfway, clamping at 0
and 2.

## Drawing

- **Between 0 and 1** is today's open: `openness = zoom`. The existing
  camera, chrome fade and trim all apply; `openness` is driven by the
  fingers instead of by time while following, and by the settle spring
  after. `Zoom` gains a "driven" mode beside its timed one.
- **Between 1 and 2** is new. Level 2 lays every workspace's card out in a
  grid sized to the screen (as the activities' row, but large, several rows
  if needed). Between the levels:
  - the shown workspace's scene windows interpolate from their exposé
    rects to their rects on its card in the grid (both known, so a lerp by
    `zoom - 1`);
  - the other cards come in from where the grid puts them, scaled from
    larger and faded in, as the activities' row does on open;
  - the row of miniatures fades out, as the grid replaces it.
- Picking a card at level 2 (click, `Enter`, arrows) goes back to level 1
  on that workspace, the same zoom in reverse, rather than closing.

Thumbnails: the cards' previews are registered already (the row shows
them), so level 2 costs no registration, only placement. Staged
registration (phase 6) still holds for the open.

## Config

- `overview.zoom_levels`: `false` (default) or `true`. Off, `Ctrl`+wheel and
  pinch do nothing new.
- `overview.pinch_span`: delta per level, default about 400, for touchpads
  that report more or less.
- The builder gets both, with the preview's mouse wheel (with `Ctrl`) as the
  pinch.

## Build, in order

1. Step 0: log pinch deltas on the laptop; decide continuous or stepping.
2. `ZoomLevels` model, tested.
3. Level 0 ↔ 1 driven by it (driven `Zoom`), including closing on 0.
4. Level 2 grid layout, tested like `activities` (rects inside the screen,
   apart, in order).
5. Level 1 ↔ 2 interpolation, and picking at level 2.
6. Config, docs, builder.

Each step on the tryout branch, cut to the test branch for the laptop. No new
Windows APIs in any step.
