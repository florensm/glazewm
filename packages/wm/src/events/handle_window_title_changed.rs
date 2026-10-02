use tracing::info;
use wm_common::{try_warn, WindowRuleEvent};
use wm_platform::NativeWindow;

use crate::{
  commands::window::{
    auto_stack_managed_window, is_auto_stacked, is_placement_command,
    manage_window, run_window_rules_except,
  },
  traits::WindowGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_title_changed(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let Some(window) = state.window_from_native(native_window) else {
    // A window held back for auto-stacking can be placed once it has a
    // title.
    if state.auto_stack.is_held(native_window.id()) {
      manage_window(native_window.clone(), None, state, config)?;
    }

    return Ok(());
  };

  info!("Window title changed: {window}");

  let title = try_warn!(window.native().title());

  window.update_native_properties(|properties| {
    properties.title = title;
  });

  auto_stack_managed_window(window.clone(), state, config)?;

  // The window may have been replaced while joining its stack.
  let Some(window) = state.window_from_native(native_window) else {
    return Ok(());
  };

  // Run window rules for title change events. Windows placed by an
  // auto-stack rule keep their placement.
  let is_auto_stacked = is_auto_stacked(&window, state);

  run_window_rules_except(
    window,
    &WindowRuleEvent::TitleChange,
    |command| is_auto_stacked && is_placement_command(command),
    state,
    config,
  )?;

  Ok(())
}
