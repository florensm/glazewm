use anyhow::Context;
use wm_platform::Direction;

use super::{
  focused_window_in, move_column_in_direction, rows, ColumnPlace,
};
use crate::{
  commands::container::set_focused_descendant,
  models::{Container, TilingContainer},
  traits::{CommonGetters, TilingSizeGetters},
  wm_state::WmState,
};

/// Swaps `window` with the window that last had focus in the column next
/// to its own in `direction`: each takes the other's place and size, and
/// focus moves along with `window`. Columns of a single window each swap
/// places like `move` does.
///
/// Does nothing outside a scrolling layout, for vertical directions, or
/// at the strip's end.
pub fn swap_window(
  window: &Container,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if !matches!(direction, Direction::Left | Direction::Right) {
    return Ok(());
  }

  let Some(place) = ColumnPlace::of(window) else {
    return Ok(());
  };
  let Some(target_column) = place.neighbour(direction).cloned() else {
    return Ok(());
  };

  if rows(&place.column).len() == 1 && rows(&target_column).len() == 1 {
    move_column_in_direction(window, direction, state)?;
    return Ok(());
  }

  let (Ok(source), Ok(target)) = (
    window.as_tiling_container(),
    focused_window_in(&target_column).as_tiling_container(),
  ) else {
    return Ok(());
  };

  swap_containers(&source, &target)?;
  set_focused_descendant(window, None);

  state
    .pending_sync
    .queue_container_to_redraw(place.workspace)
    .queue_focus_change()
    .queue_tab_bar_update();

  Ok(())
}

/// Swaps the places of `a` and `b` in the tree, including their place in
/// their parents' focus order and their sizes.
fn swap_containers(
  a: &TilingContainer,
  b: &TilingContainer,
) -> anyhow::Result<()> {
  let a_parent = a.parent().context("No parent.")?;
  let b_parent = b.parent().context("No parent.")?;
  let (a_index, b_index) = (a.index(), b.index());
  let (a_size, b_size) = (a.tiling_size(), b.tiling_size());

  if a_parent == b_parent {
    a_parent.borrow_children_mut().swap(a_index, b_index);
    return Ok(());
  }

  a_parent.borrow_children_mut()[a_index] = b.clone().into();
  b_parent.borrow_children_mut()[b_index] = a.clone().into();

  for (parent, from, to) in [(&a_parent, a, b), (&b_parent, b, a)] {
    for id in parent.borrow_child_focus_order_mut().iter_mut() {
      if *id == from.id() {
        *id = to.id();
      }
    }
  }

  *a.borrow_parent_mut() = Some(b_parent);
  *b.borrow_parent_mut() = Some(a_parent);
  a.set_tiling_size(b_size);
  b.set_tiling_size(a_size);

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};
  use wm_platform::Direction;

  use super::swap_window;
  use crate::{
    commands::container::set_focused_descendant,
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::child_ids,
    traits::{CommonGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  #[test]
  fn swaps_with_the_focused_window_of_the_next_column() {
    let (left, top, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![left.clone().into(), column.clone().into()])
      .call();
    column.set_tiling_size(0.7);
    left.set_tiling_size(0.4);
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&bottom.clone().into(), None);
    set_focused_descendant(&left.clone().into(), None);

    swap_window(&left.clone().into(), &Direction::Right, &mut state)
      .unwrap();

    assert_eq!(child_ids(&workspace), vec![bottom.id(), column.id()]);
    assert_eq!(child_ids(&column), vec![top.id(), left.id()]);
    assert!((bottom.tiling_size() - 0.4).abs() < 1e-4);
    assert_eq!(
      state.focused_container().map(|focused| focused.id()),
      Some(left.id())
    );
  }

  #[test]
  fn lone_windows_swap_columns() {
    let (left, right) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![left.clone().into(), right.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&left.clone().into(), None);

    swap_window(&left.clone().into(), &Direction::Right, &mut state)
      .unwrap();
    assert_eq!(child_ids(&workspace), vec![right.id(), left.id()]);
  }
}
