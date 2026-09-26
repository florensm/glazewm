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

  /// See [`set_frames_held`](Self::set_frames_held).
  frames_held: bool,
}

impl NativeColorThemeOverlay {
  /// Starts theming `source`, whose frame is `rect`, with the overlay
  /// shown directly above `anchor`.
  ///
  /// With a `placeholder` (already themed), the overlay shows it right
  /// away, before the capture has started, until the first themed frame
  /// covers it: for windows that appear abruptly (menus, dropdowns,
  /// dialogs), which would otherwise show their original colors meanwhile.
  /// With `frames_held`, it starts out as if
  /// [`set_frames_held`](Self::set_frames_held) was called.
  pub fn create(
    source: HWND,
    rect: &Rect,
    theme: &ColorTheme,
    anchor: HWND,
    placeholder: Option<Color>,
    frames_held: bool,
  ) -> crate::Result<Self> {
    let mut window =
      OverlayWindow::create(OverlayKind::ColorTheme, rect, anchor)?;
    let overlay = window.hwnd();
    let mut placed = false;

    let capture = ThemedCapture::start(
      source,
      overlay,
      rect,
      theme,
      placeholder,
      frames_held,
      || {
        // Without a placeholder there is nothing to show before the
        // first frame.
        if placeholder.is_some() {
          if let Err(err) = window.place_above(rect, anchor) {
            tracing::warn!("{err}");
          }
          placed = true;
        }
      },
    )?;

    if !placed {
      if let Err(err) = window.place_above(rect, anchor) {
        tracing::warn!("{err}");
      }
    }

    Ok(Self {
      capture,
      window,
      theme: theme.clone(),
      rect: rect.clone(),
      frames_held,
    })
  }

  /// Switches to `theme`; no-op when unchanged.
  pub fn set_theme(&mut self, theme: &ColorTheme) {
    // Runs every tick: a pointer check, since a changed theme is always
    // a newly loaded one.
    if self.theme.is_same(theme) {
      return;
    }

    self.theme = theme.clone();
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

  /// Holds the themed frames back while `held`, showing only the fill.
  ///
  /// For a window's open animation: the capture shows the window at its
  /// final place, not where the animation has it, so the overlay stays a
  /// plain fill until the animation ends.
  pub fn set_frames_held(&mut self, held: bool) {
    if self.frames_held != held {
      self.frames_held = held;
      self.capture.set_frames_held(held);
    }
  }

  #[must_use]
  pub fn frames_held(&self) -> bool {
    self.frames_held
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
