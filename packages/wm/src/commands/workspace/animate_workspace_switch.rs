//! Fade animation for workspace switching.
//!
//! Duration, easing and enabled-state are driven by
//! [`WorkspaceSwitchAnimationConfig`] so users can tune or disable the
//! effect from `config.yaml`:
//!
//! ```yaml
//! general:
//!   workspace_switch_animation:
//!     enabled: true          # set to false (or duration_ms: 0) to disable
//!     duration_ms: 180
//!     easing: ease_out_quad  # linear | ease_out_quad | ease_out_cubic | ease_in_out
//! ```

use std::time::Duration;

use tracing::warn;
use wm_common::{AnimationEasing, OpacityValue};
use wm_platform::NativeWindow;

use crate::{
  models::Workspace,
  traits::{CommonGetters, WindowGetters},
};

/// Number of opacity steps used for each fade direction.
const FADE_STEPS: u64 = 15;

/// Maps a linear progress value `t` (0–1) through the chosen easing curve.
#[inline]
fn apply_easing(t: f32, easing: &AnimationEasing) -> f32 {
  match easing {
    AnimationEasing::Linear => t,
    AnimationEasing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
    AnimationEasing::EaseOutCubic => {
      let inv = 1.0 - t;
      1.0 - inv * inv * inv
    }
    AnimationEasing::EaseInOut => {
      if t < 0.5 {
        2.0 * t * t
      } else {
        1.0 - (-2.0_f32 * t + 2.0).powi(2) / 2.0
      }
    }
  }
}

fn workspace_windows(workspace: &Workspace) -> Vec<NativeWindow> {
  workspace
    .descendants()
    .filter_map(|c| c.as_window_container().ok())
    .map(|w| w.native().clone())
    .collect()
}

/// Fades workspace windows from opaque to transparent, then restores
/// opacity asynchronously after platform_sync hides them (avoids flash).
///
/// `duration_ms` is the total length of the fade-out in milliseconds.
/// Passing `0` is a no-op (caller should guard with the `enabled` flag
/// from [`WorkspaceSwitchAnimationConfig`]).
pub fn fade_out_workspace(
  workspace: &Workspace,
  duration_ms: u64,
  easing: &AnimationEasing,
) {
  if duration_ms == 0 {
    return;
  }

  let windows = workspace_windows(workspace);

  if windows.is_empty() {
    return;
  }

  let step_delay = Duration::from_millis(duration_ms / FADE_STEPS);

  for step in (0..=FADE_STEPS).rev() {
    let t = 1.0 - (step as f32 / FADE_STEPS as f32);
    let eased = 1.0 - apply_easing(t, easing);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha = (255.0_f32 * eased).round().clamp(0.0, 255.0) as u8;
    let opacity = OpacityValue::from_alpha(alpha);

    for window in &windows {
      if let Err(e) = window.set_transparency(&opacity) {
        warn!("Fade-out: failed to set window transparency: {e}");
      }
    }

    std::thread::sleep(step_delay);
  }

  // Restore full opacity after platform_sync has hidden the windows so
  // they look normal if shown again later.
  tokio::task::spawn(async move {
    tokio::time::sleep(Duration::from_millis(100)).await;

    let full_opacity = OpacityValue::from_alpha(u8::MAX);
    for window in &windows {
      if let Err(e) = window.set_transparency(&full_opacity) {
        warn!(
          "Fade-out restore: failed to reset window transparency: {e}"
        );
      }
    }
  });
}

pub fn collect_fade_in_windows(workspace: &Workspace) -> Vec<NativeWindow> {
  workspace_windows(workspace)
}

/// Fades windows from 0 % to 100 % opacity using the chosen easing curve.
///
/// `duration_ms` is the total length of the fade-in in milliseconds.
/// Passing `0` immediately makes all windows fully opaque (no animation).
pub fn schedule_fade_in(
  windows: Vec<NativeWindow>,
  duration_ms: u64,
  easing: AnimationEasing,
) {
  if windows.is_empty() {
    return;
  }

  // With animation disabled just make sure windows are fully opaque.
  if duration_ms == 0 {
    let full_opacity = OpacityValue::from_alpha(u8::MAX);
    for window in &windows {
      if let Err(e) = window.set_transparency(&full_opacity) {
        warn!("Fade-in (instant): failed to reset transparency: {e}");
      }
    }
    return;
  }

  tokio::task::spawn(async move {
    let step_delay = Duration::from_millis(duration_ms / FADE_STEPS);
    let zero_opacity = OpacityValue::from_alpha(0);
    for window in &windows {
      if let Err(e) = window.set_transparency(&zero_opacity) {
        warn!("Fade-in: failed to set initial 0% transparency: {e}");
      }
    }

    tokio::time::sleep(Duration::from_millis(16)).await;

    for step in 1..=FADE_STEPS {
      let t = step as f32 / FADE_STEPS as f32;
      let eased = apply_easing(t, &easing);
      #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
      let alpha = (255.0_f32 * eased).round().clamp(0.0, 255.0) as u8;
      let opacity = OpacityValue::from_alpha(alpha);

      for window in &windows {
        if let Err(e) = window.set_transparency(&opacity) {
          warn!("Fade-in: failed to set window transparency: {e}");
        }
      }

      if step < FADE_STEPS {
        tokio::time::sleep(step_delay).await;
      }
    }
  });
}
