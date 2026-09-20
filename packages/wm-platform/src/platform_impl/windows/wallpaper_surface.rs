//! Bakes the desktop wallpaper into a blurred, opaque composition surface.
//!
//! # Why this exists at all
//!
//! Acrylic samples whatever sits behind a window and blurs it every frame.
//! In a tiling layout nothing *is* behind a tiled window except the
//! wallpaper, so for the common case that live sampling reproduces, frame
//! after frame, an image that never changes. This module computes that
//! image once instead.
//!
//! Two things follow from baking rather than sampling, and both matter
//! more than the blur itself:
//!
//! - The surface can be **opaque**. A translucent surface forces DWM to
//!   blend it against everything beneath it on every composition pass; an
//!   opaque one lets DWM skip what's behind entirely. That removes a
//!   blending layer rather than making one cheaper.
//! - The blur happens **once**, at bake time, not per frame -- so the
//!   overlay's visual tree is a plain surface brush with no effect graph
//!   on it at all, unlike the acrylic path in `composition`.
//!
//! # Cropping
//!
//! One surface is baked per monitor, the full size of that monitor. Each
//! window's overlay shows the part of it matching the window's position,
//! by offsetting the shared brush -- a sprite offset, not a re-blur, so N
//! windows on a monitor cost one bake between them.
//!
//! Full resolution costs `width * height * 4` bytes of VRAM per surface
//! (~20 MB on a 3440x1440 display), and there is one surface per distinct
//! set of knobs -- normally one, or two when `focused_window` and
//! `other_windows` are configured differently. Baking at reduced
//! resolution and letting the brush scale it back up would be nearly free
//! visually, since the blur has already destroyed the detail, and is the
//! obvious lever if that ever matters.
//!
//! # Threading
//!
//! Everything here runs on the composition thread, reached through
//! `composition::run_on_composition_thread`. The cache and the graphics
//! device are thread-locals of that thread.

use std::{
  cell::RefCell,
  os::windows::ffi::OsStrExt,
  path::Path,
  sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, OnceLock,
  },
  time::{Duration, Instant},
};

use windows::{
  core::{ComInterface, PCWSTR},
  Foundation::{
    Numerics::{Matrix3x2, Vector2, Vector4},
    TypedEventHandler,
  },
  Graphics::{
    DirectX::{DirectXAlphaMode, DirectXPixelFormat},
    SizeInt32,
  },
  Win32::{
    Foundation::{GENERIC_READ, POINT},
    Graphics::{
      Direct2D::{
        CLSID_D2D12DAffineTransform, CLSID_D2D1Blend, CLSID_D2D1Border,
        CLSID_D2D1Composite, CLSID_D2D1Contrast, CLSID_D2D1Crop,
        CLSID_D2D1Exposure, CLSID_D2D1Flood, CLSID_D2D1GaussianBlur,
        CLSID_D2D1HighlightsShadows, CLSID_D2D1Opacity,
        CLSID_D2D1Saturation, CLSID_D2D1Turbulence,
        Common::{
          D2D1_BLEND_MODE_OVERLAY, D2D1_COLOR_F,
          D2D1_COMPOSITE_MODE_SOURCE_OVER, D2D_POINT_2F, D2D_RECT_F,
        },
        ID2D1Bitmap, ID2D1DeviceContext, ID2D1Effect,
        D2D1_2DAFFINETRANSFORM_PROP_INTERPOLATION_MODE,
        D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX,
        D2D1_ANTIALIAS_MODE_ALIASED, D2D1_BLEND_PROP_MODE,
        D2D1_BORDER_EDGE_MODE_CLAMP, D2D1_BORDER_EDGE_MODE_WRAP,
        D2D1_BORDER_PROP_EDGE_MODE_X, D2D1_BORDER_PROP_EDGE_MODE_Y,
        D2D1_CONTRAST_PROP_CONTRAST, D2D1_CROP_PROP_RECT,
        D2D1_EXPOSURE_PROP_EXPOSURE_VALUE, D2D1_FLOOD_PROP_COLOR,
        D2D1_GAUSSIANBLUR_OPTIMIZATION_QUALITY,
        D2D1_GAUSSIANBLUR_PROP_OPTIMIZATION,
        D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION,
        D2D1_HIGHLIGHTSANDSHADOWS_PROP_HIGHLIGHTS,
        D2D1_HIGHLIGHTSANDSHADOWS_PROP_SHADOWS,
        D2D1_INTERPOLATION_MODE_CUBIC, D2D1_INTERPOLATION_MODE_LINEAR,
        D2D1_OPACITY_PROP_OPACITY, D2D1_PROPERTY_TYPE,
        D2D1_PROPERTY_TYPE_ENUM, D2D1_PROPERTY_TYPE_FLOAT,
        D2D1_PROPERTY_TYPE_MATRIX_3X2, D2D1_PROPERTY_TYPE_UINT32,
        D2D1_PROPERTY_TYPE_VECTOR2, D2D1_PROPERTY_TYPE_VECTOR4,
        D2D1_SATURATION_PROP_SATURATION,
        D2D1_TURBULENCE_PROP_BASE_FREQUENCY,
        D2D1_TURBULENCE_PROP_NUM_OCTAVES, D2D1_TURBULENCE_PROP_SIZE,
      },
      Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO,
        MONITOR_DEFAULTTONEAREST,
      },
      Imaging::{
        GUID_WICPixelFormat32bppPBGRA, WICConvertBitmapSource,
        WICDecodeMetadataCacheOnLoad,
      },
    },
    System::WinRT::Composition::ICompositionDrawingSurfaceInterop,
    UI::WindowsAndMessaging::{
      GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
      SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    },
  },
  UI::Composition::{
    CompositionDrawingSurface, CompositionStretch,
    CompositionSurfaceBrush, Compositor,
  },
};

use super::{
  graphics_device::{self, with_graphics_device, GraphicsDevice},
  wallpaper::{DesktopSignature, MonitorWallpaper, WallpaperFit},
};
use crate::{BlurOverlayParams, Rect};

/// The subset of [`BlurOverlayParams`] rendered *into* the baked image,
/// and so the part that decides whether an existing bake can be reused.
///
/// Everything left out -- tint, opacity, corner radius, parallax -- is
/// applied live by the visual tree or by the crop, and changing one of
/// those must not throw away a surface that is still correct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BakeKnobs {
  pub blur_amount: f32,
  pub saturation: f32,
  pub exposure: f32,
  pub contrast: f32,
  pub highlights: f32,
  pub shadows: f32,
  pub grain: f32,
}

impl From<BlurOverlayParams> for BakeKnobs {
  fn from(params: BlurOverlayParams) -> Self {
    Self {
      blur_amount: params.blur_amount,
      saturation: params.saturation,
      exposure: params.exposure,
      contrast: params.contrast,
      highlights: params.highlights,
      shadows: params.shadows,
      grain: params.grain,
    }
  }
}

