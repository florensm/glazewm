# TODO: `tryout/todo-cleanup`

State as of 2026-11-03, for picking this up in another session. Live
testing needs a Windows desktop session (GlazeWM is Windows-only); a cloud
session can build, lint and unit-test (see "Checks" at the end). Items
marked fixed were seen fixed live on the user's desktop, from screen
recordings of their repros and their own use, unless noted otherwise.

## Done on this branch

On top of `feat/todo-cleanup` (the macOS/`hide_method` cleanup):

- Review/audit fixes: infallible `NativeWindow` queries, dead API removed,
  `ResizeSession` dedup, hot-corner multi-monitor fix, 3 stale tests fixed,
  `errors.log` noise (IPC close handshake, frame fallback at debug, overlay
  access-denied once per window).
- `537d6437` Workspace-switch teardown spread over ticks instead of three
  `DwmFlush` calls in one tick (~16 ms stall, ~28 ms frame per switch).
- `9e0234cf`, `dc3c9c0e`, `73a8faa7` Scrolling layout peeks drawn the way the
  workspace switch draws: pinned to the monitor, with copies of the border
  ring (`overlay_copies.rs`, `window_peek.rs`, `scroll_view.rs`).
- `b33c0ef8` Backdrop copies: windows with a backdrop get pinned copies too
  (`backdrop_source.rs`): a crop of a monitor-wide backdrop per style plus a
  stretched vignette copy, both under the window's thumbnail.
- `835a9b0e` Shadow copies of moving windows, removed again in `e1f259b0`:
  they drew stripes beside windows in switches and scrolls. Moving windows
  show no shadow; the real one returns as they land. Don't bring them back
  without a live A/B.

Smoothness pass on `tryout/smooth` (2026-11-10, from `tryout/particles`;
findings, evidence and what is left in `SMOOTHNESS_REVIEW.md`). None of
it seen live: checks 14-19 below.

- `0ab906a6` Frames are evaluated on a steady estimate of the vblank
  times (`FrameClock`, `wm-platform/src/frame_clock.rs`) instead of at
  the time the timer thread happened to wake. `d8598e3f` Scrolls are
  paced by their monitor's vblank like every other animation.
  `25cffc9f` The overview steps by the same clock.
- `4f63c742` A move retargeted mid-way carries on from exactly what was
  drawn, instead of up to half a frame of travel off.
- `4da212d7` No `DwmFlush` on the WM thread at the end of every move,
  resize and open: the overlay move it waited for was already made.
- `be26317b` The wallpaper surface cache holds five per display, so two
  displays no longer bake each other's surfaces out.
- `a6a2bf33` A closed overview frees its card pictures' pixels.
- `84b0b949` The backdrop device's adapter is logged, as a warning when
  another adapter drives a display.

Open, close, minimize and restore pass (2026-11-10, from
`tryout/smooth-shades`; findings, evidence and what is left in
`ANIMATION_REVIEW.md`). None of it seen live: checks 29-37 below.

- `4f37740`, `b5f3592` A window opening in a tiling layout is moved to
  its tile as its animation starts, not after it ends: it was cloaked by
  `manage_window` already, which made the first-frame set-up skip it.
  `0ad7cf1` A `fade`/`none` open shows its stand-in (it never did).
  `356f882` A stand-in that slides or zooms in is only shown by its own
  frames.
- `983ef0d` A zoom ends at full size before the window is handed back.
- `7074e09` Zoom, close and minimize frames are evaluated on the frame
  clock like the rest (`0ab906a6` missed them).
- `9a23bf4` A restore ends a minimize animation still playing instead of
  being dropped. `9dfab1f` A close animation ends when its window is
  already gone. `5f39127` No close or minimize animation without a
  stand-in to draw (elevated windows).

