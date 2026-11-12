//! The windows a surrogate's copy of a window's backdrop is cut from (see
//! `OverlayCopies::carry_backdrop`).
//!
//! The backdrop shows the desktop behind its window, so it can't be one
//! copy moving with the window, as its ring or shadow is: it is the part
//! of a backdrop spanning the whole monitor that lies behind the window,
//! cut out anew every frame, with a copy of the window's own vignette
//! over it.

use std::{cell::RefCell, rc::Rc};

use windows::Win32::{
  Foundation::RECT, UI::WindowsAndMessaging::HWND_BOTTOM,
};

use crate::{
  overlay_window::{OverlayKind, OverlayWindow},
  platform_impl::composition::{
    BackdropVisual, SurrogateFill, VignetteVisual,
  },
  BackdropOverlayParams, Color, Rect,
};

/// Sources no copy uses that are kept for their style coming back, e.g.
/// as focus returns to a window.
const MAX_IDLE_SOURCES: usize = 2;

/// Side of a [`FillSource`]'s window, in physical pixels.
const FILL_SOURCE_SIZE: i32 = 8;

/// Pixels of a [`FillSource`]'s edges its copies leave out: a stretched
/// thumbnail blends its edge pixels with what is past them.
const FILL_SOURCE_MARGIN: i32 = 2;

thread_local! {
  /// Every source in use, shared by the copies of windows with the same
  /// style on the same monitor.
  static SOURCES: RefCell<Vec<Rc<BackdropSource>>> =
    const { RefCell::new(Vec::new()) };

  /// Every fill source in use, shared by the copies of windows with the
  /// same edge color.
  static FILL_SOURCES: RefCell<Vec<Rc<FillSource>>> =
    const { RefCell::new(Vec::new()) };
}

/// Drops all but [`MAX_IDLE_SOURCES`] of `sources` no copy uses.
fn drop_idle<T>(sources: &mut Vec<Rc<T>>) {
  let mut idle = 0;
  sources.retain(|source| {
    if Rc::strong_count(source) > 1 {
      return true;
    }
    idle += 1;
    idle <= MAX_IDLE_SOURCES
  });
}

/// A backdrop spanning a monitor, in a window hidden by its alpha, which
/// thumbnails of it leave out.
pub(crate) struct BackdropSource {
  /// Declared before `window`, which its visual tree is rooted to.
  composition: RefCell<BackdropVisual>,
  window: OverlayWindow,
  monitor: Rect,
  params: BackdropOverlayParams,
}

impl BackdropSource {
  /// The source of `params`' backdrop over `monitor`, shared with the
  /// other copies of it.
  ///
  /// Drawn without the vignette and rounding, which are each window's own
  /// (see `VignetteSource`), and pinned to the desktop.
  pub(crate) fn acquire(
    monitor: &Rect,
    params: BackdropOverlayParams,
  ) -> crate::Result<Rc<Self>> {
    let params = BackdropOverlayParams {
      vignette: 0.0,
      corner_radius: 0.0,
      parallax: 1.0,
      ..params
    };

    let cached = SOURCES.with(|sources| {
      sources
        .borrow()
        .iter()
        .find(|source| {
          &source.monitor == monitor && source.params == params
        })
        .cloned()
    });

    let source = if let Some(source) = cached {
      source
    } else {
      let source = Rc::new(Self::create(monitor, params)?);
      SOURCES.with(|sources| sources.borrow_mut().push(source.clone()));
      source
    };

    // A changed wallpaper is picked up as the source is put to use.
    if let Err(err) = source.composition.borrow_mut().sync_backdrop() {
      tracing::warn!("Backdrop source refresh failed: {err}.");
    }

    SOURCES.with(|sources| drop_idle(&mut sources.borrow_mut()));

    Ok(source)
  }

  fn create(
    monitor: &Rect,
    params: BackdropOverlayParams,
  ) -> crate::Result<Self> {
    let mut window = OverlayWindow::create(
      OverlayKind::BackdropSource,
      monitor,
      HWND_BOTTOM,
    )?;
    window.set_alpha(0)?;
    let composition =
      BackdropVisual::create(window.hwnd(), monitor, params)?;
    window.place_at_bottom(monitor)?;

    Ok(Self {
      composition: RefCell::new(composition),
      window,
      monitor: monitor.clone(),
      params,
    })
  }

