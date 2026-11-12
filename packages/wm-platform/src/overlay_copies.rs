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
  backdrop_source::{BackdropSource, FillSource, VignetteSource},
  dwm_thumbnail::{clip_rects, Look, Thumbnail},
  isolation, lerp_f32,
  native_surrogate::{fill_start, placed_rects},
  BackdropOverlayParams, Color, Rect, StyleBlend,
};

/// Where a surrogate draws its window: at `frame`, cut at `clip`, into a
/// surrogate whose top-left is at `origin`.
#[derive(Clone)]
struct Placement {
  frame: Rect,
  clip: Rect,
  origin: (i32, i32),
  /// The window's visible frame as drawn, when it is not `frame` scaled
  /// from where the window rested: one drawn 1:1 keeps its invisible
  /// borders' width, which scaling would not.
  visible_frame: Option<Rect>,
}

/// A thumbnail of a window drawn around another one.
struct OverlayCopy {
  thumbnail: Thumbnail,
  /// Where the window the thumbnail is of is.
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
      ..
    } = placement;

    match copy_rects(&self.rest, &self.window_rest, frame, clip, *origin) {
      Some((rc_src, rc_dst)) => {
        let _ = self.thumbnail.show(rc_src, rc_dst);
      }
      None => self.thumbnail.hide(),
    }
  }

  /// Draws the copy stretched over `rect`, cut and placed as `placement`
  /// says.
  fn place_over(&mut self, rect: &Rect, placement: &Placement) {
    match copy_rects(
      &self.rest,
      &self.rest,
      rect,
      &placement.clip,
      placement.origin,
    ) {
      Some((rc_src, rc_dst)) => {
        let _ = self.thumbnail.show(rc_src, rc_dst);
      }
      None => self.thumbnail.hide(),
    }
  }
}

/// One of the thumbnails a [`RingCopy`] is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingPart {
  /// The whole ring, stretched along with its window.
  Whole,
  /// One cell of the ring cut three by three, by column and row: its
  /// corners drawn 1:1, its edges stretched along their length only. The
  /// middle cell, inside the ring, is never drawn.
  Cell(Band, Band),
}

/// Which third of a ring cut three by three, along one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Band {
  Start,
  Middle,
  End,
}

impl RingPart {
  /// The parts a ring is drawn with: whole, or with `corner_reach`, in the
  /// eight cells around its middle.
  fn all(corner_reach: Option<i32>) -> Vec<Self> {
    const BANDS: [Band; 3] = [Band::Start, Band::Middle, Band::End];

    if corner_reach.is_none() {
      return vec![Self::Whole];
    }

    BANDS
      .iter()
      .flat_map(|&row| BANDS.iter().map(move |&column| (column, row)))
      .filter(|&cell| cell != (Band::Middle, Band::Middle))
      .map(|(column, row)| Self::Cell(column, row))
      .collect()
  }
}

/// A copy of a window's border ring: whole, scaled with its window, or in
/// cells, so a window resized on its copy keeps its ring's width and
/// corners (see [`ring_part_rects`]).
struct RingCopy {
  /// One thumbnail of `source` per part, in the order registered.
  parts: Vec<(RingPart, Thumbnail)>,
  source: isize,
  /// Where `source` is.
  rest: Rect,
  /// Where the window it is drawn around was when `source` was put at
  /// `rest`, which it keeps to that window as it moves.
  window_rest: Rect,
  /// How far in from its outer edge its corners reach, when drawn in
  /// cells.
  corner_reach: Option<i32>,
}

impl RingCopy {
  /// Draws the ring around its window as `placement` places it.
  fn place(&mut self, placement: &Placement) {
    let Placement {
      frame,
      clip,
      origin,
      ..
    } = placement;

    for (part, thumbnail) in &mut self.parts {
      match ring_part_rects(
        *part,
        &self.rest,
        &self.window_rest,
        frame,
        clip,
        *origin,
        self.corner_reach.unwrap_or_default(),
      ) {
        Some((rc_src, rc_dst)) => {
          let _ = thumbnail.show(rc_src, rc_dst);
        }
        None => thumbnail.hide(),
      }
    }
  }

