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

## Next: one way to animate windows

`STAND_INS_PLAN.md`: what the three stand-in systems (move/resize,
workspace switch, scroll) do now, what merging them improves, and the
steps to get there, each checked live before the next.

Step 0 (baseline recordings) isn't done yet; every later step needs it.

Step 1 checked 2026-11-03, no code changed: the duplication between
`WorkspaceSurrogate` and `WindowPeek` is real but thin (about 80 lines
of wrappers, and the `Live` choice made twice), and they differ in five
ways the plan didn't list, one of them the hand-back that step 2 makes
one. Recommended: shrink step 1 to a shared core plus one `Live` choice,
do step 2 on that core, and fold the two types after it if still wanted.
Waiting for the user's decision; details under step 1 in the plan.

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