/// Everything that determines a baked surface's pixels.
///
/// Compared for exact equality to decide whether an existing bake can be
/// reused. The `f32` knobs are safe to compare that way for the same
/// reason `NativeBlurOverlay`'s setters are: they only ever change when a
/// caller passes a genuinely different, config-resolved number, never
/// through arithmetic that could drift.
#[derive(Clone, Debug, PartialEq)]
struct WallpaperKey {
  wallpaper: MonitorWallpaper,
  knobs: BakeKnobs,
}

/// A baked surface and the description it was baked from.
struct CachedSurface {
  key: WallpaperKey,
  surface: CompositionDrawingSurface,

  /// Value of [`CACHE_CLOCK`] when this entry was last handed out, so the
  /// cap below evicts the least recently used rather than an arbitrary
  /// one.
  last_used: u64,
}

/// Most baked surfaces kept alive at once.
///
/// A display needs two in normal use, since `focused_window` and
/// `other_windows` can carry different knobs, so this covers a couple of
/// them. The cap is not about steady state: it exists because changing a
/// knob produces a *new* key rather than replacing an old one, so every
/// config reload that touches the backdrop would otherwise leave its
/// predecessor's surface -- ~20 MB on a 3440x1440 display -- cached for
/// the life of the process.
///
/// Evicting is always safe: a surface a brush still points at stays alive
/// through that reference, and dropping it from the cache only means a
/// later request for the same key re-bakes instead of reusing it.
const MAX_CACHED_SURFACES: usize = 4;

/// Monotonic counter stamped onto entries as they are used.
static CACHE_CLOCK: AtomicU64 = AtomicU64::new(0);

thread_local! {
  /// Baked surfaces, one per distinct [`WallpaperKey`] -- in practice one
  /// per monitor, since every window resolves its knobs from the same
  /// config.
  ///
  /// A `Vec` rather than a map because the key is a structural description
  /// with no cheap hash and the list is monitor-count long; a linear scan of
  /// three entries on window creation is not worth a hashing story.
  static SURFACES: RefCell<Vec<CachedSurface>> =
    const { RefCell::new(Vec::new()) };
}

/// Bumped whenever something outside this module may have changed what the
/// desktop looks like: the wallpaper itself, or the display layout the
/// bakes are sized against.
///
/// Overlays compare their own stored value against this on each sync tick,
/// so the steady-state cost of noticing a wallpaper change is one relaxed
/// load per overlay -- nothing queries the shell or the filesystem until
/// the counter actually moves.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Marks every baked surface as possibly stale.
///
/// Cheap and safe to over-call: a bump only makes overlays re-query the
/// wallpaper, and an unchanged description still hits the same cached
/// surface without re-baking.
pub(crate) fn invalidate() {
  GENERATION.fetch_add(1, Ordering::Relaxed);
}

/// The current value of [`GENERATION`], for an overlay to store alongside
/// whatever it bound.
pub(crate) fn generation() -> u64 {
  GENERATION.load(Ordering::Relaxed)
}

/// How long a wallpaper change can go unnoticed when no broadcast arrives.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Process-start reference point for [`LAST_POLL_MS`], which stores an
/// offset rather than an `Instant` so the throttle check stays a plain
/// atomic load on the hot path.
static POLL_EPOCH: OnceLock<Instant> = OnceLock::new();

/// Milliseconds since [`POLL_EPOCH`] at the last poll.
static LAST_POLL_MS: AtomicU64 = AtomicU64::new(0);

/// The desktop settings seen at the last poll.
static LAST_SEEN: Mutex<Option<DesktopSignature>> = Mutex::new(None);

/// Re-checks the desktop wallpaper at most once per [`POLL_INTERVAL`],
/// bumping the generation when it has actually changed.
///
/// A backstop for the `WM_SETTINGCHANGE` hook rather than a replacement:
/// the broadcast is instant when it arrives, but the shell does not send
/// `SPI_SETDESKWALLPAPER` for every way a wallpaper can change -- the
/// Settings app, a slideshow rotation, and Windows Spotlight do not all
/// announce themselves alike -- and a backdrop that never notices its own
/// wallpaper is a far worse failure than a two-second delay.
///
/// Called from the per-tick sync path, so the throttled-out case is one
/// atomic load and a comparison, with no shell call.
pub(crate) fn poll_for_changes() {
  let elapsed = POLL_EPOCH.get_or_init(Instant::now).elapsed();

  #[allow(clippy::cast_possible_truncation)]
  let now_ms = elapsed.as_millis() as u64;
  #[allow(clippy::cast_possible_truncation)]
  let interval_ms = POLL_INTERVAL.as_millis() as u64;

  let last_ms = LAST_POLL_MS.load(Ordering::Relaxed);
  if now_ms.saturating_sub(last_ms) < interval_ms {
    return;
  }

  // Claims this interval's poll, so the other overlays syncing on the same
  // tick take the cheap path above instead of queueing on the lock behind
  // a shell query each of them would only repeat.
  if LAST_POLL_MS
    .compare_exchange(
      last_ms,
      now_ms,
      Ordering::Relaxed,
      Ordering::Relaxed,
    )
    .is_err()
  {
    return;
  }

  let signature = super::wallpaper::desktop_signature();

  let Ok(mut seen) = LAST_SEEN.lock() else {
    return;
  };

  if seen.as_ref() == Some(&signature) {
    return;
  }

  // The first poll establishes the baseline. Treating it as a change would
  // invalidate every surface moments after startup, re-baking each one to
  // produce the image it already had.
  let is_first = seen.is_none();
  *seen = Some(signature);

  if !is_first {
    invalidate();
  }
}

/// Returns a brush painting the blurred wallpaper for the monitor `rect`
/// sits on, baking one if no matching surface is cached.
///
/// The brush is positioned by the caller via [`set_crop`]: it is returned
/// aligned to the surface's top-left and unstretched, so an offset alone
/// selects which part of the monitor-sized image shows.
///
/// Must be called on the composition thread.
pub(crate) fn crop_brush(
  compositor: &Compositor,
  rect: &Rect,
  params: BlurOverlayParams,
) -> windows::core::Result<(CompositionSurfaceBrush, Rect)> {
  let monitor = monitor_bounds(rect);
  let surface = surface_for(compositor, &monitor, params.into())?;

  let brush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
  brush.SetStretch(CompositionStretch::None)?;

  // `Stretch::None` centers the surface in the sprite by default.
  // Anchoring both axes to the top-left instead is what makes
  // `SetOffset` mean "scroll the wallpaper", which is the whole cropping
  // mechanism.
  brush.SetHorizontalAlignmentRatio(0.0)?;
  brush.SetVerticalAlignmentRatio(0.0)?;

  set_crop(&brush, rect, &monitor, params.parallax);
  Ok((brush, monitor))
}

