use std::time::Instant;

use wm_common::{
  EasingFunction, WindowTransitionParams, WindowTransitionStyle,
};
use wm_platform::Rect;

use crate::animation::state::WindowAnimationState;

/// Pixels a transit's clock travels per unit of its entry track.
///
/// The clock's rects stand for the amount in, so that its early
/// completion stays below a pixel (see `WindowAnimationState`) and a
/// spring's speed carries over through `WindowAnimationState::retargeted`.
const AMOUNT_SCALE: f32 = 10_000.0;

/// Shortest a transit lasts, unless its transition is instant: one frame
/// at 60 Hz.
const MIN_DURATION_MS: u32 = 17;

/// Which way a window's entry track runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
  In,
  Out,
}

impl Direction {
  /// The amount in the track runs to.
  fn end(self) -> f32 {
    match self {
      Self::In => 1.0,
      Self::Out => 0.0,
    }
  }

  fn opposite(self) -> Self {
    match self {
      Self::In => Self::Out,
      Self::Out => Self::In,
    }
  }
}

/// A window entering or leaving: zoomed, slid or faded by how far its
/// entry track has run.
#[derive(Clone, Debug)]
pub struct Transit {
  pub direction: Direction,
  pub style: WindowTransitionStyle,
  /// Opacity at the away end, as a fraction of the window's own.
  pub away_opacity: f32,
  /// How far in the window was when this transit began: 0 for an open,
  /// 1 for a close, anything between for one turned round mid-way.
  pub from: f32,
  /// Runs 0 to 1 over the transition's duration and easing. Its rects
  /// stand for the amount in (see [`AMOUNT_SCALE`]); only its progress is
  /// read.
  pub clock: WindowAnimationState,
}

impl Transit {
  /// A transit `direction` from `from` in, over `params`' duration scaled
  /// by how far it has to go (at least one frame). Its clock is unstarted.
  pub fn new(
    direction: Direction,
    params: &WindowTransitionParams,
    from: f32,
  ) -> Self {
    let clock = WindowAnimationState::new_movement(
      amount_rect(from),
      amount_rect(direction.end()),
      scaled_duration_ms(params.duration_ms, direction.end() - from),
      params.easing.clone(),
    );

    Self {
      direction,
      style: params.style.clone(),
      away_opacity: params.away_opacity,
      from,
      clock,
    }
  }

  /// How far in the window is at `now`: 0 away, 1 at rest.
  pub fn amount_at(&self, now: Instant) -> f32 {
    let progress = self.clock.eased_progress_at(now);

    match self.direction {
      Direction::In => self.from + (1.0 - self.from) * progress,
      Direction::Out => self.from * (1.0 - progress),
    }
  }

  pub fn is_complete(&self) -> bool {
    self.clock.is_complete()
  }

  /// The transit the other way, from where this one is at `now`, over
  /// `params`' duration scaled by how far it has left to go (at least
  /// one frame). Its clock is unstarted: the caller starts it at `now`.
  ///
  /// A spring keeps the speed this one has at `now`, so it carries on a
  /// moment before turning round; other easings start from rest.
  pub fn reversed(
    &self,
    params: &WindowTransitionParams,
    now: Instant,
  ) -> Self {
    let mut reversed =
      Self::new(self.direction.opposite(), params, self.amount_at(now));

    if matches!(params.easing, EasingFunction::Spring(_)) {
      reversed.clock = self.clock.retargeted(
        reversed.clock.target_rect.clone(),
        scaled_duration_ms(
          params.duration_ms,
          reversed.direction.end() - reversed.from,
        ),
        params.easing.clone(),
        now,
      );
    }

    reversed
  }
}

