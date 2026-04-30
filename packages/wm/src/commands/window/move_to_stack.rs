use anyhow::Context;

use crate::{
  commands::container::{move_container_within_tree, wrap_in_stack_container},
  models::{StackContainer, TilingContainer, TilingWindow},
  traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves `window` into the named stack on its workspace.
///
/// If a `StackContainer` with the given name already exists on the workspace,
/// the window is appended to it. Otherwise a new single-window named stack is
/// created in place of the window. If the window is already in the target
/// stack this is a no-op.
pub fn move_to_stack(
  window: &TilingWindow,
  name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace = window.workspace().context("No workspace.")?;

  // Find an existing named stack on this workspace.
  let existing_stack = workspace
    .descendants()
    .filter_map(|c| c.as_stack().cloned())
    .find(|s| s.name().as_deref() == Some(name));

  if let Some(stack) = existing_stack {
    // Already in this stack — no-op.
    if window.parent().map(|p| p.id()) == Some(stack.id()) {
      return Ok(());
    }

    let insert_index = stack.child_count();
    move_container_within_tree(
      &window.clone().into(),
      &stack.clone().into(),
      insert_index,
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
  } else {
    // Create a new named stack wrapping this window.
    let stack = StackContainer::new(
      window.gaps_config().clone(),
      config.value.stack.tab_bar_height.clone(),
    );
    stack.set_name(name.to_string());

    let parent = window.parent().context("No parent.")?;
    wrap_in_stack_container(
      &stack,
      &parent,
      &[TilingContainer::TilingWindow(window.clone())],
    )?;

    for child in stack.tiling_children() {
      state.pending_sync.queue_container_to_redraw(child);
    }
  }

  Ok(())
}
