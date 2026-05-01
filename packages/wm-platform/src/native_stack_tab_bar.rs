use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
      BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW,
      EndPaint, FillRect, HBRUSH, InvalidateRect, SelectObject, SetBkMode,
      SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY,
      DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL,
      OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, DrawIconEx,
      GetClassLongPtrW, GetWindowLongPtrW, PostMessageW, RegisterClassW,
      SendMessageW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CREATESTRUCTW,
      DI_NORMAL, GCLP_HICONSM, GWLP_USERDATA, HICON, SW_HIDE, SWP_NOACTIVATE,
      SWP_SHOWWINDOW,
      WM_APP, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_ERASEBKGND, WM_GETICON,
      WM_LBUTTONDOWN, WM_PAINT, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
      WS_POPUP, WS_VISIBLE,
    },
  },
};

use crate::{Color, Dispatcher, Rect};

/// Custom message used to update tab state from the tokio thread.
///
/// `WPARAM` carries a heap-allocated `Box<TabUpdate>` that must be
/// recovered with `Box::from_raw`. `LPARAM` is unused.
const WM_UPDATE_TABS: u32 = WM_APP + 1;

static TAB_BAR_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Information about a single tab in the tab bar.
pub struct TabInfo {
  /// Display title for the tab.
  pub title: String,

  /// Handle (`HWND`) of the managed window, used to fetch its icon.
  pub hwnd: isize,
}

/// Color scheme for the stack tab bar.
#[derive(Clone)]
pub struct TabBarColors {
  pub background: Color,
  pub active: Color,
  pub inactive: Color,
  pub text: Color,
}

/// Payload sent via `WM_UPDATE_TABS` to update the tab bar from any thread.
struct TabUpdate {
  tabs: Vec<TabInfo>,
  active_index: usize,
  rect: Rect,
}

/// Per-window state stored in `GWLP_USERDATA`.
struct TabBarState {
  tabs: Vec<TabInfo>,
  active_index: usize,
  rect: Rect,
  colors: TabBarColors,
  on_click: Box<dyn Fn(usize) + Send + 'static>,
}

/// A GDI-painted tab bar overlay for a `StackContainer`.
///
/// The window is a `WS_POPUP | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` overlay
/// created on the event-loop thread. Tab state is updated from the tokio
/// thread via `SendMessageW(WM_UPDATE_TABS)` (synchronous). Click events are
/// routed back through the `on_click` closure, which is expected to send a
/// message on a tokio channel.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeStackTabBar {
  hwnd: isize,
}

// SAFETY: `hwnd` is a valid Win32 window handle that can be passed between
// threads. All WNDPROC processing happens on the event-loop thread. We only
// store the raw handle value here so that we can post messages to it.
unsafe impl Send for NativeStackTabBar {}

impl NativeStackTabBar {
  /// Creates a new tab bar window positioned at the top of `rect`.
  ///
  /// The window is created synchronously on the event-loop thread via
  /// `dispatcher.dispatch_sync()`. The `on_click` closure is called with
  /// the zero-based tab index whenever the user clicks a tab.
  pub fn create(
    dispatcher: &Dispatcher,
    rect: &Rect,
    tabs: Vec<TabInfo>,
    active_index: usize,
    colors: TabBarColors,
    on_click: Box<dyn Fn(usize) + Send + 'static>,
  ) -> crate::Result<Self> {
    let state = Box::new(TabBarState {
      tabs,
      active_index,
      rect: rect.clone(),
      colors,
      on_click,
    });

    // Transmit the pointer as a plain `usize` so the closure is `Send`.
    let state_ptr_val = Box::into_raw(state) as usize;
    let rect = rect.clone();

    let hwnd = dispatcher.dispatch_sync(move || -> crate::Result<isize> {
      ensure_class_registered();

      let state_ptr = state_ptr_val as *mut TabBarState;

      // SAFETY: Creating a valid top-level popup window with known-good
      // parameters. `state_ptr` is valid for the duration of the window's
      // lifetime and is freed in `WM_DESTROY`.
      let handle = unsafe {
        CreateWindowExW(
          WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
          w!("GlazeWM_TabBar"),
          w!(""),
          WS_POPUP | WS_VISIBLE,
          rect.left,
          rect.top,
          rect.width(),
          rect.height(),
          None,
          None,
          GetModuleHandleW(None).unwrap_or_default(),
          Some(state_ptr.cast()),
        )
      };

      if handle.0 == 0 {
        // Window creation failed — free state to avoid a leak.
        // SAFETY: We just allocated this pointer and creation failed, so
        // no other code has taken ownership.
        unsafe { drop(Box::from_raw(state_ptr)) };
        return Err(crate::Error::Platform(
          "Failed to create tab bar window.".to_string(),
        ));
      }

      Ok(handle.0)
    })??;

    Ok(Self { hwnd })
  }

