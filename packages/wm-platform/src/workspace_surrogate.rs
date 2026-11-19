use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  companion::Companion,
  hidden_alpha::HiddenAlpha,
  native_surrogate::{placed_rects, to_logical},
  overlays::copies::{map_rect, scale_alpha, OverlayCopies},
  platform_impl::wallpaper_surface,
  window_class,
  window_edges::compute_border_inset,
  BackdropOverlayParams, CornerStyle, NativeSurrogate, NativeWindow,
  PeekSurface, Rect, StyleBlend, TabBarPlace,
};

/// Thumbnail opacity a surrogate is warmed up at under its still-shown
/// real window: rendered by DWM, but too faint to see.
const WARM_UP_OPACITY: u8 = 1;

/// `(rcSource, rcDestination)` drawing the visible frame of a window that
/// rests at `rect` with its frame at `rest_frame`, now at `frame`, cut at
/// `clip`, into a surrogate whose top-left is at `origin`. `None` when
/// none of the frame is inside `clip`.
///
/// The frame is sampled from `border_inset` on: the invisible resize
/// borders around it show nothing on screen, but a thumbnail can draw them
/// dark, e.g. of a layered (semi-transparent) window.
fn frame_rects(
  rect: &Rect,
  rest_frame: &Rect,
  frame: &Rect,
  clip: &Rect,
  border_inset: RECT,
  origin: (i32, i32),
) -> Option<(RECT, RECT)> {
  let drawn = map_rect(rest_frame, rect, frame);
  let visible = drawn.intersection(clip)?;

  placed_rects(
    &drawn,
    &visible,
    (rest_frame.width(), rest_frame.height()),
    border_inset,
    origin,
  )
}

/// Inset carrying `rect` to `frame_rect`, the window's frame within it.
fn frame_inset(rect: &Rect, frame_rect: &Rect) -> RECT {
  RECT {
    left: frame_rect.left - rect.left,
    top: frame_rect.top - rect.top,
    right: rect.right - frame_rect.right,
    bottom: rect.bottom - frame_rect.bottom,
  }
}

/// The rank from the bottom of windows at `depths` in the z-order, counted
/// in windows down from one of them (`None` for one not found): 0 is the
/// lowest, and ones not found rank lowest.
fn ranks_bottom_first(depths: &[Option<i64>]) -> Vec<usize> {
  let mut bottom_first = (0..depths.len()).collect::<Vec<_>>();
  bottom_first.sort_by_key(|&index| {
    let depth = depths.get(index).copied().flatten();
    (depth.is_some(), std::cmp::Reverse(depth))
  });

  let mut ranks = vec![0; depths.len()];
  for (rank, index) in bottom_first.into_iter().enumerate() {
    if let Some(slot) = ranks.get_mut(index) {
      *slot = rank;
    }
  }
  ranks
}

/// Each of `windows`' rank in the z-order from the bottom, 0 the lowest:
/// the order their [`WorkspaceSurrogate`]s, each just below its window,
/// stack in. Ones not found rank lowest.
#[must_use]
pub fn stacking_ranks(windows: &[HWND]) -> Vec<usize> {
  ranks_bottom_first(&window_class::z_depths(windows))
}

/// Surrogate overlay for a single window participating in a
/// workspace-switch animation.
///
/// Both outgoing and incoming windows move together so the whole workspace
/// slides as a single panel. The surrogate is created at the monitor's
/// working area (`viewport`) and never repositioned; all per-frame
/// animation is expressed via `rcSource`/`rcDestination` in
/// `DwmUpdateThumbnailProperties`, avoiding any per-frame `SetWindowPos`
/// calls, and its window's ring and backdrop are drawn as copies (see
/// `OverlayCopies`). The surrogate is hidden (via `SW_HIDE`) when the
/// visible area is empty.
///
/// A hosted one (see [`hosted`](Self::hosted)) is pinned on a
/// [`PeekSurface`] instead of a window of its own.
pub struct WorkspaceSurrogate {
  inner: NativeSurrogate,
  /// Final screen rect of the window (target position for incoming,
  /// current screen rect for outgoing).
  pub rect: Rect,
  /// Working area used as the surrogate window's fixed position and size.
  ///
  /// The surrogate is created at `viewport` and stays there for the
  /// entire animation; `rcDestination` coordinates are expressed relative
  /// to `viewport`'s top-left corner in every per-frame thumbnail update.
  viewport: Rect,
  /// The window's own alpha (0–255) from the window-effects config. The
  /// thumbnail is drawn at `window_opacity()` instead.
  opacity: u8,
  /// The non-full-opacity end of the opacity animation, as a fraction of
  /// `window_opacity()` (0.0–1.0).
  ///
  /// For outgoing windows this is the final opacity fraction (start =
  /// 1.0, end = `opacity_endpoint`). For incoming windows it is the
  /// initial opacity fraction (start = `opacity_endpoint`, end = 1.0).
  /// At `1.0` (default) the opacity is constant throughout the
  /// animation; at `0.0` the window fully fades out or in.
  opacity_endpoint: f32,
  /// The fraction of `window_opacity()` the window's thumbnail was last
  /// drawn at, or `None` while it warms up (see
  /// [`show_initial`](Self::show_initial)).
  drawn_frac: Option<f32>,
  /// Invisible border insets of the source window, in physical pixels.
  /// Every per-frame `rcSource` sample starts past them, so only the
  /// window's visible frame is drawn (see [`frame_rects`]).
  border_inset: RECT,