/// Points `brush` at the part of `monitor`'s baked wallpaper that lies
/// under `rect`, so the overlay shows exactly what the desktop shows
/// there.
///
/// Cheap enough for the per-move path: a single agile property write, no
/// device work and no re-bake.
pub(crate) fn set_crop(
  brush: &CompositionSurfaceBrush,
  rect: &Rect,
  monitor: &Rect,
  parallax: f32,
) {
  // At `parallax == 1.0` this is exactly the window's offset within its
  // monitor, so the image sits still against the desktop. Scaling it down
  // makes the image trail the window rather than track it, which is what
  // reads as the backdrop sitting further away.
  #[allow(clippy::cast_precision_loss)]
  let offset = Vector2 {
    X: (monitor.x() - rect.x()) as f32 * parallax,
    Y: (monitor.y() - rect.y()) as f32 * parallax,
  };

  if let Err(err) = brush.SetOffset(offset) {
    tracing::warn!("Wallpaper backdrop crop update failed: {err}.");
  }
}

/// Swaps `brush` onto another monitor's baked wallpaper, baking it if
/// needed. Called when a window is moved across displays.
///
/// Must be called on the composition thread.
pub(crate) fn rebind(
  compositor: &Compositor,
  brush: &CompositionSurfaceBrush,
  monitor: &Rect,
  knobs: BakeKnobs,
) -> windows::core::Result<()> {
  let surface = surface_for(compositor, monitor, knobs)?;
  brush.SetSurface(&surface)
}

/// The bounds of the monitor `rect` sits on.
///
/// Uses the rect's center rather than its origin so a window straddling
/// two displays takes the wallpaper of the one it is mostly on, matching
/// how the rest of the WM assigns windows to monitors.
pub(crate) fn monitor_bounds(rect: &Rect) -> Rect {
  let center = rect.center_point();
  let point = POINT {
    x: center.x,
    y: center.y,
  };

  // SAFETY: `MonitorFromPoint` accepts any point and, with
  // `MONITOR_DEFAULTTONEAREST`, always returns a valid monitor.
  let monitor =
    unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };

  let mut info = MONITORINFO {
    #[allow(clippy::cast_possible_truncation)]
    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
    ..Default::default()
  };

  // SAFETY: `monitor` is valid and `info.cbSize` is set as documented.
  let ok = unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool();
  if !ok {
    return virtual_screen();
  }

  Rect::from_ltrb(
    info.rcMonitor.left,
    info.rcMonitor.top,
    info.rcMonitor.right,
    info.rcMonitor.bottom,
  )
}

/// The bounding rectangle of every display, which is the frame
/// [`WallpaperFit::Span`] lays its image out across.
fn virtual_screen() -> Rect {
  // SAFETY: All four metrics are valid indices and return plain integers.
  unsafe {
    Rect::from_xy(
      GetSystemMetrics(SM_XVIRTUALSCREEN),
      GetSystemMetrics(SM_YVIRTUALSCREEN),
      GetSystemMetrics(SM_CXVIRTUALSCREEN),
      GetSystemMetrics(SM_CYVIRTUALSCREEN),
    )
  }
}

thread_local! {
  /// The last wallpaper description read for a monitor, and the generation
  /// it was read at.
  ///
  /// `MonitorWallpaper::query` is a `CoCreateInstance` plus several COM
  /// round-trips and a file stat, and `surface_for` needs the description on
  /// every call -- including the ones that go on to hit the surface cache and
  /// bake nothing at all. Since the only things that can change it also bump
  /// [`GENERATION`] (the shell broadcast and the poll both do), a description
  /// read at the current generation is still current by construction.
  ///
  /// Without this, switching focus between two windows whose backdrop knobs
  /// differ pays a full shell query per overlay -- on the WM thread, which
  /// blocks on the composition thread while it happens -- and a workspace
  /// switch pays it once per window at once.
  static WALLPAPER_MEMO: RefCell<Vec<(Rect, u64, MonitorWallpaper)>> =
    const { RefCell::new(Vec::new()) };
}

/// The wallpaper on `monitor`, re-reading it only when the generation has
/// moved since the last read.
fn wallpaper_for(monitor: &Rect) -> MonitorWallpaper {
  let current = generation();

  let memo = WALLPAPER_MEMO.with(|memo| {
    memo
      .borrow()
      .iter()
      .find(|(rect, gen, _)| rect == monitor && *gen == current)
      .map(|(_, _, wallpaper)| wallpaper.clone())
  });

  if let Some(wallpaper) = memo {
    return wallpaper;
  }

  let wallpaper = MonitorWallpaper::query(monitor, &virtual_screen());

  WALLPAPER_MEMO.with(|memo| {
    let mut memo = memo.borrow_mut();
    memo.retain(|(rect, _, _)| rect != monitor);
    memo.push((monitor.clone(), current, wallpaper.clone()));
  });

  wallpaper
}

/// Returns the cached surface for `monitor` at these knobs, baking one if
/// the wallpaper, the layout, or the knobs have changed since the last
/// bake.
///
/// Compares the knobs exactly, matching `WallpaperKey`'s own `PartialEq`
/// and `NativeBlurOverlay`'s setters: these are config-resolved numbers
/// that are either the same value or a different one, never the result of
/// arithmetic that could drift.
#[allow(clippy::float_cmp)]
fn surface_for(
  compositor: &Compositor,
  monitor: &Rect,
  knobs: BakeKnobs,
) -> windows::core::Result<CompositionDrawingSurface> {
  let key = WallpaperKey {
    wallpaper: wallpaper_for(monitor),
    knobs,
  };

  let now = CACHE_CLOCK.fetch_add(1, Ordering::Relaxed);

  let cached = SURFACES.with(|cache| {
    cache
      .borrow_mut()
      .iter_mut()
      .find(|entry| entry.key == key)
      .map(|entry| {
        entry.last_used = now;
        entry.surface.clone()
      })
  });

  if let Some(surface) = cached {
    return Ok(surface);
  }

  let surface = with_graphics_device(compositor, |device, is_new| {
    if is_new {
      subscribe_device_replaced(compositor, device)?;
    }
    bake(device, &key)
  })
  .inspect_err(|_| {
    // Almost every failure below the composition layer means the GPU
    // device went away; dropping it here is what lets the next attempt
    // rebuild rather than re-failing forever against a dead device.
    graphics_device::reset();
  })?;

  SURFACES.with(|cache| {
    let mut cache = cache.borrow_mut();

    // Drop only the entry this bake actually replaces: same monitor, same
    // knobs, older wallpaper. Evicting every entry for the monitor instead
    // would thrash whenever `focused_window` and `other_windows` are
    // configured with different knobs -- the two would take turns baking
    // each other out on every focus change.
    cache.retain(|entry| {
      entry.key.wallpaper.monitor != key.wallpaper.monitor
        || entry.key.knobs != key.knobs
    });

    cache.push(CachedSurface {
      key,
      surface: surface.clone(),
      last_used: now,
    });

    // Anything above the cap is a knob set nothing is asking for any more
    // -- most often the values in force before the last config reload.
    while cache.len() > MAX_CACHED_SURFACES {
      let Some(oldest) = cache
        .iter()
        .enumerate()
        .min_by_key(|(_, entry)| entry.last_used)
        .map(|(index, _)| index)
      else {
        break;
      };

      cache.remove(oldest);
    }
  });

  Ok(surface)
}