  /// Posts a tab-state update to the tab bar window.
  ///
  /// Uses `PostMessageW` (fire-and-forget) to avoid blocking the tokio
  /// thread, which could deadlock if the Win32 event-loop thread is itself
  /// waiting on a `SetWindowPos` for a managed application window.
  pub fn update(&self, rect: &Rect, tabs: Vec<TabInfo>, active_index: usize) {
    let update = Box::new(TabUpdate {
      tabs,
      active_index,
      rect: rect.clone(),
    });

    let ptr = Box::into_raw(update) as usize;

    // SAFETY: `self.hwnd` is a valid window handle. `PostMessageW` queues
    // the message without blocking; ownership of the `Box<TabUpdate>`
    // transfers to the WNDPROC, which recovers and frees it in
    // `WM_UPDATE_TABS`. If the post fails the pointer is leaked (window
    // is gone), which is acceptable as the app is shutting down anyway.
    unsafe {
      let _ = PostMessageW(
        HWND(self.hwnd),
        WM_UPDATE_TABS,
        WPARAM(ptr),
        LPARAM(0),
      );
    }
  }

  /// Repositions, resizes, and shows the tab bar at the given rect.
  ///
  /// Passes the explicit position so the bar is always at the correct
  /// location regardless of any previously cached state.
  pub fn show_at(&self, rect: &Rect) {
    // SAFETY: `self.hwnd` is a valid window handle.
    unsafe {
      let _ = SetWindowPos(
        HWND(self.hwnd),
        HWND(0isize),
        rect.left,
        rect.top,
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
      );
    }
  }

  /// Hides the tab bar window without destroying it.
  ///
  /// Used to suppress the overlay during workspace-switch animations so the
  /// bar does not float over the workspace-surrogate slides.
  pub fn hide(&self) {
    // SAFETY: `self.hwnd` is a valid window handle.
    unsafe {
      let _ = ShowWindow(HWND(self.hwnd), SW_HIDE);
    }
  }
}

