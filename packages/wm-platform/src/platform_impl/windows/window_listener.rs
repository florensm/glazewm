use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
  },
  time::{Duration, Instant},
};

use tokio::sync::mpsc;
use windows::{
  core::w,
  Win32::{
    Foundation::HWND,
    System::SystemInformation::GetTickCount,
    UI::{
      Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
      WindowsAndMessaging::{
        ChangeWindowMessageFilterEx, GetDesktopWindow,
        RegisterShellHookWindow, RegisterWindowMessageW,
        EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
        EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
        EVENT_OBJECT_PARENTCHANGE, EVENT_OBJECT_REORDER,
        EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
        EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND,
        EVENT_SYSTEM_MOVESIZESTART, HSHELL_HIGHBIT, HSHELL_REDRAW,
        MSGFLT_ALLOW, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT,
        WINEVENT_SKIPOWNPROCESS,
      },
    },
  },
};

use super::NativeWindow;
use crate::{Dispatcher, WindowEvent, WindowId};

/// Shell hook notification for a window that is flashing its taskbar
/// button, which the `windows` crate doesn't define.
const HSHELL_FLASH: u32 = HSHELL_REDRAW | HSHELL_HIGHBIT;

/// Whether a `ZOrderChanged` event is queued and not yet taken by the WM.
///
/// Restacks come in bursts (one per window an animation frame restacks),
/// and the WM re-checks every overlay's z-order on each event. Its check
/// reads the current stacking, so one event queued after a burst is
/// equivalent to the whole burst.
static Z_ORDER_CHANGE_QUEUED: AtomicBool = AtomicBool::new(false);

/// Lets the next restack queue a `ZOrderChanged` event again. Called when
/// one is taken off the queue.
pub(crate) fn z_order_change_dequeued() {
  Z_ORDER_CHANGE_QUEUED.store(false, Ordering::Release);
}

thread_local! {
  /// Sender for window events. For use with hook procedure.
  static EVENT_TX: OnceLock<mpsc::UnboundedSender<WindowEvent>> = const { OnceLock::new() };
}

/// Oldest a window event's own timestamp is believed: anything older is
/// a `GetTickCount` wrap, and the event is dated when it arrived instead.
const MAX_EVENT_AGE: Duration = Duration::from_secs(60);

/// Platform-specific implementation of [`WindowEventNotification`]: when
/// the OS raised the event.
#[derive(Clone, Debug)]
pub struct WindowEventNotificationInner {
  occurred_at: Instant,
}

impl WindowEventNotificationInner {
  /// For an event the OS stamped with `event_time`, a `GetTickCount`
  /// reading, so accurate to the system timer's tick (up to ~16ms).
  fn raised_at(event_time: u32) -> Self {
    let now = Instant::now();

    // SAFETY: No preconditions.
    let age = unsafe { GetTickCount() }.wrapping_sub(event_time);
    let occurred_at = Some(Duration::from_millis(u64::from(age)))
      .filter(|age| *age <= MAX_EVENT_AGE)
      .and_then(|age| now.checked_sub(age))
      .unwrap_or(now);

    Self { occurred_at }
  }

  /// See [`WindowEventNotification::occurred_at`].
  pub(crate) fn occurred_at(&self) -> Instant {
    self.occurred_at
  }
}

/// Platform-specific implementation of [`WindowListener`].
#[derive(Debug)]
pub(crate) struct WindowListener {
  hook_handles: Vec<HWINEVENTHOOK>,
  shell_hook_callback_id: Option<usize>,
  dispatcher: Dispatcher,
}

impl WindowListener {
  /// Implements [`WindowListener::new`].
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    // Only taskbar-flash detection depends on the shell hook, so the WM
    // keeps running without it (e.g. under a replacement shell or Wine).
    let shell_hook_callback_id =
      match Self::hook_shell_events(event_tx.clone(), dispatcher) {
        Ok(id) => Some(id),
        Err(err) => {
          tracing::warn!(
            "Shell hook unavailable; attention requests won't be \
             detected: {err}"
          );
          None
        }
      };

