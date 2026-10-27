//! A compositor-style animation layer: one `DirectComposition` tree over a
//! monitor, holding live copies of everything on it, which DWM animates
//! on its own clock.
//!
//! The WM covers the monitor with a scene, changes the real windows
//! beneath it at once, as it does with animations off, and lets the scene
//! play the motion before lifting it over the windows it then matches.
//! No window is ever swapped for its copy in view: every cloak and uncloak
//! happens under the cover.
//!
//! Copies are thumbnail visuals from `DwmpCreateSharedThumbnailVisual`, a
//! private `dwmapi.dll` export, so a scene is one visual tree that commits
//! atomically. What lies beneath the moving windows is a copy of every
//! other window on the monitor, desktop included, rather than a live
//! capture of the screen without them: on Windows 11 build 26200,
//! `DwmpUpdateSharedMultiWindowVisual` ignores the windows it is asked to
//! leave out.

use std::{
  cell::{OnceCell, RefCell},
  collections::HashSet,
  ffi::c_void,
  ptr::null_mut,
  rc::Rc,
  sync::OnceLock,
  time::{Duration, Instant},
};

use windows::{
  core::{w, ComInterface, Interface, HRESULT, PCSTR, PCWSTR},
  Foundation::Numerics::Matrix3x2,
  Win32::{
    Foundation::{BOOL, HMODULE, HWND, LPARAM, POINT, RECT, SIZE, TRUE},
    Graphics::{
      Direct3D::D3D_DRIVER_TYPE_HARDWARE,
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11Texture2D, D3D11_BOX,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
      },
      DirectComposition::{
        DCompositionCreateDevice3, IDCompositionAnimation,
        IDCompositionDesktopDevice, IDCompositionEffectGroup,
        IDCompositionScaleTransform, IDCompositionSurface,
        IDCompositionTarget, IDCompositionVisual, IDCompositionVisual2,
        DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
      },
      Dwm::{
        DwmUnregisterThumbnail, DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY,
        DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
        DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
      },
      Dxgi::{
        Common::{
          DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM,
        },
        IDXGIDevice,
      },
    },
    System::{
      LibraryLoader::{GetProcAddress, LoadLibraryW},
      Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, EnumWindows, GetClassNameW,
      GetWindow, GetWindowRect, IsIconic, IsWindowVisible, SetWindowPos,
      GW_HWNDPREV, GW_OWNER, HWND_TOPMOST, SET_WINDOW_POS_FLAGS,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
      SWP_SHOWWINDOW, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
      WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  backdrop_canvas, companion,
  platform_impl::{
    composition::{VIGNETTE_CLEAR, VIGNETTE_RADIUS},
    NativeWindow,
  },
  window_class, BackdropOverlayParams, Rect,
};

const CLASS_NAME: PCWSTR = w!("GlazeWM_Scene");
const CLASS: &str = "GlazeWM_Scene";

/// Most windows looked through above a scene for one raised over it.
const MAX_WINDOWS_ABOVE: usize = 256;

/// Side of the square image of the vignette copies stretch over windows.
const VIGNETTE_SIZE: u32 = 256;

/// Prefix of the WM's own window classes.
const OWN_CLASS_PREFIX: &str = "GlazeWM_";

/// The WM's own windows a scene copies: what it draws around and behind
/// windows. Its others stand in for windows (surrogates, peeks) or cover
/// the screen (overview, iris), and are left out.
const COPIED_OWN_CLASSES: [&str; 3] = [
  "GlazeWM_BorderOverlay",
  "GlazeWM_BackdropOverlay",
  "GlazeWM_TabBar",
];

/// Opacity a scene is first composited at, under the screen it copies:
/// drawn by DWM, but too faint to see. The copies are then known to have
/// content before they cover anything.
const WARM_UP_OPACITY: f32 = 1.0 / 255.0;

/// Undocumented `dwThumbnailFlags` value the published reverse
/// engineering of `DwmpCreateSharedThumbnailVisual` passes.
const SHARED_THUMBNAIL_FLAGS: u32 = 2;

/// How many owners up a window is followed to the window it goes with.
const MAX_OWNER_DEPTH: usize = 8;

type CreateSharedThumbnailVisual = unsafe extern "system" fn(
  destination: HWND,
  source: HWND,
  flags: u32,
  properties: *const DWM_THUMBNAIL_PROPERTIES,
  device: *mut c_void,
  visual: *mut *mut c_void,
  thumbnail: *mut isize,
) -> HRESULT;

type QueryWindowThumbnailSourceSize = unsafe extern "system" fn(
  source: HWND,
  client_area_only: BOOL,
  size: *mut SIZE,
)
  -> HRESULT;

/// What `GetProcAddress` returns, before it's cast to its signature.
type RawExport = unsafe extern "system" fn() -> isize;

/// Private `dwmapi.dll` exports, which have ordinals but no names.
struct DwmPrivate {
  create_thumbnail_visual: CreateSharedThumbnailVisual,
  query_source_size: QueryWindowThumbnailSourceSize,
}

impl DwmPrivate {
  /// The exports, or `None` on a Windows build without them.
  fn get() -> Option<&'static Self> {
    static EXPORTS: OnceLock<Option<DwmPrivate>> = OnceLock::new();

    EXPORTS
      .get_or_init(|| {
        let exports = Self::load();
        if exports.is_none() {
          tracing::warn!(
            "DWM lacks the thumbnail exports scenes are built from."
          );
        }
        exports
      })
      .as_ref()
  }

  fn load() -> Option<Self> {
    // SAFETY: `dwmapi.dll` is a system library.
    let dwmapi = unsafe { LoadLibraryW(w!("dwmapi.dll")) }.ok()?;

    // SAFETY: Each export is cast to the signature its reverse engineering
    // documents; function pointers share one size.
    unsafe {
      Some(Self {
        create_thumbnail_visual: std::mem::transmute::<
          RawExport,
          CreateSharedThumbnailVisual,
        >(export(dwmapi, 147)?),
        query_source_size: std::mem::transmute::<
          RawExport,
          QueryWindowThumbnailSourceSize,
        >(export(dwmapi, 162)?),
      })
    }
  }
}

