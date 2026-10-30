//! The overview's pictures: each workspace card, its highlights, labels
//! and tags, the search field and the window being dragged. They are drawn
//! into bitmaps once and composited by DWM as thumbnails, scaled to
//! wherever they are on each frame.
//!
//! A card is drawn over its windows' previews: where a preview goes the
//! card is cut out, and titles and borders sit on top of the preview's
//! edges. GDI can't draw onto translucent pixels, so text and icons are
//! drawn opaquely into scratch bitmaps first and blended in.

use windows::Win32::Foundation::HWND;

use crate::{
  overview_layout::{
    CardLook, CardMetrics, Placement, RectF, LABEL_HEIGHT,
  },
  overview_state::{SearchField, ShortcutGroup},
  surface::Surface,
  text::{draw_text, is_installed, text_width, Align, Font, Fonts},
  window_icons::draw_window_icon,
  Color, CornerRadii, OverviewStyle, Rect,
};

/// Height and font size of the title strip under a window on a card.
const TILE_CAPTION: (f32, f32) = (16.0, 9.0);

/// Height and font size of the title strip of the dragged window.
const GHOST_CAPTION: (f32, f32) = (18.0, 9.0);

/// Size of the dragged window's picture, in logical pixels.
pub(crate) const GHOST_SIZE: (f32, f32) = (200.0, 124.0);

/// Everything a card's picture depends on, so it is only redrawn when
/// something in it changes. Selection and hover aren't: they show as
/// highlights over it, so moving them redraws nothing.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct CardPicture {
  pub label: String,
  pub look: CardLook,

  /// Whether its workspace is displayed on its monitor.
  pub is_displayed: bool,

  /// Whether it is the "+" card of a workspace yet to be activated.
  pub is_new: bool,

  /// Whether it is the special workspace's card, tinted with the accent.
  pub is_special: bool,

  /// Whether a window is carried or dragged, which the special
  /// workspace's card invites to be dropped on.
  pub is_receiving: bool,

  /// Handles of minimized windows, shown as icons in the header.
  pub minimized: Vec<isize>,

  /// Where a scrolling workspace's monitor is on its strip, when the card
  /// shows more than that.
  pub viewport: Option<RectF>,

  pub tiles: Vec<TilePicture>,

  /// Where its stacks' tab bars show, cut out like the previews, with the
  /// bars' corners.
  pub tab_bars: Vec<(RectF, CornerRadii)>,

  /// Bumped as window icons arrive, so cards showing them redraw.
  pub icons: u64,
}

impl CardPicture {
  /// This picture for a card drawn `factor` times its size.
  fn scaled(&self, factor: f32) -> Self {
    let mut picture = self.clone();
    picture.viewport =
      self.viewport.map(|viewport| viewport.scaled(factor));

    for tile in &mut picture.tiles {
      tile.rect = tile.rect.scaled(factor);
      tile.hole = tile.hole.scaled(factor);
      tile.outline = tile.outline.scaled(factor);
    }

    let scale = |radius: i32| round(to_f32(radius) * factor);
    for (rect, radii) in &mut picture.tab_bars {
      *rect = rect.scaled(factor);
      *radii = CornerRadii {
        top_left: scale(radii.top_left),
        top_right: scale(radii.top_right),
        bottom_right: scale(radii.bottom_right),
        bottom_left: scale(radii.bottom_left),
      };
    }

    picture
  }
}

/// A window on a card.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TilePicture {
  pub hwnd: isize,

  /// Where its preview shows.
  pub rect: RectF,

  /// Where the card is cut out for its preview: `rect`, grown to take in
  /// the window's own border and backdrop around it.
  pub hole: RectF,

  /// What its border goes around: `rect`, plus its stack's tab bar.
  pub outline: RectF,

  /// Title on a strip along its bottom; `None` for a stack's window,
  /// whose tab bar shows it.
  pub title: Option<String>,

  /// Whether its live preview shows; the card is cut out for it.
  pub has_preview: bool,

  pub opacity: f32,
  pub border_width: f32,
  pub border: Color,
}

/// Corner radius of a card, in logical pixels.
const CARD_RADIUS: f32 = 14.0;

/// How far a card's shadow fades out, and how far below the card it is
/// cast, in logical pixels.
const SHADOW: (f32, f32) = (22.0, 5.0);

/// Most pixels a card's picture is drawn with, 32 MiB of them. A card
/// bigger still, e.g. a long strip on a 4K monitor, is drawn smaller and
/// scaled up by DWM, so it can't take hundreds of MB or long to redraw.
const MAX_PICTURE_PIXELS: f32 = 8_388_608.0;

/// Corner radius of a card at scale 1, in pixels.
pub(crate) fn card_radius(metrics: &CardMetrics) -> f32 {
  CARD_RADIUS * metrics.scale_factor * metrics.chrome
}

/// Pixel size of a card at scale 1.
#[allow(clippy::cast_possible_truncation)]
fn card_size(metrics: &CardMetrics) -> (i32, i32) {
  (metrics.width.ceil() as i32, metrics.height.ceil() as i32)
}

/// Room around a card in its picture for its shadow, in the picture's
/// pixels.
fn shadow_margin(metrics: &CardMetrics) -> i32 {
  round((SHADOW.0 + SHADOW.1) * metrics.scale_factor * metrics.chrome)
}

