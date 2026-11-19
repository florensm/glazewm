//! Text for the WM's own pictures (the overview's and the stack tab
//! bars'), laid out and rasterized by DirectWrite, the way modern Windows
//! apps draw theirs.
//!
//! Text is drawn white on black into a scratch bitmap through a Direct2D
//! DC render target, with grayscale anti-aliasing so its coverage can be
//! used as a mask, then blended into the picture in its color: the
//! pictures are translucent, which neither GDI nor `ClearType` can draw
//! onto.

use std::{cell::OnceCell, collections::HashMap};

use windows::{
  core::{w, HSTRING},
  Foundation::Numerics::Matrix3x2,
  Win32::{
    Foundation::{E_FAIL, RECT},
    Graphics::{
      Direct2D::{
        Common::{
          D2D1_ALPHA_MODE_IGNORE, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
          D2D_POINT_2F,
        },
        D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1Factory,
        ID2D1SolidColorBrush, D2D1_DRAW_TEXT_OPTIONS_CLIP,
        D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
        D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
        D2D1_RENDER_TARGET_TYPE_SOFTWARE, D2D1_RENDER_TARGET_USAGE_NONE,
        D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
      },
      DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection,
        IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
        DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_SEMI_BOLD,
        DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT,
        DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
        DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TEXT_METRICS,
        DWRITE_TRIMMING, DWRITE_TRIMMING_GRANULARITY_CHARACTER,
        DWRITE_WORD_WRAPPING_NO_WRAP,
      },
      Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
  },
};

use crate::{paint, surface::Surface, Color, Rect};

/// Where text sits across the rect it is drawn into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Align {
  Left,
  Center,
  Right,
}

impl From<Align> for DWRITE_TEXT_ALIGNMENT {
  fn from(align: Align) -> Self {
    match align {
      Align::Left => DWRITE_TEXT_ALIGNMENT_LEADING,
      Align::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
      Align::Right => DWRITE_TEXT_ALIGNMENT_TRAILING,
    }
  }
}

/// A font at one size and weight, set up for single lines: vertically
/// centered, cut off with an ellipsis where they don't fit. Draws nothing
/// if DirectWrite is unavailable.
#[derive(Clone)]
pub(crate) struct Font(Option<IDWriteTextFormat>);

/// The fonts of one family, created as they are first asked for.
pub(crate) struct Fonts {
  family: HSTRING,
  created: HashMap<(i32, bool), Font>,
}

impl Fonts {
  pub fn new(family: &str) -> Self {
    Self {
      family: HSTRING::from(family),
      created: HashMap::new(),
    }
  }

  /// The font `size` pixels high, semi-bold if `is_bold`.
  pub fn get(&mut self, size: f32, is_bold: bool) -> Font {
    // Keyed by tenths of a pixel, so sizes that only differ by rounding
    // share a font.
    #[allow(clippy::cast_possible_truncation)]
    let key = ((size * 10.0).round() as i32, is_bold);
    let family = &self.family;

    self
      .created
      .entry(key)
      .or_insert_with(|| {
        Font(
          with_dwrite(|dwrite| {
            create_format(dwrite, family, size.max(1.0), is_bold)
          })
          .flatten(),
        )
      })
      .clone()
  }
}

/// Family to fall back on when the one asked for isn't installed.
const FALLBACK_FAMILY: &str = "Segoe UI";

/// Whether font `family` is installed.
pub(crate) fn is_installed(family: &str) -> bool {
  with_dwrite(|dwrite| {
    system_fonts(dwrite)
      .is_some_and(|fonts| has_family(&fonts, &HSTRING::from(family)))
  })
  .unwrap_or(false)
}

fn system_fonts(dwrite: &IDWriteFactory) -> Option<IDWriteFontCollection> {
  let mut collection = None;
  // SAFETY: `collection` outlives the call.
  unsafe { dwrite.GetSystemFontCollection(&raw mut collection, false) }
    .ok()?;
  collection
}

fn has_family(fonts: &IDWriteFontCollection, family: &HSTRING) -> bool {
  let (mut index, mut exists) = (0, false.into());
  // SAFETY: `family`, `index` and `exists` outlive the call.
  unsafe { fonts.FindFamilyName(family, &raw mut index, &raw mut exists) }
    .is_ok()
    && exists.as_bool()
}