  /// Full on-screen rect the source window occupies this animation frame,
  /// *before* the monitor clip. Lets the border overlay follow the window
  /// through a slide: the ring hangs outside the window's own rect, so
  /// it has to be placed from the unclipped position and clipped
  /// separately (see `NativeBorderOverlay::pin_or_slide`). `None` once
  /// the window is entirely off-screen.
  unclipped_rect: Option<Rect>,

  /// The monitor this window rests on, moved and scaled along with the
  /// workspace this frame. Only what the window shows at rest slides with
  /// it, so e.g. the columns a scrolling layout parks past the monitor
  /// edge stay hidden. `None` once the window is entirely off-screen.
  panel: Option<Rect>,

  /// Inset carrying `rect` to the window's *frame* rect, derived at
  /// construction from the `frame_rect` the caller passed alongside it.
  ///
  /// It cannot be inferred: callers disagree on what `rect`
  /// means. The incoming branch applies `total_border_delta` and so
  /// passes a physical rect including the invisible resize border,
  /// while the outgoing branch passes a frame rect that already
  /// excludes it.
  ///
  /// The border overlay is placed from the frame rect in steady state, so
  /// a ring built from a physical rect bulges into the configured gap on
  /// every side and collides with its neighbour's mid-slide. Pinned, the
  /// window is drawn as its frame alone, too (see [`frame_rects`]).
  frame_inset: RECT,

  /// The real window's own alpha while it is hidden by alpha for a
  /// handover with the surrogate (see [`take_over`]).
  ///
  /// [`take_over`]: WorkspaceSurrogate::take_over
  hidden_alpha: HiddenAlpha,

  /// The window the surrogate shows.
  source: isize,

  /// Whether the surrogate was put above its window (see
  /// [`cover_window`]).
  ///
  /// [`cover_window`]: WorkspaceSurrogate::cover_window
  covers_window: bool,

  /// Copies of the window's border ring and backdrop. See [`carry_ring`]
  /// and [`carry_backdrop`].
  ///
  /// [`carry_ring`]: WorkspaceSurrogate::carry_ring
  /// [`carry_backdrop`]: WorkspaceSurrogate::carry_backdrop
  copies: OverlayCopies,

  /// The clip of the surface it draws on, if hosted, whose top-left its
  /// thumbnails are placed from.
  host_clip: Option<Rect>,
}

impl WorkspaceSurrogate {
  /// Creates a hidden surrogate for a workspace-switch animation.
  ///
  /// `viewport` is the monitor's working area, not its bounds: a window
  /// covering a whole monitor counts as a fullscreen app to Explorer.
  /// `rect` is the source window's screen rect, used as the thumbnail
  /// registration dimensions and the reference for per-frame coordinate
  /// math.
  ///
  /// `frame_rect` is the same window's frame rect (what
  /// `DWMWA_EXTENDED_FRAME_BOUNDS` reports), which callers pass explicitly
  /// because they disagree on whether `rect` includes the invisible resize
  /// border. It places the overlays that sit outside the window, i.e. the
  /// border ring, and what of the window is drawn -- pass the same value
  /// as `rect` when the two coincide.
  ///
  /// `opacity_endpoint` controls how far the opacity animates away from
  /// the effect opacity. For outgoing windows pass
  /// `config.opacity_outgoing`; for incoming windows pass
  /// `config.opacity_incoming`. At `1.0` the opacity is held constant;
  /// at `0.0` the window fully fades.
  ///
  /// The surrogate is created hidden. For outgoing windows, call
  /// [`show_initial`] before cloaking the real window to avoid a blank
  /// frame. For incoming windows in stationary styles (fade/zoom), call
  /// [`show_incoming`] before the animation loop to pre-warm the DWM
  /// thumbnail.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  /// [`show_incoming`]: WorkspaceSurrogate::show_incoming
  pub fn new(
    hwnd: HWND,
    rect: &Rect,
    frame_rect: &Rect,
    viewport: &Rect,
    opacity: u8,
    opacity_endpoint: f32,
  ) -> crate::Result<Self> {
    // Spanning the whole viewport, the surrogate must not be rounded, nor
    // deflated by an invisible border it doesn't have.
    let inner = NativeSurrogate::create(
      hwnd,
      viewport,
      rect,
      opacity,
      RECT::default(),
      &CornerStyle::Square,
      // Workspace surrogates should sit just below the source window;
      // they don't compete with resize surrogates since
      // workspace-switch and resize animations are mutually
      // exclusive.
      hwnd,
    )?;

    Ok(Self {
      inner,
      rect: rect.clone(),
      viewport: viewport.clone(),
      opacity,
      opacity_endpoint: opacity_endpoint.clamp(0.0, 1.0),
      drawn_frac: Some(1.0),
      border_inset: compute_border_inset(hwnd),
      unclipped_rect: None,
      panel: None,
      frame_inset: frame_inset(rect, frame_rect),
      hidden_alpha: HiddenAlpha::default(),
      source: hwnd.0,
      covers_window: false,
      copies: OverlayCopies::default(),
      host_clip: None,
    })
  }

