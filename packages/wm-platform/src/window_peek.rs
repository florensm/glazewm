use std::sync::OnceLock;

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::{
      DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE,
      DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWMWCP_ROUND,
      DWMWCP_ROUNDSMALL, DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY,
      DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
      DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, SetWindowPos, ShowWindow,
      SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOSENDCHANGING,
      SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  companion::Companion, dwm_thumbnail::DwmThumbnail,
  window_batch::WindowBatch, window_class, CornerStyle, NativeWindow,
  NativeWindowWindowsExt, Rect,
};

const CLASS_NAME: PCWSTR = w!("GlazeWM_Peek");

/// A click-through, live thumbnail of a window, cut to the part of it
/// that lies inside a clip rect.
///
/// Shows the partly visible columns of a scrolling layout: the real
/// window stays cloaked, so nothing of it reaches past its monitor.
pub struct WindowPeek {
  hwnd: HWND,

  /// Of the window, or of its companion, which then draws it (e.g. a
  /// recolored copy of it). `None` if registration failed.
  thumbnail: Option<DwmThumbnail>,
  companion: Option<Companion>,

  /// The window's invisible resize borders.
  border_inset: RECT,
  is_visible: bool,
  opacity: u8,

  /// Where the peek's own window was last put.
  placed: Option<Rect>,

  /// The window's visible frame where it was last placed, clipped or not.
  frame_rect: Option<Rect>,
  visible_rect: Option<Rect>,
}

impl WindowPeek {
  /// Creates a hidden peek of `window`, which is `rect` large.
  pub fn new(
    window: &NativeWindow,
    rect: &Rect,
    corner_style: &CornerStyle,
  ) -> crate::Result<Self> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    window_class::ensure_class_registered(
      &REGISTERED,
      CLASS_NAME,
      window_class::default_wnd_proc,
    );

    let source = window.hwnd();
    let border_inset = border_inset(source);
    let frame = to_frame(rect, &border_inset);

