use std::cell::Cell;

use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{
      VIRTUAL_KEY, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL,
      VK_RMENU, VK_RSHIFT, VK_RWIN,
    },
    WindowsAndMessaging::{
      CallNextHookEx, MSLLHOOKSTRUCT, WM_MOUSEHWHEEL, WM_MOUSEWHEEL,
    },
  },
};

use super::{
  is_async_key_down, mask_modifier_release, mask_modifiers_later,
  MouseHookThread, WM_MASK_MODIFIERS,
};
use crate::{Key, KeyCode};

/// Callback stored in [`HOOK`] for wheel events.
type HookCallback = Box<dyn FnMut(WheelEvent) -> bool>;

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
    let is_down = |key_code: u16| is_async_key_down(VIRTUAL_KEY(key_code));

    match key {
      Key::Win => is_down(VK_LWIN.0) || is_down(VK_RWIN.0),
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
#[derive(Debug)]
pub struct WheelHook {
  thread: MouseHookThread,
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
    let thread = MouseHookThread::spawn(
      "wheel-hook",
      move || HOOK.with(|state| state.set(Some(Box::new(callback)))),
      Some(Self::hook_proc),
      |msg| {
        if msg.message == WM_MASK_MODIFIERS {
          mask_modifier_release();
        }
      },
    )?;

    Ok(Self { thread })
  }

  /// Terminates the hook by unregistering it and ending its thread.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.thread.terminate()
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

    mask_modifiers_later();

    LRESULT(1)
  }
}
