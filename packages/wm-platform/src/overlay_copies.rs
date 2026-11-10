//! Copies of what is drawn with a window, its border ring and backdrop,
//! carried by a surrogate that stands in for the window while it moves (a
//! `WorkspaceSurrogate` or a `WindowPeek`).
//!
//! Each is a thumbnail of a window of its own, resting where the window
//! does (a ring parked by `NativeBorderOverlay::park`), so it moves in the
//! same DWM thumbnail updates as the
//! window's copy rather than through its own window or visual tree, which
//! reach DWM on schedules of their own. The backdrop is cut from one that
//! spans the monitor instead (see `BackdropSource`).

use std::rc::Rc;

use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  backdrop_source::{BackdropSource, VignetteSource},
  dwm_thumbnail::Thumbnail,
  isolation, lerp_f32,
  native_surrogate::placed_rects,
  BackdropOverlayParams, Rect, StyleBlend,
};

/// Where a surrogate draws its window: at `frame`, cut at `clip`, into a
/// surrogate whose top-left is at `origin`.
#[derive(Clone)]
struct Placement {
  frame: Rect,
  clip: Rect,
  origin: (i32, i32),
}

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
  /// Draws the copy around its window as `placement` places it.
  fn place(&mut self, placement: &Placement) {
    let Placement {
      frame,
      clip,
      origin,
    } = placement;

    match copy_rects(&self.rest, &self.window_rest, frame, clip, *origin) {
      Some((rc_src, rc_dst)) => {
        let _ = self.thumbnail.show(rc_src, rc_dst);
      }
      None => self.thumbnail.hide(),
    }
  }
}

/// The part behind a window of a backdrop spanning its monitor, in one
/// style.
struct CropCopy {
  /// Declared before `source`, so it is unregistered before that window
  /// goes.
  thumbnail: Thumbnail,
  source: Rc<BackdropSource>,
  params: BackdropOverlayParams,
}

impl CropCopy {
  /// Draws the crop for a window with its frame at `window_frame`, cut and
  /// placed as `placement` says.
  fn place(&mut self, window_frame: &Rect, placement: &Placement) {
    let monitor = self.source.monitor();

    match window_frame.intersection(&placement.clip) {
      Some(shown) => {
        let _ = self.thumbnail.show(
          shifted(&shown, (monitor.left, monitor.top)),
          shifted(&shown, placement.origin),
        );
      }
      None => self.thumbnail.hide(),
    }
  }
}

/// A window's backdrop in each style it may be drawn in, of which one is
/// shown at a time, or two crossfading: per style, the part of the
/// monitor's behind the window, and over them all one copy of the window's
/// vignette.
///
/// All are registered up front, so a change of style, e.g. as focus moves
/// mid-scroll, crossfades between them rather than registering a new one,
/// whose window would have yet to be composited. The vignette is drawn at
/// full strength and its copy's opacity scaled to the strength of the
/// style shown, which is what the vignette's own opacity does.
struct BackdropCopy {
  /// The styles asked for, which `crops` holds those of that could be
  /// drawn.
  styles: Vec<BackdropOverlayParams>,
  /// In the order registered, so each draws over the ones before it.
  crops: Vec<CropCopy>,
  vignette: Option<VignetteCopy>,
  /// The window's rect and frame when carried, which its frame is mapped
  /// from as the window moves.
  window_rest: Rect,
  frame_rest: Rect,
  /// Indices into `crops` of the style drawn and of one it fades from,
  /// `t` of the way, if any. Settled, `from` is `current` and `t` is 1.
  current: Option<usize>,
  from: Option<usize>,
  t: f32,
  /// Whether it is drawn, and at what opacity when it is.
  is_shown: bool,
  opacity: u8,
  /// Where it was last placed.
  placement: Option<Placement>,
}

/// A window's vignette, with the copy of it.
struct VignetteCopy {
  /// Declared before `source`, so it is unregistered before that window
  /// goes.
  copy: OverlayCopy,
  source: VignetteSource,
}

