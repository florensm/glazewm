//! Eased motion, and the cubic segments a `DirectComposition` animation
//! function approximates it with.

/// CSS-style cubic bezier easing between `(0, 0)` and `(1, 1)`.
#[derive(Clone, Copy, Debug)]
pub struct Bezier(pub [f32; 4]);

impl Bezier {
  pub const LINEAR: Bezier = Bezier([0.0, 0.0, 1.0, 1.0]);

  /// Point at parameter `t` of the curve, as linear and eased progress.
  fn point(self, t: f64) -> (f64, f64) {
    let [x1, y1, x2, y2] = self.0.map(f64::from);
    // As in CSS, control points can't leave the unit interval in `x`, so
    // `x(t)` stays monotonic.
    let (x1, x2) = (x1.clamp(0.0, 1.0), x2.clamp(0.0, 1.0));
    let u = 1.0 - t;
    let blend = |a: f64, b: f64| {
      3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    (blend(x1, x2), blend(y1, y2))
  }

  /// Eased progress at linear progress `x`.
  ///
  /// Solves `x(t)` by bisection rather than Newton's method, which stalls
  /// on curves that start vertical, like Hyprland's `default`.
  pub fn ease(self, x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..48 {
      let mid = f64::midpoint(low, high);
      if self.point(mid).0 < x {
        low = mid;
      } else {
        high = mid;
      }
    }

    self.point(f64::midpoint(low, high)).1
  }
}

/// A scalar moving from `from` to `to` over `duration_secs` along `curve`.
#[derive(Clone, Copy, Debug)]
pub struct Tween {
  pub from: f64,
  pub to: f64,
  pub duration_secs: f64,
  pub curve: Bezier,
}

/// One piece of an animation function: `secs` after `begin`, its value
/// is `c[0] + c[1] u + c[2] u² + c[3] u³` with `u = secs - begin`.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
  pub begin: f64,
  pub coefficients: [f64; 4],
}

#[cfg(test)]
impl Segment {
  pub fn value(&self, secs: f64) -> f64 {
    let u = secs - self.begin;
    let [c0, c1, c2, c3] = self.coefficients;
    ((c3 * u + c2) * u + c1) * u + c0
  }
}

impl Tween {
  /// Value `secs` into the tween.
  pub fn at(&self, secs: f64) -> f64 {
    let progress = if self.duration_secs > 0.0 {
      secs / self.duration_secs
    } else {
      1.0
    };
    self.from + (self.to - self.from) * self.curve.ease(progress)
  }

  /// Cubic Hermite pieces through samples at most `max_secs` apart, and
  /// closer where the curve is steep; empty for an instant tween.
  ///
  /// Tangents are Fritsch-Carlson's, so a stretch that only rises or only
  /// falls never overshoots.
  #[allow(clippy::many_single_char_names)]
  pub fn segments(&self, max_secs: f64) -> Vec<Segment> {
    if self.duration_secs <= 0.0 {
      return Vec::new();
    }

    let knots = self.knots(max_secs);
    let p = knots.iter().map(|&t| self.at(t)).collect::<Vec<_>>();
    let h = knots.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
    let d = (0..h.len())
      .map(|k| (p[k + 1] - p[k]) / h[k])
      .collect::<Vec<_>>();

    let last = h.len();
    let mut m = vec![0.0; last + 1];
    m[0] = d[0];
    m[last] = d[last - 1];
    for k in 1..last {
      if d[k - 1] * d[k] > 0.0 {
        m[k] = f64::midpoint(d[k - 1], d[k]);
      }
    }

    for k in 0..last {
      if d[k] == 0.0 {
        m[k] = 0.0;
        m[k + 1] = 0.0;
        continue;
      }

      let (a, b) = (m[k] / d[k], m[k + 1] / d[k]);
      let length = a.hypot(b);
      if length > 3.0 {
        m[k] = 3.0 / length * a * d[k];
        m[k + 1] = 3.0 / length * b * d[k];
      }
    }

    (0..last)
      .map(|k| {
        let (p0, p1, m0, m1, h) = (p[k], p[k + 1], m[k], m[k + 1], h[k]);
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

  /// Sample times: evenly spaced, plus evenly spaced in the bezier's own
  /// parameter, which bunches them where the curve is steep in time.
  ///
  /// A curve whose end is vertical (Hyprland's `default` starts so) grows
  /// like a square root there, which no cubic follows; ever closer knots
  /// toward each end keep the piece it is cut off in a few pixels long.
  #[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
  )]
  fn knots(&self, max_secs: f64) -> Vec<f64> {
    let count = (self.duration_secs / max_secs).ceil().max(1.0) as usize;
    let even =
      (0..=count).map(|k| self.duration_secs * k as f64 / count as f64);
    let along_curve = (1..CURVE_KNOTS)
      .map(|i| f64::from(i) / f64::from(CURVE_KNOTS))
      .chain((1..=END_KNOTS).flat_map(|k| {
        let near = 0.5_f64.powi(k);
        [near, 1.0 - near]
      }))
      .map(|t| self.duration_secs * self.curve.point(t).0);

    let mut knots = even.chain(along_curve).collect::<Vec<_>>();
    knots.sort_by(f64::total_cmp);
    knots.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    knots
  }
}

/// Knots placed along the bezier's parameter, on top of the evenly timed
/// ones.
const CURVE_KNOTS: u32 = 96;

/// Knots halving the distance toward each end of the bezier's parameter.
const END_KNOTS: i32 = 16;

#[cfg(test)]
mod tests {
  use super::{Bezier, Segment, Tween};

