use std::collections::VecDeque;

use anyhow::Context;

use crate::{
  models::StackContainer,
  traits::{CommonGetters, TilingSizeGetters},
};

/// Removes a `StackContainer` from the tree and moves its children into
/// the parent container.
///
/// The stack's size is split evenly between its children.
#[allow(clippy::needless_pass_by_value)]
pub fn flatten_stack_container(
  stack: StackContainer,
) -> anyhow::Result<()> {
  let parent = stack.parent().context("No parent.")?;

  // Tabs all fill the stack, so their own tiling sizes carry no layout
  // information. Split the stack's slot evenly instead.
  #[allow(clippy::cast_precision_loss)]
  let child_size = stack.tiling_size() / stack.child_count().max(1) as f32;

  let updated_children = stack.children().into_iter().inspect(|child| {
    *child.borrow_parent_mut() = Some(parent.clone());

    if let Ok(tiling_child) = child.as_tiling_container() {
      tiling_child.set_tiling_size(child_size);
    }
  });

  let index = stack.index();
  let focus_index = stack.focus_index();

  // Insert children at the stack's original position in the parent.
  for (child_index, child) in updated_children.enumerate() {
    parent
      .borrow_children_mut()
      .insert(index + child_index, child);
  }

  // Insert children at the stack's original focus position in the parent.
  for (child_focus_index, child_id) in
    stack.borrow_child_focus_order().iter().enumerate()
  {
    parent
      .borrow_child_focus_order_mut()
      .insert(focus_index + child_focus_index, *child_id);
  }

  // Remove the stack container from the tree.
  parent
    .borrow_children_mut()
    .retain(|c| c.id() != stack.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != stack.id());

  *stack.borrow_parent_mut() = None;
  *stack.borrow_children_mut() = VecDeque::new();

  Ok(())
}
