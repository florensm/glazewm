//! Screenshots of one monitor, for `compare_branches.ps1 -Screenshots`:
//! two per scenario run, one mid-animation and one at rest, never more.

use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{bail, Context};
use windows::Win32::{
  Foundation::HWND,
  Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC,
    DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, SRCCOPY,
  },
  UI::HiDpi::{
    SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
  },
};

/// A monitor's rect in physical pixels, as the WM's IPC reports it.
#[derive(Clone, Copy, Debug)]
pub struct Area {
  pub x: i32,
  pub y: i32,
  pub width: i32,
  pub height: i32,
}

/// A captured image: RGB rows, top to bottom.
pub struct Shot {
  width: u32,
  height: u32,
  rgb: Vec<u8>,
}

/// Makes this process per-monitor DPI aware, so screen coordinates are
/// physical pixels, as the WM reports monitors in. Only notes a failure:
/// shots of a monitor at 100% scale are right either way.
pub fn use_physical_pixels() {
  // SAFETY: Takes a constant context and touches no memory of ours.
  if let Err(err) = unsafe {
    SetProcessDpiAwarenessContext(
      DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    )
  } {
    println!("Could not make the bench DPI aware: {err}.");
  }
}

/// Copies `area` of the screen as DWM last composed it.
pub fn capture(area: Area) -> anyhow::Result<Shot> {
  let (Ok(width), Ok(height)) =
    (u32::try_from(area.width), u32::try_from(area.height))
  else {
    bail!("Invalid capture area {area:?}.");
  };
  if width == 0 || height == 0 {
    bail!("Invalid capture area {area:?}.");
  }

  let mut info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>())?,
      biWidth: area.width,
      // Negative: rows top to bottom.
      biHeight: -area.height,
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let mut bgra = vec![0u8; width as usize * height as usize * 4];

  // SAFETY: Every handle is created and released here, in reverse order,
  // whatever fails in between; `bgra` holds exactly the `height` rows of
  // 32-bit pixels `info` describes.
  let lines = unsafe {
    let screen = GetDC(HWND(0));
    if screen.is_invalid() {
      bail!("GetDC failed for the screen.");
    }
    let memory = CreateCompatibleDC(screen);
    let bitmap = CreateCompatibleBitmap(screen, area.width, area.height);
    let previous = SelectObject(memory, bitmap);

    // `CAPTUREBLT` includes layered windows.
    let copied = BitBlt(
      memory,
      0,
      0,
      area.width,
      area.height,
      screen,
      area.x,
      area.y,
      SRCCOPY | CAPTUREBLT,
    );
    let lines = copied.is_ok().then(|| {
      GetDIBits(
        memory,
        bitmap,
        0,
        height,
        Some(bgra.as_mut_ptr().cast()),
        &raw mut info,
        DIB_RGB_COLORS,
      )
    });

    SelectObject(memory, previous);
    let _ = DeleteObject(bitmap);
    let _ = DeleteDC(memory);
    ReleaseDC(HWND(0), screen);

    copied.context("BitBlt from the screen failed.")?;
    lines.unwrap_or(0)
  };

  if u32::try_from(lines).ok() != Some(height) {
    bail!("GetDIBits copied {lines} of {height} rows.");
  }

  let rgb = bgra
    .as_chunks::<4>()
    .0
    .iter()
    .flat_map(|[blue, green, red, _]| [*red, *green, *blue])
    .collect();

  Ok(Shot { width, height, rgb })
}

/// Writes `shot` to `path` as a PNG, making its directory if needed.
pub fn save(shot: &Shot, path: &Path) -> anyhow::Result<()> {
  if let Some(dir) = path.parent() {
    std::fs::create_dir_all(dir)?;
  }

  let file = File::create(path)
    .with_context(|| format!("Failed to create {}.", path.display()))?;
  let mut encoder =
    png::Encoder::new(BufWriter::new(file), shot.width, shot.height);
  encoder.set_color(png::ColorType::Rgb);
  encoder.set_depth(png::BitDepth::Eight);
  encoder.set_compression(png::Compression::Fast);
  encoder
    .write_header()
    .and_then(|mut writer| writer.write_image_data(&shot.rgb))
    .with_context(|| format!("Failed to write {}.", path.display()))
}