fn export(module: HMODULE, ordinal: u16) -> Option<RawExport> {
  // SAFETY: `GetProcAddress` takes an ordinal in the low word of the name
  // pointer.
  unsafe {
    GetProcAddress(module, PCSTR(usize::from(ordinal) as *const u8))
  }
}

/// The `DirectComposition` device scenes are built with, and the D3D
/// device it renders through.
struct SceneDevice {
  dcomp: IDCompositionDesktopDevice,
  d3d: ID3D11Device,
  exports: &'static DwmPrivate,

  /// See [`SceneDevice::vignette`].
  vignette: OnceCell<IDCompositionSurface>,
}

thread_local! {
  /// The thread's device: created on first use, which is slow enough
  /// (D3D initialization) to keep, and kept until it fails.
  static DEVICE: RefCell<Option<Rc<SceneDevice>>> =
    const { RefCell::new(None) };
}

impl SceneDevice {
  fn shared() -> crate::Result<Rc<Self>> {
    DEVICE.with(|device| {
      if let Some(device) = device.borrow().as_ref() {
        return Ok(device.clone());
      }

      let created = Rc::new(Self::create()?);
      *device.borrow_mut() = Some(created.clone());
      Ok(created)
    })
  }

  /// Drops the thread's device, e.g. after its GPU was reset, for the
  /// next scene to create anew.
  fn discard() {
    DEVICE.with(|device| device.borrow_mut().take());
  }

  fn create() -> crate::Result<Self> {
    let exports = DwmPrivate::get().ok_or_else(|| {
      crate::Error::Platform("DWM has no thumbnail visuals.".to_string())
    })?;

    let mut d3d: Option<ID3D11Device> = None;
    // SAFETY: The out pointer outlives the call.
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

    let d3d = d3d.ok_or_else(|| {
      crate::Error::Platform("No D3D11 device.".to_string())
    })?;
    let dxgi = d3d.cast::<IDXGIDevice>()?;

    // SAFETY: `dxgi` is a live DXGI device.
    let dcomp = unsafe { DCompositionCreateDevice3(&dxgi) }?;

    Ok(Self {
      dcomp,
      d3d,
      exports,
      vignette: OnceCell::new(),
    })
  }

  /// The vignette as backdrop overlays draw it, at full strength over a
  /// square, for copies to stretch over their windows. Made on first use.
  fn vignette(&self) -> crate::Result<IDCompositionSurface> {
    if let Some(surface) = self.vignette.get() {
      return Ok(surface.clone());
    }

    let side = VIGNETTE_SIZE;
    let pixels = (0..side * side)
      .flat_map(|index| {
        let at = |pixel: u32| (f64::from(pixel) + 0.5) / f64::from(side);
        let alpha = vignette_alpha(at(index % side), at(index / side));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let alpha = (alpha * 255.0).round() as u8;
        // Premultiplied black.
        [0, 0, 0, alpha]
      })
      .collect::<Vec<u8>>();

    // SAFETY: Plain `DirectComposition` and D3D calls on live objects of
    // this device, `pixels` holding the `side`-square image they read.
    let surface = unsafe {
      let surface = self.dcomp.CreateSurface(
        side,
        side,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_ALPHA_MODE_PREMULTIPLIED,
      )?;

      let mut offset = POINT::default();
      let texture =
        surface.BeginDraw::<ID3D11Texture2D>(None, &raw mut offset)?;
      let (left, top) = (
        u32::try_from(offset.x).unwrap_or(0),
        u32::try_from(offset.y).unwrap_or(0),
      );
      let area = D3D11_BOX {
        left,
        top,
        front: 0,
        right: left + side,
        bottom: top + side,
        back: 1,
      };

      self.d3d.GetImmediateContext()?.UpdateSubresource(
        &texture,
        0,
        Some(&raw const area),
        pixels.as_ptr().cast(),
        side * 4,
        0,
      );
      surface.EndDraw()?;
      surface
    };

    let _ = self.vignette.set(surface.clone());
    Ok(surface)
  }
}

/// A window to copy into a scene.
#[derive(Clone, Debug)]
pub struct SceneWindow {
  pub hwnd: HWND,

  /// Its window rect and visible frame, where the caller knows them
  /// better than the window does yet, e.g. right after an asynchronous
  /// move. Read from the window when `None`.
  pub placement: Option<(Rect, Rect)>,

  /// Constant alpha its copy is drawn at, e.g. one it takes on with
  /// focus. When `None`, the one it's layered with, which thumbnails
  /// leave out.
  pub alpha: Option<u8>,

  /// Radius of its visible frame's corners, in physical pixels.
  pub corner_radius: f32,

  /// The backdrop drawn beneath it in place of its backdrop overlay.
  pub backdrop: Option<SceneBackdrop>,

  /// Whether its group can't do without it: a group fails to be made
  /// without a required window, and is made without an optional one.
  pub required: bool,
}

impl SceneWindow {
  /// Whether its copy draws `hwnd`: itself, or its backdrop overlay.
  fn draws(&self, hwnd: HWND) -> bool {
    self.hwnd == hwnd
      || self
        .backdrop
        .is_some_and(|backdrop| backdrop.overlay == hwnd)
  }

  /// An optional window, copied as and where it is.
  fn as_is(hwnd: HWND) -> Self {
    Self {
      hwnd,
      placement: None,
      alpha: None,
      corner_radius: 0.0,
      backdrop: None,
      required: false,
    }
  }
}

/// A window's backdrop as its overlay draws it, but from the blurred
/// wallpaper of the whole monitor, which stays put as the window moves: a
/// copy of the overlay would carry its crop of it along.
#[derive(Clone, Copy, Debug)]
pub struct SceneBackdrop {
  /// The overlay drawing it, left out of the scene.
  pub overlay: HWND,
  pub params: BackdropOverlayParams,
}

/// A group of copies posed together, e.g. a workspace's windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SceneGroupId(u32);

/// A scalar animation for DWM to play.
#[derive(Clone, Debug)]
pub struct SceneMotion {
  pub begins_at: Instant,
  pub duration: Duration,

  /// Cubic pieces of the motion, the first starting at 0 seconds.
  pub segments: Vec<SceneSegment>,

  /// Value from `duration` on.
  pub end: f32,
}

