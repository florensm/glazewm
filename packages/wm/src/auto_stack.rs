use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use wm_common::{AutoStackRuleConfig, WindowMatchConfig};
use wm_platform::{NativeWindow, WindowId};

use crate::{models::NativeWindowProperties, user_config::UserConfig};

/// Properties of a window that rule it out of auto-stacking whatever its
/// title is.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowTraits {
  pub has_owner: bool,
  pub is_dialog: bool,
  pub is_tool_window: bool,
}

impl WindowTraits {
  /// Reads the traits of `native`.
  ///
  /// Dialogs are `#32770` windows and windows with a modal dialog frame.
  #[cfg(target_os = "windows")]
  pub fn of(
    native: &NativeWindow,
    props: &NativeWindowProperties,
  ) -> Self {
    use wm_platform::{
      NativeWindowWindowsExt, WS_EX_DLGMODALFRAME, WS_EX_TOOLWINDOW,
    };

    Self {
      has_owner: native.has_owner_window(),
      is_dialog: props.class_name == "#32770"
        || native.has_window_style_ex(WS_EX_DLGMODALFRAME),
      is_tool_window: native.has_window_style_ex(WS_EX_TOOLWINDOW),
    }
  }

  /// Reads the traits of `native`.
  #[cfg(not(target_os = "windows"))]
  pub fn of(
    _native: &NativeWindow,
    _props: &NativeWindowProperties,
  ) -> Self {
    Self::default()
  }
}

/// What to do with a window that is about to be managed.
#[derive(Debug, PartialEq)]
pub enum AutoStackDecision<'a> {
  /// Place the window in the stack named by the rule.
  Join(&'a AutoStackRuleConfig),

  /// The title is still empty but the rest of a rule matches, so the
  /// window could still turn out to belong in a stack.
  Wait,

  /// Place the window normally.
  Skip,
}

/// Decides whether a window joins a stack, using the first rule that
/// applies to it.
pub fn decide<'a>(
  rules: &'a [AutoStackRuleConfig],
  props: &NativeWindowProperties,
  traits: WindowTraits,
) -> AutoStackDecision<'a> {
  if traits.is_dialog || traits.is_tool_window {
    return AutoStackDecision::Skip;
  }

  let mut could_match_later = false;

  for rule in rules {
    if traits.has_owner && !rule.allow_owned {
      continue;
    }

    let matches_any = |configs: &[WindowMatchConfig]| {
      configs
        .iter()
        .any(|config| UserConfig::window_matches(config, props))
    };

    if matches_any(&rule.exclude) {
      continue;
    }

    if matches_any(&rule.match_window) {
      return AutoStackDecision::Join(rule);
    }

    if props.title.trim().is_empty() {
      could_match_later |= rule.match_window.iter().any(|config| {
        let untitled = WindowMatchConfig {
          window_title: None,
          ..config.clone()
        };

        UserConfig::window_matches(&untitled, props)
      });
    }
  }

  if could_match_later {
    AutoStackDecision::Wait
  } else {
    AutoStackDecision::Skip
  }
}

/// A window held back, cloaked and unmanaged, until its title is known.
struct HeldWindow {
  native: NativeWindow,
  deadline: Instant,
}

/// Auto-stacking bookkeeping that outlives a single event.
#[derive(Default)]
pub struct AutoStackState {
  /// Windows waiting for a title, keyed by handle.
  held: HashMap<WindowId, HeldWindow>,

  /// Windows that have been placed in a stack by a rule, or were taken
  /// out of one. They are never auto-stacked again, so a window removed
  /// from its stack isn't pulled back in by a later title change.
  settled: HashSet<WindowId>,
}

impl AutoStackState {
  /// Holds `native` back until `timeout` has passed.
  ///
  /// A window that is already held keeps its original deadline, so title
  /// changes can't keep it hidden forever.
  pub fn hold(&mut self, native: &NativeWindow, timeout: Duration) {
    self.held.entry(native.id()).or_insert_with(|| HeldWindow {
      native: native.clone(),
      deadline: Instant::now() + timeout,
    });
  }

  /// Stops holding the window, returning it if it was held.
  pub fn release(&mut self, id: WindowId) -> Option<NativeWindow> {
    self.held.remove(&id).map(|held| held.native)
  }

  pub fn is_held(&self, id: WindowId) -> bool {
    self.held.contains_key(&id)
  }

