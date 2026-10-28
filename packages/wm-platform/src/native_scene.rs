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
//! Copies are grouped, e.g. by workspace, and within a group by item: a
//! window with its border, backdrop and tab bar, which moves and resizes
//! on its own. A resizing copy is stretched, as Hyprland stretches a
//! window's surface, but keeps its corners round, its border's width and
//! its backdrop in place.
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
        IDCompositionRectangleClip, IDCompositionScaleTransform,
        IDCompositionSurface, IDCompositionTarget, IDCompositionVisual,
        IDCompositionVisual2,
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
/// windows. Its others stand in for windows (peeks) or cover the screen
/// (scenes, the overview), and are left out.
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
#[derive(Clone, Debug, PartialEq)]
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

  pub fit: SceneFit,
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
      fit: SceneFit::Whole,
    }
  }
}

/// How a copy covers its placement, and follows its item's frame as that
/// resizes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneFit {
  /// At its window's own size, whatever size it reaches, from its
  /// placement's corner, and cut to its frame as that resizes, e.g. for a
  /// window whose app redraws it at its new size beneath the scene.
  Whole,

  /// Its whole window, one without invisible resize borders, stretched
  /// over its placement and cut in nine at these insets, in pixels: each
  /// edge keeps to the frame's nearest edge, the corners keep their size,
  /// the edges stretch along their length, and the middle is left out,
  /// e.g. for a border ring.
  Frame {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
  },
}

/// A window's backdrop as its overlay draws it, but from the blurred
/// wallpaper of the whole monitor, which stays put as the window moves: a
/// copy of the overlay would carry its crop of it along.
#[derive(Clone, Copy, Debug, PartialEq)]
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
      Self::Moving(_) if scale.abs() < f32::EPSILON => Self::Fixed(add),
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

  /// Scale of each of its items, about its frame's center.
  pub item_scale: SceneValue,
}

/// A window and what is drawn with it, whose copies move as one within
/// their group.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneItem {
  /// Names the item within its group, e.g. after its window's ID.
  pub key: u128,

  /// Its window's visible frame at rest, which its own motion moves and
  /// scales.
  pub frame: Rect,

  /// The window, then what's drawn with it, e.g. its border. Windows they
  /// own come along, staying put as the item moves.
  pub windows: Vec<SceneWindow>,
}

/// Where an item is drawn relative to its frame at rest: offset in
/// pixels, scale of the frame about its top-left corner, zoom of the whole
/// item about the frame's center, and opacity.
#[derive(Clone, Debug)]
pub struct ItemPose {
  pub offset_x: SceneValue,
  pub offset_y: SceneValue,
  pub scale_x: SceneValue,
  pub scale_y: SceneValue,
  pub zoom: SceneValue,
  pub opacity: SceneValue,
}

/// A live copy of a window in a scene.
struct WindowCopy {
  /// Its thumbnails: one, or one per piece of a frame.
  thumbnails: Vec<isize>,
  backdrop: Option<BackdropCopy>,
  shape: Shape,
}

impl WindowCopy {
  fn unregister(&self) {
    for thumbnail in &self.thumbnails {
      // SAFETY: The handle came from DWM and is released once, here.
      let _ = unsafe { DwmUnregisterThumbnail(*thumbnail) };
    }
    if let Some(backdrop) = &self.backdrop {
      backdrop.unregister();
    }
  }
}

/// How a thumbnail maps its window.
#[derive(Clone, Copy)]
enum Mapping {
  /// At its own size, up to this one as it grows, e.g. while its app
  /// resizes it beneath a scene.
  Whole(i32, i32),

  /// All of it stretched over this size, for a window without invisible
  /// resize borders: their pixels would be stretched in too.
  Stretched(i32, i32),
}

/// How a copy follows its item's frame as that resizes.
enum Shape {
  /// At its own size, with its clip's far edges keeping to the frame's.
  Whole {
    /// Scales its backdrop's vignette over the frame, if it has one.
    vignette: Option<IDCompositionScaleTransform>,
    clip: IDCompositionRectangleClip,

    /// The clip's right and bottom edges at rest, where they keep to the
    /// item's frame.
    right: Option<f32>,
    bottom: Option<f32>,
  },

  /// The pieces of a [`SceneFit::Frame`].
  Pieces(Vec<Piece>),
}

/// A piece of a frame copy, moved and stretched along with the frame.
struct Piece {
  visual: IDCompositionVisual2,
  scale: IDCompositionScaleTransform,
  x: Span,
  y: Span,
}

impl Piece {
  /// Keeps it to its item's frame, `width` by `height` at rest, as that is
  /// scaled by `scale_x` and `scale_y`.
  ///
  /// # Safety
  ///
  /// Its objects must be live and of `device`.
  unsafe fn pose(
    &self,
    device: &SceneDevice,
    scale_x: &SceneValue,
    scale_y: &SceneValue,
    width: f32,
    height: f32,
  ) -> crate::Result<()> {
    Settable::new(device, &self.x.offset(scale_x, width))?.set(
      |value| self.visual.SetOffsetX2(value),
      |animation| self.visual.SetOffsetX(animation),
    )?;
    Settable::new(device, &self.y.offset(scale_y, height))?.set(
      |value| self.visual.SetOffsetY2(value),
      |animation| self.visual.SetOffsetY(animation),
    )?;
    Settable::new(device, &self.x.stretch(scale_x, width))?.set(
      |value| self.scale.SetScaleX2(value),
      |animation| self.scale.SetScaleX(animation),
    )?;
    Settable::new(device, &self.y.stretch(scale_y, height))?.set(
      |value| self.scale.SetScaleY2(value),
      |animation| self.scale.SetScaleY(animation),
    )
  }
}

/// Where a piece of a frame copy lies along one axis, and how it follows
/// the frame growing by `d` pixels along it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Span {
  /// Start at rest, in the scene's coordinates.
  start: f32,
  length: f32,

  /// Shares of `d` its start moves by, and its length grows by.
  moves: f32,
  grows: f32,
}

