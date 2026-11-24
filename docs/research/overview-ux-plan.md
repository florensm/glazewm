# Research plan: a better-feeling overview

A brief for an agent (e.g. Claude on the web, pointed at this repo). Read
it, do the research, and make the artifacts it asks for.

The overview works, but it doesn't feel great. The best guess is that it
"zooms out too much", but nobody has pinned down why. The goal is research
on what makes a great window overview, then interactive artifacts to feel
the differences, then concrete numbers to put back into the code.

## The overview today

The code: `packages/wm-platform/src/overview/` (layout in `layout.rs` and
`expose.rs`, runtime in `native/`) and `packages/wm/src/overview/`. Docs:
`docs/features/overview.md`. Config: `resources/assets/sample-config/overview.yaml`.

- **Layouts:** "exposé" (the default: the shown workspace's windows gathered
  in the middle), "activities" (the workspace large and as laid out, under a
  row of workspace miniatures) and "stack" (every workspace stacked top to
  bottom). `Tab` switches between them.
- **Exposé sizing:** the room is the screen minus 48 px margins, minus a
  compact workspace strip on top (7% of the height, 44–96 px, plus labels),
  minus 24 px of wallpaper padding. The windows form one cluster that keeps
  their positions relative to each other, pushed apart to 28 px gaps. The
  cluster takes at most 85% of the room and never shows a window larger than
  it is (`expose.rs`). A full-screen window ends at about 55–60% of the
  screen.
- **Opening:** the whole desktop shrinks onto a rounded "slab" of wallpaper
  over a dark surround (or, as an option, the wallpaper blurs instead). The
  windows fly from their real positions into the exposé. Easing is a
  critically damped spring (stiffness 800). The open is nominally 280 ms,
  the close 220 ms. Stagger is optional, off by default.
- **Feedback:** the picked window grows 2.5%, and the selection is a 2 px
  accent ring.
- **Search:** typing filters the windows by title or process name, letters
  in order with gaps included ("vsc" finds VS Code). `Enter` picks the top
  match.
- **Keyboard:** the arrows move spatially between windows. `Shift`+`←`/`→`
  sends a window to the neighbouring workspace, and `Alt`+digit goes to a
  workspace.

## 1. Research

Use primary sources and cite them, with a commit permalink where you can.
Compare:

- macOS Mission Control and App Exposé
- GNOME Shell Activities (`js/ui/workspace.js`, `windowPreview.js`,
  `overviewControls.js`, `workspacesView.js`)
- KDE Plasma Overview and Window View (the KWin effects)
- Windows Task View
- niri's overview (its code and wiki)
- Hyprland's hyprexpo and hyprspace

Wherever the source is open, use the real constants from it. For each
overview, find:

1. How small windows end up: max scale, share of the screen, margins and
   gaps, and the layout method (rows, grid, free; whether windows keep their
   relative positions).
2. What moves on open and close: the whole desktop, or only the windows;
   wallpaper blur or dim.
3. Animation: durations, easing, stagger, and how the close differs from the
   open.
4. The workspace strip: its size and position, and its labels.
5. Window titles, icons, hover and selection feedback, and dimming of the
   other windows.
6. Search and keyboard navigation.

Then a synthesis: what makes an overview feel like it "zooms out too much"
versus well judged, and concrete target numbers for this one. Mark anything
measured from videos (macOS, Windows) as an observation, not a fact.

Write it to `docs/research/overview-ux.md`.

## 2. Artifacts

Self-contained (plain HTML/JS or React) and smooth at 60 fps.

1. **Zoom playground:** a fake 16:9 desktop with 1–8 mock windows (tiled,
   floating, one big), and a button that animates opening the overview into
   an exposé and closing it again. Sliders for:
   - the cluster's max share of the room, the margins and the gap
   - whether the wallpaper shrinks onto a slab or stays full screen (blur or
     dim)
   - the open and close durations
   - the easing: spring stiffness and damping, or a cubic-bezier
   - the stagger

   Add presets copying macOS, GNOME, KDE, niri and the current GlazeWM as
   closely as the research allows, so they can be compared side by side.
2. **Layout comparison:** the same windows laid out as GlazeWM's cluster
   (relative positions kept), GNOME-style rows and a grid, with a toggle and
   a readout of each window's final scale.
3. **Design recommendation:** the 5–10 changes most likely to make this
   overview feel better, ranked, each with the number to use and why, and
   where in the code it would go.

## 3. Back to the code

The values that feel right in the playground go into `expose.rs` (cluster
share, gap), `layout.rs` (margins, strip size) and the config defaults in
`packages/wm-common/src/parsed_config.rs`. Don't change the code as part of
this brief; the recommendations come first.
