use std::collections::VecDeque;

use anyhow::Context;

use crate::{
  models::{Container, TilingContainer},
  traits::{CommonGetters, TilingSizeGetters},
};

/// Inserts `container` (a split or stack) into the tree where
/// `target_children` are, adopting them.
pub fn wrap_in_container(
  container: &(impl TilingSizeGetters + Clone + Into<Container>),
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
    .insert(starting_index, container.clone().into());

  let starting_focus_index = target_children
    .iter()
    .map(CommonGetters::focus_index)
    .min()
    .context("Failed to get starting focus index.")?;

  target_parent
    .borrow_child_focus_order_mut()
    .insert(starting_focus_index, container.id());

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

  // Set the container's parent and tiling size.
  *container.borrow_parent_mut() = Some(target_parent.clone());
  container.set_tiling_size(total_tiling_size);

  // Move the children from their original parent to the container.
  for target_child in target_children {
    *target_child.borrow_parent_mut() = Some(container.clone().into());

    container
      .borrow_children_mut()
      .push_back(target_child.clone().into());

    target_parent
      .borrow_children_mut()
      .retain(|child| child != &target_child.clone().into());

    target_parent
      .borrow_child_focus_order_mut()
      .retain(|id| id != &target_child.id());

    // Scale the tiling size to the new container.
    target_child
      .set_tiling_size(target_child.tiling_size() / total_tiling_size);
  }

  // Add original focus order to the container.
  *container.borrow_child_focus_order_mut() = sorted_focus_ids;

  Ok(())
}
