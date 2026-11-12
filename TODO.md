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

38. **Border and backdrop of a fading close** (`15deb4d`): `window_close:
    { enabled: true, style: "none", opacity_to: 0.0, duration_ms: 3000 }`
    on a window with a border and a backdrop, the last one on its
    workspace. Border and backdrop fade with the window; before, they
    stayed at full strength for the three seconds and went in one
    frame. Same with `window_open: { style: "none", opacity_from: 0.0 }`:
    they fade in with it, and are at full strength as it lands. A `zoom`
    close looks as before.
39. **Which closes show** (reported 2026-11-10: the close animation shows
    or fails depending on whether it is the workspace's last window; the
    code only explains one direction). `window_close` on, `duration_ms:
    3000`, with the close binding: (a) the only window on a workspace,
    (b) one of two in a tiling layout, (c) a middle column in the
    scrolling layout. Expected from the code: (a) plays in full; in (b)
    and (c) it plays underneath while the neighbours cover it as they
    fill the gap, since a closing window is drawn below them by choice
    (`STAND_INS_PLAN.md` task 4). If (a) is the one that vanishes at
    once, that is a bug not found by reading: note the style, and whether
    `errors.log` has "No stand-in for" or "did not finish".
    To see more of a close beside neighbours without changing that
    order: `window_close: { style: "zoom", easing: "ease_out_cubic" }`.
    A window closed by its own X or Alt+F4 never animates: it is gone
    before the WM hears of it.
40. **Scroll after the peek split** (`56bf2a8`, a move with no behaviour
    change): `scroll: { duration_ms: 3000 }` on the scrolling layout,
    against task 0's baseline. Scroll from rest, press the scroll binding
    quickly several times, open a new column, close one. Each looks as in
    the baseline: peeks glide, borders and backdrops move with them,
    windows hand back at the end. Not: a blank column, a window shown
    twice, a border left behind.
41. **Entry track and the `peeks` switch** (`STAND_INS_PLAN.md` task 2:
    maths, scaled drawing and the switch, none of it used yet). Once with
    `GLAZEWM_ISOLATE` unset and once with `GLAZEWM_ISOLATE=peeks`: open,
    close, minimize and restore a window in a tiling layout, scroll a
    column. All look as before; with `peeks` set, the log's start-up
    warning lists `peeks: true` and nothing else differs.
42. **Open on peeks, tiling layouts** (`STAND_INS_PLAN.md` 3a, the gate
    for everything after it). On a tiling-layout workspace, with
    `GLAZEWM_ISOLATE=peeks`, open a window with each `window_open` style
    (`zoom`, each `slide_*`, `none` with `opacity_from` below 1) at
    `duration_ms: 2000` and at 150. Compare with task 0's baseline and
    with the same build with the switch unset. Include a see-through
    window with a backdrop and one themed by shades. Should show:
    - the window enters with its border and backdrop moving and fading
      with it, never a frame apart;
    - no flash, white frame or lighter look as a see-through or
      shades-themed window lands;
    - nothing shown twice or not at all at the start or the end;
    - opening a second window mid-way neither restarts nor misplaces the
      first;
    - neighbours making room stay in step with it.

    Then, each while a window is still opening at 2000 ms: close it, move
    it to another workspace, switch workspace. Nothing of it may stay on
    screen, and it must be visible and usable when you come back to it.
    Windows whose peek would be `Live` (rounded corners, or parallax not
    1) keep the old path (a restore too, before task 5). Keep `perf.log`
    (`GLAZEWM_PERF=1`) of one run with the switch on and one with it off:
    `tick ms/frame` during an open beside three windows, for task 7. If
    any point is worse than with the switch off and the cause is not a
    plain bug, stop and report with a screenshot; do not tune hand-over
    timings.
