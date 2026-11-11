use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  hidden_alpha::HiddenAlpha,
  native_surrogate::{placed_rects, to_logical},
  overlay_copies::{map_rect, scale_alpha, OverlayCopies},
  platform_impl::wallpaper_surface,
  resize_session::compute_border_inset,
  BackdropOverlayParams, CornerStyle, NativeSurrogate, NativeWindow, Rect,
  StyleBlend, SurrogateBatch,
};

/// Thumbnail opacity a peek is warmed up at under its still-shown real
/// window: rendered by DWM, but too faint to see.
const WARM_UP_OPACITY: u8 = 1;

/// How a [`WindowPeek`] draws its window.
enum PeekMode {
  /// Spans `viewport`, the area it is clipped to, and never moves: each
  /// frame only places the thumbnail in it, along with copies of the
  /// window's ring and backdrop, all in the same DWM updates (see
  /// `OverlayCopies`). As a `WorkspaceSurrogate` pinned to its viewport.
  Pinned { viewport: Rect },
  /// Shrunk to the window's visible part and moved with it every frame,
  /// for a backdrop overlay, which can't be copied, to follow.
  Live,
}

/// How a [`WindowPeek`] draws its window's own thumbnail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Drawn {
  /// Too faint to see, under its real window that still shows, while DWM
  /// warms the thumbnail up (see [`WindowPeek::take_over`]).
  WarmingUp,
  /// At the window's opacity, or opaque for its companion (see
  /// `NativeSurrogate::stand_in_opacity`).
  Shown,
  /// Not at all, its real window shown again (see
  /// [`WindowPeek::hand_back`]).
  HandedBack,
}

/// A click-through, live thumbnail of a window, cut to the part of it
/// that lies inside a clip rect.
///
/// Shows the columns of a scrolling layout that are moving or partly
/// visible: the real window stays cloaked, so nothing of it reaches past
/// its monitor.
pub struct WindowPeek {
  /// Declared before `surrogate`, so they are unregistered before the
  /// window they are drawn on goes.
  copies: OverlayCopies,
  surrogate: NativeSurrogate,
  mode: PeekMode,
  /// The window the peek shows.
  source: HWND,
  border_inset: RECT,
  /// Where the real window is, which its backdrop copy rests around.
  rest: Rect,
  drawn: Drawn,
  /// Whether the backdrop copy is to be drawn while the window is (see
  /// [`show_backdrop`](Self::show_backdrop)).
  shows_backdrop: bool,
  /// The window's own alpha, as last placed.
  opacity: u8,
  /// Fraction of their opacity the window and its copies are drawn at, as
  /// last placed (see [`place_scaled`](Self::place_scaled)).
  frac: f32,
  /// The real window's own alpha while it is hidden by it for a handover
  /// with the peek (see [`take_over`](Self::take_over) and
  /// [`prepare_hand_back`](Self::prepare_hand_back)).
  hidden_alpha: HiddenAlpha,
  /// The window's visible frame where it was last placed, clipped or not.
  frame_rect: Option<Rect>,
  visible_rect: Option<Rect>,
}

impl WindowPeek {
  /// Creates a hidden peek of `window`, which is at `rect`, clipped to
  /// `clip`. Unless it is `live`, it never moves (see [`PeekMode`]).
  pub fn new(
    window: &NativeWindow,
    rect: &Rect,
    clip: &Rect,
    corner_style: &CornerStyle,
    live: bool,
  ) -> crate::Result<Self> {
    let hwnd = window.hwnd();
    let border_inset = compute_border_inset(hwnd);

    // Just below the window it shows, like the other surrogates.
    let (surrogate, mode) = if live {
      let surrogate = NativeSurrogate::create(
        hwnd,
        &to_logical(rect, &border_inset),
        rect,
        u8::MAX,
        border_inset,
        corner_style,
        hwnd,
        false,
      )?;
      (surrogate, PeekMode::Live)
    } else {
      // Spanning the clip, it must not be rounded: the thumbnail shows the
      // window's own corners.
      let surrogate = NativeSurrogate::create(
        hwnd,
        clip,
        rect,
        u8::MAX,
        RECT::default(),
        &CornerStyle::Square,
        hwnd,
        false,
      )?;
      let viewport = clip.clone();
      (surrogate, PeekMode::Pinned { viewport })
    };

    Ok(Self {
      copies: OverlayCopies::default(),
      surrogate,
      mode,
      source: hwnd,
      border_inset,
      rest: rect.clone(),
      drawn: Drawn::Shown,
      shows_backdrop: false,
      opacity: u8::MAX,
      frac: 1.0,
      hidden_alpha: HiddenAlpha::default(),
      frame_rect: None,
      visible_rect: None,
    })
  }

