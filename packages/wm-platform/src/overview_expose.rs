//! Geometry of the exposé: the shown workspace's windows spread out in
//! rows under a bar of every workspace.

use std::ops::Range;

use crate::overview_layout::{CardMetrics, RectF};

/// Height of the bar of workspaces, its gap to the top of the overview,
/// and the inset of its pills, in logical pixels.
const BAR_HEIGHT: f32 = 46.0;
const BAR_TOP: f32 = 16.0;
const BAR_INSET: f32 = 5.0;

/// Gap between pills, between windows, and between rows of windows
/// (room for the title tag under one), in logical pixels.
const PILL_GAP: f32 = 4.0;
const WINDOW_GAP: f32 = 24.0;
const ROW_GAP: f32 = 52.0;

/// Most a window is scaled, so even a lone one shows as spread out.
const MAX_WINDOW_SCALE: f32 = 0.8;

/// The bar of workspaces and its pills.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bar {
  pub rect: RectF,
  pub pills: Vec<RectF>,
}

/// The bar for pills `widths` wide: centered along the top of the
/// overview, its pills shrunk alike when they don't fit.
pub(crate) fn bar(metrics: &CardMetrics, widths: &[f32]) -> Bar {
  let (inset, gap) = (metrics.px(BAR_INSET), metrics.px(PILL_GAP));
  let max_width = metrics.view.0 - 2.0 * metrics.px(48.0);

  #[allow(clippy::cast_precision_loss)]
  let gaps = gap * widths.len().saturating_sub(1) as f32 + 2.0 * inset;
  let total = widths.iter().sum::<f32>();
  let shrink = ((max_width - gaps) / total.max(1.0)).clamp(0.01, 1.0);

  let width = total * shrink + gaps;
  let rect = RectF::new(
    (metrics.view.0 - width) / 2.0,
    metrics.px(BAR_TOP),
    width,
    metrics.px(BAR_HEIGHT),
  );

  let mut x = rect.x + inset;
  let pills = widths
    .iter()
    .map(|width| {
      let pill = RectF::new(
        x,
        rect.y + inset,
        width * shrink,
        rect.h - 2.0 * inset,
      );
      x += pill.w + gap;
      pill
    })
    .collect();

  Bar { rect, pills }
}

/// Height of a pill on the bar.
pub(crate) fn pill_height(metrics: &CardMetrics) -> f32 {
  metrics.px(BAR_HEIGHT - 2.0 * BAR_INSET)
}

/// Where the windows are spread: below the bar, clear of the hints.
pub(crate) fn spread_area(metrics: &CardMetrics) -> RectF {
  let top = metrics.px(BAR_TOP + BAR_HEIGHT + 30.0);
  let bottom = metrics.bottom_hint_y() - metrics.px(20.0);
  let margin = metrics.px(48.0).max(metrics.view.0 * 0.04);

  RectF::new(
    margin,
    top,
    (metrics.view.0 - 2.0 * margin).max(0.0),
    (bottom - top).max(0.0),
  )
}

/// Windows of sizes `sizes` (as on screen, in order) spread out in `area`
/// in justified rows: the windows of a row as tall as each other and the
/// rows as wide as each other, all as large as fits. No window grows past
/// `MAX_WINDOW_SCALE` its size; one that would stays smaller, centered in
/// its row. Rows keep the windows' order and are centered.
///
/// Returns each window's rect, in the order given.
pub(crate) fn spread(
  metrics: &CardMetrics,
  sizes: &[(f32, f32)],
  area: &RectF,
) -> Vec<RectF> {
  let gaps = (metrics.px(WINDOW_GAP), metrics.px(ROW_GAP));
  let sizes = sizes
    .iter()
    .map(|(width, height)| (width.max(1.0), height.max(1.0)))
    .collect::<Vec<_>>();

  // Of the splits into rows showing the most of the windows, the one with
  // the fewest rows.
  let best = (1..=sizes.len())
    .map(|count| justify(&sizes, split_rows(&sizes, count), area, gaps))
    .reduce(|best, other| {
      if other.covered > best.covered * 1.001 {
        other
      } else {
        best
      }
    });

  best.map_or_else(Vec::new, |layout| layout.place(&sizes, area, gaps))
}

/// Windows split into rows, and how tall each window shows.
struct Justified {
  rows: Vec<Range<usize>>,
  heights: Vec<f32>,
  /// Area the windows cover.
  covered: f32,
}

