//! Spike: animates windows through one `DirectComposition` scene covering
//! the monitor, as a compositor would, instead of a surrogate window per
//! window.
//!
//! The scene holds a live capture of the desktop minus the animated
//! windows, and a thumbnail visual of each of them on a panel above it,
//! both from private `dwmapi.dll` exports. The panel is the only thing
//! that moves, like a workspace's render offset in Hyprland, and DWM
//! itself samples its motion from the animation committed to it. While
//! the scene shows exactly the screen beneath it, the real windows are
//! cloaked and uncloaked under it, so no frame should show either.
//!
//! On the monitor under the cursor:
//!
//! 1. the scene goes up over the screen;
//! 2. the windows are cloaked beneath it and slide off the monitor;
//! 3. at `--interrupt-at` they are uncloaked beneath it and slide back
//!    from wherever they are, like a switch reversed mid-flight;
//! 4. the scene fades out over the real windows.
//!
//! With `--no-backdrop` the desktop isn't captured, so the scene can't
//! hide the real windows: they stay cloaked until the slide back ends,
//! like today's surrogates, for comparison.
//!
//! Quit the WM first, as it reacts to the cloaks. The `uncloak_all`
//! example restores windows left cloaked if this is killed midway.
//!
//! # Example usage
//!
//! ```text
//! cargo run -p wm-platform --release --example scene_spike -- \
//!   --direction left --duration-ms 300 --interrupt-at 0.6
//! ```

fn main() {
  #[cfg(target_os = "windows")]
  {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Err(err) = windows_impl::run(&args) {
      eprintln!("scene_spike failed: {err}");
      std::process::exit(1);
    }
  }

  #[cfg(not(target_os = "windows"))]
  println!("Nothing to do: the scene is Windows-only.");
}

#[cfg(target_os = "windows")]
mod curve;

#[cfg(target_os = "windows")]
mod windows_impl {
  use std::{
    cell::Cell,
    ffi::c_void,
    ptr::{null, null_mut},
  };

  use windows::{
    core::{s, w, ComInterface, Error, Interface, Result, HRESULT, PCSTR},
    Win32::{
      Foundation::{
        BOOL, E_FAIL, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE,
        WPARAM,
      },
      Graphics::{
        Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Direct3D11::{
          D3D11CreateDevice, ID3D11Device,
          D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
        },
        DirectComposition::{
          DCompositionCreateDevice3, IDCompositionAnimation,
          IDCompositionDesktopDevice, IDCompositionEffectGroup,
          IDCompositionTarget, IDCompositionVisual, IDCompositionVisual2,
        },
        Dwm::{
          DwmFlush, DwmGetCompositionTimingInfo, DwmUnregisterThumbnail,
          DWM_THUMBNAIL_PROPERTIES, DWM_TIMING_INFO, DWM_TNP_OPACITY,
          DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
          DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
        },
        Dxgi::IDXGIDevice,
        Gdi::{
          GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HMONITOR,
          MONITORINFO, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL,
        },
      },
      System::{
        LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
        Performance::{
          QueryPerformanceCounter, QueryPerformanceFrequency,
        },
      },
      UI::{
        HiDpi::{
          SetProcessDpiAwarenessContext,
          DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        },
        WindowsAndMessaging::{
          CreateWindowExW, DefWindowProcW, DestroyWindow,
          DispatchMessageW, EnumWindows, GetCursorPos, GetWindowLongPtrW,
          PeekMessageW, RegisterClassW, SetWindowPos, TranslateMessage,
          GWL_EXSTYLE, HWND_TOPMOST, MSG, PM_REMOVE, SWP_NOACTIVATE,
          SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WNDCLASSW,
          WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
          WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
        },
      },
    },
  };
  use wm_platform::{NativeWindow, NativeWindowWindowsExt, Rect};

  use super::curve::{Bezier, Tween};

  /// Longest cubic segment an eased motion is approximated by; see
  /// `Tween::segments`.
  const SEGMENT_SECS: f64 = 0.004;

  /// Opacity the scene is warmed up at before it covers the screen:
  /// composited by DWM, but too faint to see.
  const WARM_UP_OPACITY: f32 = 1.0 / 255.0;

  /// Hyprland's built-in `default` bezier.
  const HYPRLAND_DEFAULT_BEZIER: [f32; 4] = [0.0, 0.75, 0.15, 1.0];

