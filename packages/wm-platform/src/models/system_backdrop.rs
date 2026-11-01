use serde::{Deserialize, Serialize};

/// Material DWM paints behind a window's transparent pixels
/// (`DWMWA_SYSTEMBACKDROP_TYPE`).
///
/// Only visible where the app leaves its surface unpainted (or extends
/// its frame into the client area), so text and other drawn content stay
/// fully opaque on top of it.
///
/// # Platform-specific
///
/// Only has an effect on Windows 11 22H2 (build 22621) and later.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SystemBackdrop {
  /// Lets DWM decide, which is what an app that never set one gets.
  #[default]
  Auto,
  /// No material.
  None,
  /// Tinted, blurred wallpaper (`DWMSBT_MAINWINDOW`).
  Mica,
  /// Live blur of whatever is behind the window
  /// (`DWMSBT_TRANSIENTWINDOW`).
  Acrylic,
  /// Mica Alt, a stronger-tinted Mica (`DWMSBT_TABBEDWINDOW`).
  Tabbed,
}
