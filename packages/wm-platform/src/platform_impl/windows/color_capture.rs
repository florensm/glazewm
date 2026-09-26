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
  Arc, Mutex, PoisonError, Weak,
};

use windows::{
  core::{factory, ComInterface, IInspectable},
  Foundation::{EventRegistrationToken, TypedEventHandler},
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
        D3D11_BIND_CONSTANT_BUFFER, D3D11_BOX, D3D11_BUFFER_DESC,
        D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_SDK_VERSION,
        D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
        D3D11_USAGE_STAGING, D3D11_VIEWPORT,
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
    CompositionColorBrush, CompositionStretch,
    Desktop::DesktopWindowTarget, SpriteVisual,
  },
};

use super::{
  composition::{to_ui_color, with_composition_thread, FILL_PARENT},
  image_finder::ImageFinder,
};
use crate::{
  color_theme::{picture_extent, ColorTheme},
  Color, Rect,
};

const VERTEX_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_vs.cso"));
const PIXEL_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_ps.cso"));

const PIXEL_FORMAT: DirectXPixelFormat =
  DirectXPixelFormat::B8G8R8A8UIntNormalized;

/// Most images per window the shader keeps in their own colors; sizes its
/// constant buffer.
pub(crate) const MAX_IMAGE_RECTS: usize = 32;

/// Most pixels an image is judged from; larger images are sampled on a
/// grid.
const PICTURE_MAX_SAMPLES: i32 = 512 * 512;

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
///
/// Beneath the frame sits an optional solid fill, covering whatever part
/// of the overlay the last frame doesn't reach (e.g. while it grows ahead
/// of the window during an animation).
pub(crate) struct ThemedCapture {
  session: GraphicsCaptureSession,
  frame_pool: Direct3D11CaptureFramePool,
  frame_arrived: EventRegistrationToken,
  renderer: Arc<Mutex<AssertSend<Renderer>>>,
  failed: Arc<AtomicBool>,

  /// Finds the window's pictures, kept in their own colors; `None` if
  /// its thread couldn't start, in which case everything is themed.
  images: Option<Arc<ImageFinder>>,

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
    ensure_capturable(source)?;

    let failed = Arc::new(AtomicBool::new(false));
    let size = (
      u32::try_from(rect.width().max(1))?,
      u32::try_from(rect.height().max(1))?,
    );
    let output = CaptureOutput::create(theme, size)?;
    let swap_chain = AssertSend(output.swap_chain.clone());
    let source_raw = source.0;
    let overlay_raw = overlay.0;

    // Composition and WGC objects are agile, but are created on the
    // composition thread since it is guaranteed to have WinRT initialized.
    let (target, sprite, fill, fill_brush, item) =
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
        sprite.SetRelativeSizeAdjustment(FILL_PARENT)?;
        sprite.SetIsVisible(false)?;

        let fill_brush = compositor.CreateColorBrush()?;
        let fill = compositor.CreateSpriteVisual()?;
        fill.SetBrush(&fill_brush)?;
        fill.SetRelativeSizeAdjustment(FILL_PARENT)?;
        fill.SetIsVisible(false)?;

        let root = compositor.CreateContainerVisual()?;
        root.SetRelativeSizeAdjustment(FILL_PARENT)?;
        root.Children()?.InsertAtBottom(&fill)?;
        root.Children()?.InsertAtTop(&sprite)?;
        target.SetRoot(&root)?;

        let interop =
          factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        // SAFETY: `source` is a window handle; a stale one just fails.
        let item: GraphicsCaptureItem =
          unsafe { interop.CreateForWindow(HWND(source_raw))? };

