use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use anyhow::{bail, Context};
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;
#[cfg(target_os = "windows")]
use wm_common::TitleBarVisibility;
use wm_common::{
  FloatingStateConfig, FullscreenStateConfig, InvokeCommand, WindowState,
  WmEvent,
};
#[cfg(target_os = "windows")]
use wm_common::{InvokeFocusCommand, InvokeMoveCommand};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{
  Dispatcher, Keybinding, LengthValue, OverviewAction, PlatformEvent,
  RectDelta, WindowEvent,
};

#[cfg(target_os = "windows")]
use crate::commands::window::close_duplicate_tabs;
#[cfg(target_os = "windows")]
use crate::commands::window::detach_window_for_close;
use crate::{
  commands::{
    column::{
      center_column, center_visible_columns, consume_or_expel_window,
      consume_window_into_column, cycle_column_width, cycle_window_height,
      expand_column_to_available_width, expel_window_from_column,
      focus_column, focus_window_in_column, maximize_column, move_column,
      reset_window_height, set_column_display, swap_window, ColumnTarget,
      RowTarget,
    },
    container::{
      focus_container_by_id, focus_in_direction, set_tiling_direction,
      toggle_tiling_direction,
    },
    general::{
      cycle_focus, disable_binding_mode, enable_binding_mode,
      platform_sync, reload_config, shell_exec, toggle_pause,
    },
    monitor::focus_monitor,
    window::{
      cycle_stack_focus, float_out_of_stack, focus_stack_index,
      focus_urgent_window, ignore_window, manage_held_window,
      move_stack_tab, move_to_stack, move_window_in_direction,
      move_window_to_workspace, resize_window, set_window_position,
      set_window_size, set_window_urgency, stack_absorb_neighbor,
      stack_all, toggle_stack, unstack_all, update_window_state,
      WindowPositionTarget,
    },
    workspace::{
      focus_special_workspace, focus_workspace,
      move_window_to_special_workspace, move_workspace_in_direction,
      set_layout, update_workspace_config,
    },
  },
  events::{
    handle_display_settings_changed, handle_mouse_move,
    handle_window_attention_requested, handle_window_destroyed,
    handle_window_focused, handle_window_hidden,
    handle_window_minimize_ended, handle_window_minimized,
    handle_window_moved_or_resized, handle_window_reparented,
    handle_window_shown, handle_window_title_changed,
    handle_z_order_changed,
  },
  ipc_server::IpcServer,
  models::{workspace_target, Container},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: mpsc::UnboundedReceiver<WmEvent>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  pub animation_tick_rx: mpsc::UnboundedReceiver<()>,
  /// Actions taken in the overview, with their session. Never yields on
  /// platforms without the overview.
  pub overview_action_rx: mpsc::UnboundedReceiver<(u64, OverviewAction)>,
  pub state: WmState,
  /// Actions taken in tab bars, keyed by stack ID. Never yields on
  /// platforms without tab bars.
  pub tab_action_rx:
    mpsc::UnboundedReceiver<(uuid::Uuid, wm_platform::TabAction)>,
}

impl WindowManager {
  pub fn new(
    config: &mut UserConfig,
    dispatcher: Dispatcher,
  ) -> anyhow::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();
    let (animation_tick_tx, animation_tick_rx) = mpsc::unbounded_channel();
    let (overview_action_tx, overview_action_rx) =
      mpsc::unbounded_channel();
    let (tab_action_tx, tab_action_rx) = mpsc::unbounded_channel();

    let mut state = WmState::new(
      dispatcher,
      event_tx,
      exit_tx,
      animation_tick_tx,
      overview_action_tx,
      tab_action_tx,
    );
    state.populate(config)?;

    // The first switch played on a scene would otherwise wait for the
    // scene's device to be created, D3D included.
    #[cfg(target_os = "windows")]
    if config.value.animations.workspace_switch.enabled {
      if let Err(err) = wm_platform::NativeScene::prepare() {
        tracing::warn!("Workspace switches can't play on scenes: {err}");
      }
    }

    // Start animation timer if `populate` created any animations. This
    // mirrors the `ensure_timer_running` call at the end of
    // `process_event` for the initial population path.
    state.animation_manager.ensure_timer_running();

