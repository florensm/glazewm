//! How the overview's flourishes move: ripples spreading out of cards,
//! sparks bursting out of where a window lands, and the selection's glow
//! breathing. Kept free of drawing so it can be tested.

use std::f32::consts::TAU;

use crate::overview_layout::{ease_out_cubic, RectF};

/// How long a ripple takes to spread out and fade.
const RIPPLE_MS: f32 = 480.0;

/// How long a spark flies before it has faded out.
const SPARK_MS: f32 = 750.0;

/// How long one breath of the glow takes, in seconds, and how far it
/// dims at its faintest.
const PULSE_PERIOD_S: f32 = 1.8;
const PULSE_DEPTH: f32 = 0.35;

/// A ring spreading out of a rect and fading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ripple {
  from: RectF,

  /// How far past `from` it spreads, on every side.
  reach: f32,

  age_ms: f32,
}

impl Ripple {
  pub fn new(from: RectF, reach: f32) -> Self {
    Self {
      from,
      reach,
      age_ms: 0.0,
    }
  }

  /// Advances it by `dt` seconds. Returns whether it still shows.
  pub fn step(&mut self, dt: f32) -> bool {
    self.age_ms += dt * 1000.0;
    self.age_ms < RIPPLE_MS
  }

  /// Where it is and how strongly it shows, once it hasn't faded.
  pub fn shown(&self) -> Option<(RectF, f32)> {
    let progress = self.age_ms / RIPPLE_MS;
    (progress < 1.0).then(|| {
      let spread = ease_out_cubic(progress) * self.reach;
      (self.from.inset(-spread), (1.0 - progress).powi(2))
    })
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Spark {
  x: f32,
  y: f32,
  vx: f32,
  vy: f32,
  size: f32,
}

/// Sparks flung out of a point, slowing, falling and fading as they go.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Burst {
  sparks: Vec<Spark>,

  /// How fast they fall, in pixels per second squared.
  gravity: f32,

  age_ms: f32,
}

impl Burst {
  /// `count` sparks flung every way out of (`x`, `y`) at up to `speed`
  /// pixels per second, about `size` pixels across, scattered by
  /// `scatter`.
  #[allow(clippy::cast_precision_loss)]
  pub fn new(
    (x, y): (f32, f32),
    count: usize,
    speed: f32,
    size: f32,
    scatter: u32,
  ) -> Self {
    let mut random = Random(scatter | 1);
    let sparks = (0..count)
      .map(|index| {
        // Evenly around, each nudged off its place, so none bunch up.
        let angle =
          (index as f32 + random.next() * 0.8) / count as f32 * TAU;
        let speed = speed * (0.45 + 0.55 * random.next());
        Spark {
          x,
          y,
          vx: angle.cos() * speed,
          // A little upward, as from a splash.
          vy: angle.sin() * speed - speed * 0.35,
          size: size * (0.6 + 0.6 * random.next()),
        }
      })
      .collect();

    Self {
      sparks,
      gravity: speed * 2.4,
      age_ms: 0.0,
    }
  }

  /// Advances it by `dt` seconds. Returns whether it still shows.
  pub fn step(&mut self, dt: f32) -> bool {
    self.age_ms += dt * 1000.0;
    let drag = (-3.0 * dt).exp();

    for spark in &mut self.sparks {
      spark.x += spark.vx * dt;
      spark.y += spark.vy * dt;
      spark.vx *= drag;
      spark.vy = spark.vy * drag + self.gravity * dt;
    }

    self.age_ms < SPARK_MS
  }

  /// Each spark's rect and how strongly it shows, shrinking and fading
  /// as it flies.
  pub fn sparks(&self) -> impl Iterator<Item = (RectF, f32)> + '_ {
    let life = (1.0 - self.age_ms / SPARK_MS).clamp(0.0, 1.0);

    self.sparks.iter().map(move |spark| {
      let size = spark.size * (0.35 + 0.65 * life);
      (
        RectF::new(spark.x - size / 2.0, spark.y - size / 2.0, size, size),
        life * life,
      )
    })
  }
}

/// How strongly the breathing glow shows `elapsed_s` after it started:
/// fully at first, dimming and brightening again once per period.
pub(crate) fn pulse(elapsed_s: f32) -> f32 {
  let phase = (elapsed_s / PULSE_PERIOD_S * TAU).cos();
  1.0 - PULSE_DEPTH * (1.0 - phase) / 2.0
}

/// A seed for the next burst, different from one burst to the next.
pub(crate) fn seed() -> u32 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(1, |since| since.subsec_nanos())
}

/// Xorshift: plenty for scattering sparks.
struct Random(u32);

impl Random {
  /// A number from 0 to 1.
  #[allow(clippy::cast_precision_loss)]
  fn next(&mut self) -> f32 {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 17;
    self.0 ^= self.0 << 5;
    (self.0 >> 8) as f32 / (1 << 24) as f32
  }
}

#[cfg(test)]
mod tests {
  use super::{pulse, Burst, Ripple};
  use crate::overview_layout::RectF;

  #[test]
  fn ripple_spreads_out_and_fades() {
    let from = RectF::new(100.0, 100.0, 200.0, 100.0);
    let mut ripple = Ripple::new(from, 40.0);
    let (rect, alpha) = ripple.shown().expect("shown");
    assert_eq!(rect, from);
    assert!((alpha - 1.0).abs() < f32::EPSILON);

    assert!(ripple.step(0.2));
    let (rect, alpha) = ripple.shown().expect("shown");
    assert!(rect.x < from.x && rect.w > from.w && rect.w < from.w + 80.0);
    assert!(alpha < 1.0);

    assert!(!ripple.step(0.3));
    assert_eq!(ripple.shown(), None);
  }

  #[test]
  fn burst_flings_sparks_every_way_then_ends() {
    let mut burst = Burst::new((500.0, 500.0), 12, 600.0, 8.0, 7);
    burst.step(0.1);

    let centers = burst
      .sparks()
      .map(|(rect, _)| rect.center())
      .collect::<Vec<_>>();
    assert_eq!(centers.len(), 12);
    assert!(centers.iter().any(|(x, _)| *x < 480.0));
    assert!(centers.iter().any(|(x, _)| *x > 520.0));
    assert!(centers.iter().any(|(_, y)| *y < 480.0));

    let mut frames = 0;
    while burst.step(1.0 / 60.0) {
      frames += 1;
      assert!(frames < 120, "burst never ended");
    }
    assert!(burst.sparks().all(|(_, alpha)| alpha < 0.01));
  }

  #[test]
  fn bursts_differ_by_seed_only() {
    let burst = |seed| Burst::new((0.0, 0.0), 6, 500.0, 8.0, seed);
    assert_eq!(burst(3), burst(3));
    assert_ne!(burst(3), burst(4));
  }

  #[test]
  fn pulse_breathes_between_full_and_dimmed() {
    assert!((pulse(0.0) - 1.0).abs() < 1e-6);
    assert!((pulse(0.9) - 0.65).abs() < 1e-3);
    assert!((pulse(1.8) - 1.0).abs() < 1e-3);
  }
}
