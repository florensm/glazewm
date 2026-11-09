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
    in the middle; `↑` `↓` and the wheel move between them, `→` goes in
    among the windows. A scrolling strip reaches past both sides at full
    length (up to four monitors), stepping to a window pans just enough to
    show it, and the wheel turned sideways (or `Shift` + wheel) pans it
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

1. Move/resize sessions skip the companion wait for transparent windows
   (`effect_opacity < 255`), so shades can still flash there.
2. A long scroll can show a one-frame hole as a column enters mid-move
   (with or without shades).
3. A peek reused for several moves keeps its companion-wait start time
   (only `NativeSurrogate::revive` resets `HandoffWait`, and peeks aren't
   revived), so a later hand-back with a slow companion may stop waiting at
   once.
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
   (`HANDOFF_MAX_WAIT`) timing out on a slower machine, or the focus
   change after the switch raising the focused window above its copy.

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

With `wine64` installed, the linked test executables (paths printed by the
command above) run too, one test at a time: run in parallel, the `wm` ones
stall under Wine. Tests needing a real desktop (a message-only window, or
`Windows.UI.Composition`) fail there.

```bash
WINEDEBUG=-all wine64 <test executable> --test-threads=1
```
