use std::{
  cell::{Cell, RefCell},
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Condvar, Mutex, PoisonError,
  },
  thread::{self, JoinHandle},
  time::Instant,
};

use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
  Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONULL},
  System::Threading::GetCurrentThreadId,
  UI::{
    Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    Input::KeyboardAndMouse::{
      VK_ESCAPE, VK_LMENU, VK_LWIN, VK_RMENU, VK_RWIN,
    },
    WindowsAndMessaging::{
      CallNextHookEx, GetAncestor, GetCursorPos, GetWindowRect,
      PostThreadMessageW, SetCursorPos, SetWindowPos, WindowFromPoint,
      EVENT_SYSTEM_DESKTOPSWITCH, GA_ROOT, MSG, MSLLHOOKSTRUCT,
      SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, WINEVENT_OUTOFCONTEXT,
      WM_APP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN,
      WM_RBUTTONUP,
    },
  },
};

use super::{
  is_async_key_down, mask_modifier_release, mask_modifiers_later,
  MouseHookThread, FOREGROUND_INPUT_IDENTIFIER, WM_MASK_MODIFIERS,
};
use crate::{
  grab_listener::{
    warp_target, GrabButton, GrabShared, Live, MoveOutcome, ReleaseOutcome,
  },
  platform_impl, Gesture, GrabEnd, GrabEvent, GrabModifier, Point, Rect,
  ResizeEdges, WindowId,
};

/// Posted by the hook to its own thread once a gesture began, for the work
/// the hook itself has to leave out to return quickly.
const WM_GESTURE_BEGAN: u32 = WM_APP + 2;

/// Posted to the hook's thread when Escape is pressed during a gesture.
const WM_CANCEL_GESTURE: u32 = WM_APP + 3;

/// The hook's thread while a gesture is under way, for the keyboard hook
/// to cancel it on Escape, or 0.
static GESTURE_THREAD: AtomicU32 = AtomicU32::new(0);

/// Whether an Escape press was taken to cancel a gesture, so that its
/// repeats and release are taken too.
static ESCAPE_TAKEN: AtomicBool = AtomicBool::new(false);

