//! Scrolling layout: the workspace's children are columns on a horizontal
//! strip that can be wider than the workspace, scrolled so the focused
//! column is on screen.
//!
//! A column is a window, a stack, or a split of windows stacked
//! vertically. Its `tiling_size` is its width as a fraction of the
//! workspace width; unlike in the other layouts, the columns' sizes don't
//! add up to 1.

use wm_common::{FocusFitMethod, Layout};
use wm_platform::{LengthUnit, LengthValue, Rect};

use super::Insertion;
use crate::{
  models::{Container, Monitor, TilingContainer, Workspace},
  traits::{CommonGetters, PositionGetters, TilingSizeGetters},
  wm_state::WmState,
};

/// Smallest column width, as a fraction of the workspace width.
pub const MIN_COLUMN_SIZE: f32 = 0.1;

/// The scrolling workspace that `container` is a column of, if any.
///
/// Only tiling containers are columns; a floating or fullscreen window of
/// the workspace (e.g. one being dragged) is not on the strip.
pub fn column_workspace(
  container: &impl CommonGetters,
) -> Option<Workspace> {
  container
    .as_tiling_container()
    .ok()?
    .parent()
    .and_then(|parent| parent.as_workspace().cloned())
    .filter(|workspace| workspace.layout() == Layout::Scrolling)
}