/// Re-draws every cached surface when the composition graphics device
/// replaces its rendering device.
///
/// The surfaces themselves survive a device swap -- brushes keep pointing
/// at them -- but their *contents* do not, so without this every backdrop
/// would go blank after a driver update or a TDR until something happened
/// to force a re-bake.
fn subscribe_device_replaced(
  compositor: &Compositor,
  device: &GraphicsDevice,
) -> windows::core::Result<()> {
  let compositor = compositor.clone();

  device
    .composition
    .RenderingDeviceReplaced(&TypedEventHandler::new(move |_, _| {
      let entries = SURFACES.with(|cache| {
        cache
          .borrow()
          .iter()
          .map(|entry| (entry.key.clone(), entry.surface.clone()))
          .collect::<Vec<_>>()
      });

      for (key, surface) in entries {
        let result = with_graphics_device(&compositor, |device, _| {
          draw_into(device, &surface, &key)
        });

        if let Err(err) = result {
          tracing::warn!(
            "Wallpaper backdrop redraw after device loss failed: {err}."
          );
        }
      }

      Ok(())
    }))?;

  Ok(())
}

/// Allocates a monitor-sized surface and draws the blurred wallpaper into
/// it.
///
/// `DirectXAlphaMode::Ignore` is the point of the whole style: it makes
/// the surface opaque, which is what lets DWM stop compositing everything
/// behind the overlay.
fn bake(
  device: &GraphicsDevice,
  key: &WallpaperKey,
) -> windows::core::Result<CompositionDrawingSurface> {
  let monitor = &key.wallpaper.monitor;

  let surface = device.composition.CreateDrawingSurface2(
    SizeInt32 {
      Width: monitor.width(),
      Height: monitor.height(),
    },
    DirectXPixelFormat::B8G8R8A8UIntNormalized,
    DirectXAlphaMode::Ignore,
  )?;

  if !draw_into(device, &surface, key)? {
    // Not an error -- a color-only desktop renders exactly like this --
    // but it is the answer to "why is my wallpaper backdrop a flat
    // color?", so it is worth saying once per bake rather than leaving
    // the user to guess.
    tracing::debug!(
      "Wallpaper backdrop for {:?} baked from the desktop background        color alone; no wallpaper image was drawn.",
      key.wallpaper.monitor
    );
  }

  Ok(surface)
}

/// Renders `key`'s wallpaper into an already-allocated `surface`,
/// reporting whether a wallpaper image actually made it in (as opposed to
/// the background color alone).
///
/// Split from [`bake`] so a device-loss redraw can refill the very
/// surfaces existing brushes already point at, instead of allocating new
/// ones and having to re-plumb every overlay.
fn draw_into(
  device: &GraphicsDevice,
  surface: &CompositionDrawingSurface,
  key: &WallpaperKey,
) -> windows::core::Result<bool> {
  let interop: ICompositionDrawingSurfaceInterop = surface.cast()?;
  let mut origin = POINT::default();

  // SAFETY: `interop` wraps a live surface; `origin` outlives the call,
  // and the requested interface is the one `BeginDraw` documents.
  let context: ID2D1DeviceContext =
    unsafe { interop.BeginDraw(None, &raw mut origin) }?;

  let drawn = draw(device, &context, origin, key);

  // SAFETY: Paired with the `BeginDraw` above, and must run even when the
  // draw failed -- leaving a surface open would wedge every later bake.
  let ended = unsafe { interop.EndDraw() };

  ended.and(drawn)
}

/// Draws the blurred wallpaper into the region of `context`'s atlas
/// starting at `origin`, reporting whether an image was drawn over the
/// background.
///
/// A composition surface hands back a device context targeting a *shared*
/// atlas, of which only `origin`-plus-monitor-size belongs to this
/// surface, so everything here is clipped to that region -- an unclipped
/// `Clear` alone would wipe unrelated surfaces.
fn draw(
  device: &GraphicsDevice,
  context: &ID2D1DeviceContext,
  origin: POINT,
  key: &WallpaperKey,
) -> windows::core::Result<bool> {
  let monitor = &key.wallpaper.monitor;

  #[allow(clippy::cast_precision_loss)]
  let (width, height) = (monitor.width() as f32, monitor.height() as f32);
  #[allow(clippy::cast_precision_loss)]
  let (left, top) = (origin.x as f32, origin.y as f32);

  let clip = D2D_RECT_F {
    left,
    top,
    right: left + width,
    bottom: top + height,
  };

  // SAFETY: `context` is the live context `BeginDraw` returned.
  // Composition surfaces are sized in raw pixels, so pinning DPI to 96
  // makes one DIP one pixel and lets the layout math below stay in
  // pixels throughout.
  let background = to_color_f(key.wallpaper.background);

  unsafe {
    context
      .PushAxisAlignedClip(&raw const clip, D2D1_ANTIALIAS_MODE_ALIASED);
    context.Clear(Some(&raw const background));
  }

  let result =
    draw_wallpaper(device, context, left, top, width, height, key);

  // SAFETY: Paired with the `PushAxisAlignedClip` above; must run even on
  // failure so the context is left balanced.
  unsafe {
    context.PopAxisAlignedClip();
  }

  result
}

/// Composes and draws the wallpaper image itself, returning whether it
/// drew one. No-op when the desktop has no image, in which case the
/// background `Clear` already produced the right result.
#[allow(clippy::too_many_arguments)]
fn draw_wallpaper(
  device: &GraphicsDevice,
  context: &ID2D1DeviceContext,
  left: f32,
  top: f32,
  width: f32,
  height: f32,
  key: &WallpaperKey,
) -> windows::core::Result<bool> {
  let Some(path) = key.wallpaper.image.as_deref() else {
    return Ok(false);
  };

  let (bitmap, size) = match load_bitmap(device, context, path) {
    Ok(loaded) => loaded,
    Err(err) => {
      // A wallpaper path that can't be decoded (removed mid-slideshow, an
      // unsupported codec) is a normal desktop state, not a pipeline
      // failure: the background color alone is exactly what Windows itself
      // would show, so this must not fall the whole style back to acrylic.
      tracing::debug!(
        "Wallpaper image {path:?} could not be decoded: {err}."
      );
      return Ok(false);
    }
  };

  let placement = place(size, &key.wallpaper);
  let desktop =
    compose_desktop(context, &bitmap, placement, width, height, key)?;

  let blur = effect(context, &CLSID_D2D1GaussianBlur, &desktop)?;
  set_float(
    &blur,
    D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0,
    key.knobs.blur_amount,
  )?;
  set_enum(
    &blur,
    D2D1_GAUSSIANBLUR_PROP_OPTIMIZATION.0,
    D2D1_GAUSSIANBLUR_OPTIMIZATION_QUALITY.0,
  )?;

  // The blur output extends infinitely (see `compose_desktop`), so it is
  // bounded to the monitor here and then again at the end of the chain.
  // Bounding an infinite image with the clip alone would make D2D
  // rasterize far more than it needs to, and the grain stage below needs
  // a finite extent to generate over.
  let cropped = crop_to(context, &blur, width, height)?;

  let graded = grade(context, &cropped, key)?;
  let grained = add_grain(context, &graded, width, height, key)?;

  // Re-bound the chain before it reaches `DrawImage`. The crop above does
  // not survive it: `Vignette` fills beyond its input with the vignette
  // color and `Turbulence` generates over the whole plane -- `Size` bounds
  // where its noise varies, not how far its output extends -- so either
  // one hands back an image with an infinite output rect, and
  // `DrawImage` refuses an unbounded image once a target offset is given
  // (E_INVALIDARG, raised at `EndDraw` rather than by the draw call
  // itself).
  let finished = crop_to(context, &grained, width, height)?;

  // SAFETY: Every effect above is live, and the target offset places the
  // graph's (0, 0) at this surface's own corner of the atlas.
  unsafe {
    let output = finished.GetOutput()?;
    let target = D2D_POINT_2F { x: left, y: top };
    context.DrawImage(
      &output,
      Some(&raw const target),
      None,
      D2D1_INTERPOLATION_MODE_LINEAR,
      D2D1_COMPOSITE_MODE_SOURCE_OVER,
    );
  }

  Ok(true)
}