/// One piece of a [`SceneMotion`]: `u` seconds into it, its value is
/// `c[0] + c[1] u + c[2] u² + c[3] u³`.
#[derive(Clone, Copy, Debug)]
pub struct SceneSegment {
  /// Seconds into the motion the piece starts at.
  pub begin_secs: f64,
  pub coefficients: [f32; 4],
}

/// A property of a scene, fixed or moving.
#[derive(Clone, Debug)]
pub enum SceneValue {
  Fixed(f32),
  Moving(SceneMotion),
}

impl SceneValue {
  /// `scale * value + add` at every moment.
  fn mapped(&self, scale: f32, add: f32) -> Self {
    match self {
      Self::Fixed(value) => Self::Fixed(scale * value + add),
      Self::Moving(motion) => Self::Moving(SceneMotion {
        begins_at: motion.begins_at,
        duration: motion.duration,
        segments: motion
          .segments
          .iter()
          .map(|segment| {
            let [c0, c1, c2, c3] = segment.coefficients;
            SceneSegment {
              begin_secs: segment.begin_secs,
              coefficients: [
                scale * c0 + add,
                scale * c1,
                scale * c2,
                scale * c3,
              ],
            }
          })
          .collect(),
        end: scale * motion.end + add,
      }),
    }
  }
}

/// Where and how a group is drawn.
#[derive(Clone, Debug)]
pub struct ScenePose {
  pub offset_x: SceneValue,
  pub offset_y: SceneValue,
  pub opacity: SceneValue,

  /// Scale of the whole group, about the monitor's center.
  pub scale: SceneValue,

  /// Scale of each copy, about its own center.
  pub copy_scale: SceneValue,
}

/// A live copy of a window in a scene.
struct WindowCopy {
  thumbnail: isize,

  /// Scales the copy about its own center.
  scale: IDCompositionScaleTransform,

  backdrop: Option<BackdropCopy>,
}

impl WindowCopy {
  fn unregister(&self) {
    // SAFETY: The handle came from DWM and is released once, here.
    let _ = unsafe { DwmUnregisterThumbnail(self.thumbnail) };
    if let Some(backdrop) = &self.backdrop {
      backdrop.unregister();
    }
  }
}

/// A window's backdrop in its copy: a vignette over the monitor's blurred
/// wallpaper, which its group's motion moves the other way, so it stays
/// put on screen (see [`SceneWindow::backdrop`]).
struct BackdropCopy {
  /// Its thumbnail of the monitor's backdrop canvas.
  thumbnail: isize,

  /// Holds the backdrop beneath the window, under its opacity.
  visual: IDCompositionVisual2,

  /// Holds the canvas's thumbnail, offset against the group's motion.
  blur: IDCompositionVisual2,

  /// Offset of `blur` with the group at rest, in the copy's coordinates.
  rest: (f32, f32),

  /// Share of the group's motion `blur` is moved back by.
  parallax: f32,
}

impl BackdropCopy {
  fn unregister(&self) {
    // SAFETY: The handle came from DWM and is released once, here.
    let _ = unsafe { DwmUnregisterThumbnail(self.thumbnail) };
  }
}

struct Group {
  id: SceneGroupId,

  /// Moved by the group's offset; holds the rest.
  panel: IDCompositionVisual2,
  effect: IDCompositionEffectGroup,

  /// Scales the group about the monitor's center.
  scale: IDCompositionScaleTransform,

  /// Holds the copies, under `scale`.
  content: IDCompositionVisual2,
  copies: Vec<WindowCopy>,
}

/// A click-through, never-activated topmost popup over a monitor, drawing
/// only what its composition tree holds.
///
/// Like the overlays, it lives on the WM's thread, which pumps no
/// messages, so `WS_EX_TRANSPARENT` keeps it from being hit-tested.
struct Host {
  hwnd: HWND,
  target: IDCompositionTarget,
}

impl Host {
  fn create(device: &SceneDevice, monitor: &Rect) -> crate::Result<Self> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    window_class::ensure_class_registered(
      &REGISTERED,
      CLASS_NAME,
      window_class::default_wnd_proc,
    );

    // SAFETY: The class is registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE
          | WS_EX_TOOLWINDOW
          | WS_EX_TRANSPARENT
          | WS_EX_TOPMOST
          | WS_EX_NOREDIRECTIONBITMAP,
        CLASS_NAME,
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
      return Err(crate::Error::Platform(
        "Failed to create scene window.".to_string(),
      ));
    }

    // SAFETY: `hwnd` is a window of this thread with no target yet.
    match unsafe { device.dcomp.CreateTargetForHwnd(hwnd, true) } {
      Ok(target) => Ok(Self { hwnd, target }),
      Err(err) => {
        // SAFETY: Created above, and not handed out.
        let _ = unsafe { DestroyWindow(hwnd) };
        Err(err.into())
      }
    }
  }

  fn show(&self) -> crate::Result<()> {
    self.to_top(SWP_SHOWWINDOW)
  }

  /// Moves it to the top of the topmost windows.
  fn to_top(&self, flags: SET_WINDOW_POS_FLAGS) -> crate::Result<()> {
    // SAFETY: `self.hwnd` is a window of this thread.
    unsafe {
      SetWindowPos(
        self.hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | flags,
      )
    }?;
    Ok(())
  }

  /// Whether a window other than a scene's sits above it.
  fn is_overlapped(&self) -> bool {
    let mut above = self.hwnd;

    for _ in 0..MAX_WINDOWS_ABOVE {
      // SAFETY: A stale handle just returns `HWND(0)`.
      above = unsafe { GetWindow(above, GW_HWNDPREV) };
      if above.0 == 0 {
        return false;
      }
      if class_name(above) != CLASS {
        return true;
      }
    }

    true
  }
}

impl Drop for Host {
  fn drop(&mut self) {
    // SAFETY: `self.hwnd` is a window of this thread, destroyed once.
    let _ = unsafe { DestroyWindow(self.hwnd) };
  }
}

/// Live copies of everything on a monitor, drawn over it by DWM.
///
/// Changes take effect together on [`commit`](Self::commit); dropping the
/// scene takes it off the screen at once.
pub struct NativeScene {
  device: Rc<SceneDevice>,
  host: Host,
  monitor: Rect,
  root: IDCompositionVisual2,
  fade: IDCompositionEffectGroup,

