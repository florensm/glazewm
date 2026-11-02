# TODO: `tryout/todo-cleanup`

State as of the night of 2026-11-02, for picking this up in another
session. Live testing needs a Windows desktop session (GlazeWM is
Windows-only); a cloud session can build, lint and unit-test (see "Checks"
at the end). Nothing below marked fixed has been seen working on a real
desktop yet: everything from the overnight session was reasoned from the
code and checked by lints and unit tests only.

## Done on this branch

On top of `feat/todo-cleanup` (the macOS/`hide_method` cleanup):

- Review/audit fixes: infallible `NativeWindow` queries, dead API removed,
  `ResizeSession` dedup, hot-corner multi-monitor fix, 3 stale tests fixed,
  `errors.log` noise (IPC close handshake, frame fallback at debug, overlay
  access-denied once per window).
- `537d6437` Workspace-switch teardown spread over ticks instead of three
  `DwmFlush` calls in one tick (~16 ms stall, ~28 ms frame per switch).
- `835a9b0e` Window shadows carried through a workspace switch. Windows 11's
  shadow was measured and fitted as two Gaussian layers
  (`window_shadow.rs`), painted into a nine-grid composition surface.
- `9e0234cf`, `dc3c9c0e`, `73a8faa7` Scrolling layout peeks drawn the way the
  workspace switch draws: pinned to the monitor, with copies of the border
  ring and shadow (`overlay_copies.rs`, `window_peek.rs`, `scroll_view.rs`).
- `b33c0ef8` Backdrop copies: windows with a backdrop get pinned copies too
  (`backdrop_source.rs`): a crop of a monitor-wide backdrop per style plus a
  stretched vignette copy, both under the window's thumbnail.

How the pieces fit: every moving window is a DWM thumbnail in a surrogate
window that spans the monitor and never moves (`WorkspaceSurrogate` for
switches, `WindowPeek` for scrolling). Its ring, shadow and backdrop are more
thumbnails in the same window (`OverlayCopies`), so they all move in the same
DWM updates. The real overlays are handed over around them: covered by the
copy, hidden a composited frame later, shown again under the copy at the end
(`Handover` in `scroll_view.rs`, `WsCleanupStep` in `animation/manager.rs`).
The real window itself is swapped with its stand-in by its alpha
(`HiddenAlpha`): a semi-transparent window goes to alpha 0 before it is
cloaked or uncloaked, so the swap lands in one frame. Windows whose
backdrop can't be copied (rounded, or parallax != 1) keep the old moving
("live") surrogate.

## Fixed overnight (unverified live)

From the open issues of the last live test (user's config: border 2px,
transparency 80%, backdrop on for focused and other windows, square
corners, scrolling layout):