impl Justified {
  /// Each window's rect: rows top to bottom, windows left to right, each
  /// centered.
  fn place(
    &self,
    sizes: &[(f32, f32)],
    area: &RectF,
    (gap, row_gap): (f32, f32),
  ) -> Vec<RectF> {
    let row_height = |row: &Range<usize>| {
      self.heights[row.clone()]
        .iter()
        .copied()
        .fold(0.0, f32::max)
    };
    let width_of = |index: usize| {
      let (width, height) = sizes[index];
      width / height * self.heights[index]
    };

    #[allow(clippy::cast_precision_loss)]
    let total_height = self.rows.iter().map(row_height).sum::<f32>()
      + row_gap * self.rows.len().saturating_sub(1) as f32;
    let mut y = area.y + (area.h - total_height) / 2.0;
    let mut rects = Vec::with_capacity(sizes.len());

    for row in &self.rows {
      let height = row_height(row);
      #[allow(clippy::cast_precision_loss)]
      let width = row.clone().map(width_of).sum::<f32>()
        + gap * row.len().saturating_sub(1) as f32;
      let mut x = area.x + (area.w - width) / 2.0;

      for index in row.clone() {
        let (window_width, window_height) =
          (width_of(index), self.heights[index]);
        rects.push(RectF::new(
          x,
          y + (height - window_height) / 2.0,
          window_width,
          window_height,
        ));
        x += window_width + gap;
      }

      y += height + row_gap;
    }

    rects
  }
}

/// `rows` of windows `sizes` justified in `area`: each row as tall as
/// fills the area's width, all shrunk alike to fit its height, then each
/// window kept to `MAX_WINDOW_SCALE` its size.
fn justify(
  sizes: &[(f32, f32)],
  rows: Vec<Range<usize>>,
  area: &RectF,
  (gap, row_gap): (f32, f32),
) -> Justified {
  let aspect = |index: usize| sizes[index].0 / sizes[index].1;

  #[allow(clippy::cast_precision_loss)]
  let fills = rows
    .iter()
    .map(|row| {
      let room = area.w - gap * row.len().saturating_sub(1) as f32;
      room / row.clone().map(aspect).sum::<f32>()
    })
    .collect::<Vec<_>>();

  #[allow(clippy::cast_precision_loss)]
  let room = area.h - row_gap * rows.len().saturating_sub(1) as f32;
  let shrink = (room / fills.iter().sum::<f32>().max(1.0)).clamp(0.0, 1.0);

  let mut heights = vec![0.0; sizes.len()];
  for (row, fill) in rows.iter().zip(&fills) {
    for index in row.clone() {
      heights[index] = (fill * shrink)
        .min(sizes[index].1 * MAX_WINDOW_SCALE)
        .max(0.0);
    }
  }

  let covered = (0..sizes.len())
    .map(|index| aspect(index) * heights[index] * heights[index])
    .sum();

  Justified {
    rows,
    heights,
    covered,
  }
}

/// `sizes` split in order into up to `count` rows of about the same
/// width at the same height.
fn split_rows(sizes: &[(f32, f32)], count: usize) -> Vec<Range<usize>> {
  let aspects = sizes
    .iter()
    .map(|(width, height)| width / height)
    .collect::<Vec<_>>();
  #[allow(clippy::cast_precision_loss)]
  let target = aspects.iter().sum::<f32>() / count.max(1) as f32;

  let mut rows = Vec::with_capacity(count);
  let (mut start, mut width) = (0, 0.0);

  for (index, aspect) in aspects.iter().enumerate() {
    let rows_left = count - rows.len();

    // A row ends before the window whose middle would pass the target, or
    // as late as still leaves a window for every row after it.
    let is_full = width + aspect / 2.0 > target && rows_left > 1;
    let must_end = sizes.len() - index < rows_left;

    if index > start && (is_full || must_end) {
      rows.push(start..index);
      (start, width) = (index, 0.0);
    }
    width += aspect;
  }

  rows.push(start..sizes.len());
  rows
}

#[cfg(test)]
mod tests {
  use super::{bar, spread, spread_area};
  use crate::{
    overview_layout::{CardMetrics, RectF},
    OverviewLayoutMode,
  };

  fn metrics() -> CardMetrics {
    CardMetrics::new(1920.0, 1040.0, 1.0, OverviewLayoutMode::Expose)
  }

  fn assert_inside(rect: &RectF, area: &RectF) {
    assert!(
      rect.x >= area.x - 0.01
        && rect.y >= area.y - 0.01
        && rect.right() <= area.right() + 0.01
        && rect.bottom() <= area.bottom() + 0.01,
      "{rect:?} not inside {area:?}"
    );
  }

