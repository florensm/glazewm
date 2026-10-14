use std::{collections::HashMap, time::Instant};

use anyhow::Context;
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;
use wm_common::{BindingModeConfig, HideCorner, WindowState, WmEvent};
use wm_platform::{
  Direction, Dispatcher, Display, NativeWindow, OverviewAction, Point,
  Rect, TabAction,
};
#[cfg(target_os = "windows")]
use wm_platform::{
  NativeBackdropOverlay, NativeBorderOverlay, NativeDropIndicator,
  NativeStackTabBar, NativeWindowWindowsExt, OpacityValue,
};

#[cfg(target_os = "windows")]
use crate::overview::Overview;
use crate::{
  animation::AnimationManager,
  auto_stack::AutoStackState,
  commands::{
    container::set_focused_descendant,
    general::platform_sync,
    monitor::{add_monitor, move_bounded_workspaces_to_new_monitor},
    window::{manage_window, unmanage_window},
  },
  models::{
    Container, Monitor, NativeMonitorProperties, RootContainer,
    WindowContainer, Workspace, WorkspaceTarget,
  },
  pending_sync::PendingSync,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
};

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  pub root_container: RootContainer,

  pub dispatcher: Dispatcher,

  pub pending_sync: PendingSync,

  /// Manager for window animations.
  pub animation_manager: AnimationManager,

  /// Tracks the target position for each window to prevent animation
  /// restart loops.
  pub window_target_positions: HashMap<Uuid, Rect>,

  /// Name of the most recently focused workspace.
  ///
  /// Used for the `general.toggle_workspace_on_refocus` option on
  /// workspace focus.
  pub recent_workspace_name: Option<String>,

  /// The previously focused window that had focus effects applied.
  ///
  /// Used to efficiently update window effects by only removing focus
  /// effects from the previous window rather than all windows when focus
  /// changes.
  pub prev_effects_window: Option<WindowContainer>,

  /// Time since a previously focused window was unmanaged or minimized.
  ///
  /// Used to decide whether to override incoming focus events.
  pub unmanaged_or_minimized_timestamp: Option<Instant>,

  /// Configs of currently enabled binding modes.
  pub binding_modes: Vec<BindingModeConfig>,

  /// Windows that the WM should ignore. Windows can be added via the
  /// `ignore` command.
  pub ignored_windows: Vec<NativeWindow>,

  /// Windows held back or placed by `stack.auto_stack` rules.
  pub auto_stack: AutoStackState,

  /// Windows kept usable while their app shows a blocking popup.
  #[cfg(target_os = "windows")]
  pub stay_interactive: crate::stay_interactive::StayInteractive,

  /// Whether the WM is paused.
  pub is_paused: bool,

  /// Whether the OS focused window is the same as the WM focused window.
  pub is_focus_synced: bool,

  /// Backdrop overlay windows keyed by managed-window UUID.
  ///
  /// Each overlay is a `WS_POPUP` window rooting a
  /// `Windows.UI.Composition` visual tree, positioned directly behind its
  /// managed window in z-order and flush with that window's DWM frame
  /// rect. It renders a crop of the pre-blurred desktop wallpaper, so a
  /// window left semi-transparent by the `transparency` effect reads as
  /// frosted glass over it.
  #[cfg(target_os = "windows")]
  pub backdrop_overlays: HashMap<Uuid, NativeBackdropOverlay>,

  /// Border overlay windows keyed by managed-window UUID -- a persistent,
  /// self-drawn stand-in for the OS's `DWMWA_BORDER_COLOR`, which isn't
  /// carried along by DWM thumbnails and so vanishes during transitions.
  /// See `NativeBorderOverlay`'s doc comment.
  #[cfg(target_os = "windows")]
  pub border_overlays: HashMap<Uuid, NativeBorderOverlay>,

  /// Highlight of where a dragged window will land, created on the first
  /// drag that has somewhere to land.
  #[cfg(target_os = "windows")]
  pub drop_indicator: Option<NativeDropIndicator>,

  /// Overview of the focused workspace's windows.
  #[cfg(target_os = "windows")]
  pub overview: Overview,

  /// Whether the initial state has been populated.
  has_initialized: bool,

  /// Sender for emitting WM-related events.
  event_tx: mpsc::UnboundedSender<WmEvent>,

  /// Sender for gracefully shutting down the WM.
  exit_tx: mpsc::UnboundedSender<()>,

  /// Sender for actions taken in tab bars, keyed by stack ID.
  pub tab_action_tx: mpsc::UnboundedSender<(Uuid, TabAction)>,

  /// Live tab bar windows keyed by their `StackContainer` ID.
  #[cfg(target_os = "windows")]
  pub tab_bars: HashMap<Uuid, NativeStackTabBar>,

  /// Tab bar settings resolved from the config, cleared on reload.
  #[cfg(target_os = "windows")]
  pub tab_bar_settings: Option<crate::tab_bars::TabBarSettings>,
}

