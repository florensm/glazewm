//! The window overview: a screen per monitor, with a card per workspace
//! of that monitor showing its windows live where they sit, to switch to
//! a workspace or window, or move windows between workspaces.
//!
//! The overview is drawn and driven by `NativeOverview` on a thread of its
//! own; the WM decides what it shows and carries out what is picked.

use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use anyhow::Context;
use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{
  HotCorner, Layout, OverviewLayout, WindowState, WorkspaceConfig,
};
use wm_platform::{
  BackdropOverlayParams, BorderOverlayParams, Color, MouseButton,
  MouseEvent, NativeBorderOverlay, NativeOverview, OverviewAction,
  OverviewFrame, OverviewLayoutMode, OverviewOverlay, OverviewOverlayKind,
  OverviewScreen, OverviewStyle, OverviewWindow, OverviewWorkspace, Point,
  Rect, TabFrame,
};

use crate::{
  commands::{
    general::{surrogate_effects_for, SyncableOverlay},
    workspace::{activate_workspace, summon_special_workspace},
  },
  layout::{is_hidden_in_layout, scrolling},
  models::{
    is_inactive_stack_child, Container, Monitor, WindowContainer,
    Workspace, WorkspaceTarget,
  },
  tab_bars::{tab_frame, TabBarSettings},
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

  /// What was shown, from its close until the next sync hides it.
  closing: Option<OpenOverview>,

  /// Whether the native overview is on screen: from its open until it
  /// says it hid, which can be a zoom after its close.
  is_shown: bool,

  /// The cursor's visit to the hot corner, which opens the overview once
  /// per visit.
  hot_corner: CornerVisit,

  /// Where the native overview sends user actions, with their session.
  action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
}

/// Where the cursor is with respect to the hot corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CornerVisit {
  /// Out of it.
  Away,

  /// In it, and opening the overview at the instant given if it stays.
  Resting(Instant),

  /// In it, the overview opened or not to be opened on this visit.
  Spent,
}

struct OpenOverview {
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
      closing: None,
      is_shown: false,
      hot_corner: CornerVisit::Away,
      action_tx,
    }
  }

  pub fn is_open(&self) -> bool {
    self.open.is_some()
  }

  /// Whether the overview is on screen, which it still is for a moment
  /// after its close, zooming into the pick.
  pub fn is_shown(&self) -> bool {
    self.is_shown
  }

  /// When the cursor resting in the hot corner opens the overview, while
  /// it does.
  pub fn hot_corner_deadline(&self) -> Option<Instant> {
    match self.hot_corner {
      CornerVisit::Resting(due) => Some(due),
      CornerVisit::Away | CornerVisit::Spent => None,
    }
  }

  /// Takes in that the native overview of `session` hid. Returns whether
  /// that was the one on screen.
  pub fn mark_hidden(&mut self, session: u64) -> bool {
    let is_latest = self
      .native
      .as_ref()
      .is_some_and(|native| native.session() == session);

    is_latest && std::mem::take(&mut self.is_shown)
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
    self.closing = self.open.take();
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

/// Opens the overview on every monitor, laid out as `layout`, with the
/// keyboard on the focused one.
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

  let open = OpenOverview {
    accent: config
      .value
      .overview
      .accent_color
      .as_ref()
      .unwrap_or(&config.value.window_effects.focused_window.border.color)
      .resolve(),
  };
  let frame = overview_frame(state, open.accent, config);
  if frame.screens.is_empty() {
    anyhow::bail!("No monitors to show the overview on.");
  }

  state.overview.native()?.open(frame, layout);
  state.overview.open = Some(open);
  state.overview.closing = None;
  state.overview.is_shown = true;
  Ok(())
}

/// Pixels from a monitor's corner, either way, that count as in it.
const HOT_CORNER_SIZE: i32 = 2;

/// Notes the cursor moving into or out of the hot corner of a monitor,
/// opening the overview once it rests there for `hot_corner_delay_ms`.
pub fn check_hot_corner(
  event: &MouseEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let MouseEvent::Move { position, .. } = event else {
    return Ok(());
  };

  let corner = config.value.overview.hot_corner;
  let is_in = corner != HotCorner::None
    && !state.is_paused
    && state.monitors().iter().any(|monitor| {
      is_in_corner(&monitor.native_properties().bounds, corner, position)
    });

  let visit = &mut state.overview.hot_corner;
  match (*visit, is_in) {
    (_, false) => *visit = CornerVisit::Away,
    (CornerVisit::Away, true) => {
      let delay = u64::from(config.value.overview.hot_corner_delay_ms);
      *visit = CornerVisit::Resting(
        Instant::now() + Duration::from_millis(delay),
      );
    }
    _ => {}
  }

  process_hot_corner(state, config)
}

