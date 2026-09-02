//! Microbenchmark: is moving N windows or updating N DWM thumbnails cheaper?
//!
//! The animation tick's dominant cost is `EndDeferWindowPos` moving one
//! surrogate window per animating window, every frame. The proposed
//! alternative is one monitor-sized surrogate window holding N thumbnails,
//! repositioned per frame with `DwmUpdateThumbnailProperties` and no window
//! moves at all. This measures both so the rewrite can be judged before it
//! is written.
//!
//! Run with `cargo run -p wm-platform --release --example surrogate_cost`.
//! Windows are created off to the side and destroyed on exit.

#![cfg(target_os = "windows")]

use std::time::{Duration, Instant};

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM, LRESULT, BOOL},
    Graphics::Dwm::{
      DwmExtendFrameIntoClientArea, DwmRegisterThumbnail,
      DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
      DWM_THUMBNAIL_PROPERTIES, DWM_TNP_RECTDESTINATION, DWM_TNP_SOURCECLIENTAREAONLY,
      DWM_TNP_VISIBLE,
    },
    UI::Controls::MARGINS,
    UI::WindowsAndMessaging::{
      BeginDeferWindowPos, CreateWindowExW, DeferWindowPos, DefWindowProcW,
      DestroyWindow, EndDeferWindowPos, EnumWindows, GetWindowRect,
      GetWindowTextLengthW, IsWindowVisible, RegisterClassW, ShowWindow,
      SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING,
      SWP_NOZORDER, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

/// Frames measured per scenario.
const FRAMES: usize = 300;

/// Collects visible top-level windows to use as realistic thumbnail sources.
unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
  // SAFETY: `lparam` is the `Vec<HWND>` passed by the caller below.
  let found = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
  // SAFETY: `hwnd` is a valid handle supplied by `EnumWindows`.
  unsafe {
    if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
      let mut r = RECT::default();
      if GetWindowRect(hwnd, &raw mut r).is_ok()
        && r.right - r.left > 200
        && r.bottom - r.top > 200
      {
        found.push(hwnd);
      }
    }
  }
  BOOL(1)
}

/// Minimal window procedure; these windows never handle a message.
unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: parameters forwarded unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Creates a bare popup window of the given geometry.
fn make_window(x: i32, y: i32, w: i32, h: i32) -> HWND {
  // SAFETY: static class name, registered once below; all params valid.
  unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
      w!("GlazeWM_CostBench"),
      w!(""),
      WS_POPUP,
      x,
      y,
      w,
      h,
      None,
      None,
      None,
      None,
    )
  }
}

/// Median of a set of per-frame durations.
fn median(mut v: Vec<Duration>) -> Duration {
  v.sort_unstable();
  v[v.len() / 2]
}

