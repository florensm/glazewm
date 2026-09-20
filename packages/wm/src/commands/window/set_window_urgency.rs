use tracing::info;
use wm_common::WmEvent;

use crate::{
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// Marks a window as urgent (or clears it), broadcasting the change.
///
/// Urgency is purely advisory; the WM doesn't act on it itself. It's meant
/// for status bars to highlight windows that asked for attention while
/// they were in the background.
///
/// Marking an already-urgent window is *not* a no-op: each request is a
/// fresh alert (e.g. a second chat message), and consumers want to react
/// to it again. Clearing an already-cleared window is, since that runs on
/// every focus change.
pub fn set_window_urgency(
  window: &WindowContainer,
  is_urgent: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if !is_urgent && !window.is_urgent() {
    return Ok(());
  }

  info!("Window urgency set to {is_urgent}: {window}");
  window.set_is_urgent(is_urgent);

  state.emit_event(WmEvent::WindowUrgencyChanged {
    updated_window: window.to_dto()?,
    workspace_name: window
      .workspace()
      .map(|workspace| workspace.config().name),
  });

  Ok(())
}
