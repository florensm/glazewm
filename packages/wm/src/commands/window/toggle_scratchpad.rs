use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;
use wm_platform::Rect;

use crate::{
  commands::container::{attach_container, detach_container, set_focused_descendant},
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Shows or hides all scratchpad windows on the focused monitor.
///
/// If any scratchpad window is currently visible as an overlay, all are
/// hidden by moving them back to the detached scratchpad workspace, which
/// causes `platform_sync` to cloak them. Otherwise, all stashed windows
/// are shown by moving them to the focused workspace, positioning each on
/// the focused monitor, and queuing them for redraw so `platform_sync`
/// unclocks them.
pub fn toggle_scratchpad(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let shown = state.scratchpad_shown_windows();

  if !shown.is_empty() {
    hide_scratchpad_windows(shown, state)
  } else {
    let stashed = state.scratchpad_windows();
    if stashed.is_empty() {
      return Ok(());
    }
    show_scratchpad_windows(stashed, state, config)
  }
}

/// Moves scratchpad windows to the focused workspace and unclooks them.
fn show_scratchpad_windows(
  windows: Vec<WindowContainer>,
  state: &mut WmState,
  _config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|c| c.workspace())
    .context("No focused workspace.")?;

  let monitor_rect = focused_workspace
    .monitor()
    .context("No monitor for focused workspace.")?
    .to_rect()?;

  info!("Showing {} scratchpad window(s).", windows.len());

  let mut window_dtos = Vec::new();

  for window in &windows {
    let non_tiling = window
      .as_non_tiling_window()
      .context("Scratchpad window is not non-tiling.")?
      .clone();

    // Center on the focused monitor when no user-defined placement exists.
    if !non_tiling.has_custom_floating_placement() {
      let w = monitor_rect.width() * 4 / 5;
      let h = monitor_rect.height() * 3 / 5;
      let x = monitor_rect.x() + (monitor_rect.width() - w) / 2;
      let y = monitor_rect.y() + (monitor_rect.height() - h) / 2;
      non_tiling.set_floating_placement(Rect::from_xy(x, y, w, h));
    }

    // Move from the detached scratchpad workspace to the focused workspace.
    detach_container(non_tiling.clone().into())?;
    attach_container(
      &non_tiling.clone().into(),
      &focused_workspace.clone().into(),
      Some(focused_workspace.child_count()),
    )?;

    state
      .pending_sync
      .queue_container_to_redraw(non_tiling.clone())
      .queue_workspace_to_reorder(focused_workspace.clone());

    window_dtos.push(non_tiling.to_dto()?);
  }

  // Focus the first shown scratchpad window.
  if let Some(first) = windows.first() {
    set_focused_descendant(&first.clone().into(), None);
    state.pending_sync.queue_focus_change();
  }

  state.emit_event(WmEvent::ScratchpadToggled {
    shown: true,
    windows: window_dtos,
  });

  Ok(())
}

/// Moves visible scratchpad windows back to the detached scratchpad workspace.
fn hide_scratchpad_windows(
  windows: Vec<WindowContainer>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  info!("Hiding {} scratchpad window(s).", windows.len());

  // Only reassign focus if the currently focused container is one of the
  // scratchpad windows being hidden. If the user is already focused on a
  // different workspace, we must not steal focus from it.
  let focused = state.focused_container();
  let is_scratchpad_focused = focused
    .as_ref()
    .is_some_and(|f| windows.iter().any(|w| w.id() == f.id()));

  // `focus_target_after_removal` only returns `Some` when the passed window
  // IS the currently focused container, so we must pass the actual focused
  // window — not `windows.first()`, which may differ.
  let focus_target = if is_scratchpad_focused {
    focused
      .as_ref()
      .and_then(|f| f.as_window_container().ok())
      .and_then(|w| state.focus_target_after_removal(&w))
  } else {
    None
  };

  let scratchpad_ws = state.scratchpad_workspace.clone().into();
  let mut window_dtos = Vec::new();

  for window in &windows {
    let current_workspace =
      window.workspace().context("Scratchpad overlay has no workspace.")?;

    detach_container(window.clone().into())?;
    attach_container(&window.clone().into(), &scratchpad_ws, None)?;

    state
      .pending_sync
      .queue_container_to_redraw(window.clone())
      .queue_workspace_to_reorder(current_workspace);

    window_dtos.push(window.to_dto()?);
  }

  // Restore focus to the workspace.
  if let Some(target) = focus_target {
    set_focused_descendant(&target, None);
    state.pending_sync.queue_focus_change();
  }

  state.emit_event(WmEvent::ScratchpadToggled {
    shown: false,
    windows: window_dtos,
  });

  Ok(())
}
