//! Standalone proof-of-concept for a `Windows.UI.Composition` based acrylic
//! blur-behind pipeline with a continuously adjustable blur radius and a
//! continuously adjustable corner radius.
//!
//! Validates the full chain end to end, outside of any `GlazeWM` state:
//!   1. A bare `WS_POPUP` + `WS_EX_NOREDIRECTIONBITMAP` host window.
//!   2. `ACCENT_ENABLE_HOSTBACKDROP` (+ `DWMWA_USE_HOSTBACKDROPBRUSH` on
//!      Win11) so a `CompositionBackdropBrush` samples live desktop pixels
//!      from behind the window rather than rendering black/opaque.
//!   3. A hand-implemented `IGraphicsEffectD2D1Interop` describing the
//!      built-in D2D1 Gaussian blur effect (`CLSID_D2D1GaussianBlur`), fed
//!      through `Compositor::CreateEffectFactory`.
//!   4. A `CompositionRoundedRectangleGeometry` + `CompositionGeometricClip`
//!      applied to the visual for a continuous corner radius.
//!
//! Usage:
//!   cargo run -p composition-blur-spike
//!
//! Controls (host window must have focus):
//!   Up/Down    -- increase/decrease blur amount (D2D1 std deviation).
//!   Left/Right -- decrease/increase corner radius.
//!   Esc        -- exit.
//!
//! Close the spike window (or press Esc) to exit.
//!
//! # Findings (Phase 0 spike results)
//!
//! - The full chain works: host backdrop brush, hand-implemented
//!   `IGraphicsEffectD2D1Interop` Gaussian-blur effect graph, and rounded
//!   geometric clip all build and render without error.
//! - `Compositor::CreateEffectFactory` validates the effect description
//!   against D2D1's *real* registered property schema for the built-in
//!   Gaussian-blur effect (3 properties: `StandardDeviation`,
//!   `Optimization`, `BorderMode`) and fails with `E_INVALIDARG` if any are
//!   missing, even though only `StandardDeviation` needs to be
//!   runtime-adjustable (see `GetPropertyCount`/`GetProperty`).
//! - Confirmed empirically (not just per docs): the async D2D1 shader-graph
//!   compile (`CompositionEffectFactoryLoadStatus::Pending` ->
//!   `::Success`) only progresses while the thread that created the
//!   `Compositor` is actively pumping messages/its dispatcher queue. With
//!   no pump at all, `LoadStatus` was observed stuck at `Pending` for a
//!   full 2 seconds; with the `PeekMessage` loop below running, it settles
//!   to `Success` within ~1 pump interval. This confirms `GlazeWM`'s
//!   tokio-driven main loop (no Win32 message pump) cannot host the
//!   `Compositor` directly -- Phase 1 needs a dedicated, continuously
//!   pumped OS thread for it (see the plan's Threading note).

#![cfg(target_os = "windows")]

use std::cell::RefCell;

use windows::{
  core::{implement, w, ComInterface, Result, GUID, HSTRING, PCWSTR},
  Foundation::Numerics::Vector2,
  Foundation::PropertyValue,
  Graphics::Effects::{
    IGraphicsEffect, IGraphicsEffect_Impl, IGraphicsEffectSource,
    IGraphicsEffectSource_Impl,
  },
  UI::Composition::{
    CompositionEffectBrush, CompositionEffectFactory,
    CompositionEffectSourceParameter, CompositionRoundedRectangleGeometry,
    Compositor, Desktop::DesktopWindowTarget,
  },
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_HOSTBACKDROPBRUSH},
    System::{
      Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
      LibraryLoader::{GetModuleHandleW, GetProcAddress},
      Threading::GetCurrentThreadId,
      WinRT::{
        Composition::ICompositorDesktopInterop,
        Graphics::Direct2D::{
          IGraphicsEffectD2D1Interop, IGraphicsEffectD2D1Interop_Impl,
          GRAPHICS_EFFECT_PROPERTY_MAPPING,
          GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT,
        },
        CreateDispatcherQueueController, DispatcherQueueOptions,
        DQTAT_COM_ASTA, DQTYPE_THREAD_CURRENT,
      },
    },
    UI::{
      HiDpi::{
        SetProcessDpiAwarenessContext,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
      },
      Input::KeyboardAndMouse::{VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RIGHT, VK_UP},
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, PeekMessageW,
        PostQuitMessage, RegisterClassW, SetWindowPos, ShowWindow,
        TranslateMessage, CW_USEDEFAULT, HWND_TOP, MSG, PM_REMOVE, SW_SHOW,
        SWP_NOACTIVATE, SWP_NOZORDER, WM_DESTROY, WM_KEYDOWN, WM_QUIT,
        WNDCLASSW, WS_EX_NOREDIRECTIONBITMAP, WS_POPUP, WS_VISIBLE,
      },
    },
  },
};

