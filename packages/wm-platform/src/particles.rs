//! Bursts of particles thrown off by windows and the overview, e.g.
//! sparks as a resize lands.
//!
//! Every particle's path, fade and spin are worked out here, ahead of
//! time, so the compositor can play a whole burst on its own (see
//! `native_particles`) with nothing drawing a frame of it.

use std::{f32::consts::TAU, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{Color, Rect};

/// How a burst's particles look and move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleStyle {
  /// Small bright sparks thrown out, falling as they fade.
  Sparks,
  /// Soft dust drifting out and up.
  Dust,
  /// Warm specks floating up and flickering out.
  Embers,
  /// Flecks in several colors, tumbling and fluttering down.
  Confetti,
  /// Tiny twinkles popping in place, one after another.
  Glints,
  /// Rings rising and wobbling, swelling as they pop.
  Bubbles,
}

/// Which particles an event throws off, if any, as configured.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ParticleEffect {
  #[default]
  None,
  Sparks,
  Dust,
  Embers,
  Confetti,
  Glints,
  Bubbles,
}

impl ParticleEffect {
  /// The style it throws particles off in, or `None` for none.
  #[must_use]
  pub fn style(self) -> Option<ParticleStyle> {
    match self {
      Self::None => None,
      Self::Sparks => Some(ParticleStyle::Sparks),
      Self::Dust => Some(ParticleStyle::Dust),
      Self::Embers => Some(ParticleStyle::Embers),
      Self::Confetti => Some(ParticleStyle::Confetti),
      Self::Glints => Some(ParticleStyle::Glints),
      Self::Bubbles => Some(ParticleStyle::Bubbles),
    }
  }
}

/// How particle bursts look, wherever they are thrown off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleLook {
  /// Their color, or `None` for the color of whatever throws them off.
  pub color: Option<Color>,
  /// Particle diameter, in logical pixels.
  pub size: f32,
  /// How many particles, as a multiple of each style's own amount.
  pub density: f32,
  /// How long a particle lasts, give or take.
  pub duration: Duration,
}

impl Default for ParticleLook {
  fn default() -> Self {
    Self {
      color: None,
      size: 5.0,
      density: 1.0,
      duration: Duration::from_millis(650),
    }
  }
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

/// Where on screen a burst's particles set off from.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticleSource {
  /// The given edges of a rect, setting off outward.
  Edges(Rect, ParticleEdges),
  /// Around a point, setting off every way.
  Point(f32, f32),
}

/// A burst of particles on screen.
#[derive(Clone, Debug)]
pub struct ParticleBurst {
  pub source: ParticleSource,
  /// Area the particles are cut to, e.g. the source's monitor.
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
  /// Whether it shows over topmost windows too, e.g. over the overview.
  pub is_topmost: bool,
}

/// Points on a particle's path, evenly spaced over its life, which the
/// compositor joins with straight lines.
pub(crate) const PATH_KEYS: usize = 12;

/// One particle of a planned burst.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Particle {
  /// Where its center is at each of [`PATH_KEYS`] even steps of its life,
  /// relative to the burst's bounds.
  pub path: [(f32, f32); PATH_KEYS],
  /// Width and height it starts at, in pixels.
  pub size: (f32, f32),
  pub color: Color,
  /// Its opacity and its scale over its life, as `(progress, value)` keys.
  pub opacity: Vec<(f32, f32)>,
  pub scale: Vec<(f32, f32)>,
  /// How far it turns over its life, in degrees.
  pub spin: f32,
  pub delay: Duration,
  pub duration: Duration,
}

/// A burst worked out: the part of the screen it covers and its
/// particles.
#[derive(Clone, Debug)]
pub(crate) struct BurstPlan {
  pub style: ParticleStyle,
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
  /// Particles around a point at a density of 1, and how far from it they
  /// set off.
  per_point: usize,
  point_radius: f32,
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
  /// How far it sways sideways, and how many times a second.
  sway: (f32, f32),
  /// Its size and life, as multiples of the burst's, and its height as a
  /// multiple of its width.
  size: (f32, f32),
  aspect: (f32, f32),
  life: (f32, f32),
  /// Most it turns over its life either way, in degrees.
  spin: f32,
  /// Longest it starts after the burst does, as a share of the burst's
  /// duration.
  stagger: f32,
  opacity: &'static [(f32, f32)],
  scale: &'static [(f32, f32)],
  /// Most each opacity key between the first and last is nudged either
  /// way, so particles flicker apart.
  flicker: f32,
}