How the pieces fit: every moving window is a DWM thumbnail in a surrogate
window that spans the monitor and never moves (`WorkspaceSurrogate` for
switches, `WindowPeek` for scrolling). Its ring and backdrop are more
thumbnails in the same window (`OverlayCopies`), so they all move in the same
DWM updates. The real overlays are handed over around them: covered by the
copy, hidden a composited frame later, shown again under the copy at the end
(`Handover` in `scroll_view.rs`, `WsCleanupStep` in `animation/manager.rs`).
The real window itself is swapped with its stand-in by its alpha
(`HiddenAlpha`): a semi-transparent window goes to alpha 0 before it is
cloaked or uncloaked, so the swap lands in one frame. A window drawn by a
companion (shades) is handed back only once the companion is over it again
(`HandoffWait`, kept by `NativeSurrogate`). Windows whose backdrop can't be
copied (rounded, or parallax != 1) keep the old moving ("live") surrogate.

## Fixed

User's config: border 2px, transparency 80%, backdrop on for focused and
other windows, square corners, scrolling layout.

1. **Vertical bars left and right of windows** (scrolling and switches).
   - `1b59888` AnyDesk's minimized helper window kept its border and
     backdrop overlays at its last on-screen rect, behind the tiled
     windows; the gaps between windows showed slices of it. Overlays of
     minimized windows are now hidden.
   - The shadow copies (see above).
   - Earlier suspects, kept: `8a1fc9c` pinned copies draw only the visible
     frame, not the invisible resize borders a thumbnail of a layered
     window can draw dark; `0c93737` the real ring and backdrop are
     concealed right after the flush that shows their copies, before
     anything moves.
2. **Fast keyboard scrolling flickered.**
   - `5d43d2b` Peeks carry both focus styles' copies from the start, so a
     focus change only swaps which show instead of re-registering them.
   - `d6ba457`, `73f415e` Peeks hand over by alpha like the workspace
     switch, so a semi-transparent window is no longer drawn twice (real
     window plus peek) at the start and end of a move.
3. `e6241cb` A switch started while the last one was still tearing down
   could show a handed-back window without its backdrop until it was taken
   over. Not checked live.
4. **Columns vanished at the end of a scroll** (only backdrop and ring
   left, 40-80 ms).
   - `599f0aa` Two causes. A move retargeted mid-way (quick key presses)
     left cloaked windows at its first target, so landing moved them as
     they were uncloaked, discarding their pixels until their apps
     repainted; cloaked moving columns now follow each new target. And
     the landing's z-order after a still-low focused window took windows
     below their own backdrop, where one frame makes DWM stop drawing them
     until something above changes; a window being handed back keeps its
     place. No blink in 57 s of the user's own scrolling. The workspace
     switch hands back the same way; no blink seen there since, but not
     checked from a recording.
5. **Windows themed by shades flashed uncolored as moves landed.**
   - `d5316f10` Hand-back windows keep their z-order place, so they don't
     rise above their companion. A companion only counts as back once it
     is above its window, not just shown. Workspace switches wait for
     companions like peeks do (up to 200 ms) before handing windows back.
   - `4a4fe227` A random white flash: the stand-in went in the same
     composition the companion came back in. Windows are now handed back a
     composition after the companion is first seen covering them. No flash
     in 20 recorded landings.
   - `2e795fb6` The wait moved into `NativeSurrogate` (was three copies).
     Not checked live.

Stretch items from the old list:

