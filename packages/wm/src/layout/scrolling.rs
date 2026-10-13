//! Scrolling layout: the workspace's children are columns on a horizontal
//! strip that can be wider than the workspace, scrolled so the focused
//! column is on screen.
//!
//! A column is a window, a stack, or a split of windows stacked
//! vertically. Its `tiling_size` is its width as a fraction of the
//! workspace width; unlike in the other layouts, the columns' sizes don't
//! add up to 1.

use wm_common::Layout;
use wm_platform::Rect;

use super::Insertion;
use crate::{
  models::{Container, TilingContainer, Workspace},
  traits::{CommonGetters, PositionGetters, TilingSizeGetters},
  wm_state::WmState,
};

/// Smallest column width, as a fraction of the workspace width.
pub const MIN_COLUMN_SIZE: f32 = 0.1;

/// The scrolling workspace that `container` is a column of, if any.
pub fn column_workspace(
  container: &impl CommonGetters,
) -> Option<Workspace> {
  container
    .parent()
    .and_then(|parent| parent.as_workspace().cloned())
    .filter(|workspace| workspace.layout() == Layout::Scrolling)
}

/// Rect of a column on its workspace's strip.
pub(super) fn column_rect<T>(
  column: &T,
  workspace: &Workspace,
) -> anyhow::Result<Rect>
where
  T: TilingSizeGetters,
{
  let workspace_rect = workspace.to_rect()?;
  let (gap, _) = column.inner_gaps()?;

  // Sized so that columns whose sizes add up to 1 fill the workspace
  // exactly, gaps included.
  #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
  let width = ((workspace_rect.width() + gap) as f32
    * column.tiling_size())
  .round() as i32
    - gap;

  let prev_column = column
    .prev_siblings()
    .find_map(|sibling| sibling.as_tiling_container().ok());

  let x = match prev_column {
    Some(prev) => {
      let prev_rect = prev.to_rect()?;
      prev_rect.x() + prev_rect.width() + gap
    }
    None => workspace_rect.x() - workspace.scroll_offset(),
  };

  Ok(Rect::from_xy(
    x,
    workspace_rect.y(),
    width,
    workspace_rect.height(),
  ))
}

/// A new tiling window becomes a column right of the focused one.
pub(super) fn insertion_target(workspace: &Workspace) -> Insertion {
  let index = focused_column(workspace)
    .map_or(workspace.child_count(), |column| column.index() + 1);

  Insertion::At {
    parent: workspace.clone().into(),
    index,
  }
}

/// The column holding the workspace's most recently focused window.
fn focused_column(workspace: &Workspace) -> Option<TilingContainer> {
  workspace
    .child_focus_order()
    .find_map(|child| child.as_tiling_container().ok())
}

/// Scrolls `workspace` just far enough to show its focused column whole,
/// and no further than its columns reach.
///
/// Returns whether the scroll offset changed.
pub fn follow_focus(workspace: &Workspace) -> anyhow::Result<bool> {
  let columns = workspace.tiling_children().collect::<Vec<_>>();
  let (Some(first), Some(last)) = (columns.first(), columns.last()) else {
    return Ok(update_offset(workspace, 0));
  };

  let workspace_rect = workspace.to_rect()?;
  let offset = workspace.scroll_offset();

  let mut target = offset;
  if let Some(column) = focused_column(workspace) {
    let rect = column.to_rect()?;
    let overflow_left = workspace_rect.x() - rect.x();
    let overflow_right = rect.x() + rect.width()
      - (workspace_rect.x() + workspace_rect.width());

    // A column wider than the workspace is aligned to its left edge.
    if overflow_left > 0 {
      target -= overflow_left;
    } else if overflow_right > 0 {
      target += overflow_right.min(-overflow_left);
    }
  }

  // Strip width, independent of the current offset.
  let (first_rect, last_rect) = (first.to_rect()?, last.to_rect()?);
  let strip_width = last_rect.x() + last_rect.width() - first_rect.x();
  let max_offset = (strip_width - workspace_rect.width()).max(0);

  Ok(update_offset(workspace, target.clamp(0, max_offset)))
}

fn update_offset(workspace: &Workspace, offset: i32) -> bool {
  let is_changed = workspace.scroll_offset() != offset;
  workspace.set_scroll_offset(offset);
  is_changed
}

/// Whether `container`'s column lies within its workspace horizontally.
///
/// Containers outside a scrolling workspace are always on screen.
pub fn is_on_screen(container: &Container) -> bool {
  let Some(column) = container
    .self_and_ancestors()
    .find(|ancestor| column_workspace(ancestor).is_some())
  else {
    return true;
  };

  let (Some(workspace), Ok(rect)) = (column.workspace(), column.to_rect())
  else {
    return true;
  };

  workspace.to_rect().is_ok_and(|workspace_rect| {
    rect.x() >= workspace_rect.x()
      && rect.x() + rect.width()
        <= workspace_rect.x() + workspace_rect.width()
  })
}

