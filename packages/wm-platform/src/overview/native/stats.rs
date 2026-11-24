//! Timing of the overview's opens and closes, logged one line each while
//! `GLAZEWM_PERF` is set: how soon the first frame showed, how long each
//! frame's work took, and how many frames were missed. Open and close the
//! overview by hand a few times and compare the lines between builds.

use std::time::{Duration, Instant};

/// Timing of one open or close, from its start until nothing moves.
pub(super) struct MotionStats {
  label: &'static str,
  started: Instant,
  first_frame: Option<Duration>,
  /// Time each frame's work took.
  work: Vec<Duration>,
  /// Time between frames, past the first.
  intervals: Vec<Duration>,
  /// The monitor's frame period, which a missed frame takes over 1.5 of.
  period: Duration,
}

impl MotionStats {
  pub fn new(
    label: &'static str,
    started: Instant,
    period: Duration,
  ) -> Self {
    Self {
      label,
      started,
      first_frame: None,
      work: Vec::new(),
      intervals: Vec::new(),
      period,
    }
  }

  /// Notes that a frame showed at `at`, the first of which counts.
  pub fn shown(&mut self, at: Instant) {
    self
      .first_frame
      .get_or_insert_with(|| at.saturating_duration_since(self.started));
  }

  /// Notes a frame whose work took `work`, `interval` after the last.
  pub fn frame(&mut self, work: Duration, interval: Option<Duration>) {
    self.work.push(work);
    self.intervals.extend(interval);
  }

  /// One line of it all.
  pub fn summary(&self) -> String {
    let ms = |duration: Duration| duration.as_secs_f64() * 1000.0;
    let mut work = self.work.clone();
    work.sort_unstable();
    let missed = self
      .intervals
      .iter()
      .filter(|interval| **interval > self.period.mul_f32(1.5))
      .count();
    format!(
      "Overview {} perf: first frame {:.1} ms, {} frames, work p50 {:.2} \
       ms / p95 {:.2} ms / max {:.2} ms, {missed} missed (period {:.1} ms).",
      self.label,
      self.first_frame.map_or(f64::NAN, ms),
      work.len(),
      ms(percentile(&work, 0.5)),
      ms(percentile(&work, 0.95)),
      ms(work.last().copied().unwrap_or_default()),
      ms(self.period),
    )
  }
}

/// The `share` (0 to 1) percentile of `sorted`, nearest rank.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
fn percentile(sorted: &[Duration], share: f32) -> Duration {
  if sorted.is_empty() {
    return Duration::ZERO;
  }
  let rank = (share * sorted.len() as f32).ceil() as usize;
  sorted[rank.clamp(1, sorted.len()) - 1]
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use super::{percentile, MotionStats};

  #[test]
  fn percentiles_take_the_nearest_rank() {
    let sorted =
      (1..=20_u64).map(Duration::from_millis).collect::<Vec<_>>();
    assert_eq!(percentile(&sorted, 0.5), Duration::from_millis(10));
    assert_eq!(percentile(&sorted, 0.95), Duration::from_millis(19));
    assert_eq!(percentile(&[], 0.5), Duration::ZERO);
  }

  /// It counts the first frame from the start, every frame's work, and
  /// frames more than 1.5 periods apart as missed.
  #[test]
  fn summary_counts_first_frame_work_and_misses() {
    let started = Instant::now();
    let period = Duration::from_micros(16_667);
    let mut stats = MotionStats::new("open", started, period);
    stats.shown(started + Duration::from_millis(12));
    stats.shown(started + Duration::from_millis(30));
    stats.frame(Duration::from_millis(2), None);
    stats.frame(Duration::from_millis(3), Some(period));
    stats.frame(Duration::from_millis(1), Some(period * 3));

    let summary = stats.summary();
    assert!(summary.contains("first frame 12.0 ms"), "{summary}");
    assert!(summary.contains("3 frames"), "{summary}");
    assert!(summary.contains("max 3.00 ms"), "{summary}");
    assert!(summary.contains("1 missed"), "{summary}");
  }
}