/// Applies the color-grading chain -- exposure, contrast, saturation -- to
/// the already-blurred, monitor-sized image.
///
/// Vignette is deliberately absent. It is the one knob whose effect varies
/// with position, so baking it into an image shared by every window on the
/// monitor anchors the falloff to the *screen*: a window at the edge gets
/// a uniformly dark crop and one in the middle gets the bright centre, and
/// neither looks like a vignette on that window. It is applied per overlay
/// instead, by a gradient visual in `composition`.
///
/// Each stage is skipped at its neutral value rather than added as a no-op
/// node, so a config that sets none of these knobs produces byte-identical
/// output to the graph before they existed.
///
/// Compares against the neutral values exactly, for the same reason the
/// cache does: these are config-resolved numbers, and "the user did not
/// set this" is an exact value, not an approximate one.
#[allow(clippy::float_cmp)]
fn grade(
  context: &ID2D1DeviceContext,
  input: &ID2D1Effect,
  key: &WallpaperKey,
) -> windows::core::Result<ID2D1Effect> {
  let mut image = input.clone();

  if key.knobs.exposure != 0.0 {
    let exposure = effect(context, &CLSID_D2D1Exposure, &image)?;
    set_float(
      &exposure,
      D2D1_EXPOSURE_PROP_EXPOSURE_VALUE.0,
      key.knobs.exposure,
    )?;
    image = exposure;
  }

  if key.knobs.contrast != 0.0 {
    let contrast = effect(context, &CLSID_D2D1Contrast, &image)?;
    set_float(
      &contrast,
      D2D1_CONTRAST_PROP_CONTRAST.0,
      key.knobs.contrast,
    )?;
    image = contrast;
  }

  // One node covers both, since the effect takes them together. Skipped
  // only when neither is set -- unlike the others this is a tone-selective
  // stage, so it is worth its own node whenever either end is being moved.
  if key.knobs.highlights != 0.0 || key.knobs.shadows != 0.0 {
    let tone = effect(context, &CLSID_D2D1HighlightsShadows, &image)?;
    set_float(
      &tone,
      D2D1_HIGHLIGHTSANDSHADOWS_PROP_HIGHLIGHTS.0,
      key.knobs.highlights,
    )?;
    set_float(
      &tone,
      D2D1_HIGHLIGHTSANDSHADOWS_PROP_SHADOWS.0,
      key.knobs.shadows,
    )?;
    image = tone;
  }

  if key.knobs.saturation != 1.0 {
    let saturation = effect(context, &CLSID_D2D1Saturation, &image)?;
    set_float(
      &saturation,
      D2D1_SATURATION_PROP_SATURATION.0,
      key.knobs.saturation,
    )?;
    image = saturation;
  }

  Ok(image)
}

/// Overlays monochrome fractal noise -- the grain that separates frosted
/// glass from an out-of-focus photograph.
///
/// `Turbulence` generates color noise, so it is desaturated first: colored
/// speckle over a blurred image reads as chroma artifacting rather than
/// texture. Blending in `Overlay` mode rather than compositing keeps the
/// underlying luminance, lightening light areas and darkening dark ones
/// instead of washing the whole image toward grey.
fn add_grain(
  context: &ID2D1DeviceContext,
  input: &ID2D1Effect,
  width: f32,
  height: f32,
  key: &WallpaperKey,
) -> windows::core::Result<ID2D1Effect> {
  if key.knobs.grain <= 0.0 {
    return Ok(input.clone());
  }

  let noise = context_effect(context, &CLSID_D2D1Turbulence)?;

  // Bounds the generator to the monitor. Left at its default it produces
  // an infinite field, which would put the whole graph back to an
  // unbounded extent immediately after the crop that established one.
  set_vector2(
    &noise,
    D2D1_TURBULENCE_PROP_SIZE.0,
    Vector2 {
      X: width,
      Y: height,
    },
  )?;

  // High frequency, single octave: this wants per-pixel dither, not the
  // cloud-like structure that lower frequencies and stacked octaves give.
  set_vector2(
    &noise,
    D2D1_TURBULENCE_PROP_BASE_FREQUENCY.0,
    Vector2 { X: 0.5, Y: 0.5 },
  )?;
  set_uint(&noise, D2D1_TURBULENCE_PROP_NUM_OCTAVES.0, 1u32)?;

  let grey = effect(context, &CLSID_D2D1Saturation, &noise)?;
  set_float(&grey, D2D1_SATURATION_PROP_SATURATION.0, 0.0_f32)?;

  let faded = effect(context, &CLSID_D2D1Opacity, &grey)?;
  set_float(&faded, D2D1_OPACITY_PROP_OPACITY.0, key.knobs.grain)?;

  let blend = context_effect(context, &CLSID_D2D1Blend)?;
  set_enum(&blend, D2D1_BLEND_PROP_MODE.0, D2D1_BLEND_MODE_OVERLAY.0)?;

  // SAFETY: Both inputs are live effects; `Blend` takes exactly two, with
  // input 1 blended onto input 0.
  unsafe {
    blend.SetInput(0, &input.GetOutput()?, true);
    blend.SetInput(1, &faded.GetOutput()?, true);
  }

  Ok(blend)
}

