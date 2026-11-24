//! GNOME Shell's rows for the exposé: `UnalignedLayoutStrategy` in
//! `js/ui/workspace.js` (GPL-2.0-or-later), ported function by function.
//! Rows are filled top to bottom by where windows sit on screen, each
//! window shorter than its monitor grown up to 1.5× first, and the row
//! count chosen for scale over room filled.

use crate::overview::layout::RectF;

/// Largest scale a window shows at: `WINDOW_PREVIEW_MAXIMUM_SCALE`.
pub(crate) const MOST_SCALE: f32 = 0.95;

/// Most a window shorter than its monitor grows.
const SMALL_WINDOW_BUMP: f32 = 1.5;

/// How much more a row count's scale counts than the room it fills.
const LAYOUT_SCALE_WEIGHT: f32 = 1.0;
const LAYOUT_SPACE_WEIGHT: f32 = 0.1;

/// A row: indices into the windows, left to right, and its size at scale
/// 1 with each window bumped.
#[derive(Default)]
struct Row {
  windows: Vec<usize>,
  full_width: f32,
  full_height: f32,
}

/// Where each of `windows` (screen rects) shows over `area`, `spacing`
/// apart, on a monitor `monitor_height` tall. In the order given.
pub(crate) fn rows(
  windows: &[RectF],
  area: &RectF,
  spacing: f32,
  monitor_height: f32,
) -> Vec<RectF> {
  // `_computeWindowScale`.
  let bump = windows
    .iter()
    .map(|window| {
      let ratio = (window.h / monitor_height.max(1.0)).min(1.0);
      SMALL_WINDOW_BUMP + (1.0 - SMALL_WINDOW_BUMP) * ratio
    })
    .collect::<Vec<_>>();

  // `_createBestLayout`: one more row while that scores better.
  let mut best: Option<(Vec<Row>, (f32, f32))> = None;
  let mut last_columns = None;
  for count in 1..=windows.len() {
    let columns = windows.len().div_ceil(count);
    if last_columns == Some(columns) {
      break;
    }
    let rows = compute_layout(windows, &bump, count);
    let measure = scale_and_space(&rows, area, spacing);
    if best
      .as_ref()
      .is_some_and(|(_, best)| !is_better(*best, measure))
    {
      break;
    }
    best = Some((rows, measure));
    last_columns = Some(columns);
  }

  best.map_or_else(Vec::new, |(rows, (scale, _))| {
    window_slots(windows, &bump, &rows, scale, area, spacing)
  })
}

/// `computeLayout`: `windows` into `count` rows, each about as wide.
#[allow(clippy::cast_precision_loss)]
fn compute_layout(
  windows: &[RectF],
  bump: &[f32],
  count: usize,
) -> Vec<Row> {
  let size = |index: usize| {
    (
      windows[index].w * bump[index],
      windows[index].h * bump[index],
    )
  };
  let total = (0..windows.len()).map(|index| size(index).0).sum::<f32>();
  let ideal = total / count as f32;
  let mut sorted = (0..windows.len()).collect::<Vec<_>>();
  sorted.sort_by(|&a, &b| {
    windows[a].center().1.total_cmp(&windows[b].center().1)
  });

  let mut rows = Vec::with_capacity(count);
  let mut next = 0;
  for index in 0..count {
    let mut row = Row::default();
    while let Some(&window) = sorted.get(next) {
      let (width, height) = size(window);
      // As GNOME does, even for a window that then starts the next row.
      row.full_height = row.full_height.max(height);
      // `_keepSameRow`: under the ideal width, or nearer it than without.
      let keeps = row.full_width + width <= ideal
        || (1.0 - (row.full_width + width) / ideal).abs()
          < (1.0 - row.full_width / ideal).abs();
      if !keeps && index != count - 1 {
        break;
      }
      row.windows.push(window);
      row.full_width += width;
      next += 1;
    }
    row.windows.sort_by(|&a, &b| {
      windows[a].center().0.total_cmp(&windows[b].center().0)
    });
    rows.push(row);
  }
  rows
}