  /// `WINDOWCOMPOSITIONATTRIB` that keeps a window out of DWM's live
  /// captures, here the desktop capture drawn in that same window.
  const WCA_EXCLUDED_FROM_LIVEPREVIEW: u32 = 13;

  /// Undocumented `dwThumbnailFlags` value the published reverse
  /// engineering of `DwmpCreateSharedThumbnailVisual` passes.
  const SHARED_THUMBNAIL_FLAGS: u32 = 2;

  /// `DwmpUpdateSharedMultiWindowVisual` flags its reverse engineering
  /// requires.
  const MULTI_WINDOW_FLAGS: u32 = 1;

  type CreateSharedThumbnailVisual = unsafe extern "system" fn(
    destination: HWND,
    source: HWND,
    flags: u32,
    properties: *const DWM_THUMBNAIL_PROPERTIES,
    device: *mut c_void,
    visual: *mut *mut c_void,
    thumbnail: *mut isize,
  )
    -> HRESULT;

  type QueryWindowThumbnailSourceSize =
    unsafe extern "system" fn(
      source: HWND,
      client_area_only: BOOL,
      size: *mut SIZE,
    ) -> HRESULT;

  type CreateSharedMultiWindowVisual =
    unsafe extern "system" fn(
      destination: HWND,
      device: *mut c_void,
      visual: *mut *mut c_void,
      thumbnail: *mut isize,
    ) -> HRESULT;

  type UpdateSharedMultiWindowVisual =
    unsafe extern "system" fn(
      thumbnail: isize,
      include: *const HWND,
      include_count: u32,
      exclude: *const HWND,
      exclude_count: u32,
      source: *const RECT,
      destination: *const SIZE,
      flags: u32,
    ) -> HRESULT;

  #[repr(C)]
  struct WindowCompositionAttribData {
    attribute: u32,
    data: *mut c_void,
    size: usize,
  }

  type SetWindowCompositionAttribute = unsafe extern "system" fn(
    hwnd: HWND,
    data: *mut WindowCompositionAttribData,
  )
    -> BOOL;

  fn fail(message: &str) -> Error {
    Error::new(E_FAIL, message.into())
  }

  #[derive(Clone, Copy, Debug)]
  enum Direction {
    Left,
    Right,
    Up,
    Down,
  }

  /// Run settings, from the command line.
  struct Options {
    direction: Direction,
    duration_secs: f64,
    /// Share of the slide away after which the windows turn back.
    interrupt_at: f64,
    lift_secs: f64,
    /// Physical pixels; 0 leaves thumbnails unclipped at the corners.
    corner_radius: f32,
    bezier: Bezier,
    backdrop: bool,
    max_windows: usize,
  }

  impl Options {
    fn parse(args: &[String]) -> Result<Self> {
      let mut options = Self {
        direction: Direction::Left,
        duration_secs: 0.3,
        interrupt_at: 1.0,
        lift_secs: 0.06,
        corner_radius: 0.0,
        bezier: Bezier(HYPRLAND_DEFAULT_BEZIER),
        backdrop: true,
        max_windows: 6,
      };

      let mut args = args.iter();
      while let Some(flag) = args.next() {
        if flag == "--no-backdrop" {
          options.backdrop = false;
          continue;
        }

        let value = args
          .next()
          .ok_or_else(|| fail(&format!("{flag} needs a value")))?;

        match flag.as_str() {
          "--direction" => {
            options.direction = match value.as_str() {
              "left" => Direction::Left,
              "right" => Direction::Right,
              "up" => Direction::Up,
              "down" => Direction::Down,
              _ => return Err(fail("--direction is left/right/up/down")),
            };
          }
          "--duration-ms" => {
            options.duration_secs = number::<f64>(flag, value)? / 1000.0;
          }
          "--interrupt-at" => {
            options.interrupt_at =
              number::<f64>(flag, value)?.clamp(0.0, 1.0);
          }
          "--lift-ms" => {
            options.lift_secs = number::<f64>(flag, value)? / 1000.0;
          }
          "--corner-radius" => {
            options.corner_radius = number(flag, value)?;
          }
          "--max-windows" => options.max_windows = number(flag, value)?,
          "--bezier" => {
            let points = value
              .split(',')
              .map(|point| number::<f32>(flag, point))
              .collect::<Result<Vec<_>>>()?;
            let [x1, y1, x2, y2] = points[..] else {
              return Err(fail("--bezier takes x1,y1,x2,y2"));
            };
            options.bezier = Bezier([x1, y1, x2, y2]);
          }
          _ => return Err(fail(&format!("unknown flag {flag}"))),
        }
      }

      Ok(options)
    }
  }

