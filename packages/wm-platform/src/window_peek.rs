use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  hidden_alpha::HiddenAlpha,
  native_surrogate::{placed_rects, to_logical},
  overlay_copies::{map_rect, scale_alpha, OverlayCopies},
  perf::{self, Stage},
  platform_impl::wallpaper_surface,
  window_edges::compute_border_inset,
  BackdropOverlayParams, Color, CornerStyle, NativeSurrogate,
  NativeWindow, PeekSurface, Rect, StyleBlend, TabBarPlace,
};

/// Thumbnail opacity a peek is warmed up at under its still-shown real
/// window: rendered by DWM, but too faint to see.
const WARM_UP_OPACITY: u8 = 1;

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
///
/// A hosted one (see [`hosted`](Self::hosted)) draws on a `PeekSurface`
/// shared with other peeks, as a pinned one spanning the surface's clip.
pub struct WindowPeek {
  /// Declared before `surrogate`, so they are unregistered before the
  /// window they are drawn on goes.
  copies: OverlayCopies,
  surrogate: NativeSurrogate,
  /// The area the peek spans and is clipped to. It never moves: each
  /// frame only places the thumbnail in it, along with copies of the
  /// window's ring and backdrop, all in the same DWM updates (see
  /// `OverlayCopies`). As a `WorkspaceSurrogate` pinned to its
  /// viewport.
  viewport: Rect,
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
  /// Whether it stands by under a workspace switch's surrogate (see
  /// [`stand_by`](Self::stand_by)).
  stands_by: bool,
}

impl WindowPeek {
  /// Creates a hidden peek of `window`, which is at `rect`, spanning and
  /// clipped to `clip`.
  pub fn new(
    window: &NativeWindow,
    rect: &Rect,
    clip: &Rect,
  ) -> crate::Result<Self> {
    let hwnd = window.hwnd();
    let border_inset = compute_border_inset(hwnd);

    // Just below the window it shows, like the other surrogates. Spanning
    // the clip, it must not be rounded: the thumbnail shows the window's
    // own corners.
    let surrogate = NativeSurrogate::create(
      hwnd,
      clip,
      rect,
      u8::MAX,
      RECT::default(),
      &CornerStyle::Square,
      hwnd,
    )?;

    Ok(Self {
      copies: OverlayCopies::default(),
      surrogate,
      viewport: clip.clone(),
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
      stands_by: false,
    })
  }

  /// Creates a hidden peek of `window`, which is at `rect`, drawn on
  /// `surface` as a pinned one spanning its clip is on a window of its
  /// own. `surface` goes by `window`, which `is_shown` or is cloaked, as
  /// `SurfaceWindow::admit` decides.
  #[must_use]
  pub fn hosted(
    window: &NativeWindow,
    rect: &Rect,
    surface: &PeekSurface,
    is_shown: bool,
  ) -> Self {
    let hwnd = window.hwnd();

    Self {
      copies: OverlayCopies::held(),
      surrogate: NativeSurrogate::create_hosted(
        surface,
        hwnd,
        rect,
        u8::MAX,
        is_shown,
      ),
      viewport: surface.clip().clone(),
      source: hwnd,
      border_inset: compute_border_inset(hwnd),
      rest: rect.clone(),
      drawn: Drawn::Shown,
      shows_backdrop: false,
      opacity: u8::MAX,
      frac: 1.0,
      hidden_alpha: HiddenAlpha::default(),
      frame_rect: None,
      visible_rect: None,
      stands_by: false,
    }
  }

