//! Fits eased motions to the cubic pieces DWM plays a scene's animations
//! from, so DWM moves things on its own clock exactly as `apply_easing`
//! would.

use std::time::Duration;

use wm_common::EasingFunction;

use super::engine::apply_easing;

/// Longest piece a motion is cut into.
const MAX_SEGMENT_SECS: f64 = 0.004;

/// Knots spread evenly along a bezier's own parameter, on top of the
/// evenly timed ones, which bunches them where it is steep in time.
const CURVE_KNOTS: u32 = 96;

/// Knots halving the distance toward each end of a bezier's parameter.
const END_KNOTS: i32 = 16;

/// Knots closer than this are merged: DWM takes coefficients as `f32`,
/// which shorter pieces would lose to rounding.
const MIN_KNOT_GAP_SECS: f64 = 1e-6;

/// Share of a piece the curve's slope at its ends is measured over.
const SLOPE_STEP_SHARE: f64 = 0.01;

/// One piece of a motion from 0 to 1: `u` seconds after `begin`, it is at
/// `c[0] + c[1] u + c[2] u² + c[3] u³`.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
  pub begin: f64,
  pub coefficients: [f64; 4],
}

/// `easing`'s progress from 0 to 1 over `duration`, as cubic Hermite
/// pieces through samples of it. Empty for an instant motion.
///
/// Each piece takes the curve's own slope at its ends. On a piece that
/// only rises or only falls, they are cut down (Fritsch-Carlson) where
/// they would carry it past its ends, so a steep start can't overshoot; a
/// piece holding a turn of the curve keeps them, to follow the turn. A
/// bezier gets extra samples along its own parameter: one starting
/// vertical, like Hyprland's `default`, grows like a square root there,
/// which evenly timed pieces can't follow.
#[allow(clippy::many_single_char_names)]
pub fn fit(easing: &EasingFunction, duration: Duration) -> Vec<Segment> {
  let duration_secs = duration.as_secs_f64();
  if duration_secs <= 0.0 {
    return Vec::new();
  }

  #[allow(clippy::cast_possible_truncation)]
  let at = |secs: f64| {
    f64::from(apply_easing(
      (secs / duration_secs).clamp(0.0, 1.0) as f32,
      easing,
    ))
  };

  let knots = knots(easing, duration_secs);
  let p = knots.iter().map(|&secs| at(secs)).collect::<Vec<_>>();
  let h = knots.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
  let d = (0..h.len())
    .map(|k| (p[k + 1] - p[k]) / h[k])
    .collect::<Vec<_>>();

  let m = slopes(&knots, &h, at);

  (0..h.len())
    .map(|k| {
      let (m0, m1) = limited(m[k], m[k + 1], d[k]);
      let (p0, p1, h) = (p[k], p[k + 1], h[k]);
      Segment {
        begin: knots[k],
        coefficients: [
          p0,
          m0,
          (3.0 * (p1 - p0) / h - 2.0 * m0 - m1) / h,
          (2.0 * (p0 - p1) / h + m0 + m1) / (h * h),
        ],
      }
    })
    .collect()
}

/// The curve's slope at each knot, by a difference over a sliver of the
/// pieces beside it: centered between two, one-sided at the ends.
fn slopes(knots: &[f64], h: &[f64], at: impl Fn(f64) -> f64) -> Vec<f64> {
  let last = h.len();

  (0..=last)
    .map(|k| {
      let before = if k > 0 { h[k - 1] } else { 0.0 };
      let after = if k < last { h[k] } else { 0.0 };
      let step = SLOPE_STEP_SHARE * before.max(after);
      let low = knots[k] - if k > 0 { step } else { 0.0 };
      let high = knots[k] + if k < last { step } else { 0.0 };
      (at(high) - at(low)) / (high - low)
    })
    .collect()
}

/// End slopes `m0` and `m1` of a piece with mean slope `secant`, cut down
/// per Fritsch-Carlson, unless the piece holds a turn.
fn limited(m0: f64, m1: f64, secant: f64) -> (f64, f64) {
  if secant == 0.0 || m0 * secant < 0.0 || m1 * secant < 0.0 {
    return (m0, m1);
  }

  let (a, b) = (m0 / secant, m1 / secant);
  let length = a.hypot(b);
  if length > 3.0 {
    (3.0 / length * a * secant, 3.0 / length * b * secant)
  } else {
    (m0, m1)
  }
}

