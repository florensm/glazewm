//! Where the exposé spreads a workspace's windows: in rows filling the
//! room's width, in about the order they sit on screen.

use crate::{overview::layout::RectF, OverviewExposeArrangement};

/// Spreads windows whose screen rects are `windows` over `area`, `gap`
/// apart and at most `max_per_row` to a row (`0` for any number), as
/// large as fits.
///
/// Each row is scaled to fill `area`'s width, unless that would show a
/// window larger than `max_scale` of its size or the rows wouldn't fit
/// `area`'s height. In it, windows keep their sizes relative to each
/// other, centered on the row (`Natural`), or are all as tall (`Grid`).
/// Of the row counts, the one covering the most of `area` wins. Rows go
/// top to bottom by where each window's middle sits on screen, and each
/// row left to right. Returns a rect per window, in the order given.
pub(crate) fn spread(
  windows: &[RectF],
  area: &RectF,
  gap: f32,
  max_scale: f32,
  max_per_row: usize,
  arrangement: OverviewExposeArrangement,
) -> Vec<RectF> {
  if windows.is_empty() {
    return Vec::new();
  }
  let max_per_row = match max_per_row {
    0 => windows.len(),
    max => max,
  };

  // What a row scales: each window as it is, or one tall at its shape.
  let shapes = windows
    .iter()
    .map(|window| match arrangement {
      OverviewExposeArrangement::Natural => {
        RectF::new(0.0, 0.0, window.w.max(1.0), window.h.max(1.0))
      }
      OverviewExposeArrangement::Grid => {
        let aspect = if window.h > 0.0 {
          window.w / window.h
        } else {
          1.0
        };
        RectF::new(0.0, 0.0, aspect, 1.0)
      }
    })
    .collect::<Vec<_>>();

  let mut order = (0..windows.len()).collect::<Vec<_>>();
  order.sort_by(|&a, &b| {
    let ((ax, ay), (bx, by)) = (windows[a].center(), windows[b].center());
    ay.total_cmp(&by).then(ax.total_cmp(&bx)).then(a.cmp(&b))
  });

  // The row count covering the most, the fewest rows on a tie.
  let mut best: Option<(f32, Vec<Vec<usize>>, Vec<f32>)> = None;
  for row_count in windows.len().div_ceil(max_per_row)..=windows.len() {
    let rows =
      partition(&shapes, &order, row_count, max_per_row, area.w, gap);
    let scales = row_scales(windows, &shapes, &rows, area, gap, max_scale);
    let coverage = rows
      .iter()
      .zip(&scales)
      .map(|(row, scale)| {
        let shown = row
          .iter()
          .map(|&index| shapes[index].w * shapes[index].h)
          .sum::<f32>();
        shown * scale * scale
      })
      .sum::<f32>();
    if best.as_ref().is_none_or(|(best, ..)| coverage > *best) {
      best = Some((coverage, rows, scales));
    }
  }
  let Some((_, mut rows, scales)) = best else {
    return Vec::new();
  };

  let heights = rows
    .iter()
    .zip(&scales)
    .map(|(row, scale)| row_height(&shapes, row) * scale)
    .collect::<Vec<_>>();
  #[allow(clippy::cast_precision_loss)]
  let total_height = heights.iter().sum::<f32>()
    + gap * (rows.len().saturating_sub(1)) as f32;

  let mut rects = vec![RectF::default(); windows.len()];
  let mut y = area.y + (area.h - total_height) / 2.0;
  for ((row, scale), height) in rows.iter_mut().zip(scales).zip(heights) {
    row.sort_by(|&a, &b| {
      windows[a].center().0.total_cmp(&windows[b].center().0)
    });

    #[allow(clippy::cast_precision_loss)]
    let width = row_width(&shapes, row) * scale
      + gap * (row.len().saturating_sub(1)) as f32;
    let mut x = area.x + (area.w - width) / 2.0;
    for &index in row.iter() {
      let (w, h) = (shapes[index].w * scale, shapes[index].h * scale);
      rects[index] = RectF::new(x, y + (height - h) / 2.0, w, h);
      x += w + gap;
    }
    y += height + gap;
  }

  rects
}

