use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wm_platform::Direction;

use crate::{
  dtos::{ContainerDto, WindowDto},
  parsed_config::{BindingModeConfig, ParsedConfig},
  Layout, TilingDirection,
};

// Short-lived and serialized for subscribers, so boxing the large
// variants would only add allocations.
#[allow(clippy::large_enum_variant)]
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
  LayoutChanged {
    workspace: ContainerDto,
    new_layout: Layout,
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
  /// A keybinding pressed while a program holds the overlay (see
  /// `overlay begin`), for that program to act on instead of the WM.
  OverlayInput {
    input: OverlayInput,
  },
}

/// What a keybinding means to a program holding the overlay, e.g. an
/// overview: the WM doesn't run it on the desktop behind.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OverlayInput {
  /// A `focus --direction` or `move --direction`: move the selection.
  Move {
    direction: Direction,
  },
  /// Go to the named workspace.
  FocusWorkspace {
    name: String,
  },
  /// Send the selected window to the named workspace.
  MoveToWorkspace {
    name: String,
  },
  /// Show the next or previous workspace.
  NextWorkspace,
  PreviousWorkspace,
  /// Close the selected window.
  CloseWindow,
}

impl WmEvent {
  /// Runs `f` on every window this event holds.
  pub fn for_each_window_mut(
    &mut self,
    f: &mut impl FnMut(&mut WindowDto),
  ) {
    let container = match self {
      WmEvent::FocusChanged { focused_container }
      | WmEvent::FocusedContainerMoved { focused_container } => {
        focused_container
      }
      WmEvent::MonitorAdded { added_monitor } => added_monitor,
      WmEvent::MonitorUpdated { updated_monitor } => updated_monitor,
      WmEvent::TilingDirectionChanged {
        direction_container,
        ..
      } => direction_container,
      WmEvent::LayoutChanged { workspace, .. } => workspace,
      WmEvent::WindowManaged { managed_window } => managed_window,
      WmEvent::WindowUrgencyChanged { updated_window, .. } => {
        updated_window
      }
      WmEvent::WorkspaceActivated {
        activated_workspace,
      } => activated_workspace,
      WmEvent::WorkspaceUpdated { updated_workspace } => updated_workspace,
      _ => return,
    };

    container.for_each_window_mut(f);
  }
}
