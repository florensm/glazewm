use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use wm_common::{AutoStackRuleConfig, InvokeCommand, WindowMatchConfig};
use wm_platform::{Keybinding, NativeWindow, Rect, WindowId};

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

  /// The title is still empty but the rest of this rule (the first such
  /// one) matches, so the window could still turn out to belong in its
  /// stack.
  Wait(&'a AutoStackRuleConfig),

  /// A rule matches, but the window is of a kind that is never stacked.
  /// Carries the reason, for logging.
  Blocked(&'static str),

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
  let mut could_match_later = None;

  for rule in rules {
    let matches_any = |configs: &[WindowMatchConfig]| {
      configs
        .iter()
        .any(|config| UserConfig::window_matches(config, props))
    };

    if matches_any(&rule.exclude) {
      continue;
    }

    let blocked_reason = if traits.is_dialog {
      Some("it is a dialog")
    } else if traits.is_tool_window {
      Some("it is a tool window")
    } else if traits.has_owner && !rule.allow_owned {
      Some("it has an owner window (set `allow_owned: true` to allow)")
    } else {
      None
    };

    if let Some(reason) = blocked_reason {
      if matches_any(&rule.match_window) {
        return AutoStackDecision::Blocked(reason);
      }

      continue;
    }

    if matches_any(&rule.match_window) {
      return AutoStackDecision::Join(rule);
    }

    let matches_untitled = || {
      rule.match_window.iter().any(|config| {
        let untitled = WindowMatchConfig {
          window_title: None,
          ..config.clone()
        };

        UserConfig::window_matches(&untitled, props)
      })
    };

    if could_match_later.is_none()
      && props.title.trim().is_empty()
      && matches_untitled()
    {
      could_match_later = Some(rule);
    }
  }

  could_match_later
    .map_or(AutoStackDecision::Skip, AutoStackDecision::Wait)
}

/// Whether an untitled window at `frame` is big enough to join `rule`'s
/// stack before it has a title, per `join_untitled`.
///
/// Percentages are of `monitor_area`, and pixels are scaled by
/// `scale_factor`.
pub fn joins_untitled(
  rule: &AutoStackRuleConfig,
  frame: &Rect,
  monitor_area: &Rect,
  scale_factor: f32,
) -> bool {
  rule.join_untitled.as_ref().is_some_and(|join| {
    frame.width()
      >= join
        .min_width
        .to_px(monitor_area.width(), Some(scale_factor))
      && frame.height()
        >= join
          .min_height
          .to_px(monitor_area.height(), Some(scale_factor))
  })
}

/// A window that joined a stack before it had a title.
pub struct ProvisionalJoin {
  /// Name of the stack it joined.
  pub stack_name: String,

  /// Commands of its window rules that would have placed it elsewhere,
  /// skipped as it joined. Run if it leaves the stack again.
  pub skipped_commands: Vec<InvokeCommand>,
}

/// A window held back, cloaked and unmanaged, until its title is known.
struct HeldWindow {
  native: NativeWindow,
  deadline: Instant,
}

/// How long after joining a new window gets its `send_keys_on_join`, so
/// that the app is ready for input.
const SEND_KEYS_DELAY: Duration = Duration::from_millis(250);

/// How long `send_keys_on_join` keeps waiting for the window to be in the
/// foreground before giving up.
const SEND_KEYS_TIMEOUT: Duration = Duration::from_secs(5);

/// How long until a pending key press is tried again: the next key
/// combination, or a re-check for the foreground.
const SEND_KEYS_RETRY: Duration = Duration::from_millis(150);

/// Key combinations waiting to be pressed in a window that joined a stack.
pub struct PendingKeys {
  pub native: NativeWindow,
  pub keys: Vec<Keybinding>,

  /// Duplicate tabs closed once the keys are done, since closing them
  /// can take the foreground away from `native` before the keys are
  /// pressed.
  pub then_close: Vec<NativeWindow>,

  due: Instant,
  give_up: Instant,
}

impl PendingKeys {
  /// Whether it is too late to retry.
  pub fn is_expired(&self, now: Instant) -> bool {
    now >= self.give_up
  }
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

  /// Windows in a stack whose title, once they get one, decides whether
  /// they stay. See `join_untitled`.
  provisional: HashMap<WindowId, ProvisionalJoin>,

  /// Key combinations to press in windows that just joined a stack.
  pending_keys: Vec<PendingKeys>,
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

  /// Earliest deadline of the held windows and pending key presses.
  pub fn next_deadline(&self) -> Option<Instant> {
    self
      .held
      .values()
      .map(|held| held.deadline)
      .chain(self.pending_keys.iter().map(|pending| pending.due))
      .min()
  }

  /// Presses `keys` in `native` shortly, once it is in the foreground.
  pub fn queue_keys(
    &mut self,
    native: &NativeWindow,
    keys: Vec<Keybinding>,
    then_close: Vec<NativeWindow>,
  ) {
    let now = Instant::now();
    self.pending_keys.push(PendingKeys {
      native: native.clone(),
      keys,
      then_close,
      due: now + SEND_KEYS_DELAY,
      give_up: now + SEND_KEYS_TIMEOUT,
    });
  }

  /// Removes and returns the key presses that are due.
  pub fn take_due_keys(&mut self, now: Instant) -> Vec<PendingKeys> {
    let (due, waiting) = std::mem::take(&mut self.pending_keys)
      .into_iter()
      .partition(|pending| pending.due <= now);
    self.pending_keys = waiting;
    due
  }

  /// Tries `pending` again shortly, with the time limit it started with.
  pub fn retry_keys(&mut self, mut pending: PendingKeys, now: Instant) {
    pending.due = now + SEND_KEYS_RETRY;
    self.pending_keys.push(pending);
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
    self.provisional.remove(&id);
  }

  /// Records that `id` joined a stack before it had a title.
  pub fn join_provisionally(
    &mut self,
    id: WindowId,
    join: ProvisionalJoin,
  ) {
    self.provisional.insert(id, join);
  }

  pub fn provisional(&self, id: WindowId) -> Option<&ProvisionalJoin> {
    self.provisional.get(&id)
  }

  pub fn take_provisional(
    &mut self,
    id: WindowId,
  ) -> Option<ProvisionalJoin> {
    self.provisional.remove(&id)
  }

  /// Drops everything known about a destroyed window, since its handle
  /// can be reused by a new window.
  pub fn forget(&mut self, id: WindowId) {
    self.held.remove(&id);
    self.settled.remove(&id);
    self.provisional.remove(&id);
    self
      .pending_keys
      .retain(|pending| pending.native.id() != id);
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{
    AutoStackRuleConfig, DuplicateTabs, MatchType, WindowMatchConfig,
  };

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
      send_keys_on_join: Vec::new(),
      duplicates: DuplicateTabs::Keep,
      join_untitled: None,
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
      AutoStackDecision::Wait(&rules[0])
    );
    assert_eq!(
      decide(&rules, &window("notepad", ""), WindowTraits::default()),
      AutoStackDecision::Skip
    );
  }

  #[test]
  fn untitled_window_joins_if_big_enough() {
    use wm_common::JoinUntitledConfig;
    use wm_platform::{LengthValue, Rect};

    use super::joins_untitled;

    let monitor = Rect::from_xy(0, 0, 2000, 1000);
    let rule = AutoStackRuleConfig {
      join_untitled: Some(JoinUntitledConfig {
        min_width: LengthValue::from_px(600),
        min_height: "50%".parse().unwrap(),
      }),
      ..details_rule()
    };

    let joins = |width, height, scale| {
      joins_untitled(
        &rule,
        &Rect::from_xy(0, 0, width, height),
        &monitor,
        scale,
      )
    };

    assert!(joins(600, 500, 1.0));
    assert!(!joins(599, 500, 1.0));
    assert!(!joins(600, 499, 1.0));
    // Pixels scale with the monitor; percentages don't.
    assert!(!joins(800, 500, 1.5));
    assert!(joins(900, 500, 1.5));
    assert!(!joins_untitled(
      &details_rule(),
      &Rect::from_xy(0, 0, 2000, 1000),
      &monitor,
      1.0
    ));
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
      assert!(matches!(
        decide(&rules, &props, traits),
        AutoStackDecision::Blocked(_)
      ));
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

  #[test]
  fn pending_keys_wait_then_retry_until_they_expire() {
    use std::time::{Duration, Instant};

    use wm_platform::{Key, Keybinding, NativeWindow};

    use super::AutoStackState;

    let native = NativeWindow::mock();
    let mut auto_stack = AutoStackState::default();
    let start = Instant::now();
    auto_stack.queue_keys(
      &native,
      vec![Keybinding::new(vec![Key::Ctrl, Key::P]).unwrap()],
      Vec::new(),
    );

    assert!(auto_stack.next_deadline().is_some());
    assert!(auto_stack.take_due_keys(start).is_empty());

    let due = auto_stack.take_due_keys(start + Duration::from_secs(1));
    assert_eq!(due.len(), 1);
    assert!(auto_stack.next_deadline().is_none());

    let pending = due.into_iter().next().unwrap();
    assert!(!pending.is_expired(start + Duration::from_secs(1)));
    assert!(pending.is_expired(start + Duration::from_secs(6)));

    auto_stack.retry_keys(pending, start + Duration::from_secs(1));
    auto_stack.forget(native.id());
    assert!(auto_stack.next_deadline().is_none());
  }
}