  /// Holds the groups, between the copies of windows beneath them and of
  /// topmost ones above them.
  groups_parent: IDCompositionVisual2,
  stills: Vec<WindowCopy>,
  groups: Vec<Group>,
  next_group: u32,
}

impl NativeScene {
  /// Whether this Windows build has the private exports scenes are built
  /// from.
  #[must_use]
  pub fn is_supported() -> bool {
    DwmPrivate::get().is_some()
  }

  /// Creates the device scenes are built with, which the first scene
  /// otherwise waits for.
  pub fn prepare() -> crate::Result<()> {
    SceneDevice::shared().map(|_| ())
  }

  /// Covers `monitor` with copies of every window shown on it, those of
  /// `group` (and the windows they own) in a group of their own, and
  /// returns once the cover is on screen.
  ///
  /// Blocks for two compositions: one drawing the copies faintly, one
  /// showing them. Fails if a window of `group` can't be copied, or no
  /// opaque copy covers the monitor to hide the real windows beneath.
  pub fn cover(
    monitor: &Rect,
    group: &[SceneWindow],
  ) -> crate::Result<(Self, SceneGroupId)> {
    let device = SceneDevice::shared()?;

    let built = Self::build(device, monitor).and_then(|mut scene| {
      let id = scene.cover_with(group)?;
      Ok((scene, id))
    });

    if built.is_err() {
      SceneDevice::discard();
    }

    built
  }

  fn build(
    device: Rc<SceneDevice>,
    monitor: &Rect,
  ) -> crate::Result<Self> {
    let host = Host::create(&device, monitor)?;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    let (root, fade, groups_parent) = unsafe {
      let root = device.dcomp.CreateVisual()?;
      let fade = device.dcomp.CreateEffectGroup()?;
      fade.SetOpacity2(WARM_UP_OPACITY)?;
      root.SetEffect(&fade)?;
      (root, fade, device.dcomp.CreateVisual()?)
    };

    Ok(Self {
      device,
      host,
      monitor: monitor.clone(),
      root,
      fade,
      groups_parent,
      stills: Vec::new(),
      groups: Vec::new(),
      next_group: 0,
    })
  }

  fn cover_with(
    &mut self,
    group: &[SceneWindow],
  ) -> crate::Result<SceneGroupId> {
    let started = Instant::now();
    let order = top_level_windows();
    let members = self.resolve(group, &order);

    let mut below = Vec::new();
    let mut above = Vec::new();
    for &hwnd in &order {
      if hwnd == self.host.hwnd
        || members.iter().any(|member| member.draws(hwnd))
        || !self.is_still(hwnd)
      {
        continue;
      }

      if window_class::is_topmost(hwnd) {
        above.push(hwnd);
      } else {
        below.push(hwnd);
      }
    }

    // Bottom-most first, each added above the ones before it.
    let mut has_floor = false;
    for hwnd in below.iter().rev() {
      match self.copy(&SceneWindow::as_is(*hwnd), &self.root.clone()) {
        Ok(Some((copy, covers_monitor))) => {
          has_floor |= covers_monitor;
          self.stills.push(copy);
        }
        Ok(None) => {}
        Err(err) => tracing::debug!(
          "Scene left out {}: {err}",
          window_class::describe(*hwnd)
        ),
      }
    }

    if !has_floor {
      return Err(crate::Error::Platform(
        "No opaque window covers the monitor.".to_string(),
      ));
    }

    // SAFETY: Plain `DirectComposition` call on live visuals.
    unsafe { self.root.AddVisual(&self.groups_parent, false, None) }?;

    for hwnd in above.iter().rev() {
      match self.copy(&SceneWindow::as_is(*hwnd), &self.root.clone()) {
        Ok(Some((copy, _))) => self.stills.push(copy),
        Ok(None) => {}
        Err(err) => tracing::debug!(
          "Scene left out {}: {err}",
          window_class::describe(*hwnd)
        ),
      }
    }

    let id = self.add_resolved_group(&members, &order)?;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    unsafe {
      self.host.target.SetRoot(&self.root)?;
      self.device.dcomp.Commit()?;
      self.device.dcomp.WaitForCommitCompletion()?;
    }
    let built = Instant::now();

    self.host.show()?;
    crate::dwm_flush();

    // SAFETY: Plain `DirectComposition` calls on live objects.
    unsafe {
      self.fade.SetOpacity2(1.0)?;
      self.device.dcomp.Commit()?;
    }
    crate::dwm_flush();

    tracing::debug!(
      "Scene covered {} windows in {:?} ({:?} building).",
      self.stills.len() + members.len(),
      started.elapsed(),
      built - started,
    );

    Ok(id)
  }

  /// Adds a group of `windows` (and the windows they own) above the other
  /// groups, posed at rest until [`pose`](Self::pose)d.
  ///
  /// Fails, adding nothing, if a required one of `windows` can't be
  /// copied.
  pub fn add_group(
    &mut self,
    windows: &[SceneWindow],
  ) -> crate::Result<SceneGroupId> {
    let order = top_level_windows();
    let members = self.resolve(windows, &order);
    self.add_resolved_group(&members, &order)
  }

  /// Puts the scene back above windows raised over it since it was shown,
  /// e.g. ones shown on top beneath it, which some apps do a while later.
  pub fn raise(&self) -> crate::Result<()> {
    if self.host.is_overlapped() {
      self.host.to_top(SWP_NOOWNERZORDER)?;
    }
    Ok(())
  }

