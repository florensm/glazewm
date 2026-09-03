/// Color, width, corner radius, and opacity for a `NativeBorderOverlay`,
/// bundled so the set of overlay knobs travels as one value through
/// `SessionOptions`/`ResizeSession`/`upsert_border_overlay` instead of a
/// same-typed positional-argument list that's easy to mis-order at the many
/// call sites (static sync, workspace-switch, and move/resize/open/close
/// tracking) that all thread the same values. Mirrors `BlurOverlayParams`.
///
/// Lives in `models` (rather than alongside `NativeBorderOverlay`) so it's
/// visible from both crate roots this crate builds under -- `lib.rs` for
/// normal builds and the separate `test.rs` harness, which only declares a
/// subset of modules but always re-exports `models::*`.
use crate::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderOverlayParams {
  /// Color of the border ring.
  pub color: Color,
  /// Ring thickness, in physical pixels.
  pub width: f32,
  /// Corner radius, in pixels, of the ring's outer edge. No-op in the SWCA
  /// fallback.
  pub corner_radius: f32,
  /// Opacity of the overlay's whole composited visual, from `0.0` to
  /// `1.0`.
  pub opacity: f32,
}
