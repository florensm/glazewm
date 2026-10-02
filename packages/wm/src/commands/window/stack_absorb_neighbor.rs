use anyhow::Context;
use wm_platform::Direction;

use super::{join_stack, new_stack};
use crate::{
  commands::container::wrap_in_stack_container,
  models::{
    StackContainer, TilingContainer, TilingWindow, WindowContainer,
  },
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Absorbs the adjacent tiling neighbor in `direction` into a stack with
/// the focused window.
///
/// If the focused window is already in a `StackContainer`, the neighbor is
/// added to that stack. Otherwise a new stack is created containing both.
/// A neighbouring stack is merged in; a `SplitContainer` neighbor is
/// ignored.
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
    .map_or_else(|| window.clone().into(), |s| s.clone().into());

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

  // A neighbouring stack is merged in, a split isn't absorbed.
  let absorbed: Vec<WindowContainer> = match neighbor {
    TilingContainer::TilingWindow(window) => vec![window.into()],
    TilingContainer::Stack(stack) => stack.windows(),
    TilingContainer::Split(_) => return Ok(()),
  };

  // Get or create the stack to absorb into.
  let stack: StackContainer = if let Some(s) = parent.as_stack().cloned() {
    s
  } else {
    let pivot_parent = pivot.parent().context("No parent.")?;
    let new_stack = new_stack(config);
    wrap_in_stack_container(
      &new_stack,
      &pivot_parent,
      &[window.clone().into()],
    )?;
    new_stack
  };

  // Kept in their order, wherever they go among the tabs.
  let first_index =
    stack.new_tab_index(config.value.stack.new_tab_position);

  for (offset, absorbed_window) in absorbed.into_iter().enumerate() {
    join_stack(
      absorbed_window,
      &stack,
      first_index + offset,
      state,
      config,
    )?;
  }

  // Redraw all stack children and the surrounding layout.
  let stack_parent = stack.parent().context("Stack has no parent.")?;
  state
    .pending_sync
    .queue_containers_to_redraw(stack.windows());
  state
    .pending_sync
    .queue_containers_to_redraw(stack_parent.tiling_children());

  Ok(())
}
