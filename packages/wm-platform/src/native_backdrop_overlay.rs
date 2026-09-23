use windows::Win32::Foundation::HWND;

use crate::{
  overlay_window::{OverlayKind, OverlayWindow},
  platform_impl::composition::BackdropVisual,
  BackdropOverlayParams, Color, Rect, SurrogateBatch,
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

  /// Current tint/blur-amount/corner-radius/opacity/saturation, applied
  /// as the composition tree's live properties.
  params: BackdropOverlayParams,

  /// Last rect applied, used to skip redundant `SetWindowPos` calls when
  /// the overlay hasn't actually moved.
  rect: Rect,
}

/// Generates a `NativeBackdropOverlay` setter for a single `f32` knob
/// shared with the `BackdropVisual` composition pipeline: no-ops when
/// `value` matches the last-applied `params.$field`, otherwise stores it
/// and forwards to the matching `BackdropVisual` setter.
macro_rules! backdrop_overlay_setter {
  (
    $(#[$doc:meta])*
    $setter:ident, $field:ident
  ) => {
    $(#[$doc])*
    #[allow(clippy::float_cmp)]
    pub fn $setter(&mut self, value: f32) {
      if self.params.$field == value {
        return;
      }
      self.params.$field = value;

      if let Err(e) = self.composition.$setter(value) {
        tracing::warn!(
          concat!(
            "Backdrop overlay ",
            stringify!($field),
            " update failed: {e}."
          ),
          e = e
        );
      }
    }
  };
}

impl NativeBackdropOverlay {
  /// Creates a new backdrop overlay sized and positioned to `rect`, shown
  /// directly behind `anchor` -- typically the managed window it tracks,
  /// or its surrogate while one is active.
  ///
  /// There is no non-composition path: without `Windows.UI.Composition`
  /// (pre-Windows 10 1803) there is no overlay rather than a partial one.
  /// The window is deliberately not a host backdrop: the wallpaper crop is
  /// opaque, and keeping what sits beneath it composited is the exact cost
  /// the backdrop exists to remove.
  pub fn create(
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

  /// Returns whether the overlay window is currently shown.
  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.window.is_visible()
  }

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

  /// Queues a reposition into `batch` instead of issuing an immediate
  /// `SetWindowPos`, for the common per-tick case where the overlay is
  /// already visible, `anchor` hasn't changed, and only its position/size
  /// changed.
  ///
  /// All overlays/surrogates queued into the same [`SurrogateBatch`] are
  /// repositioned atomically when the batch is committed, so this overlay
  /// moves in the same DWM composition frame as the window it's paired
  /// with, instead of each issuing its own synchronous `SetWindowPos` --
  /// cost that scales with tick rate, most visible on high-refresh-rate
  /// displays where the animation manager ticks in lockstep with vsync.
  ///
  /// Falls back to [`set_rect`] when the overlay is hidden or `anchor`
  /// changed: re-showing needs `SWP_SHOWWINDOW` and an anchor change needs
  /// a z-order move, and the batch applies neither (its flags are
  /// `SWP_NOZORDER` with no show bit, shared with surrogates).
  ///
  /// [`set_rect`]: NativeBackdropOverlay::set_rect
  pub fn defer_rect(
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

  /// Corrects z-order drift by putting the overlay back directly behind
  /// `anchor`, without touching its rect. See
  /// [`OverlayWindow::sync_z_order`] for `force`.
  pub fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()> {
    self.window.sync_z_order(anchor, force)
  }

  /// Updates the tint; re-applies only when the value changes.
  pub fn set_tint(&mut self, tint: Color) {
    if self.params.tint == tint {
      return;
    }
    self.params.tint = tint;

    if let Err(e) = self.composition.set_tint(tint) {
      tracing::warn!(
        "Backdrop overlay composition tint update failed: {e}."
      );
    }
  }

  /// Applies all seven baked knobs together, re-rendering at most once.
  ///
  /// Kept separate from the per-knob setters so a caller with a whole new
  /// `BackdropOverlayParams` -- which is every caller in practice, since
  /// params are resolved per focus state -- pays one bake rather than one
  /// per changed knob.
  fn set_bake_knobs(&mut self, params: BackdropOverlayParams) {
    self.params.blur_amount = params.blur_amount;
    self.params.saturation = params.saturation;
    self.params.exposure = params.exposure;
    self.params.contrast = params.contrast;
    self.params.highlights = params.highlights;
    self.params.shadows = params.shadows;
    self.params.grain = params.grain;

    if let Err(e) = self.composition.set_bake_knobs(params) {
      tracing::warn!("Backdrop overlay bake-knob update failed: {e}.");
    }
  }

  backdrop_overlay_setter!(
    /// Updates the blur radius/intensity; re-applies only when the value
    /// changes.
    ///
    /// Compares the raw `f32` for exact equality, same as `set_tint`'s
    /// ABGR comparison -- the value only ever changes when a caller
    /// passes a genuinely different, config-resolved number, not
    /// through any arithmetic that could introduce drift.
    set_blur_amount,
    blur_amount
  );

  backdrop_overlay_setter!(
    /// Updates the corner radius, in pixels; re-applies only when the
    /// value changes.
    ///
    /// See `set_blur_amount` for why exact `f32` equality is intentional
    /// here.
    set_corner_radius,
    corner_radius
  );

  backdrop_overlay_setter!(
    /// Updates the overlay's own opacity (blur + tint together, as one
    /// unit); re-applies only when the value changes.
    ///
    /// See `set_blur_amount` for why exact `f32` equality is intentional
    /// here.
    set_opacity,
    opacity
  );

  backdrop_overlay_setter!(
    /// Updates the saturation of the blurred backdrop; re-applies only
    /// when the value changes.
    ///
    /// See `set_blur_amount` for why exact `f32` equality is intentional
    /// here.
    set_saturation,
    saturation
  );

  backdrop_overlay_setter!(
    /// Updates the exposure baked into the wallpaper backdrop, in stops.
    set_exposure,
    exposure
  );

  backdrop_overlay_setter!(
    /// Updates the contrast baked into the wallpaper backdrop.
    set_contrast,
    contrast
  );

  backdrop_overlay_setter!(
    /// Updates the highlight recovery baked into the wallpaper backdrop:
    /// negative pulls bright areas down, leaving the rest alone.
    set_highlights,
    highlights
  );

  backdrop_overlay_setter!(
    /// Updates the shadow lift baked into the wallpaper backdrop.
    set_shadows,
    shadows
  );

  backdrop_overlay_setter!(
    /// Updates the vignette baked into the wallpaper backdrop.
    set_vignette,
    vignette
  );

  backdrop_overlay_setter!(
    /// Updates the grain baked into the wallpaper backdrop.
    set_grain,
    grain
  );

  /// Updates how far the wallpaper backdrop's crop follows the window.
  ///
  /// Not generated by [`backdrop_overlay_setter`] because it is the one
  /// knob that needs the overlay's current rect to re-apply: it re-aims an
  /// existing surface rather than re-rendering one, so there is nothing to
  /// rebuild, only a new offset to compute.
  #[allow(clippy::float_cmp)]
  pub fn set_parallax(&mut self, value: f32) {
    if self.params.parallax == value {
      return;
    }
    self.params.parallax = value;

    self.composition.set_parallax(value, &self.rect);
  }

  /// Applies `params`, re-applying only whichever fields actually changed
  /// (each setter no-ops internally on an unchanged value). Convenience
  /// for the call sites that already have a full `BackdropOverlayParams`
  /// rather than one field at a time.
  ///
  /// Also the per-tick point at which the overlay notices the desktop
  /// wallpaper changing underneath it.
  pub fn apply(&mut self, params: BackdropOverlayParams) {
    self.set_tint(params.tint);
    self.set_corner_radius(params.corner_radius);
    self.set_opacity(params.opacity);
    self.set_vignette(params.vignette);

    // The seven baked knobs go in one call rather than one setter each.
    // Applied singly they walk through six intermediate combinations, each
    // of which is a separate full-monitor bake -- see
    // `BackdropVisual::set_bake_knobs`.
    self.set_bake_knobs(params);
    self.set_parallax(params.parallax);

    // Unlike the setters above, this reacts to a change *outside* the
    // config -- the user swapping their wallpaper, or the displays being
    // rearranged. `apply` is the one call every tracked overlay gets on
    // every tick, which is what makes it the place to notice.
    if let Err(e) = self.composition.sync_backdrop(&self.rect) {
      tracing::warn!("Wallpaper backdrop refresh failed: {e}.");
    }
  }

  /// Hides the overlay without destroying it.
  pub fn hide(&mut self) {
    self.window.hide();
  }
}
