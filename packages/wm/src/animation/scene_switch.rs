//! Workspace switches played on a scene covering their monitor (see
//! [`NativeScene`]), the way Hyprland plays them: the switch itself
//! happens at once, as with animations off, while each workspace's copies
//! move as one, like its render offset.
//!
//! The scene covers the monitor before anything changes beneath it, and
//! keeps the outgoing workspace's overlays and tab bars on screen under it
//! until their copies have gone. The incoming workspace's copies are added
//! once its windows are shown, and both workspaces start moving together.
//! A switch landing while one plays turns each workspace from wherever it
//! is, over the full duration again.

use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{
  EasingFunction, WindowState, WorkspaceSwitchAnimationConfig,
  WorkspaceSwitchDirection, WorkspaceSwitchStyle,
};
use wm_platform::{
  perf::{self, Stage},
  NativeScene, NativeWindowWindowsExt, Overlay, Rect, SceneBackdrop,
  SceneGroupId, SceneMotion, ScenePose, SceneSegment, SceneValue,
  SceneWindow,
};

use super::{curve, engine::apply_easing};
use crate::{
  commands::{
    general::backdrop_overlay_params_for, workspace::is_switch_animated,
  },
  models::{WindowContainer, Workspace},
  pending_sync::SceneSwitchRequest,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// How long the scene takes to fade off the windows it matches once
/// nothing moves, which blends in the shadows its copies lack.
const LIFT: Duration = Duration::from_millis(60);

/// Smallest scale a copy shrinks to, in place of 0, which leaves its
/// transform without an inverse.
const MIN_COPY_SCALE: f32 = 0.001;

/// The scenes playing workspace switches, one per monitor.
#[derive(Default)]
pub struct SceneSwitches {
  scenes: Vec<MonitorScene>,
}

struct MonitorScene {
  monitor_id: Uuid,
  bounds: Rect,
  native: NativeScene,
  groups: Vec<WorkspaceGroup>,
  lifting_since: Option<Instant>,

  /// Set when the switch it plays failed beneath it, for the next tick to
  /// drop it and let go of what it held.
  is_abandoned: bool,
}

/// A workspace's copies, moving as one.
struct WorkspaceGroup {
  workspace_id: Uuid,
  id: SceneGroupId,

  /// `None` while it waits for the switch beneath to happen, to start
  /// moving with the incoming workspace's copies.
  motion: Option<GroupMotion>,
  is_leaving: bool,

  /// Its windows, whose overlays and tab bars are kept on screen beneath
  /// the scene while it leaves, since copies of hidden windows are blank.
  held: Vec<Uuid>,
}

/// Where a group is drawn: offset in pixels, opacity, scale about the
/// monitor's center, and each copy's scale about its own.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
  x: f32,
  y: f32,
  opacity: f32,
  scale: f32,
  copy_scale: f32,
}

impl Pose {
  const REST: Self = Self {
    x: 0.0,
    y: 0.0,
    opacity: 1.0,
    scale: 1.0,
    copy_scale: 1.0,
  };

  fn lerp(&self, to: &Self, t: f32) -> Self {
    let between = |from: f32, to: f32| from + (to - from) * t;

    Self {
      x: between(self.x, to.x),
      y: between(self.y, to.y),
      opacity: between(self.opacity, to.opacity),
      scale: between(self.scale, to.scale),
      copy_scale: between(self.copy_scale, to.copy_scale),
    }
  }
}

#[derive(Clone, Debug)]
struct GroupMotion {
  from: Pose,
  to: Pose,
  started: Instant,
  duration: Duration,
  easing: EasingFunction,
}

impl GroupMotion {
  fn pose_at(&self, now: Instant) -> Pose {
    if self.is_done(now) {
      return self.to;
    }

    let progress =
      now.saturating_duration_since(self.started).as_secs_f32()
        / self.duration.as_secs_f32();
    self
      .from
      .lerp(&self.to, apply_easing(progress, &self.easing))
  }

  fn is_done(&self, now: Instant) -> bool {
    now.saturating_duration_since(self.started) >= self.duration
  }

