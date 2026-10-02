use wm_common::{VecDequeExt, WindowState};

use super::{join_stack, wrap_window_in_stack};
use crate::{
  commands::container::flatten_stack_container,
  models::{Container, StackContainer, Workspace},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Puts all tiling windows of `workspace` into one stack: the stack of
/// its most recently focused tiling window, or a new one in that
/// window's place.
pub fn stack_all(
  workspace: &Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(anchor) = workspace
    .descendant_focus_order()
    .find(Container::is_tiling_window)
    .and_then(|container| container.as_window_container().ok())
  else {
    return Ok(());
  };

  let stack = if let Some(stack) = anchor
    .parent()
    .and_then(|parent| parent.as_stack().cloned())
  {
    stack
  } else {
    let stack = StackContainer::new(
      config.value.gaps.clone(),
      config.value.stack.tab_bar_height.clone(),
      config.value.stack.tab_bar_position.clone(),
    );

    wrap_window_in_stack(&anchor, &stack, state)?;
    stack
  };

  // In layout order, so the tabs read like the windows did.
  let others = workspace
    .descendants()
    .filter_map(|container| container.as_window_container().ok())
    .filter(|window| {
      window.state() == WindowState::Tiling
        && window
          .parent()
          .is_none_or(|parent| parent.id() != stack.id())
    })
    .collect::<Vec<_>>();

  for window in others {
    let index = stack.child_count();
    join_stack(window, &stack, index, state, config)?;
  }

  state
    .pending_sync
    .queue_containers_to_redraw(workspace.tiling_children());

  Ok(())
}

/// Takes apart every stack on `workspace`, tiling or floating.
///
/// The windows of a floating stack are cascaded so that they don't hide
/// each other. Unstacked windows are never auto-stacked again.
pub fn unstack_all(
  workspace: &Workspace,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let stacks = workspace
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .collect::<Vec<_>>();

  for stack in stacks {
    let windows = stack.windows();

    for (index, window) in windows.iter().enumerate() {
      state.auto_stack.mark_settled(window.native().id());

      if !stack.is_tiling() {
        let placement = window.floating_placement();
        let offset = i32::try_from(index).unwrap_or(0) * 32;
        window.set_own_floating_placement(
          placement.translate_to_coordinates(
            placement.x() + offset,
            placement.y() + offset,
          ),
        );
      }
    }

    flatten_stack_container(stack)?;
    state.pending_sync.queue_containers_to_redraw(windows);
  }

  state
    .pending_sync
    .queue_containers_to_redraw(workspace.tiling_children())
    .queue_workspace_to_reorder(workspace.clone());

  Ok(())
}

/// Moves the active tab of the stack `container` is in (or is) one
/// position right, or left if `prev`, wrapping around.
pub fn move_stack_tab(
  container: &Container,
  prev: bool,
  state: &mut WmState,
) {
  let Some(stack) = container
    .as_stack()
    .cloned()
    .or_else(|| container.parent().and_then(|p| p.as_stack().cloned()))
  else {
    return;
  };

  let Some(active) = stack.active_child() else {
    return;
  };

  let count = stack.child_count();
  let index = active.index();
  let target = if prev {
    (index + count - 1) % count
  } else {
    (index + 1) % count
  };

  stack.borrow_children_mut().shift_to_index(target, active);
  state.pending_sync.queue_tab_bar_update();
}

#[cfg(test)]
mod tests {
  use wm_common::ParsedConfig;

  use super::{move_stack_tab, stack_all, unstack_all};
  use crate::{
    commands::container::set_focused_descendant,
    models::{
      Monitor, SplitContainer, StackContainer, TilingWindow,
      WindowContainer, Workspace,
    },
    traits::{CommonGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// The windows of `workspace` that are in a stack.
  fn stacked_windows(workspace: &Workspace) -> Vec<WindowContainer> {
    workspace
      .descendants()
      .filter_map(|container| container.as_window_container().ok())
      .filter(|window| {
        window
          .parent()
          .is_some_and(|parent| parent.as_stack().is_some())
      })
      .collect()
  }

  fn run(workspace: &Workspace, test: impl FnOnce(&mut WmState)) {
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let mut state = WmState::mock(vec![monitor]);
    test(&mut state);

    // Dropping `WmState` would restore its mock windows via Win32 calls.
    std::mem::forget(state);
  }

  #[test]
  fn stack_all_gathers_every_tiling_window() {
    let left = TilingWindow::mock().call();
    let top = TilingWindow::mock().call();
    let bottom = TilingWindow::mock().call();
    let split = SplitContainer::mock()
      .tiling_containers(vec![top.into(), bottom.into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![left.clone().into(), split.into()])
      .call();
    let config = UserConfig::from_parsed(ParsedConfig::default());

    run(&workspace, |state| {
      set_focused_descendant(&left.clone().into(), None);
      stack_all(&workspace, state, &config).unwrap();
    });

    let stack = left.parent().unwrap();
    assert!(stack.as_stack().is_some());
    assert_eq!(stack.child_count(), 3);
    assert_eq!(workspace.child_count(), 1);
    assert!(
      (stack.as_tiling_container().unwrap().tiling_size() - 1.0).abs()
        < 1e-4
    );
  }

  #[test]
  fn unstack_all_flattens_every_stack() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .name("details".to_string())
      .tiling_containers(vec![first.into(), second.into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();

    run(&workspace, |state| unstack_all(&workspace, state).unwrap());

    assert!(stack.is_detached());
    assert!(stacked_windows(&workspace).is_empty());
    assert_eq!(workspace.tiling_children().count(), 2);
  }

  #[test]
  fn move_stack_tab_wraps_around() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();

    run(&workspace, |state| {
      set_focused_descendant(&second.clone().into(), None);
      move_stack_tab(&second.clone().into(), false, state);
    });

    assert_eq!(second.index(), 0);
    assert_eq!(first.index(), 1);
  }
}
