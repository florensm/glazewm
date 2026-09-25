//! The color theme pipeline: `Windows.Graphics.Capture` frames of the
//! themed window, run through a D3D11 pixel shader into a composition
//! swap chain shown by the overlay above that window.
//!
//! Frames never leave the GPU. The frame pool is free-threaded, so frames
//! are themed and presented on a system thread-pool thread as they arrive,
//! independent of the WM's own loop; WGC only delivers a frame when the
//! window's content changed, so an idle window costs nothing.

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc, Mutex, PoisonError,
};

use windows::{
  core::{factory, ComInterface, IInspectable},
  Foundation::{
    EventRegistrationToken, Numerics::Vector2, TypedEventHandler,
  },
  Graphics::{
    Capture::{
      Direct3D11CaptureFrame, Direct3D11CaptureFramePool,
      GraphicsCaptureItem, GraphicsCaptureSession,
    },
    DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    SizeInt32,
  },
  Win32::{
    Foundation::{CloseHandle, E_FAIL, HANDLE, HWND},
    Graphics::{
      Direct3D::{
        D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_DRIVER_TYPE,
        D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP,
      },
      Direct3D11::{
        D3D11CreateDevice, ID3D11Buffer, ID3D11Device,
        ID3D11DeviceContext, ID3D11Multithread, ID3D11PixelShader,
        ID3D11RenderTargetView, ID3D11Texture2D, ID3D11VertexShader,
        D3D11_BIND_CONSTANT_BUFFER, D3D11_BUFFER_DESC,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
        D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT, D3D11_VIEWPORT,
      },
      Dxgi::{
        Common::{
          DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
        },
        IDXGIDevice, IDXGIFactory2, IDXGISwapChain1, DXGI_SCALING_STRETCH,
        DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        DXGI_USAGE_RENDER_TARGET_OUTPUT,
      },
    },
    Security::{
      GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    },
    System::{
      Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken,
        PROCESS_QUERY_LIMITED_INFORMATION,
      },
      WinRT::{
        Composition::{ICompositorDesktopInterop, ICompositorInterop},
        Direct3D11::{
          CreateDirect3D11DeviceFromDXGIDevice,
          IDirect3DDxgiInterfaceAccess,
        },
        Graphics::Capture::IGraphicsCaptureItemInterop,
      },
    },
    UI::WindowsAndMessaging::{
      GetWindowDisplayAffinity, GetWindowThreadProcessId, WDA_NONE,
    },
  },
  UI::Composition::{
    CompositionStretch, Desktop::DesktopWindowTarget, SpriteVisual,
  },
};

use super::composition::with_composition_thread;
use crate::{color_theme::ColorTheme, Rect};

const VERTEX_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_vs.cso"));
const PIXEL_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_ps.cso"));

const PIXEL_FORMAT: DirectXPixelFormat =
  DirectXPixelFormat::B8G8R8A8UIntNormalized;

/// One buffer is held back as the last frame (see `Renderer::last_frame`),
/// leaving one free for WGC to fill.
const FRAME_POOL_BUFFERS: i32 = 2;

/// Wraps a value that is not `Send` so it can cross threads.
///
/// Only used for D3D/DXGI objects that are either free-threaded or only
/// ever touched under [`ThemedCapture::renderer`]'s lock.
struct AssertSend<T>(T);

// SAFETY: See the type's doc comment; every use documents why moving the
// wrapped value across threads is sound.
unsafe impl<T> Send for AssertSend<T> {}

/// A running capture of one window, themed and presented into a visual
/// rooted on an overlay window.
///
/// The visual stays hidden until the first themed frame is on screen, and
/// is hidden again if the pipeline fails, so the overlay never shows
/// anything but a correctly themed frame: the real window underneath
/// always shows through otherwise.
pub(crate) struct ThemedCapture {
  session: GraphicsCaptureSession,
  frame_pool: Direct3D11CaptureFramePool,
  frame_arrived: EventRegistrationToken,
  renderer: Arc<Mutex<AssertSend<Renderer>>>,
  failed: Arc<AtomicBool>,