  /// Creates a hidden surrogate as [`new`](Self::new) does one pinned to
  /// `viewport`, drawn on `surface` instead of a window of its own, which
  /// goes just below `hwnd` as `SurfaceWindow::admit` decides.
  ///
  /// `None` for a window in the always-on-top band, which would take the
  /// surface and its other tenants there, or with a companion, which only
  /// a window of its own can keep covering at the hand-back (see
  /// [`cover_window`](Self::cover_window)).
  #[must_use]
  pub fn hosted(
    surface: &PeekSurface,
    hwnd: HWND,
    rect: &Rect,
    frame_rect: &Rect,
    viewport: &Rect,
    opacity: u8,
    opacity_endpoint: f32,
  ) -> Option<Self> {
    if window_class::is_topmost(hwnd) || Companion::find(hwnd).is_some() {
      return None;
    }

    Some(Self {
      // Admitted as below a shown window, cloaked or not: a switch's
      // surface stays below every incoming window until the cleanup.
      inner: NativeSurrogate::create_hosted(
        surface, hwnd, rect, opacity, true,
      ),
      rect: rect.clone(),
      viewport: viewport.clone(),
      opacity,
      opacity_endpoint: opacity_endpoint.clamp(0.0, 1.0),
      drawn_frac: Some(1.0),
      border_inset: compute_border_inset(hwnd),
      unclipped_rect: None,
      panel: None,
      frame_inset: frame_inset(rect, frame_rect),
      hidden_alpha: HiddenAlpha::default(),
      source: hwnd.0,
      covers_window: false,
      copies: OverlayCopies::held(),
      host_clip: Some(surface.clip().clone()),
    })
  }

  /// Whether it draws on a `PeekSurface` (see [`hosted`](Self::hosted)).
  #[must_use]
  pub fn is_hosted(&self) -> bool {
    self.host_clip.is_some()
  }

  /// The window it shows.
  #[must_use]
  pub fn source(&self) -> HWND {
    HWND(self.source)
  }

  /// The window's frame where it rests, with its ring if it carries one.
  #[must_use]
  pub fn rest_reach(&self) -> Rect {
    let frame = self.rest_frame();
    match self.copies.ring_rest() {
      Some(ring) => frame.union(ring),
      None => frame,
    }
  }

  /// The first and last registration orders of its thumbnails on the
  /// surface it is hosted on, held ones included (see
  /// `WindowPeek::stacking`). `None` with none.
  #[must_use]
  pub fn stacking(&self) -> Option<(u64, u64)> {
    self
      .inner
      .thumbnail_order()
      .into_iter()
      .chain(self.copies.orders())
      .fold(None, |span, order| {
        Some(span.map_or((order, order), |(first, last): (u64, u64)| {
          (first.min(order), last.max(order))
        }))
      })
  }

  /// Registers every thumbnail anew over everything on its surface, in the
  /// order a surrogate stacks them: backdrop, window, ring. Each part is
  /// let go before its old thumbnails go, the top one first, as
  /// `WindowPeek::register_anew` does. Returns whether it did: nothing
  /// changes if any registration fails.
  pub fn register_anew(&mut self) -> bool {
    let dest = self.hwnd();
    let Some(anew) = self.copies.backdrop_held_anew(dest) else {
      return false;
    };
    let window = match self.inner.thumbnail_held_anew(self.source()) {
      None if self.inner.has_thumbnail() => return false,
      window => window,
    };
    let Some(mut anew) = self.copies.above_window_held_anew(dest, anew)
    else {
      return false;
    };

    self.copies.put_above_window_anew(&mut anew);
    if let Some(window) = window {
      self.inner.put_thumbnail(window);
    }
    self.copies.put_backdrop_anew(anew);
    true
  }

  /// Where its thumbnails are placed from: its own window's top-left, the
  /// viewport's, or its surface's.
  fn origin(&self) -> (i32, i32) {
    let area = self.host_clip.as_ref().unwrap_or(&self.viewport);
    (area.left, area.top)
  }

  /// What the window is drawn cut at, with the workspace's `panel` this
  /// frame on `monitor`: also the viewport for a hosted one, which its
  /// own window's bounds would cut it at.
  fn cut(&self, panel: &Rect, monitor: &Rect) -> Option<Rect> {
    let clip = panel.intersection(monitor)?;
    if self.is_hosted() {
      clip.intersection(&self.viewport)
    } else {
      Some(clip)
    }
  }

