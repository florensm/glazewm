//! Standalone proof-of-concept for the DirectComposition + Windows.Graphics.
//! Capture rendering pipeline.
//!
//! Captures a live target window via `Windows.Graphics.Capture`, feeds each
//! frame into a DirectComposition surface, and renders it on a 3D-rotating
//! visual with perspective. This validates the full "live pixels + real 3D
//! transform" path against windows-rs 0.52 before the pipeline is ported into
//! `wm-platform` as `NativeDcompSurrogate`.
//!
//! Usage:
//!   cargo run -p dcomp-spike -- "<window title substring>"
//!   cargo run -p dcomp-spike              # captures the foreground window
//!
//! Requires Windows 11 for the borderless capture path (`SetIsBorderRequired`).
//! Close the spike window to exit.

#![cfg(target_os = "windows")]

use std::time::Instant;

use windows::{
  core::{factory, w, ComInterface, Interface, Result},
  Foundation::Numerics::Matrix4x4,
  Graphics::{
    Capture::{
      Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
    },
    DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    SizeInt32,
  },
  Win32::{
    Foundation::{BOOL, COLORREF, HMODULE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::{
      Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL},
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
        ID3D11Texture2D, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice,
        IDCompositionMatrixTransform3D, IDCompositionSurface,
        IDCompositionTarget, IDCompositionVisual,
      },
      Dwm::{DwmSetWindowAttribute, DWMWA_CLOAK},
      Dxgi::{
        Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM},
        IDXGIDevice,
      },
      Gdi::CreateSolidBrush,
    },
    System::WinRT::{
      Direct3D11::{
        CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
      },
      Graphics::Capture::IGraphicsCaptureItemInterop,
      RoInitialize, RO_INIT_MULTITHREADED,
    },
    UI::{
      HiDpi::{
        SetProcessDpiAwarenessContext,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
      },
      Input::KeyboardAndMouse::VK_SPACE,
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, EnumWindows,
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
        IsWindowVisible, PeekMessageW, PostQuitMessage, RegisterClassW,
        TranslateMessage, CW_USEDEFAULT, MSG, PM_REMOVE, WM_DESTROY,
        WM_KEYDOWN, WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
      },
    },
  },
};

/// Host window client size in physical pixels. The captured content is scaled
/// to fit inside this and rotated in 3D.
const HOST_WIDTH: u32 = 1100;
const HOST_HEIGHT: u32 = 800;

/// Window procedure: posts a quit message when the window is destroyed.
unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_DESTROY => {
      // SAFETY: No preconditions; signals the message loop to exit.
      unsafe { PostQuitMessage(0) };
      LRESULT(0)
    }
    // SAFETY: All parameters are forwarded unchanged.
    _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
  }
}

/// Mutable context threaded through [`EnumWindows`] to locate a window whose
/// title contains a target substring.
struct FindContext {
  /// Lowercased title substring to match.
  needle: String,
  /// First matching top-level window, set when found.
  found: Option<HWND>,
}

/// `EnumWindows` callback: records the first visible, titled window whose
/// (lowercased) title contains the needle, then halts enumeration.
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
  // SAFETY: `lparam` carries a `&mut FindContext` supplied by the caller below
  // and is valid for the duration of the synchronous `EnumWindows` call.
  let ctx = unsafe { &mut *(lparam.0 as *mut FindContext) };

  // SAFETY: `hwnd` is a valid window handle provided by the enumerator.
  if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
    return BOOL(1);
  }

  // SAFETY: `hwnd` is valid; the length excludes the terminating null.
  let len = unsafe { GetWindowTextLengthW(hwnd) };
  if len == 0 {
    return BOOL(1);
  }

  let mut buffer = vec![0u16; (len + 1) as usize];
  // SAFETY: `buffer` has space for `len + 1` UTF-16 code units.
  let read = unsafe { GetWindowTextW(hwnd, &mut buffer) };
  let title =
    String::from_utf16_lossy(&buffer[..read as usize]).to_lowercase();

  if title.contains(&ctx.needle) {
    ctx.found = Some(hwnd);
    return BOOL(0);
  }
  BOOL(1)
}

