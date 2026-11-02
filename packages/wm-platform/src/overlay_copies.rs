//! Copies of what is drawn around a window, its border ring and its
//! shadow, carried by a surrogate that stands in for the window while it
//! moves (a `WorkspaceSurrogate` or a `WindowPeek`).
//!
//! Each is a thumbnail of a window of its own, resting where the window
//! does (a ring parked by `NativeBorderOverlay::park`, or a
//! `WindowShadow`), so it moves in the same DWM thumbnail updates as the
//! window's copy rather than through its own window or visual tree, which
//! reach DWM on schedules of their own.

use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  dwm_thumbnail::Thumbnail,
  native_surrogate::placed_rects,
  window_shadow::{ShadowStyle, WindowShadow},
  CornerStyle, NativeWindow, Rect,
};

/// A thumbnail of a window drawn around another one.
struct OverlayCopy {
  thumbnail: Thumbnail,
  /// The window the thumbnail is of.
  source: isize,
  /// Where `source` is: its rect while the window rests.
  rest: Rect,
}

impl OverlayCopy {
  /// Draws the copy around a window resting at `window_rest`, now at
  /// `frame`, cut at `clip`, into a surrogate whose top-left is at
  /// `origin`.
  fn place(
    &mut self,
    window_rest: &Rect,
    frame: &Rect,
    clip: &Rect,
    origin: (i32, i32),
  ) {
    match copy_rects(&self.rest, window_rest, frame, clip, origin) {
      Some((rc_src, rc_dst)) => {
        let _ = self.thumbnail.show(rc_src, rc_dst);
      }
      None => self.thumbnail.hide(),
    }
  }
}

/// A window's shadow, with the copy of it.
struct ShadowCopy {
  /// Declared before `_source`, so it is unregistered before that window
  /// goes.
  copy: OverlayCopy,
  _source: WindowShadow,
}

/// The copies of a window's ring and shadow that a surrogate draws.
///
/// Must be dropped, or [`clear`](Self::clear)ed, before the surrogate's
/// own window is destroyed.
#[derive(Default)]
pub(crate) struct OverlayCopies {
  ring: Option<OverlayCopy>,
  shadow: Option<ShadowCopy>,
}

impl OverlayCopies {
  /// Draws the ring of `source`, an overlay window resting at `rest`, on
  /// `dest` from now on. Called again with the same `source`, only
  /// updates `rest`, e.g. for a width change that resized it. Returns
  /// whether it does.
  pub(crate) fn carry_ring(
    &mut self,
    dest: HWND,
    source: HWND,
    rest: &Rect,
  ) -> bool {
    if let Some(ring) =
      self.ring.as_mut().filter(|ring| ring.source == source.0)
    {
      ring.rest = rest.clone();
      return true;
    }

    self.ring =
      Thumbnail::register(dest, source).map(|thumbnail| OverlayCopy {
        thumbnail,
        source: source.0,
        rest: rest.clone(),
      });
    self.ring.is_some()
  }

  pub(crate) fn carries_ring(&self) -> bool {
    self.ring.is_some()
  }

  /// Draws `window`'s shadow on `dest` from now on, around its frame
  /// resting at `frame`, as for the window drawn active or not. Returns
  /// whether it does: not for a window DWM draws no shadow around.
  ///
  /// After [`carry_ring`](Self::carry_ring), so the shadow falls over the
  /// ring as on screen: thumbnails stack in the order they're registered.
  pub(crate) fn carry_shadow(
    &mut self,
    dest: HWND,
    window: &NativeWindow,
    frame: &Rect,
    is_active: bool,
    corner_style: CornerStyle,
  ) -> bool {
    let Some(style) =
      ShadowStyle::of(window.hwnd(), is_active, corner_style)
    else {
      return false;
    };

    let source = match WindowShadow::create(frame, style) {
      Ok(source) => source,
      Err(err) => {
        tracing::warn!("Failed to draw a window's shadow: {err}");
        return false;
      }
    };

    self.shadow =
      Thumbnail::register(dest, source.hwnd()).map(|thumbnail| {
        ShadowCopy {
          copy: OverlayCopy {
            thumbnail,
            source: source.hwnd().0,
            rest: source.rect().clone(),
          },
          _source: source,
        }
      });
    self.shadow.is_some()
  }

  /// Draws both around a window resting at `window_rest`, now at `frame`,
  /// cut at `clip`, into a surrogate whose top-left is at `origin`.
  pub(crate) fn place(
    &mut self,
    window_rest: &Rect,
    frame: &Rect,
    clip: &Rect,
    origin: (i32, i32),
  ) {
    for copy in self.copies() {
      copy.place(window_rest, frame, clip, origin);
    }
  }

