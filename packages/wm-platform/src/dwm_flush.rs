use crate::perf;

/// Waits for the next DWM composition frame to complete.
///
/// Used to pace animation ticks to the compositor's frames without
/// timer-resolution jitter.
/// On non-Windows platforms this is a no-op.
pub fn dwm_flush() {
  let _scope = perf::scope(perf::Stage::DwmFlush);

  #[cfg(target_os = "windows")]
  unsafe {
    // SAFETY: No preconditions; `DwmFlush` is safe to call from any thread
    // and blocks until the next DWM composition frame is ready.
    let _ = windows::Win32::Graphics::Dwm::DwmFlush();
  }
}