  fn number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T> {
    value
      .trim()
      .parse()
      .map_err(|_| fail(&format!("{flag}: `{value}` is not a number")))
  }

  /// QPC clock the animations are timed in.
  struct Clock {
    frequency: f64,
  }

  impl Clock {
    fn new() -> Result<Self> {
      let mut frequency = 0;
      // SAFETY: `frequency` outlives the call.
      unsafe { QueryPerformanceFrequency(&raw mut frequency) }?;
      #[allow(clippy::cast_precision_loss)]
      Ok(Self {
        frequency: frequency as f64,
      })
    }

    fn now() -> i64 {
      let mut counter = 0;
      // SAFETY: `counter` outlives the call; QPC can't fail on XP+.
      let _ = unsafe { QueryPerformanceCounter(&raw mut counter) };
      counter
    }

    #[allow(clippy::cast_precision_loss)]
    fn secs_between(&self, from: i64, to: i64) -> f64 {
      (to - from) as f64 / self.frequency
    }
  }

  /// `tween` as a `DirectComposition` animation function, sampled from
  /// QPC `begin`.
  #[allow(clippy::cast_possible_truncation)]
  fn animation(
    device: &IDCompositionDesktopDevice,
    tween: &Tween,
    begin: i64,
  ) -> Result<IDCompositionAnimation> {
    // SAFETY: Plain DirectComposition calls on live objects.
    unsafe {
      let animation = device.CreateAnimation()?;
      animation.SetAbsoluteBeginTime(begin)?;
      for segment in tween.segments(SEGMENT_SECS) {
        let [c0, c1, c2, c3] = segment.coefficients.map(|c| c as f32);
        animation.AddCubic(segment.begin, c0, c1, c2, c3)?;
      }
      animation.End(tween.duration_secs, tween.to as f32)?;
      Ok(animation)
    }
  }

  /// Private `dwmapi.dll` exports, which have ordinals but no names.
  ///
  /// On Windows 10 up to 21H1, 163 and 164 are the `VirtualDesktop`
  /// predecessors of the `MultiWindow` functions, whose update takes no
  /// trailing flags. Passing them anyway is harmless under the x64 calling
  /// convention.
  struct DwmPrivate {
    create_thumbnail_visual: CreateSharedThumbnailVisual,
    query_source_size: QueryWindowThumbnailSourceSize,
    create_multi_window_visual: CreateSharedMultiWindowVisual,
    update_multi_window_visual: UpdateSharedMultiWindowVisual,
    set_composition_attribute: SetWindowCompositionAttribute,
  }

  impl DwmPrivate {
    fn load() -> Result<Self> {
      // SAFETY: `dwmapi.dll` is a system library.
      let dwmapi = unsafe { LoadLibraryW(w!("dwmapi.dll")) }?;
      // SAFETY: `user32.dll` is loaded into every GUI process.
      let user32 = unsafe { GetModuleHandleW(w!("user32.dll")) }?;

      // SAFETY: Each export is transmuted to the signature its reverse
      // engineering documents; function pointers share one size.
      unsafe {
        Ok(Self {
          create_thumbnail_visual: std::mem::transmute::<
            RawExport,
            CreateSharedThumbnailVisual,
          >(export(dwmapi, 147)?),
          query_source_size: std::mem::transmute::<
            RawExport,
            QueryWindowThumbnailSourceSize,
          >(export(dwmapi, 162)?),
          create_multi_window_visual: std::mem::transmute::<
            RawExport,
            CreateSharedMultiWindowVisual,
          >(export(dwmapi, 163)?),
          update_multi_window_visual: std::mem::transmute::<
            RawExport,
            UpdateSharedMultiWindowVisual,
          >(export(dwmapi, 164)?),
          set_composition_attribute: std::mem::transmute::<
            RawExport,
            SetWindowCompositionAttribute,
          >(
            GetProcAddress(user32, s!("SetWindowCompositionAttribute"))
              .ok_or_else(|| fail("no SetWindowCompositionAttribute"))?,
          ),
        })
      }
    }
  }

  /// What `GetProcAddress` returns, before it's cast to its signature.
  type RawExport = unsafe extern "system" fn() -> isize;

