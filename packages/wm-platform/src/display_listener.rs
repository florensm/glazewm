use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher};

/// A listener for system-wide display setting changes.
///
/// Detects changes to display configuration including resolution changes,
/// display connections/disconnections, and working area changes.
pub struct DisplayListener {
  event_rx: mpsc::UnboundedReceiver<()>,

  /// Inner platform-specific display listener.
  inner: platform_impl::DisplayListener,
}

impl DisplayListener {
  /// Creates a new [`DisplayListener`].
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner = platform_impl::DisplayListener::new(event_tx, dispatcher)?;
    Ok(Self { event_rx, inner })
  }

  /// Returns when the next display settings change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<()> {
    let event = self.event_rx.recv().await;

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Display);
    }

    event
  }

  /// Returns the next display event if one is already queued.
  ///
  /// Unlike [`next_event`], never waits. Lets the main loop service events
  /// that piled up while the previous animation frame was running.
  ///
  /// [`next_event`]: DisplayListener::next_event
  pub fn try_next_event(&mut self) -> Option<()> {
    let event = self.event_rx.try_recv().ok();

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Display);
    }

    event
  }

  /// Terminates the display listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}