  /// The motion for DWM to play, `fit` being its easing's fitted shape.
  fn scene_pose(&self, fit: &[curve::Segment]) -> ScenePose {
    let channel = |from: f32, to: f32| {
      if (to - from).abs() < f32::EPSILON || fit.is_empty() {
        return SceneValue::Fixed(to);
      }

      SceneValue::Moving(SceneMotion {
        begins_at: self.started,
        duration: self.duration,
        segments: fit
          .iter()
          .map(|segment| scaled(segment, from, to))
          .collect(),
        end: to,
      })
    };

    ScenePose {
      offset_x: channel(self.from.x, self.to.x),
      offset_y: channel(self.from.y, self.to.y),
      opacity: channel(self.from.opacity, self.to.opacity),
      scale: channel(self.from.scale, self.to.scale),
      copy_scale: channel(self.from.copy_scale, self.to.copy_scale),
    }
  }
}

/// `segment` of a motion from 0 to 1, as one from `from` to `to`.
#[allow(clippy::cast_possible_truncation)]
fn scaled(segment: &curve::Segment, from: f32, to: f32) -> SceneSegment {
  let delta = f64::from(to - from);
  let [c0, c1, c2, c3] = segment.coefficients;

  SceneSegment {
    begin_secs: segment.begin,
    coefficients: [
      (f64::from(from) + delta * c0) as f32,
      (delta * c1) as f32,
      (delta * c2) as f32,
      (delta * c3) as f32,
    ],
  }
}

impl MonitorScene {
  fn group_mut(
    &mut self,
    workspace_id: Uuid,
  ) -> Option<&mut WorkspaceGroup> {
    self
      .groups
      .iter_mut()
      .find(|group| group.workspace_id == workspace_id)
  }

  /// Whether its one workspace not leaving is the one its monitor shows,
  /// i.e. the monitor didn't change workspace without a switch on it.
  fn matches(&self, displayed: &HashMap<Uuid, Uuid>) -> bool {
    let mut staying = self.groups.iter().filter(|group| !group.is_leaving);

    staying.next().is_some_and(|group| {
      displayed.get(&self.monitor_id) == Some(&group.workspace_id)
    }) && staying.next().is_none()
  }

  /// Puts it back above windows shown on top beneath it, e.g. floating
  /// ones of the incoming workspace.
  fn raise(&self) {
    if let Err(err) = self.native.raise() {
      tracing::debug!("Scene couldn't be raised: {err}");
    }
  }

  /// Whether nothing moves anymore: every workspace that left is gone, and
  /// the one that came in is in place.
  fn is_settled(&self, now: Instant) -> bool {
    self.groups.iter().all(|group| {
      !group.is_leaving
        && group
          .motion
          .as_ref()
          .is_some_and(|motion| motion.is_done(now))
    })
  }
}

impl SceneSwitches {
  pub fn is_active(&self) -> bool {
    !self.scenes.is_empty()
  }

  /// Whether a scene still shows copies of `window_id`'s overlays and tab
  /// bar, which are then left on screen beneath it.
  pub fn holds(&self, window_id: &Uuid) -> bool {
    self.scenes.iter().any(|scene| {
      scene
        .groups
        .iter()
        .any(|group| group.is_leaving && group.held.contains(window_id))
    })
  }

  /// Takes every scene off the screen at once, e.g. as the monitors
  /// change.
  pub fn clear(&mut self) {
    self.scenes.clear();
  }

  fn scene_mut(&mut self, monitor_id: Uuid) -> Option<&mut MonitorScene> {
    self
      .scenes
      .iter_mut()
      .find(|scene| scene.monitor_id == monitor_id)
  }
}

