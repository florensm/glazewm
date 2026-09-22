use tracing::info;

use crate::{
  commands::{
    container::focus_container_by_id, workspace::focus_workspace,
  },
  models::WorkspaceTarget,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Focuses the window that most recently requested attention.
///
/// Switches to the window's workspace first if it isn't displayed. Does
/// nothing when no window is urgent.
pub fn focus_urgent_window(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let urgent_window = state
    .windows()
    .into_iter()
    .filter_map(|window| {
      window.urgency_alert_at().map(|alert_at| (alert_at, window))
    })
    .max_by_key(|(alert_at, _)| *alert_at)
    .map(|(_, window)| window);

  let Some(window) = urgent_window else {
    return Ok(());
  };

  info!("Focusing urgent window: {window}");

  // Focusing the window emits a focus event, which is what ultimately
  // clears its urgency.
  if let Some(workspace) = window.workspace() {
    if !workspace.is_displayed() {
      focus_workspace(
        WorkspaceTarget::Name(workspace.config().name),
        state,
        config,
      )?;
    }
  }

  focus_container_by_id(&window.id(), state)
}
