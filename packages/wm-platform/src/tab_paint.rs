//! Anti-aliased drawing into a premultiplied 32-bit pixel buffer, for
//! shapes GDI can only draw aliased (rounded corners, diagonal strokes).
//!
//! Pixels are `0xAARRGGBB`, premultiplied by alpha, top-down, as used by a
//! 32-bit DIB section passed to `UpdateLayeredWindow`.

use crate::tab_layout::TabRect;

/// Straight (non-premultiplied) RGBA color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

/// Radius of each corner of a rounded rect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CornerRadii {
  pub top_left: i32,
  pub top_right: i32,
  pub bottom_right: i32,
  pub bottom_left: i32,
}

impl CornerRadii {
  /// The same `radius` on every corner.
  #[must_use]
  pub fn uniform(radius: i32) -> Self {
    Self {
      top_left: radius,
      top_right: radius,
      bottom_right: radius,
      bottom_left: radius,
    }
  }

  /// Limits each radius to half of `rect`'s shorter side.
  fn clamped(self, rect: TabRect) -> Self {
    let max = rect.width().min(rect.height()) / 2;
    let clamp = |radius: i32| radius.clamp(0, max.max(0));

    Self {
      top_left: clamp(self.top_left),
      top_right: clamp(self.top_right),
      bottom_right: clamp(self.bottom_right),
      bottom_left: clamp(self.bottom_left),
    }
  }
}

/// A pixel buffer of `width` x `height` premultiplied pixels.
pub struct Canvas<'a> {
  pub pixels: &'a mut [u32],
  pub width: i32,
  pub height: i32,
}

/// Samples per axis when estimating a pixel's coverage.
const SUBSAMPLES: i32 = 4;

impl Canvas<'_> {
  /// Blends `color` over the pixel at (`x`, `y`) with `coverage` in 0..=1.
  #[allow(clippy::many_single_char_names)]
  fn blend(&mut self, x: i32, y: i32, color: Rgba, coverage: f32) {
    if x < 0 || y < 0 || x >= self.width || y >= self.height {
      return;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha =
      (f32::from(color.a) * coverage.clamp(0.0, 1.0)).round() as u32;
    if alpha == 0 {
      return;
    }

    let Ok(index) = usize::try_from(y * self.width + x) else {
      return;
    };
    let Some(pixel) = self.pixels.get_mut(index) else {
      return;
    };

    let premultiply = |channel: u8| u32::from(channel) * alpha / 255;
    let inverse = 255 - alpha;
    let channel = |shift: u32| (*pixel >> shift) & 0xff;

    let a = alpha + channel(24) * inverse / 255;
    let r = premultiply(color.r) + channel(16) * inverse / 255;
    let g = premultiply(color.g) + channel(8) * inverse / 255;
    let b = premultiply(color.b) + channel(0) * inverse / 255;

    *pixel = (a << 24) | (r << 16) | (g << 8) | b;
  }

  /// Fills `rect` with `color`, rounding its corners by `radius`.
  pub fn fill_rounded_rect(
    &mut self,
    rect: TabRect,
    radius: i32,
    color: Rgba,
  ) {
    self.fill_rect_with_corners(rect, CornerRadii::uniform(radius), color);
  }

  /// Fills `rect` with `color`, rounding each corner by its own radius.
  pub fn fill_rect_with_corners(
    &mut self,
    rect: TabRect,
    radii: CornerRadii,
    color: Rgba,
  ) {
    let radii = radii.clamped(rect);

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      for x in rect.left.max(0)..rect.right.min(self.width) {
        let coverage = rounded_rect_coverage(x, y, rect, radii);
        self.blend(x, y, color, coverage);
      }
    }
  }

  /// Draws a line from (`x0`, `y0`) to (`x1`, `y1`) of the given
  /// `thickness`, with round caps.
  pub fn stroke_line(
    &mut self,
    (x0, y0): (f32, f32),
    (x1, y1): (f32, f32),
    thickness: f32,
    color: Rgba,
  ) {
    let half = thickness / 2.0;

    #[allow(clippy::cast_possible_truncation)]
    let (left, top, right, bottom) = (
      (x0.min(x1) - half).floor() as i32,
      (y0.min(y1) - half).floor() as i32,
      (x0.max(x1) + half).ceil() as i32,
      (y0.max(y1) + half).ceil() as i32,
    );

    for y in top..=bottom {
      for x in left..=right {
        #[allow(clippy::cast_precision_loss)]
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        let distance = distance_to_segment((px, py), (x0, y0), (x1, y1));

        // One pixel of falloff gives a smooth edge.
        self.blend(x, y, color, half + 0.5 - distance);
      }
    }
  }
}