/// Covers the outgoing workspace's monitor with a scene before the switch
/// happens beneath it, or readies the scene already covering it. Returns
/// whether the monitor is covered.
pub fn cover(
  state: &mut WmState,
  config: &UserConfig,
  request: &SceneSwitchRequest,
) -> bool {
  let _scope = perf::scope(Stage::SceneCover);

  let Some(outgoing) = workspace(state, request.outgoing) else {
    return false;
  };
  let Some(monitor) = outgoing.monitor() else {
    return false;
  };

  let props = monitor.native_properties();
  let wearing_focus =
    state.prev_effects_window.as_ref().map(CommonGetters::id);
  let (windows, held) =
    workspace_windows(state, config, &outgoing, wearing_focus, &props);

  let scenes = &mut state.animation_manager.scenes;
  if let Some(scene) = scenes.scene_mut(monitor.id()) {
    scene.raise();

    // Back over the windows beneath, on screen before they change, in
    // case it was fading off them.
    if scene.lifting_since.take().is_some() {
      let restored = scene
        .native
        .set_opacity(&SceneValue::Fixed(1.0))
        .and_then(|()| scene.native.commit());
      if restored.is_err() {
        scene.is_abandoned = true;
        return false;
      }
      wm_platform::dwm_flush();
    }

    // The incoming workspace's overlays and tab bars come back with its
    // windows. One the monitor stopped showing without a switch on the
    // scene, e.g. by a pick in the overview, isn't beneath it anymore.
    scene.groups.retain_mut(|group| {
      if group.workspace_id == request.incoming {
        group.held.clear();
      }

      let is_stale =
        !group.is_leaving && group.workspace_id != outgoing.id();
      if is_stale {
        scene.native.remove_group(group.id);
      }
      !is_stale
    });

    if let Some(group) = scene.group_mut(outgoing.id()) {
      group.is_leaving = true;
      group.held = held;
      return true;
    }

    // E.g. a workspace that came to the monitor while the scene was up.
    let added = scene
      .native
      .add_group(&windows)
      .and_then(|id| scene.native.commit().map(|()| id));

    return match added {
      Ok(id) => {
        scene.groups.push(WorkspaceGroup {
          workspace_id: outgoing.id(),
          id,
          motion: None,
          is_leaving: true,
          held,
        });
        true
      }
      Err(err) => {
        tracing::warn!(
          "Scene couldn't copy the outgoing workspace: {err}"
        );
        scene.is_abandoned = true;
        false
      }
    };
  }

  match NativeScene::cover(&props.bounds, &windows) {
    Ok((native, id)) => {
      scenes.scenes.push(MonitorScene {
        monitor_id: monitor.id(),
        bounds: props.bounds.clone(),
        native,
        groups: vec![WorkspaceGroup {
          workspace_id: outgoing.id(),
          id,
          motion: None,
          is_leaving: true,
          held,
        }],
        lifting_since: None,
        is_abandoned: false,
      });
      true
    }
    Err(err) => {
      tracing::warn!("No scene for the workspace switch: {err}");
      false
    }
  }
}

/// Adds the incoming workspace's copies once its windows are shown beneath
/// the scene, and starts both workspaces moving.
pub fn enter(
  state: &mut WmState,
  config: &UserConfig,
  request: &SceneSwitchRequest,
) {
  let _scope = perf::scope(Stage::SceneEnter);

  let Some(incoming) = workspace(state, request.incoming) else {
    abandon(state, request);
    return;
  };
  let Some(monitor) = incoming.monitor() else {
    abandon(state, request);
    return;
  };

  let props = monitor.native_properties();
  let focused = state.focused_container().map(|c| c.id());
  let (windows, _) =
    workspace_windows(state, config, &incoming, focused, &props);

  let ws_config = &config.value.animations.workspace_switch;
  let duration = Duration::from_millis(u64::from(
    ws_config.easing.duration_ms(ws_config.duration_ms),
  ));
  let fit = curve::fit(&ws_config.easing, duration);

  let Some(scene) = state.animation_manager.scenes.scene_mut(monitor.id())
  else {
    abandon(state, request);
    return;
  };
  scene.raise();
  let now = Instant::now();

  // One still leaving turns back from wherever it is.
  let entry = away(ws_config, request, &scene.bounds, false);
  let start = match scene
    .groups
    .iter()
    .position(|g| g.workspace_id == incoming.id())
  {
    Some(index) => {
      let group = scene.groups.remove(index);
      scene.native.remove_group(group.id);
      group
        .motion
        .map_or(Pose::REST, |motion| motion.pose_at(now))
    }
    None => entry,
  };

  let id = match scene.native.add_group(&windows) {
    Ok(id) => id,
    Err(err) => {
      tracing::warn!("Scene couldn't copy the incoming workspace: {err}");
      abandon(state, request);
      return;
    }
  };

  let exit = away(ws_config, request, &scene.bounds, true);
  for group in &mut scene.groups {
    if group.workspace_id == request.outgoing {
      let from =
        group.motion.as_ref().map_or(Pose::REST, |m| m.pose_at(now));
      group.motion = Some(GroupMotion {
        from,
        to: exit,
        started: now,
        duration,
        easing: ws_config.easing.clone(),
      });
    }
  }

  scene.groups.push(WorkspaceGroup {
    workspace_id: incoming.id(),
    id,
    motion: Some(GroupMotion {
      from: start,
      to: Pose::REST,
      started: now,
      duration,
      easing: ws_config.easing.clone(),
    }),
    is_leaving: false,
    held: Vec::new(),
  });

  let posed = scene
    .groups
    .iter()
    .filter(|group| {
      group.workspace_id == request.outgoing
        || group.workspace_id == incoming.id()
    })
    .filter_map(|group| {
      let motion = group.motion.as_ref()?;
      Some((group.id, motion.scene_pose(&fit)))
    })
    .collect::<Vec<_>>();

  let played = posed
    .iter()
    .try_for_each(|(id, pose)| scene.native.pose(*id, pose))
    .and_then(|()| scene.native.commit());

  match played {
    Ok(()) => perf::note_switch_start("scene"),
    Err(err) => {
      tracing::warn!("Scene couldn't play the workspace switch: {err}");
      abandon(state, request);
    }
  }
}

