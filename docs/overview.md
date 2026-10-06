# Window overview

A full-screen view of the focused monitor's workspaces, one card each,
showing their windows live and where the layout puts them. Switch to a
workspace or window, move windows between workspaces, search them by title,
or close them. Only available on Windows.

On open, the windows of the focused workspace shrink from where they are
into its card, while the wallpaper behind them blurs.

## Layouts

- **Carousel** (default): the cards in a row, the selected one centered and
  larger than the rest. Looking inside a workspace zooms its card in further.
- **Grid**: every card at once, `grid_columns` per row.

After the last workspace there is a **+** card for a new one: the workspace
`move --next-empty-workspace` would use, i.e. an unused workspace from the
config or, with `general.dynamic_workspaces` on, a new dynamic one (see
[Dynamic workspaces](./dynamic-workspaces-and-urgency.md)). Drop or send a
window on it to move the window there, or pick it to switch to it. There is
no **+** card when the monitor already shows an empty workspace, or when
there is nowhere to go.

`Tab` switches between them while open.

## Usage

### Commands

| Command | Behaviour |
| --- | --- |
| `toggle-overview` (`alt+o` in the sample config) | Open the overview as a carousel. While open, switch to the carousel, or close it if already showing it. |
| `toggle-overview --grid` | Same, for the grid. |

### Mouse

| Input | Behaviour |
| --- | --- |
| Click a window | Focus it, switching to its workspace, and close. |
| Click a card elsewhere | Switch to its workspace and close. |
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
| `Space` | Look inside the selected workspace. | Pick up the selected window. | Drop it on the selected workspace. |
| `Enter` | Switch to the selected workspace and close. | Focus the selected window and close. | Drop it on the selected workspace. |
| `x` | | Close the selected window. | |
| `Backspace` | | Back to browsing. | Put the window down. |
| `Tab` | Switch between carousel and grid. | Same, back to browsing in the grid. | Same. |
| `Escape` | Close without changing anything. | Same. | Same. |

Digits go to the workspace named after them (`0` is `10`). When no
workspace on the monitor has a numeric name, they go by position instead.

### Search

`/` starts a search: type to match window titles and process names; matches
are outlined in `search_color` and the rest fade. `Enter` ends typing and
selects the best match, after which `n` / `N` step through the matches.
While typing, `Escape` clears the search and `Backspace` on an empty query
ends it. While carrying a
window, a search picks the workspace to drop it on.

### Focus

Closing the overview without a pick gives focus back to the window that had
it. Focusing another window some other way (e.g. clicking the taskbar)
closes the overview. Focus-follows-cursor is paused while it is open.

Workspaces and windows changing while it is open (a window opening,
closing, or being moved) show up in it right away.

Previews of other workspaces need `general.hide_method: "cloak"` (the
default): with `"hide"`, their windows have nothing for DWM to show.

## Config

```yaml
overview:
  # Blur radius of the wallpaper behind the cards.
  backdrop_blur: 40

  # Tint over the blurred wallpaper; its alpha is how dark it gets.
  backdrop_tint: "#00000066"

  # Selection and highlights: a hex color, "accent" for the OS accent
  # color, or a `{ file, key }` mapping, as for border colors.
  accent_color: "accent"

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
```

Colors take an optional alpha (`#rrggbbaa`). Every key is optional; the
values above are the defaults.

```yaml
keybindings:
  - commands: ["toggle-overview"]
    bindings: ["alt+o"]
```

The sample config moves `resize --height +2%` from `alt+o` to `alt+shift+o`
to make room for it.

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
to the window behind it. When a pick changes focus, the WM focuses the new
window first and only then hides the overview, so the OS never picks a
window to activate in between.

## Follow-ups

Not in this version:

- Workspaces of all monitors at once.
- Windows growing back into place on close.
- Fading highlights on hover and selection.
- Picking minimized windows with the mouse (they are listed on their card,
  and reachable by search).
