//! Geometry of the window overview's cards: a card per workspace, laid
//! out as a row of miniatures over the shown workspace, or as a stack of
//! strips, each showing its windows where they really sit, scaled down.
//!
//! Values are physical pixels relative to the overview's top-left corner,
//! kept fractional so animated positions don't snap to whole pixels until
//! drawn.

use std::time::Instant;

use crate::{OverviewCurve, OverviewLayoutMode};

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

  pub fn center(&self) -> (f32, f32) {
    (self.x + self.w / 2.0, self.y + self.h / 2.0)
  }

  /// This rect moved by (`dx`, `dy`).
  pub fn translate(&self, dx: f32, dy: f32) -> Self {
    Self::new(self.x + dx, self.y + dy, self.w, self.h)
  }

  /// The rect `progress` of the way from this one to `to`.
  pub fn lerp(&self, to: &Self, progress: f32) -> Self {
    let lerp = |a: f32, b: f32| a + (b - a) * progress;
    Self::new(
      lerp(self.x, to.x),
      lerp(self.y, to.y),
      lerp(self.w, to.w),
      lerp(self.h, to.h),
    )
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

  /// This rect with its coordinates multiplied by `factor`.
  pub fn scaled(&self, factor: f32) -> Self {
    Self::new(
      self.x * factor,
      self.y * factor,
      self.w * factor,
      self.h * factor,
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
  /// Most normal card widths a card is widened to; a strip wider still is
  /// letterboxed by its `CardView`.
  max_widths: f32,
  pub spacing: f32,
  pub scale_factor: f32,
  /// How much larger than their normal size a card's name, captions and
  /// borders are drawn, for cards meant to be shown large.
  pub chrome: f32,
  pub look: CardLook,
  /// Size of the overview itself.
  pub view: (f32, f32),
}

/// What a card shows besides its windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardLook {
  /// A header with the workspace's name, and each window's title.
  Classic,

  /// Just the windows on a rounded panel; the name shows beside it.
  Clean,
}

/// How a layout has its cards drawn.
struct DrawnSize {
  /// Width as a part of the overview's width.
  part: f32,
  chrome: f32,
  max_widths: f32,
  /// Narrowest width, in logical pixels.
  min_width: f32,
  look: CardLook,
}

/// Part of the overview's width a card drawn at most 900 logical pixels
/// wide takes; wider layouts' cards may be drawn that much wider.
const REFERENCE_CARD_WIDTH: f32 = 0.26;

/// Part of the overview's width the stack shows a monitor's width of a
/// workspace at.
const STACK_CARD_WIDTH: f32 = 0.45;

/// Most monitor widths of a strip the stack shows at full length; a
/// strip wider still is shrunk to fit that, as its `CardView` does. Kept
/// low as a card's picture is drawn as wide as its strip.
const STACK_MAX_WIDTHS: f32 = 4.0;

/// How a card is drawn for an overview opened as `layout`.
///
/// Cards are drawn at the size the layout mostly shows them at, since
/// DWM only scales the drawing: shown much larger, its text would blur.
fn drawn_size(layout: OverviewLayoutMode) -> DrawnSize {
  let (part, chrome, max_widths, min_width, look) = match layout {
    OverviewLayoutMode::Activities => {
      (0.12, 0.8, MINIATURE_WIDTHS, 120.0, CardLook::Clean)
    }
    OverviewLayoutMode::Stack => (
      STACK_CARD_WIDTH,
      1.1,
      STACK_MAX_WIDTHS,
      380.0,
      CardLook::Classic,
    ),
  };

  DrawnSize {
    part,
    chrome,
    max_widths,
    min_width,
    look,
  }
}

impl CardMetrics {
  /// Cards for an overview of `width` x `height` covering a monitor with
  /// that same area, opened as `layout`, in a sensible range of sizes.
  pub fn new(
    width: f32,
    height: f32,
    scale_factor: f32,
    layout: OverviewLayoutMode,
  ) -> Self {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
      scale_factor
    } else {
      1.0
    };
    let px = |length: f32| length * scale_factor;

    let DrawnSize {
      part,
      chrome,
      max_widths,
      min_width,
      look,
    } = drawn_size(layout);
    let max_width = px(900.0) * part / REFERENCE_CARD_WIDTH;
    let card_width =
      (width * part).min(max_width).max(px(min_width)).min(width);
    let padding = px(9.0) * chrome;
    let header = match look {
      CardLook::Classic => px(28.0) * chrome,
      CardLook::Clean => padding,
    };
    let aspect = width / height;

    Self {
      width: card_width,
      height: header + (card_width - 2.0 * padding) / aspect + padding,
      header,
      padding,
      aspect,
      stretch: 1.0,
      max_widths,
      spacing: px(12.0),
      scale_factor,
      chrome,
      look,
      view: (width, height),
    }
  }

  /// A card showing `content` of the monitor's `area`: as tall as a
  /// normal card, and as much wider as `content` is wider than `area`, up
  /// to `max_widths` normal cards. Content wider still is letterboxed by
  /// its `CardView`.
  pub fn for_content(&self, area: &RectF, content: &RectF) -> Self {
    let content = content.union(area);
    let ratio = (content.w / content.h) / (area.w / area.h);

    // Exactly `self` for content that fits, so such cards are unchanged.
    if !ratio.is_finite() || ratio <= 1.0 {
      return *self;
    }

    let tiles = self.width - 2.0 * self.padding;
    let max_stretch =
      (self.max_widths * self.width - 2.0 * self.padding) / tiles;
    let stretch = ratio.min(max_stretch);

    Self {
      width: tiles * stretch + 2.0 * self.padding,
      stretch,
      ..*self
    }
  }

  /// These metrics for a card drawn `factor` times its size.
  pub fn scaled(&self, factor: f32) -> Self {
    Self {
      width: self.width * factor,
      height: self.height * factor,
      header: self.header * factor,
      padding: self.padding * factor,
      spacing: self.spacing * factor,
      scale_factor: self.scale_factor * factor,
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

  /// `length` logical pixels.
  pub fn px(&self, length: f32) -> f32 {
    length * self.scale_factor
  }

  /// Top edge of the search field when it sits at the bottom of the
  /// overview.
  pub fn bottom_search_y(&self) -> f32 {
    self.view.1 - self.px(SEARCH_HEIGHT + 10.0)
  }
}

/// Height of the search field line, in logical pixels.
pub(crate) const SEARCH_HEIGHT: f32 = 36.0;

/// Where a card sits: its center, scale and opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
  pub cx: f32,
  pub cy: f32,
  pub scale: f32,

  /// Lowered for cards out of the way of the selected one.
  pub opacity: f32,
}

impl Placement {
  pub fn new(cx: f32, cy: f32, scale: f32) -> Self {
    Self {
      cx,
      cy,
      scale,
      opacity: 1.0,
    }
  }

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

/// Where every card goes, and the search field's line under them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Targets {
  pub cards: Vec<Placement>,
  pub search_y: f32,
}

/// What the layouts need of a card.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardSlot {
  /// Its width at scale 1.
  pub width: f32,

  /// How far across the card, at scale 1, the middle of what its monitor
  /// shows now is: off its middle for a scrolling workspace whose strip
  /// reaches further one way than the other.
  pub view_center: f32,

  /// Span across the card, at scale 1, to keep on screen, e.g. that of
  /// the window being picked in it.
  pub keep: Option<(f32, f32)>,
}

