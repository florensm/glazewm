//! Geometry of the window overview: a grid of window previews fitted into
//! the overview's area, what a point in it hits, and how the selection
//! moves between previews.
//!
//! All values are physical pixels relative to the overview's top-left
//! corner.

use std::ops::Range;

use crate::{Direction, Rect};

/// What the user did in the overview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewAction {
  /// Focus the window with this handle.
  Pick(isize),

  /// Close the overview, giving focus back to the window that had it.
  Cancel,

  /// Another window took focus, which closed the overview.
  Deactivated,
}

/// Inputs of an overview layout.
#[derive(Clone, Copy, Debug)]
pub struct OverviewLayoutParams<'a> {
  pub width: i32,
  pub height: i32,

  /// Space around and between previews.
  pub gap: i32,

  /// Space between a preview's thumbnail and the edge of its highlight.
  pub padding: i32,

  /// Height of the title under each thumbnail.
  pub title_height: i32,

  /// Size of each window, in display order.
  pub window_sizes: &'a [(i32, i32)],
}

/// Geometry of one window's preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewCell {
  /// The preview's highlight, used for hit-testing: thumbnail and title,
  /// plus padding.
  pub cell: Rect,

  /// Where the window's thumbnail goes, keeping its aspect ratio.
  pub thumbnail: Rect,

  /// Title area under the thumbnail.
  pub title: Rect,
}

/// Where a preview's icon and title text go within its title area.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewLabel {
  /// Icon square, if there is room for one.
  pub icon: Option<Rect>,
  pub text: Rect,
}

/// Laid-out overview.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OverviewLayout {
  /// One cell per window, in display order.
  pub cells: Vec<OverviewCell>,

  /// Ranges of `cells` making up each grid row, top to bottom.
  pub rows: Vec<Range<usize>>,
}

impl OverviewLayout {
  /// Lays out one preview per window in a grid of equal slots.
  ///
  /// Picks the column count that shows the windows largest overall.
  /// Thumbnails keep their window's aspect ratio and are never shown
  /// larger than the window itself. A short last row is centered.
  #[must_use]
  pub fn new(params: &OverviewLayoutParams) -> Self {
    let count = params.window_sizes.len();

    // On a tie, e.g. when every window fits at full size, the later and
    // wider grid wins.
    let columns = (1..=count)
      .filter_map(|columns| Some((columns, total_area(params, columns)?)))
      .max_by(|(_, a), (_, b)| a.total_cmp(b))
      .map(|(columns, _)| columns);

    let Some(columns) = columns else {
      return Self::default();
    };

    let slot = Slot::new(params, columns);
    let cells = params
      .window_sizes
      .iter()
      .enumerate()
      .map(|(index, size)| {
        let row = index / columns;
        let in_row = columns.min(count - row * columns);
        slot.cell(params, row, index % columns, in_row, *size)
      })
      .collect();

    let rows = (0..count)
      .step_by(columns)
      .map(|start| start..(start + columns).min(count))
      .collect();

    Self { cells, rows }
  }

  /// Index of the preview at (`x`, `y`), or `None` for the background.
  #[must_use]
  pub fn hit_test(&self, x: i32, y: i32) -> Option<usize> {
    self.cells.iter().position(|cell| {
      let cell = &cell.cell;
      x >= cell.left && x < cell.right && y >= cell.top && y < cell.bottom
    })
  }

