use std::collections::HashMap;

use regex::Regex;
use wm_common::{StackConfig, TabBarPosition, TabCloseButton};
use wm_platform::{
  BorderOverlayParams, Color, CornerRadii, Dispatcher, NativeStackTabBar,
  Rect, TabBarStyle, TabCloseMode, TabFrame, TabInfo, UiThread,
};

use crate::{
  animation::AnimationManager,
  commands::general::border_overlay_params_for,
  layout::scrolling,
  models::{NativeWindowProperties, StackContainer, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Tab bar settings resolved from the config once, rather than on every
/// sync: colors can come from files, and title overrides are compiled.
pub struct TabBarSettings {
  background: Color,
  active_background: Color,
  hover_background: Color,
  urgent_background: Color,
  inactive_background: Color,
  text: Color,
  inactive_text: Color,
  title_overrides: Vec<(Regex, String)>,
}

impl TabBarSettings {
  /// Resolves the tab bar settings of `config`.
  ///
  /// Invalid title override patterns are skipped with a warning.
  pub fn from_config(config: &StackConfig) -> Self {
    let title_overrides = config
      .tab_title_overrides
      .iter()
      .filter_map(|title_override| {
        match Regex::new(&title_override.regex) {
          Ok(regex) => Some((regex, title_override.replace.clone())),
          Err(err) => {
            tracing::warn!(
              "Invalid tab title override '{}': {err}",
              title_override.regex
            );
            None
          }
        }
      })
      .collect();

    Self {
      // Opaque, so the bar reads as part of its window.
      background: Color {
        a: u8::MAX,
        ..config.tab_bar_background.resolve()
      },
      active_background: config.tab_active_background.resolve(),
      hover_background: config.tab_hover_background.resolve(),
      urgent_background: config.tab_urgent_background.resolve(),
      inactive_background: config.tab_inactive_background.resolve(),
      text: config.tab_text_color.resolve(),
      inactive_text: config.tab_inactive_text_color.resolve(),
      title_overrides,
    }
  }

  /// Title to show on the tab of a window titled `title`.
  pub fn tab_title(&self, title: &str) -> String {
    let replaced = self.title_overrides.iter().fold(
      title.to_string(),
      |title, (regex, replace)| {
        regex.replace_all(&title, replace.as_str()).into_owned()
      },
    );

    let trimmed = replaced.trim();
    if trimmed.is_empty() {
      title.trim().to_string()
    } else {
      trimmed.to_string()
    }
  }

  /// Label of the tab of a window: its title, or its process name for a
  /// window without one.
  fn tab_label(&self, properties: &NativeWindowProperties) -> String {
    let title = self.tab_title(&properties.title);

    if title.is_empty() {
      properties.process_name.clone()
    } else {
      title
    }
  }

  /// Tab bar style at `scale_factor`, framed by `border` when the
  /// stack's window has one.
  fn style(
    &self,
    config: &StackConfig,
    scale_factor: f32,
    position: &TabBarPosition,
    border: Option<&BorderOverlayParams>,
  ) -> TabBarStyle {
    let px = |length: &wm_platform::LengthValue| {
      length.to_px(0, Some(scale_factor))
    };
    let corner_radius = px(&config.tab_corner_radius);

    TabBarStyle {
      background: self.background,
      active_background: self.active_background,
      hover_background: self.hover_background,
      urgent_background: self.urgent_background,
      inactive_background: self.inactive_background,
      text: self.text,
      inactive_text: self.inactive_text,
      font_family: config.tab_font_family.clone(),
      font_size: px(&config.tab_font_size),
      corner_radius,
      strip_radii: strip_radii(position, corner_radius, border),
      min_tab_width: px(&config.tab_min_width),
      max_tab_width: px(&config.tab_max_width),
      show_icons: config.show_tab_icons,
      show_numbers: config.show_tab_numbers,
      close_button: match config.tab_close_button {
        TabCloseButton::Hover => TabCloseMode::Hover,
        TabCloseButton::Always => TabCloseMode::Always,
        TabCloseButton::Never => TabCloseMode::Never,
      },
      scale_factor,
    }
  }
}

/// Corners of the tab bar's strip: square where it meets its window, and
/// on the outer side rounded like the inside of the window's `border`, or
/// by `corner_radius` without one.
fn strip_radii(
  position: &TabBarPosition,
  corner_radius: i32,
  border: Option<&BorderOverlayParams>,
) -> CornerRadii {
  // The ring's inner edge, which the strip sits against.
  #[allow(clippy::cast_possible_truncation)]
  let outer = border.map_or(corner_radius, |border| {
    (border.corner_radius - border.width).round().max(0.0) as i32
  });

  match position {
    TabBarPosition::Top => CornerRadii {
      top_left: outer,
      top_right: outer,
      ..CornerRadii::default()
    },
    TabBarPosition::Bottom => CornerRadii {
      bottom_right: outer,
      bottom_left: outer,
      ..CornerRadii::default()
    },
  }
}

/// `bar` extended by `overlap` towards its window.
fn reach_under_window(
  bar: &Rect,
  position: &TabBarPosition,
  overlap: i32,
) -> Rect {
  match position {
    TabBarPosition::Top => {
      Rect::from_ltrb(bar.left, bar.top, bar.right, bar.bottom + overlap)
    }
    TabBarPosition::Bottom => {
      Rect::from_ltrb(bar.left, bar.top - overlap, bar.right, bar.bottom)
    }
  }
}

/// The rect the border of `window` shown at `frame` goes around: the
/// window plus its stack's tab bar when it is the stack's active tab.
pub fn window_with_tab_bar(
  window: &WindowContainer,
  frame: &Rect,
) -> Rect {
  let stack = window
    .parent()
    .and_then(|parent| parent.as_stack().cloned());

  match stack {
    Some(stack)
      if stack
        .active_child()
        .is_some_and(|active| active.id() == window.id()) =>
    {
      stack.outer_rect(frame)
    }
    _ => frame.clone(),
  }
}

/// [`window_with_tab_bar`] for the window `window_id`, e.g. one a peek
/// draws at `frame`.
pub fn with_tab_bar_of(
  state: &WmState,
  window_id: uuid::Uuid,
  frame: &Rect,
) -> Rect {
  state
    .container_by_id(window_id)
    .and_then(|container| container.as_window_container().ok())
    .map_or_else(
      || frame.clone(),
      |window| window_with_tab_bar(&window, frame),
    )
}

/// The tab bar of `stack`, or `None` when it has no tab bar, no tabs or
/// isn't shown. The overview draws the same bar on its cards.
pub fn tab_frame(
  stack: &StackContainer,
  settings: &TabBarSettings,
  config: &UserConfig,
  focused_id: Option<uuid::Uuid>,
) -> Option<TabFrame> {
  tab_frame_at(stack, &stack.to_rect().ok()?, settings, config, focused_id)
}

/// The tab bar of `stack` as it is on screen: on the peek drawing its
/// column, if one does, and cut off like it.
fn shown_tab_frame(
  stack: &StackContainer,
  animation_manager: &AnimationManager,
  settings: &TabBarSettings,
  config: &UserConfig,
  focused_id: Option<uuid::Uuid>,
) -> Option<TabFrame> {
  let active = stack.active_child()?;
  let peek = animation_manager.scroll_view.peek_of(&active.id());

  // A column kept off the monitor beside it shows only through a peek,
  // which it has none of while out of view.
  if peek.is_none() && !scrolling::shows_real_window(&active) {
    return None;
  }

  let frame = tab_frame(stack, settings, config, focused_id)?;

  Some(match peek {
    Some((clip, anchor)) => TabFrame {
      clip: Some(clip),
      anchor: anchor.0,
      ..frame
    },
    None => frame,
  })
}

/// The tab bar of `stack` placed at `stack_rect`.
fn tab_frame_at(
  stack: &StackContainer,
  stack_rect: &Rect,
  settings: &TabBarSettings,
  config: &UserConfig,
  focused_id: Option<uuid::Uuid>,
) -> Option<TabFrame> {
  let height = stack.tab_bar_height_px();
  if height <= 0 || !stack.shows_tab_bar() {
    return None;
  }

  let position = stack.tab_bar_position();
  let rect = match position {
    TabBarPosition::Top => Rect::from_ltrb(
      stack_rect.left,
      stack_rect.top,
      stack_rect.right,
      stack_rect.top + height,
    ),
    TabBarPosition::Bottom => Rect::from_ltrb(
      stack_rect.left,
      stack_rect.bottom - height,
      stack_rect.right,
      stack_rect.bottom,
    ),
  };

  let windows = stack.windows();

  let active = stack.active_child()?;
  let active_index = windows.iter().position(|w| w.id() == active.id())?;
  let active_window = windows.get(active_index)?;
  let anchor = active_window.native().id().0;
  let is_focused = focused_id == Some(active_window.id());
  let border = border_overlay_params_for(is_focused, config);
  let window_effects = if is_focused {
    &config.value.window_effects.focused_window
  } else {
    &config.value.window_effects.other_windows
  };

  let tabs = windows
    .iter()
    .map(|window| TabInfo {
      title: settings.tab_label(&window.native_properties()),
      hwnd: window.native().id().0,
      is_urgent: window.urgency_alert_at().is_some(),
    })
    .collect();

  let scale_factor = stack
    .monitor()
    .map_or(1.0, |monitor| monitor.native_properties().scale_factor);

  // The strip reaches under the window's rounded corners, so they don't
  // leave a notch next to the bar.
  #[allow(clippy::cast_possible_truncation)]
  let overlap = (window_effects.window_corner_radius_px().ceil() as i32)
    .clamp(0, (stack_rect.height() - height).max(0));

  Some(TabFrame {
    outer_rect: reach_under_window(&rect, &position, overlap),
    rect,
    clip: None,
    tabs,
    active_index,
    anchor,
    style: settings.style(
      &config.value.stack,
      scale_factor,
      &position,
      border.as_ref(),
    ),
  })
}

/// Which tab bars `sync_tab_bars` puts back behind their stack's window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restack {
  None,
  All,
  /// The bar of this stack, e.g. one whose window is being dragged.
  Stack(uuid::Uuid),
}

/// Creates, updates, hides or destroys the tab bar of every stack.
///
/// Bars of stacks on hidden workspaces are kept, hidden, rather than
/// recreated on every workspace switch. Bars of stacks in a workspace
/// switch's slide are hidden until it ends.
///
/// Bars are put back directly behind their stack's active window after a
/// focus change, and per `restack` otherwise.
///
/// Bars go with their stacks' move animations and the scroll view's
/// moving columns, so this also runs on every frame of those.
pub fn sync_tab_bars(
  state: &mut WmState,
  config: &UserConfig,
  restack: Restack,
) {
  let focused_id = state.focused_container().map(|c| c.id());
  let settings = state.tab_bar_settings.get_or_insert_with(|| {
    TabBarSettings::from_config(&config.value.stack)
  });

  // A focus change can raise other windows over the bar's anchor, so the
  // bar is put back behind it.
  let restack_all = matches!(restack, Restack::All)
    || state.pending_sync.needs_focus_update();

  let animation_manager = &state.animation_manager;

  // Bars a scene still shows a copy of are left on screen beneath it.
  let held = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .filter(|stack| {
      stack
        .windows()
        .iter()
        .any(|window| animation_manager.scenes.holds(&window.id()))
    })
    .map(|stack| stack.id())
    .collect::<std::collections::HashSet<_>>();

  let frames = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .map(|stack| {
      let is_shown = stack.workspace().is_some_and(|ws| ws.is_displayed());

      let frame = is_shown
        .then(|| {
          shown_tab_frame(
            &stack,
            animation_manager,
            settings,
            config,
            focused_id,
          )
        })
        .flatten();

      (stack.id(), frame)
    })
    .collect::<HashMap<_, _>>();

  state.tab_bars.retain(|id, _| frames.contains_key(id));

  for (stack_id, frame) in frames {
    if held.contains(&stack_id) {
      continue;
    }

    let Some(frame) = frame else {
      if let Some(bar) = state.tab_bars.get_mut(&stack_id) {
        bar.hide();
      }
      continue;
    };

    if let Some(bar) = state.tab_bars.get_mut(&stack_id) {
      let restack = restack_all || restack == Restack::Stack(stack_id);
      bar.update(frame, restack);
      continue;
    }

    let tab_action_tx = state.tab_action_tx.clone();
    let on_action = Box::new(move |action| {
      let _ = tab_action_tx.send((stack_id, action));
    });

    match NativeStackTabBar::create(&tab_bar_dispatcher(state), on_action)
    {
      Ok(mut bar) => {
        bar.update(frame, true);
        state.tab_bars.insert(stack_id, bar);
      }
      Err(err) => tracing::warn!("Failed to create tab bar: {err}"),
    }
  }
}

/// Dispatcher for the thread tab bars are created on, starting it with the
/// first bar.
///
/// Tab bars repaint on every animation frame that moves their stack, and
/// the main event loop's thread also runs the keyboard hook: painting
/// there delayed every key press system-wide. Falls back to that thread,
/// for good, if a thread of their own can't be started, so that all bars
/// stay on one thread.
fn tab_bar_dispatcher(state: &WmState) -> Dispatcher {
  let thread = state.tab_bar_thread.get_or_init(|| {
    UiThread::spawn("glazewm-tab-bars")
      .inspect_err(|err| {
        tracing::warn!("Failed to start tab bar thread: {err}");
      })
      .ok()
  });

  thread
    .as_ref()
    .map_or_else(|| state.dispatcher.clone(), |t| t.dispatcher().clone())
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, StackConfig, TabBarPosition, TabTitleOverride};
  use wm_platform::{
    BorderOverlayParams, Color, CornerRadii, LengthValue, Rect,
  };

  use super::{
    reach_under_window, shown_tab_frame, strip_radii, window_with_tab_bar,
    TabBarSettings,
  };
  use crate::{
    models::{Monitor, StackContainer, TilingWindow, Workspace},
    test_utils::{mock_config, MOCK_MONITOR_WIDTH},
    traits::{CommonGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  /// A scrolling workspace whose second column, a stack, runs past the
  /// right edge of its monitor, optionally with another monitor there.
  fn stack_past_edge(
    has_right_neighbor: bool,
  ) -> (WmState, StackContainer) {
    let stack = StackContainer::mock()
      .tab_bar_height(LengthValue::from_px(28))
      .tiling_containers(vec![
        TilingWindow::mock().call().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![
        TilingWindow::mock().call().into(),
        stack.clone().into(),
      ])
      .call();
    stack.set_tiling_size(0.7);

    let mut monitors =
      vec![Monitor::mock().workspaces(vec![workspace]).call()];
    if has_right_neighbor {
      let bounds = Rect::from_xy(MOCK_MONITOR_WIDTH, 0, 1000, 1000);
      monitors.push(
        Monitor::mock()
          .bounds(bounds.clone())
          .working_area(bounds)
          .call(),
      );
    }

    (WmState::mock(monitors), stack)
  }

  #[test]
  fn bar_of_a_column_without_its_peek_shows_only_past_a_free_edge() {
    let config = mock_config();
    let settings = TabBarSettings::from_config(&config.value.stack);

    let (state, stack) = stack_past_edge(false);
    let frame = shown_tab_frame(
      &stack,
      &state.animation_manager,
      &settings,
      &config,
      None,
    );
    assert!(frame.is_some_and(|frame| frame.clip.is_none()));

    let (state, stack) = stack_past_edge(true);
    let frame = shown_tab_frame(
      &stack,
      &state.animation_manager,
      &settings,
      &config,
      None,
    );
    assert!(frame.is_none());
  }

  #[test]
  fn border_goes_around_the_active_tab_and_its_bar() {
    let stack = StackContainer::mock()
      .tab_bar_height(LengthValue::from_px(28))
      .tiling_containers(vec![
        TilingWindow::mock().call().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();

    let active = stack.active_child().unwrap();
    let inactive = stack
      .windows()
      .into_iter()
      .find(|window| window.id() != active.id())
      .unwrap();
    let active = active.as_window_container().unwrap();
    let frame = Rect::from_ltrb(0, 28, 300, 200);

    assert_eq!(
      window_with_tab_bar(&active, &frame),
      Rect::from_ltrb(0, 0, 300, 200)
    );
    assert_eq!(window_with_tab_bar(&inactive, &frame), frame);
  }

  #[test]
  fn bar_reaches_under_its_window() {
    let bar = Rect::from_ltrb(0, 100, 300, 128);

    assert_eq!(
      reach_under_window(&bar, &TabBarPosition::Top, 8),
      Rect::from_ltrb(0, 100, 300, 136)
    );
    assert_eq!(
      reach_under_window(&bar, &TabBarPosition::Bottom, 8),
      Rect::from_ltrb(0, 92, 300, 128)
    );
  }

  #[test]
  fn strip_is_square_where_it_meets_its_window() {
    let border = BorderOverlayParams {
      color: Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
      },
      width: 2.0,
      corner_radius: 10.0,
      opacity: 1.0,
    };

    assert_eq!(
      strip_radii(&TabBarPosition::Bottom, 6, Some(&border)),
      CornerRadii {
        bottom_right: 8,
        bottom_left: 8,
        ..CornerRadii::default()
      }
    );
    assert_eq!(
      strip_radii(&TabBarPosition::Top, 6, None),
      CornerRadii {
        top_left: 6,
        top_right: 6,
        ..CornerRadii::default()
      }
    );
  }

  #[test]
  fn title_overrides_apply_in_order() {
    let config = StackConfig {
      tab_title_overrides: vec![
        TabTitleOverride {
          regex: "^Details for item ".to_string(),
          replace: String::new(),
        },
        TabTitleOverride {
          regex: " — .+$".to_string(),
          replace: String::new(),
        },
        TabTitleOverride {
          regex: "([".to_string(),
          replace: String::new(),
        },
      ],
      ..StackConfig::default()
    };

    let settings = TabBarSettings::from_config(&config);

    assert_eq!(
      settings.tab_title("Details for item 4711 — ACME — MyApp"),
      "4711"
    );
  }

  #[test]
  fn override_emptying_the_title_keeps_the_original() {
    let config = StackConfig {
      tab_title_overrides: vec![TabTitleOverride {
        regex: ".*".to_string(),
        replace: String::new(),
      }],
      ..StackConfig::default()
    };

    let settings = TabBarSettings::from_config(&config);

    assert_eq!(settings.tab_title(" Notes "), "Notes");
  }
}