- `1db72ca` Config files are parsed into a document once each.
- `822eab9` Unknown config keys are logged as a warning at load (e.g. the
  user's leftover `overlay_tracking` and `hide_method`). Keys inside untagged
  values (window rule matchers, file border colors) go unreported.
- `a8ec5e2` No window titles in info logs any more.
- `f3e2966` The ~4.7k "Failed to send window event" lines at exit: the
  window hooks were unhooked from the wrong thread (`UnhookWinEvent` fails
  there), so they kept firing after the listener's channel closed.

Audited with no further change needed: the rest of the per-keypress scroll
path (only peeks for columns entering view are created mid-scroll), the
workspace switch start (`platform_sync.rs`) and teardown (`WsCleanupStep`).

## Not checked live yet

Desktop: exit GlazeWM first (it runs from `target\release` and locks the
file), `git pull` on `tryout/todo-cleanup`, `cargo build --release`, run.

1. **Shades** after `2e795fb6`: no white flash at scroll landings and
   workspace switches.
2. **A column scrolling in at the right edge**: drawn cut at the monitor
   edge, no strip of border or backdrop beside it.
3. **Two quick workspace switches** (second one before the first settles):
   the windows coming back keep their backdrop throughout.
4. **End of a workspace switch**, from a recording: no window blinking out.
5. `errors.log`: the unknown-keys warning at start, and no thousands of
   "Failed to send window event" lines after exiting the WM.
6. **Focus fade** (`animations.focus_change`, `a789f49d` plus fixes
   `a3c596a5`, `fa9f0d5c`), easiest seen with `duration_ms: 2000`:
   keyboard focus at rest and with a scroll fades once, with no jump back
   after the scroll lands; a column moving mid-fade keeps its backdrop;
   quick focus back and forth never jumps. `enabled: false` for an A/B.
7. **Lit top edge** (`border.highlight: 0.6`, `cfa9a5c0`): lighter along
   the ring's own top edge at rest, while scrolling and in a workspace
   switch.
8. **Border glow and rim** (`border.glow`, `border.rim: 0.6`): the glow
   fades out smoothly with no visible steps or lines at the corners, stays
   inside the gap (never over a neighbor or its border), and rides along
   with scrolls and workspace switches without popping as they land; a
   focus change fades it in and out. The rim is a bright hairline on the
   ring's inner edge. Also a quick look at frame times with both on
   while resizing a column.
9. **Particles** (`animations.particles.on_resize: "sparks"`,
   `on_close: "dust"`): particles off the edges a keyboard or mouse resize
   moves, starting as the resize lands; off a window's edges where it
   closes (`close` command, the app's own close button, or the app hiding
   itself). `on_move`: off the edges facing the way a window went (`move`,
   swaps, mouse drags, floating `position`), as it lands. `on_open`: off a
   new window once its open animation ends, not for windows already open
   as the WM starts. `on_focus`: off the window taking focus, not again for
   one that already bursts (opened, or on a workspace just switched to).
   `on_workspace_switch`: off every window of the workspace switched to, or
   out of the monitor's middle for an empty one, as the slide or cut
   lands. Go through every style, and a list of two or three at once:
   confetti flecks spin and flutter edge on, bubbles are rings with a
   highlight, glints are four-ray twinkles. Check they never take a click,
   never show over the overview, vanish cleanly, and that holding a resize
   key doesn't pile them up (at most 8 bursts at once). `GLAZEWM_PERF`
   frame times should not move, as the compositor plays them.
10. **Overview particles** (`overview.effects.drop_particles`,
    `close_particles`): drop a window on another card, and close one from
    the overview; the burst shows over the overview, also once the WM
    moves windows and the overview raises itself again, and clicks still
    reach the cards under it. `open_particles`: off the shown workspace once
    the opening zoom settles. `exit_particles`: off the windows of the
    workspace zoomed into, landing with the zoom and staying over the
    desktop as the overview hides; with `close_duration_ms: 0` at once.

11. **Overview layouts** (stage, carousel and grid removed; `stack` added,
    niri-style): `Tab` switches between activities and stack. In the
    stack, workspaces sit top to bottom at one zoom with the selected one
    in the middle; `↑` `↓` and the wheel move between them, `Space` goes
    in among the windows (not `→` any more, see 26). A scrolling strip
    reaches past both sides at full length (up to four monitors),
    stepping to a window centers it (see 28), and the wheel turned
    sideways (or `Shift` + wheel) pans it
    without the strip jumping back to the picked window. Pointing at a
    window must not pan it. Check the open and close zoom on a wide strip,
    and memory with a few wide strips open (each card's picture is as wide
    as its strip).
12. **Snappier overview**: rings (selection, hover, picked window, drop
    half, glow) fade in on what they mark instead of travelling over; the
    zoom no longer overshoots and defaults to 200 ms open, 150 ms close.
    A config still saying `layout: "stage"`, `"carousel"` or `"grid"` now
    fails to load, naming the valid values.

13. **Overview zoom, one camera** (`zoom_style`, `zoom_easing`,
    `switch_style`): open and close, in both layouts, by `Enter` and by a
    click (on a miniature, a scene window, a stack card off the middle):
    windows start and end exactly on the real ones, nothing pops in or
    jumps, the miniature row comes in from above with the zoom. A click on
    a miniature of a workspace not shown slides it in while zooming, and
    still lands exact. `zoom_style: "fade"`: crossfades with the desktop,
    no flash of wallpaper at the end. `zoom_easing: "spring(0.6, 400,
    0.001)"` swings past and back without breaking the fades. Each
    `switch_style` with the arrows and the wheel.

Smoothness pass (`tryout/smooth`; on the laptop `test/smooth`, by eye and
`errors.log` only):

14. **Steady motion** (`0ab906a6`, `d8598e3f`, `25cffc9f`): scroll a
    scrolling layout with single presses and with a held key, switch
    workspaces, move and resize, open and close the overview, on each
    monitor. Motion is even, starts at once, never runs ahead and snaps
    back, and never freezes mid-way. Desktop: also with a game or a
    video running on the other monitor, and once on the 60 Hz monitor
    right after one on the faster one. Laptop: on battery (60 Hz) and
    plugged in, and across the internal panel and the external monitor.
    `errors.log`: no "Animation timer stalled" lines.
15. **Retargeted moves** (`4f63c742`): quick repeated scroll presses,
    each before the last lands: the strip glides on with no hitch or
    step back at a press. Same for a resize key held down in a tiling
    layout: the windows keep their place at each repeat.
16. **Ends of moves** (`4da212d7`): move and resize a window with a
    backdrop and a border, also past a neighbour whose own move ends
    first, and a window in a stack (tab bar): as each lands, the backdrop
    and ring don't flicker, and the ring keeps framing the tab bar.
    Desktop: `perf_bench` `resize` and `move` before and after (commands
    in `SMOOTHNESS_REVIEW.md`): about one `dwm_flush` call less per
    burst.
17. **Two displays** (`be26317b`): with windows on both and the overview
    opened once, change focus back and forth and open new windows: no
    hitch as a backdrop appears, and backdrops show the right wallpaper
    on each display. Task Manager, Details tab: GlazeWM's memory and GPU
    memory don't climb as windows are opened and closed.
18. **Overview memory** (`a6a2bf33`): open the overview in the stack
    layout, close it, note GlazeWM's memory in Task Manager; it is lower
    than with the overview open, and opening it again draws every card
    (none blank, none stale).
19. **Adapter log** (`84b0b949`): after a start with a backdrop
    configured, `errors.log` has no "Backdrops are drawn on" warning when
    one GPU drives every display. On the laptop, note what it says with
    and without the external monitor, and whether Defender leaves the
    build alone; this commit is the only one with new Windows calls.

Shades hand-backs (not part of `tryout/smooth`):

20. **Shades with moves and resizes** (`909b925`, `72d6bc2`, `1fbda69`,
    `2ddb24f`), with transparency on (e.g. 80%) and a shades-themed
    window: go from 1 to 2 tiled windows and back (open a second window,
    close it), open a second window of the themed app, and resize the
    themed window with the keyboard. No white frame at any landing, and
    no lighter, see-through look while it moves. Then once with
    transparency off.

21. **Shades with switches and scrolls** (`5e76d21`, `d34367e`,
    `cab7cc8`, `5d7a47f`), with transparency on (e.g. 80%) and a
    shades-themed window: switch to its workspace and away several times,
    and scroll its column out of view and back in. Wait a few seconds,
    then do both again. No white frame at any landing, and no lighter,
    see-through look while it moves. Then once with transparency off.

22. **Shades with stack tabs** (`604fdca`): stack two or more
    shades-themed windows, then switch tabs back and forth, by keyboard
    (`cycle-stack-focus`) and by clicking the tab bar, slowly and quickly.
    Again with animations disabled. No white frame on the new tab, and
    the previous tab never shows over it.

Overview keys and search (`tryout/smooth-shades`; on the laptop
`test/smooth-shades`). Unit-tested, and run under Wine off Windows, but
none of it seen live. Keys are in `docs/overview.md`.

23. **Typing searches** (`90a28c6`): open the overview and type a
    letter, a digit, `/`, `x`, `s`: each goes to the search field, none
    runs a command. `Space` on an empty field still looks inside a
    workspace, and types a space once there is a query. `Escape` clears
    the query first and closes on the second press; `Backspace` erases,
    then steps back. `F1` and the **F1 Shortcuts** chip open the sheet,
    `Delete` closes the picked window. With a non-US keyboard layout
    too: symbols typed with AltGr go to the query.
24. **Search follows and animates** (`90a28c6`): with matches on other
    workspaces and on the other monitor, type slowly: at each character
    the selection glides to the best match on the same springs as an
    arrow press (the activities slide the workspace in, the stack rolls
    to its card), nothing snaps, and the keyboard ring moves to the
    other monitor. `Tab` / `Shift`+`Tab`, the arrows and `Home` / `End`
    step through the matches while the field keeps "2 of 5" up to date,
    and typing on still narrows the search. `Enter` zooms into the
    picked match. The old jump on `Enter` snapped because nothing was
    selected before the pick held the cards; if a jump still snaps,
    note which layout and whether it crosses a monitor.
25. **The overview owns the keys** (`e776843`): with the overview open,
    `alt+h/j/k/l` and `alt+arrows` move the overview's selection and
    the workspace behind it does not change focus (close with `Escape`
    and check the same window is focused as before). `alt+shift+h` etc.
    do the same. `alt+1`..`9` switch to that workspace and close;
    `alt+shift+3` with a window picked sends it to workspace 3 and the
    overview stays open; `alt+shift+q` closes the picked window, not
    the one focused behind. `alt+r` (resize mode) and `alt+t` do
    nothing while it is open, so `h` typed afterwards still searches. `alt+w` still closes it,
    `alt+shift+r` still reloads the config. Right after it closes (by
    `Enter`, `Escape`, a click, and by clicking the taskbar so it
    deactivates), `alt+h` moves focus on the desktop again. No beep on
    any `Alt` chord. Also with `alt+h` unbound in the config: the
    overview's own chord still moves the selection.
26. **Arrows point where they go** (`9e14a6d`): in the stack, `↑` `↓`
    step workspaces; `←` and `→` do the same thing as each other: bump
    with one monitor, cross to the monitor that side with two, and
    neither goes in among the windows. `Space` goes in; there `←` `→`
    walk the columns and bump at the ends (or cross to a monitor that
    side), and `↑` / `↓` off the windows go to the workspace above or
    below. In the activities, `↓` goes into the windows and `↑` back
    out, as before. Say if crossing to the other monitor with `←` `→`
    in the stack feels wrong: it could bump instead.
27. **Defender** on the laptop: this build adds no Windows call the
    binary didn't already make (`GetKeyState` moved into the overview
    too, `WM_SYSKEYDOWN` and one more `WM_APP` message are handled).
    Note whether it runs clean.
