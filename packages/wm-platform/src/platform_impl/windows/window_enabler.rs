//! Re-enables windows off the calling thread.
//!
//! `EnableWindow` sends `WM_ENABLE` to the window and waits until its app
//! has handled it, so calling it from the WM thread would freeze the WM
//! for as long as the app is busy or hung.

use std::{
  sync::{
    mpsc::{self, Sender},
    Mutex, OnceLock, PoisonError,
  },
  thread,
};

use windows::Win32::{
  Foundation::HWND, UI::Input::KeyboardAndMouse::EnableWindow,
};

fn worker() -> Option<&'static Mutex<Sender<isize>>> {
  static WORKER: OnceLock<Option<Mutex<Sender<isize>>>> = OnceLock::new();

  WORKER
    .get_or_init(|| {
      let (sender, receiver) = mpsc::channel::<isize>();

      let spawned = thread::Builder::new()
        .name("window-enabler".to_string())
        .spawn(move || {
          for handle in receiver {
            // SAFETY: A stale handle just makes the call fail.
            unsafe {
              let _ = EnableWindow(HWND(handle), true);
            }
          }
        });

      match spawned {
        Ok(_) => Some(Mutex::new(sender)),
        Err(err) => {
          tracing::warn!("Failed to start window enabler thread: {err}");
          None
        }
      }
    })
    .as_ref()
}

/// Queues `EnableWindow(handle, TRUE)` on the worker thread.
pub(crate) fn enable_async(handle: isize) {
  let Some(worker) = worker() else {
    return;
  };

  let _ = worker
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .send(handle);
}
