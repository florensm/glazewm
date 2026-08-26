use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::NativeWindow;

use crate::{
  commands::window::unmanage_window,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_hidden(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    info!("Window hidden: {window}");

    // Update the display state.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Hiding
    {
      window.set_display_state(DisplayState::Hidden);
      return Ok(());
    }

    // On Windows, skip unmanagement if GlazeWM itself is the one holding
    // this window cloaked for an in-progress animation (resize/move/open/
    // close surrogate, or a workspace-switch slide). Workspace-switch
    // cloaking via `HideMethod::Cloak` is already handled above via the
    // `Hiding` display-state guard, so this only matters for
    // surrogate-cloaked windows whose display state is still `Shown`.
    //
    // Checking `has_active_surrogate` instead of the raw `is_cloaked()`
    // state matters because cloaking is not exclusively a GlazeWM signal:
    // some UWP apps also self-cloak to indicate they've legitimately
    // hidden (see `is_cloaked`'s doc comment), and a blanket `is_cloaked()`
    // skip would leave such an app tracked forever. It would also silently
    // drop a real hide event for a window the user minimized/hid while
    // GlazeWM's own animation happened to have it cloaked at that instant
    // -- `platform_sync` unconditionally uncloaks surrogate-tracked windows
    // once their animation ends, which would then re-reveal a window the
    // user tried to hide.
    #[cfg(target_os = "windows")]
    if state.animation_manager.has_active_surrogate(&window.id()) {
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