  const HYPRLAND_DEFAULT: Bezier = Bezier([0.0, 0.75, 0.15, 1.0]);
  const EASE_OUT_BACK: Bezier = Bezier([0.34, 1.56, 0.64, 1.0]);
  const CSS_EASE: Bezier = Bezier([0.25, 0.1, 0.25, 1.0]);

  fn slide(curve: Bezier) -> Tween {
    Tween {
      from: 0.0,
      to: -2560.0,
      duration_secs: 0.3,
      curve,
    }
  }

  /// Value of `segments` at `secs`, as DWM samples them.
  fn sampled(segments: &[Segment], secs: f64) -> f64 {
    segments
      .iter()
      .rev()
      .find(|segment| segment.begin <= secs)
      .map_or(f64::NAN, |segment| segment.value(secs))
  }

  /// Largest gap between `tween` and its segments from `after` on.
  fn worst_error(tween: &Tween, after: f64) -> f64 {
    let segments = tween.segments(0.004);
    (0..=120_000)
      .map(|i| tween.duration_secs * f64::from(i) / 120_000.0)
      .filter(|secs| *secs >= after)
      .map(|secs| (sampled(&segments, secs) - tween.at(secs)).abs())
      .fold(0.0, f64::max)
  }

  #[test]
  fn bezier_ends_at_its_anchors() {
    for curve in
      [HYPRLAND_DEFAULT, EASE_OUT_BACK, CSS_EASE, Bezier::LINEAR]
    {
      assert!(curve.ease(0.0).abs() < 1e-9);
      assert!((curve.ease(1.0) - 1.0).abs() < 1e-9);
    }
    assert!((Bezier::LINEAR.ease(0.3) - 0.3).abs() < 1e-9);
  }

  #[test]
  fn segments_stay_within_half_a_pixel() {
    for curve in [EASE_OUT_BACK, CSS_EASE, Bezier::LINEAR] {
      let error = worst_error(&slide(curve), 0.0);
      assert!(error < 0.5, "{curve:?} is off by {error} px");
    }
  }

  /// A curve starting vertical can't be followed in its first
  /// microseconds, but DWM's first sample lands at the first composition
  /// after the commit, far later.
  #[test]
  fn vertical_start_is_followed_from_the_first_frame() {
    let error = worst_error(&slide(HYPRLAND_DEFAULT), 0.0005);
    assert!(error < 0.5, "off by {error} px");
  }

  #[test]
  fn monotone_slide_never_overshoots() {
    let tween = slide(HYPRLAND_DEFAULT);
    let segments = tween.segments(0.004);
    for i in 0..=6000 {
      let value = sampled(&segments, 0.3 * f64::from(i) / 6000.0);
      assert!((-2560.0 - 1e-6..=1e-6).contains(&value), "{value}");
    }
  }

  #[test]
  fn segments_join_and_end_on_target() {
    let tween = slide(CSS_EASE);
    let segments = tween.segments(0.004);
    for pair in segments.windows(2) {
      let joint = pair[1].begin;
      assert!((pair[0].value(joint) - pair[1].value(joint)).abs() < 1e-6);
    }
    let end = sampled(&segments, tween.duration_secs);
    assert!((end - tween.to).abs() < 1e-6);
  }

  #[test]
  fn instant_tween_has_no_segments() {
    let tween = Tween {
      duration_secs: 0.0,
      ..slide(CSS_EASE)
    };
    assert!(tween.segments(0.004).is_empty());
  }
}