  /// A peek of `window`, at `rest`, made ready on `surface` before any
  /// motion asks for it, as [`set_ready`](Self::set_ready) keeps a landed
  /// one: its window thumbnail, the backdrop copies in `styles` under a
  /// ring `ring_width` wide, and with `ring`, the copy of that ring
  /// overlay cut by its corner reach (see
  /// [`reserve_ring`](Self::reserve_ring)), all registered held in the
  /// order a tile stacks them. The surface is not moved.
  #[must_use]
  pub fn ready(
    window: &NativeWindow,
    rest: &Rect,
    surface: &PeekSurface,
    styles: &[Option<BackdropOverlayParams>],
    ring_width: Option<f32>,
    ring: Option<(HWND, Option<i32>)>,
  ) -> Self {
    let hwnd = window.hwnd();
    let mut this = Self {
      copies: OverlayCopies::held(),
      surrogate: NativeSurrogate::create_ready(surface, hwnd, rest),
      viewport: surface.clip().clone(),
      source: hwnd,
      border_inset: compute_border_inset(hwnd),
      rest: rest.clone(),
      drawn: Drawn::Shown,
      shows_backdrop: false,
      opacity: u8::MAX,
      frac: 1.0,
      hidden_alpha: HiddenAlpha::default(),
      frame_rect: None,
      visible_rect: None,
      stands_by: false,
    };
    this.carry_styles(styles, ring_width);
    if let Some((source, corner_reach)) = ring {
      this.ready_ring(source, corner_reach);
    }
    this
  }

  /// Has a ready peek keep a copy of the ring overlay `source` cut by
  /// `corner_reach`, registered held over its other thumbnails, for its
  /// next motion's [`carry_ring`](Self::carry_ring) to take; one it keeps
  /// already in that cut stays. Returns whether it keeps one.
  pub fn ready_ring(
    &mut self,
    source: HWND,
    corner_reach: Option<i32>,
  ) -> bool {
    self.surrogate.is_ready() && self.reserve_ring(source, corner_reach)
  }

  /// Whether a ready peek keeps a copy of the ring overlay `source` cut by
  /// `corner_reach` (see [`ready_ring`](Self::ready_ring)).
  #[must_use]
  pub fn keeps_ready_ring(
    &self,
    source: HWND,
    corner_reach: Option<i32>,
  ) -> bool {
    self.copies.keeps_spare_ring(source, corner_reach)
  }

  /// Whether it draws on a `PeekSurface` (see [`hosted`](Self::hosted)).
  #[must_use]
  pub fn is_hosted(&self) -> bool {
    self.surrogate.is_hosted()
  }

  /// Keeps a hosted peek's surface from going down from now on, its window
  /// just cloaked under it with no raise to follow (see
  /// `SurfaceWindow::hold_place`). None for one of its own.
  pub fn hold_place(&self) {
    self.surrogate.hold_place();
  }

  /// Keeps a hosted peek, with every registration it has, for its tile's
  /// next motion (see [`reclaim`](Self::reclaim)): held, drawing nothing,
  /// on its surface as a ready peek rather than a tenant. Gives its window
  /// back the alpha a handover set aside, as dropping it would. None for
  /// one of its own.
  pub fn set_ready(&mut self) {
    if !self.is_hosted() {
      return;
    }

    self.hide();
    self.hidden_alpha.restore();
    self.copies.set_aside();
    self.surrogate.retire();
  }

  /// Has a peek kept by [`set_ready`](Self::set_ready) draw `window`, at
  /// `rect`, for a new motion as [`hosted`](Self::hosted) makes one anew,
  /// before [`carry_styles`](Self::carry_styles) with `styles` and
  /// `ring_width`: every field as there, every registration kept while
  /// what it was made from holds, and back on its surface, which goes just
  /// below `window` as `SurfaceWindow::admit` decides.
  ///
  /// A tile's registrations stack backdrop, window, fill, ring. Those
  /// kept keep their order; a window thumbnail or backdrop registered
  /// anew goes on top, so the fill and ring set aside above it go, to be
  /// registered on top of it in turn as they are carried.
  pub fn reclaim(
    &mut self,
    window: &NativeWindow,
    rect: &Rect,
    styles: &[Option<BackdropOverlayParams>],
    ring_width: Option<f32>,
  ) {
    let hwnd = window.hwnd();
    let border_inset = compute_border_inset(hwnd);
    let frame = to_logical(rect, &border_inset);

    let backdrops = copyable(styles);
    let monitor = (!backdrops.is_empty())
      .then(|| wallpaper_surface::monitor_bounds(&self.viewport));
    let keeps_backdrop = self.copies.reclaim_backdrop(
      &backdrops,
      ring_width,
      monitor.as_ref(),
      rect,
      &frame,
    );
    let keeps_window = !self.surrogate.reclaim(hwnd, rect, u8::MAX);
    if !keeps_backdrop || !keeps_window {
      self.copies.drop_spares();
    }

    self.source = hwnd;
    self.border_inset = border_inset;
    self.rest = rect.clone();
    self.drawn = Drawn::Shown;
    self.shows_backdrop = false;
    self.opacity = u8::MAX;
    self.frac = 1.0;
    self.hidden_alpha = HiddenAlpha::default();
    self.frame_rect = None;
    self.visible_rect = None;
    self.stands_by = false;
  }