  /// Index of the preview the selection moves to from `from`.
  ///
  /// Left and right step through the previews in reading order. Up and
  /// down move to the horizontally nearest preview in the adjacent row.
  /// The selection stays put at the edges.
  #[must_use]
  pub fn move_selection(
    &self,
    from: usize,
    direction: &Direction,
  ) -> usize {
    let Some(last) = self.cells.len().checked_sub(1) else {
      return from;
    };
    let from = from.min(last);

    match direction {
      Direction::Left => from.saturating_sub(1),
      Direction::Right => (from + 1).min(last),
      Direction::Up | Direction::Down => {
        let Some(row) =
          self.rows.iter().position(|row| row.contains(&from))
        else {
          return from;
        };

        let target = match direction {
          Direction::Up => row.checked_sub(1),
          _ => Some(row + 1),
        }
        .and_then(|row| self.rows.get(row));

        let center = self.center_x(from);
        target
          .and_then(|row| {
            row.clone().min_by_key(|index| {
              (self.center_x(*index) - center).unsigned_abs()
            })
          })
          .unwrap_or(from)
      }
    }
  }

  fn center_x(&self, index: usize) -> i32 {
    self
      .cells
      .get(index)
      .map_or(0, |cell| i32::midpoint(cell.cell.left, cell.cell.right))
  }
}

/// Places an icon of `icon_size` and a title `text_width` wide, side by
/// side and centered, within `title`. The text is cut to the room left.
#[must_use]
pub fn label_layout(
  title: &Rect,
  icon_size: Option<i32>,
  spacing: i32,
  text_width: i32,
) -> OverviewLabel {
  let icon_size = icon_size.filter(|size| {
    *size > 0
      && *size + spacing <= title.width()
      && *size <= title.height()
  });
  let icon_width = icon_size.map_or(0, |size| size + spacing);
  let text_width = text_width.clamp(0, title.width() - icon_width);
  let left = title.left + (title.width() - icon_width - text_width) / 2;

  OverviewLabel {
    icon: icon_size.map(|size| {
      Rect::from_xy(
        left,
        title.top + (title.height() - size) / 2,
        size,
        size,
      )
    }),
    text: Rect::from_ltrb(
      left + icon_width,
      title.top,
      left + icon_width + text_width,
      title.bottom,
    ),
  }
}

/// Size of a grid slot, and the room in it for a thumbnail.
struct Slot {
  width: i32,
  height: i32,
  thumbnail_width: i32,
  thumbnail_height: i32,
  columns: i32,
}

impl Slot {
  fn new(params: &OverviewLayoutParams, columns: usize) -> Self {
    let rows = params.window_sizes.len().div_ceil(columns);
    let columns = i32::try_from(columns).unwrap_or(i32::MAX);
    let rows = i32::try_from(rows).unwrap_or(i32::MAX);

    let width =
      (params.width - params.gap.saturating_mul(columns + 1)) / columns;
    let height =
      (params.height - params.gap.saturating_mul(rows + 1)) / rows.max(1);

    Self {
      width,
      height,
      thumbnail_width: width - 2 * params.padding,
      thumbnail_height: height - 2 * params.padding - params.title_height,
      columns,
    }
  }

  fn fits(&self) -> bool {
    self.thumbnail_width > 0 && self.thumbnail_height > 0
  }

  /// Cell of the window of `size` in `column` of `row`, which holds
  /// `in_row` windows.
  fn cell(
    &self,
    params: &OverviewLayoutParams,
    row: usize,
    column: usize,
    in_row: usize,
    size: (i32, i32),
  ) -> OverviewCell {
    let to_i32 = |value: usize| i32::try_from(value).unwrap_or(0);
    let stride_x = self.width + params.gap;
    let stride_y = self.height + params.gap;

    let row_offset = (self.columns - to_i32(in_row)) * stride_x / 2;
    let slot_left = params.gap + to_i32(column) * stride_x + row_offset;
    let slot_top = params.gap + to_i32(row) * stride_y;

    let (width, height) = self.fit(size);
    let cell_width = width + 2 * params.padding;
    let cell_height = height + params.title_height + 2 * params.padding;
    let left = slot_left + (self.width - cell_width) / 2;
    let top = slot_top + (self.height - cell_height) / 2;

    let thumbnail = Rect::from_xy(
      left + params.padding,
      top + params.padding,
      width,
      height,
    );
    let title = Rect::from_ltrb(
      thumbnail.left,
      thumbnail.bottom,
      thumbnail.right,
      thumbnail.bottom + params.title_height,
    );

    OverviewCell {
      cell: Rect::from_xy(left, top, cell_width, cell_height),
      thumbnail,
      title,
    }
  }

