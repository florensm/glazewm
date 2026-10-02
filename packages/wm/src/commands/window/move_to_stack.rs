use anyhow::Context;
use tracing::info;

use super::{join_stack, wrap_window_in_stack};
use crate::{
  auto_stack::{decide, AutoStackDecision, WindowTraits},
  commands::container::set_focused_descendant,
  models::{StackContainer, WindowContainer},
  traits::{CommonGetters, WindowGetters},
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
/// workspace, giving it the stack's state.
///
/// When no such stack exists, one is created in place of the window.
/// Returns the window, which is a new container if its state changed
/// between tiling and non-tiling.
pub fn move_to_stack(
  window: WindowContainer,
  name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  if let Some(stack) = find_named_stack(state, name) {
    let index = stack.new_tab_index(config.value.stack.new_tab_position);
    return join_stack(window, &stack, index, state, config);
  }

  let stack = new_named_stack(name, &config.value.gaps, config);
  wrap_window_in_stack(&window, &stack, state)?;
  Ok(window)
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

  let rule =
    match decide(&config.value.stack.auto_stack, &properties, traits) {
      AutoStackDecision::Join(rule) => rule,
      AutoStackDecision::Blocked(reason) => {
        info!(
          "Not auto-stacking window '{}' because {reason}.",
          properties.title
        );
        return Ok(());
      }
      AutoStackDecision::Wait | AutoStackDecision::Skip => return Ok(()),
    };

  let name = rule.name.clone();
  let had_focus = window.has_focus(None);
  info!("Auto-stacking window into stack '{name}': {window}");

  let window = move_to_stack(window, &name, state, config)?;
  state.auto_stack.mark_settled(native_id);

  // Make it the active tab, without taking focus from another window.
  let stack = window.parent().context("No parent.")?;
  let end_ancestor = (!had_focus).then_some(&stack);
  set_focused_descendant(&window.clone().into(), end_ancestor);

  state.pending_sync.queue_container_to_redraw(stack);

  Ok(())
}

/// Whether `window` was put in its current stack by an auto-stack rule.
pub fn is_auto_stacked(window: &WindowContainer, state: &WmState) -> bool {
  state.auto_stack.is_settled(window.native().id())
    && window
      .parent()
      .is_some_and(|parent| parent.as_stack().is_some())
}
