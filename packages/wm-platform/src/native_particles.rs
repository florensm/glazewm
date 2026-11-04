//! Shows [`ParticleBurst`]s: each in a click-through window of its own,
//! just large enough for its particles, on a thread running a message
//! loop, where a timer takes the window down once its particles are gone.
//! The compositor plays everything in between.
//!
//! The overview's bursts show from its own thread, each owned by the
//! overview's window: an owned window always stays above its owner, which
//! the overview raises again as the WM moves windows under it.

use std::{
  cell::RefCell, collections::HashMap, sync::OnceLock, time::Duration,
};

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
      DefWindowProcW, DestroyWindow, KillTimer,
      SetLayeredWindowAttributes, SetTimer, SetWindowPos, HWND_TOP,
      HWND_TOPMOST, LWA_ALPHA, SWP_NOACTIVATE, SWP_SHOWWINDOW,
      WINDOW_EX_STYLE, WM_DESTROY, WM_TIMER, WS_EX_LAYERED,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
      WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    },
  },
};

use crate::{
  particles::{fresh_seed, plan},
  platform_impl::composition::ParticleVisual,
  window_class, Dispatcher, ParticleBurst,
};

/// Bursts on screen at once, past which new ones are dropped, so a held
/// key can't pile them up.
const MAX_BURSTS: usize = 8;

/// How long a burst's window stays past its last particle's end, so no
/// frame of it is cut short.
const LINGER: Duration = Duration::from_millis(100);

/// The timer that takes a burst's window down.
const TEARDOWN_TIMER: usize = 1;

thread_local! {
  /// The bursts on screen from this thread, by window.
  static BURSTS: RefCell<HashMap<isize, ParticleVisual>> =
    RefCell::new(HashMap::new());
}

impl ParticleBurst {
  /// Shows the burst from `dispatcher`'s thread, which must run a message
  /// loop for its windows' timers. Returns at once.
  pub fn spawn(self, dispatcher: &Dispatcher) {
    let seed = fresh_seed();

    let result = dispatcher.dispatch_async(move || {
      if let Err(err) = show(&self, seed, None) {
        tracing::warn!("Failed to show particles: {err}");
      }
    });
    if let Err(err) = result {
      tracing::warn!("Failed to dispatch particles: {err}");
    }
  }

  /// Shows the burst above `owner`, a topmost window of this thread,
  /// which must run a message loop for the burst's timer.
  pub(crate) fn show_above(&self, owner: HWND) {
    if let Err(err) = show(self, fresh_seed(), Some(owner)) {
      tracing::warn!("Failed to show particles: {err}");
    }
  }
}

/// Shows `burst`, scattered by `seed`, on this thread; above `owner` if
/// given, else above every window that isn't topmost.
fn show(
  burst: &ParticleBurst,
  seed: u64,
  owner: Option<HWND>,
) -> crate::Result<()> {
  if BURSTS.with(|bursts| bursts.borrow().len()) >= MAX_BURSTS {
    return Ok(());
  }
  let Some(plan) = plan(burst, seed) else {
    return Ok(());
  };

  register_class();

  let (topmost, insert_after) = match owner {
    Some(_) => (WS_EX_TOPMOST, HWND_TOPMOST),
    None => (WINDOW_EX_STYLE::default(), HWND_TOP),
  };

  // Layered and transparent, so it is passed over by clicks and by
  // `WindowFromPoint`.
  let hwnd = window_class::create_owned_popup(
    w!("GlazeWM_Particles"),
    WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT
      | WS_EX_LAYERED
      | WS_EX_NOREDIRECTIONBITMAP
      | topmost,
    &plan.bounds,
    owner,
    "particles",
  )?;

  let visual = (|| {
    // SAFETY: `hwnd` was just created on this thread.
    unsafe { SetLayeredWindowAttributes(hwnd, None, u8::MAX, LWA_ALPHA) }?;
    ParticleVisual::create(hwnd, &plan)
  })();
  let visual = match visual {
    Ok(visual) => visual,
    Err(err) => {
      // SAFETY: `hwnd` was created on this thread and nothing else holds
      // it.
      let _ = unsafe { DestroyWindow(hwnd) };
      return Err(err);
    }
  };
  BURSTS.with(|bursts| bursts.borrow_mut().insert(hwnd.0, visual));

  let lifetime_ms = u32::try_from((plan.lifetime + LINGER).as_millis())
    .unwrap_or(u32::MAX);

  // SAFETY: `hwnd` is a valid window of this thread; the timer is handled
  // by its window procedure.
  let shown = unsafe {
    SetWindowPos(
      hwnd,
      insert_after,
      plan.bounds.x(),
      plan.bounds.y(),
      plan.bounds.width(),
      plan.bounds.height(),
      SWP_NOACTIVATE | SWP_SHOWWINDOW,
    )
    .map(|()| SetTimer(hwnd, TEARDOWN_TIMER, lifetime_ms, None))
  };

  match shown {
    Ok(timer) if timer != 0 => Ok(()),
    Ok(_) => {
      tear_down(hwnd);
      Err(crate::Error::Platform(
        "Failed to time the particles' window.".to_string(),
      ))
    }
    Err(err) => {
      tear_down(hwnd);
      Err(err.into())
    }
  }
}

fn register_class() {
  static REGISTERED: OnceLock<()> = OnceLock::new();

  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_Particles"),
    particles_wnd_proc,
  );
}

/// Takes a burst's window down, and with it its visual tree (see
/// `particles_wnd_proc`).
fn tear_down(hwnd: HWND) {
  // SAFETY: `hwnd` is a window of this thread; a stale one only makes the
  // calls fail.
  unsafe {
    let _ = KillTimer(hwnd, TEARDOWN_TIMER);
    let _ = DestroyWindow(hwnd);
  }
}

/// Window procedure for a burst's window: takes it down when its timer
/// fires, and otherwise leaves it to the default handling, closes aside.
unsafe extern "system" fn particles_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if msg == WM_TIMER && wparam.0 == TEARDOWN_TIMER {
    tear_down(hwnd);
    return LRESULT(0);
  }
  // Its visual tree goes while the window it is rooted to still exists,
  // also when the window is destroyed with the one that owns it.
  if msg == WM_DESTROY {
    let visual = BURSTS.with(|bursts| bursts.borrow_mut().remove(&hwnd.0));
    drop(visual);
  }
  if window_class::is_close_request(msg, wparam) {
    return LRESULT(0);
  }

  // SAFETY: All parameters are forwarded unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
