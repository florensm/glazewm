use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{GetStockObject, HBRUSH, BLACK_BRUSH},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetLayeredWindowAttributes, SetWindowPos, HWND_TOP, HTTRANSPARENT,
      LWA_ALPHA, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING,
      SWP_SHOWWINDOW, WM_NCHITTEST, WNDCLASSW, WS_EX_LAYERED,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    },
  },
};

use crate::Rect;

/// Ensures the overlay window class is registered exactly once per process.
static OVERLAY_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Window procedure for the overlay.
///
/// Returns `HTTRANSPARENT` for `WM_NCHITTEST` so that mouse events fall
/// through to the windows beneath the overlay, while the overlay itself
/// remains visually rendered. All other messages are forwarded to
/// `DefWindowProcW`.
///
/// SAFETY: All parameters are forwarded unchanged to Win32 APIs.
unsafe extern "system" fn overlay_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if msg == WM_NCHITTEST {
    return LRESULT(HTTRANSPARENT as isize);
  }
  // SAFETY: All parameters are valid and passed through unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Registers the `GlazeWM_ScratchpadOverlay` window class.
fn ensure_class_registered() {
  OVERLAY_CLASS_REGISTERED.get_or_init(|| {
    // SAFETY: `BLACK_BRUSH` is a valid stock object that lives for the
    // duration of the process.
    let hbr_black = unsafe { HBRUSH(GetStockObject(BLACK_BRUSH).0) };

    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_ScratchpadOverlay"),
      lpfnWndProc: Some(overlay_wnd_proc),
      hbrBackground: hbr_black,
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialised `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Semi-transparent black overlay drawn over the full monitor when the
/// scratchpad is shown.
///
/// `WM_NCHITTEST` returns `HTTRANSPARENT` so clicks fall through to the
/// scratchpad windows (TOPMOST) or the desktop beneath. The window is
/// excluded from GlazeWM management because `WS_EX_NOACTIVATE` and
/// `WS_EX_TOOLWINDOW` cause the `check_is_manageable` guard to reject it.
/// Dropped when the scratchpad is hidden.
pub struct NativeScratchpadOverlay {
  /// Raw `HWND` value of the overlay window.
  hwnd: isize,
}

impl NativeScratchpadOverlay {
  /// Creates and shows a full-monitor dim overlay.
  ///
  /// `opacity` is clamped to `0.0–1.0` and mapped to a Win32 alpha value.
  /// The overlay is placed at `HWND_TOP` so it sits above all non-topmost
  /// windows but below the scratchpad windows, which use `shown_on_top`
  /// (TOPMOST).
  pub fn new(monitor_rect: &Rect, opacity: f32) -> crate::Result<Self> {
    ensure_class_registered();

    // SAFETY: Class name and window name are static wide strings. No parent
    // or menu handles are needed for a borderless overlay.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
        w!("GlazeWM_ScratchpadOverlay"),
        w!(""),
        WS_POPUP,
        monitor_rect.x(),
        monitor_rect.y(),
        monitor_rect.width(),
        monitor_rect.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create scratchpad overlay window.".to_string(),
      ));
    }

    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;

    // SAFETY: `hwnd` is a valid window handle returned by `CreateWindowExW`.
    unsafe {
      SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA)?;
    }

    // Show above non-topmost windows without stealing activation.
    // SAFETY: `hwnd` and `HWND_TOP` are valid.
    unsafe {
      SetWindowPos(
        hwnd,
        HWND_TOP,
        monitor_rect.x(),
        monitor_rect.y(),
        monitor_rect.width(),
        monitor_rect.height(),
        SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING,
      )?;
    }

    Ok(Self { hwnd: hwnd.0 })
  }
}

impl Drop for NativeScratchpadOverlay {
  fn drop(&mut self) {
    // SAFETY: `hwnd` is the overlay window created by this struct and has
    // not been destroyed elsewhere.
    let _ = unsafe { DestroyWindow(HWND(self.hwnd)) };
  }
}
