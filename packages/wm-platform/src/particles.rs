//! Bursts of particles a window throws off, e.g. sparks as a resize lands.
//!
//! Every particle's path is worked out here, ahead of time, so the
//! compositor can play a whole burst on its own (see `native_particles`)
//! with the WM drawing no frame of it.

use std::time::Duration;

use crate::{Color, Rect};

/// How a burst's particles look and move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleStyle {
  /// Small bright sparks thrown out from the edges, falling as they fade.
  Sparks,
  /// Soft dust drifting out and up from the edges, as a window vanishes.
  Dust,
}

/// Which edges of a rect a burst comes from.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleEdges {
  pub left: bool,
  pub top: bool,
  pub right: bool,
  pub bottom: bool,
}

impl ParticleEdges {
  pub const ALL: Self = Self {
    left: true,
    top: true,
    right: true,
    bottom: true,
  };

  /// The edges of `to` that moved from where they were in `from`.
  #[must_use]
  pub fn moved(from: &Rect, to: &Rect) -> Self {
    Self {
      left: from.left != to.left,
      top: from.top != to.top,
      right: from.right != to.right,
      bottom: from.bottom != to.bottom,
    }
  }

  #[must_use]
  pub fn any(&self) -> bool {
    self.left || self.top || self.right || self.bottom
  }
}

/// A burst of particles from the edges of a rect on screen.
#[derive(Clone, Debug)]
pub struct ParticleBurst {
  /// The rect whose `edges` the particles come from.
  pub rect: Rect,
  pub edges: ParticleEdges,
  /// Area the particles are cut to, e.g. the rect's monitor.
  pub clip: Rect,
  pub style: ParticleStyle,
  pub color: Color,
  /// Particle diameter, in pixels.
  pub size: f32,
  /// How many particles, as a multiple of the style's own amount.
  pub density: f32,
  /// How long a particle lasts, give or take.
  pub duration: Duration,
  /// How long the burst waits before it starts.
  pub delay: Duration,
}

/// Points on a particle's path, evenly spaced over its life, which the
/// compositor joins with straight lines.
pub(crate) const PATH_KEYS: usize = 7;

/// One particle of a planned burst.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Particle {
  /// Where its center is at each of [`PATH_KEYS`] even steps of its life,
  /// relative to the burst's bounds.
  pub path: [(f32, f32); PATH_KEYS],
  /// Diameter it starts at, in pixels.
  pub size: f32,
  /// Opacity it starts at, fading out from there.
  pub opacity: f32,
  /// Its size at the end, as a multiple of `size`.
  pub end_scale: f32,
  pub delay: Duration,
  pub duration: Duration,
}

/// A burst worked out: the part of the screen it covers and its
/// particles.
#[derive(Clone, Debug)]
pub(crate) struct BurstPlan {
  /// Covers every particle all its life, cut to the burst's clip.
  pub bounds: Rect,
  pub particles: Vec<Particle>,
  /// How long until the last particle is gone.
  pub lifetime: Duration,
}

/// How a style's particles are spawned and move, in pixels and seconds.
struct Motion {
  /// Length of edge per particle at a density of 1.
  spacing: f32,
  /// Particles per edge, before scaling by density.
  per_edge: (usize, usize),
  /// Furthest a particle sets off from its edge's outward normal, in
  /// radians.
  spread: f32,
  speed: (f32, f32),
  /// Rate its velocity decays at.
  drag: f32,
  /// Downward acceleration; negative rises.
  gravity: f32,
  /// Where it starts along the edge's outward normal; negative is inside.
  start: (f32, f32),
  /// Its size and life, as multiples of the burst's.
  size: (f32, f32),
  life: (f32, f32),
  opacity: f32,
  end_scale: f32,
  /// Longest it starts after the burst does, so they don't all set off at
  /// once.
  stagger: Duration,
}