/// Gives up the scene covering the switch's monitor, e.g. when the sync
/// beneath it failed, for the next tick to reveal the windows as they are.
pub fn abandon(state: &mut WmState, request: &SceneSwitchRequest) {
  let monitor_id = [request.outgoing, request.incoming]
    .into_iter()
    .find_map(|id| workspace(state, id)?.monitor())
    .map(|monitor| monitor.id());

  for scene in &mut state.animation_manager.scenes.scenes {
    // With no monitor left to tell, every scene goes.
    if monitor_id.is_none_or(|id| id == scene.monitor_id) {
      scene.is_abandoned = true;
    }
  }
}

/// Retires the workspaces that have left, letting their overlays and tab
/// bars go, then fades each scene off its monitor once nothing moves.
pub fn tick(state: &mut WmState) {
  if !state.animation_manager.scenes.is_active() {
    return;
  }

  let _scope = perf::scope(Stage::SceneTick);
  let now = Instant::now();
  let mut released = false;

  let displayed = state
    .monitors()
    .into_iter()
    .filter_map(|monitor| {
      Some((monitor.id(), monitor.displayed_workspace()?.id()))
    })
    .collect::<HashMap<_, _>>();
  let is_overview_shown = state.overview.is_shown();

  state.animation_manager.scenes.scenes.retain_mut(|scene| {
    // A group still waiting for its motion here missed the end of the
    // sync that covered the monitor.
    if scene.is_abandoned
      || scene.groups.iter().any(|group| group.motion.is_none())
    {
      tracing::warn!("Scene dropped: its switch didn't play.");
      released = true;
      return false;
    }

    // What's beneath changed without it, which leaves the windows to show
    // as they are, as with animations off.
    if is_overview_shown || !scene.matches(&displayed) {
      tracing::debug!("Scene dropped: its monitor changed beneath it.");
      released = true;
      return false;
    }

    scene.raise();

    let mut retired = false;
    scene.groups.retain(|group| {
      let is_gone = group.is_leaving
        && group
          .motion
          .as_ref()
          .is_some_and(|motion| motion.is_done(now));
      if is_gone {
        scene.native.remove_group(group.id);
        retired = true;
      }
      !is_gone
    });

    if retired {
      released = true;
      if scene.native.commit().is_err() {
        return false;
      }
    }

    // Lifted only from the tick after any overlays it let go were hidden.
    if retired || !scene.is_settled(now) {
      return true;
    }

    match scene.lifting_since {
      None => {
        let lift = SceneValue::Moving(SceneMotion {
          begins_at: now,
          duration: LIFT,
          segments: vec![SceneSegment {
            begin_secs: 0.0,
            coefficients: [1.0, -1.0 / LIFT.as_secs_f32(), 0.0, 0.0],
          }],
          end: 0.0,
        });
        scene.lifting_since = Some(now);
        scene.native.set_opacity(&lift).is_ok()
          && scene.native.commit().is_ok()
      }
      Some(since) => now.saturating_duration_since(since) < LIFT,
    }
  });

  if released {
    state.pending_sync.queue_tab_bar_update();
  }
}

