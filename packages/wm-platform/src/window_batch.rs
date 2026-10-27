use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, SetWindowPos,
    SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE, SWP_NOCOPYBITS,
    SWP_NOSENDCHANGING, SWP_NOZORDER,
  },
};

use crate::Rect;

/// Collects moves of the WM's own windows, e.g. overlays, and applies them
/// atomically in a single `DeferWindowPos` transaction.
///
/// Sequential `SetWindowPos` calls can straddle a DWM composition
/// boundary, letting a window and its overlays desync for one frame.
/// Batching them into one transaction lands every move in the same
/// composition frame.
///
/// When the transaction cannot be created or fails mid-way, [`commit`]
/// falls back to individual `SetWindowPos` calls so no reposition is lost.
///
/// [`commit`]: WindowBatch::commit
///
/// # Platform-specific
///
/// Only available on Windows.
#[derive(Default)]
pub struct WindowBatch {
  /// Queued moves as `(hwnd, target rect)` pairs.
  entries: Vec<(isize, Rect)>,
}

impl WindowBatch {
  /// Creates an empty batch.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Queues a move of `hwnd`, a top-level window this process owns;
  /// applied on [`commit`].
  ///
  /// [`commit`]: WindowBatch::commit
  pub(crate) fn push(&mut self, hwnd: isize, rect: Rect) {
    self.entries.push((hwnd, rect));
  }

  /// `SetWindowPos` flags shared by both commit paths.
  ///
  /// Notably *without* `SWP_NOSENDCHANGING` — see [`deferred_flags`] and
  /// [`individual_flags`].
  ///
  /// [`deferred_flags`]: WindowBatch::deferred_flags
  /// [`individual_flags`]: WindowBatch::individual_flags
  fn base_flags() -> SET_WINDOW_POS_FLAGS {
    SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOZORDER
  }

  /// Flags passed to `DeferWindowPos`.
  ///
  /// `SWP_NOSENDCHANGING` is deliberately excluded: `DeferWindowPos`
  /// rejects it with `ERROR_INVALID_PARAMETER` even though
  /// `SetWindowPos` accepts it (and the documentation lists it for
  /// both). Including it made every batch fail on its very first entry,
  /// so the whole transaction was dead code that silently fell through
  /// to [`commit_individually`] — and, far worse, abandoned the
  /// transaction's `HDWP`, leaking one USER object per animation frame.
  ///
  /// The batched windows are all windows this process owns, whose
  /// window procedure is `DefWindowProcW`, so the `WM_WINDOWPOSCHANGING`
  /// message this flag would have suppressed costs a same-thread dispatch
  /// into the default handler and nothing else.
  ///
  /// [`commit_individually`]: WindowBatch::commit_individually
  fn deferred_flags() -> SET_WINDOW_POS_FLAGS {
    Self::base_flags()
  }

  /// Flags passed to the per-window `SetWindowPos` fallback.
  ///
  /// `SetWindowPos` does honor `SWP_NOSENDCHANGING`, so the fallback keeps
  /// skipping the `WM_WINDOWPOSCHANGING` round-trip.
  fn individual_flags() -> SET_WINDOW_POS_FLAGS {
    Self::base_flags() | SWP_NOSENDCHANGING
  }