impl CardSlot {
  pub fn new(metrics: &CardMetrics) -> Self {
    Self {
      width: metrics.width,
      view_center: metrics.width / 2.0,
      keep: None,
    }
  }
}

/// Height of the name shown under (or over) a card that has no header,
/// and its gap to the card, in logical pixels.
pub(crate) const LABEL_HEIGHT: f32 = 20.0;
pub(crate) const LABEL_GAP: f32 = 6.0;

/// Most normal card widths a miniature is widened to.
const MINIATURE_WIDTHS: f32 = 2.0;

/// Every card in miniature in a row along the top, their names under
/// them, and below them the scene: where the selected workspace shows
/// large. Returns the cards' targets and the scene.
pub(crate) fn activities(
  metrics: &CardMetrics,
  slots: &[CardSlot],
) -> (Targets, RectF) {
  let (view_width, view_height) = metrics.view;
  let (margin, gap, top) =
    (metrics.px(48.0), metrics.px(16.0), metrics.px(18.0));
  let label = metrics.px(LABEL_GAP + LABEL_HEIGHT);

  // As tall as a part of the overview, shrunk for a long row to fit.
  let height = (view_height * 0.12)
    .clamp(metrics.px(64.0), metrics.px(170.0))
    .min(metrics.height);
  let widths = slots.iter().map(|slot| slot.width).sum::<f32>();
  #[allow(clippy::cast_precision_loss)]
  let gaps = gap * slots.len().saturating_sub(1) as f32;
  let scale = (height / metrics.height)
    .min((view_width - 2.0 * margin - gaps) / widths.max(1.0))
    .max(0.01);

  let row_width = widths * scale + gaps;
  let cy = top + metrics.height * scale / 2.0;
  let mut x = (view_width - row_width) / 2.0;
  let cards = slots
    .iter()
    .map(|slot| {
      let width = slot.width * scale;
      let placement = Placement::new(x + width / 2.0, cy, scale);
      x += width + gap;
      placement
    })
    .collect();

  let search_y = metrics.bottom_search_y();
  let scene_top = top + metrics.height * scale + label + metrics.px(22.0);
  let scene = RectF::new(
    margin,
    scene_top,
    view_width - 2.0 * margin,
    (search_y - metrics.px(16.0) - scene_top).max(0.0),
  );

  (Targets { cards, search_y }, scene)
}

/// Space kept free beside a panned strip's picked window, and between the
/// stack's cards, in logical pixels.
const STACK_MARGIN: f32 = 48.0;
const STACK_GAP: f32 = 28.0;

