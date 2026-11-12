//! Helps WPF windows lay out to the sizes the WM gives them.
//!
//! A WPF window with `SizeToContent` other than `Manual` keeps its size in
//! `WM_WINDOWPOSCHANGING`, which the WM's `SWP_NOSENDCHANGING` skips: its
//! HWND takes the WM's size, but WPF doesn't lay its content out to it
//! (`HwndSource.Process_WM_SIZE`), leaving the rest unpainted. WPF turns
//! `SizeToContent` off and lays out to the HWND's size on the `WM_SIZING`
//! that starts a user's resize (`HwndSource.DisableSizeToContent`), and
//! then follows every later resize, so the WM sends it one, until it is
//! answered.
//!
//! A WPF window also lays out at most to the maximum it reports in
//! `WM_GETMINMAXINFO` (`Window.GetWindowMinMax`), which the WM's sizes
//! bypass as well, so a size beyond it is logged.
//!
//! Its `RECT` is marshaled across processes by the system, so it must be
//! sent, not posted. It is sent from a worker thread with
//! `SMTO_ABORTIFHUNG`, as for `window_icons`, so a busy app never stalls
//! the WM's main loop. Outcomes go to [`crate::WPF_LOG_TARGET`].

use std::{
  collections::{HashMap, VecDeque},
  sync::{Condvar, LazyLock, Mutex, PoisonError},
  thread,
};

use windows::Win32::{
  Foundation::{GetLastError, ERROR_TIMEOUT, HWND, LPARAM, RECT, WPARAM},
  UI::WindowsAndMessaging::{
    GetWindowRect, SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_BLOCK,
    WMSZ_BOTTOMRIGHT, WM_SIZING,
  },
};

use crate::{platform_impl, WPF_LOG_TARGET};

/// How long an app gets to answer. Long, as it doesn't hold up the WM, and
/// windows that get it have often just opened, while their app is busy.
const SIZING_TIMEOUT_MS: u32 = 3000;

/// How long an app gets to answer `WM_GETMINMAXINFO`.
const LIMITS_TIMEOUT_MS: u32 = 500;

/// Class name prefix of WPF's top-level windows
/// (`HwndWrapper[app;;guid]`).
const WPF_CLASS_PREFIX: &str = "HwndWrapper[";

/// Whether a window of class `class_name` is a WPF top-level window.
pub(crate) fn is_wpf_class(class_name: &str) -> bool {
  class_name.starts_with(WPF_CLASS_PREFIX)
}

/// How a window answered `WM_SIZING`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
  Answered,
  TimedOut,
  /// Failed with this Win32 error, e.g. 5 for a window of a process of
  /// higher integrity (UIPI), or 1400 for one gone.
  Failed(u32),
}

/// A window's maximum size, as far as it is known.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MaxSize {
  #[default]
  Unasked,
  /// It didn't answer.
  Unknown,
  Known(i32, i32),
}

/// What is known of a WPF window the WM has sized.
#[derive(Debug, Default)]
struct Record {
  /// How its last `WM_SIZING` went, if it was sent one.
  outcome: Option<Outcome>,

  max_size: MaxSize,

  /// Whether a size beyond its maximum was logged.
  is_beyond_max_logged: bool,
}

impl Record {
  /// Whether `WM_SIZING` is (still) to be sent: until answered once, as
  /// `SizeToContent` stays off from then on.
  fn needs_sizing(&self) -> bool {
    self.outcome != Some(Outcome::Answered)
  }

  /// Whether `size` goes past the window's known maximum, not yet logged.
  fn is_new_beyond_max(&self, (width, height): (i32, i32)) -> bool {
    !self.is_beyond_max_logged
      && matches!(
        self.max_size,
        MaxSize::Known(max_w, max_h) if width > max_w || height > max_h
      )
  }

  /// Whether a request for `size` has anything left to do.
  fn has_work(&self, size: (i32, i32)) -> bool {
    self.needs_sizing()
      || self.max_size == MaxSize::Unasked
      || self.is_new_beyond_max(size)
  }
}

#[derive(Default)]
struct Worker {
  state: Mutex<WorkerState>,
  signal: Condvar,
}

#[derive(Default)]
struct WorkerState {
  /// Windows to handle, each once, oldest first.
  queue: VecDeque<isize>,
  /// Newest size requested of each queued window.
  sizes: HashMap<isize, (i32, i32)>,
  records: HashMap<isize, Record>,
}

fn worker() -> Option<&'static Worker> {
  static WORKER: LazyLock<Worker> = LazyLock::new(Worker::default);
  static IS_RUNNING: LazyLock<bool> = LazyLock::new(|| {
    thread::Builder::new()
      .name("size-to-content".to_string())
      .spawn(|| WORKER.run())
      .inspect_err(|err| {
        tracing::warn!("Failed to start SizeToContent thread: {err}");
      })
      .is_ok()
  });

  IS_RUNNING.then_some(&*WORKER)
}