/// `_computeScaleAndSpace`: the scale `rows` fit `area` at, and the part
/// of it they fill.
#[allow(clippy::cast_precision_loss)]
fn scale_and_space(
  rows: &[Row],
  area: &RectF,
  spacing: f32,
) -> (f32, f32) {
  // The first widest, as GNOME picks it.
  let widest = rows
    .iter()
    .fold(None::<&Row>, |widest, row| match widest {
      Some(widest) if widest.full_width >= row.full_width => Some(widest),
      _ => Some(row),
    })
    .map_or((0.0, 0), |row| (row.full_width, row.windows.len()));
  let grid_h = rows.iter().map(|row| row.full_height).sum::<f32>();
  let h_spacing = (widest.1 as f32 - 1.0) * spacing;
  let v_spacing = (rows.len() as f32 - 1.0) * spacing;
  let scale = ((area.w - h_spacing) / widest.0.max(1.0))
    .min((area.h - v_spacing) / grid_h.max(1.0))
    .min(MOST_SCALE);
  let space = (widest.0 * scale + h_spacing)
    * (grid_h * scale + v_spacing)
    / (area.w * area.h).max(1.0);
  (scale, space)
}

/// `_isBetterScaleAndSpace`.
fn is_better(
  (old_scale, old_space): (f32, f32),
  (scale, space): (f32, f32),
) -> bool {
  let space_power = (space - old_space) * LAYOUT_SPACE_WEIGHT;
  let scale_power = (scale - old_scale) * LAYOUT_SCALE_WEIGHT;
  match (scale > old_scale, space > old_space) {
    (true, true) => true,
    (true, false) => scale_power > space_power,
    (false, true) => space_power > scale_power,
    (false, false) => false,
  }
}

/// `computeWindowSlots`: each window's rect, its rows centered in `area`,
/// windows bottom-aligned in theirs (centered in a lone row).
#[allow(clippy::cast_precision_loss)]
fn window_slots(
  windows: &[RectF],
  bump: &[f32],
  rows: &[Row],
  scale: f32,
  area: &RectF,
  spacing: f32,
) -> Vec<RectF> {
  let spacings = |count: usize| (count as f32 - 1.0) * spacing;
  let heights = rows
    .iter()
    .map(|row| row.full_height * scale)
    .collect::<Vec<_>>();
  let height_no_spacing = heights.iter().sum::<f32>();
  let v_spacing = spacings(rows.len());
  let add_v = ((area.h - v_spacing) / height_no_spacing.max(1.0)).min(1.0);

  // Each row's left edge, top, height and extra scale.
  let mut compensation = 0.0;
  let mut y = 0.0;
  let mut placed = Vec::with_capacity(rows.len());
  for (row, height) in rows.iter().zip(&heights) {
    let h_spacing = spacings(row.windows.len());
    let width_no_spacing = row.full_width * scale;
    let add_h = if width_no_spacing > 0.0 {
      ((area.w - h_spacing) / width_no_spacing).min(1.0)
    } else {
      1.0
    };
    let add = if add_h < add_v {
      compensation += (add_v - add_h) * height;
      add_h
    } else {
      add_v
    };
    let x = area.x
      + (area.w - (width_no_spacing * add + h_spacing)).max(0.0) / 2.0;
    let top = area.y
      + (area.h - (height_no_spacing + v_spacing)).max(0.0) / 2.0
      + y;
    y += height * add + spacing;
    placed.push((x, top, height * add, add));
  }
  compensation /= 2.0;

  let mut rects = vec![RectF::default(); windows.len()];
  for (row, (mut x, top, row_h, add)) in rows.iter().zip(placed) {
    let top = top + compensation;
    for &index in &row.windows {
      let window = &windows[index];
      let slot_scale = scale * bump[index] * add;
      let (slot_w, slot_h) =
        (window.w * slot_scale, window.h * slot_scale);
      let shown = slot_scale.min(MOST_SCALE);
      let (w, h) = (window.w * shown, window.h * shown);
      let wy = if rows.len() == 1 {
        top + (row_h - h) / 2.0
      } else {
        top + row_h - slot_h + (slot_h - h)
      };
      rects[index] = RectF::new(x + (slot_w - w) / 2.0, wy, w, h);
      x += slot_w + spacing;
    }
  }
  rects
}