  /// Draws both at `opacity`.
  pub(crate) fn set_opacity(&mut self, opacity: u8) {
    for copy in self.copies() {
      let _ = copy.thumbnail.set_opacity(opacity);
    }
  }

  /// Brings the ring copy to full opacity.
  pub(crate) fn cover_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      let _ = ring.thumbnail.set_opacity(u8::MAX);
    }
  }

  pub(crate) fn hide_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      ring.thumbnail.hide();
    }
  }

  pub(crate) fn hide_shadow(&mut self) {
    if let Some(shadow) = &mut self.shadow {
      shadow.copy.thumbnail.hide();
    }
  }

  /// Unregisters both, before the window they are drawn on goes.
  pub(crate) fn clear(&mut self) {
    self.ring = None;
    self.shadow = None;
  }

  fn copies(&mut self) -> impl Iterator<Item = &mut OverlayCopy> {
    [
      self.ring.as_mut(),
      self.shadow.as_mut().map(|shadow| &mut shadow.copy),
    ]
    .into_iter()
    .flatten()
  }
}

/// `(rcSource, rcDestination)` drawing a copy that rests at `copy_rest`,
/// around a window resting at `window_rest`, with that window now at
/// `frame`, cut at `clip`, into a surrogate whose top-left is at `origin`.
/// `None` when nothing is left.
///
/// The copy is stretched along with the window, which a window resized as
/// it moves is too.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub(crate) fn copy_rects(
  copy_rest: &Rect,
  window_rest: &Rect,
  frame: &Rect,
  clip: &Rect,
  origin: (i32, i32),
) -> Option<(RECT, RECT)> {
  let scale = |to: i32, from: i32| {
    if from > 0 {
      to as f32 / from as f32
    } else {
      1.0
    }
  };
  let (scale_x, scale_y) = (
    scale(frame.width(), window_rest.width()),
    scale(frame.height(), window_rest.height()),
  );
  let x = |x: i32| {
    frame.left + ((x - window_rest.left) as f32 * scale_x).round() as i32
  };
  let y = |y: i32| {
    frame.top + ((y - window_rest.top) as f32 * scale_y).round() as i32
  };

  let copy_frame = Rect::from_ltrb(
    x(copy_rest.left),
    y(copy_rest.top),
    x(copy_rest.right),
    y(copy_rest.bottom),
  );
  let visible = copy_frame.intersection(clip)?;

  placed_rects(
    &copy_frame,
    &visible,
    (copy_rest.width(), copy_rest.height()),
    RECT::default(),
    origin,
  )
}

#[cfg(test)]
mod tests {
  use super::{copy_rects, Rect, RECT};

  const MONITOR: Rect = Rect {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1080,
  };

  /// `(left, top, right, bottom)` of a `RECT`.
  type Edges = (i32, i32, i32, i32);

  fn ltrb(rect: RECT) -> Edges {
    (rect.left, rect.top, rect.right, rect.bottom)
  }

  /// `(rcSource, rcDestination)` of a 3 px ring around a window resting at
  /// (100, 100, 800x600), with the window at `frame`.
  fn ring_at(frame: &Rect) -> Option<(Edges, Edges)> {
    let window = Rect::from_xy(100, 100, 800, 600);

    copy_rects(&window.inset(-3), &window, frame, &MONITOR, (0, 0))
      .map(|(src, dst)| (ltrb(src), ltrb(dst)))
  }

  #[test]
  fn copy_moves_whole_with_its_window() {
    assert_eq!(
      ring_at(&Rect::from_xy(400, 100, 800, 600)),
      Some(((0, 0, 806, 606), (397, 97, 1203, 703)))
    );
  }

  #[test]
  fn copy_is_cut_at_the_clip() {
    assert_eq!(
      ring_at(&Rect::from_xy(1500, 100, 800, 600)),
      Some(((0, 0, 423, 606), (1497, 97, 1920, 703)))
    );
  }

  #[test]
  fn copy_scales_with_its_window() {
    assert_eq!(
      ring_at(&Rect::from_xy(200, 200, 400, 300)),
      Some(((0, 0, 806, 606), (198, 198, 602, 502)))
    );
  }

  #[test]
  fn copy_stretches_with_a_window_resized_on_one_axis() {
    assert_eq!(
      ring_at(&Rect::from_xy(100, 100, 400, 600)),
      Some(((0, 0, 806, 606), (98, 97, 502, 703)))
    );
  }

  #[test]
  fn copy_off_the_clip_is_not_drawn() {
    assert_eq!(ring_at(&Rect::from_xy(2000, 100, 800, 600)), None);
  }
}