/// Pixel size of a picture of a card drawn at `metrics`, with room for its
/// shadow around it.
fn padded_size(metrics: &CardMetrics) -> (i32, i32) {
  let (width, height) = card_size(metrics);
  let margin = shadow_margin(metrics);
  (width + 2 * margin, height + 2 * margin)
}

/// The metrics a card's picture is drawn at, and how many times its size
/// that is: 1 unless that would take more than `MAX_PICTURE_PIXELS`.
fn drawn_metrics(metrics: &CardMetrics) -> (CardMetrics, f32) {
  let (width, height) = padded_size(metrics);
  let factor = (MAX_PICTURE_PIXELS / (to_f32(width) * to_f32(height)))
    .sqrt()
    .min(1.0);

  (metrics.scaled(factor), factor)
}

/// Pixel size of a card's picture: the card at scale 1, with room for its
/// shadow around it.
///
/// Always that size however big the card is shown: DWM scales it, so
/// moving the selection never redraws a card. A picture redrawn at a new
/// size flickers, since the new bitmap and its thumbnail's source rect
/// can't be swapped in at once.
pub(crate) fn picture_size(metrics: &CardMetrics) -> (i32, i32) {
  padded_size(&drawn_metrics(metrics).0)
}

/// Where a card's picture goes, shadow and all, for the card to show
/// where `placement` puts it.
pub(crate) fn picture_rect(
  metrics: &CardMetrics,
  placement: &Placement,
) -> RectF {
  let (drawn, factor) = drawn_metrics(metrics);
  let (width, height) = padded_size(&drawn);

  // Overview pixels per pixel of the picture.
  let scale = placement.scale / factor;
  let margin = to_f32(shadow_margin(&drawn)) * scale;
  let card = placement.rect(metrics);

  RectF::new(
    card.x - margin,
    card.y - margin,
    to_f32(width) * scale,
    to_f32(height) * scale,
  )
}

/// `color` with its alpha set to `alpha` (0 to 1).
pub(crate) fn with_alpha(color: Color, alpha: f32) -> Color {
  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  let alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
  Color { a: alpha, ..color }
}

/// `color` with its alpha multiplied by `opacity`.
pub(crate) fn fade(color: Color, opacity: f32) -> Color {
  with_alpha(color, f32::from(color.a) / 255.0 * opacity)
}

/// Converts logical lengths into the pixels of a card's picture.
#[derive(Clone, Copy)]
struct Units {
  scale_factor: f32,
}

impl Units {
  /// `length` logical pixels.
  fn px(self, length: f32) -> f32 {
    length * self.scale_factor
  }
}

/// Draws a card with its shadow into `surface`, which must be
/// `picture_size` big.
///
/// `notify` is told when a window icon that wasn't cached yet arrives.
pub(crate) fn draw_card(
  surface: &mut Surface,
  picture: &CardPicture,
  metrics: &CardMetrics,
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let (metrics, factor) = drawn_metrics(metrics);
  let picture = picture.scaled(factor);

  let (width, height) = card_size(&metrics);
  let Some(mut face) = Surface::new(width, height) else {
    return;
  };
  draw_face(&mut face, &picture, &metrics, style, notify);

  let units = Units {
    scale_factor: metrics.scale_factor * metrics.chrome,
  };
  let margin = shadow_margin(&metrics);
  let radius = round(units.px(CARD_RADIUS));
  let (blur, drop) = SHADOW;

  surface.pixels().fill(0);
  let mut canvas = surface.canvas();
  // Not under the card: its holes show what is under it, not its shadow.
  canvas.fill_shadow_around(
    &Rect::from_xy(margin, margin + round(units.px(drop)), width, height),
    &Rect::from_xy(margin, margin, width, height),
    radius,
    units.px(blur),
    Color {
      r: 0,
      g: 0,
      b: 0,
      a: 80,
    },
  );
  canvas.draw_premultiplied(margin, margin, face.pixels(), width, 1.0);
}

