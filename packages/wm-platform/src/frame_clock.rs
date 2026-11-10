use std::{
  collections::VecDeque,
  time::{Duration, Instant},
};

/// Wake-to-wake times kept to measure the refresh period afresh.
const PERIOD_WINDOW: usize = 9;

/// Wake-to-wake times needed before their median is taken for the period.
const MIN_PERIOD_SAMPLES: usize = 5;

/// Wake-to-wake times taken for a refresh period: 25 to 370 Hz.
const PLAUSIBLE_PERIOD_US: std::ops::RangeInclusive<f64> =
  2_700.0..=40_000.0;

/// Share of a wake's distance from its expected time the clock moves by.
const PHASE_GAIN: f64 = 0.1;

/// Share of that distance the period changes by. With [`PHASE_GAIN`] this
/// settles without overshooting much (damping of about 0.7).
const PERIOD_GAIN: f64 = 0.005;

/// How much earlier than expected a wake may be, in periods, before the
/// clock restarts from it.
const EARLY_TOLERANCE: f64 = 0.125;

/// How much later than expected a wake may be, in periods, before it is
/// left out as a late wake-up.
const LATE_TOLERANCE: f64 = 0.25;

/// Wakes in a row outside both tolerances after which the period is
/// measured afresh: the refresh rate changed.
const MAX_MISSES: u32 = 3;

/// How far the measured period may be from a multiple of the clock's, as
/// a share of it, to count as that multiple.
const MULTIPLE_TOLERANCE: f64 = 0.1;

/// Time without a wake after which the clock no longer tells the time, and
/// restarts from the next wake.
const MAX_GAP: Duration = Duration::from_millis(100);

/// Estimates when each vblank happened from the times a thread woke up for
/// them.
///
/// A wake lands a varying while after its vblank, later still on a loaded
/// machine, while frames are shown a steady period apart. Animations
/// evaluated at the raw wake times therefore advance unevenly: at 10
/// px/ms, one wake 2 ms late is a 20 px stutter. The clock steps by its
/// period instead, and both lean only slightly toward each wake.
///
/// A wake can be late but never early, so one well ahead of the estimate
/// means the estimate is off (e.g. the refresh rate rose), and the clock
/// restarts from it rather than run ahead of the display.
#[derive(Debug)]
pub struct FrameClock {
  /// The refresh period: the display mode's at first, then refined by
  /// each wake.
  period_us: f64,
  /// Latest wake-to-wake times within [`PLAUSIBLE_PERIOD_US`].
  deltas: VecDeque<f64>,
  last_wake: Option<Instant>,
  /// Estimated time of the latest vblank.
  vblank: Option<Instant>,
  /// Wakes in a row outside the tolerances.
  misses: u32,
}

impl FrameClock {
  /// Refresh period assumed when the display's is unknown.
  pub const DEFAULT_PERIOD: Duration = Duration::from_micros(16_667);

  #[must_use]
  pub fn new(nominal_period: Duration) -> Self {
    Self {
      period_us: nominal_period.as_secs_f64() * 1e6,
      deltas: VecDeque::with_capacity(PERIOD_WINDOW),
      last_wake: None,
      vblank: None,
      misses: 0,
    }
  }

  /// Starts over for a display with `nominal_period`.
  pub fn reset(&mut self, nominal_period: Duration) {
    *self = Self::new(nominal_period);
  }

