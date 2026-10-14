use anyhow::Context;
use wm_common::WmEvent;

use super::{activate_workspace, sort_workspaces};
use crate::{
  commands::container::move_container_within_tree,
  models::{Monitor, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves `workspace` onto `target_monitor`, activating another workspace
/// on its origin monitor if it was the only one there.
pub fn move_workspace_to_monitor(
  workspace: &Workspace,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let origin_monitor = workspace.monitor().context("No monitor.")?;

  // Read before the move, after which a focused workspace is displayed
  // on the target monitor instead. A new monitor has none yet.
  let displayed_workspace = target_monitor.displayed_workspace();

  move_container_within_tree(
    &workspace.clone().into(),
    &target_monitor.clone().into(),
    target_monitor.child_count(),
    state,
  )?;

  let windows = workspace
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&workspace.to_rect()?),
    );
  }

  state
    .pending_sync
    .queue_container_to_redraw(workspace.clone());

  if let Some(displayed_workspace) = displayed_workspace {
    state
      .pending_sync
      .queue_container_to_redraw(displayed_workspace);
  }

  match origin_monitor.child_count() {
    0 => {
      // Prevent origin monitor from having no workspaces.
      activate_workspace(None, Some(origin_monitor), state, config)?;
    }
    _ => {
      // Redraw the workspace on the origin monitor.
      state.pending_sync.queue_container_to_redraw(
        origin_monitor
          .displayed_workspace()
          .context("No displayed workspace.")?,
      );
    }
  }

  sort_workspaces(target_monitor, config)?;

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace.to_dto()?,
  });

  Ok(())
}