/// Opens the overview once the cursor has rested in the hot corner long
/// enough, unless a mouse button is held, e.g. dragging a window there.
pub fn process_hot_corner(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let CornerVisit::Resting(due) = state.overview.hot_corner else {
    return Ok(());
  };
  if Instant::now() < due {
    return Ok(());
  }
  state.overview.hot_corner = CornerVisit::Spent;

  let is_button_held = [MouseButton::Left, MouseButton::Right]
    .iter()
    .any(|button| state.dispatcher.is_mouse_down(button));
  if is_button_held || state.overview.is_shown() {
    return Ok(());
  }

  let layout = to_layout_mode(config.value.overview.layout);
  toggle_overview(state, config, layout)
}

/// Whether `position` is in `corner` of `bounds`.
fn is_in_corner(
  bounds: &Rect,
  corner: HotCorner,
  position: &Point,
) -> bool {
  let near = |edge: i32, at: i32| (at - edge).abs() < HOT_CORNER_SIZE;
  let (left, top) = (bounds.left, bounds.top);
  let (right, bottom) = (bounds.right - 1, bounds.bottom - 1);

  bounds.contains_point(position)
    && match corner {
      HotCorner::None => false,
      HotCorner::TopLeft => {
        near(left, position.x) && near(top, position.y)
      }
      HotCorner::TopRight => {
        near(right, position.x) && near(top, position.y)
      }
      HotCorner::BottomLeft => {
        near(left, position.x) && near(bottom, position.y)
      }
      HotCorner::BottomRight => {
        near(right, position.x) && near(bottom, position.y)
      }
    }
}

/// The native overview's layout for the config's `layout`.
pub fn to_layout_mode(layout: OverviewLayout) -> OverviewLayoutMode {
  match layout {
    OverviewLayout::Activities => OverviewLayoutMode::Activities,
    OverviewLayout::Stage => OverviewLayoutMode::Stage,
    OverviewLayout::Carousel => OverviewLayoutMode::Carousel,
    OverviewLayout::Grid => OverviewLayoutMode::Grid,
  }
}

/// Brings the native overview in line with the WM's state: hides it once
/// closed, and shows workspaces and windows as they change while open.
///
/// Called at the end of a platform sync, after focus has been synced.
pub fn sync_overview(state: &mut WmState, config: &UserConfig) {
  let is_open = state.overview.open.is_some();
  let shown = state
    .overview
    .open
    .as_ref()
    .or(state.overview.closing.as_ref());

  // `None` once hidden, or once every monitor is gone.
  let frame = shown
    .map(|shown| overview_frame(state, shown.accent, config))
    .filter(|frame| !frame.screens.is_empty());

  let overview = &mut state.overview;
  overview.closing = None;

  let Some(native) = &mut overview.native else {
    overview.open = None;
    return;
  };

  match frame {
    Some(frame) if is_open => native.update(frame),
    // Its zoom into the picked workspace lands on what the pick brought
    // on screen.
    Some(frame) => {
      tracing::debug!(
        target: wm_platform::OVERVIEW_LOG_TARGET,
        "Hiding the overview on the pick's frame."
      );
      native.hide_on(frame);
    }
    None => {
      overview.open = None;
      native.hide();
    }
  }
}

/// What the overview shows: a screen per monitor, left to right.
fn overview_frame(
  state: &WmState,
  accent: Color,
  config: &UserConfig,
) -> OverviewFrame {
  let focused = state.focused_container();
  let focused_monitor = focused
    .as_ref()
    .and_then(CommonGetters::monitor)
    .map(|monitor| monitor.id());

  let mut monitors = state.monitors();
  monitors.sort_by_key(|monitor| {
    let bounds = monitor.native_properties().bounds;
    (bounds.x(), bounds.y())
  });

  let looks = WindowLooks::new(state, config);
  let screens = monitors
    .iter()
    .map(|monitor| {
      let is_focused = focused_monitor == Some(monitor.id());
      overview_screen(state, monitor, is_focused, &looks, config)
    })
    .collect();

  let overview = &config.value.overview;

  OverviewFrame {
    screens,
    digit_workspaces: digit_workspaces(state, config),
    focused_window: focused
      .and_then(|focused| focused.as_window_container().ok())
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
      switch_duration_ms: overview.switch_duration_ms,
      highlight_duration_ms: overview.highlight_duration_ms,
      bounce: overview.bounce,
      effects: overview.effects,
    },
  }
}

