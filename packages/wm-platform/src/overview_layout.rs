//! Geometry of the window overview: a card per workspace, laid out as a
//! carousel or a grid, each showing its windows where they really sit,
//! scaled down.
//!
//! Values are physical pixels relative to the overview's top-left corner,
//! kept fractional so animated positions don't snap to whole pixels until
//! drawn.

/// A rectangle with fractional coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RectF {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
}

impl RectF {
  pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
    Self { x, y, w, h }
  }

  pub fn right(&self) -> f32 {
    self.x + self.w
  }

  pub fn bottom(&self) -> f32 {
    self.y + self.h
  }

  pub fn contains(&self, x: f32, y: f32) -> bool {
    x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
  }

  /// This rect shrunk by `by` on every side.
  pub fn inset(&self, by: f32) -> Self {
    Self::new(
      self.x + by,
      self.y + by,
      (self.w - 2.0 * by).max(0.0),
      (self.h - 2.0 * by).max(0.0),
    )
  }

  /// The overlap of two rects, if any.
  pub fn intersect(&self, other: &Self) -> Option<Self> {
    let left = self.x.max(other.x);
    let top = self.y.max(other.y);
    let right = self.right().min(other.right());
    let bottom = self.bottom().min(other.bottom());

    (right > left && bottom > top)
      .then(|| Self::new(left, top, right - left, bottom - top))
  }

  /// The smallest rect containing both.
  pub fn union(&self, other: &Self) -> Self {
    let left = self.x.min(other.x);
    let top = self.y.min(other.y);
    Self::new(
      left,
      top,
      self.right().max(other.right()) - left,
      self.bottom().max(other.bottom()) - top,
    )
  }

  /// Edges rounded to whole pixels, so adjacent rects stay adjacent.
  #[allow(clippy::cast_possible_truncation)]
  pub fn to_rect(self) -> crate::Rect {
    crate::Rect::from_ltrb(
      self.x.round() as i32,
      self.y.round() as i32,
      self.right().round() as i32,
      self.bottom().round() as i32,
    )
  }

  #[allow(clippy::cast_precision_loss)]
  pub fn from_rect(rect: &crate::Rect) -> Self {
    Self::new(
      rect.left as f32,
      rect.top as f32,
      rect.width() as f32,
      rect.height() as f32,
    )
  }
}

/// Fixed sizes of a card at scale 1, derived from the overview's size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardMetrics {
  pub width: f32,
  pub height: f32,
  /// Height of the strip above the windows holding the workspace's name.
  pub header: f32,
  pub padding: f32,
  /// Width over height of the area a card maps, i.e. the monitor's.
  pub aspect: f32,
  /// How many times wider than a normal card's its tile area is, for a
  /// card widened to show a wide workspace.
  pub stretch: f32,
  pub spacing: f32,
  pub scale_factor: f32,
  /// Size of the overview itself.
  pub view: (f32, f32),
}

/// Most a card is widened to, in normal card widths.
const MAX_CARD_WIDTHS: f32 = 2.0;

impl CardMetrics {
  /// Cards for an overview of `width` x `height` covering a monitor with
  /// that same area: about a quarter of its width, in a sensible range.
  pub fn new(width: f32, height: f32, scale_factor: f32) -> Self {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
      scale_factor
    } else {
      1.0
    };
    let px = |length: f32| length * scale_factor;

    let card_width = (width * 0.26).clamp(px(380.0), px(900.0));
    let header = px(28.0);
    let padding = px(9.0);
    let aspect = width / height;

    Self {
      width: card_width,
      height: header + (card_width - 2.0 * padding) / aspect + padding,
      header,
      padding,
      aspect,
      stretch: 1.0,
      spacing: px(12.0),
      scale_factor,
      view: (width, height),
    }
  }

  /// A card showing `content` of the monitor's `area`: as tall as a
  /// normal card, and as much wider as `content` is wider than `area`, up
  /// to `MAX_CARD_WIDTHS` normal cards. Content wider still is
  /// letterboxed by its `CardView`.
  pub fn for_content(&self, area: &RectF, content: &RectF) -> Self {
    let content = content.union(area);
    let ratio = (content.w / content.h) / (area.w / area.h);

    // Exactly `self` for content that fits, so such cards are unchanged.
    if !ratio.is_finite() || ratio <= 1.0 {
      return *self;
    }

    let tiles = self.width - 2.0 * self.padding;
    let max_stretch =
      (MAX_CARD_WIDTHS * self.width - 2.0 * self.padding) / tiles;
    let stretch = ratio.min(max_stretch);

    Self {
      width: tiles * stretch + 2.0 * self.padding,
      stretch,
      ..*self
    }
  }

  /// Where windows are drawn within a card at scale 1.
  pub fn tile_area(&self) -> RectF {
    let width = self.width - 2.0 * self.padding;
    RectF::new(
      self.padding,
      self.header,
      width,
      width / (self.aspect * self.stretch),
    )
  }

  fn px(&self, length: f32) -> f32 {
    length * self.scale_factor
  }
}

/// Where a card sits: its center and scale.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Placement {
  pub cx: f32,
  pub cy: f32,
  pub scale: f32,
}

impl Placement {
  /// The card's rect in the overview.
  pub fn rect(&self, metrics: &CardMetrics) -> RectF {
    let (width, height) =
      (metrics.width * self.scale, metrics.height * self.scale);
    RectF::new(
      self.cx - width / 2.0,
      self.cy - height / 2.0,
      width,
      height,
    )
  }