  /// Scale at which a window of `size` fits the slot, at most 1.
  fn scale(&self, (width, height): (i32, i32)) -> f64 {
    let (width, height) =
      (f64::from(width.max(1)), f64::from(height.max(1)));

    (f64::from(self.thumbnail_width) / width)
      .min(f64::from(self.thumbnail_height) / height)
      .min(1.0)
  }

  /// Thumbnail size of a window of `size`, at least 1x1.
  fn fit(&self, size: (i32, i32)) -> (i32, i32) {
    let scale = self.scale(size);

    #[allow(clippy::cast_possible_truncation)]
    let scaled = |length: i32| {
      ((f64::from(length.max(1)) * scale).round() as i32).max(1)
    };

    (scaled(size.0), scaled(size.1))
  }
}

/// Total thumbnail area with `columns` columns, or `None` if the slots
/// have no room for a thumbnail.
fn total_area(
  params: &OverviewLayoutParams,
  columns: usize,
) -> Option<f64> {
  let slot = Slot::new(params, columns);
  if !slot.fits() {
    return None;
  }

  Some(
    params
      .window_sizes
      .iter()
      .map(|size| {
        let (width, height) = slot.fit(*size);
        f64::from(width) * f64::from(height)
      })
      .sum(),
  )
}

#[cfg(test)]
mod tests {
  use super::{label_layout, OverviewLayout, OverviewLayoutParams};
  use crate::{Direction, Rect};

  fn layout(
    width: i32,
    height: i32,
    sizes: &[(i32, i32)],
  ) -> OverviewLayout {
    OverviewLayout::new(&OverviewLayoutParams {
      width,
      height,
      gap: 20,
      padding: 10,
      title_height: 30,
      window_sizes: sizes,
    })
  }

  fn row_lengths(layout: &OverviewLayout) -> Vec<usize> {
    layout.rows.iter().map(ExactSizeIterator::len).collect()
  }

  #[test]
  fn no_windows_is_empty() {
    assert_eq!(layout(1920, 1080, &[]), OverviewLayout::default());
  }

  #[test]
  fn four_landscape_windows_form_a_square_grid() {
    let layout = layout(1920, 1080, &[(1920, 1080); 4]);
    assert_eq!(row_lengths(&layout), [2, 2]);
  }

  #[test]
  fn tall_windows_share_one_row() {
    let layout = layout(1920, 1080, &[(600, 1400); 3]);
    assert_eq!(row_lengths(&layout), [3]);
  }

  #[test]
  fn small_windows_are_not_upscaled() {
    let layout = layout(1920, 1080, &[(400, 300), (200, 100)]);
    assert_eq!(row_lengths(&layout), [2]);
    assert_eq!(layout.cells[0].thumbnail.width(), 400);
    assert_eq!(layout.cells[0].thumbnail.height(), 300);
    assert_eq!(layout.cells[1].thumbnail.width(), 200);
  }

  #[test]
  fn thumbnails_keep_aspect_ratio_and_fit_their_cells() {
    let sizes = [(1920, 1080), (800, 1200), (1000, 1000), (3000, 500)];
    let layout = layout(1920, 1080, &sizes);

    for (cell, (width, height)) in layout.cells.iter().zip(sizes) {
      let thumbnail = &cell.thumbnail;
      let expected = f64::from(width) / f64::from(height);
      let actual =
        f64::from(thumbnail.width()) / f64::from(thumbnail.height());
      assert!((expected - actual).abs() / expected < 0.02, "{cell:?}");

      assert!(cell.cell.contains_rect(thumbnail));
      assert!(cell.cell.contains_rect(&cell.title));
      assert_eq!(cell.title.top, thumbnail.bottom);
      assert!(Rect::from_ltrb(0, 0, 1920, 1080).contains_rect(&cell.cell));
    }
  }

