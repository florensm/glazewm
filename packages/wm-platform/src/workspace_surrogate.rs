use windows::Win32::Foundation::{HWND, RECT};

use crate::{
  dwm_thumbnail::Thumbnail,
  native_surrogate::{placed_rects, to_logical},
  platform_impl,
  resize_session::compute_border_inset,
  CornerStyle, NativeSurrogate, NativeWindow, OpacityValue, Rect,
  SurrogateBatch,
};

/// Thumbnail opacity a surrogate is warmed up at under its still-shown
/// real window: rendered by DWM, but too faint to see.
const WARM_UP_OPACITY: u8 = 1;

/// A window's border ring, drawn into its [`WorkspaceSurrogate`] as a
/// thumbnail of the ring's own overlay window, parked where the window
/// rests (see `NativeBorderOverlay::park`).
///
/// The ring then moves in the same DWM thumbnail updates as the window,
/// rather than through the overlay's own window or visual tree, which
/// reach DWM on schedules of their own.
struct RingCopy {
  thumbnail: Thumbnail,
  /// The overlay window the thumbnail is of.
  source: isize,
  /// Where `source` is: the ring's rect while its window rests.
  rest: Rect,
}

/// `alpha` scaled by `frac` (0.0–1.0).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn scale_alpha(alpha: u8, frac: f32) -> u8 {
  (f32::from(alpha) * frac.clamp(0.0, 1.0)).round() as u8
}

/// Where `item`, placed relative to `rest`, is once `rest` has moved to
/// `frame`, scaled `scale` times.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn map_rect(item: &Rect, rest: &Rect, frame: &Rect, scale: f32) -> Rect {
  let map = |value: i32, from: i32, to: i32| {
    to + ((value - from) as f32 * scale).round() as i32
  };

  Rect::from_ltrb(
    map(item.left, rest.left, frame.left),
    map(item.top, rest.top, frame.top),
    map(item.right, rest.left, frame.left),
    map(item.bottom, rest.top, frame.top),
  )
}

/// `(rcSource, rcDestination)` drawing a ring that rests at `ring_rest`,
/// around a window resting at `window_rest`, with that window at `frame`,
/// `scale` times its size at rest, cut at `panel` and `monitor`, into a
/// surrogate whose top-left is at `origin`. `None` when nothing is left.
fn ring_rects(
  ring_rest: &Rect,
  window_rest: &Rect,
  frame: &Rect,
  scale: f32,
  panel: &Rect,
  monitor: &Rect,
  origin: (i32, i32),
) -> Option<(RECT, RECT)> {
  let ring_frame = map_rect(ring_rest, window_rest, frame, scale);
  let visible = ring_frame.intersection(panel)?.intersection(monitor)?;

  placed_rects(
    &ring_frame,
    &visible,
    (ring_rest.width(), ring_rest.height()),
    RECT::default(),
    origin,
  )
}

/// Positioning strategy for a [`WorkspaceSurrogate`].
enum SurrogateMode {
  /// The surrogate spans the whole monitor viewport, created once and
  /// never repositioned; all motion is expressed via
  /// `rcSource`/`rcDestination` clipping, avoiding any per-frame
  /// `SetWindowPos` call. Used for windows with no backdrop configured
  /// (the common case).
  PinnedViewport,
  /// The surrogate is sized to the window's own (clipped-to-monitor)
  /// visible rect and genuinely moved/resized every frame via a batched
  /// `SetWindowPos`. Used when the window has a `backdrop` configured, so
  /// the backdrop overlay can be repositioned onto this surrogate's own
  /// footprint each frame — pinning the surrogate to the full viewport
  /// would leave the overlay nothing meaningful to follow.
  ///
  /// The thumbnail is cut only at the panel, which slides with the
  /// window, and the surrogate's bounds cut it at the monitor.
  /// Thumbnail updates apply at once while the batched moves land
  /// later, often a composited frame later; cut at the monitor too, a
  /// window sliding across the monitor's right or bottom edge changed
  /// both every frame, and frames pairing the new thumbnail with the
  /// old bounds flashed a strip at that edge with no window in it.
  Live,
}