/// Builds the effect graph reproducing what the desktop looks like on this
/// monitor, with an infinite extent so the blur that follows has real
/// pixels to sample past every edge.
///
/// Without that extension the blur would read transparent black just
/// outside the monitor and fade the wallpaper out along all four borders
/// -- the artifact is subtle in the middle of a screen and glaring at its
/// edges, which is exactly where tiled windows sit.
fn compose_desktop(
  context: &ID2D1DeviceContext,
  bitmap: &ID2D1Bitmap,
  placement: Placement,
  width: f32,
  height: f32,
  key: &WallpaperKey,
) -> windows::core::Result<ID2D1Effect> {
  let transform = effect(context, &CLSID_D2D12DAffineTransform, bitmap)?;
  set_matrix(
    &transform,
    D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX.0,
    Matrix3x2 {
      M11: placement.scale_x,
      M12: 0.0,
      M21: 0.0,
      M22: placement.scale_y,
      M31: placement.offset_x,
      M32: placement.offset_y,
    },
  )?;
  set_enum(
    &transform,
    D2D1_2DAFFINETRANSFORM_PROP_INTERPOLATION_MODE.0,
    D2D1_INTERPOLATION_MODE_CUBIC.0,
  )?;

  // Tiling is the one fit whose own repetition already covers the plane,
  // so it extends by wrapping and needs no background underneath it at
  // all.
  if placement.tiled {
    return border(context, &transform, D2D1_BORDER_EDGE_MODE_WRAP.0);
  }

  // Every other fit can leave part of the monitor uncovered (`Fit` and
  // `Center` letterbox; a smaller-than-monitor image at `Center` more so),
  // where Windows shows the desktop background color -- so the color goes
  // underneath as a flood, cropped to the monitor so the composite has a
  // finite extent for the border below to clamp.
  let flood = context_effect(context, &CLSID_D2D1Flood)?;
  set_vector4(
    &flood,
    D2D1_FLOOD_PROP_COLOR.0,
    to_vector4(key.wallpaper.background),
  )?;

  let background = effect(context, &CLSID_D2D1Crop, &flood)?;
  set_vector4(
    &background,
    D2D1_CROP_PROP_RECT.0,
    Vector4 {
      X: 0.0,
      Y: 0.0,
      Z: width,
      W: height,
    },
  )?;

  let composite = context_effect(context, &CLSID_D2D1Composite)?;

  // SAFETY: Both inputs are live effects; `Composite` takes exactly two,
  // and input 1 is drawn over input 0.
  unsafe {
    composite.SetInput(0, &background.GetOutput()?, true);
    composite.SetInput(1, &transform.GetOutput()?, true);
  }

  border(context, &composite, D2D1_BORDER_EDGE_MODE_CLAMP.0)
}

/// Bounds `input` to a `width` x `height` rectangle at the origin.
///
/// Used at both ends of the post-blur chain: once to give the vignette a
/// meaningful rectangle to fall off within, and once at the very end
/// because several effects hand back an unbounded output that `DrawImage`
/// will not accept.
fn crop_to(
  context: &ID2D1DeviceContext,
  input: &ID2D1Effect,
  width: f32,
  height: f32,
) -> windows::core::Result<ID2D1Effect> {
  let cropped = effect(context, &CLSID_D2D1Crop, input)?;
  set_vector4(
    &cropped,
    D2D1_CROP_PROP_RECT.0,
    Vector4 {
      X: 0.0,
      Y: 0.0,
      Z: width,
      W: height,
    },
  )?;
  Ok(cropped)
}

/// Extends `input` past its own extent in both axes with the given edge
/// mode, making the result infinite.
fn border(
  context: &ID2D1DeviceContext,
  input: &ID2D1Effect,
  edge_mode: i32,
) -> windows::core::Result<ID2D1Effect> {
  let border = effect(context, &CLSID_D2D1Border, input)?;
  set_enum(&border, D2D1_BORDER_PROP_EDGE_MODE_X.0, edge_mode)?;
  set_enum(&border, D2D1_BORDER_PROP_EDGE_MODE_Y.0, edge_mode)?;
  Ok(border)
}

/// Creates an effect with no input wired up yet.
fn context_effect(
  context: &ID2D1DeviceContext,
  id: &windows::core::GUID,
) -> windows::core::Result<ID2D1Effect> {
  // SAFETY: `id` names a D2D built-in effect and `context` is live.
  unsafe { context.CreateEffect(id) }
}

/// Creates an effect taking `input` (a bitmap or another effect's output)
/// as its only input.
fn effect<P>(
  context: &ID2D1DeviceContext,
  id: &windows::core::GUID,
  input: &P,
) -> windows::core::Result<ID2D1Effect>
where
  P: EffectInput,
{
  let effect = context_effect(context, id)?;
  let image = input.as_image()?;

  // SAFETY: `effect` was just created and every effect used here takes at
  // least one input.
  unsafe {
    effect.SetInput(0, &image, true);
  }

  Ok(effect)
}

/// Anything that can feed a D2D effect: a decoded bitmap, or the output of
/// an earlier effect in the graph.
trait EffectInput {
  fn as_image(
    &self,
  ) -> windows::core::Result<windows::Win32::Graphics::Direct2D::ID2D1Image>;
}

impl EffectInput for ID2D1Bitmap {
  fn as_image(
    &self,
  ) -> windows::core::Result<windows::Win32::Graphics::Direct2D::ID2D1Image>
  {
    self.cast()
  }
}

impl EffectInput for ID2D1Effect {
  fn as_image(
    &self,
  ) -> windows::core::Result<windows::Win32::Graphics::Direct2D::ID2D1Image>
  {
    // SAFETY: `self` is a live effect; `GetOutput` is always valid on one.
    unsafe { self.GetOutput() }
  }
}

/// Writes one effect property, given its D2D type and the raw bytes of a
/// value of exactly that type.
///
/// Private on purpose. D2D's property system is untyped at the ABI -- it
/// takes a byte buffer whose length has to match the property's declared
/// type -- and it does not reject a buffer of the wrong size, it stores
/// it. A mismatch therefore surfaces much later, as `E_INVALIDARG` from
/// `EndDraw` with nothing to point at the property that caused it. An
/// unsuffixed float literal is `f64` in Rust, so `0.6` passed to a `FLOAT`
/// property is eight bytes where four were expected: exactly that bug,
/// from code that reads as correct. The typed wrappers below are the only
/// way in.
fn set_property_bytes<T: Copy>(
  effect: &ID2D1Effect,
  index: i32,
  kind: D2D1_PROPERTY_TYPE,
  value: T,
) -> windows::core::Result<()> {
  #[allow(clippy::cast_sign_loss)]
  let index = index as u32;

  // SAFETY: The slice covers exactly `value`'s own bytes and does not
  // outlive it; `SetValue` copies out of it before returning.
  unsafe {
    let bytes = std::slice::from_raw_parts(
      (&raw const value).cast::<u8>(),
      std::mem::size_of::<T>(),
    );
    effect.SetValue(index, kind, bytes)
  }
}

/// Sets a `FLOAT` effect property.
fn set_float(
  effect: &ID2D1Effect,
  index: i32,
  value: f32,
) -> windows::core::Result<()> {
  set_property_bytes(effect, index, D2D1_PROPERTY_TYPE_FLOAT, value)
}

/// Sets an `ENUM` effect property from the discriminant of a D2D enum.
fn set_enum(
  effect: &ID2D1Effect,
  index: i32,
  value: i32,
) -> windows::core::Result<()> {
  #[allow(clippy::cast_sign_loss)]
  set_property_bytes(effect, index, D2D1_PROPERTY_TYPE_ENUM, value as u32)
}

