//! Where a dragged window lands when it is dropped back into tiling.
//!
//! Decided in one place for both the drop itself and the indicator shown
//! during the drag, so the indicator always shows what the drop does.

use anyhow::Context;
use wm_common::{Layout, TilingDirection};
use wm_platform::{Point, Rect};

use super::column_drop::{drop_target, ColumnDrop};
use crate::{
  models::{
    DirectionContainer, NonTilingWindow, StackContainer, TilingContainer,
    Workspace,
  },
  traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
  wm_state::WmState,
};

/// Where a dragged window goes when it's dropped back into tiling.
///
/// Refers to containers rather than indices, so that it stays valid while
/// the dropped window is put back into the tree.
#[derive(Clone, Debug)]
pub enum WindowDrop {
  /// Into `parent`, which has no tiling children (e.g. an empty
  /// workspace).
  Into(DirectionContainer),
  /// Beside `target` within `parent`, on the side given by `position`.
  /// With `should_split`, `target` is first wrapped in a split of the
  /// other direction, for the window to share its slot.
  Beside {
    parent: DirectionContainer,
    target: TilingContainer,
    position: DropPosition,
    should_split: bool,
  },
  /// Among the columns of `workspace`, which has the scrolling layout.
  Column {
    workspace: Workspace,
    drop: ColumnDrop,
  },
}

/// Side of a container that a window is dropped on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropPosition {
  Top,
  Bottom,
  Left,
  Right,
}

impl DropPosition {
  /// Whether the window goes before the container it's dropped on.
  pub fn is_before(self) -> bool {
    matches!(self, Self::Top | Self::Left)
  }
}

/// The workspace a window dropped at `point` goes to: the one displayed
/// under `point`, or else the window's own.
pub fn drop_workspace(
  window: &NonTilingWindow,
  point: &Point,
  state: &WmState,
) -> Option<Workspace> {
  state
    .monitor_at_point(point)
    .and_then(|monitor| monitor.displayed_workspace())
    .or_else(|| window.workspace())
}

/// Gets where `window`, being dragged as a temporary floating window,
/// goes when dropped at `point`.
///
/// Returns `None` if there's no workspace to drop it on.
pub fn window_drop(
  window: &NonTilingWindow,
  point: &Point,
  state: &WmState,
) -> anyhow::Result<Option<WindowDrop>> {
  let Some(workspace) = drop_workspace(window, point, state) else {
    return Ok(None);
  };

  // Decided on the columns as shown during the drag, before the window is
  // put back into the tree.
  if workspace.layout() == Layout::Scrolling {
    if let Some(drop) = drop_target(&workspace, point)? {
      return Ok(Some(WindowDrop::Column { workspace, drop }));
    }
  }

  // A window of a stack is dragged with its whole stack.
  let moved_id = window
    .parent()
    .filter(|parent| parent.as_stack().is_some())
    .map_or(window.id(), |stack| stack.id());

  // The deepest direction container under `point`, leaving out the
  // dragged window and its stack.
  let parent = state
    .containers_at_point(&workspace.clone().into(), point)
    .into_iter()
    .filter(|container| {
      !container
        .self_and_ancestors()
        .any(|ancestor| ancestor.id() == moved_id)
    })
    .filter_map(|container| container.as_direction_container().ok())
    .fold(DirectionContainer::from(workspace), |acc, container| {
      if container.ancestors().count() > acc.ancestors().count() {
        container
      } else {
        acc
      }
    });

  if parent.tiling_children().count() == 0 {
    return Ok(Some(WindowDrop::Into(parent)));
  }

  let target = parent
    .children()
    .into_iter()
    .filter_map(|container| container.as_tiling_container().ok())
    .try_fold(None, |acc: Option<TilingContainer>, container| match acc {
      Some(acc) => {
        let is_nearer = acc.to_rect()?.distance_to_point(point)
          < container.to_rect()?.distance_to_point(point);

        anyhow::Ok(Some(if is_nearer { acc } else { container }))
      }
      None => Ok(Some(container)),
    })?
    .context("No nearest container.")?;

  let position = drop_position(point, &target.to_rect()?);

  // A window dropped across the parent's direction shares the target's
  // slot rather than taking one of its own.
  let should_split = target.is_tiling_window()
    && match parent.tiling_direction() {
      TilingDirection::Horizontal => {
        matches!(position, DropPosition::Top | DropPosition::Bottom)
      }
      TilingDirection::Vertical => {
        matches!(position, DropPosition::Left | DropPosition::Right)
      }
    };

  Ok(Some(WindowDrop::Beside {
    parent,
    target,
    position,
    should_split,
  }))
}

