/// Color, width, corner radius, and opacity for a
/// `NativeBorderOverlay`, bundled so the set of overlay knobs travels
/// as one value through `SessionOptions`/`ResizeSession`/
/// `upsert_overlay` instead of a same-typed positional-argument list
/// that's easy to mis-order at the many call sites (static sync,
/// workspace-switch, and move/resize/open/close tracking) that all
/// thread the same values. Mirrors `BackdropOverlayParams`.
///
/// Lives in `models` (rather than alongside `NativeBorderOverlay`) so
/// it's visible from both crate roots this crate builds under --
/// `lib.rs` for normal builds and the separate `test.rs` harness,
/// which only declares a subset of modules but always re-exports
/// `models::*`.
use crate::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderOverlayParams {
  /// Color of the border ring.
  pub color: Color,
  /// Ring thickness, in physical pixels.
  pub width: f32,
  /// Corner radius, in pixels, of the ring's outer edge.
  pub corner_radius: f32,
  /// Opacity of the overlay's whole composited visual, from `0.0` to
  /// `1.0`.
  pub opacity: f32,
  /// How far the ring lightens toward white at its top edge, from `0.0`
  /// (off) to `1.0`.
  pub highlight: f32,
  /// Brightness of a hairline along the ring's inner edge, like the edge
  /// of a pane of glass, from `0.0` (off) to `1.0`.
  pub rim: f32,
  /// A soft glow of the ring's color around it.
  pub glow: BorderGlow,
}

/// A soft glow of a border ring's color, fading out away from the ring.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderGlow {
  /// How far it reaches past the ring, in whole physical pixels; `0.0`
  /// draws none.
  pub reach: f32,
  /// Its opacity against the ring, from `0.0` to `1.0`.
  pub opacity: f32,
}

impl BorderGlow {
  /// Its opacity `distance` pixels past the ring, easing out to none at
  /// `reach`.
  #[must_use]
  pub fn opacity_at(&self, distance: f32) -> f32 {
    if self.reach <= 0.0 {
      return 0.0;
    }

    let fade = 1.0 - (distance / self.reach).clamp(0.0, 1.0);
    self.opacity.clamp(0.0, 1.0) * fade * fade
  }

  /// Interpolates its opacity `t` of the way to `to`. Mid-way it reaches
  /// as far as the farther of the two, so the overlay it outsets is only
  /// resized as a blend starts or ends.
  #[must_use]
  pub fn lerp(&self, to: &Self, t: f32) -> Self {
    let reach = if t <= 0.0 {
      self.reach
    } else if t >= 1.0 {
      to.reach
    } else {
      self.reach.max(to.reach)
    };

    Self {
      reach,
      opacity: crate::lerp_f32(self.opacity, to.opacity, t),
    }
  }

  /// Number of pixel-wide bands it is drawn in, outward from the ring.
  #[must_use]
  pub fn bands(&self) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
      self.reach.round().max(0.0) as usize
    }
  }

  /// The opacity of each band of a glow `t` of the way from `from` to
  /// `to` (see [`bands`](Self::bands)). Each band's own opacity is
  /// blended, so a glow fading into a shorter one changes shape smoothly
  /// rather than at the end.
  #[must_use]
  pub fn band_opacities(from: &Self, to: &Self, t: f32) -> Vec<f32> {
    (0..from.lerp(to, t).bands())
      .map(|band| {
        #[allow(clippy::cast_precision_loss)]
        let distance = band as f32 + 0.5;
        crate::lerp_f32(
          from.opacity_at(distance),
          to.opacity_at(distance),
          t,
        )
      })
      .collect()
  }
}

impl BorderOverlayParams {
  /// Interpolates every knob `t` of the way to `to`.
  #[must_use]
  pub fn lerp(&self, to: &Self, t: f32) -> Self {
    Self {
      color: self.color.lerp(&to.color, t),
      width: crate::lerp_f32(self.width, to.width, t),
      corner_radius: crate::lerp_f32(
        self.corner_radius,
        to.corner_radius,
        t,
      ),
      opacity: crate::lerp_f32(self.opacity, to.opacity, t),
      highlight: crate::lerp_f32(self.highlight, to.highlight, t),
      rim: crate::lerp_f32(self.rim, to.rim, t),
      glow: self.glow.lerp(&to.glow, t),
    }
  }

