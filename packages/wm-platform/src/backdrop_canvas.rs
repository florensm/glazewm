//! Invisible windows each showing a monitor's backdrop over all of it, for
//! scenes to draw moving windows' backdrops from (see
//! `BackdropCanvasVisual`).

use std::{cell::RefCell, sync::OnceLock};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{COLORREF, HWND},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, SetLayeredWindowAttributes,
      ShowWindow, LWA_ALPHA, SW_SHOWNOACTIVATE, WS_EX_LAYERED,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::{
    composition::BackdropCanvasVisual, wallpaper_surface::BakeKnobs,
  },
  window_class, BackdropOverlayParams, Color, Rect,
};

const CLASS_NAME: PCWSTR = w!("GlazeWM_BackdropCanvas");

/// Most canvases kept: a monitor's windows show one or two backdrops, and
/// one still in a scene must outlive it.
const MAX_CANVASES: usize = 8;

thread_local! {
  /// The thread's canvases, least recently used first.
  static CANVASES: RefCell<Vec<BackdropCanvas>> =
    const { RefCell::new(Vec::new()) };
}

/// The canvas showing `monitor`'s backdrop as `params` draw it, made on
/// first use and kept for later ones.
pub(crate) fn canvas_for(
  monitor: &Rect,
  params: &BackdropOverlayParams,
) -> crate::Result<HWND> {
  let knobs = BakeKnobs::from(*params);

  CANVASES.with(|canvases| {
    let mut canvases = canvases.borrow_mut();

    let found = canvases.iter().position(|canvas| {
      canvas.monitor == *monitor
        && canvas.knobs == knobs
        && canvas.tint == params.tint
    });

    let canvas = if let Some(index) = found {
      let mut canvas = canvases.remove(index);
      canvas.visual.sync()?;
      canvas
    } else {
      if canvases.len() == MAX_CANVASES {
        canvases.remove(0);
      }
      BackdropCanvas::create(monitor, knobs, params.tint)?
    };

    let hwnd = canvas.window.0;
    canvases.push(canvas);
    Ok(hwnd)
  })
}

struct BackdropCanvas {
  /// Declared before `window`: fields drop in declaration order, and the
  /// tree must go before the `HWND` it is rooted to.
  visual: BackdropCanvasVisual,
  window: CanvasWindow,
  monitor: Rect,
  knobs: BakeKnobs,
  tint: Color,
}

impl BackdropCanvas {
  fn create(
    monitor: &Rect,
    knobs: BakeKnobs,
    tint: Color,
  ) -> crate::Result<Self> {
    let window = CanvasWindow::create(monitor)?;
    let visual =
      BackdropCanvasVisual::create(window.0, monitor, knobs, tint)?;

    Ok(Self {
      visual,
      window,
      monitor: monitor.clone(),
      knobs,
      tint,
    })
  }
}

/// A click-through window over a monitor, never seen: shown, but layered
/// fully transparent, an opacity its thumbnails leave out.
///
/// Not cloaked: DWM draws a cloaked window's composition tree into none
/// of its thumbnails, which then show nothing (Windows 11 build 26200).
struct CanvasWindow(HWND);

impl CanvasWindow {
  fn create(monitor: &Rect) -> crate::Result<Self> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    window_class::ensure_class_registered(
      &REGISTERED,
      CLASS_NAME,
      window_class::default_wnd_proc,
    );

    // SAFETY: The class is registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_LAYERED
          | WS_EX_NOACTIVATE
          | WS_EX_TOOLWINDOW
          | WS_EX_TRANSPARENT
          | WS_EX_NOREDIRECTIONBITMAP,
        CLASS_NAME,
        w!(""),
        WS_POPUP,
        monitor.x(),
        monitor.y(),
        monitor.width(),
        monitor.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create backdrop canvas window.".to_string(),
      ));
    }
    let window = Self(hwnd);

    // SAFETY: `hwnd` is a window of this thread.
    unsafe {
      SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA)?;
      ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }

    Ok(window)
  }
}

impl Drop for CanvasWindow {
  fn drop(&mut self) {
    // SAFETY: Created by this thread, and destroyed once, here.
    let _ = unsafe { DestroyWindow(self.0) };
  }
}