  /// Binds the visual to the overlay's `HWND`; dropping it unbinds.
  _target: DesktopWindowTarget,
}

impl ThemedCapture {
  /// Starts capturing `source` into a visual rooted on `overlay`, which
  /// must have been created with `WS_EX_NOREDIRECTIONBITMAP`.
  pub(crate) fn start(
    source: HWND,
    overlay: HWND,
    rect: &Rect,
    theme: &ColorTheme,
  ) -> crate::Result<Self> {
    if !GraphicsCaptureSession::IsSupported()? {
      return Err(crate::Error::Platform(
        "Windows.Graphics.Capture is not supported on this system."
          .to_string(),
      ));
    }

    if is_more_elevated(source) {
      return Err(crate::Error::Platform(
        "Window belongs to an elevated process, which cannot be captured \
         without running the WM as administrator."
          .to_string(),
      ));
    }

    // A window excluded from capture arrives as solid black, which the
    // theme would turn into an opaque slab over the real window.
    let mut affinity = 0u32;
    // SAFETY: `affinity` outlives the call; a stale `source` just fails.
    if unsafe { GetWindowDisplayAffinity(source, &raw mut affinity) }
      .is_ok()
      && affinity != WDA_NONE.0
    {
      return Err(crate::Error::Platform(
        "Window blocks screen capture (display affinity).".to_string(),
      ));
    }

    let failed = Arc::new(AtomicBool::new(false));
    let size = (
      u32::try_from(rect.width().max(1))?,
      u32::try_from(rect.height().max(1))?,
    );
    let device = GpuDevice::create(theme, size)?;
    let swap_chain = AssertSend(device.swap_chain.clone());
    let source_raw = source.0;
    let overlay_raw = overlay.0;

    // Composition and WGC objects are agile, but are created on the
    // composition thread since it is guaranteed to have WinRT initialized.
    let (target, sprite, item) =
      with_composition_thread(move |compositor, _| {
        // SAFETY: `overlay` is a live top-level window.
        let target = unsafe {
          compositor
            .cast::<ICompositorDesktopInterop>()?
            .CreateDesktopWindowTarget(HWND(overlay_raw), false)?
        };

        // SAFETY: `swap_chain` is a composition swap chain that nothing
        // presents to until the capture below starts.
        let surface = unsafe {
          compositor
            .cast::<ICompositorInterop>()?
            .CreateCompositionSurfaceForSwapChain(&swap_chain.0)?
        };

        // Drawn 1:1 from the top-left corner: the swap chain always
        // matches the captured window's size, which the overlay does too.
        let brush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
        brush.SetStretch(CompositionStretch::None)?;
        brush.SetHorizontalAlignmentRatio(0.0)?;
        brush.SetVerticalAlignmentRatio(0.0)?;

        let sprite = compositor.CreateSpriteVisual()?;
        sprite.SetBrush(&brush)?;
        sprite.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
        sprite.SetIsVisible(false)?;
        target.SetRoot(&sprite)?;

        let interop =
          factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        // SAFETY: `source` is a window handle; a stale one just fails.
        let item: GraphicsCaptureItem =
          unsafe { interop.CreateForWindow(HWND(source_raw))? };

        Ok((target, sprite, item))
      })?;

    let item_size = item.Size()?;
    let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
      &device.winrt_device,
      PIXEL_FORMAT,
      FRAME_POOL_BUFFERS,
      item_size,
    )?;
    let session = frame_pool.CreateCaptureSession(&item)?;

    // The cursor is drawn by the system above the overlay already, and a
    // second copy baked into the frame would lag behind it.
    if let Err(err) = session.SetIsCursorCaptureEnabled(false) {
      tracing::debug!("Could not exclude the cursor from capture: {err}.");
    }

