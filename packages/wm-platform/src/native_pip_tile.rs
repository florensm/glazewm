use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Dwm::{
      DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
      DWM_THUMBNAIL_PROPERTIES, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
      DWM_TNP_SOURCECLIENTAREAONLY,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW,
      LoadCursorW, PostMessageW, RegisterClassW, SetWindowLongPtrW,
      SetWindowPos, GWLP_USERDATA, IDC_HAND, SWP_NOACTIVATE, SWP_NOZORDER,
      WM_APP, WM_CLOSE, WM_DESTROY, WM_LBUTTONDOWN, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
    },
  },
};

use crate::{
  native_surrogate::{apply_corner_preference, register_thumbnail},
  CornerStyle, Dispatcher, Rect,
};

/// Custom message used to reposition/resize the tile from any thread.
///
/// `WPARAM` carries a heap-allocated `Box<Rect>` that must be recovered
/// with `Box::from_raw`. `LPARAM` is unused.
const WM_UPDATE_PIP_RECT: u32 = WM_APP + 1;

static PIP_TILE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Per-window state stored in `GWLP_USERDATA`.
struct PipTileState {
  /// Invoked on the event-loop thread whenever the tile is clicked.
  on_click: Box<dyn Fn() + Send + 'static>,
  /// DWM thumbnail handle, or `0` if registration failed.
  thumbnail: isize,
}

/// A live, clickable thumbnail of a minimized window, docked at a fixed
/// screen rect.
///
/// Unlike [`NativeSurrogate`], which is transparent to input
/// (`WS_EX_TRANSPARENT`) because it only exists for the duration of an
/// animation, this window is a normal interactive `WS_POPUP |
/// WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` overlay: clicking it fires
/// `on_click` instead of passing the click through to whatever is behind
/// it. `WS_EX_NOACTIVATE` still prevents the click from stealing
/// foreground/focus.
///
/// Threading follows [`NativeStackTabBar`]: the window is created
/// synchronously on the event-loop thread via `dispatcher.dispatch_sync`,
/// repositioned via a fire-and-forget `PostMessageW`, and torn down via
/// `PostMessageW(WM_CLOSE)` from `Drop` so a caller on a different thread
/// (the tokio/state thread) never calls `DestroyWindow` directly.
///
/// [`NativeSurrogate`]: crate::NativeSurrogate
/// [`NativeStackTabBar`]: crate::NativeStackTabBar
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativePipTile {
  hwnd: isize,
}

// SAFETY: `hwnd` is a valid Win32 window handle that can be passed between
// threads. All WNDPROC processing happens on the event-loop thread; we
// only store the raw handle value here so that we can post messages to it.
unsafe impl Send for NativePipTile {}

impl NativePipTile {
  /// Creates a new PIP tile at `rect`, showing a live thumbnail of
  /// `source_hwnd`.
  ///
  /// The window is created synchronously on the event-loop thread via
  /// `dispatcher.dispatch_sync`. `on_click` is called (on the event-loop
  /// thread) whenever the tile is clicked.
  pub fn create(
    dispatcher: &Dispatcher,
    source_hwnd: HWND,
    rect: &Rect,
    corner_style: &CornerStyle,
    on_click: Box<dyn Fn() + Send + 'static>,
  ) -> crate::Result<Self> {
    let rect = rect.clone();
    let corner_style = *corner_style;

    let hwnd = dispatcher.dispatch_sync(move || -> crate::Result<isize> {
      ensure_class_registered();

      // SAFETY: Creating a valid top-level popup window with known-good
      // parameters.
      let handle = unsafe {
        CreateWindowExW(
          WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
          w!("GlazeWM_PipTile"),
          w!(""),
          WS_POPUP | WS_VISIBLE,
          rect.left,
          rect.top,
          rect.width(),
          rect.height(),
          None,
          None,
          GetModuleHandleW(None).unwrap_or_default(),
          None,
        )
      };

      if handle.0 == 0 {
        return Err(crate::Error::Platform(
          "Failed to create PIP tile window.".to_string(),
        ));
      }

      apply_corner_preference(handle, &corner_style);

      let thumbnail = register_thumbnail(
        handle,
        source_hwnd,
        rect.width(),
        rect.height(),
        RECT::default(),
      )
      .unwrap_or(0);

      let state = Box::new(PipTileState { on_click, thumbnail });
      let state_ptr = Box::into_raw(state);

      // Set after creation (rather than via `lpCreateParams` in `WM_CREATE`)
      // because the state must carry the thumbnail handle, which only
      // exists once `register_thumbnail` above has run against the
      // already-created `handle`.
      //
      // SAFETY: `handle` was just created above and is not yet visible to
      // the wndproc until this call returns.
      unsafe {
        SetWindowLongPtrW(handle, GWLP_USERDATA, state_ptr as isize);
      }

      Ok(handle.0)
    })??;

    Ok(Self { hwnd })
  }

