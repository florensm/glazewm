//! What the overview shows, built from the WM's state.

use std::collections::HashMap;

use uuid::Uuid;
use wm_common::{
  EasingFunction, EffectSet, Layout, WindowState, WorkspaceConfig,
};
use wm_platform::{
  NativeBackdropOverlay, NativeBorderOverlay, OverlayParams,
  OverviewCurve, OverviewFrame, OverviewOverlay, OverviewOverlayKind,
  OverviewScreen, OverviewStyle, OverviewWallpaper, OverviewWindow,
  OverviewWorkspace, Rect, TabFrame,
};

use super::OpenOverview;
use crate::{
  animation::engine::apply_easing,
  commands::general::{surrogate_effects_for, SyncableOverlay},
  layout::{is_hidden_in_layout, scrolling},
  models::{
    is_inactive_stack_child, Container, Monitor, WindowContainer,
    Workspace, WorkspaceTarget,
  },
  tab_bars::{tab_frame, TabBarSettings},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  window_overlays::WindowOverlays,
  wm_state::WmState,
};

/// What the overview shows: a screen per monitor, left to right.
pub(super) fn overview_frame(
  state: &WmState,
  open: &OpenOverview,
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
  let (look, wallpaper, animation) =
    (&overview.look, &overview.wallpaper, &overview.animation);
  let colors = &look.colors;
  let open_easing = animation.open_easing();

  OverviewFrame {
    screens,
    digit_workspaces: digit_workspaces(state, config),
    focused_window: focused
      .and_then(|focused| focused.as_window_container().ok())
      .map(|window| window.native().id().0),
    style: OverviewStyle {
      wallpaper: OverviewWallpaper {
        style: wallpaper.style,
        color: open.frame_color,
        padding: wallpaper.padding,
        shadow: wallpaper.shadow,
        blur: wallpaper.blur,
        tint: wallpaper.tint,
      },
      accent: open.accent,
      card: colors.card,
      corner_radius: look.corner_radius.max(0.0),
      surface: colors.surface,
      caption: colors.caption,
      text: colors.text,
      subtext: colors.subtext,
      search: colors.search,
      font_family: look.font_family.clone(),
      zoom_style: animation.style,
      open_curve: zoom_curve(open_easing),
      close_curve: zoom_curve(&animation.easing),
      chrome_from: animation.clamped_chrome_from(),
      open_duration_ms: open_easing.duration_ms(animation.open_ms),
      close_duration_ms: animation.easing.duration_ms(animation.close_ms),
      switch_style: animation.switch,
      switch_duration_ms: animation.switch_ms,
      highlight_duration_ms: animation.highlight_ms,
      bounce: animation.bounce,
      effects: overview.effects,
      window_highlight: look.window_highlight,
      strip_view: overview.strip_view,
      stack: overview.stack,
      expose: overview.expose,
      activities: overview.activities,
      workspace_row: overview.workspace_row,
      search_hint: overview.search_hint,
      particles: config.value.animations.particles.look(),
    },
  }
}

/// Steps `zoom_curve` samples `easing` at.
const ZOOM_CURVE_STEPS: u16 = 64;