impl BackdropCopy {
  /// Draws the backdrop of its window as `placement` places it.
  fn place(&mut self, placement: &Placement) {
    self.placement = Some(placement.clone());
    self.place_current();
  }

  /// Draws the style shown where the backdrop was last placed.
  fn place_current(&mut self) {
    let Some(placement) = &self.placement else {
      return;
    };
    let window_frame =
      map_rect(&self.frame_rest, &self.window_rest, &placement.frame);

    for index in [self.from, self.current].into_iter().flatten() {
      if let Some(crop) = self.crops.get_mut(index) {
        crop.place(&window_frame, placement);
      }
    }
    if let Some(vignette) = &mut self.vignette {
      vignette.copy.place(placement);
    }
  }

  /// Draws the style `blend` fades to, `t` of the way over the one it
  /// fades from, of those carried, or none.
  #[allow(clippy::float_cmp)]
  fn show_style_blend(
    &mut self,
    blend: Option<&StyleBlend<BackdropOverlayParams>>,
  ) {
    let index = |params: &BackdropOverlayParams| {
      self.crops.iter().position(|crop| &crop.params == params)
    };
    let (from, current, t) = blend.map_or((None, None, 1.0), |blend| {
      crop_fade(index(&blend.from), index(&blend.to), blend.t)
    });
    if (from, current, t) == (self.from, self.current, self.t) {
      return;
    }

    let placed = [self.from, self.current];
    let adds_crop = [from, current]
      .into_iter()
      .any(|index| index.is_some() && !placed.contains(&index));
    self.from = from;
    self.current = current;
    self.t = t;
    if adds_crop {
      self.place_current();
    }
    self.apply_opacity();
  }

  /// How opaque each crop is drawn, as a share of the backdrop's opacity.
  ///
  /// Mid-fade, the lower of the two is drawn whole and the upper one by
  /// how far the fade is toward it: two opaque crops faded against each
  /// other would let what is behind them show through.
  fn crop_weights(&self) -> Vec<f32> {
    crop_weights(self.crops.len(), self.from, self.current, self.t)
  }

  /// Applies whether it is shown, its opacity and its styles to its
  /// thumbnails: those shown more first, so a change of style never
  /// leaves a frame with neither.
  fn apply_opacity(&mut self) {
    let opacity = left_out(
      if self.is_shown { self.opacity } else { 0 },
      isolation::current().no_backdrop,
    );
    let weights = self.crop_weights();

    let mut order = (0..self.crops.len()).collect::<Vec<_>>();
    order.sort_by(|a, b| weights[*b].total_cmp(&weights[*a]));
    for index in order {
      let _ = self.crops[index]
        .thumbnail
        .set_opacity(scale_alpha(opacity, weights[index]));
    }

    let strength_of = |index: Option<usize>| {
      index
        .and_then(|index| self.crops.get(index))
        .map_or(0.0, |crop| vignette_strength(&crop.params))
    };
    let strength =
      lerp_f32(strength_of(self.from), strength_of(self.current), self.t);
    if let Some(vignette) = &mut self.vignette {
      let _ = vignette
        .copy
        .thumbnail
        .set_opacity(scale_alpha(opacity, strength));
    }
  }
}

/// The crops a fade from crop `from` to crop `to`, `t` of the way, draws,
/// as `(from, current, t)` for [`crop_weights`].
///
/// Settled on `to` once done, or when both are one crop: styles with the
/// same backdrop share a crop, which fading against itself would draw at
/// `t`, leaving the window without its backdrop at the fade's start.
fn crop_fade(
  from: Option<usize>,
  to: Option<usize>,
  t: f32,
) -> (Option<usize>, Option<usize>, f32) {
  if t < 1.0 && from != to {
    (from, to, t.max(0.0))
  } else {
    (to, to, 1.0)
  }
}

