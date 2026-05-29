use anyhow::Context;
use tracing::info;
use wm_common::{FloatingStateConfig, WindowState, WmEvent};

use crate::{
  commands::{
    container::{attach_container, detach_container, set_focused_descendant},
    window::update_window_state,
    workspace::activate_workspace,
  },
  models::{ScratchpadOrigin, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Sends the focused window to the scratchpad, or restores it if already shown.
///
/// **Send flow:** converts the window to a floating `shown_on_top` overlay,
/// records its origin workspace and previous state, then moves it to the
/// detached scratchpad workspace so `platform_sync` cloaks it automatically.
///
/// **Restore flow:** if the window is a visible scratchpad overlay (has a
/// `scratchpad_origin` and lives on a regular workspace), it is moved back to
/// its origin workspace and its previous state (`Tiling` or `Floating`) is
/// restored.
///
/// No-ops if the window is already hidden inside the scratchpad workspace.
pub fn send_to_scratchpad(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // If the window is currently shown as a scratchpad overlay, restore it
  // rather than sending it to the scratchpad again.
  let is_shown_overlay = window
    .as_non_tiling_window()
    .is_some_and(|nw| nw.scratchpad_origin().is_some())
    && !state
      .scratchpad_workspace
      .descendants()
      .any(|c| c.id() == window.id());

  if is_shown_overlay {
    return restore_from_scratchpad(window, state, config);
  }

  // No-op if the window is already hidden inside the scratchpad workspace.
  if state
    .scratchpad_workspace
    .descendants()
    .any(|c| c.id() == window.id())
  {
    return Ok(());
  }

  info!("Sending window to scratchpad: {window}.");

  let workspace = window.workspace().context("Window has no workspace.")?;
  let workspace_name = workspace.config().name.clone();
  let prev_state = window.state();

  // Convert to floating with always-on-top so it behaves as an overlay
  // when shown. The original state is preserved in `ScratchpadOrigin` for
  // accurate restoration.
  let window = update_window_state(
    window,
    WindowState::Floating(FloatingStateConfig {
      centered: false,
      shown_on_top: true,
    }),
    state,
    config,
  )?;

  let non_tiling = window
    .as_non_tiling_window()
    .context("Window is not non-tiling after floating conversion.")?
    .clone();

  // Mark with scratchpad origin for later restoration.
  non_tiling.set_scratchpad_origin(Some(ScratchpadOrigin {
    workspace_name,
    prev_state,
  }));

  // Determine focus target before detaching so the workspace focus order
  // is still intact.
  let focus_target = state.focus_target_after_removal(&non_tiling.clone().into());

  // Move to the detached scratchpad workspace using raw detach + attach
  // because there is no common ancestor with the scratchpad workspace.
  let scratchpad_ws = state.scratchpad_workspace.clone().into();
  detach_container(non_tiling.clone().into())?;
  attach_container(&non_tiling.clone().into(), &scratchpad_ws, None)?;

  // Cancel any in-flight animation (e.g. from the tiling→floating
  // conversion). The scratchpad workspace has no monitor, so every
  // animation tick would error with "No monitor." until the animation
  // completed.
  state.animation_manager.remove_animation(&non_tiling.id());

  // Queue for redraw: `platform_sync` will cloak the window because the
  // scratchpad workspace's `is_displayed()` is always false.
  state
    .pending_sync
    .queue_container_to_redraw(non_tiling.clone());

  // Restore focus within the workspace.
  if let Some(target) = focus_target {
    set_focused_descendant(&target, None);
    state.pending_sync.queue_focus_change();
  }

  state.emit_event(WmEvent::ScratchpadToggled {
    shown: false,
    windows: vec![non_tiling.to_dto()?],
  });

  Ok(())
}

/// Restores a shown scratchpad overlay to its origin workspace.
///
/// Clears the window's `scratchpad_origin`, destroys the dim overlay if no
/// other scratchpad windows remain visible, then moves the window to its
/// origin workspace and restores its previous state.
fn restore_from_scratchpad(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let non_tiling = window
    .as_non_tiling_window()
    .context("Shown scratchpad window is not non-tiling.")?
    .clone();

  let origin = non_tiling
    .scratchpad_origin()
    .context("Window has no scratchpad origin.")?;

  info!("Restoring window from scratchpad: {window}.");

  let current_workspace =
    window.workspace().context("Window has no workspace.")?;

  // Compute focus fallback before detaching.
  let focus_target = state.focus_target_after_removal(&window);

  // Clear the scratchpad marker before checking shown windows so the count
  // reflects the post-restore state.
  non_tiling.set_scratchpad_origin(None);

  // Destroy the dim overlay if this was the last shown scratchpad window.
  #[cfg(target_os = "windows")]
  if state.scratchpad_shown_windows().is_empty() {
    state.scratchpad_overlay = None;
  }

  // Find or activate the origin workspace.
  let origin_workspace = match state.workspace_by_name(&origin.workspace_name) {
    Some(ws) => ws,
    None => {
      activate_workspace(
        Some(&origin.workspace_name),
        None,
        state,
        config,
      )?;
      state
        .workspace_by_name(&origin.workspace_name)
        .context("Failed to activate origin workspace.")?
    }
  };

  let current_monitor =
    current_workspace.monitor().context("No monitor.")?;
  let origin_monitor =
    origin_workspace.monitor().context("No monitor.")?;

  // Adjust floating placement and DPI when crossing monitors.
  if origin_monitor.id() != current_monitor.id() {
    if current_monitor
      .has_dpi_difference(&origin_monitor.clone().into())?
    {
      non_tiling.set_has_pending_dpi_adjustment(true);
    }

    non_tiling.set_floating_placement(
      non_tiling
        .floating_placement()
        .translate_to_center(&origin_workspace.to_rect()?),
    );
  }

  // Capture the DTO before moving so `parent_id` still reflects the current
  // workspace.
  let window_dto = non_tiling.to_dto()?;

  // Move to the origin workspace (still floating).
  detach_container(non_tiling.clone().into())?;
  attach_container(
    &non_tiling.clone().into(),
    &origin_workspace.clone().into(),
    Some(origin_workspace.child_count()),
  )?;

  // Cancel any in-flight animation before restoring state.
  state.animation_manager.remove_animation(&non_tiling.id());

  // Restore the window's previous state (e.g. tiling in origin workspace).
  // For `Tiling`, `update_window_state` uses the stored `insertion_target`
  // to place the window back in its original tiling position.
  let restored = update_window_state(
    non_tiling.clone().into(),
    origin.prev_state,
    state,
    config,
  )?;

  // The window was removed from the shown workspace; reorder it.
  let is_same_workspace = origin_workspace.id() == current_workspace.id();
  state
    .pending_sync
    .queue_workspace_to_reorder(current_workspace);

  // If restoring to the same workspace the overlay was shown on, focus the
  // restored window. Otherwise restore focus to whatever was behind the
  // overlay (the user stays on the current workspace).
  if is_same_workspace {
    set_focused_descendant(&restored.clone().into(), None);
    state.pending_sync.queue_focus_change();
  } else if let Some(target) = focus_target {
    set_focused_descendant(&target, None);
    state.pending_sync.queue_focus_change();
  }

  state.emit_event(WmEvent::ScratchpadToggled {
    shown: false,
    windows: vec![window_dto],
  });

  Ok(())
}
