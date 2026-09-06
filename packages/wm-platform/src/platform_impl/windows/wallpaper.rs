//! Desktop-wallpaper discovery for [`BackdropStyle::Wallpaper`].
//!
//! Reports what the OS is actually showing behind everything else: which
//! image file is assigned to a given monitor, how Windows lays it out
//! there, and the solid color painted wherever the image doesn't reach.
//! Rendering that description is `wallpaper_surface`'s job.
//!
//! [`BackdropStyle::Wallpaper`]: crate::BackdropStyle::Wallpaper

use std::{path::PathBuf, time::SystemTime};

use windows::{
  core::{PCWSTR, PWSTR},
  Win32::{
    Foundation::COLORREF,
    Graphics::Gdi::{GetSysColor, COLOR_BACKGROUND},
    System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL},
    UI::{
      Shell::{
        DesktopWallpaper, IDesktopWallpaper, DESKTOP_WALLPAPER_POSITION,
        DWPOS_CENTER, DWPOS_FIT, DWPOS_SPAN, DWPOS_STRETCH, DWPOS_TILE,
      },
      WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETDESKWALLPAPER,
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
      },
    },
  },
};

use crate::{platform_impl::com::COM_INIT, Color, Rect};

/// How Windows lays the wallpaper image out across a monitor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WallpaperFit {
  /// Native size, centered; cropped when larger than the monitor.
  Center,
  /// Native size, repeated from the monitor's top-left corner.
  Tile,
  /// Scaled to the monitor exactly, ignoring aspect ratio.
  Stretch,
  /// Scaled to sit entirely within the monitor, preserving aspect ratio.
  Fit,
  /// Scaled to cover the monitor, preserving aspect ratio; cropped.
  Fill,
  /// One image covering the whole virtual desktop, of which this monitor
  /// shows its own sub-rectangle.
  Span,
}

impl From<DESKTOP_WALLPAPER_POSITION> for WallpaperFit {
  fn from(position: DESKTOP_WALLPAPER_POSITION) -> Self {
    match position {
      DWPOS_CENTER => Self::Center,
      DWPOS_TILE => Self::Tile,
      DWPOS_STRETCH => Self::Stretch,
      DWPOS_FIT => Self::Fit,
      DWPOS_SPAN => Self::Span,
      // `DWPOS_FILL`, and anything a future Windows adds.
      _ => Self::Fill,
    }
  }
}

/// The wallpaper Windows is currently showing on one monitor.
///
/// Equality is what decides whether an already-baked surface can be
/// reused, so every field here is one a change in would alter the rendered
/// pixels.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MonitorWallpaper {
  /// Image assigned to this monitor, or `None` when the desktop is a
  /// plain color (also the case briefly during logon, before the shell
  /// applies one).
  pub image: Option<PathBuf>,

  /// Last-write time of `image`. Tracked because a slideshow, a theme
  /// switch, and Windows Spotlight all rewrite the *same* path rather
  /// than pointing at a new one, so the path alone cannot distinguish a
  /// stale bake from a current one.
  pub modified: Option<SystemTime>,

  /// Layout of `image` within `monitor`.
  pub fit: WallpaperFit,

  /// Solid color painted wherever `image` doesn't reach. Always fully
  /// opaque: it is the bottom of the desktop, with nothing behind it.
  pub background: Color,

  /// Monitor bounds, in virtual-desktop coordinates.
  pub monitor: Rect,

  /// Virtual-desktop bounds. Only [`WallpaperFit::Span`] reads this, but
  /// it is part of a bake's identity regardless: rearranging monitors
  /// changes which slice of a spanned image this one shows while
  /// changing nothing else here.
  pub virtual_screen: Rect,
}

impl MonitorWallpaper {
  /// Queries what the OS is showing on the monitor occupying `monitor`.
  ///
  /// Never fails: an unreachable `IDesktopWallpaper` falls back to
  /// `SPI_GETDESKWALLPAPER`, which knows the image but not the per-monitor
  /// assignment or the fit, and a missing image degrades to the background
  /// color alone.
  pub(crate) fn query(monitor: &Rect, virtual_screen: &Rect) -> Self {
    let (image, fit, background) = match desktop_wallpaper() {
      Some(wallpaper) => query_via_com(&wallpaper, monitor),
      None => (
        legacy_wallpaper_path(),
        WallpaperFit::Fill,
        system_background(),
      ),
    };

    let modified = image.as_ref().and_then(|path| {
      std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
    });

    Self {
      image,
      modified,
      fit,
      background,
      monitor: monitor.clone(),
      virtual_screen: virtual_screen.clone(),
    }
  }
}