  /// `local`, in the coordinates of a card at scale 1, in the overview.
  pub fn map(&self, metrics: &CardMetrics, local: &RectF) -> RectF {
    let card = self.rect(metrics);
    RectF::new(
      card.x + local.x * self.scale,
      card.y + local.y * self.scale,
      local.w * self.scale,
      local.h * self.scale,
    )
  }

  /// The overview point (`x`, `y`) in card coordinates at scale 1, if it
  /// is over this card.
  pub fn unmap(
    &self,
    metrics: &CardMetrics,
    x: f32,
    y: f32,
  ) -> Option<(f32, f32)> {
    let card = self.rect(metrics);
    if !card.contains(x, y) || self.scale <= 0.0 {
      return None;
    }
    Some(((x - card.x) / self.scale, (y - card.y) / self.scale))
  }
}

/// Where every card goes, and the line of key hints under them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Targets {
  pub cards: Vec<Placement>,
  pub hint_y: f32,

  /// Top edge of the special workspace's docked card, or the bottom of
  /// the overview when there is none.
  pub dock_top: f32,
}

/// What the layouts need of a card.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardSlot {
  /// Its width at scale 1.
  pub width: f32,

  /// The special workspace's card, set apart after the others.
  pub is_special: bool,
}

impl CardSlot {
  pub fn new(metrics: &CardMetrics, is_special: bool) -> Self {
    Self {
      width: metrics.width,
      is_special,
    }
  }
}

/// The special workspace's card at rest, relative to a normal card.
const DOCK_SCALE: f32 = 0.42;

/// The special workspace's card while a window is carried or dragged, so
/// it is an easy place to drop it on.
const DOCK_RECEIVING_SCALE: f32 = 0.78;

/// The special workspace's card, docked at the bottom edge of the
/// overview, apart from the other workspaces.
pub(crate) fn dock(
  metrics: &CardMetrics,
  slot: CardSlot,
  is_receiving: bool,
) -> Placement {
  let scale = if is_receiving {
    DOCK_RECEIVING_SCALE
  } else {
    DOCK_SCALE
  };
  // A wide strip's card stays clear of the overview's sides.
  let scale = scale.min((metrics.view.0 - metrics.px(160.0)) / slot.width);
  let height = metrics.height * scale;

  Placement {
    cx: metrics.view.0 / 2.0,
    cy: metrics.view.1 - metrics.px(28.0) - height / 2.0,
    scale,
  }
}

/// Room at the bottom of the overview that the dock can grow into.
fn dock_reserve(metrics: &CardMetrics, slots: &[CardSlot]) -> f32 {
  if slots.iter().any(|slot| slot.is_special) {
    metrics.height * DOCK_RECEIVING_SCALE + metrics.px(28.0)
  } else {
    0.0
  }
}

/// Places the special workspace's cards on the dock and the others with
/// `place`, which gets only the others' slots and, as `selected`, the
/// index among them of `centered` to center on.
fn with_dock(
  metrics: &CardMetrics,
  slots: &[CardSlot],
  centered: usize,
  is_receiving: bool,
  place: impl FnOnce(&[CardSlot], usize) -> (Vec<Placement>, f32),
) -> Targets {
  let regular = slots
    .iter()
    .enumerate()
    .filter(|(_, slot)| !slot.is_special)
    .map(|(index, _)| index)
    .collect::<Vec<_>>();

  let regular_selected = regular
    .iter()
    .position(|index| *index >= centered)
    .unwrap_or(regular.len())
    .min(regular.len().saturating_sub(1));

  let regular_slots = regular
    .iter()
    .map(|index| slots[*index])
    .collect::<Vec<_>>();
  let (placed, hint_y) = place(&regular_slots, regular_selected);

  let mut dock_top = metrics.view.1;
  let mut placed = placed.into_iter();
  let cards = slots
    .iter()
    .map(|slot| {
      if slot.is_special {
        let placement = dock(metrics, *slot, is_receiving);
        dock_top = dock_top.min(placement.rect(metrics).y);
        placement
      } else {
        placed.next().unwrap_or_default()
      }
    })
    .collect();

  Targets {
    cards,
    hint_y,
    dock_top,
  }
}

/// One row of cards with the `centered` one in the middle at full size,
/// and the special workspace's card docked below.
///
/// `zoomed` is for looking inside the centered workspace: it grows to
/// 1.5x and its neighbours sit far back. `is_receiving` grows the dock
/// while a window is carried or dragged.
pub(crate) fn carousel(
  metrics: &CardMetrics,
  slots: &[CardSlot],
  centered: usize,
  zoomed: bool,
  is_receiving: bool,
) -> Targets {
  let reserve = dock_reserve(metrics, slots);

  with_dock(metrics, slots, centered, is_receiving, |slots, selected| {
    let scale_of = |index: usize| match (index == selected, zoomed) {
      (true, true) => 1.5,
      (true, false) => 1.0,
      (false, true) => 0.32,
      (false, false) => 0.66,
    };

    let mut starts = Vec::with_capacity(slots.len());
    let mut x = 0.0;
    for (index, slot) in slots.iter().enumerate() {
      starts.push(x);
      x += slot.width * scale_of(index) + metrics.spacing;
    }

    let selected_width =
      slots.get(selected).map_or(0.0, |slot| slot.width)
        * scale_of(selected);
    let offset = metrics.view.0 / 2.0
      - (starts.get(selected).copied().unwrap_or(0.0)
        + selected_width / 2.0);

    // The row grows with the zoom, so a busy workspace gets the room it
    // needs, and is centered in what the dock leaves.
    let row_height = metrics.height * if zoomed { 1.5 } else { 1.02 };
    let top =
      (metrics.view.1 - reserve - row_height - metrics.px(30.0)) / 2.0;
    let cy = top + row_height / 2.0;

    let cards = starts
      .iter()
      .zip(slots)
      .enumerate()
      .map(|(index, (start, slot))| {
        let scale = scale_of(index);
        Placement {
          cx: start + slot.width * scale / 2.0 + offset,
          cy,
          scale,
        }
      })
      .collect();

    (cards, top + row_height + metrics.px(16.0))
  })
}