    Ok(Self {
      event_rx,
      exit_rx,
      animation_tick_rx,
      overview_action_rx,
      state,
      tab_action_rx,
    })
  }

  /// Carries out an action taken in the overview, unless it comes from an
  /// earlier session than the open one.
  // LINT: The overview only exists on Windows.
  #[cfg_attr(
    not(target_os = "windows"),
    allow(clippy::unused_self, clippy::unnecessary_wraps)
  )]
  pub fn process_overview_action(
    &mut self,
    session: u64,
    action: OverviewAction,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
      let _ = (session, action, config);
      Ok(())
    }

    #[cfg(target_os = "windows")]
    {
      let state = &mut self.state;

      // What the WM held off while it was on screen resumes.
      if action == OverviewAction::Hidden {
        return if state.overview.mark_hidden(session) {
          platform_sync(state, config)
        } else {
          Ok(())
        };
      }

      if state.overview.open_session() != Some(session) {
        return Ok(());
      }

      let focused_workspace = state
        .focused_container()
        .and_then(|focused| focused.workspace())
        .map(|workspace| workspace.config().name);
      let window = window_by_handle(state, &action);

      let (commands, subject) = match action {
        OverviewAction::FocusWindow { monitor, .. } => {
          state.overview.close();
          let Some(window) = window else {
            state.pending_sync.queue_focus_change();
            return platform_sync(state, config);
          };

          let workspace = window.workspace().map(|w| w.config().name);
          let mut commands = Vec::new();

          // A window-level focus doesn't switch the displayed workspace
          // on its own.
          if let Some(name) = workspace
            .filter(|name| Some(name) != focused_workspace.as_ref())
          {
            crate::overview::prepare_target(
              state, config, &name, monitor, true,
            )?;
            commands.push(focus_workspace_command(name));
          }

          commands.push(InvokeCommand::Focus(InvokeFocusCommand {
            container_id: Some(window.id()),
            ..Default::default()
          }));

          // The overview was the transition; the switch behind it is
          // instant.
          state.pending_sync.suppress_animations();
          (commands, None)
        }
        OverviewAction::FocusWorkspace { name, monitor } => {
          state.overview.close();

          // Focusing the focused workspace would toggle back to the
          // previous one when `toggle_workspace_on_refocus` is set.
          if Some(&name) == focused_workspace.as_ref() {
            state.pending_sync.queue_focus_change();
            return platform_sync(state, config);
          }

          crate::overview::prepare_target(
            state, config, &name, monitor, true,
          )?;
          state.pending_sync.suppress_animations();
          (vec![focus_workspace_command(name)], None)
        }
        OverviewAction::MoveWindow {
          workspace, monitor, ..
        } => {
          let Some(window) = window else {
            return Ok(());
          };

          crate::overview::prepare_target(
            state, config, &workspace, monitor, false,
          )?;
          (
            vec![move_to_workspace_command(workspace)],
            Some(window.id()),
          )
        }
        OverviewAction::CloseWindow(_) => {
          let Some(window) = window else {
            return Ok(());
          };
          (vec![InvokeCommand::Close], Some(window.id()))
        }
        OverviewAction::Cancel => {
          state.overview.close();
          state.pending_sync.queue_focus_change();
          return platform_sync(state, config);
        }
        // Focus went elsewhere and the overview closed itself.
        OverviewAction::Deactivated => {
          state.overview.close();
          return platform_sync(state, config);
        }
        OverviewAction::Hidden => return Ok(()),
      };

      self
        .process_commands(&commands, subject, config)
        .map(|_| ())
    }
  }

  /// Carries out an action taken in the tab bar of stack `stack_id`.
  pub fn process_tab_action(
    &mut self,
    stack_id: Uuid,
    action: wm_platform::TabAction,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    use wm_common::VecDequeExt;
    use wm_platform::TabAction;

    use crate::models::StackContainer;

    let Some(stack) = self.state.container_by_id(stack_id) else {
      return Ok(());
    };

    let tab = |index| {
      stack
        .as_stack()
        .and_then(|stack| stack.windows().into_iter().nth(index))
    };

    let (command, subject) = match action {
      TabAction::Activate(index) => (
        InvokeCommand::FocusStackIndex { index },
        Some(stack.clone()),
      ),
      TabAction::Close(index) => {
        (InvokeCommand::Close, tab(index).map(Into::into))
      }
      TabAction::Detach(index) => {
        (InvokeCommand::ToggleStack, tab(index).map(Into::into))
      }
      TabAction::Float(index) => {
        let Some(window) = tab(index) else {
          return Ok(());
        };

        let state = &mut self.state;
        float_out_of_stack(&window, state, config)?;
        if !state.is_paused {
          platform_sync(state, config)?;
        }

        return Ok(());
      }
      TabAction::Cycle { prev } => (
        InvokeCommand::CycleStackFocus { prev },
        stack.as_stack().and_then(StackContainer::active_child),
      ),
      TabAction::Move { from, to } => {
        let Some(moved) = tab(from) else {
          return Ok(());
        };
        let moved: Container = moved.into();

        stack.borrow_children_mut().shift_to_index(to, moved);

        let state = &mut self.state;
        state.pending_sync.queue_container_to_redraw(stack);
        if !state.is_paused {
          platform_sync(state, config)?;
        }

        return Ok(());
      }
    };

    let Some(subject) = subject else {
      return Ok(());
    };

    self
      .process_commands(&vec![command], Some(subject.id()), config)
      .map(|_| ())
  }

  /// Places the windows held back for auto-stacking whose wait for a
  /// title ran out, and presses the `send_keys_on_join` keys that are due.
  pub fn process_auto_stack_timeouts(
    &mut self,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let state = &mut self.state;

    for native_window in
      state.auto_stack.expired(std::time::Instant::now())
    {
      manage_held_window(native_window, state, config)?;
    }

    #[cfg(target_os = "windows")]
    send_due_keys(state);

    if !state.is_paused && state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    self.state.animation_manager.ensure_timer_running();

    Ok(())
  }

  /// When [`process_closing_windows`](Self::process_closing_windows) is
  /// next due.
  // LINT: Close animations only exist on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(clippy::unused_self))]
  pub fn next_closing_check(&self) -> Option<std::time::Instant> {
    #[cfg(target_os = "windows")]
    return self.state.closing_windows.next_check();
    #[cfg(not(target_os = "windows"))]
    None
  }

  /// Follows up on windows closed behind a close animation: forgets the
  /// closed ones, and puts back the ones whose app kept them open.
  #[cfg(not(target_os = "windows"))]
  #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
  pub fn process_closing_windows(
    &mut self,
    _config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  /// Follows up on windows closed behind a close animation: forgets the
  /// closed ones, and puts back the ones whose app kept them open.
  #[cfg(target_os = "windows")]
  pub fn process_closing_windows(
    &mut self,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    use wm_platform::NativeWindow;

    use crate::{
      closing_windows::{CloseOutcome, ClosingAction, WindowProbe},
      commands::window::manage_window,
    };

    let state = &mut self.state;

    let actions =
      state
        .closing_windows
        .poll(std::time::Instant::now(), |handle| {
          let native = NativeWindow::from_handle(handle);
          WindowProbe {
            is_alive: native.is_valid(),
            is_shown: native.has_window_style(wm_platform::WS_VISIBLE),
            has_modal_prompt: native.has_modal_prompt(),
          }
        });

    for action in actions {
      match action {
        ClosingAction::SendClose { id, handle } => {
          tracing::warn!(
            "Close animation of {id} did not finish; closing it now."
          );

          // Dropped so that a late finish can't close the window again
          // after it has been put back.
          state.animation_manager.remove_animation(&id);
          state.backdrop_overlays.remove(&id);
          state.border_overlays.remove(&id);

          if let Err(err) = NativeWindow::from_handle(handle).close() {
            warn!("Failed to close window: {:?}", err);
          }
        }
        ClosingAction::Resolved {
          id,
          handle,
          outcome,
        } => {
          let native = NativeWindow::from_handle(handle);

          // Managed again in the meantime, so its cloak is the WM's to
          // decide as for any other window.
          if outcome == CloseOutcome::Closed
            || state.window_from_native(&native).is_some()
          {
            continue;
          }

          // Uncloaked even when hidden by the app (e.g. to its tray
          // icon), so it doesn't come back invisible when shown again.
          let _ = native.set_cloaked(false);

          if outcome == CloseOutcome::Refused {
            tracing::info!(
              "Window {id} stayed open after being closed; managing it \
               again."
            );

            // Logged rather than returned, so the remaining windows are
            // still handled.
            if let Err(err) = manage_window(native, None, state, config) {
              warn!("Failed to manage window {id} again: {err:?}");
            }
          }
        }
      }
    }

    if !state.is_paused && state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    self.state.animation_manager.ensure_timer_running();

    Ok(())
  }

  pub fn process_event(
    &mut self,
    event: PlatformEvent,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let state = &mut self.state;

    match event {
      PlatformEvent::DisplaySettingsChanged => {
        handle_display_settings_changed(state, config)
      }
      PlatformEvent::Keybinding(keybinding_event) => {
        // Find the keybinding config that matches this keybinding.
        let commands = config
          .active_keybinding_configs(
            &self.state.binding_modes,
            self.state.is_paused,
          )
          .find(|kb_config| {
            kb_config.bindings.contains(&keybinding_event.keybinding)
          })
          .filter(|kb_config| {
            is_off_cooldown(
              &mut self.state.keybinding_cooldowns,
              &keybinding_event.keybinding,
              kb_config.cooldown_ms,
              keybinding_event.time,
            )
          })
          .map(|kb_config| kb_config.commands.clone());

        if let Some(commands) = commands {
          self.process_commands(&commands, None, config)?;
        }

        // Return early since we don't want to redraw twice.
        return Ok(());
      }
      PlatformEvent::Mouse(event) => {
        handle_mouse_move(&event, state, config)
      }
      PlatformEvent::Window(window_event) => match window_event {
        WindowEvent::Focused { window, .. } => {
          handle_window_focused(&window, state, config)?;
          #[cfg(target_os = "windows")]
          crate::stay_interactive::keep_stacked_windows_interactive(
            &window, state, config,
          )?;
          Ok(())
        }
        WindowEvent::Shown { window, .. } => {
          #[cfg(target_os = "windows")]
          let shown = window.clone();

          handle_window_shown(window, state, config)?;
          #[cfg(target_os = "windows")]
          crate::stay_interactive::keep_stacked_windows_interactive(
            &shown, state, config,
          )?;
          Ok(())
        }
        WindowEvent::Hidden { window, .. } => {
          handle_window_hidden(&window, state, config)
        }
        WindowEvent::MovedOrResized {
          window,
          is_interactive_start,
          is_interactive_end,
          ..
        } => handle_window_moved_or_resized(
          &window,
          is_interactive_start,
          is_interactive_end,
          state,
          config,
        ),
        WindowEvent::Minimized { window, .. } => {
          handle_window_minimized(&window, state, config)
        }
        WindowEvent::MinimizeEnded { window, .. } => {
          handle_window_minimize_ended(&window, state, config)
        }
        WindowEvent::TitleChanged { window, .. } => {
          handle_window_title_changed(&window, state, config)
        }
        WindowEvent::AttentionRequested { window, .. } => {
          handle_window_attention_requested(&window, state)
        }
        WindowEvent::Reparented { window, .. } => {
          handle_window_reparented(window, state, config)
        }
        WindowEvent::ZOrderChanged { .. } => {
          handle_z_order_changed(state);
          Ok(())
        }
        WindowEvent::Destroyed { window_id, .. } => {
          handle_window_destroyed(window_id, state)
        }
      },
    }?;

    if !state.is_paused && state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    self.state.animation_manager.ensure_timer_running();

    Ok(())
  }

  /// Runs an animation frame in place of a timer thread that stopped
  /// ticking, and replaces that thread.
  pub fn recover_stalled_animations(
    &mut self,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    self.state.animation_manager.replace_stalled_timer();
    self.update_animations(config)
  }

  /// Updates all active animations and redraws windows that are animating.
  pub fn update_animations(
    &mut self,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    use crate::animation::AnimationManager;
    // Access animation_manager through state to avoid double borrow
    AnimationManager::update_internal(&mut self.state, config)
  }

  pub fn process_commands(
    &mut self,
    commands: &Vec<InvokeCommand>,
    subject_container_id: Option<Uuid>,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let state = &mut self.state;

    // Get the container to run WM commands with.
    let subject_container = match subject_container_id {
      Some(id) => state.container_by_id(id).with_context(|| {
        format!("No container found with the given ID '{id}'.")
      })?,
      None => state
        .focused_container()
        .context("No subject container for command.")?,
    };

    let new_subject_container_id = WindowManager::run_commands(
      commands,
      subject_container,
      state,
      config,
    )?;

    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    // Start animation timer if animations were created by a command (e.g.
    // startup commands or IPC commands). Without this, surrogate
    // animations started outside of the platform event loop would
    // never tick, leaving windows permanently cloaked.
    self.state.animation_manager.ensure_timer_running();

    Ok(new_subject_container_id)
  }

  pub fn run_commands(
    commands: &Vec<InvokeCommand>,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let mut current_subject_container = subject_container;

    for command in commands {
      WindowManager::run_command(
        command,
        current_subject_container.clone(),
        state,
        config,
      )?;

      // Update the subject container in case the container type changes.
      // For example, when going from a tiling to a floating window.
      current_subject_container =
        if current_subject_container.is_detached() {
          match state.container_by_id(current_subject_container.id()) {
            Some(container) => container,
            None => break,
          }
        } else {
          current_subject_container
        }
    }

    Ok(current_subject_container.id())
  }

  #[allow(clippy::too_many_lines)]
  pub fn run_command(
    command: &InvokeCommand,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    // No-op if WM is currently paused.
    if state.is_paused && *command != InvokeCommand::WmTogglePause {
      return Ok(());
    }

    if subject_container.is_detached() {
      bail!("Cannot run command because subject container is detached.");
    }

    match &command {
      InvokeCommand::AdjustBorders(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            let args = args.clone();
            let border_delta = RectDelta::new(
              args.left.unwrap_or(LengthValue::from_px(0)),
              args.top.unwrap_or(LengthValue::from_px(0)),
              args.right.unwrap_or(LengthValue::from_px(0)),
              args.bottom.unwrap_or(LengthValue::from_px(0)),
            );

            window.set_border_delta(border_delta);
            state.pending_sync.queue_container_to_redraw(window);

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Close => {
        match subject_container.as_window_container() {
          Ok(window) => {
            // A scene plays the close from a copy of the window, detached
            // from the layout so the rest reflows meanwhile, and sends it
            // `WM_CLOSE` once that's done.
            #[cfg(target_os = "windows")]
            if let Some(request) = crate::animation::scene::close_request(
              state, config, &window,
            ) {
              let close_config = &config.value.animations.window_close;
              detach_window_for_close(
                &window,
                std::time::Duration::from_millis(u64::from(
                  close_config
                    .easing
                    .duration_ms(close_config.duration_ms),
                )),
                state,
              )?;
              state.pending_sync.queue_scene_close(request);
              return Ok(());
            }

            // Without an animation, it closes at once.
            if let Err(err) = window.native().close() {
              warn!("Failed to close window: {:?}", err);
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Focus(args) => {
        if let Some(direction) = &args.direction {
          focus_in_direction(&subject_container, direction, state)?;
        }

        if let Some(target) = workspace_target!(&args) {
          focus_workspace(target, state, config)?;
        }

        if let Some(container_id) = &args.container_id {
          focus_container_by_id(container_id, state)?;
        }

        if let Some(monitor_index) = &args.monitor {
          focus_monitor(*monitor_index, state, config)?;
        }

        if args.urgent_window {
          focus_urgent_window(state, config)?;
        }

        if args.special_workspace {
          focus_special_workspace(state, config)?;
        }

        let column_target = if args.first_column {
          Some(ColumnTarget::First)
        } else if args.last_column {
          Some(ColumnTarget::Last)
        } else if args.next_column {
          Some(ColumnTarget::Next)
        } else if args.prev_column {
          Some(ColumnTarget::Previous)
        } else {
          args.column.map(ColumnTarget::Index)
        };

        if let Some(target) = column_target {
          focus_column(&subject_container, target, state);
        }

        let row_target = if args.column_top {
          Some(RowTarget::Top)
        } else if args.column_bottom {
          Some(RowTarget::Bottom)
        } else {
          args.window_in_column.map(RowTarget::Index)
        };

        if let Some(target) = row_target {
          focus_window_in_column(&subject_container, target, state);
        }

        Ok(())
      }
      // No-op at runtime. `force-manage` is a marker that is evaluated
      // before a window is managed (see `check_is_manageable`).
      InvokeCommand::ForceManage => Ok(()),
      InvokeCommand::Ignore => {
        match subject_container.as_window_container() {
          Ok(window) => ignore_window(window, state),
          _ => Ok(()),
        }
      }
      InvokeCommand::MoveCursor(args) => {
        if args.direction.is_some() {
          state.pending_sync.queue_cursor_move();
        }
        Ok(())
      }
      InvokeCommand::Move(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if let Some(direction) = &args.direction {
              move_window_in_direction(
                window.clone(),
                direction,
                state,
                config,
              )?;
            }

            if let Some(target) = workspace_target!(&args) {
              move_window_to_workspace(
                window.clone(),
                target,
                state,
                config,
              )?;
            }

            if args.special_workspace {
              move_window_to_special_workspace(
                window.clone(),
                state,
                config,
              )?;
            }

            let column_target = if args.first_column {
              Some(ColumnTarget::First)
            } else if args.last_column {
              Some(ColumnTarget::Last)
            } else {
              args.column.map(ColumnTarget::Index)
            };

            if let Some(target) = column_target {
              move_column(&window.into(), target, state)?;
            }
            Ok(())
          }

          _ => Ok(()),
        }
      }
      InvokeCommand::MoveWorkspace { direction } => {
        let workspace =
          subject_container.workspace().context("No workspace.")?;

        move_workspace_in_direction(&workspace, direction, state, config)
      }
      InvokeCommand::Position(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if args.centered {
              set_window_position(
                window,
                &WindowPositionTarget::Centered,
                state,
              )
            } else {
              set_window_position(
                window,
                &WindowPositionTarget::Coordinates(args.x_pos, args.y_pos),
                state,
              )
            }
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::UpdateWorkspaceConfig {
        workspace,
        new_config,
      } => {
        let workspace = if let Some(workspace_name) = workspace {
          state
            .workspace_by_name(workspace_name)
            .context("Workspace doesn't exist.")?
        } else {
          subject_container.workspace().context("No workspace.")?
        };
        update_workspace_config(&workspace, state, config, new_config)
      }
      InvokeCommand::Resize(args) => {
        match subject_container.as_window_container() {
          Ok(window) => resize_window(
            &window,
            args.width.clone(),
            args.height.clone(),
            state,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetFloating {
        centered,
        shown_on_top,
        x_pos,
        y_pos,
        width,
        height,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;
          let centered = centered.unwrap_or(floating_defaults.centered);

          let window = update_window_state(
            window.clone(),
            WindowState::Floating(FloatingStateConfig {
              centered,
              shown_on_top: shown_on_top
                .unwrap_or(floating_defaults.shown_on_top),
            }),
            state,
            config,
          )?;

          // Allow size and position to be set if window has not previously
          // been manually placed.
          if !window.has_custom_floating_placement() {
            if width.is_some() || height.is_some() {
              set_window_size(
                window.clone(),
                width.clone(),
                height.clone(),
                state,
              )?;
            }

            if centered {
              set_window_position(
                window,
                &WindowPositionTarget::Centered,
                state,
              )?;
            } else if x_pos.is_some() || y_pos.is_some() {
              set_window_position(
                window,
                &WindowPositionTarget::Coordinates(*x_pos, *y_pos),
                state,
              )?;
            }
          }

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::SetFullscreen {
        maximized,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_behavior.state_defaults.fullscreen;

          let window = update_window_state(
            window.clone(),
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              shown_on_top: shown_on_top
                .unwrap_or(fullscreen_defaults.shown_on_top),
            }),
            state,
            config,
          )?;

          // WM-initiated, so this transition is allowed to animate (see
          // `PendingSync::wm_fullscreen_toggles`).
          state.pending_sync.mark_wm_fullscreen_toggle(window.id());

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::SetMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              WindowState::Minimized,
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window,
              WindowState::Tiling,
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetUrgency { urgent } => {
        match subject_container.as_window_container() {
          Ok(window) => {
            set_window_urgency(&window, urgent.unwrap_or(true), state)?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTitleBarVisibility {
        // LINT: `visibility` is only used on Windows.
        #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
        visibility,
      } => match subject_container.as_window_container() {
        #[cfg(target_os = "windows")]
        Ok(window) => {
          _ = window.native().set_title_bar_visibility(
            *visibility == TitleBarVisibility::Shown,
          );
          Ok(())
        }
        _ => Ok(()),
      },
      // LINT: `args` is only used on Windows.
      #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
      InvokeCommand::SetTransparency(args) => {
        match subject_container.as_window_container() {
          #[cfg(target_os = "windows")]
          Ok(window) => {
            if let Some(opacity) = &args.opacity {
              _ = window.native().set_transparency(opacity);
            }

            if let Some(opacity_delta) = &args.opacity_delta {
              _ = window.native().adjust_transparency(opacity_delta);
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ShellExec {
        hide_window,
        command,
      } => shell_exec(&command.join(" "), *hide_window, state),
      InvokeCommand::Size(args) => {
        match subject_container.as_window_container() {
          Ok(window) => set_window_size(
            window,
            args.width.clone(),
            args.height.clone(),
            state,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleFloating {
        centered,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;

          let centered = centered.unwrap_or(floating_defaults.centered);
          let target_state = WindowState::Floating(FloatingStateConfig {
            centered,
            shown_on_top: shown_on_top
              .unwrap_or(floating_defaults.shown_on_top),
          });

          let window = update_window_state(
            window.clone(),
            window.toggled_state(target_state, config),
            state,
            config,
          )?;

          if !window.has_custom_floating_placement() && centered {
            set_window_position(
              window,
              &WindowPositionTarget::Centered,
              state,
            )?;
          }

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleFullscreen {
        maximized,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_behavior.state_defaults.fullscreen;

          let target_state =
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              shown_on_top: shown_on_top
                .unwrap_or(fullscreen_defaults.shown_on_top),
            });

          let window = update_window_state(
            window.clone(),
            window.toggled_state(target_state, config),
            state,
            config,
          )?;

          // WM-initiated, so this transition is allowed to animate (see
          // `PendingSync::wm_fullscreen_toggles`).
          state.pending_sync.mark_wm_fullscreen_toggle(window.id());

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              window.toggled_state(WindowState::Minimized, config),
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleOverview { layout } => {
        let layout = layout.unwrap_or(config.value.overview.layout);

        #[cfg(target_os = "windows")]
        crate::overview::toggle_overview(
          state,
          config,
          match layout {
            wm_common::OverviewLayout::Activities => {
              wm_platform::OverviewLayoutMode::Activities
            }
            wm_common::OverviewLayout::Stage => {
              wm_platform::OverviewLayoutMode::Stage
            }
            wm_common::OverviewLayout::Carousel => {
              wm_platform::OverviewLayoutMode::Carousel
            }
            wm_common::OverviewLayout::Grid => {
              wm_platform::OverviewLayoutMode::Grid
            }
          },
        )?;

        #[cfg(not(target_os = "windows"))]
        {
          let _ = layout;
          tracing::warn!("The overview is only available on Windows.");
        }

        Ok(())
      }
      InvokeCommand::ToggleTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              window.toggled_state(WindowState::Tiling, config),
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleStack => {
        if let Ok(window) = subject_container.as_window_container() {
          toggle_stack(&window, state, config)?;
          state.pending_sync.queue_focus_change();
        }
        Ok(())
      }
      InvokeCommand::CycleStackFocus { prev } => {
        cycle_stack_focus(&subject_container, *prev, state);
        state.pending_sync.queue_focus_change();
        Ok(())
      }
      InvokeCommand::FocusStackIndex { index } => {
        focus_stack_index(&subject_container, *index, state);
        state.pending_sync.queue_focus_change();
        Ok(())
      }
      InvokeCommand::StackAbsorbNeighbor { direction } => {
        if let Some(window) = subject_container.as_tiling_window().cloned()
        {
          stack_absorb_neighbor(&window, direction, state, config)?;
          state.pending_sync.queue_focus_change();
        }
        Ok(())
      }
      InvokeCommand::MoveToStack { name } => {
        if let Ok(window) = subject_container.as_window_container() {
          let window = move_to_stack(window, name, state, config)?;

          // Only re-assert focus for the focused window, so a rule run on
          // a background window doesn't steal focus.
          if window.has_focus(None) {
            state.pending_sync.queue_focus_change();
          }
        }
        Ok(())
      }
      InvokeCommand::StackAll => {
        if let Some(workspace) = subject_container.workspace() {
          stack_all(&workspace, state, config)?;
          state.pending_sync.queue_focus_change();
        }
        Ok(())
      }
      InvokeCommand::UnstackAll => {
        if let Some(workspace) = subject_container.workspace() {
          unstack_all(&workspace, state)?;
          state.pending_sync.queue_focus_change();
        }
        Ok(())
      }
      InvokeCommand::MoveStackTab { prev } => {
        move_stack_tab(&subject_container, *prev, state);
        Ok(())
      }
      InvokeCommand::FloatOutOfStack => {
        if let Ok(window) = subject_container.as_window_container() {
          float_out_of_stack(&window, state, config)?;
        }
        Ok(())
      }
      InvokeCommand::StayInteractive => {
        #[cfg(target_os = "windows")]
        if let Ok(window) = subject_container.as_window_container() {
          state.stay_interactive.mark(window.native().id());
        }
        Ok(())
      }
      InvokeCommand::ToggleTilingDirection => {
        toggle_tiling_direction(subject_container, state, config)
      }
      InvokeCommand::SetTilingDirection { tiling_direction } => {
        set_tiling_direction(
          subject_container,
          state,
          config,
          tiling_direction,
        )
      }
      InvokeCommand::SetLayout { layout } => {
        let workspace =
          subject_container.workspace().context("No workspace.")?;
        set_layout(&workspace, *layout, state)
      }
      InvokeCommand::CycleColumnWidth { back } => {
        cycle_column_width(&subject_container, !back, state, config)
      }
      InvokeCommand::CycleWindowHeight { back } => {
        cycle_window_height(&subject_container, !back, state, config)
      }
      InvokeCommand::ResetWindowHeight => {
        reset_window_height(&subject_container, state);
        Ok(())
      }
      InvokeCommand::MaximizeColumn => {
        maximize_column(&subject_container, state);
        Ok(())
      }
      InvokeCommand::ExpandColumnToAvailableWidth => {
        expand_column_to_available_width(&subject_container, state)
      }
      InvokeCommand::CenterColumn => {
        center_column(&subject_container, state);
        Ok(())
      }
      InvokeCommand::CenterVisibleColumns => {
        center_visible_columns(&subject_container, state)
      }
      InvokeCommand::ConsumeOrExpelWindow { direction } => {
        consume_or_expel_window(
          &subject_container,
          direction,
          state,
          config,
        )
      }
      InvokeCommand::ConsumeWindowIntoColumn => {
        consume_window_into_column(&subject_container, state, config)
      }
      InvokeCommand::ExpelWindowFromColumn => {
        expel_window_from_column(&subject_container, state)
      }
      InvokeCommand::SwapWindow { direction } => {
        swap_window(&subject_container, direction, state)
      }
      InvokeCommand::ToggleColumnTabbedDisplay => {
        set_column_display(&subject_container, None, state, config)
      }
      InvokeCommand::SetColumnDisplay { display } => set_column_display(
        &subject_container,
        Some(*display),
        state,
        config,
      ),
      InvokeCommand::WmCycleFocus {
        omit_floating,
        omit_fullscreen,
        omit_minimized,
        omit_tiling,
      } => cycle_focus(
        *omit_floating,
        *omit_fullscreen,
        *omit_minimized,
        *omit_tiling,
        state,
        config,
      ),
      InvokeCommand::WmDisableBindingMode { name } => {
        disable_binding_mode(name, state);
        Ok(())
      }
      InvokeCommand::WmEnableBindingMode { name } => {
        enable_binding_mode(name, state, config)
      }
      InvokeCommand::WmExit => state.emit_exit(),
      InvokeCommand::WmRedraw => {
        state
          .pending_sync
          .queue_container_to_redraw(state.root_container.clone());

        Ok(())
      }
      InvokeCommand::WmReloadConfig => reload_config(state, config),
      InvokeCommand::WmTogglePause => {
        toggle_pause(state);
        Ok(())
      }
    }
  }

  /// Runs cleanup tasks when the WM is exiting.
  pub(crate) fn cleanup(
    &mut self,
    config: &mut UserConfig,
    ipc_server: &mut IpcServer,
  ) {
    self.state.emit_event(WmEvent::ApplicationExiting);

    // Ensure that the WM is unpaused, otherwise, shutdown commands won't
    // get executed.
    self.state.is_paused = false;

    // Run user's shutdown commands.
    if let Err(err) = self.process_commands(
      &config.value.general.shutdown_commands.clone(),
      None,
      config,
    ) {
      tracing::warn!("Failed to run shutdown commands: {:?}", err);
    }

    // Emit remaining WM events before exiting.
    while let Ok(wm_event) = self.event_rx.try_recv() {
      tracing::info!(
        "Emitting WM event before shutting down: {:?}",
        wm_event
      );

      if let Err(err) = ipc_server.process_event(wm_event) {
        tracing::warn!("{:?}", err);
      }
    }
  }
}

/// The managed window an overview action refers to, if any.
#[cfg(target_os = "windows")]
fn window_by_handle(
  state: &WmState,
  action: &OverviewAction,
) -> Option<crate::models::WindowContainer> {
  let hwnd = match action {
    OverviewAction::FocusWindow { hwnd, .. }
    | OverviewAction::CloseWindow(hwnd)
    | OverviewAction::MoveWindow { hwnd, .. } => *hwnd,
    _ => return None,
  };

  state
    .windows()
    .into_iter()
    .find(|window| window.native().id().0 == hwnd)
}

/// Whether `keybinding` may trigger at `now`, starting its `cooldown_ms`
/// when it may.
///
/// `cooldowns` holds when each keybinding on cooldown comes off it.
fn is_off_cooldown(
  cooldowns: &mut HashMap<Keybinding, Instant>,
  keybinding: &Keybinding,
  cooldown_ms: Option<u32>,
  now: Instant,
) -> bool {
  cooldowns.retain(|_, ready_at| *ready_at > now);

  if cooldowns.contains_key(keybinding) {
    return false;
  }

  if let Some(cooldown_ms) = cooldown_ms {
    cooldowns.insert(
      keybinding.clone(),
      now + Duration::from_millis(u64::from(cooldown_ms)),
    );
  }

  true
}

/// `focus --workspace <name>`, for a workspace readied by
/// `prepare_target`; the special workspace too, which it has summoned.
#[cfg(target_os = "windows")]
fn focus_workspace_command(name: String) -> InvokeCommand {
  InvokeCommand::Focus(InvokeFocusCommand {
    workspace: Some(name),
    ..Default::default()
  })
}

/// `move --workspace <name>`, for a workspace readied by
/// `prepare_target`.
#[cfg(target_os = "windows")]
fn move_to_workspace_command(name: String) -> InvokeCommand {
  InvokeCommand::Move(InvokeMoveCommand {
    workspace: Some(name),
    ..Default::default()
  })
}

/// Presses the next due `send_keys_on_join` key combination in windows
/// that are in the foreground, retrying the others until they time out.
/// Once a window is done, the duplicate tabs it replaces are closed.
#[cfg(target_os = "windows")]
fn send_due_keys(state: &mut WmState) {
  let now = std::time::Instant::now();

  for mut pending in state.auto_stack.take_due_keys(now) {
    let Some(window) = state.window_from_native(&pending.native) else {
      close_duplicate_tabs(&pending.then_close);
      continue;
    };

    let Some(keybinding) = pending.keys.first() else {
      close_duplicate_tabs(&pending.then_close);
      continue;
    };

    let result = window.native().press_keys(keybinding.keys());

    match result {
      Ok(true) => {
        pending.keys.remove(0);

        if pending.keys.is_empty() {
          tracing::info!("Sent keys on join to window: {window}");
          close_duplicate_tabs(&pending.then_close);
        } else {
          state.auto_stack.retry_keys(pending, now);
        }
      }
      Ok(false) if pending.is_expired(now) => {
        tracing::info!(
          "Not sending keys on join to window, since it never got focus: \
           {window}"
        );
        close_duplicate_tabs(&pending.then_close);
      }
      Ok(false) => state.auto_stack.retry_keys(pending, now),
      Err(err) => {
        tracing::warn!("Failed to send keys on join to {window}: {err}");
        close_duplicate_tabs(&pending.then_close);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{
    collections::HashMap,
    time::{Duration, Instant},
  };

  use wm_platform::{Key, Keybinding};

  use super::is_off_cooldown;

  #[test]
  fn keybindings_cool_down_on_their_own() {
    let mut cooldowns = HashMap::new();
    let slow = Keybinding::new(vec![Key::Alt, Key::WheelDown]).unwrap();
    let fast = Keybinding::new(vec![Key::Alt, Key::WheelUp]).unwrap();
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);

    assert!(is_off_cooldown(&mut cooldowns, &slow, Some(500), at(0)));
    assert!(!is_off_cooldown(&mut cooldowns, &slow, Some(500), at(100)));

    // Another keybinding's shorter cooldown doesn't cut it short.
    assert!(is_off_cooldown(&mut cooldowns, &fast, Some(100), at(200)));
    assert!(is_off_cooldown(&mut cooldowns, &fast, Some(100), at(400)));
    assert!(!is_off_cooldown(&mut cooldowns, &slow, Some(500), at(400)));

    assert!(is_off_cooldown(&mut cooldowns, &slow, Some(500), at(500)));
    assert!(is_off_cooldown(&mut cooldowns, &slow, None, at(1000)));
    assert!(is_off_cooldown(&mut cooldowns, &slow, None, at(1000)));
  }
}