impl ParticleStyle {
  fn motion(self) -> Motion {
    match self {
      Self::Sparks => Motion {
        spacing: 36.0,
        per_edge: (3, 24),
        spread: 55.0_f32.to_radians(),
        speed: (120.0, 380.0),
        drag: 3.0,
        gravity: 700.0,
        start: (0.0, 2.0),
        size: (0.5, 1.0),
        life: (0.6, 1.0),
        opacity: 1.0,
        end_scale: 0.3,
        stagger: Duration::from_millis(50),
      },
      Self::Dust => Motion {
        spacing: 44.0,
        per_edge: (3, 18),
        spread: 70.0_f32.to_radians(),
        speed: (25.0, 90.0),
        drag: 1.2,
        gravity: -120.0,
        start: (-20.0, 0.0),
        size: (1.3, 2.4),
        life: (0.9, 1.35),
        opacity: 0.55,
        end_scale: 1.8,
        stagger: Duration::from_millis(90),
      },
    }
  }
}

/// `SplitMix64`: small, fast and good enough to scatter particles.
struct Rng(u64);

impl Rng {
  fn next_u64(&mut self) -> u64 {
    self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = self.0;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
  }

  /// Uniform in `[0, 1)`.
  #[allow(clippy::cast_precision_loss)]
  fn unit(&mut self) -> f32 {
    (self.next_u64() >> 40) as f32 / (1_u64 << 24) as f32
  }

  fn range(&mut self, (low, high): (f32, f32)) -> f32 {
    low + (high - low) * self.unit()
  }
}

/// One edge of the burst's rect: where it runs and its outward normal.
struct Edge {
  from: (f32, f32),
  to: (f32, f32),
  normal: (f32, f32),
}

impl Edge {
  fn length(&self) -> f32 {
    (self.to.0 - self.from.0).hypot(self.to.1 - self.from.1)
  }
}

#[allow(clippy::cast_precision_loss)]
fn edges(rect: &Rect, which: ParticleEdges) -> Vec<Edge> {
  let (left, top, right, bottom) = (
    rect.left as f32,
    rect.top as f32,
    rect.right as f32,
    rect.bottom as f32,
  );

  [
    (which.left, (left, top), (left, bottom), (-1.0, 0.0)),
    (which.top, (left, top), (right, top), (0.0, -1.0)),
    (which.right, (right, top), (right, bottom), (1.0, 0.0)),
    (which.bottom, (left, bottom), (right, bottom), (0.0, 1.0)),
  ]
  .into_iter()
  .filter(|(is_emitting, ..)| *is_emitting)
  .map(|(_, from, to, normal)| Edge { from, to, normal })
  .collect()
}

/// Where a particle set off at `velocity` from `start` is `t` seconds on,
/// slowed by `drag` and pulled by `gravity`.
fn position_at(
  start: (f32, f32),
  velocity: (f32, f32),
  motion: &Motion,
  t: f32,
) -> (f32, f32) {
  let drag = motion.drag.max(1e-3);
  // How far a unit of the initial velocity carries it by `t`.
  let carry = (1.0 - (-drag * t).exp()) / drag;
  let fall = motion.gravity / drag * (t - carry);

  (
    start.0 + velocity.0 * carry,
    start.1 + velocity.1 * carry + fall,
  )
}