/// Where a [`SurrogateMode::Live`] surrogate draws its window for one
/// animation frame, in the coordinates of [`WorkspaceSurrogate::rect`].
#[derive(Clone, Debug)]
pub struct LiveFrame {
  /// The window, before any clip.
  pub frame: Rect,
  /// The part of `frame` inside the sliding panel.
  pub shown: Rect,
  /// The part of `shown` on the monitor: the surrogate's own rect.
  pub visible: Rect,
}

/// Surrogate overlay for a single window participating in a
/// workspace-switch animation.
///
/// Both outgoing and incoming windows move together so the whole workspace
/// slides as a single panel. For windows with no live backdrop
/// ([`SurrogateMode::PinnedViewport`]), the surrogate is created at the
/// full monitor rect (`viewport`) and never repositioned; all per-frame
/// animation is expressed via `rcSource`/`rcDestination` in
/// `DwmUpdateThumbnailProperties`, avoiding any per-frame `SetWindowPos`
/// calls. Windows with a live backdrop ([`SurrogateMode::Live`]) are
/// instead moved/resized every frame to their current visible rect,
/// batched via [`SurrogateBatch`]. The surrogate is hidden (via `SW_HIDE`)
/// when the visible area is empty.
pub struct WorkspaceSurrogate {
  inner: NativeSurrogate,
  /// Final screen rect of the window (target position for incoming,
  /// current screen rect for outgoing).
  pub rect: Rect,
  /// Monitor rect used as the surrogate window's fixed position and size
  /// in [`SurrogateMode::PinnedViewport`] mode.
  ///
  /// In that mode the surrogate is created at `viewport` and stays there
  /// for the entire animation; `rcDestination` coordinates are
  /// expressed relative to `viewport`'s top-left corner in every
  /// per-frame thumbnail update. In [`SurrogateMode::Live`] mode this
  /// is unused for positioning — the surrogate is moved/resized
  /// directly to its absolute screen rect — but per-frame visible-strip
  /// clipping still uses the monitor bounds passed to each `update_*`
  /// call, so a sliding window never spills onto an adjacent monitor.
  viewport: Rect,
  /// DWM thumbnail opacity (0–255) derived from the window-effects
  /// config.
  opacity: u8,
  /// The non-full-opacity end of the opacity animation, as a fraction of
  /// `opacity` (0.0–1.0).
  ///
  /// For outgoing windows this is the final opacity fraction (start =
  /// 1.0, end = `opacity_endpoint`). For incoming windows it is the
  /// initial opacity fraction (start = `opacity_endpoint`, end = 1.0).
  /// At `1.0` (default) the opacity is constant throughout the
  /// animation; at `0.0` the window fully fades out or in.
  opacity_endpoint: f32,
  /// Positioning strategy, chosen once at creation based on whether the
  /// window has a backdrop configured.
  mode: SurrogateMode,
  /// Invisible border insets of the source window, in physical pixels.
  /// Zero in [`SurrogateMode::PinnedViewport`] mode. Used to offset every
  /// per-frame `rcSource` sample past the invisible border, matching the
  /// deflation already baked into `rect` and the surrogate's created
  /// size.
  border_inset: RECT,
  /// Live footprint for the current animation frame, in
  /// [`SurrogateMode::Live`] mode only -- lets a caller (the backdrop
  /// overlay tracker in `AnimationManager::update_internal`) follow the
  /// surrogate's actual footprint instead of only its final `rect`.
  /// `None` in [`SurrogateMode::PinnedViewport`] mode (no single
  /// "screen rect" applies there), and `None` whenever the visible
  /// strip is currently empty (fully off-screen, or zoomed to nothing).
  live_frame: Option<LiveFrame>,

  /// Full on-screen rect the source window occupies this animation frame,
  /// *before* the monitor clip -- in both [`SurrogateMode`]s, unlike
  /// `live_frame`. Lets the border overlay follow the window through a
  /// slide: the ring hangs outside the window's own rect, so it has to be
  /// placed from the unclipped position and clipped separately (see
  /// `NativeBorderOverlay::pin_or_slide`). `None` once the window is
  /// entirely off-screen.
  unclipped_rect: Option<Rect>,

