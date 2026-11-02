use core::{
  fmt,
  mem::{self, ManuallyDrop},
};

use crate::Dispatcher;

/// Binds a value to the current event loop thread.
///
/// `ThreadBound<T>` holds a value created on an event loop thread and
/// drops it on that same thread, using the provided [`Dispatcher`]. This
/// lets the wrapper be held across threads (`Send + Sync`) even when `T`
/// itself is not thread-safe. It gives no access to the value.
///
/// Inspired by:
/// - `threadbound::ThreadBound` <https://github.com/dtolnay/threadbound>
/// - `dispatch2::MainThreadBound` <https://github.com/madsmtm/objc2/tree/main/crates/dispatch2>
///
/// NOTE: Dropping the wrapper schedules the inner value to be dropped on
/// the event loop thread. If the event loop has already stopped, the drop
/// is skipped to avoid running `T`'s destructor on the wrong thread,
/// potentially leaking the value.
pub struct ThreadBound<T> {
  value: ManuallyDrop<T>,
  dispatcher: Dispatcher,
}

// SAFETY: The inner value is never accessed, and only dropped on the
// event loop thread it was created on.
unsafe impl<T> Send for ThreadBound<T> {}
unsafe impl<T> Sync for ThreadBound<T> {}

impl<T> ThreadBound<T> {
  /// Binds a value to the current event loop thread.
  ///
  /// # Panics
  ///
  /// Panics if `dispatcher` is not tied to an event loop running on the
  /// current thread.
  #[inline]
  pub fn new(inner: T, dispatcher: Dispatcher) -> Self {
    assert_eq!(std::thread::current().id(), dispatcher.thread_id());

    Self {
      value: ManuallyDrop::new(inner),
      dispatcher,
    }
  }
}

impl<T> fmt::Debug for ThreadBound<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ThreadBound").finish_non_exhaustive()
  }
}

impl<T> Drop for ThreadBound<T> {
  #[allow(
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr,
    clippy::as_conversions,
    clippy::ref_as_ptr
  )]
  fn drop(&mut self) {
    if mem::needs_drop::<T>() {
      // TODO: This is pretty cursed. Should be a better way.
      let value_ptr =
        std::ptr::from_mut::<ManuallyDrop<T>>(&mut self.value) as usize;

      let _ = self.dispatcher.dispatch_sync(|| unsafe {
        // SAFETY: The value is dropped on the event loop thread, which is
        // the same thread that it originated from (guaranteed by `new`).
        // Additionally, the value is never used again after this point.
        ManuallyDrop::drop(&mut *(value_ptr as *mut ManuallyDrop<T>));
      });
    }
  }
}