/// Host window client size in physical pixels.
const HOST_WIDTH: i32 = 900;
const HOST_HEIGHT: i32 = 600;

/// `HOST_WIDTH`/`HOST_HEIGHT` as `f32`, for composition APIs that size
/// visuals in DIPs rather than integer pixels. The constants are small and
/// fixed, so the cast is always exact.
#[allow(clippy::cast_precision_loss)]
const HOST_SIZE_F32: (f32, f32) = (HOST_WIDTH as f32, HOST_HEIGHT as f32);

/// `CLSID_D2D1GaussianBlur`, the built-in D2D1 Gaussian-blur effect.
const CLSID_D2D1_GAUSSIAN_BLUR: GUID =
  GUID::from_u128(0x1feb_6d69_2fe6_4ac9_8c58_1d7f_93e7_a6a5);

/// D2D1 Gaussian-blur effect property index: standard deviation (blur
/// radius, in DIPs). See `D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION`.
const PROP_STANDARD_DEVIATION: u32 = 0;

/// D2D1 Gaussian-blur effect property index: optimization mode (speed vs.
/// quality). See `D2D1_GAUSSIANBLUR_PROP_OPTIMIZATION`.
const PROP_OPTIMIZATION: u32 = 1;

/// D2D1 Gaussian-blur effect property index: edge-sampling mode. See
/// `D2D1_GAUSSIANBLUR_PROP_BORDER_MODE`.
const PROP_BORDER_MODE: u32 = 2;

/// `D2D1_GAUSSIANBLUR_OPTIMIZATION_BALANCED`.
const D2D1_GAUSSIANBLUR_OPTIMIZATION_BALANCED: u32 = 1;

/// `D2D1_BORDER_MODE_SOFT`.
const D2D1_BORDER_MODE_SOFT: u32 = 0;

/// Accent state: acrylic blur-behind. Unused here, kept for reference
/// against `wm-platform`'s `swca.rs`.
#[allow(dead_code)]
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

/// Accent state: host backdrop -- samples live desktop content from behind
/// the window (the mechanism this spike validates), rather than blurring
/// `GlazeWM`'s own solid color like `ACCENT_ENABLE_ACRYLICBLURBEHIND` does.
const ACCENT_ENABLE_HOSTBACKDROP: u32 = 5;

/// `WCA_ACCENT_POLICY` attribute index for
/// `SetWindowCompositionAttribute`.
const WCA_ACCENT_POLICY: u32 = 19;

#[repr(C)]
struct AccentPolicy {
  accent_state: u32,
  accent_flags: u32,
  gradient_color: u32,
  animation_id: u32,
}

#[repr(C)]
struct WindowCompositionAttribData {
  attrib: u32,
  pv_data: *mut std::ffi::c_void,
  cb_data: usize,
}

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

