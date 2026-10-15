use std::{
  cell::Cell,
  time::{Duration, Instant},
};

use wm_common::{EasingFunction, SpringParams};
use wm_platform::{OpacityValue, Rect};

use crate::animation::engine::{animation_progress_at, apply_easing};

/// Residual rect travel distance, in pixels, at which a non-overshooting
/// animation completes early.
///
/// Kept below one pixel so the single-frame snap from the early-completion
/// position to the exact target is imperceptible for any travel distance.
/// Mirrors `WS_COMPLETE_THRESHOLD_PX` used by the workspace-switch driver.
const COMPLETE_THRESHOLD_PX: f32 = 1.0;

/// Time over which `matching_slope` measures an animation's speed.
const SPEED_SAMPLE: Duration = Duration::from_millis(8);

/// Steepest start a retargeted curve gets, so a near-zero remaining
/// distance can't make it shoot off.
const MAX_SLOPE: f32 = 8.0;

/// Fastest start a retargeted spring gets, either way, in progress per
/// second, for the same reason.
const MAX_SPRING_VELOCITY: f32 = 40.0;

/// Smallest first control point `x` of a retargeted curve, which its
/// initial slope is set through.
const MIN_CONTROL_X: f32 = 0.05;

/// State of an individual window animation.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  /// Time of the first rendered frame, set by [`start_clock`].
  ///
  /// Progress reads as 0.0 until then. The relayout that creates an
  /// animation also evaluates it, but its first tick only lands after
  /// that relayout's session setup and cloaking -- measured ~70ms on a
  /// 150ms resize. Anchoring the clock at that first evaluation made the
  /// first visible frame jump to ~80% eased progress.
  ///
  /// [`start_clock`]: WindowAnimationState::start_clock
  start_time: Cell<Option<Instant>>,
  /// Time to wait before advancing progress.
  ///
  /// Used for staggered workspace-switch animations where each window
  /// starts at a different offset within the shared duration window.
  pub start_delay: Duration,
  pub duration: Duration,
  pub easing: EasingFunction,

  // Position animation.
  pub start_rect: Rect,
  pub target_rect: Rect,

  // Opacity animation; `None` when fade is disabled.
  pub start_opacity: Option<OpacityValue>,
  pub target_opacity: Option<OpacityValue>,
}

