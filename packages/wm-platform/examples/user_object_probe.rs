//! Diagnostic: which animation-path primitive leaks USER objects?
//!
//! Measures `GetGuiResources(GR_USEROBJECTS)` around each Win32 primitive the
//! per-frame animation path uses, in an isolated process, so a leak's type
//! can be identified without touching a live `GlazeWM`.
//!
//! Findings that motivated `SurrogateBatch::commit`'s flag split (Windows 11
//! 26200):
//!
//! - `DeferWindowPos` fails with `ERROR_INVALID_PARAMETER` when passed
//!   `SWP_NOSENDCHANGING`, even though `SetWindowPos` accepts it and the
//!   documentation lists it for both.
//! - An `HDWP` from `BeginDeferWindowPos` that is never passed to
//!   `EndDeferWindowPos` leaks exactly one USER object, permanently.
//!   Following `DeferWindowPos`'s documented "abandon the operation" advice
//!   is what leaks; calling `EndDeferWindowPos` on the handle after a failed
//!   `DeferWindowPos` releases it cleanly.
//! - `HTHUMBNAIL` handles from `DwmRegisterThumbnail` are *not* USER
//!   objects: leaking 500 of them moves the count by zero.
//!
//! Run with `cargo run -p wm-platform --release --example user_object_probe`.

#![cfg(target_os = "windows")]
#![allow(clippy::print_stdout, clippy::as_conversions)]

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Dwm::{
      DwmRegisterThumbnail, DwmUnregisterThumbnail,
      DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES,
      DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE, DWM_TNP_VISIBLE,
    },
    System::Threading::{GetCurrentProcess, GetGuiResources, GR_USEROBJECTS},
    UI::WindowsAndMessaging::{
      BeginDeferWindowPos, CreateWindowExW, DeferWindowPos, DefWindowProcW,
      DestroyWindow, EndDeferWindowPos, EnumWindows,
      GetWindowRect, GetWindowTextLengthW, IsWindowVisible, RegisterClassW,
      SetWindowPos, SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE, SWP_NOCOPYBITS,
      SWP_NOSENDCHANGING, SWP_NOZORDER, WNDCLASSW, WS_EX_NOACTIVATE,
      WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

/// Returns this process's current USER object count.
fn user_objects() -> u32 {
  // SAFETY: The pseudo-handle from `GetCurrentProcess` is always valid.
  unsafe { GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS) }
}

/// Window procedure for the probe's own overlay windows.
unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: Forwarding to the default handler is always valid.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Collects visible top-level windows to use as thumbnail sources.
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

/// Creates `count` hidden popup windows used as batch/thumbnail targets.
fn create_windows(count: usize) -> Vec<HWND> {
  let class = w!("GlazeWM_UserProbe");
  let wnd_class = WNDCLASSW {
    lpfnWndProc: Some(wnd_proc),
    lpszClassName: class,
    ..Default::default()
  };
  // SAFETY: `wnd_class` is fully initialized and lives for the call.
  unsafe { RegisterClassW(&raw const wnd_class) };

  (0..count)
    .map(|i| {
      // SAFETY: The class was registered above; all arguments are valid.
      unsafe {
        CreateWindowExW(
          WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
          class,
          w!(""),
          WS_POPUP,
          -4000,
          100 + (i as i32) * 10,
          400,
          300,
          None,
          None,
          None,
          None,
        )
      }
    })
    .collect()
}

/// Runs `iterations` of a `BeginDeferWindowPos`/`DeferWindowPos`/
/// `EndDeferWindowPos` transaction over `windows`, mirroring
/// `SurrogateBatch::commit`'s happy path.
fn probe_deferwindowpos(windows: &[HWND], iterations: usize) {
  for i in 0..iterations {
    // SAFETY: All handles are live windows owned by this process.
    unsafe {
      let Ok(mut hdwp) = BeginDeferWindowPos(windows.len() as i32) else {
        println!("  BeginDeferWindowPos failed at iteration {i}.");
        return;
      };
      for (n, hwnd) in windows.iter().enumerate() {
        match DeferWindowPos(
          hdwp,
          *hwnd,
          HWND(0),
          -4000,
          100 + (n as i32) * 10 + (i % 2) as i32,
          400,
          300,
          SWP_NOACTIVATE
            | SWP_NOCOPYBITS
            | SWP_NOSENDCHANGING
            | SWP_NOZORDER,
        ) {
          Ok(next) => hdwp = next,
          Err(err) => {
            println!("  DeferWindowPos failed at iteration {i}: {err}.");
            return;
          }
        }
      }
      if EndDeferWindowPos(hdwp).is_err() {
        println!("  EndDeferWindowPos failed at iteration {i}.");
      }
    }
  }
}