  #[test]
  fn a_lone_window_is_centered_and_still_spread_out() {
    let metrics = metrics();
    let area = spread_area(&metrics);
    let rects = spread(&metrics, &[(1920.0, 1040.0)], &area);

    let (cx, cy) = rects[0].center();
    let (area_cx, area_cy) = area.center();
    assert!((cx - area_cx).abs() < 0.01 && (cy - area_cy).abs() < 0.01);
    assert!(rects[0].w <= 1920.0 * 0.8 + 0.01);
    assert_inside(&rects[0], &area);
  }

  #[test]
  fn windows_keep_their_order_and_never_overlap() {
    let metrics = metrics();
    let area = spread_area(&metrics);
    let sizes = [
      (960.0, 1040.0),
      (480.0, 520.0),
      (480.0, 520.0),
      (1920.0, 1040.0),
      (640.0, 1040.0),
      (1280.0, 700.0),
      (300.0, 900.0),
    ];
    let rects = spread(&metrics, &sizes, &area);
    assert_eq!(rects.len(), sizes.len());

    for (index, rect) in rects.iter().enumerate() {
      assert_inside(rect, &area);

      // Every later window is right of it on its row, or on a row below.
      for later in &rects[index + 1..] {
        let is_right = later.x >= rect.right() - 0.01
          && later.y < rect.bottom()
          && later.bottom() > rect.y;
        let is_below = later.y >= rect.bottom() - 0.01;
        assert!(is_right || is_below, "{later:?} before {rect:?}");
        assert!(later.intersect(rect).is_none());
      }
    }
  }

  #[test]
  fn a_row_is_one_height_and_fills_the_width() {
    let metrics = metrics();
    let area = spread_area(&metrics);
    let rects =
      spread(&metrics, &[(800.0, 1000.0), (1200.0, 1000.0)], &area);

    // As tall as each other, as wide as the area, side by side.
    assert!((rects[0].h - rects[1].h).abs() < 0.01);
    assert!((rects[0].w / rects[0].h - 0.8).abs() < 0.001);
    let width = rects[1].right() - rects[0].x;
    assert!((width - area.w).abs() < 0.5 || rects[0].h >= 800.0 - 0.01);
  }

  #[test]
  fn a_small_window_stays_small_beside_large_ones() {
    let metrics = metrics();
    let area = spread_area(&metrics);
    let rects = spread(
      &metrics,
      &[(900.0, 1000.0), (300.0, 200.0), (900.0, 1000.0)],
      &area,
    );

    assert!(rects[1].h <= 200.0 * 0.8 + 0.01, "never past its size");
    assert!(rects[0].h > rects[1].h);
    let (_, middle) = rects[0].center();
    assert!((rects[1].center().1 - middle).abs() < 0.01, "centered");
  }

  #[test]
  fn many_windows_take_more_rows() {
    let metrics = metrics();
    let area = spread_area(&metrics);

    let two = spread(&metrics, &[(960.0, 1040.0); 2], &area);
    assert!((two[0].y - two[1].y).abs() < 0.01, "two side by side");

    let nine = spread(&metrics, &[(960.0, 1040.0); 9], &area);
    #[allow(clippy::cast_possible_truncation)]
    let rows = nine
      .iter()
      .map(|rect| rect.y.round() as i32)
      .collect::<std::collections::BTreeSet<_>>();
    assert!(rows.len() > 1);
    for rect in &nine {
      assert_inside(rect, &area);
    }
  }

  #[test]
  fn nothing_to_spread_gives_nothing() {
    let metrics = metrics();
    assert!(spread(&metrics, &[], &spread_area(&metrics)).is_empty());
  }

  #[test]
  fn the_bar_centers_its_pills_and_shrinks_them_to_fit() {
    let metrics = metrics();

    let small = bar(&metrics, &[100.0, 140.0, 80.0]);
    assert!((small.rect.center().0 - 960.0).abs() < 0.01);
    for pair in small.pills.windows(2) {
      assert!(pair[1].x > pair[0].right());
    }
    assert!((small.pills[1].w - 140.0).abs() < 0.01);
    assert_inside(&small.pills[0], &small.rect);
    assert_inside(&small.pills[2], &small.rect);

    let wide = bar(&metrics, &[400.0; 10]);
    assert!(wide.rect.x >= 47.99 && wide.rect.right() <= 1920.0 - 47.99);
    assert!(wide.pills[0].w < 400.0);
  }
}
