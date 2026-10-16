use std::sync::mpsc;

use crate::{Dispatcher, EventLoop};

/// A thread of its own running a Win32 message loop, for windows that
/// must answer their messages promptly.
///
/// The main event loop also runs the keyboard and window-event hooks,
/// whose callbacks wait behind every message that thread processes;
/// windows that draw there delay input system-wide.
///
/// Stops the loop on drop. Windows still on the thread are destroyed by
/// the OS when it exits.
pub struct UiThread {
  dispatcher: Dispatcher,
}

impl UiThread {
  /// Spawns the thread and waits for its event loop to be ready.
  pub fn spawn(name: &str) -> crate::Result<Self> {
    let (dispatcher_tx, dispatcher_rx) = mpsc::channel();

    std::thread::Builder::new().name(name.to_string()).spawn(
      move || match EventLoop::new() {
        Ok((event_loop, dispatcher)) => {
          let _ = dispatcher_tx.send(Ok(dispatcher));

          if let Err(err) = event_loop.run() {
            tracing::warn!("UI thread's event loop failed: {err}");
          }
        }
        Err(err) => {
          let _ = dispatcher_tx.send(Err(err));
        }
      },
    )?;

    let dispatcher = dispatcher_rx.recv().map_err(|_| {
      crate::Error::Platform("UI thread exited on startup.".to_string())
    })??;

    Ok(Self { dispatcher })
  }

  /// Dispatcher running closures on the thread.
  #[must_use]
  pub fn dispatcher(&self) -> &Dispatcher {
    &self.dispatcher
  }
}

impl Drop for UiThread {
  fn drop(&mut self) {
    // Not joined: shutdown must not wait on a thread that may be busy.
    let _ = self.dispatcher.stop_event_loop();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn runs_closures_on_its_own_thread() {
    let ui_thread = UiThread::spawn("test-ui").expect("UI thread spawns");

    let thread_name = ui_thread
      .dispatcher()
      .dispatch_sync(|| std::thread::current().name().map(String::from))
      .expect("dispatch succeeds");

    assert_eq!(thread_name.as_deref(), Some("test-ui"));
  }
}
