# Layouts

Each workspace arranges its tiling windows with a layout. Floating,
fullscreen and minimized windows are not affected.

| Layout | Behaviour |
| --- | --- |
| `split` (default) | New windows tile next to the focused one. Splits are made by hand with `toggle-tiling-direction`. |
| `dwindle` | Each new window halves the focused window along its longer side, taking the half the cursor is in (the right/bottom half when the cursor is elsewhere). |
| `scrolling` | Windows are columns on a strip that scrolls sideways to keep the focused column on screen. |

## Config

```yaml
general:
  # Layout of workspaces that don't set their own.
  default_layout: "split"

layouts:
  scrolling:
    # Width of a new column, as a percentage of the workspace or in pixels,
    # or "window" to keep the width the window has as it joins the strip
    # (for a new window, the size its app opened it at).
    default_column_width: "50%"
    # Widths `cycle-column-width` steps through.
    preset_column_widths: ["33.333%", "50%", "66.667%"]
    # When focusing a column centers it: "never" scrolls just far enough
    # to show it, "always" keeps it in the middle, and "on_overflow"
    # centers it only when it doesn't fit on screen together with the
    # column focused before it.
    center_focused_column: "never"
    # Keep the only column of a workspace in its middle.
    always_center_single_column: false
    # Heights `cycle-window-height` steps through.
    preset_window_heights: ["33.333%", "50%", "66.667%"]
    # How a column of one window shows the windows joining it: "normal"
    # stacks them, "tabbed" makes them the tabs of a stack.
    default_column_display: "normal"
    # With `general.focus_follows_cursor`, how far focusing the window
    # under the cursor may scroll the strip, as a percentage of the
    # workspace or in pixels. "0%" focuses only windows fully on screen.
    # Left out, there is no limit.
    # focus_follows_cursor_max_scroll: "0%"

animations:
  # How a scrolling layout's columns move and resize, the strip
  # scrolling included. A spring (`spring(damping_ratio, stiffness,
  # epsilon)`) lasts as long as it takes to settle, so `duration_ms` only
  # applies to other easings, though 0 still makes it instant. A spring
  # carries a column's speed over when it is sent somewhere else
  # mid-move.
  scroll:
    enabled: true
    duration_ms: 220
    easing: "spring(1.0, 800, 0.0001)"

workspaces:
  - name: "1"
  - name: "2"
    layout: "scrolling"
```

`set-layout <layout>` changes the focused workspace's layout at runtime.
The window tree is kept: `dwindle` only changes where new windows go, and
switching to or from `scrolling` resets the widths of the workspace's
top-level windows (to `default_column_width`, so with `"window"` switching to
`scrolling` keeps them as wide as they were). A reload re-applies a workspace's `layout` only when
that entry changed, so a layout picked with `set-layout` survives
unrelated reloads.

## Scrolling

A column is a window, windows stacked one above the other, or a stack of
tabs. New windows open as a column right of the focused one.

- `focus --direction left/right` goes to the column beside the focused
  one, to the window in it that last had focus; `up/down` goes between
  the windows of a column, or its tabs. Past either end, focus goes on to
  the monitor that way.
- `move --direction left/right` moves the whole focused column along the
  strip, and past either end onto the monitor that way. `up/down` moves
  the window within its column, or its tab within its stack.
- Dragging a window onto a column joins it above or below, depending on
  the half it is dropped on; dropping it between columns or past either
  end makes a new column.
- Resizing a column's width (`resize --width`, `size --width`) leaves the
  other columns alone. Dragging its left edge resizes it leftwards,
  keeping its right edge in place.
- A column is never narrower than the apps of its windows allow (Windows
  only). Its width stays as set, for when they allow it again.
- `toggle-tiling-direction` and `set-tiling-direction` do nothing: columns
  always run left to right, and the windows of a column top to bottom.

| Command | Behaviour |
| --- | --- |
| `consume-or-expel-window --direction left\|right` | Move the focused window into the column on that side, as its last window; when it shares its column, move it out into a new column on that side instead, as wide as its old one. |
| `consume-window-into-column` | Move the top window of the column right of the focused one into it, as its last window. |
| `expel-window-from-column` | Move the bottom window of the focused column into a new column right of it. |
| `swap-window --direction left\|right` | Swap the focused window with the window that last had focus in the column on that side; each takes the other's place and size. |
| `cycle-column-width [--back]` | Step the focused column through `preset_column_widths`, wrapping around: from where it was last set, else to the first preset wider than it (with `--back`, the last narrower). |
| `cycle-window-height [--back]` | The same for the focused window's height within its column, with `preset_window_heights`. |
| `reset-window-height` | Give the focused window an equal share of its column's height again. |
| `maximize-column` | Make the focused column as wide as the workspace, or back to its width before. |
| `expand-column-to-available-width` | Widen the focused column to fill the room the columns fully on screen leave, and show them all. |
| `center-column` | Scroll the focused column to the middle, once. |
| `center-visible-columns` | Scroll the columns fully on screen to the middle, together. |
| `toggle-column-tabbed-display` | Show the focused column's windows as the tabs of a stack, or stacked one above the other again. |
| `set-column-display normal\|tabbed` | Show them one way or the other. |
| `focus --first-column`, `--last-column`, `--column <n>` | Go to the first, last or nth column, to the window in it that last had focus. |
| `focus --next-column`, `--prev-column` | Go to the column right or left of the focused one, wrapping around at the strip's ends. |
| `focus --window-in-column <n>`, `--column-top`, `--column-bottom` | Go to the nth, top or bottom window of the focused column. |
| `move --first-column`, `--last-column`, `--column <n>` | Move the focused column to the start, end or nth place of the strip. |

`cycle-column-width` and `cycle-window-height` also go by
`switch-preset-column-width` and `switch-preset-window-height`.

With `default_column_display: "tabbed"`, a window joining a column of a
single window by `consume-or-expel-window` or `consume-window-into-column`
makes the two the tabs of a stack, rather than stacking them.

The view follows focus:

- Focusing a column, by keyboard or by clicking a column partly off
  screen, scrolls the strip just far enough to show it whole, or centers
  it as `center_focused_column` says. Changing the focused column's width
  scrolls it back into view the same way.
- Otherwise the view stays put relative to the focused column: columns
  opening, closing or resizing beside it move the others, not it. When the
  focused column closes, the one right of it slides into its place and
  takes focus (past the strip's end, the one left of it). A column opened
  and closed again before focus moves on gives focus back to the column it
  was opened from, which returns to where it was.
- Columns past a monitor edge with no other monitor behind it keep their
  real window and simply run off screen, so they stay live and slide back
  in smoothly. Where a column would reach onto another monitor, it is
  hidden and its visible part is shown as a click-through preview
  instead; clicking it focuses the column.
- The overview shows a scrolling workspace's whole strip in its card.
- With `focus_follows_cursor`, hovering a column partly off screen
  focuses and scrolls to it, unless `focus_follows_cursor_max_scroll`
  limits how far that may scroll.
- Keybindings can step along the strip with the mouse wheel (see
  `wheel_up` and friends in the README); the sample config binds
  alt+shift+wheel and the horizontal wheel to `focus --direction`.

## Dragging windows

While a tiling window is dragged, a highlight shows where it will land if
dropped: the half of the window or column it goes beside, or all of an
empty workspace. Any dragged window over a stack's tab bar highlights the
stack it would join. The highlight glides from one spot to the next as
the cursor moves. Only available on Windows.

```yaml
window_behavior:
  drop_indicator:
    enabled: true
    # Same values as a border's color. Left out, it is the focused
    # window's border color.
    # color: "accent"
```