  fn export(module: HMODULE, ordinal: u16) -> Result<RawExport> {
    // SAFETY: `GetProcAddress` takes an ordinal in the low word of the
    // name pointer.
    unsafe {
      GetProcAddress(module, PCSTR(usize::from(ordinal) as *const u8))
    }
    .ok_or_else(|| fail(&format!("dwmapi.dll has no export #{ordinal}")))
  }

  /// Click-through, never-activated topmost popup over the monitor,
  /// drawing only what its composition tree holds.
  struct Host(HWND);

  impl Host {
    fn create(monitor: &Rect) -> Result<Self> {
      extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
      ) -> LRESULT {
        // SAFETY: All parameters are forwarded unchanged.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
      }

      let class = w!("GlazeWM_SceneSpike");
      let wnd_class = WNDCLASSW {
        lpszClassName: class,
        lpfnWndProc: Some(window_proc),
        ..Default::default()
      };

      // SAFETY: `wnd_class` is fully initialized with a static class name.
      unsafe { RegisterClassW(&raw const wnd_class) };

      // SAFETY: The class was registered above.
      let hwnd = unsafe {
        CreateWindowExW(
          WS_EX_NOACTIVATE
            | WS_EX_TOOLWINDOW
            | WS_EX_TRANSPARENT
            | WS_EX_TOPMOST
            | WS_EX_NOREDIRECTIONBITMAP,
          class,
          w!(""),
          WS_POPUP,
          monitor.x(),
          monitor.y(),
          monitor.width(),
          monitor.height(),
          None,
          None,
          None,
          None,
        )
      };

      if hwnd.0 == 0 {
        return Err(Error::from_win32());
      }

      Ok(Self(hwnd))
    }

    fn show(&self) -> Result<()> {
      // SAFETY: `self.0` is a window this process created.
      unsafe {
        SetWindowPos(
          self.0,
          HWND_TOPMOST,
          0,
          0,
          0,
          0,
          SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
      }
    }
  }

  impl Drop for Host {
    fn drop(&mut self) {
      // SAFETY: `self.0` is a window this process created.
      let _ = unsafe { DestroyWindow(self.0) };
    }
  }

  /// Monitor-wide composition tree: the desktop capture at the bottom, and
  /// above it the panel holding every animated window's thumbnail.
  struct Scene {
    device: IDCompositionDesktopDevice,
    target: IDCompositionTarget,
    /// Fades the whole scene.
    fade: IDCompositionEffectGroup,
    /// Moves every window together.
    panel: IDCompositionVisual2,
    /// Visuals created by DWM, kept alive with the tree.
    visuals: Vec<IDCompositionVisual>,
    thumbnails: Vec<isize>,
    /// Windows the scene draws, which may be cloaked under it. One DWM
    /// can't make a thumbnail of stays shown, and in the capture.
    windows: Vec<NativeWindow>,
    has_backdrop: bool,
  }

  impl Scene {
    fn build(
      dwm: &DwmPrivate,
      host: &Host,
      monitor: &Rect,
      windows: &[NativeWindow],
      options: &Options,
    ) -> Result<Self> {
      let device = create_device()?;

      // SAFETY: Plain DirectComposition calls on live objects.
      let (target, root, fade, panel) = unsafe {
        let target = device.CreateTargetForHwnd(host.0, true)?;
        let root = device.CreateVisual()?;
        let fade = device.CreateEffectGroup()?;
        fade.SetOpacity2(WARM_UP_OPACITY)?;
        root.SetEffect(&fade)?;
        let panel = device.CreateVisual()?;
        (target, root, fade, panel)
      };

      let mut scene = Self {
        device,
        target,
        fade,
        panel,
        visuals: Vec::new(),
        thumbnails: Vec::new(),
        windows: Vec::new(),
        has_backdrop: false,
      };

      // Bottom-most first, each added above the ones before it.
      for window in windows.iter().rev() {
        match scene.add_window(
          dwm,
          host,
          monitor,
          window,
          options.corner_radius,
        ) {
          Ok(()) => scene.windows.push(window.clone()),
          Err(err) => println!("Left a window out ({err})."),
        }
      }

      // SAFETY: Plain DirectComposition call on live visuals.
      unsafe { root.AddVisual(&scene.panel, false, None) }?;

      if options.backdrop {
        match scene.add_backdrop(dwm, host, monitor, &root) {
          Ok(()) => scene.has_backdrop = true,
          Err(err) => println!(
            "No desktop capture ({err}); comparing against today's \
             handoff instead."
          ),
        }
      }

      // SAFETY: Plain DirectComposition calls on live objects.
      unsafe {
        scene.target.SetRoot(&root)?;
        scene.device.Commit()?;
        scene.device.WaitForCommitCompletion()?;
      }

      Ok(scene)
    }

