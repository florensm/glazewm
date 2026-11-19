//! Clicks on windows drawn moving by stand-ins: a scrolling layout's
//! moving columns, and the windows of a sliding workspace switch.
//!
//! Their real windows are cloaked meanwhile, and the stand-ins
//! click-through, so a click there would land on the desktop or a window
//! behind. `ClickGuard` keeps such a press from it instead, and it
//! focuses the window drawn under the cursor, as a compositor that hit
//! tests windows where it draws them would (e.g. niri). The click itself
//! doesn't reach the app: it can't be handed to a stand-in, and replaying
//! it once the window lands would need synthesized input.

use wm_common::{DisplayState, WindowState};
use wm_platform::{Point, Rect};

use crate::{
  commands::container::set_focused_descendant,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  wm_state::WmState,
};

/// Has `ClickGuard` keep presses on the windows drawn moving this frame,
/// or none once nothing moves. Called every animation frame, since the
/// guard lapses unless renewed.
pub fn sync_motion_guard(state: &mut WmState) {
  let areas = if state.is_paused || state.overview.is_shown() {
    Vec::new()
  } else {
    moving_areas(state)
  };

  let exempt = if areas.is_empty() {
    Vec::new()
  } else {
    live_windows_over_stand_ins(state)
  };

  state.click_guard.guard_motion(areas, exempt);
}

/// Focuses the window drawn moving at `point`, which a press there was
/// kept from (see [`sync_motion_guard`]). A press on a window sliding out
/// in a workspace switch only does nothing.
pub fn focus_moving_window_at(point: &Point, state: &mut WmState) {
  let Some(target) = moving_window_at(point, state) else {
    return;
  };

  let is_focused = state
    .focused_container()
    .is_some_and(|focused| focused.id() == target.id());

  if !is_focused {
    set_focused_descendant(&target.as_container(), None);
    state.pending_sync.queue_focus_change();
  }
}

/// Where windows are drawn moving this frame.
fn moving_areas(state: &WmState) -> Vec<Rect> {
  let manager = &state.animation_manager;

  manager
    .peeks
    .moving_rects()
    .map(|(_, rect)| rect.clone())
    .chain(
      manager
        .switching_rects()
        .into_iter()
        .map(|(_, _, rect)| rect),
    )
    .collect()
}

/// Rects of shown non-tiling windows that no stand-in draws, which sit
/// above the moving columns of a scrolling layout and keep their clicks.
fn live_windows_over_stand_ins(state: &WmState) -> Vec<Rect> {
  let manager = &state.animation_manager;

  state
    .windows()
    .into_iter()
    .filter(|window| {
      window.state() != WindowState::Tiling
        && window.display_state() == DisplayState::Shown
        && window.workspace().is_some_and(|ws| ws.is_displayed())
        && !manager.peeks.draws_window(&window.id())
        && !manager.is_in_workspace_switch(&window.id())
    })
    .filter_map(|window| window.to_rect().ok())
    .collect()
}

/// The window drawn moving at `point`: a moving column, or a window
/// sliding in with a workspace switch, floating ones over tiling ones.
fn moving_window_at(
  point: &Point,
  state: &WmState,
) -> Option<WindowContainer> {
  let manager = &state.animation_manager;
  let window_by_id = |id| {
    state
      .container_by_id(id)
      .and_then(|container| container.as_window_container().ok())
  };

  let column = manager
    .peeks
    .moving_rects()
    .find(|(_, rect)| rect.contains_point(point))
    .and_then(|(id, _)| window_by_id(id));

  if column.is_some() {
    return column;
  }

  // One sliding out is left alone: the switch is leaving it.
  let incoming = manager
    .switching_rects()
    .into_iter()
    .filter(|(_, is_incoming, rect)| {
      *is_incoming && rect.contains_point(point)
    })
    .filter_map(|(id, _, _)| window_by_id(id))
    .collect::<Vec<_>>();

  incoming
    .iter()
    .find(|window| window.state() != WindowState::Tiling)
    .or_else(|| incoming.first())
    .cloned()
}