impl WmState {
  pub fn new(
    dispatcher: Dispatcher,
    event_tx: mpsc::UnboundedSender<WmEvent>,
    exit_tx: mpsc::UnboundedSender<()>,
    animation_tick_tx: mpsc::UnboundedSender<()>,
    overview_action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
    tab_action_tx: mpsc::UnboundedSender<(Uuid, TabAction)>,
  ) -> Self {
    // Without an overview, its actions channel is closed right away.
    #[cfg(not(target_os = "windows"))]
    drop(overview_action_tx);

    Self {
      root_container: RootContainer::new(),
      dispatcher,
      pending_sync: PendingSync::default(),
      animation_manager: AnimationManager::new(animation_tick_tx),
      window_target_positions: HashMap::new(),
      #[cfg(target_os = "windows")]
      backdrop_overlays: HashMap::new(),
      #[cfg(target_os = "windows")]
      border_overlays: HashMap::new(),
      #[cfg(target_os = "windows")]
      drop_indicator: None,
      #[cfg(target_os = "windows")]
      overview: Overview::new(overview_action_tx),
      prev_effects_window: None,
      recent_workspace_name: None,
      unmanaged_or_minimized_timestamp: None,
      binding_modes: Vec::new(),
      ignored_windows: Vec::new(),
      auto_stack: AutoStackState::default(),
      #[cfg(target_os = "windows")]
      stay_interactive: crate::stay_interactive::StayInteractive::default(
      ),
      is_paused: false,
      is_focus_synced: false,
      has_initialized: false,
      event_tx,
      exit_tx,
      tab_action_tx,
      #[cfg(target_os = "windows")]
      tab_bars: HashMap::new(),
      #[cfg(target_os = "windows")]
      tab_bar_settings: None,
    }
  }

  /// Populates the initial WM state by creating containers for all
  /// existing windows and monitors.
  pub fn populate(
    &mut self,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    // Get the originally focused window when the WM was started.
    let focused_window = self.dispatcher.focused_window().ok();

    // Create a monitor, and consequently a workspace, for each detected
    // native monitor.
    for native_display in self.dispatcher.sorted_displays()? {
      if let Ok(native_properties) =
        NativeMonitorProperties::try_from(&native_display)
      {
        let monitor =
          add_monitor(native_display, native_properties, self)?;
        move_bounded_workspaces_to_new_monitor(&monitor, self, config)?;
      }
    }

    // Manage windows in reverse z-order (bottom to top). This helps to
    // preserve the original stacking order.
    for native_window in
      self.dispatcher.visible_windows()?.into_iter().rev()
    {
      let nearest_workspace = self
        .nearest_monitor(&native_window)
        .and_then(|m| m.displayed_workspace());

      if let Some(workspace) = nearest_workspace {
        manage_window(
          native_window,
          Some(workspace.into()),
          self,
          config,
        )?;
      }
    }

    let container_to_focus = focused_window
      .and_then(|focused_window| {
        self.window_from_native(&focused_window).map(Into::into)
      })
      .or_else(|| self.windows().pop().map(Into::into))
      .or_else(|| self.workspaces().pop().map(Into::into))
      .context("Failed to get container to focus.")?;

    set_focused_descendant(&container_to_focus, None);
    self.is_focus_synced = true;

    self
      .pending_sync
      .queue_focus_change()
      .queue_all_effects_update();

    for workspace in self.workspaces() {
      self.pending_sync.queue_workspace_to_reorder(workspace);
    }

    platform_sync(self, config)?;
    self.has_initialized = true;

    Ok(())
  }

  pub fn monitors(&self) -> Vec<Monitor> {
    self.root_container.monitors()
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .monitors()
      .iter()
      .flat_map(Monitor::workspaces)
      .collect()
  }