/// Draws a card itself into `surface`, which must be `card_size` big.
fn draw_face(
  surface: &mut Surface,
  picture: &CardPicture,
  metrics: &CardMetrics,
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let units = Units {
    scale_factor: metrics.scale_factor * metrics.chrome,
  };

  surface.pixels().fill(0);
  let card = Rect::from_xy(0, 0, surface.width, surface.height);
  let radius = round(units.px(CARD_RADIUS));

  let is_inviting = picture.is_special && picture.is_receiving;

  let (fill, border, border_width) = if is_inviting {
    (
      with_alpha(style.accent, 0.15),
      with_alpha(style.accent, 0.85),
      2.5,
    )
  } else if picture.is_displayed {
    (
      with_alpha(style.accent, 0.12),
      with_alpha(style.accent, 0.8),
      1.5,
    )
  } else if picture.is_special {
    (
      with_alpha(style.accent, 0.08),
      with_alpha(style.accent, 0.5),
      1.5,
    )
  } else if picture.is_new {
    (fade(style.card, 0.5), with_alpha(style.accent, 0.2), 1.0)
  } else {
    (style.card, with_alpha(style.accent, 0.2), 1.0)
  };

  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(&card, radius, fill);
    canvas.stroke_rounded_rect(
      &card,
      radius,
      round(units.px(border_width)),
      border,
    );
  }

  if let Some(viewport) = &picture.viewport {
    draw_viewport(surface, viewport, units, style);
  }

  let mut fonts = Fonts::new(&style.font_family);
  if picture.look == CardLook::Classic {
    draw_header(surface, &mut fonts, picture, units, style, notify);
  }

  if picture.is_new {
    let color = with_alpha(style.subtext, 0.5);
    draw_plus(surface, &metrics.tile_area().to_rect(), units, color);
  }

  if picture.is_special && picture.tiles.is_empty() {
    let color = if is_inviting {
      style.accent
    } else {
      with_alpha(style.subtext, 0.6)
    };
    draw_text(
      surface,
      &fonts.get(units.px(12.0), false),
      "Drop windows here",
      &metrics.tile_area().to_rect(),
      color,
      Align::Center,
    );
  }

  {
    let mut canvas = surface.canvas();
    for (rect, radii) in &picture.tab_bars {
      canvas.erase_rect_with_corners(&rect.to_rect(), *radii);
    }
  }

  // Previews are cut out in stacking order, so a floating window's hole
  // clears what was drawn for the tiled window under it.
  let tile_radius = match picture.look {
    CardLook::Classic => 4.0,
    CardLook::Clean => 6.0,
  };
  for tile in &picture.tiles {
    draw_tile(
      surface,
      &mut fonts,
      tile,
      (units, tile_radius),
      style,
      notify,
    );
  }
}

/// Marks out where a scrolling workspace's monitor is on its strip: a
/// lighter frame around it, showing between its windows.
fn draw_viewport(
  surface: &mut Surface,
  viewport: &RectF,
  units: Units,
  style: &OverviewStyle,
) {
  let frame = viewport.inset(-units.px(5.0)).to_rect();
  let radius = round(units.px(9.0));

  let mut canvas = surface.canvas();
  canvas.fill_rounded_rect(&frame, radius, with_alpha(style.text, 0.05));
  canvas.stroke_rounded_rect(
    &frame,
    radius,
    round(units.px(1.5)).max(1),
    with_alpha(style.subtext, 0.32),
  );
}

/// Draws a "+" in the middle of `area`.
fn draw_plus(
  surface: &mut Surface,
  area: &Rect,
  units: Units,
  color: Color,
) {
  let length = round(
    units
      .px(44.0)
      .min(to_f32(area.height()) * 0.4)
      .min(to_f32(area.width()) * 0.4),
  );
  let thickness = round(units.px(4.0)).max(1);
  let (center_x, center_y) = (
    i32::midpoint(area.left, area.right),
    i32::midpoint(area.top, area.bottom),
  );

  let mut canvas = surface.canvas();
  for (width, height) in [(length, thickness), (thickness, length)] {
    canvas.fill_rounded_rect(
      &Rect::from_xy(
        center_x - width / 2,
        center_y - height / 2,
        width,
        height,
      ),
      thickness / 2,
      color,
    );
  }
}

/// Draws a card's name on the left, and its minimized windows (nothing to
/// draw them with) as small icons on the right.
fn draw_header(
  surface: &mut Surface,
  fonts: &mut Fonts,
  picture: &CardPicture,
  units: Units,
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let header = units.px(28.0);
  let half = to_f32(surface.width) / 2.0;

  // The special workspace's name follows a diamond in the accent color.
  let mut label_left = units.px(12.0);
  if picture.is_special {
    label_left += draw_diamond(
      surface,
      (label_left, header / 2.0),
      units,
      style.accent,
    );
  }

  draw_text(
    surface,
    &fonts.get(units.px(12.0), picture.is_displayed),
    &picture.label,
    &RectF::new(label_left, 0.0, half, header).to_rect(),
    if picture.is_displayed {
      style.accent
    } else {
      style.subtext
    },
    Align::Left,
  );

  let mut x = to_f32(surface.width) - units.px(10.0);
  for hwnd in &picture.minimized {
    x -= units.px(16.0);
    let icon =
      RectF::new(x, units.px(7.0), units.px(14.0), units.px(14.0));
    draw_window_icon(surface, *hwnd, &icon.to_rect(), 0.6, notify);
  }
}

/// Draws the special workspace's diamond with its left corner at `left`,
/// `center_y`. Returns how much room it takes, its gap after it included.
fn draw_diamond(
  surface: &mut Surface,
  (left, center_y): (f32, f32),
  units: Units,
  color: Color,
) -> f32 {
  let size = units.px(4.0);
  let center_x = left + size;
  let corners = [
    (center_x, center_y - size),
    (center_x + size, center_y),
    (center_x, center_y + size),
    (center_x - size, center_y),
  ];

  let mut canvas = surface.canvas();
  for (index, from) in corners.iter().enumerate() {
    let to = corners[(index + 1) % corners.len()];
    canvas.stroke_line(*from, to, units.px(1.5), color);
  }

  size * 2.0 + units.px(6.0)
}

