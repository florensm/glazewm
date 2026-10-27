//! Animations played on scenes covering their monitors (see
//! [`NativeScene`]), the way Hyprland plays them: the WM changes the real
//! windows at once, as with animations off, while their copies move into
//! place on the scene.
//!
//! Before a sync changes anything, [`plan`] works out what it animates: a
//! workspace switch, and windows moving, resizing, opening or closing.
//! [`cover`] covers their monitors with scenes, or readies the scenes
//! already covering them, the sync happens beneath at once, and [`play`]
//! brings each scene in line with what it left, starting the motions. Once
//! nothing moves and the real windows have landed, [`tick`] fades each
//! scene off them.
//!
//! A workspace's windows are a group of the scene, moving as one in a
//! switch, and each window an item of it, moving and resizing on its own.
//! A motion landing while one plays turns from wherever it is.

use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{
  DisplayState, EasingFunction, TabBarPosition, WindowState,
  WindowTransitionParams, WindowTransitionStyle,
  WorkspaceSwitchAnimationConfig, WorkspaceSwitchDirection,
  WorkspaceSwitchStyle,
};
use wm_platform::{
  perf::{self, Stage},
  ItemPose, NativeScene, NativeWindow, NativeWindowWindowsExt, Overlay,
  Rect, SceneBackdrop, SceneFit, SceneGroupId, SceneItem, SceneMotion,
  ScenePose, SceneSegment, SceneValue, SceneWindow, HWND,
};

use super::{curve, engine::apply_easing, state::WindowAnimationState};
use crate::{
  commands::general::{
    backdrop_overlay_params_for, border_hidden_for_fullscreen,
    border_overlay_params_for,
  },
  layout::{is_hidden_in_layout, scrolling::column_of},
  models::{
    is_inactive_stack_child, NativeMonitorProperties, WindowContainer,
    Workspace,
  },
  pending_sync::{SceneCloseRequest, SceneSwitchRequest},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// How long the scene takes to fade off the windows it matches once
/// nothing moves, which blends in the shadows its copies lack.
const LIFT: Duration = Duration::from_millis(60);

/// Smallest scale an item shrinks to, in place of 0, which leaves its
/// transform without an inverse.
const MIN_SCALE: f32 = 0.001;

/// Longest a scene with nothing left moving waits for the windows moved
/// beneath it to get there, before fading off them anyway.
const LANDING_HOLD: Duration = Duration::from_millis(250);

/// How far a window may be from where the layout puts it and still count
/// as there: the frame DWM reports and the one derived from the layout can
/// round apart.
const LANDED_SLACK_PX: i32 = 2;

/// How long a new window's copy stays hidden before it enters, for its
/// app to paint a first frame.
const OPEN_PAINT_GRACE: Duration = Duration::from_millis(30);

/// The scenes covering monitors, one per monitor.
#[derive(Default)]
pub struct Scenes {
  scenes: Vec<MonitorScene>,
}

struct MonitorScene {
  monitor_id: Uuid,
  bounds: Rect,
  native: NativeScene,
  groups: Vec<WorkspaceGroup>,

  /// When nothing moved anymore, from which windows still moving beneath
  /// it get [`LANDING_HOLD`] to land.
  settled_since: Option<Instant>,
  lifting_since: Option<Instant>,

  /// Set when what it plays failed beneath it, for the next tick to drop
  /// it and let go of what it held.
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
  items: Vec<ItemState>,
}

/// An item of a group, and what it plays.
struct ItemState {
  key: u128,

  /// The window it draws, unless it is a stack's tab bar on its own.
  window_id: Option<Uuid>,

  /// What it was built from.
  built: SceneItem,

  /// Its frame at rest, which may lie elsewhere than where it was built:
  /// it is offset there rather than rebuilt.
  rest: Rect,
  motion: Option<ItemMotion>,

  /// Whether its window was moved beneath the scene, and may not be there
  /// yet.
  is_landing: bool,

  /// Whether its window went to another monitor, which it only moves
  /// towards on this one.
  is_passing: bool,

  /// The window's handle while its copy plays its close, to close it once
  /// that's done.
  closing: Option<isize>,
}

impl ItemState {
  fn new(built: SceneItem, window_id: Option<Uuid>) -> Self {
    Self {
      key: built.key,
      window_id,
      rest: built.frame.clone(),
      built,
      motion: None,
      is_landing: false,
      is_passing: false,
      closing: None,
    }
  }

  /// Where and how it is drawn at `now`.
  fn drawn_at(&self, now: Instant) -> Drawn {
    self.motion.as_ref().map_or_else(
      || Drawn::at_rest(self.rest.clone()),
      |motion| motion.drawn_at(now),
    )
  }

  fn is_moving(&self, now: Instant) -> bool {
    self
      .motion
      .as_ref()
      .is_some_and(|motion| !motion.is_done(now))
  }

  /// The pose DWM plays it from.
  fn pose(&self) -> ItemPose {
    match &self.motion {
      Some(motion) => motion.pose(&self.built.frame),
      None => rest_pose(&self.rest, &self.built.frame),
    }
  }
}

/// Where and how an item is drawn.
#[derive(Clone, Debug, PartialEq)]
struct Drawn {
  /// Where its window's visible frame is drawn.
  frame: Rect,

  /// Scale about the frame's center.
  zoom: f32,
  opacity: f32,
}

impl Drawn {
  fn at_rest(frame: Rect) -> Self {
    Self {
      frame,
      zoom: 1.0,
      opacity: 1.0,
    }
  }
}

/// An item moving from one place to another.
#[derive(Clone, Debug)]
struct ItemMotion {
  /// The motion of its frame, timed and eased, which the rest follows.
  animation: WindowAnimationState,
  started: Instant,
  from: Drawn,
  to: Drawn,

  /// Whether it is hidden until it starts, e.g. a new window yet to
  /// paint.
  is_hidden_first: bool,
}

impl ItemMotion {
  fn new(
    from: Drawn,
    to: Drawn,
    duration_ms: u32,
    easing: EasingFunction,
    now: Instant,
  ) -> Self {
    let animation = WindowAnimationState::new_movement(
      from.frame.clone(),
      to.frame.clone(),
      duration_ms,
      easing,
    );
    animation.start_clock(now);

    Self {
      animation,
      started: now,
      from,
      to,
      is_hidden_first: false,
    }
  }

  /// A motion from where this one is at `now` to `to`, carrying on at
  /// the speed it has there.
  fn retargeted(
    &self,
    to: Drawn,
    duration_ms: u32,
    easing: EasingFunction,
    now: Instant,
  ) -> Self {
    let animation = self.animation.retargeted(
      to.frame.clone(),
      duration_ms,
      easing,
      now,
    );
    animation.start_clock(now);

    Self {
      animation,
      started: now,
      from: self.drawn_at(now),
      to,
      is_hidden_first: false,
    }
  }

  fn drawn_at(&self, now: Instant) -> Drawn {
    let progress = self.animation.eased_progress_at(now);
    let between = |from: f32, to: f32| from + (to - from) * progress;

    let has_started = now.saturating_duration_since(self.started)
      >= self.animation.start_delay;
    let opacity = if self.is_hidden_first && !has_started {
      0.0
    } else {
      between(self.from.opacity, self.to.opacity)
    };

    Drawn {
      frame: self.from.frame.interpolate(&self.to.frame, progress),
      zoom: between(self.from.zoom, self.to.zoom),
      opacity,
    }
  }

  fn is_done(&self, now: Instant) -> bool {
    self.animation.is_complete_at(now)
  }

  /// The motion for DWM to play, for an item built at `built`.
  #[allow(clippy::cast_precision_loss)]
  fn pose(&self, built: &Rect) -> ItemPose {
    let fit = curve::fit(&self.animation.easing, self.animation.duration);
    let timing = Timing {
      started: self.started,
      delay: self.animation.start_delay,
      duration: self.animation.duration,
      fit: &fit,
    };

    let (from, to) = (&self.from, &self.to);
    let x = |drawn: &Drawn| (drawn.frame.left - built.left) as f32;
    let y = |drawn: &Drawn| (drawn.frame.top - built.top) as f32;
    let scale_x = |drawn: &Drawn| {
      (drawn.frame.width() as f32 / built.width().max(1) as f32)
        .max(MIN_SCALE)
    };
    let scale_y = |drawn: &Drawn| {
      (drawn.frame.height() as f32 / built.height().max(1) as f32)
        .max(MIN_SCALE)
    };

    ItemPose {
      offset_x: timing.channel(x(from), x(to), None),
      offset_y: timing.channel(y(from), y(to), None),
      scale_x: timing.channel(scale_x(from), scale_x(to), None),
      scale_y: timing.channel(scale_y(from), scale_y(to), None),
      zoom: timing.channel(
        from.zoom.max(MIN_SCALE),
        to.zoom.max(MIN_SCALE),
        None,
      ),
      opacity: timing.channel(
        from.opacity,
        to.opacity,
        self.is_hidden_first.then_some(0.0),
      ),
    }
  }
}

/// The pose of an item at rest at `rest`, built at `built`.
#[allow(clippy::cast_precision_loss)]
fn rest_pose(rest: &Rect, built: &Rect) -> ItemPose {
  let fixed = SceneValue::Fixed;

  ItemPose {
    offset_x: fixed((rest.left - built.left) as f32),
    offset_y: fixed((rest.top - built.top) as f32),
    scale_x: fixed(rest.width() as f32 / built.width().max(1) as f32),
    scale_y: fixed(rest.height() as f32 / built.height().max(1) as f32),
    zoom: fixed(1.0),
    opacity: fixed(1.0),
  }
}

/// When a motion plays, and its easing's fitted shape (see
/// [`curve::fit`]).
struct Timing<'a> {
  started: Instant,

  /// How long it waits before moving.
  delay: Duration,
  duration: Duration,
  fit: &'a [curve::Segment],
}

