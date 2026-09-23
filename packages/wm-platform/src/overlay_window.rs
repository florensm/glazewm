use std::sync::OnceLock;

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, GetWindow, SetWindowPos, ShowWindow,
      GW_HWNDPREV, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING,
      SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
      WS_POPUP,
    },
  },
};

use crate::{window_class, Rect, SurrogateBatch};

/// Which overlay an [`OverlayWindow`] backs; each gets its own window
/// class so the overlays can be told apart in z-order dumps.
#[derive(Clone, Copy, Debug)]
pub(crate) enum OverlayKind {
  Backdrop,
  Border,
}

impl OverlayKind {
  fn class_name(self) -> PCWSTR {
    match self {
      Self::Backdrop => w!("GlazeWM_BackdropOverlay"),
      Self::Border => w!("GlazeWM_BorderOverlay"),
    }
  }

  fn registered(self) -> &'static OnceLock<()> {
    static BACKDROP: OnceLock<()> = OnceLock::new();
    static BORDER: OnceLock<()> = OnceLock::new();

    match self {
      Self::Backdrop => &BACKDROP,
      Self::Border => &BORDER,
    }
  }

  /// Prefix for log and error messages.
  fn label(self) -> &'static str {
    match self {
      Self::Backdrop => "Backdrop overlay",
      Self::Border => "Border overlay",
    }
  }
}

/// The Win32 window behind a backdrop or border overlay: a click-through
/// popup, rendered entirely by a composition visual tree, kept directly
/// behind an anchor window in z-order.
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
    let ex_style = WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT
      | WS_EX_NOREDIRECTIONBITMAP;

    // SAFETY: The class is registered above. No parent `HWND` is needed.
    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        kind.class_name(),
        w!(""),
        WS_POPUP,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(format!(
        "Failed to create {} window.",
        kind.label().to_lowercase()
      )));
    }

    Ok(Self {
      hwnd: hwnd.0,
      kind,
      anchor: anchor.0,
      is_visible: false,
    })
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
    if force || prev != insert_after {
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

  pub(crate) fn hide(&mut self) {
    self.is_visible = false;

    // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
    unsafe {
      let _ = ShowWindow(self.hwnd(), SW_HIDE);
    }
  }
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
  fn apply(&mut self, params: Self::Params);

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
}
