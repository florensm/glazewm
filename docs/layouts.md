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
    column_width: "50%"
    # Widths `cycle-column-width` steps through.
    preset_column_widths: ["33.333%", "50%", "66.667%"]
    # How the strip follows the focused column: "fit" scrolls just far
    # enough to show it, "center" keeps it in the middle.
    focus_fit_method: "fit"

animations:
  # How the scrolling strip moves.
  scroll:
    enabled: true
    duration_ms: 220
    easing: "cubic_bezier(0.16, 1, 0.3, 1)"

workspaces:
  - name: "1"
  - name: "2"
    layout: "scrolling"
```

`set-layout <layout>` changes the focused workspace's layout at runtime.
The window tree is kept: `dwindle` only changes where new windows go, and
switching to or from `scrolling` resets the widths of the workspace's
top-level windows (to `column_width`, so with `"window"` switching to
`scrolling` keeps them as wide as they were). A reload re-applies a workspace's `layout` only when
that entry changed, so a layout picked with `set-layout` survives
unrelated reloads.

## Scrolling

- A column is a window, a stack, or windows stacked vertically. Move a
  window left/right into a neighbouring column to stack it there, or out
  of its column to make it a column of its own. Dragging a window onto a
  column joins it above or below, depending on the half it is dropped on;
  dropping it between columns or past either end makes a new column.
- Resizing a column's width (`resize --width`) leaves the other columns
  alone. `cycle-column-width` steps the focused column through
  `preset_column_widths`, narrowest to widest and back around; a column
  between presets goes to the next wider one.
- `focus --first-column` and `focus --last-column` jump to either end of
  the strip, to the window there that last had focus.
- Focusing a column, by keyboard or by clicking a column partly off
  screen, scrolls the strip just far enough to show it whole, or centers
  it with `focus_fit_method: "center"`.
- Columns past a monitor edge with no other monitor behind it keep their
  real window and simply run off screen, so they stay live and slide back
  in smoothly. Where a column would reach onto another monitor, it is
  hidden and its visible part is shown as a click-through preview
  instead; clicking it focuses the column.
- The overview shows a scrolling workspace's whole strip in its card.

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