/// Sets a `UINT32` effect property.
fn set_uint(
  effect: &ID2D1Effect,
  index: i32,
  value: u32,
) -> windows::core::Result<()> {
  set_property_bytes(effect, index, D2D1_PROPERTY_TYPE_UINT32, value)
}

/// Sets a `VECTOR2` effect property.
fn set_vector2(
  effect: &ID2D1Effect,
  index: i32,
  value: Vector2,
) -> windows::core::Result<()> {
  set_property_bytes(effect, index, D2D1_PROPERTY_TYPE_VECTOR2, value)
}

/// Sets a `VECTOR4` effect property.
fn set_vector4(
  effect: &ID2D1Effect,
  index: i32,
  value: Vector4,
) -> windows::core::Result<()> {
  set_property_bytes(effect, index, D2D1_PROPERTY_TYPE_VECTOR4, value)
}

/// Sets a `MATRIX_3X2` effect property.
fn set_matrix(
  effect: &ID2D1Effect,
  index: i32,
  value: Matrix3x2,
) -> windows::core::Result<()> {
  set_property_bytes(effect, index, D2D1_PROPERTY_TYPE_MATRIX_3X2, value)
}

/// Decodes the wallpaper file into a D2D bitmap, returning it with its
/// pixel dimensions.
fn load_bitmap(
  device: &GraphicsDevice,
  context: &ID2D1DeviceContext,
  path: &Path,
) -> windows::core::Result<(ID2D1Bitmap, (u32, u32))> {
  let wide: Vec<u16> = path
    .as_os_str()
    .encode_wide()
    .chain(std::iter::once(0))
    .collect();

  // SAFETY: `wide` is a null-terminated path that outlives the call, and
  // every interface below is used immediately after being handed back.
  unsafe {
    let decoder = device.imaging.CreateDecoderFromFilename(
      PCWSTR(wide.as_ptr()),
      None,
      GENERIC_READ,
      WICDecodeMetadataCacheOnLoad,
    )?;

    let frame = decoder.GetFrame(0)?;

    // D2D can only take premultiplied BGRA; wallpapers are routinely
    // 24-bit JPEG or indexed PNG, so the conversion is the common path
    // rather than a fallback.
    let converted =
      WICConvertBitmapSource(&GUID_WICPixelFormat32bppPBGRA, &frame)?;

    let (mut width, mut height) = (0, 0);
    converted.GetSize(&raw mut width, &raw mut height)?;

    let bitmap = context.CreateBitmapFromWicBitmap(&converted, None)?;
    Ok((bitmap, (width, height)))
  }
}

/// Where the wallpaper image lands within its monitor, in that monitor's
/// own pixel coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Placement {
  scale_x: f32,
  scale_y: f32,
  offset_x: f32,
  offset_y: f32,
  /// Whether the placed image repeats to cover the monitor.
  tiled: bool,
}

/// Reproduces Windows' own wallpaper layout for one monitor.
///
/// `Span` is the only fit laid out against the whole virtual desktop
/// rather than a single monitor: one image covers every display, and each
/// monitor shows its own slice of it. `Tile` anchors its repetition at the
/// virtual origin -- the primary display's top-left corner, which Windows
/// guarantees is `(0, 0)` -- so the pattern lines up seam-free across
/// displays instead of restarting on each.
#[allow(clippy::cast_precision_loss)]
fn place(image: (u32, u32), wallpaper: &MonitorWallpaper) -> Placement {
  let (image_width, image_height) = (image.0 as f32, image.1 as f32);
  let monitor = &wallpaper.monitor;

  // A zero-dimension image can't be laid out at all; scaling by 1 leaves
  // it invisible rather than producing infinities.
  if image_width <= 0.0 || image_height <= 0.0 {
    return Placement {
      scale_x: 1.0,
      scale_y: 1.0,
      offset_x: 0.0,
      offset_y: 0.0,
      tiled: false,
    };
  }

  if wallpaper.fit == WallpaperFit::Tile {
    return Placement {
      scale_x: 1.0,
      scale_y: 1.0,
      offset_x: -(monitor.x() as f32).rem_euclid(image_width),
      offset_y: -(monitor.y() as f32).rem_euclid(image_height),
      tiled: true,
    };
  }

  let frame = match wallpaper.fit {
    WallpaperFit::Span => &wallpaper.virtual_screen,
    _ => monitor,
  };
  let (frame_width, frame_height) =
    (frame.width() as f32, frame.height() as f32);

  let (scale_x, scale_y) = match wallpaper.fit {
    WallpaperFit::Center => (1.0, 1.0),
    WallpaperFit::Stretch => {
      (frame_width / image_width, frame_height / image_height)
    }
    WallpaperFit::Fit => {
      let scale =
        (frame_width / image_width).min(frame_height / image_height);
      (scale, scale)
    }
    // `Fill` and `Span` both cover their frame and crop the overflow.
    _ => {
      let scale =
        (frame_width / image_width).max(frame_height / image_height);
      (scale, scale)
    }
  };

  let (drawn_width, drawn_height) =
    (image_width * scale_x, image_height * scale_y);

  Placement {
    scale_x,
    scale_y,
    offset_x: (frame.x() - monitor.x()) as f32
      + (frame_width - drawn_width) / 2.0,
    offset_y: (frame.y() - monitor.y()) as f32
      + (frame_height - drawn_height) / 2.0,
    tiled: false,
  }
}

/// Converts a color into the 0-1 float form D2D render targets take.
fn to_color_f(color: crate::Color) -> D2D1_COLOR_F {
  D2D1_COLOR_F {
    r: f32::from(color.r) / 255.0,
    g: f32::from(color.g) / 255.0,
    b: f32::from(color.b) / 255.0,
    a: f32::from(color.a) / 255.0,
  }
}

/// Converts a color into the straight 0-1 float vector D2D effect
/// properties take.
fn to_vector4(color: crate::Color) -> Vector4 {
  Vector4 {
    X: f32::from(color.r) / 255.0,
    Y: f32::from(color.g) / 255.0,
    Z: f32::from(color.b) / 255.0,
    W: f32::from(color.a) / 255.0,
  }
}

#[cfg(test)]
mod tests {
  use super::{place, Placement};
  use crate::{
    platform_impl::wallpaper::{MonitorWallpaper, WallpaperFit},
    Color, Rect,
  };

