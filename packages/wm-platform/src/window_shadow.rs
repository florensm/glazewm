//! The drop shadow DWM draws around a window, redrawn for the copies of
//! the window that stand in for it while it moves: DWM leaves its shadow
//! out of thumbnails, so a copy alone would move without one.

use windows::Win32::{
  Foundation::{BOOL, HWND},
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_NCRENDERING_ENABLED},
  UI::{
    HiDpi::GetDpiForWindow,
    WindowsAndMessaging::{
      SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOSENDCHANGING,
      SWP_SHOWWINDOW,
    },
  },
};

use crate::{
  overlay_window::{OverlayKind, OverlayWindow},
  paint::{Canvas, ShadowLayer},
  platform_impl::{self, composition::ShadowVisual},
  resize_session::compute_border_inset,
  CornerStyle, Rect,
};

/// Windows 11's shadow around an active window at 96 DPI, fitted to its
/// falloff measured on each side of one (to within 1.3/255 RMS).
const ACTIVE_LAYERS: [ShadowLayer; 2] = [
  ShadowLayer {
    alpha: 0.181,
    sigma: 13.0,
    offset: 0.0,
  },
  ShadowLayer {
    alpha: 0.236,
    sigma: 17.75,
    offset: 34.0,
  },
];

/// [`ACTIVE_LAYERS`] for an inactive window, which DWM gives a fainter,
/// tighter shadow (fitted to within 1.0/255 RMS).
const INACTIVE_LAYERS: [ShadowLayer; 2] = [
  ShadowLayer {
    alpha: 0.143,
    sigma: 6.25,
    offset: 1.5,
  },
  ShadowLayer {
    alpha: 0.151,
    sigma: 17.5,
    offset: 34.75,
  },
];

/// How many of its sigmas a layer reaches past its rect: the rest is
/// below a tenth of a level at the layers' alpha.
const REACH_SIGMAS: f32 = 3.0;

/// How a window's shadow looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShadowStyle {
  /// Whether the window is drawn active, with the deeper shadow.
  pub is_active: bool,
  /// The window's DPI, as a percentage of 96.
  pub scale_percent: u32,
  /// Radius of the window's rounded corners, in pixels.
  pub corner_radius: i32,
}

/// Picture of a shadow around a small window, which a nine-grid stretches
/// to the size of any larger one: its edges are the same all along their
/// middle, and its middle is the window, which casts nothing.
pub(crate) struct ShadowImage {
  pub width: i32,
  pub height: i32,
  /// Premultiplied `0xAARRGGBB` pixels, top-down.
  pub pixels: Vec<u32>,
  /// Left, top, right and bottom nine-grid insets, in pixels.
  pub insets: [i32; 4],
}

impl ShadowStyle {
  /// The style of `hwnd`'s shadow, drawn active or not, or `None` when DWM
  /// draws it none: frameless, maximized, or with no frame rendering.
  pub(crate) fn of(
    hwnd: HWND,
    is_active: bool,
    corner_style: CornerStyle,
  ) -> Option<Self> {
    let window = platform_impl::NativeWindow::new(hwnd.0);
    if window.is_maximized() || !is_frame_rendered(hwnd) {
      return None;
    }

    // DWM draws the shadow in the invisible resize border of windows that
    // have one, which a frameless window lacks.
    let border = compute_border_inset(hwnd);
    if border.left == 0 && border.right == 0 && border.bottom == 0 {
      return None;
    }

    // SAFETY: A stale handle just returns 0.
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let scale_percent = if dpi == 0 { 100 } else { dpi * 100 / 96 };

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let corner_radius =
      (corner_style.approx_radius_px() * scale_percent as f32 / 100.0)
        .round() as i32;

    Some(Self {
      is_active,
      scale_percent,
      corner_radius,
    })
  }

  /// The shadow's layers at this style's scale.
  #[allow(clippy::cast_precision_loss)]
  fn layers(self) -> [ShadowLayer; 2] {
    let scale = self.scale_percent as f32 / 100.0;
    let layers = if self.is_active {
      ACTIVE_LAYERS
    } else {
      INACTIVE_LAYERS
    };

    layers.map(|layer| ShadowLayer {
      sigma: layer.sigma * scale,
      offset: layer.offset * scale,
      ..layer
    })
  }

