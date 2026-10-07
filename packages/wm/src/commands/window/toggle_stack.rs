use anyhow::Context;
use wm_common::WindowState;

use super::{float_out_of_stack, new_stack, wrap_window_in_stack};
use crate::{
  commands::container::flatten_stack_container,
  models::{StackContainer, TilingWindow, WindowContainer},
  traits::{CommonGetters, TilingSizeGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Toggles `window` into or out of a `StackContainer`.
///
/// A window taken out of a tiling stack is tiled right after it, and one
/// taken out of a floating stack floats on its own. A window not in a
/// stack is wrapped in a new stack in its place, keeping its state.
pub fn toggle_stack(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = window.parent().context("Window has no parent.")?;

  match (window, parent.as_stack()) {
    (WindowContainer::TilingWindow(window), Some(stack)) => {
      remove_from_tiling_stack(window, stack, state)
    }
    (WindowContainer::NonTilingWindow(_), Some(_)) => {
      float_out_of_stack(window, state, config)
    }
    (_, None) if window.state() == WindowState::Minimized => Ok(()),
    (_, None) => wrap_window_in_stack(window, &new_stack(config), state),
  }
}

/// Moves `window` out of its tiling `stack`, tiled right after it.
pub(crate) fn remove_from_tiling_stack(
  window: &TilingWindow,
  stack: &StackContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let stack_parent =
    stack.parent().context("Stack container has no parent.")?;

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

  // A window taken out of a stack is never auto-stacked again.
  state.auto_stack.mark_settled(window.native().id());

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
  let remaining_size: f32 = stack_siblings
    .iter()
    .map(TilingSizeGetters::tiling_size)
    .sum();
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

  if stack.is_redundant_with(stack.child_count()) {
    flatten_stack_container(stack.clone())?;
  }

  // Redraw the ejected window and all former stack siblings. Without
  // this, siblings that were Hidden as inactive stack children remain
  // cloaked after the restructure (ghost windows).
  state.pending_sync.queue_container_to_redraw(window.clone());
  for sibling in stack_siblings {
    state.pending_sync.queue_container_to_redraw(sibling);
  }

  Ok(())
}
