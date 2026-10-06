//! The pinned window's preview: a small, live, always-on-top thumbnail of
//! a window that is out of sight. Clicking it goes back to the window,
//! dragging moves it, and right-clicking unpins.
//!
//! It runs on the overview's thread and shares its message loop.

use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::{
      Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR,
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
      },
      Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO,
        MONITOR_DEFAULTTONEAREST,
      },
    },
    UI::{
      Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos,
        LoadCursorW, RegisterClassW, SetCursor, SetWindowPos, ShowWindow,
        HWND_TOPMOST, IDC_HAND, IDC_SIZEALL, MA_NOACTIVATE,
        SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE,
        WM_CAPTURECHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
        WM_MOUSEMOVE, WM_RBUTTONUP, WM_SETCURSOR, WNDCLASSW,
        WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_POPUP,
      },
    },
  },
};

use crate::{
  native_overview,
  overview_layout::{cover_crop, keep_inside, pin_size, RectF, Tile},
  overview_thumbnails::{visible_frame, Preview, FALLBACK_FRAME},
  OverviewAction, PinFrame,
};

/// Space between the preview and the edges of the working area it first
/// shows up in, in logical pixels.
const MARGIN: f32 = 16.0;

/// Cursor travel before a press becomes a drag, in logical pixels.
const DRAG_THRESHOLD: f32 = 6.0;

pub(crate) struct PinPreview {
  window: HWND,
  pin: PinFrame,
  preview: Preview,

  /// Top-left corner on screen, kept while hidden and moved by dragging.
  position: (i32, i32),
  size: (i32, i32),

  drag: Option<Drag>,
  is_shown: bool,
}

/// A left-button press on the preview, which becomes a drag once the
/// cursor has moved far enough.
#[derive(Clone, Copy)]
struct Drag {
  cursor: POINT,
  origin: (i32, i32),
  is_moving: bool,
}

impl PinPreview {
  /// Creates the hidden preview of `pin`'s window, placed in the
  /// bottom-right corner of its working area.
  pub fn new(pin: PinFrame) -> Option<Self> {
    let window = create_window()?;

    let mut preview = Preview::new(
      pin.hwnd,
      Tile {
        rect: RectF::default(),
        crop: RectF::new(0.0, 0.0, 1.0, 1.0),
      },
    );
    preview.register(window);

    let mut this = Self {
      window,
      pin,
      preview,
      position: (0, 0),
      size: (0, 0),
      drag: None,
      is_shown: false,
    };
    this.apply_border();

    let (width, height) = this.measure();
    let area = RectF::from_rect(&this.pin.area);
    let margin = MARGIN * this.pin.scale_factor;

    #[allow(clippy::cast_possible_truncation)]
    let position = (
      (area.right() - margin - width).round() as i32,
      (area.bottom() - margin - height).round() as i32,
    );
    this.position = position;
    this.size = to_pixels((width, height));

    Some(this)
  }

  /// Handle of the pinned window.
  pub fn hwnd(&self) -> isize {
    self.pin.hwnd
  }

  /// Takes on `pin`, which is for the same window.
  pub fn update(&mut self, pin: PinFrame) {
    let is_restyled = pin.border != self.pin.border;
    self.pin = pin;

    if is_restyled {
      self.apply_border();
    }
  }

  /// Whether the WM wants it shown.
  pub fn is_wanted(&self) -> bool {
    self.pin.is_visible
  }

  /// Shows it above everything, at the window's current shape.
  pub fn show(&mut self) {
    // Keeps the bottom-right corner put if the window changed shape.
    let size = to_pixels(self.measure());
    let corner = (
      self.position.0 + self.size.0 - size.0,
      self.position.1 + self.size.1 - size.1,
    );
    self.size = size;
    self.position = self.inside_monitor(corner);

    // SAFETY: `self.window` is the preview's window, on this thread.
    unsafe {
      let _ = SetWindowPos(
        self.window,
        HWND_TOPMOST,
        self.position.0,
        self.position.1,
        self.size.0,
        self.size.1,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
      );
    }

    #[allow(clippy::cast_precision_loss)]
    let dest = RectF::new(0.0, 0.0, size.0 as f32, size.1 as f32);
    self.preview.place(&dest, 1.0);
    self.is_shown = true;
  }