/// Every card at scale 1 in a column, the `selected` one in the middle of
/// the room above the search line. Each card lines up the middle of what
/// its monitor shows with the middle of the overview, so a strip reaches
/// past either side as far as its columns do.
///
/// The selected card is panned `pan` further along its strip, in card
/// units at scale 1, but only as far as keeps the overview's middle on
/// it, and then as little as keeps its `keep` span in view. Returns the
/// targets and the pan it ends up with.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn stack(
  metrics: &CardMetrics,
  slots: &[CardSlot],
  selected: usize,
  pan: f32,
) -> (Targets, f32) {
  let view_width = metrics.view.0;
  let margin = metrics.px(STACK_MARGIN);
  let search_y = metrics.bottom_search_y();
  let middle = (view_width / 2.0, (search_y - metrics.px(16.0)) / 2.0);
  let step = metrics.height + metrics.px(STACK_GAP);
  let mut panned = 0.0;

  let cards = slots
    .iter()
    .enumerate()
    .map(|(index, slot)| {
      let resting = middle.0 - slot.view_center;
      let mut left = resting;

      if index == selected {
        left -=
          pan.clamp(-slot.view_center, slot.width - slot.view_center);

        if let Some((from, to)) = slot.keep {
          let (low, high) = (margin, view_width - margin);
          if to - from > high - low || left + from < low {
            left = low - from;
          } else if left + to > high {
            left = high - to;
          }
        }
        panned = resting - left;
      }

      let offset = index as f32 - selected as f32;
      Placement::new(
        left + slot.width / 2.0,
        middle.1 + offset * step,
        1.0,
      )
    })
    .collect();

  (Targets { cards, search_y }, panned)
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

  /// `content` (screen coordinates) shown as large as fits in `dest`,
  /// centered, keeping its shape.
  pub fn fitted(content: &RectF, dest: &RectF) -> Self {
    let scale = if content.w > 0.0 && content.h > 0.0 {
      (dest.w / content.w).min(dest.h / content.h).max(0.0)
    } else {
      0.0
    };

    Self {
      content: *content,
      origin: (
        dest.x + (dest.w - content.w * scale) / 2.0,
        dest.y + (dest.h - content.h * scale) / 2.0,
      ),
      scale_x: scale,
      scale_y: scale,
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

  /// The screen point that point (`x`, `y`) of the card shows: where on
  /// screen its windows sit. `None` for a card showing nothing.
  #[allow(clippy::cast_possible_truncation)]
  pub fn unmap(&self, x: f32, y: f32) -> Option<(i32, i32)> {
    let is_shown = self.scale_x > 0.0
      && self.scale_y > 0.0
      && self.scale_x.is_finite()
      && self.scale_y.is_finite();

    is_shown.then(|| {
      (
        (self.content.x + (x - self.origin.0) / self.scale_x).round()
          as i32,
        (self.content.y + (y - self.origin.1) / self.scale_y).round()
          as i32,
      )
    })
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

  /// The zoom at eased `progress` from 0 to 1, or a little past 1 while
  /// it overshoots.
  pub fn zoom(from: &RectF, to: &RectF, progress: f32) -> Self {
    if from.w <= 0.0 || from.h <= 0.0 {
      return Self::IDENTITY;
    }

    let progress = progress.max(0.0);
    let lerp = |a: f32, b: f32| a + (b - a) * progress;
    let (start_x, start_y) = (to.w / from.w, to.h / from.h);

    Self {
      scale_x: lerp(start_x, 1.0),
      scale_y: lerp(start_y, 1.0),
      offset_x: lerp(to.x - from.x * start_x, 0.0),
      offset_y: lerp(to.y - from.y * start_y, 0.0),
    }
  }

  /// Everything grown `by` (e.g. `0.05` for 5% larger) about the middle
  /// of `view`.
  pub fn grown(view: &RectF, by: f32) -> Self {
    let scale = 1.0 + by;
    let (cx, cy) = view.center();
    Self {
      scale_x: scale,
      scale_y: scale,
      offset_x: cx * (1.0 - scale),
      offset_y: cy * (1.0 - scale),
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

  /// Openness `elapsed_ms` in, paced by `curve`. A curve overshooting
  /// takes it past where it heads for a moment.
  pub fn openness(&self, elapsed_ms: f32, curve: &OverviewCurve) -> f32 {
    let to = if self.is_closing { 0.0 } else { 1.0 };
    self.from + (to - self.from) * curve.at(self.progress(elapsed_ms))
  }

  pub fn is_finished(&self, elapsed_ms: f32) -> bool {
    self.progress(elapsed_ms) >= 1.0
  }

  /// How long it takes from start to finish.
  pub fn duration(&self) -> std::time::Duration {
    std::time::Duration::from_secs_f32(self.duration_ms.max(0.0) / 1000.0)
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

/// Like `ease_out_cubic`, but running past 1 before settling back, as a
/// `Spring` with the same `bounce` does; `ease_out_cubic` for `0`.
pub(crate) fn ease_out_back(progress: f32, bounce: f32) -> f32 {
  // Peaks at `4 c^3 / (27 (c + 1)^2)` past 1: about 4% at a bounce of
  // 0.25.
  let c = 4.0 * bounce.clamp(0.0, MAX_BOUNCE);
  let t = progress.clamp(0.0, 1.0) - 1.0;
  1.0 + t * t * ((c + 1.0) * t + c)
}

/// Openness from which the overview's chrome starts to show.
const CHROME_FROM: f32 = 0.8;

/// How strongly the overview's chrome (hints, names, highlights) shows at
/// `openness`: only once the zoom has nearly settled, so an open shows
/// the windows shrinking first, and gone as soon as a close starts.
pub(crate) fn chrome_opacity(openness: f32) -> f32 {
  let progress =
    ((openness - CHROME_FROM) / (1.0 - CHROME_FROM)).clamp(0.0, 1.0);
  progress * progress * (3.0 - 2.0 * progress)
}

/// How much larger, relative to its shorter side, a mark starts out than
/// what it settles onto.
const SETTLE_GROWTH: f32 = 0.04;

/// Where a highlight shows: on what it marks, fading in there (settling
/// onto it from a little larger) whenever that changes rather than
/// travelling over from where it was, and fading out where it is.
pub(crate) struct Mark<K> {
  target: Option<K>,

  /// How long it takes to settle and about how long to fade.
  duration_ms: f32,

  /// Where it settles from, with its corners' scale, and since when.
  from: (RectF, f32),
  started: Instant,

  shown: (RectF, f32),
  alpha: Spring,
}

impl<K: Copy + PartialEq> Mark<K> {
  /// One that settles and fades over `duration_ms`; at once for `0`.
  pub fn new(duration_ms: f32) -> Self {
    Self {
      target: None,
      duration_ms,
      from: (RectF::default(), 1.0),
      started: Instant::now(),
      shown: (RectF::default(), 1.0),
      alpha: Spring::new(0.0, duration_ms, 0.002),
    }
  }

  /// Settles and fades over `duration_ms` from now on.
  pub fn retune(&mut self, duration_ms: f32) {
    self.duration_ms = duration_ms;
    self.alpha.retune(duration_ms);
  }

  /// Follows `target`: the key of what to mark, where that is now and its
  /// corners' scale. `None` fades it out where it is. Returns whether it
  /// is still moving.
  pub fn follow(
    &mut self,
    target: Option<(K, RectF, f32)>,
    now: Instant,
  ) -> bool {
    let Some((key, rect, scale)) = target else {
      self.target = None;
      self.fade_to(0.0);
      return !self.alpha.is_settled();
    };

    if self.target != Some(key) {
      self.from = if self.duration_ms > 0.0 {
        self.alpha.snap(0.0);
        (rect.inset(-rect.w.min(rect.h) * SETTLE_GROWTH), scale)
      } else {
        (rect, scale)
      };
      self.started = now;
      self.target = Some(key);
    }
    self.fade_to(1.0);

    let elapsed = now.duration_since(self.started).as_secs_f32() * 1000.0;
    let progress = if self.duration_ms > 0.0 {
      (elapsed / self.duration_ms).min(1.0)
    } else {
      1.0
    };
    // Onto where its target is now, which may itself be moving.
    let eased = ease_out_cubic(progress);
    self.shown = (
      self.from.0.lerp(&rect, eased),
      self.from.1 + (scale - self.from.1) * eased,
    );

    progress < 1.0 || !self.alpha.is_settled()
  }

  fn fade_to(&mut self, alpha: f32) {
    if self.duration_ms > 0.0 {
      self.alpha.target = alpha;
    } else {
      // Snapped here: its fade only steps before it follows, which would
      // land it a frame late.
      self.alpha.snap(alpha);
    }
  }

  /// Advances its fade by `dt` seconds. Returns whether it still fades.
  pub fn step(&mut self, dt: f32) -> bool {
    self.alpha.step(dt)
  }

  /// Where it shows, its corners' scale and how strongly, if at all.
  pub fn shown(&self) -> Option<(RectF, f32, f32)> {
    (self.alpha.value > 0.002).then_some((
      self.shown.0,
      self.shown.1,
      self.alpha.value,
    ))
  }

  /// Gone at once, e.g. on close.
  pub fn reset(&mut self) {
    self.target = None;
    self.alpha.snap(0.0);
  }
}

/// A rect whose every side eases toward its target, like a `Spring`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpringRect {
  x: Spring,
  y: Spring,
  width: Spring,
  height: Spring,
}

impl SpringRect {
  /// One resting at `rect` that settles within about `settle_ms`.
  pub fn new(rect: RectF, settle_ms: f32) -> Self {
    Self {
      x: Spring::new(rect.x, settle_ms, 0.25),
      y: Spring::new(rect.y, settle_ms, 0.25),
      width: Spring::new(rect.w, settle_ms, 0.25),
      height: Spring::new(rect.h, settle_ms, 0.25),
    }
  }

  /// Settles within about `settle_ms`, overshooting by `bounce`, from
  /// now on.
  pub fn retune(&mut self, settle_ms: f32, bounce: f32) {
    for spring in
      [&mut self.x, &mut self.y, &mut self.width, &mut self.height]
    {
      spring.retune(settle_ms);
      spring.set_bounce(bounce);
    }
  }

  /// Where it is now.
  pub fn value(&self) -> RectF {
    RectF::new(
      self.x.value,
      self.y.value,
      self.width.value,
      self.height.value,
    )
  }

  /// Where it heads.
  pub fn target(&self) -> RectF {
    RectF::new(
      self.x.target,
      self.y.target,
      self.width.target,
      self.height.target,
    )
  }

  /// Points it at `rect`; there at once if `snap`.
  pub fn retarget(&mut self, rect: &RectF, snap: bool) {
    for (spring, value) in [
      (&mut self.x, rect.x),
      (&mut self.y, rect.y),
      (&mut self.width, rect.w),
      (&mut self.height, rect.h),
    ] {
      if snap {
        spring.snap(value);
      } else {
        spring.target = value;
      }
    }
  }

  /// Stops it where it is now.
  pub fn hold(&mut self) {
    self.retarget(&self.value(), true);
  }

  /// Knocks it about `dx` and `dy` off where it heads, from where it
  /// springs back.
  pub fn kick(&mut self, dx: f32, dy: f32) {
    self.x.kick(dx);
    self.y.kick(dy);
  }

  /// Advances by `dt` seconds. Returns whether it is still moving.
  pub fn step(&mut self, dt: f32) -> bool {
    [&mut self.x, &mut self.y, &mut self.width, &mut self.height]
      .into_iter()
      .fold(false, |is_moving, spring| spring.step(dt) | is_moving)
  }
}

/// Most bounce a `Spring` takes: past it, things wobble more than they
/// settle.
pub(crate) const MAX_BOUNCE: f32 = 0.8;

/// A number that eases toward its target instead of jumping to it.
///
/// Critically damped unless given a bounce, which lets it overshoot its
/// target and swing back before settling. Retargeting it mid-flight keeps
/// its velocity rather than restarting the curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Spring {
  pub value: f32,
  pub target: f32,
  velocity: f32,
  /// Rate, per second, at which its distance to its target dies away
  /// (its natural frequency times its damping ratio), the same for any
  /// bounce; `None` jumps to its target on the next step.
  decay: Option<f32>,
  /// Its damping ratio, from 1 (critically damped) down to
  /// `1 - MAX_BOUNCE`.
  damping: f32,
  /// How close to its target, in its own units, it counts as settled.
  epsilon: f32,
}

impl Spring {
  /// A critically damped spring resting at `value` that settles within
  /// about `settle_ms`; on its next step for `0`.
  pub fn new(value: f32, settle_ms: f32, epsilon: f32) -> Self {
    Self {
      value,
      target: value,
      velocity: 0.0,
      decay: Self::decay(settle_ms),
      damping: 1.0,
      epsilon,
    }
  }

  /// The same spring, overshooting by `bounce`.
  #[must_use]
  pub fn with_bounce(mut self, bounce: f32) -> Self {
    self.set_bounce(bounce);
    self
  }

  fn decay(settle_ms: f32) -> Option<f32> {
    (settle_ms > 0.0).then(|| 4.6 / (settle_ms / 1000.0))
  }

  /// Settles within about `settle_ms` from now on, carrying on from where
  /// it is and as fast.
  pub fn retune(&mut self, settle_ms: f32) {
    self.decay = Self::decay(settle_ms);
  }

  /// Overshoots by `bounce` from now on: from `0`, settling without
  /// overshooting, up to `MAX_BOUNCE`. Settles about as fast either way.
  pub fn set_bounce(&mut self, bounce: f32) {
    self.damping = 1.0 - bounce.clamp(0.0, MAX_BOUNCE);
  }

  /// Knocks it about `distance` off its target at the farthest, from
  /// where it springs back.
  pub fn kick(&mut self, distance: f32) {
    let Some(decay) = self.decay else {
      return;
    };

    // Set off from rest at its target with velocity `v`, it is farthest
    // after `t = atan(w / decay) / w`, at `v e^(-decay t) sin(w t) / w`,
    // which tends to `v / (e decay)` as `w` goes to 0.
    let peak_per_velocity = match self.ringing(decay) {
      Some(ringing) => {
        let time = (ringing / decay).atan() / ringing;
        (-decay * time).exp() * (ringing * time).sin() / ringing
      }
      None => 1.0 / (std::f32::consts::E * decay),
    };
    self.velocity += distance / peak_per_velocity;
  }

  /// Frequency, in radians per second, at which it swings about its
  /// target; `None` while critically damped.
  fn ringing(&self, decay: f32) -> Option<f32> {
    (self.damping < 1.0).then(|| {
      decay * (1.0 - self.damping * self.damping).sqrt() / self.damping
    })
  }

  pub fn snap(&mut self, value: f32) {
    self.value = value;
    self.target = value;
    self.velocity = 0.0;
  }

  /// Scales where it is, where it heads and how fast, e.g. to the same
  /// size in other units.
  pub fn rescale(&mut self, factor: f32) {
    self.value *= factor;
    self.target *= factor;
    self.velocity *= factor;
  }

  /// How fast it moves, in its units per second.
  pub fn velocity(&self) -> f32 {
    self.velocity
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

    let Some(decay) = self.decay else {
      self.snap(self.target);
      return true;
    };

    // The exact solution rather than an integration step, which a short
    // settle time or a long frame makes blow up.
    let offset = self.value - self.target;
    let envelope = (-decay * dt).exp();

    if let Some(ringing) = self.ringing(decay) {
      let swing = (self.velocity + decay * offset) / ringing;
      let (sin, cos) = (ringing * dt).sin_cos();
      self.value = self.target + (offset * cos + swing * sin) * envelope;
      self.velocity = ((swing * ringing - decay * offset) * cos
        - (offset * ringing + decay * swing) * sin)
        * envelope;
    } else {
      let pull = self.velocity + decay * offset;
      self.value = self.target + (offset + pull * dt) * envelope;
      self.velocity = (self.velocity - decay * pull * dt) * envelope;
    }

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
  use std::time::{Duration, Instant};

  use super::{
    activities, chrome_opacity, cover_crop, ease_out_back, ease_out_cubic,
    stack, Camera, CardLook, CardMetrics, CardSlot, CardView, Mark,
    Placement, RectF, Spring, SpringRect, Tile, Zoom, STACK_MAX_WIDTHS,
  };
  use crate::{OverviewCurve, OverviewLayoutMode};

  /// Cards drawn for the stack.
  fn metrics() -> CardMetrics {
    metrics_for(OverviewLayoutMode::Stack)
  }

  fn metrics_for(layout: OverviewLayoutMode) -> CardMetrics {
    CardMetrics::new(1920.0, 1040.0, 1.0, layout)
  }

  /// `count` normal cards.
  fn slots(metrics: &CardMetrics, count: usize) -> Vec<CardSlot> {
    vec![CardSlot::new(metrics); count]
  }

  /// The card of a scrolling workspace whose strip is `widths` monitors
  /// wide.
  fn wide_slot(metrics: &CardMetrics, widths: f32) -> CardSlot {
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let strip = RectF::new(0.0, 0.0, 1920.0 * widths, 1040.0);
    CardSlot::new(&metrics.for_content(&area, &strip))
  }

  /// `metrics` of a card `width` wide.
  fn with_width(metrics: &CardMetrics, width: f32) -> CardMetrics {
    CardMetrics { width, ..*metrics }
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

    assert_close(metrics.width, 1920.0 * 0.45);
    assert_close(tiles.w / tiles.h, 1920.0 / 1040.0);
    assert_close(metrics.height, tiles.bottom() + metrics.padding);
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
    let card = Placement::new(960.0, 400.0, 1.0);
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
  fn a_grown_camera_scales_about_the_middle() {
    let view = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let camera = Camera::grown(&view, 0.1);

    assert_rect_close(
      &camera.apply(&view),
      &RectF::new(-96.0, -52.0, 2112.0, 1144.0),
    );
    let (x, y) = camera.unapply(960.0, 520.0);
    assert_close(x, 960.0);
    assert_close(y, 520.0);
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
    let curve = OverviewCurve::default();
    let open = Zoom::open(2, 250);
    assert_close(open.openness(0.0, &curve), 0.0);
    assert!(open.openness(100.0, &curve) > 0.85, "fast start");
    assert!(
      (0..=250_u16).all(|ms| open.openness(f32::from(ms), &curve) <= 1.0)
    );
    assert_close(open.openness(250.0, &curve), 1.0);
    assert!(open.is_finished(250.0) && !open.is_finished(249.0));

    let close = Zoom::close(2, 1.0, 200);
    assert!(close.is_closing);
    assert_close(close.openness(0.0, &curve), 1.0);
    assert_close(close.openness(200.0, &curve), 0.0);

    // Paced by the curve given, e.g. a straight line.
    let line = OverviewCurve::new(vec![0.0, 1.0]);
    assert_close(open.openness(125.0, &line), 0.5);
  }

  #[test]
  fn zoom_closes_a_half_open_overview_in_half_the_time() {
    let curve = OverviewCurve::default();
    let close = Zoom::close(0, 0.5, 200);
    assert_close(close.openness(0.0, &curve), 0.5);
    assert!(close.is_finished(100.0));
    assert_close(close.openness(100.0, &curve), 0.0);

    let instant = Zoom::close(0, 1.0, 0);
    assert!(instant.is_finished(0.0));
    assert_close(instant.openness(0.0, &curve), 0.0);
  }

  #[test]
  fn view_unmaps_a_card_point_back_onto_the_screen() {
    let metrics = metrics();
    let area = RectF::new(1920.0, 40.0, 1920.0, 1040.0);
    let view = CardView::new(&metrics, &area, &area);
    let window = RectF::new(2420.0, 340.0, 600.0, 400.0);

    let (x, y) = view.map(&window).center();
    assert_eq!(view.unmap(x, y), Some((2720, 540)));

    let nothing = CardView::fitted(&area, &RectF::default());
    assert_eq!(nothing.unmap(0.0, 0.0), None);
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
    let card = Placement::new(960.0, 400.0, 1.0);

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
  fn chrome_shows_only_as_the_zoom_settles() {
    assert_close(chrome_opacity(0.0), 0.0);
    assert_close(chrome_opacity(0.8), 0.0);
    assert_close(chrome_opacity(1.0), 1.0);

    let steps =
      (80..=100_u8).map(|step| chrome_opacity(f32::from(step) / 100.0));
    let rising = steps.clone().zip(steps.skip(1)).all(|(a, b)| b >= a);
    assert!(rising, "fades in without dipping");
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
  fn easing_back_overshoots_by_its_bounce() {
    for progress in [0.0, 0.3, 0.7, 1.0] {
      assert_close(ease_out_back(progress, 0.0), ease_out_cubic(progress));
    }
    assert_close(ease_out_back(0.0, 0.25), 0.0);
    assert_close(ease_out_back(1.0, 0.25), 1.0);
    assert_close(ease_out_back(2.0, 0.25), 1.0);

    let peak = |bounce: f32| {
      (0..=100_u8)
        .map(|step| ease_out_back(f32::from(step) / 100.0, bounce))
        .fold(0.0_f32, f32::max)
    };
    assert!(peak(0.25) > 1.02);
    assert!(peak(0.5) > peak(0.25));
  }

  #[test]
  fn bouncy_spring_overshoots_then_settles_about_as_fast() {
    let mut spring = Spring::new(0.0, 260.0, 0.25).with_bounce(0.3);
    spring.target = 100.0;

    let mut frames = 0;
    let mut farthest = 0.0_f32;
    while spring.step(1.0 / 60.0) {
      farthest = farthest.max(spring.value);
      frames += 1;
      assert!(frames < 120, "spring never settled");
    }

    assert!(farthest > 103.0 && farthest < 115.0, "{farthest}");
    assert_close(spring.value, 100.0);
    assert!(!spring.step(1.0 / 60.0));
  }

  /// Bouncy or not, a short settle time over long frames.
  #[test]
  fn bouncy_spring_settles_at_any_frame_rate() {
    for dt in [1.0 / 30.0, 1.0 / 60.0, 1.0 / 240.0] {
      let mut spring = Spring::new(0.0, 40.0, 0.25).with_bounce(0.8);
      spring.target = 100.0;

      let mut frames = 0;
      while spring.step(dt) {
        assert!(spring.value.abs() < 300.0, "blew up");
        frames += 1;
        assert!(frames < 240, "spring never settled");
      }
      assert_close(spring.value, 100.0);
    }
  }

  #[test]
  fn kicked_spring_swings_out_about_as_far_and_back() {
    for bounce in [0.0, 0.25, 0.6] {
      let mut spring = Spring::new(50.0, 300.0, 0.25).with_bounce(bounce);
      spring.kick(20.0);

      let mut farthest = 0.0_f32;
      let mut frames = 0;
      while spring.step(1.0 / 240.0) {
        farthest = farthest.max(spring.value - 50.0);
        frames += 1;
        assert!(frames < 480, "spring never settled");
      }

      assert!((farthest - 20.0).abs() < 0.5, "{bounce}: {farthest}");
      assert_close(spring.value, 50.0);
    }

    let mut instant = Spring::new(50.0, 0.0, 0.25);
    instant.kick(20.0);
    assert!(!instant.step(1.0 / 60.0), "an instant spring isn't kicked");
  }

  #[test]
  fn instant_spring_jumps_to_its_target() {
    let mut spring = Spring::new(0.0, 0.0, 0.25);
    spring.target = 100.0;

    assert!(spring.step(1.0 / 60.0));
    assert_close(spring.value, 100.0);
    assert!(!spring.step(1.0 / 60.0));
  }

  /// Even a short settle time over long frames, which an integration step
  /// made blow up.
  #[test]
  fn spring_settles_at_any_frame_rate() {
    for dt in [1.0 / 30.0, 1.0 / 60.0, 1.0 / 240.0] {
      let mut spring = Spring::new(0.0, 40.0, 0.25);
      spring.target = 100.0;

      let mut frames = 0;
      while spring.step(dt) {
        assert!(spring.value <= 100.0 + f32::EPSILON);
        frames += 1;
        assert!(frames < 120, "spring never settled");
      }
      assert_close(spring.value, 100.0);
    }
  }

  /// Retuned mid-flight, it carries on from where it is.
  #[test]
  fn retuned_spring_carries_on_from_where_it_is() {
    let mut spring = Spring::new(0.0, 300.0, 0.25);
    spring.target = 100.0;
    spring.step(1.0 / 60.0);
    let value = spring.value;

    spring.retune(1000.0);
    assert_close(spring.value, value);
    spring.step(1.0 / 60.0);
    assert!(spring.value > value && spring.value < 100.0);

    spring.retune(0.0);
    spring.step(1.0 / 60.0);
    assert_close(spring.value, 100.0);
  }

  /// A rect held mid-flight stays where it was, for a zoom to set off
  /// from.
  #[test]
  fn held_spring_rect_stops_where_it_is() {
    let mut rect =
      SpringRect::new(RectF::new(0.0, 0.0, 100.0, 100.0), 300.0);
    rect.retarget(&RectF::new(500.0, 0.0, 100.0, 100.0), false);
    rect.step(1.0 / 60.0);
    rect.step(1.0 / 60.0);

    let held = rect.value();
    assert!(held.x > 0.0 && held.x < 500.0, "not mid-flight");
    rect.hold();

    assert!(!rect.step(1.0 / 60.0));
    assert_eq!(rect.value(), held);
    assert_eq!(rect.target(), held);
  }

  #[test]
  fn cards_widen_with_their_strip_up_to_twice() {
    let metrics = metrics_for(OverviewLayoutMode::Activities);
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
  fn activities_line_up_miniatures_over_the_scene() {
    let metrics = metrics_for(OverviewLayoutMode::Activities);
    let slots = [
      CardSlot::new(&metrics),
      wide_slot(&metrics, 3.0),
      CardSlot::new(&metrics),
    ];

    let (targets, scene) = activities(&metrics, &slots);
    let rects = targets
      .cards
      .iter()
      .zip(&slots)
      .map(|(card, slot)| card.rect(&with_width(&metrics, slot.width)))
      .collect::<Vec<_>>();

    // One row, at one height, in order, centered.
    for pair in rects.windows(2) {
      assert_close(pair[0].h, pair[1].h);
      assert_close(pair[0].y, pair[1].y);
      assert!(pair[1].x > pair[0].right());
    }
    assert_close(rects[0].x, 1920.0 - rects[2].right());

    // The scene is below them, clear of the hints.
    assert!(scene.y > rects[0].bottom());
    assert!(scene.bottom() < targets.search_y);
    assert!(scene.w > 1920.0 * 0.8);
  }

  #[test]
  fn many_miniatures_shrink_to_fit_the_row() {
    let metrics = metrics_for(OverviewLayoutMode::Activities);
    let slots = slots(&metrics, 20);

    let (targets, _) = activities(&metrics, &slots);
    let first = targets.cards[0].rect(&metrics);
    let last = targets.cards[19].rect(&metrics);

    assert!(first.x >= 47.99 && last.right() <= 1920.0 - 47.99);
  }

  #[test]
  fn cards_are_drawn_larger_for_layouts_that_show_them_larger() {
    let width = |layout| metrics_for(layout).width;

    assert!(
      width(OverviewLayoutMode::Activities)
        < width(OverviewLayoutMode::Stack)
    );

    // Clean cards have no header, but keep the monitor's shape.
    let miniature = metrics_for(OverviewLayoutMode::Activities);
    assert_eq!(miniature.look, CardLook::Clean);
    assert_close(miniature.header, miniature.padding);
    assert_close(
      miniature.tile_area().w / miniature.tile_area().h,
      1920.0 / 1040.0,
    );
    assert_eq!(metrics().look, CardLook::Classic);
  }

  #[test]
  fn a_fitted_view_centers_its_content() {
    let content = RectF::new(0.0, 0.0, 3840.0, 1040.0);
    let view =
      CardView::fitted(&content, &RectF::new(100.0, 200.0, 960.0, 960.0));

    assert_rect_close(
      &view.map(&content),
      &RectF::new(100.0, 200.0 + (960.0 - 260.0) / 2.0, 960.0, 260.0),
    );
  }

  #[test]
  fn a_mark_settles_onto_its_next_target_without_travel() {
    let mut mark = Mark::new(170.0);
    let t0 = Instant::now();
    let a = RectF::new(0.0, 0.0, 100.0, 100.0);
    let b = RectF::new(400.0, 0.0, 200.0, 100.0);

    mark.follow(Some((1, a, 1.0)), t0);
    for _ in 0..60 {
      mark.step(1.0 / 60.0);
    }

    // On `b` from the start, a little larger, fading in from nothing.
    mark.follow(Some((2, b, 1.0)), t0);
    assert!(mark.shown().is_none(), "faded in from nothing");
    mark.step(1.0 / 60.0);
    mark.follow(Some((2, b, 1.0)), t0 + Duration::from_millis(16));
    let (rect, ..) = mark.shown().expect("fading in");
    assert!(rect.x < b.x && rect.x > b.x - 10.0, "{rect:?}");
    assert!(rect.w > b.w);

    for _ in 0..60 {
      mark.step(1.0 / 60.0);
    }
    mark.follow(Some((2, b, 1.0)), t0 + Duration::from_millis(400));
    assert_eq!(mark.shown().map(|(rect, ..)| rect), Some(b));
  }

  #[test]
  fn a_mark_fades_out_where_it_is() {
    let mut mark = Mark::new(170.0);
    let rect = RectF::new(10.0, 10.0, 50.0, 50.0);
    let t0 = Instant::now();

    mark.follow(Some((1, rect, 1.0)), t0);
    for _ in 0..60 {
      mark.step(1.0 / 60.0);
    }

    mark.follow(None::<(i32, RectF, f32)>, t0);
    for _ in 0..60 {
      mark.step(1.0 / 60.0);
    }
    assert!(mark.shown().is_none());
  }

  #[test]
  fn an_instant_mark_shows_on_its_target_at_once() {
    let mut mark = Mark::new(0.0);
    let t0 = Instant::now();
    let a = RectF::new(0.0, 0.0, 100.0, 100.0);
    let b = RectF::new(400.0, 0.0, 200.0, 100.0);

    assert!(!mark.follow(Some((1, a, 1.0)), t0));
    assert_eq!(mark.shown(), Some((a, 1.0, 1.0)));

    assert!(!mark.follow(Some((2, b, 1.0)), t0));
    assert_eq!(mark.shown(), Some((b, 1.0, 1.0)));

    assert!(!mark.follow(None, t0));
    assert!(mark.shown().is_none());
  }

  /// A slot for the stack's card of a strip spanning `from` to `to`
  /// monitor widths of the monitor's own, keeping `keep` (in card units).
  fn strip_slot(
    metrics: &CardMetrics,
    from: f32,
    to: f32,
    keep: Option<(f32, f32)>,
  ) -> (CardSlot, CardMetrics) {
    let area = RectF::new(0.0, 0.0, 1920.0, 1040.0);
    let strip =
      RectF::new(1920.0 * from, 0.0, 1920.0 * (to - from), 1040.0);
    let card = metrics.for_content(&area, &strip);
    let view = CardView::new(&card, &area, &strip).map(&area);
    let slot = CardSlot {
      view_center: view.x + view.w / 2.0,
      keep,
      ..CardSlot::new(&card)
    };
    (slot, card)
  }

  #[test]
  fn the_stack_centers_the_selected_card_and_stacks_the_rest() {
    let metrics = metrics();
    let (targets, pan) = stack(&metrics, &slots(&metrics, 4), 1, 0.0);
    let rects = targets
      .cards
      .iter()
      .map(|card| card.rect(&metrics))
      .collect::<Vec<_>>();

    assert_close(pan, 0.0);
    assert_close(targets.cards[1].cx, 960.0);
    assert_close(targets.cards[1].cy, (targets.search_y - 16.0) / 2.0);
    for pair in rects.windows(2) {
      assert!(pair[1].y > pair[0].bottom(), "top to bottom, apart");
      assert_close(pair[0].x, pair[1].x);
    }
    for card in &targets.cards {
      assert_close(card.scale, 1.0);
    }
  }

  #[test]
  fn the_stack_shows_a_strip_at_full_length_around_its_view() {
    let metrics = metrics();
    // Two monitors' worth left of the view, one right of it.
    let (slot, card) = strip_slot(&metrics, -2.0, 2.0, None);
    let (targets, _) = stack(&metrics, &[slot], 0, 0.0);
    let rect = targets.cards[0].rect(&card);

    assert_close(card.tile_area().w, metrics.tile_area().w * 4.0);
    assert!(rect.x < 0.0 && rect.right() > 1920.0, "past both sides");
    assert_close(rect.x + slot.view_center, 960.0);
  }

  #[test]
  fn the_stack_letterboxes_only_strips_past_its_widest() {
    let metrics = metrics();
    let (_, widest) =
      strip_slot(&metrics, 0.0, STACK_MAX_WIDTHS + 2.0, None);
    let (_, long) =
      strip_slot(&metrics, 0.0, STACK_MAX_WIDTHS - 1.0, None);

    assert_close(widest.width, metrics.width * STACK_MAX_WIDTHS);
    assert_close(
      long.tile_area().w,
      metrics.tile_area().w * (STACK_MAX_WIDTHS - 1.0),
    );
  }

  #[test]
  fn panning_stops_with_the_middle_on_the_strip() {
    let metrics = metrics();
    let (slot, card) = strip_slot(&metrics, -1.0, 3.0, None);

    let (targets, pan) = stack(&metrics, &[slot], 0, 400.0);
    assert_close(pan, 400.0);
    let rect = targets.cards[0].rect(&card);
    assert_close(rect.x + slot.view_center + 400.0, 960.0);

    let (targets, pan) = stack(&metrics, &[slot], 0, 1.0e6);
    assert_close(pan, slot.width - slot.view_center);
    assert_close(targets.cards[0].rect(&card).right(), 960.0);

    let (_, pan) = stack(&metrics, &[slot], 0, -1.0e6);
    assert_close(pan, -slot.view_center);
  }

  #[test]
  fn the_stack_pans_just_enough_to_show_what_it_keeps() {
    let metrics = metrics();
    let (resting, card) = strip_slot(&metrics, 0.0, 4.0, None);
    let far = resting.width - 300.0;
    let (slot, _) =
      strip_slot(&metrics, 0.0, 4.0, Some((far, far + 200.0)));

    let (targets, pan) = stack(&metrics, &[slot], 0, 0.0);
    let rect = targets.cards[0].rect(&card);
    assert!(pan > 0.0);
    assert_close(rect.x + far + 200.0, 1920.0 - 48.0);

    // Already in view, it stays where it was panned to.
    let (_, kept) = stack(&metrics, &[slot], 0, pan + 50.0);
    assert_close(kept, pan + 50.0);
  }
}
