use std::time::{Duration, Instant};

use wm_common::EasingFunction;

/// Calculates the animation progress at an explicit `now` instant (0.0 to
/// 1.0).
///
/// Allows callers to supply a predictive timestamp (e.g. vsync wake-up
/// time plus an estimated pipeline offset) so the computed position aligns
/// with the DWM composition event rather than the moment `update_internal`
/// runs. Uses `saturating_duration_since` so a `now` that precedes
/// `start_time` (possible on the first frame when `pipeline_offset` >
/// elapsed) returns `0.0` instead of panicking.
pub fn animation_progress_at(
  start_time: Instant,
  duration: Duration,
  now: Instant,
) -> f32 {
  let elapsed = now.saturating_duration_since(start_time);

  if elapsed >= duration {
    return 1.0;
  }

  // Sub-millisecond, so that frames of a fast display each move.
  let progress = elapsed.as_secs_f32() / duration.as_secs_f32();
  progress.clamp(0.0, 1.0)
}

/// Applies an easing function to a linear progress value (0.0 to 1.0).
///
/// A spring's progress is the share of its own duration (see
/// `EasingFunction::duration_ms`) that has passed.
pub fn apply_easing(progress: f32, easing: &EasingFunction) -> f32 {
  match easing {
    EasingFunction::EaseOutSpring => ease_out_spring(progress),
    EasingFunction::CubicBezier(x1, y1, x2, y2) => {
      cubic_bezier(*x1, *y1, *x2, *y2, progress)
    }
    EasingFunction::Spring(spring) => {
      if progress >= 1.0 {
        return 1.0;
      }

      spring.value_at(progress * spring.duration().as_secs_f32())
    }
  }
}

/// Evaluates a CSS cubic bezier at the given `x` progress (0.0 to 1.0).
///
/// Control points `(x1, y1)` and `(x2, y2)` define the curve between the
/// implicit anchors `(0, 0)` and `(1, 1)`. Finds the curve parameter `t`
/// with `Bx(t) = x` by bisection, then returns `By(t)`: Newton's method
/// stalls on curves that start or end vertical (`x1` or `x2` at the
/// edge), like Hyprland's `default`, which cover most of their travel in
/// the first frames.
#[allow(clippy::cast_possible_truncation)]
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
  // As in CSS, `x1` and `x2` stay within the unit interval, which keeps
  // `Bx` monotonic.
  let (x1, x2) =
    (f64::from(x1.clamp(0.0, 1.0)), f64::from(x2.clamp(0.0, 1.0)));
  let (y1, y2) = (f64::from(y1), f64::from(y2));
  let blend = |a: f64, b: f64, t: f64| {
    let u = 1.0 - t;
    3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
  };

  let x = f64::from(x.clamp(0.0, 1.0));
  let (mut low, mut high) = (0.0, 1.0);
  for _ in 0..40 {
    let mid = f64::midpoint(low, high);
    if blend(x1, x2, mid) < x {
      low = mid;
    } else {
      high = mid;
    }
  }

  blend(y1, y2, f64::midpoint(low, high)) as f32
}

/// Exponentially-decaying spring easing function.
///
/// Produces an underdamped spring effect: the value overshoots past 1.0,
/// oscillates, and settles. Runs to full wall-clock duration (not cut off
/// at 99%) to preserve the bounce.
fn ease_out_spring(t: f32) -> f32 {
  if t <= 0.0 {
    return 0.0;
  }
  if t >= 1.0 {
    return 1.0;
  }
  1.0 - 2.0f32.powf(-12.0 * t) * (4.0 * std::f32::consts::PI * t).cos()
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::*;

  #[test]
  fn test_animation_progress() {
    let start = Instant::now();
    let duration = Duration::from_millis(100);

    let progress = animation_progress_at(
      start,
      duration,
      start + Duration::from_millis(50),
    );
    assert!((progress - 0.5).abs() < 0.1);
  }

  #[test]
  fn bezier_with_vertical_start_follows_its_curve() {
    // Hyprland's `default`: at `t = 0.3`, `x` is 0.0555 and `y` 0.547.
    let eased = cubic_bezier(0.0, 0.75, 0.15, 1.0, 0.0555);
    assert!((eased - 0.547).abs() < 0.005, "{eased}");

    assert!(cubic_bezier(0.0, 0.75, 0.15, 1.0, 0.0).abs() < 1e-6);
    assert!((cubic_bezier(0.0, 0.75, 0.15, 1.0, 1.0) - 1.0).abs() < 1e-6);
    assert!((cubic_bezier(0.0, 0.0, 1.0, 1.0, 0.3) - 0.3).abs() < 1e-6);
  }

  #[test]
  fn spring_starts_from_zero_and_settles_past_overshoot() {
    assert!(ease_out_spring(0.001) < 0.05);
    assert!((0..=100).any(|i| ease_out_spring(i as f32 / 100.0) > 1.05));
    assert!((ease_out_spring(0.999) - 1.0).abs() < 0.01);
  }

  #[test]
  fn test_interpolate_rect() {
    let start = Rect::from_xy(0, 0, 100, 100);
    let end = Rect::from_xy(100, 100, 200, 200);

    let mid = start.interpolate(&end, 0.5);
    assert_eq!(mid.x(), 50);
    assert_eq!(mid.y(), 50);
    assert_eq!(mid.width(), 150);
    assert_eq!(mid.height(), 150);
  }
}
