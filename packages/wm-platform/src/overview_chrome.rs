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
      CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
      DeleteObject, DrawTextW, GdiFlush, SelectObject, SetBkMode,
      SetTextColor, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER,
      BI_RGB, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DIB_RGB_COLORS,
      DRAW_TEXT_FORMAT, DT_CALCRECT, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT,
      DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, FW_NORMAL,
      FW_SEMIBOLD, HBITMAP, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS,
      TRANSPARENT,
    },
    UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL, HICON},
  },
};

use crate::{
  overview_layout::{CardMetrics, RectF},
  paint::{self, Canvas},
  window_icons, Color, OverviewStyle, Rect,
};

/// Height and font size of the title strip under a window on a card.
const TILE_CAPTION: (f32, f32) = (16.0, 9.0);

/// Height and font size of the title strip of the dragged window.
const GHOST_CAPTION: (f32, f32) = (18.0, 9.0);

/// Size of the dragged window's picture, in logical pixels.
pub(crate) const GHOST_SIZE: (f32, f32) = (200.0, 124.0);

/// Height of the key hints line, in logical pixels.
pub(crate) const HINT_HEIGHT: f32 = 20.0;

/// A 32-bit top-down DIB section selected into its own memory DC.
pub(crate) struct Surface {
  pub dc: HDC,
  bitmap: HBITMAP,
  old_bitmap: HGDIOBJ,
  bits: *mut u32,
  pub width: i32,
  pub height: i32,
}

impl Surface {
  pub fn new(width: i32, height: i32) -> Option<Self> {
    if width <= 0 || height <= 0 {
      return None;
    }

    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>())
          .ok()?,
        biWidth: width,
        // Negative height: top-down rows.
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };

    // SAFETY: `info` describes a valid 32-bit DIB and outlives the calls.
    // The DC and bitmap are released in `Drop`, or below on failure.
    unsafe {
      let dc = CreateCompatibleDC(None);
      if dc.is_invalid() {
        return None;
      }

      let mut bits = std::ptr::null_mut();
      let Ok(bitmap) = CreateDIBSection(
        dc,
        &raw const info,
        DIB_RGB_COLORS,
        &raw mut bits,
        None,
        0,
      ) else {
        let _ = DeleteDC(dc);
        return None;
      };

      Some(Self {
        dc,
        bitmap,
        old_bitmap: SelectObject(dc, bitmap),
        bits: bits.cast(),
        width,
        height,
      })
    }
  }

  pub fn pixels(&mut self) -> &mut [u32] {
    let len = usize::try_from(self.width * self.height).unwrap_or(0);

    // SAFETY: The DIB section holds `width * height` 32-bit pixels, owned
    // by `bitmap` until `Drop`, and GDI has finished drawing into it.
    unsafe {
      let _ = GdiFlush();
      std::slice::from_raw_parts_mut(self.bits, len)
    }
  }

  pub fn canvas(&mut self) -> Canvas<'_> {
    let (width, height) = (self.width, self.height);
    Canvas {
      pixels: self.pixels(),
      width,
      height,
    }
  }
}

impl Drop for Surface {
  fn drop(&mut self) {
    // SAFETY: Both handles were created in `new` and are released once.
    unsafe {
      SelectObject(self.dc, self.old_bitmap);
      let _ = DeleteObject(self.bitmap);
      let _ = DeleteDC(self.dc);
    }
  }
}

/// Everything a card's picture depends on, so it is only redrawn when
/// something in it changes.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct CardPicture {
  /// Scale it is drawn at. Everything else is in the coordinates of a
  /// card at scale 1.
  pub scale: f32,

  pub label: String,

  /// Whether this is the focused workspace.
  pub is_focused: bool,

  /// Whether it is the "+" card of a workspace yet to be activated.
  pub is_new: bool,

  pub is_hovered: bool,
  pub is_drop_target: bool,
  pub is_selected: bool,

  pub window_count: usize,

  /// Handles of minimized windows, shown as icons in the header.
  pub minimized: Vec<isize>,

  pub tiles: Vec<TilePicture>,

  /// Bumped as window icons arrive, so cards showing them redraw.
  pub icons: u64,
}

/// A window on a card.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TilePicture {
  pub hwnd: isize,
  pub rect: RectF,
  pub title: String,

  /// Whether its live preview shows; the card is cut out for it.
  pub has_preview: bool,

  pub opacity: f32,
  pub border_width: f32,
  pub border: Color,
}

