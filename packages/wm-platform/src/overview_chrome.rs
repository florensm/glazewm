//! The overview's pictures: each workspace card, its highlights, labels
//! and tags, the key hints and the window being dragged. They are drawn
//! into bitmaps once and composited by DWM as thumbnails, scaled to
//! wherever they are on each frame.
//!
//! A card is drawn over its windows' previews: where a preview goes the
//! card is cut out, and titles and borders sit on top of the preview's
//! edges. GDI can't draw onto translucent pixels, so text and icons are
//! drawn opaquely into scratch bitmaps first and blended in.

use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL, HICON},
};

use crate::{
  overview_layout::{
    CardLook, CardMetrics, Placement, RectF, LABEL_HEIGHT,
  },
  overview_state::{Hint, HintTone, SearchHint},
  paint,
  surface::Surface,
  text::{draw_text, text_width, Align, Font, Fonts},
  window_icons, Color, CornerRadii, OverviewStyle, Rect,
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

  /// Where its windows' desk shows through, cut out like the previews.
  pub desk: Option<RectF>,

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
    picture.desk = self.desk.map(|desk| desk.scaled(factor));

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
fn fade(color: Color, opacity: f32) -> Color {
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

  let tile_radius = match picture.look {
    CardLook::Classic => 4.0,
    CardLook::Clean => 6.0,
  };
  if let Some(desk) = &picture.desk {
    surface
      .canvas()
      .erase_rounded_rect(&desk.to_rect(), round(units.px(tile_radius)));
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
}

impl SliceLook {
  /// A ring of `width` pixels in `color`, rounded by `radius`.
  pub fn ring(radius: i32, width: i32, color: Color) -> Self {
    Self {
      radius,
      border: color,
      border_width: width,
    }
  }

  /// Pixels of each corner of its picture: room for the rounding and the
  /// border.
  pub fn corner(&self) -> i32 {
    self.radius.max(self.border_width).max(1) + 1
  }
}

/// Draws `look` into `surface`, `2 * corner + SLICE_MIDDLE` pixels square.
pub(crate) fn draw_slices(surface: &mut Surface, look: &SliceLook) {
  surface.pixels().fill(0);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let mut canvas = surface.canvas();

  if look.border.a > 0 && look.border_width > 0 {
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
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(&rect, radius, with_alpha(style.card, 0.94));
    canvas.stroke_rounded_rect(
      &rect,
      radius,
      round(px(1.0)).max(1),
      with_alpha(style.text, 0.12),
    );
  }

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
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(&rect, radius, with_alpha(style.card, 0.85));
    canvas.stroke_rounded_rect(
      &rect,
      radius,
      round(scale_factor).max(1),
      with_alpha(style.text, 0.12),
    );
  }

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

/// Height, side padding and corner radius of a key cap on the hints line,
/// and the gap to what the key does, in logical pixels.
const KEY_CAP: (f32, f32, f32, f32) = (20.0, 6.0, 5.0, 6.0);

/// Gap between the parts of the hints line, and padding at its ends, in
/// logical pixels.
const HINT_GAP: (f32, f32) = (16.0, 12.0);

/// Height, minimum width, padding and corner radius of the search field on
/// the hints line, in logical pixels.
const SEARCH_FIELD: (f32, f32, f32, f32) = (22.0, 200.0, 9.0, 6.0);

/// One part of the hints line, with its width.
enum HintPart<'a> {
  Lead(&'a str, i32),
  Search(&'a SearchHint, String, i32),
  Key {
    key: &'a str,
    action: &'a str,
    cap: i32,
    width: i32,
  },
}

impl HintPart<'_> {
  fn width(&self) -> i32 {
    match self {
      Self::Lead(_, width)
      | Self::Search(_, _, width)
      | Self::Key { width, .. } => *width,
    }
  }
}

/// Draws `hint`, centered, into `surface`: what is going on, the search
/// and each key as a key cap with what it does, on a pill of the cards'
/// color so it reads over whatever is behind it.
pub(crate) fn draw_hint(
  surface: &mut Surface,
  hint: &Hint,
  scale_factor: f32,
  style: &OverviewStyle,
) {
  let px = |length: f32| round(length * scale_factor);
  surface.pixels().fill(0);

  let mut fonts = Fonts::new(&style.font_family);
  let fonts = HintFonts {
    text: fonts.get(11.5 * scale_factor, false),
    key: fonts.get(10.5 * scale_factor, true),
  };
  let parts = hint_parts(hint, &fonts, scale_factor);
  let (gap, padding) = HINT_GAP;

  let gaps = i32::try_from(parts.len().saturating_sub(1)).unwrap_or(0);
  let width = (parts.iter().map(HintPart::width).sum::<i32>()
    + gaps * px(gap)
    + 2 * px(padding))
  .min(surface.width);
  let height = surface.height;
  let pill = Rect::from_xy((surface.width - width) / 2, 0, width, height);
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(
      &pill,
      height / 2,
      with_alpha(style.card, 0.85),
    );
    canvas.stroke_rounded_rect(
      &pill,
      height / 2,
      px(1.0).max(1),
      with_alpha(style.text, 0.1),
    );
  }

  let tone = match hint.tone {
    HintTone::Quiet => with_alpha(style.subtext, 0.85),
    HintTone::Accent => with_alpha(style.accent, 0.9),
    HintTone::Search => with_alpha(style.search, 0.9),
  };
  let mut x = pill.left + px(padding);
  for part in &parts {
    let rect = Rect::from_xy(x, 0, part.width(), height);
    match part {
      HintPart::Lead(text, _) => {
        draw_text(surface, &fonts.text, text, &rect, tone, Align::Left);
      }
      HintPart::Search(search, hits, _) => {
        let units = (scale_factor, tone);
        draw_search_field(
          surface, &rect, search, hits, &fonts, units, style,
        );
      }
      HintPart::Key {
        key, action, cap, ..
      } => {
        let units = (scale_factor, *cap);
        draw_key_hint(surface, &rect, (key, action), &fonts, units, style);
      }
    }
    x += part.width() + px(gap);
  }
}

/// The fonts of the hints line.
struct HintFonts {
  text: Font,
  key: Font,
}

/// `hint`'s parts, left to right, measured in `fonts` at `scale_factor`.
fn hint_parts<'a>(
  hint: &'a Hint,
  fonts: &HintFonts,
  scale_factor: f32,
) -> Vec<HintPart<'a>> {
  let px = |length: f32| round(length * scale_factor);
  let (cap_height, cap_padding, _, cap_gap) = KEY_CAP;
  let (_, field_width, field_padding, _) = SEARCH_FIELD;
  let mut parts = Vec::new();

  if let Some(lead) = &hint.lead {
    parts.push(HintPart::Lead(lead, text_width(&fonts.text, lead)));
  }
  if let Some(search) = &hint.search {
    let hits = format!(
      "{} {}",
      search.hits,
      if search.hits == 1 { "hit" } else { "hits" }
    );
    let width = (text_width(&fonts.text, &search.query)
      + text_width(&fonts.text, &hits)
      + px(2.0 * field_padding + HINT_GAP.0))
    .max(px(field_width));
    parts.push(HintPart::Search(search, hits, width));
  }
  for key in &hint.keys {
    let cap = (text_width(&fonts.key, &key.key) + px(2.0 * cap_padding))
      .max(px(cap_height));
    let width = cap + px(cap_gap) + text_width(&fonts.text, &key.action);
    parts.push(HintPart::Key {
      key: &key.key,
      action: &key.action,
      cap,
      width,
    });
  }
  parts
}

