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
    }
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
    }
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
}