/// Draws a window on a card: a hole for its preview (or its icon when it
/// has none), its title along the bottom, and a border around it and its
/// stack's tab bar.
fn draw_tile(
  surface: &mut Surface,
  fonts: &mut Fonts,
  tile: &TilePicture,
  (units, radius): (Units, f32),
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let rect = tile.rect.to_rect();
  let radius_px = units.px(radius);
  let radius = round(radius_px);
  let opacity = tile.opacity;

  if tile.has_preview {
    // Rounded concentrically with the preview's own corners.
    let grown = (tile.rect.x - tile.hole.x)
      .max(tile.rect.y - tile.hole.y)
      .max(tile.hole.right() - tile.rect.right())
      .max(tile.hole.bottom() - tile.rect.bottom());
    surface
      .canvas()
      .erase_rounded_rect(&tile.hole.to_rect(), round(radius_px + grown));
  } else {
    surface.canvas().fill_rounded_rect(
      &rect,
      radius,
      fade(style.surface, opacity),
    );

    let size = units
      .px(36.0)
      .min(to_f32(rect.height()) / 2.0)
      .min(to_f32(rect.width()) / 2.0);
    let icon = RectF::new(
      to_f32(rect.left + rect.right) / 2.0 - size / 2.0,
      to_f32(rect.top + rect.bottom) / 2.0 - size / 2.0,
      size,
      size,
    );
    draw_window_icon(
      surface,
      tile.hwnd,
      &icon.to_rect(),
      0.5 * opacity,
      notify,
    );
  }

  if let Some(title) = &tile.title {
    draw_caption(
      surface,
      fonts,
      title,
      &rect,
      (
        units.px(TILE_CAPTION.0),
        units.px(TILE_CAPTION.1),
        units.px(4.0),
      ),
      style,
      opacity,
    );
  }

  if tile.border_width > 0.0 {
    surface.canvas().stroke_rounded_rect(
      &tile.outline.to_rect(),
      radius,
      round(units.px(tile.border_width)).max(1),
      fade(tile.border, opacity),
    );
  }
}

/// Middle of a stretchable picture, wide enough that DWM's filtering at
/// its edges only ever mixes in more of it.
pub(crate) const SLICE_MIDDLE: i32 = 4;

/// A rounded ring drawn once, then stretched to any size by showing its
/// corners as they are and stretching the rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SliceLook {
  /// Corner radius, in pixels.
  pub radius: i32,
  pub border: Color,
  pub border_width: i32,

  /// How far, in pixels, a soft glow of `border` reaches out of the
  /// rounded rect, which is left empty; `0` draws the ring instead.
  pub glow: i32,
}

impl SliceLook {
  /// A ring of `width` pixels in `color`, rounded by `radius`.
  pub fn ring(radius: i32, width: i32, color: Color) -> Self {
    Self {
      radius,
      border: color,
      border_width: width,
      glow: 0,
    }
  }

  /// A glow of `color` fading out over `reach` pixels around a rect
  /// rounded by `radius`.
  pub fn glow(radius: i32, reach: i32, color: Color) -> Self {
    Self {
      radius,
      border: color,
      border_width: 0,
      glow: reach.max(1),
    }
  }

  /// Pixels of each corner of its picture: room for the rounding, the
  /// border and the glow.
  pub fn corner(&self) -> i32 {
    self.radius.max(self.border_width).max(1) + 1 + self.glow
  }
}

/// Draws `look` into `surface`, `2 * corner + SLICE_MIDDLE` pixels square.
pub(crate) fn draw_slices(surface: &mut Surface, look: &SliceLook) {
  surface.pixels().fill(0);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let mut canvas = surface.canvas();

  if look.glow > 0 {
    let inner = Rect::from_xy(
      look.glow,
      look.glow,
      rect.width() - 2 * look.glow,
      rect.height() - 2 * look.glow,
    );
    #[allow(clippy::cast_precision_loss)]
    canvas.fill_shadow_around(
      &inner,
      &inner,
      look.radius,
      look.glow as f32,
      look.border,
    );
  } else if look.border.a > 0 && look.border_width > 0 {
    canvas.stroke_rounded_rect(
      &rect,
      look.radius,
      look.border_width,
      look.border,
    );
  }
}

/// Height, padding, icon size, gap after the icon and font size of the
/// tag under the picked window, in logical pixels.
const TAG: (f32, f32, f32, f32, f32) = (30.0, 12.0, 16.0, 8.0, 12.5);

/// What the tag under the picked window shows.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagPicture {
  pub hwnd: isize,
  pub text: String,

  /// Bumped as window icons arrive, so the tag picks up its window's.
  pub icons: u64,
}

/// Pixel size of `tag`, at most `max_width` wide.
pub(crate) fn tag_size(
  tag: &TagPicture,
  style: &OverviewStyle,
  scale_factor: f32,
  max_width: f32,
) -> (i32, i32) {
  let px = |length: f32| length * scale_factor;
  let (height, padding, icon, gap, font_size) = TAG;
  let mut fonts = Fonts::new(&style.font_family);
  let text =
    to_f32(text_width(&fonts.get(px(font_size), false), &tag.text));

  let width = (px(2.0 * padding + icon + gap) + text).min(max_width);
  (round(width).max(1), round(px(height)).max(1))
}