  pub(crate) fn hwnd(&self) -> windows::Win32::Foundation::HWND {
    self.window.hwnd()
  }

  /// The monitor the source spans, whose coordinates thumbnails of it are
  /// cut in.
  pub(crate) fn monitor(&self) -> &Rect {
    &self.monitor
  }
}

/// A window's backdrop vignette alone, in a window of its own sized to the
/// window's frame and hidden by its alpha, for a copy of it to stretch
/// with the window.
pub(crate) struct VignetteSource {
  /// Declared before `window`, which its visual tree is rooted to.
  _composition: VignetteVisual,
  window: OverlayWindow,
  rect: Rect,
}

impl VignetteSource {
  /// Draws the vignette at `strength` over a window with its frame at
  /// `frame`.
  pub(crate) fn create(
    frame: &Rect,
    strength: f32,
  ) -> crate::Result<Self> {
    let mut window = OverlayWindow::create(
      OverlayKind::BackdropSource,
      frame,
      HWND_BOTTOM,
    )?;
    window.set_alpha(0)?;
    let composition = VignetteVisual::create(window.hwnd(), strength)?;
    window.place_at_bottom(frame)?;

    Ok(Self {
      _composition: composition,
      window,
      rect: frame.clone(),
    })
  }

  /// Moves the vignette over a window with its frame at `frame`.
  pub(crate) fn set_frame(&mut self, frame: &Rect) -> crate::Result<()> {
    if &self.rect == frame {
      return Ok(());
    }

    self.window.place_at_bottom(frame)?;
    self.rect = frame.clone();
    Ok(())
  }

  pub(crate) fn hwnd(&self) -> windows::Win32::Foundation::HWND {
    self.window.hwnd()
  }

  pub(crate) fn rect(&self) -> &Rect {
    &self.rect
  }
}

/// A few pixels of one color, in a window hidden by its alpha, which
/// thumbnails of it leave out: what a surrogate stretches over the part of
/// a window's frame its app has yet to fill (see
/// `OverlayCopies::carry_fill`).
///
/// A thumbnail rather than a visual on the surrogate, as it is drawn in
/// the same DWM updates as the window's own, and above the backdrop's
/// copy, which a visual rooted on the surrogate would be under.
pub(crate) struct FillSource {
  /// Declared before `window`, which its visual tree is rooted to.
  _composition: SurrogateFill,
  window: OverlayWindow,
  color: Color,
}

impl FillSource {
  /// The source of `color`, shared with the other copies of it.
  pub(crate) fn acquire(color: Color) -> crate::Result<Rc<Self>> {
    let cached = FILL_SOURCES.with(|sources| {
      sources
        .borrow()
        .iter()
        .find(|source| source.color == color)
        .cloned()
    });

    let source = if let Some(source) = cached {
      source
    } else {
      let source = Rc::new(Self::create(color)?);
      FILL_SOURCES
        .with(|sources| sources.borrow_mut().push(source.clone()));
      source
    };

    FILL_SOURCES.with(|sources| drop_idle(&mut sources.borrow_mut()));
    Ok(source)
  }

  /// At the primary monitor's top-left, which is always on screen, at the
  /// bottom of the z-order.
  fn create(color: Color) -> crate::Result<Self> {
    let rect = Rect::from_xy(0, 0, FILL_SOURCE_SIZE, FILL_SOURCE_SIZE);
    let mut window = OverlayWindow::create(
      OverlayKind::BackdropSource,
      &rect,
      HWND_BOTTOM,
    )?;
    window.set_alpha(0)?;

    let composition = SurrogateFill::create(window.hwnd())?;
    composition.set_covered((0, 0))?;
    composition.set_color(Some(color))?;
    window.place_at_bottom(&rect)?;

    Ok(Self {
      _composition: composition,
      window,
      color,
    })
  }

  pub(crate) fn hwnd(&self) -> windows::Win32::Foundation::HWND {
    self.window.hwnd()
  }

  /// The part of the source its copies draw, in its own coordinates.
  pub(crate) fn sample_rect() -> RECT {
    RECT {
      left: FILL_SOURCE_MARGIN,
      top: FILL_SOURCE_MARGIN,
      right: FILL_SOURCE_SIZE - FILL_SOURCE_MARGIN,
      bottom: FILL_SOURCE_SIZE - FILL_SOURCE_MARGIN,
    }
  }
}
