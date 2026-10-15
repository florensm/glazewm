use std::{
  cell::Cell,
  sync::mpsc,
  thread::{self, JoinHandle},
};

use windows::Win32::{
  Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
  System::Threading::GetCurrentThreadId,
  UI::{
    Input::KeyboardAndMouse::{
      GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD,
      KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
      VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU,
      VK_RSHIFT, VK_RWIN,
    },
    WindowsAndMessaging::{
      CallNextHookEx, GetMessageW, PeekMessageW, PostThreadMessageW,
      SetWindowsHookExW, UnhookWindowsHookEx, MSG, MSLLHOOKSTRUCT,
      PM_NOREMOVE, WH_MOUSE_LL, WM_APP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL,
      WM_QUIT,
    },
  },
};

use super::SENT_KEY_MARKER;
use crate::{Key, KeyCode};

/// Callback stored in [`HOOK`] for wheel events.
type HookCallback = Box<dyn FnMut(WheelEvent) -> bool>;

/// Posted by the hook to its own thread, to tap [`MASK_KEY`] once the
/// turn it swallowed is done with.
const WM_MASK_MODIFIERS: u32 = WM_APP + 1;

/// An unassigned key, whose tap keeps a later release of Alt or Win from
/// counting as a lone tap of it.
const MASK_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);

thread_local! {
  /// The callback of the hook's thread, called for every turn of a mouse
  /// wheel. Returns `true` to swallow the turn.
  static HOOK: Cell<Option<HookCallback>> = Cell::default();
}

/// A turn of a mouse wheel.
#[derive(Clone, Copy, Debug)]
pub struct WheelEvent {
  /// Whether it is the horizontal wheel, or a tilt of the vertical one.
  pub is_horizontal: bool,

  /// How far it turned, in 120ths of a notch: positive is up, or right.
  pub delta: i16,
}

impl WheelEvent {
  /// Whether `key` is held down at the time of the turn.
  pub fn is_key_down(key: Key) -> bool {
    // The set high bit, i.e. a negative state, is whether the key is down
    // now rather than as of the last message this thread read, which a
    // hook thread may never have.
    // SAFETY: No preconditions.
    let is_down =
      |key_code: u16| unsafe { GetAsyncKeyState(i32::from(key_code)) } < 0;

    match key {
      Key::Cmd | Key::Win => is_down(VK_LWIN.0) || is_down(VK_RWIN.0),
      Key::Alt => is_down(VK_LMENU.0) || is_down(VK_RMENU.0),
      Key::Ctrl => is_down(VK_LCONTROL.0) || is_down(VK_RCONTROL.0),
      Key::Shift => is_down(VK_LSHIFT.0) || is_down(VK_RSHIFT.0),
      _ => {
        KeyCode::try_from(key).is_ok_and(|key_code| is_down(key_code.0))
      }
    }
  }
}

/// A system-wide low-level mouse hook that only reports wheel turns.
///
/// Runs on a thread of its own: every mouse event in the system waits on
/// a low-level hook, so a busy thread would make the cursor stutter.
#[derive(Debug)]
pub struct WheelHook {
  thread_id: u32,
  thread: Option<JoinHandle<()>>,
}

impl WheelHook {
  /// Creates an instance of `WheelHook`.
  ///
  /// `callback` is called for every turn of a mouse wheel, and returns
  /// `true` to keep it from the window under the cursor.
  pub fn new<F>(callback: F) -> crate::Result<Self>
  where
    F: FnMut(WheelEvent) -> bool + Send + 'static,
  {
    let (started_tx, started_rx) = mpsc::channel();

    let thread = thread::Builder::new()
      .name("wheel-hook".to_string())
      .spawn(move || Self::run(Box::new(callback), &started_tx))?;

    match started_rx.recv() {
      Ok(Ok(thread_id)) => Ok(Self {
        thread_id,
        thread: Some(thread),
      }),
      Ok(Err(err)) => {
        let _ = thread.join();
        Err(err)
      }
      Err(_) => Err(crate::Error::Thread(
        "Wheel hook thread exited before hooking the mouse.".to_string(),
      )),
    }
  }

