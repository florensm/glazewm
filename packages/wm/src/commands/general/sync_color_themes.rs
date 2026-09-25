use std::collections::HashSet;

use wm_common::WindowState;
use wm_platform::{NativeColorThemeOverlay, NativeWindowWindowsExt};

use crate::{
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Creates, updates, hides, or tears down each window's
/// [`NativeColorThemeOverlay`] to match
/// [`WmState::color_theme_windows`] and the loaded themes.
///
/// The overlay is hidden, not destroyed, while its window is minimized,
/// on a hidden workspace, or animating through a surrogate: surrogates
/// show the window's original colors, and the overlay can't follow them.
pub fn sync_color_themes(state: &mut WmState, config: &UserConfig) {
  if state.color_theme_windows.is_empty()
    && state.color_theme_overlays.is_empty()
  {
    return;
  }

  let redrawing_ids: HashSet<_> = state
    .windows_to_redraw()
    .iter()
    .map(CommonGetters::id)
    .collect();

  let mut wanted_ids = HashSet::new();

  for window in state.windows() {
    let id = window.id();

    let Some(theme) = state
      .color_theme_windows
      .get(&id)
      .and_then(|name| config.color_themes.get(name))
      .copied()
    else {
      continue;
    };

    if state.color_theme_failures.contains(&id) {
      continue;
    }

    wanted_ids.insert(id);

    let should_hide = matches!(window.state(), WindowState::Minimized)
      || !window.workspace().is_some_and(|ws| ws.is_displayed())
      || state.animation_manager.has_active_surrogate(&id)
      || state.animation_manager.has_live_ws_surrogate(&id)
      || state.animation_manager.has_live_resize_tracker(&id);

    if should_hide {
      if let Some(overlay) = state.color_theme_overlays.get_mut(&id) {
        if overlay.is_visible() {
          overlay.hide();
        }
      }
      continue;
    }

    let hwnd = window.native().hwnd();

    if let Some(overlay) = state.color_theme_overlays.get_mut(&id) {
      if overlay.has_failed() {
        // Already logged by the pipeline itself.
        state.color_theme_overlays.remove(&id);
        state.color_theme_failures.insert(id);
        continue;
      }

      overlay.set_theme(&theme);

      if redrawing_ids.contains(&id) || !overlay.is_visible() {
        match window.native().frame() {
          Ok(rect) => overlay.set_rect(&rect, hwnd),
          Err(err) => tracing::debug!(
            "Color theme overlay frame() query failed for {id}: {err}."
          ),
        }
      } else if let Err(err) = overlay.sync_z_order(hwnd) {
        tracing::debug!(
          "Color theme overlay z-order sync failed for {id}: {err}."
        );
      }

      continue;
    }

    let Ok(rect) = window.native().frame() else {
      continue;
    };

    match NativeColorThemeOverlay::create(hwnd, &rect, &theme, hwnd) {
      Ok(overlay) => {
        tracing::info!("Color theme applied to {window}.");
        state.color_theme_overlays.insert(id, overlay);
      }
      Err(err) => {
        tracing::warn!("Cannot apply color theme to {window}: {err}");
        state.color_theme_failures.insert(id);
      }
    }
  }

  state
    .color_theme_overlays
    .retain(|id, _| wanted_ids.contains(id));

  // Forget windows that are gone, so the maps don't grow unbounded.
  let live_ids: HashSet<_> =
    state.windows().iter().map(CommonGetters::id).collect();
  state
    .color_theme_windows
    .retain(|id, _| live_ids.contains(id));
  state
    .color_theme_failures
    .retain(|id| live_ids.contains(id));
}
