use anyhow::Context;
use wm_common::Layout;
use wm_platform::Direction;

use super::{focused_window_in, ColumnPlace, ColumnTarget};
use crate::{
  commands::container::{
    flatten_child_split_containers, move_container_within_tree,
    set_focused_descendant,
  },
  layout::scrolling::{active_column, column_focus_target, ViewRequest},
  models::Container,
  traits::{
    CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
  },
  wm_state::WmState,
};

/// Moves the column holding `container` one place along its strip in
/// `direction`. The view stays where it is while the column slides, then
/// scrolls to show it at its new place. Past the strip's end, the column
/// moves onto the monitor in `direction`.
///
/// Returns `false`, doing nothing, when `container` isn't in a scrolling
/// layout or `direction` isn't left or right.
pub fn move_column_in_direction(
  container: &Container,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<bool> {
  if !matches!(direction, Direction::Left | Direction::Right) {
    return Ok(false);
  }

  let Some(place) = ColumnPlace::of(container) else {
    return Ok(false);
  };

  match direction {
    Direction::Left if place.index > 0 => {
      shift_column(&place, place.index - 1, state)?;
    }
    Direction::Right if place.index + 1 < place.columns.len() => {
      shift_column(&place, place.index + 1, state)?;
    }
    _ => move_column_to_monitor(&place, direction, state)?,
  }

  Ok(true)
}

/// Moves the column holding `container` to where `target` says on its
/// strip.
///
/// Does nothing outside a scrolling layout.
pub fn move_column(
  container: &Container,
  target: ColumnTarget,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(place) = ColumnPlace::of(container) else {
    return Ok(());
  };

  let index = target.resolve(place.index, place.columns.len());
  shift_column(&place, index, state)
}

/// Moves `place`'s column to `index` on its strip.
fn shift_column(
  place: &ColumnPlace,
  index: usize,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if index == place.index {
    return Ok(());
  }

  // Shifting within the same parent takes the column out first, so the
  // column now at `index` ends up on the side it came from.
  move_container_within_tree(
    &place.column.clone().into(),
    &place.workspace.clone().into(),
    place.columns[index].index(),
    state,
  )?;

  place
    .workspace
    .request_view(ViewRequest::KeepCamera { from: place.index });

  state
    .pending_sync
    .queue_containers_to_redraw(place.workspace.tiling_children());

  Ok(())
}

/// Moves `place`'s column onto the workspace shown on the monitor in
/// `direction`, if there is one: right of its focused column if it is a
/// scrolling layout, or else on the side it comes from.
fn move_column_to_monitor(
  place: &ColumnPlace,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let monitor = place.workspace.monitor().context("No monitor.")?;

  let Some(target) = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|monitor| monitor.displayed_workspace())
  else {
    return Ok(());
  };

  // Crossing monitors may need DPI adjustments, and moves where the
  // windows would float.
  let has_dpi_difference =
    monitor.has_dpi_difference(&target.clone().into())?;
  let target_rect = target.to_rect()?;

  for window in place
    .column
    .self_and_descendants()
    .filter_map(|container| container.as_window_container().ok())
  {
    if has_dpi_difference {
      window.set_has_pending_dpi_adjustment(true);
    }

    window.set_own_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&target_rect),
    );
  }

  let is_focused = place.column.has_focus(None);
  let focus_target = column_focus_target(&place.column, &place.workspace);
  let size = place.column.tiling_size();

  let index = if target.layout() == Layout::Scrolling {
    active_column(&target)
      .map_or(target.child_count(), |column| column.index() + 1)
  } else if *direction == Direction::Right {
    0
  } else {
    target.child_count()
  };

  move_container_within_tree(
    &place.column.clone().into(),
    &target.clone().into(),
    index,
    state,
  )?;

  if target.layout() == Layout::Scrolling {
    place.column.set_tiling_size(size);
  } else {
    // A column of windows stacked one above the other joins a workspace
    // tiling that way as those windows.
    flatten_child_split_containers(&target.clone().into())?;
  }

  // The workspace left behind remembers the column to focus next.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(
      &focus_target,
      Some(&place.workspace.clone().into()),
    );
  }

  if is_focused {
    set_focused_descendant(&focused_window_in(&place.column), None);
  }

  state
    .pending_sync
    .queue_container_to_redraw(place.column.clone())
    .queue_containers_to_redraw(target.tiling_children())
    .queue_containers_to_redraw(place.workspace.tiling_children())
    .queue_focus_change()
    .queue_cursor_jump()
    .queue_workspace_to_reorder(target);

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};
  use wm_platform::{Direction, Rect};

  use super::{move_column, move_column_in_direction};
  use crate::{
    commands::{column::ColumnTarget, container::set_focused_descendant},
    layout::scrolling::sync_view,
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::{child_ids, MOCK_MONITOR_WIDTH},
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  /// A scrolling workspace of a window, a two-row column and a window,
  /// with the column focused.
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
    sync_view(&workspace).unwrap();

    (state, workspace, windows)
  }

  #[test]
  fn moves_the_whole_column_along_the_strip() {
    let (mut state, workspace, [first, top, bottom, last]) = setup();
    let column = top.parent().unwrap();

    move_column_in_direction(
      &bottom.clone().into(),
      &Direction::Left,
      &mut state,
    )
    .unwrap();
    assert_eq!(
      child_ids(&workspace),
      vec![column.id(), first.id(), last.id()]
    );
    assert_eq!(child_ids(&column), vec![top.id(), bottom.id()]);

    move_column_in_direction(
      &bottom.clone().into(),
      &Direction::Right,
      &mut state,
    )
    .unwrap();
    move_column_in_direction(
      &bottom.clone().into(),
      &Direction::Right,
      &mut state,
    )
    .unwrap();
    assert_eq!(
      child_ids(&workspace),
      vec![first.id(), last.id(), column.id()]
    );
  }

  #[test]
  fn the_view_stays_put_while_the_column_moves_into_view() {
    let (mut state, workspace, [_, _, bottom, _]) = setup();
    set_focused_descendant(&bottom.clone().into(), None);
    sync_view(&workspace).unwrap();
    let offset = workspace.scroll_offset();

    // Swapping with the first column keeps it in view: nothing scrolls.
    move_column_in_direction(
      &bottom.clone().into(),
      &Direction::Left,
      &mut state,
    )
    .unwrap();
    sync_view(&workspace).unwrap();
    assert_eq!(workspace.scroll_offset(), offset);
    assert_eq!(
      bottom.to_rect().unwrap().x(),
      workspace.to_rect().unwrap().x()
    );
  }

  #[test]
  fn moves_to_either_end_or_a_position() {
    let (mut state, workspace, [first, top, bottom, last]) = setup();
    let column = top.parent().unwrap();

    move_column(&bottom.clone().into(), ColumnTarget::Last, &mut state)
      .unwrap();
    assert_eq!(
      child_ids(&workspace),
      vec![first.id(), last.id(), column.id()]
    );

    move_column(&bottom.clone().into(), ColumnTarget::First, &mut state)
      .unwrap();
    assert_eq!(
      child_ids(&workspace),
      vec![column.id(), first.id(), last.id()]
    );

    move_column(&bottom.into(), ColumnTarget::Index(2), &mut state)
      .unwrap();
    assert_eq!(
      child_ids(&workspace),
      vec![first.id(), column.id(), last.id()]
    );
  }

  #[test]
  fn a_column_moved_onto_a_split_workspace_tiles_as_its_windows() {
    let (top, bottom) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let source = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.into()])
      .call();
    let target_window = TilingWindow::mock().call();
    let target = Workspace::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![target_window.clone().into()])
      .call();

    let bounds = Rect::from_xy(MOCK_MONITOR_WIDTH, 0, 1680, 1050);
    let mut state = WmState::mock(vec![
      Monitor::mock().workspaces(vec![source]).call(),
      Monitor::mock()
        .bounds(bounds.clone())
        .working_area(bounds)
        .workspaces(vec![target.clone()])
        .call(),
    ]);
    set_focused_descendant(&bottom.clone().into(), None);

    move_column_in_direction(
      &bottom.clone().into(),
      &Direction::Right,
      &mut state,
    )
    .unwrap();

    assert_eq!(
      child_ids(&target),
      vec![top.id(), bottom.id(), target_window.id()]
    );
  }

  #[test]
  fn past_the_strip_end_the_column_moves_to_the_next_monitor() {
    let (left, top, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let source = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![left.clone().into(), column.clone().into()])
      .call();
    let target_window = TilingWindow::mock().call();
    let target = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![target_window.clone().into()])
      .call();
    column.set_tiling_size(0.7);

    let bounds = Rect::from_xy(MOCK_MONITOR_WIDTH, 0, 1680, 1050);
    let mut state = WmState::mock(vec![
      Monitor::mock().workspaces(vec![source.clone()]).call(),
      Monitor::mock()
        .bounds(bounds.clone())
        .working_area(bounds)
        .workspaces(vec![target.clone()])
        .call(),
    ]);
    set_focused_descendant(&target_window.into(), None);
    set_focused_descendant(&bottom.clone().into(), None);

    move_column_in_direction(
      &bottom.clone().into(),
      &Direction::Right,
      &mut state,
    )
    .unwrap();

    assert_eq!(child_ids(&source), vec![left.id()]);
    assert_eq!(column.parent().unwrap().id(), target.id());
    assert_eq!(column.index(), 1, "right of the target's focused column");
    assert!((column.tiling_size() - 0.7).abs() < 1e-4, "keeps its width");
    assert_eq!(
      state.focused_container().map(|focused| focused.id()),
      Some(bottom.id())
    );
    assert_eq!(
      source.descendant_focus_order().next().map(|c| c.id()),
      Some(left.id())
    );
  }
}
