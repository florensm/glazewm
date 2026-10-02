use anyhow::Context;
use wm_common::WindowState;

use super::{join_stack, wrap_window_in_stack};
use crate::{
  models::{StackContainer, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves `window` into a stack with the most recently focused other
/// window on the same workspace.
///
/// If that window is in a stack, `window` joins it; otherwise both are put
/// in a new stack in its place. The stack keeps the other window's state,
/// so stacking onto a floating window makes a floating stack.
///
/// No-op when no other window that isn't minimized exists on the
/// workspace.
pub fn stack_insert(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace = window.workspace().context("No workspace.")?;
  let own_stack =
    window.parent().filter(|parent| parent.as_stack().is_some());

  let target = workspace
    .descendant_focus_order()
    .filter_map(|container| container.as_window_container().ok())
    .find(|other| {
      other.id() != window.id()
        && other.state() != WindowState::Minimized
        && other.parent() != own_stack
    });

  let Some(target) = target else {
    return Ok(());
  };

  let stack = if let Some(stack) =
    target.parent().and_then(|p| p.as_stack().cloned())
  {
    stack
  } else {
    let stack = StackContainer::new(
      config.value.gaps.clone(),
      config.value.stack.tab_bar_height.clone(),
      config.value.stack.tab_bar_position.clone(),
    );

    wrap_window_in_stack(&target, &stack, state)?;
    stack
  };

  let index = stack.new_tab_index(config.value.stack.new_tab_position);
  join_stack(window, &stack, index, state, config)?;

  Ok(())
}
