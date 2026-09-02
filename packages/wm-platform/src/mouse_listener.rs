use tokio::sync::mpsc;

use crate::{platform_event::MouseEvent, platform_impl, Dispatcher};

/// Available mouse events that [`MouseListener`] can listen for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MouseEventKind {
  Move,
  LeftButtonDown,
  LeftButtonUp,
  RightButtonDown,
  RightButtonUp,
}

/// A listener for system-wide mouse events.
pub struct MouseListener {
  /// Receiver for outgoing mouse events.
  event_rx: mpsc::UnboundedReceiver<MouseEvent>,

  /// Inner platform-specific mouse listener.
  inner: platform_impl::MouseListener,
}

impl MouseListener {
  /// Creates a new [`MouseListener`] with the specified enabled events.
  pub fn new(
    enabled_events: &[MouseEventKind],
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner = platform_impl::MouseListener::new(
      enabled_events,
      event_tx,
      dispatcher,
    )?;

    Ok(Self { event_rx, inner })
  }

  /// Returns the next mouse event from the listener.
  ///
  /// This will block until a mouse event is available.
  pub async fn next_event(&mut self) -> Option<MouseEvent> {
    let event = self.event_rx.recv().await;

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Mouse);
    }

    event
  }

  /// Returns the next mouse event if one is already queued.
  ///
  /// Unlike [`next_event`], never waits. Used by the main loop to service
  /// events that piled up while the previous animation frame was running.
  ///
  /// [`next_event`]: MouseListener::next_event
  pub fn try_next_event(&mut self) -> Option<MouseEvent> {
    let event = self.event_rx.try_recv().ok();

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(crate::perf::EventKind::Mouse);
    }

    event
  }

  /// Enables or disables the underlying mouse listener.
  pub fn enable(&mut self, enabled: bool) -> crate::Result<()> {
    self.inner.enable(enabled)
  }

  /// Updates the set of enabled mouse events to listen for.
  pub fn set_enabled_events(
    &mut self,
    enabled_events: &[MouseEventKind],
  ) -> crate::Result<()> {
    self.inner.set_enabled_events(enabled_events)
  }

  /// Terminates the mouse listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}
