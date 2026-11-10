// The `windows` or `console` subsystem (default is `console`) determines
// whether a console window is spawned on launch, if not already ran
// through a console. The following prevents this additional console window
// in release mode.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(iterator_try_collect)]

use std::{
  fs::{self, File, OpenOptions},
  path::{Path, PathBuf},
  process,
  sync::Mutex,
  time::{Duration, Instant},
};

use anyhow::Context;
use tokio::signal;
use tracing::Level;
use tracing_subscriber::{
  fmt::{self, writer::MakeWriterExt},
  layer::{Layer, SubscriberExt},
};
use wm_common::{AppCommand, InvokeCommand, Verbosity, WmEvent};
use wm_platform::{
  Dispatcher, DisplayListener, EventLoop, KeybindingListener,
  MouseEventKind, MouseListener, PlatformEvent, SingleInstance,
  WindowListener,
};

use crate::{
  error_notifier::ErrorNotifier,
  ipc_server::{IpcMessage, IpcServer},
  sys_tray::SystemTray,
  user_config::UserConfig,
  wm::WindowManager,
};

mod animation;
mod auto_stack;
mod closing_windows;
mod commands;
mod config_files;
mod drop_indicator;
mod error_notifier;
mod events;
mod ipc_server;
mod layout;
mod models;
mod overview;
mod particles;
mod pending_sync;
mod stay_interactive;
mod switch_log;
mod sys_tray;
mod tab_bars;
mod traits;
mod user_config;
mod window_overlays;
mod wm;
mod wm_state;

#[cfg(test)]
mod test_utils;

/// Main entry point for the application.
///
/// Conditionally starts the WM or runs a CLI command based on the given
/// subcommand.
fn main() -> anyhow::Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
  let app_command = AppCommand::parse_with_default(&args);

  if let AppCommand::Start {
    config_path,
    verbosity,
  } = app_command
  {
    // Before any window exists: our overlay windows live on a thread that
    // never handles posted messages, so Windows could otherwise substitute
    // hit-testable "Not Responding" ghosts for them and swallow desktop
    // mouse input. See `disable_window_ghosting`.
    wm_platform::disable_window_ghosting();

    let rt = tokio::runtime::Runtime::new()?;
    let (event_loop, dispatcher) = EventLoop::new()?;

    let task_handle = std::thread::spawn(move || {
      rt.block_on(async {
        let start_res =
          start_wm(config_path, verbosity, &dispatcher).await;

        if let Err(err) = &start_res {
          // If unable to start the WM, the error is fatal and a message
          // dialog is shown.
          tracing::error!("{:?}", err);
          dispatcher.show_error_dialog("Fatal error", &err.to_string());
        }

        if let Err(err) = dispatcher.stop_event_loop() {
          // Forcefully exit the process to ensure the event loop is
          // stopped.
          tracing::error!("Failed to stop event loop gracefully: {}", err);
          process::exit(1);
        }

        start_res
      })
    });

    // Run event loop (blocks until shutdown).
    event_loop.run()?;

    // Wait for clean exit of the WM.
    task_handle.join().unwrap()
  } else {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(wm_cli::start(args))
  }
}

