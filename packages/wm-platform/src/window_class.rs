use std::sync::OnceLock;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
      DefWindowProcW, GetWindow, GetWindowLongPtrW, RegisterClassW,
      SetWindowPos, GWL_EXSTYLE, GW_HWNDNEXT, HWND_NOTOPMOST,
      HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING,
      SWP_NOSIZE, WNDCLASSW, WS_EX_TOPMOST,
    },
  },
};

/// Registers a window class with `wnd_proc` and `class_name`, exactly once
/// per process for the given `registered` cell.
///
/// Shared by the overlay window types ([`NativeSurrogate`],
/// [`NativeBackdropOverlay`],
/// [`NativeBorderOverlay`], [`NativeIrisOverlay`]), which each need a
/// distinct class name and (for the iris overlay) window procedure, but
/// otherwise register identically -- previously each copy-pasted its own
/// `OnceLock`
/// + `WNDCLASSW` + `RegisterClassW` call.
///
/// [`NativeSurrogate`]: crate::NativeSurrogate
/// [`NativeBackdropOverlay`]: crate::NativeBackdropOverlay
/// [`NativeBorderOverlay`]: crate::NativeBorderOverlay
/// [`NativeIrisOverlay`]: crate::NativeIrisOverlay
pub(crate) fn ensure_class_registered(
  registered: &OnceLock<()>,
  class_name: PCWSTR,
  wnd_proc: unsafe extern "system" fn(
    HWND,
    u32,
    WPARAM,
    LPARAM,
  ) -> LRESULT,
) {
  registered.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: class_name,
      lpfnWndProc: Some(wnd_proc),
      // Null background brush: SWCA/Composition (or, for the surrogate,
      // the DWM thumbnail) paint the client area; GDI never touches
      // it.
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

/// Whether `hwnd` currently sits in the always-on-top band.
pub(crate) fn is_topmost(hwnd: HWND) -> bool {
  // SAFETY: `hwnd` is a valid window handle; `GetWindowLongPtrW` only
  // reads.
  let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };

  #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
  let ex_style = ex_style as u32;

  ex_style & WS_EX_TOPMOST.0 != 0
}

/// Guard against a pathological z-order in [`insert_after_point`]; far
/// above any run of stray topmost windows seen in practice.
const MAX_INSERT_AFTER_WALK: usize = 512;

/// Insert-after handle that places a non-topmost window directly behind
/// `anchor`, or as close behind it as Windows allows.
///
/// Windows silently drops an insert-after for a non-topmost window when
/// the window right below the insert-after target is topmost --
/// `SetWindowPos` still reports success. Hidden windows flagged
/// `WS_EX_TOPMOST` (IME, tray and launcher helpers) routinely sit at the
/// bottom of the normal band, so the lowest managed window could never get
/// its overlays behind it, leaving its backdrop covering it. Inserting
/// after one of those topmost windows instead would pull the window into
/// the topmost band, so this walks past them to the first non-topmost
/// window not itself followed by a topmost one.
///
/// Returns `anchor` unchanged when it is topmost, or when nothing below it
/// is an acceptable target.
pub(crate) fn insert_after_point(anchor: HWND) -> HWND {
  if is_topmost(anchor) {
    return anchor;
  }

  let mut current = anchor;

  for _ in 0..MAX_INSERT_AFTER_WALK {
    let next = next_in_z_order(current);
    let is_next_topmost = next.0 != 0 && is_topmost(next);

    if !is_topmost(current) && !is_next_topmost {
      return current;
    }

    if next.0 == 0 {
      break;
    }

    current = next;
  }

  anchor
}

fn next_in_z_order(hwnd: HWND) -> HWND {
  // SAFETY: A stale `hwnd` just makes `GetWindow` return `HWND(0)`.
  unsafe { GetWindow(hwnd, GW_HWNDNEXT) }
}

/// Puts `overlay` in the same always-on-top band as `anchor`, if it isn't
/// already.
///
/// Must run before an overlay is positioned behind its window, and is the
/// reason overlays are never anchored to `HWND_TOPMOST` directly. Windows
/// keeps topmost windows in a band above every other window, and it will
/// not leave a non-topmost window wedged between two topmost ones -- it
/// silently drops it below the whole band. Matching the band first is what
/// makes the subsequent "insert directly behind this exact `HWND`" call
/// stick.
///
/// The alternative, passing `HWND_TOPMOST` as the insert-after target,
/// moves the overlay to the *top* of that band -- above the very window it
/// is supposed to sit behind. With an opaque backdrop that reads as the
/// window disappearing and being replaced by its own backdrop.
pub(crate) fn match_z_band(overlay: HWND, anchor: HWND) {
  let wanted = is_topmost(anchor);

  if is_topmost(overlay) == wanted {
    return;
  }

  let band = if wanted { HWND_TOPMOST } else { HWND_NOTOPMOST };

  // SAFETY: Both handles are valid; the flags leave position, size and
  // activation untouched, so this only moves the window between bands.
  if let Err(err) = unsafe {
    SetWindowPos(
      overlay,
      band,
      0,
      0,
      0,
      0,
      SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOMOVE | SWP_NOSIZE,
    )
  } {
    tracing::warn!("Overlay topmost-band sync failed: {err}.");
  }
}
