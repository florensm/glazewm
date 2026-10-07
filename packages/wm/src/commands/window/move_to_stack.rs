use anyhow::Context;
use tracing::info;

use super::{
  join_stack, new_stack, on_auto_stacked, run_window_commands,
  toggle_stack, wrap_window_in_stack,
};
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
pub fn new_named_stack(name: &str, config: &UserConfig) -> StackContainer {
  let stack = new_stack(config);

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

  let stack = new_named_stack(name, config);
  wrap_window_in_stack(&window, &stack, state)?;
  Ok(window)
}

/// Moves an already managed window into its auto-stack, if its current
/// title matches a `stack.auto_stack` rule.
///
/// Covers windows whose title only matched after they were placed. Each
/// window joins at most once, so one taken out of its stack stays out.
/// `is_new` is false for a config reload, which leaves out the rule's
/// actions for new windows (see `on_auto_stacked`).
///
/// A window that joined a stack before it had a title stays there if its
/// title matches the stack's rule, and leaves it otherwise.
pub fn auto_stack_managed_window(
  window: WindowContainer,
  is_new: bool,
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
        return leave_provisional_stack(&window, state, config);
      }
      AutoStackDecision::Skip => {
        return leave_provisional_stack(&window, state, config);
      }
      AutoStackDecision::Wait(_) => return Ok(()),
    };

  let rule = rule.clone();

  if is_in_named_stack(&window, &rule.name) {
    on_auto_stacked(&window, &rule, is_new, state);
    return Ok(());
  }

  let had_focus = window.has_focus(None);

  let window = move_to_stack(window, &rule.name, state, config)?;
  on_auto_stacked(&window, &rule, is_new, state);

  // Make it the active tab, without taking focus from another window.
  let stack = window.parent().context("No parent.")?;
  let end_ancestor = (!had_focus).then_some(&stack);
  set_focused_descendant(&window.clone().into(), end_ancestor);

  state.pending_sync.queue_container_to_redraw(stack);

  Ok(())
}

/// Takes a window that joined its stack before it had a title back out,
/// now that the title it got doesn't match, and runs the commands of its
/// window rules that were skipped to keep it in the stack.
fn leave_provisional_stack(
  window: &WindowContainer,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let native = window.native().clone();

  let Some(join) = state.auto_stack.take_provisional(native.id()) else {
    return Ok(());
  };

  if !is_in_named_stack(window, &join.stack_name) {
    return Ok(());
  }

  info!(
    "Taking window out of stack '{}', as its title doesn't match: \
     {window}",
    join.stack_name
  );

  toggle_stack(window, state, config)?;

  if let Some(window) = state.window_from_native(&native) {
    run_window_commands(window, &join.skipped_commands, state, config)?;
  }

  Ok(())
}

/// Whether `window` is a tab of the stack named `name`.
fn is_in_named_stack(window: &WindowContainer, name: &str) -> bool {
  window
    .parent()
    .and_then(|parent| parent.as_stack().and_then(StackContainer::name))
    .is_some_and(|stack_name| stack_name == name)
}

/// Whether `window` was put in its current stack by an auto-stack rule,
/// including one it joined before it had a title.
pub fn is_auto_stacked(window: &WindowContainer, state: &WmState) -> bool {
  let id = window.native().id();

  (state.auto_stack.is_settled(id)
    || state.auto_stack.provisional(id).is_some())
    && window
      .parent()
      .is_some_and(|parent| parent.as_stack().is_some())
}

#[cfg(test)]
mod tests {
  use wm_common::{
    AutoStackRuleConfig, DuplicateTabs, JoinUntitledConfig, MatchType,
    ParsedConfig, StackConfig, WindowMatchConfig,
  };

  use super::auto_stack_managed_window;
  use crate::{
    auto_stack::ProvisionalJoin,
    models::{Monitor, StackContainer, TilingWindow, Workspace},
    traits::{CommonGetters, WindowGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  fn config() -> UserConfig {
    UserConfig::from_parsed(ParsedConfig {
      stack: StackConfig {
        auto_stack: vec![AutoStackRuleConfig {
          name: "details".to_string(),
          match_window: vec![WindowMatchConfig {
            window_process: Some(MatchType::Equals {
              equals: "MyApp".to_string(),
            }),
            window_title: Some(MatchType::Regex {
              regex: "^Details for".to_string(),
            }),
            ..WindowMatchConfig::default()
          }],
          exclude: vec![],
          workspace: None,
          allow_owned: true,
          send_keys_on_join: Vec::new(),
          duplicates: DuplicateTabs::Keep,
          join_untitled: Some(JoinUntitledConfig::default()),
        }],
        ..StackConfig::default()
      },
      ..ParsedConfig::default()
    })
  }

  /// A "details" stack with a window and another that joined it untitled
  /// and since got `title`.
  fn setup(
    title: &str,
  ) -> (
    std::mem::ManuallyDrop<WmState>,
    StackContainer,
    TilingWindow,
  ) {
    let first = TilingWindow::mock()
      .process_name("MyApp".to_string())
      .title("Details for 1".to_string())
      .call();
    let joined = TilingWindow::mock()
      .process_name("MyApp".to_string())
      .title(title.to_string())
      .call();
    let stack = StackContainer::mock()
      .name("details".to_string())
      .tiling_containers(vec![first.into(), joined.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();
    let monitor = Monitor::mock().workspaces(vec![workspace]).call();

    // Never dropped, which would restore its mock windows via Win32 calls.
    let mut state =
      std::mem::ManuallyDrop::new(WmState::mock(vec![monitor]));
    state.auto_stack.join_provisionally(
      joined.native().id(),
      ProvisionalJoin {
        stack_name: "details".to_string(),
        skipped_commands: Vec::new(),
      },
    );

    (state, stack, joined)
  }

  #[test]
  fn untitled_tab_stays_once_its_title_matches() {
    let (mut state, stack, joined) = setup("Details for 2");

    auto_stack_managed_window(
      joined.clone().into(),
      true,
      &mut state,
      &mut config(),
    )
    .unwrap();

    assert_eq!(joined.parent().unwrap().id(), stack.id());
    assert!(state.auto_stack.is_settled(joined.native().id()));
    assert!(state.auto_stack.provisional(joined.native().id()).is_none());
  }

  #[test]
  fn untitled_tab_leaves_if_its_title_does_not_match() {
    let (mut state, stack, joined) = setup("MyApp");

    auto_stack_managed_window(
      joined.clone().into(),
      true,
      &mut state,
      &mut config(),
    )
    .unwrap();

    assert_ne!(joined.parent().unwrap().id(), stack.id());
    assert!(state.auto_stack.provisional(joined.native().id()).is_none());
  }

  #[test]
  fn untitled_tab_stays_while_untitled() {
    let (mut state, stack, joined) = setup("");

    auto_stack_managed_window(
      joined.clone().into(),
      true,
      &mut state,
      &mut config(),
    )
    .unwrap();

    assert_eq!(joined.parent().unwrap().id(), stack.id());
    assert!(state.auto_stack.provisional(joined.native().id()).is_some());
  }
}
