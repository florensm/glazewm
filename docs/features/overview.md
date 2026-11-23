# Window overview

A full-screen view of every monitor's workspaces and their windows, live:
each monitor shows its own (see [Monitors](#monitors)). Switch to a
workspace or window, move windows between workspaces and monitors, search
them by title, or close them.

By default it is a calm exposé, as macOS's Mission Control and Windows'
Task View are: the shown workspace's windows spread out over its
wallpaper, and a slim strip of the other workspaces along the top.

On open, each monitor's overview zooms out of the workspace it shows, as
one piece, as hyprexpo and niri do: the windows start exactly where they
are on screen and fly out into the layout, while the desktop shrinks
onto a rounded slab of its wallpaper (or, with `backdrop.style: "blur"`,
the wallpaper blurs; see [Backdrop](#backdrop)); the names and highlights only fade in as it settles, and fade out
as soon as it closes. On close, it zooms back into the workspace you
picked, by keyboard or mouse alike, and on the other monitors (or all of
them, when cancelled) into the workspace they show, until the windows sit
where they really are. `animation.zoom_style` picks a fade instead, or neither (see
[Zoom](#zoom)).

Windows look in the overview as they do on screen, with their
[window effects](#how-it-works): their `transparency`, and on the
workspace each monitor shows, the border and backdrop the WM draws for
them, scaled with them. So opening it shrinks each monitor's workspace as
you see it into the overview, and closing it grows the picked one back.
Windows of other workspaces, and of columns scrolled out of view, get the
overview's own outline instead.

## Layouts

Each layout is best at one job. Exposé, the default, spreads the
windows out as large as they go, to find one. Activities shows the
shown workspace as it is laid out, so a window dropped there lands
where it will on screen. Stack shows every workspace at the same size,
strips at full length, to move windows between them.

- **Activities**, for arranging the shown workspace: every workspace in
  miniature in a row along the top, their names under them, and below them
  the shown workspace large and as laid out. Picking another miniature
  slides that workspace in; the title of the picked window shows under it.
- **Stack**, for arranging across workspaces and scrolling strips: every
  workspace at the same zoom, stacked top to bottom with the selected one
  in the middle, as niri's overview does. Each card lines up what its
  monitor shows with the middle of the screen, so a scrolling workspace's
  strip reaches past either side at its full length. Picking a window on a
  strip pans the strip to put that window in the middle, and the wheel
  turned sideways (or with `Shift`) pans it by hand (see [Scrolling
  workspaces](#scrolling-workspaces)).
- **Exposé** (default), for finding a window: the activities' row of
  miniatures, and below it the shown workspace's windows spread out side
  by side, as macOS's Mission Control does, in rows each scaled to fill
  the room's width, so they show as large as they can (but never larger
  than in the activities). In a row, windows keep their sizes relative to
  each other, centered on it (`expose.arrangement: "natural"`), or are all
  as tall, as a photo grid lays out pictures (`"grid"`). Every tab of its
  stacks shows too, each a window of its own. Rows follow where the
  windows sit on screen, top to bottom and left to right, as many as cover
  the most room; `expose.max_per_row` caps how many go in a row, e.g. to
  wrap a scrolling workspace's columns onto rows, the left ones on top.
  Opening, they fly out of where they are; closing, they fly back as the
  zoom lands, the picked one over the others and, of a stack, shown on it.
  A window dropped on another lands beside it, on the side nearest the
  cursor, as in the activities; dropped elsewhere it only moves.

Minimized windows show in the exposé too, in a row along the bottom: a
small window each, with its app's icon in the middle and its title, as
Windows draws nothing for them. Picking one, by mouse, the arrows or
search, restores it. Dragged onto a workspace, one moves there, still
minimized.

On the cards, each window shows its app's icon in its middle.

The row of miniatures comes in three sizes (`workspace_row`): a slim
strip (`"compact"`, the default), larger (`"shown"`), or none
(`"hidden"`). Hidden, and with `search_hint: false` (the default), the
activities and the exposé show nothing but the shown workspace: its windows over its
wallpaper (on its panel, with a framed backdrop), as far from every edge
of the screen. Workspaces are still gone through with `←` `→`, the wheel
and the digit chords, and typing still searches.

`overview.layout` sets the layout `toggle-overview` opens on, and `Tab`
goes through them while it is open (unless a search is on, which it then
steps through). Cards are drawn at the size each layout
shows them, so their text stays sharp.

Selection and hover show as highlights that fade in on the next workspace
or window, settling onto it, rather than travelling over to it: the
selected card in a ring, a hovered one in a soft glow, and the workspace
on screen tinted. A card lifts under the cursor and sinks while held
down, and an arrow or wheel step with nowhere to go bumps the selection
that way. A dragged window pops up under the cursor and trails it, and
once let go of flies into the card it lands on, or back home if it moved
nowhere. Flourishes can join in, all off by default (see
`overview.effects`): a ripple and a burst of particles out of the card
a window lands on, a soft glow following the selection, and more. In the
activities, a newly shown workspace slides in from the side its miniature
is on while the last one slides out the other way, as a workspace switch
does (`animation.switch_style` picks a fade, or the windows flying out of their
miniature instead). A click on a workspace goes there at once, zooming
into it as `Enter` does: in the activities, a workspace not shown yet
starts coming in, and its windows land where they really are as the zoom
ends.

The special workspace's card comes after the others (see
[Special workspace](#special-workspace)).

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
| `toggle-overview --layout <layout>` | Open the overview on `activities`, `stack` or `expose`. While open, switch to it, or close it if already showing it. |

### Mouse

| Input | Behaviour |
| --- | --- |
| Point at a window | Pick it: it is highlighted, and in the activities its title shows. The stack leaves its strip where it is. |
| Click a window | Focus it, switching to its workspace, and close. |
| Click a workspace | Switch to it and close, zooming into it. |
| Click the background | Close without changing anything. |
| Drag a window onto a workspace | Move it there, laid out where dropping it there on screen would put it: beside the window under the cursor, on the side nearest the cursor, which a highlight shows; or into the column there in a scrolling layout. Works within a workspace too, to rearrange it. A window that isn't tiling, or dropped on the special workspace or on a miniature in the activities, only moves. The overview stays open. |
| Middle-click a window | Close the window. |
| Click the **×** on the picked window | Close the window. It shows on the top-right corner of the window under the cursor, or picked with the keyboard, in the activities' and the exposé's large view. |
| Click **F1 Shortcuts** | Show or hide the keyboard shortcuts. |
| Wheel | Show or select the next or previous workspace. |
| Wheel sideways, or `Shift` + wheel | Pan the stack's selected strip, half a monitor per notch, as far as keeps the middle of the screen on it. |

### Keyboard

Typing searches: every letter, digit and symbol goes to the
[search](#search), from anywhere, so no command is on a key that types.
Commands are on the keys that type nothing (arrows, `Tab`, `Delete`,
`F1`, ...) and on `Alt` chords.

The keyboard goes one step deeper with each `Space`: browsing workspaces,
picking a window inside one, then carrying that window to another
workspace. Like GNOME's overview, it shows no key hints: `F1`, or the
**F1 Shortcuts** chip beside the search field, opens a sheet of them,
which any key or click closes.

An arrow goes to what lies that way on screen:

- **Activities**: `←` `→` go along the row of workspaces, and `↓` goes
  down into the shown workspace's windows, which lie below the row. Among
  the windows, the arrows go to the nearest window that way, and `↑` off
  their top edge goes back up to the row.
- **Stack**: `↑` `↓` go up and down the stack of workspaces. Their
  windows show on the cards themselves, so no arrow points into them:
  `Space` goes in. Among the windows, the arrows go to the nearest window
  that way, and `↑` or `↓` off their edge goes to the workspace above or
  below.
- With nothing that way on the monitor, an arrow goes on to the monitor
  that way (see [Monitors](#monitors)), or bumps when there is none.
  `←` and `→` in the stack therefore never go into a workspace: they
  cross to the monitor beside this one or bump, both alike.

| Key | Browsing workspaces | Picking a window | Carrying a window |
| --- | --- | --- | --- |
| Arrows, or `Alt`+`H` `J` `K` `L` | Select a workspace, or go in among its windows where an arrow points at them. | Select the nearest window that way, or go back out. | Select a workspace. |
| `Home` / `End` | First / last workspace. | First / last window. | First / last workspace. |
| `Alt`+`1`–`9`, `0` | Switch to that workspace and close. | Same. | Drop it there. |
| `Alt`+`Shift`+`1`–`9`, `0` | | Send the selected window there; the next window gets selected. | Drop it there. |
| `Alt`+`-`, `Alt`+`Shift`+`-` | The same two, for the [special workspace](#special-workspace). | Same. | Same. |
| `Space` | Look inside the selected workspace. | Pick up the selected window. | Drop it on the selected workspace. |
| `Enter` | Switch to the selected workspace and close. | Focus the selected window and close. | Drop it on the selected workspace. |
| `Delete` | | Close the selected window. | |
| `Backspace` | | Back to browsing. | Put the window down. |
| `Tab` / `Shift`+`Tab` | Switch to the next or previous layout. | Same. | Same. |
| `F1` | Show the keyboard shortcuts. | Same. | Same. |
| `Escape` | Close without changing anything. | Same. | Same. |
| Any letter, digit or symbol | Start or extend a [search](#search). | Same. | Same. |

While a search is on, some of these keys work on it instead: see
[Search](#search).

Why these keys: with every typed character searching, a command on `x`,
`s`, `n`, `h` or a digit could never be told from the start of a query.
Keys that type nothing can, and so can `Alt` chords, which Windows never
turns into characters. The chords mirror the sample config's `focus` and
`move` keybindings (`alt+h`, `alt+1`, `alt+shift+1`, ...), so the hands
stay where they are on the desktop.

Digits go to the workspace named after them (`0` is `10`), like a
`focus --workspace` binding would: one with a card on any monitor, or an
inactive one from the config. When no workspace has a numeric name, they
go by the cards' positions instead.

#### Your own keybindings

While the overview is open it has the keyboard, and the WM's keybindings
act on it instead of on the workspace behind it. They are read from your
config, whatever keys they are on:

| Command bound | In the overview |
| --- | --- |
| `focus --direction <d>`, `move --direction <d>` | The arrow that way. |
| `focus --workspace <name>` | Switch to it and close (or drop the carried window there). |
| `move --workspace <name>` | Send the selected window there, or drop the carried one. |
| `focus --special-workspace`, `move --special-workspace` | The same two, for the special workspace. |
| `focus --next-workspace`, `--prev-workspace`, and their `-active` forms | Select the next or previous workspace, as the wheel does. |
| `close` | Close the selected window. |
| `toggle-overview`, `shell-exec`, and the `wm-*` commands but `wm-enable-binding-mode` | Run as ever. |
| `wm-enable-binding-mode` | Nothing: a mode's keybindings are often on bare keys, which would then be taken from the search. |
| Anything else (e.g. `resize`, `toggle-floating`) | Nothing: it would act on the desktop behind the overview. |

A keybinding with several commands does the first of them the overview
has a use for (`move --workspace 1` of a binding that moves a window and
follows it), unless one of them runs as ever, in which case they all do.
The `Alt` chords in the table above are the overview's own, for keys
your config leaves unbound; a key your config binds goes by this table.
Regular keybindings are back as soon as the overview closes, however it
closes.

### Monitors

Each monitor shows the workspaces on it, and the keyboard is on one
monitor at a time: the focused one at first. A ring in `colors.accent`
around its edges shows which.

- An arrow with nothing that way on a monitor goes on to the nearest
  monitor that way that lines up with it: past either end of its row or
  stack, across it (e.g. `↑` in the activities, `←` `→` in the stack),
  or off an edge of the windows that doesn't lead back to the
  workspaces. Along the row or stack, it lands on the nearer end of the
  next monitor's; across it, on the workspace that monitor shows. With no
  monitor that way, it bumps. `Home`, `End` and the wheel stay on the
  monitor.
- Pointing at another monitor moves the keyboard there.
- Drag a window onto a workspace on another monitor, or carry it there
  with the keyboard (`Space`, arrows, `Space`), to move it there.
- Digits going by the cards' positions count on the monitor with the
  keyboard.
- An inactive workspace picked on a monitor (the **+**, or a digit's) is
  activated on it, unless `bind_to_monitor` binds it to another.

### Search

Typing goes to the search from anywhere: there is no key to start one.
It matches window titles and process names, on every monitor. While
idle, a "Type to search" field sits at the bottom of every monitor; once
something is typed, a search box takes its place near the top: the
query, which match it is on (e.g. "2 of 5"), and a list of the matches,
best first, each with its app's icon, its title and process with what
matched lit in `colors.search`, and its workspace. The list shows six at
a time and scrolls to keep its match in view. Windows that don't match
fade to a quarter, workspaces with none dim as a whole, and in the
activities' large view the matches grow a little. In the exposé, the
matches spread out over the room alone, growing as the query narrows,
and the rest fade out; with no match left, every window stays.

- Typing never moves the view: the list goes to the best match, and the
  ring and title tag mark it wherever it shows, but the selected
  workspace, window and monitor stay where they are.
- `Tab` / `Shift`+`Tab`, the arrows (`↓` `→` next, `↑` `←` previous) and
  `Alt`+`H` `J` `K` `L` step through the matches, around the ends, and
  `Home` / `End` go to the first and last. Each step shows the match,
  moving there the way the arrows would, to another workspace or monitor
  too. The search stays on: keep typing to narrow it down.
- Pointing at a match, or at its row in the list, puts the list on it;
  clicking a row goes to that match.
- `Enter` goes to the match the list is on. Without a match it does
  nothing. A minimized match is restored as it is gone to, as
  `toggle-minimized` would.
- `Backspace` takes back a character, and `Escape` clears the query,
  leaving the selection where the last step put it. With no query left
  they do what they always do: `Backspace` steps back, `Escape` closes.
- `Space` types a space once there is a query, for titles of several
  words. To pick up a match with `Space`, step to it and clear the query
  first.
- The `Alt` chords and `Delete` act on the match the list is on as on
  any window: `Alt`+`Shift`+`3` sends it to workspace 3.

While carrying a window, stepping through a search's matches aims the
drop at their workspaces, and `Enter` drops it on the workspace of the
match the list is on.

### Special workspace

The [special workspace](./special-workspace.md) shows on every monitor's
overview, wherever it is, after the other workspaces. It is tinted with
the accent color and marked with a diamond before its name. While it is
inactive or empty, it shows "Drop windows here".

- `Alt`+`-` works like `Alt` with a digit for it, and `Alt`+`Shift`+`-`
  like `Alt`+`Shift` with one: switch to it, send the selected window
  there, or drop the carried window there. Digits going by the cards'
  positions skip it. It is reached like any other workspace.
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
not just the part on screen, and its miniature grows up to twice as
wide. Large in the activities, it shows only the columns on screen, cut
off at the monitor's edges as on the desktop; with `strip_view: "whole"`
its whole strip is scaled to fit instead. The exposé spreads every
column, on screen or not. In the stack,
a strip shows at its full length, up to four monitors wide, reaching past
the screen's sides. On a card, a lighter frame
marks out the part on screen. A strip wider still is scaled to fit, with
room left above and below. Columns scrolled out of view can be picked,
dragged and searched like any other window; picking one focuses it, which
scrolls it into view.

In the stack, a window picked on a strip with the keyboard or a step
through a search goes to the middle of the screen, the strip panning under it; one pointed
at stays where it is, under the cursor. Selecting another workspace lets
the strip slide back to what its monitor shows. The pan is the
overview's alone: the workspace itself only scrolls once a window is
picked with `Enter` or a click, which focuses it. A card no wider than
its monitor doesn't pan. Nothing pans in the activities.

Columns scrolled out of view are left out of the activities' large view
by default, so the mouse and arrows don't reach them there; search,
their miniature, the stack and the exposé still do.

The zoom on open and close starts and ends on the columns that are on
screen, the others reaching off its sides as they do on the desktop.
Picking a column out of view zooms into where the strip has scrolled to
show it.

### Zoom

| `animation.zoom_style` | Open | Close |
| --- | --- | --- |
| `zoom` (default) | Everything zooms out of the workspace on screen as one. | Everything zooms into the picked workspace until it fills the screen. |
| `fade` | It fades in over the desktop, settling from 5% larger. | It fades out, growing, onto the desktop with the pick carried out. |
| `none` | At once. | At once. |

`animation.zoom_easing` paces both, as any animation's `easing` does: a named curve
(`ease_out`, `ease_in_out_cubic`, ...), `cubic_bezier(x1, y1, x2, y2)`,
or `spring(damping_ratio, stiffness, epsilon)`, which takes as long as it
needs to settle whatever `animation.open_duration_ms` and `animation.close_duration_ms` say.
A curve or spring overshooting makes the zoom swing past and back; the
fades stay within their range.

| `animation.switch_style` | A workspace newly shown in the activities |
| --- | --- |
| `slide` (default) | Slides in from the side its miniature is on, the last one sliding out the other way. |
| `fade` | Fades in in place as the last one fades out. |
| `miniature` | Its windows fly out of its miniature, the last one's back into theirs. |


### Backdrop

| `backdrop.style` | Behind the overview |
| --- | --- |
| `framed` (default) | The wallpaper, sharp, on a rounded panel over `backdrop.frame.color`: the desktop shrinks as a whole. |
| `blur` | The wallpaper over the whole screen, blurred by `backdrop.blur` and tinted by `backdrop.tint`. |

The framed panel sits where the shown workspace does: in the activities
and the exposé, behind the large workspace, `backdrop.frame.padding`
past its windows, sliding with it; in the stack, under the
selected card, whose background then covers it. It starts out over the
whole screen and shrinks with the zoom, its corners rounding off and its
shadow fading in as it does, so nothing jumps as it opens or closes.

## Config

```yaml
overview:
  # Layout `toggle-overview` opens on: "expose", "activities" or
  # "stack".
  layout: "expose"

  # Whether the activities and the exposé show their row of miniatures:
  # "compact" (a slim strip, as macOS's Spaces bar and Windows' Task View
  # show their desktops), "shown" (larger) or "hidden" (see "Layouts"
  # above).
  workspace_row: "compact"

  # Whether the "Type to search" field shows along the bottom. Without
  # it, typing still searches and `F1` still shows the shortcuts.
  search_hint: false

  # How much of a scrolling workspace the activities show large: "visible"
  # (the columns on screen) or "whole" (the whole strip, scaled to fit).
  strip_view: "visible"

  # Corner radius of a workspace card, in logical pixels, which the
  # windows drawn on it are rounded by too (at most their own few pixels).
  # 0 makes them square, e.g. to go with square windows.
  card_corner_radius: 14

  # Font of the overview's text, drawn with DirectWrite. Segoe UI when the
  # family isn't installed (Segoe UI Variable is Windows 11's).
  font_family: "Segoe UI Variable Text"

  # Opening the overview by resting the cursor in a corner.
  hot_corner:
    # Corner of a monitor that opens the overview as the cursor rests in it:
    # 'none', 'top_left', 'top_right', 'bottom_left' or 'bottom_right'. It
    # opens once per visit, and not while a mouse button is held.
    corner: "none"

    # How long the cursor rests in the hot corner before the overview
    # opens, in milliseconds; 0 opens it at once.
    delay_ms: 300

  # How the exposé spreads the windows: at most `max_per_row` side by side
  # before wrapping onto another row (0 for as many as shows them
  # largest), `gap` logical pixels apart.
  #
  # `arrangement` sizes the windows of a row: "natural" (as they are
  # against each other, centered on the row) or "grid" (all as tall).
  expose:
    max_per_row: 0
    gap: 28
    arrangement: "natural"

  # What shows behind the cards.
  backdrop:
    # What shows behind the cards: "blur" (the wallpaper blurred and tinted,
    # over the whole screen) or "framed" (the wallpaper, sharp, on a rounded
    # panel where the shown workspace is; see "Backdrop" above).
    style: "framed"

    # Blur radius of the wallpaper behind the cards.
    blur: 40

    # Tint over the blurred wallpaper; its alpha is how dark it gets.
    tint: "#00000066"

    # Look of the "framed" backdrop. `color` is around the panel, given
    # like `colors.accent` ("accent" for the OS accent color).
    # `corner_radius` and `padding` (wallpaper around the shown
    # workspace's windows) are in logical pixels.
    frame:
      color: "#141416ff"
      corner_radius: 14
      padding: 24
      shadow: true

  colors:
    # Selection and highlights: a hex color, "accent" for the OS accent
    # color, or a `{ file, key }` mapping, as for border colors. Left out, it
    # is `window_effects.focused_window.border.color`, so the overview
    # matches your borders.
    # accent: "accent"

    # Background of a workspace card.
    card: "#1c1c20e6"

    # Background of a hovered card, and of a window without a preview.
    surface: "#2a2a30ff"

    # Strip a window's title sits on.
    caption: "#141418c7"

    text: "#f2f2f5ff"

    # Secondary text, e.g. the search's hit count.
    subtext: "#b4b4bcff"

    # What matched, in the search box's list.
    search: "#89b4faff"

  # How the selected or hovered window is marked: "ring" (in the accent
  # color), "glow" (a soft accent glow fading out from its edge) or
  # "none" (only the title tag under it shows which it is). `width` is the
  # ring's thickness or how far the glow reaches, and `gap` the space
  # between the window and either, in logical pixels.
  window_highlight:
    style: "ring"
    width: 2
    gap: 0

  animation:
    # How it opens and closes (see "Zoom" above): "zoom", "fade" or "none".
    zoom_style: "zoom"

    # Pace of the open and close: any animation easing, e.g. "ease_out",
    # "cubic_bezier(0.22, 1, 0.36, 1)" or "spring(1.0, 400, 0.0001)",
    # which ignores the durations below.
    zoom_easing: "cubic_bezier(0.22, 1, 0.36, 1)"

    # Length of the open; 0 opens it instantly.
    open_duration_ms: 280

    # Length of the close, into the picked workspace; 0 closes it
    # instantly.
    close_duration_ms: 220

    # How the activities change the workspace they show large: "slide",
    # "fade" or "miniature".
    switch_style: "slide"

    # About how long cards and windows take to move to a new selection or
    # layout, e.g. the shown workspace sliding over to the next; 0 moves
    # them at once.
    switch_duration_ms: 260

    # How long the selection and hover highlights take to settle onto what
    # they mark next and to fade in or out, as the shortcut sheet does; 0
    # snaps them.
    highlight_duration_ms: 140

    # How far cards and windows overshoot where they head before settling
    # back, from 0 (not at all) to 0.8 (wobbly). The zooms go by
    # `zoom_easing` instead.
    bounce: 0

  # Flourishes. All are off but `window_lift`, for an overview as calm
  # as an OS's own.
  effects:
    # Soft glow of the accent color around the selected workspace.
    glow: false
    # Whether the glow slowly breathes. Keeps the overview animating on
    # every frame while it is open, so it costs a little more.
    pulse: false
    # A ring spreading out of a card as it is clicked, or as a window
    # lands on it, and drawing in on where a window closed from the
    # overview was, once it is gone.
    ripples: false
    # Particles bursting out of where a dropped window lands, and off the
    # edges of a window closed from the overview once it is gone, both
    # shown over the overview. Each takes "none", a style, or a list of
    # styles that burst together, e.g. ["confetti", "glints"]: "sparks",
    # "dust", "embers", "confetti", "glints" or "bubbles". Their color,
    # size, density and duration are `animations.particles`'s, with the
    # accent color unless that sets one.
    drop_particles: "none"
    close_particles: "none"
    # Particles off the shown workspace as the overview settles open, and
    # off the windows of each workspace it zooms into as it closes, timed
    # to land with the zoom.
    open_particles: "none"
    exit_particles: "none"
    # Whether a card wobbles as a window lands on it.
    jelly: false
    # Delay between one card and the next as they pop in on open, in
    # milliseconds; 0 brings them in together.
    stagger_ms: 0
    # How far a hovered card leans toward the cursor, in logical pixels.
    lean: 0
    # How much the cards dim while another is hovered, from 0 to 1.
    dim: 0
    # How much a dragged window stretches with its speed, and squashes as
    # it stops, from 0 to 1.
    squash: 0
    # Fading outlines trailing a dragged window; 0 for none, at most 8.
    trail: 0
    # How much the picked window grows in the large view of the
    # activities and the exposé, as a fraction of its size, from 0 to
    # 0.15.
    window_lift: 0.025
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
- Under the preview of each window on screen are thumbnails of its own
  backdrop and border overlays, placed and scaled as the window is, and
  every preview takes its window's `transparency`, which DWM's thumbnails
  don't carry. The overlays of windows off screen are hidden, which DWM
  composes for no thumbnail; cloaked instead, they showed in previews as
  small, stale boxes.
- The card backgrounds, captions, borders, labels and the search are drawn
  once into hidden layered windows, and shown as thumbnails of those, so
  they scale with their card. Their text is laid out and rasterized by
  DirectWrite (grayscale, since the pictures are translucent).
- Highlights are one small picture each, its corners shown as drawn and
  its edges stretched, so they resize without being redrawn.
  Moving the selection or the cursor therefore redraws nothing.
- Thumbnails are registered on open and on switching layouts, and
  otherwise only again when windows come, go or move: a window's title
  changing redraws its title, nothing else. The thumbnails shown stay up
  until their replacements are placed, so nothing blinks out in between.
- Each monitor's overview window covers the whole monitor, the space
  app bars or the taskbar reserve included, while its cards and its
  large view show the working area, where the windows are. The zoom
  sets off from the working area where it really is, and what the
  backdrop shows around it (over the bars) fades in, so nothing jumps.
- The backdrop is a composition visual of the wallpaper, baked as for the
  window backdrops: sharp at first, with a blurred copy and the tint
  fading in over it. A framed backdrop instead moves and scales a sharp
  copy, clipped to a rounded rect, on every frame of the zoom, as the
  thumbnails are; its shadow is cast by a plain rounded panel of its own
  under it, a `LayerVisual`'s drop shadow: cast by the wallpaper's layer
  itself, it left the panel black.

Nothing animates behind the overview: a window moved while it is on screen
lands at once, and its preview follows it there.

Animations step once per refresh of the focused monitor, and the loop
sleeps while nothing moves. Cards and windows move on springs, damped
by `animation.bounce` (critically at `0`), so a key press mid-animation retargets
them smoothly. Each step follows the spring's exact motion over the
frame, so it moves alike at any refresh rate, and even a short
`animation.switch_duration_ms` can't blow up; a bouncier spring dies away as fast,
only swinging past its target on the way. None of it draws anything: it
moves and scales the same thumbnails as before.

The WM's keyboard hook takes every key press that matches a keybinding
before any window sees it, the overview's included. While the overview
is open, the WM hands such a keybinding to the overview as what it means
there (see [Your own keybindings](#your-own-keybindings)) instead of
running its commands. It goes by whether the overview is open, which
every close clears, so nothing can leave the keybindings switched off;
if the overview can't be reached at all, it is marked closed and the
keybinding runs.

The focused monitor's window takes the foreground while open, so keys go
to the overview rather than to the window behind it; if Windows refuses it
the foreground, the overview closes rather than stay up while keys go
elsewhere. The other monitors'
windows never take it when clicked, so the keys go to whichever monitor
has the keyboard. When a pick changes focus, the WM switches
workspace and focuses the new window (scrolling its strip into view)
right away, and the overview's zoom into that workspace waits for it (at
most 400ms), so it heads to where the windows land. The cards hold still
meanwhile, so it heads straight there even while the layout would still
be moving (e.g. the stack panning to the picked window). A strip
centered on its picked window stays so while the WM scrolls the real
strip behind it: where the card goes doesn't depend on that scroll. In the
activities, a scene still coming in is blended onto where it settles as
the zoom closes, so its windows arrive exactly on the real ones however
far along it was.

Everything a screen shows is placed through one camera: at rest it shows
the layout as is, and mid-zoom it maps where the zoomed workspace shows its
monitor onto the whole screen, so windows, cards, names and highlights
move as one, and the pointer is mapped back through it. The overview only hides once the
zoom ends, so the OS never picks a window to activate in between, and the
real windows are already where their previews end up.

## Follow-ups

Not in this version:

- Minimized windows in the activities' large view: there, only their
  icons show on their card. The exposé shows them (see
  [Layouts](#layouts)).