  /// Shows or hides it. A hosted one holds or lets go its copies too,
  /// which a window of its own would hide with it.
  fn set_shown(&mut self, is_shown: bool) {
    self.inner.set_visible(is_shown);
    if self.is_hosted() {
      self.copies.set_held(!is_shown);
    }
  }

  /// The window's frame rect where it rests: where the switch starts it
  /// (outgoing) or ends it (incoming).
  #[must_use]
  pub fn rest_frame(&self) -> Rect {
    to_logical(&self.rect, &self.frame_inset)
  }

  /// Draws the window's border ring along with it from now on: `source`,
  /// its overlay window, parked at `rest` (see
  /// `NativeBorderOverlay::park`). Called again with the same `source`,
  /// only updates `rest`, e.g. for a width change that resized it.
  /// Drawn at no opacity until [`cover_ring`](Self::cover_ring). Returns
  /// whether this one does.
  pub fn carry_ring(&mut self, source: HWND, rest: &Rect) -> bool {
    self
      .copies
      .carry_ring(self.hwnd(), source, rest, &self.rect, None, 0)
  }

  /// Whether the surrogate draws the window's border ring (see
  /// [`carry_ring`](Self::carry_ring)).
  #[must_use]
  pub fn carries_ring(&self) -> bool {
    self.copies.carries_ring()
  }

  /// Brings the ring copy to full opacity over the still-shown ring it
  /// duplicates, so the real one can be concealed once this has been
  /// composited, without a frame showing neither.
  ///
  /// Unlike the window, it needs no warm-up: drawn twice in one place, a
  /// ring looks like one.
  pub fn cover_ring(&mut self) {
    self.copies.cover_ring();
  }

  /// Draws the window's backdrop, as `params` show it, under its copy from
  /// now on, its rounded corners cut for a ring `ring_width` wide over
  /// them (see `OverlayCopies::carry_backdrop`). Returns whether it does.
  ///
  /// Before [`carry_ring`](Self::carry_ring), which goes over the window
  /// that this puts back on top of the backdrop.
  pub fn carry_backdrop(
    &mut self,
    params: BackdropOverlayParams,
    ring_width: Option<f32>,
  ) -> bool {
    if self.copies.carries_backdrop_styles(&[params], ring_width) {
      return true;
    }

    let is_new = !self.copies.carries_backdrop();
    let (dest, frame) = (self.hwnd(), self.rest_frame());
    // Cut from a backdrop of the whole monitor, as the overlay is: with
    // parallax, a cut can reach past the viewport, its work area.
    let monitor = wallpaper_surface::monitor_bounds(&self.viewport);
    if !self.copies.carry_backdrop(
      dest,
      &[params],
      ring_width,
      &monitor,
      &self.rect,
      &frame,
    ) {
      return false;
    }

    self
      .copies
      .show_backdrop_blend(Some(&StyleBlend::settled(params)));
    if is_new {
      self.copies.show_backdrop(true);
    }
    self.inner.raise_thumbnail(HWND(self.source));
    true
  }

  /// Whether the surrogate draws the window's backdrop (see
  /// [`carry_backdrop`](Self::carry_backdrop)).
  #[must_use]
  pub fn carries_backdrop(&self) -> bool {
    self.copies.carries_backdrop()
  }

  /// Brings the backdrop copy to full opacity over the backdrop it
  /// duplicates, shown or not, as [`cover_ring`](Self::cover_ring) does
  /// the ring: drawn twice in one place, a backdrop looks like one.
  pub fn cover_backdrop(&mut self) {
    self.copies.set_backdrop_opacity(u8::MAX);
  }

  /// Draws the tab bar `source` of the stack its window is the active tab
  /// of, at `place` beside the window, along with it from now on, the bar
  /// painted at its rest frame and cloaked (see
  /// `NativeStackTabBar::set_cloaked`). Called again with the same
  /// `source`, only updates `place`. Returns whether it does.
  ///
  /// The copy goes under the window, whose thumbnail is registered anew
  /// over it, the new one up before the old one goes, and the ring over
  /// both.
  pub fn carry_tab_bar(
    &mut self,
    source: HWND,
    place: TabBarPlace,
  ) -> bool {
    let dest = self.hwnd();
    if self.copies.carry_bar(dest, source, place) {
      if let Some(window) = self.inner.thumbnail_held_anew(self.source()) {
        self.inner.put_thumbnail(window);
      }
      self.copies.raise_above_window(dest);
    }
    self.copies.carries_bar()
  }

  /// Whether the surrogate draws its stack's tab bar (see
  /// [`carry_tab_bar`](Self::carry_tab_bar)).
  #[must_use]
  pub fn carries_tab_bar(&self) -> bool {
    self.copies.carries_bar()
  }

  /// Hides the backdrop, e.g. as its window leaves the screen for good.
  pub fn hide_backdrop(&mut self) {
    self.copies.show_backdrop(false);
  }

  /// Hides the ring, e.g. as its window leaves the screen for good.
  pub fn hide_ring(&mut self) {
    self.copies.hide_ring();
  }

