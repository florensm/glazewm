use std::{
  sync::{mpsc, OnceLock},
  thread,
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, DispatchMessageW, GetClassNameW,
      GetMessageW, GetWindow, PostThreadMessageW,
      SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
      ShowWindow, GWLP_HWNDPARENT, GW_HWNDPREV, LWA_ALPHA, MSG,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSENDCHANGING,
      SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, WM_QUIT, WS_EX_LAYERED,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{window_class, Rect, SurrogateBatch};

/// Which overlay an [`OverlayWindow`] backs; each gets its own window
/// class so the overlays can be told apart in z-order dumps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlayKind {
  Backdrop,
  Border,

  /// Sits *above* its window, covering it with a recolored copy.
  ColorTheme,
}

impl OverlayKind {
  fn class_name(self) -> PCWSTR {
    match self {
      Self::Backdrop => w!("GlazeWM_BackdropOverlay"),
      Self::Border => w!("GlazeWM_BorderOverlay"),
      Self::ColorTheme => w!("GlazeWM_ColorThemeOverlay"),
    }
  }

  /// Whether `hwnd` is a backdrop or border overlay window, i.e. one kept
  /// behind its window.
  pub(crate) fn is_overlay(hwnd: HWND) -> bool {
    let mut buf = [0u16; 32];
    // SAFETY: `buf` outlives the call; a stale `hwnd` just returns 0.
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    let name = &buf[..usize::try_from(len).unwrap_or(0)];

    [Self::Backdrop, Self::Border].iter().any(|kind| {
      let class_name = kind.class_name();
      // SAFETY: `class_name` returns a static, null-terminated literal.
      unsafe { class_name.as_wide() == name }
    })
  }

  fn registered(self) -> &'static OnceLock<()> {
    static BACKDROP: OnceLock<()> = OnceLock::new();
    static BORDER: OnceLock<()> = OnceLock::new();
    static COLOR_THEME: OnceLock<()> = OnceLock::new();

    match self {
      Self::Backdrop => &BACKDROP,
      Self::Border => &BORDER,
      Self::ColorTheme => &COLOR_THEME,
    }
  }

  /// Prefix for log and error messages.
  fn label(self) -> &'static str {
    match self {
      Self::Backdrop => "Backdrop overlay",
      Self::Border => "Border overlay",
      Self::ColorTheme => "Color theme overlay",
    }
  }
}

/// The Win32 window behind an overlay: a click-through popup, rendered
/// entirely by a composition visual tree, kept directly behind (or, for
/// [`OverlayKind::ColorTheme`], directly above) an anchor window in
/// z-order.
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

  /// The thread that created the window and pumps its messages, if not
  /// the WM's own (see
  /// [`create_on_own_thread`](Self::create_on_own_thread)).
  thread: Option<WindowThread>,

  /// Current owner window set by [`set_owner`](Self::set_owner), or 0.
  owner: isize,
}

impl OverlayWindow {
  /// Creates the window at `rect`, hidden, to be shown behind `anchor` by
  /// [`place`](Self::place) once its visual tree exists.
  pub(crate) fn create(
    kind: OverlayKind,
    rect: &Rect,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let hwnd = Self::create_hwnd(kind, rect)?;

    Ok(Self {
      hwnd: hwnd.0,
      kind,
      anchor: anchor.0,
      is_visible: false,
      thread: None,
      owner: 0,
    })
  }

  /// Like [`create`](Self::create), but on a dedicated thread that pumps
  /// the window's messages, which [`set_owner`](Self::set_owner) needs.
  pub(crate) fn create_on_own_thread(
    kind: OverlayKind,
    rect: &Rect,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let (hwnd, thread) = WindowThread::spawn(kind, rect)?;

    Ok(Self {
      hwnd: hwnd.0,
      kind,
      anchor: anchor.0,
      is_visible: false,
      thread: Some(thread),
      owner: 0,
    })
  }

