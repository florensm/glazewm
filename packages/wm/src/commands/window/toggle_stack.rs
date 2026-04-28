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
  _config: &UserConfig,
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
    let remaining = stack.child_count();

    // Determine the tiling size for the ejected window.
    // If it is the only child left, it takes the full stack tiling size.
    let window_tiling_size = if remaining == 1 {
      stack_tiling_size
    } else {
      // Scale window's current (stack-relative) size to the parent space.
      stack_tiling_size * window.tiling_size()
    };

    // Remove the window from the stack.
    stack
      .borrow_children_mut()
      .retain(|c| c.id() != window.id());

    stack
      .borrow_child_focus_order_mut()
      .retain(|id| *id != window.id());

    *window.borrow_parent_mut() = None;

    // Re-insert the window into the stack's parent, right after the stack.
    let insert_index = stack_index + 1;
    stack_parent
      .borrow_children_mut()
      .insert(insert_index, window.clone().into());

    stack_parent
      .borrow_child_focus_order_mut()
      .insert(stack_focus_index, window.id());

    *window.borrow_parent_mut() = Some(stack_parent.clone());
    window.set_tiling_size(window_tiling_size);

    // Flatten the stack if it is now redundant (0 or 1 children).
    if stack.child_count() <= 1 {
      flatten_stack_container(stack.clone())?;
    }

    state.pending_sync.queue_container_to_redraw(window.clone());
  } else {
    // Window is not in a stack — wrap it in a new one.
    let gaps_config = window.gaps_config().clone();
    let stack = StackContainer::new(gaps_config);
    let tiling_window: TilingContainer = window.clone().into();

    wrap_in_stack_container(&stack, &parent, &[tiling_window])?;

    for child in stack.tiling_children() {
      state.pending_sync.queue_container_to_redraw(child);
    }
  }

  Ok(())
}
