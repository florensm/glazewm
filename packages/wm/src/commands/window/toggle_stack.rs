use anyhow::Context;

use crate::{
  commands::container::{
    flatten_stack_container, wrap_in_stack_container,
  },
  models::{StackContainer, TilingContainer, TilingWindow},
  traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Toggles the focused tiling window into or out of a `StackContainer`.
///
/// If the window is already in a stack, it is moved back to the stack's
/// parent at the stack's position. If the remaining stack has one or fewer
/// children it is flattened. If the window is not in a stack, a new
/// `StackContainer` is created and the window is placed inside it.
pub fn toggle_stack(
  window: &TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = window.parent().context("Window has no parent.")?;

  if let Some(stack) = parent.as_stack() {
    // Window is in a stack — remove it and re-insert after the stack.
    let stack_parent = stack
      .parent()
      .context("Stack container has no parent.")?;

    let stack_index = stack.index();
    let stack_focus_index = stack.focus_index();
    let stack_tiling_size = stack.tiling_size();

    // Scale window's current (stack-relative) size to the parent space.
    let window_tiling_size = stack_tiling_size * window.tiling_size();

    // Collect siblings before restructuring so they can be redrawn after
    // (they may need to transition from Hidden to Showing).
    let stack_siblings: Vec<_> = stack
      .tiling_children()
      .filter(|c| c.id() != window.id())
      .collect();

    // Remove the window from the stack.
    stack
      .borrow_children_mut()
      .retain(|c| c.id() != window.id());

    stack
      .borrow_child_focus_order_mut()
      .retain(|id| *id != window.id());

    *window.borrow_parent_mut() = None;

    // Normalize remaining children so their internal tiling sizes sum to
    // 1.0 after the window is removed.
    let remaining_size: f32 =
      stack_siblings.iter().map(|c| c.tiling_size()).sum();
    if remaining_size > 0.0 {
      for child in &stack_siblings {
        child.set_tiling_size(child.tiling_size() / remaining_size);
      }
    }

    // Shrink the stack by the ejected window's share so the parent's
    // tiling sizes still sum to 1.
    stack.set_tiling_size(stack_tiling_size - window_tiling_size);

    // Re-insert the window into the stack's parent, right after the stack.
    stack_parent
      .borrow_children_mut()
      .insert(stack_index + 1, window.clone().into());

    stack_parent
      .borrow_child_focus_order_mut()
      .insert(stack_focus_index, window.id());

    *window.borrow_parent_mut() = Some(stack_parent.clone());
    window.set_tiling_size(window_tiling_size);

    // Flatten the stack if it is now redundant (0 or 1 children).
    if stack.child_count() <= 1 {
      flatten_stack_container(stack.clone())?;
    }

    // Redraw the ejected window and all former stack siblings. Without
    // this, siblings that were Hidden as inactive stack children remain
    // cloaked after the restructure (ghost windows).
    state.pending_sync.queue_container_to_redraw(window.clone());
    for sibling in stack_siblings {
      state.pending_sync.queue_container_to_redraw(sibling);
    }
  } else {
    // Window is not in a stack — wrap it in a new one.
    let gaps_config = window.gaps_config().clone();
    let tab_bar_height = config.value.stack.tab_bar_height.clone();
    let stack = StackContainer::new(gaps_config, tab_bar_height);
    let tiling_window: TilingContainer = window.clone().into();

    wrap_in_stack_container(&stack, &parent, &[tiling_window])?;

    for child in stack.tiling_children() {
      state.pending_sync.queue_container_to_redraw(child);
    }
  }

  Ok(())
}
