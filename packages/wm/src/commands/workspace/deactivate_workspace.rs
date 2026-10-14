use tracing::info;
use wm_common::WmEvent;

use crate::{
  commands::container::detach_container, models::Workspace,
  traits::CommonGetters, wm_state::WmState,
};

/// Deactivates a given workspace. This removes the container from its
/// parent monitor and emits a `WorkspaceDeactivated` event.
#[allow(clippy::needless_pass_by_value)]
pub fn deactivate_workspace(
  workspace: Workspace,
  state: &WmState,
) -> anyhow::Result<()> {
  info!("Deactivating workspace: {workspace}");

  detach_container(workspace.clone().into())?;

  state.emit_event(WmEvent::WorkspaceDeactivated {
    deactivated_id: workspace.id(),
    deactivated_name: workspace.config().name,
  });

  Ok(())
}

/// Deactivates the first of `workspaces` that is empty, hidden and not
/// kept alive. A monitor's only workspace is always displayed, so it is
/// never picked.
pub fn deactivate_unused_workspace(
  workspaces: Vec<Workspace>,
  state: &WmState,
) -> anyhow::Result<()> {
  let unused = workspaces.into_iter().find(|workspace| {
    !workspace.config().keep_alive
      && !workspace.has_children()
      && !workspace.is_displayed()
  });

  match unused {
    Some(workspace) => deactivate_workspace(workspace, state),
    None => Ok(()),
  }
}
