//! Particle bursts thrown off by windows (see `ParticlesConfig`): noted
//! as windows are resized or closed, and shown at the next sync, once the
//! resized ones' new rects are known.

use std::{cell::OnceCell, time::Duration};

use uuid::Uuid;
use wm_common::{
  DisplayState, EffectSet, InvokeCommand, Layout, WindowState,
};
use wm_platform::{
  Color, Dispatcher, ParticleBurst, ParticleEdges, ParticleStyle, Rect,
  UiThread,
};

use crate::{
  commands::general::border_overlay_params_for,
  layout::is_hidden_in_layout,
  models::{Container, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Bursts noted since the last sync, and the thread that shows them.
#[derive(Default)]
pub struct Particles {
  resized: Vec<Resized>,
  closed: Vec<Closed>,
  /// Started with the first burst; `None` inside if it could not be.
  thread: OnceCell<Option<UiThread>>,
}

/// A window being resized: where it was, and whether it animates to where
/// it goes.
struct Resized {
  window_id: Uuid,
  rect: Rect,
  animates: bool,
}

/// Where a window closed, the monitor it was on, and the effects it had.
struct Closed {
  rect: Rect,
  monitor: Rect,
  scale_factor: f32,
  set: EffectSet,
}

/// Whether `command` resizes the window it runs on.
pub fn resizes(command: &InvokeCommand) -> bool {
  matches!(
    command,
    InvokeCommand::Resize(_)
      | InvokeCommand::Size(_)
      | InvokeCommand::CycleColumnWidth { .. }
      | InvokeCommand::CycleWindowHeight { .. }
      | InvokeCommand::ResetWindowHeight
      | InvokeCommand::MaximizeColumn
      | InvokeCommand::ExpandColumnToAvailableWidth
  )
}

/// Notes that `container`, if a window on screen, is being resized, for
/// sparks from the edges that move once it lands there. `animates` says
/// whether it glides there or is already in place.
pub fn note_resize(
  state: &mut WmState,
  container: &Container,
  animates: bool,
) {
  let Ok(window) = container.as_window_container() else {
    return;
  };
  let resized = &mut state.particles.resized;
  if resized.iter().any(|noted| noted.window_id == window.id()) {
    return;
  }

  if let Some(rect) = on_screen_rect(&window) {
    resized.push(Resized {
      window_id: window.id(),
      rect,
      animates,
    });
  }
}

/// Notes that `window` is closing, for a puff of dust where it was on
/// screen.
pub fn note_close(state: &mut WmState, window: &WindowContainer) {
  let (Some(rect), Some(monitor)) =
    (on_screen_rect(window), window.monitor())
  else {
    return;
  };

  let properties = monitor.native_properties();
  let is_focused = state
    .focused_container()
    .is_some_and(|focused| focused.id() == window.id());

  state.particles.closed.push(Closed {
    rect,
    monitor: properties.bounds,
    scale_factor: properties.scale_factor,
    set: window.effect_set(is_focused),
  });
}

/// Shows the bursts noted since the last sync, as configured.
pub fn show_particles(state: &mut WmState, config: &UserConfig) {
  let resized = std::mem::take(&mut state.particles.resized);
  let closed = std::mem::take(&mut state.particles.closed);
  let settings = &config.value.animations.particles;

  if !settings.enabled || state.overview.is_shown() {
    return;
  }

  let mut bursts = Vec::new();

  if settings.on_resize {
    let is_suppressed = state.pending_sync.animations_suppressed();
    let focused_id = state.focused_container().map(|focused| focused.id());

    for resized in resized {
      let Some(window) = state.window_by_id(resized.window_id) else {
        continue;
      };
      let (Some(rect), Some(monitor)) =
        (on_screen_rect(&window), window.monitor())
      else {
        continue;
      };
      let edges = ParticleEdges::moved(&resized.rect, &rect);
      if !edges.any() {
        continue;
      }

      let properties = monitor.native_properties();
      let delay = if resized.animates && !is_suppressed {
        resize_duration(&window, config)
      } else {
        Duration::ZERO
      };
      let set = window.effect_set(focused_id == Some(window.id()));

      bursts.push(burst(
        ParticleStyle::Sparks,
        rect,
        edges,
        properties.bounds,
        properties.scale_factor,
        set,
        delay,
        config,
      ));
    }
  }

  if settings.on_close {
    bursts.extend(closed.into_iter().map(|closed| {
      burst(
        ParticleStyle::Dust,
        closed.rect,
        ParticleEdges::ALL,
        closed.monitor,
        closed.scale_factor,
        closed.set,
        Duration::ZERO,
        config,
      )
    }));
  }

  if bursts.is_empty() {
    return;
  }

  let Some(dispatcher) = dispatcher(&state.particles) else {
    return;
  };
  for burst in bursts {
    burst.spawn(&dispatcher);
  }
}

/// A burst in `style` from `edges` of `rect`, on a monitor at `monitor`,
/// for a window with `set`'s effects.
#[allow(clippy::too_many_arguments)]
fn burst(
  style: ParticleStyle,
  rect: Rect,
  edges: ParticleEdges,
  monitor: Rect,
  scale_factor: f32,
  set: EffectSet,
  delay: Duration,
  config: &UserConfig,
) -> ParticleBurst {
  let settings = &config.value.animations.particles;
  let white = Color {
    r: u8::MAX,
    g: u8::MAX,
    b: u8::MAX,
    a: u8::MAX,
  };

  #[allow(clippy::cast_precision_loss)]
  let size = settings.size.to_px(0, Some(scale_factor)) as f32;

  ParticleBurst {
    rect,
    edges,
    clip: monitor,
    style,
    color: settings
      .color
      .or_else(|| border_overlay_params_for(set, config).map(|p| p.color))
      .unwrap_or(white),
    size,
    density: settings.density,
    duration: Duration::from_millis(u64::from(settings.duration_ms)),
    delay,
  }
}

/// How long `window`'s resize takes to land: a scrolling layout's columns
/// move with the scroll animation, other windows with the resize one.
fn resize_duration(
  window: &WindowContainer,
  config: &UserConfig,
) -> Duration {
  let animations = &config.value.animations;
  let is_scrolling = matches!(window, WindowContainer::TilingWindow(_))
    && window
      .workspace()
      .is_some_and(|workspace| workspace.layout() == Layout::Scrolling);

  let (enabled, duration_ms, easing) = if is_scrolling {
    let scroll = &animations.scroll;
    (scroll.enabled, scroll.duration_ms, &scroll.easing)
  } else {
    let resize = &animations.window_resize;
    (resize.enabled, resize.duration_ms, &resize.easing)
  };

  if !enabled {
    return Duration::ZERO;
  }
  Duration::from_millis(u64::from(easing.duration_ms(duration_ms)))
}

/// `window`'s visible frame, while it is on screen.
fn on_screen_rect(window: &WindowContainer) -> Option<Rect> {
  let is_shown = window.display_state() == DisplayState::Shown
    && window.state() != WindowState::Minimized
    && !is_hidden_in_layout(window)
    && window
      .workspace()
      .is_some_and(|workspace| workspace.is_displayed());
  if !is_shown {
    return None;
  }

  let rect = window.to_rect().ok()?;
  let delta = window.total_border_delta().ok()?;
  Some(rect.apply_delta(&delta, None))
}

/// The thread bursts are shown from, started with the first one.
fn dispatcher(particles: &Particles) -> Option<Dispatcher> {
  particles
    .thread
    .get_or_init(|| {
      UiThread::spawn("glazewm-particles")
        .inspect_err(|err| {
          tracing::warn!("Failed to start particle thread: {err}");
        })
        .ok()
    })
    .as_ref()
    .map(|thread| thread.dispatcher().clone())
}

#[cfg(test)]
mod tests {
  use wm_common::{InvokeCommand, InvokeResizeCommand};

  use super::resizes;

  #[test]
  fn resize_commands_are_told_apart() {
    assert!(resizes(&InvokeCommand::Resize(InvokeResizeCommand {
      width: None,
      height: None,
    })));
    assert!(resizes(&InvokeCommand::MaximizeColumn));
    assert!(!resizes(&InvokeCommand::Close));
    assert!(!resizes(&InvokeCommand::CenterColumn));
  }
}