impl ParticleStyle {
  #[allow(clippy::too_many_lines)]
  fn motion(self) -> Motion {
    match self {
      Self::Sparks => Motion {
        spacing: 36.0,
        per_edge: (3, 24),
        per_point: 14,
        point_radius: 6.0,
        spread: 55.0_f32.to_radians(),
        speed: (120.0, 380.0),
        drag: 3.0,
        gravity: 700.0,
        start: (0.0, 2.0),
        sway: (0.0, 0.0),
        size: (0.5, 1.0),
        aspect: (1.0, 1.0),
        life: (0.6, 1.0),
        spin: 0.0,
        stagger: 0.08,
        opacity: &[(0.0, 1.0), (0.6, 0.75), (1.0, 0.0)],
        scale: &[(0.0, 1.0), (1.0, 0.3)],
        flicker: 0.0,
      },
      Self::Dust => Motion {
        spacing: 44.0,
        per_edge: (3, 18),
        per_point: 12,
        point_radius: 10.0,
        spread: 70.0_f32.to_radians(),
        speed: (25.0, 90.0),
        drag: 1.2,
        gravity: -120.0,
        start: (-20.0, 0.0),
        sway: (0.0, 0.0),
        size: (1.3, 2.4),
        aspect: (1.0, 1.0),
        life: (0.9, 1.35),
        spin: 0.0,
        stagger: 0.14,
        opacity: &[(0.0, 0.55), (1.0, 0.0)],
        scale: &[(0.0, 1.0), (1.0, 1.8)],
        flicker: 0.0,
      },
      Self::Embers => Motion {
        spacing: 52.0,
        per_edge: (2, 14),
        per_point: 10,
        point_radius: 8.0,
        spread: 40.0_f32.to_radians(),
        speed: (20.0, 70.0),
        drag: 1.0,
        gravity: -160.0,
        start: (-2.0, 2.0),
        sway: (5.0, 1.3),
        size: (0.5, 1.0),
        aspect: (1.0, 1.0),
        life: (1.2, 1.8),
        spin: 0.0,
        stagger: 0.25,
        opacity: &[
          (0.0, 0.9),
          (0.15, 1.0),
          (0.3, 0.55),
          (0.45, 0.95),
          (0.6, 0.5),
          (0.78, 0.8),
          (1.0, 0.0),
        ],
        scale: &[(0.0, 1.0), (1.0, 0.5)],
        flicker: 0.15,
      },
      Self::Confetti => Motion {
        spacing: 30.0,
        per_edge: (4, 28),
        per_point: 24,
        point_radius: 6.0,
        spread: 60.0_f32.to_radians(),
        speed: (150.0, 420.0),
        drag: 2.5,
        gravity: 520.0,
        start: (0.0, 2.0),
        sway: (10.0, 2.0),
        size: (0.9, 1.5),
        aspect: (0.35, 0.6),
        life: (0.9, 1.3),
        spin: 540.0,
        stagger: 0.06,
        opacity: &[(0.0, 1.0), (0.7, 1.0), (1.0, 0.0)],
        scale: &[(0.0, 1.0), (1.0, 0.9)],
        flicker: 0.0,
      },
      Self::Glints => Motion {
        spacing: 70.0,
        per_edge: (2, 12),
        per_point: 10,
        point_radius: 40.0,
        spread: 0.0,
        speed: (0.0, 0.0),
        drag: 1.0,
        gravity: 0.0,
        start: (-3.0, 4.0),
        sway: (0.0, 0.0),
        size: (0.8, 1.6),
        aspect: (1.0, 1.0),
        life: (0.3, 0.55),
        spin: 0.0,
        stagger: 0.7,
        opacity: &[(0.0, 0.0), (0.3, 1.0), (1.0, 0.0)],
        scale: &[(0.0, 0.2), (0.3, 1.2), (1.0, 0.0)],
        flicker: 0.0,
      },
      Self::Bubbles => Motion {
        spacing: 64.0,
        per_edge: (2, 10),
        per_point: 8,
        point_radius: 12.0,
        spread: 50.0_f32.to_radians(),
        speed: (10.0, 40.0),
        drag: 0.8,
        gravity: -90.0,
        start: (-4.0, 4.0),
        sway: (6.0, 1.5),
        size: (1.6, 3.2),
        aspect: (1.0, 1.0),
        life: (1.0, 1.5),
        spin: 0.0,
        stagger: 0.2,
        opacity: &[(0.0, 0.0), (0.15, 0.8), (0.85, 0.7), (1.0, 0.0)],
        scale: &[(0.0, 0.5), (0.85, 1.0), (1.0, 1.35)],
        flicker: 0.0,
      },
    }
  }

