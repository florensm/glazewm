//! The color theme pipeline: `Windows.Graphics.Capture` frames of the
//! themed window, run through a D3D11 pixel shader into a composition
//! swap chain shown by the overlay above that window.
//!
//! Frames never leave the GPU, except for a small sample read back to
//! measure the window's own colors (see `analysis_worker`). The frame pool
//! is free-threaded, so frames are themed and presented on a system
//! thread-pool thread as they arrive, independent of the WM's own loop;
//! WGC only delivers a frame when the window's content changed, so an
//! idle window costs nothing.

use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex, OnceLock, PoisonError, Weak,
  },
  time::Duration,
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
        D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_RENDER_TARGET,
        D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_READ,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
        D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA,
        D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
        D3D11_VIEWPORT,
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
  ui_elements::{ContentChanged, ElementWatch},
};
use crate::{
  color_levels::{estimate_levels, LevelsTracker},
  color_theme::{
    ColorTheme, ElementRect, FilterConstants, SourceLevels,
    MAX_FILTER_SLOTS, MAX_REGIONS,
  },
  Color, Rect,
};

const VERTEX_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_vs.cso"));
const PIXEL_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_ps.cso"));
const SAMPLE_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/color_theme_sample_ps.cso"));

/// Side of the square the window is sampled into to measure its colors.
/// Mirrors the shader's `ANALYSIS_SIZE`.
const ANALYSIS_SIZE: u32 = 64;

/// How often a changing window's colors are re-measured.
const ANALYSIS_INTERVAL: Duration = Duration::from_millis(500);

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
///
/// Beneath the frame sits an optional solid fill, covering whatever part
/// of the overlay the last frame doesn't reach (e.g. while it grows ahead
/// of the window during an animation).
pub(crate) struct ThemedCapture {
  source: HWND,
  session: GraphicsCaptureSession,
  frame_pool: Direct3D11CaptureFramePool,
  frame_arrived: EventRegistrationToken,
  renderer: SharedRenderer,
  failed: Arc<AtomicBool>,

  /// Accessibility queries for the theme's UI elements, while it has any.
  elements: Option<ElementWatch>,

  /// Binds the visual to the overlay's `HWND`; dropping it unbinds.
  _target: DesktopWindowTarget,
}

