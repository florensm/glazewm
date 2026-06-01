//! DirectComposition + Windows.Graphics.Capture surrogate.
//!
//! Provides a live, 3D-transformable stand-in for a real window, used by the
//! animation system for styles that the DWM-thumbnail surrogate
//! ([`NativeSurrogate`]) cannot express (rotation, perspective, flip). A window
//! is captured live via `Windows.Graphics.Capture`, its frames are copied into
//! a `DirectComposition` surface, and an [`IDCompositionMatrixTransform3D`]
//! applies an arbitrary 4x4 transform per frame.
//!
//! # Threading
//!
//! Every object here is created and updated on the single "WM thread" that
//! drives surrogate updates (the same thread that creates [`NativeSurrogate`]
//! overlays and runs the animation tick). The WGC frame pool is created
//! free-threaded so no dispatcher queue / message pump is required on that
//! thread. The WinRT apartment is initialized single-threaded to stay
//! compatible with the STA COM initialization used elsewhere for window
//! cloaking — do not switch this to multithreaded.
//!
//! [`NativeSurrogate`]: crate::NativeSurrogate

use std::sync::OnceLock;

use windows::{
  core::{factory, w, ComInterface, Interface},
  Foundation::Numerics::Matrix4x4,
  Graphics::{
    Capture::{
      Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
    },
    DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    SizeInt32,
  },
  Win32::{
    Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::{
      Direct3D::{
        D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP,
        D3D_FEATURE_LEVEL,
      },
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice,
        IDCompositionMatrixTransform3D, IDCompositionSurface,
        IDCompositionTarget, IDCompositionVisual,
      },
      Dxgi::{
        Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM},
        IDXGIDevice,
      },
    },
    System::WinRT::{
      Direct3D11::{
        CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
      },
      Graphics::Capture::IGraphicsCaptureItemInterop,
      RoInitialize, RO_INIT_SINGLETHREADED,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOCOPYBITS,
      SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE,
      SW_SHOWNOACTIVATE, WNDCLASSW, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::Rect;

/// Ensures the DComp surrogate window class is registered once per process.
static DCOMP_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Identity 4x4 transform in the neutral row-major form used by this module.
pub const IDENTITY_TRANSFORM: [[f32; 4]; 4] = [
  [1.0, 0.0, 0.0, 0.0],
  [0.0, 1.0, 0.0, 0.0],
  [0.0, 0.0, 1.0, 0.0],
  [0.0, 0.0, 0.0, 1.0],
];

/// Default window procedure wrapper with the required `extern "system"` ABI.
unsafe extern "system" fn dcomp_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: All parameters are forwarded unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Registers the transparent overlay window class exactly once.
fn ensure_class_registered() {
  DCOMP_CLASS_REGISTERED.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_DcompSurrogate"),
      lpfnWndProc: Some(dcomp_wnd_proc),
      ..Default::default()
    };
    // SAFETY: `wnd_class` is a valid `WNDCLASSW` with a static class name and a
    // valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Converts a neutral row-major 4x4 matrix into a WinRT [`Matrix4x4`].
fn to_matrix4x4(m: &[[f32; 4]; 4]) -> Matrix4x4 {
  Matrix4x4 {
    M11: m[0][0], M12: m[0][1], M13: m[0][2], M14: m[0][3],
    M21: m[1][0], M22: m[1][1], M23: m[1][2], M24: m[1][3],
    M31: m[2][0], M32: m[2][1], M33: m[2][2], M34: m[2][3],
    M41: m[3][0], M42: m[3][1], M43: m[3][2], M44: m[3][3],
  }
}

/// Shared Direct3D 11 + DirectComposition device backing every DComp surrogate.
///
/// Created once on the WM thread and reused for all surrogates. Holds the
/// immediate context used to copy captured frames and the composition device
/// whose [`commit`](DcompContext::commit) publishes all pending visual updates.
pub struct DcompContext {
  /// Direct3D 11 device shared by the WGC frame pools and DComp surfaces.
  device: ID3D11Device,
  /// Immediate context used to copy captured textures into DComp surfaces.
  context: ID3D11DeviceContext,
  /// Composition device whose visual tree is committed each frame.
  dcomp: IDCompositionDevice,
}

impl DcompContext {
  /// Creates the shared device, initializing the WinRT apartment (STA) for
  /// `Windows.Graphics.Capture`.
  ///
  /// Tries a hardware Direct3D 11 device first, falling back to the WARP
  /// software renderer so capture still works on adapters lacking BGRA support.
  pub fn new() -> crate::Result<Self> {
    // WGC activation requires an initialized WinRT apartment on this thread.
    // Single-threaded to coexist with the STA COM init used for cloaking.
    // Best-effort: `S_FALSE` (already initialized) is fine; a hard error is
    // surfaced later if capture activation fails.
    // SAFETY: No preconditions for `RoInitialize`.
    let _ = unsafe { RoInitialize(RO_INIT_SINGLETHREADED) };

    let (device, context) = Self::create_d3d_device()?;
    let dxgi: IDXGIDevice = device.cast()?;
    // SAFETY: `dxgi` is the DXGI interface of a valid D3D11 device.
    let dcomp: IDCompositionDevice =
      unsafe { DCompositionCreateDevice(&dxgi)? };

    Ok(Self {
      device,
      context,
      dcomp,
    })
  }

  /// Creates a Direct3D 11 device with BGRA support, trying hardware then WARP.
  fn create_d3d_device(
  ) -> crate::Result<(ID3D11Device, ID3D11DeviceContext)> {
    fn try_create(
      driver: D3D_DRIVER_TYPE,
    ) -> windows::core::Result<(ID3D11Device, ID3D11DeviceContext)> {
      let mut device: Option<ID3D11Device> = None;
      let mut context: Option<ID3D11DeviceContext> = None;
      let mut feature_level = D3D_FEATURE_LEVEL::default();
      // SAFETY: Out-parameters are valid stack slots; `BGRA_SUPPORT` is
      // required to back a DirectComposition surface.
      unsafe {
        D3D11CreateDevice(
          None,
          driver,
          HMODULE::default(),
          D3D11_CREATE_DEVICE_BGRA_SUPPORT,
          None,
          D3D11_SDK_VERSION,
          Some(&mut device),
          Some(&mut feature_level),
          Some(&mut context),
        )?;
      }
      match (device, context) {
        (Some(d), Some(c)) => Ok((d, c)),
        _ => Err(windows::core::Error::from_win32()),
      }
    }

    try_create(D3D_DRIVER_TYPE_HARDWARE)
      .or_else(|_| try_create(D3D_DRIVER_TYPE_WARP))
      .map_err(Into::into)
  }

  /// Publishes all pending visual-tree changes to the compositor.
  pub fn commit(&self) -> crate::Result<()> {
    // SAFETY: `self.dcomp` is a valid composition device.
    unsafe { self.dcomp.Commit()? };
    Ok(())
  }
}

/// Live capture of a single window via `Windows.Graphics.Capture`.
///
/// Frames are produced on a free-threaded pool sharing the [`DcompContext`]
/// Direct3D 11 device, so captured textures can be copied directly into a
/// DirectComposition surface on the WM thread.
struct Capture {
  /// WinRT Direct3D device backing the frame pool; reused on resize and kept
  /// alive for the session.
  device: IDirect3DDevice,
  /// Pixel format of the frame pool, reused on resize.
  format: DirectXPixelFormat,
  /// Free-threaded pool that buffers captured frames.
  frame_pool: Direct3D11CaptureFramePool,
  /// Active capture session, kept alive to continue producing frames.
  _session: GraphicsCaptureSession,
  /// Current frame-pool size in pixels (tracks the source window's size).
  size: SizeInt32,
  /// Whether at least one real frame has been copied into the surface.
  ///
  /// `Windows.Graphics.Capture` delivers its first frame a few composition
  /// cycles after `StartCapture`. Callers gate revealing the surrogate on this
  /// so the overlay is never shown blank.
  has_content: bool,
}

impl Capture {
  /// Starts capturing `source` using `ctx`'s Direct3D 11 device.
  ///
  /// Returns an error when the window cannot be captured (e.g. elevated or
  /// otherwise protected windows); callers fall back to the DWM path.
  fn start(ctx: &DcompContext, source: HWND) -> crate::Result<Self> {
    let dxgi: IDXGIDevice = ctx.device.cast()?;
    // SAFETY: `dxgi` is a valid DXGI device; the interop wraps it as a WinRT
    // Direct3D device.
    let inspectable =
      unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi)? };
    let device: IDirect3DDevice = inspectable.cast()?;

    let interop =
      factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    // SAFETY: `source` is a valid top-level window handle.
    let item: GraphicsCaptureItem =
      unsafe { interop.CreateForWindow(source)? };
    let size = item.Size()?;

    let format = DirectXPixelFormat::B8G8R8A8UIntNormalized;
    let frame_pool =
      Direct3D11CaptureFramePool::CreateFreeThreaded(&device, format, 2, size)?;
    let session = frame_pool.CreateCaptureSession(&item)?;
    // Best-effort: borderless requires Windows 11; ignored on older builds.
    let _ = session.SetIsBorderRequired(false);
    let _ = session.SetIsCursorCaptureEnabled(false);
    session.StartCapture()?;

    Ok(Self {
      device,
      format,
      frame_pool,
      _session: session,
      size,
      has_content: false,
    })
  }

  /// Copies the latest captured frame into `surface`, recreating the pool and
  /// `surface` when the source window changed size. Returns the current content
  /// size in pixels.
  ///
  /// When no new frame is available the size is returned unchanged so the
  /// caller keeps animating the existing content.
  fn update(
    &mut self,
    ctx: &DcompContext,
    visual: &IDCompositionVisual,
    surface: &mut IDCompositionSurface,
  ) -> crate::Result<(u32, u32)> {
    // `TryGetNextFrame` reports "no new frame yet" as a null WinRT result,
    // which surfaces as `Err(Error::OK)` (HRESULT `S_OK`). Treat that and an
    // explicit null as simply no frame this tick, not a failure.
    let frame = match self.frame_pool.TryGetNextFrame() {
      Ok(frame) if !frame.as_raw().is_null() => frame,
      Ok(_) => return Ok(self.current_size()),
      Err(err) if err.code().is_ok() => return Ok(self.current_size()),
      Err(err) => return Err(err.into()),
    };

    // Recreate the pool and surface when the source window changed size; WGC
    // keeps delivering old-sized frames until the pool is recreated.
    let content = frame.ContentSize()?;
    if (content.Width != self.size.Width || content.Height != self.size.Height)
      && content.Width > 0
      && content.Height > 0
    {
      drop(frame);
      self
        .frame_pool
        .Recreate(&self.device, self.format, 2, content)?;
      self.size = content;
      *surface = create_content_surface(ctx, content.Width, content.Height)?;
      // SAFETY: `visual` is valid; the new surface replaces the old content,
      // whose handle is dropped on reassignment above.
      unsafe { visual.SetContent(&*surface)? };
      return Ok(self.current_size());
    }

    let src: ID3D11Texture2D = frame
      .Surface()?
      .cast::<IDirect3DDxgiInterfaceAccess>()
      .and_then(|access| unsafe { access.GetInterface() })?;

    let mut offset = POINT::default();
    // SAFETY: `surface` is valid; `offset` is a valid out-parameter slot.
    let dst: ID3D11Texture2D =
      unsafe { surface.BeginDraw(None, &mut offset)? };

    // SAFETY: `dst` and `src` are BGRA textures on the same device; the copy
    // lands at the surface's atlas offset. `frame` stays alive until this
    // method returns, after the copy completes.
    unsafe {
      ctx.context.CopySubresourceRegion(
        &dst,
        0,
        offset.x as u32,
        offset.y as u32,
        0,
        &src,
        0,
        None,
      );
      surface.EndDraw()?;
    }
    self.has_content = true;
    Ok(self.current_size())
  }

  /// Returns the current content size in pixels, clamped to at least 1x1.
  fn current_size(&self) -> (u32, u32) {
    (self.size.Width.max(1) as u32, self.size.Height.max(1) as u32)
  }
}