  /// The colors its particles take, from a burst in `color`.
  fn palette(self, color: Color) -> Vec<Color> {
    // Bright, distinct hues for confetti thrown off in a color too gray
    // to vary.
    const FESTIVE: [[u8; 3]; 5] = [
      [0xFF, 0x5C, 0x7A],
      [0xFF, 0xD1, 0x66],
      [0x06, 0xD6, 0xA0],
      [0x4C, 0xC9, 0xF0],
      [0xC7, 0x7D, 0xFF],
    ];

    if self != Self::Confetti {
      return vec![color];
    }

    if saturation(color) < 0.15 {
      return FESTIVE
        .iter()
        .map(|&[r, g, b]| Color {
          r,
          g,
          b,
          a: u8::MAX,
        })
        .collect();
    }

    [0.0, 72.0, 144.0, 216.0, 288.0]
      .iter()
      .map(|degrees| rotate_hue(color, *degrees))
      .collect()
  }
}

/// How saturated `color` is, from `0.0` (gray) to `1.0`.
fn saturation(color: Color) -> f32 {
  let (max, min) = (
    color.r.max(color.g).max(color.b),
    color.r.min(color.g).min(color.b),
  );
  if max == 0 {
    return 0.0;
  }
  f32::from(max - min) / f32::from(max)
}

/// `color` with its hue turned `degrees` round the color wheel, keeping
/// its saturation, brightness and alpha.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rotate_hue(color: Color, degrees: f32) -> Color {
  let [r, g, b] = [color.r, color.g, color.b].map(|c| f32::from(c) / 255.0);
  let max = r.max(g).max(b);
  let chroma = max - r.min(g).min(b);

  let hue = if chroma <= f32::EPSILON {
    0.0
  } else if (max - r).abs() <= f32::EPSILON {
    60.0 * ((g - b) / chroma).rem_euclid(6.0)
  } else if (max - g).abs() <= f32::EPSILON {
    60.0 * ((b - r) / chroma + 2.0)
  } else {
    60.0 * ((r - g) / chroma + 4.0)
  };

  let hue = (hue + degrees).rem_euclid(360.0);
  let second = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
  let (r, g, b) = match (hue / 60.0) as u32 {
    0 => (chroma, second, 0.0),
    1 => (second, chroma, 0.0),
    2 => (0.0, chroma, second),
    3 => (0.0, second, chroma),
    4 => (second, 0.0, chroma),
    _ => (chroma, 0.0, second),
  };
  let low = max - chroma;
  let to_u8 =
    |value: f32| ((value + low) * 255.0).round().clamp(0.0, 255.0) as u8;

  Color {
    r: to_u8(r),
    g: to_u8(g),
    b: to_u8(b),
    a: color.a,
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

  /// `-1.0` or `1.0`, evenly.
  fn sign(&mut self) -> f32 {
    if self.next_u64() & 1 == 0 {
      -1.0
    } else {
      1.0
    }
  }

  /// One of `0..count`, evenly; `count` must not be zero.
  #[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
  )]
  fn index(&mut self, count: usize) -> usize {
    ((self.unit() * count as f32) as usize).min(count - 1)
  }
}

/// Where a particle sets off from and which way it heads first.
struct Launch {
  start: (f32, f32),
  heading: (f32, f32),
}

