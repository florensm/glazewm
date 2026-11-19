use std::{
  collections::VecDeque,
  time::{Duration, Instant},
};

use wm_platform::WindowId;

/// How long a focus request waits for its focus event. The OS reports
/// one within milliseconds, but the WM may be busy drawing an animation
/// before it reads it; a request past this is taken to have none coming.
const MAX_AGE: Duration = Duration::from_secs(1);

/// Windows the WM asked the OS to focus, whose focus event hasn't come in
/// yet.
///
/// The OS reports a foreground change asynchronously, so by the time the
/// event for one arrives the WM may have moved focus on, e.g. on a second
/// quick `focus --direction` press. That event is the echo of the earlier
/// request, not the user focusing the window, and must not pull focus
/// back to it.
#[derive(Debug, Default)]
pub struct FocusRequests {
  /// Requested windows, oldest first.
  pending: VecDeque<(WindowId, Instant)>,
}

impl FocusRequests {
  /// Records that the WM asked the OS to focus `window` at `now`.
  ///
  /// Only for a window that isn't the foreground already: the OS reports
  /// no change then, so the request would wait for an event that never
  /// comes.
  pub fn record(&mut self, window: WindowId, now: Instant) {
    self.prune(now);
    self.pending.push_back((window, now));
  }

  /// Takes the latest request for `window`, along with any older ones,
  /// whose events were dropped or coalesced. Returns whether there was
  /// one, i.e. whether a focus event for `window` is the WM's own.
  pub fn take(&mut self, window: WindowId, now: Instant) -> bool {
    self.prune(now);

    let Some(index) =
      self.pending.iter().rposition(|(id, _)| *id == window)
    else {
      return false;
    };

    self.pending.drain(..=index);
    true
  }

  fn prune(&mut self, now: Instant) {
    self
      .pending
      .retain(|(_, requested)| now.duration_since(*requested) < MAX_AGE);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn takes_only_requested_windows() {
    let now = Instant::now();
    let mut requests = FocusRequests::default();
    requests.record(WindowId(1), now);

    assert!(!requests.take(WindowId(2), now));
    assert!(requests.take(WindowId(1), now));
    assert!(!requests.take(WindowId(1), now));
  }

  /// The echo of a request clears older ones, whose events won't come.
  #[test]
  fn take_clears_older_requests() {
    let now = Instant::now();
    let mut requests = FocusRequests::default();
    requests.record(WindowId(1), now);
    requests.record(WindowId(2), now);
    requests.record(WindowId(3), now);

    assert!(requests.take(WindowId(2), now));
    assert!(!requests.take(WindowId(1), now));
    assert!(requests.take(WindowId(3), now));
  }

  /// A request focused twice is taken through its latest one.
  #[test]
  fn take_goes_through_latest_request() {
    let now = Instant::now();
    let mut requests = FocusRequests::default();
    requests.record(WindowId(1), now);
    requests.record(WindowId(2), now);
    requests.record(WindowId(1), now);

    assert!(requests.take(WindowId(1), now));
    assert!(!requests.take(WindowId(2), now));
  }

  #[test]
  fn requests_expire() {
    let now = Instant::now();
    let mut requests = FocusRequests::default();
    requests.record(WindowId(1), now);

    assert!(!requests.take(WindowId(1), now + MAX_AGE));
  }
}
