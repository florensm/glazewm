use anyhow::Context;
use tracing::info;
use wm_common::{FloatingStateConfig, WindowState, WmEvent};

use crate::{
  commands::{
    container::{attach_container, detach_container, set_focused_descendant},
    window::update_window_state,
    workspace::activate_workspace,
  },
  models::{NonTilingWindow, ScratchpadOrigin, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Sends the focused window to the scratchpad, or restores it if it is
/// already there.
///
/// **Send flow:** converts the window to a floating `shown_on_top` overlay,
/// records its origin workspace and previous state, then moves it to the
/// detached scratchpad workspace so `platform_sync` cloaks it automatically.
///
/// **Restore flow:** if the window is a visible scratchpad overlay (i.e. it
/// has a `scratchpad_origin`), it is moved back to its origin workspace and
/// its previous state (`Tiling` or `Floating`) is restored.
pub fn send_to_scratchpad(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // If the window is currently shown as a scratchpad overlay, restore it.
  if let Some(non_tiling) = window.as_non_tiling_window() {
    if non_tiling.scratchpad_origin().is_some() {
      return restore_from_scratchpad(non_tiling.clone(), state, config);
    }
  }

  // Also check the scratchpad workspace (hidden stash).
  let is_stashed = state
    .scratchpad_workspace
    .descendants()
    .any(|c| c.id() == window.id());

  if is_stashed {
    if let Some(non_tiling) = window.as_non_tiling_window() {
      if non_tiling.scratchpad_origin().is_some() {
        return restore_from_scratchpad(non_tiling.clone(), state, config);
      }
    }
  }

  info!("Sending window to scratchpad: {window}.");

  let workspace = window.workspace().context("Window has no workspace.")?;
  let workspace_name = workspace.config().name.clone();
  let prev_state = window.state();

  // Convert to floating with always-on-top so it behaves as an overlay
  // when shown. The original state is preserved in `ScratchpadOrigin` for
  // accurate restoration.
  let floating_config = FloatingStateConfig {
    centered: false,
    shown_on_top: true,
  };

  let window = update_window_state(
    window,
    WindowState::Floating(floating_config),
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

/// Restores a scratchpad window back to its origin workspace and state.
fn restore_from_scratchpad(
  non_tiling: NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let origin = non_tiling
    .scratchpad_origin()
    .context("Window is not a scratchpad window.")?;

  info!("Restoring window from scratchpad.");

  // If the overlay is still shown (window was visible as an overlay when
  // `send-to-scratchpad` was pressed), destroy it now. Without this the dim
  // overlay would persist on screen with no scratchpad window behind it.
  #[cfg(target_os = "windows")]
  {
    state.scratchpad_overlay = None;
  }

  // Clear the scratchpad marker so `platform_sync` treats it as a normal
  // window from this point on.
  non_tiling.set_scratchpad_origin(None);

  // Activate the origin workspace if it has been deactivated.
  if state.workspace_by_name(&origin.workspace_name).is_none() {
    activate_workspace(Some(&origin.workspace_name), None, state, config)?;
  }

  let target_workspace = state
    .workspace_by_name(&origin.workspace_name)
    .context("Origin workspace not found and could not be activated.")?;

  // Move from wherever the window currently lives (scratchpad workspace or
  // a regular workspace if shown as an overlay) to the origin workspace.
  detach_container(non_tiling.clone().into())?;
  attach_container(
    &non_tiling.clone().into(),
    &target_workspace.clone().into(),
    Some(target_workspace.child_count()),
  )?;

  // Restore the original window state (`Tiling` or `Floating`).
  // For `Tiling`, `update_window_state` uses the stored `insertion_target`
  // to place the window back in its original tiling position.
  let updated = update_window_state(
    non_tiling.clone().into(),
    origin.prev_state,
    state,
    config,
  )?;

  // Focus the restored window.
  set_focused_descendant(&updated.clone().into(), None);
  state.pending_sync.queue_focus_change();

  state.emit_event(WmEvent::ScratchpadToggled {
    shown: false,
    windows: vec![updated.to_dto()?],
  });

  Ok(())
}