  /// Registers the ring's parts on `dest` anew, cut by `corner_reach`, so
  /// they stack above thumbnails registered there since: as they were,
  /// or, cut anew, drawn as `placement` places them, at the opacity they
  /// were drawn at. All go up before the old ones go, so no composition
  /// shows neither; if any fails, the old ones stay. Returns whether the
  /// new ones went up.
  fn register_anew(
    &mut self,
    dest: HWND,
    corner_reach: Option<i32>,
    placement: Option<&Placement>,
  ) -> bool {
    let parts = if corner_reach == self.corner_reach {
      self
        .parts
        .iter()
        .map(|(part, thumbnail)| (*part, *thumbnail.look()))
        .collect::<Vec<_>>()
    } else {
      let opacity = self
        .parts
        .first()
        .map_or(0, |(_, thumbnail)| thumbnail.look().opacity);
      let is_shown = self
        .parts
        .iter()
        .any(|(_, thumbnail)| thumbnail.look().visible);
      self.looks(corner_reach, placement.filter(|_| is_shown), opacity)
    };

    let Some(parts) = register_all(dest, HWND(self.source), parts) else {
      return false;
    };
    self.parts = parts;
    self.corner_reach = corner_reach;
    true
  }

  /// Each part of the ring cut by `corner_reach`, with how it is drawn at
  /// `opacity` as `placement` places it, or hidden without one.
  fn looks(
    &self,
    corner_reach: Option<i32>,
    placement: Option<&Placement>,
    opacity: u8,
  ) -> Vec<(RingPart, Look)> {
    RingPart::all(corner_reach)
      .into_iter()
      .map(|part| {
        let rects = placement.and_then(|placement| {
          ring_part_rects(
            part,
            &self.rest,
            &self.window_rest,
            &placement.frame,
            &placement.clip,
            placement.origin,
            corner_reach.unwrap_or_default(),
          )
        });
        let look = rects.map_or(
          Look {
            opacity,
            ..Look::default()
          },
          |(source, dest)| Look {
            source: Some(source),
            dest,
            opacity,
            visible: true,
          },
        );
        (part, look)
      })
      .collect()
  }

  fn set_opacity(&mut self, opacity: u8) {
    for (_, thumbnail) in &mut self.parts {
      let _ = thumbnail.set_opacity(opacity);
    }
  }

  fn hide(&mut self) {
    for (_, thumbnail) in &mut self.parts {
      thumbnail.hide();
    }
  }
}

/// Registers a thumbnail of `source` on `dest` drawing each of `parts` as
/// its look says from its first frame, or none if any fails.
fn register_all(
  dest: HWND,
  source: HWND,
  parts: Vec<(RingPart, Look)>,
) -> Option<Vec<(RingPart, Thumbnail)>> {
  parts
    .into_iter()
    .map(|(part, look)| {
      Thumbnail::register_with(dest, source, look)
        .map(|thumbnail| (part, thumbnail))
    })
    .collect()
}

/// The two strips of one color that cover the part of a window's frame
/// its content does not reach (see [`fill_strips`]).
struct FillCopy {
  /// Declared before `source`, so they are unregistered before that
  /// window goes.
  right: Thumbnail,
  bottom: Thumbnail,
  source: Rc<FillSource>,
  color: Color,
  /// Whether it is drawn, and at what opacity when it is.
  is_shown: bool,
  opacity: u8,
}

impl FillCopy {
  /// Draws the strips around content covering `covered` of a frame at
  /// `frame`, cut at `clip`, into a surrogate whose top-left is at
  /// `origin`; none with no `covered`.
  fn place(
    &mut self,
    frame: &Rect,
    covered: Option<(i32, i32)>,
    clip: &Rect,
    origin: (i32, i32),
  ) {
    let strips = covered.map_or([None, None], |covered| {
      fill_strips(frame, covered, clip, origin)
    });

    for (thumbnail, strip) in
      [&mut self.right, &mut self.bottom].into_iter().zip(strips)
    {
      match strip {
        Some(dest) => {
          let _ = thumbnail.show(FillSource::sample_rect(), dest);
        }
        None => thumbnail.hide(),
      }
    }
  }