  pub fn hide(&mut self) {
    if !self.is_shown {
      return;
    }

    self.is_shown = false;
    self.drag = None;

    // SAFETY: `self.window` is the preview's window, on this thread.
    unsafe {
      ShowWindow(self.window, SW_HIDE);
      let _ = ReleaseCapture();
    }
  }

  /// Handles a message of the preview's window. Returns `None` for the
  /// default handling; `send` is given what the user asked for.
  pub fn handle(
    &mut self,
    msg: u32,
    send: &dyn Fn(OverviewAction),
  ) -> Option<LRESULT> {
    match msg {
      // Clicks must not take focus from the window being worked in.
      WM_MOUSEACTIVATE => {
        #[allow(clippy::cast_possible_wrap)]
        return Some(LRESULT(MA_NOACTIVATE as isize));
      }
      WM_LBUTTONDOWN => {
        self.drag = Some(Drag {
          cursor: cursor_position(),
          origin: self.position,
          is_moving: false,
        });

        // SAFETY: `self.window` is the preview's window, on this thread.
        unsafe {
          SetCapture(self.window);
        }
      }
      WM_MOUSEMOVE => self.on_mouse_move(),
      WM_LBUTTONUP => {
        let drag = self.drag.take();

        // SAFETY: No preconditions.
        unsafe {
          let _ = ReleaseCapture();
        }

        match drag {
          Some(drag) if drag.is_moving => {
            self.position = self.inside_monitor(self.position);
            self.move_window();
          }
          Some(_) => send(OverviewAction::JumpToPin),
          None => {}
        }
      }
      WM_CAPTURECHANGED => self.drag = None,
      WM_RBUTTONUP => send(OverviewAction::Unpin),
      WM_SETCURSOR => {
        let is_moving = self.drag.is_some_and(|drag| drag.is_moving);
        let cursor = if is_moving { IDC_SIZEALL } else { IDC_HAND };

        // SAFETY: The cursors are system cursors.
        unsafe {
          if let Ok(cursor) = LoadCursorW(None, cursor) {
            SetCursor(cursor);
          }
        }
        return Some(LRESULT(1));
      }
      _ => return None,
    }

    Some(LRESULT(0))
  }

  fn on_mouse_move(&mut self) {
    let Some(drag) = &mut self.drag else {
      return;
    };

    let cursor = cursor_position();
    let delta = (cursor.x - drag.cursor.x, cursor.y - drag.cursor.y);

    #[allow(clippy::cast_precision_loss)]
    let travel = (delta.0.abs() + delta.1.abs()) as f32;
    if !drag.is_moving && travel < DRAG_THRESHOLD * self.pin.scale_factor {
      return;
    }

    drag.is_moving = true;
    self.position = (drag.origin.0 + delta.0, drag.origin.1 + delta.1);
    self.move_window();
  }

  fn move_window(&self) {
    // SAFETY: `self.window` is the preview's window, on this thread.
    unsafe {
      let _ = SetWindowPos(
        self.window,
        None,
        self.position.0,
        self.position.1,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
      );
    }
  }

  /// Size for the window's current shape, which it also crops to.
  fn measure(&mut self) -> (f32, f32) {
    let frame =
      visible_frame(HWND(self.pin.hwnd)).unwrap_or(FALLBACK_FRAME);

    #[allow(clippy::cast_precision_loss)]
    let source = (
      (frame.right - frame.left) as f32,
      (frame.bottom - frame.top) as f32,
    );
    let size = pin_size(self.pin.width * self.pin.scale_factor, source);

    self.preview.frame = frame;
    self.preview.tile.crop = cover_crop(source, size);
    size
  }

