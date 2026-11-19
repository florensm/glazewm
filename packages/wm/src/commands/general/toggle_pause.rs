use wm_common::WmEvent;

use crate::wm_state::WmState;

/// Pauses or unpauses the WM.
pub fn toggle_pause(state: &mut WmState) {
  let is_paused = !state.is_paused;
  state.is_paused = is_paused;

  // Drags aren't followed while paused, so one in progress never ends.
  #[cfg(target_os = "windows")]
  if is_paused {
    crate::drop_indicator::hide_drop_indicator(state);
  }

  // Redraw full container tree on unpause.
  if !is_paused {
    state
      .pending_sync
      .queue_container_to_redraw(state.root_container.clone());
  }

  state.emit_event(WmEvent::PauseChanged { is_paused });
}