28. **Centered pick on a strip** (`b74aaab`): stack layout, on a
    scrolling workspace wider than the monitor. `Space`, then `←` `→`:
    each picked column glides to the middle of the screen, the first
    and last too. `↑` or `↓` to another workspace: the strip glides
    back to what the monitor really shows. `Escape`: the real strip
    behind has not scrolled. Pointing at a column with the mouse must
    not move the strip, and the wheel sideways still pans by hand.
    `Enter` on a column far off screen: the zoom lands on the real
    windows without veering. A workspace no wider than the monitor
    doesn't move when its windows are picked. Same checks through a
    search (type a title on that strip).

Open, close, minimize and restore (`ANIMATION_REVIEW.md`). 29-31 need a
workspace with a tiling layout, not the scrolling one: there a new column
opens through the scroll, without `window_open`. Long durations are only
to see it; check each once more at 150 ms.

29. **Open in a tiling layout** (`4f37740`, `b5f3592`): `window_open` as
    default (`slide_right`) with `duration_ms: 1000`. Next to one open
    window, open Notepad with a long line of text and word wrap on,
    Explorer, and a browser. Each slides in already laid out for its
    tile: text wrapped to the tile's width, nothing cut off at the right
    or bottom, no strip of wallpaper or backdrop inside it. As it lands
    nothing reflows and there is no pause before its border settles and
    it takes keys. The real window never shows at its tile while its
    stand-in is still sliding in. `errors.log`: no "did not land within
    500ms". With Vesktop or another slow app, note whether the first
    frames show an empty or wrongly sized window (review finding 12).
