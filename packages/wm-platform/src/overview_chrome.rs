//! The overview's pictures: each workspace card, its highlights, labels
//! and tags, the key hints and the window being dragged. They are drawn
//! into bitmaps once and composited by DWM as thumbnails, scaled to
//! wherever they are on each frame.
//!
//! A card is drawn over its windows' previews: where a preview goes the
//! card is cut out, and titles and borders sit on top of the preview's
//! edges. GDI can't draw onto translucent pixels, so text and icons are
//! drawn opaquely into scratch bitmaps first and blended in.

use std::collections::HashMap;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{COLORREF, HWND, RECT},
    Graphics::Gdi::{
      CreateFontW, DeleteObject, DrawTextW, SelectObject, SetBkMode,
      SetTextColor, ANTIALIASED_QUALITY, CLIP_DEFAULT_PRECIS,
      DEFAULT_CHARSET, DRAW_TEXT_FORMAT, DT_CALCRECT, DT_CENTER,
      DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
      FW_NORMAL, FW_SEMIBOLD, HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
    },
    UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL, HICON},
  },
};

use crate::{
  overview_layout::{
    CardLook, CardMetrics, Placement, RectF, LABEL_HEIGHT,
  },
  paint,
  surface::Surface,
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

  /// Whether this is the focused workspace.
  pub is_focused: bool,

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
  } else if picture.is_focused {
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
      fonts.get(units.px(12.0), false),
      "Drop windows here",
      &metrics.tile_area().to_rect(),
      color,
      DT_CENTER,
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
    fonts.get(units.px(12.0), picture.is_focused),
    &picture.label,
    &RectF::new(label_left, 0.0, half, header).to_rect(),
    if picture.is_focused {
      style.accent
    } else {
      style.subtext
    },
    DT_LEFT,
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
  let radius = round(units.px(radius));
  let opacity = tile.opacity;

  if tile.has_preview {
    surface.canvas().erase_rounded_rect(&rect, radius);
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

/// A rounded rect drawn once, then stretched to any size by showing its
/// corners as they are and stretching the rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SliceLook {
  /// Corner radius, in pixels.
  pub radius: i32,
  pub fill: Color,
  pub border: Color,
  pub border_width: i32,
}

impl SliceLook {
  /// A ring of `width` pixels in `color`, rounded by `radius`.
  pub fn ring(radius: i32, width: i32, color: Color) -> Self {
    Self {
      radius,
      fill: Color { a: 0, ..color },
      border: color,
      border_width: width,
    }
  }

  /// Pixels of each corner of its picture: room for the rounding and the
  /// border.
  pub fn corner(&self) -> i32 {
    self.radius.max(self.border_width).max(1) + 1
  }

  /// Whether it is more than a ring.
  pub fn is_filled(&self) -> bool {
    self.fill.a > 0
  }
}

/// Draws `look` into `surface`, `2 * corner + SLICE_MIDDLE` pixels square.
pub(crate) fn draw_slices(surface: &mut Surface, look: &SliceLook) {
  surface.pixels().fill(0);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  let mut canvas = surface.canvas();

  if look.is_filled() {
    canvas.fill_rounded_rect(&rect, look.radius, look.fill);
  }
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
    to_f32(text_width(fonts.get(px(font_size), false), &tag.text));

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
    fonts.get(px(font_size), false),
    &tag.text,
    &Rect::from_ltrb(
      round(px(padding + icon + gap)),
      0,
      surface.width - round(px(padding)),
      surface.height,
    ),
    style.text,
    DT_LEFT,
  );
}

/// Padding and font size of a workspace's name shown beside its card,
/// in logical pixels.
const LABEL: (f32, f32) = (9.0, 11.5);

/// A workspace's name shown beside its card.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LabelPicture {
  pub text: String,
  pub is_focused: bool,
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
    fonts.get(units.px(font_size), label.is_focused),
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
    fonts.get(units.px(font_size), label.is_focused),
    &label.text,
    &Rect::from_ltrb(round(left), 0, surface.width, surface.height),
    if label.is_focused {
      style.accent
    } else {
      with_alpha(style.text, 0.9)
    },
    DT_LEFT,
  );
}

/// Padding, icon size, gap between icons and font size of a pill on the
/// exposé's bar, and the most icons it shows, in logical pixels.
const PILL: (f32, f32, f32, f32) = (14.0, 16.0, 4.0, 12.5);
const PILL_ICONS: usize = 3;

/// A workspace's pill on the exposé's bar.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PillPicture {
  pub label: String,
  pub is_focused: bool,
  pub is_special: bool,
  pub is_new: bool,

  /// Its windows, whose icons follow its name.
  pub windows: Vec<isize>,
}