  /// Earliest deadline of the held windows.
  pub fn next_deadline(&self) -> Option<Instant> {
    self.held.values().map(|held| held.deadline).min()
  }

  /// Held windows whose deadline has passed. They stay held until they
  /// are managed, so the manageability check still treats them as shown.
  pub fn expired(&self, now: Instant) -> Vec<NativeWindow> {
    self
      .held
      .values()
      .filter(|held| held.deadline <= now)
      .map(|held| held.native.clone())
      .collect()
  }

  /// Releases and returns every held window.
  pub fn release_all(&mut self) -> Vec<NativeWindow> {
    self.held.drain().map(|(_, held)| held.native).collect()
  }

  pub fn is_settled(&self, id: WindowId) -> bool {
    self.settled.contains(&id)
  }

  pub fn mark_settled(&mut self, id: WindowId) {
    self.settled.insert(id);
  }

  /// Drops everything known about a destroyed window, since its handle
  /// can be reused by a new window.
  pub fn forget(&mut self, id: WindowId) {
    self.held.remove(&id);
    self.settled.remove(&id);
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{AutoStackRuleConfig, MatchType, WindowMatchConfig};

  use super::{decide, AutoStackDecision, WindowTraits};
  use crate::models::NativeWindowProperties;

  fn details_rule() -> AutoStackRuleConfig {
    AutoStackRuleConfig {
      name: "details".to_string(),
      match_window: vec![WindowMatchConfig {
        window_process: Some(MatchType::Equals {
          equals: "MyApp".to_string(),
        }),
        window_title: Some(MatchType::Regex {
          regex: "^Details for".to_string(),
        }),
        ..WindowMatchConfig::default()
      }],
      exclude: vec![WindowMatchConfig {
        window_title: Some(MatchType::Includes {
          includes: "(read-only)".to_string(),
        }),
        ..WindowMatchConfig::default()
      }],
      workspace: None,
      allow_owned: false,
    }
  }

  fn window(process: &str, title: &str) -> NativeWindowProperties {
    NativeWindowProperties::mock()
      .process_name(process.to_string())
      .title(title.to_string())
      .call()
  }

  #[test]
  fn matching_window_joins() {
    let rules = [details_rule()];
    let props = window("MyApp", "Details for item 42");

    assert_eq!(
      decide(&rules, &props, WindowTraits::default()),
      AutoStackDecision::Join(&rules[0])
    );
  }

  #[test]
  fn other_titles_and_processes_are_skipped() {
    let rules = [details_rule()];

    for props in [
      window("MyApp", "MyApp"),
      window("MyApp", "Item 42"),
      window("notepad", "Details for item 42"),
    ] {
      assert_eq!(
        decide(&rules, &props, WindowTraits::default()),
        AutoStackDecision::Skip
      );
    }
  }

  #[test]
  fn untitled_window_of_matching_process_waits() {
    let rules = [details_rule()];

    assert_eq!(
      decide(&rules, &window("MyApp", ""), WindowTraits::default()),
      AutoStackDecision::Wait
    );
    assert_eq!(
      decide(&rules, &window("notepad", ""), WindowTraits::default()),
      AutoStackDecision::Skip
    );
  }

  #[test]
  fn excluded_window_is_skipped() {
    let rules = [details_rule()];
    let props = window("MyApp", "Details for item 42 (read-only)");

    assert_eq!(
      decide(&rules, &props, WindowTraits::default()),
      AutoStackDecision::Skip
    );
  }

  #[test]
  fn dialogs_tool_and_owned_windows_are_skipped() {
    let rules = [details_rule()];
    let props = window("MyApp", "Details for item 42");

    for traits in [
      WindowTraits {
        is_dialog: true,
        ..WindowTraits::default()
      },
      WindowTraits {
        is_tool_window: true,
        ..WindowTraits::default()
      },
      WindowTraits {
        has_owner: true,
        ..WindowTraits::default()
      },
    ] {
      assert_eq!(decide(&rules, &props, traits), AutoStackDecision::Skip);
    }
  }

  #[test]
  fn owned_window_joins_when_allowed() {
    let rules = [AutoStackRuleConfig {
      allow_owned: true,
      ..details_rule()
    }];
    let props = window("MyApp", "Details for item 42");
    let traits = WindowTraits {
      has_owner: true,
      ..WindowTraits::default()
    };

    assert_eq!(
      decide(&rules, &props, traits),
      AutoStackDecision::Join(&rules[0])
    );
  }
}
