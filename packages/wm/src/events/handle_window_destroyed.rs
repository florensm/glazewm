use anyhow::Context;
use tracing::info;
use wm_platform::WindowId;

use crate::{
  commands::{
    container::detach_container,
    window::unmanage_window,
    workspace::deactivate_workspace,
  },
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_destroyed(
  native_window_id: WindowId,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Check managed windows first.
  let found_window = state
    .windows()
    .into_iter()
    .find(|window| window.native().id() == native_window_id);

  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace.")?;

    // Check before unmanaging so the flag is still readable on the Rc.
    #[cfg(target_os = "windows")]
    let was_shown_scratchpad = window
      .as_non_tiling_window()
      .is_some_and(|nw| nw.scratchpad_origin().is_some());

    info!("Window closed: {window}");
    unmanage_window(window, state)?;

    // Destroy the dim overlay if the closed window was the last shown
    // scratchpad overlay (scratchpad_shown_windows no longer includes it
    // because it is now detached from the tree).
    #[cfg(target_os = "windows")]
    if was_shown_scratchpad && state.scratchpad_shown_windows().is_empty() {
      state.scratchpad_overlay = None;
    }

    // Destroy parent workspace if window was killed while its workspace
    // was not displayed (e.g. via task manager).
    if !workspace.config().keep_alive
      && !workspace.has_children()
      && !workspace.is_displayed()
    {
      deactivate_workspace(workspace, state)?;
    }

    return Ok(());
  }

  // Check the scratchpad workspace for stashed windows that were closed
  // (e.g. killed via Task Manager while hidden).
  let scratchpad_window = state
    .scratchpad_windows()
    .into_iter()
    .find(|w| w.native().id() == native_window_id);

  if let Some(window) = scratchpad_window {
    info!("Stashed scratchpad window closed: {window}");
    detach_container(window.into())?;
  }

  Ok(())
}
