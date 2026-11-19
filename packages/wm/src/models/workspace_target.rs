use wm_platform::Direction;

pub enum WorkspaceTarget {
  Name(String),
  Recent,
  NextActive,
  PreviousActive,
  NextActiveInMonitor,
  PreviousActiveInMonitor,
  Next,
  Previous,
  /// First workspace without any windows, creating a new one if dynamic
  /// workspaces are enabled and none is available.
  NextEmpty,
  Direction(Direction),
}

/// The workspace the flags of a `focus` or `move` command's `args` name,
/// if any.
///
/// A macro since the two commands' args are separate types that only share
/// these fields; clap lets at most one of the flags through.
macro_rules! workspace_target {
  ($args:expr) => {{
    use $crate::models::WorkspaceTarget;

    let args = $args;
    args
      .workspace_in_direction
      .clone()
      .map(WorkspaceTarget::Direction)
      .or_else(|| args.workspace.clone().map(WorkspaceTarget::Name))
      .or_else(|| {
        [
          (args.next_active_workspace, WorkspaceTarget::NextActive),
          (args.prev_active_workspace, WorkspaceTarget::PreviousActive),
          (args.next_workspace, WorkspaceTarget::Next),
          (args.prev_workspace, WorkspaceTarget::Previous),
          (
            args.next_active_workspace_on_monitor,
            WorkspaceTarget::NextActiveInMonitor,
          ),
          (
            args.prev_active_workspace_on_monitor,
            WorkspaceTarget::PreviousActiveInMonitor,
          ),
          (args.recent_workspace, WorkspaceTarget::Recent),
          (args.next_empty_workspace, WorkspaceTarget::NextEmpty),
        ]
        .into_iter()
        .find_map(|(is_set, target)| is_set.then_some(target))
      })
  }};
}

pub(crate) use workspace_target;
