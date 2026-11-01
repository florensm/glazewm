#[cfg(target_os = "windows")]
use std::{
  sync::{Mutex, OnceLock},
  time::{Duration, Instant},
};

/// Minimum time between live registry reads. Called from the per-tick
/// overlay sync path, and the theme only changes when the user flips it
/// in Settings.
#[cfg(target_os = "windows")]
const CACHE_TTL: Duration = Duration::from_secs(1);

#[cfg(target_os = "windows")]
static LIGHT_THEME_CACHE: OnceLock<Mutex<Option<(Instant, bool)>>> =
  OnceLock::new();

/// Whether apps are set to the light theme, as opposed to dark.
///
/// Falls back to dark when the setting can't be read, which is also what
/// Windows assumes for apps when the value is missing.
///
/// # Platform-specific
///
/// - **Windows:** reads `AppsUseLightTheme` under `HKCU`.
/// - **macOS:** unsupported; always returns `false`.
#[cfg(target_os = "windows")]
#[must_use]
pub fn apps_use_light_theme() -> bool {
  let cache = LIGHT_THEME_CACHE.get_or_init(|| Mutex::new(None));

  // The cache only holds plain values, so a poisoned lock is still valid.
  let mut guard = cache
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner);

  let now = Instant::now();
  if let Some((cached_at, is_light)) = *guard {
    if now.duration_since(cached_at) < CACHE_TTL {
      return is_light;
    }
  }

  let is_light = crate::platform_impl::apps_use_light_theme()
    .inspect_err(|err| {
      tracing::debug!("Failed to read apps theme: {err}.");
    })
    .unwrap_or(false);

  *guard = Some((now, is_light));
  is_light
}

/// Whether apps are set to the light theme, as opposed to dark.
///
/// # Platform-specific
///
/// - **Windows:** reads `AppsUseLightTheme` under `HKCU`.
/// - **macOS:** unsupported; always returns `false`.
#[cfg(target_os = "macos")]
#[must_use]
pub fn apps_use_light_theme() -> bool {
  false
}
