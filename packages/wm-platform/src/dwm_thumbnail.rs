//! DWM thumbnails: live copies of one window drawn on another, which every
//! stand-in, ring copy and overview preview the WM shows is made of.

use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{
    DwmRegisterThumbnail, DwmUnregisterThumbnail,
    DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES,
    DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
    DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
  },
};

/// What a [`Thumbnail`] shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Look {
  /// Part of the source window drawn, in its own coordinates, or `None`
  /// for all of it. Once set, DWM keeps a source: re-register to draw the
  /// whole window again.
  pub source: Option<RECT>,
  /// Where it is drawn on the destination window.
  pub dest: RECT,
  pub opacity: u8,
  pub visible: bool,
}

/// A DWM thumbnail, unregistered on drop, which must come before its
/// destination window is destroyed.
///
/// Remembers what DWM shows, so each update sends only what changed and an
/// unchanged one costs nothing.
pub(crate) struct Thumbnail {
  handle: isize,
  look: Look,
}

impl Thumbnail {
  /// Registers a thumbnail of `source` on `dest`, hidden until shown.
  /// Thumbnails stack in the order they are registered.
  pub fn register(dest: HWND, source: HWND) -> Option<Self> {
    Self::register_with(dest, source, Look::default())
  }

  /// Registers a thumbnail of `source` on `dest` showing `look` from its
  /// first frame. Fails for anything but two valid top-level windows, e.g.
  /// an elevated or UWP `source`.
  pub fn register_with(
    dest: HWND,
    source: HWND,
    look: Look,
  ) -> Option<Self> {
    // SAFETY: Registration fails for anything but two valid top-level
    // windows.
    let handle = unsafe { DwmRegisterThumbnail(dest, source) }.ok()?;
    let thumbnail = Self { handle, look };

    let mut flags = DWM_TNP_RECTDESTINATION
      | DWM_TNP_OPACITY
      | DWM_TNP_VISIBLE
      | DWM_TNP_SOURCECLIENTAREAONLY;
    if look.source.is_some() {
      flags |= DWM_TNP_RECTSOURCE;
    }
    thumbnail.apply(flags, &look).ok()?;

    Some(thumbnail)
  }

  pub fn look(&self) -> &Look {
    &self.look
  }

  /// Shows `look`, sending DWM only what differs from what it shows now.
  pub fn set(&mut self, look: Look) -> crate::Result<()> {
    let mut flags = 0;
    if look.source.is_some() && look.source != self.look.source {
      flags |= DWM_TNP_RECTSOURCE;
    }
    if look.dest != self.look.dest {
      flags |= DWM_TNP_RECTDESTINATION;
    }
    if look.opacity != self.look.opacity {
      flags |= DWM_TNP_OPACITY;
    }
    if look.visible != self.look.visible {
      flags |= DWM_TNP_VISIBLE;
    }

    if flags == 0 {
      return Ok(());
    }

    self.apply(flags, &look)?;
    self.look = Look {
      source: look.source.or(self.look.source),
      ..look
    };
    Ok(())
  }

  /// Shows `source` (in the source window's coordinates) at `dest`.
  pub fn show(&mut self, source: RECT, dest: RECT) -> crate::Result<()> {
    self.set(Look {
      source: Some(source),
      dest,
      visible: true,
      ..self.look
    })
  }

  pub fn set_opacity(&mut self, opacity: u8) -> crate::Result<()> {
    self.set(Look {
      opacity,
      ..self.look
    })
  }

  pub fn hide(&mut self) {
    let _ = self.set(Look {
      visible: false,
      ..self.look
    });
  }

  /// Shows `source` at `dest`, cut to the part of `dest` inside `clip`, or
  /// hides the thumbnail when none is or `opacity` is 0.
  pub fn place(
    &mut self,
    source: RECT,
    dest: RECT,
    clip: &RECT,
    opacity: u8,
  ) {
    let Some((source, dest)) = clip_rects(source, dest, clip) else {
      self.hide();
      return;
    };

    let _ = self.set(Look {
      source: Some(source),
      dest,
      opacity,
      visible: opacity > 0,
    });
  }

  fn apply(&self, flags: u32, look: &Look) -> crate::Result<()> {
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: flags,
      rcSource: look.source.unwrap_or_default(),
      rcDestination: look.dest,
      opacity: look.opacity,
      fVisible: look.visible.into(),
      fSourceClientAreaOnly: false.into(),
    };

    // SAFETY: `self.handle` stays registered until drop, and `props`
    // outlives the call.
    unsafe {
      DwmUpdateThumbnailProperties(self.handle, &raw const props)
    }?;
    Ok(())
  }
}

impl Drop for Thumbnail {
  fn drop(&mut self) {
    // SAFETY: Registered in `register_with` and unregistered only here.
    unsafe {
      let _ = DwmUnregisterThumbnail(self.handle);
    }
  }
}

/// `(source, dest)` cut to the part of `dest` inside `clip`, with `source`
/// cut in proportion. `None` when none of `dest` is inside.
///
/// DWM draws a thumbnail wherever its destination is, also past its host
/// window, so e.g. an overview card running off its monitor would show on
/// the one beside it.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn clip_rects(
  source: RECT,
  dest: RECT,
  clip: &RECT,
) -> Option<(RECT, RECT)> {
  let visible = RECT {
    left: dest.left.max(clip.left),
    top: dest.top.max(clip.top),
    right: dest.right.min(clip.right),
    bottom: dest.bottom.min(clip.bottom),
  };

  if visible.right <= visible.left || visible.bottom <= visible.top {
    return None;
  }

  if visible == dest {
    return Some((source, dest));
  }

  let scale_x = f64::from(source.right - source.left)
    / f64::from(dest.right - dest.left);
  let scale_y = f64::from(source.bottom - source.top)
    / f64::from(dest.bottom - dest.top);
  let scaled =
    |pixels: i32, scale: f64| (f64::from(pixels) * scale).round() as i32;

  let source = RECT {
    left: source.left + scaled(visible.left - dest.left, scale_x),
    top: source.top + scaled(visible.top - dest.top, scale_y),
    right: source.right - scaled(dest.right - visible.right, scale_x),
    bottom: source.bottom - scaled(dest.bottom - visible.bottom, scale_y),
  };

  Some((source, visible))
}

#[cfg(test)]
mod tests {
  use windows::Win32::Foundation::RECT;

  use super::clip_rects;

  fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
      left,
      top,
      right,
      bottom,
    }
  }

  #[test]
  fn inside_the_clip_is_left_as_is() {
    let clip = rect(0, 0, 1920, 1080);

    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(10, 10, 410, 310), &clip),
      Some((rect(0, 0, 800, 600), rect(10, 10, 410, 310)))
    );
  }

  #[test]
  fn past_the_clip_is_cut_with_its_source() {
    // Half-size: the 100 on-screen pixels past the right edge are 200 of
    // the source.
    let clip = rect(0, 0, 1920, 1080);

    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(1720, 0, 2120, 300), &clip),
      Some((rect(0, 0, 400, 600), rect(1720, 0, 1920, 300)))
    );
    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(-100, 900, 300, 1200), &clip),
      Some((rect(200, 0, 800, 360), rect(0, 900, 300, 1080)))
    );
  }

  #[test]
  fn wholly_outside_the_clip_shows_nothing() {
    let clip = rect(0, 0, 1920, 1080);

    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(1920, 0, 2320, 300), &clip),
      None
    );
  }
}
