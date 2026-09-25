use windows::Win32::Foundation::HWND;

use crate::{
  overlay_window::{OverlayKind, OverlayWindow},
  platform_impl::color_capture::ThemedCapture,
  Color, ColorTheme, Rect, SurrogateBatch,
};

/// A click-through window directly above a managed window, showing a live
/// copy of it recolored by a [`ColorTheme`].
///
/// The copy comes from `Windows.Graphics.Capture` and is themed on the GPU
/// (see `color_capture`). Until the first themed frame is presented, and
/// after any pipeline failure, the overlay shows nothing, so the real
/// window is always visible rather than a blank.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeColorThemeOverlay {
  /// Declared before `window`: fields drop in declaration order, and the
  /// visual it roots must go before the `HWND`.
  capture: ThemedCapture,

  window: OverlayWindow,
  theme: ColorTheme,

  /// Last rect applied, used to skip redundant `SetWindowPos` calls.
  rect: Rect,
}

impl NativeColorThemeOverlay {
  /// Starts theming `source`, whose frame is `rect`, with the overlay
  /// shown directly above `anchor`.
  pub fn create(
    source: HWND,
    rect: &Rect,
    theme: &ColorTheme,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let mut window =
      OverlayWindow::create(OverlayKind::ColorTheme, rect, anchor)?;
    let capture =
      ThemedCapture::start(source, window.hwnd(), rect, theme)?;

    if let Err(err) = window.place_above(rect, anchor) {
      tracing::warn!("{err}");
    }

    Ok(Self {
      capture,
      window,
      theme: *theme,
      rect: rect.clone(),
    })
  }

  /// Switches to `theme`; no-op when unchanged.
  pub fn set_theme(&mut self, theme: &ColorTheme) {
    if &self.theme == theme {
      return;
    }

    self.theme = *theme;
    self.capture.set_theme(theme);
  }

  /// Sets a solid fill for the part of the overlay the themed frame
  /// doesn't cover, or removes it with `None`.
  ///
  /// Meant for animations, where the overlay can grow ahead of the
  /// window it copies; `color` should already be themed.
  pub fn set_fill(&self, color: Option<Color>) {
    self.capture.set_fill(color);
  }

  /// Keeps the current fill until the themed frame covers the whole
  /// overlay again, then removes it.
  ///
  /// For the end of an animation, when the window has just been resized
  /// and the last frame is still at its old size.
  pub fn release_fill(&self) {
    let size = (
      u32::try_from(self.rect.width()).unwrap_or(0),
      u32::try_from(self.rect.height()).unwrap_or(0),
    );
    self.capture.release_fill_when_covered(size);
  }

  /// Covers the overlay with `color` until the first themed frame
  /// arrives, instead of leaving it transparent; `color` should already
  /// be themed.
  pub fn set_placeholder(&self, color: Color) {
    let size = (
      u32::try_from(self.rect.width()).unwrap_or(0),
      u32::try_from(self.rect.height()).unwrap_or(0),
    );
    self.capture.set_placeholder(color, size);
  }

  /// Moves the overlay to `rect` directly above `anchor`, and shows it.
  ///
  /// Only re-asserts z-order if neither changed and the overlay is already
  /// visible: a redraw can raise the window over its overlay without
  /// moving it.
  pub fn set_rect(&mut self, rect: &Rect, anchor: HWND) {
    if self.window.is_visible()
      && self.window.anchor() == anchor
      && &self.rect == rect
    {
      if let Err(err) = self.window.sync_z_order_above(anchor) {
        tracing::warn!("{err}");
      }
      return;
    }

    if let Err(err) = self.window.place_above(rect, anchor) {
      tracing::warn!("{err}");
      return;
    }

    self.rect = rect.clone();
  }

  /// Queues a move to `rect` into `batch`, so it lands in the same DWM
  /// frame as the surrogate it covers.
  ///
  /// Falls back to [`set_rect`](Self::set_rect) when the overlay isn't
  /// already shown above `anchor`: the batch only moves windows.
  pub fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
    anchor: HWND,
  ) {
    if !self.window.is_visible() || self.window.anchor() != anchor {
      self.set_rect(rect, anchor);
      return;
    }

    if &self.rect != rect {
      batch.push(self.window.hwnd().0, rect.clone());
      self.rect = rect.clone();
    }
  }

  /// Puts the overlay back directly above `anchor` if it has drifted.
  pub fn sync_z_order(&mut self, anchor: HWND) -> crate::Result<()> {
    self.window.sync_z_order_above(anchor)
  }

  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.window.is_visible()
  }

  /// Hides the overlay without stopping the capture.
  pub fn hide(&mut self) {
    self.window.hide();
  }

  /// Whether theming stopped for good (e.g. the GPU device was lost);
  /// the overlay then shows nothing and should be dropped.
  #[must_use]
  pub fn has_failed(&self) -> bool {
    self.capture.has_failed()
  }
}
