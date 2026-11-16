//! The rows a rectangular copy of a rounded backdrop is cut into, so its
//! ends follow the corners' arcs (`RoundedBackdrop::Bands`).
//!
//! Each row at a corner is inset to where the arc is at the row's edge
//! nearer the middle, so the rows cover all of the rounded shape and
//! reach past it only by a step at each row's outer edge, which the
//! window's ring is drawn over.

use std::f32::consts::FRAC_PI_2;

use crate::Rect;

/// Most rows per corner pair: each is a thumbnail of its own, updated
/// every frame.
pub(crate) const MAX_BANDS: usize = 8;

/// How far past the arc a row may reach with no ring over it, in pixels.
const BARE_STEP: f32 = 1.5;

/// How far past an arc of `radius` its `bands` rows reach at most, cut
/// at equal angles: `radius * (sqrt(1 + sin(step)) - 1)` for a step of
/// `90° / bands`, at the outer end of the row whose middle is at 45°.
#[allow(clippy::cast_precision_loss)]
fn overshoot(radius: f32, bands: usize) -> f32 {
  let step = FRAC_PI_2 / bands.max(1) as f32;
  radius * ((1.0 + step.sin()).sqrt() - 1.0)
}

/// Rows per corner pair for corners of `radius`: the fewest that reach
/// past the arc no further than `ring_width`, of the ring drawn over the
/// edge, or [`BARE_STEP`] without one. At most [`MAX_BANDS`], and one per
/// pixel row the corner spans; `0` for square corners.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn band_count(radius: f32, ring_width: Option<f32>) -> usize {
  if radius <= 0.0 {
    return 0;
  }

  let step = ring_width.filter(|width| *width > 0.0).unwrap_or(BARE_STEP);
  let cap = MAX_BANDS.min((radius.ceil() as usize).max(1));
  (1..=cap)
    .find(|&bands| overshoot(radius, bands) <= step)
    .unwrap_or(cap)
}

/// The `2 * bands + 1` rows a copy of `frame` with corners of `radius`
/// (horizontal, vertical) is cut into, top to bottom: `bands` rows inset
/// at both ends to follow the top corners, the middle whole, then `bands`
/// following the bottom ones. A row rounding leaves empty is `None`.
///
/// `radius` is cut to half of `frame`, so the top and bottom rows never
/// overlap.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
pub(crate) fn band_rects(
  frame: &Rect,
  radius: (f32, f32),
  bands: usize,
) -> Vec<Option<Rect>> {
  let rx = radius.0.clamp(0.0, (frame.width().max(0) / 2) as f32);
  let ry = radius.1.clamp(0.0, (frame.height().max(0) / 2) as f32);
  let step = FRAC_PI_2 / bands.max(1) as f32;

  // Rows from the top edge to each cut, the last one at `ry` itself.
  let depth = |cut: usize| {
    if cut >= bands {
      ry.round() as i32
    } else {
      (ry * (1.0 - (cut as f32 * step).cos())).round() as i32
    }
  };
  let inset = |cut: usize| {
    if cut >= bands {
      0
    } else {
      (rx * (1.0 - (cut as f32 * step).sin())).floor() as i32
    }
  };
  let row = |left: i32, top: i32, right: i32, bottom: i32| {
    (right > left && bottom > top)
      .then(|| Rect::from_ltrb(left, top, right, bottom))
  };

  let top_rows = (0..bands).map(|band| {
    let inset = inset(band + 1);
    row(
      frame.left + inset,
      frame.top + depth(band),
      frame.right - inset,
      frame.top + depth(band + 1),
    )
  });
  let middle = row(
    frame.left,
    frame.top + depth(bands),
    frame.right,
    frame.bottom - depth(bands),
  );
  let bottom_rows = (0..bands).rev().map(|band| {
    let inset = inset(band + 1);
    row(
      frame.left + inset,
      frame.bottom - depth(band + 1),
      frame.right - inset,
      frame.bottom - depth(band),
    )
  });

  top_rows
    .chain(std::iter::once(middle))
    .chain(bottom_rows)
    .collect()
}

#[cfg(test)]
mod tests {
  use super::{band_count, band_rects, overshoot, MAX_BANDS};
  use crate::Rect;

  /// How far the middle of pixel `(x, y)` is outside `frame` rounded by
  /// `radius`, or a negative number inside it.
  #[allow(clippy::cast_precision_loss)]
  fn outside(frame: &Rect, radius: f32, x: i32, y: i32) -> f32 {
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    let (left, top) = (frame.left as f32, frame.top as f32);
    let (right, bottom) = (frame.right as f32, frame.bottom as f32);
    let cx = px.clamp(left + radius, right - radius);
    let cy = py.clamp(top + radius, bottom - radius);
    let (dx, dy) = (px - cx, py - cy);

    if dx == 0.0 && dy == 0.0 {
      -1.0
    } else {
      (dx * dx + dy * dy).sqrt() - radius
    }
  }

