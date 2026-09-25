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

- A new popup shows its original colors for about one frame: Windows draws
  it before the capture can see it. Avoiding that means modifying the app's
  windows.
- Zen Browser still loses the theme on some clicks. Parked (WPF is the
  target).
- A few faint colored specks remain on some small glyphs.

## Next: follow window movement (phase 2)

Symptom: dragging a themed window leaves the overlay behind; during GlazeWM's
move/resize animations the overlay is hidden, so the window shows its
original (white) colors, then snaps back to themed.

### 1. Follow interactive drags

`packages/wm/src/events/handle_window_moved_or_resized.rs`: when the window
is managed and has an entry in `state.color_theme_overlays`, and the
animation manager doesn't own it (step 2), call
`overlay.set_rect(&frame_position, window.native().hwnd())` with the frame
the handler already queries. Must run in the active-drag branch too, since
dragged windows are skipped by `platform_sync`.

### 2. Follow move/resize animations

During an animation, a surrogate window (a DWM thumbnail in the *original*
colors) stands in for the cloaked real window.

1. **Fill layer** in `color_capture.rs`. Root the visual tree on a
   `ContainerVisual` with a `SpriteVisual` + `CompositionColorBrush` fill at
   the bottom (relative size 1×1, hidden by default) and the existing
   swap-chain sprite on top. Expose `ThemedCapture::set_fill(Option<Color>)`,
   forwarded by `NativeColorThemeOverlay::set_fill`. It covers the area the
   last themed frame doesn't while the window grows, where the surrogate
   paints its sampled edge color.
2. **`ColorTheme::apply_color(Color) -> Color`** in `color_theme.rs`:
   converts to `[f32; 3]`, calls `apply`, converts back. Add a unit test.
3. **Placement accessor** on `AnimationManager`
   (`packages/wm/src/animation/manager.rs`):
   ```rust
   pub enum ColorThemePlacement {
     /// Follow the move/resize surrogate.
     Following { surrogate: HWND, rect: Rect, fill: Option<Color> },
     /// The surrogate fades out above the real window, now at its final rect.
     FadingOut { surrogate: HWND },
     /// Workspace switch, close, minimize: stay hidden.
     Hidden,
   }
   pub fn color_theme_placement(&self, id: &Uuid) -> Option<ColorThemePlacement>
   ```
   - `resize_sessions[id]` → `Following` from `surrogate_hwnd()`,
     `current_rect()` and `edge_color()`; `Hidden` if either is `None`.
   - `pending_session_cleanup` entry for `id` → `FadingOut` from its
     session's `surrogate_hwnd()`.
   - `workspace_switch` / `pending_ws_cleanup` windows,
     `pending_close_windows`, `pending_minimize_windows` → `Hidden`.
   - Otherwise `None` (not animating).
4. **`sync_color_themes`**: replace the surrogate/tracker conditions in
   `should_hide` with the placement:
   - `Some(Hidden)` → hide.
   - `Some(Following { .. })` → `set_fill(fill.map(|c| theme.apply_color(c)))`
     and `set_rect(&rect, surrogate)`.
   - `Some(FadingOut { .. })` → `set_fill(None)` and
     `set_rect(&window.native().frame()?, surrogate)`, so the overlay sits
     above the fading surrogate instead of flashing white under it.
   - `None` → `set_fill(None)` and the current behavior.
   - Only create new overlays when the placement is `None`.
5. **Run it every animation frame**: call `sync_color_themes(state, config)`
   in `AnimationManager::update_internal` just before `drop(cleanup_scope)`
   (the end of the tick, after surrogates moved), under
   `#[cfg(target_os = "windows")]`.
6. **`resync_color_theme_z_order`**: skip windows whose placement is `Some`
   (anchoring them to the real window mid-animation is wrong).

### 3. Workspace-switch slides (optional, after 1–2)

Currently hidden during the slide. To follow: `WorkspaceSurrogate::hwnd()`
plus `unclipped_rect()`; the surrogate is clipped to the monitor, so the
frame needs an offset. Only worth it if the flash is noticeable.

## Verification (run on Windows)

Build needs the Windows SDK's `fxc.exe` (or `FXC=<path>`).

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
