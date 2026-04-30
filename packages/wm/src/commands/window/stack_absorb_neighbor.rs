use anyhow::Context;
use wm_platform::Direction;

use crate::{
  commands::container::{
    move_container_within_tree, wrap_in_stack_container,
  },
  models::{StackContainer, TilingContainer, TilingWindow},
  traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Absorbs the adjacent tiling neighbor in `direction` into a stack with
/// the focused window.
///
/// If the focused window is already in a `StackContainer`, the neighbor is
/// added to that stack. Otherwise a new stack is created containing both.
/// Only `TilingWindow` neighbors are supported; `SplitContainer` and
/// `StackContainer` neighbors are ignored.
pub fn stack_absorb_neighbor(
  window: &TilingWindow,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = window.parent().context("Window has no parent.")?;

  // The pivot is what we search siblings from: the parent stack (if in
  // one) or the bare window.
  let pivot: crate::models::Container = parent
    .as_stack()
    .map(|s| s.clone().into())
    .unwrap_or_else(|| window.clone().into());

  // Find the adjacent tiling sibling of `pivot` in `direction`.
  let neighbor: TilingContainer = match direction {
    Direction::Up | Direction::Left => pivot
      .prev_siblings()
      .find_map(|s| s.as_tiling_container().ok()),
    _ => pivot
      .next_siblings()
      .find_map(|s| s.as_tiling_container().ok()),
  }
  .context("No tiling neighbor in that direction.")?;

  // Only absorb bare tiling windows for now.
  let TilingContainer::TilingWindow(neighbor_window) = neighbor else {
    return Ok(());
  };

  // Get or create the stack to absorb into.
  let stack: StackContainer = if let Some(s) = parent.as_stack().cloned() {
    s
  } else {
    let pivot_parent = pivot.parent().context("No parent.")?;
    let new_stack = StackContainer::new(
      window.gaps_config().clone(),
      config.value.stack.tab_bar_height.clone(),
    );
    wrap_in_stack_container(
      &new_stack,
      &pivot_parent,
      &[window.clone().into()],
    )?;
    new_stack
  };

  let stack_container: crate::models::Container = stack.clone().into();

  move_container_within_tree(
    &neighbor_window.into(),
    &stack_container,
    0,
    state,
  )?;

  // Redraw all stack children and the surrounding layout.
  let stack_parent = stack.parent().context("Stack has no parent.")?;
  for child in stack.tiling_children() {
    state.pending_sync.queue_container_to_redraw(child);
  }
  state
    .pending_sync
    .queue_containers_to_redraw(stack_parent.tiling_children());

  Ok(())
}
