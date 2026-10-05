# Window overview

A full-screen view of the focused workspace's windows, side by side as live
previews, to pick one to focus. Only available on Windows.

## Usage

| Input | Behaviour |
| --- | --- |
| `toggle-overview` (`alt+o` in the sample config) | Open the overview, or close it if open. |
| Hover a preview | Select it. |
| Click a preview | Focus its window and close. |
| Arrow keys | Move the selection. Left and right step through the previews in order; up and down go to the nearest preview in the row above or below. |
| `Enter` | Focus the selected window and close. |
| `Escape`, or a click on the background | Close without changing focus. |

The overview covers the working area of the focused monitor, over a dark
scrim. It shows the focused workspace's tiling windows in layout order, then
its floating and fullscreen windows; minimized windows are left out. Each
preview keeps its window's aspect ratio and is never shown larger than the
window itself. The focused window's preview has a ring around it, and is
selected when the overview opens.

Previews are live DWM thumbnails, so they keep updating while the overview is
open. A window with a companion overlay (e.g. a recolored copy drawn by a
theming tool) is shown through its companion, as during animations. A window
that closes or is minimized while the overview is open drops out of it, and
the others are laid out again. A window opening takes focus, which closes the
overview.

Closing the overview without a pick gives focus back to the window that had
it. Focusing another window some other way (e.g. clicking the taskbar, or a
keybinding that changes focus) closes the overview and leaves focus where it
went. Focus-follows-cursor is paused while the overview is open.

## Config

```yaml
overview:
  # Scrim covering the monitor while the overview is open.
  background_color: "#000000b3"

  # Space around and between the window previews.
  gap: "24px"

  # Highlight behind the preview under the cursor or picked with the arrow
  # keys.
  selection_color: "#ffffff1f"

  # Ring around the focused window's preview: a hex color, "accent" for the
  # OS accent color, or a `{ file, key }` mapping, as for border colors.
  focused_border_color: "accent"

  # Window titles.
  text_color: "#ffffff"
  font_family: "Segoe UI"
  font_size: "13px"
```

Colors take an optional alpha (`#rrggbbaa`). Lengths in `px` are scaled by
the monitor's DPI. Every key is optional; the values above are the defaults.

```yaml
keybindings:
  - commands: ["toggle-overview"]
    bindings: ["alt+o"]
```

The sample config moves `resize --height +2%` from `alt+o` to `alt+shift+o`
to make room for it.

## How it works

The overview is one layered, topmost window on the WM's event loop thread,
which draws it and handles its input; the WM thread only posts to it. Its
scrim, highlights and titles are drawn into a bitmap shown with
`UpdateLayeredWindow`, and DWM composites a thumbnail of each window on top.
It takes the foreground while open, so keyboard input goes to it rather than
to the window behind it; picks and cancels are sent back to the WM, which
focuses through the regular `focus --container-id` path and only then hides
the overview, so the OS never picks a window to activate in between.

## Follow-ups

Not in this first version:

- Windows of all workspaces and monitors.
- Dragging a window onto another workspace.
- Typing to filter windows by title.
- Open and close animations.
- A blurred backdrop instead of a plain scrim.
