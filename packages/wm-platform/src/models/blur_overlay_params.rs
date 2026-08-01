/// Tint, blur radius, corner radius, opacity, and saturation for a
/// `NativeBlurOverlay`, bundled so the growing set of overlay knobs
/// travels as one value through `SessionOptions`/`ResizeSession`/
/// `upsert_blur_overlay` instead of a same-typed positional-argument list
/// that's easy to mis-order at the many call sites (static sync,
/// workspace-switch, and move/resize/open/close tracking) that all thread
/// the same values.
///
/// Lives in `models` (rather than alongside `NativeBlurOverlay`) so it's
/// visible from both crate roots this crate builds under -- `lib.rs` for
/// normal builds and the separate `test.rs` harness, which only declares a
/// subset of modules but always re-exports `models::*`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlurOverlayParams {
  /// ABGR tint blended over the blurred backdrop.
  pub tint: u32,
  /// Blur radius/intensity. No-op in the SWCA fallback.
  pub blur_amount: f32,
  /// Corner radius, in pixels. No-op in the SWCA fallback.
  pub corner_radius: f32,
  /// Opacity of the overlay's whole composited visual (blur + tint
  /// together), from `0.0` to `1.0`. No-op in the SWCA fallback.
  pub opacity: f32,
  /// Saturation of the blurred backdrop, from `0.0` (grayscale) to `2.0`
  /// (oversaturated); `1.0` is unchanged. No-op in the SWCA fallback.
  pub saturation: f32,
}
