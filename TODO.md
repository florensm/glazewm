# TODO: `tryout/todo-cleanup`

State as of 2026-11-02, for picking this up in another session. Live testing
needs a Windows desktop session (GlazeWM is Windows-only); a cloud session
can build, lint and unit-test (see "Checks" at the end).

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
Windows whose backdrop can't be copied (rounded, or parallax != 1) keep the
old moving ("live") surrogate.

## Open issues (from live testing, user's config: border + transparency 80% + backdrop on, square corners)

1. **Vertical bars left and right of windows** during scrolling and, since
   `b33c0ef8`, workspace switches. Not seen before `b33c0ef8`, which moved
   backdrop windows onto pinned copies (so they now also get shadow and
   backdrop copies). Cause unknown. Suspects, in order:
   - The shadow copy is drawn at full strength; the real shadow of a window
     with a layered alpha is scaled by it (measured: bottom edge 230 opaque
     vs 235 at 80% over white). Scale the shadow copy by the window's effect
     opacity.
   - Shadow copies stacking over neighbouring windows' copies differently
     from the real z-order.
   - Backdrop crop or vignette copy not lined up with the window copy (the
     crop follows the visible frame mapped through `map_rect`; check
     `frame_rest`/`window_rest` passed in `WorkspaceSurrogate::carry_backdrop`
     and `WindowPeek::carry_backdrop`/`follow_window`).

   To find it: set a long `duration_ms` (e.g. 3000) for `scroll` and
   `workspace_switch` so a single screenshot catches a mid-animation frame,
   and toggle `border`/`transparency`/`backdrop` to isolate the copy.
2. **Fast keyboard scrolling flickers a lot** (slow moves much less). Each
   keypress moves focus, which flips two columns between the focused and
   unfocused backdrop style. `WindowPeek::carry_backdrop` then drops and
   re-registers the backdrop thumbnails and `NativeSurrogate::raise_thumbnail`
   unregisters and re-registers the window's own thumbnail: frames with no
   copy. Fix: register both styles' backdrop copies up front and switch by
   opacity, or move the backdrop copies into a separate window just below the
   peek so the window thumbnail never needs re-registering. The shadow copy is
   recreated on an active-state change too (`WindowPeek::carry_shadow`).
3. **Vesktop (Electron) grows/shrinks at the end of a scroll.** Likely
   Chromium not painting while cloaked, so DWM stretches its last frame if
   the window was resized (e.g. a minimum width larger than the column).
   Check the WM's rect (`glazewm query windows`) against `GetWindowRect`.
4. Workspace switch no longer blends a backdrop between focus styles
   mid-slide: the copy uses the style the window ends with (incoming) or
   starts with (outgoing).
5. During a column resize the ring copy stretches with the window, so its
   vertical edges thin out until the column lands.

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
- `config_files.rs` parses the main config file twice (three times without
  `include`).
- Warn on unknown config keys (the user's config still sets the removed
  `overlay_tracking` and `hide_method`, silently ignored).
- `close_duplicate_tabs` logs window titles at info level.
- ~4.7k "Failed to send window event: channel closed" lines at WM exit.

## Before the laptop gets it

`test/todo-cleanup` is stale (cut before these commits). Re-cut it from
`origin/tryout/todo-cleanup` following `CLAUDE.md`'s build-targets recipe.

## Checks

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Off Windows, type-check against the Windows target instead (no linking), e.g.
`rustup target add x86_64-pc-windows-gnu && cargo check --workspace --target x86_64-pc-windows-gnu`.