        Ok((target, sprite, fill, fill_brush, item))
      })?;

    let item_size = item.Size()?;
    let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
      &output.winrt_device,
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
      output,
      sprite,
      fill,
      fill_brush,
      fill_color: None,
      fill_until_covered: None,
      fill_before_first_frame: false,
      pool_size: item_size,
      last_frame: None,
      images: Vec::new(),
      is_shown: false,
      failed: failed.clone(),
    })));

    let images = start_image_finder(source, Arc::downgrade(&renderer));
    let frame_arrived = frame_pool
      .FrameArrived(&frame_handler(renderer.clone(), images.clone()))?;

    session.StartCapture()?;

    Ok(Self {
      session,
      frame_pool,
      frame_arrived,
      renderer,
      failed,
      images,
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

  /// Sets the fill beneath the frame, or removes it with `None`.
  pub(crate) fn set_fill(&self, color: Option<Color>) {
    let mut renderer =
      self.renderer.lock().unwrap_or_else(PoisonError::into_inner);

    if renderer.0.fill_color == color {
      return;
    }

    renderer.0.fill_color = color;
    renderer.0.fill_until_covered = None;
    renderer.0.fill_before_first_frame = false;

    if let Err(err) = renderer.0.sync_fill() {
      renderer.0.fail(&err);
    }
  }

  /// Keeps the current fill until a frame of at least `size` has been
  /// presented, then removes it.
  ///
  /// For the end of an animation: the window has just been resized, and
  /// the last frame, still at the old size, would otherwise leave the
  /// unthemed window showing around it until the app repaints.
  pub(crate) fn release_fill_when_covered(&self, size: (u32, u32)) {
    let mut renderer =
      self.renderer.lock().unwrap_or_else(PoisonError::into_inner);

    if renderer.0.fill_color.is_some() {
      renderer.0.fill_until_covered = Some(size);
    }
  }

  /// Shows `color` as the fill right away, even before the first frame,
  /// until a frame of at least `size` has been presented.
  ///
  /// For windows that appear abruptly (menus, dropdowns): capture takes a
  /// few frames to start, during which the overlay would otherwise be
  /// transparent and show the unthemed window.
  pub(crate) fn set_placeholder(&self, color: Color, size: (u32, u32)) {
    let mut renderer =
      self.renderer.lock().unwrap_or_else(PoisonError::into_inner);

    renderer.0.fill_color = Some(color);
    renderer.0.fill_until_covered = Some(size);
    renderer.0.fill_before_first_frame = true;

    if let Err(err) = renderer.0.sync_fill() {
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

    // Stops the finder even if the frame handler still holds it.
    if let Some(images) = self.images.take() {
      images.stop();
    }
  }
}

/// Themes each frame as it arrives, and has `images` look again, since
/// images may have moved with the content.
fn frame_handler(
  renderer: Arc<Mutex<AssertSend<Renderer>>>,
  images: Option<Arc<ImageFinder>>,
) -> TypedEventHandler<Direct3D11CaptureFramePool, IInspectable> {
  TypedEventHandler::new(
    move |pool: &Option<Direct3D11CaptureFramePool>, _| {
      if let Some(pool) = pool {
        let mut renderer =
          renderer.lock().unwrap_or_else(PoisonError::into_inner);

        if let Err(err) = renderer.0.on_frame_arrived(pool) {
          renderer.0.fail(&err);
        }
      }

      if let Some(images) = &images {
        images.content_changed();
      }

      Ok(())
    },
  )
}

/// Starts finding `source`'s images for `renderer`, or logs why not.
fn start_image_finder(
  source: HWND,
  renderer: Weak<Mutex<AssertSend<Renderer>>>,
) -> Option<Arc<ImageFinder>> {
  let finder = ImageFinder::start(source, move |found| {
    let Some(renderer) = renderer.upgrade() else {
      return;
    };
    let mut renderer =
      renderer.lock().unwrap_or_else(PoisonError::into_inner);

    // Pictures are a refinement: on failure, the window stays fully
    // themed rather than losing its theme.
    if let Err(err) = renderer.0.update_images(&found) {
      tracing::debug!("Could not update color theme images: {err}.");
    }
  });

  match finder {
    Ok(finder) => Some(Arc::new(finder)),
    Err(err) => {
      tracing::warn!("Images won't keep their colors: {err}.");
      None
    }
  }
}

/// Fails for windows the capture can't theme correctly, so they are left
/// alone rather than covered.
fn ensure_capturable(source: HWND) -> crate::Result<()> {
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
  if unsafe { GetWindowDisplayAffinity(source, &raw mut affinity) }.is_ok()
    && affinity != WDA_NONE.0
  {
    return Err(crate::Error::Platform(
      "Window blocks screen capture (display affinity).".to_string(),
    ));
  }

  Ok(())
}

/// The D3D11 device and pipeline every themed capture renders with.
///
/// Shared process-wide: creating a device takes tens of milliseconds, too
/// slow to pay on the WM's thread for every menu and tooltip. Its
/// immediate context is single-threaded, so every use holds
/// [`SharedGpu`]'s lock.
struct Gpu {
  device: ID3D11Device,
  context: ID3D11DeviceContext,

  /// `device`, as frame pools need it.
  winrt_device: IDirect3DDevice,

  vertex_shader: ID3D11VertexShader,
  pixel_shader: ID3D11PixelShader,
}

// SAFETY: `AssertSend` because D3D11 interfaces aren't `Send`; the device
// is free-threaded, and its context is only used under the mutex.
type SharedGpu = Arc<Mutex<AssertSend<Gpu>>>;

/// The shared [`Gpu`], created on first use and again after the GPU device
/// was lost (driver update, TDR).
fn shared_gpu() -> crate::Result<SharedGpu> {
  static GPU: Mutex<Option<SharedGpu>> = Mutex::new(None);

  let mut slot = GPU.lock().unwrap_or_else(PoisonError::into_inner);

  if let Some(gpu) = slot.as_ref() {
    let guard = gpu.lock().unwrap_or_else(PoisonError::into_inner);
    // SAFETY: The device is alive for as long as the `Gpu` holds it.
    if unsafe { guard.0.device.GetDeviceRemovedReason() }.is_ok() {
      return Ok(gpu.clone());
    }
  }

  let gpu = Arc::new(Mutex::new(AssertSend(Gpu::create()?)));
  *slot = Some(gpu.clone());
  Ok(gpu)
}

impl Gpu {
  fn create() -> crate::Result<Self> {
    let device = create_d3d_device()?;

    // SAFETY: `device` is a live device.
    let context = unsafe { device.GetImmediateContext()? };

    // WGC copies into the frame pools' textures from its own threads
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

    Ok(Self {
      device,
      context,
      winrt_device,
      vertex_shader: created(vertex_shader)?,
      pixel_shader: created(pixel_shader)?,
    })
  }
}

/// One capture's rendering target on the shared [`Gpu`].
struct CaptureOutput {
  gpu: SharedGpu,

  /// The shared device, as the frame pool needs it; cloned so pool calls
  /// don't need the lock.
  winrt_device: IDirect3DDevice,

  constants: ID3D11Buffer,

  /// [`FrameConstants`] for the current `swap_chain_size`.
  frame_constants: ID3D11Buffer,

  /// [`ImageConstants`] for the pictures kept in their own colors.
  image_constants: ID3D11Buffer,

  swap_chain: IDXGISwapChain1,
  swap_chain_size: (u32, u32),

  /// View of the swap chain's current back buffer. With the flip model,
  /// buffer 0 always refers to it, so this stays valid across presents
  /// until the buffers are resized.
  render_target: Option<ID3D11RenderTargetView>,
}

impl CaptureOutput {
  fn create(theme: &ColorTheme, size: (u32, u32)) -> crate::Result<Self> {
    let gpu = shared_gpu()?;
    let guard = gpu.lock().unwrap_or_else(PoisonError::into_inner);
    let device = &guard.0.device;

    let constants = create_constant_buffer(device, theme.constants())?;
    let frame_constants =
      create_constant_buffer(device, &FrameConstants::new(size))?;
    let image_constants =
      create_constant_buffer(device, &ImageConstants::new(&[]))?;

    // SAFETY: The adapter's parent is the factory that created it, and
    // every DXGI 1.2+ factory implements `IDXGIFactory2`.
    let factory: IDXGIFactory2 =
      unsafe { device.cast::<IDXGIDevice>()?.GetAdapter()?.GetParent()? };

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
        device,
        &raw const desc,
        None,
      )?
    };

    let winrt_device = guard.0.winrt_device.clone();
    drop(guard);

    Ok(Self {
      gpu,
      winrt_device,
      constants,
      frame_constants,
      image_constants,
      swap_chain,
      swap_chain_size: size,
      render_target: None,
    })
  }

  /// Uploads `theme` for the next render.
  fn set_theme(&self, theme: &ColorTheme) {
    let gpu = self.gpu.lock().unwrap_or_else(PoisonError::into_inner);

    // SAFETY: The source is a `ThemeConstants`, exactly the buffer's
    // size, and the context is only used under the lock held above.
    unsafe {
      gpu.0.context.UpdateSubresource(
        &self.constants,
        0,
        None,
        std::ptr::from_ref(theme.constants()).cast(),
        0,
        0,
      );
    }
  }

  /// Uploads the `pictures` to keep in their own colors for the next
  /// render.
  fn set_images(&self, pictures: &[Rect]) {
    let gpu = self.gpu.lock().unwrap_or_else(PoisonError::into_inner);

    // SAFETY: The source is an `ImageConstants`, exactly the buffer's
    // size, and the context is only used under the lock held above.
    unsafe {
      gpu.0.context.UpdateSubresource(
        &self.image_constants,
        0,
        None,
        std::ptr::from_ref(&ImageConstants::new(pictures)).cast(),
        0,
        0,
      );
    }
  }

  /// The part of the image reported at `rect` of `texture` (whose content
  /// is `size`) that shows a picture, or `None` for an icon; see
  /// [`picture_extent`].
  ///
  /// Reads the image and a 2px margin of page around it back from the
  /// GPU, the margin sampled at the same points the shader uses.
  fn picture_rect(
    &self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
    rect: &Rect,
  ) -> crate::Result<Option<Rect>> {
    let frame = Rect::from_ltrb(
      0,
      0,
      i32::try_from(size.0)?,
      i32::try_from(size.1)?,
    );
    let region = Rect::from_ltrb(
      (rect.left - 2).max(0),
      (rect.top - 2).max(0),
      (rect.right + 2).min(frame.right),
      (rect.bottom + 2).min(frame.bottom),
    );

    if region.width() <= 0 || region.height() <= 0 {
      return Ok(None);
    }

    let pixels = self.read_straight(texture, &region)?;
    let at = |x: i32, y: i32| -> Option<[f32; 3]> {
      if x < region.left
        || y < region.top
        || x >= region.right
        || y >= region.bottom
      {
        return None;
      }

      let index = usize::try_from(
        (y - region.top) * region.width() + (x - region.left),
      )
      .ok()?;
      pixels.get(index).copied().flatten()
    };

    let middle = (
      rect.left.midpoint(rect.right),
      rect.top.midpoint(rect.bottom),
    );
    let papers: Vec<[f32; 3]> = [
      (rect.left - 2, rect.top - 2),
      (middle.0, rect.top - 2),
      (rect.right + 1, rect.top - 2),
      (rect.left - 2, middle.1),
      (rect.right + 1, middle.1),
      (rect.left - 2, rect.bottom + 1),
      (middle.0, rect.bottom + 1),
      (rect.right + 1, rect.bottom + 1),
    ]
    .into_iter()
    .filter_map(|(x, y)| at(x, y))
    .collect();

    // Transparent pixels (off the frame, rounded corners) read as page.
    let page = papers.first().copied().unwrap_or([1.0; 3]);
    let area = i64::from(rect.width()) * i64::from(rect.height());
    #[allow(
      clippy::cast_possible_truncation,
      clippy::cast_precision_loss,
      clippy::cast_sign_loss
    )]
    let step = (area as f64 / f64::from(PICTURE_MAX_SAMPLES))
      .sqrt()
      .ceil()
      .max(1.0) as usize;

    let columns: Vec<i32> =
      (rect.left..rect.right).step_by(step).collect();
    let content: Vec<[f32; 3]> = (rect.top..rect.bottom)
      .step_by(step)
      .flat_map(|y| columns.iter().map(move |&x| (x, y)))
      .map(|(x, y)| at(x, y).unwrap_or(page))
      .collect();

    let Some([left, top, right, bottom]) =
      picture_extent(&content, columns.len(), &papers)
    else {
      return Ok(None);
    };

    // Back from sample indices to frame pixels.
    let to_frame =
      |index: usize, start: i32, end: i32| -> crate::Result<i32> {
        Ok((start + i32::try_from(index * step)?).min(end))
      };

    Ok(Some(Rect::from_ltrb(
      to_frame(left, rect.left, rect.right)?,
      to_frame(top, rect.top, rect.bottom)?,
      to_frame(right, rect.left, rect.right)?,
      to_frame(bottom, rect.top, rect.bottom)?,
    )))
  }

  /// Straight-alpha sRGB of `region` of `texture`, row-major; fully
  /// transparent pixels are `None`.
  fn read_straight(
    &self,
    texture: &ID3D11Texture2D,
    region: &Rect,
  ) -> crate::Result<Vec<Option<[f32; 3]>>> {
    let width = u32::try_from(region.width())?;
    let height = u32::try_from(region.height())?;

    let gpu = self.gpu.lock().unwrap_or_else(PoisonError::into_inner);
    let Gpu {
      device, context, ..
    } = &gpu.0;

    let desc = D3D11_TEXTURE2D_DESC {
      Width: width,
      Height: height,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_STAGING,
      BindFlags: 0,
      #[allow(clippy::cast_sign_loss)]
      CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
      MiscFlags: 0,
    };

    let mut staging = None;
    // SAFETY: `desc` describes a valid staging texture and both pointers
    // outlive the call.
    unsafe {
      device.CreateTexture2D(
        &raw const desc,
        None,
        Some(&raw mut staging),
      )?;
    }
    let staging = created(staging)?;

    let source_box = D3D11_BOX {
      left: u32::try_from(region.left)?,
      top: u32::try_from(region.top)?,
      front: 0,
      right: u32::try_from(region.right)?,
      bottom: u32::try_from(region.bottom)?,
      back: 1,
    };

    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: The box lies within the frame's content, which the frame
    // texture covers, and the destination is exactly the box's size. The
    // context is only used under the lock held above.
    unsafe {
      context.CopySubresourceRegion(
        &staging,
        0,
        0,
        0,
        0,
        texture,
        0,
        Some(&raw const source_box),
      );
      context.Map(
        &staging,
        0,
        D3D11_MAP_READ,
        0,
        Some(&raw mut mapped),
      )?;
    }

    let row_pitch = usize::try_from(mapped.RowPitch)?;
    let (width, height) =
      (usize::try_from(width)?, usize::try_from(height)?);
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
      // SAFETY: A mapped staging texture holds `height` rows of
      // `row_pitch` bytes, each starting with `width` BGRA pixels, and
      // stays mapped until `Unmap` below.
      let row = unsafe {
        std::slice::from_raw_parts(
          mapped.pData.cast::<u8>().add(y * row_pitch),
          width * 4,
        )
      };

      pixels.extend(row.as_chunks::<4>().0.iter().map(|bgra| {
        let alpha = f32::from(bgra[3]);
        (alpha > 0.0).then(|| {
          [bgra[2], bgra[1], bgra[0]].map(|c| f32::from(c) / alpha)
        })
      }));
    }

    // SAFETY: Mapped above, and not read from past this point.
    unsafe { context.Unmap(&staging, 0) };

    Ok(pixels)
  }

  /// Themes the top-left `size` of `texture` into the swap chain and
  /// presents it.
  fn render(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<()> {
    let gpu = self.gpu.clone();
    let guard = gpu.lock().unwrap_or_else(PoisonError::into_inner);
    let Gpu {
      device,
      context,
      vertex_shader,
      pixel_shader,
      ..
    } = &guard.0;

    if self.swap_chain_size != size {
      self.render_target = None;

      // SAFETY: Unbinding everything first releases the pipeline's
      // references to the old back buffers, which `ResizeBuffers`
      // requires.
      unsafe {
        context.ClearState();
        self.swap_chain.ResizeBuffers(
          0,
          size.0,
          size.1,
          DXGI_FORMAT_UNKNOWN,
          0,
        )?;
      }

      // SAFETY: The source is a `FrameConstants`, exactly the buffer's
      // size, and the context is only used under the lock held above.
      unsafe {
        context.UpdateSubresource(
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
        device.CreateRenderTargetView(
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
      device.CreateShaderResourceView(
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
    // and the immediate context is only used under the lock held above.
    unsafe {
      context.IASetInputLayout(None);
      context
        .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(vertex_shader, None);
      context.PSSetShader(pixel_shader, None);
      context.PSSetConstantBuffers(
        0,
        Some(&[
          Some(self.constants.clone()),
          Some(self.frame_constants.clone()),
          Some(self.image_constants.clone()),
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
  output: CaptureOutput,

  /// The visual presenting the swap chain; hidden until the first frame
  /// has been themed.
  sprite: SpriteVisual,

  /// Solid fill beneath `sprite`, shown only along with it (unless
  /// `fill_before_first_frame`), so a failed or not yet started pipeline
  /// never leaves an opaque slab.
  fill: SpriteVisual,
  fill_brush: CompositionColorBrush,
  fill_color: Option<Color>,

  /// Overlay size the fill is kept for; see
  /// [`ThemedCapture::release_fill_when_covered`].
  fill_until_covered: Option<(u32, u32)>,

  /// Shows the fill before the first frame; see
  /// [`ThemedCapture::set_placeholder`].
  fill_before_first_frame: bool,

  /// Size the frame pool's buffers were last created at.
  pool_size: SizeInt32,

  /// The most recent frame, kept so a theme change can re-render it
  /// without waiting for the window to repaint.
  last_frame:
    Option<(Direct3D11CaptureFrame, ID3D11Texture2D, (u32, u32))>,

  /// Rects of the pictures currently kept in their own colors.
  images: Vec<Rect>,

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
        &self.output.winrt_device,
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

    self.output.set_theme(theme);

    if let Some((_, texture, size)) = self.last_frame.clone() {
      self.present(&texture, size)?;
    }

    Ok(())
  }

  /// Keeps the pictures among the `found` images in their own colors,
  /// re-rendering the current frame if that changed which ones.
  fn update_images(&mut self, found: &[Rect]) -> crate::Result<()> {
    if self.failed.load(Ordering::Relaxed) {
      return Ok(());
    }

    // Judged on the current frame; without one, the next frame triggers
    // another query.
    let Some((_, texture, size)) = self.last_frame.clone() else {
      return Ok(());
    };

    let mut pictures = Vec::new();

    for rect in found {
      if let Some(picture) =
        self.output.picture_rect(&texture, size, rect)?
      {
        pictures.push(picture);
      }
    }

    if pictures == self.images {
      return Ok(());
    }

    tracing::debug!(
      "Color theme keeps {} of {} images in their colors: {pictures:?}.",
      pictures.len(),
      found.len(),
    );

    self.output.set_images(&pictures);
    self.images = pictures;
    self.present(&texture, size)
  }

  fn present(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<()> {
    self.output.render(texture, size)?;

    if !self.is_shown {
      self.sprite.SetIsVisible(true)?;
      self.is_shown = true;
      self.sync_fill()?;
    }

    if self
      .fill_until_covered
      .is_some_and(|until| size.0 >= until.0 && size.1 >= until.1)
    {
      self.fill_color = None;
      self.fill_until_covered = None;
      self.fill_before_first_frame = false;
      self.sync_fill()?;
    }

    Ok(())
  }

  fn sync_fill(&self) -> crate::Result<()> {
    let color = self.fill_color.filter(|_| {
      (self.is_shown || self.fill_before_first_frame)
        && !self.failed.load(Ordering::Relaxed)
    });

    if let Some(color) = color {
      self.fill_brush.SetColor(to_ui_color(color))?;
    }

    self.fill.SetIsVisible(color.is_some())?;
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

    for visual in [&self.sprite, &self.fill] {
      if let Err(err) = visual.SetIsVisible(false) {
        tracing::warn!("Failed to hide color theme visual: {err}.");
      }
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

/// Constant buffer layout shared with `cbuffer Images` in the shader.
#[repr(C)]
struct ImageConstants {
  count: u32,
  _padding: [u32; 3],

  /// `(left, top, right, bottom)` in frame pixels.
  rects: [[i32; 4]; MAX_IMAGE_RECTS],
}

// `cbuffer Images`: one packed register, then the rect `int4` array.
const _: () = assert!(
  std::mem::size_of::<ImageConstants>() == 16 * (1 + MAX_IMAGE_RECTS)
);

impl ImageConstants {
  fn new(pictures: &[Rect]) -> Self {
    let mut rects = [[0; 4]; MAX_IMAGE_RECTS];

    for (slot, rect) in rects.iter_mut().zip(pictures) {
      *slot = [rect.left, rect.top, rect.right, rect.bottom];
    }

    Self {
      // Bounded by `MAX_IMAGE_RECTS`.
      #[allow(clippy::cast_possible_truncation)]
      count: pictures.len().min(MAX_IMAGE_RECTS) as u32,
      _padding: [0; 3],
      rects,
    }
  }
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
