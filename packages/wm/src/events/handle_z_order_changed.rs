#[cfg(target_os = "windows")]
use crate::commands::general::resync_overlays_and_tab_bars;
use crate::wm_state::WmState;

// LINT: The event is only emitted on Windows.
#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
pub fn handle_z_order_changed(state: &mut WmState) {
  // E.g. the OS raising a window when a drag starts on it, or an app
  // raising its own main window over a stack.
  #[cfg(target_os = "windows")]
  resync_overlays_and_tab_bars(state);
}
