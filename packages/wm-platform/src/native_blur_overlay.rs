use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::HWND,
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_HOSTBACKDROPBRUSH},
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
  platform_impl::{
    composition::BlurVisual,
    swca::{
      apply_swca_accent,
      ACCENT_ENABLE_ACRYLICBLURBEHIND, ACCENT_ENABLE_BLURBEHIND,
      ACCENT_ENABLE_HOSTBACKDROP, ACCENT_ENABLE_TRANSPARENTGRADIENT,
      ACCENT_FLAG_USE_GRADIENT_COLOR,
    },
  },
  window_class, BackdropStyle, BlurOverlayParams, Color, Rect,
  SurrogateBatch,
};

fn ensure_class_registered() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_BlurOverlay"),
    window_class::default_wnd_proc,
  );
}

/// Creates the overlay's backdrop window.
///
/// `composition` selects `WS_EX_NOREDIRECTIONBITMAP`, which skips the GDI
/// redirection surface DWM would otherwise allocate -- correct for the
/// `Windows.UI.Composition` path, whose visual tree replaces that surface
/// entirely, but incompatible with SWCA, which composites into it. Callers
/// falling back from a failed Composition attempt must create a *new*
/// window with `composition: false` rather than reusing one created with
/// the flag set.
fn create_window(rect: &Rect, composition: bool) -> crate::Result<HWND> {
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
  let ex_style = if composition {
    WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT
      | WS_EX_NOREDIRECTIONBITMAP
  } else {
    WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT
  };

  // SAFETY: All parameters are valid. The class is guaranteed registered
  // by `ensure_class_registered`. No parent HWND is needed.
  let hwnd = unsafe {
    CreateWindowExW(
      ex_style,
      w!("GlazeWM_BlurOverlay"),
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
      "Failed to create blur overlay window.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// Applies `ACCENT_ENABLE_HOSTBACKDROP` (+ its Win11 documented equivalent,
/// `DWMWA_USE_HOSTBACKDROPBRUSH`) to `hwnd`, required for a
/// `CompositionBackdropBrush` to sample live desktop content instead of
/// rendering black/opaque.
fn apply_hostbackdrop(hwnd: HWND) {
  apply_swca_accent(hwnd, ACCENT_ENABLE_HOSTBACKDROP, 0, 0);

  let value: windows::Win32::Foundation::BOOL = true.into();
  // `BOOL` is a 4-byte struct; the cast is always exact.
  #[allow(clippy::cast_possible_truncation)]
  let size = std::mem::size_of::<windows::Win32::Foundation::BOOL>() as u32;
  // SAFETY: `hwnd` is valid; `value` is a 4-byte BOOL matching `size`.
  unsafe {
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_USE_HOSTBACKDROPBRUSH,
      std::ptr::addr_of!(value).cast(),
      size,
    );
  }
}

/// Applies the SWCA accent matching `style` to `hwnd`.
///
/// [`BackdropStyle::Blur`] uses `ACCENT_ENABLE_BLURBEHIND`, whose tint is
/// only honored when `ACCENT_FLAG_USE_GRADIENT_COLOR` is set. Every other
/// style uses `ACCENT_ENABLE_ACRYLICBLURBEHIND` with no flags, which reads
/// the tint unconditionally and renders a flat, unblurred fill if the flag
/// is set.
fn apply_swca_for_style(hwnd: HWND, style: BackdropStyle, tint: Color) {
  match style {
    BackdropStyle::Blur => {
      apply_swca_accent(
        hwnd,
        ACCENT_ENABLE_BLURBEHIND,
        ACCENT_FLAG_USE_GRADIENT_COLOR,
        tint.to_abgr(),
      );
    }
    // The tint is the entire effect here, so the flag is mandatory: without
    // it the fill renders with whatever color DWM last had for the window.
    BackdropStyle::Solid => {
      apply_swca_accent(
        hwnd,
        ACCENT_ENABLE_TRANSPARENTGRADIENT,
        ACCENT_FLAG_USE_GRADIENT_COLOR,
        tint.to_abgr(),
      );
    }
    // The Mica variants never reach here -- they're applied to the managed
    // window via `DWMWA_SYSTEMBACKDROP_TYPE` and get no overlay at all (see
    // `BackdropStyle::is_overlay_backed`) -- so acrylic is both the correct
    // arm for `Acrylic` and a harmless catch-all.
    BackdropStyle::Acrylic
    | BackdropStyle::Transient
    | BackdropStyle::Mica
    | BackdropStyle::MicaAlt => {
      apply_swca_accent(
        hwnd,
        ACCENT_ENABLE_ACRYLICBLURBEHIND,
        0,
        tint.to_abgr(),
      );
    }
  }
}

/// Creates the overlay's backing window plus, for
/// [`BackdropStyle::Acrylic`], its `Windows.UI.Composition` pipeline.
///
/// [`BackdropStyle::Blur`] deliberately skips composition entirely and goes
/// straight to SWCA -- that's the whole point of the style: no
/// host-backdrop brush and no per-frame D2D Gaussian effect graph, just the
/// OS's own fixed blur. Acrylic tries the Composition pipeline first and
/// falls back to SWCA acrylic when any step of it is unavailable.
///
/// Returns the window handle and, on the Composition path only, the
/// `BlurVisual` rooted to it.
fn create_backing_window(
  rect: &Rect,
  params: BlurOverlayParams,
) -> crate::Result<(HWND, Option<BlurVisual>)> {
  // Only acrylic wants the composition pipeline. `BlurVisual` builds one
  // fixed effect graph -- host-backdrop brush, Gaussian, saturation, tint --
  // and does not branch on style, so sending any other style through it
  // renders that style *as acrylic* while charging its full per-frame cost.
  // `Blur` and `Solid` are defined as the cheap styles precisely because
  // they skip it and take an SWCA accent instead.
  if params.style == BackdropStyle::Acrylic {
    if let Some((hwnd, visual)) = try_create_composition(rect, params) {
      return Ok((hwnd, Some(visual)));
    }
  }

  let hwnd = create_window(rect, false)?;
  apply_swca_for_style(hwnd, params.style, params.tint);
  Ok((hwnd, None))
}

/// Attempts to build the `Windows.UI.Composition` pipeline for a freshly
/// created overlay window. On any failure, destroys `hwnd` (since it was
/// created with `WS_EX_NOREDIRECTIONBITMAP`, unusable for the SWCA
/// fallback) so the caller can create a fresh window for that path.
fn try_create_composition(
  rect: &Rect,
  params: BlurOverlayParams,
) -> Option<(HWND, BlurVisual)> {
  let hwnd = match create_window(rect, true) {
    Ok(hwnd) => hwnd,
    Err(err) => {
      tracing::warn!(
        "Blur overlay composition window creation failed: {err}."
      );
      return None;
    }
  };

  apply_hostbackdrop(hwnd);

  match BlurVisual::create(hwnd, rect, params) {
    Ok(visual) => Some((hwnd, visual)),
    Err(err) => {
      tracing::warn!(
        "Composition blur pipeline unavailable, falling back to SWCA \
         acrylic: {err}."
      );
      // SAFETY: `hwnd` was just created above and not yet handed to a
      // caller; safe to destroy immediately on this failure path.
      unsafe {
        let _ = DestroyWindow(hwnd);
      }
      None
    }
  }
}

/// A persistent backdrop window that provides a blur-behind effect
/// (acrylic or plain, per [`BlurOverlayParams::style`]) for a paired
/// managed window.
///
/// Positioned directly behind an `anchor` window in z-order (typically the
/// managed window itself, or its surrogate while one is active -- see
/// [`set_rect`]/[`sync_z_order`]) and kept pixel-aligned with its DWM frame
/// rect. When the managed window is semi-transparent (via the
/// `transparency` window effect), the blurred content visible through the
/// overlay shows through the window, producing a frosted-glass look.
///
/// Anchoring directly behind the managed window (rather than e.g. the
/// global `HWND_BOTTOM`) matters because `HostBackdropBrush` only ever
/// samples whatever is visually behind the overlay *at the overlay's own
/// z-position* -- pinned to the very bottom of the system z-order, it could
/// only ever blur the bare desktop wallpaper; anchored directly behind its
/// own window, it picks up whatever's actually stacked there, other real
/// windows included.
///
/// [`set_rect`]: NativeBlurOverlay::set_rect
/// [`sync_z_order`]: NativeBlurOverlay::sync_z_order
///
/// For [`BackdropStyle::Acrylic`], renders via a
/// `Windows.UI.Composition` pipeline (live host-backdrop brush, a
/// continuously adjustable Gaussian-blur effect graph, and a continuous
/// corner-radius clip) when available, falling back to
/// `SetWindowCompositionAttribute` with `ACCENT_ENABLE_ACRYLICBLURBEHIND`
/// otherwise -- e.g. pre-Windows 10 1803, or if any step of the Composition
/// setup fails.
///
/// For [`BackdropStyle::Blur`], goes straight to
/// `SetWindowCompositionAttribute` with `ACCENT_ENABLE_BLURBEHIND` and
/// builds no Composition pipeline at all, which is what makes it the cheap
/// style: DWM applies its own fixed blur instead of this process driving a
/// D2D effect graph per frame.
///
/// Whenever there's no Composition pipeline (either style), `blur_amount`/
/// `corner_radius`/`opacity`/`saturation` become no-ops -- the OS gives no
/// such knobs for SWCA -- but `tint` keeps working.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeBlurOverlay {
  /// Raw window handle stored as `isize` so that `NativeBlurOverlay` is
  /// `Send` even though `HWND` is not.
  hwnd: isize,

  /// Current tint/blur-amount/corner-radius/opacity/saturation.
  /// Applied via SWCA in the fallback path (tint only), or as the
  /// Composition pipeline's live properties otherwise.
  params: BlurOverlayParams,

  /// Last rect applied via `set_rect`, used to skip redundant
  /// `SetWindowPos` calls when the overlay hasn't actually moved.
  rect: Rect,

  /// `HWND` of the window this overlay is positioned directly behind (its
  /// z-order anchor), as raw `isize`. `HostBackdropBrush` samples whatever
  /// is visually behind the overlay *at the overlay's own z-position* --
  /// pinning the overlay directly behind its own managed window (rather
  /// than e.g. the global `HWND_BOTTOM`) is what lets it pick up other real
  /// windows stacked there, not just the desktop wallpaper. Tracked so
  /// [`set_rect`]/[`sync_z_order`] can skip a redundant `SetWindowPos` when
  /// the anchor hasn't changed.
  ///
  /// [`set_rect`]: NativeBlurOverlay::set_rect
  /// [`sync_z_order`]: NativeBlurOverlay::sync_z_order
  anchor: isize,

  /// Whether the overlay window is currently shown.
  ///
  /// Tracked explicitly (rather than inferred from a change in `rect`) so
  /// that a caller re-showing the overlay after [`hide`] with an unchanged
  /// rect still issues the `SetWindowPos` needed to reapply
  /// `SWP_SHOWWINDOW` -- the rect-unchanged fast path in [`set_rect`] would
  /// otherwise skip that call entirely, leaving the overlay hidden.
  ///
  /// [`hide`]: NativeBlurOverlay::hide
  /// [`set_rect`]: NativeBlurOverlay::set_rect
  is_visible: bool,

  /// `Some` when the Composition pipeline is active for this overlay;
  /// `None` when running the SWCA fallback.
  composition: Option<BlurVisual>,
}

/// Generates a `NativeBlurOverlay` setter for a single `f32` knob shared
/// with the `BlurVisual` composition pipeline: no-ops when `value` matches
/// the last-applied `params.$field`, otherwise stores it and forwards to
/// the matching `BlurVisual` setter (a no-op in the SWCA fallback, since
/// `composition` is `None` there).
///
/// Not used for `set_tint`, which also has to re-apply via SWCA directly
/// in the fallback case (`tint` is the only knob SWCA supports).
macro_rules! blur_overlay_setter {
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
            concat!("Blur overlay ", stringify!($field), " update failed: {e}."),
            e = e
          );
        }
      }
    }
  };
}