    // `WS_EX_NOREDIRECTIONBITMAP` leaves the window with no GDI surface,
    // so everything outside the thumbnail is transparent.
    //
    // SAFETY: The class is registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE
          | WS_EX_TOOLWINDOW
          | WS_EX_TRANSPARENT
          | WS_EX_NOREDIRECTIONBITMAP,
        CLASS_NAME,
        w!(""),
        WS_POPUP,
        frame.x(),
        frame.y(),
        frame.width(),
        frame.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create peek window.".to_string(),
      ));
    }

    apply_corner_preference(hwnd, corner_style);
    suppress_native_border(hwnd);

    let companion = Companion::find(source);
    let thumbnail = DwmThumbnail::register(
      hwnd,
      companion.map_or(source, Companion::hwnd),
    );

    // Built before the move below, so its failure still destroys the
    // window.
    let peek = Self {
      hwnd,
      thumbnail,
      companion,
      border_inset,
      is_visible: false,
      opacity: u8::MAX,
      placed: None,
      frame_rect: None,
      visible_rect: None,
    };

    // Just below the window it shows.
    // SAFETY: Both handles are valid.
    unsafe {
      SetWindowPos(
        hwnd,
        source,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      )
    }?;

    Ok(peek)
  }

  /// Shows the part of the window, placed at `rect`, that lies inside
  /// `clip`, or hides the peek when none does. A move of a peek already
  /// shown is queued into `batch`, to land with its window's overlays.
  pub fn place(
    &mut self,
    rect: &Rect,
    clip: &Rect,
    opacity: u8,
    batch: &mut WindowBatch,
  ) {
    let frame = to_frame(rect, &self.border_inset);
    self.frame_rect = Some(frame.clone());

    let Some(visible) = frame.intersection(clip) else {
      self.set_visible(false);
      self.visible_rect = None;
      return;
    };

    // The thumbnail draws the whole window at its place, and the peek's
    // own bounds cut it.
    let (source, destination) = placed_rects(
      &frame,
      &self.border_inset,
      (visible.left, visible.top),
    );
    self.set_thumbnail_rects(source, destination);

    if self.placed.as_ref() != Some(&visible) {
      if self.is_visible {
        batch.push(self.hwnd.0, visible.clone());
      } else {
        // SAFETY: `self.hwnd` is valid until drop, and `SWP_NOZORDER`
        // makes `hWndInsertAfter` irrelevant.
        let moved = unsafe {
          SetWindowPos(
            self.hwnd,
            HWND(0),
            visible.x(),
            visible.y(),
            visible.width(),
            visible.height(),
            SWP_NOACTIVATE
              | SWP_NOCOPYBITS
              | SWP_NOSENDCHANGING
              | SWP_NOZORDER,
          )
        };
        if let Err(err) = moved {
          tracing::warn!("Failed to place peek: {err}");
        }
      }
      self.placed = Some(visible.clone());
    }

    self.set_opacity(opacity);
    self.set_visible(true);
    self.visible_rect = Some(visible);
  }

  /// The window's visible frame where it was last placed, including any
  /// part outside the clip, for overlays that frame it.
  #[must_use]
  pub fn frame_rect(&self) -> Option<&Rect> {
    self.frame_rect.as_ref()
  }

  /// Handle of the peek's own window, which overlays of the window it
  /// shows sit behind.
  #[must_use]
  pub fn hwnd(&self) -> HWND {
    self.hwnd
  }

  /// The on-screen part of the peek, if it shows anything.
  #[must_use]
  pub fn visible_rect(&self) -> Option<&Rect> {
    self.visible_rect.as_ref()
  }

  fn set_thumbnail_rects(&self, source: RECT, destination: RECT) {
    let Some(thumbnail) = &self.thumbnail else {
      return;
    };

    let (source, destination) =
      self.companion.map_or((source, destination), |companion| {
        companion.map_rects(source, destination)
      });

    let _ = thumbnail.update(&DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTSOURCE
        | DWM_TNP_RECTDESTINATION
        | DWM_TNP_VISIBLE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcSource: source,
      rcDestination: destination,
      fVisible: true.into(),
      fSourceClientAreaOnly: false.into(),
      ..Default::default()
    });
  }

  fn set_opacity(&mut self, opacity: u8) {
    if opacity == self.opacity {
      return;
    }
    self.opacity = opacity;

    if let Some(thumbnail) = &self.thumbnail {
      let _ = thumbnail.update(&DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_OPACITY,
        opacity,
        ..Default::default()
      });
    }
  }

  fn set_visible(&mut self, is_visible: bool) {
    if self.is_visible == is_visible {
      return;
    }
    self.is_visible = is_visible;

    // SAFETY: `self.hwnd` is valid until drop.
    unsafe {
      ShowWindow(
        self.hwnd,
        if is_visible {
          SW_SHOWNOACTIVATE
        } else {
          SW_HIDE
        },
      );
    }
  }
}

impl Drop for WindowPeek {
  fn drop(&mut self) {
    // The thumbnail goes before the window it is drawn in.
    self.thumbnail.take();

    // SAFETY: Created by this type, and destroyed once, here.
    let _ = unsafe { DestroyWindow(self.hwnd) };
  }
}

/// `(rcSource, rcDestination)` drawing a window whose visible frame is at
/// `frame`, past its invisible `border_inset`, into a peek whose top-left
/// is at `origin`. The frame and origin are in screen coordinates.
///
/// Invisible-border pixels are left out of the source, as DWM thumbnails
/// draw them black. Whatever lies outside the peek is cut by its bounds.
fn placed_rects(
  frame: &Rect,
  border_inset: &RECT,
  origin: (i32, i32),
) -> (RECT, RECT) {
  let source = RECT {
    left: border_inset.left,
    top: border_inset.top,
    right: border_inset.left + frame.width(),
    bottom: border_inset.top + frame.height(),
  };
  let destination = RECT {
    left: frame.left - origin.0,
    top: frame.top - origin.1,
    right: frame.right - origin.0,
    bottom: frame.bottom - origin.1,
  };

  (source, destination)
}

