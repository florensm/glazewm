use std::cell::Cell;

use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, GetAncestor, WindowFromPoint, GA_ROOT, MSLLHOOKSTRUCT,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
  },
};

use super::MouseHookThread;

/// Callback stored in [`HOOK`] for button presses.
type HookCallback = Box<dyn FnMut(HWND) -> bool>;

thread_local! {
  /// The callback of the hook's thread, called with the top-level window
  /// under the cursor for every press of a mouse button. Returns `true`
  /// to swallow the press.
  static HOOK: Cell<Option<HookCallback>> = Cell::default();

  /// Buttons whose press was swallowed, as [`button_bit`]s, so that
  /// their release is swallowed too.
  static SWALLOWED: Cell<u8> = const { Cell::new(0) };
}

/// A system-wide low-level mouse hook that can keep presses of mouse
/// buttons, and their releases, from the window under the cursor.
#[derive(Debug)]
pub struct ClickHook {
  thread: MouseHookThread,
}

impl ClickHook {
  /// Creates an instance of `ClickHook`.
  ///
  /// `callback` is called with the top-level window under the cursor for
  /// every press of a mouse button, and returns `true` to keep the press
  /// and its release from that window.
  pub fn new<F>(callback: F) -> crate::Result<Self>
  where
    F: FnMut(HWND) -> bool + Send + 'static,
  {
    let thread = MouseHookThread::spawn(
      "click-hook",
      move || HOOK.with(|state| state.set(Some(Box::new(callback)))),
      Some(Self::hook_proc),
      |_| {},
    )?;

    Ok(Self { thread })
  }

  /// Terminates the hook by unregistering it and ending its thread.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.thread.terminate()
  }

  /// Hook procedure for mouse events, for use with `SetWindowsHookExW`.
  ///
  /// Runs for every mouse event in the system, so anything but a press or
  /// release of a button is passed on right away.
  extern "system" fn hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    // SAFETY: Passes on the arguments this hook was called with.
    let pass_on = || unsafe { CallNextHookEx(None, code, wparam, lparam) };

    if code != 0 {
      return pass_on();
    }

    #[allow(clippy::cast_possible_truncation)]
    let message = wparam.0 as u32;

    // SAFETY: For `WH_MOUSE_LL` with a non-negative code, `lparam` points
    // to a valid `MSLLHOOKSTRUCT` for the duration of the call.
    let input = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };

    let Some((bit, is_press)) = button_bit(message, input.mouseData)
    else {
      return pass_on();
    };

    let swallowed = SWALLOWED.get();

    let should_swallow = if is_press {
      let is_guarded = Self::is_guarded_at(input);
      if is_guarded {
        SWALLOWED.set(swallowed | bit);
      }
      is_guarded
    } else {
      SWALLOWED.set(swallowed & !bit);
      swallowed & bit != 0
    };

    if should_swallow {
      LRESULT(1)
    } else {
      pass_on()
    }
  }

  /// Whether the callback guards the top-level window under the cursor.
  fn is_guarded_at(input: MSLLHOOKSTRUCT) -> bool {
    // `WindowFromPoint` sends no hit-test messages to windows of other
    // threads, so a hung app can't stall the hook here.
    // SAFETY: No preconditions.
    let window = unsafe { WindowFromPoint(input.pt) };
    if window.0 == 0 {
      return false;
    }

    // SAFETY: A stale handle just makes `GetAncestor` return `HWND(0)`.
    let root = unsafe { GetAncestor(window, GA_ROOT) };
    if root.0 == 0 {
      return false;
    }

    HOOK.with(|state| {
      let Some(mut callback) = state.take() else {
        return false;
      };

      let result = callback(root);
      state.set(Some(callback));
      result
    })
  }
}

/// The bit of the button that `message` presses or releases, and whether
/// it presses it. `None` for anything but a button press or release.
fn button_bit(message: u32, mouse_data: u32) -> Option<(u8, bool)> {
  match message {
    WM_LBUTTONDOWN => Some((1, true)),
    WM_LBUTTONUP => Some((1, false)),
    WM_RBUTTONDOWN => Some((1 << 1, true)),
    WM_RBUTTONUP => Some((1 << 1, false)),
    WM_MBUTTONDOWN => Some((1 << 2, true)),
    WM_MBUTTONUP => Some((1 << 2, false)),
    WM_XBUTTONDOWN | WM_XBUTTONUP => {
      // The high word of `mouseData` is which X button, 1 or 2.
      let bit = if mouse_data >> 16 == 1 {
        1 << 3
      } else {
        1 << 4
      };
      Some((bit, message == WM_XBUTTONDOWN))
    }
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn presses_and_releases_share_a_bit() {
    let pairs = [
      (WM_LBUTTONDOWN, WM_LBUTTONUP, 0),
      (WM_RBUTTONDOWN, WM_RBUTTONUP, 0),
      (WM_MBUTTONDOWN, WM_MBUTTONUP, 0),
      (WM_XBUTTONDOWN, WM_XBUTTONUP, 1 << 16),
      (WM_XBUTTONDOWN, WM_XBUTTONUP, 2 << 16),
    ];

    let mut bits = 0;
    for (press, release, data) in pairs {
      let (press_bit, is_press) = button_bit(press, data).unwrap();
      assert!(is_press);
      assert_eq!(button_bit(release, data), Some((press_bit, false)));
      assert_eq!(bits & press_bit, 0, "each button has a bit of its own");
      bits |= press_bit;
    }
  }

  #[test]
  fn other_messages_are_not_buttons() {
    use windows::Win32::UI::WindowsAndMessaging::{
      WM_MOUSEMOVE, WM_MOUSEWHEEL,
    };

    assert_eq!(button_bit(WM_MOUSEMOVE, 0), None);
    assert_eq!(button_bit(WM_MOUSEWHEEL, 0), None);
  }
}