/// Fraction of the pixel at (`x`, `y`) inside `rect` with rounded corners.
fn rounded_rect_coverage(
  x: i32,
  y: i32,
  rect: TabRect,
  radii: CornerRadii,
) -> f32 {
  let is_left = x < rect.left + rect.width() / 2;
  let is_top = y < rect.top + rect.height() / 2;

  let radius = match (is_left, is_top) {
    (true, true) => radii.top_left,
    (false, true) => radii.top_right,
    (false, false) => radii.bottom_right,
    (true, false) => radii.bottom_left,
  };

  let in_corner_x = if is_left {
    x < rect.left + radius
  } else {
    x >= rect.right - radius
  };
  let in_corner_y = if is_top {
    y < rect.top + radius
  } else {
    y >= rect.bottom - radius
  };

  if radius == 0 || !(in_corner_x && in_corner_y) {
    return 1.0;
  }

  #[allow(clippy::cast_precision_loss)]
  let (center_x, center_y, radius) = (
    if is_left {
      (rect.left + radius) as f32
    } else {
      (rect.right - radius) as f32
    },
    if is_top {
      (rect.top + radius) as f32
    } else {
      (rect.bottom - radius) as f32
    },
    radius as f32,
  );

  let mut inside = 0;
  for sub_y in 0..SUBSAMPLES {
    for sub_x in 0..SUBSAMPLES {
      #[allow(clippy::cast_precision_loss)]
      let (sample_x, sample_y) = (
        x as f32 + (sub_x as f32 + 0.5) / SUBSAMPLES as f32,
        y as f32 + (sub_y as f32 + 0.5) / SUBSAMPLES as f32,
      );
      let (dx, dy) = (sample_x - center_x, sample_y - center_y);

      if dx * dx + dy * dy <= radius * radius {
        inside += 1;
      }
    }
  }

  #[allow(clippy::cast_precision_loss)]
  let coverage = inside as f32 / (SUBSAMPLES * SUBSAMPLES) as f32;
  coverage
}

fn distance_to_segment(
  (px, py): (f32, f32),
  (x0, y0): (f32, f32),
  (x1, y1): (f32, f32),
) -> f32 {
  let (dx, dy) = (x1 - x0, y1 - y0);
  let length_sq = dx * dx + dy * dy;

  let t = if length_sq == 0.0 {
    0.0
  } else {
    (((px - x0) * dx + (py - y0) * dy) / length_sq).clamp(0.0, 1.0)
  };

  let (cx, cy) = (x0 + t * dx, y0 + t * dy);
  ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
  use super::{Canvas, CornerRadii, Rgba};
  use crate::tab_layout::TabRect;

  const WHITE: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
  };

  fn rect(left: i32, top: i32, right: i32, bottom: i32) -> TabRect {
    TabRect {
      left,
      top,
      right,
      bottom,
    }
  }

  #[test]
  fn rounded_rect_fills_center_and_clears_corners() {
    let mut pixels = vec![0u32; 20 * 20];
    let mut canvas = Canvas {
      pixels: &mut pixels,
      width: 20,
      height: 20,
    };

    canvas.fill_rounded_rect(rect(0, 0, 20, 20), 8, WHITE);

    assert_eq!(pixels[10 * 20 + 10], 0xffff_ffff);
    assert_eq!(pixels[0] >> 24, 0, "corner pixel stays transparent");
    let edge_alpha = pixels[2 * 20 + 2] >> 24;
    assert!(edge_alpha > 0 && edge_alpha < 255, "edge is anti-aliased");
  }

  #[test]
  fn square_corners_stay_filled() {
    let mut pixels = vec![0u32; 20 * 20];
    let mut canvas = Canvas {
      pixels: &mut pixels,
      width: 20,
      height: 20,
    };

    let radii = CornerRadii {
      top_left: 8,
      top_right: 8,
      ..CornerRadii::default()
    };
    canvas.fill_rect_with_corners(rect(0, 0, 20, 20), radii, WHITE);

    assert_eq!(pixels[0] >> 24, 0, "rounded corner stays transparent");
    assert_eq!(pixels[19] >> 24, 0, "rounded corner stays transparent");
    assert_eq!(pixels[19 * 20], 0xffff_ffff, "square corner is filled");
    assert_eq!(
      pixels[19 * 20 + 19],
      0xffff_ffff,
      "square corner is filled"
    );
  }

  #[test]
  fn translucent_fill_is_premultiplied() {
    let mut pixels = vec![0u32; 4];
    let mut canvas = Canvas {
      pixels: &mut pixels,
      width: 2,
      height: 2,
    };

    canvas.fill_rounded_rect(
      rect(0, 0, 2, 2),
      0,
      Rgba {
        r: 200,
        g: 100,
        b: 0,
        a: 128,
      },
    );

    let pixel = pixels[0];
    assert_eq!(pixel >> 24, 128);
    assert_eq!((pixel >> 16) & 0xff, 200 * 128 / 255);
    assert_eq!((pixel >> 8) & 0xff, 100 * 128 / 255);
  }

  #[test]
  fn line_covers_its_path_only() {
    let mut pixels = vec![0u32; 10 * 10];
    let mut canvas = Canvas {
      pixels: &mut pixels,
      width: 10,
      height: 10,
    };

    canvas.stroke_line((1.0, 1.0), (9.0, 9.0), 1.5, WHITE);

    assert!(pixels[5 * 10 + 5] >> 24 > 200);
    assert_eq!(pixels[9 * 10] >> 24, 0);
  }
}