  /// Whether the peek draws its window at all: DWM refuses a thumbnail of
  /// some, e.g. an elevated window.
  #[must_use]
  pub fn has_thumbnail(&self) -> bool {
    self.surrogate.has_thumbnail()
  }

  /// Whether the peek moves with its window (see [`PeekMode::Live`]).
  #[must_use]
  pub fn is_live(&self) -> bool {
    matches!(self.mode, PeekMode::Live)
  }

  /// Notes where the real window is, at `rect` with its frame at `frame`,
  /// which its backdrop copy then rests around.
  pub fn follow_window(&mut self, rect: &Rect, frame: &Rect) {
    if &self.rest == rect {
      return;
    }

    self.rest = rect.clone();
    self.copies.move_backdrop(rect, frame);
  }

  /// Draws a copy of its window's backdrop under it from now on, in each
  /// of `styles` the window may be drawn in (e.g. focused and not), of
  /// which [`show_style_blend`](Self::show_style_blend) picks one, or two
  /// to crossfade. The same `styles` again keep what is drawn.
  ///
  /// All are drawn up front, so a focus change mid-move only crossfades
  /// them: registering a new one then, and the window's own thumbnail
  /// anew over it, left frames with no copy at all.
  ///
  /// Only a peek that never moves carries them, in styles that can be
  /// copied (see `BackdropOverlayParams::is_copyable`), and shown while
  /// [`show_backdrop`](Self::show_backdrop) says so.
  ///
  /// Before [`carry_ring`](Self::carry_ring), which goes over the window
  /// that this may put back on top of the backdrop.
  pub fn carry_styles(
    &mut self,
    styles: &[Option<BackdropOverlayParams>],
  ) {
    let PeekMode::Pinned { viewport } = &self.mode else {
      return;
    };

    let dest = self.surrogate.hwnd();
    let frame = to_logical(&self.rest, &self.border_inset);
    let backdrops = styles
      .iter()
      .flatten()
      .copied()
      .filter(BackdropOverlayParams::is_copyable)
      .collect::<Vec<_>>();

    if !backdrops.is_empty()
      && !self.copies.carries_backdrop_styles(&backdrops)
    {
      // Cut from a backdrop of the whole monitor, which the clip, its work
      // area, is part of.
      let monitor = wallpaper_surface::monitor_bounds(viewport);

      if self
        .copies
        .carry_backdrop(dest, &backdrops, &monitor, &self.rest, &frame)
      {
        self.surrogate.raise_thumbnail(self.source);
      }
    }
  }

  /// Draws the backdrop copy in the style `blend` fades to, over the one
  /// it fades from, of those carried (see
  /// [`carry_styles`](Self::carry_styles)).
  pub fn show_style_blend(
    &mut self,
    blend: &StyleBlend<BackdropOverlayParams>,
  ) {
    self.copies.show_backdrop_blend(Some(blend));
  }

  /// Whether the peek draws the window's backdrop in any style (see
  /// [`carry_styles`](Self::carry_styles)).
  #[must_use]
  pub fn carries_backdrop(&self) -> bool {
    self.copies.carries_backdrop()
  }

  /// Draws the backdrop copy from now on, while the peek draws the window
  /// at all: once the real backdrop no longer shows, or has been
  /// composited under it.
  pub fn show_backdrop(&mut self) {
    self.shows_backdrop = true;
    self.apply_backdrop();
  }

  /// Stops drawing the backdrop copy.
  pub fn hide_backdrop(&mut self) {
    self.shows_backdrop = false;
    self.apply_backdrop();
  }

