use anyhow::Context;

use crate::{
  commands::container::{
    move_container_within_tree, wrap_in_stack_container,
  },
  models::{StackContainer, TilingWindow},
  traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves the focused tiling window into a stack with the most-recently-
/// focused other tiling window on the same workspace.
///
/// If the target window already belongs to a `StackContainer`, the focused
/// window is added to that stack. Otherwise a new `StackContainer` is
/// created wrapping the target, and the focused window is added to it.
///
/// No-op when no other tiling window exists on the workspace.
pub fn stack_insert(
  window: &TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace = window.workspace().context("No workspace.")?;

  // Find the most-recently-focused tiling window on the workspace that is
  // not the current window.
  let focus_order: Vec<_> = workspace.descendant_focus_order().collect();
  let target = focus_order
    .iter()
    .filter_map(|c| c.as_tiling_window().cloned())
    .find(|w| w.id() != window.id());

  let Some(target) = target else {
    return Ok(());
  };

  let target_parent = target.parent().context("No parent.")?;

  let stack: StackContainer =
    if let Some(s) = target_parent.as_stack().cloned() {
      s
    } else {
      let new_stack = StackContainer::new(
        target.gaps_config().clone(),
        config.value.stack.tab_bar_height.clone(),
      );
      wrap_in_stack_container(
        &new_stack,
        &target_parent,
        &[target.clone().into()],
      )?;
      new_stack
    };

  move_container_within_tree(
    &window.clone().into(),
    &stack.clone().into(),
    0,
    state,
  )?;

  for child in stack.tiling_children() {
    state.pending_sync.queue_container_to_redraw(child);
  }

  if let Some(p) = stack.parent() {
    state
      .pending_sync
      .queue_containers_to_redraw(p.tiling_children());
  }

  Ok(())
}