  /// Poses group `id`, from the next commit.
  pub fn pose(
    &mut self,
    id: SceneGroupId,
    pose: &ScenePose,
  ) -> crate::Result<()> {
    let device = self.device.clone();
    let Some(group) = self.groups.iter().find(|group| group.id == id)
    else {
      return Ok(());
    };

    let copy_scale = Settable::new(&device, &pose.copy_scale)?;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    unsafe {
      Settable::new(&device, &pose.offset_x)?.set(
        |value| group.panel.SetOffsetX2(value),
        |animation| group.panel.SetOffsetX(animation),
      )?;
      Settable::new(&device, &pose.offset_y)?.set(
        |value| group.panel.SetOffsetY2(value),
        |animation| group.panel.SetOffsetY(animation),
      )?;
      Settable::new(&device, &pose.opacity)?.set(
        |value| group.effect.SetOpacity2(value),
        |animation| group.effect.SetOpacity(animation),
      )?;
      Settable::new(&device, &pose.scale)?.set_scale(&group.scale)?;

      for copy in &group.copies {
        copy_scale.set_scale(&copy.scale)?;

        if let Some(backdrop) = &copy.backdrop {
          let back = -backdrop.parallax;
          let (x, y) = backdrop.rest;
          Settable::new(&device, &pose.offset_x.mapped(back, x))?.set(
            |value| backdrop.blur.SetOffsetX2(value),
            |animation| backdrop.blur.SetOffsetX(animation),
          )?;
          Settable::new(&device, &pose.offset_y.mapped(back, y))?.set(
            |value| backdrop.blur.SetOffsetY2(value),
            |animation| backdrop.blur.SetOffsetY(animation),
          )?;
        }
      }
    }

    Ok(())
  }

  /// Removes group `id`'s copies, from the next commit.
  pub fn remove_group(&mut self, id: SceneGroupId) {
    let Some(index) = self.groups.iter().position(|group| group.id == id)
    else {
      return;
    };

    let group = self.groups.remove(index);
    // SAFETY: Plain `DirectComposition` call on live visuals.
    let _ = unsafe { self.groups_parent.RemoveVisual(&group.panel) };
    for copy in &group.copies {
      copy.unregister();
    }
  }

  /// Sets the whole scene's opacity, e.g. to fade it off the windows it
  /// matches, from the next commit.
  pub fn set_opacity(
    &mut self,
    opacity: &SceneValue,
  ) -> crate::Result<()> {
    let opacity = Settable::new(&self.device, opacity)?;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    unsafe {
      opacity.set(
        |value| self.fade.SetOpacity2(value),
        |animation| self.fade.SetOpacity(animation),
      )
    }
  }

  /// Puts every change since the last commit on screen together.
  pub fn commit(&self) -> crate::Result<()> {
    // SAFETY: Plain `DirectComposition` call on a live device.
    let committed = unsafe { self.device.dcomp.Commit() };
    if committed.is_err() {
      SceneDevice::discard();
    }
    Ok(committed?)
  }

  /// The windows a scene group of `windows` copies: `windows`, then the
  /// shown windows of `order` they own or that are their companions.
  fn resolve(
    &self,
    windows: &[SceneWindow],
    order: &[HWND],
  ) -> Vec<SceneWindow> {
    let explicit =
      windows.iter().map(|w| w.hwnd.0).collect::<HashSet<_>>();
    let mut members = windows.to_vec();

    for &hwnd in order {
      if hwnd == self.host.hwnd
        || windows.iter().any(|window| window.draws(hwnd))
      {
        continue;
      }

      // SAFETY: A stale handle just reads as hidden.
      let is_shown = unsafe { IsWindowVisible(hwnd) }.as_bool();

      if is_shown
        && goes_with(hwnd, &explicit)
        && !NativeWindow::new(hwnd.0).is_cloaked().unwrap_or(true)
      {
        members.push(SceneWindow::as_is(hwnd));
      }
    }

    members
  }

  /// Adds a group of `members`, stacked as their windows are in `order`.
  /// The required ones must be copied, unless not shown on the monitor.
  fn add_resolved_group(
    &mut self,
    members: &[SceneWindow],
    order: &[HWND],
  ) -> crate::Result<SceneGroupId> {
    let device = self.device.clone();
    let id = SceneGroupId(self.next_group);
    self.next_group += 1;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    let group = unsafe {
      let panel = device.dcomp.CreateVisual()?;
      let effect = device.dcomp.CreateEffectGroup()?;
      panel.SetEffect(&effect)?;

      let scale = device.dcomp.CreateScaleTransform()?;
      #[allow(clippy::cast_precision_loss)]
      {
        scale.SetCenterX2(self.monitor.width() as f32 / 2.0)?;
        scale.SetCenterY2(self.monitor.height() as f32 / 2.0)?;
      }
      let content = device.dcomp.CreateVisual()?;
      content.SetTransform(&scale)?;
      panel.AddVisual(&content, false, None)?;

      Group {
        id,
        panel,
        effect,
        scale,
        content,
        copies: Vec::new(),
      }
    };

    let mut stacked = members.to_vec();
    // Bottom-most first, as each is added above the ones before it.
    // Windows gone meanwhile go first, and fail to copy.
    stacked.sort_by_key(|member| {
      std::cmp::Reverse(
        order
          .iter()
          .position(|hwnd| *hwnd == member.hwnd)
          .unwrap_or(usize::MAX),
      )
    });

    let mut copies = Vec::new();
    for member in &stacked {
      match self.copy(member, &group.content) {
        Ok(Some((copy, _))) => copies.push(copy),
        Ok(None) => {}
        Err(err) if member.required => {
          for copy in &copies {
            copy.unregister();
          }
          return Err(crate::Error::Platform(format!(
            "Can't copy {}: {err}",
            window_class::describe(member.hwnd)
          )));
        }
        Err(err) => tracing::debug!(
          "Scene left out {}: {err}",
          window_class::describe(member.hwnd)
        ),
      }
    }

    // SAFETY: Plain `DirectComposition` call on live visuals.
    unsafe { self.groups_parent.AddVisual(&group.panel, false, None) }?;

    self.groups.push(Group { copies, ..group });
    Ok(id)
  }