/// [`BackdropCopy::crop_weights`] of `count` crops, drawn in `current`'s
/// style, `t` of the way from `from`'s.
fn crop_weights(
  count: usize,
  from: Option<usize>,
  current: Option<usize>,
  t: f32,
) -> Vec<f32> {
  let mut weights = vec![0.0; count];
  let mut set = |index: usize, weight: f32| {
    if let Some(slot) = weights.get_mut(index) {
      *slot = weight;
    }
  };

  match (from, current) {
    (Some(from), Some(to)) if from != to => {
      let upper = from.max(to);
      set(from.min(to), 1.0);
      set(upper, if upper == to { t } else { 1.0 - t });
    }
    (_, Some(to)) => set(to, t),
    (Some(from), None) => set(from, 1.0 - t),
    (None, None) => {}
  }
  weights
}

/// `opacity`, or none for a copy `GLAZEWM_ISOLATE` leaves out (see
/// `isolation`).
fn left_out(opacity: u8, is_left_out: bool) -> u8 {
  if is_left_out {
    0
  } else {
    opacity
  }
}

/// Strength of the vignette `params` give, as the overlay composites it,
/// under its own opacity.
fn vignette_strength(params: &BackdropOverlayParams) -> f32 {
  (params.vignette * params.opacity).clamp(0.0, 1.0)
}

/// `alpha` scaled by `frac` (0.0–1.0).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn scale_alpha(alpha: u8, frac: f32) -> u8 {
  (f32::from(alpha) * frac.clamp(0.0, 1.0)).round() as u8
}

/// The copies of a window's ring and backdrop that a surrogate draws.
///
/// Must be dropped, or [`clear`](Self::clear)ed, before the surrogate's
/// own window is destroyed.
#[derive(Default)]
pub(crate) struct OverlayCopies {
  ring: Option<OverlayCopy>,
  backdrop: Option<BackdropCopy>,
  /// Where they were last placed.
  placement: Option<Placement>,
}

