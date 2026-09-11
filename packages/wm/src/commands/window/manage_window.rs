use anyhow::Context;
use tracing::info;
use wm_common::{WindowRuleEvent, WindowState, WmEvent};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{NativeWindow, RectDelta};

use crate::{
  commands::{
    container::{attach_container, set_focused_descendant},
    window::run_window_rules,
  },
  models::{
    Container, Monitor, NativeWindowProperties, NonTilingWindow,
    TilingWindow, WindowContainer,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn manage_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let Some(native_properties) =
    check_is_manageable(&native_window, config).unwrap_or(None)
  else {
    return Ok(());
  };

  // Cloak as early as possible to minimise the visible flash before the
  // window is repositioned and animated. Non-tiling windows are uncloaked
  // by `platform_sync` for their target position; tiling windows are
  // uncloaked by the slide-in animation.
  //
  // Held by a guard so that every early return between here and a
  // successful hand-off undoes the cloak. See `CloakGuard`.
  #[cfg(target_os = "windows")]
  let cloak_guard = CloakGuard::cloak(&native_window);

  // Create the window instance. This may fail if the window handle has
  // already been destroyed, or if there's no nearest monitor/workspace to
  // place it in (e.g. a monitor/workspace reconfiguration race).
  let window = match create_window(
    native_window,
    native_properties,
    target_parent,
    state,
    config,
  ) {
    Ok(window) => window,
    Err(err) => {
      tracing::warn!("Operation failed: {:?}", err);
      // `cloak_guard` undoes the cloak as it drops.
      return Ok(());
    }
  };

  // Set the newly added window as focus descendant. This means the window
  // rules will be run as if the window is focused.
  set_focused_descendant(&window.clone().into(), None);

  // Window might be detached if `ignore` command has been invoked.
  let updated_window = run_window_rules(
    window.clone(),
    &WindowRuleEvent::Manage,
    state,
    config,
  )?;

  if let Some(window) = updated_window {
    info!("New window managed: {window}");

    state.emit_event(WmEvent::WindowManaged {
      managed_window: window.to_dto()?,
    });

    // OS focus should be set to the newly added window in case it's not
    // already focused.
    state.pending_sync.queue_focus_change();

    // Normally, a `PlatformEvent::WindowFocused` event is what triggers
    // focus effects and workspace reordering to be applied. However, when
    // a window is first launched, this event can come before the
    // window is managed, and so we need to force an update here.
    state.pending_sync.queue_focused_effect_update();
    state.pending_sync.queue_workspace_to_reorder(
      window.workspace().context("No workspace.")?,
    );

    // Sibling containers need to be redrawn if the window is tiling.
    state.pending_sync.queue_container_to_redraw(
      if window.state() == WindowState::Tiling {
        window.parent().context("No parent.")?
      } else {
        window.into()
      },
    );

    // The window is managed and queued for redraw, so `platform_sync`
    // (non-tiling) or the slide-in animation (tiling) now owns the
    // uncloak. Every earlier return leaves the guard to undo it.
    #[cfg(target_os = "windows")]
    cloak_guard.release();
  }
  // Otherwise the window was detached by an `ignore` rule, and the guard
  // uncloaks it so that it displays normally without GlazeWM managing it.

  Ok(())
}

/// Undoes [`manage_window`]'s early cloak unless explicitly released.
///
/// A window is cloaked before the WM knows whether it will end up managing
/// it, so that there is no visible flash at the window's original
/// position. Any path that gives up after that point has to undo the
/// cloak, because nothing else can. A cloaked window keeps `WS_VISIBLE`,
/// so [`NativeWindow::is_visible`] reports it as hidden while user32 still
/// hit-tests it: the window becomes invisible *and* swallows every click
/// over its rect. Neither `WmState`'s shutdown uncloak nor the watcher
/// process can recover it, since both only know about managed windows.
///
/// Using a guard rather than an uncloak on each early return means paths
/// added later are covered by construction.
#[cfg(target_os = "windows")]
struct CloakGuard {
  /// The cloaked window, taken once the cloak becomes someone else's
  /// responsibility.
  window: Option<NativeWindow>,
}

#[cfg(target_os = "windows")]
impl CloakGuard {
  /// Cloaks `window` and returns a guard that uncloaks it on drop.
  fn cloak(window: &NativeWindow) -> Self {
    let _ = window.set_cloaked(true);

    Self {
      window: Some(window.clone()),
    }
  }

  /// Hands responsibility for the uncloak to the caller, leaving the
  /// window cloaked.
  fn release(mut self) {
    self.window = None;
  }
}

#[cfg(target_os = "windows")]
impl Drop for CloakGuard {
  fn drop(&mut self) {
    if let Some(window) = self.window.take() {
      tracing::warn!(
        "Uncloaking window {:?}: it was cloaked for management that did \
         not complete.",
        window.id()
      );

      let _ = window.set_cloaked(false);
    }
  }
}

/// Checks if a window is manageable and retrieves its native properties.
///
/// Windows matched by a `force-manage` window rule skip the built-in
/// manageability checks (visibility is still required).
///
/// Returns `Ok(Some(properties))` if the window is manageable and its
/// properties were retrieved successfully.
fn check_is_manageable(
  native_window: &NativeWindow,
  config: &UserConfig,
) -> anyhow::Result<Option<NativeWindowProperties>> {
  if !native_window.is_visible()? {
    return Ok(None);
  }

  // Ensure window has a valid process name, title, etc.
  let native_properties = NativeWindowProperties::try_from(native_window)?;

  // Bypass the checks below for windows matched by a `force-manage`
  // window rule.
  if config.is_force_managed(&native_properties) {
    return Ok(Some(native_properties));
  }

  #[cfg(target_os = "macos")]
  {
    use wm_platform::NativeWindowExtMacOs;

    let is_standard_window = native_window.role()? == "AXWindow"
      && native_window.subrole()? == "AXStandardWindow";

    if !is_standard_window {
      tracing::debug!(
        "Skipping non-standard window: {native_properties:?}"
      );
      return Ok(None);
    }
  }

  #[cfg(target_os = "windows")]
  {
    use wm_platform::{
      NativeWindowWindowsExt, WS_CAPTION, WS_CHILD, WS_EX_NOACTIVATE,
      WS_EX_TOOLWINDOW,
    };

    // Ensure window is top-level (i.e. not a child window). Ignore
    // windows that cannot be focused or if they're unavailable in
    // task switcher (alt+tab menu).
    if native_window.has_window_style(WS_CHILD)
      || native_window
        .has_window_style_ex(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW)
    {
      tracing::debug!(
        "Skipping window with unmanageable styles (candidate for a `force-manage` rule): {native_properties:?}"
      );
      return Ok(None);
    }

    // Some applications spawn top-level windows for menus that
    // should be ignored. This includes the autocomplete popup in
    // Notepad++ and title bar menu in Keepass. Although not
    // foolproof, these can typically be identified by having an
    // owner window and no title bar.
    if native_window.has_owner_window()
      && !native_window.has_window_style(WS_CAPTION)
    {
      tracing::debug!(
        "Skipping owned window without caption (candidate for a `force-manage` rule): {native_properties:?}"
      );
      return Ok(None);
    }
  }

  Ok(Some(native_properties))
}

fn create_window(
  native_window: NativeWindow,
  native_properties: NativeWindowProperties,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  let nearest_monitor = state
    .nearest_monitor(&native_window)
    .context("No nearest monitor.")?;

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No nearest workspace.")?;

  let gaps_config = config.value.gaps.clone();
  let window_state =
    window_state_to_create(&native_properties, &nearest_monitor, config)?;

  // Attach the new window as the first child of the target parent (if
  // provided), otherwise, add as a sibling of the focused container.
  let (target_parent, target_index) = match target_parent {
    Some(parent) => (parent, 0),
    None => insertion_target(&window_state, state)?,
  };

  let target_workspace =
    target_parent.workspace().context("No target workspace.")?;

  let prefers_centered = config
    .value
    .window_behavior
    .state_defaults
    .floating
    .centered;

  // Calculate where window should be placed when floating is enabled. Use
  // the original width/height of the window and optionally position it in
  // the center of the workspace.
  let is_same_workspace = nearest_workspace.id() == target_workspace.id();
  let floating_placement = {
    let placement = if !is_same_workspace || prefers_centered {
      native_properties
        .frame
        .translate_to_center(&target_workspace.to_rect()?)
    } else {
      native_properties.frame.clone()
    };

    // Clamp the window size to be within the workspace's outer gaps. 10px
    // is arbitrary - helps differentiate from tiling windows.
    let max_workspace_rect = target_workspace.max_workspace_rect()?;
    placement.clamp_size(
      max_workspace_rect.width() - 10,
      max_workspace_rect.height() - 10,
    )
  };

  // Window has no border delta unless it's later changed via the
  // `adjust_borders` command.
  let border_delta = RectDelta::zero();

  let window_container: WindowContainer = match window_state {
    WindowState::Tiling => TilingWindow::new(
      None,
      native_window,
      native_properties,
      None,
      border_delta,
      floating_placement,
      false,
      gaps_config,
      Vec::new(),
      None,
    )
    .into(),
    _ => NonTilingWindow::new(
      None,
      native_window,
      native_properties,
      window_state,
      None,
      border_delta,
      None,
      floating_placement,
      !prefers_centered,
      Vec::new(),
      None,
    )
    .into(),
  };

  attach_container(
    &window_container.clone().into(),
    &target_parent,
    Some(target_index),
  )?;

  // The OS might spawn the window on a different monitor to the target
  // parent, so adjustments might need to be made because of DPI.
  if nearest_monitor
    .has_dpi_difference(&window_container.clone().into())?
  {
    window_container.set_has_pending_dpi_adjustment(true);
  }

  Ok(window_container)
}

/// Gets the initial state for a window based on its native state.
///
/// Note that maximized windows are initialized as tiling.
fn window_state_to_create(
  native_properties: &NativeWindowProperties,
  nearest_monitor: &Monitor,
  config: &UserConfig,
) -> anyhow::Result<WindowState> {
  if native_properties.is_minimized {
    return Ok(WindowState::Minimized);
  }

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No workspace.")?;

  // Only initialize as fullscreen if the window *exceeds* the workspace
  // bounds (due to the 1px inset).
  //
  // For example, with 0px outer gaps and a window that covers the entire
  // workspace, it would still not be initialized as fullscreen. The window
  // needs to be within the workspace's outer gaps by at least 1px on each
  // side.
  if !native_properties.is_maximized
    && native_properties
      .frame
      .inset(1)
      .contains_rect(&nearest_workspace.max_workspace_rect()?)
  {
    return Ok(WindowState::Fullscreen(
      config
        .value
        .window_behavior
        .state_defaults
        .fullscreen
        .clone(),
    ));
  }

  // Initialize windows that can't be resized as floating.
  if !native_properties.is_resizable {
    return Ok(WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ));
  }

  Ok(WindowState::default_from_config(&config.value))
}

/// Gets where to insert a new window in the container tree.
///
/// Rules:
/// - For non-tiling windows: Always append to the workspace.
/// - For tiling windows:
///   1. Try to insert after the focused tiling window (or its parent
///      stack) if one exists.
///   2. If a non-tiling window is focused, try to insert after the first
///      tiling window found.
///   3. If no tiling windows exist, append to the workspace.
///
/// New windows are never inserted into an existing `StackContainer`
/// automatically. Stacking is always an explicit user action via
/// `stack-insert` or `stack-absorb-neighbor`.
///
/// Returns tuple of (parent container, insertion index).
fn insertion_target(
  window_state: &WindowState,
  state: &WmState,
) -> anyhow::Result<(Container, usize)> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  // For tiling windows, try to find a suitable tiling window to insert
  // next to.
  if *window_state == WindowState::Tiling {
    let sibling = match focused_container {
      Container::TilingWindow(_) => Some(focused_container),
      _ => focused_workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window),
    };

    if let Some(sibling) = sibling {
      return Ok((
        sibling.parent().context("No parent.")?,
        sibling.index() + 1,
      ));
    }
  }

  // Default to appending to workspace.
  Ok((
    focused_workspace.clone().into(),
    focused_workspace.child_count(),
  ))
}