  /// Copies `window` above `parent`'s other children, cut to the part of
  /// its visible frame on the monitor. Returns the copy, with whether it
  /// opaquely covers the whole monitor, or `None` if it isn't shown on
  /// the monitor.
  fn copy(
    &self,
    window: &SceneWindow,
    parent: &IDCompositionVisual2,
  ) -> crate::Result<Option<(WindowCopy, bool)>> {
    // A window the caller places is copied even while hidden, e.g. a tab
    // bar its thread hasn't shown yet: its copy shows once it is. A
    // cloaked one still counts as shown, as DWM keeps drawing it for
    // thumbnails. SAFETY: A stale handle just reads as hidden.
    if window.placement.is_none()
      && !unsafe { IsWindowVisible(window.hwnd) }.as_bool()
    {
      return Ok(None);
    }

    let Some((outer, frame)) = window
      .placement
      .clone()
      .or_else(|| placement_of(window.hwnd))
    else {
      return Ok(None);
    };

    let Some(shown) = frame.intersection(&self.monitor) else {
      return Ok(None);
    };

    // Thumbnails leave out the opacity a window is layered with.
    let alpha = window
      .alpha
      .or_else(|| {
        NativeWindow::new(window.hwnd.0)
          .layered_alpha()
          .ok()
          .flatten()
      })
      .unwrap_or(u8::MAX);

    let (content, thumbnail) = self.thumbnail(window.hwnd, alpha)?;

    // Without its backdrop, the copy is still worth showing.
    let backdrop = window.backdrop.and_then(|backdrop| {
      self
        .backdrop(&backdrop.params, &outer, &frame)
        .inspect_err(|err| {
          tracing::debug!("Scene left out a backdrop: {err}");
        })
        .ok()
    });

    // SAFETY: Plain `DirectComposition` calls on live objects.
    let placed = unsafe {
      self.place(
        window,
        &content,
        backdrop.as_ref(),
        &outer,
        &frame,
        &shown,
        parent,
      )
    };

    let copy = WindowCopy {
      thumbnail,
      scale: match placed {
        Ok(scale) => scale,
        Err(err) => {
          // SAFETY: Registered above and not handed out.
          let _ = unsafe { DwmUnregisterThumbnail(thumbnail) };
          if let Some(backdrop) = &backdrop {
            backdrop.unregister();
          }
          return Err(err);
        }
      },
      backdrop,
    };

    let covers_monitor =
      alpha == u8::MAX && frame.contains_rect(&self.monitor);
    Ok(Some((copy, covers_monitor)))
  }

  /// A thumbnail of all of `source` at `alpha`, with its handle.
  fn thumbnail(
    &self,
    source: HWND,
    alpha: u8,
  ) -> crate::Result<(IDCompositionVisual, isize)> {
    let mut size = SIZE::default();
    // SAFETY: `size` outlives the call.
    unsafe {
      (self.device.exports.query_source_size)(
        source,
        false.into(),
        &raw mut size,
      )
    }
    .ok()?;

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
      opacity: alpha,
      fVisible: true.into(),
      fSourceClientAreaOnly: false.into(),
    };

    let mut raw = null_mut();
    let mut thumbnail = 0;
    // SAFETY: Out pointers and `properties` outlive the call, and the
    // device is live.
    unsafe {
      (self.device.exports.create_thumbnail_visual)(
        self.host.hwnd,
        source,
        SHARED_THUMBNAIL_FLAGS,
        &raw const properties,
        self.device.dcomp.as_raw(),
        &raw mut raw,
        &raw mut thumbnail,
      )
    }
    .ok()?;

