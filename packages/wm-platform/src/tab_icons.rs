//! Icons of tabbed windows, fetched off the UI thread.
//!
//! Apps such as WPF set their icon with `WM_SETICON`, so the class icon is
//! often empty and the real one has to be asked for with `WM_GETICON`.
//! That is a cross-process message, which would freeze the tab bar's
//! thread (the WM's event loop) while a busy app takes its time to answer.
//! It is therefore sent from a worker thread with `SMTO_ABORTIFHUNG` and a
//! short timeout, and the result cached per window.

use std::{
  collections::HashMap,
  sync::{
    mpsc::{self, Sender},
    Mutex, OnceLock, PoisonError,
  },
  thread,
};

use windows::Win32::{
  Foundation::{HWND, LPARAM, WPARAM},
  UI::WindowsAndMessaging::{
    CopyIcon, DestroyIcon, GetClassLongPtrW, PostMessageW,
    SendMessageTimeoutW, GCLP_HICON, GCLP_HICONSM, HICON, ICON_BIG,
    ICON_SMALL, ICON_SMALL2, SMTO_ABORTIFHUNG, SMTO_BLOCK, WM_GETICON,
  },
};

/// How long an app gets to answer `WM_GETICON`.
const ICON_TIMEOUT_MS: u32 = 100;

enum CachedIcon {
  /// A fetch is queued or running.
  Pending,
  /// Owned copy of the window's icon, or `None` if it has none.
  Ready(Option<isize>),
}

/// A request to fetch `window`'s icon, then post `notify_msg` to `notify`.
struct Request {
  window: isize,
  notify: isize,
  notify_msg: u32,
}

fn cache() -> &'static Mutex<HashMap<isize, CachedIcon>> {
  static CACHE: OnceLock<Mutex<HashMap<isize, CachedIcon>>> =
    OnceLock::new();
  CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn worker() -> Option<&'static Mutex<Sender<Request>>> {
  static WORKER: OnceLock<Option<Mutex<Sender<Request>>>> =
    OnceLock::new();

  WORKER
    .get_or_init(|| {
      let (sender, receiver) = mpsc::channel::<Request>();

      let spawned = thread::Builder::new()
        .name("tab-icons".to_string())
        .spawn(move || {
          for request in receiver {
            let icon = fetch_icon(HWND(request.window));

            cache()
              .lock()
              .unwrap_or_else(PoisonError::into_inner)
              .insert(request.window, CachedIcon::Ready(icon));

            // SAFETY: Posting to a destroyed window just fails.
            unsafe {
              let _ = PostMessageW(
                HWND(request.notify),
                request.notify_msg,
                WPARAM(0),
                LPARAM(0),
              );
            }
          }
        });

      match spawned {
        Ok(_) => Some(Mutex::new(sender)),
        Err(err) => {
          tracing::warn!("Failed to start tab icon thread: {err}");
          None
        }
      }
    })
    .as_ref()
}

/// Returns `window`'s icon, or `None` while it is being fetched or if it
/// has none.
///
/// The first call for a window queues a fetch and returns its class icon
/// meanwhile; `notify_msg` is posted to `notify` once the real icon is
/// cached.
pub(crate) fn icon_for(
  window: isize,
  notify: HWND,
  notify_msg: u32,
) -> Option<HICON> {
  let mut cache = cache().lock().unwrap_or_else(PoisonError::into_inner);

  match cache.get(&window) {
    Some(CachedIcon::Ready(icon)) => return icon.map(HICON),
    Some(CachedIcon::Pending) => return class_icon(HWND(window)),
    None => {}
  }

  let Some(worker) = worker() else {
    return class_icon(HWND(window));
  };

  let request = Request {
    window,
    notify: notify.0,
    notify_msg,
  };

  if worker
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .send(request)
    .is_ok()
  {
    cache.insert(window, CachedIcon::Pending);
  }

  class_icon(HWND(window))
}

/// Drops the cached icon of `window`, so it is fetched again next time
/// (e.g. after its title changed, which apps often do with their icon).
pub(crate) fn invalidate(window: isize) {
  let removed = cache()
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .remove(&window);

  if let Some(CachedIcon::Ready(Some(icon))) = removed {
    // SAFETY: The cache owns this copy and no longer hands it out.
    unsafe {
      let _ = DestroyIcon(HICON(icon));
    }
  }
}

/// Asks `window` for its icon, falling back to its class icon. Returns an
/// owned copy.
fn fetch_icon(window: HWND) -> Option<isize> {
  let asked =
    [ICON_SMALL2, ICON_SMALL, ICON_BIG]
      .into_iter()
      .find_map(|kind| {
        let mut result = 0usize;

        // SAFETY: `result` outlives the call. A hung or slow app makes the
        // call fail after `ICON_TIMEOUT_MS` instead of blocking.
        let answered = unsafe {
          SendMessageTimeoutW(
            window,
            WM_GETICON,
            WPARAM(kind as usize),
            LPARAM(0),
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            ICON_TIMEOUT_MS,
            Some(&raw mut result),
          )
        };

        let icon = isize::try_from(result).unwrap_or_default();
        (answered.0 != 0 && icon != 0).then_some(HICON(icon))
      });

  let icon = asked.or_else(|| class_icon(window))?;

  // A copy, since the app may destroy its icon at any time.
  // SAFETY: `icon` is a valid icon handle or makes `CopyIcon` fail.
  unsafe { CopyIcon(icon) }.ok().map(|copy| copy.0)
}

/// The window class's small (or large) icon. Read without messaging the
/// window, so it never blocks.
fn class_icon(window: HWND) -> Option<HICON> {
  [GCLP_HICONSM, GCLP_HICON].into_iter().find_map(|index| {
    // SAFETY: A stale handle just returns 0.
    let handle = unsafe { GetClassLongPtrW(window, index) };
    isize::try_from(handle)
      .ok()
      .filter(|handle| *handle != 0)
      .map(HICON)
  })
}