  /// Gets workspaces sorted by their position in the user config.
  pub fn sorted_workspaces(&self, config: &UserConfig) -> Vec<Workspace> {
    let mut workspaces = self.workspaces();
    config.sort_workspaces(&mut workspaces);
    workspaces
  }

  pub fn windows(&self) -> Vec<WindowContainer> {
    self
      .root_container
      .descendants()
      .filter_map(|container| container.try_into().ok())
      .collect()
  }

  /// Gets the monitor that encompasses the largest portion of a given
  /// window.
  ///
  /// Defaults to the first monitor if the nearest monitor is invalid.
  pub fn nearest_monitor(
    &self,
    native_window: &NativeWindow,
  ) -> Option<Monitor> {
    self
      .monitor_from_native(
        &self.dispatcher.nearest_display(native_window).ok()?,
      )
      .or(self.monitors().first().cloned())
  }

  /// Gets monitor that corresponds to the given `Display`.
  pub fn monitor_from_native(
    &self,
    native_display: &Display,
  ) -> Option<Monitor> {
    self
      .monitors()
      .into_iter()
      .find(|monitor| monitor.native() == *native_display)
  }

  /// Gets the closest monitor in a given direction.
  ///
  /// Uses i3wm's algorithm for finding best guess.
  pub fn monitor_in_direction(
    &self,
    origin_monitor: &Monitor,
    direction: &Direction,
  ) -> anyhow::Result<Option<Monitor>> {
    let origin_rect = origin_monitor.native_properties().bounds;

    // Create a tuple of monitors and their rect.
    let monitors_with_rect = self
      .monitors()
      .into_iter()
      .map(|monitor| {
        let rect = monitor.native_properties().bounds;
        anyhow::Ok((monitor, rect))
      })
      .try_collect::<Vec<_>>()?;

    let closest_monitor = monitors_with_rect
      .into_iter()
      .filter(|(_, rect)| match direction {
        Direction::Right => {
          rect.x() > origin_rect.x() && rect.y_overlap(&origin_rect) > 0
        }
        Direction::Left => {
          rect.x() < origin_rect.x() && rect.y_overlap(&origin_rect) > 0
        }
        Direction::Down => {
          rect.y() > origin_rect.y() && rect.x_overlap(&origin_rect) > 0
        }
        Direction::Up => {
          rect.y() < origin_rect.y() && rect.x_overlap(&origin_rect) > 0
        }
      })
      .min_by(|(_, rect_a), (_, rect_b)| match direction {
        Direction::Right => rect_a.x().cmp(&rect_b.x()),
        Direction::Left => rect_b.x().cmp(&rect_a.x()),
        Direction::Down => rect_a.y().cmp(&rect_b.y()),
        Direction::Up => rect_b.y().cmp(&rect_a.y()),
      })
      .map(|(monitor, _)| monitor);

    Ok(closest_monitor)
  }