impl OverlayCopies {
  /// Draws the ring of `source`, an overlay window at `rest` around a
  /// window at `window_rest`, on `dest` from now on, where the others were
  /// last placed if they were. Called again with the same `source`, only
  /// updates where the two are, e.g. for a width change that resized the
  /// ring. Returns whether it does.
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
    if let (Some(ring), Some(placement)) =
      (&mut self.ring, &self.placement)
    {
      ring.place(placement);
    }
    self.ring.is_some()
  }

  pub(crate) fn carries_ring(&self) -> bool {
    self.ring.is_some()
  }

  /// Registers the ring copy on `dest` anew as it is, so it stacks above
  /// thumbnails registered there since. The new one goes up before the
  /// old one goes, so no composition shows neither.
  pub(crate) fn raise_ring(&mut self, dest: HWND) {
    let Some(ring) = &mut self.ring else {
      return;
    };

    if let Some(thumbnail) = Thumbnail::register_with(
      dest,
      HWND(ring.source),
      *ring.thumbnail.look(),
    ) {
      ring.thumbnail = thumbnail;
    }
  }

  /// Draws the backdrops `styles` give a window at `window_rest`, with its
  /// frame at `frame`, on `monitor`, on `dest` from now on, in whichever
  /// of them [`show_backdrop_blend`](Self::show_backdrop_blend) picks,
  /// once [`show_backdrop`](Self::show_backdrop) says so. Returns whether
  /// it does; the same `styles` again keep what is drawn.
  ///
  /// Before anything else is registered on `dest`, as it goes under the
  /// window: the caller then registers the window's own thumbnail anew,
  /// on top (see `NativeSurrogate::raise_thumbnail`).
  pub(crate) fn carry_backdrop(
    &mut self,
    dest: HWND,
    styles: &[BackdropOverlayParams],
    monitor: &Rect,
    window_rest: &Rect,
    frame: &Rect,
  ) -> bool {
    if self.carries_backdrop_styles(styles) {
      return true;
    }

    // Unregistered before the new ones are, so they don't stack under
    // them.
    let (is_shown, opacity, blend, placement) = self
      .backdrop
      .take()
      .map_or((false, 0, None, None), |backdrop| {
        let params = |index: Option<usize>| {
          index
            .and_then(|index| backdrop.crops.get(index))
            .map(|crop| crop.params)
        };
        let blend = StyleBlend::between(
          params(backdrop.from),
          params(backdrop.current),
          backdrop.t,
          |params| BackdropOverlayParams {
            opacity: 0.0,
            ..params
          },
        );
        (
          backdrop.is_shown,
          backdrop.opacity,
          blend,
          backdrop.placement,
        )
      });

    let mut crops = Vec::<CropCopy>::new();
    for &params in styles {
      if crops.iter().any(|crop| crop.params == params) {
        continue;
      }

      let source = match BackdropSource::acquire(monitor, params) {
        Ok(source) => source,
        Err(err) => {
          tracing::warn!("Failed to draw a window's backdrop: {err}");
          continue;
        }
      };
      if let Some(thumbnail) = Thumbnail::register(dest, source.hwnd()) {
        crops.push(CropCopy {
          thumbnail,
          source,
          params,
        });
      }
    }
    if crops.is_empty() {
      return false;
    }

    let has_vignette = crops
      .iter()
      .any(|crop| vignette_strength(&crop.params) > 0.0);
    let vignette = has_vignette
      .then(|| VignetteSource::create(frame, 1.0))
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
      styles: styles.to_vec(),
      crops,
      vignette,
      window_rest: window_rest.clone(),
      frame_rest: frame.clone(),
      current: None,
      from: None,
      t: 1.0,
      is_shown,
      opacity,
      placement,
    };
    backdrop.show_style_blend(blend.as_ref());
    self.backdrop = Some(backdrop);
    true
  }

  pub(crate) fn carries_backdrop(&self) -> bool {
    self.backdrop.is_some()
  }

  /// Whether the backdrop is drawn in exactly `styles`.
  pub(crate) fn carries_backdrop_styles(
    &self,
    styles: &[BackdropOverlayParams],
  ) -> bool {
    self
      .backdrop
      .as_ref()
      .is_some_and(|backdrop| same_set(&backdrop.styles, styles))
  }

  /// Draws the backdrop in the style `blend` fades to, over the one it
  /// fades from, from now on, of those carried, or in none.
  pub(crate) fn show_backdrop_blend(
    &mut self,
    blend: Option<&StyleBlend<BackdropOverlayParams>>,
  ) {
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.show_style_blend(blend);
    }
  }

  /// Draws the backdrop or not from now on.
  pub(crate) fn show_backdrop(&mut self, is_shown: bool) {
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.is_shown = is_shown;
      backdrop.apply_opacity();
    }
  }

  /// Draws the backdrop, while shown, at `opacity`, independently of the
  /// ring.
  pub(crate) fn set_backdrop_opacity(&mut self, opacity: u8) {
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.opacity = opacity;
      backdrop.apply_opacity();
    }
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
    let placement = Placement {
      frame: frame.clone(),
      clip: clip.clone(),
      origin,
    };

    if let Some(ring) = &mut self.ring {
      ring.place(&placement);
    }
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.place(&placement);
    }
    self.placement = Some(placement);
  }

  /// Draws the ring and backdrop at `opacity`.
  pub(crate) fn set_opacity(&mut self, opacity: u8) {
    self.set_ring_opacity(opacity);
    self.set_backdrop_opacity(opacity);
  }

  /// Draws the ring at `opacity`, independently of the backdrop.
  pub(crate) fn set_ring_opacity(&mut self, opacity: u8) {
    if let Some(ring) = &mut self.ring {
      let _ = ring
        .thumbnail
        .set_opacity(left_out(opacity, isolation::current().no_ring));
    }
  }

  /// Brings the ring copy to full opacity.
  pub(crate) fn cover_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      let _ = ring
        .thumbnail
        .set_opacity(left_out(u8::MAX, isolation::current().no_ring));
    }
  }

  pub(crate) fn hide_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      ring.thumbnail.hide();
    }
  }

  /// Unregisters them all, before the window they are drawn on goes.
  pub(crate) fn clear(&mut self) {
    self.ring = None;
    self.backdrop = None;
    self.placement = None;
  }
}

