//! The shadow pictures `ShadowVisual`s stretch over their windows, one per
//! `ShadowStyle`, painted once and kept in composition surfaces.
//!
//! Lives on the composition thread with the graphics device it draws
//! through (see `graphics_device`'s threading notes).

use std::cell::RefCell;

use windows::{
  core::ComInterface,
  Graphics::{
    DirectX::{DirectXAlphaMode, DirectXPixelFormat},
    SizeInt32,
  },
  Win32::{
    Foundation::POINT,
    Graphics::{
      Direct2D::{
        Common::{
          D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT, D2D_RECT_F,
          D2D_SIZE_U,
        },
        ID2D1DeviceContext, D2D1_ANTIALIAS_MODE_ALIASED,
        D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
        D2D1_BITMAP_PROPERTIES,
      },
      Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
    System::WinRT::Composition::ICompositionDrawingSurfaceInterop,
  },
  UI::Composition::{CompositionDrawingSurface, Compositor},
};

use super::graphics_device::{self, with_graphics_device};
use crate::window_shadow::{ShadowImage, ShadowStyle};

/// A style's shadow picture, with its nine-grid insets.
struct CachedShadow {
  style: ShadowStyle,
  surface: CompositionDrawingSurface,
  insets: [i32; 4],
}

thread_local! {
  /// Every style drawn so far: a handful at most, one per window state,
  /// DPI and corner style in use.
  static SHADOWS: RefCell<Vec<CachedShadow>> = const { RefCell::new(Vec::new()) };
}

/// The surface holding `style`'s shadow picture, with its left, top, right
/// and bottom nine-grid insets, painted on first use.
pub(crate) fn surface_for(
  compositor: &Compositor,
  style: ShadowStyle,
) -> windows::core::Result<(CompositionDrawingSurface, [i32; 4])> {
  let cached = SHADOWS.with(|cache| {
    cache
      .borrow()
      .iter()
      .find(|shadow| shadow.style == style)
      .map(|shadow| (shadow.surface.clone(), shadow.insets))
  });

  if let Some(cached) = cached {
    return Ok(cached);
  }

  let image = style.paint();
  let surface = with_graphics_device(compositor, |device| {
    let surface = device.composition.CreateDrawingSurface2(
      SizeInt32 {
        Width: image.width,
        Height: image.height,
      },
      DirectXPixelFormat::B8G8R8A8UIntNormalized,
      DirectXAlphaMode::Premultiplied,
    )?;
    draw_into(&surface, &image)?;
    Ok(surface)
  })
  // Almost always a lost GPU device, which the next attempt rebuilds.
  .inspect_err(|_| graphics_device::reset())?;

  SHADOWS.with(|cache| {
    cache.borrow_mut().push(CachedShadow {
      style,
      surface: surface.clone(),
      insets: image.insets,
    });
  });

  Ok((surface, image.insets))
}

/// Repaints every cached picture, whose surfaces outlive a replaced
/// rendering device but whose contents don't.
pub(crate) fn redraw_cached(compositor: &Compositor) {
  let shadows = SHADOWS.with(|cache| {
    cache
      .borrow()
      .iter()
      .map(|shadow| (shadow.style, shadow.surface.clone()))
      .collect::<Vec<_>>()
  });

  for (style, surface) in shadows {
    let image = style.paint();
    let result =
      with_graphics_device(compositor, |_| draw_into(&surface, &image));

    if let Err(err) = result {
      tracing::warn!("Shadow redraw after device loss failed: {err}.");
    }
  }
}

/// Copies `image` into `surface`, which is its size.
fn draw_into(
  surface: &CompositionDrawingSurface,
  image: &ShadowImage,
) -> windows::core::Result<()> {
  let interop: ICompositionDrawingSurfaceInterop = surface.cast()?;
  let mut origin = POINT::default();

  // SAFETY: `interop` wraps a live surface; `origin` outlives the call,
  // and the requested interface is the one `BeginDraw` documents.
  let context: ID2D1DeviceContext =
    unsafe { interop.BeginDraw(None, &raw mut origin) }?;

  let drawn = draw(&context, origin, image);

  // SAFETY: Paired with the `BeginDraw` above, and must run even when the
  // draw failed: a surface left open wedges every later draw.
  let ended = unsafe { interop.EndDraw() };

  ended.and(drawn)
}

/// Draws `image` at `origin` of `context`'s atlas, clipped to its own
/// region of it (see `wallpaper_surface::draw`).
fn draw(
  context: &ID2D1DeviceContext,
  origin: POINT,
  image: &ShadowImage,
) -> windows::core::Result<()> {
  #[allow(clippy::cast_precision_loss)]
  let destination = D2D_RECT_F {
    left: origin.x as f32,
    top: origin.y as f32,
    right: (origin.x + image.width) as f32,
    bottom: (origin.y + image.height) as f32,
  };
  let properties = D2D1_BITMAP_PROPERTIES {
    pixelFormat: D2D1_PIXEL_FORMAT {
      format: DXGI_FORMAT_B8G8R8A8_UNORM,
      alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
    },
    dpiX: 96.0,
    dpiY: 96.0,
  };
  let size = D2D_SIZE_U {
    width: u32::try_from(image.width).unwrap_or_default(),
    height: u32::try_from(image.height).unwrap_or_default(),
  };

  // SAFETY: `context` is the live context `BeginDraw` returned, `pixels`
  // holds `width * height` 32-bit pixels at a pitch of `width * 4`, and
  // every pointer outlives its call. Pinning DPI to 96 makes one DIP one
  // pixel, as in `wallpaper_surface::draw`.
  unsafe {
    context.SetDpi(96.0, 96.0);
    let bitmap = context.CreateBitmap(
      size,
      Some(image.pixels.as_ptr().cast()),
      size.width * 4,
      &raw const properties,
    )?;

    context.PushAxisAlignedClip(
      &raw const destination,
      D2D1_ANTIALIAS_MODE_ALIASED,
    );
    context.Clear(None);
    context.DrawBitmap(
      &bitmap,
      Some(&raw const destination),
      1.0,
      D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
      None,
    );
    context.PopAxisAlignedClip();
  }

  Ok(())
}
