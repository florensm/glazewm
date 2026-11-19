use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
  time::{Duration, Instant},
};

use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC,
    DeleteObject, GetDC, GetPixel, ReleaseDC, SelectObject, HGDIOBJ,
    SRCCOPY,
  },
  UI::WindowsAndMessaging::{IsIconic, IsWindow, IsWindowVisible},
};

use crate::{
  native_surrogate::to_logical, platform_impl::NativeWindow, window_class,
  Color, Rect,
};

/// Pixels inset from the content edge when sampling the backdrop color.
///
/// Matches `EDGE_SAMPLE_INSET_PX` in `native_surrogate.rs` so both the
/// sampled backdrop and the (fallback) edge-extension thumbnails read from
/// the same source strip.
const EDGE_SAMPLE_INSET: i32 = 4;

/// Shared cache of recently sampled surrogate backdrop colors, keyed by
/// window handle. Mirrors the shape `AnimationManager` stores on the WM's
/// main thread; wrapped in `Arc<Mutex<_>>` so
/// [`sample_edge_color_async`]'s background thread can insert into the
/// same map the main thread reads from, without routing the result back
/// through the event loop.
pub type EdgeColorCache = Arc<Mutex<HashMap<isize, (Color, Instant)>>>;

/// Samples `hwnd`'s surrogate backdrop color on a background thread and
/// inserts it into `cache` once ready, instead of blocking the caller.
///
/// The two-`BitBlt` GPU->CPU readback takes 26-114ms, too long for the
/// WM's main thread -- and it stalls DWM while it runs, so it must not
/// overlap an animation either: sampled at a session's start, it tripled
/// keypress-to-motion latency. Callers sample once animations are idle,
/// for the window's *next* session. Reads the window's current rect, and
/// is a no-op if the window is gone, hidden, cloaked or minimized (its
/// screen pixels are something else) or sampling fails.
///
/// A sample is dropped if `animating` is set, or the window has moved, by
/// the time it is read: a stand-in or a border ring may then have crossed
/// the strips it reads, and a ring's flat color wins over content.
///
/// `prune_len`/`ttl` mirror the caller's own cache-eviction policy (e.g.
/// `AnimationManager`'s
/// `EDGE_COLOR_CACHE_PRUNE_LEN`/`EDGE_COLOR_CACHE_TTL`) -- pruning happens
/// here, on the background thread, rather than on the caller's next
/// synchronous insert.
pub fn sample_edge_color_async(
  hwnd: HWND,
  cache: EdgeColorCache,
  prune_len: usize,
  ttl: Duration,
  animating: Arc<AtomicBool>,
) {
  let hwnd_raw = hwnd.0;
  tokio::task::spawn_blocking(move || {
    let hwnd = HWND(hwnd_raw);
    let Some(source_rect) = sampleable_rect(hwnd) else {
      return;
    };
    let logical_src =
      to_logical(&source_rect, &compute_border_inset(hwnd));

    if let Some(color) = sample_edge_color(
      logical_src.x(),
      logical_src.y(),
      logical_src.width(),
      logical_src.height(),
    ) {
      if animating.load(Ordering::Relaxed)
        || sampleable_rect(hwnd).as_ref() != Some(&source_rect)
      {
        return;
      }
      if let Ok(mut map) = cache.lock() {
        if map.len() >= prune_len {
          map.retain(|_, (_, inserted_at)| inserted_at.elapsed() < ttl);
        }
        map.insert(hwnd_raw, (color, Instant::now()));
      }
    }
  });
}

/// Returns `hwnd`'s window rect if its pixels are on screen to sample.
fn sampleable_rect(hwnd: HWND) -> Option<Rect> {
  // SAFETY: Every call only reads state of `hwnd`; a stale handle fails.
  unsafe {
    if !IsWindow(hwnd).as_bool()
      || !IsWindowVisible(hwnd).as_bool()
      || IsIconic(hwnd).as_bool()
    {
      return None;
    }

    if NativeWindow::new(hwnd.0).is_cloaked().unwrap_or(true) {
      return None;
    }
  }

  window_rect(hwnd.0)
}

/// A window's `GetWindowRect`, or `None` if its handle went stale.
fn window_rect(hwnd: isize) -> Option<Rect> {
  NativeWindow::new(hwnd).frame_with_shadows().ok()
}

/// Pixels sampled along each edge strip by [`sample_edge_color`].
const EDGE_SAMPLES: i32 = 32;