/// Where `burst`'s particles set off from and head, before spreading.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
fn launches(
  burst: &ParticleBurst,
  motion: &Motion,
  rng: &mut Rng,
) -> Vec<Launch> {
  match &burst.source {
    ParticleSource::Edges(rect, which) => {
      let (left, top, right, bottom) = (
        rect.left as f32,
        rect.top as f32,
        rect.right as f32,
        rect.bottom as f32,
      );
      let edges = [
        (which.left, (left, top), (left, bottom), (-1.0, 0.0)),
        (which.top, (left, top), (right, top), (0.0, -1.0)),
        (which.right, (right, top), (right, bottom), (1.0, 0.0)),
        (which.bottom, (left, bottom), (right, bottom), (0.0, 1.0)),
      ];

      let mut launches = Vec::new();
      for (is_emitting, from, to, normal) in edges {
        if !is_emitting {
          continue;
        }

        let length = (to.0 - from.0).hypot(to.1 - from.1);
        let wanted = (length / motion.spacing * burst.density).round();
        let fewest = motion.per_edge.0 as f32;
        let most =
          (motion.per_edge.1 as f32 * burst.density).ceil().max(fewest);

        for _ in 0..wanted.clamp(fewest, most) as usize {
          let along = rng.unit();
          let out = rng.range(motion.start);
          launches.push(Launch {
            start: (
              from.0 + (to.0 - from.0) * along + normal.0 * out,
              from.1 + (to.1 - from.1) * along + normal.1 * out,
            ),
            heading: normal,
          });
        }
      }
      launches
    }
    ParticleSource::Point(x, y) => {
      let count =
        (motion.per_point as f32 * burst.density).round().max(1.0) as usize;

      (0..count)
        .map(|index| {
          // Evenly around, each nudged off its place, so none bunch up.
          let angle = (index as f32 + rng.unit() * 0.8) / count as f32 * TAU;
          let heading = (angle.cos(), angle.sin());
          let out = rng.range((0.0, motion.point_radius));
          Launch {
            start: (x + heading.0 * out, y + heading.1 * out),
            heading,
          }
        })
        .collect()
    }
  }
}

/// Where a particle set off at `velocity` from `start` is `t` seconds on,
/// slowed by drag, pulled by gravity and swaying from `phase`.
fn position_at(
  start: (f32, f32),
  velocity: (f32, f32),
  motion: &Motion,
  phase: f32,
  t: f32,
) -> (f32, f32) {
  let drag = motion.drag.max(1e-3);
  // How far a unit of the initial velocity carries it by `t`.
  let carry = (1.0 - (-drag * t).exp()) / drag;
  let fall = motion.gravity / drag * (t - carry);
  let (sway, rate) = motion.sway;

  (
    start.0 + velocity.0 * carry + sway * (TAU * rate * t + phase).sin(),
    start.1 + velocity.1 * carry + fall,
  )
}

