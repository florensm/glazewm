use crate::{rounded_backdrop::RoundedBackdrop, Color};

/// Tint, blur radius, corner radius, opacity, saturation and grading knobs
/// for a `NativeBackdropOverlay`, bundled so the growing set of overlay
/// knobs travels as one value through
/// `upsert_overlay` instead of a same-typed positional-argument list
/// that's easy to mis-order at the many call sites (static sync,
/// workspace-switch, and move/resize/open/close tracking) that all thread
/// the same values.
///
/// Mirrors `BackdropEffectConfig`, which documents what each knob is for;
/// only the differences from it are noted here. `corner_radius` is the one
/// field with no config counterpart -- it is derived from the window's
/// corner style so the overlay can't disagree with the window's own
/// rendered corners.
///
/// Every knob except `corner_radius`, `vignette` and `parallax` is baked
/// into the per-monitor wallpaper image, so changing one re-renders that
/// image once rather than costing anything per frame.
///
/// Lives in `models` (rather than alongside `NativeBackdropOverlay`) so
/// it's visible from both crate roots this crate builds under -- `lib.rs`
/// for normal builds and the separate `test.rs` harness, which only
/// declares a subset of modules but always re-exports `models::*`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackdropOverlayParams {
  /// Tint blended over the blurred backdrop. Already resolved to a
  /// concrete color, unlike the config's optional one.
  pub tint: Color,
  /// Blur radius/intensity.
  pub blur_amount: f32,
  /// Corner radius, in pixels.
  pub corner_radius: f32,
  /// Opacity of the overlay's whole composited visual (blur + tint
  /// together), from `0.0` to `1.0`.
  pub opacity: f32,
  /// Saturation of the blurred backdrop, from `0.0` (grayscale) to `2.0`
  /// (oversaturated); `1.0` is unchanged.
  pub saturation: f32,
  /// Exposure adjustment in stops; `0.0` is unchanged, negative darkens.
  pub exposure: f32,
  /// Contrast adjustment from `-1.0` to `1.0`; `0.0` is unchanged.
  pub contrast: f32,
  /// Highlight recovery from `-1.0` to `1.0`; `0.0` is unchanged,
  /// negative pulls bright areas down.
  pub highlights: f32,
  /// Shadow lift from `-1.0` to `1.0`; `0.0` is unchanged, positive opens
  /// dark areas up.
  pub shadows: f32,
  /// Strength of a darkening gradient toward the overlay's own edges,
  /// from `0.0` (off) to `1.0`. Rendered as a radial-gradient visual
  /// rather than baked, since it is measured from the window's own
  /// rect.
  pub vignette: f32,
  /// Opacity of a monochrome noise layer over the blurred image, from
  /// `0.0` (off) to `1.0`.
  pub grain: f32,
  /// How much the backdrop crop follows the window, as a multiplier on
  /// the window's offset within its monitor. Not baked -- it selects a
  /// different part of an already-rendered surface, so it costs one
  /// property write on move and nothing at all otherwise.
  pub parallax: f32,
}

impl BackdropOverlayParams {
  /// Whether a surrogate can draw this backdrop as copies of it (see
  /// `OverlayCopies::carry_backdrop`): pinned to the desktop, as the copy
  /// is cut from a backdrop of the whole monitor, and square unless
  /// `GLAZEWM_ROUNDED_BACKDROP` says how to draw rounded corners (see
  /// `rounded_backdrop`).
  #[must_use]
  pub fn is_copyable(&self) -> bool {
    self.is_copyable_with(crate::rounded_backdrop::current())
  }

  /// [`is_copyable`](Self::is_copyable), drawing rounded corners as
  /// `rounded` says.
  fn is_copyable_with(&self, rounded: RoundedBackdrop) -> bool {
    (self.corner_radius <= 0.0 || rounded.is_copied())
      && (self.parallax - 1.0).abs() < 1e-3
  }
}

#[cfg(test)]
mod tests {
  use super::{BackdropOverlayParams, RoundedBackdrop};
  use crate::Color;

  fn backdrop(corner_radius: f32, parallax: f32) -> BackdropOverlayParams {
    BackdropOverlayParams {
      tint: Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
      },
      blur_amount: 20.0,
      corner_radius,
      opacity: 1.0,
      saturation: 1.0,
      exposure: 0.0,
      contrast: 0.0,
      highlights: 0.0,
      shadows: 0.0,
      vignette: 0.0,
      grain: 0.0,
      parallax,
    }
  }

  #[test]
  fn rounded_corners_are_copied_only_when_switched_on() {
    let rounded = backdrop(8.0, 1.0);

    assert!(!rounded.is_copyable_with(RoundedBackdrop::Live));
    assert!(rounded.is_copyable_with(RoundedBackdrop::Bands));
    assert!(rounded.is_copyable_with(RoundedBackdrop::Own));
    assert!(backdrop(0.0, 1.0).is_copyable_with(RoundedBackdrop::Live));
  }

  #[test]
  fn parallax_is_never_copied() {
    for rounded in [
      RoundedBackdrop::Live,
      RoundedBackdrop::Bands,
      RoundedBackdrop::Own,
    ] {
      assert!(!backdrop(0.0, 0.5).is_copyable_with(rounded));
      assert!(!backdrop(8.0, 0.5).is_copyable_with(rounded));
    }
  }
}
