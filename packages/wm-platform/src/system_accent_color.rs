use crate::Color;

/// Returns the OS's current accent/colorization color -- the color Windows
/// uses to tint title bars/taskbar/window borders when the user has that
/// personalization option enabled.
///
/// # Platform-specific
///
/// - **Windows:** reads `DwmGetColorizationColor`.
/// - **macOS:** unsupported; always returns an error.
#[cfg(target_os = "windows")]
pub fn system_accent_color() -> crate::Result<Color> {
  crate::platform_impl::system_accent_color()
}

/// Returns the OS's current accent/colorization color -- the color Windows
/// uses to tint title bars/taskbar/window borders when the user has that
/// personalization option enabled.
///
/// # Platform-specific
///
/// - **Windows:** reads `DwmGetColorizationColor`.
/// - **macOS:** unsupported; always returns an error.
#[cfg(target_os = "macos")]
pub fn system_accent_color() -> crate::Result<Color> {
  Err(crate::Error::Platform(
    "system_accent_color is not supported on macOS.".to_string(),
  ))
}