/// Where a workspace ends up as it leaves, or starts from as it enters.
fn away(
  config: &WorkspaceSwitchAnimationConfig,
  request: &SceneSwitchRequest,
  bounds: &Rect,
  is_leaving: bool,
) -> Pose {
  let opacity = if is_leaving {
    config.opacity_outgoing
  } else {
    config.opacity_incoming
  }
  .clamp(0.0, 1.0);

  match config.style {
    WorkspaceSwitchStyle::Slide => {
      // The incoming workspace comes from the side it sits on, and the
      // outgoing one leaves the other way.
      #[allow(clippy::cast_precision_loss)]
      let side = request.direction.signum() as f32
        * if is_leaving { -1.0 } else { 1.0 };

      #[allow(clippy::cast_precision_loss)]
      let (x, y) = match request.axis {
        WorkspaceSwitchDirection::Horizontal => {
          (side * bounds.width() as f32, 0.0)
        }
        WorkspaceSwitchDirection::Vertical => {
          (0.0, side * bounds.height() as f32)
        }
      };

      Pose {
        x,
        y,
        opacity,
        scale: 1.0 - config.zoom_factor.clamp(0.0, 1.0),
        copy_scale: 1.0,
      }
    }
    WorkspaceSwitchStyle::Zoom => Pose {
      opacity,
      copy_scale: MIN_COPY_SCALE,
      ..Pose::REST
    },
    WorkspaceSwitchStyle::Fade | WorkspaceSwitchStyle::Iris => Pose {
      opacity,
      ..Pose::REST
    },
  }
}

fn workspace(state: &WmState, id: Uuid) -> Option<Workspace> {
  state.container_by_id(id)?.as_workspace().cloned()
}

/// What a scene copies of `workspace`: each window shown in its layout,
/// placed where the layout has it (a column's peek shows it there too),
/// with its backdrop and border and its stack's tab bar. Returns them with
/// the IDs of the windows.
///
/// `wearing_focus` is the window drawn with the focused window's effects,
/// which it may not wear yet.
fn workspace_windows(
  state: &WmState,
  config: &UserConfig,
  workspace: &Workspace,
  wearing_focus: Option<Uuid>,
  monitor: &crate::models::NativeMonitorProperties,
) -> (Vec<SceneWindow>, Vec<Uuid>) {
  let mut copies = Vec::new();
  let mut ids = Vec::new();

  let focused_alpha = config
    .value
    .window_effects
    .focused_window
    .transparency
    .alpha();
  let wears_focus_effects =
    state.prev_effects_window.as_ref().map(CommonGetters::id);

  let windows = workspace
    .descendants()
    .filter_map(|container| container.as_window_container().ok())
    .filter(is_switch_animated);

  for window in windows {
    let Some(placement) = placement(&window) else {
      continue;
    };

    let is_focused = wearing_focus == Some(window.id());
    let backdrop = state
      .backdrop_overlays
      .get(&window.id())
      .filter(|overlay| overlay.is_visible())
      .zip(backdrop_overlay_params_for(is_focused, config))
      .map(|(overlay, params)| SceneBackdrop {
        overlay: overlay.hwnd(),
        params,
      });

    copies.push(SceneWindow {
      hwnd: window.native().hwnd(),
      placement: Some(placement),
      // One taking focus gets its focused transparency only once the
      // focus lands, after its copy is made.
      alpha: (is_focused && wears_focus_effects != Some(window.id()))
        .then_some(focused_alpha),
      corner_radius: corner_radius(
        &window,
        is_focused,
        config,
        monitor.scale_factor,
      ),
      backdrop,
      required: true,
    });

    if let Some(border) = state
      .border_overlays
      .get(&window.id())
      .filter(|overlay| overlay.is_visible())
    {
      copies.push(SceneWindow {
        hwnd: border.hwnd(),
        placement: None,
        alpha: None,
        corner_radius: 0.0,
        backdrop: None,
        required: false,
      });
    }

    ids.push(window.id());
  }

  let tab_bars = workspace
    .descendants()
    .filter_map(|container| container.as_stack().map(CommonGetters::id))
    .filter_map(|stack_id| state.tab_bars.get(&stack_id));

  for bar in tab_bars {
    if let Some(rect) = bar.shown_rect() {
      copies.push(SceneWindow {
        hwnd: bar.hwnd(),
        placement: Some((rect.clone(), rect)),
        alpha: None,
        corner_radius: 0.0,
        backdrop: None,
        required: false,
      });
    }
  }

  (copies, ids)
}

