use serde::{Deserialize, Serialize};

use super::{
  MonitorDto, RootContainerDto, SplitContainerDto, StackContainerDto,
  WindowDto, WorkspaceDto,
};

/// User-friendly representation of a container.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContainerDto {
  Root(RootContainerDto),
  Monitor(MonitorDto),
  Workspace(WorkspaceDto),
  Split(SplitContainerDto),
  Stack(StackContainerDto),
  Window(WindowDto),
}

impl ContainerDto {
  /// Runs `f` on every window in this container, itself included.
  pub fn for_each_window_mut(
    &mut self,
    f: &mut impl FnMut(&mut WindowDto),
  ) {
    let children = match self {
      ContainerDto::Window(window) => return f(window),
      ContainerDto::Root(root) => &mut root.children,
      ContainerDto::Monitor(monitor) => &mut monitor.children,
      ContainerDto::Workspace(workspace) => &mut workspace.children,
      ContainerDto::Split(split) => &mut split.children,
      ContainerDto::Stack(stack) => &mut stack.children,
    };

    for child in children {
      child.for_each_window_mut(f);
    }
  }
}
