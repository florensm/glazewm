//! The Direct3D/Direct2D device stack backing the wallpaper backdrop.
//!
//! `Windows.UI.Composition` can only hand out a drawable surface through a
//! `CompositionGraphicsDevice`, and that in turn only wraps a real D2D
//! device -- so the wallpaper backdrop, unlike acrylic's host-backdrop
//! brush, needs a rendering stack of its own: D3D11 -> DXGI -> D2D1, plus
//! a WIC factory to decode the wallpaper file.
//!
//! # Threading
//!
//! Composition objects are agile, but the D2D and WIC interfaces here are
//! not, and neither is `ID3D11Device`. Everything in this module therefore
//! lives in thread-local storage on the composition thread (see
//! `composition`'s module docs) and is only reachable from a closure
//! running there.

use std::cell::RefCell;

use windows::{
  core::ComInterface,
  Win32::{
    Foundation::E_FAIL,
    Graphics::{
      Direct2D::{D2D1CreateDevice, ID2D1Device},
      Direct3D::{
        D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP,
      },
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_SDK_VERSION,
      },
      Dxgi::IDXGIDevice,
      Imaging::{CLSID_WICImagingFactory, IWICImagingFactory},
    },
    System::{
      Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
      WinRT::Composition::ICompositorInterop,
    },
  },
  UI::Composition::{CompositionGraphicsDevice, Compositor},
};

/// The rendering stack a baked wallpaper surface is drawn with.
pub(crate) struct GraphicsDevice {
  /// Root of the stack. Never called into directly -- both `_d2d` and
  /// `composition` are built on it and would be invalidated by its loss,
  /// so it's held for its lifetime alone.
  _d3d: ID3D11Device,

  /// Held for the same reason as `_d3d`: `composition` wraps it, and
  /// drawing goes through the device context `BeginDraw` hands back
  /// rather than one created here.
  _d2d: ID2D1Device,

  /// Allocates the composition surfaces the baked wallpaper is drawn
  /// into.
  pub(crate) composition: CompositionGraphicsDevice,

  /// Decodes the wallpaper image file.
  pub(crate) imaging: IWICImagingFactory,
}

impl GraphicsDevice {
  fn create(compositor: &Compositor) -> windows::core::Result<Self> {
    let d3d = create_d3d_device()?;

    // SAFETY: Every D3D11 device implements `IDXGIDevice`.
    let dxgi: IDXGIDevice = d3d.cast()?;

    // SAFETY: `dxgi` is a live device; passing no creation properties
    // takes D2D's defaults (single-threaded, matching this module's
    // thread-local ownership, and no debug layer).
    let d2d = unsafe { D2D1CreateDevice(&dxgi, None)? };

    // SAFETY: `compositor` is live, and `CreateGraphicsDevice` accepts an
    // `ID2D1Device` as its rendering device.
    let composition = unsafe {
      compositor
        .cast::<ICompositorInterop>()?
        .CreateGraphicsDevice(&d2d)?
    };

    // SAFETY: `CLSID_WICImagingFactory` is a valid in-process CLSID and
    // the inferred interface is the one it implements.
    let imaging = unsafe {
      CoCreateInstance(
        &CLSID_WICImagingFactory,
        None,
        CLSCTX_INPROC_SERVER,
      )?
    };

    Ok(Self {
      _d3d: d3d,
      _d2d: d2d,
      composition,
      imaging,
    })
  }
}

/// Creates a BGRA-capable D3D11 device, preferring the GPU and falling
/// back to the software rasterizer.
///
/// `D3D11_CREATE_DEVICE_BGRA_SUPPORT` is mandatory, not an optimization:
/// D2D interop refuses a device created without it. The WARP fallback
/// covers machines with no usable D3D11 driver at all (some VMs, some RDP
/// sessions) -- a baked wallpaper is drawn once and then only read by the
/// compositor, so software rasterization there costs nothing per frame.
fn create_d3d_device() -> windows::core::Result<ID3D11Device> {
  fn create(
    driver: D3D_DRIVER_TYPE,
  ) -> windows::core::Result<ID3D11Device> {
    let mut device = None;

    // SAFETY: All out-parameters are optional and `device` outlives the
    // call; passing no adapter is required when a driver type is given.
    unsafe {
      D3D11CreateDevice(
        None,
        driver,
        None,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        None,
        D3D11_SDK_VERSION,
        Some(&raw mut device),
        None,
        None,
      )?;
    }

    device.ok_or_else(|| windows::core::Error::from(E_FAIL))
  }

  create(D3D_DRIVER_TYPE_HARDWARE).or_else(|err| {
    tracing::debug!("No hardware D3D11 device for the wallpaper backdrop, falling back to WARP: {err}.");
    create(D3D_DRIVER_TYPE_WARP)
  })
}

thread_local! {
  /// The composition thread's graphics device, built on first use.
  ///
  /// `RefCell` rather than `OnceCell` because [`reset`] has to be able to
  /// tear a lost device down and let the next bake build a fresh one.
  static GRAPHICS_DEVICE: RefCell<Option<GraphicsDevice>> =
    const { RefCell::new(None) };
}

/// Runs `f` against the composition thread's graphics device, creating it
/// on first use.
///
/// `f` is told whether the device was built by this very call, which is
/// the only moment a `RenderingDeviceReplaced` subscription can be
/// attached exactly once per device.
///
/// Fails rather than panics when the device is already borrowed further up
/// the stack: a `RenderingDeviceReplaced` callback can land on this thread
/// while a bake is in flight, and losing that one redraw is preferable to
/// taking the process down.
pub(crate) fn with_graphics_device<T>(
  compositor: &Compositor,
  f: impl FnOnce(&GraphicsDevice, bool) -> windows::core::Result<T>,
) -> windows::core::Result<T> {
  GRAPHICS_DEVICE.with(|cell| {
    let mut slot = cell.try_borrow_mut().map_err(|_| {
      windows::core::Error::new(
        E_FAIL,
        "Graphics device is already in use on this thread.".into(),
      )
    })?;

    let is_new = slot.is_none();
    if is_new {
      *slot = Some(GraphicsDevice::create(compositor)?);
    }

    let device = slot.as_ref().expect("populated above");
    f(device, is_new)
  })
}

/// Drops the cached device so the next [`with_graphics_device`] call
/// builds a fresh one.
///
/// Called after a draw fails, which on this stack almost always means the
/// GPU device was removed or reset (driver update, TDR, adapter change);
/// every interface above is invalidated together in that case, so the
/// whole stack goes rather than any single one being repaired.
pub(crate) fn reset() {
  GRAPHICS_DEVICE.with(|cell| {
    if let Ok(mut slot) = cell.try_borrow_mut() {
      slot.take();
    }
  });
}