/// Applies `ACCENT_ENABLE_HOSTBACKDROP` to `hwnd` via the undocumented
/// `SetWindowCompositionAttribute` API. Best-effort: logs and returns on
/// failure rather than treating it as fatal, since the Win11 path
/// (`DWMWA_USE_HOSTBACKDROPBRUSH`) is applied separately and may suffice on
/// its own.
fn apply_hostbackdrop_accent(hwnd: HWND) {
  // SAFETY: user32.dll is always loaded in every Win32 process.
  let Some(module) = (unsafe { GetModuleHandleW(w!("user32.dll")).ok() })
  else {
    eprintln!("[spike] failed to get user32.dll module handle");
    return;
  };

  // SAFETY: `module` is a valid handle; the ASCII string is
  // null-terminated via the `s!`-equivalent `w!` literal cast below.
  let Some(proc) = (unsafe {
    GetProcAddress(module, windows::core::s!("SetWindowCompositionAttribute"))
  }) else {
    eprintln!("[spike] SetWindowCompositionAttribute export not found (pre-Win10 1607)");
    return;
  };

  // SAFETY: `proc` is a valid export with the expected calling convention.
  let set_wca: SetWindowCompositionAttributeFn = unsafe {
    std::mem::transmute::<
      unsafe extern "system" fn() -> isize,
      SetWindowCompositionAttributeFn,
    >(proc)
  };

  let mut policy = AccentPolicy {
    accent_state: ACCENT_ENABLE_HOSTBACKDROP,
    accent_flags: 0,
    gradient_color: 0,
    animation_id: 0,
  };
  let mut data = WindowCompositionAttribData {
    attrib: WCA_ACCENT_POLICY,
    pv_data: std::ptr::addr_of_mut!(policy).cast(),
    cb_data: std::mem::size_of::<AccentPolicy>(),
  };

  // SAFETY: `hwnd` is a valid window handle; `data`/`policy` are
  // stack-allocated and live for the duration of this call.
  let ok = unsafe { set_wca(hwnd, std::ptr::addr_of_mut!(data)) != 0 };
  eprintln!("[spike] ACCENT_ENABLE_HOSTBACKDROP applied = {ok}");
}

/// Sets `DWMWA_USE_HOSTBACKDROPBRUSH` (Win11's documented equivalent of the
/// undocumented SWCA path above). Best-effort.
fn apply_hostbackdrop_dwm_attribute(hwnd: HWND) {
  let value: windows::Win32::Foundation::BOOL = true.into();
  // `BOOL` is a 4-byte struct; the cast is always exact.
  #[allow(clippy::cast_possible_truncation)]
  let size = std::mem::size_of::<windows::Win32::Foundation::BOOL>() as u32;
  // SAFETY: `hwnd` is valid; `value` is a 4-byte BOOL matching `size`, the
  // size expected for `DWMWA_USE_HOSTBACKDROPBRUSH`.
  let res = unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_USE_HOSTBACKDROPBRUSH,
      std::ptr::addr_of!(value).cast(),
      size,
    )
  };
  eprintln!("[spike] DWMWA_USE_HOSTBACKDROPBRUSH result = {res:?}");
}

/// Hand-implemented D2D1 Gaussian-blur effect description.
///
/// `Compositor::CreateEffectFactory` takes an `IGraphicsEffect` describing
/// an effect graph. `Win2D`'s `GaussianBlurEffect` convenience type requires
/// the `Win2D` winmd, which `windows-rs`'s metadata-driven binding generator
/// cannot consume -- so this hand-implements the `IGraphicsEffectD2D1Interop`
/// COM shape directly against the D2D1 built-in Gaussian-blur effect
/// (`CLSID_D2D1GaussianBlur`), the same approach Microsoft's own
/// `Windows.UI.Composition-Win32-Samples` uses in C++.
#[implement(IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectD2D1Interop)]
struct GaussianBlurEffect {
  source: IGraphicsEffectSource,
  /// Initial blur std-deviation baked into the effect graph at factory
  /// creation. Runtime adjustment after that happens via the resulting
  /// brush's `Properties().InsertScalar("Blur.BlurAmount", ..)`, not this
  /// field -- see `GetNamedPropertyMapping`.
  initial_amount: f32,
  name: RefCell<HSTRING>,
}

impl GaussianBlurEffect {
  fn new(source: IGraphicsEffectSource, initial_amount: f32) -> Self {
    Self {
      source,
      initial_amount,
      name: RefCell::new(HSTRING::from("Blur")),
    }
  }
}

impl IGraphicsEffectSource_Impl for GaussianBlurEffect {}

impl IGraphicsEffect_Impl for GaussianBlurEffect {
  fn Name(&self) -> Result<HSTRING> {
    Ok(self.name.borrow().clone())
  }

  fn SetName(&self, name: &HSTRING) -> Result<()> {
    *self.name.borrow_mut() = name.clone();
    Ok(())
  }
}

impl IGraphicsEffectD2D1Interop_Impl for GaussianBlurEffect {
  fn GetEffectId(&self) -> Result<GUID> {
    Ok(CLSID_D2D1_GAUSSIAN_BLUR)
  }

