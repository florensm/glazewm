//! The overview's pictures: each workspace card, the key hints and the
//! window being dragged. They are drawn into bitmaps once and composited
//! by DWM as thumbnails, scaled to wherever their card is on each frame.
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
      DEFAULT_CHARSET, DRAW_TEXT_FORMAT, DT_CENTER, DT_END_ELLIPSIS,
      DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL,
      FW_SEMIBOLD, HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
    },
    UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL, HICON},
  },
};

use crate::{
  overview_layout::{CardMetrics, RectF, ZOOMED_SCALE},
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

/// Height of the key hints line, in logical pixels.
pub(crate) const HINT_HEIGHT: f32 = 20.0;

/// Everything a card's picture depends on, so it is only redrawn when
/// something in it changes.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct CardPicture {
  pub label: String,

  /// Whether this is the focused workspace.
  pub is_focused: bool,

  /// Whether it is the "+" card of a workspace yet to be activated.
  pub is_new: bool,

  pub is_hovered: bool,
  pub is_drop_target: bool,
  pub is_selected: bool,

  /// Handles of minimized windows, shown as icons in the header.
  pub minimized: Vec<isize>,

  pub tiles: Vec<TilePicture>,

  /// Where its stacks' tab bars show, cut out like the previews, with the
  /// bars' corners.
  pub tab_bars: Vec<(RectF, CornerRadii)>,

  /// Bumped as window icons arrive, so cards showing them redraw.
  pub icons: u64,
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

/// Scale of a card's picture, relative to a card at scale 1: the largest a
/// card is shown at, so DWM only ever scales it down and its text and
/// borders stay sharp.
pub(crate) const PICTURE_SCALE: f32 = ZOOMED_SCALE;

/// Gap between a selected card and its selection ring, in logical pixels.
const SELECTION_GAP: f32 = 3.0;

/// Width of the selection ring, in logical pixels.
const SELECTION_WIDTH: f32 = 3.0;

/// Pixel size of a card's picture: the card at [`PICTURE_SCALE`], with
/// room around it for the selection ring.
///
/// The same however big the card is shown: DWM scales it, so moving the
/// selection never redraws a card. A picture redrawn at a new size
/// flickers, since the new bitmap and its thumbnail's source rect can't
/// be swapped in at once.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn card_size(metrics: &CardMetrics) -> (i32, i32) {
  let outset = ring_outset(metrics);
  (
    (metrics.width * PICTURE_SCALE).ceil() as i32 + 2 * outset,
    (metrics.height * PICTURE_SCALE).ceil() as i32 + 2 * outset,
  )
}

/// What a card's picture covers, in the coordinates of a card at scale 1:
/// the card and its selection ring.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn picture_area(metrics: &CardMetrics) -> RectF {
  let (width, height) = card_size(metrics);
  let outset = ring_outset(metrics) as f32 / PICTURE_SCALE;

  RectF::new(
    -outset,
    -outset,
    width as f32 / PICTURE_SCALE,
    height as f32 / PICTURE_SCALE,
  )
}