30. **Fade open** (`0ad7cf1`): `window_open: { style: "fade",
    opacity_from: 0.0, duration_ms: 1000 }`. The window fades in over
    the second; before, nothing showed and it popped in at the end. With
    transparency on it ends at the window's own alpha, with no step.
31. **End of a zoom** (`983ef0d`): `window_open: { style: "zoom",
    duration_ms: 1000 }`, and restoring a minimized window from the
    taskbar (`window_minimize` default). The window lands exactly at
    full size: no last jump outward, no smaller copy fading over it, and
    its border and backdrop at full strength as it lands. With
    transparency on and off.
32. **Zoom, close and minimize pacing** (`7074e09`): `window_close: {
    enabled: true, style: "zoom", duration_ms: 500 }` and
    `window_minimize.duration_ms: 500`. Close and minimize a window
    beside another: the shrink is even, and the neighbour filling the
    gap moves in step with it, never a frame ahead.
33. **Restore during a minimize** (`9a23bf4`): `window_minimize.
    duration_ms: 1000`. `toggle-minimized` twice within the second; then
    minimize and click the window's taskbar button within the second.
    Both times the window ends in its tile with its border and backdrop.
    Not: an empty tile, or the window over its neighbours without a
    border. It grows from small again rather than turning round where
    it was (review finding 9).