  fn GetNamedPropertyMapping(
    &self,
    name: &PCWSTR,
    index: *mut u32,
    mapping: *mut GRAPHICS_EFFECT_PROPERTY_MAPPING,
  ) -> Result<()> {
    // SAFETY: `name` is a valid, null-terminated wide string for the
    // duration of this call, per the WinRT effect-description contract.
    let name = unsafe { name.to_string() }.unwrap_or_default();

    // Confirmed by running the real integration in wm-platform: the
    // composition engine calls this with the dotted `"Blur.BlurAmount"`
    // path, not the bare property name -- match on the segment after the
    // last `.` so this works either way.
    let property_name = name.rsplit('.').next().unwrap_or(&name);

    if property_name == "BlurAmount" {
      // SAFETY: `index`/`mapping` are valid out-parameters supplied by the
      // composition engine for this call.
      unsafe {
        *index = PROP_STANDARD_DEVIATION;
        *mapping = GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT;
      }
      Ok(())
    } else {
      Err(windows::core::Error::from(
        windows::Win32::Foundation::E_INVALIDARG,
      ))
    }
  }

  fn GetPropertyCount(&self) -> Result<u32> {
    // D2D1's built-in Gaussian-blur effect registers exactly three
    // properties; `CreateEffectFactory` validates the description against
    // that schema and fails with `E_INVALIDARG` if any are missing, even
    // though only `StandardDeviation` is runtime-adjustable here (see
    // `GetNamedPropertyMapping`).
    Ok(3)
  }

  fn GetProperty(
    &self,
    index: u32,
  ) -> Result<windows::Foundation::IPropertyValue> {
    match index {
      PROP_STANDARD_DEVIATION => {
        PropertyValue::CreateSingle(self.initial_amount)?.cast()
      }
      PROP_OPTIMIZATION => {
        PropertyValue::CreateUInt32(D2D1_GAUSSIANBLUR_OPTIMIZATION_BALANCED)?
          .cast()
      }
      PROP_BORDER_MODE => {
        PropertyValue::CreateUInt32(D2D1_BORDER_MODE_SOFT)?.cast()
      }
      _ => Err(windows::core::Error::from(
        windows::Win32::Foundation::E_INVALIDARG,
      )),
    }
  }

  fn GetSource(&self, index: u32) -> Result<IGraphicsEffectSource> {
    if index == 0 {
      Ok(self.source.clone())
    } else {
      Err(windows::core::Error::from(
        windows::Win32::Foundation::E_INVALIDARG,
      ))
    }
  }

  fn GetSourceCount(&self) -> Result<u32> {
    Ok(1)
  }
}

/// Window procedure: posts a quit message on destroy, otherwise defers to
/// the default handler. Keyboard handling happens in the message loop so it
/// has access to the composition state.
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

