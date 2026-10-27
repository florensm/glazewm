use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::NativeWindow;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

#[cfg(target_os = "windows")]
use crate::commands::window::unmanage_if_embedded;
use crate::{
  commands::window::unmanage_window, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_hidden(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    info!("Window hidden: {window}");

    // Hidden while being embedded into another window. Checked before the
    // display-state guards below, which would keep it managed.
    #[cfg(target_os = "windows")]
    if unmanage_if_embedded(window.clone(), state)? {
      return Ok(());
    }

    // Update the display state.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Hiding
    {
      window.set_display_state(DisplayState::Hidden);
      return Ok(());
    }

    // DWM cloaks owned windows along with their owner, e.g. while the WM
    // animates the owner. The window itself is still shown, and unmanaging
    // it would re-manage it as a new window once the owner is uncloaked.
    #[cfg(target_os = "windows")]
    if window.native().is_cloaked_by_owner() {
      return Ok(());
    }

    // Unmanage the window if it's not in a display state transition. Also,
    // since window events are not 100% guaranteed to be in correct order,
    // we need to ignore events where the window is not actually hidden.
    if (config.value.general.hide_method == HideMethod::PlaceInCorner
      || window.display_state() == DisplayState::Shown)
      && !window.native().is_visible().unwrap_or(false)
    {
      unmanage_window(window, state)?;
    }
  }

  Ok(())
}