/// `monitor`'s screen: its workspaces, the "+" card, and the special
/// workspace's card.
fn overview_screen(
  state: &WmState,
  monitor: &Monitor,
  is_focused: bool,
  looks: &WindowLooks,
  config: &UserConfig,
) -> OverviewScreen {
  let properties = monitor.native_properties();
  let area = &properties.working_area;
  let displayed = monitor
    .displayed_workspace()
    .map(|workspace| workspace.id());

  let mut workspaces = monitor
    .workspaces()
    .iter()
    .filter(|workspace| !workspace.is_special())
    .map(|workspace| {
      workspace_card(state, workspace, area, displayed, looks, config)
    })
    .collect::<Vec<_>>();

  workspaces.extend(new_workspace(state, monitor, area, config));
  workspaces
    .push(special_workspace(state, area, displayed, looks, config));

  OverviewScreen {
    id: monitor.id().as_u128(),
    rect: properties.working_area,
    scale_factor: properties.scale_factor,
    is_focused,
    workspaces,
  }
}

/// `workspace`'s card, for a monitor with working `area` that displays
/// workspace `displayed`.
fn workspace_card(
  state: &WmState,
  workspace: &Workspace,
  area: &Rect,
  displayed: Option<Uuid>,
  looks: &WindowLooks,
  config: &UserConfig,
) -> OverviewWorkspace {
  let workspace_config = workspace.config();

  OverviewWorkspace {
    label: workspace_label(&workspace_config),
    name: workspace_config.name,
    is_displayed: displayed == Some(workspace.id()),
    is_new: false,
    is_special: workspace.is_special(),
    content_rect: content_rect(workspace, area),
    windows: overview_windows(workspace)
      .iter()
      .map(|window| overview_window(state, window, looks, config))
      .collect(),
    tab_bars: workspace_tab_bars(state, workspace, config),
  }
}

/// The special workspace's card, shown on every monitor's screen at
/// `area`: wherever it is, or empty while it is inactive.
fn special_workspace(
  state: &WmState,
  area: &Rect,
  displayed: Option<Uuid>,
  looks: &WindowLooks,
  config: &UserConfig,
) -> OverviewWorkspace {
  let special = state.workspaces().into_iter().find(Workspace::is_special);
  let own_area = special.as_ref().and_then(|workspace| {
    Some(workspace.monitor()?.native_properties().working_area)
  });

  let (Some(workspace), Some(own_area)) = (special, own_area) else {
    let special_config = config.special_workspace_config();

    return OverviewWorkspace {
      label: workspace_label(&special_config),
      name: special_config.name,
      is_special: true,
      content_rect: area.clone(),
      ..OverviewWorkspace::default()
    };
  };

  // Shown as on its own monitor, moved onto this one.
  let mut card =
    workspace_card(state, &workspace, &own_area, displayed, looks, config);
  translate_card(
    &mut card,
    area.x() - own_area.x(),
    area.y() - own_area.y(),
  );
  card
}

/// Moves everything `card` shows by (`dx`, `dy`).
fn translate_card(card: &mut OverviewWorkspace, dx: i32, dy: i32) {
  let translate = |rect: &Rect| {
    rect.translate_to_coordinates(rect.x() + dx, rect.y() + dy)
  };

  card.content_rect = translate(&card.content_rect);
  for window in &mut card.windows {
    window.rect = translate(&window.rect);
  }
  for bar in &mut card.tab_bars {
    bar.rect = translate(&bar.rect);
    bar.outer_rect = translate(&bar.outer_rect);
  }
}

/// Name shown on a workspace's card.
fn workspace_label(workspace_config: &WorkspaceConfig) -> String {
  workspace_config
    .display_name
    .clone()
    .unwrap_or_else(|| workspace_config.name.clone())
}

/// The "+" card: the workspace `move --next-empty-workspace` would
/// activate or create from `monitor`. `None` when that is an existing
/// workspace, which has a card of its own, or when there is none.
fn new_workspace(
  state: &WmState,
  monitor: &Monitor,
  area: &Rect,
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
    .map_or_else(|| name.clone(), workspace_label);

  Some(OverviewWorkspace {
    name,
    label,
    is_new: true,
    content_rect: area.clone(),
    ..OverviewWorkspace::default()
  })
}

