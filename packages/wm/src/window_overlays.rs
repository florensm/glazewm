use wm_platform::{
  NativeBackdropOverlay, NativeBorderOverlay, Overlay, OverlayParams,
  Rect, SurrogateBatch, HWND,
};

/// A managed window's overlays, kept in one entry so they are created,
/// moved, hidden and destroyed together.
///
/// An overlay stays alive while its effect is configured for either focus
/// state, hidden while it isn't wanted, so a focus change re-shows it
/// instead of rebuilding it. See `sync_overlays`.
#[derive(Default)]
pub struct WindowOverlays {
  /// Blurred-wallpaper backdrop behind the window, which a window left
  /// semi-transparent by the `transparency` effect reads as frosted glass
  /// over. See `NativeBackdropOverlay`.
  pub backdrop: Option<NativeBackdropOverlay>,

  /// Self-drawn border ring around the window, a stand-in for the OS's
  /// `DWMWA_BORDER_COLOR`, which DWM thumbnails don't carry along. See
  /// `NativeBorderOverlay`.
  pub border: Option<NativeBorderOverlay>,
}

impl WindowOverlays {
  /// Whether the window has neither overlay.
  pub fn is_empty(&self) -> bool {
    self.backdrop.is_none() && self.border.is_none()
  }

  /// Moves both overlays to `rect` behind `anchor`.
  pub fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
    anchor: HWND,
  ) {
    if let Some(overlay) = &mut self.backdrop {
      overlay.defer_rect(batch, rect, anchor);
    }

    if let Some(overlay) = &mut self.border {
      overlay.defer_rect(batch, rect, anchor);
    }
  }

  /// Hides each shown overlay that `params` has no params for.
  ///
  /// Guarded on `is_visible` because `hide` issues a `ShowWindow`
  /// unconditionally.
  pub fn hide_untracked(&mut self, params: &OverlayParams) {
    if let Some(overlay) = self
      .backdrop
      .as_mut()
      .filter(|overlay| params.backdrop.is_none() && overlay.is_visible())
    {
      overlay.hide();
    }

    if let Some(overlay) = self
      .border
      .as_mut()
      .filter(|overlay| params.border.is_none() && overlay.is_visible())
    {
      overlay.hide();
    }
  }
}

/// Where a window's overlays go: directly behind `anchor`, the backdrop
/// at `rect` and the border around `border_rect`.
pub struct OverlayPlacement {
  /// Window the overlays sit directly behind: the managed window, or its
  /// surrogate while one stands in for it.
  pub anchor: HWND,

  /// The window's rect, which the backdrop fills.
  pub rect: Rect,

  /// The rect the border goes around: `rect`, plus the tab bar of the
  /// window's stack where that is framed with it.
  pub border_rect: Rect,
}

impl OverlayPlacement {
  /// Placement with the border around `rect` itself.
  pub fn new(anchor: HWND, rect: Rect) -> Self {
    Self {
      anchor,
      border_rect: rect.clone(),
      rect,
    }
  }
}
