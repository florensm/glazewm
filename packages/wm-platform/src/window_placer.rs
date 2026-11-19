//! Places windows at the newest rect given, each from a thread of its own,
//! for gestures that move them at the mouse's rate.

use std::{
  collections::HashMap,
  fmt,
  sync::{Arc, Condvar, Mutex, PoisonError},
  thread,
};

use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    SetWindowPos, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOSENDCHANGING,
    SWP_NOZORDER,
  },
};

use crate::{Rect, WindowId};

/// Places each window it is given at the newest rect it was given, with a
/// synchronous `SetWindowPos` on a thread per window.
///
/// The call waits on the window's app, which paces it: a slow app gets
/// the current rect each time it is ready rather than a backlog, and never
/// holds up another window. Unlike the WM's other repositions it neither
/// redraws the frame (`SWP_FRAMECHANGED`) nor discards the window's pixels
/// (`SWP_NOCOPYBITS`), so a step costs the app no more than a step of a
/// drag by its own border does.
#[derive(Default)]
pub struct WindowPlacer {
  workers: Mutex<HashMap<WindowId, Arc<Slot>>>,
}

#[derive(Default)]
struct Slot {
  state: Mutex<SlotState>,
  signal: Condvar,
}

#[derive(Default)]
struct SlotState {
  rect: Option<Rect>,

  /// The rect taken from `rect` that is being applied, if any.
  applying: Option<Rect>,

  quit: bool,
}

impl WindowPlacer {
  /// Places `window` at `rect`, replacing a rect not yet applied.
  pub fn place(&self, window: WindowId, rect: Rect) {
    let mut workers = self.lock();

    let slot = if let Some(slot) = workers.get(&window) {
      slot.clone()
    } else {
      let slot = Arc::new(Slot::default());
      let spawned = {
        let slot = slot.clone();
        thread::Builder::new()
          .name("window-placer".to_string())
          .spawn(move || slot.run(window))
      };

      if let Err(err) = spawned {
        tracing::warn!("Failed to start a window placer: {err}");
        return;
      }

      workers.insert(window, slot.clone());
      slot
    };

    slot.lock().rect = Some(rect);
    slot.signal.notify_one();
  }

  /// Whether `window` has a rect waiting or being applied, i.e. it may
  /// not be where it was last placed yet.
  #[must_use]
  pub fn is_placing(&self, window: WindowId) -> bool {
    self.lock().get(&window).is_some_and(|slot| {
      let state = slot.lock();
      state.rect.is_some() || state.applying.is_some()
    })
  }

  /// The size `window` may be at by the time it has applied every rect it
  /// was given, smallest per axis: of the rect waiting and the one being
  /// applied. `None` with neither.
  #[must_use]
  pub fn unlanded(&self, window: WindowId) -> Option<(i32, i32)> {
    let workers = self.lock();
    let state = workers.get(&window)?.lock();

    [&state.rect, &state.applying]
      .into_iter()
      .flatten()
      .map(|rect| (rect.width(), rect.height()))
      .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1)))
  }

  /// Drops the rect not yet applied to `window`, and its thread.
  pub fn forget(&self, window: WindowId) {
    if let Some(slot) = self.lock().remove(&window) {
      slot.quit();
    }
  }

  /// Lets every rect not yet applied land, then ends the threads.
  ///
  /// Threads are not joined: one waiting on a hung app would hold up the
  /// caller.
  pub fn finish(&self) {
    for (_, slot) in self.lock().drain() {
      slot.finish();
    }
  }

  /// Drops every rect not yet applied, and every thread. A call under way
  /// still lands.
  pub fn clear(&self) {
    for (_, slot) in self.lock().drain() {
      slot.quit();
    }
  }

  fn lock(
    &self,
  ) -> std::sync::MutexGuard<'_, HashMap<WindowId, Arc<Slot>>> {
    self.workers.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl Drop for WindowPlacer {
  fn drop(&mut self) {
    self.clear();
  }
}

impl fmt::Debug for WindowPlacer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowPlacer")
      .field("windows", &self.lock().len())
      .finish()
  }
}

impl Slot {
  fn lock(&self) -> std::sync::MutexGuard<'_, SlotState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn quit(&self) {
    self.lock().rect = None;
    self.finish();
  }

  fn finish(&self) {
    self.lock().quit = true;
    self.signal.notify_one();
  }

  fn run(&self, window: WindowId) {
    loop {
      let (rect, quit) = {
        let mut state = self.lock();

        while state.rect.is_none() && !state.quit {
          state = self
            .signal
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner);
        }

        let rect = state.rect.take();
        state.applying.clone_from(&rect);
        (rect, state.quit)
      };

      if let Some(rect) = rect {
        place_window(window, &rect);
        self.lock().applying = None;
      }

      if quit {
        return;
      }
    }
  }
}

fn place_window(window: WindowId, rect: &Rect) {
  // SAFETY: A stale handle just fails the call. With `SWP_NOZORDER`,
  // `hWndInsertAfter` is ignored.
  let result = unsafe {
    SetWindowPos(
      HWND(window.0),
      HWND(0),
      rect.x(),
      rect.y(),
      rect.width(),
      rect.height(),
      SWP_NOACTIVATE
        | SWP_NOZORDER
        | SWP_NOOWNERZORDER
        | SWP_NOSENDCHANGING,
    )
  };

  if let Err(err) = result {
    tracing::debug!("Failed to place window {:#x}: {err}", window.0);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_a_window_with_a_rect_in_hand_is_placing() {
    let placer = WindowPlacer::default();
    let window = WindowId(0);
    assert!(!placer.is_placing(window));

    placer.place(window, Rect::from_xy(0, 0, 100, 100));
    placer.forget(window);
    assert!(!placer.is_placing(window));

    placer.place(window, Rect::from_xy(0, 0, 100, 100));
    placer.finish();
    assert!(!placer.is_placing(window));
  }

  #[test]
  fn unlanded_takes_the_smallest_of_waiting_and_applying() {
    let placer = WindowPlacer::default();
    let window = WindowId(0);
    assert_eq!(placer.unlanded(window), None);

    placer.lock().insert(window, Arc::new(Slot::default()));
    if let Some(slot) = placer.lock().get(&window) {
      let mut state = slot.lock();
      state.applying = Some(Rect::from_xy(0, 0, 950, 600));
      state.rect = Some(Rect::from_xy(0, 0, 980, 500));
    }
    assert_eq!(placer.unlanded(window), Some((950, 500)));

    placer.clear();
    assert_eq!(placer.unlanded(window), None);
  }
}
