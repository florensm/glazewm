use serde::{Deserialize, Serialize};

/// Backdrop material applied behind a window via DWM or SWCA.
///
/// `Acrylic` uses `ACCENT_ENABLE_ACRYLICBLURBEHIND` (SWCA) via a persistent
/// `NativeBlurOverlay` placed behind the managed window. This approach avoids
/// the `WS_EX_LAYERED`/SWCA conflict that arises when applying SWCA directly
/// to a layered window.
///
/// `Mica` and `MicaAlt` use `DWMWA_SYSTEMBACKDROP_TYPE` on the managed
/// window (Windows 11 22H2+). On older Windows versions, the DWM call fails
/// and a plain blur-behind via SWCA (`ACCENT_ENABLE_BLURBEHIND`) is applied
/// as a best-effort fallback.
///
/// # Platform-specific
///
/// Only has an effect on Windows 10 1803+ (Acrylic) or Windows 11 22H2+
/// (Mica/MicaAlt). On unsupported platforms/versions the effect is silently
/// skipped.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackdropStyle {
  /// Frosted-glass acrylic that blurs content behind the window.
  #[default]
  Acrylic,

  /// Mica material that samples the desktop wallpaper.
  Mica,

  /// Tabbed Mica variant with a slightly stronger wallpaper tint.
  MicaAlt,
}