34. **Window gone mid-close** (`9dfab1f`): `window_close: { enabled:
    true, duration_ms: 5000 }`. In a terminal run `ping -n 3 localhost
    >nul & taskkill /f /im notepad.exe`, then focus Notepad and press
    the close binding at once. When Notepad dies, its border and
    backdrop go with it instead of playing on around nothing.
    `errors.log`: no "Failed to send WM_CLOSE". Once is enough.
35. **Close with no stand-in** (`5f39127`): `window_close` on. Close
    Task Manager (elevated) with the close binding: it closes at once,
    not after the animation's length with nothing drawn.
36. **Slide restore** (`356f882`): `window_minimize: { style:
    "slide_right", duration_ms: 1000 }`. Minimize the rightmost window,
    restore it from the taskbar: no second copy of it beside its place
    or on the monitor to the right as the slide starts.
37. **To settle the review's proposed items**, a look each:
    - Finding 10: `window_minimize.duration_ms: 1000`, `toggle-minimized`
      twice, two seconds apart. Does the window zoom back in, or come
      back late with no zoom? `errors.log`: "did not land within 500ms"?
    - Finding 9: `window_open: { style: "zoom", duration_ms: 2000 }`,
      open two windows a second apart. Does the first restart from
      small, or sit at the wrong place until it lands?
    - Finding 11: `window_close: { enabled: true, duration_ms: 3000 }`,
      record a close and step through its first frames. Is there one
      with neither the window nor its stand-in?

