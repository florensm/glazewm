use wm_platform::NativeWindow;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

use crate::{
  commands::window::set_window_urgency,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Handles a window asking for the user's attention.
///
/// The window is only marked as urgent; focus is deliberately left alone
/// so that the user (or a status bar subscribed to
/// `window_urgency_changed`) decides when to switch to it.
#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
pub fn handle_window_attention_requested(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(window) = state.window_from_native(native_window) else {
    return Ok(());
  };

  // A window that's already focused has nothing to notify about. Windows
  // can flash their taskbar button while focused (e.g. to acknowledge a
  // denied action).
  if window.has_focus(None) {
    return Ok(());
  }

  set_window_urgency(&window, true, state)?;

  // Once the window is flagged, the flashing taskbar button is redundant
  // for anyone surfacing urgency elsewhere -- and it keeps an auto-hidden
  // taskbar open until the window is focused.
  #[cfg(target_os = "windows")]
  if config.value.general.suppress_taskbar_flash {
    window.native().stop_flashing();
  }

  Ok(())
}