/// A monitor-independent fingerprint of the desktop's wallpaper settings.
///
/// Exists because the shell's `WM_SETTINGCHANGE` broadcast cannot be
/// relied on: `SPI_SETDESKWALLPAPER` arrives for some ways of changing the
/// wallpaper and not others (the Settings app, a slideshow rotation, and
/// Windows Spotlight do not all announce themselves the same way). Polling
/// this instead bounds how long a stale backdrop can persist, without
/// needing to know which paths broadcast.
///
/// Deliberately ignores the per-monitor assignment: a change to any
/// monitor also moves one of the fields here, and the per-monitor detail
/// is re-queried during the re-bake that follows anyway.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DesktopSignature {
  image: Option<PathBuf>,
  modified: Option<SystemTime>,
  fit: WallpaperFit,
  background: Color,
}

/// Fingerprints the desktop's current wallpaper settings.
///
/// One `CoCreateInstance` plus one `stat`; meant to be called on a timer,
/// not per frame.
pub(crate) fn desktop_signature() -> DesktopSignature {
  let (image, fit, background) = match desktop_wallpaper() {
    Some(wallpaper) => {
      // SAFETY: `wallpaper` is a live interface pointer and neither call
      // takes arguments.
      let fit = unsafe { wallpaper.GetPosition() }
        .map_or(WallpaperFit::Fill, WallpaperFit::from);

      // SAFETY: As above.
      let background = unsafe { wallpaper.GetBackgroundColor() }
        .map_or_else(|_| system_background(), from_colorref);

      (wallpaper_path(&wallpaper, None), fit, background)
    }
    None => (
      legacy_wallpaper_path(),
      WallpaperFit::Fill,
      system_background(),
    ),
  };

  let modified = image.as_ref().and_then(|path| {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
  });

  DesktopSignature {
    image,
    modified,
    fit,
    background,
  }
}

/// Creates a fresh `IDesktopWallpaper`, or `None` on a system/shell state
/// that doesn't offer one.
///
/// Deliberately not cached: wallpaper is only queried when a surface is
/// (re)baked, so a per-query `CoCreateInstance` costs nothing measurable,
/// and a fresh instance sidesteps the stale-pointer-after-Explorer-restart
/// problem that `ComInit::with_retry` exists to paper over for the
/// interfaces that *are* cached.
fn desktop_wallpaper() -> Option<IDesktopWallpaper> {
  // Guarantees this thread has an apartment before `CoCreateInstance`.
  COM_INIT.with(|com| drop(com.borrow()));

  // SAFETY: `DesktopWallpaper` is a valid CLSID and the inferred interface
  // is the one it implements.
  unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL) }.ok()
}

/// Reads the image path, fit, and background color for `monitor`.
fn query_via_com(
  wallpaper: &IDesktopWallpaper,
  monitor: &Rect,
) -> (Option<PathBuf>, WallpaperFit, Color) {
  // SAFETY: `wallpaper` is a live interface pointer, and neither call
  // takes arguments.
  let fit = unsafe { wallpaper.GetPosition() }
    .map_or(WallpaperFit::Fill, WallpaperFit::from);

  // SAFETY: As above.
  let background = unsafe { wallpaper.GetBackgroundColor() }
    .map_or_else(|_| system_background(), from_colorref);

  let image = monitor_device_path(wallpaper, monitor)
    .and_then(|monitor_id| wallpaper_path(wallpaper, Some(&monitor_id)))
    // A monitor with no per-display assignment -- the common case of one
    // image across every display -- reports it under a null ID instead.
    .or_else(|| wallpaper_path(wallpaper, None));

  (image, fit, background)
}

/// Finds the device path of the monitor occupying `monitor`.
///
/// `IDesktopWallpaper` keys everything on opaque device-path strings while
/// the rest of this crate works in virtual-desktop rectangles, and the
/// interface offers no lookup in that direction -- so this walks its
/// monitor list and matches on the rectangle it reports for each.
fn monitor_device_path(
  wallpaper: &IDesktopWallpaper,
  monitor: &Rect,
) -> Option<Vec<u16>> {
  // SAFETY: `wallpaper` is a live interface pointer.
  let count = unsafe { wallpaper.GetMonitorDevicePathCount() }.ok()?;

  (0..count).find_map(|index| {
    // SAFETY: `index` is within the count just reported.
    let path = unsafe { wallpaper.GetMonitorDevicePathAt(index) }.ok()?;
    let owned = take_com_wide_string(path)?;

    // SAFETY: `owned` outlives the call and is null-terminated.
    let rect =
      unsafe { wallpaper.GetMonitorRECT(PCWSTR(owned.as_ptr())) }.ok()?;

    (Rect::from_ltrb(rect.left, rect.top, rect.right, rect.bottom)
      == *monitor)
      .then_some(owned)
  })
}