fn main() {
  // SAFETY: a single static class registered once for this process.
  unsafe {
    let class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_CostBench"),
      lpfnWndProc: Some(wnd_proc),
      ..Default::default()
    };
    RegisterClassW(&raw const class);
  }

  let mut sources: Vec<HWND> = Vec::new();
  // SAFETY: `collect` only appends to the vector behind this pointer.
  unsafe {
    let _ = EnumWindows(
      Some(collect),
      LPARAM(std::ptr::from_mut(&mut sources) as isize),
    );
  }

  for n in [4usize, 8, 12] {
    if sources.len() < n {
      println!("only {} source windows; skipping n={n}", sources.len());
      continue;
    }

    // --- A: today's design. N surrogate-alike windows, each carrying a DWM
    // thumbnail of a real window and an extended glass frame, moved *and
    // resized* every frame -- a resize animation changes the surrogate's
    // size on every tick, which is far more work for the compositor than
    // the pure translation an earlier version of this benchmark measured.
    let windows: Vec<HWND> = (0..n)
      .map(|i| {
        #[allow(clippy::cast_possible_truncation)]
        let hwnd = make_window(100 + (i as i32) * 40, 100, 900, 700);
        // SAFETY: `hwnd` was just created.
        unsafe {
          let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
          };
          let _ = DwmExtendFrameIntoClientArea(hwnd, &raw const margins);
          let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        hwnd
      })
      .collect();

    // One thumbnail per surrogate, as the real thing has.
    let mut a_thumbs = Vec::with_capacity(n);
    for (hwnd, src) in windows.iter().zip(sources.iter()) {
      // SAFETY: both are valid top-level windows.
      if let Ok(t) = unsafe { DwmRegisterThumbnail(*hwnd, *src) } {
        let props = DWM_THUMBNAIL_PROPERTIES {
          dwFlags: DWM_TNP_RECTDESTINATION
            | DWM_TNP_VISIBLE
            | DWM_TNP_SOURCECLIENTAREAONLY,
          rcDestination: RECT { left: 0, top: 0, right: 900, bottom: 700 },
          fVisible: BOOL(1),
          fSourceClientAreaOnly: BOOL(0),
          ..Default::default()
        };
        // SAFETY: `t` was just registered.
        unsafe { let _ = DwmUpdateThumbnailProperties(t, &raw const props); }
        a_thumbs.push(t);
      }
    }

    let mut a = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
      // Sweep the width like a resize animation does, rather than nudging
      // by a pixel at constant size.
      #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
      let w = 600 + ((frame % 60) as i32) * 10;
      let start = Instant::now();
      // SAFETY: all handles are windows created just above.
      unsafe {
        if let Ok(mut hdwp) = BeginDeferWindowPos(n as i32) {
          for (i, hwnd) in windows.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let x = 100 + (i as i32) * 40;
            match DeferWindowPos(
              hdwp, *hwnd, HWND(0), x, 100, w, 700,
              SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING | SWP_NOZORDER,
            ) {
              Ok(next) => hdwp = next,
              Err(_) => break,
            }
          }
          let _ = EndDeferWindowPos(hdwp);
        }
      }
      a.push(start.elapsed());
    }
    for t in &a_thumbs {
      // SAFETY: registered above.
      unsafe { let _ = DwmUnregisterThumbnail(*t); }
    }
    for hwnd in &windows {
      // SAFETY: created above and not yet destroyed.
      unsafe { let _ = DestroyWindow(*hwnd); }
    }

    // --- B: one host window, N thumbnails, no window moves. ---
    let host = make_window(100, 100, 3000, 1400);
    // SAFETY: `host` was just created.
    unsafe { let _ = ShowWindow(host, SW_SHOWNOACTIVATE); }

    let mut thumbs = Vec::with_capacity(n);
    for src in sources.iter().take(n) {
      // SAFETY: `host` and `*src` are valid top-level windows.
      if let Ok(t) = unsafe { DwmRegisterThumbnail(host, *src) } {
        thumbs.push(t);
      }
    }

    let mut b = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
      #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
      let w = 600 + ((frame % 60) as i32) * 10;
      let start = Instant::now();
      for (i, t) in thumbs.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let x = (i as i32) * 40;
        let props = DWM_THUMBNAIL_PROPERTIES {
          dwFlags: DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE,
          rcDestination: RECT { left: x, top: 0, right: x + w, bottom: 700 },
          fVisible: BOOL(1),
          ..Default::default()
        };
        // SAFETY: `*t` is a thumbnail registered just above.
        unsafe { let _ = DwmUpdateThumbnailProperties(*t, &raw const props); }
      }
      b.push(start.elapsed());
    }

    for t in &thumbs {
      // SAFETY: registered above, not yet unregistered.
      unsafe { let _ = DwmUnregisterThumbnail(*t); }
    }
    // SAFETY: created above; thumbnails already unregistered.
    unsafe { let _ = DestroyWindow(host); }

    let ma = median(a);
    let mb = median(b);
    println!(
      "n={n:2}  move {n} windows: {:>7.3} ms/frame ({:>6.3} ms each)   |   \
       update {} thumbnails: {:>7.3} ms/frame ({:>6.3} ms each)",
      ma.as_secs_f64() * 1000.0,
      ma.as_secs_f64() * 1000.0 / n as f64,
      thumbs.len(),
      mb.as_secs_f64() * 1000.0,
      mb.as_secs_f64() * 1000.0 / thumbs.len().max(1) as f64,
    );
  }
}