/// Creates a DComp content surface of the given size with ignored alpha.
///
/// Alpha is ignored so that windows whose captured frames carry zero alpha
/// (a common WGC quirk) still render opaque; surrounding transparency comes
/// from the visual not covering those pixels, not from the surface alpha.
fn create_content_surface(
  ctx: &DcompContext,
  width: i32,
  height: i32,
) -> crate::Result<IDCompositionSurface> {
  // SAFETY: Dimensions and format are valid for a BGRA composition surface.
  let surface = unsafe {
    ctx.dcomp.CreateSurface(
      width.max(1) as u32,
      height.max(1) as u32,
      DXGI_FORMAT_B8G8R8A8_UNORM,
      DXGI_ALPHA_MODE_IGNORE,
    )?
  };
  Ok(surface)
}

/// Live, 3D-transformable surrogate for a single window.
///
/// Owns a transparent overlay window bound to a DirectComposition target whose
/// single visual shows the live capture of the source window. A 4x4 matrix
/// transform and per-visual opacity are applied each frame, then published via
/// [`DcompContext::commit`].
///
/// Falls back gracefully: if the source window cannot be captured,
/// [`create`](NativeDcompSurrogate::create) returns an error so the caller can
/// use the DWM-thumbnail path instead.
///
/// # Platform-specific
///
/// Windows only. Borderless capture requires Windows 11; 3D transforms require
/// DirectComposition (Windows 8+).
pub struct NativeDcompSurrogate {
  /// Transparent overlay window hosting the composition target.
  hwnd: isize,
  /// Composition target binding the visual tree to the overlay window.
  _target: IDCompositionTarget,
  /// Root visual showing the captured content.
  visual: IDCompositionVisual,
  /// 3D matrix transform applied to the visual each frame.
  transform: IDCompositionMatrixTransform3D,
  /// Surface receiving captured frames; recreated on source resize.
  surface: IDCompositionSurface,
  /// Live window capture.
  capture: Capture,
  /// Cached visibility; guards redundant `ShowWindow` calls.
  is_visible: bool,
}

