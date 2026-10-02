use anyhow::Context;
use tracing::info;
use wm_common::WindowState;

use super::update_window_state;
use crate::{
  auto_stack::{decide, AutoStackDecision, WindowTraits},
  commands::container::{
    move_container_within_tree, set_focused_descendant,
    wrap_in_stack_container,
  },
  models::{
    StackContainer, TilingContainer, TilingWindow, WindowContainer,
  },
  traits::{CommonGetters, TilingSizeGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Finds the stack named `name` on any workspace.
pub fn find_named_stack(
  state: &WmState,
  name: &str,
) -> Option<StackContainer> {
  state
    .root_container
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .find(|stack| stack.name().as_deref() == Some(name))
}

/// Creates an empty stack named `name`, ready to receive windows.
pub fn new_named_stack(
  name: &str,
  gaps_config: &wm_common::GapsConfig,
  config: &UserConfig,
) -> StackContainer {
  let stack = StackContainer::new(
    gaps_config.clone(),
    config.value.stack.tab_bar_height.clone(),
    config.value.stack.tab_bar_position.clone(),
  );

  stack.set_name(name.to_string());
  stack
}

/// Moves `window` into the stack named `name`, which may be on any
/// workspace.
///
/// When no such stack exists, one is created in place of the window. No-op
/// if the window is already in it.
pub fn move_to_stack(
  window: &TilingWindow,
  name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(stack) = find_named_stack(state, name) else {
    let stack = new_named_stack(name, &window.gaps_config(), config);
    let parent = window.parent().context("No parent.")?;

    wrap_in_stack_container(
      &stack,
      &parent,
      &[TilingContainer::TilingWindow(window.clone())],
    )?;

    state.pending_sync.queue_container_to_redraw(stack);
    return Ok(());
  };

  if window
    .parent()
    .is_some_and(|parent| parent.id() == stack.id())
  {
    return Ok(());
  }

  let previous_workspace = window.workspace().context("No workspace.")?;

  move_container_within_tree(
    &window.clone().into(),
    &stack.clone().into(),
    stack.child_count(),
    state,
  )?;

  state
    .pending_sync
    .queue_container_to_redraw(previous_workspace)
    .queue_container_to_redraw(stack.parent().context("No parent.")?);

  Ok(())
}

/// Moves an already managed window into its auto-stack, if its current
/// title matches a `stack.auto_stack` rule.
///
/// Covers windows whose title only matched after they were placed. Each
/// window joins at most once, so one taken out of its stack stays out.
pub fn auto_stack_managed_window(
  window: WindowContainer,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let native_id = window.native().id();

  if state.auto_stack.is_settled(native_id) {
    return Ok(());
  }

  let properties = window.native_properties();
  let traits = WindowTraits::of(&window.native(), &properties);

  let AutoStackDecision::Join(rule) =
    decide(&config.value.stack.auto_stack, &properties, traits)
  else {
    return Ok(());
  };

  let name = rule.name.clone();
  let had_focus = window.has_focus(None);
  info!("Auto-stacking window into stack '{name}': {window}");

  let window = match window {
    WindowContainer::TilingWindow(window) => window,
    WindowContainer::NonTilingWindow(_) => {
      match update_window_state(
        window,
        WindowState::Tiling,
        state,
        config,
      )? {
        WindowContainer::TilingWindow(window) => window,
        WindowContainer::NonTilingWindow(_) => return Ok(()),
      }
    }
  };

  move_to_stack(&window, &name, state, config)?;
  state.auto_stack.mark_settled(native_id);

  // Make it the active tab, without taking focus from another window.
  let stack = window.parent().context("No parent.")?;
  let end_ancestor = (!had_focus).then_some(&stack);
  set_focused_descendant(&window.clone().into(), end_ancestor);

  state.pending_sync.queue_container_to_redraw(stack);

  Ok(())
}
