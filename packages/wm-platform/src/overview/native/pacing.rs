//! What a screen times its animations by, and how many cards it draws a
//! frame.

use std::time::Instant;

/// When the frame being drawn shows, which the zoom and the entrance are
/// timed by: the vblank estimate while animating, so they advance evenly
/// however late each wake-up is.
#[derive(Clone, Copy, Debug)]
pub(super) struct FrameTime(Instant);

impl FrameTime {
  pub fn new(at: Instant) -> Self {
    Self(at)
  }

  /// Moves on to the frame shown at `now`; never back, as a vblank
  /// estimate can be a little before a time read off the clock.
  pub fn advance(&mut self, now: Instant) {
    self.0 = self.0.max(now);
  }

  /// Starts something at `now`, e.g. a zoom on input. Returns when it
  /// started: `now` rather than the last frame's time, which is stale
  /// while nothing moved, and would have it seem long under way.
  pub fn start(&mut self, now: Instant) -> Instant {
    self.advance(now);
    now
  }

  /// Milliseconds from `started` to the frame being drawn.
  pub fn since(&self, started: Instant) -> f32 {
    self.0.saturating_duration_since(started).as_secs_f32() * 1000.0
  }
}

/// Most cards drawn in full in a frame, unless forced: each is a large
/// software drawing.
pub(super) const CARD_DRAWS_PER_FRAME: usize = 2;

/// Order to draw `count` cards in: `selected` first, as the one most
/// looked at, then those after it, then those before.
pub(super) fn draw_order(
  count: usize,
  selected: usize,
) -> impl Iterator<Item = usize> {
  let selected = selected.min(count.saturating_sub(1));
  (selected..count).chain(0..selected)
}

/// The cards a frame may still draw in full.
#[derive(Debug)]
pub(super) struct DrawBudget {
  left: Option<usize>,
  is_behind: bool,
}

impl DrawBudget {
  /// `CARD_DRAWS_PER_FRAME` draws, or any number when `force`.
  pub fn new(force: bool) -> Self {
    Self {
      left: (!force).then_some(CARD_DRAWS_PER_FRAME),
      is_behind: false,
    }
  }

  /// Takes a draw. Returns whether one was left; if not, the card waits
  /// for a later frame.
  pub fn take(&mut self) -> bool {
    match &mut self.left {
      None => true,
      Some(0) => {
        self.is_behind = true;
        false
      }
      Some(left) => {
        *left -= 1;
        true
      }
    }
  }

  /// Whether a card was left to draw on a later frame.
  pub fn is_behind(&self) -> bool {
    self.is_behind
  }
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use super::{draw_order, DrawBudget, FrameTime, CARD_DRAWS_PER_FRAME};

  /// A start after a pause counts from then, not from the last frame, and
  /// an earlier vblank estimate after it doesn't take the time back.
  #[test]
  fn a_start_after_a_pause_counts_from_then() {
    let frame = Instant::now();
    let mut time = FrameTime::new(frame);

    let later = frame + Duration::from_secs(5);
    let started = time.start(later);
    assert_eq!(started, later);
    assert!(time.since(started).abs() < f32::EPSILON);

    time.advance(frame + Duration::from_millis(4_997));
    assert!(time.since(started).abs() < f32::EPSILON, "never back");

    time.advance(later + Duration::from_millis(16));
    assert!((time.since(started) - 16.0).abs() < 0.01);
  }

  #[test]
  fn cards_are_drawn_selected_first_then_around() {
    assert_eq!(draw_order(4, 2).collect::<Vec<_>>(), vec![2, 3, 0, 1]);
    assert_eq!(draw_order(3, 9).collect::<Vec<_>>(), vec![2, 0, 1]);
    assert_eq!(draw_order(0, 0).count(), 0);
  }

  /// A frame draws a few cards and leaves the rest behind; forced, it
  /// draws them all.
  #[test]
  fn a_frame_draws_a_few_cards_unless_forced() {
    let mut budget = DrawBudget::new(false);
    for _ in 0..CARD_DRAWS_PER_FRAME {
      assert!(budget.take());
    }
    assert!(!budget.is_behind());
    assert!(!budget.take());
    assert!(budget.is_behind());

    let mut forced = DrawBudget::new(true);
    assert!((0..20).all(|_| forced.take()));
    assert!(!forced.is_behind());
  }
}