/// Whether `a` and `b` hold the same items, ignoring order and repeats.
fn same_set<T: PartialEq>(a: &[T], b: &[T]) -> bool {
  a.iter().all(|item| b.contains(item))
    && b.iter().all(|item| a.contains(item))
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
  use super::{
    copy_rects, crop_fade, crop_weights, map_rect, same_set, scale_alpha,
    shifted, vignette_strength, BackdropOverlayParams, Rect, RECT,
  };
  use crate::Color;

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

  #[test]
  fn same_set_ignores_order_and_repeats() {
    assert!(same_set(&[1, 2], &[2, 1, 2]));
    assert!(!same_set(&[1, 2], &[1]));
    assert!(!same_set(&[1], &[1, 3]));
    assert!(same_set::<i32>(&[], &[]));
  }

  /// A full-strength vignette's copy drawn at the strength of a style
  /// darkens as that style's own vignette does.
  #[test]
  fn vignette_copy_opacity_carries_the_style_strength() {
    let params = BackdropOverlayParams {
      tint: Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0x30,
      },
      blur_amount: 30.0,
      corner_radius: 0.0,
      opacity: 1.0,
      saturation: 1.0,
      exposure: 0.0,
      contrast: 0.0,
      highlights: 0.0,
      shadows: 0.0,
      vignette: 0.2,
      grain: 0.0,
      parallax: 1.0,
    };
    assert_eq!(scale_alpha(u8::MAX, vignette_strength(&params)), 51);

    let faded = BackdropOverlayParams {
      opacity: 0.5,
      vignette: 0.4,
      ..params
    };
    assert_eq!(scale_alpha(u8::MAX, vignette_strength(&faded)), 51);
    assert_eq!(scale_alpha(128, vignette_strength(&faded)), 26);
  }

  #[test]
  fn settled_style_draws_alone() {
    assert_eq!(crop_weights(2, Some(1), Some(1), 1.0), vec![0.0, 1.0]);
    assert_eq!(crop_weights(2, None, None, 1.0), vec![0.0, 0.0]);
  }

  #[test]
  fn crossfade_keeps_the_lower_crop_whole() {
    // Fading up to the upper crop draws it over the whole lower one.
    assert_eq!(crop_weights(2, Some(0), Some(1), 0.25), vec![1.0, 0.25]);
    // Fading down to the lower one fades the upper one out over it.
    assert_eq!(crop_weights(2, Some(1), Some(0), 0.25), vec![1.0, 0.75]);
  }

  #[test]
  fn fade_within_one_crop_draws_it_whole() {
    assert_eq!(crop_fade(Some(0), Some(0), 0.0), (Some(0), Some(0), 1.0));
    assert_eq!(crop_fade(None, None, 0.5), (None, None, 1.0));
  }

  #[test]
  fn fade_between_crops_runs_until_done() {
    assert_eq!(
      crop_fade(Some(0), Some(1), 0.25),
      (Some(0), Some(1), 0.25)
    );
    assert_eq!(crop_fade(Some(0), None, -0.1), (Some(0), None, 0.0));
    assert_eq!(crop_fade(Some(0), Some(1), 1.2), (Some(1), Some(1), 1.0));
  }

  #[test]
  fn crossfade_with_one_side_not_carried_fades_alone() {
    assert_eq!(crop_weights(2, None, Some(1), 0.25), vec![0.0, 0.25]);
    assert_eq!(crop_weights(2, Some(0), None, 0.25), vec![0.75, 0.0]);
  }
}
