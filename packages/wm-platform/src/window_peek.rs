use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  companion::HandoffWait,
  native_surrogate::{placed_rects, to_logical},
  overlay_copies::OverlayCopies,
  platform_impl::wallpaper_surface,
  resize_session::compute_border_inset,
  BackdropOverlayParams, CornerStyle, NativeSurrogate, NativeWindow, Rect,
  SurrogateBatch,
};

/// How a [`WindowPeek`] draws its window.
enum PeekMode {
  /// Spans `viewport`, the area it is clipped to, and never moves: each
  /// frame only places the thumbnail in it, along with copies of the
  /// window's ring and shadow, all in the same DWM updates (see
  /// `OverlayCopies`). As a `WorkspaceSurrogate` pinned to its viewport.
  Pinned { viewport: Rect },
  /// Shrunk to the window's visible part and moved with it every frame,
  /// for a backdrop overlay, which can't be copied, to follow.
  Live,
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
  /// See [`WindowPeek::awaits_companion`].
  companion_wait: HandoffWait,
  border_inset: RECT,
  /// Where the real window is, which its shadow rests around.
  rest: Rect,
  /// Whether the shadow copy is drawn: only while the real window is
  /// cloaked, so it shows no shadow of its own.
  casts_shadow: bool,
  /// How the shadow copy was last drawn, as active or not with the
  /// window's corners; `None` until it first was.
  shadow_look: Option<(bool, CornerStyle)>,
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
      companion_wait: HandoffWait::default(),
      border_inset,
      rest: rect.clone(),
      casts_shadow: false,
      shadow_look: None,
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
  /// which its shadow and backdrop then rest around.
  pub fn follow_window(&mut self, rect: &Rect, frame: &Rect) {
    if &self.rest == rect {
      return;
    }

    self.rest = rect.clone();
    self.copies.move_shadow(rect, frame);
    self.copies.move_backdrop(rect, frame);
  }

  /// Draws the window's backdrop, as `params` show it, under its copy from
  /// now on, while [`show_backdrop`](Self::show_backdrop) says so (see
  /// `OverlayCopies::carry_backdrop`). Returns whether the peek does: only
  /// one that never moves can, and only a backdrop that can be copied (see
  /// `BackdropOverlayParams::is_copyable`).
  ///
  /// Before [`carry_ring`](Self::carry_ring) and
  /// [`carry_shadow`](Self::carry_shadow), which go over the window that
  /// this puts back on top of the backdrop.
  pub fn carry_backdrop(&mut self, params: BackdropOverlayParams) -> bool {
    let PeekMode::Pinned { viewport } = &self.mode else {
      return false;
    };
    if !params.is_copyable() {
      return false;
    }
    if self.copies.backdrop_params() == Some(params) {
      return true;
    }

    // Cut from a backdrop of the whole monitor, which the clip, its work
    // area, is part of.
    let monitor = wallpaper_surface::monitor_bounds(viewport);
    let frame = to_logical(&self.rest, &self.border_inset);
    let dest = self.surrogate.hwnd();

    if !self
      .copies
      .carry_backdrop(dest, params, &monitor, &self.rest, &frame)
    {
      return false;
    }

    self.surrogate.raise_thumbnail(self.source);
    true
  }

  /// Whether the peek draws the window's backdrop (see
  /// [`carry_backdrop`](Self::carry_backdrop)).
  #[must_use]
  pub fn carries_backdrop(&self) -> bool {
    self.copies.carries_backdrop()
  }

  /// Draws the backdrop copy from now on: once the real backdrop no
  /// longer shows, or has been composited under it.
  pub fn show_backdrop(&mut self) {
    self.copies.show_backdrop(true);
  }

  /// Stops drawing the backdrop copy.
  pub fn hide_backdrop(&mut self) {
    self.copies.show_backdrop(false);
  }

  /// Draws the window's border ring along with it from now on: `source`,
  /// its overlay window, at `ring_rect` around the window at
  /// `window_rect`, both where they are now. Called again with the same
  /// `source`, only updates where the two are. Returns whether the peek
  /// does: only one that never moves can carry a ring, as for a
  /// `WorkspaceSurrogate`.
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
    self.copies.carry_ring(dest, source, ring_rect, window_rect)
  }

  /// Whether the peek draws the window's border ring.
  #[must_use]
  pub fn carries_ring(&self) -> bool {
    self.copies.carries_ring()
  }

  /// Draws `window`'s shadow along with it from now on, as for the window
  /// drawn active or not with `corner_style`'s corners, redrawing it when
  /// either changed. Returns whether the peek does: only one that never
  /// moves can, and only for a window DWM draws a shadow around.
  pub fn carry_shadow(
    &mut self,
    window: &NativeWindow,
    is_active: bool,
    corner_style: CornerStyle,
  ) -> bool {
    if self.is_live() {
      return false;
    }

    if self.shadow_look != Some((is_active, corner_style)) {
      self.shadow_look = Some((is_active, corner_style));

      let dest = self.surrogate.hwnd();
      let frame = to_logical(&self.rest, &self.border_inset);
      self.copies.carry_shadow(
        dest,
        window,
        &self.rest,
        &frame,
        is_active,
        corner_style,
      );
    }

    true
  }

  /// Draws the shadow copy or not from the next placement on: only while
  /// the real window is cloaked, as it otherwise shows its own.
  pub fn cast_shadow(&mut self, casts_shadow: bool) {
    self.casts_shadow = casts_shadow;
    if !casts_shadow {
      self.copies.set_shadow_opacity(0);
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

    self.visible_rect = match &mut self.mode {
      PeekMode::Live => self.surrogate.show_clipped(
        &logical,
        clip,
        self.border_inset,
        opacity,
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
        self.show_pinned(rect, &logical, clip, opacity, origin)
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
  /// it. Its ring and shadow copies do stretch, to frame it.
  ///
  /// Only the frame is drawn, sampled past the invisible resize borders as
  /// `show_clipped` does (see `WorkspaceSurrogate`'s `frame_rects`).
  fn show_pinned(
    &mut self,
    rect: &Rect,
    frame: &Rect,
    clip: &Rect,
    opacity: u8,
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
    self.surrogate.set_window_opacity(opacity);
    self.copies.place(rect, clip, origin);
    self.copies.cover_ring();
    self.copies.set_backdrop_opacity(u8::MAX);
    // At the window's opacity, as DWM draws a layered window's shadow.
    self.copies.set_shadow_opacity(if self.casts_shadow {
      opacity
    } else {
      0
    });
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

  /// Puts the peek directly above its window, which is about to be
  /// uncloaked, so the peek keeps covering it until dropped.
  pub fn cover_window(&self) {
    if let Err(err) = self.surrogate.raise_above(self.source) {
      tracing::warn!("Failed to raise a peek above its window: {err}");
    }
  }

  /// Whether the peek should keep covering its uncloaked window: it shows
  /// the window's companion, which is not back on screen yet.
  pub fn awaits_companion(&mut self) -> bool {
    self.companion_wait.awaits(self.surrogate.companion())
  }

  /// The on-screen part of the peek, if it shows anything.
  #[must_use]
  pub fn visible_rect(&self) -> Option<&Rect> {
    self.visible_rect.as_ref()
  }
}