impl NativeDcompSurrogate {
  /// Creates a surrogate overlay covering `rect` and begins capturing
  /// `source_hwnd`.
  ///
  /// `rect` is the overlay's screen rectangle; the captured content is scaled
  /// and transformed within it by the matrix passed to
  /// [`set_transform`](NativeDcompSurrogate::set_transform). When
  /// `initially_visible` is `false` the overlay is created hidden.
  ///
  /// Returns an error if the source window cannot be captured or the overlay
  /// cannot be created; the caller should fall back to the DWM path.
  pub fn create(
    ctx: &DcompContext,
    source_hwnd: HWND,
    rect: &Rect,
    initially_visible: bool,
  ) -> crate::Result<Self> {
    // Start capture first so an uncapturable window fails before any window or
    // composition resources are allocated.
    let mut capture = Capture::start(ctx, source_hwnd)?;
    let (cw, ch) = capture.current_size();

    ensure_class_registered();

    // Transparent, non-interactive overlay. `WS_EX_NOREDIRECTIONBITMAP` gives
    // the window no redirection surface so DirectComposition composes it with
    // per-pixel alpha — pixels the visual does not cover are see-through.
    // `WS_EX_TOPMOST` keeps the transition above reflowing sibling windows
    // (close) and above the real window if the OS briefly uncloaks it while
    // it holds focus (focus tilt).
    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOREDIRECTIONBITMAP
          | WS_EX_NOACTIVATE
          | WS_EX_TOOLWINDOW
          | WS_EX_TOPMOST
          | WS_EX_TRANSPARENT,
        w!("GlazeWM_DcompSurrogate"),
        w!(""),
        WS_POPUP,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        None,
        None,
        None,
        None,
      )
    };
    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create DComp surrogate window.".to_string(),
      ));
    }

    let mut surface = create_content_surface(ctx, cw as i32, ch as i32)?;

    // Build and root the visual tree. The target binds the tree to `hwnd`.
    // SAFETY: `hwnd` is a valid top-level window; `surface` and `transform`
    // are valid content and effect for the visual.
    let (target, visual, transform) = unsafe {
      let target = ctx.dcomp.CreateTargetForHwnd(hwnd, BOOL(1))?;
      let visual = ctx.dcomp.CreateVisual()?;
      let transform = ctx.dcomp.CreateMatrixTransform3D()?;
      visual.SetContent(&surface)?;
      visual.SetEffect(&transform)?;
      target.SetRoot(&visual)?;
      (target, visual, transform)
    };

    // Pull the first frame so the surrogate shows content immediately.
    let _ = capture.update(ctx, &visual, &mut surface);

    let show_flag = if initially_visible {
      SWP_SHOWWINDOW
    } else {
      windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS::default()
    };
    // Place the overlay in the topmost band so reflowing siblings and a
    // briefly-uncloaked source window cannot draw over the transition.
    // SAFETY: `hwnd` is valid; position is unchanged (set at creation).
    unsafe {
      SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | show_flag,
      )?;
    }

    ctx.commit()?;

    Ok(Self {
      hwnd: hwnd.0,
      _target: target,
      visual,
      transform,
      surface,
      capture,
      is_visible: initially_visible,
    })
  }

  /// Pulls the latest captured frame into the composition surface, handling
  /// source-window resizes. Returns the current captured content size.
  ///
  /// Does not commit; callers batch a [`DcompContext::commit`] after updating
  /// all surrogates for the frame.
  pub fn update_capture(
    &mut self,
    ctx: &DcompContext,
  ) -> crate::Result<(u32, u32)> {
    self.capture.update(ctx, &self.visual, &mut self.surface)
  }

  /// Sets the 3D transform applied to the captured content this frame.
  ///
  /// The matrix is in row-vector convention (`point * matrix`), row-major.
  /// Does not commit.
  pub fn set_transform(&self, matrix: &[[f32; 4]; 4]) -> crate::Result<()> {
    let m = to_matrix4x4(matrix);
    // SAFETY: `self.transform` is a valid 3D matrix transform.
    unsafe { self.transform.SetMatrix(&m)? };
    Ok(())
  }

  /// Shows or hides the overlay without activating it. No-op when already in
  /// the requested state.
  pub fn set_visible(&mut self, visible: bool) {
    if self.is_visible == visible {
      return;
    }
    self.is_visible = visible;
    // SAFETY: `HWND(self.hwnd)` is valid until `drop`.
    unsafe {
      ShowWindow(
        HWND(self.hwnd),
        if visible { SW_SHOWNOACTIVATE } else { SW_HIDE },
      );
    }
  }

  /// Repositions the overlay window to `rect` without altering its z-order.
  pub fn reposition(&self, rect: &Rect) -> crate::Result<()> {
    // SAFETY: `HWND(self.hwnd)` is the overlay created in `create`.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOZORDER,
      )?;
    }
    Ok(())
  }

  /// Returns the raw handle of the overlay window.
  pub fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Whether at least one real captured frame has been drawn into the surface.
  pub fn has_content(&self) -> bool {
    self.capture.has_content
  }
}