/// Draws `tag` into `surface`, `tag_size` big: its window's icon and
/// title on a pill.
pub(crate) fn draw_tag(
  surface: &mut Surface,
  tag: &TagPicture,
  style: &OverviewStyle,
  scale_factor: f32,
  notify: (HWND, u32),
) {
  let px = |length: f32| length * scale_factor;
  let (_, padding, icon, gap, font_size) = TAG;
  surface.pixels().fill(0);

  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let radius = surface.height / 2;
  surface.canvas().fill_rounded_rect(
    &rect,
    radius,
    with_alpha(style.card, 0.94),
  );

  let icon_rect = RectF::new(
    px(padding),
    (to_f32(surface.height) - px(icon)) / 2.0,
    px(icon),
    px(icon),
  );
  draw_window_icon(surface, tag.hwnd, &icon_rect.to_rect(), 1.0, notify);

  let mut fonts = Fonts::new(&style.font_family);
  draw_text(
    surface,
    &fonts.get(px(font_size), false),
    &tag.text,
    &Rect::from_ltrb(
      round(px(padding + icon + gap)),
      0,
      surface.width - round(px(padding)),
      surface.height,
    ),
    style.text,
    Align::Left,
  );
}

/// Padding and font size of a workspace's name shown beside its card,
/// in logical pixels.
const LABEL: (f32, f32) = (9.0, 11.5);

/// A workspace's name shown beside its card.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LabelPicture {
  pub text: String,
  pub is_displayed: bool,
  pub is_special: bool,
}

/// Pixel size of `label`.
pub(crate) fn label_size(
  label: &LabelPicture,
  style: &OverviewStyle,
  scale_factor: f32,
) -> (i32, i32) {
  let units = Units { scale_factor };
  let (padding, font_size) = LABEL;
  let mut fonts = Fonts::new(&style.font_family);
  let text = text_width(
    &fonts.get(units.px(font_size), label.is_displayed),
    &label.text,
  );
  let diamond = if label.is_special {
    units.px(14.0)
  } else {
    0.0
  };

  (
    round(units.px(2.0 * padding) + diamond + to_f32(text)).max(1),
    round(units.px(LABEL_HEIGHT)).max(1),
  )
}

/// Draws `label` into `surface`, `label_size` big: the name on a pill,
/// in the accent color for the focused workspace.
pub(crate) fn draw_label(
  surface: &mut Surface,
  label: &LabelPicture,
  style: &OverviewStyle,
  scale_factor: f32,
) {
  let units = Units { scale_factor };
  let (padding, font_size) = LABEL;
  surface.pixels().fill(0);

  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let radius = surface.height / 2;
  surface.canvas().fill_rounded_rect(
    &rect,
    radius,
    with_alpha(style.card, 0.82),
  );

  let mut left = units.px(padding);
  if label.is_special {
    left += draw_diamond(
      surface,
      (left, to_f32(surface.height) / 2.0),
      units,
      style.accent,
    );
  }

  let mut fonts = Fonts::new(&style.font_family);
  draw_text(
    surface,
    &fonts.get(units.px(font_size), label.is_displayed),
    &label.text,
    &Rect::from_ltrb(round(left), 0, surface.width, surface.height),
    if label.is_displayed {
      style.accent
    } else {
      with_alpha(style.text, 0.9)
    },
    Align::Left,
  );
}

/// Font size and padding of the note shown where a workspace has no
/// windows, in logical pixels.
const NOTE_FONT: f32 = 15.0;
const NOTE_PADDING: f32 = 16.0;

/// Pixel size of the note `text`.
pub(crate) fn note_size(
  text: &str,
  style: &OverviewStyle,
  scale_factor: f32,
) -> (i32, i32) {
  let px = |length: f32| length * scale_factor;
  let mut fonts = Fonts::new(&style.font_family);
  let width = text_width(&fonts.get(px(NOTE_FONT), false), text);
  (
    width.max(1) + round(px(2.0 * NOTE_PADDING)),
    round(px(NOTE_FONT * 2.2)).max(1),
  )
}

/// Draws the note `text` into `surface`, `note_size` big: on a pill, to
/// read on any wallpaper.
pub(crate) fn draw_note(
  surface: &mut Surface,
  text: &str,
  style: &OverviewStyle,
  scale_factor: f32,
) {
  surface.pixels().fill(0);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let radius = surface.height / 2;
  surface.canvas().fill_rounded_rect(
    &rect,
    radius,
    with_alpha(style.card, 0.85),
  );

  let mut fonts = Fonts::new(&style.font_family);
  draw_text(
    surface,
    &fonts.get(NOTE_FONT * scale_factor, false),
    text,
    &rect,
    with_alpha(style.subtext, 0.8),
    Align::Center,
  );
}

/// Draws the picture of a dragged or carried window titled `title` into
/// `surface`. Its preview shows through the middle.
pub(crate) fn draw_ghost(
  surface: &mut Surface,
  title: &str,
  scale_factor: f32,
  style: &OverviewStyle,
) {
  let px = |length: f32| length * scale_factor;
  surface.pixels().fill(0);

  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let mut fonts = Fonts::new(&style.font_family);

  draw_caption(
    surface,
    &mut fonts,
    title,
    &rect,
    (px(GHOST_CAPTION.0), px(GHOST_CAPTION.1), px(4.0)),
    style,
    1.0,
  );

  surface.canvas().stroke_rounded_rect(
    &rect,
    round(px(8.0)),
    round(px(2.0)).max(1),
    style.accent,
  );
}

/// Width at least of the search field, its padding and the gap between
/// its parts, in logical pixels.
const SEARCH_FIELD: (f32, f32, f32) = (360.0, 14.0, 10.0);

