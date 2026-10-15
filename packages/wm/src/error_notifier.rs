use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

use wm_platform::Dispatcher;

/// How long an error identical to the last one shown stays silent.
const REPEAT_INTERVAL: Duration = Duration::from_secs(60);

/// Shows non-fatal errors to the user without blocking the WM thread.
///
/// The dialog runs on a thread of its own, at most one at a time, and an
/// error identical to the last one shown is not shown again within
/// [`REPEAT_INTERVAL`]. An error that fails on every animation frame would
/// otherwise bring its dialog back the moment it is dismissed. Every
/// error is still logged.
pub struct ErrorNotifier {
  dispatcher: Dispatcher,
  is_open: Arc<AtomicBool>,
  last_shown: Option<(String, Instant)>,
}

impl ErrorNotifier {
  pub fn new(dispatcher: Dispatcher) -> Self {
    Self {
      dispatcher,
      is_open: Arc::new(AtomicBool::new(false)),
      last_shown: None,
    }
  }

  /// Logs `err` and shows it in a dialog, unless one is already open or
  /// it was just shown.
  pub fn report(&mut self, err: &anyhow::Error) {
    tracing::error!("{err:?}");

    let message = err.to_string();
    let now = Instant::now();

    if !should_show(self.last_shown.as_ref(), &message, now)
      || self.is_open.swap(true, Ordering::AcqRel)
    {
      return;
    }

    self.last_shown = Some((message.clone(), now));

    let dispatcher = self.dispatcher.clone();
    let is_open = self.is_open.clone();

    let spawned = std::thread::Builder::new()
      .name("glazewm-error-dialog".into())
      .spawn(move || {
        dispatcher.show_error_dialog("Non-fatal error", &message);
        is_open.store(false, Ordering::Release);
      });

    if let Err(err) = spawned {
      tracing::warn!("Failed to show error dialog: {err}");
      self.is_open.store(false, Ordering::Release);
    }
  }
}

fn should_show(
  last_shown: Option<&(String, Instant)>,
  message: &str,
  now: Instant,
) -> bool {
  last_shown.is_none_or(|(last, at)| {
    last != message || now.duration_since(*at) >= REPEAT_INTERVAL
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shows_first_error() {
    assert!(should_show(None, "a", Instant::now()));
  }

  #[test]
  fn silences_repeat_until_interval_passes() {
    let now = Instant::now();
    let last = ("a".to_string(), now);

    assert!(!should_show(Some(&last), "a", now + REPEAT_INTERVAL / 2));
    assert!(should_show(Some(&last), "a", now + REPEAT_INTERVAL));
  }

  #[test]
  fn shows_different_error() {
    let now = Instant::now();
    let last = ("a".to_string(), now);

    assert!(should_show(Some(&last), "b", now));
  }
}
