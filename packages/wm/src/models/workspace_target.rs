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
  #[allow(dead_code)]
  Direction(Direction),
}
