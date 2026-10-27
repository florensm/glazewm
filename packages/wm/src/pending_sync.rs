use std::collections::{HashMap, HashSet};

use uuid::Uuid;
use wm_common::WorkspaceSwitchDirection;

use crate::{
  models::{Container, Workspace},
  traits::CommonGetters,
};

/// A pending workspace switch for a scene to play (see
/// `animation::scene_switch`), requested by `focus_workspace`.
// LINT: Only played on Windows.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct SceneSwitchRequest {
  /// The workspace leaving its monitor.
  pub outgoing: Uuid,
  /// The workspace taking its place.
  pub incoming: Uuid,
  /// `+1` if `incoming` comes after `outgoing` on their monitor, `-1` if
  /// before.
  pub direction: i32,
  pub axis: WorkspaceSwitchDirection,
}

/// A window closing on a scene (see `animation::scene`): its copy plays
/// the close once the WM has let go of it, and the window is then sent
/// `WM_CLOSE`.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub struct SceneCloseRequest {
  pub window_id: Uuid,

  /// Its handle, which `WM_CLOSE` is sent to.
  pub handle: isize,
  pub monitor_id: Uuid,

  /// Its copy as the close starts.
  pub item: wm_platform::SceneItem,
  pub params: wm_common::WindowTransitionParams,
}

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PendingSync {
  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: HashMap<Uuid, Container>,

  /// Workspaces where z-order should be updated. Windows that match the
  /// focused window's state should be brought to the front.
  workspaces_to_reorder: Vec<Workspace>,

  /// Whether native focus should be reassigned to the WM's focused
  /// container.
  needs_focus_update: bool,

  /// Whether window effect for the focused window should be updated.
  needs_focused_effect_update: bool,

  /// Whether window effects for all windows should be updated.
  needs_all_effects_update: bool,

  /// Whether to jump the cursor to the focused container (if enabled in
  /// user config).
  needs_cursor_jump: bool,

  /// Whether to move the cursor to the focused window, as asked for by
  /// `move-cursor`. Done once the layout is settled, so it lands where
  /// the window ends up (e.g. after a scrolling layout scrolls).
  needs_cursor_move: bool,

  /// Whether stack tab bars need redrawing and restacking for a change
  /// that doesn't move any window (e.g. a tab's urgency, or the end of a
  /// drag that raised the stack's window).
  needs_tab_bar_update: bool,

  /// Window IDs that just underwent a tiling/floating state change this
  /// sync cycle. Used by `platform_sync` to allow `window_move`
  /// animations across the state boundary (e.g. tiling → floating or
  /// floating → tiling).
  window_state_changes: HashSet<Uuid>,

  /// Window IDs whose fullscreen state was changed by a WM command this
  /// sync cycle (`set-fullscreen` / `toggle-fullscreen`), as opposed to
  /// the app driving itself fullscreen.
  ///
  /// Gates the one exception to the blanket "never animate fullscreen
  /// windows" rule. That rule exists because exclusive-fullscreen games
  /// mode-set the display, and covering one with a scene kicks it out of
  /// fullscreen, reverts the resolution and re-triggers the relayout in a
  /// loop. An app driving itself fullscreen is detected
  /// in `handle_window_moved_or_resized` and routed through
  /// `update_window_state` -- the same path a WM command takes -- so
  /// `window_state_changes` alone cannot tell the two apart. Only the
  /// command path sets this, so a mode-setting game never reaches the
  /// animated path.
  wm_fullscreen_toggles: HashSet<Uuid>,

  /// Pending workspace switch for a scene to play, consumed by
  /// `platform_sync` to cover the monitor before the real windows are
  /// switched.
  scene_switch: Option<SceneSwitchRequest>,

  /// Windows closing on scenes, consumed by `platform_sync` to cover
  /// their monitors while their copies are still on screen.
  #[cfg(target_os = "windows")]
  scene_closes: Vec<SceneCloseRequest>,

  /// Whether animations should be skipped for this sync cycle.
  ///
  /// Set on display setting changes: animating a relayout caused by a
  /// resolution or working-area change is visually meaningless, and a
  /// scene covering the monitor would kick exclusive-fullscreen games out
  /// of fullscreen (reverting the resolution and re-triggering the
  /// event in a loop). In-flight animations of redrawn windows are
  /// cancelled and their windows snap to their target rect. Also set for
  /// a workspace switch played on a scene, beneath which it happens at
  /// once.
  animations_suppressed: bool,
}

impl PendingSync {
  pub fn has_changes(&self) -> bool {
    !self.containers_to_redraw.is_empty()
      || !self.workspaces_to_reorder.is_empty()
      || self.needs_focus_update
      || self.needs_focused_effect_update
      || self.needs_all_effects_update
      || self.needs_cursor_jump
      || self.needs_cursor_move
      || self.needs_tab_bar_update
      || self.scene_switch.is_some()
      || self.has_scene_closes()
  }

  #[cfg(target_os = "windows")]
  fn has_scene_closes(&self) -> bool {
    !self.scene_closes.is_empty()
  }

  #[cfg(not(target_os = "windows"))]
  #[allow(clippy::unused_self)]
  fn has_scene_closes(&self) -> bool {
    false
  }