impl Drop for NativeDcompSurrogate {
  fn drop(&mut self) {
    // SAFETY: `self.hwnd` is the overlay window created in `create`. The
    // composition objects and capture session release on field drop; the
    // window is destroyed last.
    unsafe {
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}

// ---------------------------------------------------------------------------
// 3D transition sessions
//
// A [`DcompSession`] wraps a [`NativeDcompSurrogate`] with the per-frame 4x4
// transform for one of the supported cinematic styles. The animation system in
// the `wm` crate owns these sessions but never touches Windows APIs directly:
// it calls [`DcompSession::apply_frame`] with the eased animation progress and
// batches a single [`DcompContext::commit`] per tick.
// ---------------------------------------------------------------------------

/// Maximum flip angle for the card-flip open/close style, in degrees.
///
/// A full quarter turn: the card is edge-on (invisible) at the extreme and
/// face-on (flat) when settled, giving the signature "card turning" look.
const FLIP_MAX_DEG: f32 = 90.0;

/// Perspective depth for the flip, as a multiple of the content width. Smaller
/// values foreshorten more strongly. Tuned for a bold, cinematic turn.
const FLIP_DEPTH_FACTOR: f32 = 1.15;

/// Maximum swing angle for the hinge (door) open/close style, in degrees.
///
/// The content pivots about its left edge and swings *away* from the viewer, so
/// large angles foreshorten (never magnify) and stay within the overlay.
const HINGE_MAX_DEG: f32 = 78.0;

/// Perspective depth for the hinge, as a multiple of the content width.
const HINGE_DEPTH_FACTOR: f32 = 1.3;

/// Maximum in-plane (Z) rotation for the spin open/close style, in degrees.
const SPIN_MAX_DEG: f32 = 16.0;

/// How far the spin style shrinks at its extreme, as a fraction below 1.0
/// (e.g. 0.18 = scales up from 0.82). Staying below 1.0 avoids edge clipping.
const SPIN_SHRINK: f32 = 0.18;

/// Maximum lean angle for the focus tilt style, in degrees.
const TILT_MAX_DEG: f32 = 13.0;

/// Perspective depth for the tilt, as a multiple of the content height.
const TILT_DEPTH_FACTOR: f32 = 1.0;

/// Extra uniform scale applied at the start of the focus tilt ("pop"), as a
/// fraction above 1.0. Settles to 1.0 (no scale) when the animation completes.
const TILT_POP: f32 = 0.05;

/// Multiplies two row-major 4x4 matrices in the row-vector convention.
///
/// `mat_mul(a, b)` produces the matrix that applies `a` first, then `b`
/// (`point * a * b`).
fn mat_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
  let mut out = [[0.0_f32; 4]; 4];
  for (i, row) in out.iter_mut().enumerate() {
    for (j, cell) in row.iter_mut().enumerate() {
      *cell = (0..4).map(|k| a[i][k] * b[k][j]).sum();
    }
  }
  out
}

/// Builds a translation matrix in the row-vector convention.
fn mat_translate(tx: f32, ty: f32, tz: f32) -> [[f32; 4]; 4] {
  [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [tx, ty, tz, 1.0],
  ]
}

/// Builds a uniform scale matrix.
fn mat_scale(s: f32) -> [[f32; 4]; 4] {
  [
    [s, 0.0, 0.0, 0.0],
    [0.0, s, 0.0, 0.0],
    [0.0, 0.0, s, 0.0],
    [0.0, 0.0, 0.0, 1.0],
  ]
}

/// Builds a rotation about the X axis (lean), in the row-vector convention.
fn mat_rotate_x(rad: f32) -> [[f32; 4]; 4] {
  let (s, c) = rad.sin_cos();
  [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, c, s, 0.0],
    [0.0, -s, c, 0.0],
    [0.0, 0.0, 0.0, 1.0],
  ]
}

