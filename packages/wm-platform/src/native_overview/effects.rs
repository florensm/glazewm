//! The overview's flourishes on a screen: the selection's glow, ripples
//! out of cards and sparks out of where a window lands. Each is a picture
//! drawn once and only moved, stretched and faded after.

use std::time::Instant;

use windows::Win32::Foundation::{HWND, RECT};

use super::{overlays::RING_RADIUS, Screen};
use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{with_alpha, SliceLook},
  overview_effects::{pulse, seed, Burst, Ripple},
  overview_layout::{chrome_opacity, CardMetrics, RectF},
  overview_overlays::{Highlight, NineSlice},
  overview_thumbnails::{to_opacity, to_win32, Picture},
  Color, OverviewEffects, OverviewStyle, Rect,
};

/// How far the glow reaches past the selection's ring, in logical pixels.
const GLOW_REACH: f32 = 22.0;

/// How far a ripple spreads past what it spreads out of, in logical
/// pixels.
const RIPPLE_REACH: f32 = 36.0;

/// Most ripples showing at once.
const RIPPLES: usize = 3;

/// How fast sparks fly out, in logical pixels per second, and about how
/// big they are, in logical pixels.
const SPARK_SPEED: f32 = 620.0;
const SPARK_SIZE: f32 = 9.0;

/// How far a card swells as a window lands on it, relative to its size.
const JELLY: f32 = 0.07;

/// Size a spark is drawn at, in logical pixels: larger than it shows, so
/// it stays round when scaled.
const SPARK_DRAWN: f32 = 16.0;

/// The overview's flourishes on a screen.
pub(super) struct Effects {
  glow: Highlight<usize>,

  /// When the glow started breathing.
  pulse_from: Instant,

  /// Each ripple's ring, and where it spreads, while it does.
  ripples: Vec<(NineSlice, Option<Ripple>)>,

  /// Slot the next ripple takes, going round them.
  next_ripple: usize,

  /// A spark, shown once per spark of a burst, and its color and size.
  spark: Picture,
  spark_drawn: Option<(Color, i32)>,
  sparks: Vec<Thumbnail>,

  burst: Option<Burst>,
}

impl Effects {
  /// Effects whose glow glides over `highlight_ms`.
  pub(super) fn new(highlight_ms: f32) -> Option<Self> {
    Some(Self {
      glow: Highlight::new(highlight_ms)?,
      pulse_from: Instant::now(),
      ripples: (0..RIPPLES)
        .map(|_| NineSlice::new().map(|slices| (slices, None)))
        .collect::<Option<_>>()?,
      next_ripple: 0,
      spark: Picture::new()?,
      spark_drawn: None,
      sparks: Vec::new(),
      burst: None,
    })
  }

  /// Glides the glow over `highlight_ms`, overshooting by `bounce`.
  pub(super) fn retune(&mut self, highlight_ms: f32, bounce: f32) {
    self.glow.glide.retune(highlight_ms, bounce);
  }

  /// Registers them on `host`, bottom to top, with room for `sparks`
  /// sparks.
  pub(super) fn register(&mut self, host: HWND, sparks: u32) {
    self.glow.slices.register(host);
    for (slices, _) in &mut self.ripples {
      slices.register(host);
    }
    self.sparks = (0..sparks)
      .filter_map(|_| Thumbnail::register(host, self.spark.window))
      .collect();
  }

  /// Hands their thumbnails over to `retired`, still registered.
  pub(super) fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    self.glow.slices.retire(retired);
    for (slices, _) in &mut self.ripples {
      slices.retire(retired);
    }
    retired.append(&mut self.sparks);
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

