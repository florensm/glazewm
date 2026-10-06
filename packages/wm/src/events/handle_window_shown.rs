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

    #[cfg(target_os = "windows")]
    if reconcile_shown_tab(&window, state, config)? {
      return Ok(());
    }

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

/// Handles a hidden stack tab that something other than the WM made
/// visible, e.g. Windows uncloaking a window its app brought to the
/// foreground.
///
/// The tab becomes the active one if it is the foreground window, and is
/// hidden again otherwise, so that the shown window always matches the
/// active tab of the tab bar. Returns whether it was such a tab.
#[cfg(target_os = "windows")]
fn reconcile_shown_tab(
  window: &crate::models::WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<bool> {
  use crate::{
    commands::{general::platform_sync, window::activate_stack_child},
    models::is_inactive_stack_child,
    traits::CommonGetters,
  };

  let is_hidden = matches!(
    window.display_state(),
    DisplayState::Hidden | DisplayState::Hiding
  );

  // Windows on hidden workspaces are handled as focus steals instead.
  let is_on_displayed_workspace =
    window.workspace().is_some_and(|ws| ws.is_displayed());

  if !is_inactive_stack_child(window)
    || !is_hidden
    || !is_on_displayed_workspace
    || state.animation_manager.has_active_surrogate(&window.id())
    || !window.native().is_visible().unwrap_or(false)
  {
    return Ok(false);
  }

  let Some(stack) = window.parent().and_then(|p| p.as_stack().cloned())
  else {
    return Ok(false);
  };

  let is_foreground = state
    .dispatcher
    .focused_window()
    .is_ok_and(|foreground| foreground.id() == window.native().id());

  if is_foreground {
    info!("Showing the tab of a window brought to the front: {window}");
    activate_stack_child(&stack, &window.clone().into(), state);
    state.pending_sync.queue_focus_change();
  } else {
    // Marked as shown so that the next sync hides it again.
    window.set_display_state(DisplayState::Shown);
  }

  state
    .pending_sync
    .queue_containers_to_redraw(stack.windows())
    .queue_tab_bar_update();

  platform_sync(state, config)?;
  Ok(true)
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
    .filter(is_inactive_stack_child)
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