/// Draws the search field into `rect` of the hints line: the query, the
/// caret while typing, and the `hits` on the right, outlined in `tone`.
fn draw_search_field(
  surface: &mut Surface,
  rect: &Rect,
  search: &SearchHint,
  hits: &str,
  fonts: &HintFonts,
  (scale_factor, tone): (f32, Color),
  style: &OverviewStyle,
) {
  let px = |length: f32| round(length * scale_factor);
  let (field_height, _, field_padding, field_radius) = SEARCH_FIELD;
  let field = Rect::from_xy(
    rect.left,
    rect.top + (rect.height() - px(field_height)) / 2,
    rect.width(),
    px(field_height),
  );
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(
      &field,
      px(field_radius),
      with_alpha(style.card, 0.9),
    );
    canvas.stroke_rounded_rect(
      &field,
      px(field_radius),
      px(1.0).max(1),
      tone,
    );
  }

  let inner = Rect::from_ltrb(
    field.left + px(field_padding),
    rect.top,
    field.right - px(field_padding),
    rect.bottom,
  );
  draw_text(
    surface,
    &fonts.text,
    &search.query,
    &inner,
    style.text,
    Align::Left,
  );
  draw_text(
    surface,
    &fonts.text,
    hits,
    &inner,
    with_alpha(style.subtext, 0.85),
    Align::Right,
  );

  if search.is_typing {
    let caret_height = px(13.0);
    let caret = Rect::from_xy(
      inner.left + text_width(&fonts.text, &search.query) + px(1.0),
      rect.top + (rect.height() - caret_height) / 2,
      px(1.0).max(1),
      caret_height,
    );
    surface.canvas().fill_rounded_rect(&caret, 0, tone);
  }
}