/// Reads the image assigned to `monitor_id`, or the desktop-wide one when
/// `None`, mapping the empty string Windows returns for "no image" to
/// `None`.
fn wallpaper_path(
  wallpaper: &IDesktopWallpaper,
  monitor_id: Option<&[u16]>,
) -> Option<PathBuf> {
  let id = monitor_id.map_or(PCWSTR::null(), |id| PCWSTR(id.as_ptr()));

  // SAFETY: `id` is either null (meaning desktop-wide) or a live,
  // null-terminated buffer that outlives the call.
  let raw = unsafe { wallpaper.GetWallpaper(id) }.ok()?;
  let value = take_com_wide_string(raw)?;

  // `take_com_wide_string` keeps the terminator; it isn't part of the
  // path.
  let path = String::from_utf16_lossy(&value[..value.len() - 1]);
  (!path.is_empty()).then(|| PathBuf::from(path))
}

/// Copies a COM-allocated wide string into an owned, still null-terminated
/// buffer and frees the original.
///
/// The terminator is kept so the result can be handed straight back to
/// another `IDesktopWallpaper` call as a `PWSTR`.
fn take_com_wide_string(raw: PWSTR) -> Option<Vec<u16>> {
  if raw.is_null() {
    return None;
  }

  // SAFETY: `raw` is a COM-allocated, null-terminated wide string whose
  // ownership the callee transfers to us, so freeing it here is correct
  // and happens exactly once.
  let owned = unsafe {
    let len = raw.as_wide().len();
    let mut buffer = Vec::with_capacity(len + 1);
    buffer.extend_from_slice(std::slice::from_raw_parts(raw.0, len));
    buffer.push(0);
    CoTaskMemFree(Some(raw.0.cast()));
    buffer
  };

  Some(owned)
}

/// Reads the wallpaper path without COM.
///
/// Knows only the desktop-wide image -- no per-monitor assignment, no fit
/// -- so this is strictly the degraded path taken when `IDesktopWallpaper`
/// can't be created at all.
fn legacy_wallpaper_path() -> Option<PathBuf> {
  let mut buffer = [0u16; 260];

  // SAFETY: `buffer` is `MAX_PATH` wide characters, the documented size
  // for `SPI_GETDESKWALLPAPER`, and its length is passed alongside it.
  unsafe {
    SystemParametersInfoW(
      SPI_GETDESKWALLPAPER,
      buffer.len().try_into().unwrap_or(u32::MAX),
      Some(buffer.as_mut_ptr().cast()),
      SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
    )
  }
  .ok()?;

  let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
  let path = String::from_utf16_lossy(&buffer[..len]);
  (!path.is_empty()).then(|| PathBuf::from(path))
}

/// The desktop background color, from the system color table.
fn system_background() -> Color {
  // SAFETY: `COLOR_BACKGROUND` is a valid system color index.
  from_colorref(COLORREF(unsafe { GetSysColor(COLOR_BACKGROUND) }))
}

/// Unpacks a `COLORREF` (`0x00BBGGRR`) into an opaque `Color`.
#[allow(clippy::cast_possible_truncation)]
fn from_colorref(color: COLORREF) -> Color {
  Color {
    r: color.0 as u8,
    g: (color.0 >> 8) as u8,
    b: (color.0 >> 16) as u8,
    a: 255,
  }
}

#[cfg(test)]
mod tests {
  use windows::Win32::UI::Shell::{
    DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT,
    DWPOS_SPAN, DWPOS_STRETCH, DWPOS_TILE,
  };

  use super::WallpaperFit;

  /// Every documented `DWPOS_*` value maps to its own fit, and an
  /// unrecognized one degrades to `Fill` -- what Windows itself defaults
  /// new installs to -- rather than panicking.
  #[test]
  fn fit_maps_every_position() {
    assert_eq!(WallpaperFit::from(DWPOS_CENTER), WallpaperFit::Center);
    assert_eq!(WallpaperFit::from(DWPOS_TILE), WallpaperFit::Tile);
    assert_eq!(WallpaperFit::from(DWPOS_STRETCH), WallpaperFit::Stretch);
    assert_eq!(WallpaperFit::from(DWPOS_FIT), WallpaperFit::Fit);
    assert_eq!(WallpaperFit::from(DWPOS_FILL), WallpaperFit::Fill);
    assert_eq!(WallpaperFit::from(DWPOS_SPAN), WallpaperFit::Span);

    assert_eq!(
      WallpaperFit::from(DESKTOP_WALLPAPER_POSITION(99)),
      WallpaperFit::Fill
    );
  }
}
