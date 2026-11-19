use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  native_surrogate::to_logical, resize_session::compute_border_inset,
  CornerStyle, NativeSurrogate, NativeWindow, NativeWindowWindowsExt,
  Rect, SurrogateBatch,
};

/// A click-through, live thumbnail of a window, cut to the part of it
/// that lies inside a clip rect.
///
/// Shows the partly visible columns of a scrolling layout: the real
/// window stays cloaked, so nothing of it reaches past its monitor.
pub struct WindowPeek {
  surrogate: NativeSurrogate,
  border_inset: RECT,
  /// The window's visible frame where it was last placed, clipped or not.
  frame_rect: Option<Rect>,
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
      border_inset,
      corner_style,
      // Just below the window it shows, like the other surrogates.
      hwnd,
      false,
    )?;

    Ok(Self {
      surrogate,
      border_inset,
      frame_rect: None,
      visible_rect: None,
    })
  }

  /// Shows the part of the window, placed at `rect`, that lies inside
  /// `clip`, or hides the peek when none does. A move of a peek already
  /// shown is queued into `batch`, to land with its window's overlays.
  pub fn place(
    &mut self,
    rect: &Rect,
    clip: &Rect,
    opacity: u8,
    batch: &mut SurrogateBatch,
  ) {
    let logical = to_logical(rect, &self.border_inset);

    self.visible_rect = self.surrogate.show_clipped(
      &logical,
      clip,
      self.border_inset,
      opacity,
      Some(batch),
    );
    self.frame_rect = Some(logical);
  }

  /// The window's visible frame where it was last placed, including any
  /// part outside the clip, for overlays that frame it.
  #[must_use]
  pub fn frame_rect(&self) -> Option<&Rect> {
    self.frame_rect.as_ref()
  }

  /// Handle of the peek's own window, which overlays of the window it
  /// shows sit behind.
  #[must_use]
  pub fn hwnd(&self) -> HWND {
    self.surrogate.hwnd()
  }

  /// The on-screen part of the peek, if it shows anything.
  #[must_use]
  pub fn visible_rect(&self) -> Option<&Rect> {
    self.visible_rect.as_ref()
  }
}