/// Works out `burst`, scattered by `seed`, or `None` when it has nothing
/// to show: no edges, no density, or nothing of it inside its clip.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
pub(crate) fn plan(burst: &ParticleBurst, seed: u64) -> Option<BurstPlan> {
  if burst.density <= 0.0 || burst.size <= 0.0 {
    return None;
  }

  let motion = burst.style.motion();
  let mut rng = Rng(seed);
  let mut particles = Vec::new();
  let mut extent = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);

  for edge in edges(&burst.rect, burst.edges) {
    let wanted = (edge.length() / motion.spacing * burst.density).round();
    let fewest = motion.per_edge.0 as f32;
    let most = (motion.per_edge.1 as f32 * burst.density)
      .ceil()
      .max(fewest);
    let count = wanted.clamp(fewest, most) as usize;

    for _ in 0..count {
      let along = rng.unit();
      let out = rng.range(motion.start);
      let start = (
        edge.from.0
          + (edge.to.0 - edge.from.0) * along
          + edge.normal.0 * out,
        edge.from.1
          + (edge.to.1 - edge.from.1) * along
          + edge.normal.1 * out,
      );

      let angle = rng.range((-motion.spread, motion.spread));
      let pace = rng.range(motion.speed);
      let (sin, cos) = angle.sin_cos();
      let velocity = (
        (edge.normal.0 * cos - edge.normal.1 * sin) * pace,
        (edge.normal.0 * sin + edge.normal.1 * cos) * pace,
      );

      let life = burst.duration.mul_f32(rng.range(motion.life));
      let size = burst.size * rng.range(motion.size);
      let path: [(f32, f32); PATH_KEYS] = std::array::from_fn(|key| {
        let t = life.as_secs_f32() * key as f32 / (PATH_KEYS - 1) as f32;
        position_at(start, velocity, &motion, t)
      });

      let radius = size * motion.end_scale.max(1.0) / 2.0 + 1.0;
      for (x, y) in path {
        extent = (
          extent.0.min(x - radius),
          extent.1.min(y - radius),
          extent.2.max(x + radius),
          extent.3.max(y + radius),
        );
      }

      particles.push(Particle {
        path,
        size,
        opacity: motion.opacity,
        end_scale: motion.end_scale,
        delay: burst.delay + motion.stagger.mul_f32(rng.unit()),
        duration: life,
      });
    }
  }

  if particles.is_empty() {
    return None;
  }

  let bounds = Rect::from_ltrb(
    extent.0.floor() as i32,
    extent.1.floor() as i32,
    extent.2.ceil() as i32,
    extent.3.ceil() as i32,
  )
  .intersection(&burst.clip)?;

  let origin = (bounds.left as f32, bounds.top as f32);
  for particle in &mut particles {
    for point in &mut particle.path {
      *point = (point.0 - origin.0, point.1 - origin.1);
    }
  }

  let lifetime = particles
    .iter()
    .map(|particle| particle.delay + particle.duration)
    .max()
    .unwrap_or_default();

  Some(BurstPlan {
    bounds,
    particles,
    lifetime,
  })
}

