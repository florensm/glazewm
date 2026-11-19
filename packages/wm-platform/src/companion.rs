use std::{
  sync::{Mutex, PoisonError},
  time::{Duration, Instant},
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    UI::WindowsAndMessaging::{
      EnumWindows, GetPropW, GetWindow, GetWindowRect, IsWindowVisible,
      GW_HWNDNEXT,
    },
  },
};

use crate::{platform_impl::NativeWindow, window_class};

/// Window property another app sets on its own overlay window to declare
/// it the companion of the window it draws over (the value is that
/// window's handle), e.g. shades' recolored copy of a window.
///
/// The contract: the companion covers exactly the window's DWM extended
/// frame bounds from above it in the z-order, and while the window is
/// cloaked the companion stays shown but cloaked, so DWM keeps rendering
/// it for thumbnails. Animations then show the companion in the window's
/// place, and hand the window back once the companion covers it again.
const COMPANION_PROPERTY: PCWSTR = w!("shades.CompanionOf");

/// How long a scan for companions is reused. A workspace switch creates a
/// surrogate per window in one tick, which then costs one `EnumWindows`
/// instead of one each; a companion appearing within this window is missed
/// for the animation, which then shows the window itself.
const SNAPSHOT_TTL: Duration = Duration::from_millis(50);

/// Every `(window, companion)` pair from a scan, and when it ran.
type Snapshot = (Instant, Vec<(isize, isize)>);

static SNAPSHOT: Mutex<Option<Snapshot>> = Mutex::new(None);

/// Most windows [`Companion::covers_window`] walks down past, bounding a
/// walk of a z-order that changes under it.
const MAX_Z_ORDER_WALK: usize = 4096;

/// Upper bound on how long a stand-in keeps covering a window after it is
/// uncloaked, for a companion owner that never puts the companion back on
/// screen.
const HANDOFF_MAX_WAIT: Duration = Duration::from_millis(200);

/// Waits out a companion's return over its window when a stand-in showing
/// it hands the window back.
///
/// Once the real window is uncloaked, the companion's owner only learns of
/// it from a window event and shows the companion some time later, as it
/// restacks the companion over a window raised past it. Until then the
/// window shows as it really is (e.g. without shades' colors), so the
/// stand-in should keep covering it.
#[derive(Debug, Default)]
pub(crate) struct HandoffWait {
  /// When the companion was first found not covering its window.
  started_at: Option<Instant>,
  /// Whether the companion covered its window on the last call.
  covered_last: bool,
}

impl HandoffWait {
  /// Whether to keep covering the window: `companion` is shown by the
  /// stand-in and hasn't been over the window since the last call, and
  /// the wait hasn't timed out.
  ///
  /// Called once a tick, a composition apart: a companion found covering
  /// is handed the window on the next call, once what its owner did to put
  /// it back has been composited. Handed it at once, the stand-in could go
  /// a composition before the companion showed, and the window showed
  /// uncolored for a frame.
  pub(crate) fn awaits(&mut self, companion: Option<Companion>) -> bool {
    companion.is_some_and(|companion| {
      self.awaits_cover(companion.covers_window())
    })
  }

  /// [`awaits`](Self::awaits), for a companion that `covers` its window
  /// this call or not.
  fn awaits_cover(&mut self, covers: bool) -> bool {
    let covered_before = std::mem::replace(&mut self.covered_last, covers);
    if covers {
      return !covered_before;
    }

    let started = *self.started_at.get_or_insert_with(Instant::now);
    started.elapsed() < HANDOFF_MAX_WAIT
  }
}

/// A companion window (see [`COMPANION_PROPERTY`]) of a managed window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Companion {
  hwnd: isize,

  /// The window the companion draws over.
  source: isize,

  /// Where the companion's origin lies in the window's own coordinates,
  /// i.e. the window's invisible left and top resize borders.
  origin: (i32, i32),
}

