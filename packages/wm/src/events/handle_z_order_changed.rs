use crate::{
  commands::general::resync_overlays_and_tab_bars, wm_state::WmState,
};

pub fn handle_z_order_changed(state: &mut WmState) {
  // E.g. the OS raising a window when a drag starts on it, or an app
  // raising its own main window over a stack.
  resync_overlays_and_tab_bars(state);
}