// Only the drop indicator, which is Windows-only, previews a drop.
#[cfg(target_os = "windows")]
impl WindowDrop {
  /// The workspace the window is dropped on.
  pub fn workspace(&self) -> Option<Workspace> {
    match self {
      Self::Into(parent) | Self::Beside { parent, .. } => {
        parent.workspace()
      }
      Self::Column { workspace, .. } => Some(workspace.clone()),
    }
  }

  /// Where to show the drop while dragging: the half of the target on the
  /// side the window goes, or all of an empty parent.
  pub fn indicator_rect(&self) -> anyhow::Result<Rect> {
    Ok(match self {
      Self::Into(parent) => parent.to_rect()?,
      Self::Beside {
        target, position, ..
      } => half(&target.to_rect()?, *position),
      Self::Column {
        drop: ColumnDrop::Join { target, is_before },
        ..
      } => {
        let position = if *is_before {
          DropPosition::Top
        } else {
          DropPosition::Bottom
        };
        half(&target.to_rect()?, position)
      }
      Self::Column {
        drop: ColumnDrop::NewColumn { anchor, is_before },
        ..
      } => {
        let position = if *is_before {
          DropPosition::Left
        } else {
          DropPosition::Right
        };
        half(&anchor.to_rect()?, position)
      }
    })
  }
}

/// The half of `rect` on the side given by `position`.
#[cfg(target_os = "windows")]
fn half(rect: &Rect, position: DropPosition) -> Rect {
  let center = rect.center_point();

  match position {
    DropPosition::Left => {
      Rect::from_ltrb(rect.left, rect.top, center.x, rect.bottom)
    }
    DropPosition::Right => {
      Rect::from_ltrb(center.x, rect.top, rect.right, rect.bottom)
    }
    DropPosition::Top => {
      Rect::from_ltrb(rect.left, rect.top, rect.right, center.y)
    }
    DropPosition::Bottom => {
      Rect::from_ltrb(rect.left, center.y, rect.right, rect.bottom)
    }
  }
}

/// Gets the drop position for a window based on the mouse position.
///
/// This approach divides the window rect into an "X", creating four
/// triangular quadrants, to determine which side the cursor is closest to.
fn drop_position(mouse_pos: &Point, rect: &Rect) -> DropPosition {
  let delta_x = mouse_pos.x - rect.center_point().x;
  let delta_y = mouse_pos.y - rect.center_point().y;

  if delta_x.abs() > delta_y.abs() {
    // Window is in the left or right triangle.
    if delta_x > 0 {
      DropPosition::Right
    } else {
      DropPosition::Left
    }
  } else {
    // Window is in the top or bottom triangle.
    if delta_y > 0 {
      DropPosition::Bottom
    } else {
      DropPosition::Top
    }
  }
}

/// The stack, other than `exclude`, whose tab bar is at `point` on a
/// displayed workspace.
pub fn tab_bar_at(
  point: &Point,
  exclude: Option<&StackContainer>,
  state: &WmState,
) -> Option<StackContainer> {
  state
    .root_container
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .filter(|stack| {
      exclude.is_none_or(|exclude| exclude.id() != stack.id())
        && stack.shows_tab_bar()
        && stack.workspace().is_some_and(|ws| ws.is_displayed())
    })
    .find(|stack| {
      stack.to_rect().is_ok_and(|rect| {
        rect.contains_point(point)
          && !stack.content_rect(&rect).contains_point(point)
      })
    })
}

#[cfg(test)]
mod tests {
  use wm_platform::{Point, Rect};

  #[cfg(target_os = "windows")]
  use super::half;
  use super::{drop_position, DropPosition};

  #[cfg(target_os = "windows")]
  #[test]
  fn halves_a_rect_on_each_side() {
    let rect = Rect::from_xy(100, 50, 400, 200);

    assert_eq!(
      half(&rect, DropPosition::Left),
      Rect::from_xy(100, 50, 200, 200)
    );
    assert_eq!(
      half(&rect, DropPosition::Right),
      Rect::from_xy(300, 50, 200, 200)
    );
    assert_eq!(
      half(&rect, DropPosition::Top),
      Rect::from_xy(100, 50, 400, 100)
    );
    assert_eq!(
      half(&rect, DropPosition::Bottom),
      Rect::from_xy(100, 150, 400, 100)
    );
  }

  #[test]
  fn picks_the_side_nearest_the_cursor() {
    let rect = Rect::from_xy(0, 0, 400, 200);
    let at = |x, y| drop_position(&Point { x, y }, &rect);

    assert_eq!(at(20, 100), DropPosition::Left);
    assert_eq!(at(380, 100), DropPosition::Right);
    assert_eq!(at(200, 10), DropPosition::Top);
    assert_eq!(at(200, 190), DropPosition::Bottom);
  }
}