  /// Applies whether it is shown and its opacity to its thumbnails.
  fn apply_opacity(&mut self) {
    let opacity = if self.is_shown { self.opacity } else { 0 };
    let _ = self.right.set_opacity(opacity);
    let _ = self.bottom.set_opacity(opacity);
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
      placement.visible_frame.clone().unwrap_or_else(|| {
        map_rect(&self.frame_rest, &self.window_rest, &placement.frame)
      });

    for index in [self.from, self.current].into_iter().flatten() {
      if let Some(crop) = self.crops.get_mut(index) {
        crop.place(&window_frame, placement);
      }
    }
    if let Some(vignette) = &mut self.vignette {
      match &placement.visible_frame {
        Some(visible_frame) => {
          vignette.copy.place_over(visible_frame, placement);
        }
        None => vignette.copy.place(placement),
      }
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
  ring: Option<RingCopy>,
  backdrop: Option<BackdropCopy>,
  fill: Option<FillCopy>,
  /// Where they were last placed.
  placement: Option<Placement>,
}

impl OverlayCopies {
  /// Draws the ring of `source`, an overlay window at `rest` around a
  /// window at `window_rest`, on `dest` from now on, where the others were
  /// last placed if they were: whole, or with `corner_reach` (see
  /// `NativeBorderOverlay::corner_reach`), in cells. Called again with the
  /// same `source`, only updates where the two are, e.g. for a width
  /// change that resized the ring, and how it is cut. Returns whether it
  /// does.
  pub(crate) fn carry_ring(
    &mut self,
    dest: HWND,
    source: HWND,
    rest: &Rect,
    window_rest: &Rect,
    corner_reach: Option<i32>,
  ) -> bool {
    if let Some(ring) =
      self.ring.as_mut().filter(|ring| ring.source == source.0)
    {
      ring.rest = rest.clone();
      ring.window_rest = window_rest.clone();
      if ring.corner_reach != corner_reach {
        ring.register_anew(dest, corner_reach, self.placement.as_ref());
      }
      return true;
    }

    let ring = RingCopy {
      parts: Vec::new(),
      source: source.0,
      rest: rest.clone(),
      window_rest: window_rest.clone(),
      corner_reach,
    };
    let looks = ring.looks(corner_reach, self.placement.as_ref(), 0);
    self.ring = register_all(dest, source, looks)
      .map(|parts| RingCopy { parts, ..ring });
    self.ring.is_some()
  }

  /// Cuts the ring copy by `corner_reach` from now on, as
  /// [`carry_ring`](Self::carry_ring) does, leaving where it is as it is.
  pub(crate) fn cut_ring(
    &mut self,
    dest: HWND,
    corner_reach: Option<i32>,
  ) {
    if let Some(ring) = self
      .ring
      .as_mut()
      .filter(|ring| ring.corner_reach != corner_reach)
    {
      ring.register_anew(dest, corner_reach, self.placement.as_ref());
    }
  }

  /// Where the ring's window was when it was last carried.
  pub(crate) fn ring_rest(&self) -> Option<&Rect> {
    self.ring.as_ref().map(|ring| &ring.rest)
  }

  pub(crate) fn carries_ring(&self) -> bool {
    self.ring.is_some()
  }

  /// Registers the ring copy on `dest` anew as it is, so it stacks above
  /// thumbnails registered there since. The new one goes up before the
  /// old one goes, so no composition shows neither.
  pub(crate) fn raise_ring(&mut self, dest: HWND) {
    if let Some(ring) = &mut self.ring {
      ring.register_anew(dest, ring.corner_reach, None);
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
    self.raise_fill(dest);
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

  /// Covers the part of the window's frame its content does not reach in
  /// `color` from now on, once [`place_fill`](Self::place_fill) says
  /// where: two strips over the backdrop, under the ring. Over the
  /// window's own thumbnail too, which they only reach into by the rows
  /// its hidden border leaves see-through (see `fill_start`). Returns
  /// whether it does; the same `color` again keeps what is drawn.
  pub(crate) fn carry_fill(&mut self, dest: HWND, color: Color) -> bool {
    if self.fill.as_ref().is_some_and(|fill| fill.color == color) {
      return true;
    }

    let source = match FillSource::acquire(color) {
      Ok(source) => source,
      Err(err) => {
        tracing::warn!("Failed to draw a window's fill: {err}");
        return false;
      }
    };
    // The new strips go up as the old ones were drawn, before they go.
    let (looks, is_shown, opacity) = self.fill.as_ref().map_or_else(
      || ((Look::default(), Look::default()), false, 0),
      |fill| {
        (
          (*fill.right.look(), *fill.bottom.look()),
          fill.is_shown,
          fill.opacity,
        )
      },
    );
    let (Some(right), Some(bottom)) = (
      Thumbnail::register_with(dest, source.hwnd(), looks.0),
      Thumbnail::register_with(dest, source.hwnd(), looks.1),
    ) else {
      return false;
    };

    self.fill = Some(FillCopy {
      right,
      bottom,
      source,
      color,
      is_shown,
      opacity,
    });
    self.raise_ring(dest);
    true
  }

  /// Draws the fill around content covering `covered` of the window's
  /// visible frame at `frame`, cut at `clip`, into a surrogate whose
  /// top-left is at `origin`; none with no `covered`.
  pub(crate) fn place_fill(
    &mut self,
    frame: &Rect,
    covered: Option<(i32, i32)>,
    clip: &Rect,
    origin: (i32, i32),
  ) {
    if let Some(fill) = &mut self.fill {
      fill.place(frame, covered, clip, origin);
    }
  }

  /// Draws the fill, while shown, at `opacity`: as faded as the window's
  /// thumbnail, not at its alpha (see `fade_fraction`).
  pub(crate) fn set_fill_opacity(&mut self, opacity: u8) {
    if let Some(fill) = &mut self.fill {
      fill.opacity = opacity;
      fill.apply_opacity();
    }
  }

  /// Draws the fill or not from now on.
  pub(crate) fn show_fill(&mut self, is_shown: bool) {
    if let Some(fill) = &mut self.fill {
      fill.is_shown = is_shown;
      fill.apply_opacity();
    }
  }

  /// Registers the fill strips on `dest` anew as they are, so they stack
  /// above thumbnails registered there since, then the ring above them.
  fn raise_fill(&mut self, dest: HWND) {
    let Some(fill) = &mut self.fill else {
      return;
    };

    let source = fill.source.hwnd();
    if let (Some(right), Some(bottom)) = (
      Thumbnail::register_with(dest, source, *fill.right.look()),
      Thumbnail::register_with(dest, source, *fill.bottom.look()),
    ) {
      fill.right = right;
      fill.bottom = bottom;
      self.raise_ring(dest);
    }
  }

  /// Draws the ring and backdrop with their window, now at `frame`, cut
  /// at `clip`, into a surrogate whose top-left is at `origin`.
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
      visible_frame: None,
    };

    if let Some(ring) = &mut self.ring {
      ring.place(&placement);
    }
    if let Some(backdrop) = &mut self.backdrop {
      backdrop.place(&placement);
    }
    self.placement = Some(placement);
  }

  /// Draws the backdrop with its window, now at `frame`, its visible frame
  /// at `visible_frame` if known exactly, cut at `clip`, into a surrogate
  /// whose top-left is at `origin`. The ring follows on
  /// [`place_ring`](Self::place_ring).
  pub(crate) fn place_backdrop(
    &mut self,
    frame: &Rect,
    visible_frame: Option<&Rect>,
    clip: &Rect,
    origin: (i32, i32),
  ) {
    let placement = Placement {
      frame: frame.clone(),
      clip: clip.clone(),
      origin,
      visible_frame: visible_frame.cloned(),
    };

    if let Some(backdrop) = &mut self.backdrop {
      backdrop.place(&placement);
    }
    self.placement = Some(placement);
  }

  /// Draws the ring where the backdrop was last placed.
  pub(crate) fn place_ring(&mut self) {
    if let (Some(ring), Some(placement)) =
      (&mut self.ring, &self.placement)
    {
      ring.place(placement);
    }
  }

  /// Draws the ring and backdrop at `opacity`.
  pub(crate) fn set_opacity(&mut self, opacity: u8) {
    self.set_ring_opacity(opacity);
    self.set_backdrop_opacity(opacity);
  }

  /// Draws the ring at `opacity`, independently of the backdrop.
  pub(crate) fn set_ring_opacity(&mut self, opacity: u8) {
    if let Some(ring) = &mut self.ring {
      ring.set_opacity(left_out(opacity, isolation::current().no_ring));
    }
  }

  /// Brings the ring copy to full opacity.
  pub(crate) fn cover_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      ring.set_opacity(left_out(u8::MAX, isolation::current().no_ring));
    }
  }