/// Sample times, in seconds: evenly spaced, plus, for a bezier, spaced
/// along its parameter.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
fn knots(easing: &EasingFunction, duration_secs: f64) -> Vec<f64> {
  let count = (duration_secs / MAX_SEGMENT_SECS).ceil().max(1.0) as usize;
  let mut knots = (0..=count)
    .map(|k| duration_secs * k as f64 / count as f64)
    .collect::<Vec<_>>();

  if let EasingFunction::CubicBezier(x1, _, x2, _) = *easing {
    let (x1, x2) =
      (f64::from(x1.clamp(0.0, 1.0)), f64::from(x2.clamp(0.0, 1.0)));
    let x_at = |t: f64| {
      let u = 1.0 - t;
      3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t
    };

    let along = (1..CURVE_KNOTS)
      .map(|i| f64::from(i) / f64::from(CURVE_KNOTS))
      .chain((1..=END_KNOTS).flat_map(|k| {
        let near = 0.5_f64.powi(k);
        [near, 1.0 - near]
      }));
    knots.extend(along.map(|t| duration_secs * x_at(t)));
  }

  knots.sort_by(f64::total_cmp);
  knots.dedup_by(|a, b| (*a - *b).abs() < MIN_KNOT_GAP_SECS);
  knots
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use wm_common::{EasingFunction, SpringParams};

  use super::{fit, Segment};
  use crate::animation::engine::apply_easing;

  const HYPRLAND_DEFAULT: EasingFunction =
    EasingFunction::CubicBezier(0.0, 0.75, 0.15, 1.0);
  const EASE_OUT_BACK: EasingFunction =
    EasingFunction::CubicBezier(0.34, 1.56, 0.64, 1.0);
  const CSS_EASE: EasingFunction =
    EasingFunction::CubicBezier(0.25, 0.1, 0.25, 1.0);
  const LINEAR: EasingFunction =
    EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0);

  /// Value of `segments` `secs` into their motion, as DWM samples them.
  fn sampled(segments: &[Segment], secs: f64) -> f64 {
    segments
      .iter()
      .rev()
      .find(|segment| segment.begin <= secs)
      .map_or(f64::NAN, |segment| {
        let u = secs - segment.begin;
        let [c0, c1, c2, c3] = segment.coefficients;
        ((c3 * u + c2) * u + c1) * u + c0
      })
  }

  /// Pixels a monitor-wide slide is off from `easing` at worst, from
  /// `after` seconds on.
  ///
  /// Leaves out the last piece: springs snap to exactly 1 as they end,
  /// which it eases into instead.
  fn worst_error(easing: &EasingFunction, millis: u64, after: f64) -> f64 {
    let duration = Duration::from_millis(millis);
    let secs = duration.as_secs_f64();
    let segments = fit(easing, duration);
    let last_begins = segments.last().map_or(secs, |last| last.begin);

    (0..=60_000)
      .map(|i| secs * f64::from(i) / 60_000.0)
      .filter(|t| *t >= after && *t < last_begins)
      .map(|t| {
        #[allow(clippy::cast_possible_truncation)]
        let wanted = f64::from(apply_easing((t / secs) as f32, easing));
        (sampled(&segments, t) - wanted).abs() * 2560.0
      })
      .fold(0.0, f64::max)
  }

  #[test]
  fn fits_beziers_and_springs_within_half_a_pixel() {
    let spring = EasingFunction::Spring(SpringParams {
      damping_ratio: 0.5,
      ..SpringParams::default()
    });
    for easing in [
      EASE_OUT_BACK,
      CSS_EASE,
      LINEAR,
      EasingFunction::EaseOutSpring,
      EasingFunction::Spring(SpringParams::default()),
      spring,
    ] {
      let millis = u64::from(easing.duration_ms(300));
      let error = worst_error(&easing, millis, 0.0);
      assert!(error < 0.5, "{easing:?} is off by {error} px");
    }
  }

  /// A curve starting vertical can't be followed in its first
  /// microseconds, but DWM's first sample lands at the first composition
  /// after the commit, far later.
  #[test]
  fn follows_a_vertical_start_from_the_first_frame() {
    let error = worst_error(&HYPRLAND_DEFAULT, 300, 0.0005);
    assert!(error < 0.5, "off by {error} px");
  }

  #[test]
  fn monotone_easing_never_overshoots() {
    let segments = fit(&HYPRLAND_DEFAULT, Duration::from_millis(300));
    for i in 0..=6000 {
      let value = sampled(&segments, 0.3 * f64::from(i) / 6000.0);
      assert!((-1e-6..=1.0 + 1e-6).contains(&value), "{value}");
    }
  }

  #[test]
  fn pieces_join_and_end_at_one() {
    let segments = fit(&CSS_EASE, Duration::from_millis(250));
    for pair in segments.windows(2) {
      let joint = pair[1].begin;
      assert!(
        (sampled(&pair[..1], joint) - sampled(&pair[1..], joint)).abs()
          < 1e-9
      );
    }
    assert!((sampled(&segments, 0.25) - 1.0).abs() < 1e-6);
  }

  #[test]
  fn instant_motion_has_no_pieces() {
    assert!(fit(&CSS_EASE, Duration::ZERO).is_empty());
  }
}