/// The search icon of Windows' icon fonts, and those fonts, newest first.
const SEARCH_GLYPH: &str = "\u{e721}";
const ICON_FAMILIES: [&str; 2] =
  ["Segoe Fluent Icons", "Segoe MDL2 Assets"];

/// Height, side padding and corner radius of a key cap, in logical
/// pixels.
const KEY_CAP: (f32, f32, f32) = (22.0, 7.0, 6.0);

/// Where the search line's controls are, in its picture.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SearchControls {
  pub field: Rect,
  pub shortcuts: Rect,
}

/// Draws the search line, centered, into `surface`: the search field (a
/// search icon, the query or what to do, and its matches, ringed in the
/// accent while typing goes to it) and beside it the chip that opens the
/// keyboard shortcuts, ringed while they are open.
pub(crate) fn draw_search(
  surface: &mut Surface,
  search: &SearchField,
  shows_shortcuts: bool,
  scale_factor: f32,
  style: &OverviewStyle,
) -> SearchControls {
  let px = |length: f32| round(length * scale_factor);
  let (min_width, padding, gap) = SEARCH_FIELD;
  surface.pixels().fill(0);

  let mut fonts = Fonts::new(&style.font_family);
  let font = fonts.get(13.0 * scale_factor, false);
  let key_font = fonts.get(11.0 * scale_factor, true);
  let icon = ICON_FAMILIES
    .into_iter()
    .find(|family| is_installed(family))
    .map(|family| Fonts::new(family).get(14.0 * scale_factor, false));

  let (text, text_color) =
    match (search.query.is_empty(), search.is_typing) {
      (true, true) => ("Type to search", with_alpha(style.subtext, 0.6)),
      (true, false) => ("Search windows", with_alpha(style.subtext, 0.7)),
      (false, _) => (search.query.as_str(), style.text),
    };

  // The matches, or else the key that starts a search.
  let (hits, slash) = match (search.hits, search.query.is_empty()) {
    (_, true) => (String::new(), !search.is_typing),
    (0, false) => ("No matches".to_string(), false),
    (1, false) => ("1 match".to_string(), false),
    (hits, false) => (format!("{hits} matches"), false),
  };

  let icon_width = icon
    .as_ref()
    .map_or(0, |icon| text_width(icon, SEARCH_GLYPH));
  let typed_width = text_width(&font, text);
  let end_width = if slash {
    key_cap_width(&key_font, "/", scale_factor)
  } else {
    text_width(&font, &hits)
  };
  let chip_width = shortcuts_chip_width(&font, &key_font, scale_factor);

  let height = surface.height;
  let field_width =
    (icon_width + typed_width + end_width + 3 * px(gap) + 2 * px(padding))
      .max(px(min_width));
  let total = (field_width + px(gap) + chip_width).min(surface.width);
  let field = Rect::from_xy(
    (surface.width - total) / 2,
    0,
    total - px(gap) - chip_width,
    height,
  );
  let chip = Rect::from_xy(field.right + px(gap), 0, chip_width, height);

  draw_search_pill(surface, &field, search.is_typing, scale_factor, style);
  draw_search_pill(surface, &chip, shows_shortcuts, scale_factor, style);

  let mut left = field.left + px(padding);
  if let Some(icon) = &icon {
    let rect = Rect::from_xy(left, 0, icon_width, height);
    let color = with_alpha(style.subtext, 0.8);
    draw_text(surface, icon, SEARCH_GLYPH, &rect, color, Align::Left);
    left += icon_width + px(gap);
  }

  let right = field.right - px(padding);
  let end_left = right - end_width;
  let text_rect = Rect::from_ltrb(left, 0, end_left - px(gap), height);
  draw_text(surface, &font, text, &text_rect, text_color, Align::Left);
  if slash {
    let cap = key_cap_rect(end_left, height, end_width, scale_factor);
    draw_key_cap(surface, &cap, "/", &key_font, scale_factor, style);
  } else {
    draw_text(
      surface,
      &font,
      &hits,
      &Rect::from_ltrb(end_left, 0, right, height),
      with_alpha(style.subtext, 0.85),
      Align::Right,
    );
  }

  // Before the placeholder, after the query.
  if search.is_typing {
    let typed = if search.query.is_empty() {
      0
    } else {
      typed_width
    };
    let caret_height = px(16.0);
    let caret = Rect::from_xy(
      (left + typed).min(text_rect.right),
      (height - caret_height) / 2,
      px(1.5).max(1),
      caret_height,
    );
    surface.canvas().fill_rounded_rect(
      &caret,
      0,
      with_alpha(style.accent, 0.9),
    );
  }

  draw_shortcuts_chip(
    surface,
    &chip,
    [&font, &key_font],
    scale_factor,
    style,
  );

  SearchControls {
    field,
    shortcuts: chip,
  }
}

/// Label of the chip that opens the shortcut sheet.
const SHORTCUTS_LABEL: &str = "Shortcuts";

/// Width of the chip that opens the shortcut sheet.
fn shortcuts_chip_width(
  font: &Font,
  key_font: &Font,
  scale_factor: f32,
) -> i32 {
  let (_, padding, gap) = SEARCH_FIELD;
  key_cap_width(key_font, "?", scale_factor)
    + text_width(font, SHORTCUTS_LABEL)
    + round((2.0 * padding + gap) * scale_factor)
}

