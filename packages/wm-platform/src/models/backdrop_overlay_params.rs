use crate::{Color, Rect};

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

/// Where in the backdrop image of `monitor` the backdrop of a window at
/// `frame` is cut from: the offset of the frame's top-left from the
/// image's, which is the window's offset within its monitor times
/// `parallax`.
///
/// Rounded to whole pixels, so a thumbnail's source rect (see
/// `OverlayCopies::carry_backdrop`) cuts exactly what the overlay shows.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub(crate) fn parallax_crop_origin(
  frame: &Rect,
  monitor: &Rect,
  parallax: f32,
) -> (i32, i32) {
  let scaled = |offset: i32| (offset as f32 * parallax).round() as i32;
  (
    scaled(frame.left - monitor.left),
    scaled(frame.top - monitor.top),
  )
}

#[cfg(test)]
mod tests {
  use super::parallax_crop_origin;
  use crate::Rect;

  const MONITOR: Rect = Rect {
    left: 1920,
    top: 0,
    right: 3840,
    bottom: 1080,
  };

  #[test]
  fn crop_follows_the_window_at_full_parallax() {
    let frame = Rect::from_xy(2020, 50, 800, 600);
    assert_eq!(parallax_crop_origin(&frame, &MONITOR, 1.0), (100, 50));
  }

  #[test]
  fn crop_trails_the_window_below_full_parallax() {
    let frame = Rect::from_xy(2020, 50, 800, 600);
    assert_eq!(parallax_crop_origin(&frame, &MONITOR, 0.5), (50, 25));
    assert_eq!(parallax_crop_origin(&frame, &MONITOR, 0.0), (0, 0));
  }

  #[test]
  fn crop_rounds_to_whole_pixels_symmetrically() {
    let right = Rect::from_xy(1921, 3, 10, 10);
    let left = Rect::from_xy(1919, -3, 10, 10);
    assert_eq!(parallax_crop_origin(&right, &MONITOR, 0.5), (1, 2));
    assert_eq!(parallax_crop_origin(&left, &MONITOR, 0.5), (-1, -2));
  }
}
