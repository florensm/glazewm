//! Copies of what is drawn with a window, its border ring, shadow and
//! backdrop, carried by a surrogate that stands in for the window while it
//! moves (a `WorkspaceSurrogate` or a `WindowPeek`).
//!
//! Each is a thumbnail of a window of its own, resting where the window
//! does (a ring parked by `NativeBorderOverlay::park`, or a
//! `WindowShadow`), so it moves in the same DWM thumbnail updates as the
//! window's copy rather than through its own window or visual tree, which
//! reach DWM on schedules of their own. The backdrop is cut from one that
//! spans the monitor instead (see `BackdropSource`).

use std::rc::Rc;

use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  backdrop_source::{BackdropSource, VignetteSource},
  dwm_thumbnail::Thumbnail,
  native_surrogate::placed_rects,
  window_shadow::{ShadowStyle, WindowShadow},
  BackdropOverlayParams, CornerStyle, NativeWindow, Rect,
};

/// A thumbnail of a window drawn around another one.
struct OverlayCopy {
  thumbnail: Thumbnail,
  /// The window the thumbnail is of.
  source: isize,
  /// Where `source` is.
  rest: Rect,
  /// Where the window it is drawn around was when `source` was put at
  /// `rest`, which it keeps to that window as it moves.
  window_rest: Rect,
}

impl OverlayCopy {
  /// Draws the copy around its window, now at `frame`, cut at `clip`, into
  /// a surrogate whose top-left is at `origin`.
  fn place(&mut self, frame: &Rect, clip: &Rect, origin: (i32, i32)) {
    match copy_rects(&self.rest, &self.window_rest, frame, clip, origin) {
      Some((rc_src, rc_dst)) => {
        let _ = self.thumbnail.show(rc_src, rc_dst);
      }
      None => self.thumbnail.hide(),
    }
  }
}

/// A window's shadow, with the copy of it.
struct ShadowCopy {
  /// Declared before `source`, so it is unregistered before that window
  /// goes.
  copy: OverlayCopy,
  source: WindowShadow,
}

/// A window's backdrop: the part of the monitor's behind the window, and
/// a copy of the window's vignette over it.
struct BackdropCopy {
  /// Declared before `source`, so it is unregistered before that window
  /// goes.
  crop: Thumbnail,
  vignette: Option<VignetteCopy>,
  source: Rc<BackdropSource>,
  params: BackdropOverlayParams,
  /// The window's rect and frame when carried, which its frame is mapped
  /// from as the window moves.
  window_rest: Rect,
  frame_rest: Rect,
  /// Whether it is drawn, and at what opacity when it is.
  is_shown: bool,
  opacity: u8,
}

/// A window's vignette, with the copy of it.
struct VignetteCopy {
  /// Declared before `source`, so it is unregistered before that window
  /// goes.
  copy: OverlayCopy,
  source: VignetteSource,
}

impl BackdropCopy {
  /// Draws the backdrop of its window, now at `frame`, cut at `clip`, into
  /// a surrogate whose top-left is at `origin`.
  fn place(&mut self, frame: &Rect, clip: &Rect, origin: (i32, i32)) {
    let window_frame =
      map_rect(&self.frame_rest, &self.window_rest, frame);
    let monitor = self.source.monitor();

    match window_frame.intersection(clip) {
      Some(shown) => {
        let _ = self.crop.show(
          shifted(&shown, (monitor.left, monitor.top)),
          shifted(&shown, origin),
        );
      }
      None => self.crop.hide(),
    }

    if let Some(vignette) = &mut self.vignette {
      vignette.copy.place(frame, clip, origin);
    }
  }

  /// Applies whether it is shown, and its opacity, to its thumbnails.
  fn apply_opacity(&mut self) {
    let opacity = if self.is_shown { self.opacity } else { 0 };
    let _ = self.crop.set_opacity(opacity);
    if let Some(vignette) = &mut self.vignette {
      let _ = vignette.copy.thumbnail.set_opacity(opacity);
    }
  }
}

/// The copies of a window's ring, shadow and backdrop that a surrogate
/// draws.
///
/// Must be dropped, or [`clear`](Self::clear)ed, before the surrogate's
/// own window is destroyed.
#[derive(Default)]
pub(crate) struct OverlayCopies {
  ring: Option<OverlayCopy>,
  shadow: Option<ShadowCopy>,
  backdrop: Option<BackdropCopy>,
}

impl OverlayCopies {
  /// Draws the ring of `source`, an overlay window at `rest` around a
  /// window at `window_rest`, on `dest` from now on. Called again with the
  /// same `source`, only updates where the two are, e.g. for a width
  /// change that resized the ring. Returns whether it does.
  pub(crate) fn carry_ring(
    &mut self,
    dest: HWND,
    source: HWND,
    rest: &Rect,
    window_rest: &Rect,
  ) -> bool {
    if let Some(ring) =
      self.ring.as_mut().filter(|ring| ring.source == source.0)
    {
      ring.rest = rest.clone();
      ring.window_rest = window_rest.clone();
      return true;
    }

    self.ring =
      Thumbnail::register(dest, source).map(|thumbnail| OverlayCopy {
        thumbnail,
        source: source.0,
        rest: rest.clone(),
        window_rest: window_rest.clone(),
      });
    self.ring.is_some()
  }