/// Where a window resting at `rest` is drawn when `amount` in, and at
/// what fraction of its opacity. `None` while it has no size.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn transit_frame(
  rest: &Rect,
  style: &WindowTransitionStyle,
  away_opacity: f32,
  amount: f32,
) -> Option<(Rect, f32)> {
  let (width, height) = (rest.width(), rest.height());
  if width <= 0 || height <= 0 {
    return None;
  }

  let amount = amount.clamp(0.0, 1.0);
  let away = |extent: i32| (extent as f32 * (1.0 - amount)).round() as i32;
  let moved = |dx: i32, dy: i32| {
    rest.translate_to_coordinates(rest.x() + dx, rest.y() + dy)
  };

  let frame = match style {
    WindowTransitionStyle::None => rest.clone(),
    WindowTransitionStyle::Zoom => {
      let inset = |extent: i32| {
        (extent as f32 * (1.0 - amount) / 2.0).round() as i32
      };
      let (dx, dy) = (inset(width), inset(height));
      let (left, top) = (rest.left + dx, rest.top + dy);
      let (right, bottom) = (rest.right - dx, rest.bottom - dy);
      if right <= left || bottom <= top {
        return None;
      }

      Rect::from_ltrb(left, top, right, bottom)
    }
    WindowTransitionStyle::SlideRight => moved(away(width), 0),
    WindowTransitionStyle::SlideLeft => moved(-away(width), 0),
    WindowTransitionStyle::SlideTop => moved(0, -away(height)),
    WindowTransitionStyle::SlideBottom => moved(0, away(height)),
  };

  let opacity =
    (away_opacity + (1.0 - away_opacity) * amount).clamp(0.0, 1.0);

  Some((frame, opacity))
}

/// A clock rect standing for `amount` in (see [`AMOUNT_SCALE`]).
#[allow(clippy::cast_possible_truncation)]
fn amount_rect(amount: f32) -> Rect {
  Rect::from_xy((amount * AMOUNT_SCALE).round() as i32, 0, 0, 0)
}