  /// Determines the preferred hide corner for each monitor. Used for
  /// [`HideMethod::PlaceInCorner`].
  ///
  /// The corner is chosen by simulating a 400x400 window frame in the
  /// bottom-left and bottom-right of the monitor's working area, then
  /// picking the side that overlaps the least with other monitors'
  /// working areas (ties favor bottom-right).
  pub fn monitors_by_hide_corner(&self) -> Vec<(Monitor, HideCorner)> {
    const TEST_FRAME_SIZE: i32 = 400;
    const VISIBLE_SLIVER: i32 = 1;

    let monitors = self.monitors();
    let working_areas = monitors
      .iter()
      .map(|monitor| monitor.native_properties().working_area)
      .collect::<Vec<_>>();

    monitors
      .into_iter()
      .enumerate()
      .map(|(idx, monitor)| {
        let monitor_rect = &working_areas[idx];
        let test_frame_y = monitor_rect.bottom - TEST_FRAME_SIZE;

        let left_test_frame = Rect::from_xy(
          monitor_rect.left - TEST_FRAME_SIZE + VISIBLE_SLIVER,
          test_frame_y,
          TEST_FRAME_SIZE,
          TEST_FRAME_SIZE,
        );

        let right_test_frame = Rect::from_xy(
          monitor_rect.right - VISIBLE_SLIVER,
          test_frame_y,
          TEST_FRAME_SIZE,
          TEST_FRAME_SIZE,
        );

        let overlap_area = |test_frame: &Rect| -> i32 {
          working_areas
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, rect)| test_frame.intersection_area(rect))
            .sum()
        };

        let left_overlap = overlap_area(&left_test_frame);
        let right_overlap = overlap_area(&right_test_frame);

        let corner = if left_overlap < right_overlap {
          HideCorner::BottomLeft
        } else {
          HideCorner::BottomRight
        };

        (monitor, corner)
      })
      .collect()
  }

  /// Gets window that corresponds to the given `NativeWindow`.
  pub fn window_from_native(
    &self,
    native_window: &NativeWindow,
  ) -> Option<WindowContainer> {
    self
      .windows()
      .into_iter()
      .find(|window| &*window.native() == native_window)
  }

  pub fn workspace_by_name(
    &self,
    workspace_name: &str,
  ) -> Option<Workspace> {
    self
      .workspaces()
      .into_iter()
      .find(|workspace| workspace.config().name == workspace_name)
  }

  /// Gets a workspace and its name by the given target.
  ///
  /// Returns a tuple of the workspace name and the `Workspace` instance
  /// if active.
  #[allow(clippy::too_many_lines)]
  pub fn workspace_by_target(
    &self,
    origin_workspace: &Workspace,
    target: WorkspaceTarget,
    config: &UserConfig,
  ) -> anyhow::Result<(Option<String>, Option<Workspace>)> {
    let (name, workspace) = match target {
      WorkspaceTarget::Name(name) => {
        #[allow(clippy::match_bool)]
        match origin_workspace.config().name == name {
          false => (Some(name.clone()), self.workspace_by_name(&name)),
          // Toggle the workspace if it's already focused.
          true if config.value.general.toggle_workspace_on_refocus => (
            self.recent_workspace_name.clone(),
            self
              .recent_workspace_name
              .as_ref()
              .and_then(|name| self.workspace_by_name(name)),
          ),
          true => (None, None),
        }
      }
      WorkspaceTarget::Recent => (
        self.recent_workspace_name.clone(),
        self
          .recent_workspace_name
          .as_ref()
          .and_then(|name| self.workspace_by_name(name)),
      ),
      WorkspaceTarget::NextActive | WorkspaceTarget::PreviousActive => {
        let workspaces = self.sorted_workspaces(config);
        neighbour_workspace(
          origin_workspace,
          workspaces,
          matches!(target, WorkspaceTarget::NextActive),
        )?
      }
      WorkspaceTarget::NextActiveInMonitor
      | WorkspaceTarget::PreviousActiveInMonitor => {
        let monitor = origin_workspace
          .monitor()
          .context("No monitor in workspace")?;

        let mut workspaces = monitor.workspaces();
        config.sort_workspaces(&mut workspaces);

        neighbour_workspace(
          origin_workspace,
          workspaces,
          matches!(target, WorkspaceTarget::NextActiveInMonitor),
        )?
      }
      WorkspaceTarget::Next | WorkspaceTarget::Previous => {
        let workspace_names =
          config.ordered_workspace_names(&self.workspaces());
        let origin_name = origin_workspace.config().name;
        let origin_index = cycle_origin(
          origin_workspace,
          workspace_names.iter().position(|name| *name == origin_name),
        )?;

        let name = cycle_neighbour(
          &workspace_names,
          origin_index,
          matches!(target, WorkspaceTarget::Next),
        )
        .cloned();
        let workspace =
          name.as_ref().and_then(|name| self.workspace_by_name(name));

        (name, workspace)
      }
      WorkspaceTarget::NextEmpty => {
        let workspaces = self.workspaces();
        let origin_monitor_id =
          origin_workspace.monitor().map(|monitor| monitor.id());

        // The origin is deliberately included; an already-empty origin
        // makes this a no-op instead of bouncing between two empty
        // workspaces on every invocation.
        let empty_workspaces = self
          .sorted_workspaces(config)
          .into_iter()
          .filter(|workspace| {
            !workspace.has_children() && !workspace.is_special()
          })
          .collect::<Vec<_>>();

        // Prefer an empty workspace on the origin's monitor, then a
        // workspace config that isn't in use (which activates on the
        // origin's monitor), then an empty workspace elsewhere, and
        // finally a new dynamic workspace.
        let empty_workspace = empty_workspaces.iter().find(|workspace| {
          workspace.monitor().map(|monitor| monitor.id())
            == origin_monitor_id
        });

        match empty_workspace {
          Some(workspace) => {
            (Some(workspace.config().name), Some(workspace.clone()))
          }
          None => match config.next_inactive_workspace_config(&workspaces)
          {
            Some(workspace_config) => {
              (Some(workspace_config.name.clone()), None)
            }
            None => match empty_workspaces.first() {
              Some(workspace) => {
                (Some(workspace.config().name), Some(workspace.clone()))
              }
              None => (
                config.value.general.dynamic_workspaces.then(|| {
                  config.next_dynamic_workspace_name(&workspaces)
                }),
                None,
              ),
            },
          },
        }
      }

      WorkspaceTarget::Direction(direction) => {
        let origin_monitor =
          origin_workspace.monitor().context("No focused monitor.")?;

        let target_workspace = self
          .monitor_in_direction(&origin_monitor, &direction)?
          .and_then(|monitor| monitor.displayed_workspace());

        (
          target_workspace
            .as_ref()
            .map(|workspace| workspace.config().name),
          target_workspace,
        )
      }
    };

    Ok((name, workspace))
  }

  /// Name of the workspace to go back to from the special workspace on
  /// `monitor`: the monitor's most recently focused other workspace, else
  /// the most recently focused workspace anywhere.
  pub fn workspace_before_special(
    &self,
    monitor: &Monitor,
  ) -> Option<String> {
    monitor
      .child_focus_order()
      .filter_map(|child| child.as_workspace().cloned())
      .find(|workspace| !workspace.is_special())
      .map(|workspace| workspace.config().name)
      .or_else(|| self.recent_workspace_name.clone())
  }

  /// Gets windows that should be redrawn.
  ///
  /// When redrawing after a command that changes a window's type (e.g.
  /// tiling -> floating), the original detached window might still be
  /// queued for a redraw and should be filtered out.
  pub fn windows_to_redraw(&self) -> Vec<WindowContainer> {
    self
      .pending_sync
      .containers_to_redraw()
      .values()
      .flat_map(CommonGetters::self_and_descendants)
      .filter(|container| !container.is_detached())
      .filter_map(|container| container.try_into().ok())
      .collect()
  }

  /// Gets the currently focused container. This can either be a window or
  /// a workspace without any descendant windows.
  pub fn focused_container(&self) -> Option<Container> {
    self.root_container.descendant_focus_order().next()
  }

  /// Emits a WM event through an MSPC channel.
  ///
  /// Does not emit events while the WM is paused or populating initial
  /// state. This is to prevent events (e.g. workspace activation events)
  /// from being emitted via IPC server before the initial state is
  /// prepared.
  pub fn emit_event(&self, event: WmEvent) {
    if self.has_initialized
      && (!self.is_paused || matches!(event, WmEvent::PauseChanged { .. }))
    {
      if let Err(err) = self.event_tx.send(event) {
        warn!("Failed to send event: {}", err);
      }
    }
  }

  /// Starts graceful shutdown via an MSPC channel.
  pub fn emit_exit(&self) -> anyhow::Result<()> {
    self.exit_tx.send(())?;
    Ok(())
  }

  pub fn container_by_id(&self, id: Uuid) -> Option<Container> {
    self
      .root_container
      .self_and_descendants()
      .find(|container| container.id() == id)
  }

  /// Gets container to focus after the given window is unmanaged,
  /// minimized, or moved to another workspace.
  pub fn focus_target_after_removal(
    &self,
    removed_window: &WindowContainer,
  ) -> Option<Container> {
    // If the removed window is not focused, no need to change focus.
    if self.focused_container() != Some(removed_window.clone().into()) {
      return None;
    }

    // A scrolling layout hands focus along its strip.
    if let Some(target) =
      crate::layout::scrolling::focus_target_after_removal(removed_window)
    {
      return Some(target);
    }

    // Get descendant focus order excluding the removed container.
    let workspace = removed_window.workspace()?;
    let descendant_focus_order = workspace
      .descendant_focus_order()
      .filter(|descendant| descendant.id() != removed_window.id())
      .collect::<Vec<_>>();

    // Get focus target that matches the removed window type. This applies
    // for windows that aren't in a minimized state.
    let focus_target_of_type = descendant_focus_order
      .iter()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .find(|descendant| {
        matches!(
          (descendant.state(), removed_window.state()),
          (WindowState::Tiling, WindowState::Tiling)
            | (WindowState::Floating(_), WindowState::Floating(_))
            | (WindowState::Fullscreen(_), WindowState::Fullscreen(_))
        )
      })
      .map(Into::into);

    if focus_target_of_type.is_some() {
      return focus_target_of_type;
    }

    let non_minimized_focus_target = descendant_focus_order
      .iter()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .find(|descendant| descendant.state() != WindowState::Minimized)
      .map(Into::into);

    non_minimized_focus_target
      .or(descendant_focus_order.first().cloned())
      .or(Some(workspace.into()))
  }

  /// Returns all containers that contain the given point.
  #[allow(clippy::unused_self)]
  pub fn containers_at_point(
    &self,
    origin_container: &Container,
    point: &Point,
  ) -> Vec<Container> {
    origin_container
      .descendants()
      .filter(|descendant| {
        descendant
          .to_rect()
          .is_ok_and(|rect| rect.contains_point(point))
      })
      .collect()
  }

  /// Returns the monitor that contains the given point.
  pub fn monitor_at_point(&self, point: &Point) -> Option<Monitor> {
    self
      .monitors()
      .iter()
      .find(|monitor| {
        monitor
          .to_rect()
          .is_ok_and(|rect| rect.contains_point(point))
      })
      .cloned()
  }

  /// Cleans up windows that are no longer alive.
  ///
  /// This addresses the "ghost window" issue where applications may
  /// terminate without sending window destroy events, leaving invalid
  /// windows in WM state.
  ///
  /// See: <https://github.com/glzr-io/glazewm/issues/1219>
  pub fn cleanup_invalid_windows(&mut self) -> anyhow::Result<()> {
    let invalid_windows = self
      .windows()
      .into_iter()
      .filter(|window| !window.native().is_valid());

    for window in invalid_windows {
      tracing::info!("Removing invalid window: {}", window);
      unmanage_window(window, self)?;
    }

    // Prune ignored windows that are no longer valid.
    self.ignored_windows.retain(NativeWindow::is_valid);

    Ok(())
  }
}