/// A card's place in the grid, in columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GridCell {
  pub row: usize,
  pub column: usize,
  pub span: usize,
}

/// Where the grid puts each card: rows of up to `columns` columns and a
/// widened card taking two of them. The special workspace's card, which
/// is docked below the grid, counts as a row of its own after them, so
/// moving down from the last row reaches it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GridCells {
  pub cells: Vec<GridCell>,
  pub columns: usize,

  /// Columns taken in each row.
  row_spans: Vec<usize>,
}

impl GridCells {
  pub fn new(
    metrics: &CardMetrics,
    slots: &[CardSlot],
    columns: usize,
  ) -> Self {
    let span_of = |slot: &CardSlot| {
      if slot.width > metrics.width {
        2
      } else {
        1
      }
    };
    let regular = slots.iter().filter(|slot| !slot.is_special);
    let total_span = regular.clone().map(span_of).sum::<usize>();
    let max_span = regular.map(span_of).max().unwrap_or(1);
    let columns = columns.clamp(1, total_span.max(1)).max(max_span);

    let mut cells = vec![GridCell::default(); slots.len()];
    let mut row_spans: Vec<usize> = Vec::new();

    for (index, slot) in slots.iter().enumerate() {
      if slot.is_special {
        continue;
      }

      let span = span_of(slot);
      let last_row = row_spans.len().saturating_sub(1);
      let cell = match row_spans.last_mut() {
        Some(taken) if *taken + span <= columns => {
          *taken += span;
          GridCell {
            row: last_row,
            column: *taken - span,
            span,
          }
        }
        _ => {
          row_spans.push(span);
          GridCell {
            row: row_spans.len() - 1,
            column: 0,
            span,
          }
        }
      };
      cells[index] = cell;
    }

    for (index, slot) in slots.iter().enumerate() {
      if slot.is_special {
        row_spans.push(columns);
        cells[index] = GridCell {
          row: row_spans.len() - 1,
          column: 0,
          span: columns,
        };
      }
    }

    Self {
      cells,
      columns,
      row_spans,
    }
  }

  /// Columns taken in `row`.
  pub fn taken(&self, row: usize) -> usize {
    self.row_spans.get(row).copied().unwrap_or(0)
  }

  /// Where the middle of `cell` is across the grid, in columns, with a
  /// short row centered.
  #[allow(clippy::cast_precision_loss)]
  fn center(&self, cell: &GridCell) -> f32 {
    self.columns.saturating_sub(self.taken(cell.row)) as f32 / 2.0
      + cell.column as f32
      + cell.span as f32 / 2.0
  }

  /// The card in the row `delta` rows from card `index`'s that sits
  /// closest under (or over) it.
  pub fn vertical_neighbour(
    &self,
    index: usize,
    delta: isize,
  ) -> Option<usize> {
    let from = self.cells.get(index)?;
    let row = from.row.checked_add_signed(delta)?;
    let center = self.center(from);

    self
      .cells
      .iter()
      .enumerate()
      .filter(|(_, cell)| cell.row == row)
      .min_by(|(_, a), (_, b)| {
        (self.center(a) - center)
          .abs()
          .total_cmp(&(self.center(b) - center).abs())
      })
      .map(|(index, _)| index)
  }
}

/// Every card at once, in `cells`, as large as fits above the special
/// workspace's docked card.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn grid(
  metrics: &CardMetrics,
  slots: &[CardSlot],
  cells: &GridCells,
  is_receiving: bool,
) -> Targets {
  let regular_cells = slots
    .iter()
    .zip(&cells.cells)
    .filter(|(slot, _)| !slot.is_special)
    .map(|(_, cell)| *cell)
    .collect::<Vec<_>>();
  let rows = regular_cells
    .iter()
    .map(|cell| cell.row + 1)
    .max()
    .unwrap_or(1);

  let gap = metrics.px(14.0);
  let (columns_f, rows_f) = (cells.columns as f32, rows as f32);

  // Room is left for the hint line, a carried window parked above the
  // dock, and the dock.
  let available_width = metrics.view.0 - metrics.px(120.0);
  let available_height =
    metrics.view.1 - metrics.px(240.0) - dock_reserve(metrics, slots);
  let scale = ((available_width - gap * (columns_f - 1.0))
    / (columns_f * metrics.width))
    .min(
      (available_height - gap * (rows_f - 1.0))
        / (rows_f * metrics.height),
    )
    .clamp(0.1, 1.0);

  let (width, height) = (metrics.width * scale, metrics.height * scale);
  let total_width = columns_f * width + (columns_f - 1.0) * gap;
  let total_height = rows_f * height + (rows_f - 1.0) * gap;
  let left = (metrics.view.0 - total_width) / 2.0;
  let top = (metrics.view.1
    - dock_reserve(metrics, slots)
    - total_height
    - metrics.px(30.0))
    / 2.0;

  with_dock(metrics, slots, 0, is_receiving, |_, _| {
    let cards = regular_cells
      .iter()
      .map(|cell| {
        // A short row is centered rather than hanging off the left.
        let taken = cells.taken(cell.row) as f32;
        let row_left = left + (columns_f - taken) * (width + gap) / 2.0;
        let span = cell.span as f32;

        Placement {
          cx: row_left
            + cell.column as f32 * (width + gap)
            + span * width / 2.0
            + (span - 1.0) * gap / 2.0,
          cy: top + cell.row as f32 * (height + gap) + height / 2.0,
          scale,
        }
      })
      .collect();

    (cards, top + total_height + metrics.px(16.0))
  })
}