1. **Vertical bars left and right of windows** (scrolling and switches).
   - `8a1fc9c` Most likely cause: pinned copies drew the window's whole
     rect, its invisible resize borders (~7 px left/right/bottom) included,
     where the live surrogates they replaced started past them. A
     thumbnail of a layered (80% transparent) window can draw that border
     dark. Pinned copies now draw only the visible frame, at the same place.
   - `7e57367` The shadow copy is now drawn at the window's opacity, as DWM
     draws a layered window's shadow (was full strength, ~25% too dark).
   - `0c93737` At the start of a scroll, the real ring and backdrop were
     only concealed on the first motion tick, after their copies had moved:
     a frame could show them at rest beside the moved copies (thin vertical
     lines, slivers of backdrop at a column's trailing edge). They are now
     concealed right after the flush that shows their copies, before
     anything moves.
2. **Fast keyboard scrolling flickered.**
   - `5d43d2b` A focus change re-registered both columns' backdrop copies,
     the window's own thumbnail and a new shadow window mid-move. Peeks now
     carry both focus styles' copies from the start (a crop per style, both
     shadows, one full-strength vignette whose copy opacity carries the
     style's strength) and a focus change only swaps which show. Costs one
     more shadow window per peek, made with the peek.
   - `d6ba457`, `73f415e` A semi-transparent window was drawn twice (real
     window plus peek, which reads as nearly solid) for a few frames at the
     start of every move, and its shadow went missing for a frame. Peeks now
     hand over by alpha like the workspace switch: warmed up too faint to
     see, then thumbnail up and window alpha to 0 in one frame as it is
     cloaked; at the end the reverse right after the uncloak.
3. `e6241cb` A switch started while the last one was still tearing down
   could show a handed-back window without its backdrop until it was taken
   over.

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

## Test tomorrow

Desktop: exit GlazeWM first (it runs from `target\release` and locks the
file), `git pull` on `tryout/todo-cleanup`, `cargo build --release`, run.
Check `errors.log` for the new unknown-keys warning, then:

1. **Fast keyboard scroll** (hold or hammer the focus keys across several
   columns): no flicker, no column blinking out, shadows steady as focus
   moves.
2. **Slow single scroll steps** with the active window moving: no
   brightness flash at the start or end (window drawn twice), no lines or
   backdrop slivers at a column's trailing edge on the first frame.
3. **Workspace switch with backdrop on**: no vertical bars beside windows
   mid-slide; shadows as dark as the real ones at rest.
4. **A column scrolling in at the right edge**: drawn cut at the monitor
   edge, no strip of border or backdrop beside it.
5. **Vesktop at the end of a scroll** (still open, see below): watch for the
   grow/shrink, and compare `glazewm query windows` with its real size.
6. **Two quick workspace switches** (second one before the first settles):
   the windows coming back keep their backdrop throughout.
7. Exit the WM: `errors.log` shouldn't get thousands of "Failed to send
   window event" lines.
8. If bars or flicker remain, isolate them with `GLAZEWM_ISOLATE` (below),
   one restart per flag: `noshadow`, `noring`, `nobackdrop`, then `live`.
   With a long `duration_ms` (e.g. 3000) for `scroll` and
   `workspace_switch`, one screenshot catches a mid-animation frame.

Laptop: pull `test/todo-cleanup`, then build and test natively (the cloud
session could only cross-check it, see "Before the laptop gets it").

## Live-check switch: `GLAZEWM_ISOLATE`

Inert unless set. A comma-separated list, read once at WM start (a warning
in the log says it is on):

- `noshadow`, `nobackdrop`, `noring`: draw that copy of a moving window
  (pinned surrogates and peeks) at no opacity. Everything else, including
  the real overlays' handover, runs as usual, so an artifact that goes away
  with one of them is drawn by that copy.
- `live`: copy no backdrop, so windows with one move in surrogates and
  peeks that move with them, as before `b33c0ef8`.

Set it for one run, e.g. from PowerShell in the repo:

```powershell
$env:GLAZEWM_ISOLATE = "noshadow,noring"; .\target\release\glazewm.exe
```

## Still open

1. **Vesktop (Electron) grows/shrinks at the end of a scroll.** Likely
   Chromium not painting while cloaked, so DWM stretches its last frame if
   the window was resized (e.g. a minimum width larger than the column).
   Check the WM's rect (`glazewm query windows`) against `GetWindowRect`.
2. Workspace switch no longer blends a backdrop between focus styles
   mid-slide: the copy uses the style the window ends with (incoming) or
   starts with (outgoing). Now cheap to fix: a surrogate could carry both
   styles' crops as peeks do and crossfade their opacities.
3. During a column resize the ring copy stretches with the window, so its
   vertical edges thin out until the column lands.
4. A peek's ring copy is registered after its shadow, so the shadow doesn't
   fall over the ring as on screen (the workspace switch has it right).
   Barely visible (~15% shadow over a 2 px ring).
5. A peek reused for several moves keeps its companion-wait start time
   (`HandoffWait`), so a later hand-back with a slow companion (shades) may
   stop waiting at once.
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

## Before the laptop gets it

`test/todo-cleanup` was re-cut overnight from `origin/tryout/todo-cleanup`
following `CLAUDE.md`'s build-targets recipe. The cloud session had no
Windows toolchain, so `cargo build --release && cargo test --workspace`
couldn't run there; it ran the cross-target checks below instead. Confirm
the native build and tests on Windows before relying on it.

## Checks

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Off Windows, check against the Windows target instead. With
`gcc-mingw-w64-x86-64` installed this also links the tests, and with Wine
installed the unit tests run (tests that need a real desktop may fail):

```bash
rustup target add x86_64-pc-windows-gnu
```

```bash
cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu -- -D warnings
```

```bash
cargo test --workspace --no-run --target x86_64-pc-windows-gnu
```
