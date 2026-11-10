//! What a scrolling layout shows of its columns besides the real windows.
//!
//! At rest, columns show their real windows, also where they run off a
//! monitor edge with no other monitor behind it. Only a column that would
//! reach onto another monitor is cloaked, and its visible part is shown
//! by a `WindowPeek`: a click-through thumbnail clipped to the monitor.
//!
//! While the strip moves (a scroll, or columns opening, closing or
//! resizing across the monitor edge), every moving column is drawn by a
//! peek and its real window is cloaked, like the workspace-switch slide:
//! the peeks go up first, too faint to see, one composited frame later
//! they take over from the real windows as those are cloaked, and all of
//! them move in lock-step, their borders, backdrops and tab bars with
//! them. At the end, columns shown at rest get their real window back,
//! ones reaching onto another monitor keep their peek, and the rest are
//! dropped. Both swaps hide a semi-transparent real window by its alpha
//! first (see `WindowPeek::take_over`), so no frame shows a window twice
//! or not at all.
//!
//! As for a workspace switch, a peek spans its monitor and never moves:
//! only its thumbnail does, along with copies of the window's border ring,
//! shadow and backdrop, in the same DWM updates. A window whose backdrop
//! can't be copied (rounded, or drifting with parallax) has its peek move
//! with it instead (see `WindowPeek`).
//!
//! A peek's border, backdrop and tab bar are cut at the monitor edge like
//! the peek itself, so nothing of a column shows on the monitor beside it.

use std::collections::HashSet;

use uuid::Uuid;
use wm_common::{DisplayState, Layout, WindowState};
use wm_platform::{Rect, SurrogateBatch};