impl Timing<'_> {
  /// A value going from `from` to `to`, at `waiting` (else `from`) while
  /// it waits to start.
  fn channel(
    &self,
    from: f32,
    to: f32,
    waiting: Option<f32>,
  ) -> SceneValue {
    let waiting = waiting.unwrap_or(from);
    let waits =
      !self.delay.is_zero() && (waiting - to).abs() >= f32::EPSILON;
    if ((to - from).abs() < f32::EPSILON && !waits) || self.fit.is_empty()
    {
      return SceneValue::Fixed(to);
    }

    let delay = self.delay.as_secs_f64();
    let wait = waits.then_some(SceneSegment {
      begin_secs: 0.0,
      coefficients: [waiting, 0.0, 0.0, 0.0],
    });
    let segments = wait
      .into_iter()
      .chain(self.fit.iter().map(|segment| {
        let mut scaled = scaled(segment, from, to);
        scaled.begin_secs += delay;
        scaled
      }))
      .collect();

    SceneValue::Moving(SceneMotion {
      begins_at: self.started,
      duration: self.delay + self.duration,
      segments,
      end: to,
    })
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

/// Where a group is drawn: offset in pixels, opacity, scale about the
/// monitor's center, and each item's scale about its own.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
  x: f32,
  y: f32,
  opacity: f32,
  scale: f32,
  item_scale: f32,
}

impl Pose {
  const REST: Self = Self {
    x: 0.0,
    y: 0.0,
    opacity: 1.0,
    scale: 1.0,
    item_scale: 1.0,
  };

  fn lerp(&self, to: &Self, t: f32) -> Self {
    let between = |from: f32, to: f32| from + (to - from) * t;

    Self {
      x: between(self.x, to.x),
      y: between(self.y, to.y),
      opacity: between(self.opacity, to.opacity),
      scale: between(self.scale, to.scale),
      item_scale: between(self.item_scale, to.item_scale),
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
  /// A group resting where it is, e.g. one only its windows move in.
  fn at_rest(now: Instant) -> Self {
    Self {
      from: Pose::REST,
      to: Pose::REST,
      started: now,
      duration: Duration::ZERO,
      easing: EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0),
    }
  }

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
    let timing = Timing {
      started: self.started,
      delay: Duration::ZERO,
      duration: self.duration,
      fit,
    };
    let (from, to) = (&self.from, &self.to);

    ScenePose {
      offset_x: timing.channel(from.x, to.x, None),
      offset_y: timing.channel(from.y, to.y, None),
      opacity: timing.channel(from.opacity, to.opacity, None),
      scale: timing.channel(from.scale, to.scale, None),
      item_scale: timing.channel(from.item_scale, to.item_scale, None),
    }
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
  /// ones of an incoming workspace.
  fn raise(&self) {
    if let Err(err) = self.native.raise() {
      tracing::debug!("Scene couldn't be raised: {err}");
    }
  }

  /// Back over the windows beneath, on screen before they change, in
  /// case it was fading off them. Returns whether it is.
  fn restore(&mut self) -> bool {
    self.raise();
    self.settled_since = None;

    if self.lifting_since.take().is_some() {
      let restored = self
        .native
        .set_opacity(&SceneValue::Fixed(1.0))
        .and_then(|()| self.native.commit());
      if restored.is_err() {
        self.is_abandoned = true;
        return false;
      }
      wm_platform::dwm_flush();
    }

    true
  }

  /// Whether nothing moves anymore: every workspace that left is gone,
  /// the one that came in is in place, and so are its items.
  fn is_settled(&self, now: Instant) -> bool {
    self.groups.iter().all(|group| {
      !group.is_leaving
        && group
          .motion
          .as_ref()
          .is_some_and(|motion| motion.is_done(now))
        && group
          .items
          .iter()
          .all(|item| !item.is_moving(now) && item.closing.is_none())
    })
  }
}

impl Scenes {
  pub fn is_active(&self) -> bool {
    !self.scenes.is_empty()
  }

  /// Whether a scene still shows copies of `window_id`'s overlays and tab
  /// bar, which are then left on screen beneath it: one of a workspace
  /// leaving, or one closing.
  pub fn holds(&self, window_id: &Uuid) -> bool {
    self
      .scenes
      .iter()
      .flat_map(|scene| &scene.groups)
      .any(|group| {
        (group.is_leaving && group.held.contains(window_id))
          || group.items.iter().any(|item| {
            item.closing.is_some() && item.window_id == Some(*window_id)
          })
      })
  }

  fn scene_mut(&mut self, monitor_id: Uuid) -> Option<&mut MonitorScene> {
    self
      .scenes
      .iter_mut()
      .find(|scene| scene.monitor_id == monitor_id)
  }

  /// The item drawing `window_id` in its displayed workspace, if a scene
  /// has one.
  fn item(&self, window_id: Uuid) -> Option<&ItemState> {
    self
      .scenes
      .iter()
      .flat_map(|scene| &scene.groups)
      .filter(|group| !group.is_leaving)
      .flat_map(|group| &group.items)
      .find(|item| item.window_id == Some(window_id) && !item.is_passing)
  }
}

/// What a sync plays on scenes, worked out before it changes anything.
#[derive(Default)]
pub struct ScenePlan {
  switch: Option<SceneSwitchRequest>,

  /// Windows moving or resizing.
  motions: HashMap<Uuid, PlannedMotion>,

  /// Windows opening, with how they enter.
  opens: HashMap<Uuid, WindowTransitionParams>,
  closes: Vec<SceneCloseRequest>,

  /// Monitors the sync animates on, which it happens beneath once
  /// covered.
  monitors: Vec<Uuid>,
}

/// A move or resize of a window a sync plays.
struct PlannedMotion {
  /// Where and how it is drawn as the sync starts.
  from: Drawn,
  duration_ms: u32,
  easing: EasingFunction,

