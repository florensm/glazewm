use wm_platform::NativeWindow;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

#[cfg(target_os = "windows")]
use crate::{
  commands::window::unmanage_if_embedded, events::handle_window_shown,
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