/// The visible frame of a window at `rect`, past its `inset`.
fn to_frame(rect: &Rect, inset: &RECT) -> Rect {
  Rect::from_ltrb(
    rect.left + inset.left,
    rect.top + inset.top,
    rect.right - inset.right,
    rect.bottom - inset.bottom,
  )
}

/// The invisible resize borders of `hwnd`, between its window rect and its
/// visible frame.
fn border_inset(hwnd: HWND) -> RECT {
  let Some((window, frame)) = window_class::bounds_and_frame(hwnd) else {
    return RECT::default();
  };

  RECT {
    left: (frame.left - window.left).max(0),
    top: (frame.top - window.top).max(0),
    right: (window.right - frame.right).max(0),
    bottom: (window.bottom - frame.bottom).max(0),
  }
}

/// Rounds `hwnd`'s corners as `corner_style` says, as DWM doesn't round a
/// `WS_POPUP | WS_EX_TOOLWINDOW` window by default. No-op on Windows 10.
fn apply_corner_preference(hwnd: HWND, corner_style: &CornerStyle) {
  let preference = match corner_style {
    CornerStyle::Default | CornerStyle::Rounded => DWMWCP_ROUND,
    CornerStyle::Square => DWMWCP_DONOTROUND,
    CornerStyle::SmallRounded => DWMWCP_ROUNDSMALL,
  };

  // SAFETY: `preference` outlives the call.
  unsafe {
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_WINDOW_CORNER_PREFERENCE,
      std::ptr::from_ref(&preference.0).cast(),
      u32::try_from(std::mem::size_of::<i32>()).unwrap_or(u32::MAX),
    );
  }
}

/// Leaves out the 1px border Windows 11 draws around a rounded popup,
/// which would show inside the window's border overlay. No-op on Windows
/// 10.
fn suppress_native_border(hwnd: HWND) {
  let color = DWMWA_COLOR_NONE;

  // SAFETY: `color` outlives the call.
  unsafe {
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_BORDER_COLOR,
      std::ptr::from_ref(&color).cast(),
      u32::try_from(std::mem::size_of::<u32>()).unwrap_or(u32::MAX),
    );
  }
}

#[cfg(test)]
mod tests {
  use windows::Win32::Foundation::RECT;

  use super::{placed_rects, Rect};

  fn ltrb(rect: RECT) -> (i32, i32, i32, i32) {
    (rect.left, rect.top, rect.right, rect.bottom)
  }

  /// A window cut at its peek's far edge keeps the same thumbnail however
  /// far past it it lies, so only the peek's own move changes.
  #[test]
  fn a_window_cut_at_the_far_edge_keeps_its_thumbnail() {
    let inset = RECT {
      left: 7,
      top: 0,
      right: 7,
      bottom: 7,
    };

    // A 400 px window past a monitor that ends at x = 1000.
    for left in [700, 750, 900] {
      let frame = Rect::from_xy(left, 100, 400, 300);
      let (source, destination) =
        placed_rects(&frame, &inset, (left, 100));

      assert_eq!(ltrb(source), (7, 0, 407, 300));
      assert_eq!(ltrb(destination), (0, 0, 400, 300));
    }
  }

  /// A window cut at its peek's origin is drawn offset past it, whole.
  #[test]
  fn a_window_cut_at_the_origin_is_drawn_offset() {
    let frame = Rect::from_xy(-150, 100, 400, 300);
    let (source, destination) =
      placed_rects(&frame, &RECT::default(), (0, 100));

    assert_eq!(ltrb(source), (0, 0, 400, 300));
    assert_eq!(ltrb(destination), (-150, 0, 250, 300));
  }
}
