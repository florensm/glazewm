//! Carrying out what is picked in the overview.

use anyhow::Context;
use wm_common::{
  InvokeCommand, InvokeFocusCommand, InvokeMoveCommand, WindowState,
};
use wm_platform::OverviewAction;

use crate::{
  commands::{
    general::platform_sync,
    window::update_window_state,
    workspace::{activate_workspace, summon_special_workspace},
  },
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm::WindowManager,
  wm_state::WmState,
};

impl WindowManager {
  /// Carries out an action taken in the overview, unless it comes from an
  /// earlier session than the open one.
  #[allow(clippy::too_many_lines)]
  pub fn process_overview_action(
    &mut self,
    session: u64,
    action: OverviewAction,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let state = &mut self.state;

    // What the WM held off while it was on screen resumes.
    if action == OverviewAction::Hidden {
      return if state.overview.mark_hidden(session) {
        platform_sync(state, config)
      } else {
        Ok(())
      };
    }

    if state.overview.open_session() != Some(session) {
      return Ok(());
    }
    tracing::debug!("Carrying out overview action {action:?}.");

    let focused_workspace = state
      .focused_container()
      .and_then(|focused| focused.workspace())
      .map(|workspace| workspace.config().name);
    let window = window_by_handle(state, &action);

    // A window placed in a workspace's layout, once it is on it.
    let mut placement = None;

    let (commands, subject) = match action {
      OverviewAction::FocusWindow { monitor, .. } => {
        state.overview.close();
        let Some(window) = window else {
          state.pending_sync.queue_focus_change();
          return platform_sync(state, config);
        };

        let window = restore_if_minimized(window, state, config)?;

        let workspace = window.workspace().map(|w| w.config().name);
        let mut commands = Vec::new();

        // A window-level focus doesn't switch the displayed workspace
        // on its own.
        if let Some(name) =
          workspace.filter(|name| Some(name) != focused_workspace.as_ref())
        {
          prepare_target(state, config, &name, monitor, true)?;
          commands.push(focus_workspace_command(name));
        }

        commands.push(InvokeCommand::Focus(InvokeFocusCommand {
          container_id: Some(window.id()),
          ..Default::default()
        }));

        // The overview was the transition; the switch behind it is
        // instant.
        state.pending_sync.suppress_animations();
        (commands, None)
      }
      OverviewAction::FocusWorkspace { name, monitor } => {
        state.overview.close();

        // Focusing the focused workspace would toggle back to the
        // previous one when `toggle_workspace_on_refocus` is set.
        if Some(&name) == focused_workspace.as_ref() {
          state.pending_sync.queue_focus_change();
          return platform_sync(state, config);
        }

        prepare_target(state, config, &name, monitor, true)?;
        state.pending_sync.suppress_animations();
        (vec![focus_workspace_command(name)], None)
      }
      OverviewAction::MoveWindow {
        workspace, monitor, ..
      } => {
        let Some(window) = window else {
          return Ok(());
        };

        prepare_target(state, config, &workspace, monitor, false)?;
        (
          vec![move_to_workspace_command(workspace)],
          Some(window.id()),
        )
      }
      OverviewAction::PlaceWindow {
        workspace,
        monitor,
        point,
        ..
      } => {
        let Some(window) = window else {
          return Ok(());
        };

        let commands =
          place_commands(&window, &workspace, monitor, state, config)?;
        placement = Some((window.id(), workspace, point));
        (commands, Some(window.id()))
      }
      OverviewAction::CloseWindow(_) => {
        let Some(window) = window else {
          return Ok(());
        };
        (vec![InvokeCommand::Close], Some(window.id()))
      }
      OverviewAction::Cancel => {
        state.overview.close();
        state.pending_sync.queue_focus_change();
        return platform_sync(state, config);
      }
      // Focus went elsewhere and the overview closed itself.
      OverviewAction::Deactivated => {
        state.overview.close();
        return platform_sync(state, config);
      }
      OverviewAction::Hidden => return Ok(()),
    };

    self.process_commands(&commands, subject, config)?;

    match placement {
      Some((window_id, workspace, point)) => place_window(
        window_id,
        &workspace,
        &point,
        &mut self.state,
        config,
      ),
      None => Ok(()),
    }
  }
}