  /// Draws the backdrop copy or not, as asked and as the window is drawn:
  /// over a real window handed back, an opaque copy would hide it.
  fn apply_backdrop(&mut self) {
    self.copies.show_backdrop(
      self.shows_backdrop && self.drawn != Drawn::HandedBack,
    );
  }

  /// Draws the window's border ring along with it from now on, at once,
  /// over the ring itself: `source`, its overlay window, at `ring_rect`
  /// around the window at `window_rect`, both where they are now. Called
  /// again with the same `source`, only updates where the two are.
  /// Returns whether the peek does: only one that never moves can carry
  /// a ring, as for a `WorkspaceSurrogate`.
  pub fn carry_ring(
    &mut self,
    source: HWND,
    ring_rect: &Rect,
    window_rect: &Rect,
  ) -> bool {
    if self.is_live() {
      return false;
    }

    let dest = self.surrogate.hwnd();
    let carries =
      self.copies.carry_ring(dest, source, ring_rect, window_rect);
    // Drawn twice in one place, a ring looks like one. Faded with its
    // window, as last placed (see `place_scaled`).
    if self.frac < 1.0 {
      self
        .copies
        .set_ring_opacity(scale_alpha(u8::MAX, self.frac));
    } else {
      self.copies.cover_ring();
    }
    carries
  }

  /// Whether the peek draws the window's border ring.
  #[must_use]
  pub fn carries_ring(&self) -> bool {
    self.copies.carries_ring()
  }

  /// Draws the window too faint to see until
  /// [`take_over`](Self::take_over), under its real window that still
  /// shows, while DWM warms the thumbnail up.
  pub fn warm_up(&mut self) {
    self.draw(Drawn::WarmingUp);
  }

  /// Takes over showing the window from `window`, which is about to be
  /// cloaked: the thumbnail goes to the window's opacity (see
  /// [`Drawn::Shown`]) and a semi-transparent `window` to no alpha at all,
  /// in that order, so the swap lands in one frame and the cloak unseen
  /// (see `HiddenAlpha`). [`restore_alpha`](Self::restore_alpha) gives
  /// `window` its alpha back once the cloak is on screen.
  pub fn take_over(&mut self, window: &NativeWindow) {
    self.draw(Drawn::Shown);
    self.hidden_alpha.hide(&window.inner);
  }

  /// Draws the window from now on, if it isn't, e.g. once its real
  /// window is cloaked without a [`take_over`](Self::take_over).
  pub fn show_window(&mut self) {
    if self.drawn != Drawn::Shown {
      self.draw(Drawn::Shown);
    }
  }

  /// Switches the window's thumbnail to its companion if one was shown
  /// since the peek last looked, e.g. for a column just opened, as shades
  /// takes some time to cover it. Looks at most every 100 ms (see
  /// `NativeSurrogate::rechecks_companion`).
  pub fn refresh_companion(&mut self) {
    if self.surrogate.rechecks_companion() {
      self.adopt_companion();
    }
  }

  /// [`refresh_companion`](Self::refresh_companion), unthrottled. A peek
  /// that handed its window back keeps drawing nothing of it.
  ///
  /// The companion is drawn as the window was, then at
  /// [`window_opacity`](Self::window_opacity), and the ring copy is put
  /// back over it.
  fn adopt_companion(&mut self) {
    if self.drawn == Drawn::HandedBack
      || !self
        .surrogate
        .adopt_companion(self.source, self.window_opacity())
    {
      return;
    }

    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.raise_ring(self.surrogate.hwnd());
  }

  /// Gives the real window back the alpha a handover set aside, unless it
  /// was set again since. No-op otherwise.
  pub fn restore_alpha(&mut self) {
    self.hidden_alpha.restore();
  }

  /// Readies handing the window back to `window`, which is about to be
  /// uncloaked under the peek: a semi-transparent `window` goes to no
  /// alpha, so its uncloak lands unseen and
  /// [`hand_back`](Self::hand_back) swaps the two in one frame.
  pub fn prepare_hand_back(&mut self, window: &NativeWindow) {
    self.hidden_alpha.hide(&window.inner);
  }