/// Pixel size of a card drawn at `scale`.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn card_size(metrics: &CardMetrics, scale: f32) -> (i32, i32) {
  (
    (metrics.width * scale).ceil() as i32,
    (metrics.height * scale).ceil() as i32,
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

/// Converts lengths into the pixels of a picture drawn at some scale.
#[derive(Clone, Copy)]
struct Units {
  scale: f32,
  scale_factor: f32,
}

impl Units {
  /// `length` logical pixels.
  fn px(self, length: f32) -> f32 {
    length * self.scale_factor * self.scale
  }

  /// `rect`, given in the coordinates of a card at scale 1.
  fn rect(self, rect: &RectF) -> Rect {
    RectF::new(
      rect.x * self.scale,
      rect.y * self.scale,
      rect.w * self.scale,
      rect.h * self.scale,
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
  let units = Units {
    scale: picture.scale,
    scale_factor: metrics.scale_factor,
  };

  surface.pixels().fill(0);
  let card = Rect::from_xy(0, 0, surface.width, surface.height);
  let radius = round(units.px(14.0));

  let (fill, border, border_width) = if picture.is_drop_target {
    (with_alpha(style.accent, 0.22), style.accent, 3.0)
  } else if picture.is_focused {
    (with_alpha(style.accent, 0.12), style.accent, 1.0)
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
  draw_header(surface, &mut fonts, picture, units, style, notify);

  if picture.is_new {
    let color = if picture.is_drop_target || picture.is_selected {
      style.accent
    } else {
      with_alpha(style.subtext, 0.5)
    };
    draw_plus(surface, &units.rect(&metrics.tile_area()), units, color);
  }

  // Previews are cut out in stacking order, so a floating window's hole
  // clears what was drawn for the tiled window under it.
  for tile in &picture.tiles {
    draw_tile(surface, &mut fonts, tile, units, style, notify);
  }

  if picture.is_selected {
    surface.canvas().stroke_rounded_rect(
      &card,
      radius,
      round(units.px(3.0)).max(1),
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

/// Draws a card's name on the left, its window count on the right, and
/// its minimized windows (nothing to draw them with) as small icons
/// beside the count.
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

  draw_text(
    surface,
    fonts.get(units.px(12.0), picture.is_focused),
    &picture.label,
    &RectF::new(units.px(12.0), 0.0, half, header).to_rect(),
    if picture.is_focused {
      style.accent
    } else {
      style.subtext
    },
    DT_LEFT,
  );

  if picture.is_new {
    draw_text(
      surface,
      fonts.get(units.px(10.0), false),
      "new",
      &RectF::new(half, 0.0, half - units.px(12.0), header).to_rect(),
      with_alpha(style.subtext, 0.6),
      DT_RIGHT,
    );
    return;
  }

  if picture.window_count == 0 {
    return;
  }

  let count = picture.window_count.to_string();
  let font = fonts.get(units.px(10.0), false);
  let count_rect = RectF::new(half, 0.0, half - units.px(12.0), header);
  draw_text(
    surface,
    font,
    &count,
    &count_rect.to_rect(),
    with_alpha(style.subtext, 0.6),
    DT_RIGHT,
  );

  let mut x = count_rect.right()
    - to_f32(measure_text(surface.dc, font, &count))
    - units.px(6.0);
  for hwnd in &picture.minimized {
    x -= units.px(16.0);
    let icon =
      RectF::new(x, units.px(7.0), units.px(14.0), units.px(14.0));
    draw_window_icon(surface, *hwnd, &icon.to_rect(), 0.6, notify);
  }
}

/// Draws a window on a card: a hole for its preview (or its icon when it
/// has none), its title along the bottom, and a border.
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

  draw_caption(
    surface,
    fonts,
    &tile.title,
    &rect,
    (
      units.px(TILE_CAPTION.0),
      units.px(TILE_CAPTION.1),
      units.px(4.0),
    ),
    style,
    opacity,
  );

  surface.canvas().stroke_rounded_rect(
    &rect,
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

/// Width of `text` in `font`, measured on `dc`.
fn measure_text(dc: HDC, font: HFONT, text: &str) -> i32 {
  let mut text = text.encode_utf16().collect::<Vec<_>>();

  // An empty slice's dangling pointer must never reach `DrawTextW`, which
  // reads through it on some systems (e.g. Wine).
  if text.is_empty() {
    return 0;
  }

  // SAFETY: `dc` and `font` are valid; the old font is restored.
  unsafe {
    let old_font = SelectObject(dc, font);
    let mut rect = RECT::default();
    DrawTextW(
      dc,
      &mut text,
      &raw mut rect,
      DT_LEFT | DT_SINGLELINE | DT_NOPREFIX | DT_CALCRECT,
    );
    SelectObject(dc, old_font);
    rect.right - rect.left
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
