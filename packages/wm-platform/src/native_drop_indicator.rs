use windows::Win32::Foundation::HWND;

use crate::{
  overlay_window::{OverlayKind, OverlayWindow},
  platform_impl::composition::DropIndicatorVisual,
  window_class, Color, Rect,
};

/// Highlights where a dragged window will land: a rounded rectangle washed
/// with an accent color and outlined in it, gliding from one drop target
/// to the next.
///
/// Its window covers the whole monitor the target is on, so the rectangle
/// can glide anywhere on it while the window stays put. Being that large,
/// it would otherwise answer `WindowFromPoint` for the whole monitor (see
/// `NativeBorderOverlay`); it is only shown while the mouse button is held
/// for a drag, when the WM's own point queries are off.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeDropIndicator {
  /// The indicator's visual tree. Declared before `window`: fields drop
  /// in declaration order, and the tree must go before the `HWND` it is
  /// rooted to.
  composition: DropIndicatorVisual,

  window: OverlayWindow,

  /// Monitor rect the window covers.
  viewport: Rect,

  /// Where the rectangle was last sent, relative to `viewport`.
  target: Option<Rect>,
}

impl NativeDropIndicator {
  /// Creates the indicator, hidden, for a monitor at `viewport`.
  pub fn create(
    viewport: &Rect,
    color: Color,
    corner_radius: f32,
  ) -> crate::Result<Self> {
    let window = OverlayWindow::create(
      OverlayKind::DropIndicator,
      viewport,
      HWND(0),
    )?;
    let composition = DropIndicatorVisual::create(
      window.hwnd(),
      viewport,
      color,
      corner_radius,
    )?;

    Ok(Self {
      composition,
      window,
      viewport: viewport.clone(),
      target: None,
    })
  }

  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.window.is_visible()
  }

  /// Shows the indicator at `rect` on the monitor at `viewport`, behind
  /// `anchor` (the dragged window). Glides there when already shown on
  /// that monitor.
  pub fn show(&mut self, rect: &Rect, viewport: &Rect, anchor: HWND) {
    let target = rect.translate_to_coordinates(
      rect.x() - viewport.x(),
      rect.y() - viewport.y(),
    );

    if self.window.is_visible() && &self.viewport == viewport {
      if self.target.as_ref() != Some(&target) {
        if let Err(err) = self.composition.glide_to(&target) {
          tracing::warn!("Drop indicator glide failed: {err}");
        }
        self.target = Some(target);
      }
      return;
    }

    // Revealed, or moved to another monitor, while hidden: the window and
    // its tree commit on their own schedules, and a cleared tree shows
    // nothing whichever lands first.
    self.window.hide();

    if let Err(err) = self
      .composition
      .clear()
      .and_then(|()| self.composition.set_viewport(viewport))
      .and_then(|()| self.composition.jump_to(&target))
    {
      tracing::warn!("Drop indicator placement failed: {err}");
      return;
    }

    // Behind the dragged window, which is often in the topmost band once
    // it floats for the drag.
    window_class::match_z_band(self.window.hwnd(), anchor);

    if let Err(err) = self.window.place(viewport, anchor) {
      tracing::warn!("{err}");
      return;
    }

    if let Err(err) = self.composition.fade_in() {
      tracing::warn!("Drop indicator fade in failed: {err}");
    }

    self.viewport = viewport.clone();
    self.target = Some(target);
  }

  /// Updates the rectangle's color and outer corner radius.
  pub fn set_style(&mut self, color: Color, corner_radius: f32) {
    if let Err(err) = self
      .composition
      .set_color(color)
      .and_then(|()| self.composition.set_corner_radius(corner_radius))
    {
      tracing::warn!("Drop indicator style update failed: {err}");
    }
  }

  /// Hides the indicator. No-op when already hidden.
  pub fn hide(&mut self) {
    if !self.window.is_visible() {
      return;
    }

    self.window.hide();
    self.target = None;

    if let Err(err) = self.composition.clear() {
      tracing::warn!("Drop indicator clear failed: {err}");
    }
  }
}
