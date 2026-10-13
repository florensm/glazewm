use windows::Win32::Foundation::RECT;

use crate::{
  native_surrogate::to_logical, resize_session::compute_border_inset,
  CornerStyle, NativeSurrogate, NativeWindow, NativeWindowWindowsExt,
  Rect,
};

/// A click-through, live thumbnail of a window, cut to the part of it
/// that lies inside a clip rect.
///
/// Shows the partly visible columns of a scrolling layout: the real
/// window stays cloaked, so nothing of it reaches past its monitor.
pub struct WindowPeek {
  surrogate: NativeSurrogate,
  border_inset: RECT,
  visible_rect: Option<Rect>,
}

impl WindowPeek {
  /// Creates a hidden peek of `window`, which is `rect` large.
  pub fn new(
    window: &NativeWindow,
    rect: &Rect,
    corner_style: &CornerStyle,
  ) -> crate::Result<Self> {
    let hwnd = window.hwnd();
    let border_inset = compute_border_inset(hwnd);

    let surrogate = NativeSurrogate::create(
      hwnd,
      &to_logical(rect, &border_inset),
      rect,
      u8::MAX,
      false,
      border_inset,
      corner_style,
      // Just below the window it shows, like the other surrogates.
      hwnd,
      false,
    )?;

    Ok(Self {
      surrogate,
      border_inset,
      visible_rect: None,
    })
  }

  /// Shows the part of the window, placed at `rect`, that lies inside
  /// `clip`, or hides the peek when none does.
  pub fn place(&mut self, rect: &Rect, clip: &Rect, opacity: u8) {
    let logical = to_logical(rect, &self.border_inset);

    self.visible_rect = self.surrogate.show_clipped(
      &logical,
      clip,
      self.border_inset,
      opacity,
    );
  }

  /// The on-screen part of the peek, if it shows anything.
  #[must_use]
  pub fn visible_rect(&self) -> Option<&Rect> {
    self.visible_rect.as_ref()
  }
}
