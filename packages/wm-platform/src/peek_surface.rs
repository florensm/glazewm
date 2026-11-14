//! One window per monitor that moving tiles' peeks draw on together,
//! under `GLAZEWM_ISOLATE=onesurface` (`ONE_SURFACE_PLAN.md`).

use std::{cell::Cell, rc::Rc};

use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    DestroyWindow, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE, WS_EX_NOACTIVATE,
    WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
  },
};

use crate::{
  companion,
  native_surrogate::{self, CLASS_NAME},
  perf::{self, Stage},
  window_class, CornerStyle, Rect,
};

/// A click-through, never activated window spanning a clip rect, with no
/// thumbnail of its own, that hosted peeks draw their thumbnails on (see
/// `WindowPeek::hosted`).
///
/// Of the surrogates' window class, so everything that skips the WM's own
/// windows skips it too. Shown while any of its tenants draws, hidden
/// otherwise. Tenants stack in the order their thumbnails were
/// registered, the only order DWM has among one window's thumbnails.
///
/// Its window is destroyed once the surface and every peek hosted on it
/// are dropped: each holds it.
pub struct PeekSurface {
  window: Rc<SurfaceWindow>,
}

impl PeekSurface {
  /// Creates a hidden surface spanning `clip`.
  pub fn new(clip: &Rect) -> crate::Result<Self> {
    Ok(Self {
      window: Rc::new(SurfaceWindow::create(clip)?),
    })
  }

  /// The area it spans, which its tenants are cut at.
  #[must_use]
  pub fn clip(&self) -> &Rect {
    &self.window.clip
  }

  #[must_use]
  pub fn hwnd(&self) -> HWND {
    self.window.hwnd()
  }

  /// Whether any tenant draws on it, which shows it.
  #[must_use]
  pub fn is_shown(&self) -> bool {
    self.window.drawing.get() > 0
  }

  /// Whether no peek is hosted on it, ready ones aside.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.window.tenants.get() == 0
  }

  /// Whether no peek is hosted on it, ready ones included (see
  /// `WindowPeek::set_ready`).
  #[must_use]
  pub fn is_unused(&self) -> bool {
    self.is_empty() && self.window.ready.get() == 0
  }

  /// Whether a new peek can be hosted on it: not once it has been raised
  /// over its tenants' windows, until it is empty again (see
  /// [`SurfaceWindow::admit`]).
  #[must_use]
  pub fn takes_tenants(&self) -> bool {
    !self.window.raised.get()
  }

  /// Puts it at the top of the non-topmost windows.
  pub fn raise_to_top(&self) {
    if let Err(err) = self.window.raise_to_top() {
      tracing::warn!("Failed to raise a peek surface to the top: {err}");
    }
  }

  /// Opens or closes a pass of raises, within which a raise of the
  /// surface still at the top since its last one is skipped (see
  /// [`note_covered`](Self::note_covered)).
  pub fn set_raise_pass(&self, is_open: bool) {
    self.window.in_raise_pass.set(is_open);
    self.window.on_top.set(false);
  }

  /// Notes that something may have been put above it since it was last
  /// raised, so its next raise is not skipped.
  pub fn note_covered(&self) {
    self.window.on_top.set(false);
  }

  pub(crate) fn window(&self) -> &Rc<SurfaceWindow> {
    &self.window
  }
}

/// The window of a [`PeekSurface`], shared by the surface and the peeks
/// hosted on it, with their count.
pub(crate) struct SurfaceWindow {
  hwnd: isize,
  clip: Rect,
  /// Peeks hosted on it, ready ones aside.
  tenants: Cell<usize>,
  /// Peeks kept on it, hidden, for their tiles' next motions (see
  /// `WindowPeek::set_ready`).
  ready: Cell<usize>,
  /// Of those, the ones that draw, i.e. are shown.
  drawing: Cell<usize>,
  /// Whether it has been raised over its tenants' windows since it was
  /// last empty.
  raised: Cell<bool>,
  /// Whether a pass of raises is open (see
  /// `PeekSurface::set_raise_pass`).
  in_raise_pass: Cell<bool>,
  /// Whether it was raised to the top in the open pass, and nothing is
  /// known to have been put above it since.
  on_top: Cell<bool>,
}