  /// How far the shadow reaches past a window's left, top, right and
  /// bottom edges, along with how far into the window from each side it
  /// still changes along the edges next to it.
  fn extents(self) -> ([i32; 4], [i32; 4]) {
    let layers = self.layers();
    let farthest = |reach: fn(&ShadowLayer) -> f32| {
      #[allow(clippy::cast_possible_truncation)]
      let pixels =
        layers.iter().map(reach).fold(0.0, f32::max).ceil() as i32;
      pixels.max(0)
    };

    let side = farthest(|layer| layer.sigma * REACH_SIGMAS);
    let above =
      farthest(|layer| layer.sigma * REACH_SIGMAS - layer.offset);
    let below =
      farthest(|layer| layer.sigma * REACH_SIGMAS + layer.offset);

    // Each layer is moved down: its top edge changes the sides deeper into
    // the window than its bottom edge does.
    let radius = self.corner_radius.max(0);
    let margins = [side, above, side, below];
    let corners =
      [side + radius, below + radius, side + radius, above + radius];

    (margins, corners)
  }

  /// Left, top, right and bottom distances from a window's frame to the
  /// edges of its shadow.
  pub(crate) fn margins(self) -> [i32; 4] {
    self.extents().0
  }

  /// Paints the shadow around the smallest window a nine-grid of it can
  /// stretch to every size.
  pub(crate) fn paint(self) -> ShadowImage {
    let (
      [left, top, right, bottom],
      [inset_x, inset_top, _, inset_bottom],
    ) = self.extents();

    // One pixel between the corners, which the nine-grid stretches.
    let frame_width = inset_x * 2 + 1;
    let frame_height = inset_top + 1 + inset_bottom;
    let frame = Rect::from_xy(left, top, frame_width, frame_height);

    let width = left + frame_width + right;
    let height = top + frame_height + bottom;
    let mut pixels =
      vec![0; usize::try_from(width * height).unwrap_or_default()];

    Canvas {
      pixels: &mut pixels,
      width,
      height,
    }
    .fill_layered_shadow_around(
      &frame,
      self.corner_radius,
      &self.layers(),
    );

    ShadowImage {
      width,
      height,
      pixels,
      insets: [
        left + inset_x,
        top + inset_top,
        inset_x + right,
        inset_bottom + bottom,
      ],
    }
  }
}

/// The rect of the shadow of a window with its frame at `frame`.
fn outset(frame: &Rect, style: ShadowStyle) -> Rect {
  let [left, top, right, bottom] = style.margins();

  Rect::from_ltrb(
    frame.left - left,
    frame.top - top,
    frame.right + right,
    frame.bottom + bottom,
  )
}

/// Whether DWM renders `hwnd`'s non-client area, which its shadow is part
/// of.
fn is_frame_rendered(hwnd: HWND) -> bool {
  let mut enabled = BOOL(0);

  // SAFETY: `enabled` outlives the call and matches the attribute's size;
  // a stale handle just fails.
  unsafe {
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_NCRENDERING_ENABLED,
      std::ptr::from_mut(&mut enabled).cast(),
      u32::try_from(std::mem::size_of::<BOOL>()).unwrap_or_default(),
    )
  }
  .is_ok_and(|()| enabled.as_bool())
}

/// A window's shadow, drawn into a window of its own that is hidden by its
/// alpha: thumbnails of it leave that out, so a copy of the window can
/// carry one of it along (see `WorkspaceSurrogate::carry_shadow`).
pub(crate) struct WindowShadow {
  /// Declared before `window`, which its visual tree is rooted to.
  _composition: ShadowVisual,
  window: OverlayWindow,
  style: ShadowStyle,
  /// Where `window` is: the window's frame, outset by the margins.
  rect: Rect,
}

impl WindowShadow {
  /// Draws the shadow of a window with its frame at `frame`.
  ///
  /// Kept at the bottom of the z-order, where nothing settles behind or
  /// in front of it: where it is doesn't change what thumbnails of it
  /// show.
  pub(crate) fn create(
    frame: &Rect,
    style: ShadowStyle,
  ) -> crate::Result<Self> {
    let rect = outset(frame, style);

    let window =
      OverlayWindow::create(OverlayKind::Shadow, &rect, HWND_BOTTOM)?;
    window.set_alpha(0)?;
    let composition = ShadowVisual::create(window.hwnd(), style)?;

    let mut shadow = Self {
      _composition: composition,
      window,
      style,
      rect: Rect::from_xy(0, 0, 0, 0),
    };
    shadow.place(rect)?;

    Ok(shadow)
  }