/// `family` if it is installed, else the fallback, else the first family
/// installed at all: a family that isn't installed lays out nothing.
fn installed_family(dwrite: &IDWriteFactory, family: &HSTRING) -> HSTRING {
  let Some(fonts) = system_fonts(dwrite) else {
    return family.clone();
  };
  let fallback = HSTRING::from(FALLBACK_FAMILY);
  if has_family(&fonts, family) {
    return family.clone();
  }
  if has_family(&fonts, &fallback) {
    return fallback;
  }

  // SAFETY: Plain DirectWrite queries on live interfaces; `name` outlives
  // the call that fills it.
  let first = unsafe {
    fonts
      .GetFontFamily(0)
      .and_then(|family| family.GetFamilyNames())
      .and_then(|names| {
        let length = names.GetStringLength(0)?;
        let mut name = vec![0; length as usize + 1];
        names.GetString(0, &mut name)?;
        Ok(HSTRING::from_wide(&name[..length as usize]))
      })
  };
  first
    .ok()
    .and_then(Result::ok)
    .unwrap_or_else(|| family.clone())
}

/// Creates a text format for single, vertically centered lines, trimmed
/// with an ellipsis.
fn create_format(
  dwrite: &IDWriteFactory,
  family: &HSTRING,
  size: f32,
  is_bold: bool,
) -> Option<IDWriteTextFormat> {
  let family = installed_family(dwrite, family);
  let weight = if is_bold {
    DWRITE_FONT_WEIGHT_SEMI_BOLD
  } else {
    DWRITE_FONT_WEIGHT_NORMAL
  };

  // SAFETY: Plain DirectWrite calls on live interfaces; the strings
  // outlive them.
  unsafe {
    let format = dwrite
      .CreateTextFormat(
        &family,
        None,
        weight,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        size,
        // The user's locale; a null name is not allowed.
        w!(""),
      )
      .ok()?;
    format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP).ok()?;
    format
      .SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)
      .ok()?;

    let ellipsis = dwrite.CreateEllipsisTrimmingSign(&format).ok()?;
    let trimming = DWRITE_TRIMMING {
      granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
      delimiter: 0,
      delimiterCount: 0,
    };
    format.SetTrimming(&raw const trimming, &ellipsis).ok()?;
    Some(format)
  }
}

/// What text is rasterized with: a DC render target and its white brush.
struct TextTarget {
  target: ID2D1DCRenderTarget,
  white: ID2D1SolidColorBrush,
}

impl TextTarget {
  fn new() -> windows::core::Result<Self> {
    // SAFETY: Plain factory and resource creation; nothing is borrowed
    // past the calls.
    unsafe {
      let d2d: ID2D1Factory =
        D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

      // Software first: the bitmaps are small and read back on the CPU
      // right away, which a GPU target only adds a copy to.
      let properties = |r#type| D2D1_RENDER_TARGET_PROPERTIES {
        r#type,
        pixelFormat: D2D1_PIXEL_FORMAT {
          format: DXGI_FORMAT_B8G8R8A8_UNORM,
          alphaMode: D2D1_ALPHA_MODE_IGNORE,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
      };
      let mut result = Err(windows::core::Error::from(E_FAIL));
      for r#type in [
        D2D1_RENDER_TARGET_TYPE_SOFTWARE,
        D2D1_RENDER_TARGET_TYPE_DEFAULT,
      ] {
        let properties = properties(r#type);
        result = d2d.CreateDCRenderTarget(&raw const properties);
        if result.is_ok() {
          break;
        }
      }
      let target = result?;
      target.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);

      let white = D2D1_COLOR_F {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
      };
      let white = target.CreateSolidColorBrush(&raw const white, None)?;

      Ok(Self { target, white })
    }
  }
}

thread_local! {
  /// Each `None` once creating it failed, which leaves the pictures
  /// without text rather than retrying on every draw.
  static DWRITE: OnceCell<Option<IDWriteFactory>> = const { OnceCell::new() };
  static TARGET: OnceCell<Option<TextTarget>> = const { OnceCell::new() };
}

/// Runs `f` with DirectWrite, created on first use.
fn with_dwrite<T>(f: impl FnOnce(&IDWriteFactory) -> T) -> Option<T> {
  DWRITE.with(|dwrite| {
    dwrite
      .get_or_init(|| {
        // SAFETY: Creates a factory; nothing is borrowed past the call.
        unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }
          .inspect_err(|err| tracing::warn!("Text unavailable: {err}"))
          .ok()
      })
      .as_ref()
      .map(f)
  })
}