  /// The monitor this window rests on, moved and scaled along with the
  /// workspace this frame. Only what the window shows at rest slides with
  /// it, so e.g. the columns a scrolling layout parks past the monitor
  /// edge stay hidden. `None` once the window is entirely off-screen.
  panel: Option<Rect>,

  /// Inset carrying `rect` to the window's *frame* rect, derived at
  /// construction from the `frame_rect` the caller passed alongside it.
  ///
  /// It cannot be inferred from the mode: callers disagree on what `rect`
  /// means. The incoming branch applies `total_border_delta` and so
  /// passes a physical rect including the invisible resize border,
  /// while the outgoing branch passes a frame rect that already
  /// excludes it.
  ///
  /// Only `unclipped_rect` uses it. The border overlay is placed from the
  /// frame rect in steady state, so a ring built from a physical rect
  /// bulges into the configured gap on every side and collides with its
  /// neighbour's mid-slide.
  frame_inset: RECT,

  /// The real window's own alpha while it is hidden by alpha for a
  /// handover with the surrogate (see [`take_over`]), with its handle.
  ///
  /// [`take_over`]: WorkspaceSurrogate::take_over
  hidden_alpha: Option<(isize, u8)>,

  /// Copy of the window's border ring. See [`carry_ring`].
  ///
  /// [`carry_ring`]: WorkspaceSurrogate::carry_ring
  ring: Option<RingCopy>,
}