    /// Adds, below the panel, a live capture of the monitor without the
    /// scene's windows or the host.
    fn add_backdrop(
      &mut self,
      dwm: &DwmPrivate,
      host: &Host,
      monitor: &Rect,
      root: &IDCompositionVisual2,
    ) -> Result<()> {
      let mut excluded = BOOL::from(true);
      let mut data = WindowCompositionAttribData {
        attribute: WCA_EXCLUDED_FROM_LIVEPREVIEW,
        data: (&raw mut excluded).cast(),
        size: std::mem::size_of::<BOOL>(),
      };
      // SAFETY: `data` points at a `BOOL` that outlives the call.
      if !unsafe { (dwm.set_composition_attribute)(host.0, &raw mut data) }
        .as_bool()
      {
        println!("Couldn't keep the scene out of its own capture.");
      }

      let mut raw = null_mut();
      let mut thumbnail = 0;
      // SAFETY: Out pointers outlive the call; the device is live.
      unsafe {
        (dwm.create_multi_window_visual)(
          host.0,
          self.device.as_raw(),
          &raw mut raw,
          &raw mut thumbnail,
        )
      }
      .ok()?;
      // SAFETY: On success, `raw` is an owned `IDCompositionVisual`.
      let visual = unsafe { IDCompositionVisual::from_raw(raw) };
      self.thumbnails.push(thumbnail);

      let mut exclude = self
        .windows
        .iter()
        .map(NativeWindow::hwnd)
        .collect::<Vec<_>>();
      exclude.push(host.0);
      let source = RECT {
        left: monitor.left,
        top: monitor.top,
        right: monitor.right,
        bottom: monitor.bottom,
      };
      let size = SIZE {
        cx: monitor.width(),
        cy: monitor.height(),
      };

      // SAFETY: Arrays and structs outlive the call.
      unsafe {
        (dwm.update_multi_window_visual)(
          thumbnail,
          null(),
          0,
          exclude.as_ptr(),
          u32::try_from(exclude.len()).unwrap_or(u32::MAX),
          &raw const source,
          &raw const size,
          MULTI_WINDOW_FLAGS,
        )
      }
      .ok()?;

      // With no reference visual, `insertAbove` puts it below all others.
      // SAFETY: Plain DirectComposition call on live visuals.
      unsafe { root.AddVisual(&visual, true, None) }?;
      self.visuals.push(visual);
      Ok(())
    }

    /// Adds `window`'s thumbnail to the panel where it is on screen, cut
    /// to its visible frame.
    #[allow(clippy::cast_precision_loss)]
    fn add_window(
      &mut self,
      dwm: &DwmPrivate,
      host: &Host,
      monitor: &Rect,
      window: &NativeWindow,
      corner_radius: f32,
    ) -> Result<()> {
      let outer = window
        .frame_with_shadows()
        .map_err(|err| fail(&err.to_string()))?;
      let visible =
        window.frame().map_err(|err| fail(&err.to_string()))?;

      let mut size = SIZE::default();
      // SAFETY: `size` outlives the call.
      unsafe {
        (dwm.query_source_size)(window.hwnd(), false.into(), &raw mut size)
      }
      .ok()?;
      if (size.cx, size.cy) != (outer.width(), outer.height()) {
        println!(
          "Thumbnail source is {}x{}, window rect {}x{}.",
          size.cx,
          size.cy,
          outer.width(),
          outer.height(),
        );
      }

      let whole = RECT {
        left: 0,
        top: 0,
        right: size.cx,
        bottom: size.cy,
      };
      let properties = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_RECTDESTINATION
          | DWM_TNP_RECTSOURCE
          | DWM_TNP_OPACITY
          | DWM_TNP_VISIBLE
          | DWM_TNP_SOURCECLIENTAREAONLY,
        rcDestination: whole,
        rcSource: whole,
        opacity: u8::MAX,
        fVisible: true.into(),
        fSourceClientAreaOnly: false.into(),
      };