/// Samples the dominant background color near the trailing content edge by
/// `BitBlt`-ing two narrow strips from the already-composited screen.
///
/// Samples 32 evenly-spaced pixels along the right-edge column and 32
/// along the bottom-edge row, both at `EDGE_SAMPLE_INSET` px inward from
/// the content boundary (matching the edge-extension thumbnail source).
/// Returns `None` when GDI handle creation fails.
///
/// `content_screen_left` and `content_screen_top` are the screen
/// coordinates of the content area's top-left corner (physical rect
/// left/top plus the invisible border insets). Reading from the
/// DWM-composited screen rather than via `PrintWindow` avoids allocating a
/// full-resolution bitmap and forcing a GPU→CPU flush, which is
/// proportional to window area and becomes expensive on large displays.
fn sample_edge_color(
  content_screen_left: i32,
  content_screen_top: i32,
  content_w: i32,
  content_h: i32,
) -> Option<crate::Color> {
  if content_w <= EDGE_SAMPLE_INSET + 1
    || content_h <= EDGE_SAMPLE_INSET + 1
  {
    return None;
  }

  // Screen coordinates of the right-edge column and bottom-edge row.
  let screen_x = content_screen_left + content_w - EDGE_SAMPLE_INSET - 1;
  let screen_y = content_screen_top + content_h - EDGE_SAMPLE_INSET - 1;

  // Sample the middle half of each edge to avoid corners (rounded-corner
  // antialiasing) and the title-bar region.
  let y0 = content_screen_top + content_h / 4;
  let y1 = content_screen_top + (3 * content_h) / 4;
  let x0 = content_screen_left + content_w / 4;
  let x1 = content_screen_left + (3 * content_w) / 4;

  let strip_h = y1 - y0;
  let strip_w = x1 - x0;
  if strip_h <= 0 || strip_w <= 0 {
    return None;
  }

  // SAFETY: A null HWND argument to GetDC returns the screen DC.
  let hdc_screen = unsafe { GetDC(HWND(0)) };
  if hdc_screen.is_invalid() {
    return None;
  }

  // Right-edge strip: 1 px wide × strip_h px tall.
  // SAFETY: hdc_screen is a valid DC; dimensions are positive.
  let hdc_right = unsafe { CreateCompatibleDC(hdc_screen) };
  let hbm_right =
    unsafe { CreateCompatibleBitmap(hdc_screen, 1, strip_h) };

  if hdc_right.is_invalid() || hbm_right.is_invalid() {
    unsafe {
      if !hdc_right.is_invalid() {
        DeleteDC(hdc_right);
      }
      if !hbm_right.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(hbm_right.0));
      }
      ReleaseDC(HWND(0), hdc_screen);
    }
    return None;
  }

  // SAFETY: Both handles are valid.
  let old_right = unsafe { SelectObject(hdc_right, HGDIOBJ(hbm_right.0)) };
  // SAFETY: All DCs and dimensions are valid; coordinates are in screen
  // space.
  unsafe {
    let _ = BitBlt(
      hdc_right, 0, 0, 1, strip_h, hdc_screen, screen_x, y0, SRCCOPY,
    );
  }

  // Bottom-edge strip: strip_w px wide × 1 px tall.
  // SAFETY: hdc_screen is a valid DC; dimensions are positive.
  let hdc_bottom = unsafe { CreateCompatibleDC(hdc_screen) };
  let hbm_bottom =
    unsafe { CreateCompatibleBitmap(hdc_screen, strip_w, 1) };

  if hdc_bottom.is_invalid() || hbm_bottom.is_invalid() {
    unsafe {
      SelectObject(hdc_right, old_right);
      DeleteDC(hdc_right);
      let _ = DeleteObject(HGDIOBJ(hbm_right.0));
      if !hdc_bottom.is_invalid() {
        DeleteDC(hdc_bottom);
      }
      if !hbm_bottom.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(hbm_bottom.0));
      }
      ReleaseDC(HWND(0), hdc_screen);
    }
    return None;
  }

  let old_bottom =
    unsafe { SelectObject(hdc_bottom, HGDIOBJ(hbm_bottom.0)) };
  // SAFETY: All DCs and dimensions are valid; coordinates are in screen
  // space.
  unsafe {
    let _ = BitBlt(
      hdc_bottom, 0, 0, strip_w, 1, hdc_screen, x0, screen_y, SRCCOPY,
    );
  }

  // SAFETY: The screen DC is no longer needed; hdc_right and hdc_bottom
  // are independent allocations that outlive this scope.
  unsafe { ReleaseDC(HWND(0), hdc_screen) };

  let mut pixels = Vec::with_capacity(2 * EDGE_SAMPLES as usize);

  for i in 0..EDGE_SAMPLES {
    let sy = (strip_h - 1) * i / (EDGE_SAMPLES - 1);
    // SAFETY: hdc_right is a valid DC with hbm_right selected.
    let c = unsafe { GetPixel(hdc_right, 0, sy) };
    if c.0 != 0xFFFF_FFFF {
      pixels.push(c.0);
    }
  }
  for i in 0..EDGE_SAMPLES {
    let sx = (strip_w - 1) * i / (EDGE_SAMPLES - 1);
    // SAFETY: hdc_bottom is a valid DC with hbm_bottom selected.
    let c = unsafe { GetPixel(hdc_bottom, sx, 0) };
    if c.0 != 0xFFFF_FFFF {
      pixels.push(c.0);
    }
  }

  // SAFETY: Restore selections before freeing so GDI holds no references
  // to deleted objects.
  unsafe {
    SelectObject(hdc_right, old_right);
    DeleteDC(hdc_right);
    let _ = DeleteObject(HGDIOBJ(hbm_right.0));
    SelectObject(hdc_bottom, old_bottom);
    DeleteDC(hdc_bottom);
    let _ = DeleteObject(HGDIOBJ(hbm_bottom.0));
  }

  dominant_color(&pixels)
}

