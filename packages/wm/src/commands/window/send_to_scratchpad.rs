use anyhow::Context;
use tracing::info;
use wm_common::{FloatingStateConfig, WindowState, WmEvent};

use crate::{
  commands::{
    container::{attach_container, detach_container, set_focused_descendant},
    window::update_window_state,
  },
  models::{ScratchpadOrigin, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Sends the focused window to the scratchpad.
///
/// The window is converted to a floating, always-on-top overlay and moved
/// to the detached `__scratchpad__` workspace, where `platform_sync` cloaks
/// it automatically. Use `toggle-scratchpad` to show or hide the pool.
///
/// No-ops if the window is already in the scratchpad (hidden or currently
/// shown as an overlay).
pub fn send_to_scratchpad(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Skip if window is already a scratchpad window (shown or hidden).
  let is_scratchpad = window
    .as_non_tiling_window()
    .is_some_and(|nw| nw.scratchpad_origin().is_some())
    || state
      .scratchpad_workspace
      .descendants()
      .any(|c| c.id() == window.id());

  if is_scratchpad {
    return Ok(());
  }

  info!("Sending window to scratchpad: {window}.");

  let workspace = window.workspace().context("Window has no workspace.")?;
  let workspace_name = workspace.config().name.clone();
  let prev_state = window.state();

  // Convert to floating with always-on-top so it renders above tiling
  // windows when shown via `toggle-scratchpad`.
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

  // Mark the window so it is recognised as a scratchpad member.
  non_tiling.set_scratchpad_origin(Some(ScratchpadOrigin {
    workspace_name,
    prev_state,
  }));

  // Compute focus target before detaching so the workspace focus order is
  // still intact.
  let focus_target = state.focus_target_after_removal(&non_tiling.clone().into());

  // Move to the detached scratchpad workspace. Raw detach + attach is used
  // because there is no common ancestor with the scratchpad workspace.
  let scratchpad_ws = state.scratchpad_workspace.clone().into();
  detach_container(non_tiling.clone().into())?;
  attach_container(&non_tiling.clone().into(), &scratchpad_ws, None)?;

  // Queue for redraw: `platform_sync` will cloak the window because
  // `__scratchpad__` is never displayed.
  state
    .pending_sync
    .queue_container_to_redraw(non_tiling.clone());

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