  fn covers(rows: &[Option<Rect>], x: i32, y: i32) -> usize {
    rows
      .iter()
      .flatten()
      .filter(|row| {
        x >= row.left && x < row.right && y >= row.top && y < row.bottom
      })
      .count()
  }

  #[test]
  fn square_corners_need_no_rows() {
    assert_eq!(band_count(0.0, Some(2.0)), 0);
    assert_eq!(band_count(-3.0, None), 0);
  }

  #[test]
  fn a_ring_takes_fewer_rows_than_bare_corners() {
    // 8 px corners: 3 rows reach 1.8 px past the arc, under a 2 px ring;
    // bare, 4 rows reach 1.4 px.
    assert_eq!(band_count(8.0, Some(2.0)), 3);
    assert_eq!(band_count(8.0, None), 4);
    assert_eq!(band_count(8.0, Some(0.0)), 4);
    assert!(band_count(16.0, Some(4.0)) < band_count(16.0, None));
  }

  #[test]
  fn rows_are_capped() {
    assert_eq!(band_count(40.0, None), MAX_BANDS);
    assert_eq!(band_count(2.0, Some(0.2)), 2);
    assert_eq!(band_count(0.5, None), 1);
  }

  #[test]
  fn each_count_steps_no_further_than_asked() {
    for (radius, rows) in [(4.0, 4), (8.0, 8), (12.0, 12), (20.0, 20)] {
      for ring in [None, Some(1.0), Some(2.0), Some(4.0)] {
        let bands = band_count(radius, ring);
        if bands < MAX_BANDS.min(rows) {
          assert!(overshoot(radius, bands) <= ring.unwrap_or(1.5));
        }
      }
    }
  }

  #[test]
  fn rows_cover_the_rounded_frame_once_and_follow_its_arcs() {
    let frame = Rect::from_ltrb(100, 50, 400, 250);

    for (radius, bands) in [(8.0, 3), (8.0, 4), (12.0, 8), (20.0, 5)] {
      let rows = band_rects(&frame, (radius, radius), bands);
      assert_eq!(rows.len(), 2 * bands + 1);

      // Rounding each cut moves its ends by up to a pixel.
      let reach = overshoot(radius, bands) + 1.5;
      for y in frame.top..frame.bottom {
        for x in frame.left..frame.right {
          let covered = covers(&rows, x, y);
          let distance = outside(&frame, radius, x, y);
          assert!(covered <= 1, "({x}, {y}) drawn twice");
          if distance < 0.0 {
            assert_eq!(covered, 1, "({x}, {y}) inside left out");
          } else if covered == 1 {
            assert!(distance <= reach, "({x}, {y}) {distance} past");
          }
        }
      }
    }
  }

  #[test]
  fn middle_spans_the_frame_and_rows_narrow_toward_the_corners() {
    let frame = Rect::from_ltrb(0, 0, 300, 200);
    let rows = band_rects(&frame, (12.0, 12.0), 4);

    assert_eq!(rows[4], Some(Rect::from_ltrb(0, 12, 300, 188)));
    let widths = rows
      .iter()
      .map(|row| row.as_ref().map_or(0, Rect::width))
      .collect::<Vec<_>>();
    assert!(widths[..4].windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(widths[5..].windows(2).all(|pair| pair[0] >= pair[1]));
    assert_eq!(
      widths[..4],
      widths[5..].iter().rev().copied().collect::<Vec<_>>()[..]
    );
  }

  #[test]
  fn a_radius_past_half_the_frame_is_cut_to_it() {
    let frame = Rect::from_ltrb(0, 0, 40, 11);
    let rows = band_rects(&frame, (30.0, 30.0), 3);

    for y in frame.top..frame.bottom {
      for x in frame.left..frame.right {
        assert!(covers(&rows, x, y) <= 1);
      }
    }
    assert!(rows
      .iter()
      .flatten()
      .all(|row| row.top >= 0 && row.bottom <= 11));
  }

  #[test]
  fn elliptical_corners_follow_each_axis() {
    let frame = Rect::from_ltrb(0, 0, 400, 200);
    let rows = band_rects(&frame, (16.0, 8.0), 4);

    let top = rows.iter().flatten().next().map(|row| (row.left, row.top));
    assert_eq!(top.map(|(_, top)| top), Some(0));
    assert!(top.is_some_and(|(left, _)| left > 8));
    assert_eq!(rows[4], Some(Rect::from_ltrb(0, 8, 400, 192)));
  }
}