  /// The first and last registration orders of the thumbnails it has on
  /// the window it draws on, those held included: it draws wholly over
  /// another peek there whose last is below its first. `None` with none.
  #[must_use]
  pub fn stacking(&self) -> Option<(u64, u64)> {
    self
      .surrogate
      .thumbnail_order()
      .into_iter()
      .chain(self.copies.orders())
      .fold(None, |span, order| {
        Some(span.map_or((order, order), |(first, last): (u64, u64)| {
          (first.min(order), last.max(order))
        }))
      })
  }

  /// Registers every thumbnail of the peek anew, over everything on the
  /// window it draws on, in the order its tile stacks them: backdrop,
  /// window, fill, ring. All go up held, then each part is let go before
  /// its old thumbnails go, the top one first, so no composition shows
  /// the tile without one of them (a semi-transparent window may show
  /// twice for one). Returns whether it did: nothing changes if any
  /// registration fails.
  pub fn register_anew(&mut self) -> bool {
    let dest = self.surrogate.hwnd();
    let Some(anew) = self.copies.backdrop_held_anew(dest) else {
      return false;
    };
    let window = match self.surrogate.thumbnail_held_anew(self.source) {
      None if self.surrogate.has_thumbnail() => return false,
      window => window,
    };
    let Some(mut anew) = self.copies.above_window_held_anew(dest, anew)
    else {
      return false;
    };

    self.copies.put_above_window_anew(&mut anew);
    if let Some(window) = window {
      self.surrogate.put_thumbnail(window);
    }
    self.copies.put_backdrop_anew(anew);
    // Left below the tile now, they would stack under it once carried.
    self.copies.drop_spares();
    true
  }

  /// The window the peek shows.
  #[must_use]
  pub fn source(&self) -> HWND {
    self.source
  }

