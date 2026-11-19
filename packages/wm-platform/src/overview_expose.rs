//! Where the exposé spreads a workspace's windows: side by side in rows,
//! all at one scale, in about the order they sit on screen.

use crate::overview_layout::RectF;

/// Spreads windows whose screen rects are `windows` over `area`, `gap`
/// apart, at one scale of at most `max_scale`, as large as fits.
///
/// Rows go top to bottom by where each window's middle sits on screen,
/// and each row left to right. Returns a rect per window, in the order
/// given.
pub(crate) fn spread(
  windows: &[RectF],
  area: &RectF,
  gap: f32,
  max_scale: f32,
) -> Vec<RectF> {
  if windows.is_empty() {
    return Vec::new();
  }

  let mut order = (0..windows.len()).collect::<Vec<_>>();
  order.sort_by(|&a, &b| {
    let ((ax, ay), (bx, by)) = (windows[a].center(), windows[b].center());
    ay.total_cmp(&by).then(ax.total_cmp(&bx)).then(a.cmp(&b))
  });

  // The row count that shows them largest, the fewest rows on a tie.
  let mut best: Option<(f32, Vec<Vec<usize>>)> = None;
  for row_count in 1..=windows.len() {
    let rows = partition(windows, &order, row_count, area.w, gap);
    let scale = rows_scale(windows, &rows, area, gap);
    if best.as_ref().is_none_or(|(best, _)| scale > *best) {
      best = Some((scale, rows));
    }
  }
  let Some((scale, mut rows)) = best else {
    return Vec::new();
  };
  let scale = scale.min(max_scale).max(0.0);

  let row_heights = rows
    .iter()
    .map(|row| row_height(windows, row) * scale)
    .collect::<Vec<_>>();
  #[allow(clippy::cast_precision_loss)]
  let total_height = row_heights.iter().sum::<f32>()
    + gap * (rows.len().saturating_sub(1)) as f32;

  let mut rects = vec![RectF::default(); windows.len()];
  let mut y = area.y + (area.h - total_height) / 2.0;
  for (row, height) in rows.iter_mut().zip(row_heights) {
    row.sort_by(|&a, &b| {
      windows[a].center().0.total_cmp(&windows[b].center().0)
    });

    #[allow(clippy::cast_precision_loss)]
    let width = row_width(windows, row) * scale
      + gap * (row.len().saturating_sub(1)) as f32;
    let mut x = area.x + (area.w - width) / 2.0;
    for &index in row.iter() {
      let (w, h) = (windows[index].w * scale, windows[index].h * scale);
      rects[index] = RectF::new(x, y + (height - h) / 2.0, w, h);
      x += w + gap;
    }
    y += height + gap;
  }

  rects
}

/// Splits `order` into `row_count` runs, keeping it in order, so the
/// narrowest fit of any row is as wide as can be.
fn partition(
  windows: &[RectF],
  order: &[usize],
  row_count: usize,
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
      for start in (rows - 1)..end {
        let before = fits[rows - 1][start].0;
        let row = &order[start..end];
        let scale = before.min(row_scale(windows, row, width, gap));
        if scale > fits[rows][end].0 {
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

/// Largest scale at which `rows` fit `area`, `gap` apart.
fn rows_scale(
  windows: &[RectF],
  rows: &[Vec<usize>],
  area: &RectF,
  gap: f32,
) -> f32 {
  let across = rows
    .iter()
    .map(|row| row_scale(windows, row, area.w, gap))
    .fold(f32::INFINITY, f32::min);

  #[allow(clippy::cast_precision_loss)]
  let free_height = area.h - gap * (rows.len().saturating_sub(1)) as f32;
  let heights =
    rows.iter().map(|row| row_height(windows, row)).sum::<f32>();
  let down = if heights > 0.0 {
    free_height / heights
  } else {
    0.0
  };

  across.min(down)
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

fn row_width(windows: &[RectF], row: &[usize]) -> f32 {
  row.iter().map(|&index| windows[index].w).sum()
}

fn row_height(windows: &[RectF], row: &[usize]) -> f32 {
  row
    .iter()
    .map(|&index| windows[index].h)
    .fold(0.0, f32::max)
}

#[cfg(test)]
mod tests {
  use super::spread;
  use crate::overview_layout::RectF;

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
    let rects = spread(&[window], &AREA, 20.0, 0.5);

    assert_eq!(rects, vec![RectF::new(320.0, 180.0, 960.0, 540.0)]);
  }

  /// Windows side by side on screen stay in one row, left to right.
  #[test]
  fn side_by_side_windows_keep_their_order() {
    let right = RectF::new(960.0, 0.0, 960.0, 1080.0);
    let left = RectF::new(0.0, 0.0, 960.0, 1080.0);
    let rects = spread(&[right, left], &AREA, 20.0, 1.0);

    assert!(rects[1].right() < rects[0].x);
    assert!((rects[0].y - rects[1].y).abs() < f32::EPSILON);
    assert!((rects[0].w - rects[1].w).abs() < f32::EPSILON);
  }

  /// Tabs of a stack share their stack's rect, and still each get a spot
  /// of their own.
  #[test]
  fn windows_on_one_spot_are_spread_apart() {
    let stack = RectF::new(0.0, 0.0, 1920.0, 1080.0);
    let rects = spread(&[stack; 5], &AREA, 24.0, 1.0);

    for (index, rect) in rects.iter().enumerate() {
      assert!(within(rect, &AREA), "{rect:?}");
      for other in &rects[index + 1..] {
        assert!(!overlaps(rect, other), "{rect:?} {other:?}");
      }
    }
  }

  /// Many windows go on several rows, all at one scale, within the area
  /// and apart from each other.
  #[test]
  fn many_windows_wrap_onto_rows_at_one_scale() {
    let windows = (0..9)
      .map(|index| {
        #[allow(clippy::cast_precision_loss)]
        let x = (index % 3) as f32 * 640.0;
        #[allow(clippy::cast_precision_loss)]
        let y = (index / 3) as f32 * 360.0;
        RectF::new(x, y, 640.0, 360.0)
      })
      .collect::<Vec<_>>();
    let rects = spread(&windows, &AREA, 16.0, 1.0);

    let scale = rects[0].w / 640.0;
    let mut rows =
      rects.iter().map(|rect| rect.y.round()).collect::<Vec<_>>();
    rows.dedup();
    assert!(rows.len() > 1);

    for (index, rect) in rects.iter().enumerate() {
      assert!((rect.w / 640.0 - scale).abs() < 1e-4);
      assert!(within(rect, &AREA), "{rect:?}");
      for other in &rects[index + 1..] {
        assert!(!overlaps(rect, other), "{rect:?} {other:?}");
      }
    }
  }

  #[test]
  fn no_windows_spread_to_nothing() {
    assert!(spread(&[], &AREA, 16.0, 1.0).is_empty());
  }
}