    // Windows 11+ only; Windows 10 always draws the yellow border.
    if let Err(err) = session.SetIsBorderRequired(false) {
      tracing::debug!("Could not disable the capture border: {err}.");
    }

    let renderer = Arc::new(Mutex::new(AssertSend(Renderer {
      device,
      sprite,
      pool_size: item_size,
      last_frame: None,
      is_shown: false,
      failed: failed.clone(),
    })));

    let handler_renderer = renderer.clone();
    let frame_arrived =
      frame_pool.FrameArrived(&TypedEventHandler::<
        Direct3D11CaptureFramePool,
        IInspectable,
      >::new(move |pool, _| {
        if let Some(pool) = pool {
          let mut renderer = handler_renderer
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

          if let Err(err) = renderer.0.on_frame_arrived(pool) {
            renderer.0.fail(&err);
          }
        }

        Ok(())
      }))?;

    session.StartCapture()?;

    Ok(Self {
      session,
      frame_pool,
      frame_arrived,
      renderer,
      failed,
      _target: target,
    })
  }

  /// Switches to `theme`, re-rendering the current frame with it.
  pub(crate) fn set_theme(&self, theme: &ColorTheme) {
    let mut renderer =
      self.renderer.lock().unwrap_or_else(PoisonError::into_inner);

    if let Err(err) = renderer.0.set_theme(theme) {
      renderer.0.fail(&err);
    }
  }

  /// Whether the pipeline has stopped for good (e.g. GPU device removed).
  pub(crate) fn has_failed(&self) -> bool {
    self.failed.load(Ordering::Relaxed)
  }
}

impl Drop for ThemedCapture {
  fn drop(&mut self) {
    let _ = self.frame_pool.RemoveFrameArrived(self.frame_arrived);
    let _ = self.session.Close();
    let _ = self.frame_pool.Close();
  }
}

/// The D3D11 objects a themed frame is rendered with.
struct GpuDevice {
  device: ID3D11Device,
  context: ID3D11DeviceContext,

  /// `device`, as the frame pool needs it.
  winrt_device: IDirect3DDevice,

  vertex_shader: ID3D11VertexShader,
  pixel_shader: ID3D11PixelShader,
  constants: ID3D11Buffer,

  /// [`FrameConstants`] for the current `swap_chain_size`.
  frame_constants: ID3D11Buffer,

  swap_chain: IDXGISwapChain1,
  swap_chain_size: (u32, u32),

  /// View of the swap chain's current back buffer. With the flip model,
  /// buffer 0 always refers to it, so this stays valid across presents
  /// until the buffers are resized.
  render_target: Option<ID3D11RenderTargetView>,
}