/// The scale of each of `rows`' `shapes`: filling `area`'s width, but
/// showing none of `windows` larger than `max_scale` of its size, and all
/// shrunk alike to fit `area`'s height.
fn row_scales(
  windows: &[RectF],
  shapes: &[RectF],
  rows: &[Vec<usize>],
  area: &RectF,
  gap: f32,
  max_scale: f32,
) -> Vec<f32> {
  let mut scales = rows
    .iter()
    .map(|row| {
      let fills = row_scale(shapes, row, area.w, gap);
      let largest = row
        .iter()
        .map(|&index| windows[index].h * max_scale / shapes[index].h)
        .fold(f32::INFINITY, f32::min);
      fills.min(largest).max(0.0)
    })
    .collect::<Vec<_>>();

  #[allow(clippy::cast_precision_loss)]
  let free =
    (area.h - gap * (rows.len().saturating_sub(1)) as f32).max(0.0);
  let total = rows
    .iter()
    .zip(&scales)
    .map(|(row, scale)| row_height(shapes, row) * scale)
    .sum::<f32>();
  if total > free && total > 0.0 {
    for scale in &mut scales {
      *scale *= free / total;
    }
  }
  scales
}

/// Splits `order` into `row_count` runs of at most `max_per_row`,
/// keeping it in order, so the row whose windows of `windows` fit
/// `width` smallest fits them as large as can be. `row_count` must leave
/// room for every window.
fn partition(
  windows: &[RectF],
  order: &[usize],
  row_count: usize,
  max_per_row: usize,
  width: f32,
  gap: f32,
) -> Vec<Vec<usize>> {
  let count = order.len();
  // `fits[rows][end]`: best scale splitting `order[..end]` into `rows`,
  // and where its last row starts.
  let mut fits =
    vec![vec![(f32::NEG_INFINITY, 0); count + 1]; row_count + 1];
  fits[0][0] = (f32::INFINITY, 0);

  for rows in 1..=row_count {
    for end in rows..=count {
      for start in (rows - 1).max(end.saturating_sub(max_per_row))..end {
        let before = fits[rows - 1][start].0;
        let row = &order[start..end];
        let scale = before.min(row_scale(windows, row, width, gap));
        // On a tie, the later start: fuller rows on top.
        if scale >= fits[rows][end].0 {
          fits[rows][end] = (scale, start);
        }
      }
    }
  }

  let mut rows = Vec::with_capacity(row_count);
  let mut end = count;
  for remaining in (1..=row_count).rev() {
    let start = fits[remaining][end].1;
    rows.push(order[start..end].to_vec());
    end = start;
  }
  rows.reverse();
  rows
}

/// Largest scale at which `row` fits `width`, `gap` apart.
fn row_scale(
  windows: &[RectF],
  row: &[usize],
  width: f32,
  gap: f32,
) -> f32 {
  #[allow(clippy::cast_precision_loss)]
  let free = width - gap * (row.len().saturating_sub(1)) as f32;
  let total = row_width(windows, row);
  if total > 0.0 {
    free / total
  } else {
    0.0
  }
}

fn row_height(windows: &[RectF], row: &[usize]) -> f32 {
  row
    .iter()
    .map(|&index| windows[index].h)
    .fold(0.0, f32::max)
}

fn row_width(windows: &[RectF], row: &[usize]) -> f32 {
  row.iter().map(|&index| windows[index].w).sum()
}

#[cfg(test)]
mod tests {
  use super::spread;
  use crate::{overview::layout::RectF, OverviewExposeArrangement};

  const GRID: OverviewExposeArrangement = OverviewExposeArrangement::Grid;
  const NATURAL: OverviewExposeArrangement =
    OverviewExposeArrangement::Natural;

  const AREA: RectF = RectF::new(0.0, 0.0, 1600.0, 900.0);

  fn overlaps(a: &RectF, b: &RectF) -> bool {
    a.x < b.right()
      && b.x < a.right()
      && a.y < b.bottom()
      && b.y < a.bottom()
  }

  fn within(rect: &RectF, area: &RectF) -> bool {
    let slack = 0.01;
    rect.x >= area.x - slack
      && rect.y >= area.y - slack
      && rect.right() <= area.right() + slack
      && rect.bottom() <= area.bottom() + slack
  }

  /// A lone window is centered, kept to its shape, and no larger than
  /// `max_scale` allows.
  #[test]
  fn a_lone_window_is_centered_at_most_at_max_scale() {
    let window = RectF::new(0.0, 0.0, 1920.0, 1080.0);
    let rects = spread(&[window], &AREA, 20.0, 0.5, 4, GRID);

    assert_eq!(rects, vec![RectF::new(320.0, 180.0, 960.0, 540.0)]);
  }

  /// Windows side by side on screen stay in one row, left to right.
  #[test]
  fn side_by_side_windows_keep_their_order() {
    let right = RectF::new(960.0, 0.0, 960.0, 1080.0);
    let left = RectF::new(0.0, 0.0, 960.0, 1080.0);
    let rects = spread(&[right, left], &AREA, 20.0, 1.0, 4, GRID);

    assert!(rects[1].right() < rects[0].x);
    assert!((rects[0].y - rects[1].y).abs() < f32::EPSILON);
    assert!((rects[0].w - rects[1].w).abs() < f32::EPSILON);
  }