  /// Records a wake-up for a vblank at `wake`.
  pub fn note_vblank(&mut self, wake: Instant) {
    let since_wake = self
      .last_wake
      .map(|last| wake.saturating_duration_since(last));
    self.last_wake = Some(wake);

    let Some(vblank) = self
      .vblank
      .filter(|_| since_wake.is_some_and(|gap| gap <= MAX_GAP))
    else {
      // The first wake, or the first after a pause: earlier ones say
      // nothing about this one.
      self.deltas.clear();
      self.vblank = Some(wake);
      self.misses = 0;
      return;
    };

    if let Some(delta_us) = since_wake
      .map(|gap| gap.as_secs_f64() * 1e6)
      .filter(|delta_us| PLAUSIBLE_PERIOD_US.contains(delta_us))
    {
      if self.deltas.len() == PERIOD_WINDOW {
        self.deltas.pop_front();
      }
      self.deltas.push_back(delta_us);
    }

    let elapsed_us =
      wake.saturating_duration_since(vblank).as_secs_f64() * 1e6;

    // Vblanks since the last estimate: more than one when a wake was
    // missed altogether.
    let frames = (elapsed_us / self.period_us).round().max(1.0);
    let expected_us = frames * self.period_us;
    let error_us = elapsed_us - expected_us;

    let is_early = error_us < -self.period_us * EARLY_TOLERANCE;
    let is_late = error_us > self.period_us * LATE_TOLERANCE;

    let step_us = if is_early || is_late {
      self.misses += 1;
      let remeasures = self.misses >= MAX_MISSES;
      if remeasures {
        self.period_us =
          self.measured_period_us().unwrap_or(self.period_us);
      }

      // A lone late wake is left out; the clock carries on by itself.
      if is_late && !remeasures {
        expected_us
      } else {
        elapsed_us
      }
    } else {
      self.misses = 0;
      self.period_us = (self.period_us + PERIOD_GAIN * error_us / frames)
        .clamp(*PLAUSIBLE_PERIOD_US.start(), *PLAUSIBLE_PERIOD_US.end());
      self.adopt_multiple();

      expected_us + PHASE_GAIN * error_us
    };

    self.vblank = Some(vblank + Duration::from_secs_f64(step_us / 1e6));
  }

  /// Estimated time of the latest vblank, or `None` when no wake was
  /// recorded shortly before `now`.
  #[must_use]
  pub fn vblank_at(&self, now: Instant) -> Option<Instant> {
    let is_fresh = self
      .last_wake
      .is_some_and(|wake| now.saturating_duration_since(wake) <= MAX_GAP);

    self.vblank.filter(|_| is_fresh)
  }

  /// The refresh period in use, which may be longer than the display
  /// mode's (a panel with a dynamic refresh rate running slower).
  #[must_use]
  pub fn period(&self) -> Duration {
    Duration::from_secs_f64(self.period_us / 1e6)
  }

  /// Median of the latest wake-to-wake times, which leaves out missed and
  /// late wakes, once there are enough.
  fn measured_period_us(&self) -> Option<f64> {
    if self.deltas.len() < MIN_PERIOD_SAMPLES {
      return None;
    }

    let mut sorted = self.deltas.iter().copied().collect::<Vec<_>>();
    sorted.sort_by(f64::total_cmp);
    Some(sorted[sorted.len() / 2])
  }