impl WindowAnimationState {
  /// Creates a new movement animation.
  ///
  /// A spring easing lasts as long as it takes to settle, rather than
  /// `duration_ms`.
  pub fn new_movement(
    start_rect: Rect,
    target_rect: Rect,
    duration_ms: u32,
    easing: EasingFunction,
  ) -> Self {
    Self {
      start_time: Cell::new(None),
      start_delay: Duration::ZERO,
      duration: Duration::from_millis(u64::from(
        easing.duration_ms(duration_ms),
      )),
      easing,
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Starts the animation clock at `now`, unless it is already running.
  ///
  /// Called by every animation tick before anything is rendered, so the
  /// clock starts on the first frame that is actually shown.
  pub fn start_clock(&self, now: Instant) {
    if self.start_time.get().is_none() {
      self.start_time.set(Some(now));
    }
  }

  /// Gets the eased progress in [0.0, 1.0] at an explicit `now` instant.
  ///
  /// Allows callers to supply a predictive timestamp (e.g. vsync wake-up
  /// time plus an estimated pipeline offset) so the computed position
  /// aligns with the DWM composition event rather than the moment this
  /// code runs.
  ///
  /// `start_delay` is applied before the duration window begins: if
  /// `elapsed < start_delay`, returns 0.0 without advancing the animation.
  /// All windows initialized on the same tick share the same `start_time`,
  /// so staggering is purely a function of each window's `start_delay`.
  ///
  /// Non-overshooting curves snap to 1.0 at 99% eased progress to avoid
  /// the "stuck at destination" look. Overshooting curves run to full
  /// wall-clock duration to preserve their bounce.
  pub fn eased_progress_at(&self, now: Instant) -> f32 {
    let Some(start) = self.start_time.get() else {
      return 0.0;
    };

    let elapsed = now.saturating_duration_since(start);
    if elapsed < self.start_delay {
      return 0.0;
    }

    // Shift the clock origin past the delay so the duration window begins
    // at `start + start_delay`. `animation_progress_at` uses
    // `saturating_duration_since`, so passing a future `effective_start`
    // is safe even if `now` precedes it on the first delayed tick.
    let effective_start = start + self.start_delay;
    let raw = animation_progress_at(effective_start, self.duration, now);
    let eased = apply_easing(raw, &self.easing);
    let done = if self.easing.can_overshoot() {
      raw == 1.0
    } else if raw == 1.0 {
      true
    } else {
      // Decelerating easing spends a large fraction of its wall-clock
      // duration covering the final sliver of distance, which looks
      // "stuck" at the destination — so complete early. Gating that
      // completion on a fixed residual *pixel* distance (rather than
      // a fixed eased fraction) keeps the completion-frame snap
      // sub-pixel regardless of travel distance: a fixed `eased >=
      // 0.99` would snap ~1% of the travel, which is imperceptible
      // for a short move but a visible 10-20px jump at the end of an
      // open/close slide spanning a whole window dimension. Mirrors
      // `WS_COMPLETE_THRESHOLD_PX` in the workspace-switch driver.
      let max_travel = self.max_travel_px();
      if max_travel > 0.0 {
        (1.0 - eased) * max_travel <= COMPLETE_THRESHOLD_PX
      } else {
        // No positional travel (e.g. an opacity-only fade): there is no
        // pixel distance to gate on, so fall back to the eased fraction.
        eased >= 0.99
      }
    };
    if done {
      1.0
    } else {
      eased
    }
  }

  /// Gets the eased progress in [0.0, 1.0], snapping to 1.0 when complete.
  pub fn eased_progress(&self) -> f32 {
    self.eased_progress_at(Instant::now())
  }

  /// Remaining wall-clock time until the animation's duration window
  /// elapses at `now`.
  ///
  /// Returns the full `start_delay + duration` when the animation has not
  /// rendered its first frame yet (`start_time` unset). Used to schedule
  /// the mid-animation handoff of the real window to its final rect.
  pub fn remaining_at(&self, now: Instant) -> Duration {
    match self.start_time.get() {
      Some(start) => (start + self.start_delay + self.duration)
        .saturating_duration_since(now),
      None => self.start_delay + self.duration,
    }
  }

  /// Largest per-axis rect travel distance, in pixels, between the start
  /// and target rects.
  ///
  /// Returns the maximum of the absolute position and size deltas, giving
  /// an upper bound on how far any edge of the window moves over the
  /// animation. Returns `0` for opacity-only animations whose start and
  /// target rects are identical.
  fn max_travel_px(&self) -> f32 {
    let dx = (self.target_rect.x() - self.start_rect.x()).abs();
    let dy = (self.target_rect.y() - self.start_rect.y()).abs();
    let dw = (self.target_rect.width() - self.start_rect.width()).abs();
    let dh = (self.target_rect.height() - self.start_rect.height()).abs();
    #[allow(clippy::cast_precision_loss)]
    {
      dx.max(dy).max(dw).max(dh) as f32
    }
  }

  /// A movement from where this animation is at `now` to `target` that
  /// starts at the speed this one has there, so retargeting it mid-flight
  /// carries on instead of jolting.
  ///
  /// A cubic bezier gets there by moving its first control point to set
  /// the curve's initial slope, keeping its second, which shapes the
  /// ending, so it can't start out heading back. A spring starts at that
  /// speed, heading back first if it was. Other easings start from rest.
  pub fn retargeted(
    &self,
    target: Rect,
    duration_ms: u32,
    easing: EasingFunction,
    now: Instant,
  ) -> Self {
    let (current, _) = self.current_state_at(now);

    let easing = match easing {
      EasingFunction::CubicBezier(x1, _, x2, y2) => {
        let slope = self
          .matching_slope(&current, &target, duration_ms, now)
          .clamp(0.0, MAX_SLOPE);
        let x1 = x1.max(MIN_CONTROL_X);
        EasingFunction::CubicBezier(x1, slope * x1, x2, y2)
      }
      EasingFunction::Spring(spring) => {
        // The slope over one second is the speed per second.
        let velocity = self
          .matching_slope(&current, &target, 1000, now)
          .clamp(-MAX_SPRING_VELOCITY, MAX_SPRING_VELOCITY);
        EasingFunction::Spring(SpringParams {
          initial_velocity: velocity,
          ..spring
        })
      }
      other => other,
    };

    Self::new_movement(current, target, duration_ms, easing)
  }

  /// Initial slope (progress per progress) a movement from `current` to
  /// `target` over `duration_ms` needs to keep this animation's speed at
  /// `now`, along the axis that moves most: negative when this one moves
  /// the other way.
  #[allow(clippy::cast_precision_loss)]
  fn matching_slope(
    &self,
    current: &Rect,
    target: &Rect,
    duration_ms: u32,
    now: Instant,
  ) -> f32 {
    let deltas = |from: &Rect, to: &Rect| {
      [
        to.x() - from.x(),
        to.y() - from.y(),
        to.width() - from.width(),
        to.height() - from.height(),
      ]
    };
    let (old, new) = (
      deltas(&self.start_rect, &self.target_rect),
      deltas(current, target),
    );
    let axis = (0..new.len())
      .max_by_key(|axis| new[*axis].abs())
      .unwrap_or(0);
    if new[axis] == 0 {
      return 0.0;
    }

    let speed = self.progress_per_ms(now) * old[axis] as f32;

    speed * duration_ms as f32 / new[axis] as f32
  }

  /// Eased progress per millisecond at `now`, measured over the moments
  /// before it.
  ///
  /// An animation whose clock hasn't started (e.g. one retargeted earlier
  /// in the same frame) still reads 0 there, so it gets the speed it
  /// starts out at instead.
  fn progress_per_ms(&self, now: Instant) -> f32 {
    if self.start_time.get().is_none() {
      let duration_ms = self.duration.as_secs_f32() * 1000.0;
      if !self.start_delay.is_zero() || duration_ms <= 0.0 {
        return 0.0;
      }

      let step = 1e-3;
      let rise =
        apply_easing(step, &self.easing) - apply_easing(0.0, &self.easing);
      return rise / step / duration_ms;
    }

    let Some(earlier) = now.checked_sub(SPEED_SAMPLE) else {
      return 0.0;
    };

    (self.eased_progress_at(now) - self.eased_progress_at(earlier))
      / SPEED_SAMPLE.as_secs_f32()
      / 1000.0
  }

  /// Whether the animation has completed.
  pub fn is_complete(&self) -> bool {
    self.eased_progress() == 1.0
  }

  /// Gets the interpolated rect at the current animation progress.
  pub fn current_rect(&self) -> Rect {
    self
      .start_rect
      .interpolate(&self.target_rect, self.eased_progress())
  }

  /// Gets the interpolated rect and opacity in a single call.
  ///
  /// Prefer this over separate `current_rect` + `current_opacity` calls
  /// when both values are needed in the same frame — `eased_progress`
  /// (which runs a Newton-Raphson solve) is computed only once.
  pub fn current_state(&self) -> (Rect, Option<OpacityValue>) {
    self.current_state_at(Instant::now())
  }

  /// Gets the interpolated rect and opacity at an explicit `now` instant.
  ///
  /// Like [`current_state`], but evaluates progress at a caller-supplied
  /// predictive timestamp (e.g. a vsync wake-up led forward by a fraction
  /// of a frame) so the computed position aligns with the next DWM
  /// composition rather than the moment this code runs.
  ///
  /// [`current_state`]: WindowAnimationState::current_state
  pub fn current_state_at(
    &self,
    now: Instant,
  ) -> (Rect, Option<OpacityValue>) {
    let progress = self.eased_progress_at(now);
    let rect = self.start_rect.interpolate(&self.target_rect, progress);
    let opacity = match (&self.start_opacity, &self.target_opacity) {
      (Some(start), Some(end)) => Some(start.interpolate(end, progress)),
      _ => None,
    };
    (rect, opacity)
  }
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::*;

  /// A cubic bezier with collinear, evenly-spaced control points is
  /// exactly the identity curve, so `eased == raw`. Used to make
  /// completion behaviour deterministic in tests.
  fn linear() -> EasingFunction {
    EasingFunction::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0)
  }

  /// A long slide must not snap to the target while it is still many
  /// pixels away: at 99% progress over a 10000px slide the residual is
  /// 100px, so the animation reports the eased value rather than
  /// completing.
  #[test]
  fn long_slide_does_not_snap_at_ninety_nine_percent() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(10_000, 0, 100, 100),
      10_000,
      linear(),
    );

