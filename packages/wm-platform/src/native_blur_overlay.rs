use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, ShowWindow, HWND_BOTTOM, SWP_NOACTIVATE,
      SWP_NOSENDCHANGING, SWP_SHOWWINDOW, SW_HIDE, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::swca::{apply_swca_accent, ACCENT_ENABLE_ACRYLICBLURBEHIND},
  Rect,
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
      // Null background brush: SWCA composites the acrylic layer; GDI
      // never paints the client area.
      ..Default::default()
    };

    // SAFETY: `wnd_class` is properly initialized with a static class name
    // and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
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
/// The overlay uses `SetWindowCompositionAttribute` with
/// `ACCENT_ENABLE_ACRYLICBLURBEHIND` on its own `HWND`. This sidesteps the
/// `WS_EX_LAYERED`/SWCA compositing conflict that prevents applying SWCA
/// directly to layered managed windows.
///
/// # Platform-specific
///
/// Only available on Windows. The acrylic effect requires Windows 10 1803+;
/// on older versions the backdrop degrades gracefully to an opaque overlay
/// tinted by `tint`.
pub struct NativeBlurOverlay {
  /// Raw window handle stored as `isize` so that `NativeBlurOverlay` is
  /// `Send` even though `HWND` is not.
  hwnd: isize,

  /// Current ABGR tint applied to the overlay via SWCA.
  tint: u32,

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
}

impl NativeBlurOverlay {
  /// Creates a new blur overlay sized and positioned to `rect` with the
  /// given ABGR `tint`.
  ///
  /// The overlay is shown immediately at `HWND_BOTTOM`.
  pub fn create(rect: &Rect, tint: u32) -> crate::Result<Self> {
    ensure_class_registered();

    // SAFETY: All parameters are valid. The class is guaranteed registered
    // by `ensure_class_registered`. No parent HWND is needed.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
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

    apply_swca_accent(hwnd, ACCENT_ENABLE_ACRYLICBLURBEHIND, tint);

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
      tint,
      rect: rect.clone(),
      is_visible: true,
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

    self.rect = rect.clone();
    self.is_visible = true;
  }

  /// Updates the ABGR tint; re-applies SWCA only when the value changes.
  pub fn set_tint(&mut self, tint: u32) {
    if self.tint != tint {
      self.tint = tint;
      apply_swca_accent(self.hwnd(), ACCENT_ENABLE_ACRYLICBLURBEHIND, tint);
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

impl Drop for NativeBlurOverlay {
  fn drop(&mut self) {
    // SAFETY: `self.hwnd()` is a valid window handle and `Drop` is called
    // at most once.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}