    // SAFETY: On success, `raw` is an owned `IDCompositionVisual`.
    Ok((unsafe { IDCompositionVisual::from_raw(raw) }, thumbnail))
  }

  /// The backdrop `params` draw beneath a window at `outer`, of visible
  /// frame `frame`, in its copy's coordinates.
  #[allow(clippy::cast_precision_loss)]
  fn backdrop(
    &self,
    params: &BackdropOverlayParams,
    outer: &Rect,
    frame: &Rect,
  ) -> crate::Result<BackdropCopy> {
    let canvas = backdrop_canvas::canvas_for(&self.monitor, params)?;
    let (content, thumbnail) = self.thumbnail(canvas, u8::MAX)?;

    // Where the overlay's crop puts the image (see `set_crop`): pinned to
    // the desktop at a parallax of 1, trailing the window below it.
    let parallax = params.parallax;
    let at = |monitor: i32, frame: i32, outer: i32| {
      parallax * monitor as f32 + (1.0 - parallax) * frame as f32
        - outer as f32
    };
    let rest = (
      at(self.monitor.left, frame.left, outer.left),
      at(self.monitor.top, frame.top, outer.top),
    );

    let dcomp = &self.device.dcomp;
    // SAFETY: Plain `DirectComposition` calls on live objects of this
    // scene's device.
    let built = unsafe {
      (|| -> crate::Result<_> {
        let blur = dcomp.CreateVisual()?;
        blur.SetOffsetX2(rest.0)?;
        blur.SetOffsetY2(rest.1)?;
        blur.AddVisual(&content, false, None)?;

        let side = VIGNETTE_SIZE as f32;
        let vignette = dcomp.CreateVisual()?;
        vignette.SetContent(&self.device.vignette()?)?;
        vignette.SetBitmapInterpolationMode(
          DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
        )?;
        vignette.SetTransform2(&Matrix3x2 {
          M11: frame.width() as f32 / side,
          M12: 0.0,
          M21: 0.0,
          M22: frame.height() as f32 / side,
          M31: 0.0,
          M32: 0.0,
        })?;
        vignette.SetOffsetX2((frame.left - outer.left) as f32)?;
        vignette.SetOffsetY2((frame.top - outer.top) as f32)?;
        let strength = dcomp.CreateEffectGroup()?;
        strength.SetOpacity2(params.vignette.clamp(0.0, 1.0))?;
        vignette.SetEffect(&strength)?;

        let visual = dcomp.CreateVisual()?;
        let opacity = dcomp.CreateEffectGroup()?;
        opacity.SetOpacity2(params.opacity.clamp(0.0, 1.0))?;
        visual.SetEffect(&opacity)?;
        visual.AddVisual(&blur, false, None)?;
        visual.AddVisual(&vignette, false, None)?;

        Ok((visual, blur))
      })()
    };

    match built {
      Ok((visual, blur)) => Ok(BackdropCopy {
        thumbnail,
        visual,
        blur,
        rest,
        parallax,
      }),
      Err(err) => {
        // SAFETY: Registered above and not handed out.
        let _ = unsafe { DwmUnregisterThumbnail(thumbnail) };
        Err(err)
      }
    }
  }

  /// Builds the visuals placing `content`, a thumbnail of `window`, over
  /// the monitor, on top of its `backdrop`, and adds them above `parent`'s
  /// other children.
  ///
  /// # Safety
  ///
  /// Every object must be live and of this scene's device.
  #[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
  unsafe fn place(
    &self,
    window: &SceneWindow,
    content: &IDCompositionVisual,
    backdrop: Option<&BackdropCopy>,
    outer: &Rect,
    frame: &Rect,
    shown: &Rect,
    parent: &IDCompositionVisual2,
  ) -> crate::Result<IDCompositionScaleTransform> {
    let dcomp = &self.device.dcomp;

    // DWM draws a thumbnail from its window rect's corner, past the
    // invisible resize border, so the copy is anchored there.
    let visual = dcomp.CreateVisual()?;
    visual.SetOffsetX2((outer.left - self.monitor.left) as f32)?;
    visual.SetOffsetY2((outer.top - self.monitor.top) as f32)?;

    let scale = dcomp.CreateScaleTransform()?;
    scale.SetCenterX2(
      (i32::midpoint(frame.left, frame.right) - outer.left) as f32,
    )?;
    scale.SetCenterY2(
      (i32::midpoint(frame.top, frame.bottom) - outer.top) as f32,
    )?;
    let scaled = dcomp.CreateVisual()?;
    scaled.SetTransform(&scale)?;

    // Cut to what DWM draws of the window, rounded as it is, except where
    // the monitor's edge cuts it straight: a window hanging off the
    // monitor mustn't bring its hidden part into view as it moves.
    let clip = dcomp.CreateRectangleClip()?;
    clip.SetLeft2((shown.left - outer.left) as f32)?;
    clip.SetTop2((shown.top - outer.top) as f32)?;
    clip.SetRight2((shown.right - outer.left) as f32)?;
    clip.SetBottom2((shown.bottom - outer.top) as f32)?;

    let radius = window.corner_radius;
    if radius > 0.0 {
      let left = shown.left == frame.left;
      let top = shown.top == frame.top;
      let right = shown.right == frame.right;
      let bottom = shown.bottom == frame.bottom;
      let corner = |is_own: bool| if is_own { radius } else { 0.0 };

      clip.SetTopLeftRadiusX2(corner(left && top))?;
      clip.SetTopLeftRadiusY2(corner(left && top))?;
      clip.SetTopRightRadiusX2(corner(right && top))?;
      clip.SetTopRightRadiusY2(corner(right && top))?;
      clip.SetBottomLeftRadiusX2(corner(left && bottom))?;
      clip.SetBottomLeftRadiusY2(corner(left && bottom))?;
      clip.SetBottomRightRadiusX2(corner(right && bottom))?;
      clip.SetBottomRightRadiusY2(corner(right && bottom))?;
    }

    let clipped = dcomp.CreateVisual()?;
    clipped.SetClip(&clip)?;
    if let Some(backdrop) = backdrop {
      clipped.AddVisual(&backdrop.visual, false, None)?;
    }
    clipped.AddVisual(content, false, None)?;
    scaled.AddVisual(&clipped, false, None)?;
    visual.AddVisual(&scaled, false, None)?;
    parent.AddVisual(&visual, false, None)?;

    Ok(scale)
  }

  /// Whether `hwnd` is shown on the monitor, to be copied as is beneath or
  /// above the groups.
  fn is_still(&self, hwnd: HWND) -> bool {
    // SAFETY: Read-only queries; a stale handle just reads as hidden.
    let is_shown =
      unsafe {
        IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool()
      } && !NativeWindow::new(hwnd.0).is_cloaked().unwrap_or(true);

    if !is_shown {
      return false;
    }

    let class = class_name(hwnd);
    if class.starts_with(OWN_CLASS_PREFIX)
      && !COPIED_OWN_CLASSES.contains(&class.as_str())
    {
      return false;
    }

    let mut bounds = RECT::default();
    // SAFETY: `bounds` outlives the call.
    unsafe { GetWindowRect(hwnd, &raw mut bounds) }.is_ok()
      && Rect::from_ltrb(
        bounds.left,
        bounds.top,
        bounds.right,
        bounds.bottom,
      )
      .intersection(&self.monitor)
      .is_some()
  }
}

impl Drop for NativeScene {
  fn drop(&mut self) {
    // Unregistered before the host they draw in is destroyed.
    for copy in self
      .stills
      .iter()
      .chain(self.groups.iter().flat_map(|group| group.copies.iter()))
    {
      copy.unregister();
    }
  }
}

/// A [`SceneValue`] ready to set: fixed, or the animation function of its
/// motion.
enum Settable {
  Fixed(f32),
  Moving(IDCompositionAnimation),
}

impl Settable {
  fn new(device: &SceneDevice, value: &SceneValue) -> crate::Result<Self> {
    Ok(match value {
      SceneValue::Fixed(value) => Self::Fixed(*value),
      SceneValue::Moving(motion) => {
        Self::Moving(animation(device, motion)?)
      }
    })
  }

  /// Sets it through the setter for a fixed value or for an animation.
  ///
  /// # Safety
  ///
  /// The setters must be plain `DirectComposition` calls on live objects
  /// of the device it was made with.
  unsafe fn set(
    &self,
    fixed: impl FnOnce(f32) -> windows::core::Result<()>,
    moving: impl FnOnce(&IDCompositionAnimation) -> windows::core::Result<()>,
  ) -> crate::Result<()> {
    match self {
      Self::Fixed(value) => fixed(*value)?,
      Self::Moving(animation) => moving(animation)?,
    }

    Ok(())
  }

  /// Sets it as both of `transform`'s scales.
  ///
  /// # Safety
  ///
  /// `transform` must be live and of the device it was made with.
  unsafe fn set_scale(
    &self,
    transform: &IDCompositionScaleTransform,
  ) -> crate::Result<()> {
    self.set(
      |value| {
        transform.SetScaleX2(value)?;
        transform.SetScaleY2(value)
      },
      |animation| {
        transform.SetScaleX(animation)?;
        transform.SetScaleY(animation)
      },
    )
  }
}

