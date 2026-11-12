use std::{
  sync::mpsc,
  thread::{self, JoinHandle},
};

use windows::Win32::{
  Foundation::{HINSTANCE, LPARAM, WPARAM},
  System::Threading::GetCurrentThreadId,
  UI::{
    Input::KeyboardAndMouse::{
      GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD,
      KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
      VK_LMENU, VK_LWIN, VK_RMENU, VK_RWIN,
    },
    WindowsAndMessaging::{
      GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW,
      UnhookWindowsHookEx, HOOKPROC, MSG, PM_NOREMOVE, WH_MOUSE_LL,
      WM_APP, WM_QUIT,
    },
  },
};

use super::SENT_KEY_MARKER;

/// Posted by a hook to its own thread, to tap [`MASK_KEY`] once the input
/// it swallowed is done with (see [`mask_modifiers_later`]).
pub(crate) const WM_MASK_MODIFIERS: u32 = WM_APP + 1;

/// An unassigned key, whose tap keeps a later release of Alt or Win from
/// counting as a lone tap of it.
const MASK_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);

/// A system-wide low-level mouse hook on a thread of its own.
///
/// Every mouse event in the system waits on a low-level hook, so the hook
/// gets a thread that does nothing else: a busy one would make the cursor
/// stutter.
#[derive(Debug)]
pub(crate) struct MouseHookThread {
  thread_id: u32,
  thread: Option<JoinHandle<()>>,
}

impl MouseHookThread {
  /// Spawns a thread named `name` that runs `init`, hooks the mouse with
  /// `hook_proc`, and hands every message posted to it to `on_message`
  /// until terminated.
  ///
  /// `init` runs on the hook's thread, before it is hooked, e.g. to set
  /// up the thread-local state `hook_proc` reads.
  pub(crate) fn spawn(
    name: &str,
    init: impl FnOnce() + Send + 'static,
    hook_proc: HOOKPROC,
    on_message: fn(&MSG),
  ) -> crate::Result<Self> {
    let (started_tx, started_rx) = mpsc::channel();

    let thread =
      thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
          init();
          Self::run(hook_proc, on_message, &started_tx);
        })?;

    match started_rx.recv() {
      Ok(Ok(thread_id)) => Ok(Self {
        thread_id,
        thread: Some(thread),
      }),
      Ok(Err(err)) => {
        let _ = thread.join();
        Err(err)
      }
      Err(_) => Err(crate::Error::Thread(format!(
        "Mouse hook thread `{name}` exited before hooking the mouse."
      ))),
    }
  }

  /// Id of the hook's thread, to post messages to.
  pub(crate) fn thread_id(&self) -> u32 {
    self.thread_id
  }

  /// Unhooks the mouse and ends the thread.
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    let Some(thread) = self.thread.take() else {
      return Ok(());
    };

    // SAFETY: No preconditions; the thread's message queue exists from
    // before `spawn` returned.
    unsafe {
      PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0))
    }?;

    thread.join().map_err(|_| {
      crate::Error::Thread("Mouse hook thread panicked.".to_string())
    })
  }

  /// Hooks the mouse and pumps the thread's messages until `WM_QUIT`,
  /// without which a low-level hook is never called.
  ///
  /// Sends the thread's id through `started_tx` once hooked.
  fn run(
    hook_proc: HOOKPROC,
    on_message: fn(&MSG),
    started_tx: &mpsc::Sender<crate::Result<u32>>,
  ) {
    let mut msg = MSG::default();

    // Creates the thread's message queue, so that `WM_QUIT` can be posted
    // to it from the moment `spawn` returns.
    // SAFETY: `msg` outlives the call.
    let _ = unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE) };

    // SAFETY: Hook procedures passed here are plain functions, valid for
    // the life of the process.
    let hooked = unsafe {
      SetWindowsHookExW(WH_MOUSE_LL, hook_proc, HINSTANCE::default(), 0)
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
      on_message(&msg);
    }

    // SAFETY: `handle` is the hook installed above.
    if let Err(err) = unsafe { UnhookWindowsHookEx(handle) } {
      tracing::warn!("Failed to unhook the mouse: {err}");
    }
  }
}

impl Drop for MouseHookThread {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}

/// Whether the key `vk` is held down now.
///
/// The set high bit, i.e. a negative state, is whether the key is down now
/// rather than as of the last message this thread read, which a hook
/// thread may never have.
pub(crate) fn is_async_key_down(vk: VIRTUAL_KEY) -> bool {
  // SAFETY: No preconditions.
  unsafe { GetAsyncKeyState(i32::from(vk.0)) < 0 }
}

/// Has the calling hook thread tap [`MASK_KEY`] once its hook returns, if
/// Alt or Win is held.
///
/// Without the mouse input a hook swallowed, the window under the cursor
/// sees a lone tap of the held modifiers once they are released, which for
/// Alt opens its menu bar and for Win the Start menu. The tap is posted,
/// as the hook must return quickly; the thread's `on_message` has to call
/// [`mask_modifier_release`] for [`WM_MASK_MODIFIERS`].
pub(crate) fn mask_modifiers_later() {
  let is_held = [VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN]
    .into_iter()
    .any(is_async_key_down);

  if is_held {
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
}

/// Taps [`MASK_KEY`], marked so that the keyboard hook lets it through.
pub(crate) fn mask_modifier_release() {
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
