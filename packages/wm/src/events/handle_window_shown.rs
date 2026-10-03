use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::NativeWindow;

use crate::{
  commands::window::manage_window, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    info!("Window shown: {window}");

    // Update display state if window is already managed.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Showing
    {
      window.set_display_state(DisplayState::Shown);
    } else {
      state.pending_sync.queue_container_to_redraw(window);
    }
  } else if !state.ignored_windows.contains(&native_window) {
    #[cfg(target_os = "windows")]
    show_owner_tab(&native_window, state, config)?;

    // If the window is not managed and not explicitly ignored, manage it.
    manage_window(native_window, None, state, config)?;
  }

  Ok(())
}

/// Shows the stack tab that owns `native_window`, if that tab is hidden.
///
/// Windows hides a window along with its hidden (cloaked) owner, so a
/// popup opened by a background tab would otherwise never appear, nor be
/// managed. Its tab is shown with it instead, as the popup is about it.
#[cfg(target_os = "windows")]
fn show_owner_tab(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  use wm_platform::NativeWindowWindowsExt;

  use crate::{
    commands::{general::platform_sync, window::activate_stack_child},
    models::is_inactive_stack_child,
    traits::CommonGetters,
  };

  let Some(owner_id) = native_window.owner_window_id() else {
    return Ok(());
  };

  let Some(owner) = state
    .windows()
    .into_iter()
    .find(|window| window.native().id() == owner_id)
    .filter(|window| is_inactive_stack_child(window))
  else {
    return Ok(());
  };

  let Some(stack) = owner.parent().and_then(|p| p.as_stack().cloned())
  else {
    return Ok(());
  };

  info!("Showing the tab of a popup's owner: {owner}");
  activate_stack_child(&stack, &owner.into(), state);
  state.pending_sync.queue_focus_change();

  // Uncloaks the owner now, so that the popup is visible to manage.
  platform_sync(state, config)
}