impl WorkspaceSurrogate {
  /// Creates a hidden surrogate for a workspace-switch animation.
  ///
  /// `viewport` is the monitor rect; `rect` is the source window's screen
  /// rect, used as the thumbnail registration dimensions and the reference
  /// for per-frame coordinate math.
  ///
  /// `frame_rect` is the same window's frame rect (what
  /// `DWMWA_EXTENDED_FRAME_BOUNDS` reports), which callers pass explicitly
  /// because they disagree on whether `rect` includes the invisible resize
  /// border. It is used only to place overlays that sit outside the
  /// window, i.e. the border ring -- pass the same value as `rect` when
  /// the two coincide.
  ///
  /// `opacity_endpoint` controls how far the opacity animates away from
  /// the effect opacity. For outgoing windows pass
  /// `config.opacity_outgoing`; for incoming windows pass
  /// `config.opacity_incoming`. At `1.0` the opacity is held constant;
  /// at `0.0` the window fully fades.
  ///
  /// `corner_style` controls the DWM corner-rounding applied to the
  /// surrogate when it carries a live backdrop
  /// ([`SurrogateMode::Live`]), matching the real window's configured
  /// style. Ignored in [`SurrogateMode::PinnedViewport`] mode, where the
  /// surrogate spans the whole monitor and must not be rounded.
  ///
  /// `overlay_tint` is the WM's own configured tint
  /// (`BackdropEffectConfig::overlay_tint`), or `None` when `backdrop`
  /// isn't configured for this window. When `Some`, this switches the
  /// surrogate to [`SurrogateMode::Live`] -- sized/moved to its own
  /// footprint rather than pinned to the viewport -- so a live-tracking
  /// backdrop overlay (see [`live_frame`], driven from
  /// `AnimationManager`) has a meaningful per-frame rect to follow. The
  /// backdrop is the same overlay used in steady state, kept alive and
  /// repositioned to this surrogate's footprint for the whole slide, so
  /// there is no handoff flash between two backdrops.
  ///
  /// [`live_frame`]: WorkspaceSurrogate::live_frame
  ///
  /// The surrogate is created hidden. For outgoing windows, call
  /// [`show_initial`] before cloaking the real window to avoid a blank
  /// frame. For incoming windows in stationary styles (fade/zoom), call
  /// [`show_incoming`] before the animation loop to pre-warm the DWM
  /// thumbnail.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  /// [`show_incoming`]: WorkspaceSurrogate::show_incoming
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    hwnd: HWND,
    rect: &Rect,
    frame_rect: &Rect,
    viewport: &Rect,
    opacity: u8,
    opacity_endpoint: f32,
    corner_style: &CornerStyle,
    overlay_tint: Option<u32>,
  ) -> crate::Result<Self> {
    let carries_live_backdrop = overlay_tint.is_some();

    let mode = if carries_live_backdrop {
      SurrogateMode::Live
    } else {
      SurrogateMode::PinnedViewport
    };

    // Live mode: surrogate is sized to the window's own rect and
    // moved/resized every frame to track it, so it should match the
    // real window's rounding. PinnedViewport mode: surrogate spans the
    // full viewport and must not be rounded.
    let (source_rect, thumbnail_rect, effective_corner_style): (
      &Rect,
      &Rect,
      &CornerStyle,
    ) = if carries_live_backdrop {
      (rect, rect, corner_style)
    } else {
      (viewport, rect, &CornerStyle::Square)
    };

    // Live mode moves/resizes the surrogate to the window's own logical
    // rect every frame, so — like resize sessions — it must be
    // deflated by the window's invisible resize border to avoid
    // overshooting into the configured gap. PinnedViewport mode sizes
    // `source_rect` to the full monitor viewport, which has no
    // invisible border of its own; deflating it here would incorrectly
    // shrink the pinned surrogate.
    let border_inset = if carries_live_backdrop {
      compute_border_inset(hwnd)
    } else {
      RECT::default()
    };

    let inner = NativeSurrogate::create(
      hwnd,
      source_rect,
      thumbnail_rect,
      opacity,
      border_inset,
      effective_corner_style,
      // Workspace surrogates should sit just below the source window;
      // they don't compete with resize surrogates since
      // workspace-switch and resize animations are mutually
      // exclusive.
      hwnd,
      false,
    )?;

    // Store the deflated (logical) rect so all per-frame positioning math
    // below operates in the same coordinate space the surrogate window was
    // actually created/sized in. A no-op when `border_inset` is zero
    // (`PinnedViewport` mode).
    let stored_rect = to_logical(rect, &border_inset);

    // Derived before `stored_rect` is moved into the struct below.
    let frame_inset = RECT {
      left: frame_rect.left - stored_rect.left,
      top: frame_rect.top - stored_rect.top,
      right: stored_rect.right - frame_rect.right,
      bottom: stored_rect.bottom - frame_rect.bottom,
    };

    Ok(Self {
      inner,
      rect: stored_rect,
      viewport: viewport.clone(),
      opacity,
      opacity_endpoint: opacity_endpoint.clamp(0.0, 1.0),
      mode,
      border_inset,
      live_frame: None,
      unclipped_rect: None,
      panel: None,
      frame_inset,
      hidden_alpha: None,
      ring: None,
    })
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
  ///
  /// The ring lies outside the window, which a [`SurrogateMode::Live`]
  /// surrogate is cut to, so only one pinned to the viewport can carry it.
  /// Returns whether this one does.
  pub fn carry_ring(&mut self, source: HWND, rest: &Rect) -> bool {
    if self.is_live() {
      return false;
    }

    if let Some(ring) =
      self.ring.as_mut().filter(|ring| ring.source == source.0)
    {
      ring.rest = rest.clone();
      return true;
    }

    self.ring =
      Thumbnail::register(self.hwnd(), source).map(|thumbnail| RingCopy {
        thumbnail,
        source: source.0,
        rest: rest.clone(),
      });
    self.ring.is_some()
  }

  /// Whether the surrogate draws the window's border ring (see
  /// [`carry_ring`](Self::carry_ring)).
  #[must_use]
  pub fn carries_ring(&self) -> bool {
    self.ring.is_some()
  }

  /// Brings the ring copy to full opacity over the still-shown ring it
  /// duplicates, so the real one can be concealed once this has been
  /// composited, without a frame showing neither.
  ///
  /// Unlike the window, it needs no warm-up: drawn twice in one place, a
  /// ring looks like one.
  pub fn cover_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      let _ = ring.thumbnail.set_opacity(u8::MAX);
    }
  }

  /// Hides the ring, e.g. as its window leaves the screen for good.
  pub fn hide_ring(&mut self) {
    if let Some(ring) = &mut self.ring {
      ring.thumbnail.hide();
    }
  }

  /// Draws the ring around the window at `frame`, `scale` times its size
  /// at rest, cut at `panel` and `monitor` like the window is.
  fn place_ring(
    &mut self,
    frame: &Rect,
    scale: f32,
    panel: &Rect,
    monitor: &Rect,
  ) {
    let Some(ring) = &mut self.ring else {
      return;
    };

    match ring_rects(
      &ring.rest,
      &self.rect,
      frame,
      scale,
      panel,
      monitor,
      (self.viewport.left, self.viewport.top),
    ) {
      Some((rc_src, rc_dst)) => {
        let _ = ring.thumbnail.show(rc_src, rc_dst);
      }
      None => ring.thumbnail.hide(),
    }
  }

  /// Draws the window at `window` opacity and its ring at `ring`.
  fn set_opacities(&mut self, window: u8, ring: u8) {
    self.inner.set_window_opacity(window);
    if let Some(copy) = &mut self.ring {
      let _ = copy.thumbnail.set_opacity(ring);
    }
  }

  /// Draws the window and its ring at `frac` of their own opacities.
  fn set_opacity_frac(&mut self, frac: f32) {
    self.set_opacities(
      scale_alpha(self.opacity, frac),
      scale_alpha(u8::MAX, frac),
    );
  }

  /// Live footprint for the current animation frame, when in
  /// [`SurrogateMode::Live`] mode and currently visible.
  #[must_use]
  pub fn live_frame(&self) -> Option<&LiveFrame> {
    self.live_frame.as_ref()
  }

  /// Full on-screen rect of the source window for the current animation
  /// frame, before the monitor clip, in both [`SurrogateMode`]s. `None`
  /// once the window is entirely off-screen.
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
    self.inner.set_visible(false);
    self.hide_ring();
    self.live_frame = None;
    self.unclipped_rect = None;
    self.panel = None;
  }

  /// Draws the part of the window inside both `panel` and `monitor`, with
  /// the window at `frame` this frame, `scale` times its size at rest.
  ///
  /// In [`SurrogateMode::PinnedViewport`] mode only the thumbnail
  /// changes; in [`SurrogateMode::Live`] mode the surrogate is also moved
  /// to the visible rect, immediately without `batch` (initial placement,
  /// before the animation loop starts) and queued into it otherwise.
  ///
  /// Returns whether anything is left to draw.
  fn show_frame(
    &mut self,
    batch: Option<&mut SurrogateBatch>,
    frame: &Rect,
    scale: f32,
    panel: Rect,
    monitor: &Rect,
  ) -> bool {
    let Some((shown, visible)) =
      frame.intersection(&panel).and_then(|shown| {
        let visible = shown.intersection(monitor)?;
        Some((shown, visible))
      })
    else {
      self.clear_frame();
      return false;
    };

    // Pinned, the surrogate never moves, so cutting the thumbnail at the
    // monitor as well leaves it the one thing that changes. Live, the
    // surrogate's bounds cut it there instead (see `SurrogateMode::Live`).
    let (sampled, origin) = match self.mode {
      SurrogateMode::PinnedViewport => {
        (&visible, (self.viewport.left, self.viewport.top))
      }
      SurrogateMode::Live => (&shown, (visible.left, visible.top)),
    };
    let Some((rc_src, rc_dst)) = placed_rects(
      frame,
      sampled,
      (self.rect.width(), self.rect.height()),
      self.border_inset,
      origin,
    ) else {
      self.clear_frame();
      return false;
    };
    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    self.place_ring(frame, scale, &panel, monitor);

    if self.is_live() {
      match batch {
        Some(batch) => self.inner.defer_reposition(batch, &visible),
        None => {
          let _ = self.inner.reposition(&visible);
        }
      }
      self.live_frame = Some(LiveFrame {
        frame: frame.clone(),
        shown,
        visible,
      });
    }
    self.inner.set_visible(true);
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

  /// Whether this surrogate is tracked by a backdrop overlay
  /// ([`SurrogateMode::Live`]), as opposed to
  /// [`SurrogateMode::PinnedViewport`].
  #[must_use]
  pub fn is_live(&self) -> bool {
    matches!(self.mode, SurrogateMode::Live)
  }

  /// `HWND` of this surrogate.
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
  /// fully opaque.
  ///
  /// The ring copy stays: the ring is revealed under it only after this,
  /// and it goes with the surrogate.
  ///
  /// [`prepare_hand_back`]: WorkspaceSurrogate::prepare_hand_back
  pub fn hide_thumbnail(&mut self) {
    self.restore_alpha();
    self.inner.hide_thumbnail();
  }

  /// Shows the surrogate with the thumbnail at the window's natural
  /// (unscaled) position within the monitor viewport, cut to it like
  /// every later frame, but too faint to see, so the real window still
  /// shows as it is while DWM warms the thumbnail up. [`take_over`] then
  /// swaps the two.
  ///
  /// [`take_over`]: WorkspaceSurrogate::take_over
  pub fn show_initial(&mut self) {
    self.show_at_rest();
    self.set_opacities(WARM_UP_OPACITY, WARM_UP_OPACITY);
  }

  /// Takes over showing the window from `window`, which is about to be
  /// cloaked: the thumbnail goes to its effect opacity and a
  /// semi-transparent `window` to no alpha at all, in that order.
  ///
  /// The shell's cloak is a cross-process call of several milliseconds,
  /// which DWM can show anywhere within it: with `window` still showing,
  /// that left frames with both up (two layers of a semi-transparent
  /// window, which read as solid) or neither. Its alpha DWM applies at
  /// once, so the swap lands in one frame and the cloak lands unseen. An
  /// opaque `window` covers the thumbnail anyway. `restore_alpha` gives
  /// `window` its alpha back once the cloak is on screen.
  pub fn take_over(&mut self, window: &NativeWindow) {
    self.apply_effect_opacity();
    self.hide_by_alpha(&window.inner);
  }

  /// Readies handing the window back to `window`, which is about to be
  /// uncloaked: a semi-transparent `window` goes to no alpha, so its
  /// uncloak lands unseen and `hide_thumbnail` swaps the two in one frame
  /// (see [`take_over`]).
  ///
  /// [`take_over`]: WorkspaceSurrogate::take_over
  pub fn prepare_hand_back(&mut self, window: &NativeWindow) {
    self.hide_by_alpha(&window.inner);
  }

  /// Gives the real window back the alpha a handover set aside, unless it
  /// was set again since, e.g. by a focus change's effects. No-op
  /// otherwise.
  pub fn restore_alpha(&mut self) {
    let Some((handle, alpha)) = self.hidden_alpha.take() else {
      return;
    };

    let window = platform_impl::NativeWindow::new(handle);
    if window.layered_alpha().ok().flatten() == Some(0) {
      if let Err(err) =
        window.set_transparency(&OpacityValue::from_alpha(alpha))
      {
        tracing::warn!("Failed to restore window alpha: {err}");
      }
    }
  }

  /// Hides `window` by its alpha, setting the alpha aside, when it is
  /// semi-transparent.
  fn hide_by_alpha(&mut self, window: &platform_impl::NativeWindow) {
    // A handover still pending would otherwise set aside the 0.
    self.restore_alpha();

    let Some(alpha) = window
      .layered_alpha()
      .ok()
      .flatten()
      .filter(|alpha| *alpha < u8::MAX)
    else {
      return;
    };

    match window.set_transparency(&OpacityValue::from_alpha(0)) {
      Ok(()) => self.hidden_alpha = Some((window.hwnd().0, alpha)),
      Err(err) => tracing::warn!("Failed to hide window by alpha: {err}"),
    }
  }

  /// Draws the window where it rests, as the first frame of the switch.
  fn show_at_rest(&mut self) {
    let rect = self.rect.clone();
    let viewport = self.viewport.clone();
    self.show_frame(None, &rect, 1.0, viewport.clone(), &viewport);
  }

  /// Updates the DWM thumbnail opacity to the configured `opacity`, and
  /// the ring copy's to full, without changing the surrogate window
  /// position or size.
  pub fn apply_effect_opacity(&mut self) {
    self.set_opacity_frac(1.0);
  }

  /// Retargets the effect opacity, for a window whose transparency
  /// transitions with its focus state during the switch. Takes effect
  /// with the next per-frame update.
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
  /// effect_opacity`. At `opacity_endpoint = 0.0` the surrogate is
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
  /// movement. No repositioning is needed regardless of
  /// [`SurrogateMode`] — [`show_initial`] / [`show_incoming`] already
  /// placed the surrogate at its natural rect.
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
  /// from its own screen-space center. In
  /// [`SurrogateMode::PinnedViewport`] mode the destination rect
  /// grows/shrinks via `rcDestination`; in [`SurrogateMode::Live`] mode
  /// the surrogate window itself is moved/resized to the scaled rect,
  /// batched via `batch`. Opacity is lerped according to
  /// `opacity_endpoint`.
  #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
  pub fn update_zoom(
    &mut self,
    batch: &mut SurrogateBatch,
    eased_progress: f32,
    is_incoming: bool,
  ) {
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

    if self.show_frame(Some(batch), &frame, t, panel, &monitor) {
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
    batch: &mut SurrogateBatch,
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

    if self.show_frame(Some(batch), &frame, scale, panel, monitor) {
      self.unclipped_rect = Some(to_logical(&frame, &self.frame_inset));
      self.apply_opacity_at(eased_progress, is_incoming);
    }
  }
}

impl Drop for WorkspaceSurrogate {
  /// A window still hidden by alpha would stay invisible once shown.
  fn drop(&mut self) {
    // Unregistered before `inner` destroys the window it is drawn on.
    self.ring = None;
    self.restore_alpha();
  }
}

#[cfg(test)]
mod tests {
  use super::{ring_rects, Rect, RECT};

  const MONITOR: Rect = Rect {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1080,
  };

  fn ltrb(rect: RECT) -> (i32, i32, i32, i32) {
    (rect.left, rect.top, rect.right, rect.bottom)
  }

  /// A window at rest, and its ring 3 px wide.
  fn window_and_ring() -> (Rect, Rect) {
    let window = Rect::from_xy(100, 100, 800, 600);
    (window.clone(), window.inset(-3))
  }

  /// `(rcSource, rcDestination)` of the ring with its window at `frame`.
  fn rects_at(
    frame: &Rect,
    scale: f32,
  ) -> Option<((i32, i32, i32, i32), (i32, i32, i32, i32))> {
    let (window, ring) = window_and_ring();
    let panel = MONITOR.translate_to_coordinates(
      frame.left - window.left,
      frame.top - window.top,
    );

    ring_rects(&ring, &window, frame, scale, &panel, &MONITOR, (0, 0))
      .map(|(src, dst)| (ltrb(src), ltrb(dst)))
  }

  #[test]
  fn ring_slides_whole_with_its_window() {
    let frame = Rect::from_xy(400, 100, 800, 600);

    assert_eq!(
      rects_at(&frame, 1.0),
      Some(((0, 0, 806, 606), (397, 97, 1203, 703)))
    );
  }

  #[test]
  fn ring_is_cut_at_the_monitor_edge() {
    let frame = Rect::from_xy(1500, 100, 800, 600);

    assert_eq!(
      rects_at(&frame, 1.0),
      Some(((0, 0, 423, 606), (1497, 97, 1920, 703)))
    );
  }

  #[test]
  fn ring_scales_with_its_window() {
    let frame = Rect::from_xy(200, 200, 400, 300);

    assert_eq!(
      rects_at(&frame, 0.5),
      Some(((0, 0, 806, 606), (198, 198, 602, 502)))
    );
  }

  #[test]
  fn ring_off_the_monitor_is_not_drawn() {
    let frame = Rect::from_xy(2000, 100, 800, 600);

    assert_eq!(rects_at(&frame, 1.0), None);
  }
}