    let hook_handles = dispatcher.dispatch_sync(move || {
      EVENT_TX.with(|lock| lock.set(event_tx)).map_err(|_| {
        crate::Error::Platform(
          "Window event sender already set.".to_string(),
        )
      })?;

      Self::hook_win_events()
    })??;

    Ok(Self {
      hook_handles,
      shell_hook_callback_id,
      dispatcher: dispatcher.clone(),
    })
  }

  /// Implements [`WindowListener::terminate`].
  pub(crate) fn terminate(&mut self) {
    // `UnhookWinEvent` fails from any thread but the one that set the
    // hooks. Called from the WM's thread, it left them firing until the
    // event loop stopped, every event a warning once the channel closed.
    // Once the event loop has stopped, nothing runs them any more, and its
    // thread's exit removes them.
    let handles = std::mem::take(&mut self.hook_handles);
    let _ = self.dispatcher.dispatch_sync(move || {
      for handle in handles {
        // SAFETY: Each handle was returned by `SetWinEventHook` on this
        // thread and is unhooked once.
        let _ = unsafe { UnhookWinEvent(handle) };
      }
    });

    // The shell hook itself is released when the event loop's message
    // window is destroyed; `DeregisterShellHookWindow` is undocumented
    // and isn't exposed by the `windows` crate.
    if let Some(id) = self.shell_hook_callback_id.take() {
      let _ = self.dispatcher.deregister_wndproc_callback(id);
    }
  }

  /// Subscribes to shell hook notifications for the event loop's message
  /// window.
  ///
  /// This is the only way to observe a window asking for attention;
  /// `SetWinEventHook` has no equivalent event. The shell broadcasts these
  /// notifications via a registered window message, so a window procedure
  /// callback is used rather than the hook procedure below.
  ///
  /// Returns the ID of the registered window procedure callback.
  fn hook_shell_events(
    event_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<usize> {
    let message_window = HWND(dispatcher.message_window_handle());

    let shell_hook_message = dispatcher
      .dispatch_sync(move || {
        // SAFETY: `message_window` is a valid window handle owned by the
        // event loop thread, which this closure runs on.
        unsafe {
          let message = RegisterWindowMessageW(w!("SHELLHOOK"));

          // The WM is commonly run elevated while the shell isn't, so the
          // notifications would otherwise be dropped by UIPI.
          let _ = ChangeWindowMessageFilterEx(
            message_window,
            message,
            MSGFLT_ALLOW,
            None,
          );

          RegisterShellHookWindow(message_window)
            .as_bool()
            .then_some(message)
        }
      })?
      .ok_or_else(|| {
        crate::Error::Platform(
          "Failed to register shell hook window.".to_string(),
        )
      })?;

    dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd, message, wparam, lparam| {
        if message != shell_hook_message {
          return None;
        }

        #[allow(clippy::cast_possible_truncation)]
        if wparam as u32 == HSHELL_FLASH {
          let event = WindowEvent::AttentionRequested {
            window: NativeWindow::new(lparam).into(),
            notification: crate::WindowEventNotification(None),
          };

          if let Err(err) = event_tx.send(event) {
            tracing::warn!("Failed to send window event: {}.", err);
          }
        }

        Some(0)
      },
    ))
  }

  /// Creates several window event hooks via `SetWinEventHook`.
  ///
  /// Separate hooks are created per event range, which is more performant
  /// than a single hook covering all events.
  fn hook_win_events() -> crate::Result<Vec<HWINEVENTHOOK>> {
    let event_ranges = [
      (EVENT_OBJECT_DESTROY, EVENT_OBJECT_REORDER),
      (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND),
      (EVENT_SYSTEM_MOVESIZESTART, EVENT_SYSTEM_MOVESIZEEND),
      (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
      (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE),
      // Own hook: the events between it and `EVENT_OBJECT_NAMECHANGE`
      // include `EVENT_OBJECT_VALUECHANGE`, which controls fire
      // constantly.
      (EVENT_OBJECT_PARENTCHANGE, EVENT_OBJECT_PARENTCHANGE),
      (EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED),
    ];

    event_ranges
      .iter()
      .try_fold(Vec::new(), |mut handles, (min, max)| {
        // Create a window hook for the event range.
        let hook_handle = unsafe {
          SetWinEventHook(
            *min,
            *max,
            None,
            Some(Self::window_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
          )
        };

        if hook_handle.is_invalid() {
          return Err(crate::Error::Platform(
            "Failed to set window event hook.".to_string(),
          ));
        }

        handles.push(hook_handle);
        Ok(handles)
      })
  }

  /// Callback passed to `SetWinEventHook`.
  ///
  /// This function is called on selected window events, and forwards them
  /// through an MPSC channel.
  extern "system" fn window_event_proc(
    _hook: HWINEVENTHOOK,
    event_type: u32,
    handle: HWND,
    id_object: i32,
    id_child: i32,
    _event_thread: u32,
    event_time: u32,
  ) {
    // Top-level restacks are reported on the desktop window, not on the
    // windows that moved; restacks of other apps' child windows are
    // reported on their parent and don't matter.
    if event_type == EVENT_OBJECT_REORDER {
      // SAFETY: No preconditions.
      if handle == unsafe { GetDesktopWindow() }
        && !Z_ORDER_CHANGE_QUEUED.swap(true, Ordering::AcqRel)
        && !Self::send_event(WindowEvent::ZOrderChanged {
          notification: crate::WindowEventNotification(None),
        })
      {
        // Nothing was queued, so nothing will clear the flag.
        Z_ORDER_CHANGE_QUEUED.store(false, Ordering::Release);
      }
      return;
    }

    // Check whether the event is associated with a window object rather
    // than a UI control.
    let is_window_event =
      id_object == OBJID_WINDOW.0 && id_child == 0 && handle != HWND(0);

    if !is_window_event {
      return;
    }

    // A companion shown makes the kept scan for them stale.
    if event_type == EVENT_OBJECT_SHOW {
      crate::companion::note_shown(handle);
    }

    let notification = crate::WindowEventNotification(Some(
      WindowEventNotificationInner::raised_at(event_time),
    ));

    let event = match event_type {
      EVENT_OBJECT_DESTROY => WindowEvent::Destroyed {
        window_id: WindowId(handle.0),
        notification,
      },
      EVENT_SYSTEM_FOREGROUND => WindowEvent::Focused {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED => WindowEvent::Hidden {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      EVENT_OBJECT_LOCATIONCHANGE => WindowEvent::MovedOrResized {
        window: NativeWindow::new(handle.0).into(),
        is_interactive_start: false,
        is_interactive_end: false,
        notification,
      },
      EVENT_SYSTEM_MINIMIZESTART => WindowEvent::Minimized {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      EVENT_SYSTEM_MINIMIZEEND => WindowEvent::MinimizeEnded {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      EVENT_SYSTEM_MOVESIZESTART => WindowEvent::MovedOrResized {
        window: NativeWindow::new(handle.0).into(),
        is_interactive_start: true,
        is_interactive_end: false,
        notification,
      },
      EVENT_SYSTEM_MOVESIZEEND => WindowEvent::MovedOrResized {
        window: NativeWindow::new(handle.0).into(),
        is_interactive_start: false,
        is_interactive_end: true,
        notification,
      },
      EVENT_OBJECT_SHOW | EVENT_OBJECT_UNCLOAKED => WindowEvent::Shown {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      EVENT_OBJECT_NAMECHANGE => WindowEvent::TitleChanged {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      EVENT_OBJECT_PARENTCHANGE => WindowEvent::Reparented {
        window: NativeWindow::new(handle.0).into(),
        notification,
      },
      _ => return,
    };

    Self::send_event(event);
  }

  /// Queues `event` for the WM, returning whether it was queued.
  fn send_event(event: WindowEvent) -> bool {
    let Some(event_tx) = EVENT_TX.with(|lock| lock.get().cloned()) else {
      return false;
    };

    crate::perf::mark_event_queued(crate::perf::EventKind::Window);

    if let Err(err) = event_tx.send(event) {
      tracing::warn!("Failed to send window event: {}.", err);
      return false;
    }

    true
  }
}

impl Drop for WindowListener {
  fn drop(&mut self) {
    self.terminate();
  }
}