  /// Draws the window at `window` opacity and its ring and backdrop at
  /// `around`.
  fn set_opacities(&mut self, window: u8, around: u8) {
    self.inner.set_window_opacity(window);
    self.copies.set_opacity(around);
    self.copies.set_bar_opacity(around);
  }

  /// Draws the window, ring and backdrop at `frac` of their own
  /// opacities.
  fn set_opacity_frac(&mut self, frac: f32) {
    self.drawn_frac = Some(frac);
    self.set_opacities(
      scale_alpha(self.window_opacity(), frac),
      scale_alpha(u8::MAX, frac),
    );
  }

  /// Opacity the window's thumbnail is drawn at outside of fades: the
  /// window's own, or opaque for its companion (see
  /// `NativeSurrogate::stand_in_opacity`).
  fn window_opacity(&self) -> u8 {
    self.inner.stand_in_opacity(self.opacity)
  }

  /// Switches the window's thumbnail to its companion if one was shown
  /// since the surrogate last looked, e.g. for a window shades had not
  /// covered yet as the switch began. Looks at most every 100 ms (see
  /// `NativeSurrogate::rechecks_companion`). None for a hosted one (see
  /// [`adopt_companion`](Self::adopt_companion)).
  pub fn refresh_companion(&mut self) {
    if !self.is_hosted() && self.inner.rechecks_companion() {
      self.adopt_companion();
    }
  }

  /// [`refresh_companion`](Self::refresh_companion), unthrottled, e.g.
  /// right before the hand-back, which then takes the companion's path.
  /// A surrogate whose thumbnail is hidden, e.g. handed back, keeps it.
  ///
  /// The companion is drawn as the window was, then at the same fraction
  /// of its own opacity, and the ring copy is put back over it.
  ///
  /// None for a hosted one, which can't cover its window at the hand-back
  /// as one drawing a companion has to (see
  /// [`cover_window`](Self::cover_window)): it draws the window as it is.
  pub fn adopt_companion(&mut self) {
    if self.is_hosted() {
      return;
    }
    let opacity = self.drawn_frac.map_or(WARM_UP_OPACITY, |frac| {
      scale_alpha(self.window_opacity(), frac)
    });
    if !self.inner.adopt_companion(HWND(self.source), opacity) {
      return;
    }

    if let Some(frac) = self.drawn_frac {
      self
        .inner
        .set_window_opacity(scale_alpha(self.window_opacity(), frac));
    }
    let dest = self.hwnd();
    self.copies.raise_ring(dest);
  }

  /// Full on-screen rect of the source window for the current animation
  /// frame, before the monitor clip. `None` once the window is entirely
  /// off-screen.
  #[must_use]
  pub fn unclipped_rect(&self) -> Option<&Rect> {
    self.unclipped_rect.as_ref()
  }

  /// The monitor this window rests on as it is moved and scaled this
  /// frame, i.e. what overlays sliding along have to be clipped to.
  #[must_use]
  pub fn panel(&self) -> Option<&Rect> {
    self.panel.as_ref()
  }

  /// Hides the surrogate and clears the cached frame rects, for a frame
  /// where the window has nothing visible left to draw.
  fn clear_frame(&mut self) {
    self.set_shown(false);
    self.copies.hide_ring();
    self.unclipped_rect = None;
    self.panel = None;
  }

  /// Draws the part of the window inside both `panel` and `monitor`, with
  /// the window at `frame` this frame. Only the thumbnail changes: the
  /// surrogate never moves, so cutting the thumbnail at the monitor as
  /// well leaves it the one thing that changes.
  ///
  /// Returns whether anything is left to draw.
  fn show_frame(
    &mut self,
    frame: &Rect,
    panel: Rect,
    monitor: &Rect,
  ) -> bool {
    let Some(clip) = self.cut(&panel, monitor) else {
      self.clear_frame();
      return false;
    };

    let Some((rc_src, rc_dst)) = frame_rects(
      &self.rect,
      &self.rest_frame(),
      frame,
      &clip,
      self.border_inset,
      self.origin(),
    ) else {
      self.clear_frame();
      return false;
    };

    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    let origin = self.origin();
    self.copies.place(frame, &clip, origin);
    self.set_shown(true);
    self.panel = Some(panel);

    true
  }

  /// The opacity fraction (0.0 - 1.0) this surrogate is being drawn at for
  /// `progress`, so overlays tracking it can fade in step rather than
  /// staying at full strength over a fading window.
  #[must_use]
  pub fn opacity_frac(&self, progress: f32, is_incoming: bool) -> f32 {
    let (start_frac, end_frac): (f32, f32) = if is_incoming {
      (self.opacity_endpoint, 1.0)
    } else {
      (1.0, self.opacity_endpoint)
    };

    (start_frac + (end_frac - start_frac) * progress).clamp(0.0, 1.0)
  }

