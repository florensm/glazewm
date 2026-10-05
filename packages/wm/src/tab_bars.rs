use std::collections::HashMap;

use regex::Regex;
use wm_common::{StackConfig, TabBarPosition, TabCloseButton};
use wm_platform::{
  BorderOverlayParams, Color, CornerRadii, NativeStackTabBar, Rect,
  TabBarStyle, TabCloseMode, TabFrame, TabInfo,
};

use crate::{
  commands::general::border_overlay_params_for,
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

/// The tab bar of `stack`, or `None` when it has no tab bar, no tabs or
/// isn't shown.
fn tab_frame(
  stack: &StackContainer,
  settings: &TabBarSettings,
  config: &UserConfig,
  focused_id: Option<uuid::Uuid>,
) -> Option<TabFrame> {
  let height = stack.tab_bar_height_px();
  if height <= 0 || !stack.shows_tab_bar() {
    return None;
  }

  let stack_rect = stack.to_rect().ok()?;
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
/// recreated on every workspace switch.
///
/// Bars are put back directly behind their stack's active window after a
/// focus change, and per `restack` otherwise.
pub fn sync_tab_bars(
  state: &mut WmState,
  config: &UserConfig,
  restack: Restack,
) {
  let focused_id = state.focused_container().map(|c| c.id());
  let settings = state.tab_bar_settings.get_or_insert_with(|| {
    TabBarSettings::from_config(&config.value.stack)
  });

  let is_ws_switch_active =
    state.animation_manager.is_workspace_switch_active();

  // A focus change can raise other windows over the bar's anchor, so the
  // bar is put back behind it.
  let restack_all = matches!(restack, Restack::All)
    || state.pending_sync.needs_focus_update();

  let frames = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .map(|stack| {
      let is_shown = !is_ws_switch_active
        && stack.workspace().is_some_and(|ws| ws.is_displayed());

      let frame = is_shown
        .then(|| tab_frame(&stack, settings, config, focused_id))
        .flatten();

      (stack.id(), frame)
    })
    .collect::<HashMap<_, _>>();

  state.tab_bars.retain(|id, _| frames.contains_key(id));

  for (stack_id, frame) in frames {
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

    match NativeStackTabBar::create(&state.dispatcher, on_action) {
      Ok(mut bar) => {
        bar.update(frame, true);
        state.tab_bars.insert(stack_id, bar);
      }
      Err(err) => tracing::warn!("Failed to create tab bar: {err}"),
    }
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{StackConfig, TabBarPosition, TabTitleOverride};
  use wm_platform::{
    BorderOverlayParams, Color, CornerRadii, LengthValue, Rect,
  };

  use super::{
    reach_under_window, strip_radii, window_with_tab_bar, TabBarSettings,
  };
  use crate::{
    models::{StackContainer, TilingWindow},
    traits::CommonGetters,
  };

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