/// Workspace after (or before) `origin_workspace` among `workspaces`,
/// skipping the special workspace.
fn neighbour_workspace(
  origin_workspace: &Workspace,
  workspaces: Vec<Workspace>,
  is_next: bool,
) -> anyhow::Result<(Option<String>, Option<Workspace>)> {
  let workspaces = workspaces
    .into_iter()
    .filter(|workspace| !workspace.is_special())
    .collect::<Vec<_>>();

  let origin_index = cycle_origin(
    origin_workspace,
    workspaces
      .iter()
      .position(|workspace| workspace.id() == origin_workspace.id()),
  )?;

  let workspace =
    cycle_neighbour(&workspaces, origin_index, is_next).cloned();

  Ok((
    workspace.as_ref().map(|workspace| workspace.config().name),
    workspace,
  ))
}

/// Index of the origin when cycling from `origin_workspace`, given its
/// `position` in the cycled list. The special workspace isn't cycled
/// through, so it has none.
fn cycle_origin(
  origin_workspace: &Workspace,
  position: Option<usize>,
) -> anyhow::Result<Option<usize>> {
  if origin_workspace.is_special() {
    return Ok(None);
  }

  position
    .map(Some)
    .context("Failed to get index of given workspace.")
}

/// Item after (or before) `origin` in `items`, wrapping around. Without
/// an `origin`, the first (or last) item.
fn cycle_neighbour<T>(
  items: &[T],
  origin: Option<usize>,
  is_next: bool,
) -> Option<&T> {
  let last = items.len().checked_sub(1)?;

  let index = match (origin, is_next) {
    (Some(index), true) if index < last => index + 1,
    (Some(_) | None, true) => 0,
    (Some(index), false) => index.checked_sub(1).unwrap_or(last),
    (None, false) => last,
  };

  items.get(index)
}

