use std::time::{Duration, Instant};

use tracing::info;
use wm_common::WmEvent;

use crate::{
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// How long to wait before an already-urgent window can broadcast another
/// attention request.
///
/// A window that flashes until focused has the shell re-notify roughly
/// once a second for as long as it flashes, which would otherwise be
/// passed straight through to subscribers.
const ALERT_DEBOUNCE: Duration = Duration::from_secs(3);

/// Marks a window as urgent (or clears it), broadcasting the change.
///
/// Urgency is purely advisory; the WM doesn't act on it itself. It's meant
/// for status bars to highlight windows that asked for attention while
/// they were in the background.
///
/// Marking an already-urgent window is *not* a no-op: each request is a
/// fresh alert (e.g. a second chat message) that subscribers may want to
/// react to again, subject to `ALERT_DEBOUNCE`. Clearing an already-clear
/// window is, since that runs on every focus change.
pub fn set_window_urgency(
  window: &WindowContainer,
  is_urgent: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let alert_at = window.urgency_alert_at();

  if is_urgent {
    // A window you're looking at isn't waiting for attention. Windows can
    // flash while focused (e.g. to acknowledge a denied action), and a
    // window rule can fire on a focused window.
    if window.has_focus(None) {
      return Ok(());
    }

    // Deliberately leaves the existing timestamp in place, so that a
    // continuously flashing window broadcasts once per interval rather
    // than pushing the interval ahead of itself and never broadcasting.
    if alert_at.is_some_and(|at| at.elapsed() < ALERT_DEBOUNCE) {
      return Ok(());
    }

    window.set_urgency_alert_at(Some(Instant::now()));
    info!("Window requested attention: {window}");
  } else {
    if alert_at.is_none() {
      return Ok(());
    }

    window.set_urgency_alert_at(None);
    info!("Window urgency cleared: {window}");
  }

  state.emit_event(WmEvent::WindowUrgencyChanged {
    updated_window: window.to_dto()?,
    workspace_name: window
      .workspace()
      .map(|workspace| workspace.config().name),
  });

  Ok(())
}