      let mut raw = null_mut();
      let mut thumbnail = 0;
      // SAFETY: Out pointers and `properties` outlive the call.
      unsafe {
        (dwm.create_thumbnail_visual)(
          host.0,
          window.hwnd(),
          SHARED_THUMBNAIL_FLAGS,
          &raw const properties,
          self.device.as_raw(),
          &raw mut raw,
          &raw mut thumbnail,
        )
      }
      .ok()?;
      // SAFETY: On success, `raw` is an owned `IDCompositionVisual`.
      let content = unsafe { IDCompositionVisual::from_raw(raw) };
      self.thumbnails.push(thumbnail);

      // SAFETY: Plain DirectComposition calls on live objects.
      unsafe {
        let item = self.device.CreateVisual()?;
        item.AddVisual(&content, false, None)?;
        item.SetOffsetX2((outer.left - monitor.left) as f32)?;
        item.SetOffsetY2((outer.top - monitor.top) as f32)?;

        // Thumbnails include the transparent resize border; cut to what
        // DWM draws of the window, rounded like it if asked.
        let clip = self.device.CreateRectangleClip()?;
        clip.SetLeft2((visible.left - outer.left) as f32)?;
        clip.SetTop2((visible.top - outer.top) as f32)?;
        clip.SetRight2((visible.right - outer.left) as f32)?;
        clip.SetBottom2((visible.bottom - outer.top) as f32)?;
        if corner_radius > 0.0 {
          clip.SetTopLeftRadiusX2(corner_radius)?;
          clip.SetTopLeftRadiusY2(corner_radius)?;
          clip.SetTopRightRadiusX2(corner_radius)?;
          clip.SetTopRightRadiusY2(corner_radius)?;
          clip.SetBottomLeftRadiusX2(corner_radius)?;
          clip.SetBottomLeftRadiusY2(corner_radius)?;
          clip.SetBottomRightRadiusX2(corner_radius)?;
          clip.SetBottomRightRadiusY2(corner_radius)?;
        }
        item.SetClip(&clip)?;

        self.panel.AddVisual(&item, false, None)?;
      }

      self.visuals.push(content);
      Ok(())
    }

    fn set_opacity(&self, opacity: f32) -> Result<()> {
      // SAFETY: Plain DirectComposition calls on live objects.
      unsafe {
        self.fade.SetOpacity2(opacity)?;
        self.device.Commit()
      }
    }

    /// Moves the panel along `tween` on `direction`'s axis, from QPC
    /// `begin`.
    fn slide(
      &self,
      direction: Direction,
      tween: &Tween,
      begin: i64,
    ) -> Result<()> {
      let animation = animation(&self.device, tween, begin)?;
      // SAFETY: Plain DirectComposition calls on live objects.
      unsafe {
        match direction {
          Direction::Left | Direction::Right => {
            self.panel.SetOffsetX(&animation)?;
          }
          Direction::Up | Direction::Down => {
            self.panel.SetOffsetY(&animation)?;
          }
        }
        self.device.Commit()
      }
    }