  /// Creates the hidden window on the calling thread.
  fn create_hwnd(kind: OverlayKind, rect: &Rect) -> crate::Result<HWND> {
    window_class::ensure_class_registered(
      kind.registered(),
      kind.class_name(),
      window_class::default_wnd_proc,
    );

    // `WS_EX_TRANSPARENT` is mandatory, not cosmetic: most overlays live
    // on the WM's thread, which never pumps a Win32 message queue, so
    // Windows treats them as hung. A hit-testable overlay shows the busy
    // cursor and swallows clicks. `WS_EX_NOREDIRECTIONBITMAP` skips the
    // GDI surface the composition visual tree replaces.
    //
    // An overlay above its window additionally needs `WS_EX_LAYERED`: only
    // the combination of the two makes a top-level window invisible to
    // mouse hit-testing, so input reaches the window underneath.
    let mut ex_style = WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT
      | WS_EX_NOREDIRECTIONBITMAP;

    if kind == OverlayKind::ColorTheme {
      ex_style |= WS_EX_LAYERED;
    }

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

    if ex_style.contains(WS_EX_LAYERED) {
      // A layered window stays invisible until its attributes are set;
      // fully opaque, since the visual tree supplies its own alpha.
      // SAFETY: `hwnd` was just created by this thread.
      if let Err(err) = unsafe {
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
      } {
        // SAFETY: As above.
        let _ = unsafe { DestroyWindow(hwnd) };
        return Err(err.into());
      }
    }

    Ok(hwnd)
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

    // The window's other overlay may sit in between: backdrop and border
    // don't overlap, so their relative order is invisible, and insisting
    // on one made each re-stack displace the other every tick.
    let is_settled = prev == insert_after
      || (OverlayKind::is_overlay(prev)
        // SAFETY: A stale `prev` just makes `GetWindow` return `HWND(0)`.
        && unsafe { GetWindow(prev, GW_HWNDPREV) } == insert_after);

    if force || !is_settled {
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

  /// Makes `owner` (or no window) own the overlay. Windows keeps owned
  /// windows above their owner and lifts them along with it, e.g. when
  /// the owner opens a popup of its own.
  ///
  /// Only takes effect on a window from
  /// [`create_on_own_thread`](Self::create_on_own_thread): owning across
  /// threads attaches the threads' input queues, and one the WM's
  /// thread (which never pumps messages) joined would stall the app's
  /// input. That thread only serves this window, so no two apps' input
  /// gets tied together through it either.
  pub(crate) fn set_owner(&mut self, owner: Option<HWND>) {
    let owner = owner.map_or(0, |owner| owner.0);

    if self.thread.is_none() || self.owner == owner {
      return;
    }

    // SAFETY: `self.hwnd()` is a top-level window of this process, for
    // which `GWLP_HWNDPARENT` sets the owner. The previous value (the old
    // owner) is not needed; failure leaves `self.owner` stale, so the
    // next call retries.
    unsafe { SetWindowLongPtrW(self.hwnd(), GWLP_HWNDPARENT, owner) };
    self.owner = owner;
  }

  /// Moves the window to `rect` directly above `anchor` and shows it.
  ///
  /// Leaves the tracked state untouched on failure, so the next call
  /// retries.
  pub(crate) fn place_above(
    &mut self,
    rect: &Rect,
    anchor: HWND,
  ) -> crate::Result<()> {
    let (topmost, insert_after) =
      window_class::above_placement(anchor, self.hwnd());
    window_class::set_topmost(self.hwnd(), topmost);

    // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        insert_after,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE
          | SWP_NOSENDCHANGING
          | SWP_NOOWNERZORDER
          | SWP_SHOWWINDOW,
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

  /// Puts the window back above `anchor` (see
  /// [`window_class::above_placement`]) if it has drifted -- most commonly
  /// because `anchor` was raised over it -- without touching its rect.
  pub(crate) fn sync_z_order_above(
    &mut self,
    anchor: HWND,
  ) -> crate::Result<()> {
    let (topmost, insert_after) =
      window_class::above_placement(anchor, self.hwnd());

    let is_settled = window_class::is_topmost(self.hwnd()) == topmost
      && window_class::is_directly_above(self.hwnd(), anchor);

    if !is_settled {
      window_class::set_topmost(self.hwnd(), topmost);

      // SAFETY: `self.hwnd()` is valid for the lifetime of `self`.
      unsafe {
        SetWindowPos(
          self.hwnd(),
          insert_after,
          0,
          0,
          0,
          0,
          SWP_NOACTIVATE
            | SWP_NOSENDCHANGING
            | SWP_NOOWNERZORDER
            | SWP_NOMOVE
            | SWP_NOSIZE,
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
    // A window on its own thread can only be destroyed there; dropping
    // `self.thread` has it do so.
    if self.thread.is_some() {
      return;
    }

    // SAFETY: `self.hwnd()` is valid and `Drop` runs at most once.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}

/// A thread that owns a single overlay window and pumps its messages
/// until dropped, then destroys the window.
struct WindowThread {
  thread_id: u32,
}

impl WindowThread {
  /// Spawns the thread and creates the window on it, returning once the
  /// window exists.
  fn spawn(kind: OverlayKind, rect: &Rect) -> crate::Result<(HWND, Self)> {
    let (sender, receiver) = mpsc::channel();
    let rect = rect.clone();

    thread::Builder::new()
      .name(kind.label().to_string())
      .spawn(move || {
        let hwnd = match OverlayWindow::create_hwnd(kind, &rect) {
          Ok(hwnd) => hwnd,
          Err(err) => {
            let _ = sender.send(Err(err.to_string()));
            return;
          }
        };

        // SAFETY: No preconditions.
        let thread_id = unsafe { GetCurrentThreadId() };

        // The WM gave up waiting; nothing will ever use the window.
        if sender.send(Ok((hwnd.0, thread_id))).is_ok() {
          let mut msg = MSG::default();

          // SAFETY: `msg` outlives each call. `GetMessageW` returns 0 on
          // `WM_QUIT` and -1 on failure, both of which end the loop.
          while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
            // SAFETY: `msg` was just filled in by `GetMessageW`.
            unsafe { DispatchMessageW(&raw const msg) };
          }
        }

        // SAFETY: The window was created by this thread. It is already
        // gone if its owner was destroyed, which just fails the call.
        let _ = unsafe { DestroyWindow(hwnd) };
      })?;

    match receiver.recv() {
      Ok(Ok((hwnd, thread_id))) => Ok((HWND(hwnd), Self { thread_id })),
      Ok(Err(err)) => Err(crate::Error::Platform(err)),
      Err(_) => Err(crate::Error::Platform(format!(
        "{} thread exited before creating its window.",
        kind.label()
      ))),
    }
  }
}

impl Drop for WindowThread {
  fn drop(&mut self) {
    // SAFETY: A thread that already exited just makes the call fail.
    let _ = unsafe {
      PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0))
    };
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