  /// Whether it carries on at the speed it's moving at, as a scrolling
  /// layout's columns do, rather than starting over.
  carries_speed: bool,
}

/// Works out what the coming sync plays on scenes, before it changes
/// anything.
pub fn plan(state: &mut WmState, config: &UserConfig) -> ScenePlan {
  let switch = state.pending_sync.take_scene_switch();
  let closes = state.pending_sync.take_scene_closes();

  let is_dragging = state
    .windows()
    .iter()
    .any(|window| window.active_drag().is_some());

  let mut plan = ScenePlan {
    closes,
    ..ScenePlan::default()
  };

  // While the overview is shown, its zoom is the transition.
  if state.overview.is_shown()
    || is_dragging
    || !NativeScene::is_supported()
  {
    return plan;
  }

  for close in &plan.closes {
    plan.monitors.push(close.monitor_id);
  }

  // Beneath a switch, everything else happens at once.
  if switch.is_some() {
    plan.switch = switch;
    return plan;
  }

  if state.pending_sync.animations_suppressed() {
    return plan;
  }

  let windows = state.windows_to_redraw();
  let targets = windows
    .iter()
    .filter_map(|window| Some((window.clone(), layout_rect(window)?)))
    .collect::<Vec<_>>();

  // With any window resizing, ones only moving take the resize's timing,
  // so their edges keep together.
  let cycle_has_resize = targets.iter().any(|(window, target)| {
    state
      .window_target_positions
      .get(&window.id())
      .is_some_and(|prev| !is_same_size(prev, target))
  });

  let now = Instant::now();
  for (window, target) in targets {
    let previous =
      state.window_target_positions.get(&window.id()).cloned();

    let is_drawn = is_drawn(&window)
      && window.workspace().is_some_and(|ws| ws.is_displayed());
    let Some(monitor) = window.monitor().filter(|_| is_drawn) else {
      continue;
    };

    let Some(previous) = previous else {
      if let Some(params) = opening(&window, config) {
        plan.opens.insert(window.id(), params);
        plan.monitors.push(monitor.id());
      }
      continue;
    };

    let Some(motion) = planned_motion(
      state,
      config,
      &window,
      &previous,
      &target,
      cycle_has_resize,
      now,
    ) else {
      continue;
    };

    // Covering where it comes from too, if that's another monitor.
    if let Some(from) =
      state.monitor_at_point(&motion.from.frame.center_point())
    {
      plan.monitors.push(from.id());
    }
    plan.monitors.push(monitor.id());
    plan.motions.insert(window.id(), motion);
  }

  plan.monitors.sort_unstable();
  plan.monitors.dedup();
  plan
}

/// How `window`, drawn where `previous` was its target, moves to `target`,
/// if it animates.
fn planned_motion(
  state: &WmState,
  config: &UserConfig,
  window: &WindowContainer,
  previous: &Rect,
  target: &Rect,
  cycle_has_resize: bool,
  now: Instant,
) -> Option<PlannedMotion> {
  let id = window.id();
  let is_floating = matches!(window.state(), WindowState::Floating(_));
  let is_fullscreen = matches!(window.state(), WindowState::Fullscreen(_));

  // Fullscreen windows only animate into or out of fullscreen the WM puts
  // them in, and floating ones only as they change state.
  if (is_fullscreen && !state.pending_sync.is_wm_fullscreen_toggle(&id))
    || (is_floating && !state.pending_sync.is_window_state_change(&id))
  {
    return None;
  }

  let animations = &config.value.animations;
  let is_column = column_of(&window.as_container()).is_some();
  let is_resize = !is_same_size(previous, target);

  let (enabled, threshold) = if is_column {
    (animations.scroll.enabled, animations.scroll.threshold_px)
  } else if is_resize {
    (
      animations.window_resize.enabled,
      animations.window_resize.threshold_px,
    )
  } else {
    (
      animations.window_move.enabled,
      animations.window_move.threshold_px,
    )
  };

  if !enabled {
    return None;
  }

  let target_frame = frame_of(window, target);
  let in_flight = state
    .animation_manager
    .scenes
    .item(id)
    .and_then(|item| item.motion.as_ref().filter(|m| !m.is_done(now)));

  // One already moving heads for wherever its target went; one at rest
  // starts once the change is worth showing.
  let is_worth_it = match in_flight {
    Some(motion) => motion.to.frame != target_frame,
    None => {
      distance(previous, target)
        > i32::try_from(threshold).unwrap_or(i32::MAX)
    }
  };
  if !is_worth_it {
    return None;
  }

  let from = match state.animation_manager.scenes.item(id) {
    Some(item) => item.drawn_at(now),
    None => Drawn::at_rest(drawn_frame(state, window, previous)),
  };

  let (duration_ms, easing) = if is_column {
    (
      animations.scroll.duration_ms,
      animations.scroll.easing.clone(),
    )
  } else if is_resize
    || (cycle_has_resize && animations.window_resize.enabled)
  {
    (
      animations.window_resize.duration_ms,
      animations.window_resize.easing.clone(),
    )
  } else {
    (
      animations.window_move.duration_ms,
      animations.window_move.easing.clone(),
    )
  };

  Some(PlannedMotion {
    from,
    duration_ms,
    easing,
    carries_speed: is_column,
  })
}

/// How a new `window` enters, if it animates.
fn opening(
  window: &WindowContainer,
  config: &UserConfig,
) -> Option<WindowTransitionParams> {
  let open = &config.value.animations.window_open;
  let params = WindowTransitionParams::from_open(open);

  let is_still = params.style == WindowTransitionStyle::None
    && params.away_opacity >= 1.0;

  (open.enabled
    && !is_still
    && !matches!(
      window.state(),
      WindowState::Floating(_) | WindowState::Fullscreen(_)
    ))
  .then_some(params)
}

/// Covers the monitors `plan` animates on before the sync changes anything
/// beneath them, or readies the scenes already covering them. Monitors
/// that can't be covered have their animations dropped from `plan`,
/// playing at once.
pub fn cover(
  state: &mut WmState,
  config: &UserConfig,
  plan: &mut ScenePlan,
) {
  let _scope = perf::scope(Stage::SceneCover);

  if let Some(request) = plan.switch.take() {
    if cover_switch(state, config, &request) {
      plan.switch = Some(request);
    }
  }

  let now = Instant::now();
  let mut covered = Vec::new();

  for monitor_id in std::mem::take(&mut plan.monitors) {
    if shows_fullscreen(state, monitor_id) {
      continue;
    }

    let is_covered =
      match state.animation_manager.scenes.scene_mut(monitor_id) {
        Some(scene) => scene.restore(),
        None => cover_monitor(state, config, monitor_id, plan, now),
      };

    if is_covered {
      covered.push(monitor_id);
    }
  }

  // The windows that close keep their copies, and their overlays, until
  // their close has played. Ones no scene plays close at once.
  let closes = std::mem::take(&mut plan.closes);
  for close in closes {
    let is_held = covered.contains(&close.monitor_id)
      && hold_close(&mut state.animation_manager.scenes, &close);

    if is_held {
      plan.closes.push(close);
    } else {
      close_now(state, close.window_id, close.handle);
    }
  }

  plan.motions.retain(|id, _| {
    window_monitor(state, *id).is_some_and(|m| covered.contains(&m))
  });
  plan.opens.retain(|id, _| {
    window_monitor(state, *id).is_some_and(|m| covered.contains(&m))
  });
  plan.monitors = covered;

  if plan.switch.is_some() || !plan.monitors.is_empty() {
    state.pending_sync.suppress_animations();
  }
}

/// Whether `monitor_id` shows a fullscreen window the WM isn't putting
/// into or out of fullscreen, e.g. a game, which a window covering it
/// would kick out of exclusive fullscreen.
fn shows_fullscreen(state: &WmState, monitor_id: Uuid) -> bool {
  state
    .monitors()
    .into_iter()
    .find(|monitor| monitor.id() == monitor_id)
    .and_then(|monitor| monitor.displayed_workspace())
    .is_some_and(|workspace| {
      drawn_windows(&workspace).any(|window| {
        matches!(window.state(), WindowState::Fullscreen(_))
          && !state.pending_sync.is_wm_fullscreen_toggle(&window.id())
      })
    })
}

/// Keeps the copy of `close`'s window in its monitor's scene, for the
/// scene to play its close. Returns whether it is kept.
fn hold_close(scenes: &mut Scenes, close: &SceneCloseRequest) -> bool {
  let Some(scene) = scenes.scene_mut(close.monitor_id) else {
    return false;
  };
  let Some(group) = scene.groups.iter_mut().find(|g| !g.is_leaving) else {
    return false;
  };

  let key = close.window_id.as_u128();
  if !group.items.iter().any(|item| item.key == key) {
    if let Err(err) = scene.native.set_item(group.id, &close.item) {
      tracing::warn!("Scene couldn't copy a closing window: {err}");
      return false;
    }
    group
      .items
      .push(ItemState::new(close.item.clone(), Some(close.window_id)));
  }

  let Some(item) = group.items.iter_mut().find(|item| item.key == key)
  else {
    return false;
  };
  item.closing = Some(close.handle);
  item.window_id = Some(close.window_id);
  true
}

/// Covers the monitor `monitor_id` with a scene of what is on it now.
fn cover_monitor(
  state: &mut WmState,
  config: &UserConfig,
  monitor_id: Uuid,
  plan: &ScenePlan,
  now: Instant,
) -> bool {
  let Some(monitor) = state
    .monitors()
    .into_iter()
    .find(|monitor| monitor.id() == monitor_id)
  else {
    return false;
  };
  let Some(workspace) = monitor.displayed_workspace() else {
    return false;
  };

  let props = monitor.native_properties();
  let mut items = shown_items(state, config, &workspace, &props, plan);
  for close in plan.closes.iter().filter(|c| c.monitor_id == monitor_id) {
    items.push((close.item.clone(), Some(close.window_id)));
  }

  // New windows enter from nothing.
  let left_out = plan
    .opens
    .keys()
    .filter_map(|id| state.container_by_id(*id))
    .filter_map(|container| container.as_window_container().ok())
    .map(|window| window.native().hwnd())
    .collect::<Vec<_>>();

  let scene_items = items
    .iter()
    .map(|(item, _)| item.clone())
    .collect::<Vec<_>>();
  match NativeScene::cover(&props.bounds, &scene_items, &left_out) {
    Ok((native, id)) => {
      let items = items
        .into_iter()
        .map(|(item, window_id)| ItemState::new(item, window_id))
        .collect();

      state.animation_manager.scenes.scenes.push(MonitorScene {
        monitor_id,
        bounds: props.bounds.clone(),
        native,
        groups: vec![WorkspaceGroup {
          workspace_id: workspace.id(),
          id,
          motion: Some(GroupMotion::at_rest(now)),
          is_leaving: false,
          held: Vec::new(),
          items,
        }],
        settled_since: None,
        lifting_since: None,
        is_abandoned: false,
      });
      true
    }
    Err(err) => {
      tracing::warn!("No scene for the animation: {err}");
      false
    }
  }
}

/// Covers the outgoing workspace's monitor with a scene before the switch
/// happens beneath it, or readies the scene already covering it. Returns
/// whether the monitor is covered.
fn cover_switch(
  state: &mut WmState,
  config: &UserConfig,
  request: &SceneSwitchRequest,
) -> bool {
  let Some(outgoing) = workspace(state, request.outgoing) else {
    return false;
  };
  let Some(monitor) = outgoing.monitor() else {
    return false;
  };

  let props = monitor.native_properties();
  let items =
    shown_items(state, config, &outgoing, &props, &ScenePlan::default());
  let held = items.iter().filter_map(|(_, id)| *id).collect::<Vec<_>>();
  let now = Instant::now();

  let scenes = &mut state.animation_manager.scenes;
  if let Some(scene) = scenes.scene_mut(monitor.id()) {
    if !scene.restore() {
      return false;
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
    let scene_items = items
      .iter()
      .map(|(item, _)| item.clone())
      .collect::<Vec<_>>();
    let added = scene
      .native
      .add_group(&scene_items)
      .and_then(|id| scene.native.commit().map(|()| id));

    return match added {
      Ok(id) => {
        scene.groups.push(WorkspaceGroup {
          workspace_id: outgoing.id(),
          id,
          motion: Some(GroupMotion::at_rest(now)),
          is_leaving: true,
          held,
          items: item_states(items),
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

  let scene_items = items
    .iter()
    .map(|(item, _)| item.clone())
    .collect::<Vec<_>>();
  match NativeScene::cover(&props.bounds, &scene_items, &[]) {
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
          items: item_states(items),
        }],
        settled_since: None,
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

fn item_states(items: Vec<(SceneItem, Option<Uuid>)>) -> Vec<ItemState> {
  items
    .into_iter()
    .map(|(item, window_id)| ItemState::new(item, window_id))
    .collect()
}

/// Brings the scenes in line with what the sync left beneath them, and
/// starts what `plan` plays, or gives up the scenes it covered if the sync
/// failed.
pub fn play(
  state: &mut WmState,
  config: &UserConfig,
  plan: &ScenePlan,
  is_synced: bool,
) {
  let _scope = perf::scope(Stage::SceneEnter);
  let now = Instant::now();

  if !is_synced {
    if let Some(request) = &plan.switch {
      abandon_switch(state, request);
    }
    for scene in &mut state.animation_manager.scenes.scenes {
      if plan.monitors.contains(&scene.monitor_id) {
        scene.is_abandoned = true;
      }
    }
    return;
  }

  if let Some(request) = &plan.switch {
    enter(state, config, request, now);
  }

  let monitor_ids = state
    .animation_manager
    .scenes
    .scenes
    .iter()
    .map(|scene| scene.monitor_id)
    .collect::<Vec<_>>();

  for monitor_id in monitor_ids {
    if let Err(err) = reconcile(state, config, monitor_id, plan, now) {
      tracing::warn!(
        "Scene couldn't follow the windows beneath it: {err}"
      );
      if let Some(scene) =
        state.animation_manager.scenes.scene_mut(monitor_id)
      {
        scene.is_abandoned = true;
      }
    }
  }

  // The real windows of closing copies go, beneath the scenes.
  for close in &plan.closes {
    let _ = NativeWindow::from_handle(close.handle).set_cloaked(true);
  }

  if state.animation_manager.scenes.is_active() {
    state.animation_manager.ensure_timer_running();
  }
}

/// Brings `monitor_id`'s scene in line with its displayed workspace, and
/// starts the motions `plan` has for it.
#[allow(clippy::too_many_lines)]
fn reconcile(
  state: &mut WmState,
  config: &UserConfig,
  monitor_id: Uuid,
  plan: &ScenePlan,
  now: Instant,
) -> anyhow::Result<()> {
  let Some(monitor) = state
    .monitors()
    .into_iter()
    .find(|monitor| monitor.id() == monitor_id)
  else {
    return Ok(());
  };
  let Some(workspace) = monitor.displayed_workspace() else {
    return Ok(());
  };

  let props = monitor.native_properties();
  let mut wanted =
    post_sync_items(state, config, &workspace, &props, plan);

  // Windows moving here from another monitor pass over this one too.
  let passing = plan
    .motions
    .iter()
    .filter(|(_, motion)| {
      motion.from.frame.intersection(&props.bounds).is_some()
    })
    .filter(|(id, _)| window_monitor(state, **id) != Some(monitor_id))
    .filter_map(|(id, _)| state.container_by_id(*id))
    .filter_map(|container| container.as_window_container().ok())
    .filter_map(|window| {
      let other = window.monitor()?.native_properties();
      let item = window_item(
        state,
        config,
        &window,
        layout_placement(&window)?,
        is_focused(state, &window),
        other.scale_factor,
        true,
      );
      Some((item, Some(window.id())))
    })
    .collect::<Vec<_>>();
  let passing_keys = passing
    .iter()
    .map(|(item, _)| item.key)
    .collect::<HashSet<_>>();
  wanted.extend(passing);

  let Some(scene) = state.animation_manager.scenes.scene_mut(monitor_id)
  else {
    return Ok(());
  };
  let Some(group_index) = scene
    .groups
    .iter()
    .position(|g| !g.is_leaving && g.workspace_id == workspace.id())
  else {
    return Ok(());
  };

  let group_id = scene.groups[group_index].id;
  let mut changed = false;
  let wanted_keys = wanted
    .iter()
    .map(|(item, _)| item.key)
    .collect::<HashSet<_>>();

  for (item, window_id) in wanted {
    let planned = window_id.and_then(|id| plan.motions.get(&id));
    let open = window_id.and_then(|id| plan.opens.get(&id));
    let group = &mut scene.groups[group_index];
    let existing =
      group.items.iter_mut().find(|known| known.key == item.key);

    let is_resizing = planned.is_some_and(|planned| {
      !is_same_size(&planned.from.frame, &item.frame)
    });

    match existing {
      None => {
        scene.native.set_item(group_id, &item)?;
        let mut added = ItemState::new(item, window_id);
        added.is_passing = passing_keys.contains(&added.key);

        if let Some(params) = open {
          added.motion = Some(entry_motion(&added.rest, params, now));
        } else if let Some(planned) = planned {
          added.motion = Some(ItemMotion::new(
            planned.from.clone(),
            Drawn::at_rest(added.rest.clone()),
            planned.duration_ms,
            planned.easing.clone(),
            now,
          ));
          added.is_landing = !added.is_passing;
        }

        if added.motion.is_some() {
          scene.native.pose_item(group_id, added.key, &added.pose())?;
        }
        scene.groups[group_index].items.push(added);
        changed = true;
      }
      Some(existing) => {
        if existing.closing.is_some() {
          continue;
        }

        let is_rebuilt =
          !looks_alike(&existing.built, &item, !is_resizing);
        let has_moved = existing.rest != item.frame;

        if is_rebuilt {
          scene.native.set_item(group_id, &item)?;
          existing.built = item.clone();
        }
        existing.rest = item.frame.clone();
        existing.is_passing = passing_keys.contains(&existing.key);

        if let Some(planned) = planned {
          let to = Drawn::at_rest(existing.rest.clone());
          existing.motion = Some(match &existing.motion {
            Some(motion)
              if planned.carries_speed && !motion.is_done(now) =>
            {
              motion.retargeted(
                to,
                planned.duration_ms,
                planned.easing.clone(),
                now,
              )
            }
            _ => ItemMotion::new(
              existing.drawn_at(now),
              to,
              planned.duration_ms,
              planned.easing.clone(),
              now,
            ),
          });
          existing.is_landing = !existing.is_passing;
        } else if has_moved {
          // Moved without a motion, e.g. too little to animate.
          existing.motion = None;
          existing.is_landing = true;
        }

        if is_rebuilt || has_moved || planned.is_some() {
          scene.native.pose_item(
            group_id,
            existing.key,
            &existing.pose(),
          )?;
          changed = true;
        }
      }
    }
  }

  // Closing ones play their close.
  for close in plan.closes.iter().filter(|c| c.monitor_id == monitor_id) {
    let group = &mut scene.groups[group_index];
    let Some(item) = group
      .items
      .iter_mut()
      .find(|item| item.key == close.window_id.as_u128())
    else {
      continue;
    };

    item.motion =
      Some(exit_motion(&item.drawn_at(now), &close.params, now));
    scene.native.pose_item(group_id, item.key, &item.pose())?;
    changed = true;
  }

  // Ones whose window isn't drawn here anymore go, unless still moving
  // out.
  let group = &mut scene.groups[group_index];
  let native = &mut scene.native;
  group.items.retain(|item| {
    let is_kept = wanted_keys.contains(&item.key)
      || item.closing.is_some()
      || (item.is_passing && item.is_moving(now));
    if !is_kept {
      native.remove_item(group_id, item.key);
      changed = true;
    }
    is_kept
  });

  scene.native.restack(group_id)?;
  if changed {
    scene.settled_since = None;
  }
  scene.native.commit()?;
  Ok(())
}

/// The motion of a new window entering at `rest`, as `params` say.
fn entry_motion(
  rest: &Rect,
  params: &WindowTransitionParams,
  now: Instant,
) -> ItemMotion {
  let mut motion = ItemMotion::new(
    away_from(rest, params),
    Drawn::at_rest(rest.clone()),
    params.duration_ms,
    params.easing.clone(),
    now,
  );
  motion.animation.start_delay = OPEN_PAINT_GRACE;
  motion.is_hidden_first = true;
  motion
}

/// The motion of a closing window leaving from where it is `drawn`, as
/// `params` say.
fn exit_motion(
  drawn: &Drawn,
  params: &WindowTransitionParams,
  now: Instant,
) -> ItemMotion {
  let mut to = away_from(&drawn.frame, params);
  to.opacity *= drawn.opacity;

  ItemMotion::new(
    drawn.clone(),
    to,
    params.duration_ms,
    params.easing.clone(),
    now,
  )
}

/// Where a window at `rest` enters from or leaves to, as `params` say.
fn away_from(rest: &Rect, params: &WindowTransitionParams) -> Drawn {
  let (width, height) = (rest.width(), rest.height());
  let frame = match params.style {
    WindowTransitionStyle::SlideRight => {
      rest.translate_to_coordinates(rest.x() + width, rest.y())
    }
    WindowTransitionStyle::SlideLeft => {
      rest.translate_to_coordinates(rest.x() - width, rest.y())
    }
    WindowTransitionStyle::SlideTop => {
      rest.translate_to_coordinates(rest.x(), rest.y() - height)
    }
    WindowTransitionStyle::SlideBottom => {
      rest.translate_to_coordinates(rest.x(), rest.y() + height)
    }
    WindowTransitionStyle::None | WindowTransitionStyle::Zoom => {
      rest.clone()
    }
  };

  Drawn {
    frame,
    zoom: if params.style == WindowTransitionStyle::Zoom {
      MIN_SCALE
    } else {
      1.0
    },
    opacity: params.away_opacity.clamp(0.0, 1.0),
  }
}

/// Adds the incoming workspace's copies once its windows are shown beneath
/// the scene, and starts both workspaces moving.
#[allow(clippy::too_many_lines)]
fn enter(
  state: &mut WmState,
  config: &UserConfig,
  request: &SceneSwitchRequest,
  now: Instant,
) {
  let Some(incoming) = workspace(state, request.incoming) else {
    abandon_switch(state, request);
    return;
  };
  let Some(monitor) = incoming.monitor() else {
    abandon_switch(state, request);
    return;
  };

  let props = monitor.native_properties();
  let items = post_sync_items(
    state,
    config,
    &incoming,
    &props,
    &ScenePlan::default(),
  );

  let ws_config = &config.value.animations.workspace_switch;
  let duration = Duration::from_millis(u64::from(
    ws_config.easing.duration_ms(ws_config.duration_ms),
  ));
  let fit = curve::fit(&ws_config.easing, duration);

  let Some(scene) = state.animation_manager.scenes.scene_mut(monitor.id())
  else {
    abandon_switch(state, request);
    return;
  };
  scene.raise();

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

  let scene_items = items
    .iter()
    .map(|(item, _)| item.clone())
    .collect::<Vec<_>>();
  let id = match scene.native.add_group(&scene_items) {
    Ok(id) => id,
    Err(err) => {
      tracing::warn!("Scene couldn't copy the incoming workspace: {err}");
      abandon_switch(state, request);
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
    items: item_states(items),
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
      abandon_switch(state, request);
    }
  }
}

/// Gives up the scene covering the switch's monitor, e.g. when the sync
/// beneath it failed, for the next tick to reveal the windows as they are.
fn abandon_switch(state: &mut WmState, request: &SceneSwitchRequest) {
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

/// Retires what has played out on the scenes, and fades each one off its
/// monitor once nothing moves there and the windows beneath have landed.
#[allow(clippy::too_many_lines)]
pub fn tick(state: &mut WmState) {
  if !state.animation_manager.scenes.is_active() {
    return;
  }

  let _scope = perf::scope(Stage::SceneTick);
  let now = Instant::now();
  let mut released = false;
  let mut closed = Vec::new();

  let displayed = state
    .monitors()
    .into_iter()
    .filter_map(|monitor| {
      Some((monitor.id(), monitor.displayed_workspace()?.id()))
    })
    .collect::<HashMap<_, _>>();
  let is_overview_shown = state.overview.is_shown();

  // An interactive drag moves its window through the OS, which copies
  // can't follow.
  let is_dragging = state
    .windows()
    .iter()
    .any(|window| window.active_drag().is_some());

  let landing = landing_windows(state);

  state.animation_manager.scenes.scenes.retain_mut(|scene| {
    let drop_reason = if scene.is_abandoned {
      Some("what it played failed")
    } else if scene.groups.iter().any(|group| group.motion.is_none()) {
      // Waiting for its switch, it missed the end of the sync.
      Some("its switch didn't play")
    } else if is_overview_shown
      || is_dragging
      || !scene.matches(&displayed)
    {
      // What's beneath changed without it, which leaves the windows to
      // show as they are, as with animations off.
      Some("its monitor changed beneath it")
    } else {
      None
    };

    if let Some(reason) = drop_reason {
      tracing::debug!("Scene dropped: {reason}.");
      closed.extend(closings(scene));
      released = true;
      return false;
    }

    scene.raise();

    let mut changed = false;

    // Workspaces that left go, along with closed windows and ones that
    // moved off the monitor.
    let native = &mut scene.native;
    scene.groups.retain_mut(|group| {
      let is_gone = group.is_leaving
        && group
          .motion
          .as_ref()
          .is_some_and(|motion| motion.is_done(now));
      if is_gone {
        native.remove_group(group.id);
        changed = true;
        released = true;
        return false;
      }

      let group_id = group.id;
      group.items.retain(|item| {
        let has_played = !item.is_moving(now);
        if let (Some(handle), true) = (item.closing, has_played) {
          if let Some(window_id) = item.window_id {
            closed.push((window_id, handle));
          }
        } else if !(item.is_passing && has_played) {
          return true;
        }

        native.remove_item(group_id, item.key);
        changed = true;
        false
      });

      if !group.is_leaving {
        if let Err(err) = native.restack(group_id) {
          tracing::debug!("Scene couldn't restack: {err}");
        }
      }
      true
    });

    for group in &mut scene.groups {
      for item in &mut group.items {
        if item.is_landing
          && !item.is_moving(now)
          && item.window_id.is_some_and(|id| !landing.contains(&id))
        {
          item.is_landing = false;
        }
      }
    }

    if scene.native.commit().is_err() {
      closed.extend(closings(scene));
      released = true;
      return false;
    }

    // Lifted only from the tick after anything it let go was hidden.
    if changed || !scene.is_settled(now) {
      scene.settled_since = None;
      return true;
    }

    let settled_since = *scene.settled_since.get_or_insert(now);
    let is_landing = scene
      .groups
      .iter()
      .flat_map(|group| &group.items)
      .any(|item| item.is_landing);
    if is_landing
      && now.saturating_duration_since(settled_since) < LANDING_HOLD
    {
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

  for (window_id, handle) in closed {
    close_now(state, window_id, handle);
  }

  if released {
    state.pending_sync.queue_tab_bar_update();
  }
}

/// Takes every scene off the screen at once, e.g. as the monitors change,
/// closing the windows whose close they played.
pub fn drop_all(state: &mut WmState) {
  let scenes = std::mem::take(&mut state.animation_manager.scenes.scenes);
  let closed = scenes.iter().flat_map(closings).collect::<Vec<_>>();
  drop(scenes);

  for (window_id, handle) in closed {
    close_now(state, window_id, handle);
  }
}

/// The windows `scene` plays the close of.
fn closings(scene: &MonitorScene) -> Vec<(Uuid, isize)> {
  scene
    .groups
    .iter()
    .flat_map(|group| &group.items)
    .filter_map(|item| Some((item.window_id?, item.closing?)))
    .collect()
}

/// Sends `WM_CLOSE` to a window detached for its close, and lets go of its
/// overlays.
fn close_now(state: &mut WmState, window_id: Uuid, handle: isize) {
  state.backdrop_overlays.remove(&window_id);
  state.border_overlays.remove(&window_id);

  if let Err(err) = NativeWindow::from_handle(handle).close() {
    tracing::warn!("Failed to close window {window_id}: {err}");
  }
  state
    .closing_windows
    .mark_close_sent(window_id, Instant::now());
}

/// The windows of the scenes' items that were moved beneath them and
/// aren't where the layout puts them yet.
fn landing_windows(state: &WmState) -> HashSet<Uuid> {
  state
    .animation_manager
    .scenes
    .scenes
    .iter()
    .flat_map(|scene| &scene.groups)
    .flat_map(|group| &group.items)
    .filter(|item| item.is_landing)
    .filter_map(|item| {
      let id = item.window_id?;
      let window =
        state.container_by_id(id)?.as_window_container().ok()?;

      // A peek draws one that isn't where its column is.
      if is_hidden_in_layout(&window) {
        return None;
      }

      let frame = window.native().frame().ok()?;
      let is_off = [
        frame.left - item.rest.left,
        frame.top - item.rest.top,
        frame.right - item.rest.right,
        frame.bottom - item.rest.bottom,
      ]
      .iter()
      .any(|edge| edge.abs() > LANDED_SLACK_PX);
      is_off.then_some(id)
    })
    .collect()
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
        item_scale: 1.0,
      }
    }
    WorkspaceSwitchStyle::Zoom => Pose {
      opacity,
      item_scale: MIN_SCALE,
      ..Pose::REST
    },
    WorkspaceSwitchStyle::Fade => Pose {
      opacity,
      ..Pose::REST
    },
  }
}

fn workspace(state: &WmState, id: Uuid) -> Option<Workspace> {
  state.container_by_id(id)?.as_workspace().cloned()
}

/// What a scene copies of `workspace` as the sync starts: an item for each
/// window on screen, where it is, with the effects it wears. Returns them
/// with the IDs of their windows.
///
/// `plan`'s new windows are left out, to enter from nothing.
fn shown_items(
  state: &WmState,
  config: &UserConfig,
  workspace: &Workspace,
  props: &NativeMonitorProperties,
  plan: &ScenePlan,
) -> Vec<(SceneItem, Option<Uuid>)> {
  let wearing_focus =
    state.prev_effects_window.as_ref().map(CommonGetters::id);

  let items = drawn_windows(workspace)
    .filter(|window| !plan.opens.contains_key(&window.id()))
    .filter_map(|window| {
      // A peek draws a column where the layout last put it.
      let is_peeked = state
        .animation_manager
        .scroll_view
        .draws_window(&window.id());

      let placement = if is_peeked {
        let outer = state.window_target_positions.get(&window.id())?;
        (outer.clone(), frame_of(&window, outer))
      } else {
        let is_shown = matches!(
          window.display_state(),
          DisplayState::Shown | DisplayState::Showing
        );
        if !is_shown {
          return None;
        }
        actual_placement(&window)?
      };

      let item = window_item(
        state,
        config,
        &window,
        placement,
        wearing_focus == Some(window.id()),
        props.scale_factor,
        false,
      );
      Some((item, Some(window.id())))
    })
    .collect();

  with_tab_bars(state, workspace, items, &HashSet::new())
}

/// What a scene copies of `workspace` once the sync has happened: an item
/// for each window its layout draws, where the layout puts it, which its
/// real window may not have reached yet. Returns them with the IDs of
/// their windows.
fn post_sync_items(
  state: &WmState,
  config: &UserConfig,
  workspace: &Workspace,
  props: &NativeMonitorProperties,
  plan: &ScenePlan,
) -> Vec<(SceneItem, Option<Uuid>)> {
  let now = Instant::now();
  let mut resizing = HashSet::new();

  let items = drawn_windows(workspace)
    .filter_map(|window| {
      let id = window.id();
      let placement = layout_placement(&window)?;

      // Borders and tab bars are cut to resize with their window.
      let is_resizing = plan.motions.get(&id).is_some_and(|planned| {
        !is_same_size(&planned.from.frame, &placement.1)
      }) || state
        .animation_manager
        .scenes
        .item(id)
        .and_then(|item| item.motion.as_ref())
        .is_some_and(|motion| {
          !motion.is_done(now)
            && !is_same_size(&motion.from.frame, &motion.to.frame)
        });
      if is_resizing {
        resizing.insert(id.as_u128());
      }

      let item = window_item(
        state,
        config,
        &window,
        placement,
        is_focused(state, &window),
        props.scale_factor,
        is_resizing,
      );
      Some((item, Some(id)))
    })
    .collect();

  with_tab_bars(state, workspace, items, &resizing)
}

/// `items` with the tab bars of `workspace`'s stacks drawn with their
/// active windows, cut to resize with them if theirs are `resizing`, or
/// on their own.
fn with_tab_bars(
  state: &WmState,
  workspace: &Workspace,
  mut items: Vec<(SceneItem, Option<Uuid>)>,
  resizing: &HashSet<u128>,
) -> Vec<(SceneItem, Option<Uuid>)> {
  let stacks = workspace
    .descendants()
    .filter_map(|container| container.as_stack().cloned());

  for stack in stacks {
    let Some((hwnd, rect)) = state
      .tab_bars
      .get(&stack.id())
      .and_then(|bar| Some((bar.hwnd(), bar.shown_rect()?)))
    else {
      continue;
    };

    let active = stack.active_child().map(|child| child.id().as_u128());
    let item = items
      .iter_mut()
      .map(|(item, _)| item)
      .find(|item| Some(item.key) == active);

    // Its height stays as its window resizes beneath or above it.
    #[allow(clippy::cast_precision_loss)]
    let fit = match &item {
      Some(item) if resizing.contains(&item.key) => {
        let height = rect.height() as f32;
        let is_below = stack.tab_bar_position() == TabBarPosition::Bottom
          || rect.top >= item.frame.bottom;
        SceneFit::Frame {
          left: height,
          top: if is_below { 0.0 } else { height },
          right: height,
          bottom: if is_below { height } else { 0.0 },
        }
      }
      _ => SceneFit::Stretch,
    };

    let bar = SceneWindow {
      hwnd,
      placement: Some((rect.clone(), rect.clone())),
      alpha: None,
      corner_radius: 0.0,
      backdrop: None,
      required: false,
      fit,
    };

    match item {
      Some(item) => item.windows.push(bar),
      None => items.push((
        SceneItem {
          key: stack.id().as_u128(),
          frame: rect,
          windows: vec![bar],
        },
        None,
      )),
    }
  }

  items
}

/// The item a scene draws `window` as, placed at `placement` (its window
/// rect and visible frame), with the effects it has focused or not, and
/// its border cut to resize with it if `is_resizing`.
fn window_item(
  state: &WmState,
  config: &UserConfig,
  window: &WindowContainer,
  placement: (Rect, Rect),
  is_focused: bool,
  scale_factor: f32,
  is_resizing: bool,
) -> SceneItem {
  let id = window.id();
  let (outer, frame) = placement;

  // Drawn whether or not its overlay is on screen yet, e.g. for a column
  // scrolling into view.
  let backdrop =
    backdrop_overlay_params_for(is_focused, config).map(|params| {
      SceneBackdrop {
        overlay: state
          .backdrop_overlays
          .get(&id)
          .map_or(HWND(0), Overlay::hwnd),
        params,
      }
    });

  let mut windows = vec![SceneWindow {
    hwnd: window.native().hwnd(),
    placement: Some((outer, frame.clone())),
    alpha: intended_alpha(config, is_focused),
    corner_radius: corner_radius(window, is_focused, config, scale_factor),
    backdrop,
    required: true,
    fit: SceneFit::Stretch,
  }];

  let border = state.border_overlays.get(&id).zip(
    border_overlay_params_for(is_focused, config)
      .filter(|_| !border_hidden_for_fullscreen(window, config)),
  );
  if let Some((overlay, params)) = border {
    #[allow(clippy::cast_possible_truncation)]
    let ring = crate::tab_bars::window_with_tab_bar(window, &frame)
      .inset(-(params.width.round() as i32));

    // Its rounded corners stay as they are.
    let corner =
      (params.corner_radius + params.width) * scale_factor + 2.0;
    let fit = if is_resizing {
      SceneFit::Frame {
        left: corner,
        top: corner,
        right: corner,
        bottom: corner,
      }
    } else {
      SceneFit::Stretch
    };

    windows.push(SceneWindow {
      hwnd: overlay.hwnd(),
      placement: Some((ring.clone(), ring)),
      alpha: None,
      corner_radius: 0.0,
      backdrop: None,
      required: false,
      fit,
    });
  }

  SceneItem {
    key: id.as_u128(),
    frame,
    windows,
  }
}

/// A scene playing the close of `window`, as it is on screen now, if its
/// close animates.
pub fn close_request(
  state: &WmState,
  config: &UserConfig,
  window: &WindowContainer,
) -> Option<SceneCloseRequest> {
  let close = &config.value.animations.window_close;
  let params = WindowTransitionParams::from_close(close);
  let is_still = params.style == WindowTransitionStyle::None
    && params.away_opacity >= 1.0;
  if !close.enabled || is_still || !NativeScene::is_supported() {
    return None;
  }

  let monitor = window.monitor()?;
  let wearing_focus =
    state.prev_effects_window.as_ref().map(CommonGetters::id);

  let mut item = window_item(
    state,
    config,
    window,
    actual_placement(window)?,
    wearing_focus == Some(window.id()),
    monitor.native_properties().scale_factor,
    false,
  );

  // Its stack's tab bar stays with the stack.
  let hwnd = window.native().hwnd();
  let border = state.border_overlays.get(&window.id()).map(Overlay::hwnd);
  item
    .windows
    .retain(|copy| copy.hwnd == hwnd || Some(copy.hwnd) == border);

  Some(SceneCloseRequest {
    window_id: window.id(),
    handle: hwnd.0,
    monitor_id: monitor.id(),
    item,
    params,
  })
}

/// Whether `a` and `b` look alike once moved as far as `b`'s frame is
/// from `a`'s, ignoring how their copies are cut to resize if
/// `ignores_fit`.
fn looks_alike(a: &SceneItem, b: &SceneItem, ignores_fit: bool) -> bool {
  let (dx, dy) = (b.frame.left - a.frame.left, b.frame.top - a.frame.top);
  let moved = |rect: &Rect| {
    rect.translate_to_coordinates(rect.left + dx, rect.top + dy)
  };

  a.key == b.key
    && is_same_size(&a.frame, &b.frame)
    && a.windows.len() == b.windows.len()
    && a.windows.iter().zip(&b.windows).all(|(a, b)| {
      let a = SceneWindow {
        placement: a
          .placement
          .as_ref()
          .map(|(outer, frame)| (moved(outer), moved(frame))),
        fit: if ignores_fit { b.fit } else { a.fit },
        ..a.clone()
      };
      a == *b
    })
}

/// The windows of `workspace` its layout draws: all but minimized ones and
/// a stack's hidden tabs.
fn drawn_windows(
  workspace: &Workspace,
) -> impl Iterator<Item = WindowContainer> {
  workspace
    .descendants()
    .filter_map(|container| container.as_window_container().ok())
    .filter(is_drawn)
}

fn is_drawn(window: &WindowContainer) -> bool {
  window.state() != WindowState::Minimized
    && !is_inactive_stack_child(window)
}

fn is_focused(state: &WmState, window: &WindowContainer) -> bool {
  state
    .focused_container()
    .is_some_and(|c| c.id() == window.id())
}

/// The monitor `window_id` is on.
fn window_monitor(state: &WmState, window_id: Uuid) -> Option<Uuid> {
  Some(state.container_by_id(window_id)?.monitor()?.id())
}

/// Where the layout puts `window`'s window rect, the invisible resize
/// border included.
fn layout_rect(window: &WindowContainer) -> Option<Rect> {
  Some(
    window
      .to_rect()
      .ok()?
      .apply_delta(&window.total_border_delta().ok()?, None),
  )
}

/// Where the layout puts `window`: its window rect and visible frame.
fn layout_placement(window: &WindowContainer) -> Option<(Rect, Rect)> {
  let outer = layout_rect(window)?;
  let frame = frame_of(window, &outer);
  Some((outer, frame))
}

/// Where `window` is: its window rect and visible frame.
fn actual_placement(window: &WindowContainer) -> Option<(Rect, Rect)> {
  let native = window.native();
  Some((native.frame_with_shadows().ok()?, native.frame().ok()?))
}

/// The visible frame of `window` at window rect `outer`.
fn frame_of(window: &WindowContainer, outer: &Rect) -> Rect {
  outer.apply_delta(
    &window.native_properties().shadow_borders.inverse(),
    None,
  )
}

/// Where `window`, whose target was `previous`, is drawn now: where it is,
/// or where a peek draws its column.
fn drawn_frame(
  state: &WmState,
  window: &WindowContainer,
  previous: &Rect,
) -> Rect {
  if state
    .animation_manager
    .scroll_view
    .draws_window(&window.id())
  {
    return frame_of(window, previous);
  }

  window
    .native()
    .frame()
    .unwrap_or_else(|_| frame_of(window, previous))
}

fn is_same_size(a: &Rect, b: &Rect) -> bool {
  a.width() == b.width() && a.height() == b.height()
}

/// How far apart `a` and `b` are, edge by edge.
fn distance(a: &Rect, b: &Rect) -> i32 {
  (a.x() - b.x()).abs()
    + (a.y() - b.y()).abs()
    + (a.width() - b.width()).abs()
    + (a.height() - b.height()).abs()
}

/// The constant alpha the WM gives a window focused or not, or `None` if
/// it leaves windows' transparency to them.
fn intended_alpha(config: &UserConfig, is_focused: bool) -> Option<u8> {
  let effects = &config.value.window_effects;
  let is_managed = effects.focused_window.transparency.enabled
    || effects.other_windows.transparency.enabled;

  is_managed.then(|| {
    if is_focused {
      effects.focused_window.transparency.alpha()
    } else {
      effects.other_windows.transparency.alpha()
    }
  })
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
    EasingFunction, WindowTransitionParams, WindowTransitionStyle,
    WorkspaceSwitchAnimationConfig, WorkspaceSwitchDirection,
    WorkspaceSwitchStyle,
  };
  use wm_platform::{Rect, SceneValue};

  use super::{
    away, away_from, entry_motion, Drawn, GroupMotion, ItemMotion, Pose,
    MIN_SCALE, OPEN_PAINT_GRACE,
  };
  use crate::pending_sync::SceneSwitchRequest;

  const LINEAR: EasingFunction =
    EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0);

  fn request(direction: i32) -> SceneSwitchRequest {
    SceneSwitchRequest {
      outgoing: uuid::Uuid::new_v4(),
      incoming: uuid::Uuid::new_v4(),
      direction,
      axis: WorkspaceSwitchDirection::Horizontal,
    }
  }

  fn fixed(value: &SceneValue) -> f32 {
    match value {
      SceneValue::Fixed(value) => *value,
      SceneValue::Moving(motion) => motion.end,
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
      easing: LINEAR,
    };

    let halfway = motion.pose_at(started + Duration::from_millis(100));
    assert!((halfway.x + 500.0).abs() < 1.0);
    assert_eq!(
      motion.pose_at(started + Duration::from_millis(250)),
      motion.to
    );
  }

  #[test]
  fn a_resize_stretches_the_item_from_where_it_was() {
    let now = Instant::now();
    let from = Drawn::at_rest(Rect::from_xy(0, 0, 400, 300));
    let to = Drawn::at_rest(Rect::from_xy(100, 0, 800, 300));
    let motion = ItemMotion::new(from, to.clone(), 200, LINEAR, now);

    // Built where it rests, it starts offset back and half as wide.
    let pose = motion.pose(&to.frame);
    let SceneValue::Moving(offset_x) = &pose.offset_x else {
      panic!("it moves");
    };
    assert!((offset_x.segments[0].coefficients[0] + 100.0).abs() < 1e-3);
    let SceneValue::Moving(scale_x) = &pose.scale_x else {
      panic!("it stretches");
    };
    assert!((scale_x.segments[0].coefficients[0] - 0.5).abs() < 1e-3);
    assert!((fixed(&pose.scale_y) - 1.0).abs() < f32::EPSILON);

    let halfway = motion.drawn_at(now + Duration::from_millis(100));
    assert!((halfway.frame.width() - 600).abs() <= 1);
  }

  #[test]
  fn retargeting_starts_from_where_it_is() {
    let now = Instant::now();
    let motion = ItemMotion::new(
      Drawn::at_rest(Rect::from_xy(0, 0, 400, 300)),
      Drawn::at_rest(Rect::from_xy(1000, 0, 400, 300)),
      200,
      LINEAR,
      now,
    );

    let later = now + Duration::from_millis(100);
    let back = Drawn::at_rest(Rect::from_xy(0, 0, 400, 300));
    let turned = motion.retargeted(back, 200, LINEAR, later);
    assert!((turned.from.frame.x() - 500).abs() <= 1);
    assert!(!turned.is_done(later));
  }

  #[test]
  fn a_new_window_stays_hidden_until_it_has_painted() {
    let now = Instant::now();
    let rest = Rect::from_xy(100, 100, 400, 300);
    let params = WindowTransitionParams {
      duration_ms: 150,
      easing: LINEAR,
      style: WindowTransitionStyle::Zoom,
      away_opacity: 1.0,
    };

    let motion = entry_motion(&rest, &params, now);
    let early = motion.drawn_at(now + OPEN_PAINT_GRACE / 2);
    assert!(early.opacity.abs() < f32::EPSILON);
    assert!((early.zoom - MIN_SCALE).abs() < f32::EPSILON);

    // DWM holds it hidden too, then grows it from nothing.
    let pose = motion.pose(&rest);
    let SceneValue::Moving(opacity) = &pose.opacity else {
      panic!("it fades in");
    };
    assert!(opacity.segments[0].coefficients[0].abs() < f32::EPSILON);
    assert_eq!(opacity.segments[0].begin_secs, 0.0);
    assert!(
      opacity.segments[1].begin_secs >= OPEN_PAINT_GRACE.as_secs_f64()
    );

    let end = now + OPEN_PAINT_GRACE + Duration::from_millis(150);
    assert!(motion.is_done(end));
    assert_eq!(motion.drawn_at(end), Drawn::at_rest(rest));
  }

  #[test]
  fn closing_windows_slide_off_their_edge() {
    let rest = Rect::from_xy(100, 100, 400, 300);
    let params = WindowTransitionParams {
      duration_ms: 150,
      easing: LINEAR,
      style: WindowTransitionStyle::SlideLeft,
      away_opacity: 0.0,
    };

    let away = away_from(&rest, &params);
    assert_eq!(away.frame, Rect::from_xy(-300, 100, 400, 300));
    assert!(away.opacity.abs() < f32::EPSILON);
    assert!((away.zoom - 1.0).abs() < f32::EPSILON);
  }
}