  /// Hands the window back to its real window, uncloaked under the peek:
  /// gives it its alpha back, then stops drawing the window and its
  /// backdrop, which the real window and the real backdrop, shown as it
  /// landed, show again. The ring copy stays, over the real ring revealed
  /// under it, until the peek goes.
  pub fn hand_back(&mut self) {
    self.hidden_alpha.restore();
    self.draw(Drawn::HandedBack);
  }

  /// Draws the window's own thumbnail, and its backdrop copy with it, as
  /// `drawn` says from now on.
  ///
  /// The backdrop copy goes first and comes last: a frame between the
  /// two then shows the window twice, over the real one under the peek,
  /// rather than not at all, hidden by the opaque copy.
  fn draw(&mut self, drawn: Drawn) {
    self.drawn = drawn;
    if drawn == Drawn::HandedBack {
      self.apply_backdrop();
      self.surrogate.set_window_opacity(self.window_opacity());
    } else {
      self.surrogate.set_window_opacity(self.window_opacity());
      self.apply_backdrop();
    }
  }

  /// The opacity the window's own thumbnail is drawn at.
  fn window_opacity(&self) -> u8 {
    match self.drawn {
      Drawn::WarmingUp => WARM_UP_OPACITY,
      Drawn::Shown => scale_alpha(
        self.surrogate.stand_in_opacity(self.opacity),
        self.frac,
      ),
      Drawn::HandedBack => 0,
    }
  }

  /// Shows the part of the window, placed at `rect`, that lies inside
  /// `clip`, or hides the peek when none does. A move of a peek already
  /// shown is queued into `batch`, to land with its window's overlays.
  pub fn place(
    &mut self,
    rect: &Rect,
    clip: &Rect,
    opacity: u8,
    batch: &mut SurrogateBatch,
  ) {
    let logical = to_logical(rect, &self.border_inset);
    self.opacity = opacity;
    self.frac = 1.0;
    let window_opacity = self.window_opacity();

    self.visible_rect = match &mut self.mode {
      PeekMode::Live => self.surrogate.show_clipped(
        &logical,
        clip,
        self.border_inset,
        window_opacity,
        Some(batch),
      ),
      PeekMode::Pinned { viewport } => {
        repin(&mut self.surrogate, viewport, clip);
        let origin = (viewport.left, viewport.top);
        self.show_pinned(rect, &logical, clip, origin)
      }
    };
    self.frame_rect = Some(logical);
  }

  /// Shows the window, at rest at `rest`, drawn at `frame` (of any size)
  /// cut at `clip`, at `frac` of its `opacity`, its ring and backdrop
  /// copies placed at `frame` and faded with it. The thumbnail samples the
  /// window at its size at rest.
  ///
  /// Pinned peeks only: a live one is placed at `frame` unscaled and
  /// unfaded (see [`place`](Self::place)).
  pub fn place_scaled(
    &mut self,
    rest: &Rect,
    frame: &Rect,
    clip: &Rect,
    opacity: u8,
    frac: f32,
    batch: &mut SurrogateBatch,
  ) {
    let PeekMode::Pinned { viewport } = &mut self.mode else {
      tracing::debug!("A live peek can't draw its window scaled.");
      self.place(frame, clip, opacity, batch);
      return;
    };

    repin(&mut self.surrogate, viewport, clip);
    let origin = (viewport.left, viewport.top);
    self.opacity = opacity;
    self.frac = frac.clamp(0.0, 1.0);

    let ScaledRects { drawn, shown } =
      scaled_rects(rest, frame, clip, self.border_inset, origin);
    self.frame_rect = Some(drawn);

    let Some((visible, (rc_src, rc_dst))) = shown else {
      self.surrogate.set_visible(false);
      self.visible_rect = None;
      return;
    };

    self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place(frame, clip, origin);
    self.copies.set_opacity(scale_alpha(u8::MAX, self.frac));
    self.surrogate.set_visible(true);
    self.visible_rect = Some(visible);
  }