/// How a workspace's screen space maps onto its card at scale 1.
///
/// The card shows `content` at the scale it would show the monitor's
/// `area`, shrunk to fit and centered: for a scrolling workspace whose
/// strip is wider than the monitor, every column, letterboxed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardView {
  /// Screen area the card shows, containing `area`.
  content: RectF,
  /// Where `content`'s top-left corner lands on the card.
  origin: (f32, f32),
  scale_x: f32,
  scale_y: f32,
}

impl CardView {
  pub fn new(
    metrics: &CardMetrics,
    area: &RectF,
    content: &RectF,
  ) -> Self {
    let tiles = metrics.tile_area();
    let content = content.union(area);

    // What the card's tile area spans at the scale it shows `area`: wider
    // than `area` on a widened card.
    let shown = (area.w * metrics.stretch, area.h);

    // Exactly 1 when `content` is what is shown, so such a card maps the
    // same as one mapping `area` alone.
    let shrink = if area.w > 0.0 && area.h > 0.0 {
      (shown.0 / content.w).min(shown.1 / content.h)
    } else {
      1.0
    };

    Self {
      content,
      origin: (
        tiles.x + tiles.w * (1.0 - content.w / shown.0 * shrink) / 2.0,
        tiles.y + tiles.h * (1.0 - content.h / shown.1 * shrink) / 2.0,
      ),
      scale_x: tiles.w / shown.0 * shrink,
      scale_y: tiles.h / shown.1 * shrink,
    }
  }

  /// `rect` (screen coordinates) on the card: like a window's tile, but
  /// never clipped, so it keeps its shape.
  pub fn map(&self, rect: &RectF) -> RectF {
    RectF::new(
      self.origin.0 + (rect.x - self.content.x) * self.scale_x,
      self.origin.1 + (rect.y - self.content.y) * self.scale_y,
      rect.w * self.scale_x,
      rect.h * self.scale_y,
    )
  }

  /// Where a window at `window` (screen coordinates) goes on the card:
  /// where it sits, scaled down. `None` for a window entirely outside the
  /// card's content.
  pub fn tile(&self, window: &RectF) -> Option<Tile> {
    if window.w <= 0.0
      || window.h <= 0.0
      || !self.scale_x.is_finite()
      || !self.scale_y.is_finite()
    {
      return None;
    }

    let visible = window.intersect(&self.content)?;

    Some(Tile {
      rect: self.map(&visible),
      crop: RectF::new(
        (visible.x - window.x) / window.w,
        (visible.y - window.y) / window.h,
        visible.w / window.w,
        visible.h / window.h,
      ),
    })
  }
}

/// A window's place within a card at scale 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Tile {
  pub rect: RectF,

  /// The part of the window the tile shows, as fractions of its size;
  /// less than all of it when it reaches past its card's content.
  pub crop: RectF,
}

/// The zoom the overview opens with: at progress 0 it shows `from` filling
/// `to`, and at 1 everything sits where it belongs.
///
/// With `from` the focused workspace's tiles and `to` the monitor, the
/// windows start out exactly where they are on screen and shrink into
/// their card, with the other cards flying in around it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Camera {
  scale_x: f32,
  scale_y: f32,
  offset_x: f32,
  offset_y: f32,
}

impl Camera {
  pub const IDENTITY: Self = Self {
    scale_x: 1.0,
    scale_y: 1.0,
    offset_x: 0.0,
    offset_y: 0.0,
  };

  /// The zoom at eased `progress` from 0 to 1.
  pub fn zoom(from: &RectF, to: &RectF, progress: f32) -> Self {
    if from.w <= 0.0 || from.h <= 0.0 {
      return Self::IDENTITY;
    }

    let progress = progress.clamp(0.0, 1.0);
    let lerp = |a: f32, b: f32| a + (b - a) * progress;
    let (start_x, start_y) = (to.w / from.w, to.h / from.h);

    Self {
      scale_x: lerp(start_x, 1.0),
      scale_y: lerp(start_y, 1.0),
      offset_x: lerp(to.x - from.x * start_x, 0.0),
      offset_y: lerp(to.y - from.y * start_y, 0.0),
    }
  }

  /// The point `apply` maps onto (`x`, `y`).
  pub fn unapply(&self, x: f32, y: f32) -> (f32, f32) {
    (
      (x - self.offset_x) / self.scale_x,
      (y - self.offset_y) / self.scale_y,
    )
  }

  pub fn apply(&self, rect: &RectF) -> RectF {
    RectF::new(
      rect.x * self.scale_x + self.offset_x,
      rect.y * self.scale_y + self.offset_y,
      rect.w * self.scale_x,
      rect.h * self.scale_y,
    )
  }
}

/// The part of a `source`-sized image that fills `dest` without
/// stretching, as fractions of `source`: centered across and kept to the
/// top, so a window keeps its title bar.
pub(crate) fn cover_crop(source: (f32, f32), dest: (f32, f32)) -> RectF {
  let source_aspect = source.0 / source.1;
  let dest_aspect = dest.0 / dest.1;

  let is_valid = |aspect: f32| aspect.is_finite() && aspect > 0.0;
  if !is_valid(source_aspect) || !is_valid(dest_aspect) {
    return RectF::new(0.0, 0.0, 1.0, 1.0);
  }

  if source_aspect > dest_aspect {
    let width = dest_aspect / source_aspect;
    RectF::new((1.0 - width) / 2.0, 0.0, width, 1.0)
  } else {
    RectF::new(0.0, 0.0, 1.0, source_aspect / dest_aspect)
  }
}