/// Readies workspace `name` for a pick made on the screen of monitor
/// `monitor_id`: activates it on that monitor if it is inactive (unless
/// it is bound to a monitor), and with `summon`, brings the special
/// workspace there.
fn prepare_target(
  state: &mut WmState,
  config: &UserConfig,
  name: &str,
  monitor_id: u128,
  summon: bool,
) -> anyhow::Result<()> {
  let monitor = state
    .monitors()
    .into_iter()
    .find(|monitor| monitor.id().as_u128() == monitor_id)
    .context("No monitor for the overview's pick.")?;

  if config.is_special_workspace_name(name) {
    if summon || state.workspace_by_name(name).is_none() {
      summon_special_workspace(&monitor, state, config)?;
    }
    return Ok(());
  }

  if state.workspace_by_name(name).is_some() {
    return Ok(());
  }

  let is_bound = config.value.workspaces.iter().any(|workspace| {
    workspace.name == name && workspace.bind_to_monitor.is_some()
  });
  activate_workspace(
    Some(name),
    (!is_bound).then_some(monitor),
    state,
    config,
  )
}

/// The managed window an overview action refers to, if any.
fn window_by_handle(
  state: &WmState,
  action: &OverviewAction,
) -> Option<crate::models::WindowContainer> {
  let hwnd = match action {
    OverviewAction::FocusWindow { hwnd, .. }
    | OverviewAction::CloseWindow(hwnd)
    | OverviewAction::MoveWindow { hwnd, .. }
    | OverviewAction::PlaceWindow { hwnd, .. } => *hwnd,
    _ => return None,
  };

  state
    .windows()
    .into_iter()
    .find(|window| window.native().id().0 == hwnd)
}

/// `focus --workspace <name>`, for a workspace readied by
/// `prepare_target`; the special workspace too, which it has summoned.
/// Brings a picked minimized window back, as `toggle-minimized` does:
/// focusing it alone leaves it minimized. Others are returned as they are.
fn restore_if_minimized(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  if window.state() != WindowState::Minimized {
    return Ok(window);
  }

  let restored = window.toggled_state(WindowState::Minimized, config);
  update_window_state(window, restored, state, config)
}

fn focus_workspace_command(name: String) -> InvokeCommand {
  InvokeCommand::Focus(InvokeFocusCommand {
    workspace: Some(name),
    ..Default::default()
  })
}

