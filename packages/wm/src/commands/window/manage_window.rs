use anyhow::Context;
use tracing::info;
use wm_common::{
  AutoStackRuleConfig, DuplicateTabs, InvokeCommand, WindowRuleEvent,
  WindowState, WmEvent,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{NativeWindow, RectDelta};

use crate::{
  auto_stack::{decide, AutoStackDecision, WindowTraits},
  commands::{
    container::{
      attach_container, detach_container, set_focused_descendant,
    },
    window::{
      find_named_stack, match_stack_tabs, new_named_stack, restored_state,
      run_window_rules_except, update_stack_state, update_window_state,
    },
  },
  models::{
    Container, Monitor, NativeWindowProperties, NonTilingWindow,
    StackContainer, TilingWindow, WindowContainer,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Manages a newly shown window.
///
/// A window that could still match a `stack.auto_stack` rule once it has
/// a title is held back, cloaked, until it gets one or the wait runs out.
pub fn manage_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  manage_window_inner(native_window, target_parent, true, state, config)
}

/// Manages a window whose wait for a title ran out, placing it normally
/// unless its current title matches an auto-stack rule.
pub fn manage_held_window(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  manage_window_inner(native_window, None, false, state, config)
}

fn manage_window_inner(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  may_hold: bool,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let is_held = state.auto_stack.is_held(native_window.id());

  let Some(native_properties) =
    check_is_manageable(&native_window, is_held, config).unwrap_or(None)
  else {
    if state.auto_stack.release(native_window.id()).is_some() {
      uncloak_held_window(&native_window);
    }

    return Ok(());
  };

  let decision = if state.auto_stack.is_settled(native_window.id()) {
    AutoStackDecision::Skip
  } else {
    let traits = WindowTraits::of(&native_window, &native_properties);
    decide(&config.value.stack.auto_stack, &native_properties, traits)
  };

  // Holding relies on cloaking, which only exists on Windows.
  if decision == AutoStackDecision::Wait
    && cfg!(target_os = "windows")
    && may_hold
    && target_parent.is_none()
  {
    hold_window(&native_window, state, config);
    return Ok(());
  }

  let auto_stack_rule = join_rule(&decision, &native_properties.title);

  // Windows already open at startup are given a workspace to go to.
  let is_new = target_parent.is_none();

  state.auto_stack.release(native_window.id());

  // Cloak as early as possible to minimise the visible flash before the
  // window is repositioned and animated. Non-tiling windows are uncloaked
  // by `platform_sync` for their target position; tiling windows are
  // uncloaked by the slide-in animation.
  //
  // Held by a guard so that every early return between here and a
  // successful hand-off undoes the cloak. See `CloakGuard`.
  #[cfg(target_os = "windows")]
  let cloak_guard = CloakGuard::cloak(&native_window);

  let placement = match &auto_stack_rule {
    Some(rule) => Some(auto_stack_placement(rule, state, config)?),
    None => target_parent.map(|parent| Placement {
      parent,
      index: 0,
      created_stack: None,
    }),
  };

  let state_override = initial_state_override(
    placement
      .as_ref()
      .filter(|_| auto_stack_rule.is_some())
      .and_then(|placement| placement.parent.as_stack())
      .map(StackContainer::state),
    &native_window,
    &native_properties,
    state,
    config,
  );

  // Create the window instance. This may fail if the window handle has
  // already been destroyed, or if there's no nearest monitor/workspace to
  // place it in (e.g. a monitor/workspace reconfiguration race).
  let window = match create_window(
    native_window,
    native_properties,
    placement.clone(),
    state_override,
    state,
    config,
  ) {
    Ok(window) => window,
    Err(err) => {
      tracing::warn!("Operation failed: {:?}", err);

      // Don't leave behind a stack that was only created for this window.
      if let Some(stack) = placement.and_then(|p| p.created_stack) {
        if !stack.has_children() {
          detach_container(stack.into())?;
        }
      }

      // `cloak_guard` undoes the cloak as it drops.
      return Ok(());
    }
  };

  if let Some(rule) = &auto_stack_rule {
    on_auto_stacked(&window, rule, is_new, state);
  }

  // A stacked window only takes focus if the OS already gave it the
  // foreground, i.e. the user opened it. Otherwise it becomes the active
  // tab without pulling focus away from another app.
  let takes_focus =
    auto_stack_rule.is_none() || is_foreground(&window, state);

  // Set the newly added window as focus descendant. This means the window
  // rules will be run as if the window is focused.
  if takes_focus {
    set_focused_descendant(&window.clone().into(), None);
  } else if let Some(stack) = window.parent() {
    set_focused_descendant(&window.clone().into(), Some(&stack));
  }

  // A stacked window keeps the placement its stack gives it, so window
  // rules that would move it or change its state are skipped.
  let is_placement_command = |command: &InvokeCommand| {
    auto_stack_rule.is_some() && is_placement_command(command)
  };

  // Window might be detached if `ignore` command has been invoked.
  let updated_window = run_window_rules_except(
    window.clone(),
    &WindowRuleEvent::Manage,
    is_placement_command,
    state,
    config,
  )?;

  if let Some(window) = updated_window {
    queue_managed_window_sync(
      &window,
      takes_focus,
      placement.and_then(|p| p.created_stack),
      state,
      config,
    )?;

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

/// Emits the managed event and queues the redraw and focus updates for a
/// newly managed window.
fn queue_managed_window_sync(
  window: &WindowContainer,
  takes_focus: bool,
  created_stack: Option<StackContainer>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("New window managed: {window}");

  // Take the workspace out of fullscreen before the new window is laid
  // out, so it doesn't open hidden behind a monitor-sized window.
  exit_fullscreen_for_new_window(window, state, config)?;

  state.emit_event(WmEvent::WindowManaged {
    managed_window: window.to_dto()?,
  });

  if takes_focus {
    // OS focus should be set to the newly added window in case it's not
    // already focused.
    state.pending_sync.queue_focus_change();

    // Normally, a `PlatformEvent::WindowFocused` event is what triggers
    // focus effects and workspace reordering to be applied. However,
    // when a window is first launched, this event can come before the
    // window is managed, and so we need to force an update here.
    state.pending_sync.queue_focused_effect_update();
    state.pending_sync.queue_workspace_to_reorder(
      window.workspace().context("No workspace.")?,
    );
  }

  // Sibling containers need to be redrawn if the window is tiling. A
  // newly created stack also resizes its siblings.
  let redraw_target = match created_stack {
    Some(stack) => stack.parent().context("No parent.")?,
    None if window.state() == WindowState::Tiling => {
      window.parent().context("No parent.")?
    }
    None => window.clone().into(),
  };

  state.pending_sync.queue_container_to_redraw(redraw_target);

  Ok(())
}

/// Where a new window is attached in the container tree.
#[derive(Clone)]
struct Placement {
  parent: Container,
  index: usize,

  /// Stack created to receive the window, if the target stack didn't
  /// exist yet.
  created_stack: Option<StackContainer>,
}

/// Resolves the named stack that `rule` puts windows into, creating it
/// when it doesn't exist yet.
fn auto_stack_placement(
  rule: &AutoStackRuleConfig,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Placement> {
  if let Some(stack) = find_named_stack(state, &rule.name) {
    // A new window shows up in its stack, so a minimized stack comes back.
    if stack.state() == WindowState::Minimized {
      update_stack_state(
        &stack,
        restored_state(&stack, config),
        state,
        config,
      )?;
    }

    return Ok(Placement {
      index: stack.new_tab_index(config.value.stack.new_tab_position),
      parent: stack.into(),
      created_stack: None,
    });
  }

  let configured_workspace = rule
    .workspace
    .as_ref()
    .and_then(|name| state.workspace_by_name(name));

  let (parent, index) = match configured_workspace {
    Some(workspace) => {
      let index = workspace.child_count();
      (workspace.into(), index)
    }
    None => insertion_target(&WindowState::Tiling, false, state)?,
  };

  let stack = new_named_stack(&rule.name, config);
  attach_container(&stack.clone().into(), &parent, Some(index))?;

  Ok(Placement {
    parent: stack.clone().into(),
    index: 0,
    created_stack: Some(stack),
  })
}

/// The rule whose stack a window joins, logging why a matching window
/// doesn't.
fn join_rule(
  decision: &AutoStackDecision<'_>,
  title: &str,
) -> Option<AutoStackRuleConfig> {
  match decision {
    AutoStackDecision::Join(rule) => Some((*rule).clone()),
    AutoStackDecision::Blocked(reason) => {
      info!("Not auto-stacking window '{title}' because {reason}.");
      None
    }
    AutoStackDecision::Wait | AutoStackDecision::Skip => None,
  }
}

/// Window state a new window is created in regardless of its native
/// state, if any.
///
/// Auto-stacked windows take the state of their stack. Popups of an app
/// with stacked windows float, as they would over a `StackTabs` host.
fn initial_state_override(
  auto_stack_state: Option<WindowState>,
  native_window: &NativeWindow,
  properties: &NativeWindowProperties,
  state: &WmState,
  config: &UserConfig,
) -> Option<WindowState> {
  if auto_stack_state.is_some() {
    auto_stack_state
  } else if is_popup_of_stacked_app(
    native_window,
    properties,
    state,
    config,
  ) {
    Some(WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ))
  } else {
    None
  }
}

/// Whether `native_window` is a popup (a window with an owner) of an app
/// that has windows in a stack.
fn is_popup_of_stacked_app(
  native_window: &NativeWindow,
  properties: &NativeWindowProperties,
  state: &WmState,
  config: &UserConfig,
) -> bool {
  #[cfg(target_os = "windows")]
  let has_owner = native_window.has_owner_window();
  #[cfg(not(target_os = "windows"))]
  let has_owner = {
    let _ = native_window;
    false
  };

  config.value.stack.float_owned_popups
    && has_owner
    && state.windows().iter().any(|window| {
      window
        .parent()
        .is_some_and(|parent| parent.as_stack().is_some())
        && window.native_properties().process_name
          == properties.process_name
    })
}

/// Holds `native_window` back, cloaked, until it gets a title.
fn hold_window(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) {
  if !state.auto_stack.is_held(native_window.id()) {
    info!(
      "Holding untitled window back for auto-stacking: {:?}",
      native_window.id()
    );

    #[cfg(target_os = "windows")]
    let _ = native_window.set_cloaked(true);
  }

  let timeout = std::time::Duration::from_millis(
    config.value.stack.auto_stack_title_timeout_ms,
  );

  state.auto_stack.hold(native_window, timeout);
}

/// Undoes the cloak of a window that was held back and won't be managed.
pub fn uncloak_held_window(native_window: &NativeWindow) {
  #[cfg(target_os = "windows")]
  let _ = native_window.set_cloaked(false);

  #[cfg(not(target_os = "windows"))]
  let _ = native_window;
}

/// Records that `window` joined its stack by `rule`. A window that just
/// opened (`is_new`) also gets the rule's `duplicates` and
/// `send_keys_on_join` applied.
pub fn on_auto_stacked(
  window: &WindowContainer,
  rule: &AutoStackRuleConfig,
  is_new: bool,
  state: &mut WmState,
) {
  info!("Auto-stacking window into stack '{}': {window}", rule.name);
  state.auto_stack.mark_settled(window.native().id());

  if !is_new {
    return;
  }

  let duplicates = if rule.duplicates == DuplicateTabs::CloseOlder {
    duplicate_tabs(window)
  } else {
    Vec::new()
  };

  // The duplicates are closed after the keys are pressed, since closing
  // them can move the foreground to another window first.
  #[cfg(target_os = "windows")]
  if !rule.send_keys_on_join.is_empty() {
    state.auto_stack.queue_keys(
      &window.native(),
      rule.send_keys_on_join.clone(),
      duplicates,
    );
    return;
  }

  close_duplicate_tabs(&duplicates);
}

/// The other windows in `window`'s stack that have the same process and
/// title.
fn duplicate_tabs(window: &WindowContainer) -> Vec<NativeWindow> {
  let Some(stack) = window.parent().and_then(|p| p.as_stack().cloned())
  else {
    return Vec::new();
  };

  let properties = window.native_properties();

  stack
    .windows()
    .into_iter()
    .filter(|other| {
      other.id() != window.id()
        && is_duplicate(&properties, &other.native_properties())
    })
    .map(|other| other.native().clone())
    .collect()
}

/// Closes tabs duplicated by a window that joined their stack.
pub fn close_duplicate_tabs(duplicates: &[NativeWindow]) {
  for duplicate in duplicates {
    info!(
      "Closing tab duplicated by a new window: '{}'",
      duplicate.title().unwrap_or_default()
    );

    if let Err(err) = duplicate.close() {
      tracing::warn!("Failed to close duplicate tab: {err}");
    }
  }
}

/// Whether two windows have the same process and the same, non-empty
/// title.
fn is_duplicate(
  a: &NativeWindowProperties,
  b: &NativeWindowProperties,
) -> bool {
  !a.title.trim().is_empty()
    && a.title == b.title
    && a.process_name == b.process_name
}

/// Whether `window` is the OS foreground window.
fn is_foreground(window: &WindowContainer, state: &WmState) -> bool {
  state
    .dispatcher
    .focused_window()
    .is_ok_and(|foreground| foreground.id() == window.native().id())
}

/// Whether `command` moves a window or changes its window state.
pub fn is_placement_command(command: &InvokeCommand) -> bool {
  matches!(
    command,
    InvokeCommand::Move(_)
      | InvokeCommand::MoveToStack { .. }
      | InvokeCommand::SetFloating { .. }
      | InvokeCommand::SetFullscreen { .. }
      | InvokeCommand::SetMinimized
      | InvokeCommand::SetTiling
      | InvokeCommand::ToggleFloating { .. }
      | InvokeCommand::ToggleFullscreen { .. }
      | InvokeCommand::ToggleMinimized
      | InvokeCommand::ToggleStack
      | InvokeCommand::ToggleTiling
  )
}

/// Takes every other fullscreen window on `window`'s workspace back out of
/// fullscreen.
///
/// A fullscreen window covers the whole workspace, so a window spawned
/// onto it would otherwise open invisible behind it -- and, being tiling,
/// would shrink the fullscreen window's own tile without that being
/// visible either. Each window returns to its previous state via
/// `toggled_state`, the same path `toggle-fullscreen` uses.
///
/// Gated on `window_behavior.exit_fullscreen_on_new_window` (default
/// `true`).
fn exit_fullscreen_for_new_window(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if !config.value.window_behavior.exit_fullscreen_on_new_window {
    return Ok(());
  }

  let workspace = window.workspace().context("No workspace.")?;

  // Collected before updating, since `update_window_state` replaces
  // containers in the tree the iterator walks.
  let fullscreen_ids = workspace
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok())
    .filter(|other| {
      other.id() != window.id()
        && matches!(other.state(), WindowState::Fullscreen(_))
    })
    .map(|other| other.id())
    .collect::<Vec<_>>();

  for id in fullscreen_ids {
    // Looked up again: exiting fullscreen for a stacked window does so for
    // its whole stack, replacing the containers of the other tabs.
    let Some(fullscreen_window) = state
      .container_by_id(id)
      .and_then(|container| container.as_window_container().ok())
      .filter(|other| matches!(other.state(), WindowState::Fullscreen(_)))
    else {
      continue;
    };

    let target_state =
      fullscreen_window.toggled_state(fullscreen_window.state(), config);

    info!(
      "Exiting fullscreen for {fullscreen_window}: new window on workspace."
    );

    update_window_state(fullscreen_window, target_state, state, config)?;
  }

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
  is_held: bool,
  config: &UserConfig,
) -> anyhow::Result<Option<NativeWindowProperties>> {
  // A held window is cloaked by the WM itself, which `is_visible` would
  // report as hidden.
  #[cfg(target_os = "windows")]
  let is_visible = if is_held {
    use wm_platform::WS_VISIBLE;
    native_window.has_window_style(WS_VISIBLE)
  } else {
    native_window.is_visible()?
  };
  #[cfg(not(target_os = "windows"))]
  let is_visible = {
    let _ = is_held;
    native_window.is_visible()?
  };

  if !is_visible {
    return Ok(None);
  }

  // Ensure window has a valid process name, title, etc.
  let native_properties = NativeWindowProperties::try_from(native_window)?;

  // Checked before the `force-manage` bypass: a window another app has
  // embedded (e.g. a tabbing app) is laid out by that app, never the WM.
  #[cfg(target_os = "windows")]
  if !native_window.is_top_level() {
    tracing::debug!("Skipping embedded window: {native_properties:?}");
    return Ok(None);
  }

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
  placement: Option<Placement>,
  state_override: Option<WindowState>,
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
  let window_state = match state_override {
    Some(window_state) => window_state,
    None => {
      window_state_to_create(&native_properties, &nearest_monitor, config)?
    }
  };

  // Attach the new window at the given placement (if provided),
  // otherwise, add as a sibling of the focused container.
  let (target_parent, target_index) = match placement {
    Some(placement) => (placement.parent, placement.index),
    None => insertion_target(
      &window_state,
      config.value.stack.new_windows_join_focused_stack,
      state,
    )?,
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

  // Joining a non-tiling stack, the window takes the place of the others.
  let template = target_parent
    .as_stack()
    .filter(|stack| !stack.is_tiling())
    .and_then(|stack| stack.windows().into_iter().next());

  attach_container(
    &window_container.clone().into(),
    &target_parent,
    Some(target_index),
  )?;

  if let (WindowContainer::NonTilingWindow(window), Some(template)) =
    (&window_container, template)
  {
    match_stack_tabs(window, &template);
  }

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
/// automatically, unless configured. Stacking is otherwise an explicit
/// user action, e.g. via `toggle-stack` or `stack-absorb-neighbor`.
///
/// Returns tuple of (parent container, insertion index).
fn insertion_target(
  window_state: &WindowState,
  joins_focused_stack: bool,
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
      let parent = sibling.parent().context("No parent.")?;

      // Tile next to the sibling's stack rather than inside it, unless
      // configured otherwise.
      if parent.as_stack().is_some() && !joins_focused_stack {
        let stack_parent = parent.parent().context("No parent.")?;
        return Ok((stack_parent, parent.index() + 1));
      }

      return Ok((parent, sibling.index() + 1));
    }
  }

  // Default to appending to workspace.
  Ok((
    focused_workspace.clone().into(),
    focused_workspace.child_count(),
  ))
}

#[cfg(test)]
mod tests {
  use wm_common::{
    AutoStackRuleConfig, DuplicateTabs, ParsedConfig, WindowState,
  };

  use super::{auto_stack_placement, insertion_target};
  use crate::{
    commands::container::set_focused_descendant,
    models::{Monitor, StackContainer, TilingWindow, Workspace},
    traits::CommonGetters,
    user_config::UserConfig,
    wm_state::WmState,
  };

  fn details_rule(workspace: Option<&str>) -> AutoStackRuleConfig {
    AutoStackRuleConfig {
      name: "details".to_string(),
      match_window: vec![],
      exclude: vec![],
      workspace: workspace.map(ToString::to_string),
      allow_owned: false,
      send_keys_on_join: Vec::new(),
      duplicates: DuplicateTabs::Keep,
    }
  }

  fn config() -> UserConfig {
    UserConfig::from_parsed(ParsedConfig::default())
  }

  #[test]
  fn joins_the_named_stack_on_another_workspace() {
    let stacked = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .name("details".to_string())
      .tiling_containers(vec![stacked.into()])
      .call();
    let first = Workspace::mock().name("1".to_string()).call();
    let second = Workspace::mock()
      .name("2".to_string())
      .tiling_containers(vec![stack.clone().into()])
      .call();
    let monitor = Monitor::mock()
      .workspaces(vec![first.clone(), second])
      .call();
    let mut state = WmState::mock(vec![monitor]);
    set_focused_descendant(&first.into(), None);

    let placement =
      auto_stack_placement(&details_rule(None), &mut state, &config())
        .unwrap();

    assert_eq!(placement.parent.id(), stack.id());
    assert_eq!(placement.index, 1);
    assert!(placement.created_stack.is_none());

    // Dropping `WmState` would restore its mock windows via Win32 calls.
    std::mem::forget(state);
  }

  #[test]
  fn creates_the_stack_on_its_configured_workspace() {
    let first = Workspace::mock().name("1".to_string()).call();
    let second = Workspace::mock().name("2".to_string()).call();
    let monitor = Monitor::mock()
      .workspaces(vec![first.clone(), second.clone()])
      .call();
    let mut state = WmState::mock(vec![monitor]);
    set_focused_descendant(&first.into(), None);

    let placement = auto_stack_placement(
      &details_rule(Some("2")),
      &mut state,
      &config(),
    )
    .unwrap();

    let stack = placement.created_stack.unwrap();
    assert_eq!(stack.name().as_deref(), Some("details"));
    assert_eq!(stack.parent().unwrap().id(), second.id());
    assert_eq!(placement.parent.id(), stack.id());

    std::mem::forget(state);
  }

  #[test]
  fn creates_the_stack_next_to_the_focused_window() {
    let focused = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![focused.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let mut state = WmState::mock(vec![monitor]);
    set_focused_descendant(&focused.into(), None);

    let placement =
      auto_stack_placement(&details_rule(None), &mut state, &config())
        .unwrap();

    let stack = placement.created_stack.unwrap();
    assert_eq!(stack.parent().unwrap().id(), workspace.id());
    assert_eq!(stack.index(), 1);

    std::mem::forget(state);
  }

  #[test]
  fn new_windows_tile_next_to_a_focused_stack() {
    let stacked = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![stacked.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);
    set_focused_descendant(&stacked.into(), None);

    let (parent, index) =
      insertion_target(&WindowState::Tiling, false, &state).unwrap();
    assert_eq!(parent.id(), workspace.id());
    assert_eq!(index, stack.index() + 1);

    let (parent, index) =
      insertion_target(&WindowState::Tiling, true, &state).unwrap();
    assert_eq!(parent.id(), stack.id());
    assert_eq!(index, 1);

    std::mem::forget(state);
  }

  #[test]
  fn duplicates_need_the_same_process_and_title() {
    use super::is_duplicate;
    use crate::models::NativeWindowProperties;

    let window = |process: &str, title: &str| {
      NativeWindowProperties::mock()
        .process_name(process.to_string())
        .title(title.to_string())
        .call()
    };
    let original = window("MyApp", "Details for item 42");

    assert!(is_duplicate(
      &original,
      &window("MyApp", "Details for item 42")
    ));
    assert!(!is_duplicate(
      &original,
      &window("MyApp", "Details for item 43")
    ));
    assert!(!is_duplicate(
      &original,
      &window("notepad", "Details for item 42")
    ));
    assert!(!is_duplicate(&window("MyApp", ""), &window("MyApp", "")));
  }
}
