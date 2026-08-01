use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_HOSTBACKDROPBRUSH},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, ShowWindow, HWND_BOTTOM, SWP_NOACTIVATE,
      SWP_NOSENDCHANGING, SWP_SHOWWINDOW, SW_HIDE, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
      WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::{
    composition::BlurVisual,
    swca::{
      apply_swca_accent, ACCENT_ENABLE_ACRYLICBLURBEHIND,
      ACCENT_ENABLE_HOSTBACKDROP,
    },
  },
  BlurOverlayParams, Rect,
};

/// Ensures the blur-overlay window class is registered exactly once per
/// process.
static BLUR_OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

/// Default window procedure for the blur-overlay class.
unsafe extern "system" fn default_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: All parameters are forwarded unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_class_registered() {
  BLUR_OVERLAY_CLASS.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_BlurOverlay"),
      lpfnWndProc: Some(default_wnd_proc),
      // Null background brush: SWCA/Composition composite the acrylic
      // layer; GDI never paints the client area.
      ..Default::default()
    };

    // SAFETY: `wnd_class` is properly initialized with a static class name
    // and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
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

  let ex_style = if composition {
    WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_NOREDIRECTIONBITMAP
  } else {
    WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW
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
  apply_swca_accent(hwnd, ACCENT_ENABLE_HOSTBACKDROP, 0);

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

/// A persistent backdrop window that provides an acrylic blur-behind effect
/// for a paired managed window.
///
/// Positioned at `HWND_BOTTOM` (behind all normal windows) and kept
/// pixel-aligned with the managed window's DWM frame rect. When the managed
/// window is semi-transparent (via the `transparency` window effect), the
/// blurred desktop visible through the overlay shows through the window,
/// producing a frosted-glass look.
///
/// Renders via a `Windows.UI.Composition` pipeline (live host-backdrop
/// brush, a continuously adjustable Gaussian-blur effect graph, and a
/// continuous corner-radius clip) when available, falling back to
/// `SetWindowCompositionAttribute` with `ACCENT_ENABLE_ACRYLICBLURBEHIND`
/// otherwise -- e.g. pre-Windows 10 1803, or if any step of the Composition
/// setup fails. In the fallback, `blur_amount`/`corner_radius`/`opacity`/
/// `saturation` become no-ops (the OS gives no such knobs for SWCA
/// acrylic) but `tint` keeps working the same as before.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeBlurOverlay {
  /// Raw window handle stored as `isize` so that `NativeBlurOverlay` is
  /// `Send` even though `HWND` is not.
  hwnd: isize,

  /// Current tint/blur-amount/corner-radius/opacity/saturation. Applied
  /// via SWCA in the fallback path (tint only), or as the Composition
  /// pipeline's live properties otherwise.
  params: BlurOverlayParams,

  /// Last rect applied via `set_rect`, used to skip redundant
  /// `SetWindowPos` calls when the overlay hasn't actually moved.
  rect: Rect,

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

impl NativeBlurOverlay {
  /// Creates a new blur overlay sized and positioned to `rect`, with the
  /// given `params` (blur amount, corner radius, opacity, and saturation
  /// are only honored when the Composition pipeline is available).
  ///
  /// The overlay is shown immediately at `HWND_BOTTOM`.
  pub fn create(rect: &Rect, params: BlurOverlayParams) -> crate::Result<Self> {
    let (hwnd, composition) =
      if let Some((hwnd, visual)) = try_create_composition(rect, params) {
        (hwnd, Some(visual))
      } else {
        let hwnd = create_window(rect, false)?;
        apply_swca_accent(hwnd, ACCENT_ENABLE_ACRYLICBLURBEHIND, params.tint);
        (hwnd, None)
      };

    // SAFETY: `hwnd` is a valid window just created above.
    if let Err(e) = unsafe {
      SetWindowPos(
        hwnd,
        HWND_BOTTOM,
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

  /// Repositions and resizes the overlay to match `rect`, keeping it at
  /// `HWND_BOTTOM`, and ensures it's shown.
  ///
  /// No-op if `rect` matches the last-applied rect and the overlay is
  /// already visible, to avoid redundant `SetWindowPos` calls (and the DWM
  /// recomposite they trigger) on every sync tick for overlays that haven't
  /// actually moved. Always issues the call when re-showing after [`hide`],
  /// even at an unchanged rect, since that's what reapplies
  /// `SWP_SHOWWINDOW`.
  ///
  /// [`hide`]: NativeBlurOverlay::hide
  pub fn set_rect(&mut self, rect: &Rect) {
    if self.is_visible && &self.rect == rect {
      return;
    }

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    if let Err(e) = unsafe {
      SetWindowPos(
        self.hwnd(),
        HWND_BOTTOM,
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
    self.is_visible = true;
  }

  /// Updates the ABGR tint; re-applies only when the value changes.
  pub fn set_tint(&mut self, tint: u32) {
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
        apply_swca_accent(self.hwnd(), ACCENT_ENABLE_ACRYLICBLURBEHIND, tint);
      }
    }
  }

  /// Updates the blur radius/intensity; re-applies only when the value
  /// changes. No-op when running the SWCA fallback (no such knob exists).
  ///
  /// Compares the raw `f32` for exact equality, same as `set_tint`'s ABGR
  /// comparison -- the value only ever changes when a caller passes a
  /// genuinely different, config-resolved number, not through any
  /// arithmetic that could introduce drift.
  #[allow(clippy::float_cmp)]
  pub fn set_blur_amount(&mut self, blur_amount: f32) {
    if self.params.blur_amount == blur_amount {
      return;
    }
    self.params.blur_amount = blur_amount;

    if let Some(composition) = &mut self.composition {
      if let Err(e) = composition.set_blur_amount(blur_amount) {
        tracing::warn!("Blur overlay blur-amount update failed: {e}.");
      }
    }
  }

  /// Updates the corner radius, in pixels; re-applies only when the value
  /// changes. No-op when running the SWCA fallback (no such knob exists).
  ///
  /// See `set_blur_amount` for why exact `f32` equality is intentional
  /// here.
  #[allow(clippy::float_cmp)]
  pub fn set_corner_radius(&mut self, corner_radius: f32) {
    if self.params.corner_radius == corner_radius {
      return;
    }
    self.params.corner_radius = corner_radius;

    if let Some(composition) = &self.composition {
      if let Err(e) = composition.set_corner_radius(corner_radius) {
        tracing::warn!("Blur overlay corner-radius update failed: {e}.");
      }
    }
  }

  /// Updates the overlay's own opacity (blur + tint together, as one
  /// unit); re-applies only when the value changes. No-op when running
  /// the SWCA fallback (no such knob exists).
  ///
  /// See `set_blur_amount` for why exact `f32` equality is intentional
  /// here.
  #[allow(clippy::float_cmp)]
  pub fn set_opacity(&mut self, opacity: f32) {
    if self.params.opacity == opacity {
      return;
    }
    self.params.opacity = opacity;

    if let Some(composition) = &self.composition {
      if let Err(e) = composition.set_opacity(opacity) {
        tracing::warn!("Blur overlay opacity update failed: {e}.");
      }
    }
  }

  /// Updates the saturation of the blurred backdrop; re-applies only when
  /// the value changes. No-op when running the SWCA fallback (no such
  /// knob exists).
  ///
  /// See `set_blur_amount` for why exact `f32` equality is intentional
  /// here.
  #[allow(clippy::float_cmp)]
  pub fn set_saturation(&mut self, saturation: f32) {
    if self.params.saturation == saturation {
      return;
    }
    self.params.saturation = saturation;

    if let Some(composition) = &mut self.composition {
      if let Err(e) = composition.set_saturation(saturation) {
        tracing::warn!("Blur overlay saturation update failed: {e}.");
      }
    }
  }

  /// Applies `params`, re-applying only whichever fields actually changed
  /// (each setter no-ops internally on an unchanged value). Convenience
  /// for the call sites that already have a full `BlurOverlayParams`
  /// rather than one field at a time.
  pub fn apply(&mut self, params: BlurOverlayParams) {
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