/// Where the layout puts `window`: its window rect, the invisible resize
/// border included, and its visible frame.
fn placement(window: &WindowContainer) -> Option<(Rect, Rect)> {
  let outer = window
    .to_rect()
    .ok()?
    .apply_delta(&window.total_border_delta().ok()?, None);
  let frame = outer.apply_delta(
    &window.native_properties().shadow_borders.inverse(),
    None,
  );

  Some((outer, frame))
}

/// Radius of `window`'s corners as DWM rounds them, in physical pixels.
fn corner_radius(
  window: &WindowContainer,
  is_focused: bool,
  config: &UserConfig,
  scale_factor: f32,
) -> f32 {
  if matches!(window.state(), WindowState::Fullscreen(_)) {
    return 0.0;
  }

  let effects = if is_focused {
    &config.value.window_effects.focused_window
  } else {
    &config.value.window_effects.other_windows
  };

  effects.resolved_corner_style().approx_radius_px() * scale_factor
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use wm_common::{
    EasingFunction, WorkspaceSwitchAnimationConfig,
    WorkspaceSwitchDirection, WorkspaceSwitchStyle,
  };
  use wm_platform::Rect;

  use super::{away, GroupMotion, Pose};
  use crate::pending_sync::SceneSwitchRequest;

  fn request(direction: i32) -> SceneSwitchRequest {
    SceneSwitchRequest {
      outgoing: uuid::Uuid::new_v4(),
      incoming: uuid::Uuid::new_v4(),
      direction,
      axis: WorkspaceSwitchDirection::Horizontal,
    }
  }

  #[test]
  fn slides_in_from_the_side_the_workspace_sits_on() {
    let config = WorkspaceSwitchAnimationConfig::default();
    let bounds = Rect::from_xy(0, 0, 2560, 1440);

    let entry = away(&config, &request(1), &bounds, false);
    let exit = away(&config, &request(1), &bounds, true);
    assert!((entry.x - 2560.0).abs() < f32::EPSILON);
    assert!((exit.x + 2560.0).abs() < f32::EPSILON);

    let entry = away(&config, &request(-1), &bounds, false);
    assert!((entry.x + 2560.0).abs() < f32::EPSILON);
  }

  #[test]
  fn fade_only_changes_opacity() {
    let config = WorkspaceSwitchAnimationConfig {
      style: WorkspaceSwitchStyle::Fade,
      opacity_outgoing: 0.0,
      ..WorkspaceSwitchAnimationConfig::default()
    };
    let bounds = Rect::from_xy(0, 0, 1920, 1080);

    let exit = away(&config, &request(1), &bounds, true);
    assert_eq!(
      exit,
      Pose {
        opacity: 0.0,
        ..Pose::REST
      }
    );
  }

  #[test]
  fn turns_back_from_where_it_is() {
    let started = Instant::now();
    let motion = GroupMotion {
      from: Pose::REST,
      to: Pose {
        x: -1000.0,
        ..Pose::REST
      },
      started,
      duration: Duration::from_millis(200),
      easing: EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0),
    };

    let halfway = motion.pose_at(started + Duration::from_millis(100));
    assert!((halfway.x + 500.0).abs() < 1.0);
    assert_eq!(
      motion.pose_at(started + Duration::from_millis(250)),
      motion.to
    );
  }
}
