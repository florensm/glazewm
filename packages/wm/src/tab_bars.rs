use std::collections::HashMap;

use regex::Regex;
use wm_common::{StackConfig, TabBarPosition, TabCloseButton};
use wm_platform::{
  Color, NativeStackTabBar, Rect, TabBarStyle, TabCloseMode, TabFrame,
  TabInfo,
};

use crate::{
  models::StackContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Tab bar settings resolved from the config once, rather than on every
/// sync: colors can come from files, and title overrides are compiled.
pub struct TabBarSettings {
  background: Color,
  opacity: u8,
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
      background: config.tab_bar_background.resolve(),
      opacity: config.tab_bar_opacity.to_alpha(),
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

  /// Tab bar style at `scale_factor`.
  fn style(&self, config: &StackConfig, scale_factor: f32) -> TabBarStyle {
    let px = |length: &wm_platform::LengthValue| {
      length.to_px(0, Some(scale_factor))
    };

    TabBarStyle {
      background: self.background,
      opacity: self.opacity,
      active_background: self.active_background,
      hover_background: self.hover_background,
      urgent_background: self.urgent_background,
      inactive_background: self.inactive_background,
      text: self.text,
      inactive_text: self.inactive_text,
      font_family: config.tab_font_family.clone(),
      font_size: px(&config.tab_font_size),
      corner_radius: px(&config.tab_corner_radius),
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

/// The tab bar of `stack`, or `None` when it has no tab bar, no tabs or
/// isn't shown.
fn tab_frame(
  stack: &StackContainer,
  settings: &TabBarSettings,
  config: &StackConfig,
) -> Option<TabFrame> {
  let height = stack.tab_bar_height_px();
  if height <= 0 || !stack.shows_tab_bar() {
    return None;
  }

  let stack_rect = stack.to_rect().ok()?;
  let rect = match stack.tab_bar_position() {
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
  let anchor = windows.get(active_index)?.native().id().0;

  let tabs = windows
    .iter()
    .map(|window| TabInfo {
      title: settings.tab_title(&window.native_properties().title),
      hwnd: window.native().id().0,
      is_urgent: window.urgency_alert_at().is_some(),
    })
    .collect();

  let scale_factor = stack
    .monitor()
    .map_or(1.0, |monitor| monitor.native_properties().scale_factor);

  Some(TabFrame {
    rect,
    tabs,
    active_index,
    anchor,
    style: settings.style(config, scale_factor),
  })
}

/// Creates, updates, hides or destroys the tab bar of every stack.
///
/// Bars of stacks on hidden workspaces are kept, hidden, rather than
/// recreated on every workspace switch.
pub fn sync_tab_bars(state: &mut WmState, config: &UserConfig) {
  let settings = state.tab_bar_settings.get_or_insert_with(|| {
    TabBarSettings::from_config(&config.value.stack)
  });

  let is_ws_switch_active =
    state.animation_manager.is_workspace_switch_active();

  // A focus change can raise other windows over the bar's anchor, so the
  // bar is put back behind it.
  let restack = state.pending_sync.needs_focus_update();

  let frames = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_stack().cloned())
    .map(|stack| {
      let is_shown = !is_ws_switch_active
        && stack.workspace().is_some_and(|ws| ws.is_displayed());

      let frame = is_shown
        .then(|| tab_frame(&stack, settings, &config.value.stack))
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
  use wm_common::{StackConfig, TabTitleOverride};

  use super::TabBarSettings;

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