  pub fn clear(&mut self) -> &mut Self {
    self.containers_to_redraw.clear();
    self.workspaces_to_reorder.clear();
    self.needs_focus_update = false;
    self.needs_focused_effect_update = false;
    self.needs_all_effects_update = false;
    self.needs_cursor_jump = false;
    self.needs_cursor_move = false;
    self.needs_tab_bar_update = false;
    self.window_state_changes.clear();
    self.wm_fullscreen_toggles.clear();
    self.scene_switch = None;
    #[cfg(target_os = "windows")]
    self.scene_closes.clear();
    self.animations_suppressed = false;
    self
  }

  pub fn queue_container_to_redraw<T>(&mut self, container: T) -> &mut Self
  where
    T: Into<Container>,
  {
    let container: Container = container.into();
    self.containers_to_redraw.insert(container.id(), container);
    self
  }

  pub fn queue_containers_to_redraw<I, T>(
    &mut self,
    containers: I,
  ) -> &mut Self
  where
    I: IntoIterator<Item = T>,
    T: Into<Container>,
  {
    for container in containers {
      let container: Container = container.into();
      self.containers_to_redraw.insert(container.id(), container);
    }

    self
  }

  pub fn dequeue_container_from_redraw<T>(
    &mut self,
    container: T,
  ) -> &mut Self
  where
    T: Into<Container>,
  {
    self.containers_to_redraw.remove(&container.into().id());
    self
  }

  pub fn queue_workspace_to_reorder(
    &mut self,
    workspace: Workspace,
  ) -> &mut Self {
    self.workspaces_to_reorder.push(workspace);
    self
  }

  pub fn queue_focus_change(&mut self) -> &mut Self {
    self.needs_focus_update = true;
    self
  }

  pub fn queue_focused_effect_update(&mut self) -> &mut Self {
    self.needs_focused_effect_update = true;
    self
  }

  pub fn queue_all_effects_update(&mut self) -> &mut Self {
    self.needs_all_effects_update = true;
    self
  }

  pub fn queue_cursor_jump(&mut self) -> &mut Self {
    self.needs_cursor_jump = true;
    self
  }

  pub fn queue_cursor_move(&mut self) -> &mut Self {
    self.needs_cursor_move = true;
    self
  }

  pub fn queue_tab_bar_update(&mut self) -> &mut Self {
    self.needs_tab_bar_update = true;
    self
  }

  pub fn needs_focus_update(&self) -> bool {
    self.needs_focus_update
  }

  pub fn needs_focused_effect_update(&self) -> bool {
    self.needs_focused_effect_update
  }

  pub fn needs_all_effects_update(&self) -> bool {
    self.needs_all_effects_update
  }

  pub fn needs_cursor_jump(&self) -> bool {
    self.needs_cursor_jump
  }

  pub fn needs_cursor_move(&self) -> bool {
    self.needs_cursor_move
  }

  // LINT: Only read by Windows-only sync steps.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub fn needs_tab_bar_update(&self) -> bool {
    self.needs_tab_bar_update
  }

  pub fn containers_to_redraw(&self) -> &HashMap<Uuid, Container> {
    &self.containers_to_redraw
  }

  pub fn workspaces_to_reorder(&self) -> &Vec<Workspace> {
    &self.workspaces_to_reorder
  }

  /// Marks a window as having just changed tiling/floating state this
  /// cycle.
  pub fn mark_window_state_change(&mut self, id: Uuid) -> &mut Self {
    self.window_state_changes.insert(id);
    self
  }

  /// Returns `true` if the window changed tiling/floating state this
  /// cycle.
  pub fn is_window_state_change(&self, id: &Uuid) -> bool {
    self.window_state_changes.contains(id)
  }

  /// Marks a window's fullscreen state as changed by a WM command, rather
  /// than by the app driving itself fullscreen.
  pub fn mark_wm_fullscreen_toggle(&mut self, id: Uuid) -> &mut Self {
    self.wm_fullscreen_toggles.insert(id);
    self
  }

  /// Returns `true` if a WM command changed this window's fullscreen state
  /// this cycle.
  pub fn is_wm_fullscreen_toggle(&self, id: &Uuid) -> bool {
    self.wm_fullscreen_toggles.contains(id)
  }

  /// Requests a workspace switch played on a scene for this sync cycle.
  ///
  /// Animations of single windows are suppressed for it: beneath the
  /// scene, the switch happens as with animations off.
  pub fn request_scene_switch(
    &mut self,
    request: SceneSwitchRequest,
  ) -> &mut Self {
    self.scene_switch = Some(request);
    self.suppress_animations()
  }

  /// Consumes and returns the pending scene-played switch, if any.
  // LINT: Only read by Windows-only sync steps.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub fn take_scene_switch(&mut self) -> Option<SceneSwitchRequest> {
    self.scene_switch.take()
  }

  /// Queues a window's close for a scene to play.
  #[cfg(target_os = "windows")]
  pub fn queue_scene_close(
    &mut self,
    request: SceneCloseRequest,
  ) -> &mut Self {
    self.scene_closes.push(request);
    self
  }

  /// Consumes and returns the closes queued for scenes.
  #[cfg(target_os = "windows")]
  pub fn take_scene_closes(&mut self) -> Vec<SceneCloseRequest> {
    std::mem::take(&mut self.scene_closes)
  }

  /// Suppresses animations for this sync cycle (see
  /// `animations_suppressed` field docs).
  pub fn suppress_animations(&mut self) -> &mut Self {
    self.animations_suppressed = true;
    self
  }

  /// Returns `true` if animations should be skipped for this sync cycle.
  pub fn animations_suppressed(&self) -> bool {
    self.animations_suppressed
  }
}
