use anyhow::Context;
use tracing::info;
use wm_common::{WindowState, WorkspaceSwitchDirection};

use super::activate_workspace;
use crate::{
  commands::{
    container::set_focused_descendant,
    workspace::deactivate_unused_workspace,
  },
  layout::is_hidden_in_layout,
  models::{WindowContainer, Workspace, WorkspaceTarget},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Focuses a workspace by a given target.
///
/// This target can be a workspace name, the most recently focused
/// workspace, the next workspace, the previous workspace, or the workspace
/// in a given direction from the currently focused workspace.
///
/// The workspace will be activated if it isn't already active.
#[allow(clippy::too_many_lines)]
pub fn focus_workspace(
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let (target_workspace_name, target_workspace) =
    state.workspace_by_target(&focused_workspace, target, config)?;

  // Retrieve or activate the target workspace by its name.
  let target_workspace = match target_workspace {
    Some(_) => anyhow::Ok(target_workspace),
    _ => match target_workspace_name {
      Some(name) => {
        activate_workspace(Some(&name), None, state, config)?;

        Ok(state.workspace_by_name(&name))
      }
      _ => Ok(None),
    },
  }?;

  if let Some(target_workspace) = target_workspace {
    info!("Focusing workspace: {target_workspace}");

    // Get the currently displayed workspace on the same monitor that the
    // workspace to focus is on.
    let displayed_workspace = target_workspace
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .context("No workspace is currently displayed.")?;

    // Set focus to whichever window last had focus in workspace. If the
    // workspace has no windows, then set focus to the workspace itself.
    let container_to_focus = target_workspace
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| target_workspace.clone().into());

    set_focused_descendant(&container_to_focus, None);
    state.pending_sync.queue_focus_change();

    // Only set up the workspace-switch slide animation when the target
    // workspace differs from what is currently displayed on its monitor.
    // For cross-monitor focus switches (e.g. focusing workspace 8 which is
    // already displayed on monitor 2), `target == displayed`, so no
    // workspace layout change occurs on the target monitor and no slide
    // should play.
    if target_workspace.id() != displayed_workspace.id() {
      // Derive slide direction before queuing redraws so `platform_sync`
      // can build per-window surrogates with the correct direction. It
      // is relative to the workspace sliding out on the same monitor.
      let direction = workspace_switch_direction(
        &target_workspace.config().name,
        &displayed_workspace.config().name,
        config,
      );
      state.pending_sync.set_workspace_switch_direction(
        direction,
        workspace_switch_axis(
          &target_workspace,
          &displayed_workspace,
          config,
        ),
      );

      // Mark windows on the incoming workspace to slide in.
      let mut incoming = 0;
      for window in target_workspace
        .descendants()
        .filter_map(|c| c.as_window_container().ok())
        .filter(is_switch_animated)
      {
        incoming += 1;
        state
          .pending_sync
          .setup_workspace_switch_incoming(window.id());
      }

      // Cancel in-flight animations for outgoing windows and mark them
      // for the outgoing surrogate slide-out.
      let mut outgoing = 0;
      for window in displayed_workspace
        .descendants()
        .filter_map(|c| c.as_window_container().ok())
        .filter(is_switch_animated)
      {
        outgoing += 1;
        state.animation_manager.remove_animation(&window.id());
        state
          .pending_sync
          .setup_workspace_switch_outgoing(window.id());
      }

      let switch = &config.value.animations.workspace_switch;
      let style = if switch.enabled {
        format!("{:?}", switch.style).to_lowercase()
      } else {
        "off".to_string()
      };
      state.switch_log.begin(style, incoming, outgoing);
    }

    // Display the workspace to switch focus to.
    state
      .pending_sync
      .queue_container_to_redraw(displayed_workspace)
      .queue_container_to_redraw(target_workspace);

    deactivate_unused_workspace(state.workspaces(), state)?;

    // Save the focused workspace as recent, unless it's the special one.
    if !focused_workspace.is_special() {
      state.recent_workspace_name = Some(focused_workspace.config().name);
    }

    if config.value.general.cursor_jump.on_workspace_switch {
      state.pending_sync.queue_cursor_jump();
    }
  }

  Ok(())
}

/// Returns `+1` if `target_name` comes after `displayed_name` in the
/// workspaces' order on a monitor, `-1` if before, `0` for the same one.
///
/// Used to derive the slide direction for the workspace-switch animation,
/// so dynamic and special workspaces slide in from where they sit too.
fn workspace_switch_direction(
  target_name: &str,
  displayed_name: &str,
  config: &UserConfig,
) -> i32 {
  config.compare_workspaces(target_name, displayed_name) as i32
}

/// Axis the workspace-switch slide runs along: the configured one, or
/// vertical to and from the special workspace, which sorts last and so
/// rises from below the others.
fn workspace_switch_axis(
  target: &Workspace,
  displayed: &Workspace,
  config: &UserConfig,
) -> WorkspaceSwitchDirection {
  if target.is_special() || displayed.is_special() {
    WorkspaceSwitchDirection::Vertical
  } else {
    config.value.animations.workspace_switch.direction.clone()
  }
}

/// Whether `window` takes part in the workspace-switch animation.
///
/// Minimized windows and windows the layout hides (inactive stack tabs,
/// columns scrolled off screen) have no visible content to animate, and
/// snapshotting them causes flicker.
fn is_switch_animated(window: &WindowContainer) -> bool {
  window.state() != WindowState::Minimized && !is_hidden_in_layout(window)
}

#[cfg(test)]
mod tests {
  use wm_common::{ParsedConfig, WorkspaceSwitchDirection};

  use super::{workspace_switch_axis, workspace_switch_direction};
  use crate::{
    models::Workspace,
    test_utils::{mock_config, mock_workspace_configs},
    user_config::UserConfig,
  };

  #[test]
  fn dynamic_and_special_workspaces_slide_from_where_they_sit() {
    let mut value = ParsedConfig::default();
    value.general.dynamic_workspaces = true;
    value.workspaces = mock_workspace_configs(&["1", "2", "3"]);
    let config = UserConfig::from_parsed(value);
    let special = config.value.special_workspace.name.clone();

    assert_eq!(workspace_switch_direction("2", "1", &config), 1);
    assert_eq!(workspace_switch_direction("1", "3", &config), -1);

    // A dynamic workspace comes after the configured ones.
    assert_eq!(workspace_switch_direction("10", "3", &config), 1);
    assert_eq!(workspace_switch_direction("2", "10", &config), -1);

    // The special workspace comes last.
    assert_eq!(workspace_switch_direction(&special, "10", &config), 1);
    assert_eq!(workspace_switch_direction("1", &special, &config), -1);
  }

  #[test]
  fn the_special_workspace_slides_vertically() {
    let config = mock_config();
    let regular = Workspace::mock().call();
    let special = Workspace::mock().is_special(true).call();

    assert_eq!(
      workspace_switch_axis(&special, &regular, &config),
      WorkspaceSwitchDirection::Vertical
    );
    assert_eq!(
      workspace_switch_axis(&regular, &special, &config),
      WorkspaceSwitchDirection::Vertical
    );
    assert_eq!(
      workspace_switch_axis(&regular, &regular, &config),
      config.value.animations.workspace_switch.direction
    );
  }
}