use super::peek_view::{track_overlays, PeekView, Role};
use crate::{
  layout::is_hidden_in_layout,
  models::{is_inactive_stack_child, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Starts moving the columns of displayed scrolling layouts whose rects
/// changed, and updates the peeks of the ones at rest. Columns land at
/// once unless the sync `animates`.
///
/// Runs before `platform_sync` repositions anything: moving windows are
/// then frozen (cloaked and pre-positioned at their target) behind peeks
/// that are already on screen.
pub fn sync_scroll_view(
  state: &mut WmState,
  config: &UserConfig,
  animates: bool,
) {
  let workspaces = displayed_scrolling_workspaces(state);
  let windows = workspaces
    .iter()
    .flat_map(|(workspace, clip)| {
      tiling_windows(workspace)
        .into_iter()
        .map(move |window| (window, clip.clone()))
    })
    .collect::<Vec<_>>();
  let focused_id = state.focused_container().map(|focused| focused.id());

  // Windows the WM hasn't placed yet, i.e. just opened.
  let new_ids = windows
    .iter()
    .map(|(window, _)| window.id())
    .filter(|id| !state.window_target_positions.contains_key(id))
    .collect::<HashSet<_>>();

  let now = state.animation_manager.frame_time();
  let view = &mut state.animation_manager.peeks;

  // Windows that left the displayed scrolling layouts lose their peeks;
  // their real windows are handled like any other window's. One the
  // scroll view held cloaked is redrawn, which shows or hides it for good.
  //
  // Their peeks stay up until any that take over are on screen, e.g. that
  // of a stack's newly active tab, so the column never goes blank.
  let gone =
    view.take_departed(|id| windows.iter().any(|(w, _)| w.id() == *id));

  let scroll_config = &config.value.animations.scroll;
  let mut needs_flush = false;
  let mut adds_peek = false;
  let mut started_moving = Vec::new();
  let mut batch = SurrogateBatch::new();

  for (workspace, clip) in &workspaces {
    let moves = column_moves(view, workspace, &new_ids);

    // The strip moves as one when any column crosses the monitor edge,
    // is already drawn by a peek, or opens; moves that stay on screen are
    // left to the regular per-window animations.
    let crosses_edge = |rect: &Rect| !clip.contains_rect(rect);
    let strip_moves = scroll_config.enabled
      && animates
      && moves.iter().any(|(window, from, target)| {
        from != target
          && (crosses_edge(from)
            || crosses_edge(target)
            || view.entries.contains_key(&window.id())
            || new_ids.contains(&window.id()))
      });

    for (window, from, target) in moves {
      view.last_rects.insert(window.id(), target.clone());

      let ends_shown = !is_hidden_in_layout(&window);
      let is_on_screen = |rect: &Rect| rect.intersection_area(clip) > 0;

      // Mid-move or mid-hand-back with nowhere new to go: leave it be.
      // Retargeting it anyway would restart its move every sync.
      if view.owned_target(&window.id()) == Some(&target) {
        continue;
      }

      let role = if strip_moves && from != target {
        Role::Moving { ends_shown }
      } else if ends_shown {
        // At rest on screen: the real window shows it, once a peek still
        // covering it after a move lets go.
        if !view.is_releasing(&window.id()) {
          view.entries.remove(&window.id());
        }
        continue;
      } else if is_on_screen(&target) {
        Role::Peek
      } else {
        view.entries.remove(&window.id());
        continue;
      };

      // Nothing to draw. A window held cloaked keeps its peek though, to
      // be handed back at the end of its move: dropping it would leave
      // the real window cloaked, and its cloak event would then look
      // like the app hiding it.
      if !is_on_screen(&from)
        && !is_on_screen(&target)
        && !view.owns_window(&window.id())
      {
        view.entries.remove(&window.id());
        continue;
      }

      let is_moving = matches!(role, Role::Moving { .. });
      adds_peek |= !view.entries.contains_key(&window.id());
      needs_flush |= view.upsert(
        &window,
        &from,
        target,
        clip,
        role,
        config,
        focused_id == Some(window.id()),
        now,
        &mut batch,
      );

      if is_moving {
        started_moving.push(window.id());
      }
    }
  }

  let replaces_peek = adds_peek
    && gone
      .iter()
      .any(|(_, entry)| entry.peek.visible_rect().is_some());

  flush_overlays(state, config, batch, needs_flush || replaces_peek);

  for (id, entry) in gone {
    if let Some(window) =
      state.window_by_id(id).filter(|_| entry.role != Role::Peek)
    {
      state.pending_sync.queue_container_to_redraw(window);
    }
  }

  drive_moving(state, &started_moving);
}

/// Has the scroll view drive the columns that `started_moving`, on ticks
/// paced by their monitor's vblanks. Without those a scroll is evaluated
/// at whenever the WM thread gets to each tick, so it advances unevenly.
fn drive_moving(state: &mut WmState, started_moving: &[Uuid]) {
  // Not their own animations any more.
  for id in started_moving {
    state.animation_manager.remove_animation(id);
  }

  let monitor = started_moving
    .first()
    .and_then(|id| state.window_by_id(*id))
    .and_then(|window| window.monitor());
  if let Some(monitor) = monitor {
    state
      .animation_manager
      .ensure_waiter_for(monitor.native_properties().handle);
  }

  if state.animation_manager.peeks.is_active() {
    state.animation_manager.ensure_timer_running();
  }
}

/// Tracks the overlays of the windows the scroll view draws (see
/// [`track_overlays`]) and, if `flushes`, has DWM composite them along
/// with the peeks.
///
/// Tracked before the flush, so it composites the copies drawn over the
/// rings and backdrops handed over too. Those then go at once, before
/// anything moves: hidden on the first tick of the move instead, a frame
/// could show them still at rest beside their moved copies.
fn flush_overlays(
  state: &mut WmState,
  config: &UserConfig,
  batch: SurrogateBatch,
  flushes: bool,
) {
  track_overlays(state, config, batch);

  if flushes {
    wm_platform::dwm_flush();
    state.animation_manager.peeks.note_composited();
    track_overlays(state, config, SurrogateBatch::new());
  }
}

/// Logs windows of displayed scrolling layouts that should be shown but
/// whose real window is still cloaked, once each, for diagnosing windows
/// that disappear. Only meaningful while nothing animates.
pub fn report_stuck_windows(state: &mut WmState) {
  let stuck = displayed_scrolling_workspaces(state)
    .iter()
    .flat_map(|(workspace, _)| tiling_windows(workspace))
    .filter(|window| {
      window.display_state() == DisplayState::Shown
        && !is_hidden_in_layout(window)
        && window.native().is_cloaked().unwrap_or(false)
    })
    .collect::<Vec<_>>();

  let view = &mut state.animation_manager.peeks;
  let stuck_ids =
    stuck.iter().map(CommonGetters::id).collect::<HashSet<_>>();
  view.reported_stuck.retain(|id| stuck_ids.contains(id));

  for window in stuck {
    if !view.reported_stuck.insert(window.id()) {
      continue;
    }

    let role = view.entries.get(&window.id()).map(|entry| entry.role);
    tracing::warn!(
      "Scrolling layout window stuck cloaked: {window}, rect {:?}, last \
       rect {:?}, scroll view role {role:?}.",
      target_rect(&window).ok(),
      view.last_rects.get(&window.id()),
    );
  }
}

/// Where each tiling window of `workspace` moves from and to: from where
/// it is shown now, or for a new column from its opening rect.
fn column_moves(
  view: &PeekView,
  workspace: &Workspace,
  new_ids: &HashSet<Uuid>,
) -> Vec<(WindowContainer, Rect, Rect)> {
  let moves = tiling_windows(workspace)
    .into_iter()
    .filter_map(|window| {
      let target = target_rect(&window).ok()?;
      let from = view
        .entries
        .get(&window.id())
        .map(|entry| entry.motion.current_rect())
        .or_else(|| view.last_rects.get(&window.id()).cloned())
        .unwrap_or_else(|| target.clone());

      Some((window, from, target))
    })
    .collect::<Vec<_>>();

  // A new column opens in place, widening as its neighbours make room,
  // rather than sliding in over them.
  moves
    .iter()
    .map(|(window, from, target)| {
      let from = if new_ids.contains(&window.id()) {
        opening_rect(target, &moves, new_ids)
      } else {
        from.clone()
      };
      (window.clone(), from, target.clone())
    })
    .collect()
}

/// Where a new column opens from: collapsed to no width at its place in
/// the strip as it was before, so it widens as its neighbours make room.
fn opening_rect(
  target: &Rect,
  moves: &[(WindowContainer, Rect, Rect)],
  new_ids: &HashSet<Uuid>,
) -> Rect {
  let existing = moves
    .iter()
    .filter(|(window, _, _)| !new_ids.contains(&window.id()));

  // How far the strip scrolls: what a column left of the new one moves
  // by, or one right of it minus the room it makes.
  let scroll = existing
    .clone()
    .find(|(_, _, to)| to.x() < target.x())
    .map(|(_, from, to)| from.x() - to.x())
    .or_else(|| {
      existing
        .clone()
        .find(|(_, _, to)| to.x() > target.x())
        .map(|(_, from, to)| from.x() - to.x() + target.width())
    })
    .unwrap_or(0);

  Rect::from_xy(target.x() + scroll, target.y(), 0, target.height())
}

/// Displayed scrolling workspaces, with the monitor area their peeks are
/// clipped to.
fn displayed_scrolling_workspaces(
  state: &WmState,
) -> Vec<(Workspace, Rect)> {
  state
    .workspaces()
    .into_iter()
    .filter(|workspace| {
      workspace.layout() == Layout::Scrolling && workspace.is_displayed()
    })
    .filter_map(|workspace| {
      let clip = workspace.monitor()?.native_properties().working_area;
      Some((workspace, clip))
    })
    .collect()
}

/// The workspace's tiling windows that are shown when on screen.
fn tiling_windows(workspace: &Workspace) -> Vec<WindowContainer> {
  workspace
    .descendants()
    .filter_map(|container| container.as_window_container().ok())
    .filter(|window| {
      window.state() == WindowState::Tiling
        && !is_inactive_stack_child(window)
    })
    .collect()
}

/// Where `platform_sync` puts the real window.
fn target_rect(window: &WindowContainer) -> anyhow::Result<Rect> {
  Ok(
    window
      .to_rect()?
      .apply_delta(&window.total_border_delta()?, None),
  )
}