  /// `HWND` of this surrogate, or of its surface if hosted.
  ///
  /// Used as the backdrop-overlay tracker's z-order anchor while a
  /// workspace-switch animation is active: the surrogate is what's
  /// actually visible on screen (the real window is cloaked for the
  /// duration), so the overlay must sit directly behind *it*, not the
  /// (hidden) real window.
  #[must_use]
  pub fn hwnd(&self) -> HWND {
    self.inner.hwnd()
  }

  /// Hides the DWM thumbnail without destroying it or hiding the surrogate
  /// window, handing a window readied by [`prepare_hand_back`] its alpha
  /// back first.
  ///
  /// Called right after the real window is uncloaked. With both shown,
  /// DWM blends the thumbnail (at configured opacity) on top of the real
  /// window (also at configured opacity), a double-blend that appears
  /// fully opaque. One showing a companion, drawn opaque, first waits for
  /// the companion to cover the window again (see
  /// [`awaits_companion`](Self::awaits_companion)).
  ///
  /// The ring copy stays: the ring is revealed under it only after this,
  /// and it goes with the surrogate. So does the backdrop copy, unless the
  /// surrogate covers its window, which the copy would then hide.
  ///
  /// [`prepare_hand_back`]: WorkspaceSurrogate::prepare_hand_back
  pub fn hide_thumbnail(&mut self) {
    self.restore_alpha();
    self.inner.hide_thumbnail();

    if self.covers_window {
      self.hide_backdrop();
    }
  }

  /// Whether the surrogate should keep covering its uncloaked window
  /// before [`hide_thumbnail`](Self::hide_thumbnail): it shows the
  /// window's companion, which is not back over the window yet.
  pub fn awaits_companion(&mut self) -> bool {
    self.inner.awaits_companion()
  }

  /// Whether the surrogate draws its window's companion (e.g. shades'
  /// recolored copy) rather than the window itself.
  #[must_use]
  pub fn shows_companion(&self) -> bool {
    self.inner.shows_companion()
  }

  /// Puts the surrogate above its window, unless it already is, so it
  /// covers the window until handing it back.
  ///
  /// It otherwise sits just below the window, where an opaque window
  /// shows over it as it really is (e.g. without its companion's colors)
  /// once uncloaked or raised. A semi-transparent one is hidden by its
  /// alpha instead (see [`prepare_hand_back`]).
  ///
  /// None for a hosted one: its surface stays below every window of the
  /// switch, and it never draws a companion.
  ///
  /// [`prepare_hand_back`]: WorkspaceSurrogate::prepare_hand_back
  pub fn cover_window(&mut self) {
    if self.is_hosted() {
      return;
    }
    self.covers_window = true;
    self.inner.keep_above(HWND(self.source));
  }

  /// Whether [`cover_window`](Self::cover_window) put the surrogate above
  /// its window.
  #[must_use]
  pub fn covers_window(&self) -> bool {
    self.covers_window
  }

  /// Shows the surrogate with the thumbnail at the window's natural
  /// (unscaled) position within the monitor viewport, cut to it like
  /// every later frame, but too faint to see, so what is under it still
  /// shows as it is while DWM warms the thumbnail up. For an outgoing
  /// window, [`take_over`] then swaps the two.
  ///
  /// [`take_over`]: WorkspaceSurrogate::take_over
  pub fn show_initial(&mut self) {
    self.show_at_rest();
    self.set_opacities(WARM_UP_OPACITY, WARM_UP_OPACITY);
    self.drawn_frac = None;
  }

  /// Takes over showing the window from `window`, which is about to be
  /// cloaked: the thumbnail goes to its effect opacity and a
  /// semi-transparent `window` to no alpha at all, in that order, so the
  /// swap lands in one frame and the cloak unseen (see `HiddenAlpha`).
  /// `restore_alpha` gives `window` its alpha back once the cloak is on
  /// screen.
  pub fn take_over(&mut self, window: &NativeWindow) {
    self.apply_effect_opacity();
    self.hidden_alpha.hide(&window.inner);
  }

  /// Readies handing the window back to `window`, which is about to be
  /// uncloaked: a semi-transparent `window` goes to no alpha, so its
  /// uncloak lands unseen and `hide_thumbnail` swaps the two in one frame
  /// (see [`take_over`]). An opaque one that a companion draws over is
  /// covered instead (see [`cover_window`]). The wait for a companion
  /// starts afresh (see [`awaits_companion`](Self::awaits_companion)).
  ///
  /// [`take_over`]: WorkspaceSurrogate::take_over
  /// [`cover_window`]: WorkspaceSurrogate::cover_window
  pub fn prepare_hand_back(&mut self, window: &NativeWindow) {
    self.inner.begin_hand_back();
    self.hidden_alpha.hide(&window.inner);

    if self.shows_companion() && !self.hidden_alpha.is_hiding() {
      self.cover_window();
    }
  }

  /// Gives the real window back the alpha a handover set aside, unless it
  /// was set again since, e.g. by a focus change's effects. No-op
  /// otherwise.
  pub fn restore_alpha(&mut self) {
    self.hidden_alpha.restore();
  }

