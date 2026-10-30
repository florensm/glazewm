use std::sync::OnceLock;

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
      DestroyWindow, GetClassNameW, GetWindow, SetLayeredWindowAttributes,
      SetWindowPos, ShowWindow, GW_HWNDPREV, LWA_ALPHA, SWP_HIDEWINDOW,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE,
      SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, WS_EX_LAYERED,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT,
    },
  },
};

use crate::{
  window_class::{self, DeferredPositions},
  Rect, StyleBlend, SurrogateBatch,
};

/// Which overlay an [`OverlayWindow`] backs; each gets its own window
/// class so the overlays can be told apart in z-order dumps.
#[derive(Clone, Copy, Debug)]
pub(crate) enum OverlayKind {
  Backdrop,
  Border,
  DropIndicator,
}

impl OverlayKind {
  fn class_name(self) -> PCWSTR {
    match self {
      Self::Backdrop => w!("GlazeWM_BackdropOverlay"),
      Self::Border => w!("GlazeWM_BorderOverlay"),
      Self::DropIndicator => w!("GlazeWM_DropIndicator"),
    }
  }

  /// Whether `hwnd` is a backdrop or border overlay window.
  ///
  /// Leaves out the drop indicator, which isn't kept behind a window of
  /// its own, so that a window's overlays settle above it.
  pub(crate) fn is_overlay(hwnd: HWND) -> bool {
    let mut buf = [0u16; 32];
    // SAFETY: `buf` outlives the call; a stale `hwnd` just returns 0.
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    let name = &buf[..usize::try_from(len).unwrap_or(0)];

    [Self::Backdrop, Self::Border].iter().any(|kind| {
      let class_name = kind.class_name();
      // SAFETY: `class_name` returns a static, null-terminated literal.
      unsafe { class_name.as_wide() == name }
    })
  }

  /// Whether its window is layered, so it can be hidden by its alpha
  /// while thumbnails of it still draw it (see
  /// [`OverlayWindow::set_alpha`]).
  fn is_layered(self) -> bool {
    matches!(self, Self::Border)
  }

  fn registered(self) -> &'static OnceLock<()> {
    static BACKDROP: OnceLock<()> = OnceLock::new();
    static BORDER: OnceLock<()> = OnceLock::new();
    static DROP_INDICATOR: OnceLock<()> = OnceLock::new();

    match self {
      Self::Backdrop => &BACKDROP,
      Self::Border => &BORDER,
      Self::DropIndicator => &DROP_INDICATOR,
    }
  }

  /// Prefix for log and error messages.
  fn label(self) -> &'static str {
    match self {
      Self::Backdrop => "Backdrop overlay",
      Self::Border => "Border overlay",
      Self::DropIndicator => "Drop indicator",
    }
  }
}

/// The Win32 window behind an overlay (backdrop, border or drop
/// indicator): a click-through popup, rendered entirely by a composition
/// visual tree, kept directly behind an anchor window in z-order.
///
/// Destroys the window on drop. An overlay must drop its visual tree
/// first, since that is rooted to this `HWND`.
pub(crate) struct OverlayWindow {
  /// Raw handle, so the overlay is `Send` even though `HWND` is not.
  hwnd: isize,

  kind: OverlayKind,

  /// Window this overlay was last placed behind. Tracked so callers can
  /// skip a `SetWindowPos` when neither it nor the rect changed.
  anchor: isize,

  /// Whether the window is shown. Tracked rather than inferred from a
  /// rect change, so re-showing at an unchanged rect still goes through
  /// [`place`](Self::place) and reapplies `SWP_SHOWWINDOW`.
  is_visible: bool,
}

impl OverlayWindow {
  /// Creates the window at `rect`, hidden, to be shown behind `anchor` by
  /// [`place`](Self::place) once its visual tree exists.
  pub(crate) fn create(
    kind: OverlayKind,
    rect: &Rect,
    anchor: HWND,
  ) -> crate::Result<Self> {
    window_class::ensure_class_registered(
      kind.registered(),
      kind.class_name(),
      window_class::default_wnd_proc,
    );

    // `WS_EX_TRANSPARENT` is mandatory, not cosmetic: overlays live on the
    // WM's thread, which never pumps a Win32 message queue, so Windows
    // treats them as hung. A hit-testable overlay shows the busy cursor
    // and swallows clicks. `WS_EX_NOREDIRECTIONBITMAP` skips the GDI
    // surface the composition visual tree replaces.
    let mut ex_style = WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT
      | WS_EX_NOREDIRECTIONBITMAP;
    if kind.is_layered() {
      ex_style |= WS_EX_LAYERED;
    }

    let hwnd = window_class::create_popup(
      kind.class_name(),
      ex_style,
      rect,
      &kind.label().to_lowercase(),
    )?;

    let window = Self {
      hwnd: hwnd.0,
      kind,
      anchor: anchor.0,
      is_visible: false,
    };

    // A layered window shows nothing until its alpha is first set.
    if kind.is_layered() {
      window.set_alpha(u8::MAX)?;
    }

    Ok(window)
  }