impl PillPicture {
  fn text(&self) -> String {
    if self.is_new {
      format!("+  {}", self.label)
    } else {
      self.label.clone()
    }
  }
}

/// Width `pill` takes on the bar, in pixels.
pub(crate) fn pill_width(
  pill: &PillPicture,
  style: &OverviewStyle,
  scale_factor: f32,
) -> f32 {
  let units = Units { scale_factor };
  let mut fonts = Fonts::new(&style.font_family);
  let (padding, icon, gap, font_size) = PILL;

  let text = to_f32(text_width(
    fonts.get(units.px(font_size), pill.is_focused),
    &pill.text(),
  ));
  let icons = pill.windows.len().min(PILL_ICONS);
  #[allow(clippy::cast_precision_loss)]
  let icons = if icons > 0 {
    units.px(gap * 2.0) + icons as f32 * units.px(icon + gap)
  } else {
    0.0
  };
  let diamond = if pill.is_special { units.px(14.0) } else { 0.0 };

  units.px(2.0 * padding) + diamond + text + icons
}

/// Draws every pill of the exposé's bar into `surface`, each in its rect
/// of `rects`: its name, then the icons of its first windows if they fit.
pub(crate) fn draw_pills(
  surface: &mut Surface,
  pills: &[PillPicture],
  rects: &[RectF],
  style: &OverviewStyle,
  scale_factor: f32,
  notify: (HWND, u32),
) {
  let units = Units { scale_factor };
  let (padding, icon, gap, font_size) = PILL;
  surface.pixels().fill(0);

  let mut fonts = Fonts::new(&style.font_family);
  for (pill, rect) in pills.iter().zip(rects) {
    let natural = pill_width(pill, style, scale_factor);
    let mut left =
      rect.x + (rect.w - natural.min(rect.w)) / 2.0 + units.px(padding);
    let center_y = rect.y + rect.h / 2.0;

    if pill.is_special {
      left += draw_diamond(surface, (left, center_y), units, style.accent);
    }

    // Icons only when nothing had to be shrunk to fit.
    let icons = if natural <= rect.w + 0.5 {
      pill.windows.len().min(PILL_ICONS)
    } else {
      0
    };
    #[allow(clippy::cast_precision_loss)]
    let icons_width = if icons > 0 {
      units.px(gap * 2.0) + icons as f32 * units.px(icon + gap)
    } else {
      0.0
    };

    let text_right = rect.x + rect.w - units.px(padding) - icons_width;
    let color = if pill.is_new {
      with_alpha(style.subtext, 0.75)
    } else if pill.is_focused {
      style.text
    } else {
      with_alpha(style.text, 0.82)
    };
    draw_text(
      surface,
      fonts.get(units.px(font_size), pill.is_focused),
      &pill.text(),
      &RectF::new(left, rect.y, (text_right - left).max(0.0), rect.h)
        .to_rect(),
      color,
      DT_LEFT,
    );

    let mut x = text_right + units.px(gap * 2.0);
    for hwnd in pill.windows.iter().take(icons) {
      let icon_rect = RectF::new(
        x,
        center_y - units.px(icon) / 2.0,
        units.px(icon),
        units.px(icon),
      );
      draw_window_icon(surface, *hwnd, &icon_rect.to_rect(), 0.9, notify);
      x += units.px(icon + gap);
    }
  }
}

/// Font size of the note shown where a workspace has no windows, in
/// logical pixels.
const NOTE_FONT: f32 = 15.0;

/// Pixel size of the note `text`.
pub(crate) fn note_size(
  text: &str,
  style: &OverviewStyle,
  scale_factor: f32,
) -> (i32, i32) {
  let px = |length: f32| length * scale_factor;
  let mut fonts = Fonts::new(&style.font_family);
  let width = text_width(fonts.get(px(NOTE_FONT), false), text);
  (
    width.max(1) + round(px(8.0)),
    round(px(NOTE_FONT * 2.0)).max(1),
  )
}