impl Drop for NativeStackTabBar {
  fn drop(&mut self) {
    // Post WM_CLOSE so the event loop destroys the window and its state.
    // SAFETY: `self.hwnd` is a valid window handle.
    unsafe {
      let _ = PostMessageW(HWND(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
  }
}

fn ensure_class_registered() {
  TAB_BAR_CLASS_REGISTERED.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_TabBar"),
      lpfnWndProc: Some(tab_bar_wnd_proc),
      // No background brush: WM_ERASEBKGND returns 1 to suppress erase,
      // and WM_PAINT draws the entire client area.
      hbrBackground: HBRUSH::default(),
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Paints the tab bar client area using GDI.
///
/// Draws the background, per-tab colored rectangles, process icons fetched
/// from the managed window's class, and tab title text.
unsafe fn paint_tab_bar(hwnd: HWND, state: &TabBarState) {
  let mut ps = PAINTSTRUCT::default();
  let hdc = BeginPaint(hwnd, &mut ps);

  let width = state.rect.width();
  let height = state.rect.height();
  let n_tabs = state.tabs.len();

  if n_tabs == 0 {
    EndPaint(hwnd, &ps);
    return;
  }

  // Draw the full bar background.
  let bg_brush = CreateSolidBrush(COLORREF(state.colors.background.to_bgr()));
  let full_rect = RECT {
    left: 0,
    top: 0,
    right: width,
    bottom: height,
  };
  FillRect(hdc, &full_rect, bg_brush);
  DeleteObject(bg_brush);

  SetBkMode(hdc, TRANSPARENT);
  SetTextColor(hdc, COLORREF(state.colors.text.to_bgr()));

  // Create a proportionally sized font that fits within the tab height.
  let font_height = -(height - 6).max(8);
  let font = CreateFontW(
    font_height,
    0,
    0,
    0,
    FW_NORMAL.0 as i32,
    0,
    0,
    0,
    DEFAULT_CHARSET.0 as u32,
    OUT_DEFAULT_PRECIS.0 as u32,
    CLIP_DEFAULT_PRECIS.0 as u32,
    DEFAULT_QUALITY.0 as u32,
    0,
    w!("Segoe UI"),
  );
  let old_font = SelectObject(hdc, font);

  let tab_width = width / n_tabs as i32;

  for (i, tab) in state.tabs.iter().enumerate() {
    let x = i as i32 * tab_width;
    let actual_tab_width = if i == n_tabs - 1 {
      // Last tab absorbs remainder from integer division.
      width - x
    } else {
      tab_width
    };

    // Draw per-tab background.
    let tab_color = if i == state.active_index {
      state.colors.active.to_bgr()
    } else {
      state.colors.inactive.to_bgr()
    };
    let tab_brush = CreateSolidBrush(COLORREF(tab_color));
    let tab_rect = RECT {
      left: x,
      top: 0,
      right: x + actual_tab_width,
      bottom: height,
    };
    FillRect(hdc, &tab_rect, tab_brush);
    DeleteObject(tab_brush);

    // Attempt to load the window's small icon.
    let icon_hwnd = HWND(tab.hwnd);
    let icon_size = (height - 6).max(8);
    // 2 = ICON_SMALL2 — small icon used for the window title bar.
    let icon_lresult =
      SendMessageW(icon_hwnd, WM_GETICON, WPARAM(2), LPARAM(0));
    let hicon = if icon_lresult.0 != 0 {
      HICON(icon_lresult.0)
    } else {
      HICON(GetClassLongPtrW(icon_hwnd, GCLP_HICONSM) as isize)
    };

    let text_x = if hicon.0 != 0 {
      let icon_y = (height - icon_size) / 2;
      let _ = DrawIconEx(
        hdc,
        x + 4,
        icon_y,
        hicon,
        icon_size,
        icon_size,
        0,
        None,
        DI_NORMAL,
      );
      x + 4 + icon_size + 4
    } else {
      x + 6
    };

    // Draw tab title text.
    let mut title_wide: Vec<u16> =
      tab.title.encode_utf16().collect();
    let mut text_rect = RECT {
      left: text_x,
      top: 0,
      right: x + actual_tab_width - 4,
      bottom: height,
    };
    DrawTextW(
      hdc,
      &mut title_wide,
      &mut text_rect,
      DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
    );
  }

  SelectObject(hdc, old_font);
  DeleteObject(font);
  EndPaint(hwnd, &ps);
}

/// Window procedure for the tab bar overlay window.
unsafe extern "system" fn tab_bar_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_CREATE => {
      // Store the `TabBarState` pointer passed via `lpCreateParams`.
      let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
      let state_ptr = create_struct.lpCreateParams;
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
      LRESULT(0)
    }
    WM_ERASEBKGND => {
      // Suppress default background erase — WM_PAINT covers the full area.
      LRESULT(1)
    }
    WM_PAINT => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if state_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
      }
      paint_tab_bar(hwnd, &*state_ptr);
      LRESULT(0)
    }
    WM_LBUTTONDOWN => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if !state_ptr.is_null() {
        let state = &*state_ptr;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let x = (lparam.0 & 0xFFFF) as i16 as i32;
        let n_tabs = state.tabs.len();
        if n_tabs > 0 {
          let tab_width = state.rect.width() / n_tabs as i32;
          if tab_width > 0 {
            let index =
              ((x / tab_width) as usize).min(n_tabs.saturating_sub(1));
            (state.on_click)(index);
          }
        }
      }
      LRESULT(0)
    }
    WM_UPDATE_TABS => {
      // Recover the `TabUpdate` pointer from WPARAM.
      let update = Box::from_raw(wparam.0 as *mut TabUpdate);
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if !state_ptr.is_null() {
        let state = &mut *state_ptr;
        state.tabs = update.tabs;
        state.active_index = update.active_index;
        state.rect = update.rect;

        // SAFETY: `hwnd` is valid and `None` means the full client area.
        let _ = InvalidateRect(hwnd, None, false);
      }
      LRESULT(0)
    }
    WM_CLOSE => {
      let _ = DestroyWindow(hwnd);
      LRESULT(0)
    }
    WM_DESTROY => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if !state_ptr.is_null() {
        // Zero GWLP_USERDATA before freeing to prevent use-after-free if
        // a stray message arrives before the window is fully gone.
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        // SAFETY: We own this allocation; it was created in `create()` and
        // is freed exactly here when the window is destroyed.
        drop(Box::from_raw(state_ptr));
      }
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}

