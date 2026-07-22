use std::{collections::HashMap, env, fs, path::PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use wm_common::{
  InvokeCommand, KeybindingConfig, MatchType, ParsedConfig,
  WindowMatchConfig, WindowRuleConfig, WindowRuleEvent, WorkspaceConfig,
};

use crate::{
  models::{
    Monitor, NativeWindowProperties, WindowContainer, Workspace,
  },
  traits::{CommonGetters, WindowGetters},
};

/// Resource string for the sample config file.
const SAMPLE_CONFIG: &str =
  include_str!("../../../resources/assets/sample-config.yaml");

#[derive(Debug)]
pub struct UserConfig {
  /// Path to the user config file.
  pub path: PathBuf,

  /// Parsed user config value.
  pub value: ParsedConfig,

  /// Unparsed user config string.
  pub value_str: String,

  /// Hashmap of window rule event types (e.g. `WindowRuleEvent::Manage`)
  /// and the corresponding window rules of that type.
  window_rules_by_event: HashMap<WindowRuleEvent, Vec<WindowRuleConfig>>,

  /// Pre-compiled regexes from `stack.tab_title_overrides`, paired with
  /// their replacement strings. Built once on load and on config reload so
  /// `sync_tab_bars` does not recompile on every platform-sync tick.
  pub compiled_tab_title_overrides: Vec<(Regex, String)>,
}

impl UserConfig {
  /// Creates an instance of `UserConfig`. Reads and validates the user
  /// config from the given path.
  ///
  /// Creates a new config file from sample if it doesn't exist.
  pub fn new(config_path: Option<PathBuf>) -> anyhow::Result<Self> {
    let default_config_path = home::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/config.yaml");

    let config_path = config_path
      .or_else(|| env::var("GLAZEWM_CONFIG_PATH").ok().map(PathBuf::from))
      .unwrap_or(default_config_path);

    let (config_value, config_str) = Self::read(&config_path)?;

    let window_rules_by_event = Self::window_rules_by_event(&config_value);
    let compiled_tab_title_overrides =
      Self::compile_tab_title_overrides(&config_value);

    Ok(Self {
      path: config_path,
      value: config_value,
      value_str: config_str,
      window_rules_by_event,
      compiled_tab_title_overrides,
    })
  }

  /// Reads and validates the user config from the given path.
  ///
  /// Creates a new config file from sample if it doesn't exist.
  fn read(
    config_path: &PathBuf,
  ) -> anyhow::Result<(ParsedConfig, String)> {
    if !config_path.exists() {
      Self::create_sample(config_path)?;
    }

    let config_str = fs::read_to_string(config_path)
      .context("Unable to read config file.")?;

    // TODO: Improve error formatting of serde_yaml errors. Something
    // similar to https://github.com/AlexanderThaller/format_serde_error
    let config_value = serde_yaml::from_str(&config_str)?;

    Ok((config_value, config_str))
  }

  /// Initializes a new config file from the sample config resource.
  fn create_sample(config_path: &PathBuf) -> Result<()> {
    let parent_dir =
      config_path.parent().context("Invalid config path.")?;

    fs::create_dir_all(parent_dir).with_context(|| {
      format!("Unable to create directory {}.", config_path.display())
    })?;

    fs::write(config_path, SAMPLE_CONFIG).with_context(|| {
      format!("Unable to write to {}.", config_path.display())
    })?;

    Ok(())
  }

  pub fn reload(&mut self) -> anyhow::Result<()> {
    let (config_value, config_str) = Self::read(&self.path)?;

    self.window_rules_by_event =
      Self::window_rules_by_event(&config_value);
    self.compiled_tab_title_overrides =
      Self::compile_tab_title_overrides(&config_value);
    self.value = config_value;
    self.value_str = config_str;

    Ok(())
  }

  fn default_window_rules(
    config_value: &ParsedConfig,
  ) -> Vec<WindowRuleConfig> {
    let mut window_rules = Vec::new();

    let floating_defaults =
      &config_value.window_behavior.state_defaults.floating;

    // Default float rules.
    window_rules.push(WindowRuleConfig {
      commands: vec![InvokeCommand::SetFloating {
        centered: Some(floating_defaults.centered),
        shown_on_top: Some(floating_defaults.shown_on_top),
        x_pos: None,
        y_pos: None,
        width: None,
        height: None,
      }],
      match_window: vec![
        WindowMatchConfig {
          window_class: Some(MatchType::Equals { equals:
          // W10/W11 system dialog shown when moving and deleting files.
          "OperationStatusWindow".to_string(),
        }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_class: Some(MatchType::Equals { equals:
          // W10/W11 system dialogs (e.g. File Explorer save/open dialog).
          "#32770".to_string(),
        }),
          ..WindowMatchConfig::default()
        },
      ],
      on: vec![WindowRuleEvent::Manage],
      run_once: true,
    });

    // Default ignore rules.
    window_rules.push(WindowRuleConfig {
      commands: vec![InvokeCommand::Ignore],
      match_window: vec![
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            equals: "SearchApp".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            equals: "SearchHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            equals: "ShellExperienceHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            // W10/11 start menu.
            equals: "StartMenuExperienceHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            // W10/11 screen snipping tool.
            equals: "ScreenClippingHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            // W11 lock screen.
            equals: "LockApp".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
      ],
      on: vec![WindowRuleEvent::Manage],
      run_once: true,
    });

    window_rules
  }

  /// Compiles `stack.tab_title_overrides` regexes once so `sync_tab_bars`
  /// can apply them without recompiling on every platform-sync tick.
  ///
  /// Invalid regex patterns are logged and skipped rather than propagated
  /// as errors, so a single bad pattern does not break the entire tab bar.
  fn compile_tab_title_overrides(
    config_value: &ParsedConfig,
  ) -> Vec<(Regex, String)> {
    config_value
      .stack
      .tab_title_overrides
      .iter()
      .filter_map(|o| {
        match Regex::new(&o.regex) {
          Ok(re) => Some((re, o.replace.clone())),
          Err(err) => {
            tracing::warn!(
              "Invalid tab_title_overrides regex {:?}: {err}",
              o.regex
            );
            None
          }
        }
      })
      .collect()
  }

  fn window_rules_by_event(
    config_value: &ParsedConfig,
  ) -> HashMap<WindowRuleEvent, Vec<WindowRuleConfig>> {
    let mut window_rules_by_event = HashMap::new();

    // Combine user-defined window rules with the default ones.
    let default_window_rules = Self::default_window_rules(config_value);
    let all_window_rules = config_value
      .window_rules
      .iter()
      .chain(default_window_rules.iter());

    for window_rule in all_window_rules {
      for event_type in &window_rule.on {
        window_rules_by_event
          .entry(event_type.clone())
          .or_insert_with(Vec::new)
          .push(window_rule.clone());
      }
    }

    window_rules_by_event
  }

  /// Window rules that should be applied to the window when the given
  /// event occurs.
  pub fn pending_window_rules(
    &self,
    window: &WindowContainer,
    event: &WindowRuleEvent,
  ) -> Vec<WindowRuleConfig> {
    let native_properties = window.native_properties();

    self
      .window_rules_by_event
      .get(event)
      .unwrap_or(&Vec::new())
      .iter()
      .filter(|rule| {
        // Skip if window has already ran the rule.
        !window.done_window_rules().contains(rule)
          && Self::rule_matches(rule, &native_properties)
      })
      .cloned()
      .collect()
  }

  /// Whether a window with the given native properties is matched by a
  /// `force-manage` window rule on the `manage` event.
  ///
  /// Used to bypass the built-in manageability checks before the window
  /// enters the window rule pipeline.
  pub fn is_force_managed(
    &self,
    properties: &NativeWindowProperties,
  ) -> bool {
    self
      .window_rules_by_event
      .get(&WindowRuleEvent::Manage)
      .is_some_and(|rules| {
        rules.iter().any(|rule| {
          rule.commands.contains(&InvokeCommand::ForceManage)
            && Self::rule_matches(rule, properties)
        })
      })
  }

  /// Whether a window with the given native properties matches any of
  /// the window rule's match configs.
  fn rule_matches(
    rule: &WindowRuleConfig,
    properties: &NativeWindowProperties,
  ) -> bool {
    rule.match_window.iter().any(|match_config| {
      let is_process_match = match_config
        .window_process
        .as_ref()
        .is_none_or(|match_type| {
          // TODO: Temp fix for matching Zebar on both platforms with
          // the same process name. Consider using lowercase for every
          // `equals` match type.
          if properties.process_name == "Zebar" {
            match_type.is_match("Zebar") || match_type.is_match("zebar")
          } else {
            match_type.is_match(&properties.process_name)
          }
        });

      let is_class_match = {
        #[cfg(target_os = "windows")]
        {
          match_config.window_class.as_ref().is_none_or(|match_type| {
            match_type.is_match(&properties.class_name)
          })
        }
        #[cfg(not(target_os = "windows"))]
        {
          match_config.window_class.is_none()
        }
      };

      let is_title_match = match_config
        .window_title
        .as_ref()
        .is_none_or(|match_type| match_type.is_match(&properties.title));

      is_process_match && is_class_match && is_title_match
    })
  }

  pub fn inactive_workspace_configs(
    &self,
    active_workspaces: &[Workspace],
  ) -> Vec<&WorkspaceConfig> {
    self
      .value
      .workspaces
      .iter()
      .filter(|config| {
        !active_workspaces
          .iter()
          .any(|workspace| workspace.config().name == config.name)
      })
      .collect()
  }

  pub fn workspace_config_for_monitor(
    &self,
    monitor: &Monitor,
    active_workspaces: &[Workspace],
  ) -> Option<&WorkspaceConfig> {
    let inactive_configs =
      self.inactive_workspace_configs(active_workspaces);

    inactive_configs.into_iter().find(|&config| {
      config
        .bind_to_monitor
        .as_ref()
        .is_some_and(|monitor_index| {
          monitor.index() == *monitor_index as usize
        })
    })
  }

  /// Gets the first inactive workspace config, prioritizing configs that
  /// don't have a monitor binding.
  pub fn next_inactive_workspace_config(
    &self,
    active_workspaces: &[Workspace],
  ) -> Option<&WorkspaceConfig> {
    let inactive_configs =
      self.inactive_workspace_configs(active_workspaces);

    inactive_configs
      .iter()
      .find(|config| config.bind_to_monitor.is_none())
      .or(inactive_configs.first())
      .copied()
  }

  pub fn workspace_config_index(
    &self,
    workspace_name: &str,
  ) -> Option<usize> {
    self
      .value
      .workspaces
      .iter()
      .position(|config| config.name == workspace_name)
  }

  pub fn sort_workspaces(&self, workspaces: &mut [Workspace]) {
    workspaces.sort_by_key(|workspace| {
      self.workspace_config_index(&workspace.config().name)
    });
  }

  /// Keybinding configs that should be active for the current binding mode
  /// and pause state.
  ///
  /// When paused, only the configs with `InvokeCommand::WmTogglePause` are
  /// returned so that unpausing remains possible.
  pub fn active_keybinding_configs(
    &self,
    binding_modes: &[wm_common::BindingModeConfig],
    is_paused: bool,
  ) -> impl Iterator<Item = KeybindingConfig> {
    let source_configs = if let Some(first_mode) = binding_modes.first() {
      &first_mode.keybindings
    } else {
      &self.value.keybindings
    }
    .clone();

    source_configs.into_iter().filter(move |kb| {
      if is_paused {
        kb.commands
          .contains(&wm_common::InvokeCommand::WmTogglePause)
      } else {
        true
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{ParsedConfig, WindowTransitionStyle, WorkspaceSwitchStyle};
  use wm_platform::Rect;

  use super::*;

  /// The bundled sample config (which uses the `type` key for animation
  /// transition types) must always parse.
  #[test]
  fn sample_config_parses() {
    let config: ParsedConfig = serde_yaml::from_str(SAMPLE_CONFIG)
      .expect("sample config should parse");

    assert_eq!(
      config.animations.window_open.style,
      WindowTransitionStyle::SlideRight
    );
    assert_eq!(
      config.animations.workspace_switch.style,
      WorkspaceSwitchStyle::Slide
    );
  }

  /// Configs written before the `style` -> `type` key rename must keep
  /// parsing via the legacy aliases (including the older `direction` alias
  /// on `window_open`).
  #[test]
  fn legacy_style_keys_parse() {
    let yaml = r"
animations:
  window_open:
    style: 'zoom'
  window_close:
    style: 'slide_left'
  workspace_switch:
    style: 'fade'
";
    let config: ParsedConfig =
      serde_yaml::from_str(yaml).expect("legacy config should parse");

    assert_eq!(
      config.animations.window_open.style,
      WindowTransitionStyle::Zoom
    );
    assert_eq!(
      config.animations.window_close.style,
      WindowTransitionStyle::SlideLeft
    );
    assert_eq!(
      config.animations.workspace_switch.style,
      WorkspaceSwitchStyle::Fade
    );

    let yaml_direction = r"
animations:
  window_open:
    direction: 'slide_top'
";
    let config: ParsedConfig = serde_yaml::from_str(yaml_direction)
      .expect("direction alias should parse");
    assert_eq!(
      config.animations.window_open.style,
      WindowTransitionStyle::SlideTop
    );
  }

  /// Creates `NativeWindowProperties` with the given process name and
  /// title for testing.
  fn test_properties(
    process_name: &str,
    title: &str,
  ) -> NativeWindowProperties {
    NativeWindowProperties {
      title: title.to_string(),
      #[cfg(target_os = "windows")]
      class_name: "TestClass".to_string(),
      process_name: process_name.to_string(),
      frame: Rect::from_ltrb(0, 0, 100, 100),
      is_minimized: false,
      is_maximized: false,
      is_resizable: true,
      #[cfg(target_os = "windows")]
      shadow_borders: wm_platform::RectDelta::zero(),
    }
  }

  /// Creates a `UserConfig` with the given window rules for testing.
  fn test_config(window_rules: Vec<WindowRuleConfig>) -> UserConfig {
    let config_value = ParsedConfig {
      window_rules,
      ..ParsedConfig::default()
    };

    UserConfig {
      path: PathBuf::new(),
      window_rules_by_event: UserConfig::window_rules_by_event(
        &config_value,
      ),
      compiled_tab_title_overrides: Vec::new(),
      value: config_value,
      value_str: String::new(),
    }
  }

  /// Creates a window rule with the given command matching the given
  /// process name.
  fn test_rule(
    command: InvokeCommand,
    process_name: &str,
  ) -> WindowRuleConfig {
    WindowRuleConfig {
      commands: vec![command],
      match_window: vec![WindowMatchConfig {
        window_process: Some(MatchType::Equals {
          equals: process_name.to_string(),
        }),
        ..WindowMatchConfig::default()
      }],
      on: vec![WindowRuleEvent::Manage],
      run_once: true,
    }
  }

  #[test]
  fn force_manage_rule_matches_window() {
    let config = test_config(vec![test_rule(
      InvokeCommand::ForceManage,
      "my-launcher",
    )]);

    assert!(
      config.is_force_managed(&test_properties("my-launcher", "Launcher"))
    );
  }

  #[test]
  fn non_matching_window_is_not_force_managed() {
    let config = test_config(vec![test_rule(
      InvokeCommand::ForceManage,
      "my-launcher",
    )]);

    assert!(
      !config.is_force_managed(&test_properties("other-app", "Other"))
    );
  }

  #[test]
  fn matching_rule_without_force_manage_command_is_skipped() {
    let config =
      test_config(vec![test_rule(InvokeCommand::Ignore, "my-launcher")]);

    assert!(
      !config.is_force_managed(&test_properties("my-launcher", "Launcher"))
    );
  }

  #[test]
  fn no_windows_are_force_managed_by_default() {
    let config = test_config(Vec::new());

    assert!(!config.is_force_managed(&test_properties(
      "Flow.Launcher",
      "Flow.Launcher"
    )));
  }
}
