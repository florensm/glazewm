# Window overview

A full-screen view of the focused monitor's workspaces, one card each,
showing their windows live and where the layout puts them. Switch to a
workspace or window, move windows between workspaces, search them by title,
or close them. Only available on Windows.

On open, the windows of the focused workspace shrink from where they are
into its card, while the wallpaper behind them blurs. On close, it zooms
back into the workspace you picked (or the one you were on, when
cancelled) until its windows sit where they really are.

## Layouts

- **Carousel** (default): the cards in a row, the selected one centered and
  larger than the rest. Looking inside a workspace zooms its card in further.
- **Grid**: every card at once, `grid_columns` per row.
- **Shelf**: every card small in a shelf along the top, and the selected
  workspace large on a stage below it, its place in the shelf left open.
  Moving the selection puts another workspace on the stage. While a window
  is carried or dragged, the shelf card it would land on grows, and the
  stage keeps showing the workspace it came from.

After the last workspace there is a **+** card for a new one: the workspace
`move --next-empty-workspace` would use, i.e. an unused workspace from the
config or, with `general.dynamic_workspaces` on, a new dynamic one (see
[Dynamic workspaces](./dynamic-workspaces-and-urgency.md)). Drop or send a
window on it to move the window there, or pick it to switch to it. There is
no **+** card when the monitor already shows an empty workspace, or when
there is nowhere to go.

`Tab` cycles through them while open: carousel, grid, shelf.

## Usage

### Commands

| Command | Behaviour |
| --- | --- |
| `toggle-overview` (`alt+w` in the sample config) | Open the overview as a carousel. While open, switch to the carousel, or close it if already showing it. |
| `toggle-overview --grid` | Same, for the grid. |
| `toggle-overview --shelf` | Same, for the shelf. |

### Mouse

| Input | Behaviour |
| --- | --- |
| Click a window | Focus it, switching to its workspace, and close. |
| Click a card elsewhere | Switch to its workspace and close. On the shelf, a click on a shelf card puts it on the stage instead, and a click on the stage switches to it. |
| Click the background | Close without changing anything. |
| Drag a window onto another card | Move it to that workspace. The overview stays open. |
| Middle-click a window | Close the window. |
| Wheel | Select the next or previous workspace. |

### Keyboard

The keyboard goes one step deeper with each `Space`: browsing workspaces,
picking a window inside one, then carrying that window to another
workspace. The line under the cards shows the keys for the current step.

