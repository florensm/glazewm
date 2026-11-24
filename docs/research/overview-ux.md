# Overview UX research

The brief is `overview-ux-plan.md`. This is part 1, the research, with the
ranked recommendations at the end. Parts 2 and 3 of the brief (the
playground, the layout comparison, the concepts) live in `overview-ux/`:
open the `.html` files in a browser. `overview-ux/layouts.js` ports the
layout algorithms below (GlazeWM's cluster, GNOME's rows, a grid), and
every GlazeWM and GNOME number here was computed with it.

Sources were read at these commits:

| Project | Commit |
| --- | --- |
| GNOME Shell | [`8252a55`](https://gitlab.gnome.org/GNOME/gnome-shell/-/tree/8252a55b5f53b696c932cb93fe7945b7f1c166ec) |
| KWin (Plasma) | [`8891b7d`](https://invent.kde.org/plasma/kwin/-/tree/8891b7dadb063afbbd1058645174e031b8d29071) |
| niri | [`398026a`](https://github.com/YaLTeR/niri/tree/398026a8e2117b9a3e939edb4318f596800853e3) |
| hyprexpo (last version before it was dropped from hyprland-plugins in `3aa21f2`) | [`eaf18d5`](https://github.com/hyprwm/hyprland-plugins/tree/eaf18d55d51cef00818c5a4fdd4170f8cc2de4dc/hyprexpo) |
| Hyprspace | [`8345bf6`](https://github.com/KZDKM/Hyprspace/tree/8345bf6f623d486ca3f87f416ccf1412416eef08) |
| Hyprland | [`df40c0b`](https://github.com/hyprwm/Hyprland/tree/df40c0bd2c7c1dfa2cfd77e91c0f561500fdcc01) |

macOS and Windows are closed. Their entries are marked **observed**: from
Apple's and Microsoft's help pages and from watching the real thing, not
from code.

## GlazeWM today, measured

On a 1920×1080 monitor at 100%, with a 48 px taskbar (working area
1920×1032) and the default config:

- The exposé's room is 1776×842: the screen minus 48 px margins
  (`layout.rs` `activities`), minus the compact strip and its label
  (18 + 75.6 + 26 + 22 = 141.6 px on top), minus 24 px of
  `wallpaper.padding` (`native/screen.rs` `frame_padding`).
- The cluster takes at most 85% of that (`expose.rs` `LARGEST`).
- **A full-screen window ends at 0.69 of its size** (48% of its area).
  Three tiles end at 0.67, six at 0.65.
- **The framed slab ends at 0.84 of the screen.** So the windows shrink
  twice: the desktop shrinks onto the slab, and the windows shrink again
  inside it, to 0.82 of the slab. On screen this reads as a camera pulled
  far back, with a wide band of empty wallpaper around a smaller cluster.
- **The zoom is a critically damped spring, stiffness 800**
  (`parsed_config.rs` `SpringParams::default`), the same as niri's. It
  ignores `open_ms`/`close_ms`: 50% of the way at 60 ms, 90% at 138 ms,
  99% at 235 ms, within ε = 1e-4 at 415 ms. So most of a 31% shrink happens
  in the first ~100 ms. That is fast, and big and fast together read as a
  lurch.
- The chrome (names, highlights) only fades in past 80% openness
  (`layout.rs` `CHROME_FROM`). The picked window grows 2.5%
  (`effects.window_lift`), with a 2 px ring.

## The others

### GNOME Shell (Activities)

1. **Sizing.** Windows go in rows (`UnalignedLayoutStrategy`): sorted by
   their vertical center into rows, then by horizontal center within a
   row. The row count is the one with the best weighted mix of scale and
   used space (`LAYOUT_SCALE_WEIGHT = 1`, `LAYOUT_SPACE_WEIGHT = 0.1`).
   Windows are capped at **0.95** of their size
   ([`workspace.js#L18`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/workspace.js#L18),
   though the comment at `#L289` still says 70%). Short windows are scaled
   up to 1.5× relative to the rest, linearly by their height as a part of
   the monitor's (`_computeWindowScale`,
   [`#L165-L181`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/workspace.js#L165-L181)),
   so a calculator doesn't vanish. Spacing between windows is
   `$base_padding` = 6 px
   ([`_window-picker.scss#L5-L8`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/data/theme/gnome-shell-sass/widgets/_window-picker.scss#L5-L8)),
   plus the size of the chrome around each preview (icon, close button).
   Relative positions are kept only loosely: rows by height, order by x.
   The workspace box is the monitor's height minus the search entry, the
   thumbnails, the dash and spacing of 2% of the height
   ([`overviewControls.js#L81-L110`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/overviewControls.js#L81-L110)).
   In GlazeWM's room, rows put a full-screen window at **0.82**, against
   the cluster's 0.69.
2. **What moves.** The whole workspace shrinks onto a rounded slab of its
   wallpaper, its corners rounding from 0 to 30 px as it goes
   (`WorkspaceBackground`, `BACKGROUND_CORNER_RADIUS_PIXELS`), with a
   `0 4px 16px 4px` shadow, over a flat `$system_base_color`
   ([`_overview.scss#L7-L9`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/data/theme/gnome-shell-sass/widgets/_overview.scss#L7-L9)).
   This is what GlazeWM's framed slab copies. The difference: GNOME's
   windows then fill most of the slab, GlazeWM's sit well inside it.
3. **Animation.** One state adjustment drives everything, 0 → 1 over
   `ANIMATION_TIME` = **250 ms**, `EASE_OUT_SINE` in and `EASE_OUT_QUAD`
   out ([`overview.js#L12`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/overview.js#L12),
   [`overviewControls.js#L718-L750`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/overviewControls.js#L718-L750)).
   A gesture drives the same adjustment 1:1, and finishes with
   `EASE_OUT_CUBIC`. No stagger. Windows rearranging later wait
   `WINDOW_REPOSITIONING_DELAY` = 750 ms and ease over 200 ms.
4. **Workspace strip.** Thumbnails above the workspace, each at most
   `MAX_THUMBNAIL_SCALE` = 5% of the monitor
   ([`workspaceThumbnail.js#L19`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/workspaceThumbnail.js#L19)).
   No labels. The neighbouring workspaces peek in at the sides of the big
   one at `WORKSPACE_INACTIVE_SCALE` = 0.94, 24–80 px apart
   ([`workspacesView.js#L22-L27`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/workspacesView.js#L22-L27)).
5. **Feedback.** Hovered: the preview grows 5 px each way
   (`WINDOW_ACTIVE_SIZE_INC`), over 200 ms (`WINDOW_SCALE_TIME`), and its
   title (a tooltip-style caption) and close button fade in over 200 ms.
   Every window shows its app icon (64 px) overlapping its bottom edge by
   70% ([`windowPreview.js#L14-L27`](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/8252a55b5f53b696c932cb93fe7945b7f1c166ec/js/ui/windowPreview.js#L14-L27)).
   Titles show only on hover. No dimming of the others.
6. **Search and keys.** Typing starts a search at once, but it finds apps,
   files and settings, not open windows. Arrows move between windows,
   `Enter` picks.

### KDE Plasma (Overview and Window View)

1. **Sizing.** `ExpoLayout` packs windows into strips, row-major on
   landscape screens (`WindowHeap.qml`), choosing the strip width by
   bisection for the aspect ratio of the area. Defaults: margins around
   each window of **7% of the area's shorter side**, a smallest window of
   15%, gaps at most 1.5× the margins, and **`maxScale` = 1.0**
   ([`expolayout.h#L184-L190`](https://invent.kde.org/plasma/kwin/-/blob/8891b7dadb063afbbd1058645174e031b8d29071/src/plugins/private/expolayout.h#L184-L190)).
   Windows keep a loose order (each slot goes to the nearest window), not
   their positions.
2. **What moves.** The desktop shrinks onto a rounded card, its corner
   radius growing with the overview's progress. Behind it, the wallpaper
   is blurred (`FastBlur`, radius 64) under the theme's background color
   at 70% ([`Main.qml#L295-L316`](https://invent.kde.org/plasma/kwin/-/blob/8891b7dadb063afbbd1058645174e031b8d29071/src/plugins/overview/Main.qml#L295-L316)).
3. **Animation.** **300 ms** (`animationTime(300ms)`,
   [`overvieweffect.cpp#L184`](https://invent.kde.org/plasma/kwin/-/blob/8891b7dadb063afbbd1058645174e031b8d29071/src/plugins/overview/overvieweffect.cpp#L184)),
   scaled by the global animation speed. `OutCubic` for the desktops,
   `InOutCubic` for the windows
   ([`WindowHeapDelegate.qml#L85-L107`](https://invent.kde.org/plasma/kwin/-/blob/8891b7dadb063afbbd1058645174e031b8d29071/src/plugins/private/qml/WindowHeapDelegate.qml#L85-L107)).
   Gestures drive it 1:1. No stagger.
4. **Workspace strip.** A `DesktopBar` along the top with labelled
   thumbnails, and a separate grid mode that zooms out further still.
5. **Feedback.** Hover draws a border and shows the title and close
   button; windows of other desktops fade.
6. **Search and keys.** Typing searches windows and KRunner results
   together; arrows move spatially; `Enter` picks.

### niri

1. **Sizing.** No re-layout: each workspace is drawn as is, scaled by
   `overview.zoom` = **0.5**
   ([`misc.rs#L128-L134`](https://github.com/YaLTeR/niri/blob/398026a8e2117b9a3e939edb4318f596800853e3/niri-config/src/misc.rs#L128-L134)),
   stacked vertically, 10% of the screen height × zoom apart
   ([`monitor.rs#L1377-L1381`](https://github.com/YaLTeR/niri/blob/398026a8e2117b9a3e939edb4318f596800853e3/src/layout/monitor.rs#L1377-L1381)).
   Windows keep their exact layout.
2. **What moves.** The whole workspace, rigidly, as a camera zoom, over a
   flat `#262626` backdrop
   ([`appearance.rs#L12`](https://github.com/YaLTeR/niri/blob/398026a8e2117b9a3e939edb4318f596800853e3/niri-config/src/appearance.rs#L12)),
   each workspace casting a shadow.
3. **Animation.** A spring: damping ratio 1, stiffness 800, ε 1e-4
   ([`animations.rs#L358-L369`](https://github.com/YaLTeR/niri/blob/398026a8e2117b9a3e939edb4318f596800853e3/niri-config/src/animations.rs#L358-L369)),
   exactly GlazeWM's default. It feels right there because nothing but
   the camera moves, and a touchpad gesture usually drives it.
4. **Workspace strip.** None: the stack is the strip.
5. **Feedback.** The focus ring, as on the desktop.
6. **Search and keys.** No search. Every keybinding keeps working; the
   wheel and touchpad scroll workspaces
   ([wiki: Overview](https://github.com/YaLTeR/niri/wiki/Overview)).

### Hyprland: hyprexpo and Hyprspace

- **hyprexpo** shows workspaces as a grid of `columns` = 3 by 3 with a
  `gap_size` = 5 px over `bg_col` = `#111111`
  ([`main.cpp#L322-L327`](https://github.com/hyprwm/hyprland-plugins/blob/eaf18d55d51cef00818c5a4fdd4170f8cc2de4dc/hyprexpo/main.cpp#L322-L327)),
  each a still image of the workspace. It zooms out of the current one
  using Hyprland's `windowsMove` animation
  ([`overview.cpp#L224-L227`](https://github.com/hyprwm/hyprland-plugins/blob/eaf18d55d51cef00818c5a4fdd4170f8cc2de4dc/hyprexpo/overview.cpp#L224-L227)),
  which inherits the global default of speed 8 (800 ms, in tenths of a
  second) and the `default` curve
  ([`AnimationTree.cpp#L62`](https://github.com/hyprwm/Hyprland/blob/df40c0bd2c7c1dfa2cfd77e91c0f561500fdcc01/src/config/shared/animation/AnimationTree.cpp#L62),
  [wiki: Animations](https://wiki.hypr.land/Configuring/Animations/)).
  Most configs shorten that a lot. hyprexpo has been dropped from
  hyprland-plugins as unmaintained (`3aa21f2`).
- **Hyprspace** is a KDE-style panel: a **250 px** strip of workspace boxes
  along the top, 12 px apart, and instead of scaling previews it re-tiles
  the real windows with bigger gaps (`gapsIn` 20, `gapsOut` 60) below it
  ([`Globals.hpp#L147-L164`](https://github.com/KZDKM/Hyprspace/blob/8345bf6f623d486ca3f87f416ccf1412416eef08/src/Globals.hpp#L147-L164)).
  So the windows shrink only by the strip and the gaps, much less than a
  scaled exposé.

### macOS Mission Control and App Exposé (observed)

1. **Sizing.** Windows keep their positions relative to each other and
   are pushed apart. They are never scaled up, and a few windows stay
   large: one window shrinks only a little. "Group windows by
   application" stacks each app's windows together
   ([Apple: Mission Control settings](https://support.apple.com/guide/mac-help/change-mission-control-preferences-mchlp2345/mac)).
2. **What moves.** Only the windows. The wallpaper stays full screen,
   slightly darkened; there is no slab and no surround. That is the
   biggest difference from GlazeWM: nothing says "the camera moved back".
3. **Animation.** About 0.3 s, ease-out, with no visible stagger; closing
   reverses. A touchpad swipe drives it 1:1.
4. **Spaces bar.** A slim row of labelled names along the top, which grows
   into thumbnails when the pointer reaches it, with a **+** at its end
   ([Apple: Mission Control](https://support.apple.com/guide/mac-help/open-windows-spaces-mission-control-mh35798/mac)).
5. **Feedback.** Hovered: a blue outline and a title under the window. No
   dimming. App Exposé dims everything but one app's windows.
6. **Search and keys.** None in Mission Control. Arrows move between
   windows only once one is hovered.

### Windows Task View (observed)

1. **Sizing.** An even grid of rows of equal height, centred, windows
   capped near their own size; relative positions are lost.
2. **What moves.** Windows fade and scale in over a dark, blurred
   (acrylic) backdrop; the desktop doesn't zoom.
3. **Animation.** About 0.25 s, ease-out, a slight scale-up as it fades in.
4. **Desktops bar.** Along the bottom, labelled, with **New desktop**
   ([Microsoft: Task View](https://support.microsoft.com/topic/b4fa0333-98f8-ef43-e25c-06d4fb1d6960)).
5. **Feedback.** Icon and title above every window, always; a rounded
   outline on hover.
6. **Search and keys.** No search; arrows and `Enter`.

## Comparison

| | Full-screen window ends at | What moves | Open | Workspaces |
| --- | --- | --- | --- | --- |
| **GlazeWM** | 0.69 (cluster ≤ 85% of room) | desktop onto slab, windows further | spring k800 ζ1 (90% at 138 ms) | compact row, 7% of height |
| GNOME | ≈ 0.8 (rows, ≤ 0.95) | desktop onto slab, windows fill it | 250 ms ease-out-sine | thumbnails, 5% |
| KDE | ≈ 0.8 (strips, ≤ 1.0, 7% margins) | desktop onto card, blurred wallpaper behind | 300 ms out-cubic | labelled bar |
| niri | 0.5, rigid | camera only | spring k800 ζ1 | the stack |
| hyprexpo | 1/3, rigid | camera only | 800 ms bezier | 3×3 grid |
| Hyprspace | ≈ 0.75 (re-tiled) | windows re-tile | Hyprland windows anim | 250 px panel |
| macOS (obs.) | ≈ 0.8, large when few | windows only | ≈ 300 ms ease-out | slim names, grow on hover |
| Windows (obs.) | ≈ 0.6, grid | windows only, fade | ≈ 250 ms | bottom bar |

## Why it zooms out too much

1. **The margins stack.** 48 px margins, the strip, 24 px of wallpaper
   padding, and then the 85% cap, which is a margin of its own: 7.5% more
   on each side of a room that already has margins. Each is reasonable;
   together they put a full-screen window at 0.69. GNOME, KDE and macOS
   all land near 0.8 by applying one margin, not four.
2. **It shrinks twice.** The slab (0.84) and the windows within it (0.82
   of the slab) shrink by different amounts, so the eye reads depth: the
   wallpaper going back, and the windows further back than it. GNOME
   shrinks the slab but its windows nearly fill it. macOS and Windows
   don't shrink the wallpaper at all.
3. **It lurches.** 31% of shrinking in about 100 ms. niri gets away with
   the same spring because it zooms a rigid picture, by a gesture. Here
   the windows also travel and rearrange at the same time, so the motion
   is bigger than the camera alone and reads as a jolt rather than a
   glide.
4. **A few windows look lost.** The cluster is centred and never larger
   than life, which is right, but at 0.69 a lone window sits in a sea of
   wallpaper. macOS keeps a lone window large.
5. **Selection is quiet.** A 2 px ring and a 2.5% lift, chrome arriving
   only at 80% openness. It's tidy, but nothing invites the pointer.

What makes one feel well judged:

- Windows end at **0.75–0.85** of their size: small enough to see them as a
  set, large enough to recognise each without reading its title.
- **One** visual depth: either only the windows move (macOS), or the
  desktop and its windows shrink about alike (GNOME, niri).
- An open of **about 250–300 ms to visually settled**, ease-out, with
  the close a little faster than the open (GNOME 250/250, KDE 300/300,
  Material's guidance of shorter exits).
- Immediate, generous hover: a lift of a few percent, the title right
  there.

## Recommendations

Ranked by how much each should help. Numbers are for 1920×1080 at 100%;
all lengths are logical pixels.

1. **Cluster cap 0.85 → 0.94.** `expose.rs` `LARGEST`. The room already
   has margins; with this alone a full-screen window goes from 0.69 to
   0.77.
2. **Margins 48 → 32, wallpaper padding 24 → 16.** `layout.rs`
   `activities` (`margin`), and `OverviewWallpaperConfig::default`
   `padding` in `parsed_config.rs`. With (1), a full-screen window ends
   at **0.80**, three tiles at 0.77, and the windows fill 0.92 of the slab
   instead of 0.82: one depth, not two.
3. **A softer, slightly alive open: `spring(0.85, 520)`; keep the close at
   `spring(1, 800)`.** `SpringParams::default` in `parsed_config.rs`
   becomes the close's; the open gets its own default
   (`overview.animation.easing` split into `open_easing` and
   `close_easing`, or a fixed default for the open). 90% at 140 ms like
   today, but 99% at 196 ms with a 0.6% overshoot: it lands rather than
   stops. The close stays fast, as it should.
4. **A small stagger by default: 10 ms, all within 60 ms.**
   `effects.stagger_ms` in `parsed_config.rs`. Windows peeling off
   nearest-first makes the open read as windows moving apart, not a camera
   moving back. Keep it off for the close.
5. **Hover: lift 0.025 → 0.04, title shown at once.** `effects.window_lift`.
   GNOME's 5 px each way on an 800 px preview is 1.25%; at 0.8 scale a
   4% lift is about 2 px more than now and clearly felt.
6. **Chrome from 0.8 → 0.6 openness.** `layout.rs` `CHROME_FROM`. Names
   and highlights arrive while the windows are still settling, so the
   overview feels ready sooner.
7. **Dim the rest slightly while hovering: `effects.dim` 0 → 0.12.** A
   lighter version of App Exposé's focus.
8. **Gap 28 → 24.** `expose.gap` in `parsed_config.rs`. Closer to
   GNOME's 6 px + chrome; with larger windows, 28 looks wide.

Two directions worth trying behind options, not as defaults:

- **`wallpaper.style: "still"`**: the wallpaper stays full screen and
  dims 25%, as on macOS, and only the windows move. Removes the depth
  completely. The framed slab remains the default since it matches what
  the zoom-into-a-workspace close does.
- **Rows for many equal windows**: with six equal tiles, GNOME's rows keep
  them at 0.81 where the cluster gives 0.65. With mixed sizes (dwindle,
  floating) rows and a grid even the windows out instead: the small ones
  grow, the big one you were working in shrinks most (0.50 and 0.40
  against the cluster's 0.66 with five dwindle tiles). So the cluster
  stays the default; rows could be picked automatically only when the
  windows are about the same size. See the layout comparison.
