use wm_common::Layout;

use crate::{
  layout::scrolling::{active_column, Strip, ViewRequest},
  models::{Container, Workspace},
  traits::CommonGetters,
  wm_state::WmState,
};

/// The scrolling workspace `container` is on, if any.
fn scrolling_workspace(container: &Container) -> Option<Workspace> {
  container
    .workspace()
    .filter(|workspace| workspace.layout() == Layout::Scrolling)
}

/// Scrolls `container`'s scrolling layout to put its focused column in the
/// middle, once.
///
/// Does nothing outside a scrolling layout.
pub fn center_column(container: &Container, state: &mut WmState) {
  let Some(workspace) = scrolling_workspace(container) else {
    return;
  };

  workspace.request_view(ViewRequest::CenterActive);
  state.pending_sync.queue_container_to_redraw(workspace);
}

/// Scrolls `container`'s scrolling layout so that the columns fully on
/// screen, the focused one among them, are centered together.
///
/// Does nothing outside a scrolling layout, when the focused column isn't
/// fully on screen, or while the focused column is kept centered anyway.
pub fn center_visible_columns(
  container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(workspace) = scrolling_workspace(container) else {
    return Ok(());
  };

  let strip = Strip::new(&workspace)?;
  let is_centering =
    workspace.scrolling_config().keeps_centered(strip.len());

  let Some(active) = active_column(&workspace)
    .and_then(|column| strip.index_of(column.id()))
    .filter(|_| !is_centering)
  else {
    return Ok(());
  };

  let Some(visible) = strip.fully_visible(workspace.scroll_offset())
  else {
    return Ok(());
  };

  if !visible.contains(&active) {
    return Ok(());
  }

  let (first, last) = (*visible.start(), *visible.end());
  let block = strip.start(last) + strip.widths[last] - strip.start(first);
  let x = (strip.view_width - block) / 2 + strip.start(active)
    - strip.start(first);

  workspace.request_view(ViewRequest::PlaceActive { x });
  state.pending_sync.queue_container_to_redraw(workspace);

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::Layout;

  use super::{center_column, center_visible_columns};
  use crate::{
    commands::container::set_focused_descendant,
    layout::scrolling::sync_view,
    models::{Monitor, TilingWindow, Workspace},
    traits::{PositionGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  /// A scrolling workspace of windows of `sizes`, the first focused.
  fn setup(sizes: &[f32]) -> (WmState, Workspace, Vec<TilingWindow>) {
    let windows = sizes
      .iter()
      .map(|size| {
        let window = TilingWindow::mock().call();
        window.set_tiling_size(*size);
        window
      })
      .collect::<Vec<_>>();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(
        windows.iter().map(|w| w.clone().into()).collect(),
      )
      .call();
    let state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&windows[0].clone().into(), None);
    sync_view(&workspace).unwrap();

    (state, workspace, windows)
  }

  #[test]
  fn centers_the_focused_column() {
    let (mut state, workspace, windows) = setup(&[0.3, 0.3]);
    let center = workspace.to_rect().unwrap().center_point().x;

    center_column(&windows[0].clone().into(), &mut state);
    sync_view(&workspace).unwrap();
    assert_eq!(windows[0].to_rect().unwrap().center_point().x, center);
  }

  #[test]
  fn centers_the_columns_on_screen_together() {
    let (mut state, workspace, windows) = setup(&[0.3, 0.3, 0.6]);
    let rect = workspace.to_rect().unwrap();

    center_visible_columns(&windows[0].clone().into(), &mut state)
      .unwrap();
    sync_view(&workspace).unwrap();

    let (first, second) =
      (windows[0].to_rect().unwrap(), windows[1].to_rect().unwrap());
    let left_room = first.x() - rect.x();
    let right_room =
      rect.x() + rect.width() - (second.x() + second.width());
    assert!(
      (left_room - right_room).abs() <= 1,
      "{left_room} {right_room}"
    );
  }
}
