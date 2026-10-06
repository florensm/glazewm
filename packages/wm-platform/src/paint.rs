//! Anti-aliased drawing into a premultiplied 32-bit pixel buffer, for
//! shapes GDI can only draw aliased (rounded corners, rings) and for
//! compositing GDI output onto translucent pixels, which GDI itself
//! can't do.
//!
//! Pixels are `0xAARRGGBB`, premultiplied by alpha, top-down, as used by a
//! 32-bit DIB section passed to `UpdateLayeredWindow`.

use crate::{Color, Rect};

/// Samples per axis when estimating a pixel's coverage.
const SUBSAMPLES: i32 = 4;

/// A buffer of `width` x `height` premultiplied pixels.
pub(crate) struct Canvas<'a> {
  pub pixels: &'a mut [u32],
  pub width: i32,
  pub height: i32,
}

impl Canvas<'_> {
  /// Fills `rect` with `color`, rounding its corners by `radius`.
  pub fn fill_rounded_rect(
    &mut self,
    rect: &Rect,
    radius: i32,
    color: Color,
  ) {
    let radius = clamp_radius(rect, radius);

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      for x in rect.left.max(0)..rect.right.min(self.width) {
        let coverage = rounded_rect_coverage(x, y, rect, radius);
        self.blend(x, y, color, coverage);
      }
    }
  }

  /// Draws a ring of `thickness` just inside `rect`, rounded by `radius`
  /// on its outer edge and concentrically on its inner edge.
  pub fn stroke_rounded_rect(
    &mut self,
    rect: &Rect,
    radius: i32,
    thickness: i32,
    color: Color,
  ) {
    let radius = clamp_radius(rect, radius);
    let inner = rect.inset(thickness);
    let inner_radius = clamp_radius(&inner, radius - thickness);

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      for x in rect.left.max(0)..rect.right.min(self.width) {
        let is_inside_inner = x >= inner.left
          && x < inner.right
          && y >= inner.top
          && y < inner.bottom;

        let inner_coverage = if is_inside_inner {
          rounded_rect_coverage(x, y, &inner, inner_radius)
        } else {
          0.0
        };

        let coverage =
          rounded_rect_coverage(x, y, rect, radius) - inner_coverage;
        self.blend(x, y, color, coverage);
      }
    }
  }

  /// Blends `color` through `mask`, a row-major coverage mask (0-255)
  /// `mask_width` pixels wide, with its top-left at (`left`, `top`).
  pub fn blend_mask(
    &mut self,
    left: i32,
    top: i32,
    mask: &[u8],
    mask_width: i32,
    color: Color,
  ) {
    let Ok(row_len) = usize::try_from(mask_width) else {
      return;
    };
    if row_len == 0 {
      return;
    }

    for (row, coverages) in (0..).zip(mask.chunks_exact(row_len)) {
      for (column, coverage) in (0..).zip(coverages) {
        if *coverage > 0 {
          self.blend(
            left + column,
            top + row,
            color,
            f32::from(*coverage) / 255.0,
          );
        }
      }
    }
  }

  /// Composites premultiplied `source` pixels, `source_width` pixels wide,
  /// over the canvas with their top-left at (`left`, `top`), faded to
  /// `opacity`.
  pub fn draw_premultiplied(
    &mut self,
    left: i32,
    top: i32,
    source: &[u32],
    source_width: i32,
    opacity: f32,
  ) {
    let Ok(row_len) = usize::try_from(source_width) else {
      return;
    };
    if row_len == 0 {
      return;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let opacity = (opacity.clamp(0.0, 1.0) * 255.0).round() as u32;

    for (row, pixels) in (0..).zip(source.chunks_exact(row_len)) {
      for (column, pixel) in (0..).zip(pixels) {
        if let Some(target) = self.pixel_mut(left + column, top + row) {
          *target = over(scale(*pixel, opacity), *target);
        }
      }
    }
  }

  /// Makes `rect`, with its corners rounded by `radius`, transparent:
  /// a hole for whatever is composited underneath the canvas.
  pub fn erase_rounded_rect(&mut self, rect: &Rect, radius: i32) {
    let radius = clamp_radius(rect, radius);

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      for x in rect.left.max(0)..rect.right.min(self.width) {
        let coverage = rounded_rect_coverage(x, y, rect, radius);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let keep = ((1.0 - coverage) * 255.0).round() as u32;

        if let Some(pixel) = self.pixel_mut(x, y) {
          *pixel = scale(*pixel, keep);
        }
      }
    }
  }

  fn pixel_mut(&mut self, x: i32, y: i32) -> Option<&mut u32> {
    if x < 0 || y < 0 || x >= self.width || y >= self.height {
      return None;
    }

    let index = usize::try_from(y * self.width + x).ok()?;
    self.pixels.get_mut(index)
  }

  /// Blends `color` over the pixel at (`x`, `y`) with `coverage` in 0..=1.
  fn blend(&mut self, x: i32, y: i32, color: Color, coverage: f32) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha =
      (f32::from(color.a) * coverage.clamp(0.0, 1.0)).round() as u8;
    if alpha == 0 {
      return;
    }

    if let Some(pixel) = self.pixel_mut(x, y) {
      *pixel = over(premultiply(color, alpha), *pixel);
    }
  }
}

