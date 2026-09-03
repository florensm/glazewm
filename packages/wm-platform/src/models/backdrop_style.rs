use serde::{Deserialize, Serialize};

/// Backdrop material applied behind a window via DWM or SWCA.
///
/// `Acrylic` and `Blur` are both rendered by a persistent
/// `NativeBlurOverlay` placed behind the managed window, rather than being
/// applied to the managed window itself -- that avoids the
/// `WS_EX_LAYERED`/SWCA conflict that arises when applying SWCA directly to
/// a layered window (which the `transparency` effect makes it).
///
/// `Acrylic` renders through a `Windows.UI.Composition` effect graph (live
/// host-backdrop brush -> Gaussian blur -> saturation -> tint), falling back
/// to SWCA's `ACCENT_ENABLE_ACRYLICBLURBEHIND` when that pipeline is
/// unavailable. It's the richest material and by far the most expensive:
/// DWM has to composite a blurred, noise-textured, translucent surface every
/// frame, which shows up directly as `DwmFlush` wait time in the main loop.
///
/// `Blur` goes straight to SWCA's `ACCENT_ENABLE_BLURBEHIND` and builds no
/// composition graph at all -- the cheap, Win10-era Aero blur. It gives up
/// `blur_amount`/`corner_radius`/`opacity`/`saturation` (the OS exposes no
/// knobs for it) and keeps only `tint`, in exchange for skipping the
/// per-frame D2D effect graph entirely.
///
/// `Mica` and `MicaAlt` use `DWMWA_SYSTEMBACKDROP_TYPE` on the managed
/// window (Windows 11 22H2+) and create no overlay. They only ever affect
/// the parts of the window DWM itself draws -- the non-client frame and any
/// area the app leaves unpainted -- so a third-party app that paints its
/// client area opaquely (nearly all of them do) shows no visible change.
///
/// # Platform-specific
///
/// Only has an effect on Windows: 10 1607+ (`Blur`), 10 1803+ (`Acrylic`),
/// or 11 22H2+ (`Mica`/`MicaAlt`). On unsupported platforms/versions the
/// effect is silently skipped.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackdropStyle {
  /// Frosted-glass acrylic that blurs content behind the window.
  #[default]
  Acrylic,

  /// Plain blur-behind: blurs content behind the window with none of
  /// acrylic's noise/tint/saturation work, and no composition pipeline.
  /// Markedly cheaper than [`BackdropStyle::Acrylic`], at the cost of the
  /// `blur_amount`/`opacity`/`saturation` knobs.
  Blur,

  /// Translucent solid fill: `tint` blended over the content behind the
  /// window, with no blur pass. The cheapest overlay-backed style -- DWM
  /// composites one flat layer rather than sampling and blurring what is
  /// beneath. Ignores every knob except `tint`.
  Solid,

  /// Windows 11's own acrylic, applied by DWM to the managed window via
  /// `DWMWA_SYSTEMBACKDROP_TYPE` rather than rendered into an overlay.
  ///
  /// Far cheaper than [`BackdropStyle::Acrylic`] because DWM owns the whole
  /// effect, but it shares the Mica variants' limitation: DWM only paints it
  /// where the application leaves its own surface unpainted, which most
  /// third-party apps do not.
  Transient,

  /// Mica material that samples the desktop wallpaper.
  Mica,

  /// Tabbed Mica variant with a slightly stronger wallpaper tint.
  MicaAlt,
}

impl BackdropStyle {
  /// Whether this style is rendered by a `NativeBlurOverlay` window placed
  /// behind the managed window, as opposed to a `DwmSetWindowAttribute`
  /// call on the managed window itself.
  #[must_use]
  pub fn is_overlay_backed(self) -> bool {
    matches!(self, Self::Acrylic | Self::Blur | Self::Solid)
  }
}

#[cfg(test)]
mod tests {
  use super::BackdropStyle;

  /// Only the SWCA/composition-rendered styles get an overlay window; the
  /// DWM-applied ones go onto the managed window itself.
  ///
  /// This split is what decides whether a style renders at all on a
  /// third-party window: an overlay is our own surface and always paints,
  /// while DWM only paints its own materials where the application leaves
  /// its surface unpainted.
  #[test]
  fn overlay_backed_styles() {
    assert!(BackdropStyle::Acrylic.is_overlay_backed());
    assert!(BackdropStyle::Blur.is_overlay_backed());
    assert!(BackdropStyle::Solid.is_overlay_backed());

    assert!(!BackdropStyle::Transient.is_overlay_backed());
    assert!(!BackdropStyle::Mica.is_overlay_backed());
    assert!(!BackdropStyle::MicaAlt.is_overlay_backed());
  }
}