  /// Tabs of a stack share their stack's rect, and still each get a spot
  /// of their own.
  #[test]
  fn windows_on_one_spot_are_spread_apart() {
    let stack = RectF::new(0.0, 0.0, 1920.0, 1080.0);
    let rects = spread(&[stack; 5], &AREA, 24.0, 1.0, 4, GRID);

    for (index, rect) in rects.iter().enumerate() {
      assert!(within(rect, &AREA), "{rect:?}");
      for other in &rects[index + 1..] {
        assert!(!overlaps(rect, other), "{rect:?} {other:?}");
      }
    }
  }

  /// Many windows go on several rows, keeping their shape, within the
  /// area and apart from each other.
  #[test]
  fn many_windows_wrap_onto_rows() {
    let windows = (0..9)
      .map(|index| {
        #[allow(clippy::cast_precision_loss)]
        let x = (index % 3) as f32 * 640.0;
        #[allow(clippy::cast_precision_loss)]
        let y = (index / 3) as f32 * 360.0;
        RectF::new(x, y, 640.0, 360.0)
      })
      .collect::<Vec<_>>();
    let rects = spread(&windows, &AREA, 16.0, 1.0, 4, GRID);

    let mut rows =
      rects.iter().map(|rect| rect.y.round()).collect::<Vec<_>>();
    rows.dedup();
    assert!(rows.len() > 1);

    for (index, rect) in rects.iter().enumerate() {
      assert!((rect.w / rect.h - 640.0 / 360.0).abs() < 1e-3);
      assert!(within(rect, &AREA), "{rect:?}");
      for other in &rects[index + 1..] {
        assert!(!overlaps(rect, other), "{rect:?} {other:?}");
      }
    }
  }

  /// Windows of different shapes in a row share its height, and the row
  /// fills the area's width.
  #[test]
  fn a_row_is_justified_to_the_width() {
    let windows = [
      RectF::new(0.0, 0.0, 600.0, 1000.0),
      RectF::new(600.0, 0.0, 1300.0, 1000.0),
    ];
    let rects = spread(&windows, &AREA, 20.0, 1.0, 0, GRID);

    assert!((rects[0].h - rects[1].h).abs() < 1e-3);
    assert!((rects[0].x - AREA.x).abs() < 1e-3);
    assert!((rects[1].right() - AREA.right()).abs() < 1e-3);
    assert!((rects[0].w / rects[0].h - 0.6).abs() < 1e-3);
  }

  /// Naturally, windows of a row keep their sizes relative to each
  /// other, centered on it.
  #[test]
  fn natural_rows_keep_relative_sizes() {
    let wide = RectF::new(0.0, 0.0, 4000.0, 500.0);
    let windows = [
      RectF::new(0.0, 0.0, 800.0, 400.0),
      RectF::new(800.0, 100.0, 800.0, 200.0),
    ];
    let rects = spread(&windows, &wide, 20.0, 1.0, 0, NATURAL);

    assert!((rects[0].h / rects[1].h - 2.0).abs() < 1e-3);
    assert!((rects[0].w - rects[1].w).abs() < 1e-3);
    let middle = |rect: &RectF| rect.y + rect.h / 2.0;
    assert!((middle(&rects[0]) - middle(&rects[1])).abs() < 1e-3);
    assert!(rects[1].x > rects[0].right());
  }

  #[test]
  fn no_windows_spread_to_nothing() {
    assert!(spread(&[], &AREA, 16.0, 1.0, 0, GRID).is_empty());
  }

  /// A scrolling strip's tall columns, which fit one row best, still wrap
  /// once a row is full, its left part going on top.
  #[test]
  fn a_full_row_wraps_onto_the_next() {
    let columns = (0..7)
      .map(|index| {
        #[allow(clippy::cast_precision_loss)]
        let x = index as f32 * 500.0;
        RectF::new(x, 0.0, 480.0, 1040.0)
      })
      .collect::<Vec<_>>();
    let rects = spread(&columns, &AREA, 16.0, 1.0, 4, GRID);

    let top = rects[0].y;
    let on_top = rects.iter().filter(|rect| (rect.y - top).abs() < 0.5);
    assert_eq!(on_top.count(), 4);
    assert!(rects[4].y > rects[3].bottom());
    assert!(rects[0].x < rects[1].x && rects[4].x < rects[5].x);
    for rect in &rects {
      assert!(within(rect, &AREA), "{rect:?}");
    }
  }
}
