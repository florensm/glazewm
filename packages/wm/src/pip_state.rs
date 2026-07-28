use uuid::Uuid;
use wm_common::PipTarget;
#[cfg(target_os = "windows")]
use wm_platform::NativePipTile;

/// One window minimized as part of the active PIP group.
pub struct PipMember {
  pub window_id: Uuid,
  /// The live dock tile, created lazily by `sync_pip_tile` once the
  /// window's OS minimize has actually completed. `None` while the
  /// minimize is still in flight, and again briefly while a restore is in
  /// flight before the member is dropped from `PipState::members`.
  #[cfg(target_os = "windows")]
  pub tile: Option<NativePipTile>,
}

impl PipMember {
  /// Creates a member with no tile yet.
  #[must_use]
  pub fn new(window_id: Uuid) -> Self {
    Self {
      window_id,
      #[cfg(target_os = "windows")]
      tile: None,
    }
  }
}

/// The currently active picture-in-picture group, if any.
///
/// Populated by `toggle_pip` when a window or workspace is minimized into
/// PIP, and torn down by `sync_pip_tile` once every member has been
/// restored (its `WindowState` left `Minimized` again).
pub struct PipState {
  pub target: PipTarget,
  pub members: Vec<PipMember>,
}

impl PipState {
  /// Creates a PIP group for `window_ids`, with no tiles created yet.
  #[must_use]
  pub fn new(target: PipTarget, window_ids: Vec<Uuid>) -> Self {
    Self {
      target,
      members: window_ids.into_iter().map(PipMember::new).collect(),
    }
  }

  /// Whether `window_id` is a member of this PIP group.
  #[must_use]
  pub fn contains(&self, window_id: Uuid) -> bool {
    self.members.iter().any(|member| member.window_id == window_id)
  }
}