/// Commands bringing `window`, being placed on the named `workspace` from
/// the overview, onto it first: none if it is there already.
fn place_commands(
  window: &crate::models::WindowContainer,
  workspace: &str,
  monitor: u128,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Vec<InvokeCommand>> {
  prepare_target(state, config, workspace, monitor, false)?;

  let is_elsewhere = window
    .workspace()
    .is_none_or(|current| current.config().name != workspace);
  Ok(if is_elsewhere {
    vec![move_to_workspace_command(workspace.to_string())]
  } else {
    Vec::new()
  })
}

/// Places the window `window_id`, on the named `workspace` by now, where
/// dragging it to `point` there puts it, if it is tiling.
fn place_window(
  window_id: uuid::Uuid,
  workspace: &str,
  point: &wm_platform::Point,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let window = state
    .container_by_id(window_id)
    .and_then(|container| container.as_tiling_window().cloned());
  let workspace = state.workspace_by_name(workspace);

  if let (Some(window), Some(workspace)) = (window, workspace) {
    crate::events::place_tiling_window(
      &window, &workspace, point, state, config,
    )?;
    platform_sync(state, config)?;
  }
  Ok(())
}

/// `move --workspace <name>`, for a workspace readied by
/// `prepare_target`.
fn move_to_workspace_command(name: String) -> InvokeCommand {
  InvokeCommand::Move(InvokeMoveCommand {
    workspace: Some(name),
    ..Default::default()
  })
}

#[cfg(test)]
mod tests {
  use crate::{
    models::Workspace,
    test_utils::{mock_config_with, mock_right_monitor},
    traits::CommonGetters,
  };

  /// Two monitors showing "1" (focused) and "2", without the special
  /// workspace.
  fn two_monitors() -> (
    crate::wm_state::WmState,
    crate::models::Monitor,
    crate::models::Monitor,
  ) {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock().workspaces(vec![first.clone()]).call();
    let right = mock_right_monitor(vec![Workspace::mock()
      .name("2".to_string())
      .call()]);
    let state = WmState::mock(vec![left.clone(), right.clone()]);
    set_focused_descendant(&first.into(), None);

    (state, left, right)
  }

  /// ID of the monitor workspace `name` is on.
  fn monitor_of(
    state: &crate::wm_state::WmState,
    name: &str,
  ) -> Option<u128> {
    state
      .workspace_by_name(name)?
      .monitor()
      .map(|monitor| monitor.id().as_u128())
  }

  #[test]
  fn a_picked_minimized_window_is_restored_and_others_are_left_alone() {
    use wm_common::WindowState;

    use super::restore_if_minimized;
    use crate::{
      models::{Monitor, NonTilingWindow},
      traits::WindowGetters,
    };

    let config = mock_config_with(|_| {});
    let minimized =
      NonTilingWindow::mock().state(WindowState::Minimized).call();
    let floating = NonTilingWindow::mock().call();
    let workspace = Workspace::mock()
      .name("1".to_string())
      .non_tiling_windows(vec![minimized.clone(), floating.clone()])
      .call();
    let mut state = crate::wm_state::WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);

    let restored =
      restore_if_minimized(minimized.clone().into(), &mut state, &config)
        .unwrap();
    assert_ne!(restored.state(), WindowState::Minimized);
    assert_eq!(restored.id(), minimized.id());

    let untouched =
      restore_if_minimized(floating.clone().into(), &mut state, &config)
        .unwrap();
    assert_eq!(untouched.state(), floating.state());
  }

  #[test]
  fn a_pick_activates_an_inactive_workspace_on_its_screen() {
    use super::prepare_target;

    let config = mock_config_with(|_| {});
    let (mut state, _, right) = two_monitors();
    let right_id = right.id().as_u128();

    prepare_target(&mut state, &config, "3", right_id, true).unwrap();
    assert_eq!(monitor_of(&state, "3"), Some(right_id));
  }

  #[test]
  fn a_pick_leaves_a_bound_workspace_on_its_monitor() {
    use super::prepare_target;

    let config = mock_config_with(|value| {
      value.workspaces[2].bind_to_monitor = Some(0);
    });
    let (mut state, left, right) = two_monitors();

    prepare_target(&mut state, &config, "3", right.id().as_u128(), false)
      .unwrap();
    assert_eq!(monitor_of(&state, "3"), Some(left.id().as_u128()));
  }

  #[test]
  fn a_pick_summons_the_special_workspace_to_its_screen() {
    use super::prepare_target;

    let config = mock_config_with(|_| {});
    let (mut state, left, right) = two_monitors();
    let (left_id, right_id) = (left.id().as_u128(), right.id().as_u128());

    // Moving a window into it activates it where it was picked.
    prepare_target(&mut state, &config, "special", right_id, false)
      .unwrap();
    assert_eq!(monitor_of(&state, "special"), Some(right_id));

    prepare_target(&mut state, &config, "special", left_id, false)
      .unwrap();
    assert_eq!(monitor_of(&state, "special"), Some(right_id), "stays");

    prepare_target(&mut state, &config, "special", left_id, true).unwrap();
    assert_eq!(monitor_of(&state, "special"), Some(left_id), "summoned");
  }
}