/// Resolves the window to capture: the first window matching the title
/// substring in `argv[1]`, or the foreground window when no argument is given.
fn find_target_window() -> Result<HWND> {
  if let Some(needle) = std::env::args().nth(1) {
    let mut ctx = FindContext {
      needle: needle.to_lowercase(),
      found: None,
    };
    // SAFETY: `enum_proc` only dereferences the `&mut ctx` pointer passed as
    // `lparam`, which outlives the synchronous enumeration.
    unsafe {
      let _ = EnumWindows(
        Some(enum_proc),
        LPARAM(std::ptr::addr_of_mut!(ctx) as isize),
      );
    }
    if let Some(hwnd) = ctx.found {
      return Ok(hwnd);
    }
    eprintln!(
      "No visible window title contains \"{needle}\"; \
       falling back to the foreground window."
    );
  }

  // SAFETY: No preconditions for `GetForegroundWindow`.
  let hwnd = unsafe { GetForegroundWindow() };
  if hwnd.0 == 0 {
    return Err(windows::core::Error::from_win32());
  }
  Ok(hwnd)
}

/// Cloaks or uncloaks `hwnd` via `DWMWA_CLOAK` — the same mechanism GlazeWM
/// uses to hide a managed window while a surrogate stands in for it.
fn set_cloak(hwnd: HWND, cloaked: bool) {
  let value: i32 = i32::from(cloaked);
  // SAFETY: `hwnd` is a valid window; `value` is a 4-byte BOOL matching the
  // size expected for `DWMWA_CLOAK`.
  unsafe {
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_CLOAK,
      std::ptr::addr_of!(value).cast(),
      std::mem::size_of::<i32>() as u32,
    );
  }
}

/// Uncloaks the target window on drop, so it is always restored when the spike
/// exits (including via the `?` error path), never left invisible.
struct CloakGuard(HWND);

impl Drop for CloakGuard {
  fn drop(&mut self) {
    set_cloak(self.0, false);
  }
}

