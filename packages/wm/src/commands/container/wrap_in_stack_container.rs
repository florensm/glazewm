use std::collections::VecDeque;

use anyhow::Context;

use crate::{
  models::{Container, StackContainer, TilingContainer},
  traits::{CommonGetters, TilingSizeGetters},
};

/// Inserts a `StackContainer` into the tree, adopting the given children.
///
/// Mirrors `wrap_in_split_container`, but for `StackContainer` (no direction
/// parameter needed).
pub fn wrap_in_stack_container(
  stack: &StackContainer,
  target_parent: &Container,
  target_children: &[TilingContainer],
) -> anyhow::Result<()> {
  let starting_index = target_children
    .iter()
    .map(CommonGetters::index)
    .min()
    .context("Failed to get starting index.")?;

  target_parent
    .borrow_children_mut()
    .insert(starting_index, stack.clone().into());

  let starting_focus_index = target_children
    .iter()
    .map(CommonGetters::focus_index)
    .min()
    .context("Failed to get starting focus index.")?;

  target_parent
    .borrow_child_focus_order_mut()
    .insert(starting_focus_index, stack.id());

  // Get the total tiling size amongst all children.
  let total_tiling_size = target_children
    .iter()
    .map(TilingSizeGetters::tiling_size)
    .sum::<f32>();

  let target_children_ids = target_children
    .iter()
    .map(CommonGetters::id)
    .collect::<Vec<_>>();

  let sorted_focus_ids = target_parent
    .borrow_child_focus_order()
    .iter()
    .filter(|id| target_children_ids.contains(id))
    .copied()
    .collect::<VecDeque<_>>();

  // Set the stack container's parent and tiling size.
  *stack.borrow_parent_mut() = Some(target_parent.clone());
  stack.set_tiling_size(total_tiling_size);

  // Move the children from their original parent to the stack container.
  for target_child in target_children {
    *target_child.borrow_parent_mut() = Some(stack.clone().into());

    stack
      .borrow_children_mut()
      .push_back(target_child.clone().into());

    target_parent
      .borrow_children_mut()
      .retain(|child| child != &target_child.clone().into());

    target_parent
      .borrow_child_focus_order_mut()
      .retain(|id| id != &target_child.id());

    // Scale the tiling size to the new stack container.
    target_child
      .set_tiling_size(target_child.tiling_size() / total_tiling_size);
  }

  // Add original focus order to stack container.
  *stack.borrow_child_focus_order_mut() = sorted_focus_ids;

  Ok(())
}
