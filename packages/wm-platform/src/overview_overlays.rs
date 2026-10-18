//! What the overview shows over its cards and windows: highlights that
//! glide to whatever they highlight, and rounded panels. Each is made of
//! one small picture stretched to any size, so moving or resizing it
//! redraws nothing.

use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  overview_chrome::{draw_slices, SliceLook, SLICE_MIDDLE},
  overview_layout::{Glide, RectF},
  overview_thumbnails::{to_opacity, to_win32, Picture, Thumbnail},
};

/// A rounded rect of any size, from one picture of it: its corners shown
/// as drawn and its edges (and middle, if filled) stretched, each part a
/// thumbnail of its own.
pub(crate) struct NineSlice {
  picture: Picture,

  /// Whether it has a middle, rather than being a ring.
  is_filled: bool,

  look: Option<SliceLook>,
  parts: Vec<Thumbnail>,
}

impl NineSlice {
  pub fn new(is_filled: bool) -> Option<Self> {
    Some(Self {
      picture: Picture::new()?,
      is_filled,
      look: None,
      parts: Vec::new(),
    })
  }

  /// Draws it as `look`, unless it shows that already.
  pub fn draw(&mut self, look: SliceLook) {
    if self.look == Some(look) {
      return;
    }

    let size = 2 * look.corner() + SLICE_MIDDLE;
    self
      .picture
      .draw((size, size), |surface| draw_slices(surface, &look));
    self.look = Some(look);
  }

  /// Registers its parts on `host`, above everything registered so far.
  pub fn register(&mut self, host: HWND) {
    let count = if self.is_filled { 9 } else { 8 };
    self.parts = (0..count)
      .filter_map(|_| Thumbnail::register(host, self.picture.window))
      .collect();
  }

  /// Hands its thumbnails over to `retired`, still registered.
  pub fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    retired.append(&mut self.parts);
  }

  /// Shows it over `rect`, its corners `corner_scale` times the size they
  /// are drawn at, cut to `clip`.
  #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
  pub fn place(
    &mut self,
    rect: &RectF,
    corner_scale: f32,
    clip: &RectF,
    opacity: f32,
  ) {
    let Some(look) = self.look else {
      return;
    };

    // Whole pixels, so the parts meet without gaps.
    let dest = rect.to_rect();
    let corner = look.corner();
    let dest_corner = ((corner as f32 * corner_scale).round() as i32)
      .min(dest.width() / 2)
      .min(dest.height() / 2);

    if dest_corner <= 0 || opacity <= 0.0 {
      self.hide();
      return;
    }

    // Each corner, and a strip from the middle to stretch.
    let size = 2 * corner + SLICE_MIDDLE;
    let sources = [
      (0, corner),
      (corner + 1, corner + SLICE_MIDDLE - 1),
      (size - corner, size),
    ];
    let columns = [
      (dest.left, dest.left + dest_corner),
      (dest.left + dest_corner, dest.right - dest_corner),
      (dest.right - dest_corner, dest.right),
    ];
    let rows = [
      (dest.top, dest.top + dest_corner),
      (dest.top + dest_corner, dest.bottom - dest_corner),
      (dest.bottom - dest_corner, dest.bottom),
    ];

    let (clip, opacity) = (to_win32(clip), to_opacity(opacity));
    let mut parts = self.parts.iter_mut();

    for (row, (top, bottom)) in rows.into_iter().enumerate() {
      for (column, (left, right)) in columns.into_iter().enumerate() {
        if row == 1 && column == 1 && !self.is_filled {
          continue;
        }
        let Some(part) = parts.next() else {
          return;
        };

        if right <= left || bottom <= top {
          part.hide();
          continue;
        }

        let source = RECT {
          left: sources[column].0,
          top: sources[row].0,
          right: sources[column].1,
          bottom: sources[row].1,
        };
        let dest = RECT {
          left,
          top,
          right,
          bottom,
        };
        part.place(source, dest, &clip, opacity);
      }
    }
  }

  pub fn hide(&mut self) {
    for part in &mut self.parts {
      part.hide();
    }
  }
}

/// A ring or panel that glides to whatever it highlights.
pub(crate) struct Highlight<K> {
  pub slices: NineSlice,
  pub glide: Glide<K>,
}

impl<K: Copy + PartialEq> Highlight<K> {
  pub fn new(is_filled: bool) -> Option<Self> {
    Some(Self {
      slices: NineSlice::new(is_filled)?,
      glide: Glide::new(),
    })
  }

  /// Shows it where its glide has it, faded by `opacity`, cut to `clip`.
  pub fn place(&mut self, clip: &RectF, opacity: f32) {
    match self.glide.shown() {
      Some((rect, scale, alpha)) => {
        self.slices.place(&rect, scale, clip, alpha * opacity);
      }
      None => self.slices.hide(),
    }
  }
}

/// A picture that glides to whatever it is shown by, e.g. the title tag
/// under the picked window, redrawn only when `D` changes.
pub(crate) struct Tag<D> {
  pub picture: Picture,
  pub glide: Glide<isize>,
  pub drawn: Option<D>,
}

impl<D> Tag<D> {
  pub fn new() -> Option<Self> {
    Some(Self {
      picture: Picture::new()?,
      glide: Glide::new(),
      drawn: None,
    })
  }

  /// Shows it where its glide has it, cut to `clip`.
  pub fn place(&mut self, clip: &RectF, opacity: f32) {
    match self.glide.shown() {
      Some((rect, _, alpha)) => {
        self.picture.place(&rect, clip, alpha * opacity);
      }
      None => {
        if let Some(thumbnail) = &mut self.picture.thumbnail {
          thumbnail.hide();
        }
      }
    }
  }
}