/// Creates and shows the host window, returning its handle.
fn create_host_window() -> Result<HWND> {
  // SAFETY: A solid brush handle is valid for the lifetime of the class.
  let background = unsafe { CreateSolidBrush(COLORREF(0x0020_2020)) };

  let class = WNDCLASSW {
    lpszClassName: w!("DcompSpikeHost"),
    lpfnWndProc: Some(wnd_proc),
    hbrBackground: background,
    ..Default::default()
  };

  // SAFETY: `class` is fully initialized with a static class name and a valid
  // window procedure.
  unsafe { RegisterClassW(&class) };

  // SAFETY: The class was just registered. The four trailing handle
  // parameters are null (no parent, menu, instance, or creation data).
  let hwnd = unsafe {
    CreateWindowExW(
      Default::default(),
      w!("DcompSpikeHost"),
      w!("DComp Spike — live 3D capture"),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      HOST_WIDTH as i32,
      HOST_HEIGHT as i32,
      None,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(windows::core::Error::from_win32());
  }
  Ok(hwnd)
}

/// Creates a hardware Direct3D 11 device with BGRA support (required for
/// DirectComposition interop), returning the device and its immediate context.
fn create_d3d_device() -> Result<(ID3D11Device, ID3D11DeviceContext)> {
  let mut device: Option<ID3D11Device> = None;
  let mut context: Option<ID3D11DeviceContext> = None;
  let mut feature_level = D3D_FEATURE_LEVEL::default();

  // SAFETY: Out-parameters are valid stack slots. `BGRA_SUPPORT` is required
  // for the device to back a DirectComposition surface.
  unsafe {
    D3D11CreateDevice(
      None,
      D3D_DRIVER_TYPE_HARDWARE,
      HMODULE::default(),
      D3D11_CREATE_DEVICE_BGRA_SUPPORT,
      None,
      D3D11_SDK_VERSION,
      Some(&mut device),
      Some(&mut feature_level),
      Some(&mut context),
    )?;
  }

  let device = device.ok_or_else(windows::core::Error::from_win32)?;
  let context = context.ok_or_else(windows::core::Error::from_win32)?;
  Ok((device, context))
}

/// Live capture of a single window via `Windows.Graphics.Capture`.
///
/// Frames are produced on a free-threaded pool sharing the Direct3D 11 device
/// passed at construction, so captured textures can be copied directly into a
/// DirectComposition surface on the render thread.
struct Capture {
  /// WinRT Direct3D device backing the frame pool. Used to recreate the pool
  /// when the source window is resized, and kept alive for the session.
  device: IDirect3DDevice,
  /// Pixel format of the frame pool, reused on resize.
  format: DirectXPixelFormat,
  /// Free-threaded pool that buffers captured frames.
  frame_pool: Direct3D11CaptureFramePool,
  /// Active capture session. Kept alive to continue producing frames.
  _session: GraphicsCaptureSession,
  /// Current frame-pool size in pixels (tracks the source window's size).
  size: SizeInt32,
}

impl Capture {
  /// Starts capturing `target`, sharing `dxgi`'s Direct3D 11 device.
  ///
  /// The capture border and cursor are disabled. `SetIsBorderRequired(false)`
  /// requires Windows 11; the call is best-effort and ignored on older builds.
  fn start(dxgi: &IDXGIDevice, target: HWND) -> Result<Self> {
    // SAFETY: `dxgi` is a valid DXGI device; the interop call wraps it as a
    // WinRT Direct3D device.
    let inspectable =
      unsafe { CreateDirect3D11DeviceFromDXGIDevice(dxgi)? };
    let device: IDirect3DDevice = inspectable.cast()?;
    eprintln!("[spike]   winrt d3d device wrapped");

    let interop =
      factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    eprintln!("[spike]   capture-item interop factory ok");
    // SAFETY: `target` is a valid top-level window handle.
    let item: GraphicsCaptureItem =
      unsafe { interop.CreateForWindow(target)? };
    eprintln!("[spike]   capture item created for window");
    let size = item.Size()?;
    eprintln!("[spike]   item size = {}x{}", size.Width, size.Height);

    let format = DirectXPixelFormat::B8G8R8A8UIntNormalized;
    let frame_pool =
      Direct3D11CaptureFramePool::CreateFreeThreaded(&device, format, 2, size)?;
    eprintln!("[spike]   frame pool created");
    let session = frame_pool.CreateCaptureSession(&item)?;
    eprintln!("[spike]   capture session created");
    let _ = session.SetIsBorderRequired(false);
    let _ = session.SetIsCursorCaptureEnabled(false);
    session.StartCapture()?;
    eprintln!("[spike]   StartCapture ok");

    Ok(Self {
      device,
      format,
      frame_pool,
      _session: session,
      size,
    })
  }

  /// Pulls the latest captured frame into the composition surface, handling
  /// source-window resizes, and returns the current content size in pixels.
  ///
  /// When the captured window's size changes, the frame pool and the
  /// composition `surface` are recreated to match and `surface` is re-attached
  /// to `visual`; that tick renders the previous content (no copy) and the
  /// next frame fills the resized surface. When no new frame is available the
  /// current size is returned unchanged so the caller keeps animating.
  fn update(
    &mut self,
    dcomp: &IDCompositionDevice,
    context: &ID3D11DeviceContext,
    visual: &IDCompositionVisual,
    surface: &mut IDCompositionSurface,
  ) -> Result<(u32, u32)> {
    // `TryGetNextFrame` reports "no new frame yet" as a null WinRT result,
    // which windows-rs surfaces as `Err(Error::OK)` (HRESULT `S_OK`). Treat
    // that — and an explicit null — as simply no frame this tick, not a
    // failure. (Genuine failures carry a negative HRESULT.)
    let frame = match self.frame_pool.TryGetNextFrame() {
      Ok(frame) if !frame.as_raw().is_null() => frame,
      Ok(_) => return Ok(self.current_size()),
      Err(err) if err.code().is_ok() => return Ok(self.current_size()),
      Err(err) => return Err(err),
    };

    // Recreate the pool and surface when the source window changed size. WGC
    // keeps delivering old-sized frames until the pool is recreated.
    let content = frame.ContentSize()?;
    if (content.Width != self.size.Width || content.Height != self.size.Height)
      && content.Width > 0
      && content.Height > 0
    {
      // Release the stale frame before recreating its pool.
      drop(frame);
      self
        .frame_pool
        .Recreate(&self.device, self.format, 2, content)?;
      self.size = content;

      // SAFETY: dimensions and format are valid for a BGRA composition surface.
      let new_surface = unsafe {
        dcomp.CreateSurface(
          content.Width as u32,
          content.Height as u32,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_ALPHA_MODE_PREMULTIPLIED,
        )?
      };
      // SAFETY: `visual` is a valid visual; the new surface replaces the old
      // content, whose handle is dropped below.
      unsafe { visual.SetContent(&new_surface)? };
      *surface = new_surface;
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
      context.CopySubresourceRegion(
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
    Ok(self.current_size())
  }

  /// Returns the current content size in pixels.
  fn current_size(&self) -> (u32, u32) {
    (self.size.Width.max(1) as u32, self.size.Height.max(1) as u32)
  }
}

/// Multiplies two row-major 4x4 matrices using the row-vector convention
/// (`point * a * b` applies `a` then `b`).
fn mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
  let mut out = [[0.0_f32; 4]; 4];
  for (i, row) in out.iter_mut().enumerate() {
    for (j, cell) in row.iter_mut().enumerate() {
      *cell = (0..4).map(|k| a[i][k] * b[k][j]).sum();
    }
  }
  out
}

/// Builds the visual transform: scales the captured content (`cw` x `ch`) to
/// fit the host window, rotates it about its vertical center axis by `angle`
/// radians, applies perspective foreshortening, and centers it in the host.
fn content_transform(cw: f32, ch: f32, angle: f32) -> Matrix4x4 {
  // Uniform scale to fit the content within 80% of the host window.
  let fit = (HOST_WIDTH as f32 / cw)
    .min(HOST_HEIGHT as f32 / ch)
    * 0.8;
  let (host_cx, host_cy) = (HOST_WIDTH as f32 / 2.0, HOST_HEIGHT as f32 / 2.0);
  let (s, c) = angle.sin_cos();

  // Move the content's center to the origin.
  let to_origin = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [-cw / 2.0, -ch / 2.0, 0.0, 1.0],
  ];
  // Uniform scale-to-fit.
  let scale = [
    [fit, 0.0, 0.0, 0.0],
    [0.0, fit, 0.0, 0.0],
    [0.0, 0.0, fit, 0.0],
    [0.0, 0.0, 0.0, 1.0],
  ];
  // Rotation about the Y axis.
  let rot_y = [
    [c, 0.0, s, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [-s, 0.0, c, 0.0],
    [0.0, 0.0, 0.0, 1.0],
  ];
  // Perspective: divides x/y by (1 + z/depth) so the half rotating away from
  // the viewer foreshortens. Depth is in scaled-pixel units.
  let depth = HOST_WIDTH as f32 * 1.2;
  let perspective = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, -1.0 / depth],
    [0.0, 0.0, 0.0, 1.0],
  ];
  // Move the center to the host window's center.
  let to_host = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [host_cx, host_cy, 0.0, 1.0],
  ];

  let m = mul(
    &mul(&mul(&mul(&to_origin, &scale), &rot_y), &perspective),
    &to_host,
  );
  Matrix4x4 {
    M11: m[0][0], M12: m[0][1], M13: m[0][2], M14: m[0][3],
    M21: m[1][0], M22: m[1][1], M23: m[1][2], M24: m[1][3],
    M31: m[2][0], M32: m[2][1], M33: m[2][2], M34: m[2][3],
    M41: m[3][0], M42: m[3][1], M43: m[3][2], M44: m[3][3],
  }
}