  pub(crate) fn carries_ring(&self) -> bool {
    self.ring.is_some()
  }

  /// Draws `window`'s shadow on `dest` from now on, around its `frame`
  /// while the window is at `window_rest`, as for the window drawn active
  /// or not. Returns whether it does: not for a window DWM draws no
  /// shadow around.
  ///
  /// After [`carry_ring`](Self::carry_ring), so the shadow falls over the
  /// ring as on screen: thumbnails stack in the order they're registered.
  pub(crate) fn carry_shadow(
    &mut self,
    dest: HWND,
    window: &NativeWindow,
    window_rest: &Rect,
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
            window_rest: window_rest.clone(),
          },
          source,
        }
      });
    self.shadow.is_some()
  }

  /// Draws the backdrop `params` give a window at `window_rest`, with its
  /// frame at `frame`, on `monitor`, on `dest` from now on, shown once
  /// [`show_backdrop`](Self::show_backdrop) says so. Returns whether it
  /// does; the same `params` again keep what is drawn.
  ///
  /// Before anything else is registered on `dest`, as it goes under the
  /// window: the caller then registers the window's own thumbnail anew,
  /// on top (see `NativeSurrogate::raise_thumbnail`).
  pub(crate) fn carry_backdrop(
    &mut self,
    dest: HWND,
    params: BackdropOverlayParams,
    monitor: &Rect,
    window_rest: &Rect,
    frame: &Rect,
  ) -> bool {
    if self
      .backdrop
      .as_ref()
      .is_some_and(|backdrop| backdrop.params == params)
    {
      return true;
    }

    let (is_shown, opacity) =
      self.backdrop.as_ref().map_or((false, 0), |backdrop| {
        (backdrop.is_shown, backdrop.opacity)
      });
    // Unregistered before the new one is, so it doesn't stack under it.
    self.backdrop = None;

    let source = match BackdropSource::acquire(monitor, params) {
      Ok(source) => source,
      Err(err) => {
        tracing::warn!("Failed to draw a window's backdrop: {err}");
        return false;
      }
    };
    let Some(crop) = Thumbnail::register(dest, source.hwnd()) else {
      return false;
    };

    // Strength as the overlay composites it, under its own opacity.
    let strength = params.vignette * params.opacity;
    let vignette = (strength > 0.0)
      .then(|| VignetteSource::create(frame, strength))
      .and_then(|source| {
        source
          .inspect_err(|err| {
            tracing::warn!("Failed to draw a window's vignette: {err}");
          })
          .ok()
      })
      .and_then(|source| {
        Thumbnail::register(dest, source.hwnd()).map(|thumbnail| {
          VignetteCopy {
            copy: OverlayCopy {
              thumbnail,
              source: source.hwnd().0,
              rest: source.rect().clone(),
              window_rest: window_rest.clone(),
            },
            source,
          }
        })
      });

    let mut backdrop = BackdropCopy {
      crop,
      vignette,
      source,
      params,
      window_rest: window_rest.clone(),
      frame_rest: frame.clone(),
      is_shown,
      opacity,
    };
    backdrop.apply_opacity();
    self.backdrop = Some(backdrop);
    true
  }

  pub(crate) fn carries_backdrop(&self) -> bool {
    self.backdrop.is_some()
  }

  /// The style the backdrop is drawn in, if it is.
  pub(crate) fn backdrop_params(&self) -> Option<BackdropOverlayParams> {
    self.backdrop.as_ref().map(|backdrop| backdrop.params)
  }

  /// Draws the backdrop or not from now on.
  pub(crate) fn show_backdrop(&mut self, is_shown: bool) {
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.is_shown = is_shown;
      backdrop.apply_opacity();
    }
  }

  /// Draws the backdrop, while shown, at `opacity`, independently of the
  /// ring and shadow.
  pub(crate) fn set_backdrop_opacity(&mut self, opacity: u8) {
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.opacity = opacity;
      backdrop.apply_opacity();
    }
  }

  /// Moves the shadow around the window's `frame` while the window is at
  /// `window_rest`, e.g. once the window has been moved or resized.
  pub(crate) fn move_shadow(&mut self, window_rest: &Rect, frame: &Rect) {
    let Some(shadow) = &mut self.shadow else {
      return;
    };

    if let Err(err) = shadow.source.set_frame(frame) {
      tracing::warn!("Failed to move a window's shadow: {err}");
    }
    shadow.copy.rest = shadow.source.rect().clone();
    shadow.copy.window_rest = window_rest.clone();
  }

  /// Moves the backdrop's vignette over the window's `frame` while the
  /// window is at `window_rest`, which the backdrop then follows from.
  pub(crate) fn move_backdrop(
    &mut self,
    window_rest: &Rect,
    frame: &Rect,
  ) {
    let Some(backdrop) = &mut self.backdrop else {
      return;
    };

    backdrop.window_rest = window_rest.clone();
    backdrop.frame_rest = frame.clone();

    if let Some(vignette) = &mut backdrop.vignette {
      if let Err(err) = vignette.source.set_frame(frame) {
        tracing::warn!("Failed to move a window's vignette: {err}");
      }
      vignette.copy.rest = vignette.source.rect().clone();
      vignette.copy.window_rest = window_rest.clone();
    }
  }

  /// Draws them all with their window, now at `frame`, cut at `clip`,
  /// into a surrogate whose top-left is at `origin`.
  pub(crate) fn place(
    &mut self,
    frame: &Rect,
    clip: &Rect,
    origin: (i32, i32),
  ) {
    for copy in self.copies() {
      copy.place(frame, clip, origin);
    }
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.place(frame, clip, origin);
    }
  }

  /// Draws them all at `opacity`.
  pub(crate) fn set_opacity(&mut self, opacity: u8) {
    for copy in self.copies() {
      let _ = copy.thumbnail.set_opacity(opacity);
    }
    self.set_backdrop_opacity(opacity);
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

  /// Draws the shadow at `opacity`, independently of the ring.
  pub(crate) fn set_shadow_opacity(&mut self, opacity: u8) {
    if let Some(shadow) = &mut self.shadow {
      let _ = shadow.copy.thumbnail.set_opacity(opacity);
    }
  }

  pub(crate) fn hide_shadow(&mut self) {
    if let Some(shadow) = &mut self.shadow {
      shadow.copy.thumbnail.hide();
    }
  }

  /// Unregisters them all, before the window they are drawn on goes.
  pub(crate) fn clear(&mut self) {
    self.ring = None;
    self.shadow = None;
    self.backdrop = None;
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
pub(crate) fn copy_rects(
  copy_rest: &Rect,
  window_rest: &Rect,
  frame: &Rect,
  clip: &Rect,
  origin: (i32, i32),
) -> Option<(RECT, RECT)> {
  let copy_frame = map_rect(copy_rest, window_rest, frame);
  let visible = copy_frame.intersection(clip)?;

  placed_rects(
    &copy_frame,
    &visible,
    (copy_rest.width(), copy_rest.height()),
    RECT::default(),
    origin,
  )
}

/// `rect`, which goes with a window resting at `window_rest`, moved and
/// stretched along with that window to `frame`.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub(crate) fn map_rect(
  rect: &Rect,
  window_rest: &Rect,
  frame: &Rect,
) -> Rect {
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

  Rect::from_ltrb(x(rect.left), y(rect.top), x(rect.right), y(rect.bottom))
}

/// `rect` in the coordinates of a window whose top-left is at `origin`.
fn shifted(rect: &Rect, origin: (i32, i32)) -> RECT {
  RECT {
    left: rect.left - origin.0,
    top: rect.top - origin.1,
    right: rect.right - origin.0,
    bottom: rect.bottom - origin.1,
  }
}

#[cfg(test)]
mod tests {
  use super::{copy_rects, map_rect, shifted, Rect, RECT};

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

  /// `(rcSource, rcDestination)` of a backdrop cut from a monitor at
  /// (0, 0) for a window whose frame rests at (100, 100, 800x600) inside a
  /// rect 7 px wider on each side, with that rect moved to `rect`.
  fn crop_at(rect: &Rect) -> Option<(Edges, Edges)> {
    let window = Rect::from_xy(93, 100, 814, 607);
    let frame = Rect::from_xy(100, 100, 800, 600);

    map_rect(&frame, &window, rect)
      .intersection(&MONITOR)
      .map(|shown| {
        (ltrb(shifted(&shown, (0, 0))), ltrb(shifted(&shown, (0, 0))))
      })
  }

  #[test]
  fn backdrop_shows_the_desktop_behind_its_window_as_it_moves() {
    assert_eq!(
      crop_at(&Rect::from_xy(393, 100, 814, 607)),
      Some(((400, 100, 1200, 700), (400, 100, 1200, 700)))
    );
  }

  #[test]
  fn backdrop_is_cut_at_the_clip() {
    assert_eq!(
      crop_at(&Rect::from_xy(1493, 100, 814, 607)),
      Some(((1500, 100, 1920, 700), (1500, 100, 1920, 700)))
    );
  }

  #[test]
  fn copy_off_the_clip_is_not_drawn() {
    assert_eq!(ring_at(&Rect::from_xy(2000, 100, 800, 600)), None);
  }
}