  /// Terminates the hook by unregistering it and ending its thread.
  pub fn terminate(&mut self) -> crate::Result<()> {
    let Some(thread) = self.thread.take() else {
      return Ok(());
    };

    // SAFETY: No preconditions; the thread's message queue exists from
    // before `new` returned.
    unsafe {
      PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0))
    }?;

    thread.join().map_err(|_| {
      crate::Error::Thread("Wheel hook thread panicked.".to_string())
    })
  }

  /// Hooks the mouse and pumps the thread's messages until `WM_QUIT`,
  /// without which a low-level hook is never called.
  ///
  /// Sends the thread's id through `started_tx` once hooked.
  fn run(
    callback: HookCallback,
    started_tx: &mpsc::Sender<crate::Result<u32>>,
  ) {
    HOOK.with(|state| state.set(Some(callback)));

    let mut msg = MSG::default();

    // Creates the thread's message queue, so that `WM_QUIT` can be posted
    // to it from the moment `new` returns.
    // SAFETY: `msg` outlives the call.
    let _ = unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE) };

    // SAFETY: `hook_proc` stays valid for the life of the process.
    let hooked = unsafe {
      SetWindowsHookExW(
        WH_MOUSE_LL,
        Some(Self::hook_proc),
        HINSTANCE::default(),
        0,
      )
    };

    let handle = match hooked {
      Ok(handle) => handle,
      Err(err) => {
        let _ = started_tx.send(Err(err.into()));
        return;
      }
    };

    // SAFETY: No preconditions.
    let _ = started_tx.send(Ok(unsafe { GetCurrentThreadId() }));

    // `GetMessageW` returns 0 for `WM_QUIT` and -1 on failure.
    // SAFETY: `msg` outlives the call.
    while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
      if msg.message == WM_MASK_MODIFIERS {
        mask_modifier_release();
      }
    }

    // SAFETY: `handle` is the hook installed above.
    if let Err(err) = unsafe { UnhookWindowsHookEx(handle) } {
      tracing::warn!("Failed to unhook the mouse wheel: {err}");
    }
  }

  /// Hook procedure for mouse events, for use with `SetWindowsHookExW`.
  ///
  /// Runs for every mouse event in the system, so anything but a wheel
  /// turn is passed on right away.
  extern "system" fn hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    #[allow(clippy::cast_possible_truncation)]
    let message = wparam.0 as u32;

    if code != 0 || (message != WM_MOUSEWHEEL && message != WM_MOUSEHWHEEL)
    {
      // SAFETY: Passes on the arguments this hook was called with.
      return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // SAFETY: For `WH_MOUSE_LL` with a non-negative code, `lparam` points
    // to a valid `MSLLHOOKSTRUCT` for the duration of the call.
    let input = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };

    // The high word of `mouseData` is the signed wheel delta.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let event = WheelEvent {
      is_horizontal: message == WM_MOUSEHWHEEL,
      delta: (input.mouseData >> 16) as u16 as i16,
    };

    let should_swallow = HOOK.with(|state| {
      let Some(mut callback) = state.take() else {
        return false;
      };

      let result = callback(event);
      state.set(Some(callback));
      result
    });

    if !should_swallow {
      // SAFETY: Passes on the arguments this hook was called with.
      return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // Without the turn, the window under the cursor sees a lone tap of
    // the held modifiers once they are released, which for Alt opens its
    // menu bar and for Win the Start menu. Masked after the hook returns,
    // as it must return quickly.
    if WheelEvent::is_key_down(Key::Alt)
      || WheelEvent::is_key_down(Key::Win)
    {
      // SAFETY: No preconditions.
      let _ = unsafe {
        PostThreadMessageW(
          GetCurrentThreadId(),
          WM_MASK_MODIFIERS,
          WPARAM(0),
          LPARAM(0),
        )
      };
    }

    LRESULT(1)
  }
}

impl Drop for WheelHook {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}

/// Taps [`MASK_KEY`], marked so that the keyboard hook lets it through.
fn mask_modifier_release() {
  let input = |flags| INPUT {
    r#type: INPUT_KEYBOARD,
    Anonymous: INPUT_0 {
      ki: KEYBDINPUT {
        wVk: MASK_KEY,
        wScan: 0,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: SENT_KEY_MARKER,
      },
    },
  };

  let inputs = [input(KEYBD_EVENT_FLAGS(0)), input(KEYEVENTF_KEYUP)];

  // SAFETY: `inputs` outlives the call and `cbsize` matches `INPUT`.
  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let sent =
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };

  if sent as usize != inputs.len() {
    tracing::warn!("Failed to mask the release of a held modifier key.");
  }
}