  /// Applies all queued repositions in one `DeferWindowPos` transaction.
  ///
  /// Falls back to individual `SetWindowPos` calls when the transaction
  /// fails (e.g. a window was destroyed meanwhile).
  pub fn commit(self) {
    if self.entries.is_empty() {
      return;
    }

    let _scope = crate::perf::scope(crate::perf::Stage::BatchCommit);

    // SAFETY: All handles refer to windows owned by this process; a stale
    // handle only causes the transaction to fail, which is handled by the
    // fallback below.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let deferred = unsafe {
      let Ok(mut hdwp) = BeginDeferWindowPos(self.entries.len() as i32)
      else {
        return Self::commit_individually(
          &self.entries,
          Self::individual_flags(),
        );
      };

      let mut failed = false;
      for (hwnd, rect) in &self.entries {
        // The transaction (including prior entries) is invalidated on
        // failure; redo everything individually below.
        let Ok(next) = DeferWindowPos(
          hdwp,
          HWND(*hwnd),
          HWND(0),
          rect.x(),
          rect.y(),
          rect.width(),
          rect.height(),
          Self::deferred_flags(),
        ) else {
          failed = true;
          break;
        };
        hdwp = next;
      }

      // Always end the transaction, including after a failed
      // `DeferWindowPos`. `BeginDeferWindowPos` allocates an `HDWP`, which
      // is a USER object released only by `EndDeferWindowPos`; abandoning
      // the handle — as `DeferWindowPos`'s documentation suggests — leaks
      // exactly one USER object per abandoned transaction, permanently and
      // with no recovery short of process exit. Ending a partially built
      // transaction is safe and applies whichever entries were accepted
      // before the failure; `commit_individually` re-applies all of them
      // anyway, so the final positions are identical either way.
      let ended = EndDeferWindowPos(hdwp).is_ok();
      !failed && ended
    };

