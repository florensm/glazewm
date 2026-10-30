//! How the overview's flourishes move: ripples spreading out of cards,
//! sparks bursting out of where a window lands, the selection's glow
//! breathing, cards popping in and leaning toward the cursor, and a
//! dragged window stretching with its speed. Kept free of drawing so it
//! can be tested.

use std::f32::consts::TAU;

use crate::overview_layout::{ease_out_back, ease_out_cubic, RectF};

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

/// How small a card starts out as it pops in, relative to its size.
const POP_FROM: f32 = 0.8;

/// Speed, in pixels per second, at which a moving thing stretches all it
/// does.
const FULL_STRETCH_SPEED: f32 = 2400.0;

/// Cards popping in one after another on open, out from the one the
/// overview opens on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Entrance {
  /// Card the overview zooms out of, which is there from the start.
  from: usize,
  stagger_ms: f32,
  duration_ms: f32,
  bounce: f32,
}

impl Entrance {
  /// Cards each popping in over `duration_ms` with `bounce`,
  /// `stagger_ms` after the one before, out from card `from`.
  pub fn new(
    from: usize,
    stagger_ms: f32,
    duration_ms: f32,
    bounce: f32,
  ) -> Self {
    Self {
      from,
      stagger_ms,
      duration_ms,
      bounce,
    }
  }

  /// How far card `index` has popped in `elapsed_ms` after the open: its
  /// opacity, and its scale relative to its size.
  #[allow(clippy::cast_precision_loss)]
  pub fn appear(&self, index: usize, elapsed_ms: f32) -> (f32, f32) {
    if index == self.from {
      return (1.0, 1.0);
    }

    let delay = index.abs_diff(self.from) as f32 * self.stagger_ms;
    let progress = if self.duration_ms > 0.0 {
      ((elapsed_ms - delay) / self.duration_ms).clamp(0.0, 1.0)
    } else {
      f32::from(u8::from(elapsed_ms >= delay))
    };
    let eased = ease_out_back(progress, self.bounce);
    (progress, POP_FROM + (1.0 - POP_FROM) * eased)
  }

  /// How long after the open the last of `count` cards has popped in.
  #[allow(clippy::cast_precision_loss)]
  pub fn length_ms(&self, count: usize) -> f32 {
    let farthest = count
      .saturating_sub(1)
      .saturating_sub(self.from)
      .max(self.from);
    farthest as f32 * self.stagger_ms + self.duration_ms
  }
}

/// How far a card centered at `center`, `size` big, leans toward
/// `pointer`: up to `lean` either way, the more the nearer its edge the
/// pointer is.
pub(crate) fn lean_toward(
  center: (f32, f32),
  size: (f32, f32),
  pointer: (f32, f32),
  lean: f32,
) -> (f32, f32) {
  let toward = |pointer: f32, center: f32, size: f32| {
    if size <= 0.0 {
      return 0.0;
    }
    ((pointer - center) / (size / 2.0)).clamp(-1.0, 1.0) * lean
  };
  (
    toward(pointer.0, center.0, size.0),
    toward(pointer.1, center.1, size.1),
  )
}

/// How much something moving at `velocity` pixels per second stretches
/// across and down for a `squash` from 0 to 1: longer along its motion
/// and thinner across it.
pub(crate) fn stretch(velocity: (f32, f32), squash: f32) -> (f32, f32) {
  let squash = squash.clamp(0.0, 1.0);
  let across = (velocity.0.abs() / FULL_STRETCH_SPEED).min(1.0);
  let down = (velocity.1.abs() / FULL_STRETCH_SPEED).min(1.0);
  (
    1.0 + squash * (0.3 * across - 0.15 * down),
    1.0 + squash * (0.3 * down - 0.15 * across),
  )
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
  use super::{lean_toward, pulse, stretch, Burst, Entrance, Ripple};
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
  fn cards_pop_in_one_after_another_out_from_the_first() {
    let entrance = Entrance::new(2, 30.0, 200.0, 0.0);
    assert_eq!(entrance.appear(2, 0.0), (1.0, 1.0));

    let (alpha, scale) = entrance.appear(1, 0.0);
    assert!(alpha < f32::EPSILON && (scale - 0.8).abs() < 1e-6);

    // Its neighbours start first, either way.
    let (near, _) = entrance.appear(3, 100.0);
    let (far, _) = entrance.appear(0, 100.0);
    assert!(near > far && far > 0.0);
    assert_eq!(entrance.appear(1, 100.0), entrance.appear(3, 100.0));

    assert_eq!(entrance.appear(5, 290.0), (1.0, 1.0));
    assert!((entrance.length_ms(6) - 290.0).abs() < 1e-3);
    assert!((entrance.length_ms(3) - 260.0).abs() < 1e-3);
  }

  #[test]
  fn instant_cards_pop_in_on_their_turn() {
    let entrance = Entrance::new(0, 30.0, 0.0, 0.0);
    assert_eq!(entrance.appear(2, 59.0).0, 0.0);
    assert_eq!(entrance.appear(2, 60.0), (1.0, 1.0));
  }

  #[test]
  fn cards_lean_toward_the_pointer_up_to_their_lean() {
    let lean =
      |pointer| lean_toward((100.0, 100.0), (200.0, 100.0), pointer, 8.0);
    assert_eq!(lean((100.0, 100.0)), (0.0, 0.0));
    assert_eq!(lean((150.0, 75.0)), (4.0, -4.0));
    assert_eq!(lean((900.0, 900.0)), (8.0, 8.0));
    assert_eq!(
      lean_toward((0.0, 0.0), (0.0, 0.0), (5.0, 5.0), 8.0),
      (0.0, 0.0)
    );
  }

  #[test]
  fn moving_things_stretch_along_their_motion() {
    assert_eq!(stretch((0.0, 0.0), 1.0), (1.0, 1.0));
    assert_eq!(stretch((2400.0, 0.0), 0.0), (1.0, 1.0));

    let (across, down) = stretch((2400.0, 0.0), 1.0);
    assert!((across - 1.3).abs() < 1e-6 && (down - 0.85).abs() < 1e-6);
    let (across, down) = stretch((0.0, -9000.0), 0.5);
    assert!((across - 0.925).abs() < 1e-6 && (down - 1.15).abs() < 1e-6);
  }

  #[test]
  fn pulse_breathes_between_full_and_dimmed() {
    assert!((pulse(0.0) - 1.0).abs() < 1e-6);
    assert!((pulse(0.9) - 0.65).abs() < 1e-3);
    assert!((pulse(1.8) - 1.0).abs() < 1e-3);
  }
}
