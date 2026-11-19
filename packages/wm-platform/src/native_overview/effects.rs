//! The overview's flourishes on a screen: the selection's glow, ripples
//! out of cards, and particles out of where a window lands or closes.
//! The glow and ripples are pictures drawn once and only moved,
//! stretched and faded after; particles are bursts the compositor plays
//! in windows of their own over the screen.

use std::time::{Duration, Instant};

use windows::Win32::Foundation::HWND;

use super::{overlays::RING_RADIUS, PlaceContext, Screen};
use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{with_alpha, SliceLook},
  overview_effects::{pulse, Ripple},
  overview_layout::{chrome_opacity, CardMetrics, RectF},
  overview_overlays::{Highlight, NineSlice},
  OverviewEffects, OverviewStyle, ParticleEdges, ParticleEffects,
  ParticleSource,
};

/// How far the glow reaches past the selection's ring, in logical pixels.
const GLOW_REACH: f32 = 22.0;

/// How far a ripple spreads past what it spreads out of, in logical
/// pixels.
const RIPPLE_REACH: f32 = 36.0;

/// Most ripples showing at once.
const RIPPLES: usize = 3;

/// How far a card swells as a window lands on it, relative to its size.
const JELLY: f32 = 0.07;

/// How far a closed window's ring draws in, of the way to its middle.
const PUFF_DEPTH: f32 = 0.7;

/// The overview's flourishes on a screen.
pub(super) struct Effects {
  glow: Highlight<usize>,

  /// When the glow started breathing.
  pulse_from: Instant,

  /// Each ripple's ring, and where it spreads, while it does.
  ripples: Vec<(NineSlice, Option<Ripple>)>,

  /// Slot the next ripple takes, going round them.
  next_ripple: usize,
}

impl Effects {
  /// Effects whose glow settles over `highlight_ms`.
  pub(super) fn new(highlight_ms: f32) -> Option<Self> {
    Some(Self {
      glow: Highlight::new(highlight_ms)?,
      pulse_from: Instant::now(),
      ripples: (0..RIPPLES)
        .map(|_| NineSlice::new().map(|slices| (slices, None)))
        .collect::<Option<_>>()?,
      next_ripple: 0,
    })
  }

  /// Settles the glow over `highlight_ms`.
  pub(super) fn retune(&mut self, highlight_ms: f32) {
    self.glow.mark.retune(highlight_ms);
  }

  /// Registers them on `host`, bottom to top.
  pub(super) fn register(&mut self, host: HWND) {
    self.glow.slices.register(host);
    for (slices, _) in &mut self.ripples {
      slices.register(host);
    }
  }

  /// Hands their thumbnails over to `retired`, still registered.
  pub(super) fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    self.glow.slices.retire(retired);
    for (slices, _) in &mut self.ripples {
      slices.retire(retired);
    }
  }

  /// Draws them in `style`, at `metrics`' scale; each only if its look
  /// changed.
  #[allow(clippy::cast_possible_truncation)]
  pub(super) fn draw_looks(
    &mut self,
    style: &OverviewStyle,
    metrics: &CardMetrics,
  ) {
    let px = |length: f32| (length * metrics.scale_factor).round() as i32;
    let accent = style.accent;

    self.glow.slices.draw(SliceLook::glow(
      px(RING_RADIUS),
      px(GLOW_REACH),
      with_alpha(accent, 0.55),
    ));
    for (slices, _) in &mut self.ripples {
      slices.draw(SliceLook::ring(
        px(RING_RADIUS),
        px(3.0).max(1),
        accent,
      ));
    }
  }

  /// Gone at once, e.g. on a new open.
  pub(super) fn reset(&mut self) {
    self.glow.mark.reset();
    for (_, ripple) in &mut self.ripples {
      *ripple = None;
    }
    self.pulse_from = Instant::now();
  }

  /// Advances them by `dt` seconds. Returns whether any still moves,
  /// including one that just ended and has to be hidden.
  pub(super) fn step(&mut self, dt: f32) -> bool {
    let mut is_moving = self.glow.mark.step(dt);

    for (_, slot) in &mut self.ripples {
      if let Some(ripple) = slot {
        is_moving = true;
        if !ripple.step(dt) {
          *slot = None;
        }
      }
    }

    is_moving
  }

  /// Places them where they are now: the glow on `glow` (the selection's
  /// key, ring and corner scale), faded by `chrome`. Returns whether the
  /// glow still settles or breathes.
  pub(super) fn place(
    &mut self,
    glow: Option<(usize, RectF, f32)>,
    enabled: OverviewEffects,
    clip: &RectF,
    chrome: f32,
  ) -> bool {
    let now = Instant::now();
    let glow = glow.filter(|_| enabled.glow);
    let is_gliding = self.glow.mark.follow(glow, now);
    let breath = if enabled.pulse {
      pulse(now.duration_since(self.pulse_from).as_secs_f32())
    } else {
      1.0
    };
    self.glow.place(clip, chrome * breath);

    for (slices, ripple) in &mut self.ripples {
      match ripple.as_ref().and_then(Ripple::shown) {
        Some((rect, alpha)) => {
          slices.place(&rect, 1.0, clip, alpha * chrome);
        }
        None => slices.hide(),
      }
    }

    is_gliding || (enabled.pulse && self.glow.mark.shown().is_some())
  }

  /// Sends a ripple out of `rect`, spreading `scale_factor` times its
  /// logical reach.
  fn ripple(&mut self, rect: RectF, scale_factor: f32) {
    let index = self.next_ripple % self.ripples.len().max(1);
    if let Some((_, slot)) = self.ripples.get_mut(index) {
      *slot = Some(Ripple::new(rect, RIPPLE_REACH * scale_factor));
    }
    self.next_ripple = index + 1;
  }

  /// Sends a ring drawing in on `rect`, where a window closed.
  fn draw_in(&mut self, rect: RectF, scale_factor: f32) {
    // Stops short of where the ring's corners would meet.
    let corners = RING_RADIUS * scale_factor;
    let reach = (rect.w.min(rect.h) / 2.0 - corners).max(0.0) * PUFF_DEPTH;

    let index = self.next_ripple % self.ripples.len().max(1);
    if let Some((_, slot)) = self.ripples.get_mut(index) {
      *slot = Some(Ripple::new(rect, -reach));
    }
    self.next_ripple = index + 1;
  }
}

