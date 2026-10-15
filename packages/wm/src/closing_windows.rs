use std::time::{Duration, Instant};

use uuid::Uuid;

/// How long a window may stay shown after `WM_CLOSE` before the close is
/// taken as refused, e.g. by an in-window "Save changes?" prompt.
///
/// Apps that close normally destroy or hide their window well within
/// this. A modal prompt is caught sooner, since it disables its owner.
const CLOSE_CONFIRM_TIMEOUT: Duration = Duration::from_millis(1000);

/// How long a close animation may run before `WM_CLOSE` is sent without
/// it, in case the animation was dropped or stalled.
const ANIMATION_MAX_WAIT: Duration = Duration::from_secs(3);

/// Interval at which tracked windows are re-checked.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// State of a closing window, as read from the OS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowProbe {
  /// Whether the window still exists.
  pub is_alive: bool,

  /// Whether the window has `WS_VISIBLE`, regardless of its cloak.
  pub is_shown: bool,

  /// Whether the window accepts input. A modal dialog disables its owner.
  pub is_enabled: bool,
}

/// What became of a window the WM asked to close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseOutcome {
  /// The window was destroyed.
  Closed,

  /// The app hid the window instead, e.g. to its tray icon.
  Hidden,

  /// The window stayed open, e.g. behind a "Save changes?" prompt.
  Refused,
}

/// Step for the caller to carry out on a closing window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosingAction {
  /// The close animation never finished; send `WM_CLOSE` now.
  SendClose { id: Uuid, handle: isize },

  /// The app acted on the close, or refused it.
  Resolved {
    id: Uuid,
    handle: isize,
    outcome: CloseOutcome,
  },
}

#[derive(Debug)]
struct ClosingWindow {
  id: Uuid,
  handle: isize,
  tracked_at: Instant,

  /// When `WM_CLOSE` was sent. `None` while the close animation plays.
  close_sent_at: Option<Instant>,
}

/// Windows detached from the layout for a close animation, tracked until
/// their app destroys or hides them.
///
/// A closing window is cloaked and out of the tree, so without this a
/// window whose app refuses the close (or hides it to a tray icon) would
/// stay invisible and unmanaged for good.
#[derive(Debug, Default)]
pub struct ClosingWindows {
  entries: Vec<ClosingWindow>,
  next_check: Option<Instant>,
}

impl ClosingWindows {
  /// Starts tracking a window whose close animation just began.
  pub fn track(&mut self, id: Uuid, handle: isize, now: Instant) {
    if self.entries.iter().any(|entry| entry.id == id) {
      return;
    }

    self.entries.push(ClosingWindow {
      id,
      handle,
      tracked_at: now,
      close_sent_at: None,
    });

    self.next_check.get_or_insert(now + POLL_INTERVAL);
  }