/// Blocks until the next DWM composition frame, pacing the render loop.
fn dwm_flush() {
  // SAFETY: `DwmFlush` has no preconditions and is safe from any thread.
  unsafe {
    let _ = windows::Win32::Graphics::Dwm::DwmFlush();
  }
}

fn run() -> Result<()> {
  // SAFETY: Initializes the WinRT apartment for this thread (required for
  // Windows.Graphics.Capture). Multithreaded so the free-threaded frame pool
  // can deliver frames without a dispatcher queue.
  unsafe { RoInitialize(RO_INIT_MULTITHREADED)? };

  // SAFETY: No preconditions; improves capture/composition crispness on
  // high-DPI displays.
  unsafe {
    let _ = SetProcessDpiAwarenessContext(
      DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    );
  }

  let target = find_target_window()?;
  eprintln!("[spike] target window = {:#x}", target.0);
  let hwnd = create_host_window()?;
  eprintln!("[spike] host window created");
  let (device, context) = create_d3d_device()?;
  eprintln!("[spike] d3d11 device created");
  let dxgi: IDXGIDevice = device.cast()?;
  eprintln!("[spike] dxgi device obtained");

  // SAFETY: `dxgi` is the DXGI interface of a valid D3D11 device.
  let dcomp: IDCompositionDevice =
    unsafe { DCompositionCreateDevice(&dxgi)? };
  eprintln!("[spike] dcomp device created");

  let mut capture = Capture::start(&dxgi, target)?;
  eprintln!("[spike] capture started");
  let (cw, ch) = capture.current_size();
  eprintln!("[spike] capture size = {cw}x{ch}");

  // Composition surface sized to the captured content; updated each frame and
  // recreated by `Capture::update` when the source window is resized.
  // SAFETY: Dimensions and format are valid for a BGRA composition surface.
  let mut surface = unsafe {
    dcomp.CreateSurface(
      cw,
      ch,
      DXGI_FORMAT_B8G8R8A8_UNORM,
      DXGI_ALPHA_MODE_PREMULTIPLIED,
    )?
  };
  eprintln!("[spike] composition surface created");

  // Build and root the visual tree. `_target` and `visual` are held alive for
  // the program's duration: the target binds the tree to `hwnd`, so dropping it
  // would unbind composition from the window; `visual` is re-targeted with a
  // new surface on resize.
  //
  // SAFETY: `hwnd` is a valid top-level window; `surface` and `transform` are
  // valid content and effect for the visual.
  let (_target, visual, transform): (
    IDCompositionTarget,
    IDCompositionVisual,
    IDCompositionMatrixTransform3D,
  ) = unsafe {
    let target = dcomp.CreateTargetForHwnd(hwnd, BOOL(1))?;
    eprintln!("[spike] composition target created");
    let visual = dcomp.CreateVisual()?;
    eprintln!("[spike] visual created");
    let transform = dcomp.CreateMatrixTransform3D()?;
    eprintln!("[spike] 3d transform created");
    visual.SetContent(&surface)?;
    visual.SetEffect(&transform)?;
    target.SetRoot(&visual)?;
    eprintln!("[spike] visual tree rooted");
    (target, visual, transform)
  };

  // Always uncloak the target on exit, even on the error path.
  let _cloak_guard = CloakGuard(target);
  let mut cloaked = false;
  eprintln!(
    "[spike] focus the spike window and press SPACE to cloak/uncloak the \
     captured window — watch whether the 3D overlay keeps its content."
  );

  let start = Instant::now();
  let mut msg = MSG::default();

  loop {
    // SAFETY: `msg` is a valid stack slot; drain the queue with `PM_REMOVE`.
    unsafe {
      while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
        if msg.message == WM_QUIT {
          return Ok(());
        }
        if msg.message == WM_KEYDOWN
          && msg.wParam.0 == VK_SPACE.0 as usize
        {
          cloaked = !cloaked;
          set_cloak(target, cloaked);
          eprintln!("[spike] target cloaked = {cloaked}");
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
    }

    // Pull the latest captured frame into the composition surface, handling
    // source-window resizes; returns the current content size.
    let (cw, ch) =
      capture.update(&dcomp, &context, &visual, &mut surface)?;

    // Advance the rotation and recommit the visual tree.
    let angle = start.elapsed().as_secs_f32() * 0.8;
    let matrix = content_transform(cw as f32, ch as f32, angle);
    // SAFETY: `transform` and `dcomp` live for the program's duration.
    unsafe {
      transform.SetMatrix(&matrix)?;
      dcomp.Commit()?;
    }

    dwm_flush();
  }
}

fn main() {
  if let Err(err) = run() {
    eprintln!("dcomp-spike failed: {err}");
    std::process::exit(1);
  }
}
