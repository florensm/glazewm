use tracing::info;
use wm_common::WmEvent;
use wm_platform::Display;

use crate::{
  commands::{
    container::attach_container,
    workspace::{activate_workspace, move_workspace_to_monitor},
  },
  models::{Monitor, NativeMonitorProperties},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn add_monitor(
  native_display: Display,
  native_properties: NativeMonitorProperties,
  state: &mut WmState,
) -> anyhow::Result<Monitor> {
  // Create `Monitor` instance. This uses the working area of the monitor
  // instead of the bounds of the display. The working area excludes
  // taskbars and other reserved display space.
  let monitor = Monitor::new(native_display, native_properties);

  attach_container(
    &monitor.clone().into(),
    &state.root_container.clone().into(),
    None,
  )?;

  info!("Monitor added: {monitor}");

  state.emit_event(WmEvent::MonitorAdded {
    added_monitor: monitor.to_dto()?,
  });

  Ok(monitor)
}

pub fn move_bounded_workspaces_to_new_monitor(
  monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let bound_workspace_configs = config
    .value
    .workspaces
    .iter()
    .filter(|config| {
      config.bind_to_monitor.is_some_and(|monitor_index| {
        monitor.index() == monitor_index as usize
      })
    })
    .collect::<Vec<_>>();

  for workspace_config in bound_workspace_configs {
    let existing_workspace =
      state.workspace_by_name(&workspace_config.name);

    if let Some(existing_workspace) = existing_workspace {
      // Move workspaces that should be bound to the newly added monitor.
      move_workspace_to_monitor(
        &existing_workspace,
        monitor,
        state,
        config,
      )?;
    } else if workspace_config.keep_alive {
      // Activate all `keep_alive` workspaces for this monitor.
      activate_workspace(
        Some(&workspace_config.name),
        Some(monitor.clone()),
        state,
        config,
      )?;
    }
  }

  // Make sure the monitor has at least one workspace. This will
  // automatically prioritize bound workspace configs and fall back to the
  // first available one if needed.
  if monitor.child_count() == 0 {
    activate_workspace(None, Some(monitor.clone()), state, config)?;
  }

  Ok(())
}
