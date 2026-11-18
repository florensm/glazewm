use std::collections::HashSet;

use uuid::Uuid;

/// Overlays another program draws over the desktop, e.g. a bar's own
/// overview, each held by one IPC connection.
///
/// While any is held, the WM treats the desktop as it does under its own
/// overview: changes land without animations, nothing of its own is drawn
/// over the overlay, and the foreground window is left to it.
#[derive(Debug, Default)]
pub struct OverlayHolds {
  holds: HashSet<Uuid>,
}

impl OverlayHolds {
  /// Returns whether `id` was not held already.
  pub fn hold(&mut self, id: Uuid) -> bool {
    self.holds.insert(id)
  }

  /// Returns whether `id` was held.
  pub fn release(&mut self, id: Uuid) -> bool {
    self.holds.remove(&id)
  }

  pub fn is_held(&self) -> bool {
    !self.holds.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn held_until_every_hold_is_released() {
    let mut holds = OverlayHolds::default();
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    assert!(!holds.is_held());

    assert!(holds.hold(a));
    assert!(holds.hold(b));
    assert!(!holds.hold(a));

    assert!(holds.release(a));
    assert!(holds.is_held());
    assert!(!holds.release(a));

    assert!(holds.release(b));
    assert!(!holds.is_held());
  }
}