    let t0 = Instant::now();
    anim.start_clock(t0);
    assert_eq!(anim.eased_progress_at(t0), 0.0);

    let progress =
      anim.eased_progress_at(t0 + Duration::from_millis(9_900));
    assert!(progress < 1.0, "expected no early snap, got {progress}");
    assert!((progress - 0.99).abs() < 1e-3, "got {progress}");
  }

  /// A slide completes once it is within one pixel of the target: at 99.9%
  /// progress over a 100px slide the residual is 0.1px, so it snaps to
  /// 1.0.
  #[test]
  fn long_slide_completes_within_one_pixel() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(100, 0, 100, 100),
      10_000,
      linear(),
    );

    let t0 = Instant::now();
    anim.start_clock(t0);
    assert_eq!(anim.eased_progress_at(t0), 0.0);

    let progress =
      anim.eased_progress_at(t0 + Duration::from_millis(9_990));
    assert_eq!(progress, 1.0);
  }

  /// An opacity-only animation has zero positional travel, so completion
  /// falls back to the eased fraction: it is incomplete just below 99%
  /// and complete at/above it.
  #[test]
  fn opacity_only_completes_on_eased_fraction() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(0, 0, 100, 100),
      100,
      linear(),
    );

    let t0 = Instant::now();
    anim.start_clock(t0);
    assert_eq!(anim.eased_progress_at(t0), 0.0);

    let before = anim.eased_progress_at(t0 + Duration::from_millis(98));
    assert!((before - 0.98).abs() < 1e-3, "got {before}");

    let after = anim.eased_progress_at(t0 + Duration::from_millis(99));
    assert_eq!(after, 1.0);
  }

  fn spring() -> EasingFunction {
    EasingFunction::Spring(wm_common::SpringParams::default())
  }

  /// A spring lasts as long as it takes to settle, not the configured
  /// duration, and arrives on its own.
  #[test]
  fn spring_runs_its_own_length() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(500, 0, 100, 100),
      10,
      spring(),
    );
    assert!(anim.duration > Duration::from_millis(250));

    let t0 = Instant::now();
    anim.start_clock(t0);
    let early = anim.eased_progress_at(t0 + Duration::from_millis(30));
    assert!(early > 0.2 && early < 0.9, "got {early}");
    assert_eq!(anim.eased_progress_at(t0 + anim.duration), 1.0);
  }

  /// Retargeting a moving spring keeps it going rather than starting
  /// from rest.
  #[test]
  fn retargeted_spring_keeps_its_speed() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(1_000, 0, 100, 100),
      200,
      spring(),
    );
    let t0 = Instant::now();
    anim.start_clock(t0);
    let now = t0 + Duration::from_millis(40);

    let (current, _) = anim.current_state_at(now);
    let target = Rect::from_xy(current.x() + 1_000, 0, 100, 100);
    let retargeted = anim.retargeted(target, 200, spring(), now);

    let EasingFunction::Spring(params) = retargeted.easing else {
      panic!("not a spring");
    };
    assert!(params.initial_velocity > 0.);

    // Moving already, it covers more ground at first than from rest.
    let from_rest = WindowAnimationState::new_movement(
      current.clone(),
      retargeted.target_rect.clone(),
      200,
      spring(),
    );
    let (now2, later) = (Instant::now(), Duration::from_millis(10));
    retargeted.start_clock(now2);
    from_rest.start_clock(now2);
    assert!(
      retargeted.eased_progress_at(now2 + later)
        > from_rest.eased_progress_at(now2 + later)
    );
  }

  /// Retargeted twice before a frame starts its clock, a spring keeps the
  /// speed the first retarget handed it.
  #[test]
  fn retargeting_twice_in_a_frame_keeps_the_speed() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(1_000, 0, 100, 100),
      200,
      spring(),
    );
    let t0 = Instant::now();
    anim.start_clock(t0);
    let now = t0 + Duration::from_millis(40);

    let (current, _) = anim.current_state_at(now);
    let to = |distance| Rect::from_xy(current.x() + distance, 0, 100, 100);
    let first = anim.retargeted(to(1_000), 200, spring(), now);
    let second = first.retargeted(to(2_000), 200, spring(), now);

    let (EasingFunction::Spring(first), EasingFunction::Spring(second)) =
      (first.easing, second.easing)
    else {
      panic!("not springs");
    };

    // The same speed in pixels, over twice the distance.
    let (expected, actual) =
      (first.initial_velocity / 2., second.initial_velocity);
    assert!(expected > 0.);
    assert!((actual - expected).abs() < expected * 0.05, "{actual}");
  }

  /// A spring sent back the way it came carries on a moment before
  /// turning, where a curve starts back from rest.
  #[test]
  fn retargeted_spring_turns_back_with_its_momentum() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(1_000, 0, 100, 100),
      200,
      spring(),
    );
    let t0 = Instant::now();
    anim.start_clock(t0);
    let now = t0 + Duration::from_millis(40);

    let (current, _) = anim.current_state_at(now);
    let back = Rect::from_xy(0, 0, 100, 100);

    let EasingFunction::Spring(params) =
      anim.retargeted(back.clone(), 200, spring(), now).easing
    else {
      panic!("not a spring");
    };
    assert!(params.initial_velocity < 0.);
    assert!(params.initial_velocity >= -MAX_SPRING_VELOCITY);

    let curve = EasingFunction::CubicBezier(0.25, 0.1, 0.25, 1.0);
    let EasingFunction::CubicBezier(_, y1, ..) =
      anim.retargeted(back, 200, curve, now).easing
    else {
      panic!("not a curve");
    };
    assert!(y1.abs() < f32::EPSILON, "from rest");
    assert!(current.x() > 0);
  }

  /// `start_delay` holds the animation at progress 0.0 until the delay
  /// elapses (used by the window-open paint grace period).
  #[test]
  fn start_delay_holds_at_zero() {
    let mut anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(1_000, 0, 100, 100),
      100,
      linear(),
    );
    anim.start_delay = Duration::from_millis(30);

    let t0 = Instant::now();
    anim.start_clock(t0);
    // Still within the delay window.
    assert_eq!(anim.eased_progress_at(t0), 0.0);
    assert_eq!(
      anim.eased_progress_at(t0 + Duration::from_millis(20)),
      0.0
    );

    // 30ms in, the duration window has just begun (progress ~0, not
    // snapped).
    let after_delay =
      anim.eased_progress_at(t0 + Duration::from_millis(30));
    assert!(after_delay < 0.01, "got {after_delay}");

    // 30ms delay + 50ms into the 100ms duration → ~50% progress.
    let mid = anim.eased_progress_at(t0 + Duration::from_millis(80));
    assert!((mid - 0.5).abs() < 1e-2, "got {mid}");
  }

  /// Evaluating an animation before its first tick (as the relayout that
  /// creates it does) must not start its clock.
  #[test]
  fn clock_starts_at_first_tick_not_first_evaluation() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(1_000, 0, 100, 100),
      100,
      linear(),
    );

    let created = Instant::now();
    assert_eq!(anim.eased_progress_at(created), 0.0);

    // First tick 70ms later: the first frame still starts at zero.
    let first_tick = created + Duration::from_millis(70);
    assert_eq!(anim.eased_progress_at(first_tick), 0.0);
    anim.start_clock(first_tick);
    assert_eq!(anim.eased_progress_at(first_tick), 0.0);

    // A later tick does not move the origin.
    anim.start_clock(first_tick + Duration::from_millis(10));
    let half =
      anim.eased_progress_at(first_tick + Duration::from_millis(50));
    assert!((half - 0.5).abs() < 0.01, "got {half}");
  }
}