  /// Shows the window as it is, its rect `actual`, covering its frame at
  /// `rect` on the way to `target`, cut at `clip`, its ring and backdrop
  /// copies around `rect`. Its scale is `rect` over `target` times a
  /// cover of at least `cover` (see [`stretched_rects`]).
  ///
  /// `capped`, it samples no more than `target`'s size on either axis, for
  /// a window that may still be resizing there: a source larger than the
  /// window is drawn as a transparent hole.
  ///
  /// Returns the cover drawn with. Pinned peeks only: a live one is placed
  /// at `rect` unscaled (see [`place`](Self::place)).
  #[allow(clippy::too_many_arguments)]
  pub fn place_stretched(
    &mut self,
    actual: &Rect,
    target: &Rect,
    rect: &Rect,
    capped: bool,
    cover: (f32, f32),
    clip: &Rect,
    opacity: u8,
    batch: &mut SurrogateBatch,
  ) -> (f32, f32) {
    let PeekMode::Pinned { viewport } = &mut self.mode else {
      tracing::debug!("A live peek can't draw its window stretched.");
      self.place(rect, clip, opacity, batch);
      return (1.0, 1.0);
    };

    repin(&mut self.surrogate, viewport, clip);
    let origin = (viewport.left, viewport.top);
    self.opacity = opacity;
    self.frac = 1.0;

    let (ScaledRects { drawn, shown }, cover) = stretched_rects(
      actual,
      target,
      rect,
      capped,
      cover,
      clip,
      self.border_inset,
      origin,
    );
    self.frame_rect = Some(drawn);

    let Some((visible, (rc_src, rc_dst))) = shown else {
      self.surrogate.set_visible(false);
      self.visible_rect = None;
      return cover;
    };

    self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place(rect, clip, origin);
    self.copies.cover_ring();
    self.copies.set_backdrop_opacity(u8::MAX);
    self.surrogate.set_visible(true);
    self.visible_rect = Some(visible);
    cover
  }

  /// Draws the window at `rect`, its frame at `frame`, cut at `clip`, into
  /// a peek pinned with its top-left at `origin`. Returns the visible part
  /// of the frame, if any.
  ///
  /// Shows the window at its own scale, as `show_clipped` does: a column
  /// resized as it moves reveals more or less of it rather than stretching
  /// it. Its ring copy does stretch, to frame it.
  ///
  /// Only the frame is drawn, sampled past the invisible resize borders as
  /// `show_clipped` does (see `WorkspaceSurrogate`'s `frame_rects`).
  fn show_pinned(
    &mut self,
    rect: &Rect,
    frame: &Rect,
    clip: &Rect,
    origin: (i32, i32),
  ) -> Option<Rect> {
    let placed = frame.intersection(clip).and_then(|visible| {
      let rects = placed_rects(
        frame,
        &visible,
        (frame.width(), frame.height()),
        self.border_inset,
        origin,
      )?;
      Some((visible, rects))
    });

    let Some((visible, (rc_src, rc_dst))) = placed else {
      self.surrogate.set_visible(false);
      return None;
    };

    self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place(rect, clip, origin);
    self.copies.cover_ring();
    self.copies.set_backdrop_opacity(u8::MAX);
    self.surrogate.set_visible(true);

    Some(visible)
  }

  /// Hides the peek until it is placed again, e.g. while its window is
  /// drawn at no size.
  pub fn hide(&mut self) {
    self.surrogate.set_visible(false);
    self.visible_rect = None;
  }

  /// The window's visible frame where it was last placed, including any
  /// part outside the clip, for overlays that frame it.
  #[must_use]
  pub fn frame_rect(&self) -> Option<&Rect> {
    self.frame_rect.as_ref()
  }

  /// Handle of the peek's own window, which overlays of the window it
  /// shows sit behind.
  #[must_use]
  pub fn hwnd(&self) -> HWND {
    self.surrogate.hwnd()
  }

  /// Begins handing the window back as it lands: puts the peek directly
  /// above its window, which is about to be uncloaked, so the peek keeps
  /// covering it until it hands the window back (see
  /// [`hand_back`](Self::hand_back)), and starts a fresh companion wait.
  ///
  /// A peek is reused for move after move, so a wait kept from an earlier
  /// landing would have timed out and let a later one show the window
  /// before its companion. A companion shown since the peek last looked
  /// is picked up first, so the hand-back waits for it.
  pub fn begin_hand_back(&mut self) {
    self.adopt_companion();
    self.surrogate.begin_hand_back();

    if let Err(err) = self.surrogate.raise_above(self.source) {
      tracing::warn!("Failed to raise a peek above its window: {err}");
    }
  }

