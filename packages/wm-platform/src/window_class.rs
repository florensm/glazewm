use std::sync::OnceLock;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Dwm::{
      DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
    },
    UI::WindowsAndMessaging::{
      DefWindowProcW, GetWindow, GetWindowLongPtrW, GetWindowRect,
      IsWindowVisible, RegisterClassW, SetWindowPos, GWL_EXSTYLE,
      GW_HWNDNEXT, GW_HWNDPREV, HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE,
      WNDCLASSW, WS_EX_TOPMOST,
    },
  },
};

use crate::overlay_window::OverlayKind;

/// Registers a window class with `wnd_proc` and `class_name`, exactly once
/// per process for the given `registered` cell.
///
/// Shared by the overlay window types ([`NativeSurrogate`],
/// [`NativeBackdropOverlay`], [`NativeBorderOverlay`],
/// [`NativeIrisOverlay`]), which differ only in class name and (for the
/// iris overlay) window procedure.
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
      // Null background brush: composition (or, for the surrogate, the
      // DWM thumbnail) paints the client area; GDI never touches it.
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

/// Insert-after handle that places `overlay` directly above `anchor`.
///
/// Falls back to `HWND_TOP` when `anchor` is the highest window of its
/// band: inserting after a topmost window from below would pull the
/// overlay into the topmost band instead.
pub(crate) fn insert_above_point(anchor: HWND, overlay: HWND) -> HWND {
  // SAFETY: A stale handle just makes `GetWindow` return `HWND(0)`.
  let mut prev = unsafe { GetWindow(anchor, GW_HWNDPREV) };

  // Already in place (e.g. hidden in its slot): `SetWindowPos` silently
  // ignores a window inserted after itself -- `SWP_SHOWWINDOW` included.
  if prev == overlay {
    // SAFETY: As above.
    prev = unsafe { GetWindow(overlay, GW_HWNDPREV) };
  }

  if prev.0 == 0 || (is_topmost(prev) && !is_topmost(anchor)) {
    HWND_TOP
  } else {
    prev
  }
}

/// Where to put `overlay` so it covers `anchor`: whether it belongs in the
/// always-on-top band, and the insert-after handle within it.
///
/// While `anchor` is the highest visible window of the normal band, the
/// overlay goes to the *bottom* of the topmost band rather than directly
/// above `anchor`. On screen that's the same spot, but Windows lifts a
/// window to the top of its band whenever one of its owned popups opens (a
/// dropdown, a menu, a tooltip): in the same band, that lift covers the
/// overlay until it's noticed and undone, which shows as a flash of the
/// untouched window. As soon as another window on screen overlaps
/// `anchor` from above, the overlay goes back directly above `anchor`, so
/// it never covers unrelated windows.
pub(crate) fn above_placement(
  anchor: HWND,
  overlay: HWND,
) -> (bool, HWND) {
  if is_topmost(anchor) {
    return (true, insert_above_point(anchor, overlay));
  }

  let area = frame_bounds(anchor);
  let mut current = anchor;

  for _ in 0..MAX_INSERT_AFTER_WALK {
    // SAFETY: A stale handle just makes `GetWindow` return `HWND(0)`.
    current = unsafe { GetWindow(current, GW_HWNDPREV) };

    // Nothing is topmost at all: the top of the topmost band is also its
    // bottom.
    if current.0 == 0 {
      return (true, HWND_TOPMOST);
    }

    if current == overlay {
      continue;
    }

    // The lowest topmost window: insert right below it.
    if is_topmost(current) {
      return (true, current);
    }

    if is_shown_over(current, area.as_ref()) {
      tracing::debug!(
        "Color theme overlay kept in the normal band: {current:?} is above \
         its window."
      );
      break;
    }
  }

  (false, insert_above_point(anchor, overlay))
}

/// Moves `window` into or out of the always-on-top band, if it isn't
/// there already.
pub(crate) fn set_topmost(window: HWND, topmost: bool) {
  if is_topmost(window) == topmost {
    return;
  }

  let band = if topmost {
    HWND_TOPMOST
  } else {
    HWND_NOTOPMOST
  };

  // SAFETY: `window` is valid; the flags only move it between bands.
  if let Err(err) = unsafe {
    SetWindowPos(
      window,
      band,
      0,
      0,
      0,
      0,
      SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOMOVE | SWP_NOSIZE,
    )
  } {
    tracing::warn!("Overlay topmost-band change failed: {err}.");
  }
}

/// Whether nothing on screen over `anchor` sits between it and `overlay`
/// (see [`is_shown_over`]).
///
/// Other windows in between don't count: apps keep hidden IME helper
/// windows directly above themselves, and Windows keeps owned windows
/// above their owner, so demanding strict adjacency would restack the
/// overlay on every check for nothing.
pub(crate) fn is_directly_above(overlay: HWND, anchor: HWND) -> bool {
  let area = frame_bounds(anchor);
  let mut current = overlay;

  for _ in 0..MAX_INSERT_AFTER_WALK {
    current = next_in_z_order(current);

    // SAFETY: A stale handle just makes `IsWindowVisible` return false.
    if current.0 == 0 || current == anchor {
      return current == anchor;
    }

    if is_shown_over(current, area.as_ref()) {
      return false;
    }
  }

  false
}

/// Whether `hwnd` actually shows over part of `area`, i.e. whether an
/// overlay placed above it could hide it.
///
/// `IsWindowVisible` alone isn't enough: Windows keeps cloaked windows
/// (suspended UWP apps, shell hosts, windows on other virtual desktops)
/// visible and high in the z-order, and the WM's own border and backdrop
/// overlays sit above their windows' neighbors. Counting those would keep
/// the overlay out of the topmost band for good. With no `area`, every
/// visible, uncloaked window counts.
fn is_shown_over(hwnd: HWND, area: Option<&RECT>) -> bool {
  // SAFETY: A stale handle just makes `IsWindowVisible` return false.
  if !unsafe { IsWindowVisible(hwnd) }.as_bool()
    || is_cloaked(hwnd)
    || OverlayKind::is_overlay(hwnd)
  {
    return false;
  }

  match (area, frame_bounds(hwnd)) {
    (Some(area), Some(bounds)) => {
      bounds.left < area.right
        && area.left < bounds.right
        && bounds.top < area.bottom
        && area.top < bounds.bottom
    }
    _ => true,
  }
}

fn is_cloaked(hwnd: HWND) -> bool {
  let mut cloaked = 0u32;

  // SAFETY: `cloaked` outlives the call and matches the attribute's size;
  // a stale handle just makes the call fail.
  #[allow(clippy::cast_possible_truncation)]
  let result = unsafe {
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_CLOAKED,
      std::ptr::from_mut(&mut cloaked).cast(),
      std::mem::size_of::<u32>() as u32,
    )
  };

  result.is_ok() && cloaked != 0
}

/// The window's visible bounds, without the invisible resize borders
/// `GetWindowRect` includes, which overlap tiled neighbors.
fn frame_bounds(hwnd: HWND) -> Option<RECT> {
  let mut rect = RECT::default();

  // SAFETY: `rect` outlives the calls and matches the attribute's size; a
  // stale handle just makes them fail.
  #[allow(clippy::cast_possible_truncation)]
  let result = unsafe {
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut rect).cast(),
      std::mem::size_of::<RECT>() as u32,
    )
    .or_else(|_| GetWindowRect(hwnd, std::ptr::from_mut(&mut rect)))
  };

  result.ok().map(|()| rect)
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
