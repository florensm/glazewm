use wm_common::Layout;

use super::set_focused_descendant;
use crate::{models::Container, traits::CommonGetters, wm_state::WmState};

/// Focuses the first column, or with `is_last` the last one, of the
/// scrolling layout `origin_container` is in, which scrolls it into view.
/// Focus goes to the window in the column that last had it.
///
/// Does nothing outside a scrolling layout.
pub fn focus_edge_column(
  origin_container: &Container,
  is_last: bool,
  state: &mut WmState,
) {
  let Some(workspace) = origin_container
    .workspace()
    .filter(|workspace| workspace.layout() == Layout::Scrolling)
  else {
    return;
  };

  let mut columns = workspace.tiling_children();
  let column = if is_last {
    columns.last()
  } else {
    columns.next()
  };

  if let Some(column) = column {
    let target = column
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| column.clone().into());

    set_focused_descendant(&target, None);
    state.pending_sync.queue_focus_change().queue_cursor_jump();
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};

  use super::focus_edge_column;
  use crate::{
    commands::container::set_focused_descendant,
    layout::scrolling::follow_focus,
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    traits::CommonGetters,
    wm_state::WmState,
  };

  /// Scrolling workspace of a window, a two-row column and a window, with
  /// the column's `bottom` row focused.
  fn setup() -> (WmState, Workspace, [TilingWindow; 4]) {
    let windows = [
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    ];
    let [first, top, bottom, last] = windows.clone();
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![first.into(), column.into(), last.into()])
      .call();
    let state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&bottom.into(), None);

    (state, workspace, windows)
  }

  fn focused_id(state: &WmState) -> Option<uuid::Uuid> {
    state.focused_container().map(|focused| focused.id())
  }

  #[test]
  fn focuses_the_first_and_last_columns() {
    let (mut state, workspace, [first, _, _, last]) = setup();

    focus_edge_column(&first.clone().into(), true, &mut state);
    assert_eq!(focused_id(&state), Some(last.id()));
    follow_focus(&workspace).unwrap();
    assert!(workspace.scroll_offset() > 0, "scrolls to the last column");

    focus_edge_column(&last.into(), false, &mut state);
    assert_eq!(focused_id(&state), Some(first.id()));
    follow_focus(&workspace).unwrap();
    assert_eq!(workspace.scroll_offset(), 0);
  }

  #[test]
  fn goes_to_the_window_a_column_last_focused() {
    let (top, bottom, other) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.into(), other.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);
    set_focused_descendant(&bottom.clone().into(), None);
    set_focused_descendant(&other.clone().into(), None);

    focus_edge_column(&other.into(), false, &mut state);
    assert_eq!(focused_id(&state), Some(bottom.id()));
  }

  #[test]
  fn does_nothing_outside_a_scrolling_layout() {
    let (first, second) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let workspace = Workspace::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);
    set_focused_descendant(&second.clone().into(), None);

    focus_edge_column(&second.clone().into(), false, &mut state);
    assert_eq!(focused_id(&state), Some(second.id()));
  }
}
