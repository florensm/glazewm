use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, GetWindow, SetWindowPos, ShowWindow,
      GW_HWNDPREV, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING,
      SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
      WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::composition::BackdropVisual, window_class,
  BackdropOverlayParams, Color, Rect, SurrogateBatch,
};

fn ensure_class_registered() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_BackdropOverlay"),
    window_class::default_wnd_proc,
  );
}

/// Creates the overlay's backdrop window.
///
/// `WS_EX_NOREDIRECTIONBITMAP` skips the GDI redirection surface DWM
/// would otherwise allocate, which the composition visual tree replaces
/// entirely.
fn create_window(rect: &Rect) -> crate::Result<HWND> {
  ensure_class_registered();

  // `WS_EX_TRANSPARENT` makes the overlay invisible to hit-testing. It is
  // mandatory, not cosmetic: overlay windows are created on the WM's own
  // thread, which runs a Tokio loop and never pumps a Win32 message queue,
  // so Windows classifies every window it owns as hung. Without the flag,
  // the cursor landing on this overlay shows the busy ("working in
  // background") cursor and clicks are swallowed -- most visibly during a
  // resize animation, where the real window is cloaked and the surrogate
  // above this overlay is itself `WS_EX_TRANSPARENT`, so hit-tests fall
  // straight through onto it.
  let ex_style = WS_EX_NOACTIVATE
    | WS_EX_TOOLWINDOW
    | WS_EX_TRANSPARENT
    | WS_EX_NOREDIRECTIONBITMAP;

  // SAFETY: All parameters are valid. The class is guaranteed registered
  // by `ensure_class_registered`. No parent HWND is needed.
  let hwnd = unsafe {
    CreateWindowExW(
      ex_style,
      w!("GlazeWM_BackdropOverlay"),
      w!(""),
      WS_POPUP,
      rect.x(),
      rect.y(),
      rect.width(),
      rect.height(),
      None,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create backdrop overlay window.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// Creates the overlay's backing window and roots its
/// `Windows.UI.Composition` visual tree on it.
///
/// There is no non-composition path: the backdrop renders through the
/// visual tree, so a system without `Windows.UI.Composition` (pre-Windows
/// 10 1803) gets no overlay rather than a partial one. The alternative --
/// SWCA -- could not express an opaque wallpaper crop at all.
///
/// The window is deliberately not marked as a host backdrop: the wallpaper
/// crop is opaque, and asking DWM to keep compositing what sits beneath it
/// is the exact cost the backdrop exists to remove.
fn create_backing_window(
  rect: &Rect,
  params: BackdropOverlayParams,
) -> crate::Result<(HWND, BackdropVisual)> {
  let hwnd = create_window(rect)?;

  match BackdropVisual::create(hwnd, rect, params) {
    Ok(visual) => Ok((hwnd, visual)),
    Err(err) => {
      // SAFETY: `hwnd` was just created above and not yet handed to a
      // caller; safe to destroy immediately on this failure path.
      unsafe {
        let _ = DestroyWindow(hwnd);
      }
      Err(err)
    }
  }
}

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
  /// Raw window handle stored as `isize` so that `NativeBackdropOverlay`
  /// is `Send` even though `HWND` is not.
  hwnd: isize,

  /// Current tint/blur-amount/corner-radius/opacity/saturation, applied
  /// as the composition tree's live properties.
  params: BackdropOverlayParams,

  /// Last rect applied via `set_rect`, used to skip redundant
  /// `SetWindowPos` calls when the overlay hasn't actually moved.
  rect: Rect,

  /// `HWND` of the window this overlay is positioned directly behind (its
  /// z-order anchor), as raw `isize`.
  ///
  /// Anchored to its own managed window rather than the global
  /// `HWND_BOTTOM`: the backdrop is opaque, so pinned to the bottom of
  /// the z-order it would be hidden behind every other window instead
  /// of showing through the one it belongs to, and it has to be
  /// occluded by whatever legitimately sits above that window.
  ///
  /// Tracked so [`set_rect`]/[`sync_z_order`] can skip a redundant
  /// `SetWindowPos` when the anchor hasn't changed.
  ///
  /// [`set_rect`]: NativeBackdropOverlay::set_rect
  /// [`sync_z_order`]: NativeBackdropOverlay::sync_z_order
  anchor: isize,

  /// Whether the overlay window is currently shown.
  ///
  /// Tracked explicitly (rather than inferred from a change in `rect`) so
  /// that a caller re-showing the overlay after [`hide`] with an
  /// unchanged rect still issues the `SetWindowPos` needed to reapply
  /// `SWP_SHOWWINDOW` -- the rect-unchanged fast path in [`set_rect`]
  /// would otherwise skip that call entirely, leaving the overlay
  /// hidden.
  ///
  /// [`hide`]: NativeBackdropOverlay::hide
  /// [`set_rect`]: NativeBackdropOverlay::set_rect
  is_visible: bool,

  /// The overlay's composition visual tree.
  ///
  /// Optional only so that it can be dropped *before* the `HWND` it is
  /// rooted to, in `Drop`; an overlay that failed to build one is never
  /// constructed in the first place. Treat it as always present.
  composition: Option<BackdropVisual>,
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

      if let Some(composition) = &mut self.composition {
        if let Err(e) = composition.$setter(value) {
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
    }
  };
}

impl NativeBackdropOverlay {
  /// Creates a new backdrop overlay sized and positioned to `rect`, with
  /// the given `params` (blur amount, corner radius, opacity, and
  /// saturation are only honored when the Composition pipeline is
  /// available).
  ///
  /// The overlay is shown immediately, positioned directly behind `anchor`
  /// (see the `anchor` field doc) -- typically the `HWND` of the managed
  /// window it's tracking, or its surrogate's `HWND` while one is active.
  pub fn create(
    rect: &Rect,
    params: BackdropOverlayParams,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let (hwnd, composition) = create_backing_window(rect, params)?;

    // SAFETY: `hwnd` is a valid window just created above.
    if let Err(e) = unsafe {
      SetWindowPos(
        hwnd,
        anchor,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    } {
      tracing::warn!(
        "Backdrop overlay SetWindowPos failed on create: {e}."
      );
    }

    Ok(Self {
      hwnd: hwnd.0,
      params,
      rect: rect.clone(),
      anchor: anchor.0,
      is_visible: true,
      composition: Some(composition),
    })
  }

  /// Returns the `HWND` for this overlay.
  fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Returns whether the overlay window is currently shown.
  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.is_visible
  }

  /// Repositions and resizes the overlay to match `rect`, keeping it
  /// directly behind `anchor` (see the `anchor` field doc), and ensures
  /// it's shown.
  ///
  /// No-op if neither `rect` nor `anchor` changed and the overlay is
  /// already visible, to avoid redundant `SetWindowPos` calls (and the DWM
  /// recomposite they trigger) on every sync tick for overlays that
  /// haven't actually moved. Always issues the call when re-showing
  /// after [`hide`], even at an unchanged rect/anchor, since that's what
  /// reapplies `SWP_SHOWWINDOW`.
  ///
  /// Callers that only need to correct z-order drift (`anchor` may have
  /// changed but `rect` hasn't, e.g. after an unrelated window steals
  /// focus) without a full position sync should use [`sync_z_order`]
  /// instead -- it skips the position arguments entirely and stays cheap
  /// enough to call unconditionally every tick.
  ///
  /// [`hide`]: NativeBackdropOverlay::hide
  /// [`sync_z_order`]: NativeBackdropOverlay::sync_z_order
  pub fn set_rect(&mut self, rect: &Rect, anchor: HWND) {
    if self.is_visible && &self.rect == rect && self.anchor == anchor.0 {
      return;
    }

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    if let Err(e) = unsafe {
      SetWindowPos(
        self.hwnd(),
        anchor,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    } {
      tracing::warn!("Backdrop overlay SetWindowPos failed: {e}.");
      return;
    }

    if let Some(composition) = &mut self.composition {
      if let Err(e) = composition.set_rect(rect) {
        tracing::warn!("Backdrop overlay composition resize failed: {e}.");
      }
    }

    self.rect = rect.clone();
    self.anchor = anchor.0;
    self.is_visible = true;
  }

  /// Queues a reposition into `batch` instead of issuing an immediate
  /// `SetWindowPos`, for the common per-tick case where the overlay is
  /// already visible, `anchor` hasn't changed, and only its position/size
  /// changed.
  ///
  /// All overlays/surrogates queued into the same [`SurrogateBatch`] are
  /// repositioned atomically when the batch is committed, so this overlay
  /// moves in the same DWM composition frame as the window it's paired
  /// with (and any other windows/surrogates relaid out the same tick),
  /// instead of each issuing its own synchronous `SetWindowPos` -- cost
  /// that scales with tick rate, most visible on high-refresh-rate
  /// displays where the animation manager ticks in lockstep with vsync.
  ///
  /// Falls back to [`set_rect`] (immediate, unbatched) when the overlay
  /// isn't currently visible, or when `anchor` changed: re-showing needs
  /// `SWP_SHOWWINDOW`, and an anchor change needs an actual z-order-moving
  /// `SetWindowPos` -- neither of which `SurrogateBatch::commit` applies
  /// (its flags are `SWP_NOZORDER` with no show/hide bit, shared with
  /// surrogates, which need neither). Both fallback cases are rare
  /// relative to the steady-state reposition case: a session's anchor is
  /// set once and typically stays fixed for the animation's duration.
  ///
  /// [`set_rect`]: NativeBackdropOverlay::set_rect
  pub fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
    anchor: HWND,
  ) {
    if !self.is_visible || self.anchor != anchor.0 {
      self.set_rect(rect, anchor);
      return;
    }

    if &self.rect == rect {
      return;
    }

    batch.push(self.hwnd, rect.clone());

    if let Some(composition) = &mut self.composition {
      if let Err(e) = composition.set_rect(rect) {
        tracing::warn!("Backdrop overlay composition resize failed: {e}.");
      }
    }

    self.rect = rect.clone();
  }

  /// Corrects z-order drift by re-positioning the overlay directly behind
  /// `anchor` if it isn't already there, without touching its rect.
  ///
  /// `anchor` (typically the tracked window's own `HWND`) can drift out of
  /// sync with the overlay even when the overlay's rect hasn't changed --
  /// e.g. an unrelated window being brought to the foreground doesn't move
  /// `anchor` itself, but `anchor` being independently re-raised elsewhere
  /// (see `platform_sync`'s own z-order handling) does, and the overlay
  /// isn't part of that call so it's left behind. Cheap to call
  /// unconditionally every sync tick: `GetWindow`/`GW_HWNDPREV` is a
  /// same-process, no-op-fast check, so this only issues a real
  /// `SetWindowPos` when the overlay actually needs to move.
  ///
  /// `force` skips that check and re-asserts the placement regardless.
  /// Callers pass it when they know `anchor`'s own z-order was changed
  /// earlier in the same tick: `NativeWindow::set_z_order` issues that
  /// change with `SWP_ASYNCWINDOWPOS`, so it may not have landed yet and
  /// `GW_HWNDPREV` can still report the pre-move ordering. The check would
  /// then read "already correct", skip, and leave the overlay stranded in
  /// front of its window once the move does land -- visible as an opaque
  /// backdrop covering the window's contents until something reorders it
  /// again.
  pub fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()> {
    // `anchor` is always a real window handle, so the comparison below is
    // meaningful -- but the overlay has to be in the anchor's band first,
    // or the OS will refuse to leave it directly behind a topmost window.
    window_class::match_z_band(self.hwnd(), anchor);

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    let prev = unsafe { GetWindow(self.hwnd(), GW_HWNDPREV) };
    if !force && prev == anchor {
      self.anchor = anchor.0;
      return Ok(());
    }

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        anchor,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOMOVE | SWP_NOSIZE,
      )
    }?;

    self.anchor = anchor.0;
    Ok(())
  }

  /// Updates the tint; re-applies only when the value changes.
  pub fn set_tint(&mut self, tint: Color) {
    if self.params.tint == tint {
      return;
    }
    self.params.tint = tint;

    if let Some(composition) = &self.composition {
      if let Err(e) = composition.set_tint(tint) {
        tracing::warn!(
          "Backdrop overlay composition tint update failed: {e}."
        );
      }
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

    if let Some(composition) = &mut self.composition {
      if let Err(e) = composition.set_bake_knobs(params) {
        tracing::warn!("Backdrop overlay bake-knob update failed: {e}.");
      }
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

    if let Some(composition) = &mut self.composition {
      composition.set_parallax(value, &self.rect);
    }
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
    if let Some(composition) = &mut self.composition {
      if let Err(e) = composition.sync_backdrop(&self.rect) {
        tracing::warn!("Wallpaper backdrop refresh failed: {e}.");
      }
    }
  }

  /// Hides the overlay without destroying it.
  pub fn hide(&mut self) {
    self.is_visible = false;
    // SAFETY: `self.hwnd()` is a valid window handle.
    unsafe {
      let _ = ShowWindow(self.hwnd(), SW_HIDE);
    }
  }
}

impl Drop for NativeBackdropOverlay {
  fn drop(&mut self) {
    // Drop the Composition visual tree (if any) before destroying the
    // window it's rooted to.
    self.composition.take();

    // SAFETY: `self.hwnd()` is a valid window handle and `Drop` is called
    // at most once.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}
