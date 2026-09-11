use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher, WindowEvent};

/// A listener for system-wide window events.
pub struct WindowListener {
  event_rx: mpsc::UnboundedReceiver<WindowEvent>,

  /// Inner platform-specific window listener.
  inner: platform_impl::WindowListener,
}

impl WindowListener {
  /// Creates a new window listener.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner = platform_impl::WindowListener::new(event_tx, dispatcher)?;

    Ok(Self { event_rx, inner })
  }

  /// Returns the next window event from the listener.
  ///
  /// This will block until a window event is available.
  pub async fn next_event(&mut self) -> Option<WindowEvent> {
    let event = self.event_rx.recv().await;

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Window);
    }

    event
  }

  /// Returns the next window event if one is already queued.
  ///
  /// Unlike [`next_event`], never waits. Lets the main loop service events
  /// that piled up while the previous animation frame was running.
  ///
  /// [`next_event`]: WindowListener::next_event
  pub fn try_next_event(&mut self) -> Option<WindowEvent> {
    let event = self.event_rx.try_recv().ok();

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Window);
    }

    event
  }

  /// Terminates the window listener.
  pub fn terminate(&mut self) {
    self.inner.terminate();
  }
}
