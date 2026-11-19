//! One window per monitor that moving tiles' peeks draw on together.

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

  /// Whether it has been raised over its tenants' windows since it was
  /// last empty, or held where a tenant draws over its cloaked window, so
  /// a new tenant never moves it down (see [`SurfaceWindow::admit`]).
  #[must_use]
  pub fn is_raised(&self) -> bool {
    self.window.raised.get()
  }

  /// Whether it has been put at the top of its band since it was last
  /// empty or put at rest, so it is above every window there: a tenant
  /// that must stay under windows above its own (a grab's) can't join it.
  #[must_use]
  pub fn is_topped(&self) -> bool {
    self.window.topped.get()
  }

  /// Whether `window` is anywhere above it in the z-order.
  #[must_use]
  pub fn is_below(&self, window: HWND) -> bool {
    companion::is_above(window, self.hwnd())
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

  /// Puts it just above `window`, the cloaked window of the one tenant
  /// left on it, drawn at rest, where that tenant's own peek would be
  /// (just below a window that shows nothing). Done once until something
  /// else moves the surface.
  pub fn rest_above(&self, window: HWND) {
    if self.window.rests_above.get() == window.0 {
      return;
    }

    let _scope = perf::scope(Stage::Surface);
    self.window.on_top.set(false);
    self.window.topped.set(false);
    tracing::debug!(
      "Peek surface {:#x} put at rest above {:#x}.",
      self.hwnd().0,
      window.0
    );
    match window_class::place_above(self.hwnd(), window) {
      Ok(()) => self.window.rests_above.set(window.0),
      Err(err) => {
        tracing::warn!("Failed to put a peek surface at rest: {err}");
      }
    }
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
  /// last empty, or held where a tenant draws over its cloaked window
  /// (see [`hold_place`](Self::hold_place)).
  raised: Cell<bool>,
  /// Whether it has been put at the top of its band since it was last
  /// empty or put at rest (see `PeekSurface::is_topped`).
  topped: Cell<bool>,
  /// Whether a pass of raises is open (see
  /// `PeekSurface::set_raise_pass`).
  in_raise_pass: Cell<bool>,
  /// Whether it was raised to the top in the open pass, and nothing is
  /// known to have been put above it since.
  on_top: Cell<bool>,
  /// The window it was last put just above at rest, if nothing moved it
  /// since (see `PeekSurface::rest_above`).
  rests_above: Cell<isize>,
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
      topped: Cell::new(false),
      in_raise_pass: Cell::new(false),
      on_top: Cell::new(false),
      rests_above: Cell::new(0),
    })
  }

  pub(crate) fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Takes in a tenant drawing `window`, which still shows or not, moving
  /// the surface as [`admission`] says. One whose window is cloaked keeps
  /// the surface from going down from then on (see
  /// [`hold_place`](Self::hold_place)).
  pub(crate) fn admit(&self, window: HWND, is_shown: bool) {
    self.tenants.set(self.tenants.get() + 1);

    let admission = admission(
      self.raised.get(),
      companion::is_above(window, self.hwnd()),
      is_shown,
    );
    if !is_shown {
      self.hold_place();
    }
    if admission == Admission::Stays {
      return;
    }

    let _scope = perf::scope(Stage::Surface);
    tracing::debug!(
      "Peek surface {:#x} put {admission:?} {:#x} for a tenant.",
      self.hwnd,
      window.0
    );
    self.on_top.set(false);
    self.rests_above.set(0);
    let placed = if admission == Admission::Above {
      window_class::place_above(self.hwnd(), window)
    } else {
      // SAFETY: Both handles are valid or fail the call; the flags leave
      // position, size and activation untouched.
      unsafe {
        SetWindowPos(
          self.hwnd(),
          window,
          0,
          0,
          0,
          0,
          SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
      }
      .map_err(Into::into)
    };
    if let Err(err) = placed {
      tracing::warn!("Failed to put a peek surface by a window: {err}");
    }
  }

  /// Keeps the surface from going down for tenants to come, as one raised
  /// over its tenants' windows: one of its tenants draws over its cloaked
  /// window there, which a window put between would cover.
  pub(crate) fn hold_place(&self) {
    self.raised.set(true);
  }

  /// Puts the surface just above `window`, unless it is anywhere above it
  /// already, so it covers the window as that is uncloaked, as a landing
  /// peek of its own goes just above its window. Never down from then on
  /// (see [`hold_place`](Self::hold_place)).
  pub(crate) fn keep_above(&self, window: HWND) {
    self.hold_place();
    if companion::is_above(self.hwnd(), window) {
      return;
    }

    let _scope = perf::scope(Stage::Surface);
    tracing::debug!(
      "Peek surface {:#x} put above {:#x} to keep drawing over it.",
      self.hwnd,
      window.0
    );
    self.on_top.set(false);
    self.rests_above.set(0);
    if let Err(err) = window_class::place_above(self.hwnd(), window) {
      tracing::warn!("Failed to put a peek surface above a window: {err}");
    }
  }

  /// Lets a tenant go. The last one leaving lets the surface take new
  /// ones again.
  pub(crate) fn release(&self) {
    let tenants = self.tenants.get().saturating_sub(1);
    self.tenants.set(tenants);
    if tenants == 0 {
      self.raised.set(false);
      self.topped.set(false);
      self.rests_above.set(0);
    }
  }

  /// Lets a tenant go as [`release`](Self::release) does, kept on as a
  /// ready peek, which never keeps the surface from taking new tenants.
  pub(crate) fn retire(&self) {
    self.release();
    self.add_ready();
  }

  /// Takes in a ready peek that was never a tenant (see
  /// `NativeSurrogate::create_ready`).
  pub(crate) fn add_ready(&self) {
    self.ready.set(self.ready.get() + 1);
  }

  /// Takes a ready peek in again as a tenant drawing `window`, which still
  /// shows (see [`admit`](Self::admit)).
  pub(crate) fn rejoin(&self, window: HWND) {
    self.drop_ready();
    self.admit(window, true);
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
      tracing::debug!(
        "Peek surface {:#x} {}.",
        self.hwnd,
        if after > 0 { "shown" } else { "hidden" }
      );
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
    self.topped.set(true);
    self.rests_above.set(0);
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
    self.topped.set(true);
    self.on_top.set(false);
    self.rests_above.set(0);
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

/// Whether a surface goes just below a joining tenant's window, which
/// still shows and `is_above` the surface or not, given whether it
/// `is_raised` (see [`PeekSurface::is_raised`]).
///
/// Not raised, it goes below the window unless it is already, as a pinned
/// peek goes up just below its window, so the tenant's copies stay under
/// that window until it is cloaked. Raised, it is never moved down, which
/// would take tenants already drawn over other windows under them, but up
/// to just below a window above it, so no window between covers the
/// tenant's copies once that window is cloaked and before the surface is
/// raised again; that is where the raise takes it anyway. Over a window
/// below it, the tenant draws the window as it is until it is cloaked,
/// its backdrop copy too faint to see (see `WindowPeek::warm_up`).
fn goes_below(is_raised: bool, is_above: bool) -> bool {
  is_raised == is_above
}

/// Where a surface goes for a tenant joining it, see [`admission`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Admission {
  Stays,
  /// Just below the tenant's window.
  Below,
  /// Just above the tenant's window.
  Above,
}

/// Where a surface that `is_raised` goes for a joining tenant whose window
/// `is_above` it or not, and `is_shown` or cloaked.
///
/// Shown, as [`goes_below`] says. Cloaked (an entering window), just above
/// it, where its own peek would draw (just below a window that shows
/// nothing), and where the hand-back needs it: covering the window once
/// it is uncloaked. Unless it is raised and above the window already, as
/// it never goes down then.
fn admission(
  is_raised: bool,
  is_above: bool,
  is_shown: bool,
) -> Admission {
  if is_shown {
    if goes_below(is_raised, is_above) {
      Admission::Below
    } else {
      Admission::Stays
    }
  } else if is_raised && !is_above {
    Admission::Stays
  } else {
    Admission::Above
  }
}

#[cfg(test)]
mod tests {
  use super::{admission, goes_below, Admission};

  #[test]
  fn a_raised_surface_only_goes_up_for_a_joining_window() {
    assert!(goes_below(false, false));
    assert!(!goes_below(false, true));
    assert!(goes_below(true, true));
    assert!(!goes_below(true, false));
  }

  #[test]
  fn a_surface_goes_above_a_cloaked_window_unless_raised_over_it() {
    assert_eq!(admission(false, true, false), Admission::Above);
    assert_eq!(admission(false, false, false), Admission::Above);
    assert_eq!(admission(true, true, false), Admission::Above);
    assert_eq!(admission(true, false, false), Admission::Stays);
  }

  #[test]
  fn a_shown_window_is_admitted_as_before() {
    for (is_raised, is_above) in
      [(false, false), (false, true), (true, true), (true, false)]
    {
      assert_eq!(
        admission(is_raised, is_above, true) == Admission::Below,
        goes_below(is_raised, is_above)
      );
      assert_ne!(admission(is_raised, is_above, true), Admission::Above);
    }
  }
}
