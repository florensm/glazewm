# Grab and move

Hold a modifier and drag anywhere on a window with the mouse to move or
resize it, without its title bar or border, as AltSnap does. Off by
default.

```yaml
general:
  grab_and_move:
    enabled: true
    # Key to hold: 'alt' or 'win'.
    modifier: "alt"
    # Right-drag resizes. Off, right clicks with the modifier reach the
    # window as usual.
    resize: true
```

Reloading the config turns it on or off, or changes the key, at once.

## Gestures

| Gesture | Behaviour |
| --- | --- |
| modifier + left-drag | Moves the window, exactly like a drag by its title bar. |
| modifier + right-drag | Resizes the window from the edge or corner nearest to where it was pressed. |
| Escape while dragging | Cancels: a move puts the window back where it started, a resize puts every size back. |

The button is fixed: left moves, right resizes. The window is focused
when it is grabbed, as a click would. Neither the press nor its release
reaches the app.

Which edges a resize drags is picked from where on the window it was
pressed: the window is cut into a 3x3 grid whose side bands are 38% of
its width (or height) each. A corner cell drags its corner, a side cell
its side, and the middle cell the side it is nearest to, relative to the
window's size.

## Moving

A move is a title-bar drag in all but how it starts: the window follows
the cursor, a tiling window floats once it has moved 10px, the drop
indicator shows where it lands, and dropping it tiles it there, onto a
stack's tab bar, or onto another monitor. A stacked window takes its
stack and tab bar along. A floating window just moves. Across monitors of
different DPI, the app rescales as it crosses, as in a title-bar drag.

## Resizing

A resize behaves like a drag of the window's own border: the edges not
dragged stay where they are.

- **Split and dwindle**: the dragged edge is the boundary with the
  neighbour on that side (or the nearest container that has one), which
  alone gives or takes the room. An edge of the workspace doesn't move.
- **Scrolling**: dragging a column's left or right edge makes it wider or
  narrower without taking room from any other column; the columns beside
  it keep their widths and slide along the strip. Top and bottom edges
  move the boundary with the row above or below, within the column.
- **Stacks**: a stacked window resizes its whole stack.
- **Floating**: the window just resizes.

The neighbours resize along with the window as the mouse moves, not only
on release.

### Past the screen's edge

The cursor stays where it was pressed for the whole resize, and the edge
follows the mouse's own movement instead, so it keeps going past the
screen's edge. Pressed within 48px of the desktop's outer edge, the
cursor is first moved in from it, or that edge's movement could not be
measured.

In a scrolling layout the view keeps the column's other edge where it is
on screen while the dragged edge is in view. Once the dragged edge would
leave the view, the view scrolls to keep it at the view's edge, and stays
scrolled as the column shrinks again. A column can be made as wide as the
view; a layout set to keep columns centered (`center_focused_column:
"always"`) keeps it centered, growing both ways.

### Limits

A window is never made smaller than its app allows (its minimum width;
Windows only), nor a column narrower than 10% of the view. A floating
window keeps at least 250x140px, or its own size if it started smaller.

## Animations

Windows placed by the mouse land at once: neither a grab nor a drag by the
title bar or border animates the dragged window, and the windows that
make room for it or close up after it move at once too.

## What isn't grabbed

The press reaches the window untouched when:

- it isn't managed: ignored by a window rule, or one of the WM's own
  windows (tab bars, borders, the overview);
- it is fullscreen or maximized, minimized, a stack's hidden tab, or
  drawn moving by an animation (the press on a moving window is then the
  scroll view's, see `click_to_scroll`);
- it belongs to an elevated app (run as administrator) while the WM
  isn't elevated, which it couldn't move;
- the WM is paused, or the overview is open.

## Ending a grab

A grab ends when its button is released. It also ends, keeping what it
did, when its window closes, goes fullscreen or minimized, another window
takes the foreground (e.g. a UAC prompt), or the WM is paused, and when
its release can't be seen: the input desktop switches (a UAC prompt on
the secure desktop, Ctrl+Alt+Del, the lock screen), or the same button is
pressed again. The release of a press that was taken from the app is
always taken too, so the app never sees half a click.

The modifier doesn't open anything once released: the WM taps an
unassigned key while it is held, as it does for keybindings on the mouse
wheel, so Alt doesn't open the window's menu bar, nor Win the Start menu.

## How it works

A low-level mouse hook on a thread of its own takes the press and its
release when the modifier is held over a window it may grab. A move hands
the newest cursor position to a mover thread, which moves the window with
`SetWindowPos`; a slow app gets one current move when it is ready rather
than a backlog of stale ones, and the WM sees the moves as it sees a
title-bar drag. A resize swallows the mouse's moves, which keeps the
cursor still, and adds them up for the WM, which reads only the newest
total. The WM then lays the windows out anew at most every 8ms, without
animations; a window that hasn't yet landed at the rect it was last given
waits (up to 100ms) for a newer one, so a slow app never queues up stale
sizes.
