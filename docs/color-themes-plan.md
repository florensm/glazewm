# Color themes: status and next steps

Branch `feat/color-themes` (on top of `feat/dcomp`). Per-window "dark mode":
a window is captured with `Windows.Graphics.Capture`, recolored by a D3D11
pixel shader, and shown in a click-through overlay directly above it.

## Where things live

| Piece | File |
| --- | --- |
| Color math (CPU reference, unit-tested) | `packages/wm-platform/src/color_theme.rs` |
| Shader (direct port of the above) | `packages/wm-platform/shaders/color_theme.hlsl`, compiled by `build.rs` with `fxc` |
| Capture → shader → composition swap chain | `packages/wm-platform/src/platform_impl/windows/color_capture.rs` |
| Overlay window (layered, click-through, above its anchor) | `packages/wm-platform/src/native_color_theme_overlay.rs`, `overlay_window.rs`, `window_class.rs` |
| `color-themes.yaml` loading + hot reload | `packages/wm/src/color_themes.rs`, schema in `packages/wm-common/src/color_themes_config.rs` |
| `set-color-theme` command | `packages/wm-common/src/app_command.rs`, handled in `packages/wm/src/wm.rs` |
| Per-tick sync, popups, z-order resync | `packages/wm/src/commands/general/sync_color_themes.rs` |
| Sample themes | `resources/assets/sample-color-themes.yaml` |

## Done and verified on the dev machine

- Theme a window from a window rule or keybinding; `--off`; hot reload;
  invalid file keeps the previous themes.
- Gray ramp in OKLab, saturation threshold, soft-falloff overrides.
- Anti-aliased text: 5×3 neighborhood re-mix, ClearType fringes turned into
  grayscale AA, hairline ink estimation, light-on-dark coverage boost. Tests
  use pixels measured from a real WPF window.
- Popups (menus, dropdowns, context menus) of a themed window's process are
  themed too; overlays follow z-order changes (`EVENT_OBJECT_REORDER`), which
  fixed "theme disappears after a click" in WPF and Helium.
- One shared D3D11 device for all overlays.
- Skipped, logged once: elevated windows, windows with a display affinity.

## Known limitations (not planned)

- Images are themed like everything else. Keeping pictures (photos,
  avatars) in their own colors via UI Automation was built and removed
  again to keep the feature focused on text; it lives in commits
  `0a27f92`..`d4c5fb1` if it's picked up later. The showcase's Images tab
  is kept as a test page (its "keeps its colors" labels describe that
  feature).

- A new popup or dialog can still show its original colors for about one
  frame: Windows draws it before the WM hears of it. Its overlay now shows
  a themed fill as soon as it exists, before the capture is set up, and a
  window opening with an animation gets its overlay right away, as a fill
  following the animation (frames held back) instead of after it.
  Removing the last frame would mean modifying the app's windows.
- Always-on-top windows (floating with `shown_on_top: true`) have nothing
  above their band for the overlay to escape the popup lift to, so while
  topmost they own their overlay instead: Windows keeps owned windows
  above their owner and lifts them along. Owning across processes
  attaches input queues, so each managed window's overlay lives on its
  own message-pumping thread, never the WM's (which doesn't pump).
- Zen Browser still loses the theme on some clicks. Parked (WPF is the
  target).
- Gray text in dense glyphs (`k`, close stems) is grayed since `5779983`.
  Thin colored text (e.g. a blue link) recovers its ink's hue from the
  neighborhood's summed deviation from the paper, so its fringes no longer
  stay purple/cyan. WPF's fringes on colored text disagree in hue like
  neutral ones; they're told apart by how unbalanced that deviation is
  across channels (`wpf_link_keeps_its_color`, measured on a `#1976d2`
  link: gray pixels 80 -> 28, off-hue 8 -> 1).