/// Workspaces named "1" to "10" without a card that the overview's digit
/// keys can still go to: inactive ones in the config.
fn digit_workspaces(state: &WmState, config: &UserConfig) -> Vec<String> {
  let inactive = config.inactive_workspace_configs(&state.workspaces());

  (1..=10)
    .map(|number: usize| number.to_string())
    .filter(|name| inactive.iter().any(|config| config.name == *name))
    .collect()
}

/// Readies workspace `name` for a pick made on the screen of monitor
/// `monitor_id`: activates it on that monitor if it is inactive (unless
/// it is bound to a monitor), and with `summon`, brings the special
/// workspace there.
pub fn prepare_target(
  state: &mut WmState,
  config: &UserConfig,
  name: &str,
  monitor_id: u128,
  summon: bool,
) -> anyhow::Result<()> {
  let monitor = state
    .monitors()
    .into_iter()
    .find(|monitor| monitor.id().as_u128() == monitor_id)
    .context("No monitor for the overview's pick.")?;

  if config.is_special_workspace_name(name) {
    if summon || state.workspace_by_name(name).is_none() {
      summon_special_workspace(&monitor, state, config)?;
    }
    return Ok(());
  }

  if state.workspace_by_name(name).is_some() {
    return Ok(());
  }

  let is_bound = config.value.workspaces.iter().any(|workspace| {
    workspace.name == name && workspace.bind_to_monitor.is_some()
  });
  activate_workspace(
    Some(name),
    (!is_bound).then_some(monitor),
    state,
    config,
  )
}

/// How the WM draws windows, resolved once per frame: its effects on the
/// focused window and on the others.
struct WindowLooks {
  focused_id: Option<Uuid>,
  focused: WindowLook,
  other: WindowLook,
}

/// The effects the WM gives a window.
struct WindowLook {
  opacity: u8,
  backdrop: Option<BackdropOverlayParams>,
  border: Option<BorderOverlayParams>,
}

impl WindowLooks {
  fn new(state: &WmState, config: &UserConfig) -> Self {
    let look = |is_focused| {
      let (opacity, _, backdrop, border) =
        surrogate_effects_for(is_focused, config);
      WindowLook {
        opacity,
        backdrop,
        border,
      }
    };

    Self {
      focused_id: state.focused_container().map(|focused| focused.id()),
      focused: look(true),
      other: look(false),
    }
  }

  fn of(&self, window: &WindowContainer) -> &WindowLook {
    if self.focused_id == Some(window.id()) {
      &self.focused
    } else {
      &self.other
    }
  }
}

fn overview_window(
  state: &WmState,
  window: &WindowContainer,
  looks: &WindowLooks,
  config: &UserConfig,
) -> OverviewWindow {
  let properties = window.native_properties();
  let look = looks.of(window);
  let is_minimized = window.state() == WindowState::Minimized;
  let is_hidden_tab = is_inactive_stack_child(window);

  let overlays = if shows_overlays(window) {
    [
      window_overlay(
        &state.backdrop_overlays,
        window,
        look.backdrop,
        OverviewOverlayKind::Backdrop,
        config,
      ),
      window_overlay(
        &state.border_overlays,
        window,
        look.border,
        OverviewOverlayKind::Border,
        config,
      ),
    ]
    .into_iter()
    .flatten()
    .collect()
  } else {
    Vec::new()
  };

  OverviewWindow {
    hwnd: window.native().id().0,
    title: properties.title.trim().to_string(),
    process_name: properties.process_name.clone(),
    // Where the layout puts it, which is also where it sits on a hidden
    // workspace.
    rect: window.to_rect().unwrap_or(properties.frame),
    is_minimized,
    is_hidden_tab,
    is_scrolled_off: !scrolling::is_in_view(&window.as_container()),
    opacity: look.opacity,
    overlays,
    border: look.border.filter(|params| {
      params.opacity > 0.0
        && !NativeBorderOverlay::suppressed_for(window, config)
    }),
  }
}

/// Whether `window`'s overlays are on screen, for its preview to show
/// them: it is in view on a shown workspace.
///
/// The WM hides the others, which DWM composes for no thumbnail; cloaked
/// instead, they showed in previews as small, stale boxes.
fn shows_overlays(window: &WindowContainer) -> bool {
  window.state() != WindowState::Minimized
    && !is_hidden_in_layout(window)
    && window
      .workspace()
      .is_some_and(|workspace| workspace.is_displayed())
}

