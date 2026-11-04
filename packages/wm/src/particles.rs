//! Particle bursts thrown off by windows (see `ParticlesConfig`): noted
//! as windows are resized, moved, opened or closed, or seen as focus and
//! the shown workspaces change from one sync to the next, and shown at the
//! end of a sync, once windows' new rects are known.

use std::{
  cell::OnceCell,
  collections::HashSet,
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{DisplayState, EffectSet, InvokeCommand, WindowState};
use wm_platform::{
  Color, Dispatcher, ParticleBurst, ParticleEdges, ParticleEffects,
  ParticleLook, ParticleSource, Rect, UiThread,
};

use crate::{
  commands::general::border_overlay_params_for,
  layout::is_hidden_in_layout,
  models::{Container, Monitor, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Bursts noted since the last sync, what that sync showed, and the
/// thread that shows them.
#[derive(Default)]
pub struct Particles {
  changed: Vec<Changed>,
  closed: Vec<Closed>,
  /// What the last sync showed; `None` before the first.
  seen: Option<Seen>,
  /// Started with the first burst; `None` inside if it could not be.
  thread: OnceCell<Option<UiThread>>,
}

/// A window resized, moved or opened since the last sync.
struct Changed {
  window_id: Uuid,
  change: Change,
}

enum Change {
  /// Resized from where it was on screen.
  Resize(Rect),
  /// Moved from where it was on screen.
  Move(Rect),
  Open,
}

/// Where a window closed, the monitor it was on, and the effects it had.
struct Closed {
  rect: Rect,
  monitor: Rect,
  scale_factor: f32,
  set: EffectSet,
}

/// The focused window and each monitor's shown workspace, as of a sync.
#[derive(Clone)]
struct Seen {
  focused: Option<Uuid>,
  /// `(monitor, workspace)` IDs.
  shown: Vec<(Uuid, Uuid)>,
}

impl Seen {
  fn now(state: &WmState) -> Self {
    Self {
      focused: state
        .focused_container()
        .and_then(|focused| focused.as_window_container().ok())
        .map(|window| window.id()),
      shown: state
        .monitors()
        .iter()
        .filter_map(|monitor| {
          Some((monitor.id(), monitor.displayed_workspace()?.id()))
        })
        .collect(),
    }
  }

  /// Whether `monitor` showed a workspace other than `workspace`.
  fn showed_other(&self, monitor: Uuid, workspace: Uuid) -> bool {
    self
      .shown
      .iter()
      .any(|(id, shown)| *id == monitor && *shown != workspace)
  }
}

/// Whether `command` resizes the window it runs on.
fn resizes(command: &InvokeCommand) -> bool {
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

/// Whether `command` moves the window it runs on elsewhere.
fn moves(command: &InvokeCommand) -> bool {
  matches!(
    command,
    InvokeCommand::Move(_)
      | InvokeCommand::Position(_)
      | InvokeCommand::SwapWindow { .. }
      | InvokeCommand::ConsumeOrExpelWindow { .. }
      | InvokeCommand::ConsumeWindowIntoColumn
      | InvokeCommand::ExpelWindowFromColumn
  )
}

/// Notes that `command` resizes or moves `container`, if it does.
pub fn note_command(
  state: &mut WmState,
  container: &Container,
  command: &InvokeCommand,
) {
  if resizes(command) {
    note_resize(state, container);
  } else if moves(command) {
    note_move(state, container);
  }
}

/// Notes that `container`, if a window on screen, is being resized, for
/// particles off the edges that move once it lands.
pub fn note_resize(state: &mut WmState, container: &Container) {
  note_from(state, container, Change::Resize);
}

/// Notes that `container`, if a window on screen, is being moved, for
/// particles off the edges facing the way it went once it lands.
pub fn note_move(state: &mut WmState, container: &Container) {
  note_from(state, container, Change::Move);
}

/// Notes that `window` just opened, for particles off its edges once it
/// is in place.
pub fn note_open(state: &mut WmState, window: &WindowContainer) {
  note(state, window.id(), Change::Open);
}

/// Notes `container`'s `change` from where it is on screen now, if a
/// window there.
fn note_from(
  state: &mut WmState,
  container: &Container,
  change: impl FnOnce(Rect) -> Change,
) {
  let Ok(window) = container.as_window_container() else {
    return;
  };
  if let Some(rect) = on_screen_rect(&window) {
    note(state, window.id(), change(rect));
  }
}

/// Notes `change` to window `window_id`, unless it already changed since
/// the last sync.
fn note(state: &mut WmState, window_id: Uuid, change: Change) {
  let changed = &mut state.particles.changed;
  if !changed.iter().any(|noted| noted.window_id == window_id) {
    changed.push(Changed { window_id, change });
  }
}

/// Notes that `window` is closing, for particles off its edges where it
/// was on screen.
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

/// Shows the bursts noted since the last sync, and those for focus and
/// shown workspaces changing since, as configured.
pub fn show_particles(state: &mut WmState, config: &UserConfig) {
  let changed = std::mem::take(&mut state.particles.changed);
  let closed = std::mem::take(&mut state.particles.closed);
  let seen = Seen::now(state);
  let last = state.particles.seen.replace(seen.clone());

  // The overview throws off its own, over itself.
  if state.overview.is_shown() {
    return;
  }

  let settings = &config.value.animations.particles;
  let thrower = Thrower {
    state,
    config,
    look: settings.look(),
    focused: seen.focused,
    now: Instant::now(),
  };
  let mut bursts = Vec::new();
  // Windows already throwing particles off, which a focus change then
  // leaves be.
  let mut thrown = HashSet::new();

  if let Some(last) = &last {
    for monitor in state.monitors() {
      let Some(workspace) = monitor.displayed_workspace() else {
        continue;
      };
      if last.showed_other(monitor.id(), workspace.id()) {
        let windows = on_screen_windows(&workspace.as_container());
        if let Some(burst) = thrower.switch_burst(&monitor, &windows) {
          thrown.extend(windows.iter().map(CommonGetters::id));
          bursts.push(burst);
        }
      }
    }
  }

  for changed in changed {
    let Some(window) = state.window_by_id(changed.window_id) else {
      continue;
    };
    let Some(rect) = on_screen_rect(&window) else {
      continue;
    };

    let (effects, edges) = match &changed.change {
      Change::Resize(from) => {
        (settings.on_resize, ParticleEdges::moved(from, &rect))
      }
      Change::Move(from) => {
        (settings.on_move, ParticleEdges::leading(from, &rect))
      }
      Change::Open => (settings.on_open, ParticleEdges::ALL),
    };
    if !edges.any() {
      continue;
    }

    let source = ParticleSource::Edges(rect, edges);
    if let Some(burst) = thrower.window_burst(effects, &window, source) {
      thrown.insert(window.id());
      bursts.push(burst);
    }
  }

  let focused = seen
    .focused
    .filter(|id| {
      last.as_ref().is_some_and(|last| last.focused != Some(*id))
    })
    .filter(|id| !thrown.contains(id))
    .and_then(|id| state.window_by_id(id));
  if let Some(window) = focused {
    if let Some(rect) = on_screen_rect(&window) {
      let source = ParticleSource::Edges(rect, ParticleEdges::ALL);
      bursts.extend(thrower.window_burst(
        settings.on_focus,
        &window,
        source,
      ));
    }
  }

  bursts.extend(closed.into_iter().filter_map(|closed| {
    thrower.look.burst(
      settings.on_close,
      ParticleSource::Edges(closed.rect, ParticleEdges::ALL),
      closed.monitor,
      closed.scale_factor,
      window_color(closed.set, config),
    )
  }));

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

/// What a sync's bursts are made with.
struct Thrower<'a> {
  state: &'a WmState,
  config: &'a UserConfig,
  look: ParticleLook,
  /// The focused window, which throws particles in its focused color.
  focused: Option<Uuid>,
  now: Instant,
}

impl Thrower<'_> {
  /// A burst of `effects` from `source` on `window`, in its color, set
  /// off as it comes to rest where it is laid out.
  fn window_burst(
    &self,
    effects: ParticleEffects,
    window: &WindowContainer,
    source: ParticleSource,
  ) -> Option<ParticleBurst> {
    let properties = window.monitor()?.native_properties();
    let mut burst = self.look.burst(
      effects,
      source,
      properties.bounds,
      properties.scale_factor,
      self.color_of(window),
    )?;

    burst.delay = self
      .state
      .animation_manager
      .time_to_rest(&window.id(), self.now);
    Some(burst)
  }

  /// A burst off every edge of `windows`, the workspace `monitor` just
  /// switched to, or out of the middle of the monitor without any; in the
  /// color of the window that takes focus there, and set off as the
  /// switch lands.
  fn switch_burst(
    &self,
    monitor: &Monitor,
    windows: &[WindowContainer],
  ) -> Option<ParticleBurst> {
    let properties = monitor.native_properties();
    let source = if windows.is_empty() {
      let (x, y) = center(&properties.working_area);
      ParticleSource::Point(x, y)
    } else {
      ParticleSource::Rects(
        windows.iter().filter_map(on_screen_rect).collect(),
      )
    };
    let color = monitor
      .displayed_workspace()
      .and_then(|workspace| {
        workspace
          .descendant_focus_order()
          .find_map(|container| container.as_window_container().ok())
      })
      .map_or_else(
        || window_color(EffectSet::Focused, self.config),
        |window| self.color_of(&window),
      );

    let mut burst = self.look.burst(
      self.config.value.animations.particles.on_workspace_switch,
      source,
      properties.bounds,
      properties.scale_factor,
      color,
    )?;

    let switch = &self.config.value.animations.workspace_switch;
    if switch.enabled && !self.state.pending_sync.animations_suppressed() {
      burst.delay = Duration::from_millis(u64::from(
        switch.easing.duration_ms(switch.duration_ms),
      ));
    }
    Some(burst)
  }

  /// `window`'s border color, as focused or not.
  fn color_of(&self, window: &WindowContainer) -> Color {
    let set = window.effect_set(self.focused == Some(window.id()));
    window_color(set, self.config)
  }
}

/// Color of a window with `set`'s effects: its border's, or white
/// without one.
fn window_color(set: EffectSet, config: &UserConfig) -> Color {
  border_overlay_params_for(set, config).map_or(
    Color {
      r: u8::MAX,
      g: u8::MAX,
      b: u8::MAX,
      a: u8::MAX,
    },
    |params| params.color,
  )
}

#[allow(clippy::cast_precision_loss)]
fn center(rect: &Rect) -> (f32, f32) {
  (
    (rect.left + rect.right) as f32 / 2.0,
    (rect.top + rect.bottom) as f32 / 2.0,
  )
}

/// The windows in `container` on screen.
fn on_screen_windows(container: &Container) -> Vec<WindowContainer> {
  container
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok())
    .filter(|window| on_screen_rect(window).is_some())
    .collect()
}

/// `window`'s visible frame, while it is on screen or coming on.
fn on_screen_rect(window: &WindowContainer) -> Option<Rect> {
  let is_shown = matches!(
    window.display_state(),
    DisplayState::Shown | DisplayState::Showing
  ) && window.state() != WindowState::Minimized
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

  use super::{moves, resizes};

  #[test]
  fn resize_and_move_commands_are_told_apart() {
    let resize = InvokeCommand::Resize(InvokeResizeCommand {
      width: None,
      height: None,
    });

    assert!(resizes(&resize) && !moves(&resize));
    assert!(resizes(&InvokeCommand::MaximizeColumn));
    assert!(moves(&InvokeCommand::ExpelWindowFromColumn));
    assert!(
      !resizes(&InvokeCommand::Close) && !moves(&InvokeCommand::Close)
    );
    assert!(!moves(&InvokeCommand::CenterColumn));
  }
}
