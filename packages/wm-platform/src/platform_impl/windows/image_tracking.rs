//! Follows kept pictures as the window's content moves, between the rare
//! UI Automation queries that find them.
//!
//! Scrolling moves a picture without UI Automation saying so, and the
//! rect it was kept at then covers whatever moved in (often text, which
//! would keep its light-mode colors). Each frame, a few pixels sampled
//! from the picture when it was found are looked for again, shifted
//! vertically: found, the rect moves with the picture; not found, the
//! picture changed or scrolled away, and is no longer kept.

use crate::Rect;

/// Most samples per side of a picture's fingerprint.
const FINGERPRINT_SIDE: i32 = 12;

/// Mean sRGB difference of the fingerprint's samples up to which a
/// picture counts as found at a shift.
const MATCH_MAX_DIFFERENCE: f32 = 0.03;

/// Mean difference at no shift below which the search is skipped: the
/// picture hasn't moved (the common case, e.g. a clock repainting).
const UNCHANGED_MAX_DIFFERENCE: f32 = 0.005;

/// Straight-alpha pixels of a frame region, row-major; transparent ones
/// are `None`.
pub(crate) struct Region<'a> {
  pub rect: &'a Rect,
  pub pixels: &'a [Option<[f32; 3]>],
}

impl Region<'_> {
  fn at(&self, x: i32, y: i32) -> Option<[f32; 3]> {
    if x < self.rect.left
      || y < self.rect.top
      || x >= self.rect.right
      || y >= self.rect.bottom
    {
      return None;
    }

    let index = usize::try_from(
      (y - self.rect.top) * self.rect.width() + (x - self.rect.left),
    )
    .ok()?;
    self.pixels.get(index).copied().flatten()
  }
}

/// Pixels sampled on a grid over a picture, to find it again.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Fingerprint {
  /// Offsets from the picture's top-left corner, and the color there.
  samples: Vec<(i32, i32, [f32; 3])>,
}

impl Fingerprint {
  /// Samples `picture` from `region`, which must contain it.
  pub(crate) fn sample(picture: &Rect, region: &Region) -> Self {
    let columns = FINGERPRINT_SIDE.min(picture.width()).max(1);
    let rows = FINGERPRINT_SIDE.min(picture.height()).max(1);
    let mut samples = Vec::new();

    for row in 0..rows {
      for column in 0..columns {
        // Cell centers, so no sample sits on the picture's edge.
        let x = (2 * column + 1) * picture.width() / (2 * columns);
        let y = (2 * row + 1) * picture.height() / (2 * rows);

        if let Some(color) = region.at(picture.left + x, picture.top + y) {
          samples.push((x, y, color));
        }
      }
    }

    Self { samples }
  }

  /// The vertical shift, within `max_shift`, at which the picture last
  /// seen at `picture` is found in `region`; `None` if it isn't.
  ///
  /// Only what's shown in `view` (the area it scrolls in) counts, and
  /// most of the picture must still be there. Shifts are tried nearest
  /// first, so a picture that looks the same at several shifts (flat, or
  /// repeating) keeps the smallest.
  pub(crate) fn find_shift(
    &self,
    picture: &Rect,
    view: &Rect,
    region: &Region,
    max_shift: i32,
  ) -> Option<i32> {
    if self.samples.is_empty() {
      return None;
    }

    if self
      .difference_at(picture, view, region, 0)
      .is_some_and(|difference| difference <= UNCHANGED_MAX_DIFFERENCE)
    {
      return Some(0);
    }

    let mut best: Option<(i32, f32)> = None;

    for distance in 0..=max_shift {
      for shift in [distance, -distance] {
        if distance == 0 && shift < 0 {
          continue;
        }

        let Some(difference) =
          self.difference_at(picture, view, region, shift)
        else {
          continue;
        };

        if best.is_none_or(|(_, least)| difference < least) {
          best = Some((shift, difference));
        }
      }
    }

    best
      .filter(|&(_, difference)| difference <= MATCH_MAX_DIFFERENCE)
      .map(|(shift, _)| shift)
  }

