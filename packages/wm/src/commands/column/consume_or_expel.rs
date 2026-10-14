use wm_platform::Direction;

use super::{
  insert_column, join_column, row_of, rows, unwrap_single_row, ColumnPlace,
};
use crate::{
  commands::container::set_focused_descendant,
  models::Container,
  traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves `window` into the column next to its own in `direction`, as its
/// last window. Or, when its column holds other windows too, out of it
/// into a new column of the same width on that side.
///
/// Does nothing outside a scrolling layout, for vertical directions, or
/// for a lone window at the strip's end.
pub fn consume_or_expel_window(
  window: &Container,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if !matches!(direction, Direction::Left | Direction::Right) {
    return Ok(());
  }

  let Some(place) = ColumnPlace::of(window) else {
    return Ok(());
  };
  let Some(row) = row_of(window, &place.column) else {
    return Ok(());
  };

  if rows(&place.column).len() == 1 {
    let Some(target) = place.neighbour(direction).cloned() else {
      return Ok(());
    };

    join_column(&row, &target, state, config)?;
  } else {
    insert_column(
      &row,
      &place.workspace,
      &place.column,
      *direction == Direction::Left,
      place.column.tiling_size(),
      state,
    )?;
    unwrap_single_row(&place.column)?;
  }

  set_focused_descendant(window, None);
  state
    .pending_sync
    .queue_container_to_redraw(place.workspace)
    .queue_focus_change()
    .queue_tab_bar_update();

  Ok(())
}

/// Moves the top window of the column right of `container`'s into it, as
/// its last window. Focus stays where it is.
///
/// Does nothing outside a scrolling layout or at the strip's end.
pub fn consume_window_into_column(
  container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(place) = ColumnPlace::of(container) else {
    return Ok(());
  };
  let Some(source) = place.neighbour(&Direction::Right).cloned() else {
    return Ok(());
  };
  let Some(row) = rows(&source).into_iter().next() else {
    return Ok(());
  };

  let focused = state.focused_container();
  join_column(&row, &place.column, state, config)?;
  unwrap_single_row(&source)?;

  if let Some(focused) = focused.filter(|focused| !focused.is_detached()) {
    set_focused_descendant(&focused, None);
  }

  state
    .pending_sync
    .queue_container_to_redraw(place.workspace)
    .queue_focus_change()
    .queue_tab_bar_update();

  Ok(())
}

/// Moves the bottom window of `container`'s column out into a new column
/// of the same width right of it. Focus stays where it is.
///
/// Does nothing outside a scrolling layout or for a column of a single
/// window.
pub fn expel_window_from_column(
  container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(place) = ColumnPlace::of(container) else {
    return Ok(());
  };

  let rows = rows(&place.column);
  let (Some(row), true) = (rows.last(), rows.len() > 1) else {
    return Ok(());
  };

  let focused = state.focused_container();
  insert_column(
    row,
    &place.workspace,
    &place.column,
    false,
    place.column.tiling_size(),
    state,
  )?;
  unwrap_single_row(&place.column)?;

  if let Some(focused) = focused.filter(|focused| !focused.is_detached()) {
    set_focused_descendant(&focused, None);
  }

  state
    .pending_sync
    .queue_container_to_redraw(place.workspace)
    .queue_focus_change()
    .queue_tab_bar_update();

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{ColumnDisplay, Layout, TilingDirection};
  use wm_platform::Direction;

  use super::{
    consume_or_expel_window, consume_window_into_column,
    expel_window_from_column,
  };
  use crate::{
    commands::container::set_focused_descendant,
    layout::scrolling::sync_view,
    models::{
      Container, Monitor, SplitContainer, StackContainer, TilingWindow,
      Workspace,
    },
    test_utils::{child_ids, mock_config},
    traits::{CommonGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// Scrolling workspace of `columns`, the first `width` wide, with
  /// `focused` focused.
  fn setup(
    columns: Vec<crate::models::TilingContainer>,
    focused: &TilingWindow,
  ) -> (WmState, Workspace) {
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(columns)
      .call();
    let state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&focused.clone().into(), None);
    sync_view(&workspace).unwrap();

    (state, workspace)
  }

  fn column(rows: &[&TilingWindow]) -> SplitContainer {
    SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(
        rows.iter().map(|row| (*row).clone().into()).collect(),
      )
      .call()
  }

  fn consume(
    window: &TilingWindow,
    direction: &Direction,
    state: &mut WmState,
    config: &UserConfig,
  ) {
    consume_or_expel_window(
      &window.clone().into(),
      direction,
      state,
      config,
    )
    .unwrap();
  }

  fn focused_id(state: &WmState) -> Option<uuid::Uuid> {
    state.focused_container().map(|focused| focused.id())
  }

  #[test]
  fn a_lone_window_joins_the_column_beside_it_at_the_bottom() {
    let (left, right) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let (mut state, workspace) =
      setup(vec![left.clone().into(), right.clone().into()], &right);
    left.set_tiling_size(0.4);
    let config = mock_config();

    consume(&right, &Direction::Left, &mut state, &config);

    let column = left.parent().unwrap();
    assert_eq!(child_ids(&workspace), vec![column.id()]);
    assert_eq!(child_ids(&column), vec![left.id(), right.id()]);
    let column = column.as_tiling_container().unwrap();
    assert!((column.tiling_size() - 0.4).abs() < 1e-4, "keeps its width");
    assert_eq!(focused_id(&state), Some(right.id()));
  }

  #[test]
  fn a_window_of_a_column_is_expelled_into_one_of_its_own() {
    let (top, bottom, other) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let split = column(&[&top, &bottom]);
    let (mut state, workspace) =
      setup(vec![split.clone().into(), other.clone().into()], &top);
    split.set_tiling_size(0.6);
    let config = mock_config();

    consume(&top, &Direction::Right, &mut state, &config);

    // The column left with one window is that window again.
    assert!(split.is_detached());
    assert_eq!(
      child_ids(&workspace),
      vec![bottom.id(), top.id(), other.id()]
    );
    assert!((top.tiling_size() - 0.6).abs() < 1e-4);
    assert!((bottom.tiling_size() - 0.6).abs() < 1e-4);
    assert_eq!(focused_id(&state), Some(top.id()));
  }

  #[test]
  fn expelling_left_puts_the_new_column_before_its_own() {
    let (top, bottom) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let split = column(&[&top, &bottom]);
    let (mut state, workspace) = setup(vec![split.into()], &bottom);

    consume(&bottom, &Direction::Left, &mut state, &mock_config());
    assert_eq!(child_ids(&workspace), vec![bottom.id(), top.id()]);
  }

  #[test]
  fn a_lone_window_at_the_strip_end_stays() {
    let only = TilingWindow::mock().call();
    let (mut state, workspace) = setup(vec![only.clone().into()], &only);

    consume(&only, &Direction::Left, &mut state, &mock_config());
    assert_eq!(child_ids(&workspace), vec![only.id()]);
  }

  #[test]
  fn with_tabbed_columns_a_window_joins_as_a_tab() {
    let (left, right) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let (mut state, workspace) =
      setup(vec![left.clone().into(), right.clone().into()], &right);
    let mut config = mock_config();
    config.value.layouts.scrolling.default_column_display =
      ColumnDisplay::Tabbed;

    consume(&right, &Direction::Left, &mut state, &config);

    let stack = left.parent().unwrap();
    assert!(stack.as_stack().is_some());
    assert_eq!(child_ids(&workspace), vec![stack.id()]);
    assert_eq!(child_ids(&stack), vec![left.id(), right.id()]);
  }

  #[test]
  fn a_window_joins_a_stack_column_as_its_last_tab() {
    let (tab, other_tab, right) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let stack = StackContainer::mock()
      .tiling_containers(vec![
        tab.clone().into(),
        other_tab.clone().into(),
      ])
      .call();
    let (mut state, _) =
      setup(vec![stack.clone().into(), right.clone().into()], &right);

    consume(&right, &Direction::Left, &mut state, &mock_config());
    assert_eq!(
      child_ids(&stack),
      vec![tab.id(), other_tab.id(), right.id()]
    );
  }

  #[test]
  fn consuming_pulls_in_the_top_window_of_the_next_column() {
    let (left, top, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let split = column(&[&top, &bottom]);
    let (mut state, workspace) =
      setup(vec![left.clone().into(), split.into()], &left);

    consume_window_into_column(
      &left.clone().into(),
      &mut state,
      &mock_config(),
    )
    .unwrap();

    let column = left.parent().unwrap();
    assert_eq!(child_ids(&column), vec![left.id(), top.id()]);
    assert_eq!(child_ids(&workspace), vec![column.id(), bottom.id()]);
    assert_eq!(focused_id(&state), Some(left.id()));
  }

  #[test]
  fn expelling_moves_out_the_bottom_window_and_keeps_focus() {
    let (top, middle, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let split = column(&[&top, &middle, &bottom]);
    let (mut state, workspace) = setup(vec![split.clone().into()], &top);

    expel_window_from_column(&top.clone().into(), &mut state).unwrap();

    assert_eq!(child_ids(&workspace), vec![split.id(), bottom.id()]);
    assert_eq!(child_ids(&split), vec![top.id(), middle.id()]);
    assert_eq!(focused_id(&state), Some(top.id()));

    // A column of one window has nothing to expel.
    let lone: Container = bottom.clone().into();
    expel_window_from_column(&lone, &mut state).unwrap();
    assert_eq!(child_ids(&workspace), vec![split.id(), bottom.id()]);
  }
}
