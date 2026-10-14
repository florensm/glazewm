use wm_common::{Layout, VecDequeExt};
use wm_platform::Direction;

use super::{focused_window_in, rows, ColumnPlace};
use crate::{
  commands::container::set_focused_descendant,
  layout::scrolling::{active_column, column_of},
  models::Container,
  traits::CommonGetters,
  wm_state::WmState,
};

/// A column of a scrolling layout to go to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnTarget {
  First,
  Last,

  /// By position, counting from 1. Past the last column is the last one.
  Index(usize),

  /// The one right of the focused column, or the first after the last.
  Next,

  /// The one left of the focused column, or the last before the first.
  Previous,
}

impl ColumnTarget {
  /// Index of the column it means among `count` columns, the focused one
  /// being at `current`.
  pub fn resolve(self, current: usize, count: usize) -> usize {
    let last = count.saturating_sub(1);

    match self {
      Self::First => 0,
      Self::Last => last,
      Self::Index(position) => position.saturating_sub(1).min(last),
      Self::Next if current >= last => 0,
      Self::Next => current + 1,
      Self::Previous if current == 0 => last,
      Self::Previous => current - 1,
    }
  }
}

/// A window of a scrolling layout's column to go to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowTarget {
  Top,
  Bottom,

  /// By position from the top, counting from 1. Past the last window is
  /// the last one.
  Index(usize),
}

/// Focuses the column of `origin`'s scrolling layout that `target` says,
/// which scrolls it into view. Focus goes to the window in the column that
/// last had it.
///
/// Does nothing outside a scrolling layout.
pub fn focus_column(
  origin: &Container,
  target: ColumnTarget,
  state: &mut WmState,
) {
  let Some(workspace) = origin
    .workspace()
    .filter(|workspace| workspace.layout() == Layout::Scrolling)
  else {
    return;
  };

  let columns = workspace.tiling_children().collect::<Vec<_>>();
  if columns.is_empty() {
    return;
  }

  let current = active_column(&workspace)
    .and_then(|active| {
      columns.iter().position(|column| column.id() == active.id())
    })
    .unwrap_or(0);

  let column = &columns[target.resolve(current, columns.len())];
  set_focused_descendant(&focused_window_in(column), None);
  state.pending_sync.queue_focus_change().queue_cursor_jump();
}

/// Focuses the window of `origin`'s column that `target` says.
///
/// Does nothing outside a scrolling layout.
pub fn focus_window_in_column(
  origin: &Container,
  target: RowTarget,
  state: &mut WmState,
) {
  let Some(place) = ColumnPlace::of(origin) else {
    return;
  };

  let rows = rows(&place.column);
  let Some(last) = rows.len().checked_sub(1) else {
    return;
  };

  let index = match target {
    RowTarget::Top => 0,
    RowTarget::Bottom => last,
    RowTarget::Index(position) => position.saturating_sub(1).min(last),
  };

  set_focused_descendant(&focused_window_in(&rows[index]), None);
  state.pending_sync.queue_focus_change().queue_cursor_jump();
}

/// The tab before or after `window` in its stack for `direction` up or
/// down, when `window` is a tab in a scrolling layout's column: there,
/// tabs go up and down like the windows of a column do.
pub fn tab_in_direction(
  window: &Container,
  direction: &Direction,
) -> Option<Container> {
  let stack = window.parent()?.as_stack().cloned()?;
  column_of(window)?;

  let index = match direction {
    Direction::Up => window.index().checked_sub(1)?,
    Direction::Down => window.index() + 1,
    _ => return None,
  };

  stack.children().get(index).cloned()
}

