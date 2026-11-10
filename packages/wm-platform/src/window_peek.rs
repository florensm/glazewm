use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  hidden_alpha::HiddenAlpha,
  native_surrogate::{placed_rects, to_logical},
  overlay_copies::OverlayCopies,
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
      hidden_alpha: HiddenAlpha::default(),
      frame_rect: None,
      visible_rect: None,
    })
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
    // Drawn twice in one place, a ring looks like one.
    self.copies.cover_ring();
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
      Drawn::Shown => self.surrogate.stand_in_opacity(self.opacity),
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
        if viewport != clip {
          match self.surrogate.reposition(clip) {
            Ok(()) => *viewport = clip.clone(),
            Err(err) => tracing::warn!("Failed to move a peek: {err}"),
          }
        }

        let origin = (viewport.left, viewport.top);
        self.show_pinned(rect, &logical, clip, origin)
      }
    };
    self.frame_rect = Some(logical);
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
  /// before its companion.
  pub fn begin_hand_back(&mut self) {
    self.surrogate.begin_hand_back();

    if let Err(err) = self.surrogate.raise_above(self.source) {
      tracing::warn!("Failed to raise a peek above its window: {err}");
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
