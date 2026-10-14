//! Commands on the columns of a scrolling layout.

mod center_column;
mod column_display;
mod column_width;
mod consume_or_expel;
mod focus_column;
mod move_column;
mod swap_window;
mod window_height;

pub use center_column::*;
pub use column_display::*;
pub use column_width::*;
pub use consume_or_expel::*;
pub use focus_column::*;
pub use move_column::*;
pub use swap_window::*;
pub use window_height::*;
use wm_common::{ColumnDisplay, TilingDirection};
use wm_platform::Direction;

pub(crate) use crate::layout::scrolling::focused_window_in;
use crate::{
  commands::{
    container::{
      flatten_split_container, move_container_within_tree,
      wrap_in_split_container, wrap_in_stack_container,
    },
    window::{join_stack, new_stack},
  },
  layout::scrolling::column_of,
  models::{Container, SplitContainer, TilingContainer, Workspace},
  traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// A column of a scrolling layout, with its place on the strip.
pub(crate) struct ColumnPlace {
  pub column: TilingContainer,
  pub workspace: Workspace,

  /// Every column of the workspace, left to right.
  pub columns: Vec<TilingContainer>,

  /// Index of `column` in `columns`.
  pub index: usize,
}

impl ColumnPlace {
  /// The column holding `container`, if it is in a scrolling layout.
  pub fn of(container: &Container) -> Option<Self> {
    let (column, workspace) = column_of(container)?;
    let columns = workspace.tiling_children().collect::<Vec<_>>();
    let index =
      columns.iter().position(|other| other.id() == column.id())?;

    Some(Self {
      column,
      workspace,
      columns,
      index,
    })
  }

  /// The column next to this one in `direction`, if any.
  pub fn neighbour(
    &self,
    direction: &Direction,
  ) -> Option<&TilingContainer> {
    match direction {
      Direction::Left => {
        self.index.checked_sub(1).map(|i| &self.columns[i])
      }
      Direction::Right => self.columns.get(self.index + 1),
      _ => None,
    }
  }
}

/// The rows of `column`, top to bottom: the containers of a vertical
/// split, the tabs of a stack, or else the column itself.
pub(crate) fn rows(column: &TilingContainer) -> Vec<TilingContainer> {
  match column {
    TilingContainer::Split(split)
      if split.tiling_direction() == TilingDirection::Vertical =>
    {
      split.tiling_children().collect()
    }
    TilingContainer::Stack(stack) => stack.tiling_children().collect(),
    _ => vec![column.clone()],
  }
}

/// The row of `column` that holds `container`.
pub(crate) fn row_of(
  container: &Container,
  column: &TilingContainer,
) -> Option<TilingContainer> {
  let rows = rows(column);

  container.self_and_ancestors().find_map(|ancestor| {
    rows.iter().find(|row| row.id() == ancestor.id()).cloned()
  })
}

/// Turns `column` back into its only row, at the same width, when it is a
/// split left with a single row.
pub(crate) fn unwrap_single_row(
  column: &TilingContainer,
) -> anyhow::Result<()> {
  if let TilingContainer::Split(split) = column {
    if !split.is_detached() && split.child_count() == 1 {
      flatten_split_container(split.clone())?;
    }
  }

  Ok(())
}

/// Adds `moved` to `column` as its last row, or as its last tab if it is
/// a stack.
///
/// A column of a single row becomes a vertical split of the two, or a
/// stack of them with `default_column_display: tabbed`.
pub(crate) fn join_column(
  moved: &TilingContainer,
  column: &TilingContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = column
    .parent()
    .ok_or_else(|| anyhow::anyhow!("Column has no parent."))?;

  let is_tabbed = config.value.layouts.scrolling.default_column_display
    == ColumnDisplay::Tabbed;

  let target: Container = match column {
    TilingContainer::Split(split)
      if split.tiling_direction() == TilingDirection::Vertical =>
    {
      split.clone().into()
    }
    TilingContainer::Stack(stack) => stack.clone().into(),
    TilingContainer::TilingWindow(_) if is_tabbed => {
      let stack = new_stack(config);
      wrap_in_stack_container(
        &stack,
        &parent,
        std::slice::from_ref(column),
      )?;
      stack.into()
    }
    _ => {
      let split = SplitContainer::new(
        TilingDirection::Vertical,
        config.value.gaps.clone(),
      );
      wrap_in_split_container(
        &split,
        &parent,
        std::slice::from_ref(column),
      )?;
      split.into()
    }
  };

  if let Some(stack) = target.as_stack() {
    // Only windows are tabs, so anything else joins window by window.
    let windows = moved
      .self_and_descendants()
      .filter_map(|container| container.as_window_container().ok())
      .collect::<Vec<_>>();

    for window in windows {
      join_stack(window, stack, stack.child_count(), state, config)?;
    }
  } else {
    move_container_within_tree(
      &moved.clone().into(),
      &target,
      target.child_count(),
      state,
    )?;
  }

  state.pending_sync.queue_tab_bar_update();

  Ok(())
}

/// Makes `moved` a column of `workspace`, right before `anchor` if
/// `is_before` and else right after it, `size` wide.
pub(crate) fn insert_column(
  moved: &TilingContainer,
  workspace: &Workspace,
  anchor: &TilingContainer,
  is_before: bool,
  size: f32,
  state: &WmState,
) -> anyhow::Result<()> {
  let moved_container: Container = moved.clone().into();

  // `move_container_within_tree` takes a sibling out before putting it
  // back, which shifts everything after it.
  let is_earlier_sibling =
    moved.parent() == anchor.parent() && moved.index() < anchor.index();
  let index = anchor.index() + usize::from(!is_before)
    - usize::from(is_earlier_sibling);

  move_container_within_tree(
    &moved_container,
    &workspace.clone().into(),
    index,
    state,
  )?;

  moved.set_tiling_size(size);

  Ok(())
}