/// `motion` as an animation function, sampled by DWM on its own clock.
fn animation(
  device: &SceneDevice,
  motion: &SceneMotion,
) -> crate::Result<IDCompositionAnimation> {
  // SAFETY: Plain `DirectComposition` calls on live objects.
  unsafe {
    let animation = device.dcomp.CreateAnimation()?;
    animation.SetAbsoluteBeginTime(qpc_at(motion.begins_at))?;

    for segment in &motion.segments {
      let [c0, c1, c2, c3] = segment.coefficients;
      animation.AddCubic(segment.begin_secs, c0, c1, c2, c3)?;
    }

    animation.End(motion.duration.as_secs_f64(), motion.end)?;
    Ok(animation)
  }
}

/// `instant` on the QPC clock `DirectComposition` animations are timed
/// in.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn qpc_at(instant: Instant) -> i64 {
  static FREQUENCY: OnceLock<i64> = OnceLock::new();
  let frequency = *FREQUENCY.get_or_init(|| {
    let mut frequency = 0;
    // SAFETY: `frequency` outlives the call; QPC can't fail on XP+.
    let _ = unsafe { QueryPerformanceFrequency(&raw mut frequency) };
    frequency.max(1)
  });

  let mut counter = 0;
  // SAFETY: `counter` outlives the call; QPC can't fail on XP+.
  let _ = unsafe { QueryPerformanceCounter(&raw mut counter) };
  let now = Instant::now();

  let ticks = |elapsed: Duration| {
    (elapsed.as_secs_f64() * frequency as f64).round() as i64
  };

  match instant.checked_duration_since(now) {
    Some(ahead) => counter + ticks(ahead),
    None => counter - ticks(now.duration_since(instant)),
  }
}

/// Waits for DWM's next composition.
/// Every top-level window, topmost first.
fn top_level_windows() -> Vec<HWND> {
  unsafe extern "system" fn collect(hwnd: HWND, data: LPARAM) -> BOOL {
    // SAFETY: `data` is the `Vec` passed to `EnumWindows` below, which
    // outlives the enumeration.
    unsafe { (*(data.0 as *mut Vec<HWND>)).push(hwnd) };
    TRUE
  }

  let mut handles: Vec<HWND> = Vec::new();
  // SAFETY: `collect` only dereferences `data` as the `Vec` above, which
  // lives until `EnumWindows` returns.
  let _ = unsafe {
    EnumWindows(
      Some(collect),
      LPARAM(std::ptr::from_mut(&mut handles) as isize),
    )
  };

  handles
}

/// Whether `hwnd` goes with one of `windows`: owned by one, or drawn as
/// its companion.
fn goes_with(hwnd: HWND, windows: &HashSet<isize>) -> bool {
  if companion::companion_of(hwnd)
    .is_some_and(|of| windows.contains(&of.0))
  {
    return true;
  }

  let mut current = hwnd;
  for _ in 0..MAX_OWNER_DEPTH {
    // SAFETY: A stale handle just returns `HWND(0)`.
    current = unsafe { GetWindow(current, GW_OWNER) };
    if current.0 == 0 {
      return false;
    }
    if windows.contains(&current.0) {
      return true;
    }
  }

  false
}

/// `hwnd`'s window rect and visible frame, the window rect standing in
/// for a frame DWM doesn't report (e.g. the desktop's).
fn placement_of(hwnd: HWND) -> Option<(Rect, Rect)> {
  let to_rect = |rect: RECT| {
    Rect::from_ltrb(rect.left, rect.top, rect.right, rect.bottom)
  };

  if let Some((bounds, frame)) = window_class::bounds_and_frame(hwnd) {
    return Some((to_rect(bounds), to_rect(frame)));
  }

  let mut bounds = RECT::default();
  // SAFETY: `bounds` outlives the call.
  unsafe { GetWindowRect(hwnd, &raw mut bounds) }.ok()?;
  Some((to_rect(bounds), to_rect(bounds)))
}

/// Strength of the vignette at `(u, v)` across its square, from 0 to 1,
/// as `build_vignette_brush`'s gradient has it.
fn vignette_alpha(u: f64, v: f64) -> f64 {
  let radius = f64::from(VIGNETTE_RADIUS);
  let clear = f64::from(VIGNETTE_CLEAR);
  let reach = ((u - 0.5) / radius).hypot((v - 0.5) / radius);

  ((reach - clear) / (1.0 - clear)).clamp(0.0, 1.0)
}

fn class_name(hwnd: HWND) -> String {
  let mut buf = [0u16; 64];
  // SAFETY: `buf` outlives the call; a stale handle just returns 0.
  let len = unsafe { GetClassNameW(hwnd, &mut buf) };
  String::from_utf16_lossy(
    buf
      .get(..usize::try_from(len).unwrap_or(0))
      .unwrap_or_default(),
  )
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use super::{vignette_alpha, SceneMotion, SceneSegment, SceneValue};

  #[test]
  fn vignette_is_clear_inside_and_darkest_in_the_corners() {
    assert!(vignette_alpha(0.5, 0.5).abs() < f64::EPSILON);
    assert!(vignette_alpha(0.6, 0.5).abs() < f64::EPSILON);

    // At the edges' middles and corners, as the brush's 0.75 radius
    // ellipse with its first 45% clear puts them.
    assert!((vignette_alpha(0.0, 0.5) - 0.394).abs() < 0.001);
    assert!((vignette_alpha(0.0, 0.0) - 0.896).abs() < 0.001);
    assert!(vignette_alpha(0.0, 0.0) < 1.0);
  }

  #[test]
  fn mapped_motion_scales_and_shifts_every_moment() {
    let motion = SceneValue::Moving(SceneMotion {
      begins_at: Instant::now(),
      duration: Duration::from_millis(100),
      segments: vec![SceneSegment {
        begin_secs: 0.0,
        coefficients: [10.0, 2.0, 3.0, 4.0],
      }],
      end: 20.0,
    });

    let SceneValue::Moving(mapped) = motion.mapped(-0.5, 100.0) else {
      panic!("a motion stays moving");
    };
    assert_eq!(mapped.segments[0].coefficients, [95.0, -1.0, -1.5, -2.0]);
    assert!((mapped.end - 90.0).abs() < f32::EPSILON);

    let SceneValue::Fixed(value) = SceneValue::Fixed(4.0).mapped(2.0, 1.0)
    else {
      panic!("a fixed value stays fixed");
    };
    assert!((value - 9.0).abs() < f32::EPSILON);
  }
}