  /// How far its overlay reaches past the window: the ring and its glow.
  #[must_use]
  pub fn outset(&self) -> f32 {
    self.width + self.glow.reach
  }

  /// The ring's color at its top edge, lightened by `highlight`.
  #[must_use]
  pub fn lit_color(&self) -> Color {
    let white = Color {
      r: u8::MAX,
      g: u8::MAX,
      b: u8::MAX,
      a: self.color.a,
    };

    self.color.lerp(&white, self.highlight.clamp(0.0, 1.0))
  }

  /// The color of the hairline along the ring's inner edge: white, as
  /// opaque as `rim` is bright.
  #[must_use]
  pub fn rim_color(&self) -> Color {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha = (self.rim.clamp(0.0, 1.0) * 255.0).round() as u8;

    Color {
      r: u8::MAX,
      g: u8::MAX,
      b: u8::MAX,
      a: alpha,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn params(highlight: f32) -> BorderOverlayParams {
    BorderOverlayParams {
      color: Color {
        r: 100,
        g: 150,
        b: 200,
        a: 128,
      },
      width: 2.0,
      corner_radius: 0.0,
      opacity: 1.0,
      highlight,
      rim: 0.0,
      glow: BorderGlow::default(),
    }
  }

  fn glow(reach: f32, opacity: f32) -> BorderGlow {
    BorderGlow { reach, opacity }
  }

  #[test]
  fn lit_color_lightens_toward_white_keeping_alpha() {
    assert_eq!(params(0.0).lit_color(), params(0.0).color);
    assert_eq!(
      params(0.5).lit_color(),
      Color {
        r: 178,
        g: 203,
        b: 228,
        a: 128,
      }
    );
    assert_eq!(
      params(2.0).lit_color(),
      Color {
        r: 255,
        g: 255,
        b: 255,
        a: 128,
      }
    );
  }

  #[test]
  fn rim_color_is_white_as_opaque_as_the_rim_is_bright() {
    let rim = |rim| BorderOverlayParams { rim, ..params(0.0) }.rim_color();

    assert_eq!(rim(0.0).a, 0);
    assert_eq!(rim(0.5).a, 128);
    assert_eq!(
      rim(3.0),
      Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
      }
    );
  }

  #[test]
  fn glow_eases_out_to_none_at_its_reach() {
    let glow = glow(10.0, 0.8);

    assert!((glow.opacity_at(0.0) - 0.8).abs() < 1e-6);
    assert!((glow.opacity_at(5.0) - 0.2).abs() < 1e-6);
    assert!(glow.opacity_at(10.0).abs() < 1e-6);
    assert!(glow.opacity_at(20.0).abs() < 1e-6);
    assert!(BorderGlow::default().opacity_at(0.0).abs() < 1e-6);
  }

  #[test]
  fn glow_reaches_the_farther_of_two_mid_blend() {
    let (near, far) = (glow(4.0, 0.5), glow(12.0, 0.5));

    assert_eq!(near.lerp(&far, 0.0).bands(), 4);
    assert_eq!(near.lerp(&far, 0.5).bands(), 12);
    assert_eq!(far.lerp(&near, 0.5).bands(), 12);
    assert_eq!(far.lerp(&near, 1.0).bands(), 4);
  }

  #[test]
  fn glow_bands_blend_each_band() {
    let (off, on) = (BorderGlow::default(), glow(4.0, 1.0));

    let settled = BorderGlow::band_opacities(&on, &on, 1.0);
    assert_eq!(settled.len(), 4);
    assert!(settled.windows(2).all(|pair| pair[0] > pair[1]));

    let half = BorderGlow::band_opacities(&off, &on, 0.5);
    assert_eq!(half.len(), 4);
    for (half, settled) in half.iter().zip(&settled) {
      assert!((half - settled / 2.0).abs() < 1e-6);
    }

    assert!(BorderGlow::band_opacities(&on, &off, 1.0).is_empty());
  }

  #[test]
  fn outset_takes_in_the_glow() {
    let params = BorderOverlayParams {
      glow: glow(6.0, 0.5),
      ..params(0.0)
    };

    assert!((params.outset() - 8.0).abs() < f32::EPSILON);
  }
}