  /// Draws the window where it rests, as the first frame of the switch.
  fn show_at_rest(&mut self) {
    let rect = self.rect.clone();
    let viewport = self.viewport.clone();
    self.show_frame(&rect, viewport.clone(), &viewport);
  }

  /// Updates the DWM thumbnail opacity to its effect opacity (see
  /// [`window_opacity`](Self::window_opacity)), and the ring copy's to
  /// full, without changing the surrogate window position or size.
  pub fn apply_effect_opacity(&mut self) {
    self.set_opacity_frac(1.0);
  }

  /// Retargets the window's own alpha, for a window whose transparency
  /// transitions with its focus state during the switch. Takes effect
  /// with the next per-frame update, unless it shows a companion.
  pub fn set_effect_opacity(&mut self, opacity: u8) {
    self.opacity = opacity;
  }

  /// Shows the surrogate at the incoming animation start opacity, with the
  /// thumbnail pre-positioned at the window's location within the monitor
  /// viewport.
  ///
  /// Use for incoming windows in stationary transitions (fade, zoom) so
  /// DWM warms the thumbnail before the animation begins. The start
  /// opacity is derived from `opacity_endpoint`: `opacity_endpoint *
  /// window_opacity()`. At `opacity_endpoint = 0.0` the surrogate is
  /// invisible but the thumbnail is registered; at `1.0` it starts fully
  /// opaque. [`update_fade`] or [`update_zoom`] then drives the
  /// per-frame opacity from this initial state.
  ///
  /// [`update_fade`]: WorkspaceSurrogate::update_fade
  /// [`update_zoom`]: WorkspaceSurrogate::update_zoom
  pub fn show_incoming(&mut self) {
    self.show_at_rest();
    self.apply_opacity_at(0.0, true);
  }

  /// Advances the surrogate opacity for a fade-only transition.
  ///
  /// The surrogate stays at its target rect; only the window opacity is
  /// lerped each frame to produce a crossfade without positional
  /// movement. No repositioning is needed: [`show_initial`] /
  /// [`show_incoming`] already placed the surrogate at its natural rect.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  /// [`show_incoming`]: WorkspaceSurrogate::show_incoming
  pub fn update_fade(&mut self, eased_progress: f32, is_incoming: bool) {
    self.apply_opacity_at(eased_progress, is_incoming);
  }

  /// Animates a zoom-from-center transition to `eased_progress` (0.0 →
  /// 1.0).
  ///
  /// Each surrogate independently zooms in (incoming) or out (outgoing)
  /// from its own screen-space center: the destination rect grows/shrinks
  /// via `rcDestination`. Opacity is lerped according to
  /// `opacity_endpoint`.
  #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
  pub fn update_zoom(&mut self, eased_progress: f32, is_incoming: bool) {
    let t = if is_incoming {
      eased_progress
    } else {
      1.0 - eased_progress
    };

    let w = self.rect.width();
    let h = self.rect.height();
    let half_w = (w as f32 / 2.0 * t).round() as i32;
    let half_h = (h as f32 / 2.0 * t).round() as i32;

    if half_w <= 0 || half_h <= 0 {
      self.clear_frame();
      return;
    }

    // Anchor at the window's own screen-space center — not the surrogate's
    // local origin — so the zoom converges on the window's actual tile
    // rather than the monitor's top-left corner.
    let cx = self.rect.left + w / 2;
    let cy = self.rect.top + h / 2;
    let frame =
      Rect::from_ltrb(cx - half_w, cy - half_h, cx + half_w, cy + half_h);
    let panel = self.viewport.scale_about(cx, cy, t);
    let monitor = self.viewport.clone();

    if self.show_frame(&frame, panel, &monitor) {
      self.apply_opacity_at(eased_progress, is_incoming);
    }
  }

  /// Draws the window and its ring at their opacity at `progress` (0.0 →
  /// 1.0).
  ///
  /// Outgoing: lerps from full → `opacity_endpoint` of it. Incoming: the
  /// reverse. When `opacity_endpoint` is `1.0` (default), the opacity is
  /// constant — no fade.
  fn apply_opacity_at(&mut self, progress: f32, is_incoming: bool) {
    self.set_opacity_frac(self.opacity_frac(progress, is_incoming));
  }

