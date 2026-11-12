//! Resizing a window by dragging its edges, as by its own border: the
//! edges not dragged stay where they are.
//!
//! In a split or dwindle layout, an edge is the boundary between the
//! window (or the nearest ancestor with a neighbour on that side) and that
//! neighbour, which alone gives or takes the room. An edge of the
//! workspace can't be dragged. In a scrolling layout, a column's width
//! takes room from no other column (see `scrolling::ColumnHold` for where
//! the view goes), and its rows share their column's height as in a split.

use wm_common::{Layout, TilingDirection};
use wm_platform::{LengthValue, Rect, ResizeEdges};

use super::scrolling::{
  column_of, column_size, column_width_range, width_at, windows_min_width,
  ColumnHold,
};
use crate::{
  models::{
    Container, DirectionContainer, TilingContainer, TilingWindow,
    Workspace,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters, MIN_TILING_SIZE,
  },
};

/// Sizes of every tiling container of a workspace, to resize from.
#[derive(Debug)]
pub struct SizeSnapshot(Vec<(TilingContainer, f32)>);

impl SizeSnapshot {
  pub fn of(workspace: &Workspace) -> Self {
    Self(
      workspace
        .descendants()
        .filter_map(|container| container.as_tiling_container().ok())
        .map(|container| {
          let size = container.tiling_size();
          (container, size)
        })
        .collect(),
    )
  }

  /// Puts every container back at its size.
  pub fn restore(&self) {
    for (container, size) in &self.0 {
      container.set_tiling_size(*size);
    }
  }

  /// Whether `workspace` holds the same tiling containers, in the same
  /// places, as when this was taken.
  pub fn matches(&self, workspace: &Workspace) -> bool {
    let now = Self::of(workspace);

    now.0.len() == self.0.len()
      && now.0.iter().zip(&self.0).all(|((now, _), (then, _))| {
        now.id() == then.id()
          && now.parent().map(|p| p.id()) == then.parent().map(|p| p.id())
      })
  }
}

/// Drags `edges` of tiling `window` by `delta`, in pixels on screen
/// (positive is right or down), each as far as its neighbour and the
/// apps' minimum widths allow.
///
/// Returns the containers whose windows have to be redrawn.
pub fn drag_tiling_edges(
  window: &TilingWindow,
  edges: ResizeEdges,
  delta: (i32, i32),
) -> anyhow::Result<Vec<Container>> {
  // A stacked window's edges are its stack's.
  let unit = match window.parent().and_then(|p| p.as_stack().cloned()) {
    Some(stack) => TilingContainer::Stack(stack),
    None => window.clone().into(),
  };

  let axes = [
    (true, edges.left, edges.right, delta.0),
    (false, edges.top, edges.bottom, delta.1),
  ];

  let mut redraw = Vec::new();

  for (is_horizontal, before, after, distance) in axes {
    if distance == 0 || !(before || after) {
      continue;
    }

    // Dragging the far edge outwards grows the window, the near one
    // shrinks it.
    let growth = if after { distance } else { -distance };

    if let Some(container) =
      drag_edge(&unit, is_horizontal, after, growth)?
    {
      redraw.push(container);
    }
  }

  Ok(redraw)
}

/// Drags `edges` of `window` by `delta` as [`drag_tiling_edges`] does,
/// and in a scrolling layout holds its column on screen with `hold`.
pub fn drag_and_hold(
  window: &TilingWindow,
  edges: ResizeEdges,
  delta: (i32, i32),
  hold: Option<&mut ColumnHold>,
) -> anyhow::Result<Vec<Container>> {
  let redraw = drag_tiling_edges(window, edges, delta)?;

  if let Some(hold) = hold.filter(|_| edges.left || edges.right) {
    if let Some((column, workspace)) = column_of(&window.as_container()) {
      hold.hold(&column, &workspace, edges.left)?;
    }
  }

  Ok(redraw)
}