/// Builds a rotation about the Y axis (flip), in the row-vector convention.
fn mat_rotate_y(rad: f32) -> [[f32; 4]; 4] {
  let (s, c) = rad.sin_cos();
  [
    [c, 0.0, s, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [-s, 0.0, c, 0.0],
    [0.0, 0.0, 0.0, 1.0],
  ]
}

/// Builds an in-plane rotation about the Z axis, in the row-vector convention.
fn mat_rotate_z(rad: f32) -> [[f32; 4]; 4] {
  let (s, c) = rad.sin_cos();
  [
    [c, s, 0.0, 0.0],
    [-s, c, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
  ]
}

/// Builds a perspective matrix that divides x/y by `1 - z / depth`, so content
/// rotated toward the viewer is magnified and content rotated away foreshortens.
fn mat_perspective(depth: f32) -> [[f32; 4]; 4] {
  [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, -1.0 / depth],
    [0.0, 0.0, 0.0, 1.0],
  ]
}

/// Shape of an open/close 3D transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DcompShape {
  /// Card flip about the vertical center axis (edge-on ↔ flat).
  Flip,
  /// Door swing about the left edge (away from the viewer ↔ flat).
  Hinge,
  /// In-plane spin with a scale "pop" (rotated + small ↔ flat).
  Spin,
}