  /// Moves the shadow around a window with its frame at `frame`.
  pub(crate) fn set_frame(&mut self, frame: &Rect) -> crate::Result<()> {
    let rect = outset(frame, self.style);
    if rect == self.rect {
      return Ok(());
    }

    self.place(rect)
  }

  /// Shows the window at `rect`, which its visual tree stretches to fill.
  fn place(&mut self, rect: Rect) -> crate::Result<()> {
    // SAFETY: `self.window` is valid for the lifetime of `self`.
    unsafe {
      SetWindowPos(
        self.window.hwnd(),
        HWND_BOTTOM,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    }
    .map_err(|err| {
      crate::Error::Platform(format!(
        "Shadow window placement failed: {err}."
      ))
    })?;

    self.rect = rect;
    Ok(())
  }

  pub(crate) fn hwnd(&self) -> HWND {
    self.window.hwnd()
  }

  /// The shadow's rect around the window's frame.
  pub(crate) fn rect(&self) -> &Rect {
    &self.rect
  }
}

#[cfg(test)]
mod tests {
  use super::{ShadowImage, ShadowStyle};

  #[derive(Clone, Copy)]
  enum Side {
    Left,
    Top,
    Bottom,
  }

  fn style(is_active: bool) -> ShadowStyle {
    ShadowStyle {
      is_active,
      scale_percent: 100,
      corner_radius: 8,
    }
  }

  fn alpha_at(image: &ShadowImage, x: i32, y: i32) -> u32 {
    image.pixels[usize::try_from(y * image.width + x).unwrap()] >> 24
  }

  /// The shadow's alpha in the `distance`th pixel past the middle of its
  /// window's `side`.
  fn alpha_past(is_active: bool, side: Side, distance: i32) -> f32 {
    let style = style(is_active);
    let image = style.paint();
    let [left, top, _, bottom] = style.margins();
    // The column and row the nine-grid stretches along the edges.
    let [middle_x, middle_y, _, _] = image.insets;

    let (x, y) = match side {
      Side::Left => (left - distance, middle_y),
      Side::Top => (middle_x, top - distance),
      Side::Bottom => (middle_x, image.height - bottom - 1 + distance),
    };

    #[allow(clippy::cast_precision_loss)]
    let alpha = alpha_at(&image, x, y) as f32 / 255.0;
    alpha
  }

  #[test]
  fn shadow_matches_the_one_measured_around_windows() {
    // Brightness past an active window's frame on a white background.
    let measured = [
      (Side::Left, 3, 217.0),
      (Side::Left, 20, 244.0),
      (Side::Top, 3, 231.0),
      (Side::Bottom, 3, 180.0),
      (Side::Bottom, 40, 231.0),
    ];

    for (side, distance, brightness) in measured {
      let expected = 1.0 - brightness / 255.0;
      let alpha = alpha_past(true, side, distance);
      assert!(
        (alpha - expected).abs() < 0.03,
        "{distance}px out: {alpha} vs {expected}"
      );
    }
  }

  #[test]
  fn shadow_fades_out_within_its_margins() {
    for is_active in [true, false] {
      let image = style(is_active).paint();

      for x in 0..image.width {
        assert!(alpha_at(&image, x, 0) <= 1, "top row at {x}");
        assert!(
          alpha_at(&image, x, image.height - 1) <= 1,
          "bottom at {x}"
        );
      }
      for y in 0..image.height {
        assert!(alpha_at(&image, 0, y) <= 1, "left column at {y}");
        assert!(alpha_at(&image, image.width - 1, y) <= 1, "right at {y}");
      }
    }
  }

  #[test]
  fn window_casts_no_shadow_on_itself() {
    let image = style(true).paint();
    let [left, top, right, bottom] = image.insets;

    assert_eq!(alpha_at(&image, left, top), 0);
    assert!(left + right < image.width && top + bottom < image.height);
  }

  #[test]
  fn inactive_shadow_is_fainter() {
    for side in [Side::Left, Side::Top, Side::Bottom] {
      assert!(alpha_past(false, side, 10) < alpha_past(true, side, 10));
    }
  }
}