/// Draws the `?` key cap and label of the chip that opens the shortcut
/// sheet into `chip`, in the label's font and the key's.
fn draw_shortcuts_chip(
  surface: &mut Surface,
  chip: &Rect,
  [font, key_font]: [&Font; 2],
  scale_factor: f32,
  style: &OverviewStyle,
) {
  let px = |length: f32| round(length * scale_factor);
  let (_, padding, gap) = SEARCH_FIELD;
  let height = chip.height();

  let cap_width = key_cap_width(key_font, "?", scale_factor);
  let cap =
    key_cap_rect(chip.left + px(padding), height, cap_width, scale_factor);
  let cap =
    Rect::from_xy(cap.left, chip.top + cap.top, cap_width, cap.height());
  draw_key_cap(surface, &cap, "?", key_font, scale_factor, style);
  draw_text(
    surface,
    font,
    SHORTCUTS_LABEL,
    &Rect::from_ltrb(
      cap.right + px(gap),
      chip.top,
      chip.right - px(padding),
      chip.bottom,
    ),
    with_alpha(style.subtext, 0.9),
    Align::Left,
  );
}

/// Fills `rect` as a pill of the search line, ringed in the accent if
/// `is_ringed`.
fn draw_search_pill(
  surface: &mut Surface,
  rect: &Rect,
  is_ringed: bool,
  scale_factor: f32,
  style: &OverviewStyle,
) {
  let radius = rect.height() / 2;
  let mut canvas = surface.canvas();
  canvas.fill_rounded_rect(rect, radius, with_alpha(style.card, 0.88));
  if is_ringed {
    canvas.stroke_rounded_rect(
      rect,
      radius,
      round(2.0 * scale_factor).max(1),
      with_alpha(style.accent, 0.9),
    );
  }
}

/// Width of a key cap showing `key` in `font`.
fn key_cap_width(font: &Font, key: &str, scale_factor: f32) -> i32 {
  let (height, padding, _) = KEY_CAP;
  (text_width(font, key) + round(2.0 * padding * scale_factor))
    .max(round(height * scale_factor))
}

/// A key cap `width` wide from `left`, centered on a line `line` tall.
fn key_cap_rect(
  left: i32,
  line: i32,
  width: i32,
  scale_factor: f32,
) -> Rect {
  let height = round(KEY_CAP.0 * scale_factor);
  Rect::from_xy(left, (line - height) / 2, width, height)
}

/// Draws `key` on a key cap filling `rect`.
fn draw_key_cap(
  surface: &mut Surface,
  rect: &Rect,
  key: &str,
  font: &Font,
  scale_factor: f32,
  style: &OverviewStyle,
) {
  let radius = round(KEY_CAP.2 * scale_factor);
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(rect, radius, with_alpha(style.text, 0.1));
    canvas.stroke_rounded_rect(
      rect,
      radius,
      round(scale_factor).max(1),
      with_alpha(style.text, 0.16),
    );
  }
  draw_text(
    surface,
    font,
    key,
    rect,
    with_alpha(style.text, 0.9),
    Align::Center,
  );
}

/// Padding, title height and gap under it, column gap, group title
/// height, and row height of the shortcut sheet, in logical pixels.
const SHEET: (f32, f32, f32, f32, f32, f32) =
  (28.0, 22.0, 20.0, 36.0, 22.0, 32.0);

/// Gap between a shortcut's key cap and what it does, in logical pixels.
const SHEET_KEY_GAP: f32 = 10.0;

/// The fonts of the shortcut sheet: its title, group titles and keys, and
/// what the keys do.
fn sheet_fonts(style: &OverviewStyle, scale_factor: f32) -> [Font; 3] {
  let mut fonts = Fonts::new(&style.font_family);
  [
    fonts.get(15.0 * scale_factor, true),
    fonts.get(11.0 * scale_factor, true),
    fonts.get(12.5 * scale_factor, false),
  ]
}

/// Pixel size of the shortcut sheet listing `groups`, and each column's
/// width.
fn sheet_layout(
  groups: &[ShortcutGroup],
  scale_factor: f32,
  style: &OverviewStyle,
) -> ((i32, i32), Vec<i32>) {
  let px = |length: f32| round(length * scale_factor);
  let (padding, title, title_gap, column_gap, group_title, row) = SHEET;
  let [_, key_font, font] = sheet_fonts(style, scale_factor);

  let columns = groups
    .iter()
    .map(|group| {
      group
        .shortcuts
        .iter()
        .map(|shortcut| {
          key_cap_width(&key_font, &shortcut.key, scale_factor)
            + px(SHEET_KEY_GAP)
            + text_width(&font, &shortcut.action)
        })
        .chain([text_width(&key_font, &group.title.to_uppercase())])
        .max()
        .unwrap_or(0)
    })
    .collect::<Vec<_>>();

  let rows = groups
    .iter()
    .map(|group| group.shortcuts.len())
    .max()
    .unwrap_or(0);
  let gaps = i32::try_from(columns.len().saturating_sub(1)).unwrap_or(0);
  let size = (
    columns.iter().sum::<i32>() + gaps * px(column_gap) + 2 * px(padding),
    px(2.0 * padding + title + title_gap + group_title)
      + px(row) * i32::try_from(rows).unwrap_or(0),
  );
  (size, columns)
}

