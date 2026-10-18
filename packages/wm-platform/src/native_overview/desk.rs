//! The desktop workspaces show their windows on: the wallpaper under the
//! monitor's working area, zooming with the windows on it, so each
//! workspace looks like the monitor shrunk.
//!
//! The wallpaper is drawn on a cloaked window of its own and shown as
//! thumbnails, like the windows, so DWM moves it in step with them.

use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND, RECT},
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_CLOAK},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, SetWindowPos, ShowWindow,
      SWP_NOACTIVATE, SWP_NOZORDER, SW_SHOWNOACTIVATE, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
      WS_POPUP,
    },
  },
};

use crate::{
  overview_layout::RectF,
  overview_thumbnails::{to_opacity, to_win32, Thumbnail},
  platform_impl::composition::OverviewBackdrop,
  window_class, OverviewStyle, Rect,
};

pub(super) struct Desk {
  window: HWND,

  /// The wallpaper drawn on `window`, once it could be.
  wallpaper: Option<OverviewBackdrop>,

  /// Working area it shows, in screen coordinates.
  rect: Option<Rect>,
}

impl Desk {
  pub(super) fn new() -> Option<Self> {
    Some(Self {
      window: create_desk_window()?,
      wallpaper: None,
      rect: None,
    })
  }

  /// Shows the wallpaper under working area `rect`, baked as the
  /// overview's backdrop is, so the two share their bakes.
  pub(super) fn sync(&mut self, rect: &Rect, style: &OverviewStyle) {
    if self.rect.as_ref() != Some(rect) {
      // SAFETY: `self.window` is the desk's window, on this thread.
      unsafe {
        let _ = SetWindowPos(
          self.window,
          None,
          rect.x(),
          rect.y(),
          rect.width(),
          rect.height(),
          SWP_NOACTIVATE | SWP_NOZORDER,
        );
      }
      self.rect = Some(rect.clone());
    }

    // Only its plain wallpaper shows: the blurred copy and the tint stay
    // faded out.
    let result = match &mut self.wallpaper {
      Some(wallpaper) => {
        wallpaper.update(rect, style.backdrop_blur, style.backdrop_tint)
      }
      None => OverviewBackdrop::create(
        self.window,
        rect,
        style.backdrop_blur,
        style.backdrop_tint,
      )
      .map(|wallpaper| self.wallpaper = Some(wallpaper)),
    };

    if let Err(err) = result {
      tracing::warn!("Overview desk unavailable: {err}");
    }
  }

  /// Whether its wallpaper is drawn, for thumbnails to show.
  pub(super) fn is_drawn(&self) -> bool {
    self.wallpaper.is_some()
  }

  /// A thumbnail of it on `host`, above what was registered there before.
  pub(super) fn register(&self, host: HWND) -> Option<Thumbnail> {
    self.wallpaper.as_ref()?;
    Thumbnail::register(host, self.window)
  }
}

impl Drop for Desk {
  fn drop(&mut self) {
    self.wallpaper = None;

    // SAFETY: Created in `new` on this thread and destroyed once.
    unsafe {
      let _ = DestroyWindow(self.window);
    }
  }
}

/// Shows `thumbnail`, of the desk for working area `area`, at `dest`, cut
/// to `clip`.
pub(super) fn place(
  thumbnail: &mut Thumbnail,
  area: &RectF,
  dest: &RectF,
  clip: &RectF,
  opacity: f32,
) {
  #[allow(clippy::cast_possible_truncation)]
  let source = RECT {
    right: area.w.round() as i32,
    bottom: area.h.round() as i32,
    ..Default::default()
  };
  thumbnail.place(
    source,
    to_win32(dest),
    &to_win32(clip),
    to_opacity(opacity),
  );
}

/// Creates the cloaked window the wallpaper is drawn on: DWM keeps
/// composing it, so the overview can show it as a thumbnail.
fn create_desk_window() -> Option<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_OverviewDesk"),
    window_class::default_wnd_proc,
  );

  // `WS_EX_NOREDIRECTIONBITMAP`: the wallpaper is a composition visual
  // tree rooted on the window.
  //
  // SAFETY: The class is registered above, and the attribute matches its
  // documented `BOOL` size.
  unsafe {
    let hwnd = CreateWindowExW(
      WS_EX_NOREDIRECTIONBITMAP
        | WS_EX_TRANSPARENT
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE,
      w!("GlazeWM_OverviewDesk"),
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
