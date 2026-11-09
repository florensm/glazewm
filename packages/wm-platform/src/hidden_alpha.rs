//! A real window hidden by its alpha while a stand-in for it takes over
//! drawing it, or hands it back.
//!
//! The shell's (un)cloak is a cross-process call of several milliseconds,
//! which DWM can show anywhere within it: with a semi-transparent window
//! still showing, that left frames with both it and its stand-in up (two
//! layers of a semi-transparent window, which read as solid) or neither.
//! Its alpha DWM applies at once, so hidden by it first, the window's
//! swap with the stand-in lands in one frame and the (un)cloak unseen. An
//! opaque window covers its stand-in, or is covered by it, anyway.

use crate::{platform_impl::NativeWindow, OpacityValue};

/// A semi-transparent window's own alpha, set aside while the window is
/// hidden by it, with its handle. Given back on drop, as a window still
/// hidden by it would stay invisible once shown.
#[derive(Debug, Default)]
pub(crate) struct HiddenAlpha(Option<(isize, u8)>);

impl HiddenAlpha {
  /// Hides `window` by its alpha, setting the alpha aside, when it is
  /// semi-transparent. One it already hides stays hidden.
  pub(crate) fn hide(&mut self, window: &NativeWindow) {
    let alpha = window.layered_alpha().ok().flatten();
    if alpha == Some(0)
      && self.0.is_some_and(|(handle, _)| handle == window.hwnd().0)
    {
      return;
    }

    // A handover still pending would otherwise set aside the 0.
    self.restore();

    let Some(alpha) = window
      .layered_alpha()
      .ok()
      .flatten()
      .filter(|alpha| *alpha < u8::MAX)
    else {
      return;
    };

    match window.set_transparency(&OpacityValue::from_alpha(0)) {
      Ok(()) => self.0 = Some((window.hwnd().0, alpha)),
      Err(err) => tracing::warn!("Failed to hide window by alpha: {err}"),
    }
  }

  /// Whether it hides a window, i.e. set its alpha aside.
  #[must_use]
  pub(crate) fn is_hiding(&self) -> bool {
    self.0.is_some()
  }

  /// Gives the window back the alpha set aside, unless it was set again
  /// since, e.g. by a focus change's effects. No-op otherwise.
  pub(crate) fn restore(&mut self) {
    let Some((handle, alpha)) = self.0.take() else {
      return;
    };

    let window = NativeWindow::new(handle);
    if window.layered_alpha().ok().flatten() == Some(0) {
      if let Err(err) =
        window.set_transparency(&OpacityValue::from_alpha(alpha))
      {
        tracing::warn!("Failed to restore window alpha: {err}");
      }
    }
  }
}

impl Drop for HiddenAlpha {
  fn drop(&mut self) {
    self.restore();
  }
}