/// `easing` sampled for the overview's thread, which plays it.
fn zoom_curve(easing: &EasingFunction) -> OverviewCurve {
  let samples = (0..=ZOOM_CURVE_STEPS)
    .map(|step| {
      apply_easing(f32::from(step) / f32::from(ZOOM_CURVE_STEPS), easing)
    })
    .collect();
  if matches!(easing, EasingFunction::Spring(_)) {
    OverviewCurve::spring(samples)
  } else {
    OverviewCurve::new(samples)
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
    bounds: properties.bounds,
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

/// How the WM draws windows, resolved once per frame: its effects on the
/// focused window, on urgent ones and on the others.
struct WindowLooks {
  focused_id: Option<Uuid>,
  focused: WindowLook,
  urgent: WindowLook,
  other: WindowLook,
}

/// The effects the WM gives a window.
struct WindowLook {
  opacity: u8,
  overlays: OverlayParams,
}

impl WindowLooks {
  fn new(state: &WmState, config: &UserConfig) -> Self {
    let look = |set| {
      let (opacity, _, overlays) = surrogate_effects_for(set, config);
      WindowLook { opacity, overlays }
    };

    Self {
      focused_id: state.focused_container().map(|focused| focused.id()),
      focused: look(EffectSet::Focused),
      urgent: look(EffectSet::Urgent),
      other: look(EffectSet::Other),
    }
  }

  fn of(&self, window: &WindowContainer) -> &WindowLook {
    match window.effect_set(self.focused_id == Some(window.id())) {
      EffectSet::Focused => &self.focused,
      EffectSet::Urgent => &self.urgent,
      EffectSet::Other => &self.other,
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
      window_overlay::<NativeBackdropOverlay>(
        &state.window_overlays,
        window,
        look.overlays.backdrop,
        OverviewOverlayKind::Backdrop,
        config,
      ),
      window_overlay::<NativeBorderOverlay>(
        &state.window_overlays,
        window,
        look.overlays.border,
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
    border: look.overlays.border.filter(|params| {
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
  overlays: &HashMap<Uuid, WindowOverlays>,
  window: &WindowContainer,
  params: Option<O::Params>,
  kind: OverviewOverlayKind,
  config: &UserConfig,
) -> Option<OverviewOverlay> {
  let params = params.filter(|_| !O::suppressed_for(window, config))?;
  let overlay = overlays.get(&window.id()).and_then(O::get)?;

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
    test_utils::{mock_config_with, mock_right_monitor},
    traits::{CommonGetters, WindowGetters},
  };

  fn tiling(title: &str) -> TilingWindow {
    TilingWindow::mock().title(title.to_string()).call()
  }

  fn frame(
    state: &crate::wm_state::WmState,
    config: &crate::user_config::UserConfig,
  ) -> wm_platform::OverviewFrame {
    let open = crate::overview::OpenOverview {
      accent: wm_platform::Color::from_abgr(0),
      frame_color: wm_platform::Color::from_abgr(0),
    };
    super::overview_frame(state, &open, config)
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
  fn each_monitor_has_a_screen_left_to_right() {
    use crate::{
      commands::container::set_focused_descendant,
      models::{Container, Monitor},
      wm_state::WmState,
    };

    let config = mock_config_with(|_| {});
    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock().workspaces(vec![first]).call();
    let second = Workspace::mock().name("2".to_string()).call();
    let right = mock_right_monitor(vec![second.clone()]);
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

    let config = mock_config_with(|_| {});
    let first = Workspace::mock().name("1".to_string()).call();
    let left = Monitor::mock().workspaces(vec![first.clone()]).call();
    let right = mock_right_monitor(vec![Workspace::mock()
      .name("2".to_string())
      .call()]);
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

    let config = mock_config_with(|_| {});
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
    let right = mock_right_monitor(vec![Workspace::mock()
      .name("2".to_string())
      .call()]);
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

    let config = mock_config_with(|value| {
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

  #[test]
  fn previews_take_on_their_windows_transparency() {
    use wm_platform::OpacityValue;

    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let config = mock_config_with(|value| {
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
  fn windows_on_hidden_workspaces_still_carry_their_border() {
    use crate::{
      commands::container::set_focused_descendant, models::Monitor,
      wm_state::WmState,
    };

    let config = mock_config_with(|value| {
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
  fn a_border_reaches_past_its_window_by_its_width_and_glow() {
    use wm_platform::{
      BorderGlow, BorderOverlayParams, Color, NativeBorderOverlay, Overlay,
    };

    use super::outset;

    let frame = Rect::from_xy(100, 100, 800, 600);
    let params = BorderOverlayParams {
      color: Color::from_abgr(0),
      width: 3.0,
      corner_radius: 8.0,
      opacity: 1.0,
      highlight: 0.0,
      rim: 0.0,
      glow: BorderGlow::default(),
      reserve: 0.0,
    };

    let bounds = NativeBorderOverlay::bounds(&frame, &params);
    assert_eq!(outset(&frame, &bounds), [3, 3, 3, 3]);
    assert_eq!(outset(&frame, &frame), [0; 4], "as a backdrop does");

    let glowing = BorderOverlayParams {
      glow: BorderGlow {
        reach: 6.0,
        opacity: 0.5,
      },
      ..params
    };
    let bounds = NativeBorderOverlay::bounds(&frame, &glowing);
    assert_eq!(outset(&frame, &bounds), [9, 9, 9, 9]);

    // Sized for a farther-reaching style of the window's.
    let reserved = BorderOverlayParams {
      reserve: 12.0,
      ..params
    };
    let bounds = NativeBorderOverlay::bounds(&frame, &reserved);
    assert_eq!(outset(&frame, &bounds), [12, 12, 12, 12]);
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

  /// With space reserved on every edge (app bars, the taskbar), the
  /// overview covers the whole monitor, its cards show the working area,
  /// and a tiled window shows where it really is in it.
  #[test]
  fn a_shrunk_working_area_is_shown_inside_the_covered_monitor() {
    use wm_common::WindowState;

    use crate::{
      commands::container::set_focused_descendant,
      layout::{attach, insertion_target},
      models::Monitor,
      test_utils::{MOCK_MONITOR_HEIGHT, MOCK_MONITOR_WIDTH},
      traits::PositionGetters,
      wm_state::WmState,
    };

    let config = mock_config_with(|_| {});
    let bounds =
      Rect::from_xy(0, 0, MOCK_MONITOR_WIDTH, MOCK_MONITOR_HEIGHT);
    let working_area = Rect::from_ltrb(
      bounds.left + 14,
      bounds.top + 42,
      bounds.right - 14,
      bounds.bottom - 14,
    );
    let workspace = Workspace::mock().name("1".to_string()).call();
    let monitor = Monitor::mock()
      .bounds(bounds.clone())
      .working_area(working_area.clone())
      .workspaces(vec![workspace.clone()])
      .call();
    let state = WmState::mock(vec![monitor]);

    let window = tiling("a");
    let insertion =
      insertion_target(&WindowState::Tiling, false, &state).unwrap();
    attach(&window.clone().into(), &insertion, &config).unwrap();
    set_focused_descendant(&window.clone().into(), None);

    let screens = frame(&state, &config).screens;
    let screen = &screens[0];
    assert_eq!(screen.bounds, bounds);
    assert_eq!(screen.rect, working_area);

    let card = screen
      .workspaces
      .iter()
      .find(|card| card.name == "1")
      .expect("card of workspace 1");
    assert_eq!(card.content_rect, working_area);

    // The window lies where the layout put it, inside the working area,
    // the reserved space counted once.
    let real = window.to_rect().unwrap();
    let shown = &card.windows[0].rect;
    assert_eq!(*shown, real);
    assert!(
      shown.left >= working_area.left
        && shown.top >= working_area.top
        && shown.right <= working_area.right
        && shown.bottom <= working_area.bottom,
      "{shown:?} not in {working_area:?}"
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
