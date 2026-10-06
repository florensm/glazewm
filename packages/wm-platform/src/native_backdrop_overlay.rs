use windows::Win32::Foundation::HWND;

use crate::{
  lerp_f32,
  overlay_window::{Overlay, OverlayKind, OverlayWindow},
  platform_impl::composition::BackdropVisual,
  BackdropOverlayParams, Rect, StyleBlend, SurrogateBatch,
};

/// A persistent backdrop window rendering a crop of the pre-blurred
/// wallpaper surface behind a paired managed window.
///
/// Positioned directly behind an `anchor` window in z-order (typically the
/// managed window itself, or its surrogate while one is active -- see
/// [`set_rect`]/[`sync_z_order`]) and kept pixel-aligned with its DWM
/// frame rect. When the managed window is semi-transparent (via the
/// `transparency` window effect), the backdrop shows through the window,
/// producing a frosted-glass look.
///
/// Renders entirely through a `Windows.UI.Composition` visual tree, so a
/// system without it (pre-Windows 10 1803) gets no overlay at all rather
/// than a degraded one.
///
/// [`set_rect`]: NativeBackdropOverlay::set_rect
/// [`sync_z_order`]: NativeBackdropOverlay::sync_z_order
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeBackdropOverlay {
  /// The overlay's visual tree. Declared before `window`: fields drop in
  /// declaration order, and the tree must go before the `HWND` it is
  /// rooted to.
  composition: BackdropVisual,

  /// Anchored behind its own managed window rather than `HWND_BOTTOM`:
  /// the backdrop is opaque, so at the bottom it would sit behind every
  /// other window instead of showing through the one it belongs to.
  window: OverlayWindow,

  /// Last-applied knobs. Only the live ones (tint, corner radius,
  /// opacity, vignette, parallax) are read back, to skip unchanged
  /// writes; the visual tree tracks what its layers were baked with
  /// itself.
  params: BackdropOverlayParams,

  /// Last rect applied, used to skip redundant `SetWindowPos` calls when
  /// the overlay hasn't actually moved.
  rect: Rect,
}

impl NativeBackdropOverlay {
  /// Repositions and resizes the overlay to match `rect` behind `anchor`,
  /// and ensures it's shown.
  ///
  /// No-op if neither `rect` nor `anchor` changed and the overlay is
  /// already visible, sparing a `SetWindowPos` (and the DWM recomposite it
  /// triggers) on every sync tick. Callers that only need to correct
  /// z-order drift should use [`sync_z_order`] instead.
  ///
  /// [`sync_z_order`]: NativeBackdropOverlay::sync_z_order
  pub fn set_rect(&mut self, rect: &Rect, anchor: HWND) {
    if self.window.is_placed_behind(anchor) && &self.rect == rect {
      return;
    }

    if let Err(err) = self.window.place(rect, anchor) {
      tracing::warn!("{err}");
      return;
    }

    if let Err(e) = self.composition.set_rect(rect) {
      tracing::warn!("Backdrop overlay composition resize failed: {e}.");
    }

    self.rect = rect.clone();
  }

  /// Applies the knobs the visual tree renders live, each only when it
  /// changed. The baked knobs are left to
  /// `BackdropVisual::set_bake_blend`.
  #[allow(clippy::float_cmp)]
  fn apply_live_knobs(&mut self, params: BackdropOverlayParams) {
    let current = self.params;
    let composition = &mut self.composition;

    let result = (|| -> crate::Result<()> {
      if current.tint != params.tint {
        composition.set_tint(params.tint)?;
      }
      if current.corner_radius != params.corner_radius {
        composition.set_corner_radius(params.corner_radius)?;
      }
      if current.opacity != params.opacity {
        composition.set_opacity(params.opacity)?;
      }
      if current.vignette != params.vignette {
        composition.set_vignette(params.vignette)?;
      }
      Ok(())
    })();

    if let Err(e) = result {
      tracing::warn!("Backdrop overlay composition update failed: {e}.");
    }

    if current.parallax != params.parallax {
      composition.set_parallax(params.parallax, &self.rect);
    }

    self.params = params;
  }
}

impl Overlay for NativeBackdropOverlay {
  type Params = BackdropOverlayParams;

  /// There is no non-composition path: without `Windows.UI.Composition`
  /// (pre-Windows 10 1803) there is no overlay rather than a partial one.
  /// The window is deliberately not a host backdrop: the wallpaper crop is
  /// opaque, and keeping what sits beneath it composited is the exact cost
  /// the backdrop exists to remove.
  fn create(
    rect: &Rect,
    params: BackdropOverlayParams,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let mut window =
      OverlayWindow::create(OverlayKind::Backdrop, rect, anchor)?;
    let composition = BackdropVisual::create(window.hwnd(), rect, params)?;

    if let Err(err) = window.place(rect, anchor) {
      tracing::warn!("{err}");
    }

    Ok(Self {
      composition,
      window,
      params,
      rect: rect.clone(),
    })
  }

  /// Also the per-tick point at which the overlay notices the desktop
  /// wallpaper changing underneath it.
  fn apply_blend(&mut self, blend: StyleBlend<BackdropOverlayParams>) {
    let StyleBlend { from, to, t } = blend;

    // The baked knobs travel unblended: they select a surface rather than
    // describe one, so in-between values would each be a full-monitor
    // bake. `set_bake_blend` crossfades the two surfaces instead.
    self.apply_live_knobs(BackdropOverlayParams {
      tint: from.tint.lerp(&to.tint, t),
      corner_radius: lerp_f32(from.corner_radius, to.corner_radius, t),
      opacity: lerp_f32(from.opacity, to.opacity, t),
      vignette: lerp_f32(from.vignette, to.vignette, t),
      parallax: lerp_f32(from.parallax, to.parallax, t),
      ..to
    });

    if let Err(e) =
      self.composition.set_bake_blend(from.into(), to.into(), t)
    {
      tracing::warn!("Backdrop overlay bake-knob update failed: {e}.");
    }

    // Unlike the knobs above, this reacts to a change *outside* the
    // config -- the user swapping their wallpaper, or the displays being
    // rearranged. `apply_blend` is the one call every tracked overlay gets
    // on every tick, which is what makes it the place to notice.
    if let Err(e) = self.composition.sync_backdrop(&self.rect) {
      tracing::warn!("Wallpaper backdrop refresh failed: {e}.");
    }
  }

  fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
    anchor: HWND,
  ) {
    if !self.window.is_placed_behind(anchor) {
      self.set_rect(rect, anchor);
      return;
    }

    if &self.rect == rect {
      return;
    }

    batch.push(self.window.hwnd().0, rect.clone());

    if let Err(e) = self.composition.set_rect(rect) {
      tracing::warn!("Backdrop overlay composition resize failed: {e}.");
    }

    self.rect = rect.clone();
  }

  fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()> {
    self.window.sync_z_order(anchor, force)
  }

  fn is_visible(&self) -> bool {
    self.window.is_visible()
  }

  fn hide(&mut self) {
    self.window.hide();
  }
}