#[allow(clippy::too_many_lines)]
async fn start_wm(
  config_path: Option<PathBuf>,
  verbosity: Verbosity,
  dispatcher: &Dispatcher,
) -> anyhow::Result<()> {
  setup_logging(&verbosity)?;

  // Left on by mistake, it would make animations look broken or differ
  // from the defaults.
  let isolation = wm_platform::isolation::current();
  if isolation != wm_platform::isolation::Isolation::default() {
    tracing::warn!(
      "GLAZEWM_ISOLATE changes how moving windows are drawn: \
       {isolation:?}."
    );
  }

  // Ensure that only one instance of the WM is running.
  let _single_instance = SingleInstance::new()?;

  // Parse and validate user config.
  let mut config = UserConfig::new(config_path)?;

  // Add application icon to system tray.
  let mut tray = SystemTray::new(&config.path, dispatcher.clone())?;

  let mut wm = WindowManager::new(&mut config, dispatcher.clone())?;

  let mut ipc_server = IpcServer::start().await?;

  // Start watcher process for restoring hidden windows on crash.
  if let Err(err) = start_watcher_process() {
    tracing::warn!(
      "Failed to start watcher process: {err}{}",
      cfg!(debug_assertions)
        .then_some(".\n Run `cargo build -p wm-watcher` to build it.")
        .unwrap_or_default()
    );
  }

  // Start listening for platform events after populating initial state.
  let mut listeners = Listeners {
    window: WindowListener::new(dispatcher)?,
    display: DisplayListener::new(dispatcher)?,
    mouse: MouseListener::new(mouse_event_kinds(&config), dispatcher)?,
    keybinding: KeybindingListener::new(
      &config
        .active_keybinding_configs(&[], false)
        .flat_map(|kb| kb.bindings)
        .collect::<Vec<_>>(),
      dispatcher,
    )?,
  };

  let error_notifier = ErrorNotifier::new(dispatcher.clone());

  // Run user's startup commands.
  if let Err(err) = wm.process_commands(
    &config.value.general.startup_commands.clone(),
    None,
    &mut config,
  ) {
    error_notifier.report(&err);
  }

  // Create an interval for periodically cleaning up invalid windows.
  let mut cleanup_interval = tokio::time::interval(Duration::from_secs(5));
  cleanup_interval
    .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

  // Wakes the loop so that `process_sent_messages` runs while idle.
  let mut sent_message_interval =
    tokio::time::interval(SENT_MESSAGE_INTERVAL);
  sent_message_interval
    .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

  loop {
    wm_platform::process_sent_messages();

    // Hand queued work the thread before the next animation frame. The
    // `biased` select below puts the animation tick above every other
    // branch, and a tick is ready again as soon as the previous frame
    // finishes, so during an animation those branches are never reached:
    // window events were measured waiting a median ~210ms and up to
    // ~577ms on an eight-window relayout, and IPC commands, tab clicks
    // and timers waited for the animation to end. The catch-up is bounded
    // per frame so a burst cannot starve the animation in turn.
    if let Err(err) = catch_up_before_frame(
      &mut wm,
      &mut config,
      &mut listeners,
      &mut ipc_server,
      &mut tray,
    ) {
      error_notifier.report(&err);
    }

    let auto_stack_deadline = wm.state.auto_stack.next_deadline();
    let closing_check = wm.next_closing_check();
    let animation_watchdog =
      wm.state.animation_manager.watchdog_deadline();
    let hot_corner = wm.state.overview.hot_corner_deadline();

    let res = tokio::select! {
      // biased: evaluated top-to-bottom when multiple futures are ready
      // simultaneously. Shutdown signals are checked first, animation ticks
      // second so that window/input events never delay mid-animation frames.
      biased;
      _ = signal::ctrl_c() => {
        tracing::info!("Received SIGINT signal.");
        break;
      },
      Some(()) = wm.exit_rx.recv() => {
        tracing::info!("Exiting through WM command.");
        break;
      },
      Some(()) = tray.exit_rx.recv() => {
        tracing::info!("Exiting through system tray.");
        break;
      },
      Some(()) = wm.animation_tick_rx.recv() => {
        // Drain any stale ticks that piled up while the previous frame was
        // processing, so each update_animations call covers the freshest state.
        while wm.animation_tick_rx.try_recv().is_ok() {}
        wm.update_animations(&config)
      },
      // Reached only when no tick is pending, i.e. the timer stopped
      // ticking while something is still animating.
      () = sleep_until(animation_watchdog) => {
        wm.recover_stalled_animations(&config)
      },
      Some(event) = listeners.mouse.next_event() => {
        tracing::debug!("Received mouse event: {:?}", event);
        wm.process_event(PlatformEvent::Mouse(event), &mut config)
      },
      Some(event) = listeners.window.next_event() => {
        tracing::debug!("Received window event: {:?}", event);
        wm.process_event(PlatformEvent::Window(event), &mut config)
      },
      Some(()) = listeners.display.next_event() => {
        tracing::debug!("Received display settings changed event.");
        wm.process_event(PlatformEvent::DisplaySettingsChanged, &mut config)
      },
      Some(event) = listeners.keybinding.next_event() => {
        tracing::debug!("Received keyboard event: {:?}", event);
        wm.process_event(PlatformEvent::Keybinding(event), &mut config)
      }
      Some((session, action)) = wm.overview_action_rx.recv() => {
        tracing::debug!("Received overview action: {:?}", action);
        wm.process_overview_action(session, action, &mut config)
      }
      _ = cleanup_interval.tick() => {
        if wm.state.is_paused {
          Ok(())
        } else {
          wm.state.cleanup_invalid_windows()
        }
      },
      Some(message) = ipc_server.message_rx.recv() => {
        handle_ipc_message(message, &mut ipc_server, &mut wm, &mut config);
        Ok(())
      },
      () = sleep_until(auto_stack_deadline) => {
        wm.process_auto_stack_timeouts(&mut config)
      },
      () = sleep_until(closing_check) => {
        wm.process_closing_windows(&mut config)
      },
      () = sleep_until(hot_corner) => wm.process_hot_corner(&config),
      _ = sent_message_interval.tick() => Ok(()),
      Some((stack_id, action)) = wm.tab_action_rx.recv() => {
        wm.process_tab_action(stack_id, action, &mut config)
      },
      Some(window) = wm.guarded_click_rx.recv() => {
        wm.process_guarded_click(&window, &config)
      },
      Some(wm_event) = wm.event_rx.recv() => {
        handle_wm_event(
          wm_event,
          &wm,
          &config,
          &mut listeners,
          &mut ipc_server,
        )
      },
      Some(()) = tray.config_reload_rx.recv() => {
        reload_config_from_tray(&mut wm, &mut config)
      },
    };

    if let Err(err) = res {
      error_notifier.report(&err);
    }
  }

  tracing::info!("Window manager shutting down.");
  wm.cleanup(&mut config, &mut ipc_server);

  Ok(())
}