/// Grows `unit` by `growth` pixels through its edge on the far side if
/// `is_after`, else the near side, along the horizontal axis if
/// `is_horizontal`. Returns the container laid out anew, if any.
fn drag_edge(
  unit: &TilingContainer,
  is_horizontal: bool,
  is_after: bool,
  growth: i32,
) -> anyhow::Result<Option<Container>> {
  if is_horizontal {
    if let Some((column, workspace)) = column_of(&unit.as_container()) {
      widen_column(&column, &workspace, growth)?;
      return Ok(Some(workspace.into()));
    }
  }

  let mut node = unit.clone();

  loop {
    let Some(parent) = node
      .parent()
      .and_then(|parent| parent.as_direction_container().ok())
    else {
      return Ok(None);
    };

    // Rows can't take room from another column.
    if let DirectionContainer::Workspace(workspace) = &parent {
      if workspace.layout() == Layout::Scrolling {
        return Ok(None);
      }
    }

    let is_along = (parent.tiling_direction()
      == TilingDirection::Horizontal)
      == is_horizontal;

    if is_along {
      let neighbour = if is_after {
        node
          .next_siblings()
          .find_map(|c| c.as_tiling_container().ok())
      } else {
        node
          .prev_siblings()
          .find_map(|c| c.as_tiling_container().ok())
      };

      if let Some(neighbour) = neighbour {
        move_boundary(&node, &neighbour, &parent, is_horizontal, growth)?;
        return Ok(Some(parent.into()));
      }
    }

    match parent {
      DirectionContainer::Split(split) => node = split.into(),
      DirectionContainer::Workspace(_) => return Ok(None),
    }
  }
}

/// Moves the boundary between siblings `node` and `neighbour` of `parent`
/// so that `node` grows by `growth` pixels and `neighbour` shrinks by as
/// much, neither past its minimum.
fn move_boundary(
  node: &TilingContainer,
  neighbour: &TilingContainer,
  parent: &DirectionContainer,
  is_horizontal: bool,
  growth: i32,
) -> anyhow::Result<()> {
  let (horizontal_gap, vertical_gap) = node.inner_gaps()?;
  let parent_rect = parent.to_rect()?;

  // As `tiling_rect` shares out the parent's length.
  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let gaps = node.tiling_siblings().count() as i32;
  let available = if is_horizontal {
    parent_rect.width() - horizontal_gap * gaps
  } else {
    parent_rect.height() - vertical_gap * gaps
  };

  if available <= 0 {
    return Ok(());
  }

  #[allow(clippy::cast_precision_loss)]
  let share = |pixels: i32| pixels as f32 / available as f32;

  let min_size = |container: &TilingContainer| {
    let min_width = if is_horizontal {
      windows_min_width(container)
    } else {
      0
    };

    share(min_width).max(MIN_TILING_SIZE)
  };

  let (size, neighbour_size) =
    (node.tiling_size(), neighbour.tiling_size());

  // Neither side is made smaller than it allows, nor made to grow by a
  // resize meant to shrink it.
  let lowest = (min_size(node) - size).min(0.);
  let highest = (neighbour_size - min_size(neighbour)).max(0.);
  let change = share(growth).clamp(lowest, highest);

  node.set_tiling_size(size + change);
  neighbour.set_tiling_size(neighbour_size - change);

  Ok(())
}

/// Widens `column` of `workspace` by `growth` pixels, within its range.
fn widen_column(
  column: &TilingContainer,
  workspace: &Workspace,
  growth: i32,
) -> anyhow::Result<()> {
  let (min, max) = column_width_range(column, workspace)?;
  let width = width_at(column, workspace, column.tiling_size())?;
  let target = (width + growth).clamp(min, max);

  let size =
    column_size(column, workspace, &LengthValue::from_px(target))?;
  column
    .set_tiling_size(size.clamp(super::scrolling::MIN_COLUMN_SIZE, 1.));

  Ok(())
}