  /// Lengthens the period to the multiple of it that wakes now come at:
  /// the display runs slower than its mode says, and every wake would
  /// otherwise count as several frames.
  fn adopt_multiple(&mut self) {
    let Some(measured_us) = self.measured_period_us() else {
      return;
    };

    let multiple = (measured_us / self.period_us).round();
    let is_multiple = (measured_us - multiple * self.period_us).abs()
      < measured_us * MULTIPLE_TOLERANCE;

    if multiple >= 2.0 && is_multiple {
      self.period_us *= multiple;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const HZ_60: Duration = Duration::from_micros(16_667);
  const HZ_120: Duration = Duration::from_micros(8_333);

  /// Wake delays in microseconds, repeatable: mostly under 1.5 ms, now
  /// and then a few ms more.
  fn delays(count: usize) -> Vec<u64> {
    let mut seed = 0x2545_f491_u64;
    (0..count)
      .map(|i| {
        seed = seed
          .wrapping_mul(6_364_136_223_846_793_005)
          .wrapping_add(1_442_695_040_888_963_407);
        let delay = (seed >> 33) % 1_500;
        if i % 17 == 16 {
          delay + 3_000
        } else {
          delay
        }
      })
      .collect()
  }

  /// Feeds `clock` a wake per vblank, `period` apart from `start`, each
  /// late by its delay. Returns the estimates and the raw wake times.
  fn run(
    clock: &mut FrameClock,
    start: Instant,
    period: Duration,
    delays_us: &[u64],
  ) -> (Vec<Instant>, Vec<Instant>) {
    let mut vblank = start;
    let (mut estimates, mut wakes) = (Vec::new(), Vec::new());

    for delay in delays_us {
      vblank += period;
      let wake = vblank + Duration::from_micros(*delay);
      clock.note_vblank(wake);
      estimates.push(clock.vblank_at(wake).expect("fresh"));
      wakes.push(wake);
    }

    (estimates, wakes)
  }

  /// Largest distance of a step between consecutive `times` from
  /// `period`, in microseconds.
  fn worst_step_error(times: &[Instant], period: Duration) -> f64 {
    times
      .windows(2)
      .map(|pair| {
        let step = pair[1].duration_since(pair[0]).as_secs_f64();
        ((step - period.as_secs_f64()) * 1e6).abs()
      })
      .fold(0.0, f64::max)
  }

  #[test]
  fn steps_evenly_through_late_wakes() {
    let mut clock = FrameClock::new(HZ_60);
    let delays_us = delays(240);
    let (estimates, wakes) =
      run(&mut clock, Instant::now(), HZ_60, &delays_us);

    // Once the period is measured.
    let (raw, steady) = (
      worst_step_error(&wakes[20..], HZ_60),
      worst_step_error(&estimates[20..], HZ_60),
    );
    assert!(raw > 3_000.0, "raw wakes are uneven: {raw}");
    assert!(steady < 500.0, "estimates step evenly: {steady}");
  }

  #[test]
  fn stays_close_to_the_vblanks() {
    let mut clock = FrameClock::new(HZ_60);
    let start = Instant::now();
    let delays_us = delays(240);
    let (estimates, _) = run(&mut clock, start, HZ_60, &delays_us);

    for (i, estimate) in estimates.iter().enumerate().skip(20) {
      let vblank = start + HZ_60 * u32::try_from(i + 1).unwrap();
      let offset = estimate.duration_since(vblank);
      assert!(offset < Duration::from_millis(2), "{i}: {offset:?}");
    }
  }

  #[test]
  fn counts_a_missed_wake_as_two_frames() {
    let mut clock = FrameClock::new(HZ_60);
    let (estimates, _) = run(&mut clock, Instant::now(), HZ_60, &[0; 30]);
    let before = *estimates.last().unwrap();

    let wake = before + HZ_60 * 2;
    clock.note_vblank(wake);

    let step = clock.vblank_at(wake).unwrap().duration_since(before);
    assert!(step.abs_diff(HZ_60 * 2) < Duration::from_micros(50));
  }

  #[test]
  fn follows_the_refresh_rate_both_ways() {
    let mut clock = FrameClock::new(HZ_120);
    let start = Instant::now();

    // A panel idling at half its mode's rate.
    let (estimates, _) = run(&mut clock, start, HZ_60, &[0; 12]);
    assert!(clock.period().abs_diff(HZ_60) < Duration::from_micros(5));

    let (faster, wakes) =
      run(&mut clock, *estimates.last().unwrap(), HZ_120, &[0; 12]);
    assert!(clock.period().abs_diff(HZ_120) < Duration::from_micros(5));

    // Never ahead of the display while it catches on.
    for (estimate, wake) in faster.iter().zip(&wakes) {
      assert!(estimate <= wake);
    }
  }

  #[test]
  fn restarts_after_a_pause() {
    let mut clock = FrameClock::new(HZ_60);
    let (_, wakes) = run(&mut clock, Instant::now(), HZ_60, &[0; 10]);
    let last = *wakes.last().unwrap();

    let later = last + Duration::from_secs(3);
    assert_eq!(clock.vblank_at(later), None);

    clock.note_vblank(later);
    assert_eq!(clock.vblank_at(later), Some(later));
  }

  #[test]
  fn restarts_when_the_vblanks_shift() {
    let mut clock = FrameClock::new(HZ_60);
    let (estimates, _) = run(&mut clock, Instant::now(), HZ_60, &[0; 30]);

    // Later by 0.4 of a period from here on.
    let shifted = *estimates.last().unwrap() + HZ_60.mul_f64(0.4);
    let (estimates, wakes) = run(&mut clock, shifted, HZ_60, &[0; 6]);

    let offset = wakes[5].duration_since(estimates[5]);
    assert!(offset < Duration::from_micros(100), "{offset:?}");
  }
}