/// Shape of a focus 3D transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DcompFocus {
  /// Lean back with a slight scale pop, settling flat.
  Tilt,
}

/// A 3D transition applied by a [`DcompSession`], parameterized by phase
/// (open / close / focus) and shape.
///
/// Every transition resolves to the identity placement at `eased == 1.0`, so
/// the surrogate aligns exactly with the real window for a seamless hand-off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DcompTransitionKind {
  /// Window-open transition: animates from the shape's extreme pose to flat.
  Open(DcompShape),
  /// Window-close transition: animates from flat to the shape's extreme pose.
  Close(DcompShape),
  /// Focus transition: animates from the shape's extreme pose to flat.
  Focus(DcompFocus),
}

impl DcompTransitionKind {
  /// Whether this is a close transition (drives a `WM_CLOSE` on completion).
  pub fn is_close(self) -> bool {
    matches!(self, Self::Close(_))
  }

  /// Whether this is a focus transition.
  pub fn is_focus(self) -> bool {
    matches!(self, Self::Focus(_))
  }

  /// Computes the content transform for this transition at `eased` progress.
  ///
  /// `cw`/`ch` are the captured content size in pixels; `margin` is the
  /// transparent padding (in pixels) between the content and each overlay edge.
  fn transform(self, eased: f32, cw: f32, ch: f32, margin: f32) -> [[f32; 4]; 4] {
    // `extreme` is 0.0 at the flat/settled pose and 1.0 at the shape's most
    // extreme pose. Open and focus animate extreme→0; close animates 0→extreme.
    match self {
      Self::Open(shape) => shape_matrix(shape, 1.0 - eased, cw, ch, margin),
      Self::Close(shape) => shape_matrix(shape, eased, cw, ch, margin),
      Self::Focus(focus) => focus_matrix(focus, 1.0 - eased, cw, ch, margin),
    }
  }
}

