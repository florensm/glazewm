use std::{
  sync::{Mutex, PoisonError},
  time::{Duration, Instant},
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
    UI::WindowsAndMessaging::{
      EnumWindows, GetPropW, GetWindowRect, IsWindowVisible,
    },
  },
};

use crate::platform_impl::NativeWindow;

/// Window property another app sets on its own overlay window to declare
/// it the companion of the window it draws over (the value is that
/// window's handle), e.g. recolor's recolored copy of a window.
///
/// The contract: the companion covers exactly the window's DWM extended
/// frame bounds, and while the window is cloaked the companion stays shown
/// but cloaked, so DWM keeps rendering it for thumbnails. Animations then
/// show the companion in the window's place.
const COMPANION_PROPERTY: PCWSTR = w!("GlazeWM.CompanionOf");

/// How long a scan for companions is reused. A workspace switch creates a
/// surrogate per window in one tick, which then costs one `EnumWindows`
/// instead of one each; a companion appearing within this window is missed
/// for the animation, which then shows the window itself.
const SNAPSHOT_TTL: Duration = Duration::from_millis(50);

/// Every `(window, companion)` pair from a scan, and when it ran.
type Snapshot = (Instant, Vec<(isize, isize)>);

static SNAPSHOT: Mutex<Option<Snapshot>> = Mutex::new(None);

/// A companion window (see [`COMPANION_PROPERTY`]) of a managed window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Companion {
  hwnd: isize,

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
      origin: frame_origin(window)?,
    })
  }

  pub(crate) fn hwnd(self) -> HWND {
    HWND(self.hwnd)
  }

  /// Whether the companion is not on screen yet: hidden or cloaked.
  pub(crate) fn is_off_screen(self) -> bool {
    // SAFETY: A stale handle just returns false.
    let is_visible = unsafe { IsWindowVisible(self.hwnd()) }.as_bool();
    !is_visible
      || NativeWindow::new(self.hwnd).is_cloaked().unwrap_or(false)
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

/// Where `window`'s visible frame starts within its own rect: past its
/// invisible left and top resize borders.
fn frame_origin(window: HWND) -> Option<(i32, i32)> {
  let mut bounds = RECT::default();
  let mut frame = RECT::default();

  // SAFETY: Both rects outlive the calls and match the queried sizes; a
  // stale handle just makes them fail.
  unsafe {
    GetWindowRect(window, &raw mut bounds).ok()?;
    DwmGetWindowAttribute(
      window,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut frame).cast(),
      u32::try_from(std::mem::size_of::<RECT>()).ok()?,
    )
    .ok()?;
  }

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

  use super::map_rects;

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
