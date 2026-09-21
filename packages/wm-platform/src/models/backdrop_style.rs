use serde::{Deserialize, Serialize};

/// Backdrop material rendered behind a window.
///
/// Drawn by a persistent `NativeBlurOverlay` sitting directly behind the
/// managed window, through a `Windows.UI.Composition` visual
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
}
