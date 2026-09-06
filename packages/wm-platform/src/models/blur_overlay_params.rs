/// Backdrop style, tint, blur radius, corner radius, opacity, and
/// saturation for a `NativeBlurOverlay`, bundled so the growing set of
/// overlay knobs travels as one value through `SessionOptions`/
/// `ResizeSession`/`upsert_blur_overlay` instead of a same-typed
/// positional-argument list that's easy to mis-order at the many call sites
/// (static sync, workspace-switch, and move/resize/open/close tracking) that
/// all thread the same values.
///
/// Lives in `models` (rather than alongside `NativeBlurOverlay`) so it's
/// visible from both crate roots this crate builds under -- `lib.rs` for
/// normal builds and the separate `test.rs` harness, which only declares a
/// subset of modules but always re-exports `models::*`.
use crate::{BackdropStyle, Color};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlurOverlayParams {
  /// Which overlay-backed material to render (see
  /// [`BackdropStyle::is_overlay_backed`]).
  ///
  /// Unlike every other field here, this one cannot be re-applied to a
  /// live overlay: it selects how the overlay's backing window is created
  /// in the first place. `NativeBlurOverlay::apply` recreates the window
  /// when it changes -- see its doc comment.
  pub style: BackdropStyle,
  /// Tint blended over the blurred backdrop.
  pub tint: Color,
  /// Blur radius/intensity. No-op in the SWCA fallback and for
  /// [`BackdropStyle::Blur`].
  pub blur_amount: f32,
  /// Corner radius, in pixels. No-op in the SWCA fallback and for
  /// [`BackdropStyle::Blur`].
  pub corner_radius: f32,
  /// Opacity of the overlay's whole composited visual (blur + tint
  /// together), from `0.0` to `1.0`. No-op in the SWCA fallback and for
  /// [`BackdropStyle::Blur`].
  pub opacity: f32,
  /// Saturation of the blurred backdrop, from `0.0` (grayscale) to `2.0`
  /// (oversaturated); `1.0` is unchanged. No-op in the SWCA fallback and
  /// for [`BackdropStyle::Blur`].
  pub saturation: f32,

  /// Exposure adjustment in stops; `0.0` is unchanged, negative darkens.
  ///
  /// [`BackdropStyle::Wallpaper`] only, and not by preference: acrylic
  /// builds its graph through `Compositor::CreateEffectFactory`, which
  /// accepts only a curated subset of D2D's built-in effects and silently
  /// renders `Exposure` as a pass-through. The wallpaper bake goes through
  /// `ID2D1DeviceContext::CreateEffect`, which has no such restriction --
  /// the same reason `vignette` and `grain` exist here and nowhere else.
  pub exposure: f32,

  /// Contrast adjustment from `-1.0` to `1.0`; `0.0` is unchanged.
  /// [`BackdropStyle::Wallpaper`] only (see [`exposure`]).
  ///
  /// [`exposure`]: BlurOverlayParams::exposure
  pub contrast: f32,

  /// Strength of a darkened border around each monitor's baked image, from
  /// `0.0` (off) to `1.0`. [`BackdropStyle::Wallpaper`] only (see
  /// [`exposure`]).
  ///
  /// Preferable to lowering `opacity` for the same "calm it down" effect:
  /// this is baked in and stays opaque, where `opacity` makes the whole
  /// overlay translucent again and gives back the compositing saving the
  /// style exists for.
  ///
  /// [`exposure`]: BlurOverlayParams::exposure
  pub vignette: f32,

  /// Opacity of a monochrome noise layer over the blurred image, from
  /// `0.0` (off) to `1.0`. [`BackdropStyle::Wallpaper`] only (see
  /// [`exposure`]).
  ///
  /// The fine dither is most of what makes Windows' own acrylic read as
  /// frosted glass rather than an out-of-focus photo, and a blurred image
  /// with none of it looks conspicuously smooth.
  ///
  /// [`exposure`]: BlurOverlayParams::exposure
  pub grain: f32,

  /// How much the backdrop crop follows the window, as a multiplier on the
  /// window's offset within its monitor. `1.0` pins the image to the
  /// desktop, so it looks like the window is a window onto the real
  /// wallpaper; below that it drifts against the window as it moves, which
  /// reads as depth. [`BackdropStyle::Wallpaper`] only.
  ///
  /// Unlike every other knob here, this is not baked -- it only changes
  /// which part of an already-rendered surface is shown, so it costs one
  /// property write on move and nothing at all otherwise.
  pub parallax: f32,
}