/// Queues every scrolling workspace whose scroll offset has to change
/// for a redraw.
pub fn sync_scroll_offsets(state: &mut WmState) -> anyhow::Result<()> {
  for workspace in state.workspaces() {
    if workspace.layout() == Layout::Scrolling && follow_focus(&workspace)?
    {
      state.pending_sync.queue_container_to_redraw(workspace);
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection, WindowState};

  use super::{follow_focus, is_on_screen};
  use crate::{
    commands::container::{
      detach_container, flatten_child_split_containers,
      resize_tiling_container, set_focused_descendant,
    },
    layout::{attach, insertion_target},
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  fn config() -> UserConfig {
    UserConfig::from_parsed(wm_common::ParsedConfig::default())
  }

  /// A scrolling workspace with three half-width columns, the first one
  /// focused.
  fn setup() -> (WmState, Workspace, [TilingWindow; 3]) {
    let windows = [
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    ];
    let workspace = Workspace::mock().layout(Layout::Scrolling).call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);

    for window in &windows {
      let insertion =
        insertion_target(&WindowState::Tiling, false, &state).unwrap();
      attach(&window.clone().into(), &insertion, &config()).unwrap();
      set_focused_descendant(&window.clone().into(), None);
    }
    set_focused_descendant(&windows[0].clone().into(), None);

    (state, workspace, windows)
  }

  #[test]
  fn columns_continue_past_the_workspace() {
    let (_state, workspace, [first, second, third]) = setup();
    let workspace_rect = workspace.to_rect().unwrap();

    assert!((first.tiling_size() - 0.5).abs() < 1e-4);
    assert!((third.tiling_size() - 0.5).abs() < 1e-4);

    let (first_rect, second_rect, third_rect) = (
      first.to_rect().unwrap(),
      second.to_rect().unwrap(),
      third.to_rect().unwrap(),
    );
    assert_eq!(first_rect.x(), workspace_rect.x());
    assert_eq!(
      second_rect.x() + second_rect.width(),
      workspace_rect.x() + workspace_rect.width()
    );
    assert!(third_rect.x() >= workspace_rect.x() + workspace_rect.width());

    assert!(is_on_screen(&first.clone().into()));
    assert!(!is_on_screen(&third.clone().into()));
  }

  #[test]
  fn new_windows_open_right_of_the_focused_column() {
    let (state, workspace, [first, second, _]) = setup();

    let window = TilingWindow::mock().call();
    let insertion =
      insertion_target(&WindowState::Tiling, false, &state).unwrap();
    attach(&window.clone().into(), &insertion, &config()).unwrap();

    assert_eq!(window.parent().unwrap().id(), workspace.id());
    assert_eq!(window.index(), first.index() + 1);
    assert_eq!(second.index(), window.index() + 1);
  }

  #[test]
  fn scrolls_just_enough_to_show_the_focused_column() {
    let (_state, workspace, [first, second, third]) = setup();
    let workspace_rect = workspace.to_rect().unwrap();

    assert!(!follow_focus(&workspace).unwrap());
    assert_eq!(workspace.scroll_offset(), 0);

    set_focused_descendant(&third.clone().into(), None);
    assert!(follow_focus(&workspace).unwrap());

    let third_rect = third.to_rect().unwrap();
    assert_eq!(
      third_rect.x() + third_rect.width(),
      workspace_rect.x() + workspace_rect.width()
    );
    assert!(!is_on_screen(&first.clone().into()));
    assert!(is_on_screen(&second.clone().into()));

    // Back to the first column, which is scrolled to the left edge.
    set_focused_descendant(&first.clone().into(), None);
    assert!(follow_focus(&workspace).unwrap());
    assert_eq!(workspace.scroll_offset(), 0);
  }

  #[test]
  fn closing_and_resizing_columns_leave_the_others_alone() {
    let (_state, _workspace, [first, second, third]) = setup();

    resize_tiling_container(&first.clone().into(), 0.8);
    assert!((first.tiling_size() - 0.8).abs() < 1e-4);
    assert!((second.tiling_size() - 0.5).abs() < 1e-4);

    resize_tiling_container(&first.clone().into(), 5.);
    assert!((first.tiling_size() - 1.).abs() < 1e-4);

    detach_container(second.clone().into()).unwrap();
    assert!((first.tiling_size() - 1.).abs() < 1e-4);
    assert!((third.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn the_only_column_stays_a_column() {
    let (top, bottom) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.clone().into()])
      .call();
    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    flatten_child_split_containers(&workspace.clone().into()).unwrap();

    assert_eq!(top.parent().unwrap().id(), column.id());
    assert_eq!(top.to_rect().unwrap().x(), bottom.to_rect().unwrap().x());
  }
}
