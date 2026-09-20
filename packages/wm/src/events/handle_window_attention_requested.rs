use wm_platform::NativeWindow;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

use crate::{
  commands::window::set_window_urgency, traits::WindowGetters,
  wm_state::WmState,
};

/// Handles a window asking for the user's attention.
///
/// The window is only marked as urgent; focus is deliberately left alone
/// so that the user (or a status bar subscribed to
/// `window_urgency_changed`) decides when to switch to it.
pub fn handle_window_attention_requested(
  native_window: &NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(window) = state.window_from_native(native_window) else {
    return Ok(());
  };

  set_window_urgency(&window, true, state)?;

  // The urgency flag replaces the flashing taskbar button, which is both
  // redundant once the window is flagged and holds an auto-hidden taskbar
  // open until the window is focused.
  #[cfg(target_os = "windows")]
  window.native().stop_flashing();

  Ok(())
}
