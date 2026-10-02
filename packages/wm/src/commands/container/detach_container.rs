use anyhow::Context;

use super::{flatten_split_container, flatten_stack_container};
use crate::{
  models::Container,
  traits::{CommonGetters, TilingSizeGetters, MIN_TILING_SIZE},
};

/// Removes a container from the tree.
///
/// If the container is a tiling container, the siblings will be resized to
/// fill the freed up space. Will flatten empty parent split containers.
#[allow(clippy::needless_pass_by_value)]
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
  // All tabs of a stack share its rect, so removing one leaves the stack's
  // size, and thereby the rest of the layout, untouched. A stack that is
  // redundant afterwards is replaced by its remaining window, which takes
  // over the stack's whole slot.
  if let Some(stack) = child_to_remove
    .parent()
    .and_then(|parent| parent.as_stack().cloned())
  {
    let was_tiling = stack.is_tiling();

    stack
      .borrow_children_mut()
      .retain(|c| c.id() != child_to_remove.id());

    stack
      .borrow_child_focus_order_mut()
      .retain(|id| *id != child_to_remove.id());

    *child_to_remove.borrow_parent_mut() = None;

    if !stack.has_children() {
      if was_tiling {
        return detach_container(stack.into());
      }

      // Empty, the stack would count as tiling, but it held non-tiling
      // windows and so has no tiling slot to give back.
      let parent = stack.parent().context("No parent.")?;
      parent
        .borrow_children_mut()
        .retain(|c| c.id() != stack.id());
      parent
        .borrow_child_focus_order_mut()
        .retain(|id| *id != stack.id());
      *stack.borrow_parent_mut() = None;

      return Ok(());
    }

    if stack.is_redundant_with(stack.child_count()) {
      flatten_stack_container(stack)?;
    }

    return Ok(());
  }

  // Flatten the parent split container if it'll be empty after removing
  // the child.
  if let Some(split_parent) = child_to_remove
    .parent()
    .and_then(|parent| parent.as_split().cloned())
  {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent)?;
    }
  }

  let parent = child_to_remove.parent().context("No parent.")?;

  parent
    .borrow_children_mut()
    .retain(|c| c.id() != child_to_remove.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != child_to_remove.id());

  *child_to_remove.borrow_parent_mut() = None;

  // Resize the siblings if it is a tiling container.
  if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
    let tiling_siblings = parent.tiling_children().collect::<Vec<_>>();

    // TODO: Share logic with `resize_tiling_container`.
    let available_size =
      tiling_siblings.iter().fold(0.0, |sum, container| {
        sum + container.tiling_size() - MIN_TILING_SIZE
      });

    // Adjust size of the siblings based on the freed up space.
    for sibling in &tiling_siblings {
      let resize_factor =
        (sibling.tiling_size() - MIN_TILING_SIZE) / available_size;

      let size_delta = resize_factor * child_to_remove.tiling_size();
      sibling.set_tiling_size(sibling.tiling_size() + size_delta);
    }
  }

  Ok(())
}