If bars or flicker show, isolate them with `GLAZEWM_ISOLATE` (below), one
restart per flag: `noring`, `nobackdrop`, then `live`. With a long
`duration_ms` (e.g. 3000) for `scroll` and `workspace_switch`, one
screenshot catches a mid-animation frame.

## Live-check switch: `GLAZEWM_ISOLATE`

Inert unless set. A comma-separated list, read once at WM start (a warning
in the log says it is on):

- `nobackdrop`, `noring`: draw that copy of a moving window
  (pinned surrogates and peeks) at no opacity. Everything else, including
  the real overlays' handover, runs as usual, so an artifact that goes away
  with one of them is drawn by that copy.
- `live`: copy no backdrop, so windows with one move in surrogates and
  peeks that move with them, as before `b33c0ef8`.

Set it for one run, e.g. from PowerShell in the repo:

```powershell
$env:GLAZEWM_ISOLATE = "noring,nobackdrop"; .\target\release\glazewm.exe
```

## Next: one shared core for the switch and the scroll

`STAND_INS_PLAN.md`, revised 2026-11-04: what the stand-in systems
(move/resize, workspace switch, scroll) do now, and the steps to share
what the switch and the scroll do alike, each checked live before the
next.

1. Step 0, on the desktop: baseline recordings from a build of
   `ab822ab` or a later docs-only commit (list in the plan). Not done
   yet; every later step needs it.
2. Step 1: one core type under `WorkspaceSurrogate` and `WindowPeek`,
   and one function choosing `Live`. Moved code, no behaviour change.
   The full merge first planned is dropped: they share about 80 lines of
   wrappers, and differ in five ways the plan hadn't listed, one of them
   the hand-back step 2 makes one (findings under step 1).
3. Step 2: one overlay hand-over and hand-back on that core. Then decide
   whether folding the two types is worth it.
4. Step 3, decided after step 2: move/resize onto the core only if local
   fixes for "Still open" 1 and 5 won't do.

"Still open" 3 and 4 are side fixes, any time (4 is nearly free after
step 1).

## Still open

No app-specific fixes: the Vesktop (Electron) grow/shrink at the end of a
scroll is dropped.

1. Done, not checked live (see "Not checked live yet" 20): move/resize
   sessions skipped the companion wait for transparent windows. A session
   showing a companion is now opaque (`909b925`), covers its window until
   the companion is back (`72d6bc2`), waits up to 1 s (`1fbda69`), and
   picks up a companion made mid-animation (`2ddb24f`).
2. A long scroll can show a one-frame hole as a column enters mid-move
   (with or without shades).
3. Done, not checked live (see "Not checked live yet" 21): a peek reused
   for several moves kept its companion-wait start time, so a later
   hand-back could stop waiting at once. Every stand-in now starts the
   wait afresh as it begins a hand-back (`cab7cc8`).
4. Workspace switch no longer blends a backdrop between focus styles
   mid-slide: the copy uses the style the window ends with (incoming) or
   starts with (outgoing). Now cheap to fix: a surrogate could carry both
   styles' crops as peeks do and crossfade their opacities.
5. During a column resize the ring copy stretches with the window, so its
   vertical edges thin out until the column lands.
6. With scroll animations off, a partly visible column becoming fully
   visible drops its peek before its real window is uncloaked: possibly a
   blank frame.