/// A zoom between a card's windows filling the screen (openness 0) and
/// every card in place (openness 1): out of the card on open, into it on
/// close.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Zoom {
  /// Card whose windows fill the screen at openness 0.
  pub card: usize,
  pub is_closing: bool,
  from: f32,
  duration_ms: f32,
}

impl Zoom {
  /// Zooms out of `card` over `duration_ms`.
  #[allow(clippy::cast_precision_loss)]
  pub fn open(card: usize, duration_ms: u32) -> Self {
    Self {
      card,
      is_closing: false,
      from: 0.0,
      duration_ms: duration_ms as f32,
    }
  }

  /// Zooms into `card` from openness `from`. A partly open overview
  /// closes in that part of `duration_ms`, at the same pace.
  #[allow(clippy::cast_precision_loss)]
  pub fn close(card: usize, from: f32, duration_ms: u32) -> Self {
    let from = from.clamp(0.0, 1.0);
    Self {
      card,
      is_closing: true,
      from,
      duration_ms: duration_ms as f32 * from,
    }
  }

  /// Openness `elapsed_ms` in, eased.
  pub fn openness(&self, elapsed_ms: f32) -> f32 {
    let to = if self.is_closing { 0.0 } else { 1.0 };
    self.from
      + (to - self.from) * ease_out_cubic(self.progress(elapsed_ms))
  }

  pub fn is_finished(&self, elapsed_ms: f32) -> bool {
    self.progress(elapsed_ms) >= 1.0
  }

  fn progress(&self, elapsed_ms: f32) -> f32 {
    if self.duration_ms <= 0.0 {
      return 1.0;
    }
    (elapsed_ms / self.duration_ms).clamp(0.0, 1.0)
  }
}

/// Fast start, gentle stop.
pub(crate) fn ease_out_cubic(progress: f32) -> f32 {
  1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

/// A number that eases toward its target instead of jumping to it.
///
/// Critically damped, so it settles without overshooting, and retargeting
/// it mid-flight keeps its velocity rather than restarting the curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Spring {
  pub value: f32,
  pub target: f32,
  velocity: f32,
  stiffness: f32,
  damping: f32,
  /// How close to its target, in its own units, it counts as settled.
  epsilon: f32,
}

impl Spring {
  /// A spring resting at `value` that settles within about
  /// `settle_ms`.
  pub fn new(value: f32, settle_ms: f32, epsilon: f32) -> Self {
    let omega = 4.6 / (settle_ms.max(1.0) / 1000.0);

    Self {
      value,
      target: value,
      velocity: 0.0,
      stiffness: omega * omega,
      damping: 2.0 * omega,
      epsilon,
    }
  }

  pub fn snap(&mut self, value: f32) {
    self.value = value;
    self.target = value;
    self.velocity = 0.0;
  }

  #[allow(clippy::float_cmp)]
  pub fn is_settled(&self) -> bool {
    self.value == self.target && self.velocity == 0.0
  }

  /// Advances by `dt` seconds. Returns whether it is still moving.
  pub fn step(&mut self, dt: f32) -> bool {
    if self.is_settled() {
      return false;
    }

    let acceleration = (self.target - self.value) * self.stiffness
      - self.velocity * self.damping;
    self.velocity += acceleration * dt;
    self.value += self.velocity * dt;

    if (self.target - self.value).abs() < self.epsilon
      && self.velocity.abs() < self.epsilon * 4.0
    {
      self.snap(self.target);
    }

    true
  }
}

#[cfg(test)]
mod tests {
  use super::{
    carousel, cover_crop, ease_out_cubic, grid, Camera, CardMetrics,
    CardSlot, CardView, GridCell, GridCells, RectF, Spring, Tile, Zoom,
  };

  fn metrics() -> CardMetrics {
    CardMetrics::new(1920.0, 1040.0, 1.0)
  }

  /// `count` normal cards.
  fn slots(metrics: &CardMetrics, count: usize) -> Vec<CardSlot> {
    vec![CardSlot::new(metrics, false); count]
  }

  /// The card of a scrolling workspace whose strip is `widths` monitors
  /// wide.
  fn wide_slot(metrics: &CardMetrics, widths: f32) -> CardSlot {
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let strip = RectF::new(0.0, 0.0, 1920.0 * widths, 1040.0);
    CardSlot::new(&metrics.for_content(&area, &strip), false)
  }

  /// `metrics` of a card `width` wide.
  fn with_width(metrics: &CardMetrics, width: f32) -> CardMetrics {
    CardMetrics { width, ..*metrics }
  }

  /// Card rects of the grid of `slots`.
  fn grid_of(
    metrics: &CardMetrics,
    slots: &[CardSlot],
    columns: usize,
  ) -> Vec<RectF> {
    let cells = GridCells::new(metrics, slots, columns);
    grid(metrics, slots, &cells, false)
      .cards
      .iter()
      .zip(slots)
      .map(|(card, slot)| card.rect(&with_width(metrics, slot.width)))
      .collect()
  }

  fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
  }

  fn assert_rect_close(a: &RectF, b: &RectF) {
    assert_close(a.x, b.x);
    assert_close(a.y, b.y);
    assert_close(a.w, b.w);
    assert_close(a.h, b.h);
  }

  /// A card showing just `area`, as every non-scrolling workspace's does.
  fn tile(
    metrics: &CardMetrics,
    area: &RectF,
    window: &RectF,
  ) -> Option<Tile> {
    CardView::new(metrics, area, area).tile(window)
  }

  #[test]
  fn card_keeps_the_monitor_aspect() {
    let metrics = metrics();
    let tiles = metrics.tile_area();

    assert_close(metrics.width, 1920.0 * 0.26);
    assert_close(tiles.w / tiles.h, 1920.0 / 1040.0);
    assert_close(metrics.height, tiles.bottom() + metrics.padding);
  }

  #[test]
  fn carousel_centers_the_selected_card() {
    let metrics = metrics();
    let targets = carousel(&metrics, &slots(&metrics, 5), 2, false, false);

    assert_close(targets.cards[2].cx, 960.0);
    assert_close(targets.cards[2].scale, 1.0);
    assert_close(targets.cards[1].scale, 0.66);

    for pair in targets.cards.windows(2) {
      let (left, right) = (pair[0].rect(&metrics), pair[1].rect(&metrics));
      assert_close(right.x - left.right(), metrics.spacing);
    }
  }

  #[test]
  fn zoomed_carousel_grows_the_selected_card() {
    let metrics = metrics();
    let targets = carousel(&metrics, &slots(&metrics, 3), 0, true, false);

    assert_close(targets.cards[0].scale, 1.5);
    assert_close(targets.cards[1].scale, 0.32);
    assert_close(targets.cards[0].cx, 960.0);
    assert!(
      targets.hint_y > targets.cards[0].rect(&metrics).bottom(),
      "hints sit under the cards"
    );
  }

  #[test]
  fn grid_centers_a_short_last_row() {
    let metrics = metrics();
    let rects = grid_of(&metrics, &slots(&metrics, 7), 5);

    assert_close(rects[0].y, rects[4].y);
    assert!(rects[5].y > rects[0].bottom());
    assert_close(
      rects[5].x - rects[0].x,
      rects[4].right() - rects[6].right(),
    );

    for rect in &rects {
      assert!(rect.x >= 0.0 && rect.right() <= 1920.0);
      assert!(rect.y >= 0.0 && rect.bottom() <= 1040.0);
    }
  }

  #[test]
  fn grid_never_upscales_cards() {
    let metrics = metrics();
    let slots = slots(&metrics, 2);
    let cells = GridCells::new(&metrics, &slots, 5);
    let targets = grid(&metrics, &slots, &cells, false);
    assert!(targets.cards.iter().all(|card| card.scale <= 1.0));
  }

  #[test]
  fn tiles_keep_the_layout_of_the_workspace() {
    let metrics = metrics();
    let area = RectF::new(0.0, 40.0, 1920.0, 1040.0);
    let tiles = metrics.tile_area();

    let left =
      tile(&metrics, &area, &RectF::new(0.0, 40.0, 960.0, 1040.0))
        .unwrap_or_else(|| unreachable!());
    let right =
      tile(&metrics, &area, &RectF::new(960.0, 40.0, 960.0, 1040.0))
        .unwrap_or_else(|| unreachable!());

    assert_rect_close(
      &left.rect,
      &RectF::new(tiles.x, tiles.y, tiles.w / 2.0, tiles.h),
    );
    assert_close(right.rect.x, left.rect.right());
    assert_rect_close(&left.crop, &RectF::new(0.0, 0.0, 1.0, 1.0));
  }

  #[test]
  fn tiles_crop_windows_reaching_past_the_workspace() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let window = RectF::new(-200.0, 0.0, 400.0, 1040.0);

    let tile =
      tile(&metrics, &area, &window).unwrap_or_else(|| unreachable!());

    assert_close(tile.rect.x, metrics.tile_area().x);
    assert_rect_close(&tile.crop, &RectF::new(0.5, 0.0, 0.5, 1.0));
  }

  #[test]
  fn windows_outside_the_workspace_get_no_tile() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);

    assert_eq!(
      tile(&metrics, &area, &RectF::new(3000.0, 0.0, 100.0, 100.0)),
      None
    );
  }

  #[test]
  fn zoom_starts_with_windows_where_they_are() {
    let metrics = metrics();
    let view = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let card =
      carousel(&metrics, &slots(&metrics, 3), 1, false, false).cards[1];
    let tiles = card.map(&metrics, &metrics.tile_area());

    let window = RectF::new(960.0, 0.0, 960.0, 520.0);
    let on_card = card.map(
      &metrics,
      &tile(&metrics, &view, &window)
        .unwrap_or_else(|| unreachable!())
        .rect,
    );

    let start = Camera::zoom(&tiles, &view, 0.0);
    assert_rect_close(&start.apply(&on_card), &window);
    assert_rect_close(&start.apply(&tiles), &view);

    let end = Camera::zoom(&tiles, &view, 1.0);
    assert_rect_close(&end.apply(&on_card), &on_card);

    let (x, y) = start.unapply(window.x, window.y);
    assert_close(x, on_card.x);
    assert_close(y, on_card.y);
  }

  #[test]
  fn cover_crop_fills_without_stretching() {
    let wide = cover_crop((2000.0, 500.0), (200.0, 100.0));
    assert_eq!(wide, RectF::new(0.25, 0.0, 0.5, 1.0), "centered across");

    let tall = cover_crop((1000.0, 1000.0), (200.0, 100.0));
    assert_eq!(tall, RectF::new(0.0, 0.0, 1.0, 0.5), "kept to the top");

    let full = RectF::new(0.0, 0.0, 1.0, 1.0);
    assert_eq!(cover_crop((0.0, 0.0), (200.0, 100.0)), full);
    assert_eq!(cover_crop((800.0, 600.0), (0.0, 100.0)), full);
  }

  #[test]
  fn zoom_opens_out_of_a_card_and_closes_back_in() {
    let open = Zoom::open(2, 250);
    assert_close(open.openness(0.0), 0.0);
    assert!(open.openness(100.0) > 0.5, "fast start");
    assert_close(open.openness(250.0), 1.0);
    assert!(open.is_finished(250.0) && !open.is_finished(249.0));

    let close = Zoom::close(2, 1.0, 200);
    assert!(close.is_closing);
    assert_close(close.openness(0.0), 1.0);
    assert_close(close.openness(200.0), 0.0);
  }

  #[test]
  fn zoom_closes_a_half_open_overview_in_half_the_time() {
    let close = Zoom::close(0, 0.5, 200);
    assert_close(close.openness(0.0), 0.5);
    assert!(close.is_finished(100.0));
    assert_close(close.openness(100.0), 0.0);

    let instant = Zoom::close(0, 1.0, 0);
    assert!(instant.is_finished(0.0));
    assert_close(instant.openness(0.0), 0.0);
  }

  #[test]
  fn view_maps_like_a_tile_without_clipping() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let view = CardView::new(&metrics, &area, &area);
    let inside = RectF::new(100.0, 50.0, 800.0, 30.0);

    let mapped = view.map(&inside);
    let tiled = view.tile(&inside).expect("tile").rect;
    assert_close(mapped.x, tiled.x);
    assert_close(mapped.w, tiled.w);

    let past_edge = RectF::new(1800.0, 50.0, 400.0, 30.0);
    assert_close(
      view.map(&past_edge).w,
      400.0 * metrics.tile_area().w / 1920.0,
    );
  }

  #[test]
  fn view_of_just_the_area_fills_the_tile_area_exactly() {
    let metrics = metrics();
    let area = RectF::new(-1920.0, 40.0, 1920.0, 1040.0);
    let view = CardView::new(&metrics, &area, &area);

    assert_rect_close(&view.map(&area), &metrics.tile_area());
  }

  #[test]
  fn view_of_a_wide_strip_fits_every_column() {
    let metrics = metrics();
    let tiles = metrics.tile_area();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    // Scrolled one monitor width to the right of a strip three wide.
    let strip = RectF::new(-1920.0, 0.0, 3.0 * 1920.0, 1040.0);
    let view = CardView::new(&metrics, &area, &strip);

    let left = view
      .tile(&RectF::new(-1920.0, 0.0, 1920.0, 1040.0))
      .expect("left column");
    let right = view
      .tile(&RectF::new(1920.0, 0.0, 1920.0, 1040.0))
      .expect("right column");

    assert_close(left.rect.x, tiles.x);
    assert_close(right.rect.right(), tiles.right());
    assert_rect_close(&left.crop, &RectF::new(0.0, 0.0, 1.0, 1.0));

    // A third of the height, centered.
    assert_close(left.rect.h, tiles.h / 3.0);
    assert_close(
      left.rect.y - tiles.y,
      tiles.bottom() - left.rect.bottom(),
    );

    // The monitor is the middle third, keeping its shape.
    let screen = view.map(&area);
    assert_close(screen.x, left.rect.right());
    assert_close(screen.w / screen.h, area.w / area.h);
  }

  #[test]
  fn view_never_shows_less_than_the_area() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let view =
      CardView::new(&metrics, &area, &RectF::new(0.0, 0.0, 10.0, 10.0));

    assert_rect_close(&view.map(&area), &metrics.tile_area());
  }

  #[test]
  fn zoom_out_of_a_strip_starts_with_on_screen_windows_in_place() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let strip = RectF::new(-960.0, 0.0, 3840.0, 1040.0);
    let view = CardView::new(&metrics, &area, &strip);
    let card =
      carousel(&metrics, &slots(&metrics, 1), 0, false, false).cards[0];

    let window = RectF::new(960.0, 0.0, 960.0, 1040.0);
    let on_card =
      card.map(&metrics, &view.tile(&window).expect("tile").rect);
    let screen = card.map(&metrics, &view.map(&area));

    let start = Camera::zoom(&screen, &area, 0.0);
    assert_rect_close(&start.apply(&on_card), &window);
  }

  #[test]
  fn union_spans_both_rects() {
    let window = RectF::new(10.0, 40.0, 100.0, 60.0);
    let tab_bar = RectF::new(10.0, 30.0, 100.0, 12.0);
    assert_eq!(
      window.union(&tab_bar),
      RectF::new(10.0, 30.0, 100.0, 70.0)
    );
  }

  #[test]
  fn easing_runs_from_zero_to_one() {
    assert_close(ease_out_cubic(0.0), 0.0);
    assert_close(ease_out_cubic(1.0), 1.0);
    assert!(ease_out_cubic(0.5) > 0.5);
    assert_close(ease_out_cubic(2.0), 1.0);
  }

  #[test]
  fn spring_settles_on_target_without_overshoot() {
    let mut spring = Spring::new(0.0, 260.0, 0.25);
    spring.target = 100.0;

    let mut frames = 0;
    while spring.step(1.0 / 60.0) {
      assert!(spring.value <= 100.0 + f32::EPSILON);
      frames += 1;
      assert!(frames < 120, "spring never settled");
    }

    assert_close(spring.value, 100.0);
    assert!(frames > 5, "spring moved instantly");
    assert!(!spring.step(1.0 / 60.0));
  }

  #[test]
  fn cards_widen_with_their_strip_up_to_twice() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let strip =
      |widths: f32| RectF::new(0.0, 0.0, 1920.0 * widths, 1040.0);

    assert_eq!(metrics.for_content(&area, &area), metrics, "unchanged");

    let wider = metrics.for_content(&area, &strip(1.5));
    assert_close(wider.height, metrics.height);
    assert_close(wider.tile_area().h, metrics.tile_area().h);
    assert_close(wider.tile_area().w, metrics.tile_area().w * 1.5);

    let widest = metrics.for_content(&area, &strip(5.0));
    assert_close(widest.width, metrics.width * 2.0);
    assert_close(widest.height, metrics.height);
  }

  #[test]
  fn a_widened_card_shows_its_strip_without_letterboxing() {
    let metrics = metrics();
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let strip = RectF::new(-960.0, 0.0, 1920.0 * 1.5, 1040.0);
    let card = metrics.for_content(&area, &strip);

    assert_rect_close(
      &CardView::new(&card, &area, &strip).map(&strip),
      &card.tile_area(),
    );
  }

  #[test]
  fn carousel_centers_and_spaces_cards_of_any_width() {
    let metrics = metrics();
    let mut slots = slots(&metrics, 3);
    slots[1] = wide_slot(&metrics, 1.5);

    for selected in 0..3 {
      let targets = carousel(&metrics, &slots, selected, false, false);
      assert_close(targets.cards[selected].cx, 960.0);

      let rects = targets
        .cards
        .iter()
        .zip(&slots)
        .map(|(card, slot)| card.rect(&with_width(&metrics, slot.width)))
        .collect::<Vec<_>>();
      for pair in rects.windows(2) {
        assert_close(pair[1].x - pair[0].right(), metrics.spacing);
      }
    }
  }

  #[test]
  fn the_special_card_docks_below_the_carousel_and_grows_to_receive() {
    let metrics = metrics();
    let mut slots = slots(&metrics, 3);
    slots[2].is_special = true;

    let rects = |is_receiving| {
      carousel(&metrics, &slots, 0, false, is_receiving)
        .cards
        .iter()
        .map(|card| card.rect(&metrics))
        .collect::<Vec<_>>()
    };
    let (resting, receiving) = (rects(false), rects(true));

    // The others stay a row of their own; the special card sits below
    // them, centered.
    assert_close(resting[1].x - resting[0].right(), metrics.spacing);
    assert!(resting[2].y > resting[0].bottom());
    assert_close(resting[2].x + resting[2].w / 2.0, 960.0);
    assert!(resting[2].bottom() <= 1080.0);

    assert!(receiving[2].w > resting[2].w * 1.5);
    assert!(receiving[2].y > receiving[0].bottom());
    assert!(receiving[2].bottom() <= 1080.0);
  }

  #[test]
  fn grid_gives_a_widened_card_two_columns() {
    let metrics = metrics();
    let mut slots = slots(&metrics, 4);
    slots[2] = wide_slot(&metrics, 3.0);

    // Two normal cards, then the wide one doesn't fit on the first row.
    let cells = GridCells::new(&metrics, &slots, 3);
    let cell = |row, column, span| GridCell { row, column, span };
    assert_eq!(
      cells.cells,
      vec![cell(0, 0, 1), cell(0, 1, 1), cell(1, 0, 2), cell(1, 2, 1)]
    );

    let rects = grid_of(&metrics, &slots, 3);
    // The short first row is centered over the full second one.
    assert!(rects[2].x < rects[0].x);
    assert_close(rects[2].y, rects[3].y);
    assert!(rects[2].w > rects[0].w * 1.9);
    assert!(rects[3].x > rects[2].right());
    for rect in &rects {
      assert!(rect.x >= 0.0 && rect.right() <= 1920.0);
    }
  }

  #[test]
  fn grid_docks_the_special_card_below_its_rows() {
    let metrics = metrics();
    let mut slots = slots(&metrics, 4);
    slots[3] = CardSlot {
      is_special: true,
      ..wide_slot(&metrics, 3.0)
    };

    let cells = GridCells::new(&metrics, &slots, 5);
    assert_eq!(cells.columns, 3, "as many columns as normal cards");
    assert_eq!(cells.cells[3].row, 1, "a row below, for moving down");

    for is_receiving in [false, true] {
      let cards = grid(&metrics, &slots, &cells, is_receiving).cards;
      let rects = cards
        .iter()
        .map(|card| card.rect(&metrics))
        .collect::<Vec<_>>();
      assert!(rects[3].y > rects[0].bottom());
      assert_close(rects[3].x + rects[3].w / 2.0, 960.0);
      assert!(rects[3].x >= 0.0 && rects[3].right() <= 1920.0);
      assert!(rects[3].bottom() <= 1080.0);
    }
  }

  #[test]
  fn grid_rows_up_and_down_go_to_the_nearest_card() {
    let metrics = metrics();
    let mut slots = slots(&metrics, 5);
    slots[3] = wide_slot(&metrics, 3.0);
    slots[4].is_special = true;

    // Row 0: cards 0 1 2; row 1: wide card 3; row 2: special card 4.
    let cells = GridCells::new(&metrics, &slots, 3);
    assert_eq!(cells.vertical_neighbour(0, 1), Some(3));
    assert_eq!(cells.vertical_neighbour(2, 1), Some(3));
    assert_eq!(
      cells.vertical_neighbour(3, -1),
      Some(1),
      "over its middle"
    );
    assert_eq!(cells.vertical_neighbour(3, 1), Some(4));
    assert_eq!(cells.vertical_neighbour(4, 1), None);
    assert_eq!(cells.vertical_neighbour(0, -1), None);
  }

  #[test]
  fn grid_of_normal_cards_flows_as_before() {
    let metrics = metrics();
    let cells = GridCells::new(&metrics, &slots(&metrics, 7), 5);

    for (index, cell) in cells.cells.iter().enumerate() {
      assert_eq!(
        (cell.row, cell.column, cell.span),
        (index / 5, index % 5, 1)
      );
    }
  }
}