/// A seed that differs from burst to burst.
pub(crate) fn fresh_seed() -> u64 {
  use std::sync::atomic::{AtomicU64, Ordering};

  static COUNTER: AtomicU64 = AtomicU64::new(0);

  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.subsec_nanos());

  u64::from(nanos)
    ^ COUNTER
      .fetch_add(1, Ordering::Relaxed)
      .wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn burst(style: ParticleStyle, edges: ParticleEdges) -> ParticleBurst {
    ParticleBurst {
      rect: Rect::from_ltrb(400, 300, 1200, 900),
      edges,
      clip: Rect::from_ltrb(0, 0, 1920, 1080),
      style,
      color: Color {
        r: 255,
        g: 200,
        b: 120,
        a: 255,
      },
      size: 5.0,
      density: 1.0,
      duration: Duration::from_millis(650),
      delay: Duration::from_millis(150),
    }
  }

  fn right_edge() -> ParticleEdges {
    ParticleEdges {
      right: true,
      ..ParticleEdges::default()
    }
  }

  fn starts(plan: &BurstPlan) -> Vec<(f32, f32)> {
    #[allow(clippy::cast_precision_loss)]
    let origin = (plan.bounds.left as f32, plan.bounds.top as f32);
    plan
      .particles
      .iter()
      .map(|particle| {
        (particle.path[0].0 + origin.0, particle.path[0].1 + origin.1)
      })
      .collect()
  }

  #[test]
  fn moved_edges_are_the_ones_that_changed() {
    let from = Rect::from_ltrb(0, 0, 100, 100);
    let to = Rect::from_ltrb(0, 0, 140, 100);

    assert_eq!(ParticleEdges::moved(&from, &to), right_edge());
    assert!(!ParticleEdges::moved(&from, &from).any());
  }

  #[test]
  fn sparks_come_from_the_moved_edge_only() {
    let plan = plan(&burst(ParticleStyle::Sparks, right_edge()), 7)
      .expect("a burst");

    // 600px of edge at one spark per 36px.
    assert_eq!(plan.particles.len(), 17);
    for (x, y) in starts(&plan) {
      assert!((1200.0..=1202.5).contains(&x), "x {x}");
      assert!((300.0..=900.0).contains(&y), "y {y}");
    }
  }

  #[test]
  fn sparks_fly_outward_and_fall() {
    let plan = plan(&burst(ParticleStyle::Sparks, right_edge()), 11)
      .expect("a burst");

    for particle in &plan.particles {
      let (start, end) = (particle.path[0], particle.path[PATH_KEYS - 1]);
      assert!(end.0 > start.0, "outward");
      assert!(end.1 > start.1 - 200.0, "no rocket");
    }
    let falls = plan
      .particles
      .iter()
      .filter(|particle| {
        particle.path[PATH_KEYS - 1].1 > particle.path[0].1
      })
      .count();
    assert!(falls * 2 > plan.particles.len(), "most fall");
  }

  #[test]
  fn dust_rises_from_inside_every_edge() {
    let plan = plan(&burst(ParticleStyle::Dust, ParticleEdges::ALL), 3)
      .expect("a burst");

    let rises = plan
      .particles
      .iter()
      .filter(|particle| {
        particle.path[PATH_KEYS - 1].1 < particle.path[0].1
      })
      .count();
    assert!(rises * 2 > plan.particles.len(), "most rise");
    for (x, y) in starts(&plan) {
      assert!(
        (380.0..=1220.0).contains(&x) && (280.0..=920.0).contains(&y)
      );
    }
  }

  #[test]
  fn density_scales_the_amount_within_limits() {
    let mut dense = burst(ParticleStyle::Sparks, right_edge());
    dense.density = 2.0;
    let mut sparse = burst(ParticleStyle::Sparks, right_edge());
    sparse.density = 0.01;

    assert_eq!(plan(&dense, 1).expect("a burst").particles.len(), 33);
    assert_eq!(plan(&sparse, 1).expect("a burst").particles.len(), 3);

    sparse.density = 0.0;
    assert!(plan(&sparse, 1).is_none());
  }

  #[test]
  fn every_particle_stays_inside_the_bounds() {
    let plan = plan(&burst(ParticleStyle::Sparks, ParticleEdges::ALL), 5)
      .expect("a burst");

    #[allow(clippy::cast_precision_loss)]
    let (width, height) =
      (plan.bounds.width() as f32, plan.bounds.height() as f32);
    for particle in &plan.particles {
      for (x, y) in particle.path {
        assert!((0.0..=width).contains(&x) && (0.0..=height).contains(&y));
      }
    }
  }

  #[test]
  fn bursts_are_cut_to_their_clip() {
    let mut off_screen = burst(ParticleStyle::Sparks, right_edge());
    off_screen.clip = Rect::from_ltrb(0, 0, 300, 300);
    assert!(plan(&off_screen, 1).is_none());

    let mut cut = burst(ParticleStyle::Sparks, right_edge());
    cut.clip = Rect::from_ltrb(0, 0, 1210, 1080);
    let bounds = plan(&cut, 1).expect("a burst").bounds;
    assert!(bounds.right <= 1210);
  }

  #[test]
  fn lifetime_covers_the_last_particle() {
    let plan = plan(&burst(ParticleStyle::Dust, ParticleEdges::ALL), 9)
      .expect("a burst");

    for particle in &plan.particles {
      assert!(particle.delay >= Duration::from_millis(150));
      assert!(particle.delay + particle.duration <= plan.lifetime);
    }
  }

  #[test]
  fn same_seed_same_burst() {
    let burst = burst(ParticleStyle::Sparks, ParticleEdges::ALL);

    assert_eq!(
      plan(&burst, 42).expect("a burst").particles,
      plan(&burst, 42).expect("a burst").particles
    );
    assert_ne!(
      plan(&burst, 42).expect("a burst").particles,
      plan(&burst, 43).expect("a burst").particles
    );
  }
}
