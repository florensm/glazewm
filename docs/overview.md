# Window overview

A full-screen view of every monitor's workspaces and their windows, live:
each monitor shows its own (see [Monitors](#monitors)). Switch to a
workspace or window, move windows between workspaces and monitors, search
them by title, or close them. Only available on Windows.

On open, the windows each monitor shows fly from where they are into the
overview, while the wallpaper behind them blurs. On close, they fly back
into the workspace you picked, and on the other monitors (or all of them,
when cancelled) into the workspace they show, until they sit where they
really are.

Windows look in the overview as they do on screen, with their
[window effects](#how-it-works): their `transparency`, and the border and
backdrop the WM draws for them, scaled with them. So opening it shrinks
each monitor's workspace as you see it into the overview, and closing it
grows the picked one back.

## Layouts

- **Activities** (default): every workspace in miniature in a row along
  the top, their names under them, and below them the shown workspace
  large and as laid out. Picking another miniature slides that workspace
  in; the title of the picked window shows under it.
- **Stage**: the shown workspace large and as laid out, its name over it,
  and every other workspace small in a column on the left. Picking one from
  the column swaps it in.
- **Carousel**: the cards in a row, the selected one centered and larger
  than the rest, which fade with distance. Looking inside a workspace zooms
  its card in further.
- **Grid**: every card at once, in up to `grid_columns` columns: fewer when
  that shows the cards larger.

`overview.layout` sets the layout `toggle-overview` opens on, and `Tab`
goes through them while it is open. Cards are drawn at the size each layout
shows them, so their text stays sharp.

Selection and hover show as highlights that glide from one workspace or
window to the next. In the activities and the stage, a workspace picked
with the mouse is shown first, and gone to once picked again.

In the carousel and the grid, the special workspace's card is docked below
the others (see [Special workspace](#special-workspace)); in the other
layouts it comes after them.

After the last workspace there is a **+** for a new one: the workspace
`move --next-empty-workspace` would use, i.e. an unused workspace from the
config or, with `general.dynamic_workspaces` on, a new dynamic one (see
[Dynamic workspaces](./dynamic-workspaces-and-urgency.md)). Drop or send a
window on it to move the window there, or pick it to switch to it. There is
no **+** when the monitor already shows an empty workspace, or when there
is nowhere to go.

## Usage

### Commands

| Command | Behaviour |
| --- | --- |
| `toggle-overview` (`alt+w` in the sample config) | Open the overview on `overview.layout`. While open, switch back to that layout, or close it if already showing it. |
| `toggle-overview --layout <layout>` | Open the overview on `activities`, `stage`, `carousel` or `grid` (`expose` reads as `activities`). While open, switch to it, or close it if already showing it. |
| `toggle-overview --grid` | Same as `--layout grid`. |

### Mouse

| Input | Behaviour |
| --- | --- |
| Point at a window | Pick it (activities, stage): it is highlighted and its title shows. |
| Click a window | Focus it, switching to its workspace, and close. |
| Click another workspace | Show it (activities, stage), or switch to it and close (carousel, grid). |
| Click the shown workspace | Switch to it and close. |
| Click the background | Close without changing anything. |
| Drag a window onto another workspace | Move it to that workspace. The overview stays open. |
| Middle-click a window | Close the window. |
| Wheel | Show or select the next or previous workspace. |

### Keyboard

The keyboard goes one step deeper with each `Space`: browsing workspaces,
picking a window inside one, then carrying that window to another
workspace. The line at the bottom shows the keys for the current step.

The arrows go between workspaces along the layout: `←` `→` in the
activities and the carousel, `↑` `↓` down the stage's column, and all four
in the grid. The arrow toward the shown workspace's windows (`↓` in the
activities, `→` in the stage) goes in among them, and the arrows then go
to the nearest window that way; off their edge, back out to the
workspaces.

| Key | Browsing workspaces | Picking a window | Carrying a window |
| --- | --- | --- | --- |
| Arrows (or `h` `j` `k` `l`) | Select a workspace, or go in among its windows. | Select the nearest window that way, or go back out. | Select a workspace. |
| `Home` / `End` | First / last workspace. | First / last window. | First / last workspace. |
| `1`–`9`, `0` | Switch to that workspace and close. | Send the selected window there; the next window gets selected. | Drop it there. |
| `s` | Same as a digit, for the [special workspace](#special-workspace). | Same. | Same. |
| `Space` | Look inside the selected workspace. | Pick up the selected window. | Drop it on the selected workspace. |
| `Enter` | Switch to the selected workspace and close. | Focus the selected window and close. | Drop it on the selected workspace. |
| `x` | | Close the selected window. | |
| `Backspace` | | Back to browsing. | Put the window down. |
| `Tab` | Switch to the next layout: activities, stage, carousel, grid. | Same, back to browsing in the grid. | Same. |
| `Escape` | Close without changing anything. | Same. | Same. |

Digits go to the workspace named after them (`0` is `10`), like a
`focus --workspace` binding would: one with a card on any monitor, or an
inactive one from the config. When no workspace has a numeric name, they
go by the cards' positions instead.

### Monitors

Each monitor shows the workspaces on it, and the keyboard is on one
monitor at a time: the focused one at first. A ring in `accent_color`
around its edges, and the line of keys, show which.

- An arrow with nowhere to go on a monitor goes on to the nearest monitor
  that way that lines up with it: past either end of its row, an arrow
  its layout doesn't use (e.g. `↑` in the activities), or off any edge of
  the windows but the one they were entered by. Along the row, it lands
  on the nearer end of the next monitor's row; across it, on the
  workspace that monitor shows. `Home`, `End` and the wheel stay on the
  monitor.
- Pointing at another monitor moves the keyboard there.
- Drag a window onto a workspace on another monitor, or carry it there
  with the keyboard (`Space`, arrows, `Space`), to move it there.
- Digits going by the cards' positions count on the monitor with the
  keyboard.
- An inactive workspace picked on a monitor (the **+**, or a digit's) is
  activated on it, unless `bind_to_monitor` binds it to another.

### Search

`/` starts a search: type to match window titles and process names, on
every monitor; matches are outlined in `search_color` and the rest fade,
and the query and its hits show on every monitor. `Enter` ends typing and
selects the best match, after which `n` / `N` step through the matches,
moving the keyboard to the monitor of each. While typing, `Escape` clears
the search and `Backspace` on an empty query ends it. While carrying a
window, a search picks the workspace to drop it on.

### Special workspace

The [special workspace](./special-workspace.md) shows on every monitor's
overview, wherever it is: in the carousel and the grid, docked at the
bottom edge, apart from the other workspaces; in the other layouts, after
the other workspaces. It is tinted with the accent color and marked with a
diamond before its name. While a window is carried (`Space`) or dragged,
the dock grows and lights up so it is easy to drop the window on, and the
carried window waits beside it. While it is inactive or empty, it shows
"Drop windows here".

- `s` works like a digit key for it: switch to it, send the selected
  window there, or drop the carried window there. Digits going by the
  cards' positions skip it. `↓` (or `j`) moves down onto the dock from the
  carousel or the grid's last row, and `↑` goes back to the card it came
  from; `←` `→`, `Home`, `End` and the wheel stay in the row above it. In
  the other layouts, it is reached like any other workspace.
- Picking it, or one of its windows, summons it to the monitor it was
  picked on, like `focus --special-workspace`. Dropping a window on it
  moves the window there, wherever it is (activating it on that monitor
  if it was inactive).
- A search counts its windows once: on the monitor it is displayed on, if
  any, else on the first.

### Focus

Closing the overview without a pick gives focus back to the window that had
it. Focusing another window some other way (e.g. clicking the taskbar)
closes the overview. Focus-follows-cursor is paused while it is open.

Workspaces and windows changing while it is open (a window opening,
closing, or being moved) show up in it right away.

Previews of other workspaces need `general.hide_method: "cloak"` (the
default): with `"hide"`, their windows have nothing for DWM to show, nor
do their borders and backdrops.

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

A workspace with the scrolling layout shows its whole strip, not just the
part on screen. In the activities, the shown workspace's whole strip is
scaled to fit. On a
card, a lighter frame marks out the part on screen, and the card grows
wider with the strip: up to three monitors wide (twice in the grid and in
miniature). A strip wider still is scaled to fit, with room left above and
below. Where a layout has no room for a card that wide, the card is shown
smaller instead. In the grid, `↑` `↓` go to the card closest above or
below. Columns scrolled out of view can be picked, dragged and searched
like any other window; picking one focuses it, which scrolls it into view.

On a card, the zoom on open and close starts and ends on the columns that
are on screen; the others have nowhere on screen to come from, so they
fade in with the zoom out and fade out with the zoom in. Picking a column
out of view zooms into where the strip has scrolled to show it.

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

  # Layout `toggle-overview` opens on: "activities", "stage", "carousel"
  # or "grid".
  layout: "activities"

  # Most columns of the grid of every workspace; it takes fewer when that
  # shows the cards larger.
  grid_columns: 5

  # Length of the zoom out on open; 0 opens it instantly.
  open_duration_ms: 250

  # Length of the zoom into the picked workspace on close; 0 closes it
  # instantly.
  close_duration_ms: 200
```

Colors take an optional alpha (`#rrggbbaa`). Every key is optional; the
values above are the defaults.

```yaml
keybindings:
  - commands: ["toggle-overview"]
    bindings: ["alt+w"]
```

## How it works

The overview runs on a thread of its own, with its own message loop and a
full-screen window per monitor, so drawing and animating it never waits
on the WM. The WM thread posts it what
to show, and gets the user's picks back through its event loop, which
carries them out through the regular commands (`focus`, `move`, `close`).

Everything on screen is composited by DWM, so a frame of the animation only
moves things around and never redraws pixels:

- The window previews are DWM thumbnails of the real windows (or of their
  companion overlay, e.g. a recolored copy drawn by a theming tool, as during
  animations).
- Under each preview are thumbnails of the window's own backdrop and border
  overlays, placed and scaled as the window is, and the preview takes the
  window's `transparency`, which DWM's thumbnails don't carry. While the
  overview is on screen, the overlays of windows on other workspaces (and
  of columns scrolled out of view) are kept in place but cloaked, rather
  than hidden, so DWM keeps composing them; they are hidden again once it
  is gone.
- The card backgrounds, captions, borders, labels and hints are drawn once
  into hidden layered windows, and shown as thumbnails of those, so they
  scale with their card.
- Highlights and panels are one small picture each, its corners shown as
  drawn and its edges stretched, so they move and resize without being
  redrawn. Moving the selection or the cursor therefore redraws nothing.
- Thumbnails are registered on open and on switching layouts, and
  otherwise only again when windows come or go: a window's title changing
  redraws its title, nothing else.
- The backdrop is a composition visual of the wallpaper, sharp at first,
  with a blurred copy and the tint fading in over it.

Animations step once per refresh of the focused monitor, and the loop
sleeps while nothing moves. Cards and windows move on critically
damped springs, so a key press mid-animation retargets them smoothly.

The focused monitor's window takes the foreground while open, so keys go
to the overview rather than to the window behind it. The other monitors'
windows never take it when clicked, so the keys go to whichever monitor
has the keyboard. When a pick changes focus, the WM switches
workspace and focuses the new window right away, behind the overview's
zoom into that workspace; the overview only hides once the zoom ends, so
the OS never picks a window to activate in between, and the real windows
are already where their previews end up.

## Follow-ups

Not in this version:

- Picking minimized windows with the mouse (their icons show on their
  card, and they are reachable by search).
