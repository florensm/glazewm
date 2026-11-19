//! Effects Windows draws on a window itself: its own border, title bar,
//! corners and transparency.

use tracing::debug;
use wm_common::{EffectSet, WindowEffectConfig};
use wm_platform::OpacityValue;

use crate::{
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
};

pub(super) fn apply_window_effects(
  window: &WindowContainer,
  set: EffectSet,
  config: &UserConfig,
) {
  let window_effects = &config.value.window_effects;

  let effect_config = window_effects.for_set(set);

  if window_effects.any(|effects| effects.hide_title_bar.enabled) {
    apply_hide_title_bar_effect(window, effect_config);
  }

  // `corner_radius` counts as opting in on its own, so setting only the
  // new knob still reaches `apply_corner_effect`.
  if window_effects.any(|effects| {
    effects.corner_style.enabled || effects.corner_radius.is_some()
  }) {
    apply_corner_effect(window, effect_config);
  }

  if window_effects.any(|effects| effects.transparency.enabled) {
    apply_transparency_effect(window, effect_config);
  }

  // Keyed off every effect set rather than `effect_config` alone. Keying
  // it per-class would restore the OS border every time a window lost
  // focus with `border` enabled for only one of the two, so the 1px line
  // would blink in and out on every focus change.
  apply_native_border_effect(window, window_effects);
}

/// Suppresses the window's own Windows 11 border while `GlazeWM` draws a
/// border overlay around it, and restores it otherwise.
///
/// The OS border sits on the window's outer edge, *inside* the overlay's
/// ring, so with both present they read as two concentric lines -- most
/// obvious while a window is resizing, where the overlay tracks the
/// animated rect and the OS line tracks the window.
fn apply_native_border_effect(
  window: &WindowContainer,
  window_effects: &wm_common::WindowEffectsConfig,
) {
  let drawing_own_border =
    window_effects.any(|effects| effects.border.enabled);

  // Errors are expected on Windows 10, which has no such border.
  _ = window.native().set_native_border_hidden(drawing_own_border);
}

fn apply_hide_title_bar_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  _ = window
    .native()
    .set_title_bar_visibility(!effect_config.hide_title_bar.enabled);
}

fn apply_corner_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  _ = window
    .native()
    .set_corner_style(&effect_config.resolved_corner_style());
}

fn apply_transparency_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let transparency = if effect_config.transparency.enabled {
    &effect_config.transparency.opacity
  } else {
    // Reset the transparency to default.
    &OpacityValue::from_alpha(u8::MAX)
  };

  debug!(
    "Applying transparency to {}: alpha={}.",
    window.id(),
    transparency.to_alpha()
  );

  _ = window.native().set_transparency(transparency);
}