/// Builds the open/close transform for `shape` at `extreme` ∈ [0, 1].
///
/// `extreme == 0.0` is the flat identity placement; `extreme == 1.0` is the
/// shape's most extreme pose.
fn shape_matrix(
  shape: DcompShape,
  extreme: f32,
  cw: f32,
  ch: f32,
  margin: f32,
) -> [[f32; 4]; 4] {
  match shape {
    DcompShape::Flip => {
      let to_origin = mat_translate(-cw / 2.0, -ch / 2.0, 0.0);
      let to_center = mat_translate(cw / 2.0 + margin, ch / 2.0 + margin, 0.0);
      let core = mat_mul(
        &mat_rotate_y(extreme * FLIP_MAX_DEG.to_radians()),
        &mat_perspective(cw * FLIP_DEPTH_FACTOR),
      );
      mat_mul(&mat_mul(&to_origin, &core), &to_center)
    }
    DcompShape::Hinge => {
      // Pivot about the left edge (x = 0), vertically centered; swing away
      // from the viewer (negative angle) so content foreshortens, never
      // magnifies, regardless of angle.
      let to_origin = mat_translate(0.0, -ch / 2.0, 0.0);
      let to_anchor = mat_translate(margin, ch / 2.0 + margin, 0.0);
      let core = mat_mul(
        &mat_rotate_y(-extreme * HINGE_MAX_DEG.to_radians()),
        &mat_perspective(cw * HINGE_DEPTH_FACTOR),
      );
      mat_mul(&mat_mul(&to_origin, &core), &to_anchor)
    }
    DcompShape::Spin => {
      // Pure in-plane rotation + scale (no perspective). Scale stays below 1.0
      // so corners never clip the overlay.
      let to_origin = mat_translate(-cw / 2.0, -ch / 2.0, 0.0);
      let to_center = mat_translate(cw / 2.0 + margin, ch / 2.0 + margin, 0.0);
      let scale = mat_scale(1.0 - extreme * SPIN_SHRINK);
      let rot = mat_rotate_z(-extreme * SPIN_MAX_DEG.to_radians());
      mat_mul(&mat_mul(&mat_mul(&to_origin, &scale), &rot), &to_center)
    }
  }
}

/// Builds the focus transform for `focus` at `extreme` ∈ [0, 1].
///
/// `extreme == 0.0` is the flat identity placement; `extreme == 1.0` is the
/// shape's most extreme pose.
fn focus_matrix(
  focus: DcompFocus,
  extreme: f32,
  cw: f32,
  ch: f32,
  margin: f32,
) -> [[f32; 4]; 4] {
  match focus {
    DcompFocus::Tilt => {
      let to_origin = mat_translate(-cw / 2.0, -ch / 2.0, 0.0);
      let to_center = mat_translate(cw / 2.0 + margin, ch / 2.0 + margin, 0.0);
      let scale = mat_scale(1.0 + extreme * TILT_POP);
      let rot = mat_rotate_x(extreme * TILT_MAX_DEG.to_radians());
      let core =
        mat_mul(&mat_mul(&scale, &rot), &mat_perspective(ch * TILT_DEPTH_FACTOR));
      mat_mul(&mat_mul(&to_origin, &core), &to_center)
    }
  }
}

