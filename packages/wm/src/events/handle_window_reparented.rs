#[cfg(target_os = "windows")]
use tracing::info;
use wm_platform::NativeWindow;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

#[cfg(target_os = "windows")]
use crate::{
  commands::window::unmanage_window_passively,
  events::handle_window_shown, models::WindowContainer,
  traits::WindowGetters,
};
use crate::{user_config::UserConfig, wm_state::WmState};

// LINT: The event is only emitted on Windows.
#[cfg_attr(
  not(target_os = "windows"),
  allow(unused_variables, clippy::needless_pass_by_value)
)]
pub fn handle_window_reparented(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  #[cfg(target_os = "windows")]
  match state.window_from_native(&native_window) {
    Some(window) => {
      unmanage_if_embedded(window, state)?;
    }
    // Released back to the desktop, e.g. a tab popped out of a tabbing
    // app. Managed like any newly shown window.
    None if native_window.is_top_level() => {
      handle_window_shown(native_window, state, config)?;
    }
    None => {}
  }

  Ok(())
}

/// Unmanages `window` if another app has embedded it into one of its own
/// windows. Returns whether it did.
///
/// Tabbing apps reparent a window, then hide and re-show it within
/// milliseconds. By the time the hide event is handled the window is
/// visible again, so without this check it stays managed: tiled, bordered
/// and positioned as if it were still top-level.
#[cfg(target_os = "windows")]
pub fn unmanage_if_embedded(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<bool> {
  if window.native().is_top_level() {
    return Ok(false);
  }

  info!("Window embedded into another window: {window}");

  // Focus goes to the window it was embedded into, when managed.
  let host = window
    .native()
    .root_window()
    .and_then(|root| state.window_from_native(&root))
    .map(Into::into);

  // The WM may be holding it cloaked for an animation, which unmanaging
  // cancels; left cloaked, it would stay invisible in its new parent.
  if window.native().is_cloaked().unwrap_or(false) {
    let _ = window.native().set_cloaked(false);
  }

  unmanage_window_passively(window, host, state)?;
  Ok(true)
}