  pub(crate) fn hide_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      ring.hide();
    }
  }

  /// Unregisters them all, before the window they are drawn on goes.
  pub(crate) fn clear(&mut self) {
    self.ring = None;
    self.backdrop = None;
    self.fill = None;
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

/// `(rcSource, rcDestination)` drawing `part` of a ring copy that rests at
/// `copy_rest`, around a window resting at `window_rest`, with that window
/// now at `frame`, cut at `clip`, into a surrogate whose top-left is at
/// `origin`. `None` when nothing of it is left.
///
/// The whole ring is scaled with its window (see [`copy_rects`]). Cut
/// into cells, it keeps its distance from each edge of the window, its
/// corners, `corner_reach` in from its outer edge, drawn 1:1 and its edges
/// stretched between them, so its width and corners never change.
/// A ring smaller than two corners has them cut to half its size.
fn ring_part_rects(
  part: RingPart,
  copy_rest: &Rect,
  window_rest: &Rect,
  frame: &Rect,
  clip: &Rect,
  origin: (i32, i32),
  corner_reach: i32,
) -> Option<(RECT, RECT)> {
  let RingPart::Cell(column, row) = part else {
    return copy_rects(copy_rest, window_rest, frame, clip, origin);
  };

  let outer = Rect::from_ltrb(
    frame.left - (window_rest.left - copy_rest.left),
    frame.top - (window_rest.top - copy_rest.top),
    frame.right + (copy_rest.right - window_rest.right),
    frame.bottom + (copy_rest.bottom - window_rest.bottom),
  );
  let (src_x, dst_x) = band_span(
    column,
    copy_rest.width(),
    (outer.left, outer.right),
    corner_reach,
  )?;
  let (src_y, dst_y) = band_span(
    row,
    copy_rest.height(),
    (outer.top, outer.bottom),
    corner_reach,
  )?;

  let source = RECT {
    left: src_x.0,
    top: src_y.0,
    right: src_x.1,
    bottom: src_y.1,
  };
  let dest = Rect::from_ltrb(dst_x.0, dst_y.0, dst_x.1, dst_y.1);
  let visible = dest.intersection(clip)?;

  clip_rects(source, shifted(&dest, origin), &shifted(&visible, origin))
}

/// The span `band` covers along one axis of a ring `length` long at rest,
/// in its own coordinates, and drawn from `drawn.0` to `drawn.1`, with
/// corners `corner` long: `(source, destination)`, or `None` for one of no
/// length.
fn band_span(
  band: Band,
  length: i32,
  drawn: (i32, i32),
  corner: i32,
) -> Option<((i32, i32), (i32, i32))> {
  let corner = corner.min(length / 2).min((drawn.1 - drawn.0) / 2).max(0);

  let spans = match band {
    Band::Start => ((0, corner), (drawn.0, drawn.0 + corner)),
    Band::Middle => (
      (corner, length - corner),
      (drawn.0 + corner, drawn.1 - corner),
    ),
    Band::End => ((length - corner, length), (drawn.1 - corner, drawn.1)),
  };

  let ((src_from, src_to), (dst_from, dst_to)) = spans;
  (src_to > src_from && dst_to > dst_from).then_some(spans)
}

/// Where the fill's two strips go, as `rcDestination`s, around content
/// drawn from the top-left of a frame at `frame`, covering `covered` of
/// it, cut at `clip`, into a surrogate whose top-left is at `origin`:
/// one right of the content, the frame's height, and one under it, as
/// wide as the content, so the two never overlap. Each reaches under the
/// content's trailing edge (see `fill_start`), and is `None` where the
/// content reaches the frame's edge or none of it is inside the clip.
fn fill_strips(
  frame: &Rect,
  covered: (i32, i32),
  clip: &Rect,
  origin: (i32, i32),
) -> [Option<RECT>; 2] {
  let start = fill_start(covered);
  let right_from = if covered.0 < frame.width() {
    frame.left + start.0
  } else {
    frame.right
  };
  let bottom_from = if covered.1 < frame.height() {
    frame.top + start.1
  } else {
    frame.bottom
  };

  let strip = |rect: Rect| {
    (rect.width() > 0 && rect.height() > 0)
      .then(|| rect.intersection(clip))
      .flatten()
      .map(|shown| shifted(&shown, origin))
  };

  [
    strip(Rect::from_ltrb(
      right_from,
      frame.top,
      frame.right,
      frame.bottom,
    )),
    strip(Rect::from_ltrb(
      frame.left,
      bottom_from,
      right_from,
      frame.bottom,
    )),
  ]
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
    copy_rects, crop_fade, crop_weights, fill_strips, map_rect,
    ring_part_rects, same_set, scale_alpha, shifted, vignette_strength,
    BackdropOverlayParams, Band, Rect, RingPart, RECT,
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

  /// A window resting at (100, 100, 800x600) with a 10 px ring around it,
  /// its corners reaching 16 px in.
  const RING_WINDOW: Rect = Rect {
    left: 100,
    top: 100,
    right: 900,
    bottom: 700,
  };
  const RING: Rect = Rect {
    left: 90,
    top: 90,
    right: 910,
    bottom: 710,
  };
  const CORNER_REACH: i32 = 16;

  /// `(rcSource, rcDestination)` of each of the ring's cells, with its
  /// window at `frame`, cut at `clip`.
  fn cells_at(frame: &Rect, clip: &Rect) -> Vec<Option<(Edges, Edges)>> {
    RingPart::all(Some(CORNER_REACH))
      .into_iter()
      .map(|part| {
        ring_part_rects(
          part,
          &RING,
          &RING_WINDOW,
          frame,
          clip,
          (0, 0),
          CORNER_REACH,
        )
        .map(|(src, dst)| (ltrb(src), ltrb(dst)))
      })
      .collect()
  }

  fn area((left, top, right, bottom): Edges) -> i32 {
    (right - left) * (bottom - top)
  }

  #[test]
  fn a_ring_is_cut_into_eight_cells_around_its_middle() {
    let parts = RingPart::all(Some(CORNER_REACH));

    assert_eq!(parts.len(), 8);
    assert!(!parts.contains(&RingPart::Cell(Band::Middle, Band::Middle)));
    assert_eq!(RingPart::all(None), vec![RingPart::Whole]);
  }

  /// At rest, the cells draw the ring 1:1 and cover all of it but its
  /// middle, each pixel once.
  #[test]
  fn ring_cells_at_rest_tile_the_ring() {
    let cells = cells_at(&RING_WINDOW, &MONITOR)
      .into_iter()
      .map(|cell| cell.expect("shown"))
      .collect::<Vec<_>>();

    for (src, dst) in &cells {
      assert_eq!(
        (src.0 + RING.left, src.1 + RING.top),
        (dst.0, dst.1),
        "drawn where it rests"
      );
      assert_eq!(area(*src), area(*dst), "drawn 1:1");
    }

    let middle = RING.inset(CORNER_REACH);
    let covered = cells.iter().map(|(_, dst)| area(*dst)).sum::<i32>();
    assert_eq!(
      covered,
      RING.width() * RING.height() - middle.width() * middle.height()
    );
  }

  /// Resized, the ring keeps its corners 1:1 at the window's new corners
  /// and stretches its edges to meet them.
  #[test]
  fn ring_cells_keep_their_corners_as_the_window_is_resized() {
    let frame = Rect::from_xy(100, 100, 1200, 300);
    let cells = cells_at(&frame, &MONITOR);
    let parts = RingPart::all(Some(CORNER_REACH));
    let cell = |column, row| {
      let index = parts
        .iter()
        .position(|part| *part == RingPart::Cell(column, row))
        .expect("a cell");
      cells[index].expect("shown")
    };

    assert_eq!(
      cell(Band::Start, Band::Start),
      ((0, 0, 16, 16), (90, 90, 106, 106))
    );
    assert_eq!(
      cell(Band::End, Band::End),
      ((804, 604, 820, 620), (1294, 394, 1310, 410))
    );
    // The top edge, stretched from 788 to 1188 px between its corners.
    assert_eq!(
      cell(Band::Middle, Band::Start),
      ((16, 0, 804, 16), (106, 90, 1294, 106))
    );
    // The left edge, squeezed from 588 to 288 px.
    assert_eq!(
      cell(Band::Start, Band::Middle),
      ((0, 16, 16, 604), (90, 106, 106, 394))
    );
  }

  /// Cut at the clip, a cell partly past it is cut with its source, and
  /// one wholly past it is not drawn.
  #[test]
  fn ring_cells_are_cut_at_the_clip() {
    let clip = Rect::from_ltrb(0, 0, 500, 1080);
    let cells = cells_at(&RING_WINDOW, &clip);
    let parts = RingPart::all(Some(CORNER_REACH));
    let cell = |column, row| {
      let index = parts
        .iter()
        .position(|part| *part == RingPart::Cell(column, row))
        .expect("a cell");
      cells[index]
    };

    assert_eq!(
      cell(Band::Middle, Band::Start),
      Some(((16, 0, 410, 16), (106, 90, 500, 106)))
    );
    assert_eq!(cell(Band::End, Band::Start), None);
  }

  /// A ring drawn smaller than its two corners has them cut to half its
  /// size, with no edge between them.
  #[test]
  fn ring_cells_of_a_tiny_ring_meet_in_its_middle() {
    let frame = Rect::from_xy(100, 100, 10, 600);
    let cells = cells_at(&frame, &MONITOR);
    let parts = RingPart::all(Some(CORNER_REACH));
    let cell = |column, row| {
      let index = parts
        .iter()
        .position(|part| *part == RingPart::Cell(column, row))
        .expect("a cell");
      cells[index]
    };

    // 30 px across: corners of 15.
    assert_eq!(
      cell(Band::Start, Band::Start),
      Some(((0, 0, 15, 16), (90, 90, 105, 106)))
    );
    assert_eq!(cell(Band::Middle, Band::Start), None);
    assert_eq!(
      cell(Band::End, Band::Start),
      Some(((805, 0, 820, 16), (105, 90, 120, 106)))
    );
  }

  /// Whole, a ring scales with its window, as any copy does.
  #[test]
  fn a_whole_ring_scales_with_its_window() {
    let frame = Rect::from_xy(200, 200, 400, 300);

    assert_eq!(
      ring_part_rects(
        RingPart::Whole,
        &RING,
        &RING_WINDOW,
        &frame,
        &MONITOR,
        (0, 0),
        CORNER_REACH,
      ),
      copy_rects(&RING, &RING_WINDOW, &frame, &MONITOR, (0, 0))
    );
  }

  /// `rcDestination`s of the fill around content covering `covered` of a
  /// frame at (100, 100, 800x600), cut at `clip`.
  fn strips(covered: (i32, i32), clip: &Rect) -> [Option<Edges>; 2] {
    let frame = Rect::from_xy(100, 100, 800, 600);
    fill_strips(&frame, covered, clip, (0, 0)).map(|strip| strip.map(ltrb))
  }

  #[test]
  fn no_fill_where_the_content_covers_the_frame() {
    assert_eq!(strips((800, 600), &MONITOR), [None, None]);
    assert_eq!(strips((900, 700), &MONITOR), [None, None]);
  }

  /// Short on both axes, the fill is an L of two strips that meet without
  /// overlapping, each reaching 2 px under the content's trailing edge.
  #[test]
  fn fill_is_an_l_under_the_contents_trailing_edges() {
    assert_eq!(
      strips((600, 500), &MONITOR),
      [Some((698, 100, 900, 700)), Some((100, 598, 698, 700))]
    );
  }

  /// Short on one axis only, one strip spans the frame.
  #[test]
  fn fill_on_one_axis_spans_the_frame() {
    assert_eq!(
      strips((600, 600), &MONITOR),
      [Some((698, 100, 900, 700)), None]
    );
    assert_eq!(
      strips((800, 500), &MONITOR),
      [None, Some((100, 598, 900, 700))]
    );
  }

  #[test]
  fn fill_is_cut_at_the_clip() {
    let clip = Rect::from_ltrb(0, 0, 800, 1080);
    assert_eq!(
      strips((600, 500), &clip),
      [Some((698, 100, 800, 700)), Some((100, 598, 698, 700))]
    );

    let clip = Rect::from_ltrb(0, 0, 650, 1080);
    assert_eq!(
      strips((600, 500), &clip),
      [None, Some((100, 598, 650, 700))]
    );
  }
}