  /// Puts the peek at the top of the non-topmost windows, over windows
  /// that came above its real window while that was minimized. Its ring
  /// and backdrop copies are drawn in it, so they go with it.
  pub fn raise_to_top(&self) {
    if let Err(err) = self.surrogate.raise_to_top() {
      tracing::warn!("Failed to raise a peek to the top: {err}");
    }
  }

  /// Puts the peek at the top of the always-on-top band if `topmost`, else
  /// as [`raise_to_top`](Self::raise_to_top) does, out of that band if it
  /// was in it.
  pub fn raise_to_top_of_band(&self, topmost: bool) {
    if let Err(err) = self.surrogate.raise_to_top_of_band(topmost) {
      tracing::warn!(
        "Failed to raise a peek to the top of its band: {err}"
      );
    }
  }

  /// Whether the peek should keep covering its uncloaked window: it shows
  /// the window's companion, which is not back on screen yet.
  pub fn awaits_companion(&mut self) -> bool {
    self.surrogate.awaits_companion()
  }

  /// The on-screen part of the peek, if it shows anything.
  #[must_use]
  pub fn visible_rect(&self) -> Option<&Rect> {
    self.visible_rect.as_ref()
  }
}

/// Moves a pinned peek's `surrogate`, spanning `viewport`, to span `clip`
/// instead, if it doesn't.
fn repin(
  surrogate: &mut NativeSurrogate,
  viewport: &mut Rect,
  clip: &Rect,
) {
  if viewport != clip {
    match surrogate.reposition(clip) {
      Ok(()) => *viewport = clip.clone(),
      Err(err) => tracing::warn!("Failed to move a peek: {err}"),
    }
  }
}

/// Where a pinned peek draws a window scaled (see [`scaled_rects`]).
#[derive(Debug, PartialEq, Eq)]
struct ScaledRects {
  /// The window's frame as drawn, including any part outside the clip.
  drawn: Rect,
  /// The part of `drawn` inside the clip and the thumbnail's
  /// `(rcSource, rcDestination)` drawing it, unless none of it is.
  shown: Option<(Rect, (RECT, RECT))>,
}

/// Where a window resting at `rest`, drawn at `frame` cut at `clip`, is
/// drawn into a peek pinned with its top-left at `origin`.
///
/// The window's frame at rest is sampled, scaled to `frame`, past the
/// invisible resize borders `border_inset` (see `placed_rects`).
fn scaled_rects(
  rest: &Rect,
  frame: &Rect,
  clip: &Rect,
  border_inset: RECT,
  origin: (i32, i32),
) -> ScaledRects {
  let rest_frame = to_logical(rest, &border_inset);
  let drawn = map_rect(&rest_frame, rest, frame);
  let shown = drawn.intersection(clip).and_then(|visible| {
    let rects = placed_rects(
      &drawn,
      &visible,
      (rest_frame.width(), rest_frame.height()),
      border_inset,
      origin,
    )?;
    Some((visible, rects))
  });

  ScaledRects { drawn, shown }
}

/// Where a window at `actual`, drawn covering its frame at `rect` on the
/// way to `target`, cut at `clip`, is drawn into a peek pinned with its
/// top-left at `origin`, and the cover it is drawn with.
///
/// Sampled from its frame's top-left: all of it or, `capped`, no more than
/// `target`'s frame on either axis. What is sampled is scaled by `rect`
/// over `target` times the cover: `cover`, or more where it takes more to
/// reach across the frame. Only the part inside the frame is drawn, so
/// neither more is sampled than asked nor anything left uncovered. At
/// `target`, a window of its size is drawn 1:1 with a cover of 1.
#[allow(clippy::too_many_arguments)]
fn stretched_rects(
  actual: &Rect,
  target: &Rect,
  rect: &Rect,
  capped: bool,
  cover: (f32, f32),
  clip: &Rect,
  border_inset: RECT,
  origin: (i32, i32),
) -> (ScaledRects, (f32, f32)) {
  let actual = to_logical(actual, &border_inset);
  let target = to_logical(target, &border_inset);
  let drawn = to_logical(rect, &border_inset);
  let size = if capped {
    (
      actual.width().min(target.width()),
      actual.height().min(target.height()),
    )
  } else {
    (actual.width(), actual.height())
  };

  let (cover_x, width) =
    covering(size.0, target.width(), drawn.width(), cover.0);
  let (cover_y, height) =
    covering(size.1, target.height(), drawn.height(), cover.1);
  let part = Rect::from_xy(drawn.x(), drawn.y(), width, height);
  let shown = drawn.intersection(clip).and_then(|visible| {
    let rects = placed_rects(&part, &visible, size, border_inset, origin)?;
    Some((visible, rects))
  });

  (ScaledRects { drawn, shown }, (cover_x, cover_y))
}

