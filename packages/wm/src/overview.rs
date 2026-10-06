//! The window overview: a card per workspace of the focused monitor,
//! showing its windows live where they sit, to switch to a workspace or
//! window, or move windows between workspaces.
//!
//! The overview is drawn and driven by `NativeOverview` on a thread of its
//! own; the WM decides what it shows and carries out what is picked.
//!
//! The same thread hosts the pinned window's preview: a live thumbnail of
//! one window that floats on screen while the window is out of sight.

use anyhow::Context;
use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{
  Color, NativeOverview, OverviewAction, OverviewFrame,
  OverviewLayoutMode, OverviewStyle, OverviewWindow, OverviewWorkspace,
  PinFrame, Rect,
};

use crate::{
  commands::{
    container::focus_container_by_id, workspace::focus_workspace,
  },
  models::{
    Container, Monitor, WindowContainer, Workspace, WorkspaceTarget,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// The WM's side of the overview.
pub struct Overview {
  /// Created on first open.
  native: Option<NativeOverview>,

  /// What is shown, while open.
  open: Option<OpenOverview>,

  /// Where the native overview sends user actions, with their session.
  action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,

  pin: Option<Pin>,
}

/// The pinned window.
struct Pin {
  window_id: Uuid,

  /// Working area and scale of the monitor it was pinned on, where its
  /// preview first shows up.
  area: Rect,
  scale_factor: f32,

  /// Resolved once when pinned, since it can be read from a file.
  border: Color,

  /// What was focused before jumping to the pinned window, to jump back
  /// to.
  return_to: Option<Uuid>,
}

struct OpenOverview {
  monitor_id: Uuid,

  /// Resolved once per open, since it can be read from a file.
  accent: Color,
}

impl Overview {
  pub fn new(
    action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
  ) -> Self {
    Self {
      native: None,
      open: None,
      action_tx,
      pin: None,
    }
  }

  pub fn is_open(&self) -> bool {
    self.open.is_some()
  }

  /// Session of the open overview, which its actions carry.
  pub fn open_session(&self) -> Option<u64> {
    self
      .open
      .as_ref()
      .and(self.native.as_ref())
      .map(NativeOverview::session)
  }

  /// Marks the overview closed.
  ///
  /// The native overview stays up until the next [`sync_overview`] (or
  /// until its zoom into the picked workspace ends), so it only hides
  /// once focus has moved on; hiding the foreground window first would
  /// let the OS pick the next one.
  pub fn close(&mut self) {
    self.open = None;
  }

  /// The native overview, created on first use.
  fn native(&mut self) -> anyhow::Result<&mut NativeOverview> {
    if self.native.is_none() {
      let action_tx = self.action_tx.clone();
      let on_action = Box::new(move |session, action| {
        if let Err(err) = action_tx.send((session, action)) {
          tracing::warn!("Failed to send overview action: {err}");
        }
      });

      self.native = Some(NativeOverview::create(on_action)?);
    }

    self.native.as_mut().context("No native overview.")
  }
}

/// Opens the overview on the focused monitor, laid out as `layout`.
///
/// While open, it switches to `layout` instead, or closes when it already
/// shows that, giving focus back to the focused window.
pub fn toggle_overview(
  state: &mut WmState,
  config: &UserConfig,
  layout: OverviewLayoutMode,
) -> anyhow::Result<()> {
  if state.overview.is_open() {
    if let Some(native) = &state.overview.native {
      native.toggle(layout);
    }
    return Ok(());
  }

  let monitor = state
    .focused_container()
    .and_then(|focused| focused.monitor())
    .context("No focused monitor.")?;

  let open = OpenOverview {
    monitor_id: monitor.id(),
    accent: accent_color(config),
  };
  let frame = overview_frame(state, &monitor, open.accent, config);

  state.overview.native()?.open(frame, layout);
  state.overview.open = Some(open);
  Ok(())
}

/// Brings the native overview in line with the WM's state: hides it once
/// closed, and shows workspaces and windows as they change while open.
///
/// Called at the end of a platform sync, after focus has been synced.
pub fn sync_overview(state: &mut WmState, config: &UserConfig) {
  // `None` once closed, or once the monitor it was on is gone.
  let frame = state.overview.open.as_ref().and_then(|open| {
    let monitor = state
      .monitors()
      .into_iter()
      .find(|monitor| monitor.id() == open.monitor_id)?;

    Some(overview_frame(state, &monitor, open.accent, config))
  });

  let overview = &mut state.overview;

  match (frame, &mut overview.native) {
    (Some(frame), Some(native)) => native.update(frame),
    (_, native) => {
      overview.open = None;
      if let Some(native) = native {
        native.hide();
      }
    }
  }

  sync_pin(state, config);
}

/// Pins `window`, or unpins it if it is the pinned one. Pinning replaces
/// any earlier pin.
pub fn toggle_pin(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if state
    .overview
    .pin
    .as_ref()
    .is_some_and(|pin| pin.window_id == window.id())
  {
    state.overview.pin = None;
    return Ok(());
  }

  let properties = state
    .focused_container()
    .and_then(|focused| focused.monitor())
    .context("No focused monitor.")?
    .native_properties();

  state.overview.pin = Some(Pin {
    window_id: window.id(),
    area: properties.working_area,
    scale_factor: properties.scale_factor,
    border: accent_color(config),
    return_to: None,
  });
  Ok(())
}

pub fn unpin(state: &mut WmState) {
  state.overview.pin = None;
}

/// Focuses the pinned window, switching to its workspace if needed; or,
/// when it is already focused, goes back to what was focused before
/// jumping to it.
pub fn focus_pinned_window(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(pin) = &state.overview.pin else {
    return Ok(());
  };

  let focused = state.focused_container();
  let is_on_pin = focused
    .as_ref()
    .is_some_and(|focused| focused.id() == pin.window_id);

  let target = if is_on_pin {
    pin.return_to.and_then(|id| state.container_by_id(id))
  } else {
    state.container_by_id(pin.window_id)
  };

  if !is_on_pin {
    let return_to = focused.map(|focused| focused.id());
    if let Some(pin) = &mut state.overview.pin {
      pin.return_to = return_to;
    }
  }

  let Some(target) = target else {
    return Ok(());
  };

  if let Some(workspace) = target.workspace() {
    if !workspace.is_displayed() {
      focus_workspace(
        WorkspaceTarget::Name(workspace.config().name),
        state,
        config,
      )?;
    }
  }

  focus_container_by_id(&target.id(), state)
}

/// Shows, updates or hides the pinned window's preview, which shows only
/// while the window is out of sight. Drops the pin once its window is
/// gone.
fn sync_pin(state: &mut WmState, config: &UserConfig) {
  let frame = state.overview.pin.as_ref().and_then(|pin| {
    let window = state
      .container_by_id(pin.window_id)?
      .as_window_container()
      .ok()?;
    let is_on_screen = window
      .workspace()
      .is_some_and(|workspace| workspace.is_displayed());

    let hwnd = window.native().id().0;

    #[allow(clippy::cast_precision_loss)]
    let width = config.value.overview.pin_width as f32;

    Some(PinFrame {
      hwnd,
      area: pin.area.clone(),
      scale_factor: pin.scale_factor,
      width,
      border: pin.border,
      is_visible: !is_on_screen
        && window.state() != WindowState::Minimized,
    })
  });

  let overview = &mut state.overview;
  if frame.is_none() {
    overview.pin = None;

    // The overview's thread is only started for a pin or an open.
    if overview.native.is_none() {
      return;
    }
  }

  match overview.native() {
    Ok(native) => native.set_pin(frame),
    Err(err) => tracing::warn!("Failed to show the pinned window: {err}"),
  }
}

/// Color of the overview's highlights and the pin's border.
fn accent_color(config: &UserConfig) -> Color {
  config
    .value
    .overview
    .accent_color
    .as_ref()
    .unwrap_or(&config.value.window_effects.focused_window.border.color)
    .resolve()
}

/// What the overview shows for `monitor`.
fn overview_frame(
  state: &WmState,
  monitor: &Monitor,
  accent: Color,
  config: &UserConfig,
) -> OverviewFrame {
  let properties = monitor.native_properties();
  let focused = state.focused_container();
  let focused_workspace = focused
    .as_ref()
    .and_then(CommonGetters::workspace)
    .map(|workspace| workspace.id());

  let mut workspaces = monitor
    .workspaces()
    .iter()
    .map(|workspace| {
      let config = workspace.config();

      OverviewWorkspace {
        label: config.display_name.unwrap_or_else(|| config.name.clone()),
        name: config.name,
        is_focused: focused_workspace == Some(workspace.id()),
        is_new: false,
        windows: overview_windows(workspace)
          .iter()
          .map(overview_window)
          .collect(),
      }
    })
    .collect::<Vec<_>>();

  workspaces.extend(new_workspace(state, monitor, config));

  let overview = &config.value.overview;

  OverviewFrame {
    rect: properties.working_area,
    scale_factor: properties.scale_factor,
    workspaces,
    digit_workspaces: digit_workspaces(state, monitor, config),
    focused_window: focused
      .and_then(|focused| focused.as_window_container().ok())
      .map(|window| window.native().id().0),
    pinned_window: state
      .overview
      .pin
      .as_ref()
      .and_then(|pin| state.container_by_id(pin.window_id))
      .and_then(|pinned| pinned.as_window_container().ok())
      .map(|window| window.native().id().0),
    style: OverviewStyle {
      backdrop_blur: overview.backdrop_blur,
      backdrop_tint: overview.backdrop_tint,
      accent,
      card: overview.card_color,
      surface: overview.surface_color,
      caption: overview.caption_color,
      text: overview.text_color,
      subtext: overview.subtext_color,
      search: overview.search_color,
      font_family: overview.font_family.clone(),
      grid_columns: overview.grid_columns,
      open_duration_ms: overview.open_duration_ms,
      close_duration_ms: overview.close_duration_ms,
    },
  }
}

/// The "+" card: the workspace `move --next-empty-workspace` would
/// activate or create from `monitor`. `None` when that is an existing
/// workspace, which has a card of its own, or when there is none.
fn new_workspace(
  state: &WmState,
  monitor: &Monitor,
  config: &UserConfig,
) -> Option<OverviewWorkspace> {
  let origin = monitor.displayed_workspace()?;
  let (name, existing) = state
    .workspace_by_target(&origin, WorkspaceTarget::NextEmpty, config)
    .ok()?;

  let name = name.filter(|_| existing.is_none())?;
  let label = config
    .value
    .workspaces
    .iter()
    .find(|workspace| workspace.name == name)
    .and_then(|workspace| workspace.display_name.clone())
    .unwrap_or_else(|| name.clone());

  Some(OverviewWorkspace {
    name,
    label,
    is_focused: false,
    is_new: true,
    windows: Vec::new(),
  })
}

/// Workspaces named "1" to "10" without a card on `monitor` that the
/// overview's digit keys can still go to: active on another monitor, or
/// inactive in the config.
fn digit_workspaces(
  state: &WmState,
  monitor: &Monitor,
  config: &UserConfig,
) -> Vec<String> {
  let workspaces = state.workspaces();
  let inactive = config.inactive_workspace_configs(&workspaces);

  (1..=10)
    .map(|number: usize| number.to_string())
    .filter(|name| {
      let is_elsewhere = workspaces.iter().any(|workspace| {
        workspace.config().name == *name
          && workspace.monitor().map(|m| m.id()) != Some(monitor.id())
      });
      is_elsewhere || inactive.iter().any(|config| config.name == *name)
    })
    .collect()
}

fn overview_window(window: &WindowContainer) -> OverviewWindow {
  let properties = window.native_properties();

  OverviewWindow {
    hwnd: window.native().id().0,
    title: properties.title.trim().to_string(),
    process_name: properties.process_name.clone(),
    // Where the layout puts it, which is also where it sits on a hidden
    // workspace.
    rect: window.to_rect().unwrap_or(properties.frame),
    is_minimized: window.state() == WindowState::Minimized,
  }
}

/// Windows of `workspace`: tiling windows in layout order, then the
/// others, which are drawn above them.
fn overview_windows(workspace: &Workspace) -> Vec<WindowContainer> {
  fn collect(container: &Container, windows: &mut Vec<WindowContainer>) {
    for child in container.children() {
      match child.as_window_container() {
        Ok(window) => windows.push(window),
        Err(_) => collect(&child, windows),
      }
    }
  }

  let mut windows = Vec::new();
  collect(&workspace.as_container(), &mut windows);

  windows.sort_by_key(|window| window.state() != WindowState::Tiling);
  windows
}

#[cfg(test)]
mod tests {
  use wm_common::WindowState;

  use super::overview_windows;
  use crate::{
    models::{NonTilingWindow, SplitContainer, TilingWindow, Workspace},
    traits::WindowGetters,
  };

  fn tiling(title: &str) -> TilingWindow {
    TilingWindow::mock().title(title.to_string()).call()
  }

  #[test]
  fn windows_are_tiling_in_layout_order_then_others() {
    let split = SplitContainer::mock()
      .tiling_containers(vec![tiling("b").into(), tiling("c").into()])
      .call();

    let workspace = Workspace::mock()
      .tiling_containers(vec![
        tiling("a").into(),
        split.into(),
        tiling("d").into(),
      ])
      .non_tiling_windows(vec![
        NonTilingWindow::mock().title("floating".to_string()).call(),
        NonTilingWindow::mock()
          .title("minimized".to_string())
          .state(WindowState::Minimized)
          .call(),
      ])
      .call();

    let titles = overview_windows(&workspace)
      .iter()
      .map(|window| window.native_properties().title)
      .collect::<Vec<_>>();

    assert_eq!(titles, ["a", "b", "c", "d", "floating", "minimized"]);
  }
}
