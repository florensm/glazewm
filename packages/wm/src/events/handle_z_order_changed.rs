#[cfg(target_os = "windows")]
use wm_platform::{NativeBackdropOverlay, NativeBorderOverlay};

#[cfg(target_os = "windows")]
use crate::commands::general::resync_overlay_z_order;
use crate::wm_state::WmState;

// LINT: The event is only emitted on Windows.
#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
pub fn handle_z_order_changed(state: &mut WmState) {
  // Backdrop first, then border, matching `platform_sync`: each goes
  // directly behind the window, so the border ends up between the two.
  #[cfg(target_os = "windows")]
  {
    resync_overlay_z_order::<NativeBackdropOverlay>(state);
    resync_overlay_z_order::<NativeBorderOverlay>(state);

    // E.g. the OS raising a window when a drag starts on it, or an app
    // raising its own main window over a stack.
    for bar in state.tab_bars.values() {
      bar.keep_behind_anchor();
    }
  }
}
