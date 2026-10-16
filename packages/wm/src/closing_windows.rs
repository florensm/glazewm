use std::time::{Duration, Instant};

use uuid::Uuid;

/// How long a window may stay shown after `WM_CLOSE` before the close is
/// taken as refused, e.g. by a "Save changes?" prompt drawn inside the
/// window itself.
///
/// Apps that close normally destroy or hide their window well within
/// this, and one that is merely slow comes back for a moment and is
/// unmanaged again once it does close. A prompt in a dialog of its own is
/// caught right away instead (see [`WindowProbe::has_modal_prompt`]).
const CLOSE_CONFIRM_TIMEOUT: Duration = Duration::from_secs(2);

/// How long past its expected end a close animation may run before
/// `WM_CLOSE` is sent without it, in case it was dropped or stalled.
const ANIMATION_GRACE: Duration = Duration::from_secs(2);

/// Interval at which tracked windows are re-checked.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// State of a closing window, as read from the OS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowProbe {
  /// Whether the window still exists.
  pub is_alive: bool,

  /// Whether the window has `WS_VISIBLE`, regardless of its cloak.
  pub is_shown: bool,

  /// Whether the window is disabled behind a shown popup of its own, the
  /// way a modal dialog leaves its owner.
  pub has_modal_prompt: bool,
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

  /// When `WM_CLOSE` is sent without the animation, if it has not
  /// finished by then.
  close_due: Instant,

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
  /// Starts tracking a window whose close animation, lasting
  /// `animation_duration`, just began.
  pub fn track(
    &mut self,
    id: Uuid,
    handle: isize,
    animation_duration: Duration,
    now: Instant,
  ) {
    if self.entries.iter().any(|entry| entry.id == id) {
      return;
    }

    self.entries.push(ClosingWindow {
      id,
      handle,
      close_due: now + animation_duration + ANIMATION_GRACE,
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

      if entry.close_sent_at.is_none() && now >= entry.close_due {
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
    } else if probe.has_modal_prompt
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
    has_modal_prompt: false,
  };

  const ANIMATION: Duration = Duration::from_millis(150);

  fn tracked(now: Instant) -> (ClosingWindows, Uuid) {
    let mut windows = ClosingWindows::default();
    let id = Uuid::new_v4();
    windows.track(id, 42, ANIMATION, now);
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

    let actions = windows.poll(now + ANIMATION, |_| WindowProbe {
      has_modal_prompt: true,
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
      has_modal_prompt: true,
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
    let due = now + ANIMATION + ANIMATION_GRACE;

    let early = windows.poll(now + ANIMATION, |_| SHOWN);
    let first = windows.poll(due, |_| SHOWN);
    let second = windows.poll(due + POLL_INTERVAL, |_| SHOWN);

    assert!(early.is_empty());
    assert_eq!(first, vec![ClosingAction::SendClose { id, handle: 42 }]);
    assert!(second.is_empty());
  }

  #[test]
  fn long_animation_is_not_cut_short() {
    let now = Instant::now();
    let mut windows = ClosingWindows::default();
    windows.track(Uuid::new_v4(), 42, Duration::from_secs(5), now);

    let actions = windows.poll(now + Duration::from_secs(6), |_| SHOWN);

    assert!(actions.is_empty());
  }

  #[test]
  fn tracking_twice_keeps_one_entry() {
    let now = Instant::now();
    let (mut windows, id) = tracked(now);
    windows.track(id, 42, ANIMATION, now);

    assert_eq!(windows.drain(), vec![42]);
    assert_eq!(windows.next_check(), None);
  }
}