/// A live 3D transition for a single window, backed by a [`NativeDcompSurrogate`].
///
/// The overlay is created larger than the window (by [`margin`](Self::margin))
/// so rotated/scaled content never clips at the window edges. Each frame the
/// owner calls [`apply_frame`](Self::apply_frame) with the eased progress; the
/// transform settles to an exact 1:1 placement over the real window at progress
/// `1.0`.
///
/// # Platform-specific
///
/// Windows only. Borderless capture requires Windows 11.
pub struct DcompSession {
  /// The transformable surrogate overlay.
  surrogate: NativeDcompSurrogate,
  /// The cinematic style driving the per-frame transform.
  kind: DcompTransitionKind,
  /// Handle of the captured source window, used to send `WM_CLOSE` on a
  /// completed close transition.
  source_hwnd: isize,
  /// Transparent padding (pixels) between content and each overlay edge.
  margin: f32,
  /// Whether the overlay has been revealed (shown) yet.
  ///
  /// Stays hidden until the live capture delivers its first real frame, so the
  /// overlay is never shown blank — the caller keeps the real window visible
  /// until then and cloaks it at reveal for a seamless hand-off.
  revealed: bool,
}

impl DcompSession {
  /// Creates a transition for `source_hwnd` covering `window_rect`.
  ///
  /// The overlay is inset-padded around `window_rect` so 3D content has room to
  /// extend. It is created hidden with its starting transform; the owner drives
  /// [`apply_frame`](Self::apply_frame) and calls [`reveal`](Self::reveal) once
  /// [`has_content`](Self::has_content) reports the first captured frame.
  ///
  /// Returns an error if the window cannot be captured; callers fall back to a
  /// DWM-thumbnail style.
  pub fn create(
    ctx: &DcompContext,
    source_hwnd: isize,
    window_rect: &Rect,
    kind: DcompTransitionKind,
  ) -> crate::Result<Self> {
    let w = window_rect.width() as f32;
    let h = window_rect.height() as f32;
    // Generous, content-relative padding so the near edge of a rotated card or
    // a scaled "pop" never clips. The overlay is transparent, so over-sizing it
    // is visually free.
    let margin = (0.45 * w.max(h)).clamp(80.0, 700.0);
    let mi = margin as i32;
    let overlay = Rect::from_xy(
      window_rect.x() - mi,
      window_rect.y() - mi,
      window_rect.width() + 2 * mi,
      window_rect.height() + 2 * mi,
    );

    let surrogate =
      NativeDcompSurrogate::create(ctx, HWND(source_hwnd), &overlay, false)?;
    let mut session = Self {
      surrogate,
      kind,
      source_hwnd,
      margin,
      revealed: false,
    };
    // Apply the starting transform and publish it while still hidden.
    session.apply_frame(ctx, 0.0)?;
    ctx.commit()?;
    Ok(session)
  }

  /// Whether the overlay has captured at least one real frame and is ready to
  /// be revealed without showing blank.
  pub fn has_content(&self) -> bool {
    self.surrogate.has_content()
  }

  /// Whether the overlay has already been revealed.
  pub fn is_revealed(&self) -> bool {
    self.revealed
  }

  /// Shows the overlay. Call once [`has_content`](Self::has_content) is `true`;
  /// the overlay is topmost, so showing it before cloaking the real window
  /// hands off without a blank frame.
  pub fn reveal(&mut self) {
    self.surrogate.set_visible(true);
    self.revealed = true;
  }

  /// Pulls the latest captured frame and applies this style's transform for the
  /// given eased progress. Does not commit; the owner batches one
  /// [`DcompContext::commit`] per tick.
  pub fn apply_frame(
    &mut self,
    ctx: &DcompContext,
    eased: f32,
  ) -> crate::Result<()> {
    let (cw, ch) = self.surrogate.update_capture(ctx)?;
    let matrix =
      self.kind.transform(eased, cw as f32, ch as f32, self.margin);
    self.surrogate.set_transform(&matrix)?;
    Ok(())
  }

  /// Returns this session's transition style.
  pub fn kind(&self) -> DcompTransitionKind {
    self.kind
  }

  /// Returns the handle of the captured source window.
  pub fn source_hwnd(&self) -> isize {
    self.source_hwnd
  }
}
