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
    Self::on_dequeued(event.as_ref());
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
    Self::on_dequeued(event.as_ref());
    event
  }

  // LINT: Only does anything on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  fn on_dequeued(event: Option<&WindowEvent>) {
    #[cfg(target_os = "windows")]
    if let Some(event) = event {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Window);

      if matches!(event, WindowEvent::ZOrderChanged { .. }) {
        platform_impl::z_order_change_dequeued();
      }
    }
  }

  /// Terminates the window listener.
  pub fn terminate(&mut self) {
    self.inner.terminate();
  }
}