43. **Open on peeks, scrolling columns** (`STAND_INS_PLAN.md` 3b). On the
    scrolling workspace, with `GLAZEWM_ISOLATE=peeks`, open a new column
    with each `window_open` style (`zoom`, each `slide_*`, `none` with
    `opacity_from` below 1) at `duration_ms: 2000` and at 150, `scroll`
    left as it is. Also: one opened while the strip is still scrolling,
    one opened off the right edge. The column widens on `scroll`'s
    timing and enters by `window_open`'s; with the switch unset it only
    widens, as before. Same five points as 42, plus: it is handed back
    only once both have finished, with nothing left cloaked or on
    screen; closing it or switching workspace mid-way leaves no peek.
    Question for the owner: does a `slide_*` style read well on a column
    that is also widening? If not, columns take `zoom` and the fade only.
44. **Close and minimize on peeks** (`STAND_INS_PLAN.md` task 4). In a
    tiling layout and on the scrolling workspace, with
    `GLAZEWM_ISOLATE=peeks`, `window_close` on, at `duration_ms: 3000`
    and at 150, styles `none` (with `opacity_to` below 1), `zoom` and
    one `slide_*`; the same for `window_minimize` (`toggle-minimized`).
    Run 32, 34 and 35 again with the switch on, then:
    - the last window of a workspace, and one with a neighbour: both
      play; the second is covered as the neighbour grows over it, never
      before (the owner's z-order: a closing window stays below);
    - border and backdrop fade or shrink with the window, never linger;
    - no frame at the start with neither window nor stand-in (review
      finding 11): step through the first frames of a 60 fps recording;
    - close a window still opening (`window_open` at 2000 ms): it turns
      round where it is, with no jump;
    - a refused close ("Save changes?" in Notepad): the window comes
      back, uncloaked;
    - nothing left on screen or cloaked afterwards, and the window is
      closed or minimized once the animation ends; exiting the WM
      mid-close leaves the window open and visible.

    Windows whose peek would be `Live` (rounded corners, or parallax not
    1), and a partly visible scrolling column at rest, keep the old path.
    A minimize waits for one composition (`DwmFlush`) before the window
    is minimized; look for a hitch on pressing it.
45. **Restore on peeks** (`STAND_INS_PLAN.md` task 5). In a tiling layout
    and on the scrolling workspace, with `GLAZEWM_ISOLATE=peeks`,
    `window_minimize` at `duration_ms: 2000` and at 150, styles `zoom`,
    one `slide_*` and `none` with `opacity_to` below 1:
    - `toggle-minimized` twice, two seconds apart: the window comes back
      on its peek at once, growing, sliding or fading in, with no frame
      of it before the peek shows it, no Windows restore animation, and
      no "did not land within 500ms" in `errors.log` (review finding
      10);
    - a taskbar click on a minimized window: no frame showing it twice,
      or not at all, as the peek takes over (finding 8 on the new path);
    - both within the animation (`toggle-minimized` twice quickly, and
      minimize then click its taskbar button): the window turns round
      where it had shrunk to, with no jump, and its border and backdrop
      follow it; minimize it again on the way back: it turns round again;
    - on the scrolling workspace, a restored column widens in place as a
      new one does, and one restored mid-minimize turns round;
    - nothing left cloaked, minimized or on screen afterwards: no empty
      tile, no peek left over; the window is focused and usable;
    - restore a window minimized while other windows came above its
      place in the z-order (e.g. focus others, or a floating window over
      its tile): the restore plays on top of them, not under them or
      the wallpaper.

    Both paths, switch on and off: a restore now eases out into place
    (minimize's curve time-reversed, finding 15), where it sped up and
    stopped dead before. Floating windows, and windows whose peek would
    be `Live`, keep the old path.
46. **Moves on peeks** (`STAND_INS_PLAN.md` task 6). In a tiling layout
    and on the scrolling workspace, with `GLAZEWM_ISOLATE=peeks`,
    `window_move` at `duration_ms: 2000` and at 150, once with a spring
    easing; windows with a backdrop and a border, one in a stack:
    - swap two equal windows (`move --direction`): both glide across,
      border and backdrop with them, never a frame apart; no frame at
      the start showing a window twice or not at all, and no ring or
      backdrop left at rest beside its moving window;
    - the same swap pressed twice within the animation, and a move key
      held down: each press carries on from where the window is, at its
      speed, with no stop and restart in between;
    - a move beside a neighbour that resizes (e.g. move a window into a
      split, so its new neighbour shrinks): the moved one lands together
      with the resized one, edges in step all the way;
    - windows crossing others as they move pass over them, as on the
      old path;
    - on the scrolling workspace, swap two columns that are both on
      screen: both move on their peeks; resize one column while the
      others make room: the resized one keeps its `window_resize` style,
      the others move on peeks, and all land together;
    - close (`window_close` on), minimize, and a workspace switch during
      a move: nothing left on screen (no peek, ring or backdrop) and
      nothing left cloaked afterwards; the closed or minimized one turns
      into its close or minimize from where it was;
    - 15 and 16 again with the switch on.

    A move below `threshold_px`, a resize, a floating window (dragged or
    not), a tiling-to-floating change, a move onto another monitor, a
    window whose peek would be `Live`, and one still on an old-path
    animation keep the old path. A moving window whose next relayout
    also resizes it stays on its peek, which then shows more or less of
    it rather than stretching it, as a column does: look there for a
    gap at its edge.
47. **Floating windows open on peeks** (`STAND_INS_PLAN.md` task 6b).
    With `GLAZEWM_ISOLATE=peeks`, `window_open` at `duration_ms: 2000`
    and at 150, each style (`zoom`, each `slide_*`, `none` with
    `opacity_from` below 1): open a floating dialog (e.g. Notepad's
    Save As, or the Run box) and a floating app window (one made
    floating by a rule, over a tiled window and over another floating
    one). With the switch unset neither animates, as before. Should
    show:
    - the window enters on top of the windows it overlaps, with its
      border and backdrop, never a frame under another floating window;
    - it is handed back at its floating rect with nothing shown twice
      or not at all, and stays on top, focused, afterwards;
    - one opened while a tiled window is still entering: both play,
      neither restarts, and the tiled one hands back under it;
    - one placed partly off its monitor: cut at the monitor edge while
      it enters, as a slide is, then whole once handed back (with two
      monitors, look at the part on the other one);
    - one opened on a monitor other than the focused one plays there,
      paced by that monitor (on the fast monitor, not at 60 Hz);
    - close one mid-open (`window_close` on): it turns round where it
      is; minimize one mid-open: the same;
    - move one mid-open (`move --direction`, or a dialog that centres
      itself as it opens): its peek jumps to the new rect, as a
      floating move does unanimated, and keeps entering; nothing lands
      at the old rect.

    Fullscreen and maximized windows, windows whose peek would be
    `Live`, and dialogs or windows a rule leaves unmanaged or ignored
    open at once, as before. A floating window that opens on another
    monitor's workspace than the one it is on screen at is cut by its
    workspace's monitor: look for that with two monitors.
    Reported on the first run: the border appeared whole at the window's
    rect as it began to zoom. A ring made for an entering or leaving
    window is now made concealed, only its copy shown: look at the first
    frames of a `zoom` open at 3000 ms for a full-size border, then the
    same for a tiled window on an empty workspace.
48. **The peeks' frame before the catch-up** (`STAND_INS_PLAN.md` task
    7, a scheduling change). On the fast monitor, run 42's checks (an
    open beside three tiled windows, and a second opened mid-way) and
    46's (a swap, the same swap pressed twice, a move key held down),
    plus a scroll on the scrolling workspace, once with
    `GLAZEWM_ISOLATE=peeks` and once with `peeks,peeksfirst`, each with
    keys held so the event queue is busy (e.g. hold a focus key while
    another window opens). With `peeksfirst` nothing may look worse:
    - borders and backdrops never a frame apart from their windows, and
      fewer such frames than with `peeks` alone under a busy queue;
    - keys and IPC commands still answered during an animation, no
      later than with `peeks` alone (a peek opened or moved by a key
      during an animation may start a frame later: look for that);
    - nothing shown twice or not at all at a hand-back, nothing left
      cloaked or on screen afterwards;
    - a stack moving on the scrolling workspace: its tab bar keeps up
      with its peek (it is moved after the catch-up, not with the peeks).

    Desktop only: keep a `GLAZEWM_PERF=1` `perf.log` of each run. Read
    `peek_frame` (time from the timer's wake to the peeks' frame
    committed, frames with peeks only): `total / calls` is the mean,
    `worst` the spread's top, against the monitor's frame budget in the
    header; the plan asks whether it exceeds about a quarter of a frame
    or varies by more. With `peeksfirst`, the catch-up's own stages
    (`platform_sync` and below) count in the frame but not in `tick`;
    compare `tick` between the two runs, not `platform_sync`.
