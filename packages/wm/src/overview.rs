//! The window overview: a card per workspace of the focused monitor,
//! showing its windows live where they sit, to switch to a workspace or
//! window, or move windows between workspaces.
//!
//! The overview is drawn and driven by `NativeOverview` on a thread of its
//! own; the WM decides what it shows and carries out what is picked.

use anyhow::Context;
use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{Layout, WindowState, WorkspaceConfig};
use wm_platform::{
  Color, NativeOverview, OverviewAction, OverviewFrame,
  OverviewLayoutMode, OverviewStyle, OverviewWindow, OverviewWorkspace,
  Rect, TabFrame,
};

use crate::{
  layout::scrolling,
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

  /// Where the native overview sends user actions, with their session.
  action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
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
      closing: None,
      action_tx,
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
    accent: config
      .value
      .overview
      .accent_color
      .as_ref()
      .unwrap_or(&config.value.window_effects.focused_window.border.color)
      .resolve(),
  };
  let frame = overview_frame(state, &monitor, open.accent, config);

  state.overview.native()?.open(frame, layout);
  state.overview.open = Some(open);
  state.overview.closing = None;
  Ok(())
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

  // `None` once hidden, or once the monitor it was on is gone.
  let frame = shown.and_then(|shown| {
    let monitor = state
      .monitors()
      .into_iter()
      .find(|monitor| monitor.id() == shown.monitor_id)?;

    Some(overview_frame(state, &monitor, shown.accent, config))
  });

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
    Some(frame) => native.hide_on(frame),
    None => {
      overview.open = None;
      native.hide();
    }
  }
}

/// What the overview shows for `monitor`.
fn overview_frame(
  state: &WmState,
  monitor: &Monitor,
  accent: Color,
  config: &UserConfig,
) -> OverviewFrame {
  let properties = monitor.native_properties();
  let area = &properties.working_area;
  let focused = state.focused_container();
  let focused_workspace = focused
    .as_ref()
    .and_then(CommonGetters::workspace)
    .map(|workspace| workspace.id());

  let mut workspaces = monitor
    .workspaces()
    .iter()
    .filter(|workspace| !workspace.is_special())
    .map(|workspace| {
      workspace_card(state, workspace, area, focused_workspace, config)
    })
    .collect::<Vec<_>>();

  workspaces.extend(new_workspace(state, monitor, area, config));
  workspaces.push(special_workspace(
    state,
    area,
    focused_workspace,
    config,
  ));

  let overview = &config.value.overview;

  OverviewFrame {
    rect: properties.working_area,
    scale_factor: properties.scale_factor,
    workspaces,
    digit_workspaces: digit_workspaces(state, monitor, config),
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
    },
  }
}

/// `workspace`'s card, for a monitor with working `area`.
fn workspace_card(
  state: &WmState,
  workspace: &Workspace,
  area: &Rect,
  focused_workspace: Option<Uuid>,
  config: &UserConfig,
) -> OverviewWorkspace {
  let workspace_config = workspace.config();

  OverviewWorkspace {
    label: workspace_label(&workspace_config),
    name: workspace_config.name,
    is_focused: focused_workspace == Some(workspace.id()),
    is_new: false,
    is_special: workspace.is_special(),
    content_rect: content_rect(workspace, area),
    windows: overview_windows(workspace)
      .iter()
      .map(overview_window)
      .collect(),
    tab_bars: workspace_tab_bars(state, workspace, config),
  }
}

/// The special workspace's card, shown on every monitor's overview at
/// `area`: wherever it is, or empty while it is inactive.
fn special_workspace(
  state: &WmState,
  area: &Rect,
  focused_workspace: Option<Uuid>,
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
  let mut card = workspace_card(
    state,
    &workspace,
    &own_area,
    focused_workspace,
    config,
  );
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
    is_hidden_tab: is_inactive_stack_child(window),
    is_scrolled_off: !scrolling::is_in_view(&window.as_container()),
  }
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

  #[test]
  fn special_workspace_card_is_last_on_every_monitor() {
    use wm_common::ParsedConfig;
    use wm_platform::Color;

    use super::overview_frame;
    use crate::{
      commands::container::set_focused_descendant,
      models::Monitor,
      test_utils::{MOCK_MONITOR_HEIGHT, MOCK_MONITOR_WIDTH},
      user_config::UserConfig,
      wm_state::WmState,
    };

    let config = UserConfig::from_parsed(ParsedConfig::default());
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

    let bounds = Rect::from_xy(
      MOCK_MONITOR_WIDTH,
      0,
      MOCK_MONITOR_WIDTH,
      MOCK_MONITOR_HEIGHT,
    );
    let right = Monitor::mock()
      .bounds(bounds.clone())
      .working_area(bounds)
      .workspaces(vec![Workspace::mock().name("2".to_string()).call()])
      .call();
    let state = WmState::mock(vec![left.clone(), right.clone()]);
    set_focused_descendant(&first.into(), None);

    let frame = |monitor: &Monitor| {
      overview_frame(&state, monitor, Color::from_abgr(0), &config)
    };

    for monitor in [&left, &right] {
      let cards = frame(monitor).workspaces;
      let names = cards.iter().map(|card| card.name.as_str());

      assert_eq!(
        names.clone().filter(|name| *name == "special").count(),
        1
      );
      let last = cards.last().expect("cards");
      assert!(last.is_special && last.name == "special");
      assert_eq!(last.windows.len(), 1);
    }

    // On the other monitor, its windows show where they would sit there.
    let left_rect = frame(&left).workspaces.last().expect("card").windows
      [0]
      .rect
      .clone();
    let right_rect =
      frame(&right).workspaces.last().expect("card").windows[0]
        .rect
        .clone();
    assert_eq!(right_rect.x() - left_rect.x(), MOCK_MONITOR_WIDTH);
    assert_eq!(right_rect.y(), left_rect.y());
  }

  #[test]
  fn inactive_special_workspace_shows_an_empty_card() {
    use wm_common::ParsedConfig;
    use wm_platform::Color;

    use super::overview_frame;
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      user_config::UserConfig, wm_state::WmState,
    };

    let mut value = ParsedConfig::default();
    value.special_workspace.display_name = Some("Scratch".to_string());
    let config = UserConfig::from_parsed(value);

    let first = Workspace::mock().name("1".to_string()).call();
    let monitor = Monitor::mock().workspaces(vec![first.clone()]).call();
    let state = WmState::mock(vec![monitor.clone()]);
    set_focused_descendant(&first.into(), None);

    let frame =
      overview_frame(&state, &monitor, Color::from_abgr(0), &config);
    let last = frame.workspaces.last().expect("cards");

    assert!(last.is_special && !last.is_new && last.windows.is_empty());
    assert_eq!(last.name, "special");
    assert_eq!(last.label, "Scratch");
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