/// Draws the note `text` into `surface`, `note_size` big.
pub(crate) fn draw_note(
  surface: &mut Surface,
  text: &str,
  style: &OverviewStyle,
  scale_factor: f32,
) {
  surface.pixels().fill(0);
  let mut fonts = Fonts::new(&style.font_family);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  draw_text(
    surface,
    fonts.get(NOTE_FONT * scale_factor, false),
    text,
    &rect,
    with_alpha(style.subtext, 0.8),
    DT_CENTER,
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

/// Draws `text`, centered, into `surface`, on a pill of the cards' color
/// so it reads over whatever is behind it.
pub(crate) fn draw_hint(
  surface: &mut Surface,
  text: &str,
  color: Color,
  scale_factor: f32,
  style: &OverviewStyle,
) {
  surface.pixels().fill(0);

  let mut fonts = Fonts::new(&style.font_family);
  let font = fonts.get(10.5 * scale_factor, false);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);

  let width = (text_width(font, text) + round(28.0 * scale_factor))
    .min(surface.width);
  let pill =
    Rect::from_xy((surface.width - width) / 2, 0, width, surface.height);
  let radius = surface.height / 2;
  {
    let mut canvas = surface.canvas();
    canvas.fill_rounded_rect(&pill, radius, with_alpha(style.card, 0.9));
    canvas.stroke_rounded_rect(
      &pill,
      radius,
      round(scale_factor).max(1),
      with_alpha(style.accent, 0.16),
    );
  }

  draw_text(surface, font, text, &rect, color, DT_CENTER);
}

/// Width `text` takes on one line in `font`.
fn text_width(font: HFONT, text: &str) -> i32 {
  let mut text = text.encode_utf16().collect::<Vec<_>>();
  let Some(scratch) = Surface::new(1, 1) else {
    return 0;
  };
  let mut bounds = RECT::default();

  // SAFETY: `scratch` owns a valid DC; the old font is restored before it
  // is dropped.
  unsafe {
    let old_font = SelectObject(scratch.dc, font);
    DrawTextW(
      scratch.dc,
      &mut text,
      &raw mut bounds,
      DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX,
    );
    SelectObject(scratch.dc, old_font);
  }

  bounds.right - bounds.left
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
    fonts.get(font_size, false),
    title,
    &Rect::from_ltrb(
      strip.left + inset,
      strip.top,
      strip.right - inset,
      strip.bottom,
    ),
    fade(style.text, opacity),
    DT_LEFT,
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

/// Fonts of one family, created as they are asked for and deleted on drop.
struct Fonts {
  family: Vec<u16>,
  created: HashMap<(i32, bool), HFONT>,
}

impl Fonts {
  fn new(family: &str) -> Self {
    Self {
      family: family.encode_utf16().chain(std::iter::once(0)).collect(),
      created: HashMap::new(),
    }
  }

  /// The font `size` pixels high, semi-bold if `is_bold`. Grayscale
  /// anti-aliased, since its coverage is used as a mask.
  fn get(&mut self, size: f32, is_bold: bool) -> HFONT {
    let height = round(size).max(1);
    let family = &self.family;

    *self.created.entry((height, is_bold)).or_insert_with(|| {
      let weight = if is_bold { FW_SEMIBOLD } else { FW_NORMAL };

      // SAFETY: `family` is a null-terminated wide string that outlives
      // the call. The font is deleted in `Drop`.
      unsafe {
        CreateFontW(
          // Negative: character height rather than cell height.
          -height,
          0,
          0,
          0,
          i32::try_from(weight.0).unwrap_or(400),
          0,
          0,
          0,
          u32::from(DEFAULT_CHARSET.0),
          u32::from(OUT_DEFAULT_PRECIS.0),
          u32::from(CLIP_DEFAULT_PRECIS.0),
          u32::from(ANTIALIASED_QUALITY.0),
          0,
          PCWSTR(family.as_ptr()),
        )
      }
    })
  }
}

impl Drop for Fonts {
  fn drop(&mut self) {
    for font in self.created.values() {
      // SAFETY: Created in `get` and deleted once.
      unsafe {
        let _ = DeleteObject(*font);
      }
    }
  }
}

/// Blends `text` in `color` into `rect` of `surface`, vertically
/// centered, aligned by `align`, and cut off with an ellipsis if it
/// doesn't fit.
fn draw_text(
  surface: &mut Surface,
  font: HFONT,
  text: &str,
  rect: &Rect,
  color: Color,
  align: DRAW_TEXT_FORMAT,
) {
  let mut text = text.encode_utf16().collect::<Vec<_>>();
  if text.is_empty() || color.a == 0 {
    return;
  }

  let Some(mut scratch) = Surface::new(rect.width(), rect.height()) else {
    return;
  };

  // SAFETY: `scratch` owns a valid DC with its bitmap selected; the old
  // font is restored before it is dropped.
  unsafe {
    let old_font = SelectObject(scratch.dc, font);
    SetBkMode(scratch.dc, TRANSPARENT);
    SetTextColor(scratch.dc, COLORREF(0x00ff_ffff));

    let mut bounds = RECT {
      right: rect.width(),
      bottom: rect.height(),
      ..Default::default()
    };
    DrawTextW(
      scratch.dc,
      &mut text,
      &raw mut bounds,
      align | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
    );
    SelectObject(scratch.dc, old_font);
  }

  let mask = paint::mask_from_white_on_black(scratch.pixels());
  surface.canvas().blend_mask(
    rect.left,
    rect.top,
    &mask,
    rect.width(),
    color,
  );
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