/// Runs `iterations` of `BeginDeferWindowPos` that are abandoned without
/// `EndDeferWindowPos`, to measure whether an unclosed `HDWP` leaks.
fn probe_abandoned_hdwp(iterations: usize) {
  for _ in 0..iterations {
    // SAFETY: No preconditions; the handle is deliberately abandoned.
    unsafe {
      let _ = BeginDeferWindowPos(4);
    }
  }
}

/// Runs `iterations` transactions that fail on the first `DeferWindowPos`
/// but still call `EndDeferWindowPos` on the handle, to check whether that
/// releases the `HDWP` the failed transaction left behind.
fn probe_end_after_failure(windows: &[HWND], iterations: usize) {
  for _ in 0..iterations {
    // SAFETY: All handles are live windows owned by this process.
    unsafe {
      let Ok(hdwp) = BeginDeferWindowPos(windows.len() as i32) else {
        return;
      };
      let failed = DeferWindowPos(
        hdwp,
        windows[0],
        HWND(0),
        -4000,
        100,
        400,
        300,
        SWP_NOACTIVATE | SWP_NOSENDCHANGING,
      )
      .is_err();
      assert!(failed, "expected SWP_NOSENDCHANGING to be rejected");
      let _ = EndDeferWindowPos(hdwp);
    }
  }
}

/// Runs `iterations` of the batch transaction with the corrected flag set
/// (no `SWP_NOSENDCHANGING`), to confirm it succeeds and leaks nothing.
fn probe_fixed_batch(windows: &[HWND], iterations: usize) {
  for i in 0..iterations {
    // SAFETY: All handles are live windows owned by this process.
    unsafe {
      let Ok(mut hdwp) = BeginDeferWindowPos(windows.len() as i32) else {
        return;
      };
      let mut ok = true;
      for (n, hwnd) in windows.iter().enumerate() {
        match DeferWindowPos(
          hdwp,
          *hwnd,
          HWND(0),
          -4000,
          100 + (n as i32) * 10 + (i % 2) as i32,
          400,
          300,
          SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOZORDER,
        ) {
          Ok(next) => hdwp = next,
          Err(err) => {
            println!("  fixed batch Defer failed at {i}: {err}.");
            ok = false;
            break;
          }
        }
      }
      let _ = EndDeferWindowPos(hdwp);
      if !ok {
        return;
      }
    }
  }
}

/// Runs `iterations` of plain per-window `SetWindowPos` calls, the fallback
/// path in `SurrogateBatch::commit_individually`.
fn probe_setwindowpos(windows: &[HWND], iterations: usize) {
  for i in 0..iterations {
    for (n, hwnd) in windows.iter().enumerate() {
      // SAFETY: All handles are live windows owned by this process.
      unsafe {
        let _ = SetWindowPos(
          *hwnd,
          HWND(0),
          -4000,
          100 + (n as i32) * 10 + (i % 2) as i32,
          400,
          300,
          SWP_NOACTIVATE
            | SWP_NOCOPYBITS
            | SWP_NOSENDCHANGING
            | SWP_NOZORDER,
        );
      }
    }
  }
}

/// Registers and unregisters a DWM thumbnail `iterations` times, mirroring
/// `NativeSurrogate::reregister_thumbnail`.
fn probe_thumbnail_cycle(dest: HWND, source: HWND, iterations: usize) {
  for _ in 0..iterations {
    // SAFETY: Both handles are live top-level windows.
    unsafe {
      if let Ok(thumb) = DwmRegisterThumbnail(dest, source) {
        let _ = DwmUnregisterThumbnail(thumb);
      }
    }
  }
}

/// Registers `iterations` DWM thumbnails without unregistering any, to
/// measure whether a leaked `HTHUMBNAIL` shows up in the USER count.
fn probe_thumbnail_leak(dest: HWND, source: HWND, iterations: usize) {
  for _ in 0..iterations {
    // SAFETY: Both handles are live top-level windows.
    unsafe {
      let _ = DwmRegisterThumbnail(dest, source);
    }
  }
}

/// Updates a single registered thumbnail's rects `iterations` times, the
/// per-frame call made by `NativeSurrogate::set_thumbnail_rects`.
fn probe_thumbnail_update(dest: HWND, source: HWND, iterations: usize) {
  // SAFETY: Both handles are live top-level windows.
  let Ok(thumb) = (unsafe { DwmRegisterThumbnail(dest, source) }) else {
    println!("  DwmRegisterThumbnail failed; skipping update probe.");
    return;
  };
  for i in 0..iterations {
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTDESTINATION
        | DWM_TNP_RECTSOURCE
        | DWM_TNP_VISIBLE,
      rcDestination: RECT {
        left: 0,
        top: 0,
        right: 400,
        bottom: 300 - (i % 2) as i32,
      },
      rcSource: RECT {
        left: 0,
        top: 0,
        right: 400,
        bottom: 300 - (i % 2) as i32,
      },
      fVisible: true.into(),
      ..Default::default()
    };
    // SAFETY: `thumb` is a live thumbnail; `props` is stack-allocated.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(thumb, &raw const props);
    }
  }
  // SAFETY: `thumb` is a live thumbnail registered above.
  unsafe {
    let _ = DwmUnregisterThumbnail(thumb);
  }
}

