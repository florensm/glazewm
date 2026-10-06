//! Bitmaps the WM draws its own windows' pictures into: the overview's
//! cards and the stack tab bars.

use windows::Win32::{
  Foundation::{COLORREF, HWND, POINT, SIZE},
  Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject,
    GdiFlush, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC,
    HGDIOBJ,
  },
  UI::WindowsAndMessaging::{UpdateLayeredWindow, ULW_ALPHA},
};

use crate::paint::Canvas;

/// A 32-bit top-down DIB section selected into its own memory DC.
pub(crate) struct Surface {
  pub dc: HDC,
  bitmap: HBITMAP,
  old_bitmap: HGDIOBJ,
  bits: *mut u32,
  pub width: i32,
  pub height: i32,
}

impl Surface {
  pub fn new(width: i32, height: i32) -> Option<Self> {
    if width <= 0 || height <= 0 {
      return None;
    }

    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>())
          .ok()?,
        biWidth: width,
        // Negative height: top-down rows.
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };

    // SAFETY: `info` describes a valid 32-bit DIB and outlives the calls.
    // The DC and bitmap are released in `Drop`, or below on failure.
    unsafe {
      let dc = CreateCompatibleDC(None);
      if dc.is_invalid() {
        return None;
      }

      let mut bits = std::ptr::null_mut();
      let Ok(bitmap) = CreateDIBSection(
        dc,
        &raw const info,
        DIB_RGB_COLORS,
        &raw mut bits,
        None,
        0,
      ) else {
        let _ = DeleteDC(dc);
        return None;
      };

      Some(Self {
        dc,
        bitmap,
        old_bitmap: SelectObject(dc, bitmap),
        bits: bits.cast(),
        width,
        height,
      })
    }
  }

  pub fn pixels(&mut self) -> &mut [u32] {
    let len = usize::try_from(self.width * self.height).unwrap_or(0);

    // SAFETY: The DIB section holds `width * height` 32-bit pixels, owned
    // by `bitmap` until `Drop`, and GDI has finished drawing into it.
    unsafe {
      let _ = GdiFlush();
      std::slice::from_raw_parts_mut(self.bits, len)
    }
  }

  pub fn canvas(&mut self) -> Canvas<'_> {
    let (width, height) = (self.width, self.height);
    Canvas {
      pixels: self.pixels(),
      width,
      height,
    }
  }

  /// Shows the surface as the whole of layered `window`, moving the
  /// window's top-left corner to `position` on screen.
  pub fn show_on(
    &self,
    window: HWND,
    position: POINT,
  ) -> windows::core::Result<()> {
    let blend = BLENDFUNCTION {
      BlendOp: u8::try_from(AC_SRC_OVER).unwrap_or_default(),
      BlendFlags: 0,
      SourceConstantAlpha: u8::MAX,
      AlphaFormat: u8::try_from(AC_SRC_ALPHA).unwrap_or_default(),
    };
    let size = SIZE {
      cx: self.width,
      cy: self.height,
    };
    let source = POINT::default();

    // SAFETY: The window and the surface's DC are valid, and every
    // pointer outlives the call.
    unsafe {
      UpdateLayeredWindow(
        window,
        None,
        Some(&raw const position),
        Some(&raw const size),
        self.dc,
        Some(&raw const source),
        COLORREF(0),
        Some(&raw const blend),
        ULW_ALPHA,
      )
    }
  }
}

impl Drop for Surface {
  fn drop(&mut self) {
    // SAFETY: Both handles were created in `new` and are released once.
    unsafe {
      SelectObject(self.dc, self.old_bitmap);
      let _ = DeleteObject(self.bitmap);
      let _ = DeleteDC(self.dc);
    }
  }
}