/// Bits dropped from each channel to bucket sampled pixels by colour.
///
/// Steps of 32 absorb a see-through window's backdrop noise, which leaves
/// no two pixels alike, yet keep a ring's or another window's colour apart
/// from it; the bucket's mean then restores the precision.
const EDGE_COLOR_BUCKET_SHIFT: u32 = 5;

/// The mean of the most common colour bucket among `pixels`
/// (`COLORREF`s, `0x00BBGGRR`), or `None` for no pixels.
///
/// A tie goes to the bucket seen first.
fn dominant_color(pixels: &[u32]) -> Option<crate::Color> {
  // Per bucket: its key, member count and channel sums.
  let mut buckets: Vec<(u32, u32, [u32; 3])> = Vec::new();

  for &pixel in pixels {
    let channels =
      [pixel & 0xFF, (pixel >> 8) & 0xFF, (pixel >> 16) & 0xFF];
    let key = channels
      .iter()
      .fold(0, |key, c| (key << 8) | (c >> EDGE_COLOR_BUCKET_SHIFT));

    match buckets.iter_mut().find(|(k, ..)| *k == key) {
      Some((_, count, sums)) => {
        *count += 1;
        for (sum, c) in sums.iter_mut().zip(channels) {
          *sum += c;
        }
      }
      None => buckets.push((key, 1, channels)),
    }
  }

  let (_, count, sums) = buckets.into_iter().reduce(|best, bucket| {
    if bucket.1 > best.1 {
      bucket
    } else {
      best
    }
  })?;
  #[allow(clippy::cast_possible_truncation)]
  let mean = |sum: u32| ((sum + count / 2) / count) as u8;

  Some(crate::Color {
    r: mean(sums[0]),
    g: mean(sums[1]),
    b: mean(sums[2]),
    a: 255,
  })
}

/// Computes the invisible border insets of `hwnd` in physical pixels.
///
/// Windows adds a transparent resize border (~7 px on left, right, bottom;
/// none on top) outside the visible window frame. Compares `GetWindowRect`
/// with `DWMWA_EXTENDED_FRAME_BOUNDS` to obtain per-side inset values.
///
/// Returns a zeroed `RECT` if either API call fails.
pub(crate) fn compute_border_inset(hwnd: HWND) -> RECT {
  let Some((window, frame)) = window_class::bounds_and_frame(hwnd) else {
    return RECT::default();
  };

  RECT {
    left: (frame.left - window.left).max(0),
    top: (frame.top - window.top).max(0),
    right: (window.right - frame.right).max(0),
    bottom: (window.bottom - frame.bottom).max(0),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn colorref(r: u32, g: u32, b: u32) -> u32 {
    r | (g << 8) | (b << 16)
  }

  fn rgb(color: Color) -> (u8, u8, u8) {
    (color.r, color.g, color.b)
  }

  #[test]
  fn noisy_content_beats_a_smaller_flat_intruder() {
    // A see-through window's edge: no two pixels alike.
    let mut pixels: Vec<u32> = (0..40)
      .map(|i| colorref(40 + i % 7, 44 + i / 7, 52 + i % 5))
      .collect();
    // A border ring's flat colour crossing the strip.
    pixels.extend(std::iter::repeat_n(colorref(0, 200, 80), 24));

    let (r, g, b) =
      rgb(dominant_color(&pixels).expect("pixels were sampled"));
    assert!((40..47).contains(&r), "{r}");
    assert!((44..51).contains(&g), "{g}");
    assert!((52..59).contains(&b), "{b}");
  }

  #[test]
  fn a_clearly_dominant_color_wins_exactly() {
    let mut pixels = vec![colorref(30, 30, 30); 50];
    pixels.extend((0..14).map(|i| colorref(200, 10 * i, 255 - 10 * i)));

    assert_eq!(dominant_color(&pixels).map(rgb), Some((30, 30, 30)));
  }

  #[test]
  fn no_pixels_give_no_color() {
    assert!(dominant_color(&[]).is_none());
  }
}
