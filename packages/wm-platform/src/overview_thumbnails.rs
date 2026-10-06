//! The DWM thumbnails the window overview is made of: live previews of
//! windows, and pictures drawn into cloaked windows of its own.

use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, COLORREF, HWND, POINT, RECT, SIZE},
    Graphics::{
      Dwm::{
        DwmGetWindowAttribute, DwmRegisterThumbnail,
        DwmSetWindowAttribute, DwmUnregisterThumbnail,
        DwmUpdateThumbnailProperties, DWMWA_CLOAK,
        DWMWA_EXTENDED_FRAME_BOUNDS, DWM_THUMBNAIL_PROPERTIES,
        DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
        DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
      },
      Gdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION},
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, GetWindowRect, RegisterClassW,
      ShowWindow, UpdateLayeredWindow, SW_SHOWNOACTIVATE, ULW_ALPHA,
      WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  companion::Companion,
  overview_chrome::Surface,
  overview_layout::{RectF, Tile},
  window_class,
};

/// Shown for a window whose frame can't be read, e.g. one being
/// destroyed.
pub(crate) const FALLBACK_FRAME: RECT = RECT {
  left: 0,
  top: 0,
  right: 1600,
  bottom: 1000,
};

/// Creates a cloaked layered window to draw a picture into: DWM keeps
/// composing it, so the overview can show it as a thumbnail.
fn create_picture_window() -> Option<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  REGISTERED.get_or_init(|| {
    let class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_OverviewPicture"),
      lpfnWndProc: Some(window_class::default_wnd_proc),
      ..Default::default()
    };

    // SAFETY: `class` is fully initialized with a static class name.
    unsafe { RegisterClassW(&raw const class) };
  });

  // SAFETY: The class is registered above, and the attribute matches its
  // documented `BOOL` size.
  unsafe {
    let hwnd = CreateWindowExW(
      WS_EX_LAYERED
        | WS_EX_TRANSPARENT
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE,
      w!("GlazeWM_OverviewPicture"),
      w!(""),
      WS_POPUP,
      0,
      0,
      1,
      1,
      None,
      None,
      None,
      None,
    );

    if hwnd.0 == 0 {
      return None;
    }

    let cloak = BOOL::from(true);
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_CLOAK,
      std::ptr::from_ref(&cloak).cast(),
      u32::try_from(std::mem::size_of::<BOOL>()).unwrap_or(4),
    );
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    Some(hwnd)
  }
}

/// A DWM thumbnail of some window on the overview.
pub(crate) struct Thumbnail {
  handle: isize,

  /// Last `(source, destination, opacity)` applied, to skip unchanged
  /// updates on every frame.
  placed: Option<(RECT, RECT, u8)>,
}

impl Thumbnail {
  /// Registers a thumbnail of `source` on `dest`, hidden until placed.
  /// Thumbnails stack in the order they are registered.
  pub fn register(dest: HWND, source: HWND) -> Option<Self> {
    // SAFETY: Both are top-level windows; registration fails otherwise.
    let handle = unsafe { DwmRegisterThumbnail(dest, source) }.ok()?;

    let mut thumbnail = Self {
      handle,
      placed: Some((RECT::default(), RECT::default(), 0)),
    };
    thumbnail.hide();
    Some(thumbnail)
  }

  /// Shows `source` (in the source window's coordinates) at `dest`.
  pub fn place(&mut self, source: RECT, dest: RECT, opacity: u8) {
    if self.placed == Some((source, dest, opacity)) {
      return;
    }
    self.placed = Some((source, dest, opacity));

    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTDESTINATION
        | DWM_TNP_RECTSOURCE
        | DWM_TNP_OPACITY
        | DWM_TNP_VISIBLE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcDestination: dest,
      rcSource: source,
      opacity,
      fVisible: BOOL::from(opacity > 0 && dest.right > dest.left),
      fSourceClientAreaOnly: false.into(),
    };

    // SAFETY: `self.handle` is a registered thumbnail and `props` outlives
    // the call.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.handle, &raw const props);
    }
  }

  pub fn hide(&mut self) {
    if self.placed.take().is_none() {
      return;
    }

    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_VISIBLE,
      fVisible: false.into(),
      ..Default::default()
    };

    // SAFETY: As in `place`.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.handle, &raw const props);
    }
  }
}

impl Drop for Thumbnail {
  fn drop(&mut self) {
    // SAFETY: Registered in `register` and unregistered once.
    unsafe {
      let _ = DwmUnregisterThumbnail(self.handle);
    }
  }
}

/// A picture drawn into a cloaked window, shown as a thumbnail.
pub(crate) struct Picture {
  pub window: HWND,
  surface: Option<Surface>,
  pub thumbnail: Option<Thumbnail>,
}

impl Picture {
  pub fn new() -> Option<Self> {
    Some(Self {
      window: create_picture_window()?,
      surface: None,
      thumbnail: None,
    })
  }

  /// Size of what was drawn last.
  pub fn size(&self) -> (i32, i32) {
    self
      .surface
      .as_ref()
      .map_or((0, 0), |surface| (surface.width, surface.height))
  }

