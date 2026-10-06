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
  pub spacing: f32,
  pub scale_factor: f32,
  /// Size of the overview itself.
  pub view: (f32, f32),
}

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
      spacing: px(12.0),
      scale_factor,
      view: (width, height),
    }
  }

  /// Where windows are drawn within a card at scale 1.
  pub fn tile_area(&self) -> RectF {
    let width = self.width - 2.0 * self.padding;
    RectF::new(self.padding, self.header, width, width / self.aspect)
  }

  fn px(&self, length: f32) -> f32 {
    length * self.scale_factor
  }
}

/// Where a card sits: its center and scale.
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

/// One row of cards with the `selected` one in the middle at full size.
///
/// `zoomed` is for looking inside the selected workspace: it grows to
/// 1.5x and its neighbours sit far back.
pub(crate) fn carousel(
  metrics: &CardMetrics,
  count: usize,
  selected: usize,
  zoomed: bool,
) -> Targets {
  let selected = selected.min(count.saturating_sub(1));
  let scale_of = |index: usize| match (index == selected, zoomed) {
    (true, true) => 1.5,
    (true, false) => 1.0,
    (false, true) => 0.32,
    (false, false) => 0.66,
  };

  let mut starts = Vec::with_capacity(count);
  let mut x = 0.0;
  for index in 0..count {
    starts.push(x);
    x += metrics.width * scale_of(index) + metrics.spacing;
  }

  let selected_width = metrics.width * scale_of(selected);
  let offset = metrics.view.0 / 2.0
    - (starts.get(selected).copied().unwrap_or(0.0)
      + selected_width / 2.0);

  // The row grows with the zoom, so a busy workspace gets the room it
  // needs.
  let row_height = metrics.height * if zoomed { 1.5 } else { 1.02 };
  let top = (metrics.view.1 - row_height - metrics.px(30.0)) / 2.0;
  let cy = top + row_height / 2.0;

  Targets {
    cards: starts
      .iter()
      .enumerate()
      .map(|(index, start)| {
        let scale = scale_of(index);
        Placement {
          cx: start + metrics.width * scale / 2.0 + offset,
          cy,
          scale,
        }
      })
      .collect(),
    hint_y: top + row_height + metrics.px(16.0),
  }
}

/// Every card at once, `columns` per row, as large as fits.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn grid(
  metrics: &CardMetrics,
  count: usize,
  columns: usize,
) -> Targets {
  let columns = columns.clamp(1, count.max(1));
  let rows = count.div_ceil(columns).max(1);
  let gap = metrics.px(14.0);
  let (columns_f, rows_f) = (columns as f32, rows as f32);

  // Room is left for the hint line and a carried window parked at the
  // bottom.
  let available_width = metrics.view.0 - metrics.px(120.0);
  let available_height = metrics.view.1 - metrics.px(320.0);
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
  let top = (metrics.view.1 - total_height - metrics.px(30.0)) / 2.0;

  Targets {
    cards: (0..count)
      .map(|index| {
        let (row, column) = (index / columns, index % columns);
        // A short last row is centered rather than hanging off the left.
        let in_row = columns.min(count - row * columns) as f32;
        let row_left = left + (columns_f - in_row) * (width + gap) / 2.0;

        Placement {
          cx: row_left + column as f32 * (width + gap) + width / 2.0,
          cy: top + row as f32 * (height + gap) + height / 2.0,
          scale,
        }
      })
      .collect(),
    hint_y: top + total_height + metrics.px(16.0),
  }
}

/// A window's place within a card at scale 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Tile {
  pub rect: RectF,

  /// The part of the window the tile shows, as fractions of its size;
  /// less than all of it when it reaches past its workspace.
  pub crop: RectF,
}

/// Where a window at `window` (screen coordinates) goes on the card of a
/// workspace covering `area`: where it actually sits, scaled down. `None`
/// for a window entirely outside the area.
pub(crate) fn tile(
  metrics: &CardMetrics,
  area: &RectF,
  window: &RectF,
) -> Option<Tile> {
  if window.w <= 0.0 || window.h <= 0.0 || area.w <= 0.0 || area.h <= 0.0 {
    return None;
  }

  let visible = window.intersect(area)?;
  let tiles = metrics.tile_area();
  let (scale_x, scale_y) = (tiles.w / area.w, tiles.h / area.h);

  let rect = RectF::new(
    tiles.x + (visible.x - area.x) * scale_x,
    tiles.y + (visible.y - area.y) * scale_y,
    visible.w * scale_x,
    visible.h * scale_y,
  );

  Some(Tile {
    rect,
    crop: RectF::new(
      (visible.x - window.x) / window.w,
      (visible.y - window.y) / window.h,
      visible.w / window.w,
      visible.h / window.h,
    ),
  })
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

/// Size of a pinned window's preview `width` wide, for a window of size
/// `source`: its shape, kept between a wide strip and a tall card.
pub(crate) fn pin_size(width: f32, source: (f32, f32)) -> (f32, f32) {
  let aspect = source.0 / source.1;
  let height = if aspect.is_finite() && aspect > 0.0 {
    width / aspect
  } else {
    width * 0.625
  };

  (width, height.clamp(width * 0.4, width * 1.25))
}

/// Top-left corner closest to `position` that keeps a `size` rect inside
/// `area`, or at its top-left corner when it doesn't fit.
pub(crate) fn keep_inside(
  position: (f32, f32),
  size: (f32, f32),
  area: &RectF,
) -> (f32, f32) {
  (
    position.0.min(area.right() - size.0).max(area.x),
    position.1.min(area.bottom() - size.1).max(area.y),
  )
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
    carousel, cover_crop, ease_out_cubic, grid, keep_inside, pin_size,
    tile, Camera, CardMetrics, RectF, Spring, Zoom,
  };

  fn metrics() -> CardMetrics {
    CardMetrics::new(1920.0, 1040.0, 1.0)
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
    let targets = carousel(&metrics, 5, 2, false);

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
    let targets = carousel(&metrics, 3, 0, true);

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
    let targets = grid(&metrics, 7, 5);
    let rects = targets
      .cards
      .iter()
      .map(|card| card.rect(&metrics))
      .collect::<Vec<_>>();

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
    let targets = grid(&metrics(), 2, 5);
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
    let card = carousel(&metrics, 3, 1, false).cards[1];
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
  fn pin_size_follows_the_window_within_limits() {
    assert_eq!(pin_size(320.0, (1600.0, 1000.0)), (320.0, 200.0));
    assert_eq!(pin_size(320.0, (400.0, 2000.0)), (320.0, 400.0), "tall");
    assert_eq!(pin_size(320.0, (4000.0, 200.0)), (320.0, 128.0), "wide");
    assert_eq!(pin_size(320.0, (0.0, 0.0)), (320.0, 200.0));
  }

  #[test]
  fn keep_inside_pulls_a_rect_back_onto_the_area() {
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let size = (320.0, 200.0);

    assert_eq!(keep_inside((100.0, 100.0), size, &area), (100.0, 100.0));
    assert_eq!(
      keep_inside((1800.0, 1000.0), size, &area),
      (1600.0, 840.0)
    );
    assert_eq!(keep_inside((-50.0, -50.0), size, &area), (0.0, 0.0));
    assert_eq!(
      keep_inside((10.0, 10.0), (4000.0, 4000.0), &area),
      (0.0, 0.0),
      "too big to fit"
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
}