impl ThemedCapture {
  /// Starts capturing `source` into a visual rooted on `overlay`, which
  /// must have been created with `WS_EX_NOREDIRECTIONBITMAP`.
  ///
  /// With a `placeholder`, the overlay shows it until the first themed
  /// frame of at least `rect`'s size is presented; with `frames_held`,
  /// frames wait for [`set_frames_held`](Self::set_frames_held) to
  /// release them. `show` is called as
  /// soon as the placeholder is in place, before the slow part (GPU
  /// resources and the capture itself): a new window is on screen
  /// already, so the sooner its overlay shows, the shorter it flashes its
  /// original colors.
  pub(crate) fn start(
    source: HWND,
    overlay: HWND,
    rect: &Rect,
    theme: &ColorTheme,
    placeholder: Option<Color>,
    frames_held: bool,
    show: impl FnOnce(),
  ) -> crate::Result<Self> {
    ensure_capturable(source)?;

    let failed = Arc::new(AtomicBool::new(false));
    let size = (
      u32::try_from(rect.width().max(1))?,
      u32::try_from(rect.height().max(1))?,
    );
    let (target, sprite, fill, fill_brush) =
      create_visuals(overlay, placeholder)?;

    show();

    let output = CaptureOutput::create(theme, size)?;
    let item = attach_capture(source, &sprite, &output.swap_chain)?;

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
      fill_color: placeholder,
      fill_until_covered: placeholder.map(|_| size),
      fill_before_first_frame: placeholder.is_some(),
      frames_held,
      pool_size: item_size,
      last_frame: None,
      is_shown: false,
      failed: failed.clone(),
      theme: theme.clone(),
      analysis: Analysis::default(),
      elements: Vec::new(),
      content_changed: None,
    })));
    analysis_worker().register(&renderer);

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

    let mut capture = Self {
      source,
      session,
      frame_pool,
      frame_arrived,
      renderer,
      failed,
      elements: None,
      _target: target,
    };
    capture.sync_element_watch(theme);

    Ok(capture)
  }

  /// Switches to `theme`, re-rendering the current frame with it.
  pub(crate) fn set_theme(&mut self, theme: &ColorTheme) {
    self.with_renderer(|renderer| renderer.set_theme(theme));
    self.sync_element_watch(theme);
  }

  /// Starts, updates or stops the accessibility queries `theme` needs.
  fn sync_element_watch(&mut self, theme: &ColorTheme) {
    let kinds = theme
      .element_slots()
      .into_iter()
      .map(|(kind, _)| kind)
      .collect::<Vec<_>>();

    if kinds.is_empty() {
      if self.elements.take().is_some() {
        self.with_renderer(|renderer| {
          renderer.content_changed = None;
          renderer.set_elements(Vec::new())
        });
      }
      return;
    }

    if let Some(watch) = &self.elements {
      watch.set_kinds(kinds);
      return;
    }

    let renderer = Arc::downgrade(&self.renderer);
    let watch = ElementWatch::start(self.source, kinds, move |elements| {
      if let Some(renderer) = renderer.upgrade() {
        let mut renderer =
          renderer.lock().unwrap_or_else(PoisonError::into_inner);

        if let Err(err) = renderer.0.set_elements(elements) {
          renderer.0.fail(&err);
        }
      }
    });

    // Not touching the elements: the first query may have already
    // reported them.
    let content_changed = watch.content_changed();
    self.with_renderer(|renderer| {
      renderer.content_changed = Some(content_changed);
      Ok(())
    });
    self.elements = Some(watch);
  }

  /// Runs `f` on the renderer, stopping the pipeline if it fails.
  fn with_renderer(
    &self,
    f: impl FnOnce(&mut Renderer) -> crate::Result<()>,
  ) {
    let mut renderer =
      self.renderer.lock().unwrap_or_else(PoisonError::into_inner);

    if let Err(err) = f(&mut renderer.0) {
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

  /// Holds frames back while `held`, showing only the fill: the capture
  /// can't follow an animation, and a frame would be at the wrong place
  /// until it ends. Releasing shows the current frame right away.
  pub(crate) fn set_frames_held(&self, held: bool) {
    let mut renderer =
      self.renderer.lock().unwrap_or_else(PoisonError::into_inner);

    if renderer.0.frames_held == held {
      return;
    }

    renderer.0.frames_held = held;

    if let Err(err) = renderer.0.release_held_frame() {
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

/// The visual tree an overlay shows: a sprite for the themed frames
/// (hidden until the first one) above a solid fill, showing `placeholder`
/// right away if given.
fn create_visuals(
  overlay: HWND,
  placeholder: Option<Color>,
) -> crate::Result<(
  DesktopWindowTarget,
  SpriteVisual,
  SpriteVisual,
  CompositionColorBrush,
)> {
  let overlay_raw = overlay.0;

  // Composition objects are agile, but are created on the composition
  // thread since it is guaranteed to have WinRT initialized.
  with_composition_thread(move |compositor, _| {
    // SAFETY: `overlay` is a live top-level window.
    let target = unsafe {
      compositor
        .cast::<ICompositorDesktopInterop>()?
        .CreateDesktopWindowTarget(HWND(overlay_raw), false)?
    };

    // Its brush is set once the swap chain exists.
    let sprite = compositor.CreateSpriteVisual()?;
    sprite.SetRelativeSizeAdjustment(FILL_PARENT)?;
    sprite.SetIsVisible(false)?;

    let fill_brush = compositor.CreateColorBrush()?;
    let fill = compositor.CreateSpriteVisual()?;
    fill.SetBrush(&fill_brush)?;
    fill.SetRelativeSizeAdjustment(FILL_PARENT)?;
    fill.SetIsVisible(false)?;

    if let Some(color) = placeholder {
      fill_brush.SetColor(to_ui_color(color))?;
      fill.SetIsVisible(true)?;
    }

    let root = compositor.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(FILL_PARENT)?;
    root.Children()?.InsertAtBottom(&fill)?;
    root.Children()?.InsertAtTop(&sprite)?;
    target.SetRoot(&root)?;

    Ok((target, sprite, fill, fill_brush))
  })
}

/// Points `sprite` at `swap_chain`, and creates the capture item for
/// `source`.
fn attach_capture(
  source: HWND,
  sprite: &SpriteVisual,
  swap_chain: &IDXGISwapChain1,
) -> crate::Result<GraphicsCaptureItem> {
  let swap_chain = AssertSend(swap_chain.clone());
  let sprite = sprite.clone();
  let source_raw = source.0;

  // WGC objects are agile too; created on the composition thread for the
  // same reason.
  with_composition_thread(move |compositor, _| {
    // SAFETY: `swap_chain` is a composition swap chain that nothing
    // presents to until the capture starts.
    let surface = unsafe {
      compositor
        .cast::<ICompositorInterop>()?
        .CreateCompositionSurfaceForSwapChain(&swap_chain.0)?
    };

    // Drawn 1:1 from the top-left corner: the swap chain always matches
    // the captured window's size, which the overlay does too.
    let brush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
    brush.SetStretch(CompositionStretch::None)?;
    brush.SetHorizontalAlignmentRatio(0.0)?;
    brush.SetVerticalAlignmentRatio(0.0)?;
    sprite.SetBrush(&brush)?;

    let interop =
      factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    // SAFETY: `source` is a window handle; a stale one just fails.
    Ok(unsafe { interop.CreateForWindow(HWND(source_raw))? })
  })
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

  /// Point-samples a frame for measuring its colors.
  sample_shader: ID3D11PixelShader,
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
    let mut sample_shader = None;
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
      device.CreatePixelShader(
        SAMPLE_SHADER,
        None,
        Some(&raw mut sample_shader),
      )?;
    }

    Ok(Self {
      device,
      context,
      winrt_device,
      vertex_shader: created(vertex_shader)?,
      pixel_shader: created(pixel_shader)?,
      sample_shader: created(sample_shader)?,
    })
  }
}

/// Constant buffer layout shared with `cbuffer Filters` in the shader.
type FilterSlots = [FilterConstants; MAX_FILTER_SLOTS];

/// Constant buffer layout shared with `cbuffer Frame` in the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
struct FrameConstants {
  size: [u32; 2],
  passthrough: u32,
  region_count: u32,

  /// Source paper and ink lightness, then padding.
  levels: [f32; 4],
}

/// Constant buffer layout shared with `cbuffer Regions` in the shader.
#[repr(C)]
struct RegionConstants {
  rects: [[i32; 4]; MAX_REGIONS],

  /// Filter slot of each region in `x`.
  slots: [[u32; 4]; MAX_REGIONS],
}

/// Render target and CPU-readable copy for sampling a frame.
struct AnalysisTarget {
  texture: ID3D11Texture2D,
  view: ID3D11RenderTargetView,
  staging: ID3D11Texture2D,
}

/// One capture's rendering target on the shared [`Gpu`].
struct CaptureOutput {
  gpu: SharedGpu,

  /// The shared device, as the frame pool needs it; cloned so pool calls
  /// don't need the lock.
  winrt_device: IDirect3DDevice,

  /// [`FilterSlots`] of the current theme.
  filters: ID3D11Buffer,

  frame: FrameConstants,
  frame_constants: ID3D11Buffer,

  /// [`RegionConstants`] of the current UI elements.
  regions: ID3D11Buffer,

  swap_chain: IDXGISwapChain1,
  swap_chain_size: (u32, u32),

  /// View of the swap chain's current back buffer. With the flip model,
  /// buffer 0 always refers to it, so this stays valid across presents
  /// until the buffers are resized.
  render_target: Option<ID3D11RenderTargetView>,

  /// Created on first use, for themes that measure the window's colors.
  analysis: Option<AnalysisTarget>,
}

impl CaptureOutput {
  fn create(theme: &ColorTheme, size: (u32, u32)) -> crate::Result<Self> {
    let gpu = shared_gpu()?;
    let guard = gpu.lock().unwrap_or_else(PoisonError::into_inner);
    let device = &guard.0.device;

    let frame = FrameConstants {
      size: [size.0, size.1],
      passthrough: 0,
      region_count: 0,
      levels: [1.0, 0.0, 0.0, 0.0],
    };

    let filters: FilterSlots = theme.filter_constants();
    let filters = create_constant_buffer(device, &filters)?;
    let frame_constants = create_constant_buffer(device, &frame)?;
    let regions = create_constant_buffer(
      device,
      &RegionConstants {
        rects: [[0; 4]; MAX_REGIONS],
        slots: [[0; 4]; MAX_REGIONS],
      },
    )?;

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
      filters,
      frame,
      frame_constants,
      regions,
      swap_chain,
      swap_chain_size: size,
      render_target: None,
      analysis: None,
    })
  }

  /// Uploads `theme`'s filters for the next render.
  fn set_theme(&self, theme: &ColorTheme) {
    let gpu = self.gpu.lock().unwrap_or_else(PoisonError::into_inner);
    let filters: FilterSlots = theme.filter_constants();

    // SAFETY: The source is a `FilterSlots`, exactly the buffer's size,
    // and the context is only used under the lock held above.
    unsafe {
      gpu.0.context.UpdateSubresource(
        &self.filters,
        0,
        None,
        std::ptr::from_ref(&filters).cast(),
        0,
        0,
      );
    }
  }

  /// Uploads UI element regions, as `(ltrb, slot)`, for the next render.
  fn set_regions(&mut self, regions: &[([i32; 4], u32)]) {
    let mut constants = RegionConstants {
      rects: [[0; 4]; MAX_REGIONS],
      slots: [[0; 4]; MAX_REGIONS],
    };

    for (index, (rect, slot)) in
      regions.iter().take(MAX_REGIONS).enumerate()
    {
      constants.rects[index] = *rect;
      constants.slots[index][0] = *slot;
    }

    {
      let gpu = self.gpu.lock().unwrap_or_else(PoisonError::into_inner);

      // SAFETY: The source is a `RegionConstants`, exactly the buffer's
      // size, and the context is only used under the lock held above.
      unsafe {
        gpu.0.context.UpdateSubresource(
          &self.regions,
          0,
          None,
          std::ptr::from_ref(&constants).cast(),
          0,
          0,
        );
      }
    }

    // Bounded by `MAX_REGIONS`.
    #[allow(clippy::cast_possible_truncation)]
    let count = regions.len().min(MAX_REGIONS) as u32;
    self.update_frame(FrameConstants {
      region_count: count,
      ..self.frame
    });
  }

  /// Sets the source levels and passthrough for the next render.
  fn set_levels(&mut self, levels: SourceLevels, passthrough: bool) {
    self.update_frame(FrameConstants {
      passthrough: u32::from(passthrough),
      levels: [levels.paper, levels.ink, 0.0, 0.0],
      ..self.frame
    });
  }

  /// Uploads `frame` if it differs from the current constants.
  fn update_frame(&mut self, frame: FrameConstants) {
    if frame == self.frame {
      return;
    }

    self.frame = frame;
    let gpu = self.gpu.lock().unwrap_or_else(PoisonError::into_inner);

    // SAFETY: The source is a `FrameConstants`, exactly the buffer's
    // size, and the context is only used under the lock held above.
    unsafe {
      gpu.0.context.UpdateSubresource(
        &self.frame_constants,
        0,
        None,
        std::ptr::from_ref(&self.frame).cast(),
        0,
        0,
      );
    }
  }

  /// Themes the top-left `size` of `texture` into the swap chain and
  /// presents it.
  fn render(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<()> {
    self.update_frame(FrameConstants {
      size: [size.0, size.1],
      ..self.frame
    });

    let gpu = self.gpu.clone();
    let guard = gpu.lock().unwrap_or_else(PoisonError::into_inner);
    let Gpu {
      device,
      context,
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

    self.draw(
      &guard.0,
      texture,
      pixel_shader,
      self.render_target.clone(),
      size,
    )?;

    // SAFETY: Presents the back buffer drawn above.
    unsafe { self.swap_chain.Present(1, 0).ok()? };

    Ok(())
  }

  /// Draws a full-target pass of `pixel_shader` over `texture`.
  fn draw(
    &self,
    gpu: &Gpu,
    texture: &ID3D11Texture2D,
    pixel_shader: &ID3D11PixelShader,
    target: Option<ID3D11RenderTargetView>,
    size: (u32, u32),
  ) -> crate::Result<()> {
    let mut source = None;
    // SAFETY: `texture` is a live frame-pool texture and `source`
    // outlives the call.
    unsafe {
      gpu.device.CreateShaderResourceView(
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

    let context = &gpu.context;

    // SAFETY: Every bound object is alive for the duration of the draw,
    // and the immediate context is only used under the GPU lock, which
    // the caller holds.
    unsafe {
      context.IASetInputLayout(None);
      context
        .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&gpu.vertex_shader, None);
      context.PSSetShader(pixel_shader, None);
      context.PSSetConstantBuffers(
        0,
        Some(&[
          Some(self.filters.clone()),
          Some(self.frame_constants.clone()),
          Some(self.regions.clone()),
        ]),
      );
      context.PSSetShaderResources(0, Some(&[source]));
      context.OMSetRenderTargets(Some(&[target]), None);
      context.RSSetViewports(Some(&[viewport]));
      context.Draw(3, 0);

      // Unbound so the frame pool can hand the texture back out.
      context.PSSetShaderResources(0, Some(&[None]));
    }

    Ok(())
  }

  /// Point-samples the top-left `size` of `texture` into an
  /// [`ANALYSIS_SIZE`] square and reads it back, as straight-alpha sRGB.
  /// Mostly transparent samples (rounded corners) are left out.
  ///
  /// Blocks until the GPU has drawn it; only called off the frame path.
  fn sample(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<Vec<[f32; 3]>> {
    self.update_frame(FrameConstants {
      size: [size.0, size.1],
      ..self.frame
    });

    let gpu = self.gpu.clone();
    let guard = gpu.lock().unwrap_or_else(PoisonError::into_inner);

    if self.analysis.is_none() {
      self.analysis = Some(create_analysis_target(&guard.0.device)?);
    }

    let Some(analysis) = &self.analysis else {
      return Ok(Vec::new());
    };

    self.draw(
      &guard.0,
      texture,
      &guard.0.sample_shader,
      Some(analysis.view.clone()),
      (ANALYSIS_SIZE, ANALYSIS_SIZE),
    )?;

    let context = &guard.0.context;
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();

    // SAFETY: Both textures are alive and the same size and format, and
    // `mapped` outlives the call.
    unsafe {
      context.CopyResource(&analysis.staging, &analysis.texture);
      context.Map(
        &analysis.staging,
        0,
        D3D11_MAP_READ,
        0,
        Some(&raw mut mapped),
      )?;
    }

    let row_pitch = mapped.RowPitch as usize;
    let texels_per_row = ANALYSIS_SIZE as usize;

    // SAFETY: A mapped `ANALYSIS_SIZE` square of 4-byte texels, with rows
    // `RowPitch` apart, stays valid until `Unmap` below.
    let bytes = unsafe {
      std::slice::from_raw_parts(
        mapped.pData.cast::<u8>(),
        row_pitch * (texels_per_row - 1) + texels_per_row * 4,
      )
    };

    let samples = (0..texels_per_row)
      .flat_map(|y| {
        let row =
          &bytes[y * row_pitch..y * row_pitch + texels_per_row * 4];
        row.as_chunks::<4>().0.iter()
      })
      // BGRA.
      .filter(|texel| texel[3] >= 128)
      .map(|texel| {
        [texel[2], texel[1], texel[0]].map(|c| f32::from(c) / 255.0)
      })
      .collect();

    // SAFETY: Mapped above.
    unsafe { context.Unmap(&analysis.staging, 0) };

    Ok(samples)
  }
}

fn create_analysis_target(
  device: &ID3D11Device,
) -> crate::Result<AnalysisTarget> {
  let mut desc = D3D11_TEXTURE2D_DESC {
    Width: ANALYSIS_SIZE,
    Height: ANALYSIS_SIZE,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    #[allow(clippy::cast_sign_loss)]
    BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
    ..Default::default()
  };

  let mut texture = None;
  let mut view = None;
  let mut staging = None;

  // SAFETY: `desc` describes a valid texture and every pointer outlives
  // its call.
  unsafe {
    device.CreateTexture2D(
      &raw const desc,
      None,
      Some(&raw mut texture),
    )?;
    let texture = created(texture.clone())?;
    device.CreateRenderTargetView(&texture, None, Some(&raw mut view))?;

    desc.Usage = D3D11_USAGE_STAGING;
    desc.BindFlags = 0;
    #[allow(clippy::cast_sign_loss)]
    {
      desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
    }
    device.CreateTexture2D(
      &raw const desc,
      None,
      Some(&raw mut staging),
    )?;
  }

  Ok(AnalysisTarget {
    texture: created(texture)?,
    view: created(view)?,
    staging: created(staging)?,
  })
}

type SharedRenderer = Arc<Mutex<AssertSend<Renderer>>>;

/// Per-capture state, shared between the frame-arrived callback, the WM
/// thread, and the analysis and accessibility workers.
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

  /// Shows the fill before the first frame (a placeholder; see
  /// [`ThemedCapture::start`]).
  fill_before_first_frame: bool,

  /// Frames are rendered but not shown, and the fill stays; see
  /// [`ThemedCapture::set_frames_held`].
  frames_held: bool,

  /// Size the frame pool's buffers were last created at.
  pool_size: SizeInt32,

  /// The most recent frame, kept so a theme change can re-render it
  /// without waiting for the window to repaint.
  last_frame:
    Option<(Direct3D11CaptureFrame, ID3D11Texture2D, (u32, u32))>,

  is_shown: bool,
  failed: Arc<AtomicBool>,

  theme: ColorTheme,
  /// Measurements of the window's own colors.
  analysis: Analysis,

  /// The UI elements last reported for the window.
  elements: Vec<ElementRect>,

  /// Tells the accessibility worker the window's content changed.
  content_changed: Option<ContentChanged>,
}

/// What is known of a window's own colors.
#[derive(Default)]
struct Analysis {
  levels: LevelsTracker,

  /// Whether the colors were measured at least once.
  is_measured: bool,

  /// Whether a frame arrived since the colors were last measured.
  is_due: bool,
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
    self.analysis.is_due = true;

    if let Some(content_changed) = &self.content_changed {
      content_changed.notify();
    }

    // Measured before the first frame is shown, so an already-dark window
    // doesn't flash inverted until the analysis worker gets to it.
    if !self.analysis.is_measured && self.theme.needs_analysis() {
      self.analyze(&texture, (width, height))?;
    }

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

    self.theme = theme.clone();
    self.output.set_theme(theme);
    self.apply_levels();
    self.upload_regions();
    self.analysis.is_due = true;
    self.rerender()
  }

  fn set_elements(
    &mut self,
    elements: Vec<ElementRect>,
  ) -> crate::Result<()> {
    if self.failed.load(Ordering::Relaxed) || elements == self.elements {
      return Ok(());
    }

    self.elements = elements;
    self.upload_regions();
    self.rerender()
  }

  /// Re-measures the window's colors if it changed since the last time,
  /// re-rendering if they moved.
  fn analyze_if_due(&mut self) -> crate::Result<()> {
    if self.failed.load(Ordering::Relaxed)
      || !self.analysis.is_due
      || !self.theme.needs_analysis()
    {
      return Ok(());
    }

    let Some((_, texture, size)) = self.last_frame.clone() else {
      return Ok(());
    };

    if self.analyze(&texture, size)? {
      self.present(&texture, size)?;
    }

    Ok(())
  }

  /// Measures the window's colors from `texture`. Returns whether the
  /// levels or passthrough changed.
  fn analyze(
    &mut self,
    texture: &ID3D11Texture2D,
    size: (u32, u32),
  ) -> crate::Result<bool> {
    self.analysis.is_due = false;
    self.analysis.is_measured = true;

    let samples = self.output.sample(texture, size)?;
    let changed = estimate_levels(&samples)
      .is_some_and(|estimate| self.analysis.levels.update(estimate));

    if changed {
      self.apply_levels();
    }

    Ok(changed)
  }

  /// Uploads the measured levels and passthrough, as far as the theme
  /// uses them.
  fn apply_levels(&mut self) {
    let levels = if self.theme.detect_colors() {
      self.analysis.levels.levels()
    } else {
      SourceLevels::default()
    };

    self.output.set_levels(
      levels,
      self.theme.skip_if_dark() && self.analysis.levels.is_dark(),
    );
  }

  fn upload_regions(&mut self) {
    let size = self.output.swap_chain_size;
    let regions = self.theme.element_regions(&self.elements, size);
    self.output.set_regions(&regions);
  }

  /// Re-renders the last frame, if any, with the current settings.
  fn rerender(&mut self) -> crate::Result<()> {
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
    self.output.render(texture, size)?;

    if self.frames_held {
      return Ok(());
    }

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

  /// Shows the current frame once frames are no longer held, or keeps
  /// the fill up while they are.
  fn release_held_frame(&mut self) -> crate::Result<()> {
    if self.frames_held {
      return self.sync_fill();
    }

    match self.last_frame.clone() {
      Some((_, texture, size)) => self.present(&texture, size),
      None => self.sync_fill(),
    }
  }

  fn sync_fill(&self) -> crate::Result<()> {
    let color = self.fill_color.filter(|_| {
      (self.is_shown || self.fill_before_first_frame || self.frames_held)
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

/// Re-measures the colors of every themed window that changed, every
/// [`ANALYSIS_INTERVAL`], on a thread of its own: reading pixels back
/// waits for the GPU, which the frame path must never do.
struct AnalysisWorker {
  renderers: Mutex<Vec<Weak<Mutex<AssertSend<Renderer>>>>>,
  registered: Condvar,
}

fn analysis_worker() -> &'static AnalysisWorker {
  static WORKER: OnceLock<AnalysisWorker> = OnceLock::new();

  WORKER.get_or_init(|| {
    let spawned = std::thread::Builder::new()
      .name("color-theme-analysis".to_string())
      .spawn(|| analysis_worker().run());

    if let Err(err) = spawned {
      tracing::warn!("Failed to start color theme analysis: {err}.");
    }

    AnalysisWorker {
      renderers: Mutex::new(Vec::new()),
      registered: Condvar::new(),
    }
  })
}

impl AnalysisWorker {
  fn register(&self, renderer: &SharedRenderer) {
    self
      .renderers
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Arc::downgrade(renderer));
    self.registered.notify_one();
  }

  fn run(&self) {
    loop {
      let renderers = {
        let mut renderers = self
          .renderers
          .lock()
          .unwrap_or_else(PoisonError::into_inner);
        renderers.retain(|renderer| renderer.strong_count() > 0);

        // Sleep until a capture exists rather than ticking for nothing.
        while renderers.is_empty() {
          renderers = self
            .registered
            .wait(renderers)
            .unwrap_or_else(PoisonError::into_inner);
        }

        renderers.clone()
      };

      for renderer in renderers.iter().filter_map(Weak::upgrade) {
        let mut renderer =
          renderer.lock().unwrap_or_else(PoisonError::into_inner);

        if let Err(err) = renderer.0.analyze_if_due() {
          renderer.0.fail(&err);
        }
      }

      std::thread::sleep(ANALYSIS_INTERVAL);
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