  pub(crate) fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  pub(crate) fn anchor(&self) -> HWND {
    HWND(self.anchor)
  }

  pub(crate) fn is_visible(&self) -> bool {
    self.is_visible
  }

  /// Whether a [`place`](Self::place) at `anchor` would change nothing
  /// beyond geometry the caller already knows is unchanged.
  pub(crate) fn is_placed_behind(&self, anchor: HWND) -> bool {
    self.is_visible && self.anchor == anchor.0
  }

  /// Makes the next caller-side no-op check fail, so the following
  /// [`place`](Self::place) goes through even at an unchanged rect and
  /// anchor. Does not hide the window.
  pub(crate) fn mark_stale(&mut self) {
    self.is_visible = false;
  }

  /// Moves the window to `rect` behind `anchor` and shows it.
  ///
  /// Leaves the tracked state untouched on failure, so the next call
  /// retries.
  pub(crate) fn place(
    &mut self,
    rect: &Rect,
    anchor: HWND,
  ) -> crate::Result<()> {
    // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        window_class::insert_after_point(anchor),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    }
    .map_err(|err| {
      crate::Error::Platform(format!(
        "{} SetWindowPos failed: {err}.",
        self.kind.label()
      ))
    })?;

    self.anchor = anchor.0;
    self.is_visible = true;
    Ok(())
  }

  /// Puts the window back directly behind `anchor` if it has drifted,
  /// without touching its rect.
  ///
  /// `force` skips the drift check. Callers pass it when `anchor`'s own
  /// z-order changed earlier in the same tick: `NativeWindow::set_z_order`
  /// uses `SWP_ASYNCWINDOWPOS`, so `GW_HWNDPREV` can still report the old,
  /// correct-looking order, and skipping would strand the overlay in front
  /// of its window once the move lands.
  pub(crate) fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()> {
    // The overlay has to be in the anchor's band first, or the OS refuses
    // to leave it directly behind a topmost window.
    window_class::match_z_band(self.hwnd(), anchor);

    let insert_after = window_class::insert_after_point(anchor);

    // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
    let prev = unsafe { GetWindow(self.hwnd(), GW_HWNDPREV) };

    // The window's other overlay may sit in between: backdrop and border
    // don't overlap, so their relative order is invisible, and insisting
    // on one made each re-stack displace the other every tick.
    let is_settled = prev == insert_after
      || (OverlayKind::is_overlay(prev)
        // SAFETY: A stale `prev` just makes `GetWindow` return `HWND(0)`.
        && unsafe { GetWindow(prev, GW_HWNDPREV) } == insert_after);

    if force || !is_settled {
      // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
      unsafe {
        SetWindowPos(
          self.hwnd(),
          insert_after,
          0,
          0,
          0,
          0,
          SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOMOVE | SWP_NOSIZE,
        )
      }?;
    }

    self.anchor = anchor.0;
    Ok(())
  }

  /// Sets what DWM shows of a layered window on screen, from transparent
  /// (`0`) to opaque. Thumbnails of the window leave this alpha out, so at
  /// `0` they still draw it while the screen shows nothing.
  pub(crate) fn set_alpha(&self, alpha: u8) -> crate::Result<()> {
    // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
    unsafe {
      SetLayeredWindowAttributes(self.hwnd(), None, alpha, LWA_ALPHA)
    }
    .map_err(|err| {
      crate::Error::Platform(format!(
        "{} alpha change failed: {err}.",
        self.kind.label()
      ))
    })
  }

  pub(crate) fn hide(&mut self) {
    self.is_visible = false;

    // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
    unsafe {
      let _ = ShowWindow(self.hwnd(), SW_HIDE);
    }
  }
}

