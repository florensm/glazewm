use wm_common::{Layout, WmEvent};

use crate::{models::Workspace, wm_state::WmState};

/// Changes the layout of `workspace`.
///
/// The tree is kept as is; the new layout applies to windows added from
/// now on and to how rects are computed.
pub fn set_layout(
  workspace: &Workspace,
  layout: Layout,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if workspace.layout() == layout {
    return Ok(());
  }

  workspace.set_layout(layout);
  state
    .pending_sync
    .queue_container_to_redraw(workspace.clone());

  state.emit_event(WmEvent::LayoutChanged {
    workspace: workspace.to_dto()?,
    new_layout: layout,
  });

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::Layout;

  use super::set_layout;
  use crate::{
    models::{Monitor, TilingWindow, Workspace},
    traits::CommonGetters,
    wm_state::WmState,
  };

  #[test]
  fn changes_the_layout_and_keeps_the_tree() {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![window.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let mut state = WmState::mock(vec![monitor]);

    set_layout(&workspace, Layout::Bsp, &mut state).unwrap();

    assert_eq!(workspace.layout(), Layout::Bsp);
    assert_eq!(workspace.tiling_children().count(), 1);
  }
}