/// `start` with `edges` dragged by `delta`, each no closer to the opposite
/// edge than `min_width` and `min_height`, for a floating window.
pub fn dragged_rect(
  start: &Rect,
  edges: ResizeEdges,
  delta: (i32, i32),
  min_width: i32,
  min_height: i32,
) -> Rect {
  let (dx, dy) = delta;
  let mut rect = start.clone();

  if edges.left {
    rect.left = (start.left + dx).min(start.right - min_width);
  } else if edges.right {
    rect.right = (start.right + dx).max(start.left + min_width);
  }

  if edges.top {
    rect.top = (start.top + dy).min(start.bottom - min_height);
  } else if edges.bottom {
    rect.bottom = (start.bottom + dy).max(start.top + min_height);
  }

  rect
}

#[cfg(test)]
mod tests {
  use std::mem::ManuallyDrop;

  use wm_common::{Layout, TilingDirection};
  use wm_platform::{Rect, ResizeEdges};

  use super::{drag_tiling_edges, dragged_rect, SizeSnapshot};
  use crate::{
    layout::scrolling::sync_view,
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::{mock_gapped_window as window, mock_inner_gaps},
    traits::{PositionGetters, TilingSizeGetters, WindowGetters},
    wm_state::WmState,
  };

  const RIGHT: ResizeEdges = ResizeEdges {
    left: false,
    right: true,
    top: false,
    bottom: false,
  };

  const LEFT: ResizeEdges = ResizeEdges {
    left: true,
    right: false,
    top: false,
    bottom: false,
  };

  const BOTTOM_LEFT: ResizeEdges = ResizeEdges {
    left: true,
    right: false,
    top: false,
    bottom: true,
  };

