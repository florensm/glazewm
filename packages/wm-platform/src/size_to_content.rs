//! Turns off WPF's `SizeToContent` on windows the WM sizes.
//!
//! A WPF window with `SizeToContent` other than `Manual` keeps its size in
//! `WM_WINDOWPOSCHANGING`, which the WM's `SWP_NOSENDCHANGING` skips: its
//! HWND takes the WM's size, but WPF doesn't lay its content out to it
//! (`HwndSource.Process_WM_SIZE`), leaving the rest unpainted. WPF turns
//! `SizeToContent` off and lays out to the HWND's size on the `WM_SIZING`
//! that starts a user's resize (`HwndSource.DisableSizeToContent`), and
//! then follows every later resize, so the WM sends it one.
//!
//! Its `RECT` is marshaled across processes by the system, so it must be
//! sent, not posted. It is sent from a worker thread with
//! `SMTO_ABORTIFHUNG`, as for `window_icons`, so a busy app never stalls
//! the WM's main loop.

use std::{
  sync::{
    mpsc::{self, Sender},
    Mutex, OnceLock, PoisonError,
  },
  thread,
};

use windows::Win32::{
  Foundation::{HWND, LPARAM, RECT, WPARAM},
  UI::WindowsAndMessaging::{
    GetWindowRect, SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_BLOCK,
    WMSZ_BOTTOMRIGHT, WM_SIZING,
  },
};

/// How long an app gets to answer. Long, as it doesn't hold up the WM, and
/// windows that get it have often just opened, while their app is busy.
const SIZING_TIMEOUT_MS: u32 = 3000;

/// Class name prefix of WPF's top-level windows
/// (`HwndWrapper[app;;guid]`).
const WPF_CLASS_PREFIX: &str = "HwndWrapper[";

/// Whether a window of class `class_name` is a WPF top-level window.
pub(crate) fn is_wpf_class(class_name: &str) -> bool {
  class_name.starts_with(WPF_CLASS_PREFIX)
}

fn worker() -> Option<&'static Mutex<Sender<isize>>> {
  static WORKER: OnceLock<Option<Mutex<Sender<isize>>>> = OnceLock::new();

  WORKER
    .get_or_init(|| {
      let (sender, receiver) = mpsc::channel::<isize>();

      let spawned = thread::Builder::new()
        .name("size-to-content".to_string())
        .spawn(move || {
          for window in receiver {
            send_sizing(HWND(window));
          }
        });

      match spawned {
        Ok(_) => Some(Mutex::new(sender)),
        Err(err) => {
          tracing::warn!("Failed to start SizeToContent thread: {err}");
          None
        }
      }
    })
    .as_ref()
}

/// Queues `WM_SIZING` for `window`. Returns whether it was queued.
pub(crate) fn release(window: isize) -> bool {
  worker().is_some_and(|worker| {
    worker
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .send(window)
      .is_ok()
  })
}

/// Sends `window` a `WM_SIZING` for its bottom-right corner, at the rect
/// it has now.
fn send_sizing(window: HWND) {
  let mut rect = RECT::default();

  // SAFETY: `rect` outlives the call; a stale handle just fails.
  if unsafe { GetWindowRect(window, &raw mut rect) }.is_err() {
    return;
  }

  let mut result = 0usize;

  // SAFETY: `rect` outlives the call, and the system copies it to and
  // from the window's process. A hung or slow app makes the call fail
  // after `SIZING_TIMEOUT_MS` instead of blocking.
  let answered = unsafe {
    SendMessageTimeoutW(
      window,
      WM_SIZING,
      WPARAM(WMSZ_BOTTOMRIGHT as usize),
      LPARAM(std::ptr::from_mut(&mut rect) as isize),
      SMTO_ABORTIFHUNG | SMTO_BLOCK,
      SIZING_TIMEOUT_MS,
      Some(&raw mut result),
    )
  };

  if answered.0 == 0 {
    tracing::debug!(
      "Window {:#x} didn't answer WM_SIZING; its SizeToContent stays on.",
      window.0
    );
  }
}

#[cfg(test)]
mod tests {
  use super::is_wpf_class;

  #[test]
  fn matches_wpf_top_level_classes() {
    assert!(is_wpf_class(
      "HwndWrapper[MyApp;;0b8f3c4e-5d2a-4f1b-9c7e-1a2b3c4d5e6f]"
    ));
    assert!(is_wpf_class("HwndWrapper[]"));
  }

  #[test]
  fn ignores_other_classes() {
    assert!(!is_wpf_class("HwndWrapper"));
    assert!(!is_wpf_class("Chrome_WidgetWin_1"));
    assert!(!is_wpf_class("#32770"));
    assert!(!is_wpf_class(""));
  }
}