  /// Advances the surrogate along one axis with a simultaneous
  /// whole-workspace scale to `eased_progress` (0.0 → 1.0);
  /// `is_vertical` selects the slide axis (`false` = horizontal, `true`
  /// = vertical).
  ///
  /// Each surrogate is positioned at the scaled screen coordinates of its
  /// window (scaling from the monitor center), so the entire workspace
  /// shrinks/grows as one unit. The outgoing workspace scales from `1.0`
  /// to `1.0 - zoom_factor`; the incoming scales from `1.0 -
  /// zoom_factor` to `1.0`; at a `zoom_factor` of `0.0` it only slides.
  /// `slide_distance` controls travel on the primary axis.
  ///
  /// Computes a per-frame scale from the monitor center (both axes)
  /// combined with a slide offset on the primary axis, then draws the
  /// resulting frame through `show_frame`.
  #[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
  )]
  pub fn slide_axis(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor: &Rect,
    slide_distance: i32,
    zoom_factor: f32,
    is_vertical: bool,
  ) {
    let (monitor_x, monitor_width, monitor_y, monitor_height) =
      (monitor.x(), monitor.width(), monitor.y(), monitor.height());

    // Outgoing: scale 1.0 → (1 - zoom_factor) as it exits.
    // Incoming: scale (1 - zoom_factor) → 1.0 as it enters.
    let zoom_t = if is_incoming {
      1.0 - eased_progress
    } else {
      eased_progress
    };
    let scale = 1.0 - zoom_factor * zoom_t;

    if scale <= 0.0 {
      self.clear_frame();
      return;
    }

    // Slide offset on the primary axis.
    let slide_offset = if is_incoming {
      (direction as f32 * slide_distance as f32 * (1.0 - eased_progress))
        as i32
    } else {
      (-direction as f32 * slide_distance as f32 * eased_progress) as i32
    };

    // The whole workspace scales from the monitor center, then slides on
    // the primary axis.
    let cx = monitor_x + monitor_width / 2;
    let cy = monitor_y + monitor_height / 2;
    let (dx, dy) = if is_vertical {
      (0, slide_offset)
    } else {
      (slide_offset, 0)
    };
    let transform = |rect: &Rect| {
      let zoomed = rect.scale_about(cx, cy, scale);
      zoomed.translate_to_coordinates(zoomed.x() + dx, zoomed.y() + dy)
    };
    let frame = transform(&self.rect);
    let panel = transform(&self.viewport);

    if self.show_frame(&frame, panel, monitor) {
      self.unclipped_rect = Some(to_logical(&frame, &self.frame_inset));
      self.apply_opacity_at(eased_progress, is_incoming);
    }
  }
}

impl Drop for WorkspaceSurrogate {
  fn drop(&mut self) {
    // Unregistered before `inner` destroys the window they are drawn on.
    self.copies.clear();
  }
}

#[cfg(test)]
mod tests {
  use super::{frame_rects, ranks_bottom_first, Rect, RECT};

  const MONITOR: Rect = Rect {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1080,
  };

  const BORDER: RECT = RECT {
    left: 7,
    top: 0,
    right: 7,
    bottom: 7,
  };

  /// `(left, top, right, bottom)` of a `RECT`.
  type Edges = (i32, i32, i32, i32);

  fn ltrb(rect: RECT) -> Edges {
    (rect.left, rect.top, rect.right, rect.bottom)
  }

  /// `(rcSource, rcDestination)` of a window whose frame rests at (100,
  /// 100, 800x600) inside its 7 px resize borders, with its rect at
  /// `rect`.
  fn drawn_at(rect: &Rect) -> Option<(Edges, Edges)> {
    let window = Rect::from_xy(93, 100, 814, 607);
    let frame = Rect::from_xy(100, 100, 800, 600);

    frame_rects(&window, &frame, rect, &MONITOR, BORDER, (0, 0))
      .map(|(src, dst)| (ltrb(src), ltrb(dst)))
  }

  #[test]
  fn only_the_frame_is_drawn_where_it_rests() {
    assert_eq!(
      drawn_at(&Rect::from_xy(93, 100, 814, 607)),
      Some(((7, 0, 807, 600), (100, 100, 900, 700)))
    );
  }

  #[test]
  fn frame_slides_with_its_window() {
    assert_eq!(
      drawn_at(&Rect::from_xy(493, 100, 814, 607)),
      Some(((7, 0, 807, 600), (500, 100, 1300, 700)))
    );
  }

  #[test]
  fn frame_is_cut_at_the_clip() {
    assert_eq!(
      drawn_at(&Rect::from_xy(1493, 100, 814, 607)),
      Some(((7, 0, 427, 600), (1500, 100, 1920, 700)))
    );
  }

  /// Only the borders on screen: nothing of the window shows.
  #[test]
  fn borders_alone_draw_nothing() {
    assert_eq!(drawn_at(&Rect::from_xy(1915, 100, 814, 607)), None);
  }

  #[test]
  fn frame_scales_with_its_window() {
    assert_eq!(
      drawn_at(&Rect::from_xy(93, 100, 407, 304)),
      Some(((7, 0, 807, 600), (97, 100, 497, 400)))
    );
  }

  /// Deeper in the z-order is lower; windows above the first are higher.
  #[test]
  fn windows_rank_from_the_bottom_of_the_z_order() {
    assert_eq!(
      ranks_bottom_first(&[Some(0), Some(3), Some(-2), Some(1)]),
      vec![2, 0, 3, 1]
    );
  }

  #[test]
  fn windows_not_found_rank_lowest() {
    assert_eq!(
      ranks_bottom_first(&[Some(0), None, Some(5)]),
      vec![2, 0, 1]
    );
    assert!(ranks_bottom_first(&[]).is_empty());
  }
}
