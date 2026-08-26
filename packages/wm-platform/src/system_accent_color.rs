use crate::Color;

#[cfg(target_os = "windows")]
use std::{
  sync::{Mutex, OnceLock},
  time::{Duration, Instant},
};

/// Minimum time between live `DwmGetColorizationColor` reads.
///
/// `system_accent_color` is called from `border_overlay_params_for`, which
/// runs on `platform_sync`'s per-tick overlay sync path (up to ~175Hz while
/// any animation is active, for every managed window) -- not just on the
/// occasional focus-change event this was designed around. Without a cache,
/// `use_accent_color` would turn every animation (resize, move,
/// workspace-switch, not just border transitions) into a live DWM syscall
/// storm. The accent color only ever changes when the user picks a new one
/// in Settings, so a coarse cache is imperceptible in practice.
#[cfg(target_os = "windows")]
const CACHE_TTL: Duration = Duration::from_millis(250);

#[cfg(target_os = "windows")]
static ACCENT_COLOR_CACHE: OnceLock<Mutex<Option<(Instant, Color)>>> =
  OnceLock::new();

/// Returns the OS's current accent/colorization color -- the color Windows
/// uses to tint title bars/taskbar/window borders when the user has that
/// personalization option enabled.
///
/// Cached for [`CACHE_TTL`] to keep this cheap on the per-tick overlay sync
/// hot path; see that constant's doc comment. Read failures are never
/// cached, so a transient error doesn't get stuck for the TTL window.
///
/// # Platform-specific
///
/// - **Windows:** reads `DwmGetColorizationColor`.
/// - **macOS:** unsupported; always returns an error.
#[cfg(target_os = "windows")]
pub fn system_accent_color() -> crate::Result<Color> {
  let cache = ACCENT_COLOR_CACHE.get_or_init(|| Mutex::new(None));

  // A poisoned lock means an earlier caller panicked while holding it,
  // which never happens in this function's small, panic-free critical
  // section, so `expect` here is effectively infallible.
  let mut guard = cache.lock().expect("accent color cache mutex poisoned");

  let now = Instant::now();
  if let Some((cached_at, color)) = *guard {
    if now.duration_since(cached_at) < CACHE_TTL {
      return Ok(color);
    }
  }

  let color = crate::platform_impl::system_accent_color()?;
  *guard = Some((now, color));
  Ok(color)
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
