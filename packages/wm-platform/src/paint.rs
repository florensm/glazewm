//! Anti-aliased drawing into a premultiplied 32-bit pixel buffer, for
//! shapes GDI can only draw aliased (rounded corners, rings, diagonal
//! strokes) and for compositing GDI output onto translucent pixels, which
//! GDI itself can't do. Shared by the overview and stack tab bars.
//!
//! Pixels are `0xAARRGGBB`, premultiplied by alpha, top-down, as used by a
//! 32-bit DIB section passed to `UpdateLayeredWindow`.

use std::{iter::Chain, ops::Range};

use crate::{Color, CornerRadii, Rect};

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
    self.fill_rect_with_corners(rect, CornerRadii::uniform(radius), color);
  }

  /// Fills `rect` with `color`, rounding each corner by its own radius.
  pub fn fill_rect_with_corners(
    &mut self,
    rect: &Rect,
    radii: CornerRadii,
    color: Color,
  ) {
    let radii = radii.clamped(rect.width(), rect.height());
    let solid = premultiply(color, color.a);

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      let span = solid_span(rect, radii, y);
      if let Some((from, to)) = span {
        for pixel in self.row_mut(y, from, to) {
          *pixel = over(solid, *pixel);
        }
      }

      for x in columns_around(rect.left, rect.right, span, self.width) {
        let coverage = rounded_rect_coverage(x, y, rect, radii);
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
    let radii =
      CornerRadii::uniform(radius).clamped(rect.width(), rect.height());
    let inner = rect.inset(thickness);
    let inner_radii = CornerRadii::uniform(radii.top_left - thickness)
      .clamped(inner.width(), inner.height());

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      // Wholly inside its inner edge, the ring covers nothing.
      let hollow = solid_span(&inner, inner_radii, y);

      for x in columns_around(rect.left, rect.right, hollow, self.width) {
        let is_inside_inner = x >= inner.left
          && x < inner.right
          && y >= inner.top
          && y < inner.bottom;

        let inner_coverage = if is_inside_inner {
          rounded_rect_coverage(x, y, &inner, inner_radii)
        } else {
          0.0
        };

        let coverage =
          rounded_rect_coverage(x, y, rect, radii) - inner_coverage;
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
    color: Color,
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

    // Source columns left of the canvas.
    let clipped = usize::try_from(left.saturating_neg())
      .unwrap_or(0)
      .min(row_len);

    for (row, pixels) in (0..).zip(source.chunks_exact(row_len)) {
      let targets = self.row_mut(top + row, left, left + source_width);
      for (target, pixel) in targets.iter_mut().zip(&pixels[clipped..]) {
        *target = over(scale(*pixel, opacity), *target);
      }
    }
  }

  /// Draws the soft shadow of `rect`, rounded by `radius`, around `hole`,
  /// rounded the same, which it leaves untouched: `color` under `rect`,
  /// fading out over `blur` pixels past its edges.
  #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
  pub fn fill_shadow_around(
    &mut self,
    rect: &Rect,
    hole: &Rect,
    radius: i32,
    blur: f32,
    color: Color,
  ) {
    let blur = blur.max(1.0);
    let reach = blur.ceil() as i32;
    let hole_radii =
      CornerRadii::uniform(radius).clamped(hole.width(), hole.height());
    let radius =
      radius.min(rect.width() / 2).min(rect.height() / 2).max(0) as f32;
    let (half_width, half_height) =
      (rect.width() as f32 / 2.0, rect.height() as f32 / 2.0);
    let (center_x, center_y) =
      (rect.left as f32 + half_width, rect.top as f32 + half_height);

    for y in
      (rect.top - reach).max(0)..(rect.bottom + reach).min(self.height)
    {
      let hidden = solid_span(hole, hole_radii, y);

      for x in columns_around(
        rect.left - reach,
        rect.right + reach,
        hidden,
        self.width,
      ) {
        // How far past the rounded rect's edge, negative inside it.
        let dx = (x as f32 + 0.5 - center_x).abs() - (half_width - radius);
        let dy =
          (y as f32 + 0.5 - center_y).abs() - (half_height - radius);
        let distance =
          dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius;
        let fade = 1.0 - (distance / blur).clamp(0.0, 1.0);

        // Partly shown where the hole's corners cut through.
        let is_in_hole = x >= hole.left
          && x < hole.right
          && y >= hole.top
          && y < hole.bottom;
        let shown = if is_in_hole {
          1.0 - rounded_rect_coverage(x, y, hole, hole_radii)
        } else {
          1.0
        };

        self.blend(x, y, color, fade * fade * shown);
      }
    }
  }

  /// Makes `rect`, with its corners rounded by `radius`, transparent:
  /// a hole for whatever is composited underneath the canvas.
  pub fn erase_rounded_rect(&mut self, rect: &Rect, radius: i32) {
    self.erase_rect_with_corners(rect, CornerRadii::uniform(radius));
  }

  /// Like [`Canvas::erase_rounded_rect`], with each corner rounded by its
  /// own radius.
  pub fn erase_rect_with_corners(
    &mut self,
    rect: &Rect,
    radii: CornerRadii,
  ) {
    let radii = radii.clamped(rect.width(), rect.height());

    for y in rect.top.max(0)..rect.bottom.min(self.height) {
      let span = solid_span(rect, radii, y);
      if let Some((from, to)) = span {
        self.row_mut(y, from, to).fill(0);
      }

      for x in columns_around(rect.left, rect.right, span, self.width) {
        let coverage = rounded_rect_coverage(x, y, rect, radii);

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

  /// Row `y`'s pixels from `left` up to `right`, clipped to the canvas.
  fn row_mut(&mut self, y: i32, left: i32, right: i32) -> &mut [u32] {
    let (left, right) = (left.max(0), right.min(self.width));
    if y < 0 || y >= self.height || left >= right {
      return &mut [];
    }

    let start = usize::try_from(y * self.width + left).unwrap_or_default();
    let end = usize::try_from(y * self.width + right).unwrap_or_default();
    self.pixels.get_mut(start..end).unwrap_or_default()
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
  // Opaque, empty and clear pixels, which most are.
  if source >> 24 == 255 || target == 0 {
    return source;
  }
  if source == 0 {
    return target;
  }

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

/// Columns of row `y` wholly inside `rect` with its corners rounded by
/// `radii`, which must be clamped to it: from the first up to past the
/// last, if there are any.
fn solid_span(
  rect: &Rect,
  radii: CornerRadii,
  y: i32,
) -> Option<(i32, i32)> {
  if y < rect.top || y >= rect.bottom {
    return None;
  }

  // The corners `rounded_rect_coverage` picks for the row.
  let is_top = y < rect.top + rect.height() / 2;
  let (left_radius, right_radius) = if is_top {
    (radii.top_left, radii.top_right)
  } else {
    (radii.bottom_left, radii.bottom_right)
  };
  let inset = |radius: i32| {
    let is_corner_row = if is_top {
      y < rect.top + radius
    } else {
      y >= rect.bottom - radius
    };
    if is_corner_row {
      radius
    } else {
      0
    }
  };

  let (left, right) = (
    rect.left + inset(left_radius),
    rect.right - inset(right_radius),
  );
  (left < right).then_some((left, right))
}

/// Columns from `left` up to `right` of a canvas `width` pixels wide, less
/// those of `skip`.
fn columns_around(
  left: i32,
  right: i32,
  skip: Option<(i32, i32)>,
  width: i32,
) -> Chain<Range<i32>, Range<i32>> {
  let left = left.max(0);
  let right = right.min(width).max(left);
  let (from, to) = skip.map_or((right, right), |(from, to)| {
    (from.clamp(left, right), to.clamp(left, right))
  });

  (left..from).chain(to..right)
}

/// Fraction of the pixel at (`x`, `y`), which lies within `rect`, that is
/// inside `rect` with its corners rounded by `radii`, which must be
/// clamped to it.
fn rounded_rect_coverage(
  x: i32,
  y: i32,
  rect: &Rect,
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

  let is_in_corner_x = if is_left {
    x < rect.left + radius
  } else {
    x >= rect.right - radius
  };
  let is_in_corner_y = if is_top {
    y < rect.top + radius
  } else {
    y >= rect.bottom - radius
  };

  if radius == 0 || !(is_in_corner_x && is_in_corner_y) {
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

/// Distance from (`px`, `py`) to the segment from (`x0`, `y0`) to
/// (`x1`, `y1`).
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
  use super::{
    mask_from_white_on_black, matte_from_backgrounds,
    rounded_rect_coverage, solid_span, Canvas,
  };
  use crate::{Color, CornerRadii, Rect};

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
  fn shadows_fade_out_past_the_edge_around_their_hole() {
    let mut pixels = vec![0u32; 40 * 48];
    canvas(&mut pixels, 40).fill_shadow_around(
      &Rect::from_ltrb(10, 12, 30, 32),
      &Rect::from_ltrb(10, 10, 30, 30),
      4,
      8.0,
      WHITE,
    );
    let alpha = |x: usize, y: usize| pixels[y * 40 + x] >> 24;

    assert_eq!(alpha(20, 20), 0, "none under the hole");
    assert_eq!(alpha(20, 31), 255, "full under the rect past the hole");
    assert!(alpha(20, 34) > alpha(20, 37), "fading away from it");
    assert!(alpha(20, 34) > 0 && alpha(20, 34) < 255);
    assert_eq!(alpha(20, 41), 0, "gone past the blur");
    assert!(alpha(31, 33) < alpha(20, 33), "rounder at the corners");

    let corner = alpha(11, 28);
    assert!(corner > 0 && corner < 255, "hole's corners anti-aliased");
  }

  #[test]
  fn solid_spans_are_wholly_covered() {
    let shapes = [
      (Rect::from_ltrb(0, 0, 20, 20), CornerRadii::uniform(8)),
      (Rect::from_ltrb(-5, -3, 6, 4), CornerRadii::uniform(9)),
      (
        Rect::from_ltrb(3, 5, 10, 30),
        CornerRadii {
          top_left: 3,
          bottom_right: 6,
          ..CornerRadii::default()
        },
      ),
    ];

    for (rect, radii) in shapes {
      let radii = radii.clamped(rect.width(), rect.height());
      let mut solid = 0;

      for y in rect.top..rect.bottom {
        let Some((from, to)) = solid_span(&rect, radii, y) else {
          continue;
        };
        assert!(rect.left <= from && to <= rect.right);

        for x in from..to {
          assert!(rounded_rect_coverage(x, y, &rect, radii) >= 1.0);
          solid += 1;
        }
      }
      assert!(solid > 0);
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
  fn square_corners_stay_filled() {
    let mut pixels = vec![0u32; 20 * 20];
    let radii = CornerRadii {
      top_left: 8,
      top_right: 8,
      ..CornerRadii::default()
    };
    canvas(&mut pixels, 20).fill_rect_with_corners(
      &Rect::from_ltrb(0, 0, 20, 20),
      radii,
      WHITE,
    );

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
  fn line_covers_its_path_only() {
    let mut pixels = vec![0u32; 10 * 10];
    canvas(&mut pixels, 10).stroke_line(
      (1.0, 1.0),
      (9.0, 9.0),
      1.5,
      WHITE,
    );

    assert!(pixels[5 * 10 + 5] >> 24 > 200);
    assert_eq!(pixels[9 * 10] >> 24, 0);
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