thread_local! {
  /// State of the hook on its thread.
  static HOOK: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

/// What the hook works with on its thread.
struct HookState {
  shared: Arc<GrabShared>,
  mover: Arc<Mover>,

  /// The resize that began, for `WM_GESTURE_BEGAN` to warp the cursor
  /// for: where it was pressed, and what it drags.
  began_resize: Cell<Option<(Point, ResizeEdges)>>,

  /// Hook for switches of the input desktop, e.g. to a UAC prompt's or
  /// the lock screen, where a held press's release can't be seen.
  desktop_switch: HWINEVENTHOOK,
}

impl Drop for HookState {
  fn drop(&mut self) {
    if !self.desktop_switch.is_invalid() {
      // SAFETY: Set on this thread, which drops its state as it exits.
      let _ = unsafe { UnhookWinEvent(self.desktop_switch) };
    }
  }
}

/// A system-wide low-level mouse hook for [`crate::GrabListener`], on a
/// thread of its own, with a second thread that moves windows grabbed to
/// be moved.
///
/// The hook itself makes no call that can block: a move is handed to the
/// mover thread, whose `SetWindowPos` waits on the window's app. Only the
/// newest position is kept, so a slow app gets one current move when it
/// is ready rather than a backlog of stale ones.
pub(crate) struct GrabHook {
  thread: MouseHookThread,
  mover: Arc<Mover>,
  mover_thread: Option<JoinHandle<()>>,
}

impl GrabHook {
  pub(crate) fn new(shared: Arc<GrabShared>) -> crate::Result<Self> {
    let mover = Arc::new(Mover::default());

    let mover_thread = {
      let mover = mover.clone();
      let shared = shared.clone();
      thread::Builder::new()
        .name("grab-mover".to_string())
        .spawn(move || mover.run(&shared))?
    };

    let thread = {
      let mover = mover.clone();
      MouseHookThread::spawn(
        "grab-hook",
        move || {
          // SAFETY: An out-of-context hook, called on this thread from its
          // message loop.
          let desktop_switch = unsafe {
            SetWinEventHook(
              EVENT_SYSTEM_DESKTOPSWITCH,
              EVENT_SYSTEM_DESKTOPSWITCH,
              None,
              Some(on_desktop_switch),
              0,
              0,
              WINEVENT_OUTOFCONTEXT,
            )
          };

          if desktop_switch.is_invalid() {
            tracing::warn!("Failed to hook desktop switches for grabs.");
          }

          HOOK.with(|hook| {
            *hook.borrow_mut() = Some(HookState {
              shared,
              mover,
              began_resize: Cell::new(None),
              desktop_switch,
            });
          });
        },
        Some(hook_proc),
        on_message,
      )
    };

    match thread {
      Ok(thread) => Ok(Self {
        thread,
        mover,
        mover_thread: Some(mover_thread),
      }),
      Err(err) => {
        mover.quit();
        let _ = mover_thread.join();
        Err(err)
      }
    }
  }

  /// Drops the move not yet made, and stops Escape from being taken, as
  /// the WM ends the gesture.
  pub(crate) fn end_gesture(&self) {
    self.mover.lock().target = None;
    GESTURE_THREAD.store(0, Ordering::Relaxed);
  }

  /// Unhooks the mouse and ends both threads.
  pub(crate) fn terminate(&mut self) {
    if let Err(err) = self.thread.terminate() {
      tracing::warn!("Failed to unhook mouse grabs: {err}");
    }

    self.mover.quit();
    if let Some(thread) = self.mover_thread.take() {
      let _ = thread.join();
    }

    GESTURE_THREAD.store(0, Ordering::Relaxed);
  }
}

impl Drop for GrabHook {
  fn drop(&mut self) {
    self.terminate();
  }
}

/// Whether the keyboard hook should take key `vk`'s press or release from
/// apps: Escape during a gesture, which it cancels.
pub(crate) fn takes_key(vk: u16, is_keypress: bool) -> bool {
  if vk != VK_ESCAPE.0 {
    return false;
  }

  if !is_keypress {
    return ESCAPE_TAKEN.swap(false, Ordering::Relaxed);
  }

  // Repeats of a press already taken.
  if ESCAPE_TAKEN.load(Ordering::Relaxed) {
    return true;
  }

  let thread = GESTURE_THREAD.load(Ordering::Relaxed);
  if thread == 0 {
    return false;
  }

  // SAFETY: No preconditions; a thread gone since just fails the post.
  let is_posted = unsafe {
    PostThreadMessageW(thread, WM_CANCEL_GESTURE, WPARAM(0), LPARAM(0))
  }
  .is_ok();

  ESCAPE_TAKEN.store(is_posted, Ordering::Relaxed);
  is_posted
}

/// Hook procedure for mouse events, for use with `SetWindowsHookExW`.
///
/// Runs for every mouse event in the system: moves cost one atomic load
/// while no press is held, and anything but a left or right button is
/// passed on right away.
extern "system" fn hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: Passes on the arguments this hook was called with.
  let pass_on = || unsafe { CallNextHookEx(None, code, wparam, lparam) };

  #[allow(clippy::cast_possible_truncation)]
  let message = wparam.0 as u32;

  if code != 0
    || !matches!(
      message,
      WM_MOUSEMOVE
        | WM_LBUTTONDOWN
        | WM_LBUTTONUP
        | WM_RBUTTONDOWN
        | WM_RBUTTONUP
    )
  {
    return pass_on();
  }

  let should_swallow = HOOK.with(|hook| {
    hook
      .borrow()
      .as_ref()
      .is_some_and(|hook| hook.handle(message, lparam))
  });

  if should_swallow {
    LRESULT(1)
  } else {
    pass_on()
  }
}

/// Ends a held press as the input desktop switches: its release would be
/// made where the hook can't see it.
unsafe extern "system" fn on_desktop_switch(
  _hook: HWINEVENTHOOK,
  _event: u32,
  _hwnd: HWND,
  _object: i32,
  _child: i32,
  _thread: u32,
  _time: u32,
) {
  // An Escape release made there goes unseen too.
  ESCAPE_TAKEN.store(false, Ordering::Relaxed);

  HOOK.with(|hook| {
    if let Some(hook) = hook.borrow().as_ref() {
      hook.lose();
    }
  });
}

/// Handles the messages posted to the hook's thread.
fn on_message(msg: &MSG) {
  HOOK.with(|hook| {
    let hook = hook.borrow();
    let Some(hook) = hook.as_ref() else {
      return;
    };

    match msg.message {
      WM_MASK_MODIFIERS => mask_modifier_release(),
      WM_GESTURE_BEGAN => hook.began(),
      WM_CANCEL_GESTURE => {
        let ended = hook.shared.tracker().end_live();
        if let Some((window, live)) = ended {
          hook.end_live(window, live, GrabEnd::Cancelled);
        }
      }
      _ => {}
    }
  });
}

impl HookState {
  /// Returns whether to swallow the event.
  fn handle(&self, message: u32, lparam: LPARAM) -> bool {
    let is_held = self.shared.is_held.load(Ordering::Relaxed);
    if message == WM_MOUSEMOVE && !is_held {
      return false;
    }

    // SAFETY: For `WH_MOUSE_LL` with a non-negative code, `lparam` points
    // to a valid `MSLLHOOKSTRUCT` for the duration of the call.
    let input = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };

    // The WM's own input that lets it set the foreground window.
    if input.dwExtraInfo == FOREGROUND_INPUT_IDENTIFIER as usize {
      return false;
    }

    let point = Point {
      x: input.pt.x,
      y: input.pt.y,
    };

    match message {
      WM_LBUTTONDOWN => self.pressed(GrabButton::Left, &point),
      WM_RBUTTONDOWN => self.pressed(GrabButton::Right, &point),
      WM_LBUTTONUP if is_held => self.released(GrabButton::Left),
      WM_RBUTTONUP if is_held => self.released(GrabButton::Right),
      WM_MOUSEMOVE => self.moved(&point),
      _ => false,
    }
  }

  fn pressed(&self, button: GrabButton, point: &Point) -> bool {
    // Pressed again while held: its release went unseen.
    if self.shared.tracker().holds(button) {
      self.lose();
    }

    let Some((window, gesture)) = self.shared.press(
      button,
      point,
      || window_at(point),
      is_modifier_down,
    ) else {
      return false;
    };

    // SAFETY: No preconditions.
    GESTURE_THREAD
      .store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);

    if let Gesture::Resize(edges) = gesture {
      self.began_resize.set(Some((point.clone(), edges)));
    }

    let _ = self.shared.event_tx.send(GrabEvent::Started {
      window: platform_impl::NativeWindow::new(window.0).into(),
      gesture,
    });

    mask_modifiers_later();
    post_to_self(WM_GESTURE_BEGAN);
    true
  }

  fn released(&self, button: GrabButton) -> bool {
    let outcome = self.shared.tracker().released(button);

    match outcome {
      ReleaseOutcome::Pass => false,
      ReleaseOutcome::Swallow { window, live } => {
        self.end_held(window, live, GrabEnd::Released);
        true
      }
    }
  }

  fn moved(&self, point: &Point) -> bool {
    let outcome = {
      let mut tracker = self.shared.tracker();
      let cursor = if tracker.is_resizing() {
        cursor_position().unwrap_or_else(|| point.clone())
      } else {
        point.clone()
      };

      tracker.moved(point, &cursor, Instant::now())
    };

    match outcome {
      MoveOutcome::Pass => false,
      MoveOutcome::MoveWindow(window, position) => {
        self.mover.move_to(window, position);
        false
      }
      MoveOutcome::Measure { notify } => {
        if notify {
          let _ = self.shared.event_tx.send(GrabEvent::Resized);
        }
        true
      }
    }
  }

  /// Warps the cursor off the desktop's edge for a resize that began, as
  /// the hook itself has to return quickly.
  fn began(&self) {
    let Some((point, edges)) = self.began_resize.take() else {
      return;
    };

    // Released already.
    if !self.shared.tracker().is_resizing() {
      return;
    }

    let Some(target) = warp_target(&point, edges, is_on_desktop) else {
      return;
    };

    // Noted first: moves the hook sees from here on are measured from
    // where the cursor is warped to.
    self
      .shared
      .tracker()
      .warped(point, target.clone(), Instant::now());

    // SAFETY: No preconditions.
    if let Err(err) = unsafe { SetCursorPos(target.x, target.y) } {
      tracing::warn!("Failed to move the cursor off the edge: {err}");
    }
  }

  /// Ends the held press, if any, as one whose release the hook missed.
  fn lose(&self) {
    let dropped = self.shared.tracker().drop_held();

    if let Some((window, live)) = dropped {
      self.end_held(window, live, GrabEnd::Lost);
    }
  }

  /// Ends the held press on `window`, and its gesture if still under way.
  fn end_held(&self, window: WindowId, live: Option<Live>, end: GrabEnd) {
    self.shared.is_held.store(false, Ordering::Relaxed);

    match live {
      Some(live) => self.end_live(window, live, end),
      // Ended early: told anyway, so a listener waiting for the release
      // to unhook hears of it.
      None => {
        let _ = self.shared.event_tx.send(GrabEvent::Ended {
          window: platform_impl::NativeWindow::new(window.0).into(),
          end,
        });
      }
    }
  }

  /// Reports the end of `live` on `window`: once the window has made its
  /// last move, for a move, which a cancel puts back where it started.
  fn end_live(&self, window: WindowId, live: Live, end: GrabEnd) {
    GESTURE_THREAD.store(0, Ordering::Relaxed);

    match live {
      Live::Move { start, .. } => {
        let restore = (end == GrabEnd::Cancelled).then_some(start);
        self.mover.finish(Finish {
          window,
          restore,
          end,
        });
      }
      Live::Resize { .. } => {
        let _ = self.shared.event_tx.send(GrabEvent::Ended {
          window: platform_impl::NativeWindow::new(window.0).into(),
          end,
        });
      }
    }
  }
}