impl Companion {
  /// The companion of `window`, if one is shown (possibly cloaked).
  pub(crate) fn find(window: HWND) -> Option<Self> {
    let now = Instant::now();
    let mut snapshot =
      SNAPSHOT.lock().unwrap_or_else(PoisonError::into_inner);

    let is_fresh = snapshot
      .as_ref()
      .is_some_and(|(taken, _)| now.duration_since(*taken) < SNAPSHOT_TTL);

    if !is_fresh {
      *snapshot = Some((now, scan()));
    }

    let hwnd = snapshot
      .as_ref()?
      .1
      .iter()
      .find(|(source, _)| *source == window.0)
      .map(|(_, companion)| HWND(*companion))
      .filter(|companion| companion_of(*companion) == Some(window))?;

    Some(Self {
      hwnd: hwnd.0,
      source: window.0,
      origin: frame_origin(window)?,
    })
  }

  pub(crate) fn hwnd(self) -> HWND {
    HWND(self.hwnd)
  }

  /// Whether the companion is on screen over its window: shown, not
  /// cloaked, and above the window in the z-order.
  pub(crate) fn covers_window(self) -> bool {
    // SAFETY: A stale handle just returns false.
    let is_visible = unsafe { IsWindowVisible(self.hwnd()) }.as_bool();
    is_visible
      && !NativeWindow::new(self.hwnd).is_cloaked().unwrap_or(false)
      && is_above(self.hwnd(), HWND(self.source))
  }

  /// Maps a thumbnail's `(rcSource, rcDestination)`, with the source in
  /// the window's coordinates, onto the companion.
  pub(crate) fn map_rects(self, src: RECT, dst: RECT) -> (RECT, RECT) {
    let mut bounds = RECT::default();

    // SAFETY: `bounds` outlives the call; a stale handle just fails.
    if unsafe { GetWindowRect(self.hwnd(), &raw mut bounds) }.is_err() {
      return (src, dst);
    }

    map_rects(
      src,
      dst,
      self.origin,
      (bounds.right - bounds.left, bounds.bottom - bounds.top),
    )
  }
}

/// Shifts `src` by `-origin` and clips it to `size`, clipping `dst`
/// proportionally, so the thumbnail never samples past the companion:
/// DWM renders an oversampled source as a transparent hole.
fn map_rects(
  src: RECT,
  dst: RECT,
  origin: (i32, i32),
  size: (i32, i32),
) -> (RECT, RECT) {
  let mut src = RECT {
    left: src.left - origin.0,
    top: src.top - origin.1,
    right: src.right - origin.0,
    bottom: src.bottom - origin.1,
  };
  let mut dst = dst;

  let src_width = src.right - src.left;
  let src_height = src.bottom - src.top;

  if src_width <= 0 || src_height <= 0 {
    return (src, dst);
  }

  let scale_x = f64::from(dst.right - dst.left) / f64::from(src_width);
  let scale_y = f64::from(dst.bottom - dst.top) / f64::from(src_height);

  #[allow(clippy::cast_possible_truncation)]
  let scaled =
    |pixels: i32, scale: f64| (f64::from(pixels) * scale).round() as i32;

  let clip_left = (-src.left).max(0);
  let clip_top = (-src.top).max(0);
  let clip_right = (src.right - size.0).max(0);
  let clip_bottom = (src.bottom - size.1).max(0);

  src.left += clip_left;
  src.top += clip_top;
  src.right -= clip_right;
  src.bottom -= clip_bottom;

  dst.left += scaled(clip_left, scale_x);
  dst.top += scaled(clip_top, scale_y);
  dst.right -= scaled(clip_right, scale_x);
  dst.bottom -= scaled(clip_bottom, scale_y);

  (src, dst)
}

/// Whether `upper` is anywhere above `lower` in the z-order.
fn is_above(upper: HWND, lower: HWND) -> bool {
  let mut below = upper;

  for _ in 0..MAX_Z_ORDER_WALK {
    // SAFETY: A stale handle just makes `GetWindow` return `HWND(0)`.
    below = unsafe { GetWindow(below, GW_HWNDNEXT) };
    if below.0 == 0 {
      return false;
    }
    if below == lower {
      return true;
    }
  }

  false
}