/// Moves `window` one place up or down among its stack's tabs, when it is
/// a tab in a scrolling layout's column.
///
/// Returns whether it moved.
pub fn move_tab_in_direction(
  window: &Container,
  direction: &Direction,
  state: &mut WmState,
) -> bool {
  let (Some(target), Some(stack)) = (
    tab_in_direction(window, direction),
    window
      .parent()
      .and_then(|parent| parent.as_stack().cloned()),
  ) else {
    return false;
  };

  let index = target.index();
  stack
    .borrow_children_mut()
    .shift_to_index(index, window.clone());

  state
    .pending_sync
    .queue_container_to_redraw(stack)
    .queue_tab_bar_update();

  true
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};
  use wm_platform::Direction;

  use super::{
    focus_column, focus_window_in_column, move_tab_in_direction,
    tab_in_direction, ColumnTarget, RowTarget,
  };
  use crate::{
    commands::container::set_focused_descendant,
    layout::scrolling::sync_view,
    models::{
      Monitor, SplitContainer, StackContainer, TilingWindow, Workspace,
    },
    test_utils::child_ids,
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

    focus_column(&first.clone().into(), ColumnTarget::Last, &mut state);
    assert_eq!(focused_id(&state), Some(last.id()));
    sync_view(&workspace).unwrap();
    assert!(workspace.scroll_offset() > 0, "scrolls to the last column");

    focus_column(&last.into(), ColumnTarget::First, &mut state);
    assert_eq!(focused_id(&state), Some(first.id()));
    sync_view(&workspace).unwrap();
    assert_eq!(workspace.scroll_offset(), 0);
  }

  #[test]
  fn goes_to_the_window_a_column_last_focused() {
    let (mut state, _, [first, _, bottom, _]) = setup();
    set_focused_descendant(&first.clone().into(), None);

    focus_column(&first.into(), ColumnTarget::Index(2), &mut state);
    assert_eq!(focused_id(&state), Some(bottom.id()));
  }

  #[test]
  fn next_and_previous_wrap_around_the_strip() {
    let (mut state, _, [first, _, bottom, last]) = setup();

    focus_column(&bottom.clone().into(), ColumnTarget::Next, &mut state);
    assert_eq!(focused_id(&state), Some(last.id()));

    focus_column(&last.clone().into(), ColumnTarget::Next, &mut state);
    assert_eq!(focused_id(&state), Some(first.id()));

    focus_column(&first.into(), ColumnTarget::Previous, &mut state);
    assert_eq!(focused_id(&state), Some(last.id()));
  }

  #[test]
  fn an_index_past_the_end_is_the_last_column() {
    assert_eq!(ColumnTarget::Index(9).resolve(0, 3), 2);
    assert_eq!(ColumnTarget::Index(0).resolve(1, 3), 0);
  }

  #[test]
  fn focuses_windows_within_the_column() {
    let (mut state, _, [_, top, bottom, _]) = setup();

    focus_window_in_column(
      &bottom.clone().into(),
      RowTarget::Top,
      &mut state,
    );
    assert_eq!(focused_id(&state), Some(top.id()));

    focus_window_in_column(&top.into(), RowTarget::Index(5), &mut state);
    assert_eq!(focused_id(&state), Some(bottom.id()));
  }

  #[test]
  fn tabs_of_a_column_go_up_and_down_like_rows() {
    let (tab, other_tab) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let stack = StackContainer::mock()
      .tiling_containers(vec![
        tab.clone().into(),
        other_tab.clone().into(),
      ])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![stack.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);

    let below = tab_in_direction(&tab.clone().into(), &Direction::Down);
    assert_eq!(below.map(|below| below.id()), Some(other_tab.id()));
    assert!(
      tab_in_direction(&tab.clone().into(), &Direction::Up).is_none()
    );

    assert!(move_tab_in_direction(
      &tab.clone().into(),
      &Direction::Down,
      &mut state
    ));
    assert_eq!(child_ids(&stack), vec![other_tab.id(), tab.id()]);
    assert!(!move_tab_in_direction(
      &tab.into(),
      &Direction::Down,
      &mut state
    ));
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

    focus_column(&second.clone().into(), ColumnTarget::First, &mut state);
    assert_eq!(focused_id(&state), Some(second.id()));
  }
}