| Key | Browsing workspaces | Picking a window | Carrying a window |
| --- | --- | --- | --- |
| `←` `→` (or `h` `l`) | Select a workspace. | Select a window, left to right. | Select a workspace. |
| `↑` `↓` (or `k` `j`) | Select a workspace a row up or down (grid). | Select a window, top to bottom. | Same as browsing. |
| `Home` / `End` | First / last workspace. | First / last window. | First / last workspace. |
| `1`–`9`, `0` | Switch to that workspace and close. | Send the selected window there; the next window gets selected. | Drop it there. |
| `s` | Same as a digit, for the [special workspace](#special-workspace). | Same. | Same. |
| `Space` | Look inside the selected workspace. | Pick up the selected window. | Drop it on the selected workspace. |
| `Enter` | Switch to the selected workspace and close. | Focus the selected window and close. | Drop it on the selected workspace. |
| `x` | | Close the selected window. | |
| `Backspace` | | Back to browsing. | Put the window down. |
| `Tab` | Next layout: carousel, grid, shelf. | Same, back to browsing in the grid. | Same. |
| `Escape` | Close without changing anything. | Same. | Same. |

Digits go to the workspace named after them (`0` is `10`), like a
`focus --workspace` binding would: one with a card, an inactive one from
the config, or one on another monitor. When no workspace has a numeric
name, they go by the cards' positions instead.

On the shelf, the stage is where windows are picked: `↓` goes into the
staged workspace's windows like `Space`, and while a window is carried,
`←` `→` pick a shelf card to drop it on as the stage keeps showing where
it came from. `Backspace` then goes back to picking on the stage.

### Search

`/` starts a search: type to match window titles and process names; matches
are outlined in `search_color` and the rest fade. `Enter` ends typing and
selects the best match, after which `n` / `N` step through the matches.
While typing, `Escape` clears the search and `Backspace` on an empty query
ends it. While carrying a
window, a search picks the workspace to drop it on.

### Special workspace

The [special workspace](./special-workspace.md) has a card on every
monitor's overview, wherever it is: docked at the bottom edge, below the
carousel or the grid, apart from the other workspaces. It is tinted with
the accent color and marked with a diamond before its name. While a
window is carried (`Space`) or dragged, the dock grows and lights up so
it is easy to drop the window on. While it is inactive or empty, its card
shows "Drop windows here".

- `s` works like a digit key for it: switch to it, send the selected
  window there, or drop the carried window there. Digits going by the
  cards' positions skip it. `↓` (or `j`) moves down onto it from the
  carousel or the grid's last row, and `↑` goes back to the card it came
  from; `←` `→`, `Home`, `End` and the wheel stay in the row above it.
- On the shelf it isn't docked: it is the shelf's last card, reached with
  `←` `→` and `End` like the others.
- Picking it, or dropping a window on it, works like
  `focus --special-workspace` and `move --special-workspace`: picking it
  summons it to the overview's monitor.

### Focus

Closing the overview without a pick gives focus back to the window that had
it. Focusing another window some other way (e.g. clicking the taskbar)
closes the overview. Focus-follows-cursor is paused while it is open.

Workspaces and windows changing while it is open (a window opening,
closing, or being moved) show up in it right away.

Previews of other workspaces need `general.hide_method: "cloak"` (the
default): with `"hide"`, their windows have nothing for DWM to show.

### Stacks

A [stack](./stacks.md) shows on its card the way it does on screen: its
shown window, with its tab bar drawn by the same code as the real one.

- The arrow keys pass over a stack as one window, its shown tab.
- Click a tab on a card's tab bar to switch to that window, or drag it
  off to move that window out of its stack and onto another workspace.
- Search finds the other tabs too; the one picked is highlighted on its
  stack's tab bar, and `Enter` switches to it.
- The border of a stack's window goes around its tab bar as well, which
  takes the place of its title strip.

### Scrolling workspaces

A workspace with the scrolling layout shows its whole strip on its card,
not just the part on screen. Its card is as tall as the others and grows
wider with the strip, up to twice a normal card; a strip wider still is
scaled to fit, with room left above and below. In the grid, a widened card
takes two columns, and `↑` `↓` go to the card closest above or below. Columns scrolled
out of view can be picked, dragged and searched like any other window;
picking one focuses it, which scrolls it into view.

The zoom on open and close starts and ends on the columns that are on
screen; the others have nowhere on screen to come from, so they fade in
with the zoom out and fade out with the zoom in. Picking a column out of
view zooms into where the strip has scrolled to show it.

## Drag shelf

Dragging a window to the top edge of a monitor shows that monitor's
workspaces in a shelf along the top, over the desktop, without the full
overview. The card under the cursor grows and the line under the shelf
says where the window would go; drop it on a card to move it to that
workspace, in the state it had before the drag (a tiling window tiles
there). Dropping it on the "+" card moves it to a new workspace, and on
the special workspace's card, there.

Moving the cursor back down past the shelf closes it, and the window drops
in the layout as usual (see [Dragging windows](./layouts.md#dragging-windows));
so does dropping it on its own workspace's card.

The shelf never takes focus, so the drag carries on uninterrupted. Turn
off Windows' own "snap layouts" bar for dragging to the top of the screen
(Settings › System › Multitasking › Snap windows) so it doesn't show up
there as well. Set `drag_shelf: false` to turn the shelf off.

## Config

```yaml
overview:
  # Blur radius of the wallpaper behind the cards.
  backdrop_blur: 40

  # Tint over the blurred wallpaper; its alpha is how dark it gets.
  backdrop_tint: "#00000066"

  # Selection and highlights: a hex color, "accent" for the OS accent
  # color, or a `{ file, key }` mapping, as for border colors. Left out, it
  # is `window_effects.focused_window.border.color`, so the overview
  # matches your borders.
  # accent_color: "accent"

  # Background of a workspace card.
  card_color: "#221e24e6"

  # Background of a hovered card, and of a window without a preview.
  surface_color: "#2d292eff"

  # Strip a window's title sits on.
  caption_color: "#161217c7"

  text_color: "#e8e0e8ff"

  # Secondary text, e.g. the key hints under the cards.
  subtext_color: "#cdc3ceff"

  # Outline of windows matching a search.
  search_color: "#89b4faff"

  font_family: "Segoe UI"

  # Columns of the grid of every workspace.
  grid_columns: 5

  # Length of the zoom out on open; 0 opens it instantly.
  open_duration_ms: 250

  # Length of the zoom into the picked workspace on close; 0 closes it
  # instantly.
  close_duration_ms: 200

  # Show the drag shelf when a window is dragged to the top edge of a
  # monitor (see below).
  drag_shelf: true
```

Colors take an optional alpha (`#rrggbbaa`). Every key is optional; the
values above are the defaults.

```yaml
keybindings:
  - commands: ["toggle-overview"]
    bindings: ["alt+w"]
```

## How it works

The overview runs on a thread of its own, with its own message loop, so
drawing and animating it never waits on the WM. The WM thread posts it what
to show, and gets the user's picks back through its event loop, which
carries them out through the regular commands (`focus`, `move`, `close`).

Everything on screen is composited by DWM, so a frame of the animation only
moves things around and never redraws pixels:

- The window previews are DWM thumbnails of the real windows (or of their
  companion overlay, e.g. a recolored copy drawn by a theming tool, as during
  animations).
- The card backgrounds, captions, borders and hints are drawn once into
  hidden layered windows, and shown as thumbnails of those, so they scale
  with their card.
- The backdrop is a composition visual of the wallpaper, sharp at first,
  with a blurred copy and the tint fading in over it.

Animations step once per display refresh, and the loop sleeps while
nothing moves. The cards move on critically damped springs, so a key press
mid-animation retargets them smoothly.

The overview takes the foreground while open, so keys go to it rather than
to the window behind it. When a pick changes focus, the WM switches
workspace and focuses the new window right away, behind the overview's
zoom into that workspace; the overview only hides once the zoom ends, so
the OS never picks a window to activate in between, and the real windows
are already where their previews end up.

## Follow-ups

Not in this version:

- Workspaces of all monitors at once.
- Fading highlights on hover and selection.
- Picking minimized windows with the mouse (they are listed on their card,
  and reachable by search).