impl NativeBlurOverlay {
  /// Creates a new blur overlay sized and positioned to `rect`, with the
  /// given `params` (blur amount, corner radius, opacity, and saturation
  /// are only honored when the Composition pipeline is available).
  ///
  /// The overlay is shown immediately, positioned directly behind `anchor`
  /// (see the `anchor` field doc) -- typically the `HWND` of the managed
  /// window it's tracking, or its surrogate's `HWND` while one is active.
  pub fn create(
    rect: &Rect,
    params: BlurOverlayParams,
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
      tracing::warn!("Blur overlay SetWindowPos failed on create: {e}.");
    }

    Ok(Self {
      hwnd: hwnd.0,
      params,
      rect: rect.clone(),
      anchor: anchor.0,
      is_visible: true,
      composition,
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
  /// recomposite they trigger) on every sync tick for overlays that haven't
  /// actually moved. Always issues the call when re-showing after [`hide`],
  /// even at an unchanged rect/anchor, since that's what reapplies
  /// `SWP_SHOWWINDOW`.
  ///
  /// Callers that only need to correct z-order drift (`anchor` may have
  /// changed but `rect` hasn't, e.g. after an unrelated window steals
  /// focus) without a full position sync should use [`sync_z_order`]
  /// instead -- it skips the position arguments entirely and stays cheap
  /// enough to call unconditionally every tick.
  ///
  /// [`hide`]: NativeBlurOverlay::hide
  /// [`sync_z_order`]: NativeBlurOverlay::sync_z_order
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
      tracing::warn!("Blur overlay SetWindowPos failed: {e}.");
      return;
    }

    if let Some(composition) = &self.composition {
      if let Err(e) = composition.set_rect(rect) {
        tracing::warn!("Blur overlay composition resize failed: {e}.");
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
  /// [`set_rect`]: NativeBlurOverlay::set_rect
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

    if let Some(composition) = &self.composition {
      if let Err(e) = composition.set_rect(rect) {
        tracing::warn!("Blur overlay composition resize failed: {e}.");
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
  pub fn sync_z_order(&mut self, anchor: HWND) -> crate::Result<()> {
    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    let prev = unsafe { GetWindow(self.hwnd(), GW_HWNDPREV) };
    if prev == anchor {
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

    match &self.composition {
      Some(composition) => {
        if let Err(e) = composition.set_tint(tint) {
          tracing::warn!("Blur overlay composition tint update failed: {e}.");
        }
      }
      None => {
        apply_swca_for_style(self.hwnd(), self.params.style, tint);
      }
    }
  }

  blur_overlay_setter!(
    /// Updates the blur radius/intensity; re-applies only when the value
    /// changes. No-op when running the SWCA fallback (no such knob exists).
    ///
    /// Compares the raw `f32` for exact equality, same as `set_tint`'s ABGR
    /// comparison -- the value only ever changes when a caller passes a
    /// genuinely different, config-resolved number, not through any
    /// arithmetic that could introduce drift.
    set_blur_amount, blur_amount
  );

  blur_overlay_setter!(
    /// Updates the corner radius, in pixels; re-applies only when the
    /// value changes. No-op when running the SWCA fallback (no such knob
    /// exists).
    ///
    /// See `set_blur_amount` for why exact `f32` equality is intentional
    /// here.
    set_corner_radius, corner_radius
  );

  blur_overlay_setter!(
    /// Updates the overlay's own opacity (blur + tint together, as one
    /// unit); re-applies only when the value changes. No-op when running
    /// the SWCA fallback (no such knob exists).
    ///
    /// See `set_blur_amount` for why exact `f32` equality is intentional
    /// here.
    set_opacity, opacity
  );

  blur_overlay_setter!(
    /// Updates the saturation of the blurred backdrop; re-applies only
    /// when the value changes. No-op when running the SWCA fallback (no
    /// such knob exists).
    ///
    /// See `set_blur_amount` for why exact `f32` equality is intentional
    /// here.
    set_saturation, saturation
  );

  /// Rebuilds the overlay's backing window for a changed
  /// [`BlurOverlayParams::style`], preserving its current rect, z-order
  /// anchor, and visibility.
  ///
  /// The style is the one param that can't be re-applied in place:
  /// [`BackdropStyle::Acrylic`] roots a `Windows.UI.Composition` visual
  /// tree on a `WS_EX_NOREDIRECTIONBITMAP` window, while
  /// [`BackdropStyle::Blur`] composites through the very GDI redirection
  /// surface that flag suppresses (see [`create_window`]). The new window
  /// is fully built before the old one is torn down, so a failure here
  /// leaves the existing overlay intact and the caller retries on the next
  /// sync tick.
  fn recreate(&mut self, params: BlurOverlayParams) -> crate::Result<()> {
    let (hwnd, composition) = create_backing_window(&self.rect, params)?;

    if self.is_visible {
      // SAFETY: `hwnd` was just created above, and `self.anchor` is the
      // handle this overlay is already anchored behind.
      if let Err(e) = unsafe {
        SetWindowPos(
          hwnd,
          HWND(self.anchor),
          self.rect.x(),
          self.rect.y(),
          self.rect.width(),
          self.rect.height(),
          SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
        )
      } {
        tracing::warn!(
          "Blur overlay SetWindowPos failed on style change: {e}."
        );
      }
    }

    let previous_hwnd = self.hwnd();

    // Drop the outgoing visual tree before destroying the window it's
    // rooted to, same ordering as `Drop`.
    self.composition.take();

    // SAFETY: `previous_hwnd` is this overlay's own window, valid until
    // now and no longer referenced once `self.hwnd` is reassigned below.
    unsafe {
      let _ = DestroyWindow(previous_hwnd);
    }

    self.hwnd = hwnd.0;
    self.composition = composition;
    self.params = params;

    Ok(())
  }

  /// Applies `params`, re-applying only whichever fields actually changed
  /// (each setter no-ops internally on an unchanged value). Convenience
  /// for the call sites that already have a full `BlurOverlayParams`
  /// rather than one field at a time.
  ///
  /// A changed `style` goes through [`recreate`] instead, since it selects
  /// how the backing window itself is built; every other field is baked in
  /// by that rebuild, so no setter runs afterwards.
  ///
  /// [`recreate`]: NativeBlurOverlay::recreate
  pub fn apply(&mut self, params: BlurOverlayParams) {
    if self.params.style != params.style {
      if let Err(e) = self.recreate(params) {
        tracing::warn!("Blur overlay style change failed: {e}.");
      }
      return;
    }

    self.set_tint(params.tint);
    self.set_blur_amount(params.blur_amount);
    self.set_corner_radius(params.corner_radius);
    self.set_opacity(params.opacity);
    self.set_saturation(params.saturation);
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

impl Drop for NativeBlurOverlay {
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
