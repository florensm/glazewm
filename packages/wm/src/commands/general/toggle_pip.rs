use anyhow::Context;
use tracing::info;
use uuid::Uuid;
use wm_common::{PipTarget, WindowState};
use wm_platform::NativeWindowWindowsExt;

use crate::{
  models::Container,
  pip_state::PipState,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// Toggles picture-in-picture for `target`, rooted at `subject_container`.
///
/// `Window` minimizes just the subject window; `Workspace` minimizes every
/// window in the subject's workspace. Each minimized window is picked up by
/// `sync_pip_tile` (in `platform_sync`) once its OS minimize completes,
/// which gives it its own live, clickable dock tile.
///
/// If the resolved window set is already the active PIP group (a second
/// press of the same keybinding), every member is restored instead. If a
/// *different* PIP group is currently active, it is restored first — only
/// one PIP group can be active at a time.
pub fn toggle_pip(
  target: PipTarget,
  subject_container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let member_ids = resolve_members(target, subject_container)?;

  if let Some(pip) = &state.pip {
    let is_same_group = pip.members.len() == member_ids.len()
      && member_ids.iter().all(|id| pip.contains(*id));

    if is_same_group {
      restore_members(&member_ids, state)?;
      state.pip = None;
      return Ok(());
    }

    let prev_ids: Vec<Uuid> =
      pip.members.iter().map(|member| member.window_id).collect();
    restore_members(&prev_ids, state)?;
  }

  for window_id in &member_ids {
    let window = state
      .container_by_id(*window_id)
      .and_then(|container| container.as_window_container().ok())
      .context("PIP member window no longer exists.")?;

    if window.state() != WindowState::Minimized {
      window.native().minimize()?;
    }
  }

  info!("Starting {:?} PIP for {} window(s).", target, member_ids.len());
  state.pip = Some(PipState::new(target, member_ids));

  Ok(())
}

/// Restores every member of the active PIP group that contains
/// `window_id`, and clears `state.pip`.
///
/// Called when a PIP tile is clicked. No-op if `window_id` isn't part of
/// the active PIP group (e.g. a stale click that arrived after the group
/// was already restored some other way).
pub fn restore_pip_group(
  window_id: Uuid,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(pip) = &state.pip else {
    return Ok(());
  };

  if !pip.contains(window_id) {
    return Ok(());
  }

  let member_ids: Vec<Uuid> =
    pip.members.iter().map(|member| member.window_id).collect();

  restore_members(&member_ids, state)?;
  state.pip = None;

  Ok(())
}

/// Resolves the window IDs that `target` applies to, rooted at
/// `subject_container`.
fn resolve_members(
  target: PipTarget,
  subject_container: &Container,
) -> anyhow::Result<Vec<Uuid>> {
  match target {
    PipTarget::Window => {
      let window = subject_container
        .as_window_container()
        .context("Subject is not a window.")?;
      Ok(vec![window.id()])
    }
    PipTarget::Workspace => {
      let workspace = subject_container
        .workspace()
        .context("Subject has no workspace.")?;

      Ok(
        workspace
          .descendants()
          .filter_map(|container| container.as_window_container().ok())
          .map(|window| window.id())
          .collect(),
      )
    }
  }
}

/// Restores each window in `window_ids` that's currently minimized.
fn restore_members(
  window_ids: &[Uuid],
  state: &WmState,
) -> anyhow::Result<()> {
  for window_id in window_ids {
    if let Some(window) = state
      .container_by_id(*window_id)
      .and_then(|container| container.as_window_container().ok())
    {
      if window.state() == WindowState::Minimized {
        window.native().restore(None)?;
      }
    }
  }

  Ok(())
}