/// `window`'s overlay among `overlays`, while the WM draws one with
/// `params` for it, and how far it reaches past the window.
fn window_overlay<O: SyncableOverlay>(
  overlays: &HashMap<Uuid, O>,
  window: &WindowContainer,
  params: Option<O::Params>,
  kind: OverviewOverlayKind,
  config: &UserConfig,
) -> Option<OverviewOverlay> {
  let params = params.filter(|_| !O::suppressed_for(window, config))?;
  let overlay = overlays.get(&window.id())?;

  // Any rect does: an overlay reaches past its window by the same amount
  // wherever the window is.
  let frame = window.to_rect().ok()?;
  let bounds = O::bounds(&O::overlay_rect(window, &frame), &params);

  Some(OverviewOverlay {
    hwnd: overlay.hwnd().0,
    kind,
    outset: outset(&frame, &bounds),
  })
}

/// How far `bounds` reaches past `frame` on each side (left, top, right,
/// bottom).
fn outset(frame: &Rect, bounds: &Rect) -> [i32; 4] {
  [
    frame.left - bounds.left,
    frame.top - bounds.top,
    bounds.right - frame.right,
    bounds.bottom - frame.bottom,
  ]
}

/// What `workspace`'s card shows of the monitor's working `area`: all of
/// it, and for a scrolling workspace its whole strip.
fn content_rect(workspace: &Workspace, area: &Rect) -> Rect {
  if workspace.layout() != Layout::Scrolling {
    return area.clone();
  }

  let Ok(workspace_rect) = workspace.to_rect() else {
    return area.clone();
  };

  let columns = workspace
    .tiling_children()
    .filter_map(|column| column.to_rect().ok());

  strip_bounds(area, &workspace_rect, columns)
}

/// `area` widened on either side by how far `columns` reach past
/// `workspace`, keeping the outer gaps at both ends of the strip.
fn strip_bounds(
  area: &Rect,
  workspace: &Rect,
  columns: impl IntoIterator<Item = Rect>,
) -> Rect {
  let (left, right) =
    columns.into_iter().fold((0, 0), |(left, right), column| {
      (
        left.max(workspace.left - column.left),
        right.max(column.right - workspace.right),
      )
    });

  Rect::from_ltrb(
    area.left - left,
    area.top,
    area.right + right,
    area.bottom,
  )
}