/// Creates and shows the host window, returning its handle.
fn create_host_window() -> Result<HWND> {
  let class = WNDCLASSW {
    lpszClassName: w!("CompositionBlurSpikeHost"),
    lpfnWndProc: Some(wnd_proc),
    ..Default::default()
  };

  // SAFETY: `class` is fully initialized with a static class name and a
  // valid window procedure.
  unsafe { RegisterClassW(&raw const class) };

  // SAFETY: The class was just registered. `WS_EX_NOREDIRECTIONBITMAP`
  // skips the GDI redirection surface DWM would otherwise allocate
  // uselessly, since all pixels come from the composition visual tree.
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOREDIRECTIONBITMAP,
      w!("CompositionBlurSpikeHost"),
      w!("Composition Blur Spike"),
      WS_POPUP | WS_VISIBLE,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      HOST_WIDTH,
      HOST_HEIGHT,
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

fn run() -> Result<()> {
  // SAFETY: No preconditions; improves crispness on high-DPI displays.
  unsafe {
    let _ = SetProcessDpiAwarenessContext(
      DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    );
  }

  // Composition objects created via COM activation (`Compositor::new`,
  // `PropertyValue`, etc.) need the classic COM apartment initialized too,
  // matching `wm-platform`'s existing `COINIT_APARTMENTTHREADED` pattern.
  //
  // SAFETY: Called once, at process start, before any COM/WinRT use.
  unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }?;

  eprintln!(
    "[spike] running on OS thread {}",
    // SAFETY: No preconditions.
    unsafe { GetCurrentThreadId() }
  );

  // A `Compositor` must be created on a thread with a dispatcher queue.
  // `DQTYPE_THREAD_CURRENT` attaches the queue to *this* thread rather than
  // spinning up a new one, matching the manual `GetMessage`/`PeekMessage`
  // loop below, which pumps both our window's messages and the dispatcher
  // queue's tasks. (`DQTYPE_THREAD_DEDICATED` is the right choice once this
  // is ported into GlazeWM proper, since GlazeWM's tokio-driven main loop
  // does not pump Win32 messages -- see the plan's Threading note.)
  // `DispatcherQueueOptions` is a handful of fields; the cast is always
  // exact.
  #[allow(clippy::cast_possible_truncation)]
  let dw_size = std::mem::size_of::<DispatcherQueueOptions>() as u32;
  let options = DispatcherQueueOptions {
    dwSize: dw_size,
    threadType: DQTYPE_THREAD_CURRENT,
    apartmentType: DQTAT_COM_ASTA,
  };
  // SAFETY: `options` is fully initialized with a correct `dwSize`.
  let _dispatcher_queue_controller =
    unsafe { CreateDispatcherQueueController(options) }?;
  eprintln!("[spike] dispatcher queue controller created");

  let compositor = Compositor::new()?;
  eprintln!("[spike] compositor created");

  let hwnd = create_host_window()?;
  eprintln!("[spike] host window created");

  apply_hostbackdrop_accent(hwnd);
  apply_hostbackdrop_dwm_attribute(hwnd);

  let tree = build_visual_tree(&compositor, hwnd, 30.0)?;
  eprintln!(
    "[spike] visual tree rooted -- window should now show a live \
     blurred desktop patch"
  );

  // SAFETY: `hwnd` is valid; positions the already-created window without
  // changing its size.
  unsafe {
    let _ =
      SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER);
    let _ = ShowWindow(hwnd, SW_SHOW);
  }

  eprintln!(
    "[spike] Up/Down = blur amount, Left/Right = corner radius, Esc = exit."
  );

  run_message_loop(tree)
}

/// Live handles to the composition objects the message loop mutates in
/// response to keyboard input. `_target` and `compositor` are held alive but
/// never touched again -- dropping `_target` would unbind composition from
/// `hwnd`.
struct VisualTree {
  _target: DesktopWindowTarget,
  effect_factory: CompositionEffectFactory,
  effect_brush: CompositionEffectBrush,
  rounded_geometry: CompositionRoundedRectangleGeometry,
  blur_amount: f32,
  corner_radius: f32,
}

/// Builds the full visual tree described in the module docs: a host-backdrop
/// brush, fed through a hand-implemented Gaussian-blur effect graph, clipped
/// to a rounded rectangle, rooted onto `hwnd` via a `DesktopWindowTarget`.
fn build_visual_tree(
  compositor: &Compositor,
  hwnd: HWND,
  initial_blur_amount: f32,
) -> Result<VisualTree> {
  // SAFETY: `hwnd` is a valid, just-created top-level window.
  let target = unsafe {
    compositor
      .cast::<ICompositorDesktopInterop>()?
      .CreateDesktopWindowTarget(hwnd, false)?
  };
  eprintln!("[spike] desktop window target created");

  let host_backdrop = compositor.CreateHostBackdropBrush()?;
  eprintln!("[spike] host backdrop brush created");

  let source_param =
    CompositionEffectSourceParameter::Create(&HSTRING::from("Source"))?;
  let blur_effect: IGraphicsEffect =
    GaussianBlurEffect::new(source_param.cast()?, initial_blur_amount).into();
  eprintln!("[spike] gaussian blur effect graph description built");

  let effect_factory = compositor.CreateEffectFactory(&blur_effect)?;
  eprintln!(
    "[spike] effect factory load status (immediately after creation, \
     expected Pending) = {:?}",
    effect_factory.LoadStatus()
  );
  // Deliberately not blocking/sleeping here to wait for the async D2D1
  // shader-graph compile to settle: that compile completes via a callback
  // dispatched through the very dispatcher queue this thread owns, so
  // blocking this thread (e.g. `thread::sleep`) instead of pumping messages
  // would starve it and `LoadStatus` would misleadingly sit at `Pending`
  // forever. It's rechecked once the message loop starts pumping instead.

  let effect_brush = effect_factory.CreateBrush()?;
  effect_brush
    .SetSourceParameter(&HSTRING::from("Source"), &host_backdrop)?;
  eprintln!("[spike] effect brush created and wired to host backdrop");

  let (width, height) = HOST_SIZE_F32;

  let sprite = compositor.CreateSpriteVisual()?;
  sprite.SetBrush(&effect_brush)?;
  sprite.SetSize(Vector2 { X: width, Y: height })?;

  let corner_radius = 0.0;
  let rounded_geometry = compositor.CreateRoundedRectangleGeometry()?;
  rounded_geometry.SetSize(Vector2 { X: width, Y: height })?;
  rounded_geometry.SetCornerRadius(Vector2 {
    X: corner_radius,
    Y: corner_radius,
  })?;
  let clip = compositor.CreateGeometricClipWithGeometry(&rounded_geometry)?;
  sprite.SetClip(&clip)?;

  target.SetRoot(&sprite)?;

  Ok(VisualTree {
    _target: target,
    effect_factory,
    effect_brush,
    rounded_geometry,
    blur_amount: initial_blur_amount,
    corner_radius,
  })
}