impl Screen {
  /// Effects the style turns on, while shown.
  fn enabled_effects(&self) -> OverviewEffects {
    self
      .style()
      .map_or(OverviewEffects::NONE, |style| style.effects)
  }

  /// Sends a ripple out of card `index`, as it is clicked.
  pub(super) fn ripple(&mut self, index: usize) {
    if !self.enabled_effects().ripples {
      return;
    }
    let scale_factor = self.metrics.scale_factor;
    if let (Some(rect), Some(effects)) =
      (self.landing_rect(index), &mut self.effects)
    {
      effects.ripple(rect, scale_factor);
      self.is_dirty = true;
    }
  }

  /// Celebrates a window landing on card `index`: a ripple out of it,
  /// particles out of its middle, and a wobble.
  pub(super) fn celebrate(&mut self, index: usize) {
    self.ripple(index);

    let enabled = self.enabled_effects();
    if enabled.jelly {
      if let Some(card) = self.cards.get_mut(index) {
        card.scale.kick(card.scale.target * JELLY);
      }
    }

    if let Some(rect) = self.landing_rect(index) {
      let (x, y) = rect.translate(self.area.x, self.area.y).center();
      self.throw_particles(
        enabled.drop_particles,
        ParticleSource::Point(x, y),
        Duration::ZERO,
      );
    }
  }

  /// Throws particles off the selected workspace, as the screen has
  /// settled open.
  pub(super) fn celebrate_open(&self) {
    if let Some(rect) = self.landing_rect(self.focus.selected) {
      let on_screen = rect.translate(self.area.x, self.area.y).to_rect();
      self.throw_particles(
        self.enabled_effects().open_particles,
        ParticleSource::Edges(on_screen, ParticleEdges::ALL),
        Duration::ZERO,
      );
    }
  }

  /// Throws particles off the windows of workspace `index`, which it
  /// zooms into as it closes, timed to land with the zoom `after` it.
  pub(super) fn celebrate_exit(&self, index: usize, after: Duration) {
    let Some(workspace) = self.workspaces().get(index) else {
      return;
    };

    let rects = workspace
      .windows
      .iter()
      .filter(|window| {
        !window.is_minimized
          && !window.is_hidden_tab
          && !window.is_scrolled_off
      })
      .map(|window| window.rect.clone())
      .collect::<Vec<_>>();
    if rects.is_empty() {
      return;
    }

    self.throw_particles(
      self.enabled_effects().exit_particles,
      ParticleSource::Rects(rects),
      after,
    );
  }

  /// Sees a window closed from the overview off from `rect`, where it
  /// showed: a ring drawing in on it, and particles off its edges.
  pub(super) fn puff(&mut self, rect: RectF) {
    let enabled = self.enabled_effects();

    if enabled.ripples {
      let scale_factor = self.metrics.scale_factor;
      if let Some(effects) = &mut self.effects {
        effects.draw_in(rect, scale_factor);
        self.is_dirty = true;
      }
    }

    let on_screen = rect.translate(self.area.x, self.area.y).to_rect();
    self.throw_particles(
      enabled.close_particles,
      ParticleSource::Edges(on_screen, ParticleEdges::ALL),
      Duration::ZERO,
    );
  }

  /// Throws off `effects` from `source`, on screen, above this screen,
  /// setting off `after` that long.
  fn throw_particles(
    &self,
    effects: ParticleEffects,
    source: ParticleSource,
    after: Duration,
  ) {
    let Some(style) = self.style() else {
      return;
    };

    let burst = style.particles.burst(
      effects,
      source,
      self.area.to_rect(),
      self.metrics.scale_factor,
      style.accent,
    );
    if let Some(mut burst) = burst {
      burst.delay = after;
      burst.show_above(self.hwnd);
    }
  }

  /// Where window `hwnd` shows now, if on one of its cards or its scene.
  pub(super) fn shown_window_rect(&self, hwnd: isize) -> Option<RectF> {
    let index = self
      .workspaces()
      .iter()
      .position(|workspace| workspace.window(hwnd).is_some())?;
    let context = self.place_context();
    self.window_rect_in(index, hwnd, &context)
  }

  /// Places the effects where they are now. Returns whether the glow
  /// still settles or breathes.
  pub(super) fn place_effects(&mut self, context: &PlaceContext) -> bool {
    let enabled = self.enabled_effects();
    let reach = self.metrics.px(GLOW_REACH);
    let glow = self.ringed().and_then(|index| {
      self
        .workspace_rect(index, context)
        .map(|(rect, scale)| (index, rect.inset(-reach * scale), scale))
    });

    let view = context.view;
    self.effects.as_mut().is_some_and(|effects| {
      effects.place(glow, enabled, &view, chrome_opacity(context.fade))
    })
  }

  /// Registers the effects' thumbnails on its window.
  pub(super) fn register_effects(&mut self) {
    let host = self.hwnd;
    if let Some(effects) = &mut self.effects {
      effects.register(host);
    }
  }
}