/// `color`'s channels premultiplied by `alpha`, packed as `0xAARRGGBB`.
fn premultiply(color: Color, alpha: u8) -> u32 {
  let alpha = u32::from(alpha);
  let channel = |value: u8| u32::from(value) * alpha / 255;

  (alpha << 24)
    | (channel(color.r) << 16)
    | (channel(color.g) << 8)
    | channel(color.b)
}

/// Premultiplied `pixel` with every channel scaled by `factor` / 255.
fn scale(pixel: u32, factor: u32) -> u32 {
  if factor >= 255 {
    return pixel;
  }

  [24, 16, 8, 0].into_iter().fold(0, |result, shift| {
    result | ((((pixel >> shift) & 0xff) * factor / 255) << shift)
  })
}

/// Premultiplied `source` composited over `target`.
fn over(source: u32, target: u32) -> u32 {
  let inverse = 255 - (source >> 24);

  [24, 16, 8, 0].into_iter().fold(0, |result, shift| {
    let channel = |pixel: u32| (pixel >> shift) & 0xff;
    let value =
      (channel(source) + channel(target) * inverse / 255).min(255);
    result | (value << shift)
  })
}

/// Coverage mask of text drawn white on black: each pixel's brightest
/// channel.
pub(crate) fn mask_from_white_on_black(pixels: &[u32]) -> Vec<u8> {
  pixels
    .iter()
    .map(|pixel| {
      let [_, r, g, b] = pixel.to_be_bytes();
      r.max(g).max(b)
    })
    .collect()
}

/// Recovers premultiplied pixels with alpha from the same image drawn
/// opaquely onto black and onto white, i.e. GDI output that carries no
/// usable alpha of its own (e.g. `DrawIconEx` with masked icons).
///
/// Where the backgrounds still show through, the two differ by exactly the
/// transparency; drawn onto black, the image is already premultiplied.
pub(crate) fn matte_from_backgrounds(
  on_black: &[u32],
  on_white: &[u32],
) -> Vec<u32> {
  on_black
    .iter()
    .zip(on_white)
    .map(|(black, white)| {
      let [_, black_r, black_g, black_b] = black.to_be_bytes();
      let [_, white_r, white_g, white_b] = white.to_be_bytes();

      let transparency = white_r
        .saturating_sub(black_r)
        .max(white_g.saturating_sub(black_g))
        .max(white_b.saturating_sub(black_b));
      let alpha = 255 - transparency;

      u32::from_be_bytes([
        alpha,
        black_r.min(alpha),
        black_g.min(alpha),
        black_b.min(alpha),
      ])
    })
    .collect()
}

/// Limits `radius` to half of `rect`'s shorter side.
fn clamp_radius(rect: &Rect, radius: i32) -> i32 {
  let max = rect.width().min(rect.height()) / 2;
  radius.clamp(0, max.max(0))
}

