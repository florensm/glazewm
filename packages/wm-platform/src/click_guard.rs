use std::{
  collections::HashSet,
  sync::{Arc, Mutex, PoisonError},
};

use tokio::sync::mpsc;

use crate::{platform_impl, NativeWindow, WindowId};

/// Keeps clicks from chosen windows and reports them instead, so that a
/// click on one of them can e.g. only focus it rather than also press
/// whatever is under the cursor.
///
/// A press is kept from a window, along with its release, only if the
/// window is the one under the cursor, i.e. not covered there.
pub struct ClickGuard {
  /// Windows whose clicks are kept from them.
  guarded: Arc<Mutex<HashSet<WindowId>>>,

  /// Sender of the guarded windows that were clicked.
  click_tx: mpsc::UnboundedSender<NativeWindow>,

  /// Hook for the clicks, only while any window is guarded, so the mouse
  /// isn't hooked for nothing.
  hook: Option<platform_impl::ClickHook>,
}

impl ClickGuard {
  /// Creates an instance of `ClickGuard` that guards no windows yet.
  ///
  /// Each press on a guarded window is sent through `click_tx`.
  #[must_use]
  pub fn new(click_tx: mpsc::UnboundedSender<NativeWindow>) -> Self {
    Self {
      guarded: Arc::default(),
      click_tx,
      hook: None,
    }
  }

  /// Guards `windows` instead of the ones guarded so far.
  pub fn guard(&mut self, windows: impl IntoIterator<Item = WindowId>) {
    let windows = windows.into_iter().collect::<HashSet<_>>();

    {
      let mut guarded =
        self.guarded.lock().unwrap_or_else(PoisonError::into_inner);

      if *guarded == windows {
        return;
      }

      *guarded = windows;
    }

    if let Err(err) = self.sync_hook() {
      tracing::warn!("Failed to hook mouse clicks: {err}");
    }
  }

  /// Hooks the mouse while any window is guarded, and unhooks it
  /// otherwise.
  fn sync_hook(&mut self) -> crate::Result<()> {
    let is_guarding = !self
      .guarded
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .is_empty();

    match (&mut self.hook, is_guarding) {
      (None, true) => {
        let guarded = self.guarded.clone();
        let click_tx = self.click_tx.clone();

        self.hook = Some(platform_impl::ClickHook::new(move |hwnd| {
          let id = WindowId(hwnd.0);
          let is_guarded = guarded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(&id);

          if is_guarded {
            let window = platform_impl::NativeWindow::new(hwnd.0);
            let _ = click_tx.send(window.into());
          }

          is_guarded
        })?);
      }
      (Some(hook), false) => {
        hook.terminate()?;
        self.hook = None;
      }
      _ => {}
    }

    Ok(())
  }
}
