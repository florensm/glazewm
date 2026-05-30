use super::flatten_split_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingDirectionGetters},
};

/// Flattens any redundant split containers at the top-level of the given
/// parent container.
///
/// Only runs for containers whose active layout is a split variant; other
/// layouts (e.g. `Grid`, `Columns`) keep their child split containers as-is.
///
/// # Example
///
/// ```ignore
/// H[1 H[V[2, 3]]] -> H[1, 2, 3]
/// H[1 H[2, 3]]    -> H[1, 2, 3]
/// H[V[1]]         -> V[1]
/// ```
pub fn flatten_child_split_containers(
  parent: &Container,
) -> anyhow::Result<()> {
  let Ok(parent) = parent.as_direction_container() else {
    return Ok(());
  };

  // Only flatten within split-type layouts; non-split layouts (Grid,
  // Columns, etc.) intentionally keep their child split containers.
  if !parent.layout().is_split() {
    return Ok(());
  }

  // Get children that are either tiling windows or split containers.
  let tiling_children = parent
    .children()
    .into_iter()
    .filter(|child| child.is_tiling_window() || child.is_split())
    .collect::<Vec<_>>();

  if tiling_children.len() == 1 {
    // Handle case where the parent is a split container and has a
    // single split container child.
    if let Some(split_child) = tiling_children[0].as_split() {
      flatten_split_container(split_child.clone())?;
      parent.set_layout(parent.layout().inverse());
    }
  } else {
    let split_children = tiling_children
      .into_iter()
      .filter_map(|child| child.as_split().cloned())
      .collect::<Vec<_>>();

    for split_child in split_children.iter().filter(|split_child| {
      // Only collapse splits whose direction matches the parent.
      split_child.layout().is_split()
        && split_child.tiling_direction()
          == parent.tiling_direction()
    }) {
      // Additionally flatten redundant top-level split containers in
      // the child.
      if split_child.child_count() == 1 {
        if let Some(split_grandchild) =
          split_child.children()[0].as_split()
        {
          flatten_split_container(split_grandchild.clone())?;
        }
      }

      flatten_split_container(split_child.clone())?;
    }
  }

  Ok(())
}