/// The cover and length `sampled` pixels of a window are drawn with along
/// one axis, its frame `drawn` long on the way to `target`: `cover`, or
/// what it takes for them to reach across the frame if that is more.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn covering(
  sampled: i32,
  target: i32,
  drawn: i32,
  cover: f32,
) -> (f32, i32) {
  if sampled <= 0 || target <= 0 {
    return (1.0, drawn);
  }

  let cover = cover.max(1.0).max(target as f32 / sampled as f32);
  let length =
    (sampled as f32 * drawn as f32 / target as f32 * cover).round() as i32;

  (cover, length.max(drawn))
}

#[cfg(test)]
mod tests {
  use super::*;

  const INSET: RECT = RECT {
    left: 7,
    top: 0,
    right: 7,
    bottom: 7,
  };

  /// Drawn at rest, a window gets the rects `show_pinned` gives it.
  #[test]
  fn scaled_at_rest_matches_unscaled() {
    let rest = Rect::from_ltrb(93, 100, 907, 707);
    let clip = Rect::from_ltrb(0, 0, 1920, 1040);
    let frame = to_logical(&rest, &INSET);
    let shown = frame.intersection(&clip).and_then(|visible| {
      let rects = placed_rects(
        &frame,
        &visible,
        (frame.width(), frame.height()),
        INSET,
        (0, 0),
      )?;
      Some((visible, rects))
    });

    assert_eq!(
      scaled_rects(&rest, &rest, &clip, INSET, (0, 0)),
      ScaledRects {
        drawn: frame,
        shown,
      }
    );
  }

  /// Drawn at half its size about its centre, a window is sampled whole
  /// into half the destination.
  #[test]
  fn scaled_to_half_draws_half_the_size() {
    let rest = Rect::from_ltrb(93, 100, 907, 707);
    let half = Rect::from_ltrb(296, 250, 703, 554);
    let clip = Rect::from_ltrb(0, 0, 1920, 1040);

    let ScaledRects { drawn, shown } =
      scaled_rects(&rest, &half, &clip, INSET, (0, 0));
    let Some((visible, (src, dst))) = shown else {
      panic!("not shown");
    };

    assert_eq!(drawn, visible);
    assert_eq!(
      (src.left, src.top, src.right, src.bottom),
      (7, 0, 807, 600)
    );
    assert_eq!((dst.right - dst.left, dst.bottom - dst.top), (400, 300));
  }

  const CLIP: Rect = Rect {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1040,
  };

  fn ltrb(rect: RECT) -> (i32, i32, i32, i32) {
    (rect.left, rect.top, rect.right, rect.bottom)
  }

  fn stretched(
    actual: &Rect,
    target: &Rect,
    rect: &Rect,
    capped: bool,
    cover: (f32, f32),
  ) -> (Rect, RECT, RECT, (f32, f32)) {
    let (ScaledRects { drawn, shown }, cover) = stretched_rects(
      actual,
      target,
      rect,
      capped,
      cover,
      &CLIP,
      INSET,
      (0, 0),
    );
    let Some((visible, (src, dst))) = shown else {
      panic!("not shown");
    };
    assert_eq!(visible, drawn);

    (drawn, src, dst, cover)
  }