  /// Repositions/resizes the tile and rescales its thumbnail to fill the
  /// new rect.
  ///
  /// Fire-and-forget: posts to the event-loop thread instead of blocking
  /// the caller.
  pub fn reposition(&mut self, rect: &Rect) {
    let boxed = Box::new(rect.clone());
    let ptr = Box::into_raw(boxed) as usize;

    // SAFETY: `self.hwnd` is a valid window handle. `PostMessageW` queues
    // the message without blocking; ownership of the `Box<Rect>` transfers
    // to the WNDPROC, which recovers and frees it in `WM_UPDATE_PIP_RECT`.
    // If the post fails the pointer is leaked (window is gone), which is
    // acceptable as the app is shutting down anyway.
    unsafe {
      let _ = PostMessageW(
        HWND(self.hwnd),
        WM_UPDATE_PIP_RECT,
        WPARAM(ptr),
        LPARAM(0),
      );
    }
  }
}

impl Drop for NativePipTile {
  fn drop(&mut self) {
    // Post WM_CLOSE so the event loop destroys the window and its state.
    // SAFETY: `self.hwnd` is a valid window handle.
    unsafe {
      let _ = PostMessageW(HWND(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
  }
}

fn ensure_class_registered() {
  PIP_TILE_CLASS_REGISTERED.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_PipTile"),
      lpfnWndProc: Some(pip_tile_wnd_proc),
      // SAFETY: IDC_HAND is a valid system cursor resource.
      hCursor: unsafe { LoadCursorW(None, IDC_HAND).unwrap_or_default() },
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Window procedure for the PIP tile overlay window.
unsafe extern "system" fn pip_tile_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_LBUTTONDOWN => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PipTileState;
      if !state_ptr.is_null() {
        ((*state_ptr).on_click)();
      }
      LRESULT(0)
    }
    WM_UPDATE_PIP_RECT => {
      // Recover the boxed `Rect` from WPARAM.
      let rect = Box::from_raw(wparam.0 as *mut Rect);
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PipTileState;

      let _ = SetWindowPos(
        hwnd,
        HWND(0),
        rect.left,
        rect.top,
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOZORDER,
      );

      if !state_ptr.is_null() {
        let state = &*state_ptr;
        if state.thumbnail != 0 {
          let dst_rect = RECT {
            left: 0,
            top: 0,
            right: rect.width(),
            bottom: rect.height(),
          };
          let props = DWM_THUMBNAIL_PROPERTIES {
            dwFlags: DWM_TNP_RECTDESTINATION
              | DWM_TNP_RECTSOURCE
              | DWM_TNP_SOURCECLIENTAREAONLY,
            rcDestination: dst_rect,
            rcSource: dst_rect,
            fSourceClientAreaOnly: false.into(),
            ..Default::default()
          };
          let _ =
            DwmUpdateThumbnailProperties(state.thumbnail, &raw const props);
        }
      }
      LRESULT(0)
    }
    WM_CLOSE => {
      let _ = DestroyWindow(hwnd);
      LRESULT(0)
    }
    WM_DESTROY => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PipTileState;
      if !state_ptr.is_null() {
        // Zero GWLP_USERDATA before freeing to prevent use-after-free if a
        // stray message arrives before the window is fully gone.
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        // SAFETY: We own this allocation; it was created in `create()` and
        // is freed exactly here when the window is destroyed.
        let state = Box::from_raw(state_ptr);
        if state.thumbnail != 0 {
          let _ = DwmUnregisterThumbnail(state.thumbnail);
        }
      }
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}
