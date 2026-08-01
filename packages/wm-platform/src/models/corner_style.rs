use serde::{Deserialize, Serialize};

/// Corner style of a window's frame.
///
/// # Platform-specific
///
/// Only has an effect on Windows 11.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CornerStyle {
  #[default]
  Default,
  Square,
  Rounded,
  SmallRounded,
}

impl CornerStyle {
  /// Approximate on-screen corner radius, in DIPs at 100% scaling, that
  /// Windows 11 renders for this style via
  /// `DWMWA_WINDOW_CORNER_PREFERENCE`.
  ///
  /// Windows exposes no API for the actual radius -- these are
  /// community-measured approximations. Used to match the composition-based
  /// acrylic overlay's own corner radius to the real managed window's
  /// rendered corners, so the two don't visually mismatch.
  #[must_use]
  pub fn approx_radius_px(&self) -> f32 {
    match self {
      CornerStyle::Square => 0.0,
      CornerStyle::SmallRounded => 4.0,
      CornerStyle::Default | CornerStyle::Rounded => 8.0,
    }
  }
}