impl GpuDevice {
  fn create(theme: &ColorTheme, size: (u32, u32)) -> crate::Result<Self> {
    let device = create_d3d_device()?;

    // SAFETY: `device` is a live device.
    let context = unsafe { device.GetImmediateContext()? };

    // WGC copies into the frame pool's textures from its own threads
    // while frames are themed on the thread pool.
    // SAFETY: Every D3D11.4+ device implements `ID3D11Multithread`.
    unsafe {
      device
        .cast::<ID3D11Multithread>()?
        .SetMultithreadProtected(true);
    }

    let dxgi_device: IDXGIDevice = device.cast()?;

    // SAFETY: `dxgi_device` is a live DXGI device.
    let winrt_device: IDirect3DDevice =
      unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)? }
        .cast()?;

    let mut vertex_shader = None;
    let mut pixel_shader = None;
    // SAFETY: The bytecode was produced by fxc for these exact stages, and
    // the out-parameters outlive the calls.
    unsafe {
      device.CreateVertexShader(
        VERTEX_SHADER,
        None,
        Some(&raw mut vertex_shader),
      )?;
      device.CreatePixelShader(
        PIXEL_SHADER,
        None,
        Some(&raw mut pixel_shader),
      )?;
    }

    let constants = create_constant_buffer(&device, theme.constants())?;
    let frame_constants =
      create_constant_buffer(&device, &FrameConstants::new(size))?;

    // SAFETY: The adapter's parent is the factory that created it, and
    // every DXGI 1.2+ factory implements `IDXGIFactory2`.
    let factory: IDXGIFactory2 =
      unsafe { dxgi_device.GetAdapter()?.GetParent()? };

    let desc = DXGI_SWAP_CHAIN_DESC1 {
      Width: size.0,
      Height: size.1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
      BufferCount: 2,
      Scaling: DXGI_SCALING_STRETCH,
      SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
      AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
      ..Default::default()
    };

    // SAFETY: `desc` describes a valid composition swap chain and
    // outlives the call.
    let swap_chain = unsafe {
      factory.CreateSwapChainForComposition(
        &device,
        &raw const desc,
        None,
      )?
    };

    Ok(Self {
      device,
      context,
      winrt_device,
      vertex_shader: created(vertex_shader)?,
      pixel_shader: created(pixel_shader)?,
      constants,
      frame_constants,
      swap_chain,
      swap_chain_size: size,
      render_target: None,
    })
  }

  /// Themes the top-left `size` of `texture` into the swap chain and
  /// presents it.
  fn render(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<()> {
    if self.swap_chain_size != size {
      self.render_target = None;

      // SAFETY: Unbinding everything first releases the pipeline's
      // references to the old back buffers, which `ResizeBuffers`
      // requires.
      unsafe {
        self.context.ClearState();
        self.swap_chain.ResizeBuffers(
          0,
          size.0,
          size.1,
          DXGI_FORMAT_UNKNOWN,
          0,
        )?;
      }

      // SAFETY: The source is a `FrameConstants`, exactly the buffer's
      // size, and the context is only used under the caller's lock.
      unsafe {
        self.context.UpdateSubresource(
          &self.frame_constants,
          0,
          None,
          std::ptr::from_ref(&FrameConstants::new(size)).cast(),
          0,
          0,
        );
      }

      self.swap_chain_size = size;
    }

    if self.render_target.is_none() {
      let mut view = None;
      // SAFETY: Buffer 0 of a flip-model swap chain is its current back
      // buffer, and `view` outlives the call.
      unsafe {
        let back_buffer: ID3D11Texture2D = self.swap_chain.GetBuffer(0)?;
        self.device.CreateRenderTargetView(
          &back_buffer,
          None,
          Some(&raw mut view),
        )?;
      }
      self.render_target = Some(created(view)?);
    }
    let render_target = self.render_target.clone();

    let mut source = None;
    // SAFETY: `texture` is a live frame-pool texture and `source`
    // outlives the call.
    unsafe {
      self.device.CreateShaderResourceView(
        texture,
        None,
        Some(&raw mut source),
      )?;
    }

    #[allow(clippy::cast_precision_loss)]
    let viewport = D3D11_VIEWPORT {
      TopLeftX: 0.0,
      TopLeftY: 0.0,
      Width: size.0 as f32,
      Height: size.1 as f32,
      MinDepth: 0.0,
      MaxDepth: 1.0,
    };

    // SAFETY: Every bound object is alive for the duration of the draw,
    // and access to the immediate context is serialized by the caller's
    // lock.
    unsafe {
      let context = &self.context;
      context.IASetInputLayout(None);
      context
        .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&self.vertex_shader, None);
      context.PSSetShader(&self.pixel_shader, None);
      context.PSSetConstantBuffers(
        0,
        Some(&[
          Some(self.constants.clone()),
          Some(self.frame_constants.clone()),
        ]),
      );
      context.PSSetShaderResources(0, Some(&[source]));
      context.OMSetRenderTargets(Some(&[render_target]), None);
      context.RSSetViewports(Some(&[viewport]));
      context.Draw(3, 0);

      // Unbound so the frame pool can hand the texture back out.
      context.PSSetShaderResources(0, Some(&[None]));

      self.swap_chain.Present(1, 0).ok()?;
    }

    Ok(())
  }
}