  /// Mean difference of the samples with the picture moved down by
  /// `shift`, or `None` if too few of them land in `view` and `region`.
  fn difference_at(
    &self,
    picture: &Rect,
    view: &Rect,
    region: &Region,
    shift: i32,
  ) -> Option<f32> {
    let mut sum = 0.0;
    let mut count = 0_usize;

    for &(x, y, color) in &self.samples {
      let (x, y) = (picture.left + x, picture.top + y + shift);

      if x < view.left
        || y < view.top
        || x >= view.right
        || y >= view.bottom
      {
        continue;
      }

      if let Some(found) = region.at(x, y) {
        sum += (0..3)
          .map(|c| (found[c] - color[c]).abs())
          .fold(0.0_f32, f32::max);
        count += 1;
      }
    }

    // Most of the picture must still be in view to be sure it's there.
    #[allow(clippy::cast_precision_loss)]
    (count * 2 >= self.samples.len()).then(|| sum / count as f32)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A `width` x `height` region at the origin whose color at `(x, y)` is
  /// `color(x, y)`.
  fn region_pixels(
    width: i32,
    height: i32,
    color: impl Fn(i32, i32) -> [f32; 3],
  ) -> (Rect, Vec<Option<[f32; 3]>>) {
    let rect = Rect::from_ltrb(0, 0, width, height);
    let pixels = (0..height)
      .flat_map(|y| (0..width).map(move |x| (x, y)))
      .map(|(x, y)| Some(color(x, y)))
      .collect();
    (rect, pixels)
  }

  /// A varied "picture" filling rows `top..top + 40` of a white page.
  fn page_with_picture(top: i32) -> (Rect, Vec<Option<[f32; 3]>>) {
    region_pixels(60, 200, move |x, y| {
      if (top..top + 40).contains(&y) && (10..50).contains(&x) {
        #[allow(clippy::cast_precision_loss)]
        let t = ((x * 7 + (y - top) * 13) % 40) as f32 / 40.0;
        [t, 1.0 - t, (t * 3.0) % 1.0]
      } else {
        [1.0; 3]
      }
    })
  }

  #[test]
  fn finds_a_picture_that_scrolled() {
    let picture = Rect::from_ltrb(10, 60, 50, 100);
    let (rect, before) = page_with_picture(60);
    let fingerprint = Fingerprint::sample(
      &picture,
      &Region {
        rect: &rect,
        pixels: &before,
      },
    );

    // Scrolled up by 25 px.
    let (rect, after) = page_with_picture(35);
    let region = Region {
      rect: &rect,
      pixels: &after,
    };

    assert_eq!(
      fingerprint.find_shift(&picture, &rect, &region, 64),
      Some(-25)
    );
  }

  #[test]
  fn loses_a_picture_that_is_gone() {
    let picture = Rect::from_ltrb(10, 60, 50, 100);
    let (rect, before) = page_with_picture(60);
    let fingerprint = Fingerprint::sample(
      &picture,
      &Region {
        rect: &rect,
        pixels: &before,
      },
    );

    // Only page and text-like stripes where it was.
    let (rect, after) =
      region_pixels(
        60,
        200,
        |_, y| {
          if y % 4 == 0 {
            [0.1; 3]
          } else {
            [1.0; 3]
          }
        },
      );
    let region = Region {
      rect: &rect,
      pixels: &after,
    };

    assert_eq!(fingerprint.find_shift(&picture, &rect, &region, 64), None);
  }

  #[test]
  fn loses_a_picture_scrolled_out_of_its_view() {
    let picture = Rect::from_ltrb(10, 60, 50, 100);
    let (rect, before) = page_with_picture(60);
    let fingerprint = Fingerprint::sample(
      &picture,
      &Region {
        rect: &rect,
        pixels: &before,
      },
    );

    // Scrolled up by 35 px in a list whose view starts at y = 50: only
    // its bottom quarter still shows.
    let (rect, after) = page_with_picture(25);
    let view = Rect::from_ltrb(0, 50, 60, 200);
    let region = Region {
      rect: &rect,
      pixels: &after,
    };

    assert_eq!(fingerprint.find_shift(&picture, &view, &region, 64), None);
  }

  #[test]
  fn keeps_an_unmoved_picture_in_place() {
    let picture = Rect::from_ltrb(10, 60, 50, 100);
    let (rect, pixels) = page_with_picture(60);
    let region = Region {
      rect: &rect,
      pixels: &pixels,
    };
    let fingerprint = Fingerprint::sample(&picture, &region);

    assert_eq!(
      fingerprint.find_shift(&picture, &rect, &region, 64),
      Some(0)
    );
  }
}
