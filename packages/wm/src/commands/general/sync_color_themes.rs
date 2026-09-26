use std::collections::HashSet;

use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{
  NativeColorThemeOverlay, NativeWindow, NativeWindowWindowsExt, WindowId,
};

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
    && state.color_theme_popups.is_empty()
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
      .cloned()
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

  sync_color_theme_popups(state, config);

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

/// An unmanaged popup (menu, dropdown, tooltip) of a themed window's
/// process, themed along with it.
pub struct ColorThemePopup {
  /// The themed window whose theme the popup follows.
  pub owner: Uuid,
  pub overlay: NativeColorThemeOverlay,
}

/// Themes `native` along with its process's themed window, if it is an
/// unmanaged, visible top-level popup of that process.
///
/// Called when a window is shown, or moves while unmanaged. Popups are
/// never managed, so window rules can't reach them, but they are part of
/// the app and would otherwise flash its original colors.
pub fn sync_color_theme_popup(
  native: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) {
  if state.color_theme_overlays.is_empty()
    || state.window_from_native(native).is_some()
  {
    return;
  }

  let hwnd = native.hwnd();

  let Ok(rect) = native.frame() else {
    return;
  };

  if let Some(popup) = state.color_theme_popups.get_mut(&hwnd.0) {
    popup.overlay.set_rect(&rect, hwnd);
    return;
  }

  if !native.is_top_level()
    || !native.is_visible().unwrap_or(false)
    || rect.width() <= 0
    || rect.height() <= 0
  {
    return;
  }

  let process_id = native.process_id();

  let Some((owner, theme)) =
    state.windows().into_iter().find_map(|window| {
      let overlay = state.color_theme_overlays.get(&window.id())?;
      let theme = config
        .color_themes
        .get(state.color_theme_windows.get(&window.id())?)?;

      (overlay.is_visible() && window.native().process_id() == process_id)
        .then_some((window.id(), theme.clone()))
    })
  else {
    return;
  };

  match NativeColorThemeOverlay::create(hwnd, &rect, &theme, hwnd) {
    Ok(overlay) => {
      state
        .color_theme_popups
        .insert(hwnd.0, ColorThemePopup { owner, overlay });
    }
    // Popups come and go constantly; one that can't be captured isn't
    // worth more than a debug line.
    Err(err) => {
      tracing::debug!("Cannot apply color theme to popup {hwnd:?}: {err}");
    }
  }
}

/// Stops theming a popup once it is hidden or destroyed.
pub fn remove_color_theme_popup(window_id: WindowId, state: &mut WmState) {
  state.color_theme_popups.remove(&window_id.0);
}

/// Keeps popups in step with their owner's theme, and drops those whose
/// owner is no longer themed and shown.
fn sync_color_theme_popups(state: &mut WmState, config: &UserConfig) {
  let WmState {
    color_theme_popups,
    color_theme_overlays,
    color_theme_windows,
    ..
  } = state;

  color_theme_popups.retain(|hwnd, popup| {
    let theme = color_theme_overlays
      .get(&popup.owner)
      .filter(|overlay| overlay.is_visible())
      .and(color_theme_windows.get(&popup.owner))
      .and_then(|name| config.color_themes.get(name));

    let Some(theme) = theme else {
      return false;
    };

    if popup.overlay.has_failed() {
      return false;
    }

    popup.overlay.set_theme(theme);

    if let Err(err) = popup.overlay.sync_z_order(wm_platform::HWND(*hwnd))
    {
      tracing::debug!("Color theme popup z-order sync failed: {err}.");
    }

    true
  });
}

/// Puts every shown color theme overlay back directly above its window.
///
/// Run on every restack: an app can raise its own window over the overlay
/// without any focus change (browsers on a click, WPF when a popup
/// closes), and nothing else would notice. The already-settled case is a
/// few `GetWindow` calls per overlay.
pub fn resync_color_theme_z_order(state: &mut WmState) {
  if state.color_theme_overlays.is_empty() {
    return;
  }

  for window in state.windows() {
    if let Some(overlay) = state.color_theme_overlays.get_mut(&window.id())
    {
      if overlay.is_visible() {
        if let Err(err) = overlay.sync_z_order(window.native().hwnd()) {
          tracing::debug!(
            "Color theme overlay z-order sync failed: {err}."
          );
        }
      }
    }
  }

  for (hwnd, popup) in &mut state.color_theme_popups {
    if let Err(err) = popup.overlay.sync_z_order(wm_platform::HWND(*hwnd))
    {
      tracing::debug!("Color theme popup z-order sync failed: {err}.");
    }
  }
}