impl SurfaceWindow {
  /// Creates the window, hidden, with the styles `NativeSurrogate::create`
  /// gives a pinned peek's: unrounded, as each thumbnail shows its
  /// window's own corners.
  fn create(clip: &Rect) -> crate::Result<Self> {
    let _scope = perf::scope(Stage::Surface);
    native_surrogate::ensure_class_registered();

    // `WS_EX_NOREDIRECTIONBITMAP`: no surface of its own, so everything
    // outside its tenants' thumbnails is transparent.
    let hwnd = window_class::create_popup(
      CLASS_NAME,
      WS_EX_NOACTIVATE
        | WS_EX_TOOLWINDOW
        | WS_EX_TRANSPARENT
        | WS_EX_NOREDIRECTIONBITMAP,
      clip,
      "peek surface",
    )?;
    native_surrogate::style_like_window(hwnd, CornerStyle::Square);

    Ok(Self {
      hwnd: hwnd.0,
      clip: clip.clone(),
      tenants: Cell::new(0),
      ready: Cell::new(0),
      drawing: Cell::new(0),
      raised: Cell::new(false),
      in_raise_pass: Cell::new(false),
      on_top: Cell::new(false),
    })
  }

  pub(crate) fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  pub(crate) fn clip(&self) -> &Rect {
    &self.clip
  }

  /// Takes in a tenant drawing `window`, which still shows: the surface
  /// goes below it unless it is already, as a pinned peek goes up just
  /// below its window, so the tenant's copies, its opaque backdrop's
  /// among them, stay under that window until it is cloaked.
  ///
  /// Only before the surface has been raised (see
  /// [`PeekSurface::takes_tenants`]): moved down then, it would take
  /// tenants already drawn over other windows under them.
  pub(crate) fn admit(&self, window: HWND) {
    self.tenants.set(self.tenants.get() + 1);

    if companion::is_above(window, self.hwnd()) {
      return;
    }

    let _scope = perf::scope(Stage::Surface);
    self.on_top.set(false);
    // SAFETY: Both handles are valid or fail the call; the flags leave
    // position, size and activation untouched.
    if let Err(err) = unsafe {
      SetWindowPos(
        self.hwnd(),
        window,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      )
    } {
      tracing::warn!("Failed to put a peek surface below a window: {err}");
    }
  }

  /// Lets a tenant go. The last one leaving lets the surface take new
  /// ones again.
  pub(crate) fn release(&self) {
    let tenants = self.tenants.get().saturating_sub(1);
    self.tenants.set(tenants);
    if tenants == 0 {
      self.raised.set(false);
    }
  }

  /// Lets a tenant go as [`release`](Self::release) does, kept on as a
  /// ready peek, which never keeps the surface from taking new tenants.
  pub(crate) fn retire(&self) {
    self.release();
    self.ready.set(self.ready.get() + 1);
  }

  /// Takes a ready peek in again as a tenant drawing `window` (see
  /// [`admit`](Self::admit)).
  pub(crate) fn rejoin(&self, window: HWND) {
    self.drop_ready();
    self.admit(window);
  }

  /// Lets a ready peek go.
  pub(crate) fn drop_ready(&self) {
    self.ready.set(self.ready.get().saturating_sub(1));
  }

  /// Notes a tenant starting or stopping to draw: the first one shows the
  /// surface, and the last one hides it.
  pub(crate) fn note_drawing(&self, is_drawing: bool) {
    let before = self.drawing.get();
    let after = if is_drawing {
      before + 1
    } else {
      before.saturating_sub(1)
    };
    self.drawing.set(after);

    if (before == 0) != (after == 0) {
      let _scope = perf::scope(Stage::Surface);
      self.on_top.set(false);
      // SAFETY: `self.hwnd` is valid until drop.
      unsafe {
        ShowWindow(
          self.hwnd(),
          if after > 0 {
            SW_SHOWNOACTIVATE
          } else {
            SW_HIDE
          },
        );
      }
    }
  }

  /// Puts the surface at the top of the non-topmost windows, as
  /// `NativeSurrogate::raise_to_top` does a surrogate. Skipped while it
  /// is there already from a raise in the open pass (see
  /// `PeekSurface::set_raise_pass`).
  pub(crate) fn raise_to_top(&self) -> crate::Result<()> {
    self.raised.set(true);
    if self.on_top.get() {
      return Ok(());
    }

    let _scope = perf::scope(Stage::Surface);

    // SAFETY: `self.hwnd` is valid until drop; the flags leave position,
    // size and activation untouched.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        HWND(0),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      )
    }?;
    self.on_top.set(self.in_raise_pass.get());
    Ok(())
  }

  /// Puts the surface at the top of the always-on-top band if `topmost`,
  /// else at the top of the other windows.
  pub(crate) fn raise_to_top_of_band(
    &self,
    topmost: bool,
  ) -> crate::Result<()> {
    let _scope = perf::scope(Stage::Surface);
    self.raised.set(true);
    self.on_top.set(false);
    window_class::raise_to_top_of_band(self.hwnd(), topmost)
  }
}

impl Drop for SurfaceWindow {
  fn drop(&mut self) {
    // SAFETY: `self.hwnd` is the window created by this type. Every
    // thumbnail on it went with the peek that registered it, each of which
    // held this window until then.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}