  /// `position` moved so the preview sits fully on the working area of
  /// the monitor it is mostly on.
  fn inside_monitor(&self, position: (i32, i32)) -> (i32, i32) {
    let center = POINT {
      x: position.0 + self.size.0 / 2,
      y: position.1 + self.size.1 / 2,
    };
    let mut info = MONITORINFO {
      cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>())
        .unwrap_or_default(),
      ..Default::default()
    };

    // SAFETY: `MonitorFromPoint` always returns a monitor with
    // `MONITOR_DEFAULTTONEAREST`, and `info` outlives the call.
    let has_info = unsafe {
      let monitor = MonitorFromPoint(center, MONITOR_DEFAULTTONEAREST);
      GetMonitorInfoW(monitor, &raw mut info).as_bool()
    };
    if !has_info {
      return position;
    }

    let work = info.rcWork;
    #[allow(clippy::cast_precision_loss)]
    let area = RectF::new(
      work.left as f32,
      work.top as f32,
      (work.right - work.left) as f32,
      (work.bottom - work.top) as f32,
    );
    #[allow(clippy::cast_precision_loss)]
    let (x, y) = keep_inside(
      (position.0 as f32, position.1 as f32),
      (self.size.0 as f32, self.size.1 as f32),
      &area,
    );

    #[allow(clippy::cast_possible_truncation)]
    (x.round() as i32, y.round() as i32)
  }

  /// Rounds the corners and draws DWM's own border in the pin's color.
  /// Both are no-ops before Windows 11.
  fn apply_border(&self) {
    let border = self.pin.border;
    let color = u32::from(border.r)
      | (u32::from(border.g) << 8)
      | (u32::from(border.b) << 16);

    // SAFETY: `self.window` is valid, and each value is a stack-allocated
    // 32-bit value of the size its attribute expects.
    unsafe {
      let _ = DwmSetWindowAttribute(
        self.window,
        DWMWA_WINDOW_CORNER_PREFERENCE,
        std::ptr::from_ref(&DWMWCP_ROUND.0).cast(),
        u32::try_from(std::mem::size_of::<i32>()).unwrap_or(4),
      );
      let _ = DwmSetWindowAttribute(
        self.window,
        DWMWA_BORDER_COLOR,
        std::ptr::from_ref(&color).cast(),
        u32::try_from(std::mem::size_of::<u32>()).unwrap_or(4),
      );
    }
  }
}

impl Drop for PinPreview {
  fn drop(&mut self) {
    self.preview.thumbnail = None;

    // SAFETY: Created in `new` on this thread and destroyed once.
    unsafe {
      let _ = DestroyWindow(self.window);
    }
  }
}

fn create_window() -> Option<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  REGISTERED.get_or_init(|| {
    let class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_PinPreview"),
      lpfnWndProc: Some(wnd_proc),
      ..Default::default()
    };

    // SAFETY: `class` is fully initialized with a static class name.
    unsafe { RegisterClassW(&raw const class) };
  });

  // `WS_EX_NOREDIRECTIONBITMAP`: the window shows nothing but the
  // thumbnail on it.
  //
  // SAFETY: The class is registered above.
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_TOPMOST
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE
        | WS_EX_NOREDIRECTIONBITMAP,
      w!("GlazeWM_PinPreview"),
      w!(""),
      WS_POPUP,
      0,
      0,
      0,
      0,
      None,
      None,
      None,
      None,
    )
  };

  (hwnd.0 != 0).then_some(hwnd)
}

fn cursor_position() -> POINT {
  let mut point = POINT::default();

  // SAFETY: `point` outlives the call.
  unsafe {
    let _ = GetCursorPos(&raw mut point);
  }
  point
}

#[allow(clippy::cast_possible_truncation)]
fn to_pixels((width, height): (f32, f32)) -> (i32, i32) {
  (width.round() as i32, height.round() as i32)
}

/// Window procedure of the preview, which the overview owns.
unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  native_overview::on_pin_message(msg)
    .unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}