  #[test]
  fn cells_do_not_overlap() {
    let layout = layout(1920, 1080, &[(1280, 720); 7]);

    for (index, cell) in layout.cells.iter().enumerate() {
      for other in &layout.cells[index + 1..] {
        assert_eq!(cell.cell.intersection_area(&other.cell), 0);
      }
    }
  }

  #[test]
  fn short_last_row_is_centered() {
    let layout = layout(1920, 1080, &[(1920, 1080); 5]);
    assert_eq!(row_lengths(&layout), [3, 2]);

    let last_row = &layout.cells[3..];
    let left_margin = last_row[0].cell.left;
    let right_margin = 1920 - last_row[1].cell.right;
    assert!((left_margin - right_margin).abs() <= 2);
  }

  #[test]
  fn too_small_an_area_is_empty() {
    assert_eq!(layout(50, 50, &[(800, 600)]), OverviewLayout::default());
  }

  #[test]
  fn hit_test_finds_cells_and_background() {
    let layout = layout(1920, 1080, &[(1920, 1080); 2]);
    let cell = &layout.cells[1].cell;

    assert_eq!(layout.hit_test(cell.left, cell.top), Some(1));
    assert_eq!(layout.hit_test(cell.right - 1, cell.bottom - 1), Some(1));
    assert_eq!(layout.hit_test(cell.right, cell.top), None);
    assert_eq!(layout.hit_test(0, 0), None);
  }

  #[test]
  fn left_and_right_step_in_reading_order() {
    let layout = layout(1920, 1080, &[(1920, 1080); 5]);

    assert_eq!(layout.move_selection(2, &Direction::Right), 3);
    assert_eq!(layout.move_selection(3, &Direction::Left), 2);
    assert_eq!(layout.move_selection(0, &Direction::Left), 0);
    assert_eq!(layout.move_selection(4, &Direction::Right), 4);
  }

  #[test]
  fn up_and_down_pick_the_nearest_column() {
    // Rows of 3 and 2; the short row is centered under the gaps.
    let layout = layout(1920, 1080, &[(1920, 1080); 5]);

    assert_eq!(layout.move_selection(0, &Direction::Down), 3);
    assert_eq!(layout.move_selection(2, &Direction::Down), 4);
    assert_eq!(layout.move_selection(4, &Direction::Up), 1);
    assert_eq!(layout.move_selection(1, &Direction::Up), 1);
    assert_eq!(layout.move_selection(3, &Direction::Down), 3);
  }

  #[test]
  fn selection_out_of_range_is_clamped() {
    let layout = layout(1920, 1080, &[(1920, 1080); 2]);
    assert_eq!(layout.move_selection(9, &Direction::Left), 0);
    assert_eq!(
      OverviewLayout::default().move_selection(3, &Direction::Up),
      3
    );
  }

  #[test]
  fn label_centers_icon_and_text() {
    let title = Rect::from_ltrb(100, 0, 300, 30);
    let label = label_layout(&title, Some(20), 6, 74);

    assert_eq!(label.icon, Some(Rect::from_xy(150, 5, 20, 20)));
    assert_eq!(label.text, Rect::from_ltrb(176, 0, 250, 30));
  }

  #[test]
  fn label_cuts_long_text_and_drops_icon_without_room() {
    let title = Rect::from_ltrb(0, 0, 100, 30);

    let label = label_layout(&title, Some(20), 6, 500);
    assert_eq!(label.text, Rect::from_ltrb(26, 0, 100, 30));

    let narrow = Rect::from_ltrb(0, 0, 15, 30);
    let label = label_layout(&narrow, Some(20), 6, 500);
    assert_eq!(label.icon, None);
    assert_eq!(label.text, narrow);
  }
}