    let size = px(SPARK_DRAWN).max(2);
    if self.spark_drawn != Some((accent, size)) {
      self.spark.draw((size, size), |surface| {
        surface.pixels().fill(0);
        surface.canvas().fill_rounded_rect(
          &Rect::from_xy(0, 0, size, size),
          size / 2,
          accent,
        );
      });
      self.spark_drawn = Some((accent, size));
    }
  }

  /// Gone at once, e.g. on a new open.
  pub(super) fn reset(&mut self) {
    self.glow.glide.reset();
    for (_, ripple) in &mut self.ripples {
      *ripple = None;
    }
    self.burst = None;
    self.pulse_from = Instant::now();
  }

  /// Advances them by `dt` seconds. Returns whether any still moves,
  /// including one that just ended and has to be hidden.
  pub(super) fn step(&mut self, dt: f32) -> bool {
    let mut is_moving = self.glow.glide.step(dt);

    for (_, slot) in &mut self.ripples {
      if let Some(ripple) = slot {
        is_moving = true;
        if !ripple.step(dt) {
          *slot = None;
        }
      }
    }

    if let Some(burst) = &mut self.burst {
      is_moving = true;
      if !burst.step(dt) {
        self.burst = None;
      }
    }

    is_moving
  }

  /// Places them where they are now: the glow gliding to `glow` (the
  /// selection's key, ring and corner scale), faded by `chrome`. Returns
  /// whether the glow still glides or breathes.
  pub(super) fn place(
    &mut self,
    glow: Option<(usize, RectF, f32)>,
    enabled: OverviewEffects,
    clip: &RectF,
    chrome: f32,
  ) -> bool {
    let now = Instant::now();
    let glow = glow.filter(|_| enabled.glow);
    let is_gliding = self.glow.glide.follow(glow, now);
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

    let (width, height) = self.spark.size();
    let source = RECT {
      right: width,
      bottom: height,
      ..Default::default()
    };
    let clip_rect = to_win32(clip);
    let mut sparks = self.burst.iter().flat_map(Burst::sparks);
    for thumbnail in &mut self.sparks {
      match sparks.next() {
        Some((rect, alpha)) if alpha > 0.002 => thumbnail.place(
          source,
          to_win32(&rect),
          &clip_rect,
          to_opacity(alpha * chrome),
        ),
        _ => thumbnail.hide(),
      }
    }

    is_gliding || (enabled.pulse && self.glow.glide.shown().is_some())
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

  /// Bursts as many sparks as were registered out of (`x`, `y`).
  fn burst(&mut self, center: (f32, f32), scale_factor: f32) {
    self.burst = (!self.sparks.is_empty()).then(|| {
      Burst::new(
        center,
        self.sparks.len(),
        SPARK_SPEED * scale_factor,
        SPARK_SIZE * scale_factor,
        seed(),
      )
    });
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
  /// sparks out of its middle, and a wobble.
  pub(super) fn celebrate(&mut self, index: usize) {
    self.ripple(index);

    if self.enabled_effects().jelly {
      if let Some(card) = self.cards.get_mut(index) {
        card.scale.kick(card.scale.target * JELLY);
      }
    }

    let scale_factor = self.metrics.scale_factor;
    if let (Some(rect), Some(effects)) =
      (self.landing_rect(index), &mut self.effects)
    {
      effects.burst(rect.center(), scale_factor);
      self.is_dirty = true;
    }
  }

  /// Places the effects where they are now. Returns whether the glow
  /// still glides or breathes.
  pub(super) fn place_effects(&mut self, fade: f32, shift: f32) -> bool {
    let enabled = self.enabled_effects();
    let context = self.place_context();
    let reach = self.metrics.px(GLOW_REACH);
    let glow = self.ringed().and_then(|index| {
      self
        .workspace_rect(index, &context, shift)
        .map(|(rect, scale)| (index, rect.inset(-reach * scale), scale))
    });

    let view = self.view();
    self.effects.as_mut().is_some_and(|effects| {
      effects.place(glow, enabled, &view, chrome_opacity(fade))
    })
  }

  /// Registers the effects' thumbnails on its window, with room for as
  /// many sparks as the style bursts into.
  pub(super) fn register_effects(&mut self) {
    let sparks = self
      .enabled_effects()
      .sparks
      .min(OverviewEffects::MAX_SPARKS);
    let host = self.hwnd;
    if let Some(effects) = &mut self.effects {
      effects.register(host, sparks);
    }
  }
}