/// Shows `show` at `rect` directly behind `anchor` and hides `hide` in one
/// `DeferWindowPos` transaction, which DWM takes as one window-state
/// update: no frame has both overlays or neither.
///
/// Falls back to showing, then hiding, one call at a time, which can
/// leave a frame with both rather than one with neither.
pub(crate) fn swap(
  show: &mut OverlayWindow,
  rect: &Rect,
  anchor: HWND,
  hide: &mut OverlayWindow,
) {
  let insert_after = window_class::insert_after_point(anchor);

  let swapped =
    DeferredPositions::begin(2).is_some_and(|mut transaction| {
      transaction.defer(
        show.hwnd(),
        insert_after,
        rect,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
      ) && transaction.defer(
        hide.hwnd(),
        HWND(0),
        &Rect::default(),
        SWP_NOACTIVATE
          | SWP_NOMOVE
          | SWP_NOSIZE
          | SWP_NOZORDER
          | SWP_HIDEWINDOW,
      ) && transaction.end()
    });

  if swapped {
    show.anchor = anchor.0;
    show.is_visible = true;
    hide.is_visible = false;
    return;
  }

  if let Err(err) = show.place(rect, anchor) {
    tracing::warn!("{err}");
  }
  hide.hide();
}

impl Drop for OverlayWindow {
  fn drop(&mut self) {
    // SAFETY: `self.hwnd()` is valid and `Drop` runs at most once.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}

/// An overlay window kept directly behind a managed window
/// ([`NativeBackdropOverlay`], [`NativeBorderOverlay`]), so the WM can
/// drive every kind through one generic path.
///
/// [`NativeBackdropOverlay`]: crate::NativeBackdropOverlay
/// [`NativeBorderOverlay`]: crate::NativeBorderOverlay
pub trait Overlay: Sized {
  /// Appearance settings, resolved from the user config.
  type Params: Copy;

  /// Creates the overlay for a window at `rect`, shown directly behind
  /// `anchor` -- the managed window, or its surrogate while one is active.
  fn create(
    rect: &Rect,
    params: Self::Params,
    anchor: HWND,
  ) -> crate::Result<Self>;

  /// Applies `params`; each setting is only re-applied when it changed.
  fn apply(&mut self, params: Self::Params) {
    self.apply_blend(StyleBlend::settled(params));
  }

  /// Applies a style partway through a transition, for animating between
  /// focus states rather than snapping.
  fn apply_blend(&mut self, blend: StyleBlend<Self::Params>);

  /// Queues a reposition into `batch` instead of an immediate
  /// `SetWindowPos`, so the overlay moves in the same DWM frame as its
  /// window and every other overlay/surrogate committed with it.
  ///
  /// Falls back to an immediate reposition when the overlay is hidden or
  /// `anchor` changed: the batch's `SWP_NOZORDER` flags carry neither the
  /// show bit nor a z-order move.
  fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
    anchor: HWND,
  );

  /// [`defer_rect`](Self::defer_rect), with the overlay cut off at the
  /// edges of `clip`, e.g. its window's monitor. Hides the overlay when
  /// none of it is inside.
  fn defer_clipped_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
    clip: &Rect,
    anchor: HWND,
  );

  /// Puts the overlay back directly behind `anchor` if it has drifted,
  /// without touching its rect. See [`OverlayWindow::sync_z_order`] for
  /// `force`.
  fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()>;

  fn is_visible(&self) -> bool;

  /// Hides the overlay without destroying it.
  fn hide(&mut self);

  /// Its window, which a thumbnail of it is of.
  fn hwnd(&self) -> HWND;

  /// Where its window is while it tracks a window at `rect` with
  /// `params`.
  fn bounds(rect: &Rect, _params: &Self::Params) -> Rect {
    rect.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::{OverlayKind, OverlayWindow, HWND};
  use crate::{platform_impl, Rect};

  /// A hidden overlay window off-screen, or `None` without an interactive
  /// window station.
  fn create(kind: OverlayKind) -> Option<OverlayWindow> {
    OverlayWindow::create(
      kind,
      &Rect::from_xy(-8000, 100, 400, 300),
      HWND(0),
    )
    .ok()
  }

  fn alpha(window: &OverlayWindow) -> Option<u8> {
    platform_impl::NativeWindow::new(window.hwnd().0)
      .layered_alpha()
      .ok()
      .flatten()
  }

  /// A border can be hidden by its alpha (see
  /// `NativeBorderOverlay::park`), so it starts out layered and opaque.
  #[test]
  fn border_window_takes_an_alpha() {
    let Some(window) = create(OverlayKind::Border) else {
      return;
    };
    assert_eq!(alpha(&window), Some(u8::MAX));

    assert!(window.set_alpha(0).is_ok());
    assert_eq!(alpha(&window), Some(0));
  }

  #[test]
  fn backdrop_window_is_not_layered() {
    let Some(window) = create(OverlayKind::Backdrop) else {
      return;
    };
    assert_eq!(alpha(&window), None);
  }
}