/// Moves windows grabbed to be moved, off the hook's thread.
#[derive(Default)]
struct Mover {
  state: Mutex<MoverState>,
  signal: Condvar,
}

#[derive(Default)]
struct MoverState {
  /// Newest position for the window's top-left corner.
  target: Option<(WindowId, Point)>,
  finish: Option<Finish>,
  quit: bool,
}

/// A move that ended, to be reported once the window has made its last
/// move.
struct Finish {
  window: WindowId,
  /// Where to put the window back, for a cancelled move.
  restore: Option<Point>,
  end: GrabEnd,
}

impl Mover {
  fn lock(&self) -> std::sync::MutexGuard<'_, MoverState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn move_to(&self, window: WindowId, position: Point) {
    self.lock().target = Some((window, position));
    self.signal.notify_one();
  }

  fn finish(&self, finish: Finish) {
    self.lock().finish = Some(finish);
    self.signal.notify_one();
  }

  fn quit(&self) {
    self.lock().quit = true;
    self.signal.notify_one();
  }

  fn run(&self, shared: &GrabShared) {
    loop {
      let (target, finish) = {
        let mut state = self.lock();

        while state.target.is_none()
          && state.finish.is_none()
          && !state.quit
        {
          state = self
            .signal
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner);
        }

        if state.quit {
          return;
        }

        let finish = state.finish.take();
        let target = state.target.take();

        // A cancelled move skips to where it started.
        let target = target.filter(|_| {
          finish
            .as_ref()
            .is_none_or(|finish| finish.restore.is_none())
        });

        (target, finish)
      };

      if let Some((window, position)) = target {
        move_window(window, &position);
      }

      if let Some(finish) = finish {
        if let Some(position) = &finish.restore {
          move_window(finish.window, position);
        }

        let _ = shared.event_tx.send(GrabEvent::Ended {
          window: platform_impl::NativeWindow::new(finish.window.0).into(),
          end: finish.end,
        });
      }
    }
  }
}

/// Moves `window`'s top-left corner to `position`, waiting on its app.
fn move_window(window: WindowId, position: &Point) {
  // SAFETY: A stale handle just fails the call.
  let result = unsafe {
    SetWindowPos(
      HWND(window.0),
      HWND(0),
      position.x,
      position.y,
      0,
      0,
      SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
    )
  };

  if let Err(err) = result {
    tracing::debug!("Failed to move a grabbed window: {err}");
  }
}

/// The top-level window under `point`, with its rect.
fn window_at(point: &Point) -> Option<(WindowId, Rect)> {
  let point = POINT {
    x: point.x,
    y: point.y,
  };

  // `WindowFromPoint` sends no hit-test messages to windows of other
  // threads, so a hung app can't stall the hook here.
  // SAFETY: No preconditions.
  let window = unsafe { WindowFromPoint(point) };
  if window.0 == 0 {
    return None;
  }

  // SAFETY: A stale handle just makes `GetAncestor` return `HWND(0)`.
  let root = unsafe { GetAncestor(window, GA_ROOT) };
  if root.0 == 0 {
    return None;
  }

  let mut rect = RECT::default();
  // SAFETY: `rect` outlives the call.
  unsafe { GetWindowRect(root, &raw mut rect) }.ok()?;

  Some((
    WindowId(root.0),
    Rect::from_ltrb(rect.left, rect.top, rect.right, rect.bottom),
  ))
}

fn is_modifier_down(modifier: GrabModifier) -> bool {
  let keys = match modifier {
    GrabModifier::Alt => [VK_LMENU, VK_RMENU],
    GrabModifier::Win => [VK_LWIN, VK_RWIN],
  };

  keys.into_iter().any(is_async_key_down)
}

fn is_on_desktop(point: &Point) -> bool {
  let point = POINT {
    x: point.x,
    y: point.y,
  };

  // SAFETY: No preconditions.
  !unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONULL) }.is_invalid()
}

fn cursor_position() -> Option<Point> {
  let mut point = POINT::default();
  // SAFETY: `point` outlives the call.
  unsafe { GetCursorPos(&raw mut point) }.ok()?;
  Some(Point {
    x: point.x,
    y: point.y,
  })
}

fn post_to_self(message: u32) {
  // SAFETY: No preconditions.
  let _ = unsafe {
    PostThreadMessageW(GetCurrentThreadId(), message, WPARAM(0), LPARAM(0))
  };
}