    fn fade(&self, tween: &Tween, begin: i64) -> Result<()> {
      let animation = animation(&self.device, tween, begin)?;
      // SAFETY: Plain DirectComposition calls on live objects.
      unsafe {
        self.fade.SetOpacity(&animation)?;
        self.device.Commit()
      }
    }
  }

  impl Drop for Scene {
    fn drop(&mut self) {
      for thumbnail in &self.thumbnails {
        // SAFETY: Each handle came from DWM and is released once.
        let _ = unsafe { DwmUnregisterThumbnail(*thumbnail) };
      }
    }
  }

  fn create_device() -> Result<IDCompositionDesktopDevice> {
    let mut d3d: Option<ID3D11Device> = None;
    // SAFETY: Out pointer outlives the call.
    unsafe {
      D3D11CreateDevice(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        None,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        None,
        D3D11_SDK_VERSION,
        Some(&raw mut d3d),
        None,
        None,
      )
    }?;
    let dxgi = d3d
      .ok_or_else(|| fail("no D3D11 device"))?
      .cast::<IDXGIDevice>()?;
    // SAFETY: `dxgi` is a live DXGI device.
    unsafe { DCompositionCreateDevice3(&dxgi) }
  }

  /// Uncloaks its windows when dropped, so no exit leaves them hidden.
  struct Cloaks<'a> {
    windows: &'a [NativeWindow],
    cloaked: Cell<bool>,
  }

  impl<'a> Cloaks<'a> {
    fn new(windows: &'a [NativeWindow]) -> Self {
      Self {
        windows,
        cloaked: Cell::new(false),
      }
    }

    fn set(&self, cloaked: bool) {
      for window in self.windows {
        if let Err(err) = window.set_cloaked(cloaked) {
          println!("Couldn't (un)cloak a window: {err}.");
        }
      }
      self.cloaked.set(cloaked);
    }
  }

  impl Drop for Cloaks<'_> {
    fn drop(&mut self) {
      if self.cloaked.get() {
        self.set(false);
      }
    }
  }

  /// Monitor under the cursor, with its bounds.
  fn cursor_monitor() -> Result<(HMONITOR, Rect)> {
    let mut point = POINT::default();
    // SAFETY: `point` outlives the call.
    unsafe { GetCursorPos(&raw mut point) }?;
    // SAFETY: Plain monitor lookup.
    let monitor =
      unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };

    let mut info = MONITORINFO {
      cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>())
        .unwrap_or(0),
      ..Default::default()
    };
    // SAFETY: `info` outlives the call, with `cbSize` set.
    if !unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool() {
      return Err(fail("no monitor info"));
    }

    let bounds = info.rcMonitor;
    Ok((
      monitor,
      Rect::from_ltrb(
        bounds.left,
        bounds.top,
        bounds.right,
        bounds.bottom,
      ),
    ))
  }

  /// Shown, cloakable app windows on `monitor`, topmost first.
  fn windows_on(monitor: HMONITOR, max: usize) -> Vec<NativeWindow> {
    extern "system" fn collect(handle: HWND, data: LPARAM) -> BOOL {
      // SAFETY: `data` is the `Vec` passed to `EnumWindows` below, which
      // outlives the enumeration.
      unsafe { (*(data.0 as *mut Vec<isize>)).push(handle.0) };
      true.into()
    }

    let mut handles: Vec<isize> = Vec::new();
    // SAFETY: `collect` matches the callback signature and `handles`
    // outlives the call.
    let _ = unsafe {
      EnumWindows(
        Some(collect),
        LPARAM(std::ptr::from_mut(&mut handles) as _),
      )
    };

    handles
      .into_iter()
      .map(NativeWindow::from_handle)
      .filter(|window| {
        // SAFETY: Plain style and monitor lookups.
        let (ex_style, window_monitor) = unsafe {
          (
            GetWindowLongPtrW(window.hwnd(), GWL_EXSTYLE),
            MonitorFromWindow(window.hwnd(), MONITOR_DEFAULTTONULL),
          )
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let is_tool = ex_style as u32 & WS_EX_TOOLWINDOW.0 != 0;

        window_monitor == monitor
          && !is_tool
          && window.is_visible().unwrap_or(false)
          && !window.is_minimized().unwrap_or(true)
          && !window.is_cloaked().unwrap_or(true)
          && !window.has_owner_window()
          && !window.is_desktop_window().unwrap_or(true)
          && window.title().is_ok_and(|title| !title.trim().is_empty())
          && window.can_be_cloaked()
      })
      .take(max)
      .collect()
  }

  fn pump_messages() {
    let mut msg = MSG::default();
    // SAFETY: `msg` outlives each call.
    while unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE) }
      .as_bool()
    {
      // SAFETY: `msg` was just filled in by `PeekMessageW`.
      unsafe {
        TranslateMessage(&raw const msg);
        DispatchMessageW(&raw const msg);
      }
    }
  }

  /// Waits for `frames` DWM compositions.
  fn wait_frames(frames: u32) {
    for _ in 0..frames {
      pump_messages();
      // SAFETY: Blocks until DWM's next composition.
      if unsafe { DwmFlush() }.is_err() {
        std::thread::sleep(std::time::Duration::from_millis(16));
      }
    }
  }

  /// Waits, a composition at a time, until `secs` after QPC `from`.
  fn wait_until(clock: &Clock, from: i64, secs: f64) {
    while clock.secs_between(from, Clock::now()) < secs {
      wait_frames(1);
    }
  }

  fn timing() -> DWM_TIMING_INFO {
    let mut info = DWM_TIMING_INFO {
      cbSize: u32::try_from(std::mem::size_of::<DWM_TIMING_INFO>())
        .unwrap_or(0),
      ..Default::default()
    };
    // SAFETY: `info` outlives the call, with `cbSize` set. On failure it
    // stays zeroed, which the report shows.
    let _ = unsafe { DwmGetCompositionTimingInfo(HWND(0), &raw mut info) };
    info
  }

  /// What a run measured.
  struct Report {
    /// When the scene covered the screen.
    covered: i64,
    before: DWM_TIMING_INFO,
    after: DWM_TIMING_INFO,
  }

  /// Covers the screen with `scene`, slides the windows away and back
  /// under it, then lifts it.
  fn play(
    clock: &Clock,
    host: &Host,
    scene: &Scene,
    options: &Options,
    distance: f64,
  ) -> Result<Report> {
    // Warm up under the screen it copies, then cover it.
    host.show()?;
    wait_frames(2);
    scene.set_opacity(1.0)?;
    wait_frames(2);
    let covered = Clock::now();
    let before = timing();

    let cloaks = Cloaks::new(&scene.windows);
    cloaks.set(true);

    let away = Tween {
      from: 0.0,
      to: distance,
      duration_secs: options.duration_secs,
      curve: options.bezier,
    };
    let away_begin = Clock::now();
    scene.slide(options.direction, &away, away_begin)?;
    wait_until(
      clock,
      away_begin,
      options.duration_secs * options.interrupt_at,
    );

    if scene.has_backdrop {
      cloaks.set(false);
    }

    // Like Hyprland, a retarget starts from wherever the motion is, over
    // the full duration again.
    let back_begin = Clock::now();
    let back = Tween {
      from: away.at(clock.secs_between(away_begin, back_begin)),
      to: 0.0,
      ..away
    };
    scene.slide(options.direction, &back, back_begin)?;
    wait_until(clock, back_begin, options.duration_secs);
    wait_frames(1);

    if !scene.has_backdrop {
      cloaks.set(false);
      wait_frames(2);
    }

    let lift = Tween {
      from: 1.0,
      to: 0.0,
      duration_secs: options.lift_secs,
      curve: Bezier::LINEAR,
    };
    let lift_begin = Clock::now();
    scene.fade(&lift, lift_begin)?;
    wait_until(clock, lift_begin, options.lift_secs);
    wait_frames(2);

    Ok(Report {
      covered,
      before,
      after: timing(),
    })
  }

  pub fn run(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;

    // SAFETY: Called before any window is created.
    let _ = unsafe {
      SetProcessDpiAwarenessContext(
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
      )
    };

    let clock = Clock::new()?;
    let dwm = DwmPrivate::load()?;
    let (monitor, bounds) = cursor_monitor()?;
    let windows = windows_on(monitor, options.max_windows);
    if windows.is_empty() {
      return Err(fail("no app windows on the monitor under the cursor"));
    }

    let distance = f64::from(match options.direction {
      Direction::Left => -bounds.width(),
      Direction::Right => bounds.width(),
      Direction::Up => -bounds.height(),
      Direction::Down => bounds.height(),
    });

    let host = Host::create(&bounds)?;
    let started = Clock::now();
    let scene = Scene::build(&dwm, &host, &bounds, &windows, &options)?;
    let built = Clock::now();
    if scene.windows.is_empty() {
      return Err(fail("DWM made a thumbnail of none of the windows"));
    }

    println!("Animating {} window(s):", scene.windows.len());
    for window in &scene.windows {
      println!(
        "  {} ({})",
        window.process_name().unwrap_or_default(),
        window.class_name().unwrap_or_default(),
      );
    }

    let report = play(&clock, &host, &scene, &options, distance)?;

    println!(
      "Built in {:.1} ms, covered after {:.1} ms.",
      clock.secs_between(started, built) * 1000.0,
      clock.secs_between(started, report.covered) * 1000.0,
    );
    println!(
      "While animating, DWM composed {} frames: {} missed, {} dropped.",
      report.after.cFrame.saturating_sub(report.before.cFrame),
      report
        .after
        .cFramesMissed
        .saturating_sub(report.before.cFramesMissed),
      report
        .after
        .cFramesDropped
        .saturating_sub(report.before.cFramesDropped),
    );
    println!(
      "Mode: {}.",
      if scene.has_backdrop {
        "cover (cloaks hidden under the scene)"
      } else {
        "handoff (real windows cloaked until the end)"
      }
    );

    Ok(())
  }
}