/// Per-capture state, shared between the frame-arrived callback and the
/// WM thread.
struct Renderer {
  device: GpuDevice,

  /// The visual presenting the swap chain; hidden until the first frame
  /// has been themed.
  sprite: SpriteVisual,

  /// Size the frame pool's buffers were last created at.
  pool_size: SizeInt32,

  /// The most recent frame, kept so a theme change can re-render it
  /// without waiting for the window to repaint.
  last_frame:
    Option<(Direct3D11CaptureFrame, ID3D11Texture2D, (u32, u32))>,

  is_shown: bool,
  failed: Arc<AtomicBool>,
}

impl Renderer {
  fn on_frame_arrived(
    &mut self,
    pool: &Direct3D11CaptureFramePool,
  ) -> crate::Result<()> {
    if self.failed.load(Ordering::Relaxed) {
      return Ok(());
    }

    // Spurious callbacks without a frame are possible, and harmless.
    let Ok(frame) = pool.TryGetNextFrame() else {
      return Ok(());
    };

    let content_size = frame.ContentSize()?;
    let (Ok(width), Ok(height)) = (
      u32::try_from(content_size.Width),
      u32::try_from(content_size.Height),
    ) else {
      return Ok(());
    };

    // Minimized windows report an empty frame; keep showing the last one.
    if width == 0 || height == 0 {
      return Ok(());
    }

    let texture = frame_texture(&frame)?;
    self.present(&texture, (width, height))?;

    if content_size == self.pool_size {
      self.last_frame = Some((frame, texture, (width, height)));
    } else {
      // The window was resized. Frames from the old pool can't be kept
      // across `Recreate`, and this one was cropped to the old size.
      self.last_frame = None;
      drop(frame);
      pool.Recreate(
        &self.device.winrt_device,
        PIXEL_FORMAT,
        FRAME_POOL_BUFFERS,
        content_size,
      )?;
      self.pool_size = content_size;
    }

    Ok(())
  }

  fn set_theme(&mut self, theme: &ColorTheme) -> crate::Result<()> {
    if self.failed.load(Ordering::Relaxed) {
      return Ok(());
    }

    // SAFETY: The source is a `ThemeConstants`, exactly the buffer's
    // size, and the context is only used under the caller's lock.
    unsafe {
      self.device.context.UpdateSubresource(
        &self.device.constants,
        0,
        None,
        std::ptr::from_ref(theme.constants()).cast(),
        0,
        0,
      );
    }

    if let Some((_, texture, size)) = self.last_frame.clone() {
      self.present(&texture, size)?;
    }

    Ok(())
  }