impl Drop for WmState {
  fn drop(&mut self) {
    // Commit all active resize sessions before cleaning up windows so that
    // surrogate overlays are destroyed and windows are moved to their
    // target positions. This prevents invisible or mispositioned
    // windows after a crash or forced exit.
    #[cfg(target_os = "windows")]
    for session in self.animation_manager.drain_all_sessions() {
      if let Err(err) = session.commit() {
        warn!("Failed to commit resize session on shutdown: {:?}", err);
      }
    }

    // Windows held back for auto-stacking are cloaked but not managed.
    #[cfg(target_os = "windows")]
    for native_window in self.auto_stack.release_all() {
      let _ = native_window.set_cloaked(false);
    }

    let managed_windows = self.windows();

    for window in &managed_windows {
      // Redraw windows to their intended positions. On macOS, this will
      // unhide windows that are on other workspaces.
      if let Ok(rect) = window.to_rect() {
        if let Err(err) = window.native().set_frame(&rect) {
          warn!("Failed to redraw window on cleanup: {:?}", err);
        }
      }

      // Reset any effects on Windows.
      #[cfg(target_os = "windows")]
      {
        // Uncloak before showing — a surrogate animation may have cloaked
        // this window. Without this, the window stays invisible after
        // exit.
        let _ = window.native().set_cloaked(false);

        if let Err(err) = window.native().show() {
          warn!("Failed to show window: {:?}", err);
        }

        let _ = window.native().set_taskbar_visibility(true);
        let _ = window
          .native()
          .set_transparency(&OpacityValue::from_alpha(u8::MAX));

        // Give back the OS border that `apply_native_border_effect`
        // suppressed. `DWMWA_BORDER_COLOR` outlives this process, so a
        // window left with it set keeps no border for the rest of its
        // life -- same class of leak as the cloak reset above.
        let _ = window.native().set_native_border_hidden(false);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use wm_common::ParsedConfig;

  use super::WmState;
  use crate::{
    models::{Monitor, TilingWindow, Workspace, WorkspaceTarget},
    test_utils::mock_workspace_configs,
    user_config::UserConfig,
  };

  /// Workspaces "1" (with a window) and the special one on one monitor,
  /// "2" on another, with "1" to "3" declared in the config.
  fn setup() -> (WmState, UserConfig, Workspace, Workspace) {
    let value = ParsedConfig {
      workspaces: mock_workspace_configs(&["1", "2", "3"]),
      ..ParsedConfig::default()
    };

    let first = Workspace::mock()
      .name("1".to_string())
      .tiling_containers(vec![TilingWindow::mock().call().into()])
      .call();
    let special = Workspace::mock()
      .name("special".to_string())
      .is_special(true)
      .call();
    let state = WmState::mock(vec![
      Monitor::mock()
        .workspaces(vec![first.clone(), special.clone()])
        .call(),
      Monitor::mock()
        .workspaces(vec![Workspace::mock().name("2".to_string()).call()])
        .call(),
    ]);

    (state, UserConfig::from_parsed(value), first, special)
  }

  fn target_name(
    state: &WmState,
    config: &UserConfig,
    origin: &Workspace,
    target: WorkspaceTarget,
  ) -> Option<String> {
    state
      .workspace_by_target(origin, target, config)
      .expect("workspace target")
      .0
  }

  #[test]
  fn cycling_skips_the_special_workspace() {
    let (state, config, first, _) = setup();
    let second = state.workspace_by_name("2").expect("workspace 2");
    let name = |origin: &Workspace, target| {
      target_name(&state, &config, origin, target)
    };

    assert_eq!(
      name(&first, WorkspaceTarget::NextActive).as_deref(),
      Some("2")
    );
    assert_eq!(
      name(&second, WorkspaceTarget::NextActive).as_deref(),
      Some("1")
    );
    assert_eq!(
      name(&first, WorkspaceTarget::PreviousActive).as_deref(),
      Some("2")
    );
    assert_eq!(
      name(&first, WorkspaceTarget::NextActiveInMonitor).as_deref(),
      Some("1")
    );
    assert_eq!(
      name(&first, WorkspaceTarget::Previous).as_deref(),
      Some("3")
    );

    // "1" holds a window and the empty special workspace doesn't count.
    assert_eq!(
      name(&first, WorkspaceTarget::NextEmpty).as_deref(),
      Some("3")
    );
  }

  #[test]
  fn cycling_from_the_special_workspace_starts_at_either_end() {
    let (state, config, _, special) = setup();
    let name = |target| target_name(&state, &config, &special, target);

    assert_eq!(name(WorkspaceTarget::NextActive).as_deref(), Some("1"));
    assert_eq!(
      name(WorkspaceTarget::PreviousActive).as_deref(),
      Some("2")
    );
    assert_eq!(
      name(WorkspaceTarget::NextActiveInMonitor).as_deref(),
      Some("1")
    );
    assert_eq!(
      name(WorkspaceTarget::PreviousActiveInMonitor).as_deref(),
      Some("1")
    );
    assert_eq!(name(WorkspaceTarget::Next).as_deref(), Some("1"));
    assert_eq!(name(WorkspaceTarget::Previous).as_deref(), Some("3"));
  }
}