  /// Redraws the picture at `size` with `draw`.
  pub fn draw(
    &mut self,
    size: (i32, i32),
    draw: impl FnOnce(&mut Surface),
  ) {
    if self.size() != size {
      self.surface = Surface::new(size.0, size.1);
    }

    let Some(surface) = &mut self.surface else {
      return;
    };
    draw(surface);

    let blend = BLENDFUNCTION {
      BlendOp: u8::try_from(AC_SRC_OVER).unwrap_or_default(),
      BlendFlags: 0,
      SourceConstantAlpha: u8::MAX,
      AlphaFormat: u8::try_from(AC_SRC_ALPHA).unwrap_or_default(),
    };
    let size = SIZE {
      cx: surface.width,
      cy: surface.height,
    };
    let origin = POINT::default();

    // SAFETY: The window and the surface's DC are valid, and every
    // pointer outlives the call.
    if let Err(err) = unsafe {
      UpdateLayeredWindow(
        self.window,
        None,
        Some(&raw const origin),
        Some(&raw const size),
        surface.dc,
        Some(&raw const origin),
        COLORREF(0),
        Some(&raw const blend),
        ULW_ALPHA,
      )
    } {
      tracing::warn!("Failed to draw an overview picture: {err}");
    }
  }

  /// Shows the whole picture at `dest` on the overview.
  pub fn place(&mut self, dest: &RectF, opacity: f32) {
    let (width, height) = self.size();
    if let Some(thumbnail) = &mut self.thumbnail {
      thumbnail.place(
        RECT {
          right: width,
          bottom: height,
          ..Default::default()
        },
        to_win32(dest),
        to_opacity(opacity),
      );
    }
  }
}

impl Drop for Picture {
  fn drop(&mut self) {
    self.thumbnail = None;

    // SAFETY: Created in `new` on this thread and destroyed once.
    unsafe {
      let _ = DestroyWindow(self.window);
    }
  }
}

/// A window's live preview on a card.
pub(crate) struct Preview {
  pub hwnd: isize,
  pub tile: Tile,

  /// Visible frame of the window, in its own coordinates.
  pub frame: RECT,

  /// What the thumbnail samples instead of the window, if anything.
  companion: Option<Companion>,

  pub thumbnail: Option<Thumbnail>,
  pub opacity: f32,
}

impl Preview {
  pub fn new(hwnd: isize, tile: Tile) -> Self {
    Self {
      hwnd,
      tile,
      frame: visible_frame(HWND(hwnd)).unwrap_or(FALLBACK_FRAME),
      companion: Companion::find(HWND(hwnd)),
      thumbnail: None,
      opacity: 1.0,
    }
  }

  pub fn register(&mut self, host: HWND) {
    let source = self.companion.map_or(HWND(self.hwnd), Companion::hwnd);
    self.thumbnail = Thumbnail::register(host, source);
  }

  /// The part of the window its tile shows.
  #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
  fn source(&self) -> RECT {
    let (width, height) = (
      (self.frame.right - self.frame.left) as f32,
      (self.frame.bottom - self.frame.top) as f32,
    );
    let crop = self.tile.crop;

    RECT {
      left: self.frame.left + (crop.x * width).round() as i32,
      top: self.frame.top + (crop.y * height).round() as i32,
      right: self.frame.left + (crop.right() * width).round() as i32,
      bottom: self.frame.top + (crop.bottom() * height).round() as i32,
    }
  }

  /// Shows the preview at `dest` on the overview.
  pub fn place(&mut self, dest: &RectF, opacity: f32) {
    let source = self.source();
    let (source, dest) = match self.companion {
      Some(companion) => companion.map_rects(source, to_win32(dest)),
      None => (source, to_win32(dest)),
    };

    if let Some(thumbnail) = &mut self.thumbnail {
      thumbnail.place(source, dest, to_opacity(opacity));
    }
  }
}

/// `hwnd`'s visible frame, without its invisible resize borders, in the
/// window's own coordinates.
pub(crate) fn visible_frame(hwnd: HWND) -> Option<RECT> {
  let (bounds, frame) = window_and_frame(hwnd)?;

  let visible = RECT {
    left: frame.left - bounds.left,
    top: frame.top - bounds.top,
    right: frame.right - bounds.left,
    bottom: frame.bottom - bounds.top,
  };

  (visible.right > visible.left && visible.bottom > visible.top)
    .then_some(visible)
}

/// `hwnd`'s visible frame on screen.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn visible_frame_on_screen(hwnd: HWND) -> Option<RectF> {
  let (_, frame) = window_and_frame(hwnd)?;

  (frame.right > frame.left && frame.bottom > frame.top).then(|| {
    RectF::new(
      frame.left as f32,
      frame.top as f32,
      (frame.right - frame.left) as f32,
      (frame.bottom - frame.top) as f32,
    )
  })
}

/// `hwnd`'s window rect and its DWM extended frame bounds.
fn window_and_frame(hwnd: HWND) -> Option<(RECT, RECT)> {
  let mut bounds = RECT::default();
  let mut frame = RECT::default();

  // SAFETY: Both rects outlive the calls and match the queried sizes; a
  // stale handle just makes them fail.
  unsafe {
    GetWindowRect(hwnd, &raw mut bounds).ok()?;
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut frame).cast(),
      u32::try_from(std::mem::size_of::<RECT>()).ok()?,
    )
    .ok()?;
  }

  Some((bounds, frame))
}

fn to_win32(rect: &RectF) -> RECT {
  let rect = rect.to_rect();
  RECT {
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
  }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn to_opacity(opacity: f32) -> u8 {
  (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}