/// Longest a message sent to one of the WM thread's windows waits to be
/// handled while the loop is idle. See
/// `wm_platform::process_sent_messages`.
///
/// Broadcasts reach windows one at a time, so a broadcaster can wait this
/// long for each overlay -- well under a second for a typical layout.
const SENT_MESSAGE_INTERVAL: Duration = Duration::from_millis(100);

/// Platform event listeners, bundled for the main loop's helpers.
struct Listeners {
  keybinding: KeybindingListener,
  mouse: MouseListener,
  window: WindowListener,
  display: DisplayListener,
}

/// Mouse events to listen for under `config`: moves only for focus
/// following the cursor, or a hot corner opening the overview.
fn mouse_event_kinds(config: &UserConfig) -> &'static [MouseEventKind] {
  if config.value.general.focus_follows_cursor
    || config.value.overview.hot_corner != wm_common::HotCorner::None
  {
    &[MouseEventKind::Move, MouseEventKind::LeftButtonUp]
  } else {
    &[MouseEventKind::LeftButtonUp]
  }
}

/// Runs an IPC message. Errors are answered to the client rather than
/// shown.
fn handle_ipc_message(
  (message, response_tx, disconnection_tx): IpcMessage,
  ipc_server: &mut IpcServer,
  wm: &mut WindowManager,
  config: &mut UserConfig,
) {
  tracing::info!("Received IPC message: {:?}", message);
  wm_platform::perf::mark_input();

  if let Err(err) = ipc_server.process_message(
    message,
    &response_tx,
    &disconnection_tx,
    wm,
    config,
  ) {
    tracing::error!("{:?}", err);
  }
}

/// Forwards a WM event to IPC subscribers, and updates the listeners when
/// it changes what they listen for.
fn handle_wm_event(
  wm_event: WmEvent,
  wm: &WindowManager,
  config: &UserConfig,
  listeners: &mut Listeners,
  ipc_server: &mut IpcServer,
) -> anyhow::Result<()> {
  tracing::debug!("Received WM event: {:?}", wm_event);

  // Disable mouse listener when the WM is paused.
  if let WmEvent::PauseChanged { is_paused } = wm_event {
    let _ = listeners.mouse.enable(!is_paused);
  }

  // Update keybinding and mouse listeners on config changes. While
  // paused, only the bindings that unpause are taken from apps.
  if matches!(
    wm_event,
    WmEvent::UserConfigChanged { .. }
      | WmEvent::BindingModesChanged { .. }
      | WmEvent::PauseChanged { .. }
  ) {
    listeners.keybinding.update(
      &config
        .active_keybinding_configs(
          &wm.state.binding_modes,
          wm.state.is_paused,
        )
        .flat_map(|kb| kb.bindings)
        .collect::<Vec<_>>(),
    );

    listeners
      .mouse
      .set_enabled_events(mouse_event_kinds(config))?;
  }

  if let Err(err) = ipc_server.process_event(wm_event) {
    tracing::error!("{:?}", err);
  }

  Ok(())
}

fn reload_config_from_tray(
  wm: &mut WindowManager,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  wm.process_commands(&vec![InvokeCommand::WmReloadConfig], None, config)
    .map(|_| ())
}

/// Maximum items serviced ahead of one animation frame by
/// [`catch_up_before_frame`].
///
/// Bounds the inversion this creates: without a cap, an application
/// spamming location-change events could keep the catch-up busy and starve
/// the animation tick, turning an input-latency fix into dropped frames.
/// Eight is comfortably above the ~1.5 events per frame observed on an
/// eight-window relayout, so in practice the queues empty first.
const MAX_CATCH_UP_ITEMS: usize = 8;

