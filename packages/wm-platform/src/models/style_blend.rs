/// An overlay style partway through a transition, `t` of the way from
/// `from` to `to`.
///
/// Carried whole rather than pre-interpolated because not every knob can
/// be: a backdrop's baked knobs select a pre-rendered surface, so the
/// overlay crossfades two surfaces instead of rendering in-between values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleBlend<P> {
  pub from: P,
  pub to: P,
  /// Progress from `0.0` (`from`) to `1.0` (`to`); values outside that
  /// range hold at the nearest endpoint.
  pub t: f32,
}

impl<P: Copy> StyleBlend<P> {
  /// A blend that has already arrived at `params`.
  #[must_use]
  pub fn settled(params: P) -> Self {
    Self {
      from: params,
      to: params,
      t: 1.0,
    }
  }

  /// Blends between two styles that may each be off, or `None` when both
  /// are.
  ///
  /// A missing side takes the other side's style through `transparent`,
  /// so the overlay fades in or out in place instead of popping.
  #[must_use]
  pub fn between(
    from: Option<P>,
    to: Option<P>,
    t: f32,
    transparent: impl Fn(P) -> P,
  ) -> Option<Self> {
    let (from, to) = match (from, to) {
      (Some(from), Some(to)) => (from, to),
      (Some(from), None) => (from, transparent(from)),
      (None, Some(to)) => (transparent(to), to),
      (None, None) => return None,
    };

    Some(Self { from, to, t })
  }
}

/// Linearly interpolates from `from` to `to`.
///
/// Returns the endpoints exactly at `t <= 0.0` and `t >= 1.0`, so a
/// settled transition hands overlays the very value steady-state sync
/// resolves -- their setters skip re-applying on exact equality.
#[must_use]
pub fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
  if t <= 0.0 {
    from
  } else if t >= 1.0 {
    to
  } else {
    from + (to - from) * t
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lerp_f32_hits_endpoints_exactly() {
    assert!((lerp_f32(0.1, 0.7, 0.0) - 0.1).abs() < f32::EPSILON);
    assert!((lerp_f32(0.1, 0.7, 1.0) - 0.7).abs() < f32::EPSILON);
    assert!((lerp_f32(0.1, 0.7, 1.5) - 0.7).abs() < f32::EPSILON);
    assert!((lerp_f32(0.1, 0.7, -0.5) - 0.1).abs() < f32::EPSILON);
    assert!((lerp_f32(2.0, 6.0, 0.25) - 3.0).abs() < f32::EPSILON);
  }

  #[test]
  fn between_fades_a_missing_side() {
    let fade = |value: f32| value - 10.0;

    let blend = StyleBlend::between(Some(3.0), None, 0.5, fade);
    assert_eq!(blend.map(|b| (b.from, b.to)), Some((3.0, -7.0)));

    let blend = StyleBlend::between(None, Some(3.0), 0.5, fade);
    assert_eq!(blend.map(|b| (b.from, b.to)), Some((-7.0, 3.0)));

    let blend = StyleBlend::between(Some(1.0), Some(2.0), 0.5, fade);
    assert_eq!(blend.map(|b| (b.from, b.to)), Some((1.0, 2.0)));

    assert!(StyleBlend::<f32>::between(None, None, 0.5, fade).is_none());
  }
}
