use tracing::warn;
use wm_common::WindowTransitionParams;

use crate::{
  animation::{peek_view::Leave, AwayTransition, MakingWay},
  commands::window::detach_window_for_close,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Closes `window`, behind its close animation when that is enabled.
///
/// The animated window is taken out of the layout right away, so its
/// siblings reflow while it plays; its peek cloaks it on its first tick,
/// and `peek_view::finish_leaving` sends `WM_CLOSE` once it ends.
pub fn close_window(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if animate_away(window, AwayTransition::Close, state, config) {
    // Detached below, before the OS reports it gone.
    crate::particles::note_close(state, window);

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

/// Covers `window`, still on screen, with a peek playing its close or
/// minimize animation (see `peek_view::leave_on_peek`).
///
/// Returns `false` when the animation is disabled, the window's rect can't
/// be resolved or no peek takes it (see `Leave`), leaving the window to go
/// at once, without one.
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

  // On a peek, pinned or live.
  let leaves = crate::animation::peek_view::leave_on_peek(
    state, window, &rect, transition, &params, config, is_focused,
  ) == Leave::Leaving;
  if leaves {
    state.pending_sync.make_way(MakingWay::leaving(&params));
  }

  leaves
}