/// `duration_ms` scaled by `left`, the part of the track left to go, at
/// least [`MIN_DURATION_MS`] unless it is 0.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
fn scaled_duration_ms(duration_ms: u32, left: f32) -> u32 {
  if duration_ms == 0 {
    return 0;
  }

  let scaled = (duration_ms as f32 * left.abs().clamp(0.0, 1.0)).round();
  (scaled as u32).max(MIN_DURATION_MS)
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use wm_common::SpringParams;

  use super::*;

  fn rest() -> Rect {
    Rect::from_ltrb(100, 50, 900, 650)
  }

  fn frame_at(style: &WindowTransitionStyle, amount: f32) -> Option<Rect> {
    transit_frame(&rest(), style, 1.0, amount).map(|(frame, _)| frame)
  }

  fn opacity_at(away_opacity: f32, amount: f32) -> Option<f32> {
    transit_frame(
      &rest(),
      &WindowTransitionStyle::None,
      away_opacity,
      amount,
    )
    .map(|(_, opacity)| opacity)
  }

  fn params(easing: EasingFunction) -> WindowTransitionParams {
    WindowTransitionParams {
      duration_ms: 200,
      easing,
      style: WindowTransitionStyle::Zoom,
      away_opacity: 0.0,
    }
  }

  fn linear() -> EasingFunction {
    EasingFunction::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0)
  }

  fn assert_near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-3, "{actual} vs {expected}");
  }

  const SLIDES: [WindowTransitionStyle; 4] = [
    WindowTransitionStyle::SlideRight,
    WindowTransitionStyle::SlideLeft,
    WindowTransitionStyle::SlideTop,
    WindowTransitionStyle::SlideBottom,
  ];

  #[test]
  fn zoom_insets_from_the_centre() {
    let zoom = WindowTransitionStyle::Zoom;
    assert_eq!(frame_at(&zoom, 1.0), Some(rest()));
    assert_eq!(
      frame_at(&zoom, 0.5),
      Some(Rect::from_ltrb(300, 200, 700, 500))
    );
    assert_eq!(frame_at(&zoom, 0.0), None);

    let odd = Rect::from_xy(100, 50, 801, 601);
    assert_eq!(
      transit_frame(&odd, &zoom, 1.0, 1.0).map(|(frame, _)| frame),
      Some(odd)
    );
  }

  #[test]
  fn slides_come_in_from_their_edge() {
    assert_eq!(
      frame_at(&WindowTransitionStyle::SlideRight, 0.25),
      Some(Rect::from_ltrb(700, 50, 1500, 650))
    );
    assert_eq!(
      frame_at(&WindowTransitionStyle::SlideTop, 0.5),
      Some(Rect::from_ltrb(100, -250, 900, 350))
    );
    for slide in &SLIDES {
      assert_eq!(frame_at(slide, 1.0), Some(rest()), "{slide:?}");
    }
  }

  #[test]
  fn none_stays_at_rest() {
    for amount in [0.0, 0.3, 1.0] {
      assert_eq!(
        frame_at(&WindowTransitionStyle::None, amount),
        Some(rest())
      );
    }
  }

  #[test]
  fn opacity_runs_from_the_away_end() {
    assert!(opacity_at(0.0, 0.5).is_some_and(|o| (o - 0.5).abs() < 1e-6));
    for amount in [0.0, 0.5, 1.0] {
      assert!(
        opacity_at(1.0, amount).is_some_and(|o| (o - 1.0).abs() < 1e-6)
      );
    }
    assert!(opacity_at(0.4, 0.0).is_some_and(|o| (o - 0.4).abs() < 1e-6));
  }

  #[test]
  fn amount_runs_along_the_clock() {
    let t0 = Instant::now();
    let now = t0 + Duration::from_millis(50);
    let opening = Transit::new(Direction::In, &params(linear()), 0.0);
    let closing = Transit::new(Direction::Out, &params(linear()), 1.0);
    opening.clock.start_clock(t0);
    closing.clock.start_clock(t0);

    assert_near(opening.amount_at(now), 0.25);
    assert_near(closing.amount_at(now), 0.75);
  }

  #[test]
  fn reversed_turns_round_where_it_is() {
    let t0 = Instant::now();
    let at = |ms| t0 + Duration::from_millis(ms);
    let opening = Transit::new(Direction::In, &params(linear()), 0.0);
    opening.clock.start_clock(t0);

    let closing = opening.reversed(&params(linear()), at(50));
    assert_eq!(closing.direction, Direction::Out);
    assert_near(closing.from, 0.25);
    assert_eq!(closing.clock.duration, Duration::from_millis(50));

    // Reversed again before a frame: back in, from the same amount.
    let reopening = closing.reversed(&params(linear()), at(50));
    assert_eq!(reopening.direction, Direction::In);
    assert_near(reopening.from, 0.25);
    reopening.clock.start_clock(at(50));
    assert_near(reopening.amount_at(at(50)), 0.25);

    closing.clock.start_clock(at(50));
    assert_near(closing.amount_at(at(50)), 0.25);
    assert_near(closing.amount_at(at(75)), 0.125);
    assert_near(closing.amount_at(at(100)), 0.0);
    assert!(closing.clock.eased_progress_at(at(100)) >= 1.0);
  }

  #[test]
  fn reversed_spring_keeps_its_momentum() {
    let spring = || EasingFunction::Spring(SpringParams::default());
    let t0 = Instant::now();
    let now = t0 + Duration::from_millis(40);
    let opening = Transit::new(Direction::In, &params(spring()), 0.0);
    opening.clock.start_clock(t0);
    let amount = opening.amount_at(now);
    assert!(amount > 0.1 && amount < 0.9, "{amount}");

    let closing = opening.reversed(&params(spring()), now);
    let EasingFunction::Spring(spring) = closing.clock.easing else {
      panic!("not a spring");
    };
    // Its progress runs toward 0 in, so heading in still is backwards.
    assert!(spring.initial_velocity < 0.0);

    closing.clock.start_clock(now);
    assert_near(closing.amount_at(now), amount);
    assert!(closing.amount_at(now + Duration::from_millis(5)) > amount);
  }
}