/// The column holding `container`, and its scrolling workspace, if
/// `container` is in a scrolling layout.
pub fn column_of(
  container: &Container,
) -> Option<(TilingContainer, Workspace)> {
  container.self_and_ancestors().find_map(|ancestor| {
    let workspace = column_workspace(&ancestor)?;
    Some((ancestor.as_tiling_container().ok()?, workspace))
  })
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

/// Size giving `column` a width of `width`; the inverse of `column_rect`.
///
/// A percentage is of the workspace width, as for `column_width`.
pub fn column_size<T>(
  column: &T,
  workspace: &Workspace,
  width: &LengthValue,
) -> anyhow::Result<f32>
where
  T: TilingSizeGetters,
{
  if width.unit == LengthUnit::Percentage {
    return Ok(width.amount);
  }

  let (gap, _) = column.inner_gaps()?;
  let total = workspace.to_rect()?.width() + gap;

  #[allow(clippy::cast_precision_loss)]
  Ok((width.amount + gap as f32) / total as f32)
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

  // A focused floating window (e.g. a tiling window being dragged) leaves
  // the strip where it is, rather than scrolling to an older column.
  let focused_column = workspace
    .child_focus_order()
    .next()
    .and_then(|child| child.as_tiling_container().ok());

  // Strip width, independent of the current offset, and how far it can
  // scroll while still filling the workspace.
  let (first_rect, last_rect) = (first.to_rect()?, last.to_rect()?);
  let strip_width = last_rect.x() + last_rect.width() - first_rect.x();
  let max_offset = (strip_width - workspace_rect.width()).max(0);

  let Some(column) = focused_column else {
    return Ok(update_offset(workspace, offset.clamp(0, max_offset)));
  };
  let rect = column.to_rect()?;

  let target = match workspace.focus_fit_method() {
    FocusFitMethod::Fit => {
      let overflow_left = workspace_rect.x() - rect.x();
      let overflow_right = rect.x() + rect.width()
        - (workspace_rect.x() + workspace_rect.width());

      // A column wider than the workspace is aligned to its left edge.
      let target = if overflow_left > 0 {
        offset - overflow_left
      } else if overflow_right > 0 {
        offset + overflow_right.min(-overflow_left)
      } else {
        offset
      };

      target.clamp(0, max_offset)
    }
    // Centered even at the strip's ends, leaving room past them.
    FocusFitMethod::Center => {
      offset + rect.center_point().x - workspace_rect.center_point().x
    }
  };

  Ok(update_offset(workspace, target))
}

fn update_offset(workspace: &Workspace, offset: i32) -> bool {
  let is_changed = workspace.scroll_offset() != offset;
  workspace.set_scroll_offset(offset);
  is_changed
}

/// Whether `container`'s column lies within its workspace horizontally.
///
/// Containers outside a scrolling workspace are always in view.
pub fn is_in_view(container: &Container) -> bool {
  column_rects(container).is_none_or(|(rect, workspace_rect, _)| {
    rect.x() >= workspace_rect.x()
      && rect.x() + rect.width()
        <= workspace_rect.x() + workspace_rect.width()
  })
}

/// Whether `container`'s column is shown by its real window.
///
/// That is unless the part of it past the workspace would reach onto
/// another monitor. Past an edge with no monitor behind it, the window
/// simply runs off screen, even entirely, so it stays live and is drawn
/// like any other window when it scrolls back into view.
///
/// Containers outside a scrolling workspace are always shown.
pub fn shows_real_window(container: &Container) -> bool {
  is_in_view(container)
    || column_rects(container).is_none_or(|(rect, _, workspace)| {
      workspace.monitor().is_none_or(|monitor| {
        !overflows_onto_other_monitor(&rect, &monitor)
      })
    })
}

/// Rect of the column holding `container`, its workspace's rect, and the
/// workspace, when `container` is in a scrolling layout.
fn column_rects(container: &Container) -> Option<(Rect, Rect, Workspace)> {
  let (column, workspace) = column_of(container)?;

  Some((column.to_rect().ok()?, workspace.to_rect().ok()?, workspace))
}

/// Whether the parts of `rect` left or right of `monitor` cover any other
/// monitor.
fn overflows_onto_other_monitor(rect: &Rect, monitor: &Monitor) -> bool {
  let bounds = monitor.native_properties().bounds;

  let overflows = [
    Rect::from_ltrb(rect.left, rect.top, bounds.left, rect.bottom),
    Rect::from_ltrb(bounds.right, rect.top, rect.right, rect.bottom),
  ];

  let others = monitor
    .parent()
    .map_or_else(Default::default, |root| root.children())
    .into_iter()
    .filter_map(|child| child.as_monitor().cloned())
    .filter(|other| other.id() != monitor.id());

  others
    .map(|other| other.native_properties().bounds)
    .any(|other| {
      overflows
        .iter()
        .any(|overflow| overflow.intersection_area(&other) > 0)
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
  use wm_common::{
    FocusFitMethod, Layout, ScrollingLayoutConfig, TilingDirection,
    WindowState,
  };
  use wm_platform::Rect;

  use super::{follow_focus, is_in_view, shows_real_window};
  use crate::{
    commands::container::{
      detach_container, flatten_child_split_containers,
      resize_tiling_container, set_focused_descendant,
    },
    layout::{attach, insertion_target},
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::{mock_config as config, MOCK_MONITOR_WIDTH},
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    wm_state::WmState,
  };

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

    assert!(is_in_view(&first.clone().into()));
    assert!(!is_in_view(&third.clone().into()));
    // No monitor lies past the edge, so it stays a live window off screen.
    assert!(shows_real_window(&third.clone().into()));
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
    assert!(!is_in_view(&first.clone().into()));
    assert!(is_in_view(&second.clone().into()));

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

  /// A scrolling workspace whose second column runs past the right edge
  /// of its monitor, optionally with another monitor to the right.
  fn edge_setup(has_right_neighbor: bool) -> (WmState, TilingWindow) {
    let (first, second) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    second.set_tiling_size(0.7);

    let mut monitors =
      vec![Monitor::mock().workspaces(vec![workspace]).call()];
    if has_right_neighbor {
      let bounds = Rect::from_xy(MOCK_MONITOR_WIDTH, 0, 1000, 1000);
      monitors.push(
        Monitor::mock()
          .bounds(bounds.clone())
          .working_area(bounds)
          .call(),
      );
    }

    (WmState::mock(monitors), second)
  }

  #[test]
  fn a_column_past_a_free_edge_keeps_its_real_window() {
    let (_state, second) = edge_setup(false);
    assert!(shows_real_window(&second.into()));
  }

  #[test]
  fn a_column_reaching_onto_another_monitor_is_hidden() {
    let (_state, second) = edge_setup(true);
    assert!(!shows_real_window(&second.into()));
  }

  #[test]
  fn center_keeps_the_focused_column_in_the_middle() {
    let (_state, workspace, [first, second, _]) = setup();
    workspace.set_scrolling_config(ScrollingLayoutConfig {
      focus_fit_method: FocusFitMethod::Center,
      ..ScrollingLayoutConfig::default()
    });
    let center = workspace.to_rect().unwrap().center_point().x;

    set_focused_descendant(&second.clone().into(), None);
    follow_focus(&workspace).unwrap();
    assert_eq!(second.to_rect().unwrap().center_point().x, center);

    // Also at the strip's start, leaving room before it.
    set_focused_descendant(&first.clone().into(), None);
    follow_focus(&workspace).unwrap();
    assert_eq!(first.to_rect().unwrap().center_point().x, center);
    assert!(workspace.scroll_offset() < 0);
  }
}