7. **Shades windows blink white in workspace switches** (laptop,
   2026-11-04; not in scrolls). Cause unknown. First suspect, ruled out as
   the whole cause: the switch's companion wait only covers
   semi-transparent windows (its surrogate sits below its window,
   `workspace_surrogate.rs:294`, while the scroll raises its peek above,
   `scroll_view.rs:272`), but the blink stays with transparency at 99%,
   where the wait does cover. Left to tell apart, by when and on which
   window it shows: shades' copy not found as the switch starts (the
   surrogate then draws the uncolored window), the 200 ms wait
   (`HANDOFF_MAX_WAIT`, 1 s since `1fbda69`) timing out on a slower
   machine, or the focus change after the switch raising the focused
   window above its copy.
   Since `9dfecc5` an opaque window's surrogate covers it through the
   hand-back (`WorkspaceSurrogate::cover_window`), re-raised if focus
   puts the window back on top, so the first suspect is fixed for opaque
   windows and the last can only cost a frame; untested on the laptop.
   Since `5e76d21` and `5d7a47f` a switch surrogate draws a companion
   opaque and picks up one shades makes mid-switch (every 100 ms, and
   once more as the switch ends), so it covers or alpha-hides the window
   and waits for the copy at the hand-back. Left to tell apart: the 1 s
   wait timing out, shades making its copy only after the hand-back
   began, or a focus change during the wait setting a semi-transparent
   window's alpha again, which shows it over its surrogate (alpha-hidden
   windows aren't covered).

8. Open, close, minimize and restore: `ANIMATION_REVIEW.md` findings 9
   to 21, ranked. First: 9 (one driver with an entry track that survives
   a retarget), then 10 and 11 once check 37 says whether they show.

## Older items, not started

- Elevated windows (Task Manager, apps run as admin) can't be moved, cloaked
  or overlaid by a non-elevated WM: empty tile slot, 500 ms commit wait per
  resize, a warning per redraw. Decide: ignore/float them, or document "run
  as admin".
- Collapse the `wm-platform` facade/`platform_impl` split (pure forwarders
  since macOS was removed), about -500 lines.
- Replace the `auto-launch` dependency with a direct `HKCU\...\Run` write via
  `windows` (it also writes the exe path unquoted).
- Share the version-info block of the three `build.rs` files.

## Laptop

`test/smooth` is cut from `tryout/smooth` the same way. It was built and
tested for the Windows target off Windows only (`x86_64-pc-windows-gnu`,
tests under Wine), not natively. On the laptop, as below with
`test/smooth` for the branch name.

`test/smooth-shades` is cut from `tryout/smooth-shades` the same way,
checked the same way (Windows target and Wine, not natively).

The laptop runs `test/todo-cleanup`, re-cut from `tryout/todo-cleanup` after
each change following `CLAUDE.md`'s build-targets recipe, and built and
tested natively before it is pushed. On the laptop:

```powershell
git fetch origin; git checkout -B test/todo-cleanup origin/test/todo-cleanup
cargo build --release; cargo build --release -p wm-watcher
```

`cargo build --release` skips `wm-watcher` (not a default member).

## Checks

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Off Windows, check against the Windows target instead. With
`gcc-mingw-w64-x86-64` installed this also links the tests:

```bash
rustup target add x86_64-pc-windows-gnu
```

```bash
cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu -- -D warnings
```

```bash
cargo test --workspace --no-run --target x86_64-pc-windows-gnu
```

With `wine64` (or `wine`) installed, the linked test executables (paths printed by the
command above) run too, one test at a time: run in parallel, the `wm` ones
stall under Wine. Tests needing a real desktop (a message-only window, or
`Windows.UI.Composition`) fail there: eight of `wm-platform`'s as of
`b6c9c8a8` (`dispatcher` x4, `display`, `event_loop`, `ui_thread`,
`wallpaper_surface::tests::bakes_the_primary_monitor_end_to_end`).

```bash
WINEDEBUG=-all wine64 <test executable> --test-threads=1
```