  /// At its target, a window of its size is drawn exactly 1:1.
  #[test]
  fn stretched_at_its_target_is_one_to_one() {
    let target = Rect::from_ltrb(93, 100, 907, 707);
    let (drawn, src, dst, cover) =
      stretched(&target, &target, &target, false, (1.0, 1.0));

    assert_eq!(drawn, to_logical(&target, &INSET));
    assert_eq!(ltrb(src), (7, 0, 807, 600));
    assert_eq!(ltrb(dst), (100, 100, 900, 700));
    assert_eq!(cover, (1.0, 1.0));
  }

  /// A window yet to grow is sampled whole and stretched across its
  /// frame, wherever that is on its way.
  #[test]
  fn a_window_yet_to_grow_fills_its_frame() {
    let actual = Rect::from_ltrb(93, 100, 907, 707);
    let target = Rect::from_ltrb(93, 100, 1307, 707);
    for right in [907, 1107, 1307] {
      let rect = Rect::from_ltrb(93, 100, right, 707);
      let (drawn, src, dst, cover) =
        stretched(&actual, &target, &rect, true, (1.0, 1.0));

      assert_eq!(ltrb(src), (7, 0, 807, 600));
      assert_eq!(ltrb(dst), (100, 100, right - 7, 700));
      assert_eq!(drawn, to_logical(&rect, &INSET));
      assert_eq!(cover, (1.5, 1.0));
    }
  }

  /// Grown, it keeps the scale it had, cut at its frame's edge, and
  /// samples less of itself for it, never more.
  #[test]
  fn a_grown_window_keeps_its_cover_cut_at_its_frame() {
    let target = Rect::from_ltrb(93, 100, 1307, 707);
    let rect = Rect::from_ltrb(93, 100, 1107, 707);
    let (_, src, dst, cover) =
      stretched(&target, &target, &rect, false, (1.5, 1.0));

    // 1000 px across at 1.25 (1000 / 1200 x 1.5): 800 px of the window.
    assert_eq!(ltrb(src), (7, 0, 807, 600));
    assert_eq!(ltrb(dst), (100, 100, 1100, 700));
    assert_eq!(cover, (1.5, 1.0));
  }

  /// A window yet to shrink is sampled no wider than its target, which it
  /// may be at any time, scaled by its frame over its target.
  #[test]
  fn a_window_yet_to_shrink_samples_no_more_than_its_target() {
    let actual = Rect::from_ltrb(93, 100, 1307, 707);
    let target = Rect::from_ltrb(93, 100, 907, 707);
    let rect = Rect::from_ltrb(93, 100, 1107, 707);
    let (_, src, dst, cover) =
      stretched(&actual, &target, &rect, true, (1.0, 1.0));

    assert_eq!(ltrb(src), (7, 0, 807, 600));
    assert_eq!(ltrb(dst), (100, 100, 1100, 700));
    assert_eq!(cover, (1.0, 1.0));
  }

  /// A window that won't grow to its target still covers its frame.
  #[test]
  fn a_window_short_of_its_target_still_covers_its_frame() {
    let actual = Rect::from_ltrb(93, 100, 907, 707);
    let target = Rect::from_ltrb(93, 100, 1307, 1007);
    let (drawn, src, dst, _) =
      stretched(&actual, &target, &target, true, (1.0, 1.0));

    assert_eq!(ltrb(src), (7, 0, 807, 600));
    assert_eq!(ltrb(dst), (100, 100, 1300, 1000));
    assert_eq!(drawn, to_logical(&target, &INSET));
  }

  /// Cut at the clip, only the part of the frame inside it is drawn.
  #[test]
  fn a_stretched_window_is_cut_at_the_clip() {
    let target = Rect::from_ltrb(1513, 100, 2327, 707);
    let (ScaledRects { shown, .. }, _) = stretched_rects(
      &target,
      &target,
      &target,
      false,
      (1.0, 1.0),
      &CLIP,
      INSET,
      (0, 0),
    );
    let Some((visible, (src, dst))) = shown else {
      panic!("not shown");
    };

    assert_eq!(visible, Rect::from_ltrb(1520, 100, 1920, 700));
    assert_eq!(ltrb(src), (7, 0, 407, 600));
    assert_eq!(ltrb(dst), (1520, 100, 1920, 700));
  }
}
