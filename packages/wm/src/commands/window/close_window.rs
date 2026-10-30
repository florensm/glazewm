use tracing::warn;
#[cfg(target_os = "windows")]
use wm_common::WindowTransitionParams;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

#[cfg(target_os = "windows")]
use crate::{
  animation::AwayTransition,
  commands::window::detach_window_for_close,
  traits::{CommonGetters, PositionGetters},
};
use crate::{
  models::WindowContainer, traits::WindowGetters, user_config::UserConfig,
  wm_state::WmState,
};

/// Closes `window`, behind its close animation when that is enabled.
///
/// The animated window is cloaked and taken out of the layout right away,
/// so its siblings reflow while it plays; `AnimationManager` sends
/// `WM_CLOSE` once it ends.
#[cfg_attr(
  not(target_os = "windows"),
  allow(unused_variables, clippy::unnecessary_wraps)
)]
pub fn close_window(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  #[cfg(target_os = "windows")]
  if animate_away(window, AwayTransition::Close, state, config) {
    // Cloaked only now that the surrogate covers it: the slow
    // `IApplicationView` cloak could otherwise hide the window a frame
    // before the surrogate is composited, exposing the desktop.
    let _ = window.native().set_cloaked(true);
    let close_config = &config.value.animations.window_close;
    let duration = std::time::Duration::from_millis(u64::from(
      close_config.easing.duration_ms(close_config.duration_ms),
    ));

    return detach_window_for_close(window, duration, state);
  }

  if let Err(err) = window.native().close() {
    warn!("Failed to close window: {:?}", err);
  }

  Ok(())
}

/// Covers `window`, still on screen, with a surrogate playing its close or
/// minimize animation.
///
/// Returns `false` when the animation is disabled, the window's rect can't
/// be resolved or the surrogate fails to capture, leaving the window to
/// go without it.
#[cfg(target_os = "windows")]
pub(crate) fn animate_away(
  window: &WindowContainer,
  transition: AwayTransition,
  state: &mut WmState,
  config: &UserConfig,
) -> bool {
  let animations = &config.value.animations;
  let (enabled, params) = match transition {
    AwayTransition::Close => (
      animations.window_close.enabled,
      WindowTransitionParams::from_close(&animations.window_close),
    ),
    AwayTransition::Minimize => (
      animations.window_minimize.enabled,
      WindowTransitionParams::from_minimize(&animations.window_minimize),
    ),
  };

  if !enabled {
    return false;
  }

  let Ok(rect) = window.to_rect().and_then(|rect| {
    window
      .total_border_delta()
      .map(|delta| rect.apply_delta(&delta, None))
  }) else {
    return false;
  };

  let is_focused = state
    .focused_container()
    .is_some_and(|focused| focused.id() == window.id());

  let (effect_opacity, corner_style, backdrop_overlay, border_overlay) =
    crate::commands::general::surrogate_effects_for(is_focused, config);

  state.animation_manager.start_away_animation(
    transition,
    window.id(),
    &rect,
    effect_opacity,
    corner_style,
    backdrop_overlay,
    border_overlay,
    &params,
    &window.native(),
  );

  state
    .animation_manager
    .has_away_animation(&window.id(), transition)
}