/// Pixel size of the shortcut sheet listing `groups`.
pub(crate) fn sheet_size(
  groups: &[ShortcutGroup],
  scale_factor: f32,
  style: &OverviewStyle,
) -> (i32, i32) {
  sheet_layout(groups, scale_factor, style).0
}

/// Draws the sheet of keyboard shortcuts listing `groups` into `surface`,
/// `sheet_size` big: a column per step, each key on a key cap.
pub(crate) fn draw_sheet(
  surface: &mut Surface,
  groups: &[ShortcutGroup],
  scale_factor: f32,
  style: &OverviewStyle,
) {
  let px = |length: f32| round(length * scale_factor);
  let (padding, title, title_gap, column_gap, group_title, row) = SHEET;
  let (_, columns) = sheet_layout(groups, scale_factor, style);
  let [title_font, key_font, font] = sheet_fonts(style, scale_factor);
  surface.pixels().fill(0);

  let sheet = Rect::from_xy(0, 0, surface.width, surface.height);
  surface.canvas().fill_rounded_rect(
    &sheet,
    px(18.0),
    with_alpha(style.card, 0.96),
  );

  let top = px(padding);
  draw_text(
    surface,
    &title_font,
    "Keyboard shortcuts",
    &Rect::from_ltrb(
      px(padding),
      top,
      sheet.right - px(padding),
      top + px(title),
    ),
    style.text,
    Align::Left,
  );

  let groups_top = top + px(title + title_gap);
  let mut left = px(padding);
  for (group, width) in groups.iter().zip(columns) {
    draw_text(
      surface,
      &key_font,
      &group.title.to_uppercase(),
      &Rect::from_xy(left, groups_top, width, px(group_title)),
      with_alpha(style.subtext, 0.8),
      Align::Left,
    );

    let mut y = groups_top + px(group_title);
    for shortcut in &group.shortcuts {
      let cap_width =
        key_cap_width(&key_font, &shortcut.key, scale_factor);
      let cap = key_cap_rect(left, px(row), cap_width, scale_factor);
      let cap =
        Rect::from_xy(cap.left, y + cap.top, cap.width(), cap.height());
      draw_key_cap(
        surface,
        &cap,
        &shortcut.key,
        &key_font,
        scale_factor,
        style,
      );
      draw_text(
        surface,
        &font,
        &shortcut.action,
        &Rect::from_ltrb(
          cap.right + px(SHEET_KEY_GAP),
          y,
          left + width,
          y + px(row),
        ),
        style.text,
        Align::Left,
      );
      y += px(row);
    }
    left += width + px(column_gap);
  }
}

/// Draws a window title on a strip along the bottom of `rect`, if `rect`
/// has room for one. `metrics` is the strip's height, font size and text
/// inset.
fn draw_caption(
  surface: &mut Surface,
  fonts: &mut Fonts,
  title: &str,
  rect: &Rect,
  (height, font_size, inset): (f32, f32, f32),
  style: &OverviewStyle,
  opacity: f32,
) {
  // A sliver of a window has no room for a title.
  let has_room = to_f32(rect.height()) > height * 2.4
    && to_f32(rect.width()) > font_size * 6.7;
  if !has_room {
    return;
  }

  let strip = Rect::from_ltrb(
    rect.left,
    rect.bottom - round(height),
    rect.right,
    rect.bottom,
  );
  surface.canvas().fill_rounded_rect(
    &strip,
    0,
    fade(style.caption, opacity),
  );

  let inset = round(inset);
  draw_text(
    surface,
    &fonts.get(font_size, false),
    title,
    &Rect::from_ltrb(
      strip.left + inset,
      strip.top,
      strip.right - inset,
      strip.bottom,
    ),
    fade(style.text, opacity),
    Align::Left,
  );
}

#[allow(clippy::cast_possible_truncation)]
fn round(length: f32) -> i32 {
  length.round() as i32
}

#[allow(clippy::cast_precision_loss)]
fn to_f32(pixels: i32) -> f32 {
  pixels as f32
}

#[cfg(test)]
mod tests {
  use super::draw_search;
  use crate::{
    overview_state::SearchField, surface::Surface, Color, OverviewStyle,
  };

  /// A search right after `/`, with nothing typed: an empty query must
  /// not reach the text measuring, which crashed on it.
  #[test]
  fn draws_a_search_with_nothing_typed_yet() {
    let color = Color {
      r: 205,
      g: 214,
      b: 244,
      a: 255,
    };
    let style = OverviewStyle {
      backdrop_blur: 30.0,
      backdrop_tint: color,
      accent: color,
      card: color,
      surface: color,
      caption: color,
      text: color,
      subtext: color,
      search: color,
      font_family: "Segoe UI".to_string(),
      grid_columns: 4,
      open_duration_ms: 200,
      close_duration_ms: 200,
      switch_duration_ms: 300,
      highlight_duration_ms: 170,
      bounce: 0.25,
      effects: crate::OverviewEffects::default(),
    };
    let search = SearchField {
      query: String::new(),
      is_typing: true,
      hits: 0,
    };

    let mut surface = Surface::new(1920, 36).expect("surface");
    draw_search(&mut surface, &search, false, 1.0, &style);
    assert!(surface.pixels().iter().any(|pixel| *pixel != 0));
  }
}