/// Tries a single `BeginDeferWindowPos`/`DeferWindowPos` pair with `flags`
/// and reports whether `DeferWindowPos` succeeded, plus the USER-object
/// delta left behind.
fn probe_flags(label: &str, windows: &[HWND], flags: SET_WINDOW_POS_FLAGS) {
  let before = user_objects();
  // SAFETY: All handles are live windows owned by this process.
  let outcome = unsafe {
    match BeginDeferWindowPos(windows.len() as i32) {
      Err(err) => format!("Begin failed: {err}"),
      Ok(mut hdwp) => {
        let mut result = String::from("ok");
        for (n, hwnd) in windows.iter().enumerate() {
          match DeferWindowPos(
            hdwp,
            *hwnd,
            HWND(0),
            -4000,
            100 + (n as i32) * 10,
            400,
            300,
            flags,
          ) {
            Ok(next) => hdwp = next,
            Err(err) => {
              result = format!("Defer#{n} failed: {err}");
              break;
            }
          }
        }
        if result == "ok" {
          match EndDeferWindowPos(hdwp) {
            Ok(()) => result = String::from("ok"),
            Err(err) => result = format!("End failed: {err}"),
          }
        }
        result
      }
    }
  };
  let after = user_objects();
  println!(
    "  {label:<40} {outcome:<40} delta={}",
    i64::from(after) - i64::from(before)
  );
}

/// Prints the USER-object delta for `label` around running `f`.
fn measure(label: &str, f: impl FnOnce()) {
  let before = user_objects();
  f();
  let after = user_objects();
  println!(
    "{label:<44} before={before:>6} after={after:>6} delta={:>6}",
    i64::from(after) - i64::from(before)
  );
}

fn main() {
  let mut sources: Vec<HWND> = Vec::new();
  // SAFETY: `collect` only writes through the `Vec<HWND>` pointer passed in.
  unsafe {
    let _ = EnumWindows(
      Some(collect),
      LPARAM(std::ptr::from_mut(&mut sources) as isize),
    );
  }

  let windows = create_windows(4);
  println!("Created {} probe windows.", windows.len());
  println!("Found {} thumbnail source windows.\n", sources.len());

  println!("-- flag variations --");
  probe_flags("no flags", &windows, SET_WINDOW_POS_FLAGS(0));
  probe_flags("NOACTIVATE", &windows, SWP_NOACTIVATE);
  probe_flags("NOZORDER", &windows, SWP_NOZORDER);
  probe_flags("NOCOPYBITS", &windows, SWP_NOCOPYBITS);
  probe_flags("NOSENDCHANGING", &windows, SWP_NOSENDCHANGING);
  probe_flags(
    "NOACTIVATE|NOZORDER",
    &windows,
    SWP_NOACTIVATE | SWP_NOZORDER,
  );
  probe_flags(
    "all four (GlazeWM batch flags)",
    &windows,
    SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING | SWP_NOZORDER,
  );
  probe_flags(
    "single window, all four",
    &windows[..1],
    SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING | SWP_NOZORDER,
  );
  println!();

  measure("regressed batch (NOSENDCHANGING) x2000", || {
    probe_deferwindowpos(&windows, 2000);
  });
  measure("EndDeferWindowPos after failure x2000", || {
    probe_end_after_failure(&windows, 2000);
  });
  measure("fixed batch (no NOSENDCHANGING) x2000", || {
    probe_fixed_batch(&windows, 2000);
  });
  measure("SetWindowPos x2000x4", || {
    probe_setwindowpos(&windows, 2000);
  });
  measure("BeginDeferWindowPos abandoned x2000", || {
    probe_abandoned_hdwp(2000);
  });

  if let Some(&source) = sources.first() {
    let dest = windows[0];
    measure("DwmRegisterThumbnail+Unregister x500", || {
      probe_thumbnail_cycle(dest, source, 500);
    });
    measure("DwmUpdateThumbnailProperties x2000", || {
      probe_thumbnail_update(dest, source, 2000);
    });
    measure("DwmRegisterThumbnail leaked x500", || {
      probe_thumbnail_leak(dest, source, 500);
    });
  }

  println!("\nFinal USER objects: {}", user_objects());

  for hwnd in windows {
    // SAFETY: Handles were created by this process and not yet destroyed.
    unsafe {
      let _ = DestroyWindow(hwnd);
    }
  }
}