    if !deferred {
      Self::commit_individually(&self.entries, Self::individual_flags());
    }
  }

  /// Fallback: applies each queued move with its own `SetWindowPos` call.
  fn commit_individually(
    entries: &[(isize, Rect)],
    flags: SET_WINDOW_POS_FLAGS,
  ) {
    for (hwnd, rect) in entries {
      // SAFETY: See `commit` — failures for stale handles are ignored.
      unsafe {
        let _ = SetWindowPos(
          HWND(*hwnd),
          HWND(0),
          rect.x(),
          rect.y(),
          rect.width(),
          rect.height(),
          flags,
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::OnceLock;

  use windows::{
    core::{w, PCWSTR},
    Win32::{
      System::Threading::{
        GetCurrentProcess, GetGuiResources, GR_USEROBJECTS,
      },
      UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
      },
    },
  };

  use super::{
    BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, Rect,
    WindowBatch, HWND, SWP_NOSENDCHANGING,
  };
  use crate::window_class;

  const PROBE_CLASS: PCWSTR = w!("GlazeWM_BatchProbe");

  /// Batch commits performed by the leak regression test.
  ///
  /// Comfortably larger than the number of USER objects any concurrently
  /// running test could plausibly create, so the regressed behaviour (one
  /// leaked object per commit) is unambiguous against the tolerance below.
  const LEAK_TEST_COMMITS: usize = 500;

  /// Slack allowed on the USER-object delta in the leak regression test.
  ///
  /// Tests share one process and `GetGuiResources` counts the whole
  /// process, so an unrelated test creating a window or event loop
  /// concurrently must not fail this. The regressed behaviour leaked
  /// [`LEAK_TEST_COMMITS`] objects, an order of magnitude above this.
  const LEAK_TEST_TOLERANCE: i64 = 32;

  fn user_objects() -> i64 {
    // SAFETY: The pseudo-handle returned by `GetCurrentProcess` is always
    // valid and needs no closing.
    i64::from(unsafe {
      GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS)
    })
  }

  /// Creates a hidden, off-screen popup window for use as a move target.
  ///
  /// `index` offsets the window's y position so several probe windows do
  /// not overlap exactly.
  ///
  /// Returns `None` when window creation fails (e.g. no interactive window
  /// station), letting the caller skip rather than fail spuriously.
  fn create_probe_window(index: i32) -> Option<HWND> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    window_class::ensure_class_registered(
      &REGISTERED,
      PROBE_CLASS,
      window_class::default_wnd_proc,
    );

    // SAFETY: The class is registered above; all other arguments are
    // valid.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        PROBE_CLASS,
        w!(""),
        WS_POPUP,
        -8000,
        100 + index * 10,
        400,
        300,
        None,
        None,
        None,
        None,
      )
    };

    (hwnd.0 != 0).then_some(hwnd)
  }

  /// `DeferWindowPos` rejects `SWP_NOSENDCHANGING`, so it must never
  /// appear in the batch path's flags -- while the `SetWindowPos`
  /// fallback, which does honor it, keeps it.
  #[test]
  fn deferred_flags_exclude_no_send_changing() {
    assert_eq!(
      WindowBatch::deferred_flags().0 & SWP_NOSENDCHANGING.0,
      0,
      "`DeferWindowPos` fails with ERROR_INVALID_PARAMETER when passed \
       `SWP_NOSENDCHANGING`, which abandons the transaction's `HDWP`."
    );
    assert_ne!(
      WindowBatch::individual_flags().0 & SWP_NOSENDCHANGING.0,
      0,
      "The `SetWindowPos` fallback should still skip \
       `WM_WINDOWPOSCHANGING`."
    );
  }

  /// The OS itself must accept the batch path's flags -- this is what the
  /// regression was: a flag set the documentation permits but
  /// `DeferWindowPos` rejects at runtime, making every batch fall back.
  #[test]
  fn defer_window_pos_accepts_deferred_flags() {
    let Some(hwnd) = create_probe_window(0) else {
      return;
    };

    // SAFETY: `hwnd` is a live window owned by this process; the `HDWP` is
    // ended on every path below.
    let accepted = unsafe {
      match BeginDeferWindowPos(1) {
        Err(_) => None,
        Ok(hdwp) => {
          let result = DeferWindowPos(
            hdwp,
            hwnd,
            HWND(0),
            -8000,
            120,
            400,
            300,
            WindowBatch::deferred_flags(),
          );
          let ok = result.is_ok();
          let _ = EndDeferWindowPos(result.unwrap_or(hdwp));
          Some(ok)
        }
      }
    };

    // SAFETY: `hwnd` was created above and not yet destroyed.
    unsafe {
      let _ = DestroyWindow(hwnd);
    }

    if let Some(accepted) = accepted {
      assert!(
        accepted,
        "`DeferWindowPos` rejected `WindowBatch::deferred_flags()`, so \
         every batch would silently fall back to individual \
         `SetWindowPos` calls."
      );
    }
  }

  /// Committing batches must not accumulate USER objects.
  ///
  /// `BeginDeferWindowPos` allocates an `HDWP`, a USER object released
  /// only by `EndDeferWindowPos`. Abandoning it after a failed
  /// `DeferWindowPos` leaked one object per animation frame, exhausting
  /// the process's 10,000-object limit within a few hundred gestures.
  #[test]
  fn commit_leaks_no_user_objects() {
    let Some(first) = create_probe_window(1) else {
      return;
    };
    let Some(second) = create_probe_window(2) else {
      // SAFETY: `first` was created above and not yet destroyed.
      unsafe {
        let _ = DestroyWindow(first);
      }
      return;
    };

    // Warm-up commit: the first reposition of a freshly created window can
    // allocate one-off state that would otherwise read as a delta.
    let mut warmup = WindowBatch::new();
    warmup.push(first.0, Rect::from_xy(-8000, 100, 400, 300));
    warmup.push(second.0, Rect::from_xy(-8000, 110, 400, 300));
    warmup.commit();

    let before = user_objects();

    for i in 0..LEAK_TEST_COMMITS {
      let offset = i32::from(i % 2 == 0);
      let mut batch = WindowBatch::new();
      batch.push(first.0, Rect::from_xy(-8000, 100 + offset, 400, 300));
      batch.push(second.0, Rect::from_xy(-8000, 110 + offset, 400, 300));
      batch.commit();
    }

    let after = user_objects();

    // SAFETY: Both handles were created above and not yet destroyed.
    unsafe {
      let _ = DestroyWindow(first);
      let _ = DestroyWindow(second);
    }

    assert!(
      after - before <= LEAK_TEST_TOLERANCE,
      "{LEAK_TEST_COMMITS} batch commits leaked {} USER objects \
       (before={before}, after={after}).",
      after - before
    );
  }
}
