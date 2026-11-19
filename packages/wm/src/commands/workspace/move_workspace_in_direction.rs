use anyhow::Context;
use wm_platform::Direction;

use super::{deactivate_unused_workspace, move_workspace_to_monitor};
use crate::{
  models::Workspace, traits::CommonGetters, user_config::UserConfig,
  wm_state::WmState,
};

pub fn move_workspace_in_direction(
  workspace: &Workspace,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let origin_monitor = workspace.monitor().context("No monitor.")?;
  let target_monitor =
    state.monitor_in_direction(&origin_monitor, direction)?;

  if let Some(target_monitor) = target_monitor {
    move_workspace_to_monitor(workspace, &target_monitor, state, config)?;
    state.pending_sync.queue_cursor_jump();
    deactivate_unused_workspace(target_monitor.workspaces(), state)?;
  }

  Ok(())
}