/// Draws `key` as a key cap `cap` pixels wide at the start of `rect` of
/// the hints line, and what it does, `action`, after it.
fn draw_key_hint(
  surface: &mut Surface,
  rect: &Rect,
  (key, action): (&str, &str),
  fonts: &HintFonts,
  (scale_factor, cap): (f32, i32),
  style: &OverviewStyle,
) {
  let px = |length: f32| round(length * scale_factor);
  let (cap_height, _, cap_radius, cap_gap) = KEY_CAP;
  let cap_rect = Rect::from_xy(
    rect.left,
    rect.top + (rect.height() - px(cap_height)) / 2,
    cap,
    px(cap_height),
  );
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(
      &cap_rect,
      px(cap_radius),
      with_alpha(style.text, 0.1),
    );
    canvas.stroke_rounded_rect(
      &cap_rect,
      px(cap_radius),
      px(1.0).max(1),
      with_alpha(style.text, 0.16),
    );
  }
  draw_text(
    surface,
    &fonts.key,
    key,
    &cap_rect,
    with_alpha(style.text, 0.9),
    Align::Center,
  );

  let label = Rect::from_ltrb(
    rect.left + cap + px(cap_gap),
    rect.top,
    rect.right,
    rect.bottom,
  );
  draw_text(
    surface,
    &fonts.text,
    action,
    &label,
    with_alpha(style.subtext, 0.85),
    Align::Left,
  );
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

/// Blends the icon of window `hwnd` into `rect` of `surface`.
fn draw_window_icon(
  surface: &mut Surface,
  hwnd: isize,
  rect: &Rect,
  opacity: f32,
  (notify, notify_msg): (HWND, u32),
) {
  let size = rect.width().min(rect.height());
  let Some(icon) = window_icons::icon_for(hwnd, notify, notify_msg) else {
    return;
  };
  let Some(pixels) = draw_icon(icon, size) else {
    return;
  };

  surface
    .canvas()
    .draw_premultiplied(rect.left, rect.top, &pixels, size, opacity);
}

/// Premultiplied pixels of `icon` drawn at `size` x `size`.
///
/// Icons carry their transparency in a mask GDI won't hand over, so the
/// icon is drawn onto black and onto white and the difference recovered.
fn draw_icon(icon: HICON, size: i32) -> Option<Vec<u32>> {
  let draw_on = |background: u32| -> Option<Vec<u32>> {
    let mut scratch = Surface::new(size, size)?;
    scratch.pixels().fill(background);

    // SAFETY: `scratch` owns a valid DC and `icon` is a cached copy that
    // outlives the call.
    unsafe {
      DrawIconEx(scratch.dc, 0, 0, icon, size, size, 0, None, DI_NORMAL)
        .ok()?;
    }

    Some(scratch.pixels().to_vec())
  };

  let on_black = draw_on(0)?;
  let on_white = draw_on(0x00ff_ffff)?;
  Some(paint::matte_from_backgrounds(&on_black, &on_white))
}

#[cfg(test)]
mod tests {
  use super::{draw_hint, Hint, HintTone, SearchHint};
  use crate::{
    overview_state::KeyHint, surface::Surface, Color, OverviewStyle,
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
    };
    let hint = Hint {
      tone: HintTone::Search,
      lead: None,
      search: Some(SearchHint {
        query: String::new(),
        is_typing: true,
        hits: 0,
      }),
      keys: vec![KeyHint {
        key: "Esc".to_string(),
        action: "clear".to_string(),
      }],
    };

    let mut surface = Surface::new(1920, 30).expect("surface");
    draw_hint(&mut surface, &hint, 1.0, &style);
    assert!(surface.pixels().iter().any(|pixel| *pixel != 0));
  }
}