- Overrides as known inks (`known_ink_remix`): where a window is an
  override's `from` over the page channel by channel, it's re-mixed from
  that exact ink. On the measured `#1976d2` link: mean error vs. the
  ideal 0.024 -> 0.000, worst pixel 0.33 -> 0.008; the rest of the
  screenshot (gray text, a photo) is unchanged.
- Edges where three colors meet (a bordered colored shape on a light page)
  can leave a stray off-color pixel: the fit allows per-channel coverage,
  the re-mix uses the mean.

## Phase 2: follow window movement (implemented, needs Windows verification)

Symptom it fixes: dragging a themed window left the overlay behind; during
GlazeWM's move/resize animations the overlay was hidden, so the window
showed its original (white) colors, then snapped back to themed.

- **Drags and app-initiated moves**: `handle_window_moved_or_resized`
  moves a shown overlay to the new frame whenever no animation owns the
  window (before the active-drag branch, so drags are covered).
- **`AnimationManager::color_theme_placement`** returns
  `Following { surrogate, rect, fill }` for a move/resize session,
  `FadingOut { surrogate }` for its fade-out tail, `Hidden` for workspace
  switches, close, minimize, zoom, or a session without a surrogate, and
  `None` otherwise. Close/minimize are checked first since they also live
  in `resize_sessions`.
- **Fill layer** (`color_capture.rs`): a `ContainerVisual` root with a
  `CompositionColorBrush` sprite beneath the swap-chain sprite.
  `NativeColorThemeOverlay::set_fill` takes the surrogate's edge color,
  themed via `ColorTheme::apply_color`. The fill only shows while a themed
  frame is shown, so a failed or not yet started pipeline never leaves a
  solid slab.
- **Same-frame movement**: `defer_color_theme_overlays` queues following
  overlays into `redraw_containers`' `SurrogateBatch`, so overlay and
  surrogate move in one `DeferWindowPos` transaction (a separate
  `SetWindowPos` can land a DWM frame later and show a white sliver of the
  surrogate at the leading edge). `sync_color_themes` then handles showing,
  re-anchoring, and the fill.
- **Per tick**: `sync_color_themes` also runs at the end of
  `update_internal`, since the fade-out tail doesn't always reach
  `platform_sync`.
- **Z-order resyncs** (`resync_color_theme_z_order`,
  `resync_settling_overlays`) anchor to the surrogate while one stands in
  for the window (`color_theme_anchor`).
- New overlays are only created when the placement is `None`.

Open questions for the Windows test:

- WGC may not deliver frames for the cloaked real window mid-animation; the
  last frame stays up, and the fill covers any growth.
- A session with `effect_opacity < 255` (transparent windows) gets an
  opaque overlay over its translucent surrogate.

## Next (optional): follow workspace-switch slides

Currently hidden during the slide. To follow: `WorkspaceSurrogate::hwnd()`
plus `unclipped_rect()`; the surrogate is clipped to the monitor, so the
frame needs an offset. Only worth it if the flash is noticeable.

## Verification (run on Windows)

Build needs the Windows SDK's `fxc.exe` (or `FXC=<path>`).

On Linux, `cargo check`/`clippy --target x86_64-pc-windows-msvc` work for
type-checking with `FXC` pointing at a stub that writes an empty file to
the `/Fo` path.

```
cargo fmt --all
cargo clippy -p wm-platform -p wm-common -p wm
cargo test -p wm -p wm-common
```

`wm-platform` tests use a harness-less runner and the full run hangs on an
event-loop test; run the color tests by name:

```
cargo test -p wm-platform --test test -- --list
cargo test -p wm-platform --test test -- --exact <name> --test-threads=1
```

Manual checks with a themed WPF window:

1. Drag it: the overlay follows without lag or white flashes.
2. Toggle floating/tiling, resize, move it to another tile: stays dark
   through the animation, including its end.
3. Open a dropdown, context menu, menu, and an owned dialog: all themed;
   the main window stays themed after they close.
4. Minimize/restore and switch workspaces: overlay hides and returns.
5. Edit `color-themes.yaml` while it's open: updates within ~1 second.
