use std::{
  sync::{Arc, Mutex, PoisonError},
  time::{Duration, Instant},
};

use wm_platform::Dispatcher;

/// How long after its dialog is dismissed the same error stays silent.
const REPEAT_INTERVAL: Duration = Duration::from_secs(60);

/// Shows non-fatal errors to the user without blocking the WM thread.
///
/// The dialog runs on a thread of its own, at most one at a time; errors
/// raised while it is open are only logged. An error identical to the
/// last one shown is not shown again within [`REPEAT_INTERVAL`] of that
/// dialog being dismissed: one failing on every animation frame would
/// otherwise come straight back. Every error is still logged.
pub struct ErrorNotifier {
  dispatcher: Dispatcher,
  dialog: Arc<Mutex<DialogState>>,
}

#[derive(Default)]
struct DialogState {
  is_open: bool,

  /// Message of the last dialog, and when it was dismissed.
  last_dismissed: Option<(String, Instant)>,
}

impl ErrorNotifier {
  pub fn new(dispatcher: Dispatcher) -> Self {
    Self {
      dispatcher,
      dialog: Arc::default(),
    }
  }

  /// Logs `err` and shows it in a dialog, unless one is already open or
  /// it was just dismissed.
  pub fn report(&self, err: &anyhow::Error) {
    tracing::error!("{err:?}");

    let message = err.to_string();

    {
      let mut dialog = lock(&self.dialog);
      if dialog.is_open
        || !should_show(
          dialog.last_dismissed.as_ref(),
          &message,
          Instant::now(),
        )
      {
        return;
      }
      dialog.is_open = true;
    }

    let dispatcher = self.dispatcher.clone();
    let dialog = self.dialog.clone();

    let spawned = std::thread::Builder::new()
      .name("glazewm-error-dialog".into())
      .spawn(move || {
        dispatcher.show_error_dialog("Non-fatal error", &message);

        let mut dialog = lock(&dialog);
        dialog.is_open = false;
        dialog.last_dismissed = Some((message, Instant::now()));
      });

    if let Err(err) = spawned {
      tracing::warn!("Failed to show error dialog: {err}");
      lock(&self.dialog).is_open = false;
    }
  }
}

/// Locks `dialog`. It only holds plain values, so a poisoned lock is still
/// valid.
fn lock(
  dialog: &Mutex<DialogState>,
) -> std::sync::MutexGuard<'_, DialogState> {
  dialog.lock().unwrap_or_else(PoisonError::into_inner)
}

fn should_show(
  last_dismissed: Option<&(String, Instant)>,
  message: &str,
  now: Instant,
) -> bool {
  last_dismissed.is_none_or(|(last, at)| {
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
