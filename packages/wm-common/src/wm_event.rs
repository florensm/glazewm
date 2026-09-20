use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  dtos::ContainerDto,
  parsed_config::{BindingModeConfig, ParsedConfig},
  TilingDirection,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
  tag = "eventType",
  rename_all = "snake_case",
  rename_all_fields = "camelCase"
)]
pub enum WmEvent {
  ApplicationExiting,
  BindingModesChanged {
    new_binding_modes: Vec<BindingModeConfig>,
  },
  FocusChanged {
    focused_container: ContainerDto,
  },
  FocusedContainerMoved {
    focused_container: ContainerDto,
  },
  MonitorAdded {
    added_monitor: ContainerDto,
  },
  MonitorRemoved {
    removed_id: Uuid,
    removed_device_name: String,
  },
  MonitorUpdated {
    updated_monitor: ContainerDto,
  },
  TilingDirectionChanged {
    direction_container: ContainerDto,
    new_tiling_direction: TilingDirection,
  },
  UserConfigChanged {
    config_path: String,
    config_string: String,
    parsed_config: ParsedConfig,
  },
  WindowManaged {
    managed_window: ContainerDto,
  },
  WindowUnmanaged {
    unmanaged_id: Uuid,
    unmanaged_handle: isize,
  },
  WindowUrgencyChanged {
    updated_window: ContainerDto,
    /// Name of the workspace that the window is on.
    ///
    /// The window's `parent_id` is its immediate parent, which can be a
    /// split container, so the workspace is resolved here for the sake of
    /// consumers that group by workspace.
    workspace_name: Option<String>,
  },
  WorkspaceActivated {
    activated_workspace: ContainerDto,
  },
  WorkspaceDeactivated {
    deactivated_id: Uuid,
    deactivated_name: String,
  },
  WorkspaceUpdated {
    updated_workspace: ContainerDto,
  },
  PauseChanged {
    is_paused: bool,
  },
}