/// Runs the `PeekMessage` loop, pumping both the host window's messages and
/// the dispatcher queue attached via `DQTYPE_THREAD_CURRENT`, adjusting the
/// blur amount and corner radius live in response to arrow-key input.
fn run_message_loop(mut tree: VisualTree) -> Result<()> {
  let mut msg = MSG::default();
  let mut logged_load_status = false;
  let loop_start = std::time::Instant::now();

  loop {
    // SAFETY: `msg` is a valid stack slot.
    while unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE) }
      .as_bool()
    {
      if msg.message == WM_QUIT {
        return Ok(());
      }

      if msg.message == WM_KEYDOWN {
        if let Ok(vk) = u16::try_from(msg.wParam.0) {
          handle_key_down(&mut tree, vk);
          if vk == VK_ESCAPE.0 {
            return Ok(());
          }
        }
      }

      // SAFETY: `msg` was just filled in by `PeekMessageW` above.
      unsafe {
        let _ = TranslateMessage(&raw const msg);
        DispatchMessageW(&raw const msg);
      }
    }

    // Log the settled effect-factory load status once, after the loop has
    // had a chance to pump the async D2D1 shader-compile completion through
    // the dispatcher queue.
    if !logged_load_status && loop_start.elapsed().as_millis() > 250 {
      logged_load_status = true;
      eprintln!(
        "[spike] effect factory load status (settled, expected Success) = {:?}",
        tree.effect_factory.LoadStatus()
      );
    }
  }
}

/// Applies one arrow-key step to `tree`'s blur amount or corner radius and
/// pushes the new value to the live composition objects.
fn handle_key_down(tree: &mut VisualTree, vk: u16) {
  if vk == VK_UP.0 {
    tree.blur_amount = (tree.blur_amount + 2.0).min(200.0);
    apply_blur_amount(tree);
  } else if vk == VK_DOWN.0 {
    tree.blur_amount = (tree.blur_amount - 2.0).max(0.0);
    apply_blur_amount(tree);
  } else if vk == VK_RIGHT.0 {
    tree.corner_radius = (tree.corner_radius + 2.0).min(300.0);
    apply_corner_radius(tree);
  } else if vk == VK_LEFT.0 {
    tree.corner_radius = (tree.corner_radius - 2.0).max(0.0);
    apply_corner_radius(tree);
  }
}

fn apply_blur_amount(tree: &VisualTree) {
  let result = tree
    .effect_brush
    .Properties()
    .and_then(|p| p.InsertScalar(&HSTRING::from("Blur.BlurAmount"), tree.blur_amount));

  match result {
    Ok(()) => eprintln!("[spike] blur amount = {}", tree.blur_amount),
    Err(err) => eprintln!("[spike] failed to update blur amount: {err}"),
  }
}

fn apply_corner_radius(tree: &VisualTree) {
  let result = tree.rounded_geometry.SetCornerRadius(Vector2 {
    X: tree.corner_radius,
    Y: tree.corner_radius,
  });

  match result {
    Ok(()) => eprintln!("[spike] corner radius = {}", tree.corner_radius),
    Err(err) => eprintln!("[spike] failed to update corner radius: {err}"),
  }
}

fn main() {
  if let Err(err) = run() {
    eprintln!("composition-blur-spike failed: {err}");
    std::process::exit(1);
  }
}