  fn present(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<()> {
    self.device.render(texture, size)?;

    if !self.is_shown {
      self.sprite.SetIsVisible(true)?;
      self.is_shown = true;
    }

    Ok(())
  }

  /// Hides the visual for good, so the unthemed window shows through,
  /// and logs the cause once.
  fn fail(&mut self, err: &crate::Error) {
    if self.failed.swap(true, Ordering::Relaxed) {
      return;
    }

    tracing::warn!("Color theme stopped: {err}.");
    self.last_frame = None;

    if let Err(err) = self.sprite.SetIsVisible(false) {
      tracing::warn!("Failed to hide color theme visual: {err}.");
    }
  }
}

/// Unwraps a D3D out-parameter, which is only `None` if the call that
/// should have filled it failed.
fn created<T>(value: Option<T>) -> crate::Result<T> {
  value.ok_or_else(|| windows::core::Error::from(E_FAIL).into())
}

fn frame_texture(
  frame: &Direct3D11CaptureFrame,
) -> crate::Result<ID3D11Texture2D> {
  let access: IDirect3DDxgiInterfaceAccess = frame.Surface()?.cast()?;
  // SAFETY: WGC surfaces wrap a D3D11 texture on the pool's device.
  Ok(unsafe { access.GetInterface()? })
}

/// Constant buffer layout shared with `cbuffer Frame` in the shader.
#[repr(C)]
struct FrameConstants {
  size: [u32; 2],
  _padding: [u32; 2],
}

impl FrameConstants {
  fn new(size: (u32, u32)) -> Self {
    Self {
      size: [size.0, size.1],
      _padding: [0; 2],
    }
  }
}

fn create_constant_buffer<T>(
  device: &ID3D11Device,
  constants: &T,
) -> crate::Result<ID3D11Buffer> {
  let desc = D3D11_BUFFER_DESC {
    ByteWidth: u32::try_from(std::mem::size_of::<T>())?,
    Usage: D3D11_USAGE_DEFAULT,
    #[allow(clippy::cast_sign_loss)]
    BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
    ..Default::default()
  };
  let data = D3D11_SUBRESOURCE_DATA {
    pSysMem: std::ptr::from_ref(constants).cast(),
    ..Default::default()
  };

  let mut buffer = None;
  // SAFETY: `desc` and `data` describe a buffer exactly the size of
  // `constants`, and all pointers outlive the call.
  unsafe {
    device.CreateBuffer(
      &raw const desc,
      Some(&raw const data),
      Some(&raw mut buffer),
    )?;
  }

  created(buffer)
}

/// Creates a BGRA-capable D3D11 device, preferring the GPU and falling
/// back to the software rasterizer.
fn create_d3d_device() -> crate::Result<ID3D11Device> {
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

  Ok(create(D3D_DRIVER_TYPE_HARDWARE).or_else(|err| {
    tracing::debug!("No hardware D3D11 device for color themes, falling back to WARP: {err}.");
    create(D3D_DRIVER_TYPE_WARP)
  })?)
}

/// Owned kernel handle, closed on drop.
struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
  fn drop(&mut self) {
    // SAFETY: The handle is owned by `self` and closed exactly once here.
    unsafe {
      let _ = CloseHandle(self.0);
    }
  }
}

/// Whether `hwnd` belongs to an elevated process while the WM itself is
/// not elevated, which puts the window out of WGC's reach.
///
/// A process that can't be queried counts as elevated: from a
/// non-elevated caller, that is the case that fails.
fn is_more_elevated(hwnd: HWND) -> bool {
  // SAFETY: Returns a pseudo handle that needs no closing.
  if token_is_elevated(unsafe { GetCurrentProcess() }) == Some(true) {
    return false;
  }

  let mut process_id = 0u32;
  // SAFETY: `process_id` outlives the call; a stale `hwnd` leaves it 0.
  unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut process_id)) };

  // SAFETY: No preconditions; failure is handled.
  let Ok(process) = (unsafe {
    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
  }) else {
    return true;
  };

  let process = OwnedHandle(process);
  token_is_elevated(process.0) != Some(false)
}

/// Whether `process`'s token is elevated, or `None` if it can't be read.
fn token_is_elevated(process: HANDLE) -> Option<bool> {
  let mut token = HANDLE::default();
  // SAFETY: `process` is a live process handle and `token` outlives the
  // call.
  unsafe { OpenProcessToken(process, TOKEN_QUERY, &raw mut token) }
    .ok()?;
  let token = OwnedHandle(token);

  let mut elevation = TOKEN_ELEVATION::default();
  let mut length = 0u32;

  // SAFETY: `elevation` is exactly the size passed, and both
  // out-parameters outlive the call.
  unsafe {
    GetTokenInformation(
      token.0,
      TokenElevation,
      Some(std::ptr::from_mut(&mut elevation).cast()),
      u32::try_from(std::mem::size_of::<TOKEN_ELEVATION>()).ok()?,
      &raw mut length,
    )
  }
  .ok()?;

  Some(elevation.TokenIsElevated != 0)
}
