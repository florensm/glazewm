use anyhow::Context;
use uuid::Uuid;

use super::set_focused_descendant;
use crate::{
  commands::workspace::focus_workspace, models::WorkspaceTarget,
  traits::CommonGetters, user_config::UserConfig, wm_state::WmState,
};

/// Focuses the container with `container_id`, switching to its workspace
/// first when that isn't displayed.
pub fn focus_container_by_id(
  container_id: &Uuid,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focus_target = state
    .container_by_id(*container_id)
    .context("No container with given id")?;

  // Setting the focused descendant alone leaves a hidden workspace hidden.
  if let Some(workspace) = focus_target.workspace() {
    if !workspace.is_displayed() {
      focus_workspace(
        WorkspaceTarget::Name(workspace.config().name),
        state,
        config,
      )?;
    }
  }

  // Set focus to the target container.
  set_focused_descendant(&focus_target, None);
  state.pending_sync.queue_focus_change().queue_cursor_jump();

  Ok(())
}