/// How far the selection ring reaches out of its card, in picture pixels.
fn ring_outset(metrics: &CardMetrics) -> i32 {
  round(
    (SELECTION_GAP + SELECTION_WIDTH)
      * metrics.scale_factor
      * PICTURE_SCALE,
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

/// Converts logical lengths, and positions on a card at scale 1, into the
/// pixels of the card's picture.
#[derive(Clone, Copy)]
struct Units {
  scale_factor: f32,

  /// Where the card's top-left corner is in the picture.
  origin: f32,
}

impl Units {
  #[allow(clippy::cast_precision_loss)]
  fn new(metrics: &CardMetrics) -> Self {
    Self {
      scale_factor: metrics.scale_factor,
      origin: ring_outset(metrics) as f32,
    }
  }

  /// `length` logical pixels.
  fn px(self, length: f32) -> f32 {
    length * self.scale_factor * PICTURE_SCALE
  }

  /// `rect`, given in the coordinates of a card at scale 1.
  fn rect(self, rect: &RectF) -> Rect {
    RectF::new(
      self.origin + rect.x * PICTURE_SCALE,
      self.origin + rect.y * PICTURE_SCALE,
      rect.w * PICTURE_SCALE,
      rect.h * PICTURE_SCALE,
    )
    .to_rect()
  }
}

/// Draws a card into `surface`, which must be `card_size` big.
///
/// `notify` is told when a window icon that wasn't cached yet arrives.
pub(crate) fn draw_card(
  surface: &mut Surface,
  picture: &CardPicture,
  metrics: &CardMetrics,
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let units = Units::new(metrics);

  surface.pixels().fill(0);
  let card =
    units.rect(&RectF::new(0.0, 0.0, metrics.width, metrics.height));
  let radius = round(units.px(14.0));

  let (fill, border, border_width) = if picture.is_drop_target {
    (with_alpha(style.accent, 0.22), style.accent, 4.0)
  } else if picture.is_focused {
    (with_alpha(style.accent, 0.12), style.accent, 2.0)
  } else if picture.is_hovered {
    (style.surface, with_alpha(style.accent, 0.35), 1.0)
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

  let mut fonts = Fonts::new(&style.font_family);
  draw_header(surface, &mut fonts, picture, &card, units, style, notify);

  if picture.is_new {
    let color = if picture.is_drop_target || picture.is_selected {
      style.accent
    } else {
      with_alpha(style.subtext, 0.5)
    };
    draw_plus(surface, &units.rect(&metrics.tile_area()), units, color);
  }

  {
    let mut canvas = surface.canvas();
    for (rect, radii) in &picture.tab_bars {
      canvas.erase_rect_with_corners(&units.rect(rect), *radii);
    }
  }

  // Previews are cut out in stacking order, so a floating window's hole
  // clears what was drawn for the tiled window under it.
  for tile in &picture.tiles {
    draw_tile(surface, &mut fonts, tile, units, style, notify);
  }

  // Around the card rather than on its edge, so it reads apart from the
  // focused workspace's border.
  if picture.is_selected {
    let outset = ring_outset(metrics);
    surface.canvas().stroke_rounded_rect(
      &card.inset(-outset),
      radius + outset,
      round(units.px(SELECTION_WIDTH)).max(1),
      style.accent,
    );
  }
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
  card: &Rect,
  units: Units,
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let (left, top) = (to_f32(card.left), to_f32(card.top));
  let header = units.px(28.0);
  let half = to_f32(card.width()) / 2.0;

  draw_text(
    surface,
    fonts.get(units.px(12.0), picture.is_focused),
    &picture.label,
    &RectF::new(left + units.px(12.0), top, half, header).to_rect(),
    if picture.is_focused {
      style.accent
    } else {
      style.subtext
    },
    DT_LEFT,
  );

  let mut x = to_f32(card.right) - units.px(10.0);
  for hwnd in &picture.minimized {
    x -= units.px(16.0);
    let icon =
      RectF::new(x, top + units.px(7.0), units.px(14.0), units.px(14.0));
    draw_window_icon(surface, *hwnd, &icon.to_rect(), 0.6, notify);
  }
}

/// Draws a window on a card: a hole for its preview (or its icon when it
/// has none), its title along the bottom, and a border around it and its
/// stack's tab bar.
fn draw_tile(
  surface: &mut Surface,
  fonts: &mut Fonts,
  tile: &TilePicture,
  units: Units,
  style: &OverviewStyle,
  notify: (HWND, u32),
) {
  let rect = units.rect(&tile.rect);
  let radius = round(units.px(4.0));
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
      (&rect, radius),
      (
        units.px(TILE_CAPTION.0),
        units.px(TILE_CAPTION.1),
        units.px(4.0),
      ),
      style,
      opacity,
    );
  }

  surface.canvas().stroke_rounded_rect(
    &units.rect(&tile.outline),
    radius,
    round(units.px(tile.border_width)).max(1),
    fade(tile.border, opacity),
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
  let radius = round(px(8.0));
  let mut fonts = Fonts::new(&style.font_family);

  draw_caption(
    surface,
    &mut fonts,
    title,
    (&rect, radius),
    (px(GHOST_CAPTION.0), px(GHOST_CAPTION.1), px(4.0)),
    style,
    1.0,
  );

  surface.canvas().stroke_rounded_rect(
    &rect,
    radius,
    round(px(2.0)).max(1),
    style.accent,
  );
}

/// Draws `text`, centered, into `surface`.
pub(crate) fn draw_hint(
  surface: &mut Surface,
  text: &str,
  color: Color,
  scale_factor: f32,
  style: &OverviewStyle,
) {
  surface.pixels().fill(0);

  let mut fonts = Fonts::new(&style.font_family);
  let rect = Rect::from_xy(0, 0, surface.width, surface.height);
  draw_text(
    surface,
    fonts.get(10.0 * scale_factor, false),
    text,
    &rect,
    color,
    DT_CENTER,
  );
}

/// Draws a window title on a strip along the bottom of `rect`, if `rect`
/// has room for one, rounded like `rect`'s corners (by `radius`).
/// `metrics` is the strip's height, font size and text inset.
fn draw_caption(
  surface: &mut Surface,
  fonts: &mut Fonts,
  title: &str,
  (rect, radius): (&Rect, i32),
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
  surface.canvas().fill_rect_with_corners(
    &strip,
    CornerRadii {
      top_left: 0,
      top_right: 0,
      bottom_right: radius,
      bottom_left: radius,
    },
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

#[cfg(test)]
mod tests {
  use windows::Win32::Foundation::HWND;

  use super::{
    card_size, draw_caption, draw_card, picture_area, ring_outset, round,
    CardPicture, Fonts, Units, PICTURE_SCALE, SELECTION_WIDTH,
  };
  use crate::{
    overview_layout::CardMetrics, surface::Surface, Color, OverviewStyle,
    Rect,
  };

  fn metrics() -> CardMetrics {
    CardMetrics::new(1920.0, 1040.0, 1.0)
  }

  fn style() -> OverviewStyle {
    let color = |a: u8| Color {
      r: 200,
      g: 100,
      b: 50,
      a,
    };

    OverviewStyle {
      backdrop_blur: 0.0,
      backdrop_tint: color(0),
      accent: color(255),
      card: color(230),
      surface: color(255),
      caption: color(200),
      text: color(255),
      subtext: color(255),
      search: color(255),
      font_family: "Segoe UI".to_string(),
      grid_columns: 5,
      open_duration_ms: 0,
      close_duration_ms: 0,
    }
  }

  fn card_picture(is_selected: bool) -> CardPicture {
    CardPicture {
      label: "1".to_string(),
      is_focused: false,
      is_new: false,
      is_hovered: false,
      is_drop_target: false,
      is_selected,
      minimized: Vec::new(),
      tiles: Vec::new(),
      tab_bars: Vec::new(),
      icons: 0,
    }
  }

  fn alpha(surface: &mut Surface, x: i32, y: i32) -> u32 {
    let width = surface.width;
    usize::try_from(y * width + x)
      .ok()
      .and_then(|index| surface.pixels().get(index).copied())
      .map_or(0, |pixel| pixel >> 24)
  }

  fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
  }

  #[test]
  #[allow(clippy::cast_precision_loss)]
  fn picture_covers_the_card_and_its_ring_pixel_for_pixel() {
    let metrics = metrics();
    let (width, height) = card_size(&metrics);
    let area = picture_area(&metrics);

    assert_close(area.w * PICTURE_SCALE, width as f32);
    assert_close(area.h * PICTURE_SCALE, height as f32);
    assert_close(-area.x * PICTURE_SCALE, ring_outset(&metrics) as f32);
    assert_close(-area.y * PICTURE_SCALE, ring_outset(&metrics) as f32);
    assert!(area.right() >= metrics.width);
    assert!(area.bottom() >= metrics.height);
  }

  #[test]
  fn selection_ring_sits_around_the_card_with_a_gap() {
    let metrics = metrics();
    let style = style();
    let (width, height) = card_size(&metrics);
    let outset = ring_outset(&metrics);
    let ring = round(Units::new(&metrics).px(SELECTION_WIDTH));
    let y = height / 2;

    for is_selected in [false, true] {
      let mut surface = Surface::new(width, height).expect("surface");
      draw_card(
        &mut surface,
        &card_picture(is_selected),
        &metrics,
        &style,
        (HWND(0), 0),
      );

      assert_eq!(alpha(&mut surface, ring / 2, y) == 255, is_selected);
      assert_eq!(alpha(&mut surface, (ring + outset) / 2, y), 0);
      assert!(alpha(&mut surface, outset + ring, y) > 0);
    }
  }

  #[test]
  fn caption_strip_follows_the_rounded_corners() {
    let style = style();
    let mut surface = Surface::new(200, 100).expect("surface");

    draw_caption(
      &mut surface,
      &mut Fonts::new(&style.font_family),
      "title",
      (&Rect::from_xy(0, 0, 200, 100), 12),
      (20.0, 9.0, 4.0),
      &style,
      1.0,
    );

    assert_eq!(alpha(&mut surface, 0, 99), 0);
    assert_eq!(alpha(&mut surface, 199, 99), 0);
    assert!(alpha(&mut surface, 0, 85) > 0);
    assert!(alpha(&mut surface, 100, 99) > 0);
  }
}
