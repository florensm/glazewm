use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{EffectSet, FocusChangeAnimationConfig};

use super::engine::{animation_progress_at, apply_easing};

/// A window's effects on their way from one effect set's style to
/// another's, as it gains or loses focus.
#[derive(Clone, Copy, Debug)]
struct FocusFade {
  from: EffectSet,
  to: EffectSet,
  /// How far toward `to` it started, for one that turned a running fade
  /// back.
  start_mix: f32,
  started_at: Instant,
}

/// Where a window's fade between two effect sets' styles is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusMix {
  pub from: EffectSet,
  pub to: EffectSet,
  /// Eased progress from `from` (`0.0`) to `to` (`1.0`). A spring may
  /// overshoot it.
  pub t: f32,
}

/// The windows whose effects are fading between focus states.
#[derive(Debug, Default)]
pub struct FocusFades {
  fades: HashMap<Uuid, FocusFade>,
}

impl FocusFades {
  /// Fades `window_id` from `from`'s style to `to`'s, starting `now`.
  ///
  /// One turned back to where its running fade came from carries on from
  /// where it is, so quick focus changes back and forth never jump.
  pub fn start(
    &mut self,
    window_id: Uuid,
    from: EffectSet,
    to: EffectSet,
    now: Instant,
    config: &FocusChangeAnimationConfig,
  ) {
    let running = self.mix(&window_id, now, config);
    if running.is_some_and(|mix| mix.to == to) || from == to {
      return;
    }

    let start_mix = running
      .filter(|mix| mix.from == to && mix.to == from)
      .map_or(0.0, |mix| (1.0 - mix.t).clamp(0.0, 1.0));

    self.fades.insert(
      window_id,
      FocusFade {
        from,
        to,
        start_mix,
        started_at: now,
      },
    );
  }

  /// Where `window_id`'s fade is at `now`, if it has one.
  #[must_use]
  pub fn mix(
    &self,
    window_id: &Uuid,
    now: Instant,
    config: &FocusChangeAnimationConfig,
  ) -> Option<FocusMix> {
    self.fades.get(window_id).map(|fade| FocusMix {
      from: fade.from,
      to: fade.to,
      t: fade.start_mix
        + (1.0 - fade.start_mix)
          * apply_easing(progress(fade, now, config), &config.easing),
    })
  }

  /// Drops `window_id`'s fade, leaving it to whatever draws it instead.
  pub fn cancel(&mut self, window_id: &Uuid) {
    self.fades.remove(window_id);
  }

  /// Windows with a fade, finished or not.
  pub fn window_ids(&self) -> impl Iterator<Item = Uuid> + '_ {
    self.fades.keys().copied()
  }

  /// Drops the fades finished by `now`, returning their windows.
  pub fn take_finished(
    &mut self,
    now: Instant,
    config: &FocusChangeAnimationConfig,
  ) -> Vec<Uuid> {
    let finished = self
      .fades
      .iter()
      .filter(|(_, fade)| progress(fade, now, config) >= 1.0)
      .map(|(id, _)| *id)
      .collect::<Vec<_>>();

    for id in &finished {
      self.fades.remove(id);
    }
    finished
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.fades.is_empty()
  }
}

/// Linear progress of `fade` at `now`, over what is left of the duration
/// after its `start_mix`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn progress(
  fade: &FocusFade,
  now: Instant,
  config: &FocusChangeAnimationConfig,
) -> f32 {
  let duration_ms = config.easing.duration_ms(config.duration_ms);
  let left = (f64::from(duration_ms)
    * f64::from(1.0 - fade.start_mix.clamp(0.0, 1.0)))
  .round() as u64;

  animation_progress_at(fade.started_at, Duration::from_millis(left), now)
}

#[cfg(test)]
mod tests {
  use wm_common::EasingFunction;

  use super::*;

  fn linear() -> FocusChangeAnimationConfig {
    FocusChangeAnimationConfig {
      enabled: true,
      duration_ms: 100,
      easing: EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0),
    }
  }

  fn at(start: Instant, ms: u64) -> Instant {
    start + Duration::from_millis(ms)
  }

  #[test]
  fn fade_runs_from_one_style_to_the_other() {
    let config = linear();
    let mut fades = FocusFades::default();
    let id = Uuid::new_v4();
    let start = Instant::now();

    fades.start(id, EffectSet::Other, EffectSet::Focused, start, &config);

    let mix = fades.mix(&id, at(start, 50), &config).expect("fading");
    assert_eq!((mix.from, mix.to), (EffectSet::Other, EffectSet::Focused));
    assert!((mix.t - 0.5).abs() < 0.02);

    assert!(fades.take_finished(at(start, 50), &config).is_empty());
    assert_eq!(fades.take_finished(at(start, 100), &config), vec![id]);
    assert!(fades.is_empty());
  }

  #[test]
  fn turning_back_carries_on_from_where_it_is() {
    let config = linear();
    let mut fades = FocusFades::default();
    let id = Uuid::new_v4();
    let start = Instant::now();

    fades.start(id, EffectSet::Other, EffectSet::Focused, start, &config);
    let back = at(start, 30);
    fades.start(id, EffectSet::Focused, EffectSet::Other, back, &config);

    // 30% of the way to focused is 70% of the way back.
    let mix = fades.mix(&id, back, &config).expect("fading");
    assert_eq!((mix.from, mix.to), (EffectSet::Focused, EffectSet::Other));
    assert!((mix.t - 0.7).abs() < 0.02);

    // What is left takes what is left of the duration.
    assert!(fades.take_finished(at(start, 50), &config).is_empty());
    assert_eq!(fades.take_finished(at(start, 60), &config), vec![id]);
  }

  #[test]
  fn same_target_keeps_the_running_fade() {
    let config = linear();
    let mut fades = FocusFades::default();
    let id = Uuid::new_v4();
    let start = Instant::now();

    fades.start(id, EffectSet::Other, EffectSet::Focused, start, &config);
    fades.start(
      id,
      EffectSet::Other,
      EffectSet::Focused,
      at(start, 40),
      &config,
    );

    let mix = fades.mix(&id, at(start, 40), &config).expect("fading");
    assert!((mix.t - 0.4).abs() < 0.02);
  }

  #[test]
  fn no_fade_between_equal_sets() {
    let config = linear();
    let mut fades = FocusFades::default();
    let id = Uuid::new_v4();

    fades.start(
      id,
      EffectSet::Other,
      EffectSet::Other,
      Instant::now(),
      &config,
    );
    assert!(fades.is_empty());
  }
}
