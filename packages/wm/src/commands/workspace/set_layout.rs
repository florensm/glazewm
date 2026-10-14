use wm_common::{Layout, WmEvent};

use crate::{
  layout::scrolling::default_column_size,
  models::Workspace,
  traits::{CommonGetters, TilingSizeGetters},
  wm_state::WmState,
};

/// Changes the layout of `workspace`.
///
/// The tree is kept as is; the new layout applies to windows added from
/// now on and to how rects are computed. Only the sizes of the
/// workspace's own children change when entering or leaving a scrolling
/// layout, whose columns have widths of their own.
pub fn set_layout(
  workspace: &Workspace,
  layout: Layout,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if workspace.layout() == layout {
    return Ok(());
  }

  let was_scrolling = workspace.layout() == Layout::Scrolling;
  workspace.set_layout(layout);

  let children = workspace.tiling_children().collect::<Vec<_>>();

  if layout == Layout::Scrolling {
    for child in &children {
      child.set_tiling_size(default_column_size(child, workspace));
    }
  } else if was_scrolling {
    #[allow(clippy::cast_precision_loss)]
    let size = 1. / children.len().max(1) as f32;

    for child in &children {
      child.set_tiling_size(size);
    }
  }

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
  use wm_common::{ColumnWidth, Layout, ScrollingLayoutConfig};
  use wm_platform::Rect;

  use super::set_layout;
  use crate::{
    models::{Monitor, TilingWindow, Workspace},
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  #[test]
  fn scrolling_gives_each_child_a_column_and_back() {
    let (first, second) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let workspace = Workspace::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let mut state = WmState::mock(vec![monitor]);

    first.set_tiling_size(0.7);
    second.set_tiling_size(0.3);
    set_layout(&workspace, Layout::Scrolling, &mut state).unwrap();
    assert!((first.tiling_size() - 0.5).abs() < 1e-4);
    assert!((second.tiling_size() - 0.5).abs() < 1e-4);

    set_layout(&workspace, Layout::Split, &mut state).unwrap();
    assert!((first.tiling_size() - 0.5).abs() < 1e-4);
    assert_eq!(workspace.scroll_offset(), 0);
  }

  #[test]
  fn changes_the_layout_and_keeps_the_tree() {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![window.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let mut state = WmState::mock(vec![monitor]);

    set_layout(&workspace, Layout::Dwindle, &mut state).unwrap();

    assert_eq!(workspace.layout(), Layout::Dwindle);
    assert_eq!(workspace.tiling_children().count(), 1);
  }

  #[test]
  fn window_width_columns_keep_the_widths_windows_had() {
    let window = |width| {
      TilingWindow::mock()
        .floating_placement(Rect::from_xy(0, 0, width, 400))
        .call()
    };
    let (narrow, wide) = (window(500), window(900));
    let workspace = Workspace::mock().call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    for column in [&narrow, &wide] {
      crate::commands::container::attach_container(
        &column.clone().into(),
        &workspace.clone().into(),
        None,
      )
      .unwrap();
    }
    workspace.set_scrolling_config(ScrollingLayoutConfig {
      column_width: ColumnWidth::Window,
      ..ScrollingLayoutConfig::default()
    });

    set_layout(&workspace, Layout::Scrolling, &mut state).unwrap();

    assert_eq!(narrow.to_rect().unwrap().width(), 500);
    assert_eq!(wide.to_rect().unwrap().width(), 900);
  }
}