/// Tab bars of the stacks on `workspace`, as their own bars draw them.
fn workspace_tab_bars(
  state: &WmState,
  workspace: &Workspace,
  config: &UserConfig,
) -> Vec<TabFrame> {
  // Resolved by the first sync of the tab bars, which may not have run.
  let resolved;
  let settings = if let Some(settings) = &state.tab_bar_settings {
    settings
  } else {
    resolved = TabBarSettings::from_config(&config.value.stack);
    &resolved
  };
  let focused_id = state.focused_container().map(|focused| focused.id());

  workspace
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .filter_map(|stack| tab_frame(&stack, settings, config, focused_id))
    .collect()
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
  use wm_platform::Rect;

  use super::{overview_windows, strip_bounds};
  use crate::{
    models::{NonTilingWindow, SplitContainer, TilingWindow, Workspace},
    traits::{CommonGetters, WindowGetters},
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

  /// Config declaring workspaces "1" to "3", with `value` changed by
  /// `edit`.
  fn config_with(
    edit: impl FnOnce(&mut wm_common::ParsedConfig),
  ) -> crate::user_config::UserConfig {
    let mut value = wm_common::ParsedConfig {
      workspaces: crate::test_utils::mock_workspace_configs(&[
        "1", "2", "3",
      ]),
      ..wm_common::ParsedConfig::default()
    };
    edit(&mut value);
    crate::user_config::UserConfig::from_parsed(value)
  }

  /// A monitor to the right of the default mock monitor, showing
  /// `workspaces`.
  fn right_monitor(workspaces: Vec<Workspace>) -> crate::models::Monitor {
    use crate::test_utils::{MOCK_MONITOR_HEIGHT, MOCK_MONITOR_WIDTH};

    let bounds = Rect::from_xy(
      MOCK_MONITOR_WIDTH,
      0,
      MOCK_MONITOR_WIDTH,
      MOCK_MONITOR_HEIGHT,
    );
    crate::models::Monitor::mock()
      .bounds(bounds.clone())
      .working_area(bounds)
      .workspaces(workspaces)
      .call()
  }

  fn frame(
    state: &crate::wm_state::WmState,
    config: &crate::user_config::UserConfig,
  ) -> wm_platform::OverviewFrame {
    super::overview_frame(state, wm_platform::Color::from_abgr(0), config)
  }

  #[test]
  fn each_monitor_has_a_screen_left_to_right() {
    use crate::{
      commands::container::set_focused_descendant,
      models::{Container, Monitor},
      wm_state::WmState,
    };

    let config = config_with(|_| {});
    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock().workspaces(vec![first]).call();
    let second = Workspace::mock().name("2".to_string()).call();
    let right = right_monitor(vec![second.clone()]);
    // Detected right to left.
    let state = WmState::mock(vec![right.clone(), left.clone()]);
    set_focused_descendant(&Container::from(second), None);

    let screens = frame(&state, &config).screens;
    let ids = screens.iter().map(|screen| screen.id).collect::<Vec<_>>();
    assert_eq!(ids, [left.id().as_u128(), right.id().as_u128()]);
    assert_eq!(
      screens
        .iter()
        .map(|screen| screen.is_focused)
        .collect::<Vec<_>>(),
      [false, true]
    );

    let names = |index: usize| {
      screens[index]
        .workspaces
        .iter()
        .filter(|card| !card.is_special && !card.is_new)
        .map(|card| (card.name.as_str(), card.is_displayed))
        .collect::<Vec<_>>()
    };
    assert_eq!(names(0), [("1", true)]);
    assert_eq!(names(1), [("2", true)]);
  }

  #[test]
  fn digits_reach_only_inactive_workspaces_without_a_card() {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let config = config_with(|_| {});
    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock().workspaces(vec![first.clone()]).call();
    let right =
      right_monitor(vec![Workspace::mock().name("2".to_string()).call()]);
    let state = WmState::mock(vec![left, right]);
    set_focused_descendant(&first.into(), None);

    assert_eq!(frame(&state, &config).digit_workspaces, ["3"]);
  }

  #[test]
  fn special_workspace_card_is_last_on_every_monitor() {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      test_utils::MOCK_MONITOR_WIDTH, wm_state::WmState,
    };

    let config = config_with(|_| {});
    let window = tiling("parked");
    let special = Workspace::mock()
      .name("special".to_string())
      .is_special(true)
      .tiling_containers(vec![window.clone().into()])
      .call();
    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock()
      .workspaces(vec![special.clone(), first.clone()])
      .call();
    let right =
      right_monitor(vec![Workspace::mock().name("2".to_string()).call()]);
    let state = WmState::mock(vec![left, right]);
    set_focused_descendant(&first.into(), None);

    let screens = frame(&state, &config).screens;
    for screen in &screens {
      let cards = &screen.workspaces;
      let names = cards.iter().map(|card| card.name.as_str());

      assert_eq!(
        names.clone().filter(|name| *name == "special").count(),
        1
      );
      let last = cards.last().expect("cards");
      assert!(last.is_special && last.name == "special");
      assert_eq!(last.windows.len(), 1);
      assert!(!last.is_displayed, "hidden behind workspace 1");
    }

    // On the other monitor, its windows show where they would sit there.
    let rect = |index: usize| {
      screens[index].workspaces.last().expect("card").windows[0]
        .rect
        .clone()
    };
    assert_eq!(rect(1).x() - rect(0).x(), MOCK_MONITOR_WIDTH);
    assert_eq!(rect(1).y(), rect(0).y());
  }

  #[test]
  fn inactive_special_workspace_shows_an_empty_card() {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let config = config_with(|value| {
      value.special_workspace.display_name = Some("Scratch".to_string());
    });
    let first = Workspace::mock().name("1".to_string()).call();
    let monitor = Monitor::mock().workspaces(vec![first.clone()]).call();
    let state = WmState::mock(vec![monitor]);
    set_focused_descendant(&first.into(), None);

    let screens = frame(&state, &config).screens;
    let last = screens[0].workspaces.last().expect("cards");

    assert!(last.is_special && !last.is_new && last.windows.is_empty());
    assert_eq!(last.name, "special");
    assert_eq!(last.label, "Scratch");
  }

  /// Two monitors showing "1" (focused) and "2", without the special
  /// workspace.
  fn two_monitors() -> (
    crate::wm_state::WmState,
    crate::models::Monitor,
    crate::models::Monitor,
  ) {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock().workspaces(vec![first.clone()]).call();
    let right =
      right_monitor(vec![Workspace::mock().name("2".to_string()).call()]);
    let state = WmState::mock(vec![left.clone(), right.clone()]);
    set_focused_descendant(&first.into(), None);

    (state, left, right)
  }

  /// ID of the monitor workspace `name` is on.
  fn monitor_of(
    state: &crate::wm_state::WmState,
    name: &str,
  ) -> Option<u128> {
    state
      .workspace_by_name(name)?
      .monitor()
      .map(|monitor| monitor.id().as_u128())
  }

  #[test]
  fn a_pick_activates_an_inactive_workspace_on_its_screen() {
    use super::prepare_target;

    let config = config_with(|_| {});
    let (mut state, _, right) = two_monitors();
    let right_id = right.id().as_u128();

    prepare_target(&mut state, &config, "3", right_id, true).unwrap();
    assert_eq!(monitor_of(&state, "3"), Some(right_id));
  }

  #[test]
  fn a_pick_leaves_a_bound_workspace_on_its_monitor() {
    use super::prepare_target;

    let config = config_with(|value| {
      value.workspaces[2].bind_to_monitor = Some(0);
    });
    let (mut state, left, right) = two_monitors();

    prepare_target(&mut state, &config, "3", right.id().as_u128(), false)
      .unwrap();
    assert_eq!(monitor_of(&state, "3"), Some(left.id().as_u128()));
  }

  #[test]
  fn a_pick_summons_the_special_workspace_to_its_screen() {
    use super::prepare_target;

    let config = config_with(|_| {});
    let (mut state, left, right) = two_monitors();
    let (left_id, right_id) = (left.id().as_u128(), right.id().as_u128());

    // Moving a window into it activates it where it was picked.
    prepare_target(&mut state, &config, "special", right_id, false)
      .unwrap();
    assert_eq!(monitor_of(&state, "special"), Some(right_id));

    prepare_target(&mut state, &config, "special", left_id, false)
      .unwrap();
    assert_eq!(monitor_of(&state, "special"), Some(right_id), "stays");

    prepare_target(&mut state, &config, "special", left_id, true).unwrap();
    assert_eq!(monitor_of(&state, "special"), Some(left_id), "summoned");
  }

  #[test]
  fn previews_take_on_their_windows_transparency() {
    use wm_platform::OpacityValue;

    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let config = config_with(|value| {
      let transparency =
        &mut value.window_effects.other_windows.transparency;
      transparency.enabled = true;
      transparency.opacity = OpacityValue::from_alpha(128);
    });
    let (focused, other) = (tiling("focused"), tiling("other"));
    let first = Workspace::mock()
      .name("1".to_string())
      .tiling_containers(vec![focused.clone().into(), other.into()])
      .call();
    let state =
      WmState::mock(vec![Monitor::mock().workspaces(vec![first]).call()]);
    set_focused_descendant(&focused.into(), None);

    let screens = frame(&state, &config).screens;
    let windows = &screens[0].workspaces[0].windows;
    let opacities = windows
      .iter()
      .map(|window| (window.title.as_str(), window.opacity))
      .collect::<Vec<_>>();

    assert_eq!(opacities, [("focused", 255), ("other", 128)]);
    assert!(
      windows.iter().all(|window| window.overlays.is_empty()),
      "the WM draws no overlays for them"
    );
  }

  #[test]
  fn hot_corners_are_the_monitors_own_corners() {
    use wm_common::HotCorner;
    use wm_platform::Point;

    use super::is_in_corner;

    let bounds = Rect::from_xy(1920, 0, 1920, 1080);
    let at = |x, y| Point { x, y };

    assert!(is_in_corner(&bounds, HotCorner::TopLeft, &at(1920, 0)));
    assert!(is_in_corner(&bounds, HotCorner::TopLeft, &at(1921, 1)));
    assert!(!is_in_corner(&bounds, HotCorner::TopLeft, &at(1925, 0)));
    assert!(!is_in_corner(&bounds, HotCorner::TopLeft, &at(1919, 0)));
    assert!(is_in_corner(
      &bounds,
      HotCorner::BottomRight,
      &at(3839, 1079)
    ));
    assert!(!is_in_corner(
      &bounds,
      HotCorner::BottomRight,
      &at(3840, 1079)
    ));
    assert!(!is_in_corner(&bounds, HotCorner::None, &at(1920, 0)));
  }

  #[test]
  fn windows_on_hidden_workspaces_still_carry_their_border() {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let config = config_with(|value| {
      value.window_effects.other_windows.border.enabled = true;
    });
    let (shown, hidden) = (tiling("shown"), tiling("hidden"));
    let first = Workspace::mock()
      .name("1".to_string())
      .tiling_containers(vec![shown.clone().into()])
      .call();
    let second = Workspace::mock()
      .name("2".to_string())
      .tiling_containers(vec![hidden.into()])
      .call();
    let state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![first, second])
      .call()]);
    set_focused_descendant(&shown.into(), None);

    let screens = frame(&state, &config).screens;
    let hidden = screens[0]
      .workspaces
      .iter()
      .flat_map(|workspace| &workspace.windows)
      .find(|window| window.title == "hidden")
      .expect("hidden window");

    assert!(hidden.overlays.is_empty(), "its overlay is hidden");
    assert!(hidden.border.is_some(), "for the overview to draw it");
  }

  #[test]
  fn only_windows_on_screen_show_their_overlays() {
    use super::shows_overlays;
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let (shown, hidden) = (tiling("shown"), tiling("hidden"));
    let first = Workspace::mock()
      .name("1".to_string())
      .tiling_containers(vec![shown.clone().into()])
      .call();
    let second = Workspace::mock()
      .name("2".to_string())
      .tiling_containers(vec![hidden.clone().into()])
      .call();
    let _state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![first, second])
      .call()]);
    set_focused_descendant(&shown.clone().into(), None);

    assert!(shows_overlays(&shown.into()));
    assert!(!shows_overlays(&hidden.into()), "on a hidden workspace");
  }

  #[test]
  fn a_border_reaches_past_its_window_by_its_width() {
    use wm_platform::{
      BorderOverlayParams, Color, NativeBorderOverlay, Overlay,
    };

    use super::outset;

    let frame = Rect::from_xy(100, 100, 800, 600);
    let params = BorderOverlayParams {
      color: Color::from_abgr(0),
      width: 3.0,
      corner_radius: 8.0,
      opacity: 1.0,
    };

    let bounds = NativeBorderOverlay::bounds(&frame, &params);
    assert_eq!(outset(&frame, &bounds), [3, 3, 3, 3]);
    assert_eq!(outset(&frame, &frame), [0; 4], "as a backdrop does");
  }

  #[test]
  fn strip_bounds_widen_the_area_by_the_overflow() {
    let area = Rect::from_ltrb(0, 0, 1920, 1040);
    let workspace = Rect::from_ltrb(10, 10, 1910, 1030);

    // Scrolled 500px into a strip of three 950px-wide columns.
    let columns = [
      Rect::from_xy(-490, 10, 950, 1020),
      Rect::from_xy(470, 10, 950, 1020),
      Rect::from_xy(1430, 10, 950, 1020),
    ];

    assert_eq!(
      strip_bounds(&area, &workspace, columns),
      Rect::from_ltrb(-500, 0, 2390, 1040)
    );
  }

  #[test]
  fn strip_bounds_are_the_area_when_everything_fits() {
    let area = Rect::from_ltrb(0, 0, 1920, 1040);
    let workspace = Rect::from_ltrb(10, 10, 1910, 1030);

    assert_eq!(strip_bounds(&area, &workspace, []), area);
    assert_eq!(
      strip_bounds(&area, &workspace, [Rect::from_xy(10, 10, 900, 1020)]),
      area
    );
  }

  #[test]
  fn content_of_a_scrolling_workspace_is_its_whole_strip() {
    use wm_common::{Layout, ParsedConfig, WindowState};

    use super::content_rect;
    use crate::{
      commands::container::set_focused_descendant,
      layout::{attach, insertion_target},
      models::Monitor,
      traits::PositionGetters,
      user_config::UserConfig,
      wm_state::WmState,
    };

    let config = UserConfig::from_parsed(ParsedConfig::default());
    let workspace = Workspace::mock().layout(Layout::Scrolling).call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);

    let windows = [tiling("a"), tiling("b"), tiling("c")];
    for window in &windows {
      let insertion =
        insertion_target(&WindowState::Tiling, false, &state).unwrap();
      attach(&window.clone().into(), &insertion, &config).unwrap();
      set_focused_descendant(&window.clone().into(), None);
    }

    let area = workspace.to_rect().unwrap();
    let content = content_rect(&workspace, &area);
    let last = windows[2].to_rect().unwrap();

    assert_eq!(content.left, area.left);
    assert_eq!(content.right, last.right);
    assert!(content.right > area.right);
  }
}