/// Notes that the WM sizes WPF window `window` to `size`, which sends it
/// `WM_SIZING` until it answers, and logs a size beyond its maximum.
pub(crate) fn note_sized(window: isize, size: (i32, i32)) {
  let Some(worker) = worker() else {
    return;
  };

  let mut state = worker.lock();
  let has_work = state
    .records
    .get(&window)
    .is_none_or(|record| record.has_work(size));

  if !has_work {
    return;
  }

  if state.sizes.insert(window, size).is_none() {
    state.queue.push_back(window);
  }

  worker.signal.notify_one();
}

impl Worker {
  fn lock(&self) -> std::sync::MutexGuard<'_, WorkerState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn run(&self) {
    loop {
      let (window, size, needs_sizing, max_size) = {
        let mut state = self.lock();

        let window = loop {
          if let Some(window) = state.queue.pop_front() {
            break window;
          }
          state = self
            .signal
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner);
        };

        let size = state.sizes.remove(&window).unwrap_or_default();
        let record = state.records.entry(window).or_default();
        (window, size, record.needs_sizing(), record.max_size)
      };

      let max_size = if max_size == MaxSize::Unasked {
        platform_impl::NativeWindow::new(window)
          .size_limits(LIMITS_TIMEOUT_MS)
          .map_or(MaxSize::Unknown, |(_, (width, height))| {
            MaxSize::Known(width, height)
          })
      } else {
        max_size
      };

      let outcome = needs_sizing.then(|| send_sizing(HWND(window)));

      let mut state = self.lock();
      let record = state.records.entry(window).or_default();
      record.max_size = max_size;

      if record.is_new_beyond_max(size) {
        record.is_beyond_max_logged = true;
        if let MaxSize::Known(max_w, max_h) = max_size {
          tracing::info!(
            target: WPF_LOG_TARGET,
            "WPF window {window:#x} is sized to {}x{}, past the most it \
             allows ({max_w}x{max_h}): WPF lays its content out no \
             larger, leaving the rest unpainted.",
            size.0,
            size.1,
          );
        }
      }

      if let Some(outcome) = outcome.flatten() {
        if record.outcome != Some(outcome) {
          log_outcome(window, size, outcome);
        }
        record.outcome = Some(outcome);
      }
    }
  }
}

fn log_outcome(
  window: isize,
  (width, height): (i32, i32),
  outcome: Outcome,
) {
  match outcome {
    Outcome::Answered => tracing::info!(
      target: WPF_LOG_TARGET,
      "WPF window {window:#x} answered WM_SIZING, sized to \
       {width}x{height}: its SizeToContent is off from now on."
    ),
    Outcome::TimedOut => tracing::info!(
      target: WPF_LOG_TARGET,
      "WPF window {window:#x} didn't answer WM_SIZING within \
       {SIZING_TIMEOUT_MS}ms, or is hung: asked again at its next resize."
    ),
    Outcome::Failed(code) => tracing::info!(
      target: WPF_LOG_TARGET,
      "WM_SIZING to WPF window {window:#x} failed (error {code}): asked \
       again at its next resize."
    ),
  }
}

/// Sends `window` a `WM_SIZING` for its bottom-right corner, at the rect
/// it has now. `None` if the window is gone.
fn send_sizing(window: HWND) -> Option<Outcome> {
  let mut rect = RECT::default();

  // SAFETY: `rect` outlives the call; a stale handle just fails.
  if unsafe { GetWindowRect(window, &raw mut rect) }.is_err() {
    return None;
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

  if answered.0 != 0 {
    return Some(Outcome::Answered);
  }

  // SAFETY: No preconditions.
  let error = unsafe { GetLastError() };
  Some(match error {
    // With no error set, e.g. given up on as hung: not answered in time.
    Ok(()) => Outcome::TimedOut,
    Err(err) if err.code() == ERROR_TIMEOUT.to_hresult() => {
      Outcome::TimedOut
    }
    #[allow(clippy::cast_sign_loss)]
    Err(err) => Outcome::Failed((err.code().0 & 0xFFFF) as u32),
  })
}

#[cfg(test)]
mod tests {
  use super::{is_wpf_class, MaxSize, Outcome, Record};

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

  #[test]
  fn sizing_is_sent_until_answered() {
    let mut record = Record::default();
    assert!(record.needs_sizing());

    for outcome in [Outcome::TimedOut, Outcome::Failed(5)] {
      record.outcome = Some(outcome);
      assert!(record.needs_sizing());
    }

    record.outcome = Some(Outcome::Answered);
    assert!(!record.needs_sizing());
  }

  #[test]
  fn an_answered_window_is_queued_only_past_its_maximum() {
    let mut record = Record {
      outcome: Some(Outcome::Answered),
      ..Record::default()
    };

    // Its maximum isn't known yet.
    assert!(record.has_work((800, 600)));

    record.max_size = MaxSize::Known(1000, 700);
    assert!(!record.has_work((800, 600)));
    assert!(record.has_work((1200, 600)));

    record.is_beyond_max_logged = true;
    assert!(!record.has_work((1200, 600)));

    // Didn't answer the question: nothing to compare with.
    record.max_size = MaxSize::Unknown;
    record.is_beyond_max_logged = false;
    assert!(!record.has_work((5000, 5000)));
  }
}