  /// Whether the peek draws its window at all: DWM refuses a thumbnail of
  /// some, e.g. an elevated window.
  #[must_use]
  pub fn has_thumbnail(&self) -> bool {
    self.surrogate.has_thumbnail()
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
  /// Shown while [`show_backdrop`](Self::show_backdrop) says so. Rounded
  /// corners are
  /// cut for a ring `ring_width` wide over them (see
  /// `OverlayCopies::carry_backdrop`).
  ///
  /// Before [`carry_ring`](Self::carry_ring), which goes over the window
  /// that this may put back on top of the backdrop.
  pub fn carry_styles(
    &mut self,
    styles: &[Option<BackdropOverlayParams>],
    ring_width: Option<f32>,
  ) {
    let dest = self.surrogate.hwnd();
    let frame = to_logical(&self.rest, &self.border_inset);
    let backdrops = copyable(styles);

    if !backdrops.is_empty()
      && !self.copies.carries_backdrop_styles(&backdrops, ring_width)
    {
      // Cut from a backdrop of the whole monitor, which the clip, its work
      // area, is part of.
      let monitor = wallpaper_surface::monitor_bounds(&self.viewport);

      if self.copies.carry_backdrop(
        dest, &backdrops, ring_width, &monitor, &self.rest, &frame,
      ) {
        self.surrogate.raise_thumbnail(self.source);
        // Under the window now, they would stack below it once carried.
        self.copies.drop_spares();
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
  /// over a real window handed back, an opaque copy would hide it. The
  /// fill goes with it, drawn only with the window (see
  /// [`carry_fill`](Self::carry_fill)).
  fn apply_backdrop(&mut self) {
    self.copies.show_backdrop(
      self.shows_backdrop && self.drawn != Drawn::HandedBack,
    );
    self.copies.show_fill(self.drawn == Drawn::Shown);
  }

  /// Draws the window's border ring along with it from now on, at once,
  /// over the ring itself: `source`, its overlay window, at `ring_rect`
  /// around the window at `window_rect`, both where they are now. With
  /// `corner_reach` (see `NativeBorderOverlay::corner_reach`), it is drawn
  /// in cells, keeping its width and corners as its window's size changes
  /// (see [`cut_ring`](Self::cut_ring)). Called again with the same
  /// `source`, only updates where the two are and how it is cut.
  /// Returns whether the peek does.
  pub fn carry_ring(
    &mut self,
    source: HWND,
    ring_rect: &Rect,
    window_rect: &Rect,
    corner_reach: Option<i32>,
  ) -> bool {
    let dest = self.surrogate.hwnd();
    let carries = self.copies.carry_ring(
      dest,
      source,
      ring_rect,
      window_rect,
      corner_reach,
    );
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

  /// Registers the copy of its window's ring, `source`, cut by
  /// `corner_reach`, now, held, right over the window and backdrop
  /// registered so far, for [`carry_ring`](Self::carry_ring) to take (see
  /// `OverlayCopies::reserve_ring`): a tile's thumbnails then stack
  /// together on its surface, over the tiles registered before it, as
  /// they would be restacked. Only a hosted peek. Returns whether it did.
  pub fn reserve_ring(
    &mut self,
    source: HWND,
    corner_reach: Option<i32>,
  ) -> bool {
    if !self.is_hosted() {
      return false;
    }

    self
      .copies
      .reserve_ring(self.surrogate.hwnd(), source, corner_reach)
  }

  /// Draws the ring copy whole, or with `corner_reach` in cells, from now
  /// on, where it was carried (see [`carry_ring`](Self::carry_ring)).
  pub fn cut_ring(&mut self, corner_reach: Option<i32>) {
    self.copies.cut_ring(self.surrogate.hwnd(), corner_reach);
  }

  /// Where the ring's overlay window was when the peek last carried it.
  #[must_use]
  pub fn carried_ring_rect(&self) -> Option<&Rect> {
    self.copies.ring_rest()
  }

  /// Draws the window's stack's tab bar, the window `source`, at `place`
  /// beside it from now on: over its backdrop, under the window, at full
  /// whatever the window is drawn at, until
  /// [`drop_tab_bar`](Self::drop_tab_bar) or the peek goes. Called again
  /// with the same `source`, only updates its place. Returns whether the
  /// peek does: only a hosted one, whose surface a bar of its own would be
  /// under with every other tile there.
  ///
  /// A new copy goes under the window's own thumbnail, registered anew
  /// over it, then the fill and ring over that, each new one up before
  /// its old one goes.
  pub fn carry_tab_bar(
    &mut self,
    source: HWND,
    place: TabBarPlace,
  ) -> bool {
    if !self.is_hosted() {
      return false;
    }

    let dest = self.surrogate.hwnd();
    if self.copies.carry_bar(dest, source, place) {
      if let Some(window) = self.surrogate.thumbnail_held_anew(self.source)
      {
        self.surrogate.put_thumbnail(window);
      }
      self.copies.raise_above_window(dest);
      // Left below the window now, they would stack under it once carried.
      self.copies.drop_spares();
    }
    self.copies.carries_bar()
  }

  /// Stops drawing the tab bar (see
  /// [`carry_tab_bar`](Self::carry_tab_bar)).
  pub fn drop_tab_bar(&mut self) {
    self.copies.drop_bar();
  }

  /// Covers the part of the window's frame its content does not reach in
  /// `color` from now on, while the peek draws it filled (see
  /// [`place_filled`](Self::place_filled)) and draws the window at all.
  /// The same `color` again keeps what is drawn. Returns whether it does.
  pub fn carry_fill(&mut self, color: Color) -> bool {
    let carries = self.copies.carry_fill(self.surrogate.hwnd(), color);
    self.copies.show_fill(self.drawn == Drawn::Shown);
    carries
  }

  /// Draws the window too faint to see until
  /// [`take_over`](Self::take_over), under its real window that still
  /// shows, while DWM warms the thumbnail up; a hosted one's backdrop copy
  /// too (see [`warms_backdrop`](Self::warms_backdrop)).
  pub fn warm_up(&mut self) {
    self.draw(Drawn::WarmingUp);
  }

  /// Draws the window and its ring and backdrop copies too faint to see,
  /// placed as usual, while a workspace switch's surrogate draws them,
  /// until [`end_stand_by`](Self::end_stand_by). DWM keeps them warm
  /// meanwhile.
  pub fn stand_by(&mut self) {
    self.stands_by = true;
    self.draw(Drawn::WarmingUp);
    self.copies.show_backdrop(true);
    self.copies.set_opacity(WARM_UP_OPACITY);
  }

  /// Draws the window and its copies again after
  /// [`stand_by`](Self::stand_by), the window first. The backdrop copy
  /// stays shown for its window's style to decide (see
  /// [`show_backdrop`](Self::show_backdrop)). No-op unless standing by.
  pub fn end_stand_by(&mut self) {
    if !self.stands_by {
      return;
    }

    self.stands_by = false;
    self.drawn = Drawn::Shown;
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.cover_ring();
    self.copies.set_backdrop_opacity(u8::MAX);
  }

  /// Takes over showing the window from `window`, which is about to be
  /// cloaked: the thumbnail goes to the window's opacity (see
  /// [`Drawn::Shown`]) and a semi-transparent `window` to no alpha at all,
  /// in that order, so the swap lands in one frame and the cloak unseen
  /// (see `HiddenAlpha`). [`restore_alpha`](Self::restore_alpha) gives
  /// `window` its alpha back once the cloak is on screen.
  pub fn take_over(&mut self, window: &NativeWindow) {
    let draw_scope = perf::scope(Stage::TakeOverDraw);
    self.draw(Drawn::Shown);
    drop(draw_scope);
    let _scope = perf::scope(Stage::TakeOverAlpha);
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
    self.copies.drop_spares();
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
    let warmed_backdrop = self.warms_backdrop();
    self.drawn = drawn;
    if drawn == Drawn::HandedBack {
      self.apply_backdrop();
      self.surrogate.set_window_opacity(self.window_opacity());
    } else {
      self.surrogate.set_window_opacity(self.window_opacity());
      if warmed_backdrop && !self.warms_backdrop() {
        self
          .copies
          .set_backdrop_opacity(scale_alpha(u8::MAX, self.frac));
      }
      self.apply_backdrop();
    }
  }

  /// Whether the backdrop copy is drawn too faint to see while the window
  /// warms up: a hosted one's, whose surface may be above the real window
  /// still shown, which the opaque copy would hide.
  fn warms_backdrop(&self) -> bool {
    self.drawn == Drawn::WarmingUp && self.is_hosted()
  }

  /// The opacity the backdrop copy is drawn at, `opacity` unless it warms
  /// up (see [`warms_backdrop`](Self::warms_backdrop)).
  fn backdrop_opacity(&self, opacity: u8) -> u8 {
    if self.warms_backdrop() {
      WARM_UP_OPACITY
    } else {
      opacity
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
  /// `clip`, or hides the peek when none does.
  pub fn place(&mut self, rect: &Rect, clip: &Rect, opacity: u8) {
    let logical = to_logical(rect, &self.border_inset);
    self.opacity = opacity;
    self.frac = 1.0;

    let origin = self.origin(clip);
    self.visible_rect = self.show_pinned(rect, &logical, clip, origin);
    self.frame_rect = Some(logical);
  }

  /// Shows the window, at rest at `rest`, drawn at `frame` (of any size)
  /// cut at `clip`, at `frac` of its `opacity`, its ring and backdrop
  /// copies placed at `frame` and faded with it. The thumbnail samples the
  /// window at its size at rest.
  pub fn place_scaled(
    &mut self,
    rest: &Rect,
    frame: &Rect,
    clip: &Rect,
    opacity: u8,
    frac: f32,
  ) {
    let origin = self.origin(clip);
    self.opacity = opacity;
    self.frac = frac.clamp(0.0, 1.0);

    let ScaledRects { drawn, shown } =
      scaled_rects(rest, frame, clip, self.border_inset, origin);
    self.frame_rect = Some(drawn);

    let Some((visible, (rc_src, rc_dst))) = shown else {
      self.set_shown(false);
      self.visible_rect = None;
      return;
    };

    self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place(frame, clip, origin);
    let opacity = scale_alpha(u8::MAX, self.frac);
    self.copies.set_ring_opacity(opacity);
    self
      .copies
      .set_backdrop_opacity(self.backdrop_opacity(opacity));
    self.set_shown(true);
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
  /// Returns the cover drawn with.
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
  ) -> (f32, f32) {
    let origin = self.origin(clip);
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
      self.set_shown(false);
      self.visible_rect = None;
      return cover;
    };

    self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place(rect, clip, origin);
    self.copies.cover_ring();
    self
      .copies
      .set_backdrop_opacity(self.backdrop_opacity(u8::MAX));
    self.set_shown(true);
    self.visible_rect = Some(visible);
    cover
  }

  /// Shows the window at `rect` 1:1 from its frame's top-left, sampling
  /// `sampled` of it (invisible borders included, see [`filled_rects`]),
  /// cut at its frame and at `clip`, with the part of its frame the
  /// content does not reach covered by the fill (see
  /// [`carry_fill`](Self::carry_fill)), and its ring and backdrop copies
  /// around `rect`.
  pub fn place_filled(
    &mut self,
    sampled: (i32, i32),
    rect: &Rect,
    clip: &Rect,
    opacity: u8,
  ) {
    let origin = self.origin(clip);
    self.opacity = opacity;
    self.frac = 1.0;

    let FilledRects {
      drawn,
      covered,
      content,
    } = filled_rects(sampled, rect, clip, self.border_inset, origin);
    self.frame_rect = Some(drawn.clone());
    // The fill is flat, over the backdrop copy: logged to tell it from a
    // copy drawing nothing where a resize shows no blur.
    if covered != (drawn.width(), drawn.height()) {
      tracing::debug!(
        "Filled {covered:?} of a {}x{} frame.",
        drawn.width(),
        drawn.height()
      );
    }

    let Some(visible) = drawn.intersection(clip) else {
      self.set_shown(false);
      self.visible_rect = None;
      return;
    };

    // In this order, so a composition landing mid-run shows an edge a
    // frame late rather than a hole: backdrop, fill, content, ring.
    self.copies.place_backdrop(rect, Some(&drawn), clip, origin);
    self
      .copies
      .set_backdrop_opacity(self.backdrop_opacity(u8::MAX));
    self.copies.place_fill(&drawn, Some(covered), clip, origin);
    self.copies.set_fill_opacity(u8::MAX);
    match content {
      Some((rc_src, rc_dst)) => {
        self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
      }
      None => self.surrogate.hide_thumbnail(),
    }
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place_ring();
    self.copies.cover_ring();
    self.set_shown(true);
    self.visible_rect = Some(visible);
  }

  /// The top-left the peek draws its window cut at `clip` from: its
  /// viewport's, moved to span `clip` if it doesn't.
  fn origin(&mut self, clip: &Rect) -> (i32, i32) {
    repin(&mut self.surrogate, &mut self.viewport, clip);
    (self.viewport.left, self.viewport.top)
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
      self.set_shown(false);
      return None;
    };

    self.surrogate.set_thumbnail_rects(rc_src, rc_dst);
    self.surrogate.set_window_opacity(self.window_opacity());
    self.copies.place(rect, clip, origin);
    if self.stands_by {
      self.copies.set_opacity(WARM_UP_OPACITY);
    } else {
      self.copies.cover_ring();
      self
        .copies
        .set_backdrop_opacity(self.backdrop_opacity(u8::MAX));
    }
    self.set_shown(true);

    Some(visible)
  }

  /// Hides the peek until it is placed again, e.g. while its window is
  /// drawn at no size.
  pub fn hide(&mut self) {
    self.set_shown(false);
    self.visible_rect = None;
  }

  /// Shows or hides the peek's window, or a hosted one's thumbnails, its
  /// window's and every copy's: its surface, shared, stays as its other
  /// peeks have it. Hidden, the backdrop copy goes first, and shown, it
  /// comes last, as [`draw`](Self::draw) has them.
  fn set_shown(&mut self, shown: bool) {
    if !self.surrogate.is_hosted() {
      self.surrogate.set_visible(shown);
    } else if shown {
      self.surrogate.set_visible(true);
      self.copies.set_held(false);
    } else {
      self.copies.set_held(true);
      self.surrogate.set_visible(false);
    }
  }

  /// The window's visible frame where it was last placed, including any
  /// part outside the clip, for overlays that frame it.
  #[must_use]
  pub fn frame_rect(&self) -> Option<&Rect> {
    self.frame_rect.as_ref()
  }

  /// Handle of the peek's own window, or a hosted one's surface's, which
  /// overlays of the window it shows sit behind.
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
  ///
  /// A hosted one stays where its surface is, above every tile it draws.
  pub fn begin_hand_back(&mut self) {
    self.adopt_companion();
    self.surrogate.begin_hand_back();

    if let Err(err) = self.surrogate.raise_above(self.source) {
      tracing::warn!("Failed to raise a peek above its window: {err}");
    }
  }

  /// Puts a hosted peek's surface just above its window unless it is above
  /// it already, as [`begin_hand_back`](Self::begin_hand_back) puts a peek
  /// of its own, for a surface not raised over every tile it draws (a
  /// scrolling layout's). None for one of its own.
  pub fn stay_over_window(&self) {
    self.surrogate.lift_surface_above(self.source);
  }

  /// Puts the peek at the top of the non-topmost windows, over windows
  /// that came above its real window while that was minimized. Its ring
  /// and backdrop copies are drawn in it, so they go with it. A hosted one
  /// raises its surface, with every peek on it.
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

  /// Fraction of their opacity the window and its copies were last drawn
  /// at: below 1 only while [`place_scaled`](Self::place_scaled) fades it.
  #[must_use]
  pub fn opacity_fraction(&self) -> f32 {
    self.frac
  }
}

/// The backdrop styles of `styles` a peek draws, each copied.
fn copyable(
  styles: &[Option<BackdropOverlayParams>],
) -> Vec<BackdropOverlayParams> {
  styles.iter().flatten().copied().collect()
}

/// Moves a pinned peek's `surrogate`, spanning `viewport`, to span `clip`
/// instead, if it doesn't. A hosted one stays on its surface, which spans
/// its viewport, and is cut at `clip` as drawn.
fn repin(
  surrogate: &mut NativeSurrogate,
  viewport: &mut Rect,
  clip: &Rect,
) {
  if viewport != clip && !surrogate.is_hosted() {
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
  let drawn = scaled_frame(rest, frame, border_inset);
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

/// The visible frame of a window resting at `rest`, drawn at `frame`: its
/// frame at rest, past the invisible resize borders `border_inset`, scaled
/// as `rest` is to `frame`.
fn scaled_frame(rest: &Rect, frame: &Rect, border_inset: RECT) -> Rect {
  map_rect(&to_logical(rest, &border_inset), rest, frame)
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

/// Where a pinned peek draws a window filled (see [`filled_rects`]).
#[derive(Debug, PartialEq, Eq)]
struct FilledRects {
  /// The window's visible frame, including any part outside the clip.
  drawn: Rect,
  /// How much of the frame, from its top-left, the content covers.
  covered: (i32, i32),
  /// The thumbnail's `(rcSource, rcDestination)` drawing the content,
  /// unless none of it is inside the clip.
  content: Option<(RECT, RECT)>,
}

/// Where a window at `rect`, of which `sampled` is sure to be drawn, is
/// drawn 1:1 from its frame's top-left, cut at the frame and at `clip`,
/// into a peek pinned with its top-left at `origin`.
///
/// `sampled` is the window's size, invisible borders included, it is sure
/// to have when the frame is composited: the smallest of its size as last
/// read, `rect`'s, and any it was sent and is yet to report. Sampling more
/// than the window has draws a transparent hole.
fn filled_rects(
  sampled: (i32, i32),
  rect: &Rect,
  clip: &Rect,
  border_inset: RECT,
  origin: (i32, i32),
) -> FilledRects {
  let drawn = to_logical(rect, &border_inset);
  let covered = (
    (sampled.0 - border_inset.left - border_inset.right)
      .clamp(0, drawn.width()),
    (sampled.1 - border_inset.top - border_inset.bottom)
      .clamp(0, drawn.height()),
  );
  let part = Rect::from_xy(drawn.x(), drawn.y(), covered.0, covered.1);
  let content = part.intersection(clip).and_then(|visible| {
    placed_rects(&part, &visible, covered, border_inset, origin)
  });

  FilledRects {
    drawn,
    covered,
    content,
  }
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

  /// `(left, top, right, bottom)` of a `RECT`.
  type Edges = (i32, i32, i32, i32);

  fn ltrb(rect: RECT) -> Edges {
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

  /// `(covered, rcSource, rcDestination)` of a window at
  /// (93, 100, 814x607), its frame (100, 100, 800x600), sampling
  /// `sampled`, cut at `clip`.
  fn filled(
    sampled: (i32, i32),
    clip: &Rect,
  ) -> ((i32, i32), Option<(Edges, Edges)>) {
    let rect = Rect::from_ltrb(93, 100, 907, 707);
    let FilledRects {
      drawn,
      covered,
      content,
    } = filled_rects(sampled, &rect, clip, INSET, (0, 0));
    assert_eq!(drawn, Rect::from_ltrb(100, 100, 900, 700));

    (covered, content.map(|(src, dst)| (ltrb(src), ltrb(dst))))
  }

  /// At its size, a window covers its frame 1:1.
  #[test]
  fn filled_at_its_size_covers_its_frame() {
    assert_eq!(
      filled((814, 607), &CLIP),
      ((800, 600), Some(((7, 0, 807, 600), (100, 100, 900, 700))))
    );
  }

  /// Short of its frame, a window is drawn 1:1 from the frame's top-left,
  /// the rest left to the fill.
  #[test]
  fn filled_short_of_its_frame_leaves_the_rest_to_the_fill() {
    assert_eq!(
      filled((614, 507), &CLIP),
      ((600, 500), Some(((7, 0, 607, 500), (100, 100, 700, 600))))
    );
  }

  /// Larger than its frame, a window is cut at it, never sampled past it.
  #[test]
  fn filled_larger_than_its_frame_is_cut_at_it() {
    assert_eq!(
      filled((1214, 907), &CLIP),
      ((800, 600), Some(((7, 0, 807, 600), (100, 100, 900, 700))))
    );
  }

  /// Cut at the clip, only the content inside it is drawn; content wholly
  /// past it is not.
  #[test]
  fn filled_is_cut_at_the_clip() {
    let clip = Rect::from_ltrb(0, 0, 400, 1040);
    assert_eq!(
      filled((614, 607), &clip),
      ((600, 600), Some(((7, 0, 307, 600), (100, 100, 400, 700))))
    );

    let clip = Rect::from_ltrb(800, 0, 1920, 1040);
    assert_eq!(filled((614, 607), &clip), ((600, 600), None));
  }
}
