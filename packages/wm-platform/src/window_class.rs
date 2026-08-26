use std::sync::OnceLock;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
      DefWindowProcW, RegisterClassW, WNDCLASSW,
    },
  },
};

/// Registers a window class with `wnd_proc` and `class_name`, exactly once
/// per process for the given `registered` cell.
///
/// Shared by the overlay window types ([`NativeSurrogate`], [`NativeBlurOverlay`],
/// [`NativeBorderOverlay`], [`NativeIrisOverlay`]), which each need a distinct
/// class name and (for the iris overlay) window procedure, but otherwise
/// register identically -- previously each copy-pasted its own `OnceLock` +
/// `WNDCLASSW` + `RegisterClassW` call.
///
/// [`NativeSurrogate`]: crate::NativeSurrogate
/// [`NativeBlurOverlay`]: crate::NativeBlurOverlay
/// [`NativeBorderOverlay`]: crate::NativeBorderOverlay
/// [`NativeIrisOverlay`]: crate::NativeIrisOverlay
pub(crate) fn ensure_class_registered(
  registered: &OnceLock<()>,
  class_name: PCWSTR,
  wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
) {
  registered.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: class_name,
      lpfnWndProc: Some(wnd_proc),
      // Null background brush: SWCA/Composition (or, for the surrogate, the
      // DWM thumbnail) paint the client area; GDI never touches it.
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Default window procedure: forwards every message to `DefWindowProcW`
/// unchanged.
///
/// Shared by overlay window classes with no custom message handling --
/// their visuals are painted entirely by DWM/`Windows.UI.Composition`, so
/// the window itself never needs to handle `WM_PAINT` or anything else
/// (unlike [`NativeIrisOverlay`], which paints a GDI snapshot and supplies
/// its own window procedure instead of this one).
///
/// [`NativeIrisOverlay`]: crate::NativeIrisOverlay
pub(crate) unsafe extern "system" fn default_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: All parameters are forwarded unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