  /// A displayed workspace of `layout` with `children`, never dropped,
  /// which would restore its mock windows via Win32 calls.
  fn workspace(
    layout: Layout,
    children: Vec<crate::models::TilingContainer>,
  ) -> (Workspace, ManuallyDrop<WmState>) {
    let workspace = Workspace::mock()
      .layout(layout)
      .gaps_config(mock_inner_gaps())
      .tiling_containers(children)
      .call();
    let state = ManuallyDrop::new(WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]));
    (workspace, state)
  }

  fn assert_near(actual: i32, expected: i32) {
    assert!((actual - expected).abs() <= 1, "{actual} != {expected}");
  }

  #[test]
  fn a_split_edge_only_moves_the_boundary_it_is() {
    let (left, middle, right) = (window(), window(), window());
    let (_workspace, _state) = workspace(
      Layout::Split,
      vec![
        left.clone().into(),
        middle.clone().into(),
        right.clone().into(),
      ],
    );
    let (left_rect, middle_rect, right_rect) = (
      left.to_rect().unwrap(),
      middle.to_rect().unwrap(),
      right.to_rect().unwrap(),
    );

    drag_tiling_edges(&middle, RIGHT, (100, 0)).unwrap();

    // The left edge and the window left of it stay where they are.
    assert_eq!(left.to_rect().unwrap(), left_rect);
    assert_eq!(middle.to_rect().unwrap().left, middle_rect.left);
    assert_near(middle.to_rect().unwrap().right, middle_rect.right + 100);
    assert_near(right.to_rect().unwrap().left, right_rect.left + 100);
    assert_eq!(right.to_rect().unwrap().right, right_rect.right);
  }

  #[test]
  fn a_workspace_edge_does_not_move() {
    let (left, right) = (window(), window());
    let (_workspace, _state) = workspace(
      Layout::Split,
      vec![left.clone().into(), right.clone().into()],
    );

    let redraw = drag_tiling_edges(&left, LEFT, (-100, 0)).unwrap();

    assert!(redraw.is_empty());
    assert!((left.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn an_edge_stops_at_its_neighbours_minimum_width() {
    let (left, right) = (window(), window());
    let (workspace, _state) = workspace(
      Layout::Split,
      vec![left.clone().into(), right.clone().into()],
    );
    right.update_native_properties(|properties| {
      properties.min_width = 600;
    });

    drag_tiling_edges(&left, RIGHT, (2000, 0)).unwrap();

    let right_rect = right.to_rect().unwrap();
    assert_near(right_rect.width(), 600);
    assert_eq!(right_rect.right, workspace.to_rect().unwrap().right);
  }

  #[test]
  fn a_row_edge_moves_within_its_column_and_a_corner_drags_both() {
    let (left, top, bottom) = (window(), window(), window());
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .gaps_config(mock_inner_gaps())
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let (_workspace, _state) = workspace(
      Layout::Split,
      vec![left.clone().into(), column.clone().into()],
    );
    let (column_rect, bottom_rect) =
      (column.to_rect().unwrap(), bottom.to_rect().unwrap());

    // The top row's bottom-left corner: its left edge is the column's.
    let redraw = drag_tiling_edges(&top, BOTTOM_LEFT, (-50, 80)).unwrap();
    assert_eq!(redraw.len(), 2);

    assert_near(column.to_rect().unwrap().left, column_rect.left - 50);
    assert_eq!(column.to_rect().unwrap().right, column_rect.right);
    assert_near(bottom.to_rect().unwrap().top, bottom_rect.top + 80);
    assert_eq!(bottom.to_rect().unwrap().bottom, bottom_rect.bottom);
  }

  #[test]
  fn a_column_grows_past_the_screen_up_to_the_view_width() {
    let (first, second) = (window(), window());
    let (workspace, _state) = workspace(
      Layout::Scrolling,
      vec![first.clone().into(), second.clone().into()],
    );
    first.set_tiling_size(0.3);
    second.set_tiling_size(0.3);
    sync_view(&workspace).unwrap();
    let (first_rect, second_rect) =
      (first.to_rect().unwrap(), second.to_rect().unwrap());

    drag_tiling_edges(&second, RIGHT, (300, 0)).unwrap();

    // Wider by all of it, the other column as it was.
    assert_near(
      second.to_rect().unwrap().width(),
      second_rect.width() + 300,
    );
    assert_eq!(second.to_rect().unwrap().left, second_rect.left);
    assert_eq!(first.to_rect().unwrap(), first_rect);

    // At most as wide as the view.
    drag_tiling_edges(&second, RIGHT, (5000, 0)).unwrap();
    assert!((second.tiling_size() - 1.).abs() < 1e-4);
  }

  #[test]
  fn a_column_shrinks_no_further_than_its_app_allows() {
    let (first, second) = (window(), window());
    let (workspace, _state) = workspace(
      Layout::Scrolling,
      vec![first.clone().into(), second.clone().into()],
    );
    sync_view(&workspace).unwrap();
    first.update_native_properties(|properties| {
      properties.min_width = 500;
    });

    drag_tiling_edges(&first, RIGHT, (-5000, 0)).unwrap();

    assert_near(first.to_rect().unwrap().width(), 500);
  }

  #[test]
  fn a_snapshot_puts_sizes_back_and_notices_new_containers() {
    let (left, right) = (window(), window());
    let (workspace, _state) = workspace(
      Layout::Split,
      vec![left.clone().into(), right.clone().into()],
    );
    let snapshot = SizeSnapshot::of(&workspace);

    drag_tiling_edges(&left, RIGHT, (300, 0)).unwrap();
    assert!(left.tiling_size() > 0.6);

    snapshot.restore();
    assert!((left.tiling_size() - 0.5).abs() < 1e-4);
    assert!(snapshot.matches(&workspace));

    crate::commands::container::attach_container(
      &TilingWindow::mock().call().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();
    assert!(!snapshot.matches(&workspace));
  }

  #[test]
  fn a_floating_window_keeps_its_other_edges_and_minimum_size() {
    let start = Rect::from_ltrb(100, 100, 600, 500);

    assert_eq!(
      dragged_rect(&start, BOTTOM_LEFT, (-50, 40), 200, 100),
      Rect::from_ltrb(50, 100, 600, 540)
    );
    assert_eq!(
      dragged_rect(&start, LEFT, (450, 0), 200, 100),
      Rect::from_ltrb(400, 100, 600, 500)
    );
  }
}