impl Span {
  /// Its start as the frame, `size` long at rest, is scaled by `scale`.
  fn offset(&self, scale: &SceneValue, size: f32) -> SceneValue {
    scale.mapped(self.moves * size, self.start - self.moves * size)
  }

  /// Its stretch as the frame, `size` long at rest, is scaled by `scale`.
  fn stretch(&self, scale: &SceneValue, size: f32) -> SceneValue {
    let grow = self.grows * size / self.length;
    scale.mapped(grow, 1.0 - grow)
  }
}

/// A window's backdrop in its copy: a vignette over the monitor's blurred
/// wallpaper, which the motions of its item and group move the other way,
/// so it stays put on screen (see [`SceneWindow::backdrop`]).
struct BackdropCopy {
  /// Its thumbnail of the monitor's backdrop canvas.
  thumbnail: isize,

  /// Holds the backdrop beneath the window, under its opacity.
  visual: IDCompositionVisual2,

  /// Holds `blur`, offset against its item's motion.
  item_back: IDCompositionVisual2,

  /// Holds the canvas's thumbnail, offset against its group's motion.
  blur: IDCompositionVisual2,

  /// Offset of `blur` with the group at rest, in the copy's coordinates.
  rest: (f32, f32),

  /// Share of the motions it is moved back by.
  parallax: f32,
}

impl BackdropCopy {
  fn unregister(&self) {
    // SAFETY: The handle came from DWM and is released once, here.
    let _ = unsafe { DwmUnregisterThumbnail(self.thumbnail) };
  }
}

/// A window's copies in a group, moved, scaled and faded as one.
struct Item {
  key: u128,

  /// Its window, which it is stacked by.
  anchor: HWND,

  /// Its frame at rest, in the scene's coordinates, which its pose is
  /// relative to.
  frame: Rect,

  /// Moved by the item's offset and faded by `effect`; holds the rest.
  visual: IDCompositionVisual2,
  effect: IDCompositionEffectGroup,

  /// Scale the item about its frame's center: for its group's
  /// `item_scale`, then for its own zoom.
  zoom: IDCompositionScaleTransform,
  own_zoom: IDCompositionScaleTransform,
  copies: Vec<WindowCopy>,

  /// The windows it owns, just above it.
  pinned: Option<Pinned>,
}

impl Item {
  fn unregister(&self) {
    let pinned = self.pinned.iter().flat_map(|pinned| &pinned.copies);
    for copy in self.copies.iter().chain(pinned) {
      copy.unregister();
    }
  }

  /// Its topmost visual.
  fn top(&self) -> &IDCompositionVisual2 {
    self
      .pinned
      .as_ref()
      .map_or(&self.visual, |pinned| &pinned.visual)
  }

  /// Adds its visuals to `content`, just above `reference`, or above
  /// everything without one.
  ///
  /// # Safety
  ///
  /// Every object must be live and of the same device.
  unsafe fn insert(
    &self,
    content: &IDCompositionVisual2,
    reference: Option<&IDCompositionVisual2>,
  ) -> windows::core::Result<()> {
    match reference {
      Some(reference) => {
        content.AddVisual(&self.visual, true, reference)?;
      }
      None => content.AddVisual(&self.visual, false, None)?,
    }
    if let Some(pinned) = &self.pinned {
      content.AddVisual(&pinned.visual, true, &self.visual)?;
    }
    Ok(())
  }

  /// Takes its visuals out of `content`.
  ///
  /// # Safety
  ///
  /// Every object must be live and of the same device.
  unsafe fn detach(&self, content: &IDCompositionVisual2) {
    let _ = content.RemoveVisual(&self.visual);
    if let Some(pinned) = &self.pinned {
      let _ = content.RemoveVisual(&pinned.visual);
    }
  }
}

/// Copies of the windows an item's windows own, which stay put as it
/// moves but fade with it.
struct Pinned {
  visual: IDCompositionVisual2,
  effect: IDCompositionEffectGroup,

  /// Scales them about the item's frame's center, for its group's
  /// `item_scale`.
  zoom: IDCompositionScaleTransform,
  copies: Vec<WindowCopy>,
}

/// The windows an item copies.
struct Members {
  /// Its windows, then those drawn as their companions, which move with
  /// them.
  moving: Vec<SceneWindow>,

  /// Windows they own, which stay put.
  pinned: Vec<SceneWindow>,
}

impl Members {
  fn draws(&self, hwnd: HWND) -> bool {
    self
      .moving
      .iter()
      .chain(&self.pinned)
      .any(|member| member.draws(hwnd))
  }

  fn len(&self) -> usize {
    self.moving.len() + self.pinned.len()
  }
}

struct Group {
  id: SceneGroupId,

  /// Moved by the group's offset; holds the rest.
  panel: IDCompositionVisual2,
  effect: IDCompositionEffectGroup,

  /// Scales the group about the monitor's center.
  scale: IDCompositionScaleTransform,

  /// Holds the items, bottom-most first, under `scale` and cut to the
  /// monitor as the group rests, so its motion brings nothing beyond the
  /// monitor's edge into view.
  content: IDCompositionVisual2,
  items: Vec<Item>,
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