/// Where `window`'s visible frame starts within its own rect: past its
/// invisible left and top resize borders.
fn frame_origin(window: HWND) -> Option<(i32, i32)> {
  let (bounds, frame) = window_class::bounds_and_frame(window)?;
  Some((frame.left - bounds.left, frame.top - bounds.top))
}

/// The window `hwnd` is a companion of, per its property.
fn companion_of(hwnd: HWND) -> Option<HWND> {
  // SAFETY: A stale handle or a missing property just returns 0.
  let value = unsafe { GetPropW(hwnd, COMPANION_PROPERTY) };
  (value.0 != 0).then_some(HWND(value.0))
}

/// Every shown top-level window carrying the companion property, as
/// `(window, companion)`.
fn scan() -> Vec<(isize, isize)> {
  unsafe extern "system" fn visit(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: `lparam` is the `Vec` passed to `EnumWindows` below, which
    // outlives the enumeration and is only accessed from this callback.
    let pairs = unsafe { &mut *(lparam.0 as *mut Vec<(isize, isize)>) };

    // SAFETY: `hwnd` comes from the enumeration.
    if unsafe { IsWindowVisible(hwnd) }.as_bool() {
      if let Some(source) = companion_of(hwnd) {
        pairs.push((source.0, hwnd.0));
      }
    }

    TRUE
  }

  let mut pairs: Vec<(isize, isize)> = Vec::new();

  // SAFETY: `visit` only dereferences `lparam` as the `Vec` above, which
  // lives until `EnumWindows` returns.
  let _ = unsafe {
    EnumWindows(
      Some(visit),
      LPARAM(std::ptr::from_mut(&mut pairs) as isize),
    )
  };

  pairs
}

#[cfg(test)]
mod tests {
  use windows::Win32::Foundation::RECT;

  use super::{map_rects, HandoffWait};

  #[test]
  fn hands_back_a_call_after_the_companion_covers() {
    let mut wait = HandoffWait::default();
    assert!(wait.awaits_cover(false));
    assert!(wait.awaits_cover(true));
    assert!(!wait.awaits_cover(true));
  }

  #[test]
  fn waits_a_call_for_a_companion_covering_from_the_start() {
    let mut wait = HandoffWait::default();
    assert!(wait.awaits_cover(true));
    assert!(!wait.awaits_cover(true));
  }

  #[test]
  fn waits_again_for_a_companion_that_stops_covering() {
    let mut wait = HandoffWait::default();
    assert!(wait.awaits_cover(true));
    assert!(wait.awaits_cover(false));
    assert!(wait.awaits_cover(true));
    assert!(!wait.awaits_cover(true));
  }

  fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
      left,
      top,
      right,
      bottom,
    }
  }

  #[test]
  fn shifts_source_past_invisible_border() {
    let (src, dst) = map_rects(
      rect(7, 0, 807, 600),
      rect(0, 0, 800, 600),
      (7, 0),
      (800, 600),
    );
    assert_eq!(src, rect(0, 0, 800, 600));
    assert_eq!(dst, rect(0, 0, 800, 600));
  }

  #[test]
  fn clips_source_outside_companion() {
    // Samples the invisible border left of the frame, and 100px past a
    // companion that hasn't grown yet.
    let (src, dst) = map_rects(
      rect(0, 0, 907, 600),
      rect(0, 0, 907, 600),
      (7, 0),
      (800, 600),
    );
    assert_eq!(src, rect(0, 0, 800, 600));
    assert_eq!(dst, rect(7, 0, 807, 600));
  }

  #[test]
  fn clips_destination_proportionally_when_scaled() {
    // Half-size zoom: 200 source pixels past the companion are 100 on
    // screen.
    let (src, dst) = map_rects(
      rect(0, 0, 1000, 600),
      rect(0, 0, 500, 300),
      (0, 0),
      (800, 600),
    );
    assert_eq!(src, rect(0, 0, 800, 600));
    assert_eq!(dst, rect(0, 0, 400, 300));
  }
}