  /// A single 1920x1080 monitor at the virtual origin, plus a second one
  /// to its right so `Span` and `Tile` have somewhere to continue onto.
  fn wallpaper(fit: WallpaperFit, monitor: Rect) -> MonitorWallpaper {
    MonitorWallpaper {
      image: None,
      modified: None,
      fit,
      background: Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
      },
      monitor,
      virtual_screen: Rect::from_xy(0, 0, 3840, 1080),
    }
  }

  fn primary() -> Rect {
    Rect::from_xy(0, 0, 1920, 1080)
  }

  fn secondary() -> Rect {
    Rect::from_xy(1920, 0, 1920, 1080)
  }

  /// `Fill` covers the monitor on both axes and centers the overflow, so a
  /// 16:10 image on a 16:9 monitor is scaled by width and bled off the top
  /// and bottom equally.
  #[test]
  fn fill_covers_and_centers() {
    let placed =
      place((1920, 1200), &wallpaper(WallpaperFit::Fill, primary()));

    assert!((placed.scale_x - 1.0).abs() < f32::EPSILON);
    assert!((placed.scale_y - 1.0).abs() < f32::EPSILON);
    assert!((placed.offset_x - 0.0).abs() < f32::EPSILON);
    assert!((placed.offset_y - (1080.0 - 1200.0) / 2.0).abs() < 0.01);
    assert!(!placed.tiled);
  }

  /// `Fit` scales down until the whole image is inside the monitor,
  /// leaving the background color visible in the letterbox.
  #[test]
  fn fit_letterboxes() {
    let placed =
      place((3840, 1080), &wallpaper(WallpaperFit::Fit, primary()));

    assert!((placed.scale_x - 0.5).abs() < 0.001);
    assert!((placed.offset_x - 0.0).abs() < f32::EPSILON);
    assert!((placed.offset_y - (1080.0 - 540.0) / 2.0).abs() < 0.01);
  }

  /// `Stretch` ignores aspect ratio and matches the monitor exactly.
  #[test]
  fn stretch_matches_monitor() {
    let placed =
      place((960, 2160), &wallpaper(WallpaperFit::Stretch, primary()));

    assert!((placed.scale_x - 2.0).abs() < 0.001);
    assert!((placed.scale_y - 0.5).abs() < 0.001);
    assert!((placed.offset_x - 0.0).abs() < f32::EPSILON);
    assert!((placed.offset_y - 0.0).abs() < f32::EPSILON);
  }

  /// `Span` lays one image over the whole virtual desktop, so the second
  /// monitor sees it shifted left by that monitor's own origin -- this is
  /// what makes the two halves line up across the seam.
  #[test]
  fn span_offsets_by_monitor_origin() {
    let on_primary =
      place((3840, 1080), &wallpaper(WallpaperFit::Span, primary()));
    let on_secondary =
      place((3840, 1080), &wallpaper(WallpaperFit::Span, secondary()));

    assert!(
      (on_primary.scale_x - on_secondary.scale_x).abs() < f32::EPSILON
    );
    assert!((on_primary.offset_x - 0.0).abs() < f32::EPSILON);
    assert!((on_secondary.offset_x - -1920.0).abs() < 0.01);
  }

  /// Tiling anchors at the virtual origin rather than each monitor's own,
  /// so a monitor whose origin isn't a whole number of tiles from `(0, 0)`
  /// starts mid-tile instead of restarting the pattern.
  #[test]
  fn tile_anchors_at_virtual_origin() {
    let placed =
      place((512, 512), &wallpaper(WallpaperFit::Tile, secondary()));

    assert!(placed.tiled);
    assert!((placed.scale_x - 1.0).abs() < f32::EPSILON);
    // 1920 mod 512 == 384.
    assert!((placed.offset_x - -384.0).abs() < 0.01);
    assert!((placed.offset_y - 0.0).abs() < f32::EPSILON);
  }

  /// A zero-sized decode must not produce infinities or NaNs downstream.
  #[test]
  fn degenerate_image_is_inert() {
    let placed = place((0, 0), &wallpaper(WallpaperFit::Fill, primary()));

    assert_eq!(
      placed,
      Placement {
        scale_x: 1.0,
        scale_y: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
        tiled: false,
      }
    );
  }

  /// Bakes real surfaces for this machine's primary monitor, end to end:
  /// D3D11 device, D2D device, composition graphics device, WIC decode of
  /// whatever wallpaper is actually set, and the effect graph.
  ///
  /// The graph is what this is really for. `CreateEffect` and `SetValue`
  /// validate against D2D's registered schema for each built-in effect and
  /// fail outright on a wrong property index, a mismatched property type,
  /// or an unsupported effect -- none of which can be ruled out by
  /// reading the code, and all of which would otherwise surface as the
  /// style silently falling back to SWCA acrylic on a user's machine.
  ///
  /// Each optional stage is baked on its own so a failure names the effect
  /// that caused it instead of just the combined chain.
  #[test]
  fn bakes_the_primary_monitor_end_to_end() {
    let neutral = super::BakeKnobs {
      blur_amount: 30.0,
      saturation: 1.0,
      exposure: 0.0,
      contrast: 0.0,
      highlights: 0.0,
      shadows: 0.0,
      grain: 0.0,
    };

    let cases = [
      ("blur only", neutral),
      (
        "saturation",
        super::BakeKnobs {
          saturation: 1.4,
          ..neutral
        },
      ),
      (
        "exposure",
        super::BakeKnobs {
          exposure: -0.3,
          ..neutral
        },
      ),
      (
        "contrast",
        super::BakeKnobs {
          contrast: 0.2,
          ..neutral
        },
      ),
      (
        "highlights/shadows",
        super::BakeKnobs {
          highlights: -0.5,
          shadows: 0.3,
          ..neutral
        },
      ),
      (
        "grain",
        super::BakeKnobs {
          grain: 0.1,
          ..neutral
        },
      ),
      (
        "everything",
        super::BakeKnobs {
          saturation: 1.4,
          exposure: -0.3,
          contrast: 0.2,
          highlights: -0.5,
          shadows: 0.3,
          grain: 0.1,
          ..neutral
        },
      ),
    ];

    let monitor = super::monitor_bounds(&Rect::from_xy(0, 0, 1, 1));
    let virtual_screen = super::virtual_screen();
    let expects_image = MonitorWallpaper::query(&monitor, &virtual_screen)
      .image
      .is_some();

    let mut failures = Vec::new();

    for (label, knobs) in cases {
      let key = super::WallpaperKey {
        wallpaper: MonitorWallpaper::query(&monitor, &virtual_screen),
        knobs,
      };

      let drew =
        crate::platform_impl::composition::with_composition_thread(
          move |compositor, _| {
            super::with_graphics_device(&compositor, |device, _| {
              let surface = device.composition.CreateDrawingSurface2(
                super::SizeInt32 {
                  Width: key.wallpaper.monitor.width(),
                  Height: key.wallpaper.monitor.height(),
                },
                super::DirectXPixelFormat::B8G8R8A8UIntNormalized,
                super::DirectXAlphaMode::Ignore,
              )?;

              super::draw_into(device, &surface, &key)
            })
          },
        );

      match drew {
        Err(err) => failures.push(format!("{label}: {err:?}")),
        // An image was expected but the background alone came back, which
        // means the decode failed silently -- see `draw_wallpaper`.
        Ok(drew_image) if drew_image != expects_image => {
          failures.push(format!(
            "{label}: drew_image={drew_image}, expected {expects_image}"
          ));
        }
        Ok(_) => {}
      }
    }

    assert!(failures.is_empty(), "wallpaper bake failed -- {failures:?}");
  }
}