/// Works out `burst`, scattered by `seed`, or `None` when it has nothing
/// to show: no density or size, or nothing of it inside its clip.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub(crate) fn plan(burst: &ParticleBurst, seed: u64) -> Option<BurstPlan> {
  if burst.density <= 0.0 || burst.size <= 0.0 {
    return None;
  }

  let motion = burst.style.motion();
  let palette = burst.style.palette(burst.color);
  let mut rng = Rng(seed);
  let mut particles = Vec::new();
  let mut extent = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
  let grows = motion
    .scale
    .iter()
    .fold(1.0_f32, |most, (_, scale)| most.max(*scale));

  for launch in launches(burst, &motion, &mut rng) {
    let angle = rng.range((-motion.spread, motion.spread));
    let pace = rng.range(motion.speed);
    let (sin, cos) = angle.sin_cos();
    let (hx, hy) = launch.heading;
    let velocity =
      ((hx * cos - hy * sin) * pace, (hx * sin + hy * cos) * pace);

    let life = burst.duration.mul_f32(rng.range(motion.life));
    let phase = rng.range((0.0, TAU));
    let path: [(f32, f32); PATH_KEYS] = std::array::from_fn(|key| {
      let t = life.as_secs_f32() * key as f32 / (PATH_KEYS - 1) as f32;
      position_at(launch.start, velocity, &motion, phase, t)
    });

    let width = burst.size * rng.range(motion.size);
    let size = (width, width * rng.range(motion.aspect));
    let reach = size.0.max(size.1) * grows / 2.0 + 1.0;
    for (x, y) in path {
      extent = (
        extent.0.min(x - reach),
        extent.1.min(y - reach),
        extent.2.max(x + reach),
        extent.3.max(y + reach),
      );
    }

    let last = motion.opacity.len().saturating_sub(1);
    let opacity = motion
      .opacity
      .iter()
      .enumerate()
      .map(|(index, (progress, value))| {
        let nudge = if index == 0 || index == last {
          0.0
        } else {
          rng.range((-motion.flicker, motion.flicker))
        };
        (*progress, (value + nudge).clamp(0.0, 1.0))
      })
      .collect();

    particles.push(Particle {
      path,
      size,
      color: palette[rng.index(palette.len())],
      opacity,
      scale: motion.scale.to_vec(),
      spin: motion.spin * rng.unit() * rng.sign(),
      delay: burst.delay
        + burst.duration.mul_f32(motion.stagger * rng.unit()),
      duration: life,
    });
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
    style: burst.style,
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

  const STYLES: [ParticleStyle; 6] = [
    ParticleStyle::Sparks,
    ParticleStyle::Dust,
    ParticleStyle::Embers,
    ParticleStyle::Confetti,
    ParticleStyle::Glints,
    ParticleStyle::Bubbles,
  ];

  const AMBER: Color = Color {
    r: 255,
    g: 200,
    b: 120,
    a: 255,
  };

  fn burst(style: ParticleStyle, edges: ParticleEdges) -> ParticleBurst {
    ParticleBurst {
      source: ParticleSource::Edges(
        Rect::from_ltrb(400, 300, 1200, 900),
        edges,
      ),
      clip: Rect::from_ltrb(0, 0, 1920, 1080),
      style,
      color: AMBER,
      size: 5.0,
      density: 1.0,
      duration: Duration::from_millis(650),
      delay: Duration::from_millis(150),
      is_topmost: false,
    }
  }

  fn right_edge() -> ParticleEdges {
    ParticleEdges {
      right: true,
      ..ParticleEdges::default()
    }
  }

  /// Where each particle starts and ends, on screen.
  fn ends(plan: &BurstPlan) -> Vec<((f32, f32), (f32, f32))> {
    #[allow(clippy::cast_precision_loss)]
    let origin = (plan.bounds.left as f32, plan.bounds.top as f32);
    let on_screen = |(x, y): (f32, f32)| (x + origin.0, y + origin.1);

    plan
      .particles
      .iter()
      .map(|particle| {
        (
          on_screen(particle.path[0]),
          on_screen(particle.path[PATH_KEYS - 1]),
        )
      })
      .collect()
  }

  /// How many of `plan`'s particles end above where they start.
  fn risers(plan: &BurstPlan) -> usize {
    ends(plan)
      .iter()
      .filter(|(start, end)| end.1 < start.1)
      .count()
  }

  #[test]
  fn effects_map_to_styles() {
    assert_eq!(ParticleEffect::None.style(), None);
    assert_eq!(
      ParticleEffect::Confetti.style(),
      Some(ParticleStyle::Confetti)
    );
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
    for ((x, y), _) in ends(&plan) {
      assert!((1200.0..=1202.5).contains(&x), "x {x}");
      assert!((300.0..=900.0).contains(&y), "y {y}");
    }
  }

  #[test]
  fn sparks_fly_outward_and_fall() {
    let plan = plan(&burst(ParticleStyle::Sparks, right_edge()), 11)
      .expect("a burst");

    let ends = ends(&plan);
    assert!(ends.iter().all(|(start, end)| end.0 > start.0), "outward");
    assert!(risers(&plan) * 2 < ends.len(), "most fall");
  }

  #[test]
  fn dust_embers_and_bubbles_rise() {
    for style in [
      ParticleStyle::Dust,
      ParticleStyle::Embers,
      ParticleStyle::Bubbles,
    ] {
      let plan =
        plan(&burst(style, ParticleEdges::ALL), 3).expect("a burst");
      assert!(risers(&plan) * 2 > plan.particles.len(), "{style:?} rise");
    }
  }

  #[test]
  fn confetti_tumbles_down_in_many_colors() {
    let plan = plan(&burst(ParticleStyle::Confetti, ParticleEdges::ALL), 5)
      .expect("a burst");

    assert!(risers(&plan) * 2 < plan.particles.len(), "most fall");
    assert!(plan
      .particles
      .iter()
      .any(|particle| particle.spin.abs() > 90.0));
    assert!(plan
      .particles
      .iter()
      .all(|particle| particle.size.1 < particle.size.0));

    let mut colors = plan
      .particles
      .iter()
      .map(|particle| particle.color.to_bgr())
      .collect::<Vec<_>>();
    colors.sort_unstable();
    colors.dedup();
    assert!(colors.len() >= 3, "{} colors", colors.len());
  }

  #[test]
  fn gray_confetti_takes_festive_colors() {
    let white = Color {
      r: 255,
      g: 255,
      b: 255,
      a: 255,
    };

    let palette = ParticleStyle::Confetti.palette(white);
    assert_eq!(palette.len(), 5);
    assert!(palette.iter().all(|color| saturation(*color) > 0.3));
    assert_eq!(ParticleStyle::Sparks.palette(white), vec![white]);
  }

  #[test]
  fn glints_twinkle_in_place_one_after_another() {
    let plan = plan(&burst(ParticleStyle::Glints, ParticleEdges::ALL), 9)
      .expect("a burst");

    for particle in &plan.particles {
      assert!(particle.path.iter().all(|point| *point == particle.path[0]));
    }
    let (first, last) = plan.particles.iter().fold(
      (Duration::MAX, Duration::ZERO),
      |(first, last), particle| {
        (first.min(particle.delay), last.max(particle.delay))
      },
    );
    assert!(last - first > Duration::from_millis(150));
  }

  #[test]
  fn point_bursts_go_every_way() {
    let mut burst = burst(ParticleStyle::Sparks, ParticleEdges::ALL);
    burst.source = ParticleSource::Point(960.0, 540.0);
    let plan = plan(&burst, 13).expect("a burst");

    assert_eq!(plan.particles.len(), 14);
    let ends = ends(&plan);
    assert!(ends.iter().any(|(_, end)| end.0 < 900.0));
    assert!(ends.iter().any(|(_, end)| end.0 > 1020.0));
  }

  #[test]
  fn every_style_fades_out_to_nothing() {
    for style in STYLES {
      let plan =
        plan(&burst(style, ParticleEdges::ALL), 1).expect("a burst");
      for particle in &plan.particles {
        let (progress, opacity) =
          *particle.opacity.last().expect("opacity keys");
        assert!((progress - 1.0).abs() < f32::EPSILON, "{style:?}");
        assert!(opacity.abs() < f32::EPSILON, "{style:?}");
        assert!(particle
          .opacity
          .iter()
          .all(|(_, opacity)| (0.0..=1.0).contains(opacity)));
      }
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
    for style in STYLES {
      let plan =
        plan(&burst(style, ParticleEdges::ALL), 5).expect("a burst");

      #[allow(clippy::cast_precision_loss)]
      let (width, height) =
        (plan.bounds.width() as f32, plan.bounds.height() as f32);
      for particle in &plan.particles {
        for (x, y) in particle.path {
          assert!(
            (0.0..=width).contains(&x) && (0.0..=height).contains(&y),
            "{style:?}"
          );
        }
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
    for style in STYLES {
      let plan =
        plan(&burst(style, ParticleEdges::ALL), 9).expect("a burst");
      for particle in &plan.particles {
        assert!(particle.delay >= Duration::from_millis(150));
        assert!(particle.delay + particle.duration <= plan.lifetime);
      }
    }
  }

  #[test]
  fn same_seed_same_burst() {
    let burst = burst(ParticleStyle::Confetti, ParticleEdges::ALL);

    assert_eq!(
      plan(&burst, 42).expect("a burst").particles,
      plan(&burst, 42).expect("a burst").particles
    );
    assert_ne!(
      plan(&burst, 42).expect("a burst").particles,
      plan(&burst, 43).expect("a burst").particles
    );
  }

  #[test]
  fn hue_turns_round_the_wheel() {
    let red = Color {
      r: 255,
      g: 0,
      b: 0,
      a: 200,
    };

    let green = rotate_hue(red, 120.0);
    assert_eq!((green.r, green.g, green.b, green.a), (0, 255, 0, 200));
    let blue = rotate_hue(red, -120.0);
    assert_eq!((blue.r, blue.g, blue.b), (0, 0, 255));
    assert_eq!(rotate_hue(AMBER, 360.0), AMBER);
  }
}
