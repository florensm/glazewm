//! Dropping a dragged window onto a scrolling workspace's columns.

use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::Point;

use crate::{
  commands::container::{
    move_container_within_tree, wrap_in_split_container,
  },
  models::{Container, SplitContainer, TilingContainer, Workspace},
  traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Where a window dropped onto a scrolling workspace goes.
///
/// Refers to containers rather than indices, so that it stays valid while
/// the dropped window is put back into the tree.
#[derive(Clone, Debug)]
pub enum ColumnDrop {
  /// Into `target`'s column, above `target` if `is_before`, else below.
  Join {
    target: TilingContainer,
    is_before: bool,
  },
  /// As a new column, left of `anchor` if `is_before`, else right of it.
  NewColumn {
    anchor: TilingContainer,
    is_before: bool,
  },
}

/// Gets where a window dropped at `point` goes among `workspace`'s
/// columns: into the column under `point`, or as a new column in the gap
/// or past the strip's end that `point` is in.
///
/// The dragged window must not be tiling, so that it isn't a target
/// itself. Returns `None` if the workspace has no columns.
pub fn drop_target(
  workspace: &Workspace,
  point: &Point,
) -> anyhow::Result<Option<ColumnDrop>> {
  let mut last_column = None;

  // Columns are ordered left-to-right on the strip.
  for column in workspace.tiling_children() {
    let rect = column.to_rect()?;

    if point.x < rect.x() {
      return Ok(Some(ColumnDrop::NewColumn {
        anchor: column,
        is_before: true,
      }));
    }

    if point.x < rect.x() + rect.width() {
      return join_target(&column, point).map(Some);
    }

    last_column = Some(column);
  }

  Ok(last_column.map(|anchor| ColumnDrop::NewColumn {
    anchor,
    is_before: false,
  }))
}

/// Gets the row of `column` that a window dropped at `point` goes above or
/// below, depending on which half of the row `point` is in.
fn join_target(
  column: &TilingContainer,
  point: &Point,
) -> anyhow::Result<ColumnDrop> {
  // Only a vertical split's rows are joined one by one; any other column
  // is a single row, which keeps horizontal splits out of columns.
  let rows = match column {
    TilingContainer::Split(split)
      if split.tiling_direction() == TilingDirection::Vertical =>
    {
      split.tiling_children().collect::<Vec<_>>()
    }
    _ => vec![column.clone()],
  };

  // The lowest row starting above `point`; between rows, that's the one
  // above the gap.
  let mut target = None;
  for row in rows {
    let rect = row.to_rect()?;
    if target.is_some() && point.y < rect.y() {
      break;
    }
    target = Some((row, rect));
  }

  let (target, rect) = target.context("Column has no rows.")?;

  Ok(ColumnDrop::Join {
    is_before: point.y < rect.center_point().y,
    target,
  })
}

/// Moves `moved` (a tiling window or stack) to where `drop` says.
///
/// A column that is a single row gets wrapped in a vertical split for
/// `moved` to join.
pub fn drop_into_column(
  moved: &Container,
  drop: &ColumnDrop,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let (anchor, is_before) = match drop {
    ColumnDrop::Join { target, is_before }
    | ColumnDrop::NewColumn {
      anchor: target,
      is_before,
    } => (target, *is_before),
  };

  let mut parent = anchor.parent().context("No parent.")?;

  if matches!(drop, ColumnDrop::Join { .. }) && parent.is_workspace() {
    let split = SplitContainer::new(
      TilingDirection::Vertical,
      config.value.gaps.clone(),
    );

    wrap_in_split_container(
      &split,
      &parent,
      std::slice::from_ref(anchor),
    )?;
    parent = split.into();
  }

  move_container_within_tree(
    moved,
    &parent,
    index_beside(moved, anchor, is_before),
    state,
  )
}

/// Index among `anchor`'s siblings right before or after `anchor`, once
/// `moved` has left its current place.
fn index_beside(
  moved: &Container,
  anchor: &TilingContainer,
  is_before: bool,
) -> usize {
  let index = anchor.index() + usize::from(!is_before);

  // `move_container_within_tree` takes a sibling out before reinserting
  // it, which shifts everything after it.
  let is_earlier_sibling =
    moved.parent() == anchor.parent() && moved.index() < anchor.index();

  index - usize::from(is_earlier_sibling)
}

#[cfg(test)]
mod tests {
  use wm_common::{GapsConfig, Layout, TilingDirection};
  use wm_platform::{LengthValue, Point};

  use super::{drop_into_column, drop_target, ColumnDrop};
  use crate::{
    models::{
      Monitor, SplitContainer, StackContainer, TilingContainer,
      TilingWindow, Workspace,
    },
    traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  fn config() -> UserConfig {
    UserConfig::from_parsed(wm_common::ParsedConfig::default())
  }

  fn gaps() -> GapsConfig {
    GapsConfig {
      inner_gap: LengthValue::from_px(20),
      ..GapsConfig::default()
    }
  }

  fn window() -> TilingWindow {
    TilingWindow::mock().gaps_config(gaps()).call()
  }

  /// A scrolling workspace of half-width `columns`, 20px apart: x 0..830,
  /// 850..1680 and 1700..2530, each 1000px high.
  fn setup(columns: Vec<TilingContainer>) -> (WmState, Workspace) {
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(columns)
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    (WmState::mock(vec![monitor]), workspace)
  }

  fn vertical_split(rows: Vec<TilingContainer>) -> SplitContainer {
    SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .gaps_config(gaps())
      .tiling_containers(rows)
      .call()
  }

  fn target_at(workspace: &Workspace, x: i32, y: i32) -> ColumnDrop {
    drop_target(workspace, &Point { x, y }).unwrap().unwrap()
  }

  fn assert_join(drop: &ColumnDrop, id: uuid::Uuid, before: bool) {
    assert!(matches!(
      drop,
      ColumnDrop::Join { target, is_before }
        if target.id() == id && *is_before == before
    ));
  }

  fn assert_new_column(drop: &ColumnDrop, id: uuid::Uuid, before: bool) {
    assert!(matches!(
      drop,
      ColumnDrop::NewColumn { anchor, is_before }
        if anchor.id() == id && *is_before == before
    ));
  }

  fn child_ids(container: &impl CommonGetters) -> Vec<uuid::Uuid> {
    container.children().iter().map(CommonGetters::id).collect()
  }

  #[test]
  fn targets_the_half_of_the_column_under_the_point() {
    let (first, second) = (window(), window());
    let (_state, workspace) =
      setup(vec![first.clone().into(), second.clone().into()]);

    assert_join(&target_at(&workspace, 100, 100), first.id(), true);
    assert_join(&target_at(&workspace, 900, 900), second.id(), false);
  }

  #[test]
  fn targets_the_row_under_the_point_in_a_split_column() {
    let (top, bottom) = (window(), window());
    let column =
      vertical_split(vec![top.clone().into(), bottom.clone().into()]);
    let (_state, workspace) = setup(vec![column.into()]);

    // Rows are 490px high with a 20px gap at y 490..510.
    assert_join(&target_at(&workspace, 100, 100), top.id(), true);
    assert_join(&target_at(&workspace, 100, 400), top.id(), false);
    assert_join(&target_at(&workspace, 100, 500), top.id(), false);
    assert_join(&target_at(&workspace, 100, 600), bottom.id(), true);
    assert_join(&target_at(&workspace, 100, 990), bottom.id(), false);
  }

  #[test]
  fn targets_new_columns_in_gaps_and_past_the_ends() {
    let (first, second) = (window(), window());
    let (_state, workspace) =
      setup(vec![first.clone().into(), second.clone().into()]);

    assert_new_column(&target_at(&workspace, -10, 100), first.id(), true);
    assert_new_column(&target_at(&workspace, 840, 100), second.id(), true);
    assert_new_column(
      &target_at(&workspace, 1700, 100),
      second.id(),
      false,
    );

    let (_state, empty) = setup(vec![]);
    assert!(drop_target(&empty, &Point { x: 100, y: 100 })
      .unwrap()
      .is_none());
  }

  #[test]
  fn joining_a_single_window_column_splits_it_vertically() {
    let (first, moved) = (window(), window());
    let (state, workspace) =
      setup(vec![first.clone().into(), moved.clone().into()]);
    first.set_tiling_size(0.7);

    let drop = target_at(&workspace, 100, 100);
    drop_into_column(&moved.clone().into(), &drop, &state, &config())
      .unwrap();

    let column = first.parent().unwrap();
    let split = column.as_split().unwrap();
    assert_eq!(column.parent().unwrap().id(), workspace.id());
    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(child_ids(split), vec![moved.id(), first.id()]);
    assert!((split.tiling_size() - 0.7).abs() < 1e-4);
    assert!((moved.tiling_size() - 0.5).abs() < 1e-4);
    assert_eq!(workspace.child_count(), 1);
  }

  #[test]
  fn joins_a_split_column_between_its_rows() {
    let (top, bottom, moved) = (window(), window(), window());
    let column =
      vertical_split(vec![top.clone().into(), bottom.clone().into()]);
    let (state, workspace) =
      setup(vec![column.clone().into(), moved.clone().into()]);

    let drop = target_at(&workspace, 100, 400);
    drop_into_column(&moved.clone().into(), &drop, &state, &config())
      .unwrap();

    assert_eq!(
      child_ids(&column),
      vec![top.id(), moved.id(), bottom.id()]
    );
    assert_eq!(child_ids(&workspace), vec![column.id()]);
  }

  #[test]
  fn reorders_within_its_own_column() {
    let (moved, middle, bottom) = (window(), window(), window());
    let column = vertical_split(vec![
      moved.clone().into(),
      middle.clone().into(),
      bottom.clone().into(),
    ]);
    let (state, _workspace) = setup(vec![column.clone().into()]);

    let drop = ColumnDrop::Join {
      target: middle.clone().into(),
      is_before: false,
    };
    drop_into_column(&moved.clone().into(), &drop, &state, &config())
      .unwrap();

    assert_eq!(
      child_ids(&column),
      vec![middle.id(), moved.id(), bottom.id()]
    );
  }

  #[test]
  fn joining_a_stack_column_keeps_the_stack_whole() {
    let tab = window();
    let stack = StackContainer::mock()
      .tiling_containers(vec![tab.clone().into()])
      .call();
    let moved = window();
    let (state, workspace) =
      setup(vec![stack.clone().into(), moved.clone().into()]);

    let drop = target_at(&workspace, 100, 900);
    drop_into_column(&moved.clone().into(), &drop, &state, &config())
      .unwrap();

    let column = stack.parent().unwrap();
    assert!(column.is_split());
    assert_eq!(child_ids(&column), vec![stack.id(), moved.id()]);
    assert_eq!(child_ids(&stack), vec![tab.id()]);
  }

  #[test]
  fn new_column_takes_the_default_width() {
    let (top, moved, bottom, other) =
      (window(), window(), window(), window());
    let column = vertical_split(vec![
      top.clone().into(),
      moved.clone().into(),
      bottom.clone().into(),
    ]);
    let (state, workspace) =
      setup(vec![column.clone().into(), other.clone().into()]);

    let drop = target_at(&workspace, 1690, 100);
    drop_into_column(&moved.clone().into(), &drop, &state, &config())
      .unwrap();

    assert_eq!(
      child_ids(&workspace),
      vec![column.id(), other.id(), moved.id()]
    );
    assert_eq!(child_ids(&column), vec![top.id(), bottom.id()]);
    assert!((moved.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn dropping_into_its_own_slot_is_a_no_op() {
    let (first, moved, last) = (window(), window(), window());
    let (state, workspace) = setup(vec![
      first.clone().into(),
      moved.clone().into(),
      last.clone().into(),
    ]);
    moved.set_tiling_size(0.3);

    for drop in [
      ColumnDrop::NewColumn {
        anchor: last.clone().into(),
        is_before: true,
      },
      ColumnDrop::NewColumn {
        anchor: first.clone().into(),
        is_before: false,
      },
    ] {
      drop_into_column(&moved.clone().into(), &drop, &state, &config())
        .unwrap();

      assert_eq!(
        child_ids(&workspace),
        vec![first.id(), moved.id(), last.id()]
      );
      assert!((moved.tiling_size() - 0.3).abs() < 1e-4);
    }
  }
}