/// Runs `f` with this thread's text target, created on first use.
fn with_target<T>(f: impl FnOnce(&TextTarget) -> T) -> Option<T> {
  TARGET.with(|target| {
    target
      .get_or_init(|| {
        TextTarget::new()
          .inspect_err(|err| tracing::warn!("Text unavailable: {err}"))
          .ok()
      })
      .as_ref()
      .map(f)
  })
}

/// Width `text` takes on one line in `font`, in whole pixels.
pub(crate) fn text_width(font: &Font, text: &str) -> i32 {
  let (Some(format), false) = (&font.0, text.is_empty()) else {
    return 0;
  };
  let text = text.encode_utf16().collect::<Vec<_>>();

  with_dwrite(|dwrite| {
    // SAFETY: `text` outlives the call; the layout is used right away.
    let metrics = unsafe {
      let layout = dwrite
        .CreateTextLayout(&text, format, f32::MAX, f32::MAX)
        .ok()?;
      let mut metrics = DWRITE_TEXT_METRICS::default();
      layout.GetMetrics(&raw mut metrics).ok()?;
      metrics
    };

    #[allow(clippy::cast_possible_truncation)]
    Some(metrics.widthIncludingTrailingWhitespace.ceil() as i32)
  })
  .flatten()
  .unwrap_or(0)
}

/// Blends `text` in `color` into `rect` of `surface`, vertically
/// centered, aligned by `align`, and cut off with an ellipsis if it
/// doesn't fit.
pub(crate) fn draw_text(
  surface: &mut Surface,
  font: &Font,
  text: &str,
  rect: &Rect,
  color: Color,
  align: Align,
) {
  let Some(format) = &font.0 else {
    return;
  };
  if text.is_empty() || color.a == 0 {
    return;
  }
  let Some(mut scratch) = Surface::new(rect.width(), rect.height()) else {
    return;
  };
  let text = text.encode_utf16().collect::<Vec<_>>();

  let layout = with_dwrite(|dwrite| {
    #[allow(clippy::cast_precision_loss)]
    let (width, height) = (rect.width() as f32, rect.height() as f32);

    // SAFETY: `text` outlives the call; the layout is used right away.
    unsafe {
      let layout =
        dwrite.CreateTextLayout(&text, format, width, height).ok()?;
      layout.SetTextAlignment(align.into()).ok()?;
      Some(layout)
    }
  })
  .flatten();
  let Some(layout) = layout else {
    return;
  };

  let drawn = with_target(|target| {
    let bounds = RECT {
      right: rect.width(),
      bottom: rect.height(),
      ..Default::default()
    };

    // SAFETY: `scratch` owns a valid DC with its bitmap selected for the
    // whole draw, and every other argument outlives its call.
    unsafe {
      let TextTarget { target, white } = target;
      target.BindDC(scratch.dc, &raw const bounds).ok()?;
      target.BeginDraw();
      let (identity, black) = (
        Matrix3x2::identity(),
        D2D1_COLOR_F {
          r: 0.0,
          g: 0.0,
          b: 0.0,
          a: 1.0,
        },
      );
      target.SetTransform(&raw const identity);
      target.Clear(Some(&raw const black));
      target.DrawTextLayout(
        D2D_POINT_2F { x: 0.0, y: 0.0 },
        &layout,
        white,
        D2D1_DRAW_TEXT_OPTIONS_CLIP,
      );
      target.EndDraw(None, None).ok()
    }
  })
  .flatten();

  if drawn.is_none() {
    return;
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

#[cfg(test)]
mod tests {
  use super::{draw_text, text_width, Align, Fonts};
  use crate::{surface::Surface, Color, Rect};

  #[test]
  fn measures_and_draws_a_line() {
    let mut fonts = Fonts::new("Segoe UI");
    let font = fonts.get(14.0, false);

    let short = text_width(&font, "ab");
    let long = text_width(&font, "abcdef");
    assert!(short > 0 && long > short);
    assert_eq!(text_width(&font, ""), 0);

    // Wine offers no DC render target to draw with.
    if super::with_target(|_| ()).is_none() {
      return;
    }
    let mut surface = Surface::new(120, 24).expect("surface");
    let white = Color {
      r: 255,
      g: 255,
      b: 255,
      a: 255,
    };
    draw_text(
      &mut surface,
      &font,
      "abcdef",
      &Rect::from_xy(0, 0, 120, 24),
      white,
      Align::Left,
    );
    assert!(surface.pixels().iter().any(|pixel| *pixel != 0));
  }
}