49. **Stretch resize on peeks, instant** (`STAND_INS_PLAN.md` task 8;
    the first build's wait of up to 100 ms is gone). Config
    `window_resize: { style: "stretch", duration_ms: 2000 }`, then 150;
    peeks on (the default). In a tiling layout: grow and shrink a tile by
    keyboard, open a window beside two others, close one of three, hold a
    resize key; on the scrolling workspace, resize a column, also the
    rightmost one. Apps: Notepad with wrapped text, a browser, Explorer,
    an Electron app, a slow one (Outlook).
    - First: no wallpaper and no bare backdrop anywhere, at the key press
      or after it, on any window, the rightmost included. The picture
      covers its tile's moving rect on every frame.
    - It moves on the frame after the key, as a pure move does.
    - A window that shrinks: as it starts, the part of it that will stay
      (from its top-left) is shown enlarged, by old size over new, then
      glides to 1:1. This step at the start is the one left; judge it
      first with a large change (open a window beside a full-width one).
      Its new content arrives without any change of scale.
    - A window that grows: its old content stretches with the tile; when
      its new content arrives it keeps that scale, cut at the tile's edge,
      and eases to fit within about 80 ms. No pop, no zoom, also with the
      new content arriving late in the motion (it then lands up to 80 ms
      later).
    - As it ends: no change of scale in the last frames or at the
      hand-back for a window that took its new size.
    - Known weak: an app slower than the animation (content after about
      150 ms) that grows ends stretched and is handed back so, the old
      path's zoom at the end; a window that refuses its tile's size is
      handed back with a jump.
    - Borders: scaled with the picture along a resized axis, with a small
      step in thickness as a growing window's content arrives.
    - A held resize key: glides on, each press resizing the apps again,
      with no wallpaper at the growing window's far edge between presses.
    - Compare with `GLAZEWM_ISOLATE=nopeeks` and with `style: "fill"`:
      both must look as they did before.
50. **Fill resize: seam line and fill color** (`9e2849b`, `092278d`,
    `1abbb0b`). Config `window_resize: { style: "fill", duration_ms:
    2000 }`, then 150; see-through windows (about 80%) with a backdrop
    and a ring. Resize two neighbouring tiles so that one grows on one
    axis and shrinks on the other (the fill then shows for the whole
    motion), with a dark app over a light part of the wallpaper.
    - No 1 to 2 px line showing wallpaper or backdrop between a window's
      content and its fill, along the bottom or right of the content. If
      one still shows, note its width in pixels and the display's scale:
      the fill reaches 2 px under the content's edge.
    - The fill is the window's color as it looks on screen, not lighter
      or more see-through than the content beside it. It is flat: the
      backdrop's detail doesn't show through it.
    - Press the resize key again before the first resize has finished,
      several times, right after another resize ended: the fill never
      takes the border ring's color. A window's first resize: see 52.
    - Open, close and the fade as a resize lands: the fill fades with the
      window, with no flash of solid color.
51. **Fill resize: every window hands off mid-way** (`efee1e2`). Config
    `window_resize: { style: "fill", duration_ms: 2000 }`. A tiling
    layout of four or more windows with live content (Notepad with
    wrapped text, a browser, Explorer): open a fifth window, then close
    one of five; also with `style: "stretch"`.
    - Every window's content reflows to its new size during the motion,
      the last ones in pass order included.
    - None reflows in one step after it lands, at rest.
52. **Fill resize: first fill and steady fill color** (`27c65bb`,
    `3b3595d`). Config `window_resize: { style: "fill", duration_ms:
    2000 }`; see-through windows (about 80%) with a backdrop and a ring.
    - Restart the WM, then grow a window: its first resize already shows
      a fill. Same for a window just opened (wait for its open animation
      to land), and for one opened with `window_open.enabled: false`
      after any other animation has played.
    - A see-through window's fill is a steady color close to the app's
      own as it looks on screen, the same from one resize to the next,
      never the ring's color or another window's.
    - Should not show: any extra pause or stutter at the key press, or
      right after an open lands.
53. **Floating open over floating windows shown on top** (`cf3f0e9`).
    Config `window_open: { duration_ms: 2000 }`, `floating.shown_on_top`
    on (the default), no `nopeeks`. Open two or three floating windows
    (dialogs, or apps set to float) overlapping one spot, then open
    another floating window over them.
    - It animates on top of them the whole way, and lands where it was
      drawn, with no jump above them as it lands.
    - A tiled window opening beneath them animates under them.
    - With `shown_on_top: false`: a new focused floating window animates
      over the other floating windows, as before.
54. **Steady gaps while tiles animate** (`afb82e7`). Config `window_move`
    and `window_resize` at `duration_ms: 2000`. Three or four tiles side
    by side: open a window among them, then close one.
    - The gaps between tiles stay one steady width throughout, never
      flickering a pixel wider or narrower frame to frame.
    - A swap of two equal tiles keeps their size the whole way.
55. **Overview backdrop baked at half size** (`c6d857d`). Default
    `overview.backdrop_blur: 40`. Open the overview on each display, also
    one with a taskbar on its top or left edge.
    - The blurred wallpaper looks as before: no banding in gradients, no
      darker or lighter line along a screen edge, and it lines up with
      the sharp wallpaper as it fades in (nothing shifts or grows).
    - Change the wallpaper, and move a display, with it open: it follows.
    - With `backdrop_blur: 4` it bakes at full size, as before.
56. **WPF windows lay out to the size they get** (`ea1c112d`, `d052f721`).
    The owner's WPF app (the one left partly white in a stack): restart
    the WM on this build first (a WPF window asked before it keeps the
    wrong maximum until then). Let it join a stack, tile it, resize it
    with the resize keybinds and with a grab (Alt+right-drag), each at a
    size other than its own.
    - It fills its tile or tab at once, with no part left white, and
      reflows on every keybind and grab resize; no mouse border drag
      needed.
    - `~/.glzr/glazewm/wpf.log` (always written): for that window, one
      `WPF window 0x... answered WM_SIZING, sized to WxH: its
      SizeToContent is off from now on.` and no second one. Instead
      `didn't answer WM_SIZING ...` or `WM_SIZING ... failed (error N)`
      say the request didn't reach it (5: the app is elevated); `... past
      the most it allows (MxN) ...` says the app caps its size, which no
      fix in the WM can make it fill. No line for the window at all: its
      top-level window isn't a WPF `HwndWrapper[...]` one, so it hosts WPF
      some other way; tell which app.
    - Still white with only the `answered` line: neither cause fixed here;
      note whether it was hidden in a stack just before (a tab switched
      to) or resized while shown.
    - A floating WPF dialog that sizes itself (e.g. one growing as a
      section expands) still does after a `move` command.
    - Laptop: `WM_SIZING` is now sent until answered rather than once;
      `WM_GETMINMAXINFO` (sent before too) now carries system metrics.
      Note whether Defender flags it.
57. **Windows joining a stack stay hidden** (`6371cccf`). Three tiles,
    one themed by shades: `stack-all`, then `unstack-all`, then
    `stack-absorb-neighbor`; again with `window_move` at
    `duration_ms: 2000`.
    - No joining window, nor its shades colours, flashes over the active
      tab as it joins.
    - Hold `cycle-stack-focus`'s key down on a stack of three: no
      background tab is left on screen afterwards, also for an app whose
      windows the log says "can't be cloaked, as the shell has no view of
      it".
58. **Shades on a stack resized while a tab is hidden** (shades
    `fix/glazewm-stacks`, not pushed). Stack two shades-themed windows,
    open and close a tile beside the stack so it shrinks and grows, then
    switch to the other tab.
    - No unthemed strip along its right or bottom edge as it shows.
56. **Fullscreen: covered tiles held, own transition** (this change).
    Three tiles side by side, the middle one focused; `toggle-fullscreen`,
    then again. Repeat at `window_fullscreen.duration_ms: 2000`.
    - The other two tiles never move or resize, going in or out.
    - The window grows to the monitor and back scaled (`stretch`), with
      no edge-color strip; `style: fill` brings the old look back.
    - Fullscreen, focus another tile (`focus --direction`): the tiles
      reflow then, the focused one on top. `toggle-floating` from
      fullscreen: they reflow too.
    - A stack beside it keeps its tab bar where its windows are.
    - Close the fullscreen window, and move it to another workspace:
      the tiles fill its place.
57. **Quick focus never bounces back**: `scroll` with a spring, press
    `focus --direction right` twice fast (and left twice), many times.
    Focus always ends two columns on, never back on the first. In the log,
    a bounce caught is `Ignoring focus event from an earlier focus
    change`; `Window manually focused` at that moment means another cause.
58. **Clicks while things move**: with a long `scroll` and
    `workspace_switch` `duration_ms` (e.g. 1500):
    - Click a column mid-scroll: it takes focus and the scroll heads to
      it. Nothing behind it (desktop, other windows) gets the click.
    - Click a window sliding in mid-switch: it takes focus once the slide
      lands. A click on one sliding out does nothing.
    - A floating window over the scrolling strip still takes its clicks
      mid-scroll, and the taskbar and bars always do.
    - At rest every click reaches its window at once, apart from
      `click_to_scroll`'s partly off-screen columns, as before.
    - Defender: the click hook now also goes up for each scroll and
      switch (same `SetWindowsHookExW` as `click_to_scroll`'s, no new
      Windows call), and down after.

59. **Grab and move** (`c33fddee`, `e1abb0c2`, `1f1e2e75`,
    `docs/grab-and-move.md`). Config `general.grab_and_move: { enabled:
    true }`, default animations. Two or three tiles, one stack, one
    floating window; then a scrolling workspace.
    - Alt+left-drag the middle of a tile, fast and in circles: the window
      stays under the cursor at the spot grabbed, with no visible trail,
      for the whole drag. It floats after 10px, the drop indicator shows,
      and it drops where a title-bar drag would. Same for a stacked window
      (stack and tab bar move together, drop onto another tab bar joins
      it) and a floating window. Onto the other monitor too, also one of
      another DPI.
    - Its border keeps up with it; on release it lands at once, with no
      catch-up. Grab a window right after clicking another one: it still
      follows to the end (log: no `Window grab ended` before release).
    - Nothing animates: the dragged window lands at once, and the tiles
      making room for it or closing up after it jump too. Same for a drag
      by the title bar.
    - Escape mid-move: the window goes back where it started and tiles
      back there. The app gets no Escape.
    - Release Alt before the button, and after: no menu bar opens in the
      app (e.g. Notepad), and no menu-bar underline stays. With `modifier:
      "win"`: no Start menu.
    - The app never gets the click: no text selected, no button pressed,
      no context menu after a right-drag.
60. **Grab and resize** (same changes). Alt+right-drag near each corner,
    each side, and the middle of a tile in a split of three.
    - The cursor moves as usual, and the edge or corner nearest the press
      stays under it, fast drags included; the opposite edge and the tile
      on the far side stay put, the neighbour on the dragged side resizes
      along with it during the drag, not only on release, and never holds
      the dragged window back.
    - On release the sizes stay as they are (no jump back).
    - A tile at a workspace edge doesn't move that edge.
    - Shrink past an app's minimum (e.g. Settings): the edge stops there,
      the cursor goes on.
    - Escape mid-resize: every size goes back.
    - A stacked window resizes its stack, tab bar included; a floating
      window just resizes, never below 250x140 unless it started smaller.
    - Dragging a tile's border (no modifier) moves only that edge too, and
      lands without animating.
    - A slow app as the neighbour (Firefox, VS Code): it may trail, the
      grabbed window doesn't, and nothing plays back after release.
61. **Resize past the screen in a scrolling layout** (same changes). Three
    columns of 33%, the rightmost at the screen's right edge.
    - Alt+right-drag its right half rightwards: it widens with the
      cursor; once the cursor stops at the screen's edge, pushing the
      mouse on keeps widening it, the view scrolls to keep its right edge
      at the screen's edge, other columns keep their widths and slide
      left. It stops at the view's width. Drag back: it narrows as the
      cursor comes back, the view left where it scrolled.
    - Same pressing on the screen's very last pixel: no cursor jump, and
      pushing on still widens it.
    - Its left half leftwards: its right edge stays, its left edge goes
      left, the view scrolls once it reaches the screen's left edge.
    - A column partly off screen: grabbing it focuses it without scrolling
      it into view first.
    - With `center_focused_column: "always"` the column grows both ways,
      staying centered.
    - With "Enhance pointer precision" on, the push past the edge goes at
      the mouse's raw speed, not the cursor's; say if that feels wrong.
62. **What isn't grabbed** (same change). Each of these gets Alt+click as
    usual (e.g. a selection or menu): a window ignored by a rule, a game or
    video fullscreen, a maximized window, Task Manager (elevated, WM not
    elevated), a tab bar, the overview, the taskbar; all windows while
    paused (`wm-toggle-pause`). Turning `enabled` off and reloading
    mid-drag: the drag ends, the app gets no release.
63. **Losing a grab** (same changes). Mid-resize, press Ctrl+Alt+Del (or
    Win+L) and release the button there: back on the desktop the grab has
    ended, the sizes are as left. Same with a UAC prompt opened mid-drag
    by a scheduled task, if handy. Close the grabbed window mid-resize
    (e.g. a timed app): the grab ends, the layout fills its place. If a
    grab is ever left stuck, pressing its button again ends it (log:
    `Window grab ended: Lost`).
    - Laptop: while `grab_and_move` is on (off by default), another
      low-level mouse hook swallows modifier+clicks (no longer any mouse
      moves), and the keyboard hook swallows Escape during a grab. Calls:
      `OpenProcessToken` and `GetTokenInformation` (elevation of
      windows' processes, once per window, only while on), a
      `SetWinEventHook` for desktop switches; since `1f1e2e75` no
      `SetCursorPos`, and a synchronous `SetWindowPos` from one short-lived
      thread per window being resized. Note whether Defender flags it,
      with the feature on and off.

If bars or flicker show, isolate them with `GLAZEWM_ISOLATE` (below), one
restart per flag: `noring`, `nobackdrop`, then `live`. With a long
`duration_ms` (e.g. 3000) for `scroll` and `workspace_switch`, one
screenshot catches a mid-animation frame.

First live run, 2026-11-11 (owner, laptop, `test/stand-ins` at
`1d82d8f`, no Defender flag): open beside a window, close of a last
window, close beside a neighbour (plays, covered by the neighbour as
decided), swap and swap back mid-way, minimize and restore, close while
opening all worked. Opens and closes looked about as before. A floating
open showed its border at the final rect from the start (`f1982c2`,
entry 47). Still unseen: entries 43 (columns), 45's taskbar cases, 48.

## Live-check switch: `GLAZEWM_ISOLATE`

Inert unless set. A comma-separated list, read once at WM start (a warning
in the log says it is on):

- `nobackdrop`, `noring`: draw that copy of a moving window
  (pinned surrogates and peeks) at no opacity. Everything else, including
  the real overlays' handover, runs as usual, so an artifact that goes away
  with one of them is drawn by that copy.
- `live`: copy no backdrop, so windows with one move in surrogates and
  peeks that move with them, as before `b33c0ef8`.
- `nopeeks`: draw open, close, minimize, restore and moves with
  `ResizeSession` again, as before `STAND_INS_PLAN.md` task 7b made
  pinned peeks the default. This is how to compare the two, and the
  fall-back if the new drawing misbehaves. On peeks by default: opens,
  floating ones too (3a, 3b, 6b), closes and minimizes (task 4),
  restores of tiled windows and columns (task 5), and moves that keep a
  tiled window's or column's size (task 6). With `nopeeks` floating
  windows open without an animation, as they used to. `peeks` is still
  accepted and changes nothing.

  Entries 42 to 48 above were written before the default changed: where
  one says "switch on" or `GLAZEWM_ISOLATE=peeks`, run without the
  variable; where it says "switch off", set `nopeeks`. None of them has
  been run yet, so the default is unseen code.
- `window_close` is enabled by default since task 7b. A close the app
  refuses (a "Save changes?" prompt drawn inside the window) brings the
  window back after up to two seconds (`ANIMATION_REVIEW.md` 16).
- `peeksfirst`: on an animation tick with peeks to draw, draw the
  peeks' frame first, then work off queued events, then the rest of the
  tick (`STAND_INS_PLAN.md` task 7). Off by default, and independent of
  `nopeeks`: scrolling columns move on peeks either way. Compare
  `peeksfirst` against no variable.

Set it for one run, e.g. from PowerShell in the repo:

```powershell
$env:GLAZEWM_ISOLATE = "noring,nobackdrop"; .\target\release\glazewm.exe
```

## Next: open, close and moves on pinned peeks

`STAND_INS_PLAN.md`, rewritten 2026-11-10 (fourth version) as a task
list a new session can follow. The scroll's per-window stand-in (`WindowPeek` and its
hand-overs) becomes the one for every tiled window, one kind of
animation at a time, each behind `GLAZEWM_ISOLATE=peeks` until seen
live.

0. Baseline recordings on the desktop. Not done; task 3's check needs
   them.
1. Split the stand-in driver out of `scroll_view.rs` (moved code only).
2. Entry track: frame maths, scaled and faded drawing, the switch.
3. Open on peeks: tiling layouts first (the gate: stop here if its live
   check is worse than the old path), then scrolling columns.
4. Close and minimize, both layouts; a closing window stays below its
   neighbours (your call). 5. Restore. 6. Moves.
7. Where a stand-in frame is drawn in the tick: measured first, changed
   only if you say so.
8. Resize: your decision between three options, after 3 to 7.
9. Remove the old paths, per kind, after you flip its default.

The plan's table "Why the overview is smoother" maps each of the
overview's four advantages to the task that takes it.

The switch and the scroll sharing a core (the plan's earlier steps 1
and 2) comes after task 6: upkeep, nothing visible.

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
9. **Shades on stack tabs the shell can't cloak** (2026-11-12, from
   code). Such tabs (the log warns "can't be cloaked") are hidden with
   `SW_HIDE`, and shades drops a hidden window's overlay (`app.rs`
   `target`). Switched back to, the tab shows unthemed until shades has
   made a new overlay, and no `CompanionCover` can bridge that: it needs
   a cloaked window (`unanimated_uncloak_rect`) and a companion on screen
   or cloaked (`Companion::find`). Fix needs both sides: shades keeping a
   hidden window's overlay shown but cloaked, as for a cloaked window,
   and the cover taking hidden windows too. Open question first: whether
   a WGC session of a window survives `SW_HIDE` and delivers frames once
   it is shown again.

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
