use serde::{Deserialize, Serialize};

/// Backdrop material rendered behind a window.
///
/// Every style is drawn by a persistent `NativeBlurOverlay` sitting directly
/// behind the managed window, through a `Windows.UI.Composition` visual
/// tree. Nothing here is applied to the managed window itself: SWCA on a
/// window the `transparency` effect has made `WS_EX_LAYERED` conflicts, and
/// DWM's own materials only paint where an application leaves its surface
/// unpainted, which nearly none do.
///
/// `Wallpaper` blurs the desktop wallpaper once, into an opaque per-monitor
/// image, and shows each window the crop of it underneath that window. In a
/// tiling layout nothing is behind a tiled window except the wallpaper, so
/// this reproduces what acrylic samples while sampling nothing per frame --
/// and being opaque, it lets DWM skip compositing what is behind the overlay
/// rather than blending it every frame.
///
/// `Acrylic` samples live desktop content through a host-backdrop brush and
/// blurs it through a Gaussian effect graph, every frame. That is only worth
/// paying for when something other than the wallpaper is behind the window,
/// which in a tiling layout means floating windows -- the one case
/// `Wallpaper` structurally cannot serve, since a wallpaper crop has no idea
/// another window is stacked there.
///
/// `Solid` fills the overlay with a flat color. At a fully opaque `tint` it
/// is the cheapest style by a wide margin: one opaque visual, no sampling,
/// no blur, and DWM skips everything behind it.
///
/// # Platform-specific
///
/// Only has an effect on Windows, and only where `Windows.UI.Composition` is
/// available (10 1803+). On anything older no overlay is created and the
/// effect is silently skipped.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackdropStyle {
  /// The desktop wallpaper, blurred once per monitor into an opaque
  /// surface, of which each window shows the crop beneath it. Honors every
  /// knob; `blur_amount`/`saturation`/`exposure`/`contrast`/`grain` are
  /// baked into that image, so changing one re-renders it rather than
  /// costing anything per frame.
  #[default]
  Wallpaper,

  /// Frosted-glass acrylic that blurs live content behind the window, every
  /// frame. The expensive style, and the only one that can blur other
  /// windows rather than just the wallpaper.
  Acrylic,

  /// Flat `tint` fill. Ignores every other knob.
  Solid,
}