  /// Covers `monitor` with copies of every window shown on it but those
  /// `left_out`, those of `items` in a group of their own, and returns
  /// once the cover is on screen.
  ///
  /// Blocks for two compositions: one drawing the copies faintly, one
  /// showing them. Fails if a required window of `items` can't be copied,
  /// or no opaque copy covers the monitor to hide the real windows
  /// beneath.
  pub fn cover(
    monitor: &Rect,
    items: &[SceneItem],
    left_out: &[HWND],
  ) -> crate::Result<(Self, SceneGroupId)> {
    let device = SceneDevice::shared()?;

    let built = Self::build(device, monitor).and_then(|mut scene| {
      let id = scene.cover_with(items, left_out)?;
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
    items: &[SceneItem],
    left_out: &[HWND],
  ) -> crate::Result<SceneGroupId> {
    let started = Instant::now();
    let order = top_level_windows();
    let members = items
      .iter()
      .map(|item| self.resolve(&item.windows, &order))
      .collect::<Vec<_>>();

    let mut below = Vec::new();
    let mut above = Vec::new();
    for &hwnd in &order {
      if hwnd == self.host.hwnd
        || left_out.contains(&hwnd)
        || members.iter().any(|members| members.draws(hwnd))
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
      let window = SceneWindow::as_is(*hwnd);
      match self.copy(&window, None, &self.root.clone()) {
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
      let window = SceneWindow::as_is(*hwnd);
      match self.copy(&window, None, &self.root.clone()) {
        Ok(Some((copy, _))) => self.stills.push(copy),
        Ok(None) => {}
        Err(err) => tracing::debug!(
          "Scene left out {}: {err}",
          window_class::describe(*hwnd)
        ),
      }
    }

    let id = self.add_resolved_group(items, &members, &order)?;

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
      self.stills.len() + members.iter().map(Members::len).sum::<usize>(),
      started.elapsed(),
      built - started,
    );

    Ok(id)
  }

  /// Adds a group of `items` above the other groups, posed at rest until
  /// [`pose`](Self::pose)d.
  ///
  /// Fails, adding nothing, if a required window of `items` can't be
  /// copied.
  pub fn add_group(
    &mut self,
    items: &[SceneItem],
  ) -> crate::Result<SceneGroupId> {
    let order = top_level_windows();
    let members = items
      .iter()
      .map(|item| self.resolve(&item.windows, &order))
      .collect::<Vec<_>>();
    self.add_resolved_group(items, &members, &order)
  }

  /// Puts `item` in group `id` in place of its item of the same key, or
  /// above its other items if it has none, at rest until
  /// [`pose_item`](Self::pose_item)d.
  ///
  /// Fails, changing nothing, if a required window of `item` can't be
  /// copied.
  pub fn set_item(
    &mut self,
    id: SceneGroupId,
    item: &SceneItem,
  ) -> crate::Result<()> {
    let order = top_level_windows();
    let members = self.resolve(&item.windows, &order);
    let built = self.build_item(item, &members, &order)?;

    let Some(group) = self.groups.iter_mut().find(|group| group.id == id)
    else {
      built.unregister();
      return Ok(());
    };

    let index = group.items.iter().position(|old| old.key == item.key);
    let reference = index.map(|index| group.items[index].top().clone());

    // SAFETY: Plain `DirectComposition` calls on live visuals.
    if let Err(err) =
      unsafe { built.insert(&group.content, reference.as_ref()) }
    {
      // SAFETY: As above.
      unsafe { built.detach(&group.content) };
      built.unregister();
      return Err(err.into());
    }

    match index {
      Some(index) => {
        let old = std::mem::replace(&mut group.items[index], built);
        // SAFETY: Plain `DirectComposition` calls on live visuals.
        unsafe { old.detach(&group.content) };
        old.unregister();
      }
      None => group.items.push(built),
    }

    Ok(())
  }

  /// Removes group `id`'s item `key`, from the next commit.
  pub fn remove_item(&mut self, id: SceneGroupId, key: u128) {
    let Some(group) = self.groups.iter_mut().find(|group| group.id == id)
    else {
      return;
    };
    let Some(index) = group.items.iter().position(|item| item.key == key)
    else {
      return;
    };

    let item = group.items.remove(index);
    // SAFETY: Plain `DirectComposition` calls on live visuals.
    unsafe { item.detach(&group.content) };
    item.unregister();
  }

  /// Stacks group `id`'s items as their windows are stacked now, e.g.
  /// after one was raised, from the next commit.
  pub fn restack(&mut self, id: SceneGroupId) -> crate::Result<()> {
    let Some(group) = self.groups.iter_mut().find(|group| group.id == id)
    else {
      return Ok(());
    };

    let order = top_level_windows();
    let is_stacked = group.items.windows(2).all(|pair| {
      z_index(pair[0].anchor, &order) >= z_index(pair[1].anchor, &order)
    });
    if is_stacked {
      return Ok(());
    }

    // Bottom-most first, as each is added above the ones before it.
    group
      .items
      .sort_by_key(|item| std::cmp::Reverse(z_index(item.anchor, &order)));

    // SAFETY: Plain `DirectComposition` calls on live visuals.
    unsafe {
      group.content.RemoveAllVisuals()?;
      for item in &group.items {
        item.insert(&group.content, None)?;
      }
    }

    Ok(())
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

    let item_scale = Settable::new(&device, &pose.item_scale)?;

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

      for item in &group.items {
        item_scale.set_scale(&item.zoom)?;
        if let Some(pinned) = &item.pinned {
          item_scale.set_scale(&pinned.zoom)?;
        }

        let backdrops =
          item.copies.iter().filter_map(|copy| copy.backdrop.as_ref());
        for backdrop in backdrops {
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

  /// Poses group `id`'s item `key`, from the next commit.
  pub fn pose_item(
    &mut self,
    id: SceneGroupId,
    key: u128,
    pose: &ItemPose,
  ) -> crate::Result<()> {
    let device = self.device.clone();
    let Some(item) = self
      .groups
      .iter()
      .find(|group| group.id == id)
      .and_then(|group| group.items.iter().find(|item| item.key == key))
    else {
      return Ok(());
    };

    #[allow(clippy::cast_precision_loss)]
    let (width, height) =
      (item.frame.width() as f32, item.frame.height() as f32);
    let scale_x = Settable::new(&device, &pose.scale_x)?;
    let scale_y = Settable::new(&device, &pose.scale_y)?;
    let opacity = Settable::new(&device, &pose.opacity)?;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    unsafe {
      Settable::new(&device, &pose.offset_x)?.set(
        |value| item.visual.SetOffsetX2(value),
        |animation| item.visual.SetOffsetX(animation),
      )?;
      Settable::new(&device, &pose.offset_y)?.set(
        |value| item.visual.SetOffsetY2(value),
        |animation| item.visual.SetOffsetY(animation),
      )?;
      opacity.set(
        |value| item.effect.SetOpacity2(value),
        |animation| item.effect.SetOpacity(animation),
      )?;
      if let Some(pinned) = &item.pinned {
        opacity.set(
          |value| pinned.effect.SetOpacity2(value),
          |animation| pinned.effect.SetOpacity(animation),
        )?;
      }
      Settable::new(&device, &pose.zoom)?.set_scale(&item.own_zoom)?;

      for copy in &item.copies {
        match &copy.shape {
          Shape::Whole {
            vignette,
            clip,
            right,
            bottom,
          } => {
            if let Some(vignette) = vignette {
              scale_x.set(
                |value| vignette.SetScaleX2(value),
                |animation| vignette.SetScaleX(animation),
              )?;
              scale_y.set(
                |value| vignette.SetScaleY2(value),
                |animation| vignette.SetScaleY(animation),
              )?;
            }
            if let Some(right) = right {
              Settable::new(
                &device,
                &pose.scale_x.mapped(width, right - width),
              )?
              .set(
                |value| clip.SetRight2(value),
                |animation| clip.SetRight(animation),
              )?;
            }
            if let Some(bottom) = bottom {
              Settable::new(
                &device,
                &pose.scale_y.mapped(height, bottom - height),
              )?
              .set(
                |value| clip.SetBottom2(value),
                |animation| clip.SetBottom(animation),
              )?;
            }
          }
          Shape::Pieces(pieces) => {
            for piece in pieces {
              piece.pose(
                &device,
                &pose.scale_x,
                &pose.scale_y,
                width,
                height,
              )?;
            }
          }
        }

        if let Some(backdrop) = &copy.backdrop {
          let back = -backdrop.parallax;
          Settable::new(&device, &pose.offset_x.mapped(back, 0.0))?.set(
            |value| backdrop.item_back.SetOffsetX2(value),
            |animation| backdrop.item_back.SetOffsetX(animation),
          )?;
          Settable::new(&device, &pose.offset_y.mapped(back, 0.0))?.set(
            |value| backdrop.item_back.SetOffsetY2(value),
            |animation| backdrop.item_back.SetOffsetY(animation),
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
    for item in &group.items {
      item.unregister();
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

  /// The windows an item of `windows` copies: `windows`, then the shown
  /// windows of `order` drawn as their companions, and those they own.
  fn resolve(&self, windows: &[SceneWindow], order: &[HWND]) -> Members {
    let explicit =
      windows.iter().map(|w| w.hwnd.0).collect::<HashSet<_>>();
    let mut members = Members {
      moving: windows.to_vec(),
      pinned: Vec::new(),
    };

    for &hwnd in order {
      if hwnd == self.host.hwnd
        || windows.iter().any(|window| window.draws(hwnd))
      {
        continue;
      }

      // SAFETY: A stale handle just reads as hidden.
      let is_shown = unsafe { IsWindowVisible(hwnd) }.as_bool()
        && !NativeWindow::new(hwnd.0).is_cloaked().unwrap_or(true);
      if !is_shown {
        continue;
      }

      // A companion covers its window's frame, wherever that is.
      let companion_of = companion::companion_of(hwnd)
        .and_then(|of| windows.iter().find(|window| window.hwnd == of));

      if let Some(of) = companion_of {
        members.moving.push(SceneWindow {
          placement: of
            .placement
            .as_ref()
            .map(|(_, frame)| (frame.clone(), frame.clone())),
          corner_radius: of.corner_radius,
          ..SceneWindow::as_is(hwnd)
        });
      } else if is_owned_by(hwnd, &explicit) {
        members.pinned.push(SceneWindow::as_is(hwnd));
      }
    }

    members
  }

  /// Adds a group of `items`, of `members` (each item's windows resolved),
  /// stacked as their windows are in `order`.
  fn add_resolved_group(
    &mut self,
    items: &[SceneItem],
    members: &[Members],
    order: &[HWND],
  ) -> crate::Result<SceneGroupId> {
    let device = self.device.clone();
    let id = SceneGroupId(self.next_group);
    self.next_group += 1;

    // SAFETY: Plain `DirectComposition` calls on live objects.
    let mut group = unsafe {
      let panel = device.dcomp.CreateVisual()?;
      let effect = device.dcomp.CreateEffectGroup()?;
      panel.SetEffect(&effect)?;

      #[allow(clippy::cast_precision_loss)]
      let (width, height) =
        (self.monitor.width() as f32, self.monitor.height() as f32);
      let scale = device.dcomp.CreateScaleTransform()?;
      scale.SetCenterX2(width / 2.0)?;
      scale.SetCenterY2(height / 2.0)?;
      let scaled = device.dcomp.CreateVisual()?;
      scaled.SetTransform(&scale)?;
      panel.AddVisual(&scaled, false, None)?;

      let clip = device.dcomp.CreateRectangleClip()?;
      clip.SetLeft2(0.0)?;
      clip.SetTop2(0.0)?;
      clip.SetRight2(width)?;
      clip.SetBottom2(height)?;
      let content = device.dcomp.CreateVisual()?;
      content.SetClip(&clip)?;
      scaled.AddVisual(&content, false, None)?;

      Group {
        id,
        panel,
        effect,
        scale,
        content,
        items: Vec::new(),
      }
    };

    // Bottom-most first, as each is added above the ones before it.
    let mut stacked = items.iter().zip(members).collect::<Vec<_>>();
    stacked.sort_by_key(|(item, _)| {
      std::cmp::Reverse(
        item
          .windows
          .first()
          .map_or(usize::MAX, |window| z_index(window.hwnd, order)),
      )
    });

    let release = |group: &Group| {
      for item in &group.items {
        item.unregister();
      }
    };

    for (item, members) in stacked {
      let built = match self.build_item(item, members, order) {
        Ok(built) => built,
        Err(err) => {
          release(&group);
          return Err(err);
        }
      };

      // SAFETY: Plain `DirectComposition` calls on live visuals.
      let inserted = unsafe { built.insert(&group.content, None) };
      group.items.push(built);
      if let Err(err) = inserted {
        release(&group);
        return Err(err.into());
      }
    }

    // SAFETY: Plain `DirectComposition` call on live visuals.
    if let Err(err) =
      unsafe { self.groups_parent.AddVisual(&group.panel, false, None) }
    {
      release(&group);
      return Err(err.into());
    }

    self.groups.push(group);
    Ok(id)
  }

  /// Builds `item` from its `members`, stacked as their windows are in
  /// `order`. Fails if a required one can't be copied.
  #[allow(clippy::cast_precision_loss)]
  fn build_item(
    &self,
    item: &SceneItem,
    members: &Members,
    order: &[HWND],
  ) -> crate::Result<Item> {
    let dcomp = &self.device.dcomp;
    let center = (
      (item.frame.left - self.monitor.left) as f32
        + item.frame.width() as f32 / 2.0,
      (item.frame.top - self.monitor.top) as f32
        + item.frame.height() as f32 / 2.0,
    );

    // A visual scaled about the frame's center, with the transform doing
    // it.
    let zoomed = || -> crate::Result<_> {
      // SAFETY: Plain `DirectComposition` calls on live objects.
      unsafe {
        let zoom = dcomp.CreateScaleTransform()?;
        zoom.SetCenterX2(center.0)?;
        zoom.SetCenterY2(center.1)?;
        let visual = dcomp.CreateVisual()?;
        visual.SetTransform(&zoom)?;
        Ok((visual, zoom))
      }
    };

    let (zoomed_visual, zoom) = zoomed()?;
    let (own_visual, own_zoom) = zoomed()?;
    // SAFETY: Plain `DirectComposition` calls on live objects.
    let (visual, effect) = unsafe {
      let visual = dcomp.CreateVisual()?;
      let effect = dcomp.CreateEffectGroup()?;
      visual.SetEffect(&effect)?;
      visual.AddVisual(&zoomed_visual, false, None)?;
      zoomed_visual.AddVisual(&own_visual, false, None)?;
      (visual, effect)
    };

    let mut built = Item {
      key: item.key,
      anchor: item.windows.first().map_or(HWND(0), |window| window.hwnd),
      frame: item.frame.clone(),
      visual,
      effect,
      zoom,
      own_zoom,
      copies: Vec::new(),
      pinned: None,
    };

    if let Err(err) = self.copy_all(
      &members.moving,
      &item.frame,
      &own_visual,
      order,
      &mut built.copies,
    ) {
      built.unregister();
      return Err(err);
    }

    if !members.pinned.is_empty() {
      let pinned = zoomed().and_then(|(visual, zoom)| {
        // SAFETY: Plain `DirectComposition` calls on live objects.
        let effect = unsafe {
          let effect = dcomp.CreateEffectGroup()?;
          visual.SetEffect(&effect)?;
          effect
        };
        Ok(Pinned {
          visual,
          effect,
          zoom,
          copies: Vec::new(),
        })
      });

      let mut pinned = match pinned {
        Ok(pinned) => pinned,
        Err(err) => {
          built.unregister();
          return Err(err);
        }
      };

      let copied = self.copy_all(
        &members.pinned,
        &item.frame,
        &pinned.visual.clone(),
        order,
        &mut pinned.copies,
      );
      built.pinned = Some(pinned);
      if let Err(err) = copied {
        built.unregister();
        return Err(err);
      }
    }

    Ok(built)
  }

  /// Copies `windows` of an item of frame `frame` above `parent`'s
  /// children, stacked as they are in `order`, into `copies`. Fails if a
  /// required one can't be copied.
  fn copy_all(
    &self,
    windows: &[SceneWindow],
    frame: &Rect,
    parent: &IDCompositionVisual2,
    order: &[HWND],
    copies: &mut Vec<WindowCopy>,
  ) -> crate::Result<()> {
    let mut stacked = windows.iter().collect::<Vec<_>>();
    // Bottom-most first, as each is added above the ones before it.
    // Windows gone meanwhile go first, and fail to copy.
    stacked.sort_by_key(|window| {
      std::cmp::Reverse(z_index(window.hwnd, order))
    });

    for window in stacked {
      match self.copy(window, Some(frame), parent) {
        Ok(Some((copy, _))) => copies.push(copy),
        Ok(None) => {}
        Err(err) if window.required => {
          return Err(crate::Error::Platform(format!(
            "Can't copy {}: {err}",
            window_class::describe(window.hwnd)
          )));
        }
        Err(err) => tracing::debug!(
          "Scene left out {}: {err}",
          window_class::describe(window.hwnd)
        ),
      }
    }

    Ok(())
  }

  /// Copies `window` above `parent`'s other children, cut to its visible
  /// frame. It follows `item_frame`, its own frame when `None`, as that
  /// resizes.
  ///
  /// Returns the copy, with whether it opaquely covers the whole monitor,
  /// or `None` if it isn't shown.
  fn copy(
    &self,
    window: &SceneWindow,
    item_frame: Option<&Rect>,
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

    let item_frame = item_frame.unwrap_or(&frame);

    if let SceneFit::Frame {
      left,
      top,
      right,
      bottom,
    } = window.fit
    {
      let copy = self.copy_frame(
        window.hwnd,
        alpha,
        &outer,
        item_frame,
        [left, top, right, bottom],
        parent,
      )?;
      return Ok(Some((copy, false)));
    }

    let (content, thumbnail) = self.thumbnail(
      window.hwnd,
      alpha,
      Mapping::Whole(outer.width(), outer.height()),
    )?;

    // Without its backdrop, the copy is still worth showing.
    let backdrop = window.backdrop.and_then(|backdrop| {
      self
        .backdrop(&backdrop.params, &outer, &frame, item_frame)
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
        item_frame,
        parent,
      )
    };

    match placed {
      Ok(shape) => {
        let copy = WindowCopy {
          thumbnails: vec![thumbnail],
          backdrop: backdrop.map(|(backdrop, _)| backdrop),
          shape,
        };
        let covers_monitor =
          alpha == u8::MAX && frame.contains_rect(&self.monitor);
        Ok(Some((copy, covers_monitor)))
      }
      Err(err) => {
        // SAFETY: Registered above and not handed out.
        let _ = unsafe { DwmUnregisterThumbnail(thumbnail) };
        if let Some((backdrop, _)) = &backdrop {
          backdrop.unregister();
        }
        Err(err)
      }
    }
  }

  /// A thumbnail of `source` at `alpha`, mapped as `mapping` says, with
  /// its handle.
  fn thumbnail(
    &self,
    source: HWND,
    alpha: u8,
    mapping: Mapping,
  ) -> crate::Result<(IDCompositionVisual, isize)> {
    let properties =
      |flags, source: RECT, destination: RECT| DWM_THUMBNAIL_PROPERTIES {
        dwFlags: flags
          | DWM_TNP_RECTDESTINATION
          | DWM_TNP_OPACITY
          | DWM_TNP_VISIBLE
          | DWM_TNP_SOURCECLIENTAREAONLY,
        rcDestination: destination,
        rcSource: source,
        opacity: alpha,
        fVisible: true.into(),
        fSourceClientAreaOnly: false.into(),
      };

    match mapping {
      // Without a source rect, DWM scales whatever size the window has
      // reached over the destination.
      Mapping::Stretched(width, height) => {
        if width <= 0 || height <= 0 {
          return Err(crate::Error::Platform(
            "A copy can't be stretched over nothing.".to_string(),
          ));
        }

        let over = RECT {
          left: 0,
          top: 0,
          right: width,
          bottom: height,
        };
        self
          .create_thumbnail(source, &properties(0, RECT::default(), over))
      }
      Mapping::Whole(width, height) => {
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

        let rect = |width: i32, height: i32| RECT {
          left: 0,
          top: 0,
          right: width,
          bottom: height,
        };
        let own = rect(size.cx, size.cy);

        // Sampled past the window's edge, it is shown 1:1 at any size it
        // reaches up to `width` by `height`, and nothing beyond its edge.
        let reach = rect(size.cx.max(width), size.cy.max(height));
        if reach != own {
          let reaching = self.create_thumbnail(
            source,
            &properties(DWM_TNP_RECTSOURCE, reach, reach),
          );
          match reaching {
            Ok(created) => return Ok(created),
            Err(err) => tracing::debug!(
              "Scene copies {} at its own size: {err}",
              window_class::describe(source)
            ),
          }
        }

        self.create_thumbnail(
          source,
          &properties(DWM_TNP_RECTSOURCE, own, own),
        )
      }
    }
  }

  /// A thumbnail of `source` as `properties` say, with its handle.
  fn create_thumbnail(
    &self,
    source: HWND,
    properties: &DWM_THUMBNAIL_PROPERTIES,
  ) -> crate::Result<(IDCompositionVisual, isize)> {
    let mut raw = null_mut();
    let mut thumbnail = 0;
    // SAFETY: Out pointers and `properties` outlive the call, and the
    // device is live.
    unsafe {
      (self.device.exports.create_thumbnail_visual)(
        self.host.hwnd,
        source,
        SHARED_THUMBNAIL_FLAGS,
        std::ptr::from_ref(properties),
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
  /// frame `frame`, in its copy's coordinates, with the transform scaling
  /// its vignette along with `item_frame`.
  #[allow(clippy::cast_precision_loss)]
  fn backdrop(
    &self,
    params: &BackdropOverlayParams,
    outer: &Rect,
    frame: &Rect,
    item_frame: &Rect,
  ) -> crate::Result<(BackdropCopy, IDCompositionScaleTransform)> {
    let canvas = backdrop_canvas::canvas_for(&self.monitor, params)?;
    let (content, thumbnail) =
      self.thumbnail(canvas, u8::MAX, Mapping::Whole(0, 0))?;

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
        let item_back = dcomp.CreateVisual()?;
        item_back.AddVisual(&blur, false, None)?;

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

        let vignette_scale = dcomp.CreateScaleTransform()?;
        vignette_scale
          .SetCenterX2((item_frame.left - outer.left) as f32)?;
        vignette_scale.SetCenterY2((item_frame.top - outer.top) as f32)?;
        let scaled = dcomp.CreateVisual()?;
        scaled.SetTransform(&vignette_scale)?;
        scaled.AddVisual(&vignette, false, None)?;

        let visual = dcomp.CreateVisual()?;
        let opacity = dcomp.CreateEffectGroup()?;
        opacity.SetOpacity2(params.opacity.clamp(0.0, 1.0))?;
        visual.SetEffect(&opacity)?;
        visual.AddVisual(&item_back, false, None)?;
        visual.AddVisual(&scaled, false, None)?;

        Ok((visual, item_back, blur, vignette_scale))
      })()
    };

    match built {
      Ok((visual, item_back, blur, vignette_scale)) => Ok((
        BackdropCopy {
          thumbnail,
          visual,
          item_back,
          blur,
          rest,
          parallax,
        },
        vignette_scale,
      )),
      Err(err) => {
        // SAFETY: Registered above and not handed out.
        let _ = unsafe { DwmUnregisterThumbnail(thumbnail) };
        Err(err)
      }
    }
  }

  /// Builds the visuals placing `content`, a thumbnail of `window`, over
  /// the monitor, on top of its `backdrop`, cut to its visible `frame`,
  /// and adds them above `parent`'s other children. Returns how it follows
  /// `item_frame` as that resizes.
  ///
  /// # Safety
  ///
  /// Every object must be live and of this scene's device.
  #[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
  unsafe fn place(
    &self,
    window: &SceneWindow,
    content: &IDCompositionVisual,
    backdrop: Option<&(BackdropCopy, IDCompositionScaleTransform)>,
    outer: &Rect,
    frame: &Rect,
    item_frame: &Rect,
    parent: &IDCompositionVisual2,
  ) -> crate::Result<Shape> {
    let dcomp = &self.device.dcomp;

    // DWM draws a thumbnail from its window rect's corner, past the
    // invisible resize border, so the copy is anchored there.
    let visual = dcomp.CreateVisual()?;
    visual.SetOffsetX2((outer.left - self.monitor.left) as f32)?;
    visual.SetOffsetY2((outer.top - self.monitor.top) as f32)?;

    // Cut to what DWM draws of the window, rounded as it is.
    let right = (frame.right - outer.left) as f32;
    let bottom = (frame.bottom - outer.top) as f32;
    let frame_clip = || -> windows::core::Result<_> {
      let clip = dcomp.CreateRectangleClip()?;
      clip.SetLeft2((frame.left - outer.left) as f32)?;
      clip.SetTop2((frame.top - outer.top) as f32)?;
      clip.SetRight2(right)?;
      clip.SetBottom2(bottom)?;

      let radius = window.corner_radius;
      if radius > 0.0 {
        clip.SetTopLeftRadiusX2(radius)?;
        clip.SetTopLeftRadiusY2(radius)?;
        clip.SetTopRightRadiusX2(radius)?;
        clip.SetTopRightRadiusY2(radius)?;
        clip.SetBottomLeftRadiusX2(radius)?;
        clip.SetBottomLeftRadiusY2(radius)?;
        clip.SetBottomRightRadiusX2(radius)?;
        clip.SetBottomRightRadiusY2(radius)?;
      }
      Ok(clip)
    };

    // The content keeps its size, cut where it rests too: a window drawn
    // smaller as it shrinks shows its invisible resize borders past that,
    // which thumbnails draw. The clip following the frame holds it, over
    // a backdrop that stays put and shows where the content doesn't reach.
    let content_clipped = dcomp.CreateVisual()?;
    content_clipped.SetClip(&frame_clip()?)?;
    content_clipped.AddVisual(content, false, None)?;

    let clip = frame_clip()?;
    let clipped = dcomp.CreateVisual()?;
    clipped.SetClip(&clip)?;
    if let Some((backdrop, _)) = backdrop {
      clipped.AddVisual(&backdrop.visual, false, None)?;
    }
    clipped.AddVisual(&content_clipped, false, None)?;
    visual.AddVisual(&clipped, false, None)?;
    parent.AddVisual(&visual, false, None)?;

    Ok(Shape::Whole {
      vignette: backdrop.map(|(_, vignette)| vignette.clone()),
      clip,
      right: (frame.right == item_frame.right).then_some(right),
      bottom: (frame.bottom == item_frame.bottom).then_some(bottom),
    })
  }

  /// Copies `hwnd`, a window without invisible resize borders, at `alpha`
  /// over `outer` as the pieces of a frame cut at `insets` (left, top,
  /// right, bottom), which keep to `frame`'s edges, above `parent`'s other
  /// children.
  fn copy_frame(
    &self,
    hwnd: HWND,
    alpha: u8,
    outer: &Rect,
    frame: &Rect,
    insets: [f32; 4],
    parent: &IDCompositionVisual2,
  ) -> crate::Result<WindowCopy> {
    #[allow(clippy::cast_precision_loss)]
    let along = |start: i32, end: i32, origin: i32| {
      ((start - origin) as f32, (end - origin) as f32)
    };
    let (left, top) = (self.monitor.left, self.monitor.top);
    let xs = spans(
      along(outer.left, outer.right, left),
      along(frame.left, frame.right, left),
      (insets[0], insets[2]),
    );
    let ys = spans(
      along(outer.top, outer.bottom, top),
      along(frame.top, frame.bottom, top),
      (insets[1], insets[3]),
    );
    #[allow(clippy::cast_precision_loss)]
    let origin = ((outer.left - left) as f32, (outer.top - top) as f32);

    let mut thumbnails = Vec::new();
    let mut pieces = Vec::new();
    let release = |thumbnails: &[isize]| {
      for thumbnail in thumbnails {
        // SAFETY: Registered here and not handed out.
        let _ = unsafe { DwmUnregisterThumbnail(*thumbnail) };
      }
    };

    // The middle is left out.
    let cells = (0..3)
      .flat_map(|row| (0..3).map(move |column| (column, row)))
      .filter(|cell| *cell != (1, 1));

    for (column, row) in cells {
      let (Some(x), Some(y)) = (xs[column], ys[row]) else {
        continue;
      };

      let (content, thumbnail) = match self.thumbnail(
        hwnd,
        alpha,
        Mapping::Stretched(outer.width(), outer.height()),
      ) {
        Ok(thumbnail) => thumbnail,
        Err(err) => {
          release(&thumbnails);
          return Err(err);
        }
      };
      thumbnails.push(thumbnail);

      // SAFETY: Plain `DirectComposition` calls on live objects.
      match unsafe { self.place_piece(&content, origin, x, y, parent) } {
        Ok(piece) => pieces.push(piece),
        Err(err) => {
          release(&thumbnails);
          return Err(err);
        }
      }
    }

    Ok(WindowCopy {
      thumbnails,
      backdrop: None,
      shape: Shape::Pieces(pieces),
    })
  }

  /// Places the part of `content`, a copy drawn from `origin`, that spans
  /// `x` and `y`, above `parent`'s other children.
  ///
  /// # Safety
  ///
  /// Every object must be live and of this scene's device.
  unsafe fn place_piece(
    &self,
    content: &IDCompositionVisual,
    origin: (f32, f32),
    x: Span,
    y: Span,
    parent: &IDCompositionVisual2,
  ) -> crate::Result<Piece> {
    let dcomp = &self.device.dcomp;

    let shifted = dcomp.CreateVisual()?;
    shifted.SetOffsetX2(origin.0 - x.start)?;
    shifted.SetOffsetY2(origin.1 - y.start)?;
    shifted.AddVisual(content, false, None)?;

    let clip = dcomp.CreateRectangleClip()?;
    clip.SetLeft2(0.0)?;
    clip.SetTop2(0.0)?;
    clip.SetRight2(x.length)?;
    clip.SetBottom2(y.length)?;
    let clipped = dcomp.CreateVisual()?;
    clipped.SetClip(&clip)?;
    clipped.AddVisual(&shifted, false, None)?;

    // Stretched from its own start, which the visual above moves.
    let scale = dcomp.CreateScaleTransform()?;
    let scaled = dcomp.CreateVisual()?;
    scaled.SetTransform(&scale)?;
    scaled.AddVisual(&clipped, false, None)?;

    let visual = dcomp.CreateVisual()?;
    visual.SetOffsetX2(x.start)?;
    visual.SetOffsetY2(y.start)?;
    visual.AddVisual(&scaled, false, None)?;
    parent.AddVisual(&visual, false, None)?;

    Ok(Piece {
      visual,
      scale,
      x,
      y,
    })
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
    for copy in &self.stills {
      copy.unregister();
    }
    for item in self.groups.iter().flat_map(|group| &group.items) {
      item.unregister();
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

/// Where `hwnd` is in `order`, from its top; windows gone meanwhile are at
/// the bottom.
fn z_index(hwnd: HWND, order: &[HWND]) -> usize {
  order
    .iter()
    .position(|other| *other == hwnd)
    .unwrap_or(usize::MAX)
}

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

/// Whether `hwnd` is owned by one of `windows`.
fn is_owned_by(hwnd: HWND, windows: &HashSet<isize>) -> bool {
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

/// The spans a frame copy is cut into along one axis: its two ends,
/// `insets` long, and the middle between them, each end keeping to the
/// frame's nearest edge. `placement` and `frame` are each a start and an
/// end. Empty ones are `None`.
fn spans(
  placement: (f32, f32),
  frame: (f32, f32),
  insets: (f32, f32),
) -> [Option<Span>; 3] {
  let (start, end) = placement;
  let length = (end - start).max(0.0);
  let low = insets.0.clamp(0.0, length);
  let high = insets.1.clamp(0.0, length - low);

  // 1 for an edge keeping to the frame's end, which moves as it grows.
  let keeps_to = |edge: f32| {
    if (edge - frame.1).abs() < (edge - frame.0).abs() {
      1.0
    } else {
      0.0
    }
  };
  let (first, last) = (keeps_to(start), keeps_to(end));

  let span = |start: f32, length: f32, moves: f32, grows: f32| {
    (length > 0.0).then_some(Span {
      start,
      length,
      moves,
      grows,
    })
  };

  [
    span(start, low, first, 0.0),
    span(start + low, length - low - high, first, last - first),
    span(end - high, high, last, 0.0),
  ]
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

  use super::{
    spans, vignette_alpha, SceneMotion, SceneSegment, SceneValue, Span,
  };

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

    let SceneValue::Fixed(value) = motion.mapped(0.0, 3.0) else {
      panic!("a motion scaled to nothing stays put");
    };
    assert!((value - 3.0).abs() < f32::EPSILON);
  }

  fn fixed(value: &SceneValue) -> f32 {
    match value {
      SceneValue::Fixed(value) => *value,
      SceneValue::Moving(_) => f32::NAN,
    }
  }

  #[test]
  fn ring_ends_keep_to_the_frame_and_its_middle_stretches() {
    // A 2px ring around a frame from 100 to 500, cut 10px in.
    let [low, middle, high] =
      spans((98.0, 502.0), (100.0, 500.0), (10.0, 10.0));
    let span = |start, length, moves, grows| {
      Some(Span {
        start,
        length,
        moves,
        grows,
      })
    };
    assert_eq!(low, span(98.0, 10.0, 0.0, 0.0));
    assert_eq!(middle, span(108.0, 384.0, 0.0, 1.0));
    assert_eq!(high, span(492.0, 10.0, 1.0, 0.0));

    // The frame grown by half its width moves its far end by 200px.
    let grown = SceneValue::Fixed(1.5);
    let (Some(middle), Some(high)) = (middle, high) else {
      panic!("a ring has a middle and two ends");
    };
    assert!((fixed(&high.offset(&grown, 400.0)) - 692.0).abs() < 1e-3);
    assert!((fixed(&middle.offset(&grown, 400.0)) - 108.0).abs() < 1e-3);
    assert!(
      (fixed(&middle.stretch(&grown, 400.0)) - 584.0 / 384.0).abs() < 1e-4
    );
  }

  #[test]
  fn tab_bars_keep_their_height_beside_the_frame() {
    // A 28px bar above a frame from 128 to 400.
    let [top, middle, bottom] =
      spans((100.0, 128.0), (128.0, 400.0), (28.0, 0.0));
    assert_eq!(
      top,
      Some(Span {
        start: 100.0,
        length: 28.0,
        moves: 0.0,
        grows: 0.0,
      })
    );
    assert_eq!((middle, bottom), (None, None));

    // One below it moves with the frame's bottom.
    let [top, middle, bottom] =
      spans((400.0, 428.0), (128.0, 400.0), (0.0, 28.0));
    assert_eq!((top, middle), (None, None));
    assert_eq!(
      bottom,
      Some(Span {
        start: 400.0,
        length: 28.0,
        moves: 1.0,
        grows: 0.0,
      })
    );
  }
}