/// Time after which [`catch_up_before_frame`] leaves the rest to later
/// frames, since a single command can take far longer than an event.
const CATCH_UP_BUDGET: Duration = Duration::from_millis(4);

/// Services already-queued work before the next animation frame, up to
/// [`MAX_CATCH_UP_ITEMS`] items or [`CATCH_UP_BUDGET`].
///
/// Returns as soon as every eligible queue is empty, so a quiet loop
/// iteration costs a handful of non-blocking channel polls.
///
/// What a person is waiting on comes first: keybindings, IPC commands,
/// and tab bar and overview clicks. These run the same commands a
/// keybinding does, so they are safe at any time.
///
/// # Why the workspace-switch gate
///
/// Mouse, window and display handlers mutate layout state -- display
/// states, workspace membership, floating placement, unmanagement -- and
/// so do due auto-stack and closing-window timers, which manage windows.
/// Resize and move animations are safe: `handle_window_moved_or_resized`
/// bails out early for any window holding a `ResizeSession`, so those
/// events cost a rect query and nothing more. A workspace-switch slide has
/// no such guard on every path, and delivering these events mid-slide
/// broke the animation and left workspaces reporting no windows.
///
/// So the gate is narrow on purpose: during a slide (and its few-tick
/// cleanup) these wait for the `select!`. A slide is brief, so the
/// responsiveness win during resizes -- which is where the queue actually
/// backs up, ~195 events per burst -- is kept intact.
fn catch_up_before_frame(
  wm: &mut WindowManager,
  config: &mut UserConfig,
  listeners: &mut Listeners,
  ipc_server: &mut IpcServer,
  tray: &mut SystemTray,
) -> anyhow::Result<()> {
  let started = Instant::now();

  for _ in 0..MAX_CATCH_UP_ITEMS {
    if started.elapsed() >= CATCH_UP_BUDGET {
      break;
    }

    if let Some(event) = listeners.keybinding.try_next_event() {
      tracing::debug!("Received keyboard event ahead of tick: {event:?}");
      wm.process_event(PlatformEvent::Keybinding(event), config)?;
      continue;
    }

    if let Ok(message) = ipc_server.message_rx.try_recv() {
      handle_ipc_message(message, ipc_server, wm, config);
      continue;
    }

    if let Ok((stack_id, action)) = wm.tab_action_rx.try_recv() {
      wm.process_tab_action(stack_id, action, config)?;
      continue;
    }

    if let Ok(window) = wm.guarded_click_rx.try_recv() {
      wm.process_guarded_click(&window, config)?;
      continue;
    }

    if let Ok((session, action)) = wm.overview_action_rx.try_recv() {
      wm.process_overview_action(session, action, config)?;
      continue;
    }

    if tray.config_reload_rx.try_recv().is_ok() {
      reload_config_from_tray(wm, config)?;
      continue;
    }

    if let Ok(wm_event) = wm.event_rx.try_recv() {
      handle_wm_event(wm_event, wm, config, listeners, ipc_server)?;
      continue;
    }

    // Re-checked every iteration: handling an item above can start or end
    // a slide, and the next one must be judged against that.
    if wm.state.animation_manager.is_workspace_switch_active() {
      break;
    }

    let now = Instant::now();

    if wm
      .state
      .auto_stack
      .next_deadline()
      .is_some_and(|at| at <= now)
    {
      wm.process_auto_stack_timeouts(config)?;
      continue;
    }

    if wm.next_closing_check().is_some_and(|at| at <= now) {
      wm.process_closing_windows(config)?;
      continue;
    }

    let event = listeners
      .mouse
      .try_next_event()
      .map(PlatformEvent::Mouse)
      .or_else(|| {
        listeners.window.try_next_event().map(PlatformEvent::Window)
      })
      .or_else(|| {
        listeners
          .display
          .try_next_event()
          .map(|()| PlatformEvent::DisplaySettingsChanged)
      });

    let Some(event) = event else {
      break;
    };

    tracing::debug!("Received platform event ahead of tick: {:?}", event);
    wm.process_event(event, config)?;
  }

  Ok(())
}

