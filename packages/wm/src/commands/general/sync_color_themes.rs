use std::collections::HashSet;

use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{
  ColorTheme, NativeColorThemeOverlay, NativeWindow,
  NativeWindowWindowsExt, SurrogateBatch, WindowId, HWND,
};

use crate::{
  animation::ColorThemePlacement,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Creates, updates, hides, or tears down each window's
/// [`NativeColorThemeOverlay`] to match
/// [`WmState::color_theme_windows`] and the loaded themes.
///
/// The overlay is hidden, not destroyed, while its window is minimized
/// or on a hidden workspace. While a move/resize surrogate stands in for
/// the window, the overlay covers the surrogate instead, which shows the
/// window's original colors.
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
      .copied()
    else {
      continue;
    };

    if state.color_theme_failures.contains(&id) {
      continue;
    }

    wanted_ids.insert(id);

    let placement = state.animation_manager.color_theme_placement(&id);

    let should_hide = matches!(window.state(), WindowState::Minimized)
      || !window.workspace().is_some_and(|ws| ws.is_displayed())
      || placement == Some(ColorThemePlacement::Hidden);

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
      place_overlay(
        overlay,
        placement,
        &theme,
        &window.native(),
        redrawing_ids.contains(&id),
      );
      continue;
    }

    // Started once the animation is over: mid-animation, the capture's
    // first frames would lag behind the surrogate it has to cover.
    if placement.is_some() {
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

/// Places a shown window's overlay per its animation `placement`.
fn place_overlay(
  overlay: &mut NativeColorThemeOverlay,
  placement: Option<ColorThemePlacement>,
  theme: &ColorTheme,
  window: &NativeWindow,
  is_redrawing: bool,
) {
  match placement {
    Some(ColorThemePlacement::Following {
      surrogate,
      rect,
      fill,
    }) => {
      overlay.set_fill(fill.map(|color| theme.apply_color(color)));
      overlay.set_rect(&rect, surrogate);
    }
    // The window is already at its final rect, under the surrogate.
    Some(ColorThemePlacement::FadingOut { surrogate }) => {
      overlay.set_fill(None);
      set_rect_to_frame(overlay, window, surrogate);
    }
    Some(ColorThemePlacement::Hidden) => overlay.hide(),
    None => {
      overlay.set_fill(None);

      if is_redrawing || !overlay.is_visible() {
        set_rect_to_frame(overlay, window, window.hwnd());
      } else if let Err(err) = overlay.sync_z_order(window.hwnd()) {
        tracing::debug!("Color theme overlay z-order sync failed: {err}.");
      }
    }
  }
}

/// Moves `overlay` to `window`'s frame, directly above `anchor`.
fn set_rect_to_frame(
  overlay: &mut NativeColorThemeOverlay,
  window: &NativeWindow,
  anchor: HWND,
) {
  match window.frame() {
    Ok(rect) => overlay.set_rect(&rect, anchor),
    Err(err) => {
      tracing::debug!("Color theme overlay frame() query failed: {err}.");
    }
  }
}

/// Queues each overlay that follows a move/resize surrogate into the
/// surrogates' own `batch`, so both move in the same DWM frame and no
/// sliver of the unthemed surrogate shows at the leading edge.
///
/// Only moves overlays already shown; [`sync_color_themes`] decides the
/// rest later in the same pass.
pub fn defer_color_theme_overlays(
  state: &mut WmState,
  batch: &mut SurrogateBatch,
) {
  for (id, overlay) in &mut state.color_theme_overlays {
    if !overlay.is_visible() {
      continue;
    }

    if let Some(ColorThemePlacement::Following {
      surrogate, rect, ..
    }) = state.animation_manager.color_theme_placement(id)
    {
      overlay.defer_rect(batch, &rect, surrogate);
    }
  }
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
        .then_some((window.id(), *theme))
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
    let Some(anchor) =
      color_theme_anchor(state, &window.id(), &window.native())
    else {
      continue;
    };

    if let Some(overlay) = state.color_theme_overlays.get_mut(&window.id())
    {
      if overlay.is_visible() {
        if let Err(err) = overlay.sync_z_order(anchor) {
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

/// The window a shown color theme overlay belongs directly above: the
/// surrogate standing in for the window, if any, else the window itself.
///
/// `None` while the overlay is hidden for an animation.
pub fn color_theme_anchor(
  state: &WmState,
  id: &Uuid,
  window: &NativeWindow,
) -> Option<HWND> {
  match state.animation_manager.color_theme_placement(id) {
    None => Some(window.hwnd()),
    Some(
      ColorThemePlacement::Following { surrogate, .. }
      | ColorThemePlacement::FadingOut { surrogate },
    ) => Some(surrogate),
    Some(ColorThemePlacement::Hidden) => None,
  }
}