/// Fraction of the pixel at (`x`, `y`), which lies within `rect`, that is
/// inside `rect` with its corners rounded by `radius`.
fn rounded_rect_coverage(x: i32, y: i32, rect: &Rect, radius: i32) -> f32 {
  if radius == 0 {
    return 1.0;
  }

  let is_left = x < rect.left + radius;
  let is_right = x >= rect.right - radius;
  let is_top = y < rect.top + radius;
  let is_bottom = y >= rect.bottom - radius;

  if !(is_left || is_right) || !(is_top || is_bottom) {
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

#[cfg(test)]
mod tests {
  use super::{mask_from_white_on_black, matte_from_backgrounds, Canvas};
  use crate::{Color, Rect};

  const WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
  };

  fn canvas(pixels: &mut [u32], width: i32) -> Canvas<'_> {
    let height = i32::try_from(pixels.len()).unwrap_or(0) / width;
    Canvas {
      pixels,
      width,
      height,
    }
  }

  #[test]
  fn fill_is_premultiplied() {
    let mut pixels = vec![0u32; 4];
    canvas(&mut pixels, 2).fill_rounded_rect(
      &Rect::from_ltrb(0, 0, 2, 2),
      0,
      Color {
        r: 200,
        g: 100,
        b: 0,
        a: 128,
      },
    );

    assert_eq!(pixels[0] >> 24, 128);
    assert_eq!((pixels[0] >> 16) & 0xff, 200 * 128 / 255);
    assert_eq!((pixels[0] >> 8) & 0xff, 100 * 128 / 255);
    assert!(pixels.iter().all(|pixel| *pixel == pixels[0]));
  }

  #[test]
  fn rounded_rect_fills_center_and_clears_corners() {
    let mut pixels = vec![0u32; 20 * 20];
    canvas(&mut pixels, 20).fill_rounded_rect(
      &Rect::from_ltrb(0, 0, 20, 20),
      8,
      WHITE,
    );

    assert_eq!(pixels[10 * 20 + 10], 0xffff_ffff);
    assert_eq!(pixels[0] >> 24, 0, "corner pixel stays transparent");
    let edge_alpha = pixels[2 * 20 + 2] >> 24;
    assert!(edge_alpha > 0 && edge_alpha < 255, "edge is anti-aliased");
    assert_eq!(pixels[10], 0xffff_ffff, "straight edge is fully covered");
  }

  #[test]
  fn ring_leaves_its_inside_untouched() {
    let mut pixels = vec![0u32; 20 * 20];
    canvas(&mut pixels, 20).stroke_rounded_rect(
      &Rect::from_ltrb(0, 0, 20, 20),
      6,
      3,
      WHITE,
    );

    assert_eq!(pixels[10 * 20 + 1], 0xffff_ffff, "ring is drawn");
    assert_eq!(pixels[10 * 20 + 10], 0, "inside stays transparent");
    assert_eq!(pixels[10 * 20 + 3], 0, "ring is 3px thick");
    assert_eq!(pixels[0] >> 24, 0, "outer corner stays transparent");
  }

  #[test]
  fn translucent_blend_composites_over_existing_pixels() {
    let mut pixels = vec![0u32; 1];
    let mut canvas = canvas(&mut pixels, 1);
    canvas.fill_rounded_rect(
      &Rect::from_ltrb(0, 0, 1, 1),
      0,
      Color {
        r: 0,
        g: 0,
        b: 0,
        a: 128,
      },
    );
    canvas.fill_rounded_rect(
      &Rect::from_ltrb(0, 0, 1, 1),
      0,
      Color { a: 128, ..WHITE },
    );

    // Alpha: 128 + 128 * 127 / 255.
    assert_eq!(pixels[0] >> 24, 191);
    assert_eq!((pixels[0] >> 16) & 0xff, 128);
  }

  #[test]
  fn mask_blends_color_by_coverage() {
    let mut pixels = vec![0u32; 2 * 2];
    canvas(&mut pixels, 2).blend_mask(1, 0, &[255, 0], 1, WHITE);

    assert_eq!(pixels, [0, 0xffff_ffff, 0, 0]);
  }

  #[test]
  fn premultiplied_source_is_clipped_to_canvas() {
    let mut pixels = vec![0u32; 2 * 2];
    canvas(&mut pixels, 2).draw_premultiplied(
      1,
      1,
      &[0x8080_8080, 0xffff_ffff],
      2,
      1.0,
    );

    assert_eq!(pixels, [0, 0, 0, 0x8080_8080]);
  }

  #[test]
  fn premultiplied_source_fades_with_opacity() {
    let mut pixels = vec![0u32; 1];
    canvas(&mut pixels, 1).draw_premultiplied(
      0,
      0,
      &[0xffff_ffff],
      1,
      0.5,
    );

    assert_eq!(pixels, [0x8080_8080]);
  }

  #[test]
  fn erase_punches_a_rounded_hole() {
    let mut pixels = vec![0xffff_ffffu32; 20 * 20];
    canvas(&mut pixels, 20)
      .erase_rounded_rect(&Rect::from_ltrb(0, 0, 20, 20), 8);

    assert_eq!(pixels[10 * 20 + 10], 0, "middle is cleared");
    assert_eq!(pixels[0], 0xffff_ffff, "corner outside stays");
    let edge = pixels[2 * 20 + 2] >> 24;
    assert!(edge > 0 && edge < 255, "edge is anti-aliased");
  }

  #[test]
  fn text_mask_uses_brightest_channel() {
    assert_eq!(
      mask_from_white_on_black(&[0, 0x0040_8020, 0x00ff_ffff]),
      [0, 0x80, 0xff]
    );
  }

  #[test]
  fn matte_recovers_alpha_and_premultiplied_color() {
    let matte = matte_from_backgrounds(
      // Opaque red, half-transparent white, fully transparent.
      &[0x00ff_0000, 0x0080_8080, 0x0000_0000],
      &[0x00ff_0000, 0x00ff_ffff, 0x00ff_ffff],
    );

    assert_eq!(matte, [0xffff_0000, 0x8080_8080, 0x0000_0000]);
  }
}