/// Initialize logging with the specified verbosity level.
///
/// Error and warning logs are saved to `~/.glzr/glazewm/errors.log`, the
/// overview's trace of picks to `~/.glzr/glazewm/overview.log`, and with
/// `--verbose` every log to `~/.glzr/glazewm/debug.log`.
///
/// The latter is the only way to read verbose logs of a release build:
/// `glazewm start` launches the WM detached, and the release WM has no
/// console, so its stdout can't be redirected.
/// `WARN` is included (not just `ERROR`) so perf-diagnostic warnings (e.g.
/// slow synchronous window repositions) are captured even when the WM is
/// running detached from a terminal, without needing `-v` for the full
/// `DEBUG` firehose.
fn setup_logging(verbosity: &Verbosity) -> anyhow::Result<()> {
  let error_log_dir = std::env::home_dir()
    .context("Unable to get home directory.")?
    .join(".glzr/glazewm/");

  let error_writer = open_log_file(&error_log_dir, "errors.log")?;

  // The frame profiler reports at `INFO`, which otherwise only reaches
  // stdout -- and the release build is a `windows` subsystem binary, so a
  // detached WM has nowhere to write it. Give it its own file when (and
  // only when) profiling is enabled, filtered to the profiler's target so
  // the file stays a clean run of frame reports rather than an `INFO`
  // firehose, and errors.log stays reserved for actual problems.
  let perf_layer = wm_platform::perf::is_enabled()
    .then(|| open_log_file(&error_log_dir, "perf.log"))
    .transpose()?
    .map(|writer| {
      fmt::Layer::new().with_writer(writer).with_filter(
        tracing_subscriber::filter::Targets::new()
          .with_target(wm_platform::perf::LOG_TARGET, Level::INFO),
      )
    });

  // A few lines per pick, so always on: kept apart from the rest, as the
  // WM may be started without `--verbose`.
  let overview_layer = fmt::Layer::new()
    .with_ansi(false)
    .with_writer(open_log_file(&error_log_dir, "overview.log")?)
    .with_filter(
      tracing_subscriber::filter::Targets::new()
        .with_target(wm_platform::OVERVIEW_LOG_TARGET, Level::DEBUG),
    );

  // One line per workspace switch, so always on, in its own file.
  let switch_layer = fmt::Layer::new()
    .with_ansi(false)
    .with_writer(open_log_file(&error_log_dir, "switch.log")?)
    .with_filter(
      tracing_subscriber::filter::Targets::new()
        .with_target(switch_log::SWITCH_LOG_TARGET, Level::INFO),
    );

  let debug_layer = (verbosity.level() == Level::DEBUG)
    .then(|| open_log_file(&error_log_dir, "debug.log"))
    .transpose()?
    .map(|writer| {
      fmt::Layer::new()
        .with_ansi(false)
        .with_writer(writer.with_max_level(Level::DEBUG))
    });

  let subscriber = tracing_subscriber::registry()
    .with(
      // Output to stdout with specified verbosity level.
      fmt::Layer::new()
        .with_writer(std::io::stdout.with_max_level(verbosity.level())),
    )
    .with(
      // Output to error log file.
      fmt::Layer::new()
        .with_writer(error_writer.with_max_level(Level::WARN)),
    )
    .with(perf_layer)
    .with(overview_layer)
    .with(switch_layer)
    .with(debug_layer);

  tracing::subscriber::set_global_default(subscriber)?;

  tracing::info!(
    "Starting WM with log level {:?}.",
    verbosity.level().to_string()
  );

  Ok(())
}

/// Opens `name` in `dir` for appending, creating either if missing.
fn open_log_file(dir: &Path, name: &str) -> anyhow::Result<Mutex<File>> {
  fs::create_dir_all(dir).with_context(|| {
    format!("Unable to create log directory {}.", dir.display())
  })?;

  let path = dir.join(name);
  let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&path)
    .with_context(|| {
      format!("Unable to open log file {}.", path.display())
    })?;

  Ok(Mutex::new(file))
}

/// Launches watcher binary. This is a separate process that
/// is responsible for restoring hidden windows in case the main WM process
/// crashes.
///
/// This assumes the watcher binary exists in the same directory as the
/// WM binary.
fn start_watcher_process() -> anyhow::Result<tokio::process::Child> {
  let watcher_path = std::env::current_exe()?
    .parent()
    .context("Failed to resolve path to the watcher process.")?
    .join("glazewm-watcher");

  tokio::process::Command::new(&watcher_path)
    .spawn()
    .context("Failed to start watcher process.")
}

/// Sleeps until `deadline`, or forever when there is none.
async fn sleep_until(deadline: Option<std::time::Instant>) {
  match deadline {
    Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
    None => std::future::pending().await,
  }
}