  /// Records that the close animation finished and sent `WM_CLOSE`.
  pub fn mark_close_sent(&mut self, id: Uuid, now: Instant) {
    if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
      entry.close_sent_at.get_or_insert(now);
    }
  }

  /// When [`poll`](Self::poll) is next due, if anything is tracked.
  pub fn next_check(&self) -> Option<Instant> {
    self.next_check
  }

  /// Checks every tracked window and returns the steps to take. Resolved
  /// windows stop being tracked.
  pub fn poll(
    &mut self,
    now: Instant,
    probe: impl Fn(isize) -> WindowProbe,
  ) -> Vec<ClosingAction> {
    let mut actions = Vec::new();

    self.entries.retain_mut(|entry| {
      if let Some(outcome) = Self::outcome(entry, probe(entry.handle), now)
      {
        actions.push(ClosingAction::Resolved {
          id: entry.id,
          handle: entry.handle,
          outcome,
        });
        return false;
      }

      if entry.close_sent_at.is_none()
        && now.duration_since(entry.tracked_at) >= ANIMATION_MAX_WAIT
      {
        entry.close_sent_at = Some(now);
        actions.push(ClosingAction::SendClose {
          id: entry.id,
          handle: entry.handle,
        });
      }

      true
    });

    self.next_check =
      (!self.entries.is_empty()).then_some(now + POLL_INTERVAL);

    actions
  }

  /// Stops tracking every window and returns their handles.
  pub fn drain(&mut self) -> Vec<isize> {
    self.next_check = None;
    self.entries.drain(..).map(|entry| entry.handle).collect()
  }

  fn outcome(
    entry: &ClosingWindow,
    probe: WindowProbe,
    now: Instant,
  ) -> Option<CloseOutcome> {
    if !probe.is_alive {
      return Some(CloseOutcome::Closed);
    }

    // Only a window that was sent `WM_CLOSE` can have acted on it.
    let sent_at = entry.close_sent_at?;

    if !probe.is_shown {
      Some(CloseOutcome::Hidden)
    } else if !probe.is_enabled
      || now.duration_since(sent_at) >= CLOSE_CONFIRM_TIMEOUT
    {
      Some(CloseOutcome::Refused)
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SHOWN: WindowProbe = WindowProbe {
    is_alive: true,
    is_shown: true,
    is_enabled: true,
  };

  fn tracked(now: Instant) -> (ClosingWindows, Uuid) {
    let mut windows = ClosingWindows::default();
    let id = Uuid::new_v4();
    windows.track(id, 42, now);
    (windows, id)
  }

  fn resolved(id: Uuid, outcome: CloseOutcome) -> Vec<ClosingAction> {
    vec![ClosingAction::Resolved {
      id,
      handle: 42,
      outcome,
    }]
  }

  #[test]
  fn waits_while_animation_plays() {
    let now = Instant::now();
    let (mut windows, _) = tracked(now);

    let actions =
      windows.poll(now + Duration::from_millis(100), |_| WindowProbe {
        is_enabled: false,
        ..SHOWN
      });

    assert!(actions.is_empty());
    assert!(windows.next_check().is_some());
  }

  #[test]
  fn destroyed_window_is_closed() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);
    windows.mark_close_sent(id, now);

    let actions = windows.poll(now, |_| WindowProbe {
      is_alive: false,
      ..SHOWN
    });

    assert_eq!(actions, resolved(id, CloseOutcome::Closed));
    assert_eq!(windows.next_check(), None);
  }

  #[test]
  fn hidden_window_is_hidden() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);
    windows.mark_close_sent(id, now);

    let actions = windows.poll(now, |_| WindowProbe {
      is_shown: false,
      ..SHOWN
    });

    assert_eq!(actions, resolved(id, CloseOutcome::Hidden));
  }

  #[test]
  fn modal_prompt_refuses_immediately() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);
    windows.mark_close_sent(id, now);

    let actions = windows.poll(now, |_| WindowProbe {
      is_enabled: false,
      ..SHOWN
    });

    assert_eq!(actions, resolved(id, CloseOutcome::Refused));
  }

  #[test]
  fn shown_window_refuses_after_timeout() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);
    windows.mark_close_sent(id, now);

    let before = windows.poll(now + CLOSE_CONFIRM_TIMEOUT / 2, |_| SHOWN);
    let after = windows.poll(now + CLOSE_CONFIRM_TIMEOUT, |_| SHOWN);

    assert!(before.is_empty());
    assert_eq!(after, resolved(id, CloseOutcome::Refused));
  }

  #[test]
  fn stalled_animation_sends_close_once() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);

    let first = windows.poll(now + ANIMATION_MAX_WAIT, |_| SHOWN);
    let second =
      windows.poll(now + ANIMATION_MAX_WAIT + POLL_INTERVAL, |_| SHOWN);

    assert_eq!(first, vec![ClosingAction::SendClose { id, handle: 42 }]);
    assert!(second.is_empty());
  }

  #[test]
  fn tracking_twice_keeps_one_entry() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);
    windows.track(id, 42, now);

    assert_eq!(windows.drain(), vec![42]);
    assert_eq!(windows.next_check(), None);
  }
}
