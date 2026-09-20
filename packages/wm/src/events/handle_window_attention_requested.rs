use wm_platform::NativeWindow;

use crate::{
  commands::window::set_window_urgency, traits::CommonGetters,
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

  // A window that's already focused has nothing to notify about. Windows
  // can flash their taskbar button while focused (e.g. to acknowledge a
  // denied action).
  if window.has_focus(None) {
    return Ok(());
  }

  set_window_urgency(&window, true, state)
}
