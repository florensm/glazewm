use std::collections::{HashMap, HashSet};

use wm_platform::WindowId;

/// How often windows are re-enabled for the same popup before giving up,
/// so an app that keeps disabling them (e.g. while busy) wins.
const MAX_ATTEMPTS_PER_POPUP: u32 = 3;

/// A managed window, as seen when a popup appears.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
  pub id: WindowId,
  pub process_id: u32,
  pub is_stacked: bool,
  pub is_enabled: bool,
}

/// Windows marked by the `stay-interactive` window rule command, which
/// are kept usable while their app shows a blocking popup.
///
/// Apps such as WPF disable all their windows while a modal popup is
/// open. Stacked windows were usable meanwhile in tabbing apps that
/// embedded them as child windows, which an app doesn't disable; this
/// gives stacked top-level windows the same behavior by re-enabling them.
#[derive(Default)]
pub struct StayInteractive {
  marked: HashSet<WindowId>,

  /// Re-enable rounds done per popup.
  attempts: HashMap<WindowId, u32>,
}

impl StayInteractive {
  pub fn mark(&mut self, id: WindowId) {
    self.marked.insert(id);
  }

  pub fn is_marked(&self, id: WindowId) -> bool {
    self.marked.contains(&id)
  }

  /// Unmarks every window.
  pub fn unmark_all(&mut self) {
    self.marked.clear();
    self.attempts.clear();
  }

  /// Drops everything known about a destroyed window.
  pub fn forget(&mut self, id: WindowId) {
    self.marked.remove(&id);
    self.attempts.remove(&id);
  }

  /// Windows to re-enable now that `popup` of process `process_id` is
  /// shown or focused.
  ///
  /// Only marked windows in a stack, of the popup's process, that are
  /// disabled. A marked stacked window is never itself treated as the
  /// popup.
  pub fn windows_to_enable(
    &mut self,
    popup: WindowId,
    process_id: u32,
    candidates: &[Candidate],
  ) -> Vec<WindowId> {
    let is_stacked_tab = candidates
      .iter()
      .any(|c| c.id == popup && c.is_stacked && self.is_marked(c.id));

    if is_stacked_tab {
      return Vec::new();
    }

    let targets = candidates
      .iter()
      .filter(|c| {
        c.id != popup
          && c.process_id == process_id
          && c.is_stacked
          && !c.is_enabled
          && self.is_marked(c.id)
      })
      .map(|c| c.id)
      .collect::<Vec<_>>();

    if targets.is_empty() {
      return targets;
    }

    let attempts = self.attempts.entry(popup).or_insert(0);
    if *attempts >= MAX_ATTEMPTS_PER_POPUP {
      return Vec::new();
    }

    *attempts += 1;
    targets
  }
}

#[cfg(test)]
mod tests {
  use wm_platform::WindowId;

  use super::{Candidate, StayInteractive};

  const POPUP: WindowId = WindowId(100);

  fn tab(id: isize, is_enabled: bool) -> Candidate {
    Candidate {
      id: WindowId(id),
      process_id: 7,
      is_stacked: true,
      is_enabled,
    }
  }

  fn marked(ids: &[isize]) -> StayInteractive {
    let mut stay = StayInteractive::default();
    for id in ids {
      stay.mark(WindowId(*id));
    }
    stay
  }

  #[test]
  fn reenables_disabled_marked_tabs_of_the_popups_process() {
    let mut stay = marked(&[1, 2, 3]);
    let other_process = Candidate {
      process_id: 8,
      ..tab(3, false)
    };

    let targets = stay.windows_to_enable(
      POPUP,
      7,
      &[tab(1, false), tab(2, true), other_process],
    );

    assert_eq!(targets, vec![WindowId(1)]);
  }

  #[test]
  fn ignores_unmarked_and_unstacked_windows() {
    let mut stay = marked(&[2]);
    let unstacked = Candidate {
      is_stacked: false,
      ..tab(2, false)
    };

    assert!(stay
      .windows_to_enable(POPUP, 7, &[tab(1, false), unstacked])
      .is_empty());
  }

  #[test]
  fn unmarked_windows_are_left_disabled() {
    let mut stay = marked(&[1]);
    stay.unmark_all();

    assert!(stay
      .windows_to_enable(POPUP, 7, &[tab(1, false)])
      .is_empty());
  }

  #[test]
  fn a_stacked_tab_is_not_a_popup() {
    let mut stay = marked(&[1, 2]);

    assert!(stay
      .windows_to_enable(WindowId(1), 7, &[tab(1, true), tab(2, false)])
      .is_empty());
  }

  #[test]
  fn backs_off_after_repeated_attempts() {
    let mut stay = marked(&[1]);

    for _ in 0..3 {
      assert_eq!(
        stay.windows_to_enable(POPUP, 7, &[tab(1, false)]).len(),
        1
      );
    }

    assert!(stay
      .windows_to_enable(POPUP, 7, &[tab(1, false)])
      .is_empty());

    // A new popup gets its own attempts.
    assert_eq!(
      stay
        .windows_to_enable(WindowId(101), 7, &[tab(1, false)])
        .len(),
      1
    );
  }
}

/// Re-enables the marked stacked windows that `popup`'s app disabled, if
/// `popup` is a shown, usable window of that app, and keeps the popup on
/// top so it can't get lost behind them.
///
/// Called when a window is shown or focused: apps disable their windows
/// before showing a modal popup, so both events come after the disable.
pub fn keep_stacked_windows_interactive(
  popup: &wm_platform::NativeWindow,
  state: &mut crate::wm_state::WmState,
  config: &mut crate::user_config::UserConfig,
) -> anyhow::Result<()> {
  use wm_common::{InvokeCommand, WindowState};
  use wm_platform::NativeWindowWindowsExt;

  use crate::{
    traits::{CommonGetters, WindowGetters},
    wm::WindowManager,
  };

  // `WS_VISIBLE` rather than `is_visible`: the WM itself keeps a new popup
  // cloaked until it is placed.
  if !popup.is_enabled()
    || !popup.has_window_style(wm_platform::WS_VISIBLE)
  {
    return Ok(());
  }

  let windows = state.windows();
  let candidates = windows
    .iter()
    .filter(|window| {
      state.stay_interactive.is_marked(window.native().id())
    })
    .map(|window| Candidate {
      id: window.native().id(),
      process_id: window.native().process_id(),
      is_stacked: window
        .parent()
        .is_some_and(|parent| parent.as_stack().is_some()),
      is_enabled: window.native().is_enabled(),
    })
    .collect::<Vec<_>>();

  if candidates.is_empty() {
    return Ok(());
  }

  let targets = state.stay_interactive.windows_to_enable(
    popup.id(),
    popup.process_id(),
    &candidates,
  );

  if targets.is_empty() {
    return Ok(());
  }

  for window in windows
    .iter()
    .filter(|w| targets.contains(&w.native().id()))
  {
    tracing::info!("Re-enabling stacked window for popup: {window}");
    window.native().enable_async();
  }

  // Clicking a re-enabled window would otherwise raise it over a popup
  // that isn't shown on top.
  if let Some(popup) = state.window_from_native(popup) {
    let is_on_top =
      matches!(popup.state(), WindowState::Floating(c) if c.shown_on_top);

    if !is_on_top {
      WindowManager::run_command(
        &InvokeCommand::SetFloating {
          shown_on_top: Some(true),
          centered: None,
          x_pos: None,
          y_pos: None,
          width: None,
          height: None,
        },
        popup.into(),
        state,
        config,
      )?;
    }
  }

  Ok(())
}
