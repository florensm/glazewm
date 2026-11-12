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

use std::{collections::HashSet, time::Instant};

use uuid::Uuid;
use wm_common::{
  DisplayState, Layout, WindowState, WindowTransitionParams,
};
use wm_platform::{Rect, SurrogateBatch};

use super::{
  peek_view::{
    has_other_stand_in, is_live, is_peek_resize, is_pure_move,
    peek_styles, restore_cloaked, stretches_on_peeks, track_overlays,
    PeekView, Role,
  },
  transit::Transit,
};
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
#[allow(clippy::too_many_lines)]
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

  // Windows coming back from minimized, which re-enter as new ones do, by
  // `window_minimize`, behind the switch (`STAND_INS_PLAN.md` task 5).
  let restoring = column_restoring(config);
  let restored_ids = if restoring.is_some() {
    restored_columns(state, &windows, &new_ids)
  } else {
    HashSet::new()
  };

  // Behind the switch, a column that only moves does so on its peek, as
  // the strip does (`STAND_INS_PLAN.md` task 6), unless another stand-in
  // draws it.
  let pure_moves = column_pure_moves(config, animates);
  let other_stand_ins = match pure_moves {
    Some(_) => windows
      .iter()
      .map(|(window, _)| window.id())
      .filter(|id| has_other_stand_in(&state.animation_manager, id))
      .collect(),
    None => HashSet::new(),
  };

  // With `window_resize.style: "stretch"`, a column resized on screen goes
  // on its peek too, scaled (`STAND_INS_PLAN.md` task 8), where every
  // resized column can.
  let stretches = pure_moves.is_some() && stretches_on_peeks(config);
  let can_stretch = windows
    .iter()
    .filter(|_| stretches)
    .filter(|(window, _)| {
      can_stretch(state, window, &other_stand_ins, config)
    })
    .map(|(window, _)| window.id())
    .collect::<HashSet<_>>();

  // Columns a grab resize draws on peeks it holds (see
  // `grab::holds_on_peek`).
  let held = windows
    .iter()
    .map(|(window, _)| window)
    .filter(|window| crate::grab::holds_on_peek(window, state, config))
    .map(CommonGetters::id)
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
  let opening = column_opening(config);

  if scroll_config.enabled && animates {
    if let Some(params) = &restoring {
      turn_round_restored(view, &windows, &restored_ids, params, now);
    }
  }

  // Windows that open in place, widening as their neighbours make room:
  // new ones, and restored ones not already drawn.
  let opening_ids = new_ids
    .iter()
    .chain(restored_ids.iter().filter(|id| !view.draws_window(id)))
    .copied()
    .collect::<HashSet<_>>();
  let mut needs_flush = false;
  let mut adds_peek = false;
  let mut started_moving = Vec::new();
  let mut batch = SurrogateBatch::new();

  for (workspace, clip) in &workspaces {
    let moves = column_moves(view, workspace, &opening_ids);
    let stretches = stretches
      && moves.iter().all(|(window, from, target)| {
        opening_ids.contains(&window.id())
          || from.width() == target.width()
            && from.height() == target.height()
          || can_stretch.contains(&window.id())
      });

    // The strip moves as one when any column crosses the monitor edge,
    // is already drawn by a peek, or opens; moves that stay on screen are
    // left to the regular per-window animations. Behind the switch, only
    // resizes are, so a peek drawing one column doesn't take a resized
    // neighbour off its own (see `moves_on_peek`).
    let crosses_edge = |rect: &Rect| !clip.contains_rect(rect);
    let strip_moves = scroll_config.enabled
      && animates
      && moves.iter().any(|(window, from, target)| {
        from != target
          && (crosses_edge(from)
            || crosses_edge(target)
            || (pure_moves.is_none()
              && view.entries.contains_key(&window.id()))
            || opening_ids.contains(&window.id()))
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

      let is_held = held.contains(&window.id())
        && (from != target || view.is_held(&window.id()));
      let role = if is_held
        || moves_on_peek(
          &from,
          &target,
          strip_moves,
          pure_moves,
          pure_moves.filter(|_| stretches),
          view.entries.contains_key(&window.id()),
          other_stand_ins.contains(&window.id()),
        ) {
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
      let is_restored = restored_ids.contains(&window.id());
      adds_peek |= !view.entries.contains_key(&window.id());
      let entering = if new_ids.contains(&window.id()) {
        opening.as_ref()
      } else if is_restored {
        restoring.as_ref()
      } else {
        None
      };
      let transit = entering.and_then(Transit::opening);
      // Restored behind a cloak before its peek goes up, which then
      // needn't warm up under it.
      if is_moving && is_restored {
        restore_cloaked(&window.native(), &target);
      }
      needs_flush |= view.upsert(
        &window,
        &from,
        target,
        clip,
        role,
        transit.filter(|_| !is_held),
        stretches,
        is_held,
        config,
        focused_id == Some(window.id()),
        now,
        &mut batch,
      );
      if is_moving && is_restored {
        view.raise_moving(&window.id());
      }

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

/// The transition a new column enters by as it widens on the scroll's
/// timing: `window_open`, behind the switch (`STAND_INS_PLAN.md` task 3b).
fn column_opening(config: &UserConfig) -> Option<WindowTransitionParams> {
  let open_config = &config.value.animations.window_open;
  (wm_platform::isolation::current().peeks && open_config.enabled)
    .then(|| WindowTransitionParams::from_open(open_config))
}

/// The threshold a column's move has to pass to go on its peek alone,
/// behind the switch: the scroll's, as for its per-window animation (see
/// [`moves_on_peek`]).
fn column_pure_moves(config: &UserConfig, animates: bool) -> Option<u32> {
  let scroll_config = &config.value.animations.scroll;
  (wm_platform::isolation::current().peeks
    && scroll_config.enabled
    && animates)
    .then_some(scroll_config.threshold_px)
}

/// Whether a column moving from `from` to `target` is drawn moving by its
/// peek: as the strip moves as one, or, given `pure_moves`' threshold
/// (see [`column_pure_moves`]), as a peek draws it already, or as it only
/// moves (see [`is_pure_move`]) or, given `stretches`' threshold when the
/// strip stretches, resizes (see [`is_peek_resize`]), with no other
/// stand-in drawing it. Otherwise a column resized on screen keeps the
/// per-window path.
fn moves_on_peek(
  from: &Rect,
  target: &Rect,
  strip_moves: bool,
  pure_moves: Option<u32>,
  stretches: Option<u32>,
  is_drawn: bool,
  has_other_stand_in: bool,
) -> bool {
  let alone = |threshold_px| {
    !has_other_stand_in && is_pure_move(from, target, threshold_px)
  };
  let resized = |threshold_px| {
    !has_other_stand_in && is_peek_resize(from, target, threshold_px)
  };

  from != target
    && (strip_moves
      || pure_moves
        .is_some_and(|threshold_px| is_drawn || alone(threshold_px))
      || stretches.is_some_and(resized))
}

/// Whether `window`, a column, can be drawn stretched as it resizes on
/// its peek: not one another stand-in draws, nor one whose peek would be
/// `Live`, nor one at its new size already, as an interactive resize
/// leaves it, which keeps the per-window path as before.
fn can_stretch(
  state: &WmState,
  window: &WindowContainer,
  other_stand_ins: &HashSet<Uuid>,
  config: &UserConfig,
) -> bool {
  let is_moving = state.animation_manager.peeks.is_moving(&window.id());
  let is_resized_already =
    !is_moving
      && target_rect(window).is_ok_and(|target| {
        state.window_target_positions.get(&window.id()).is_some_and(
          |from| {
            (from.width(), from.height())
              != (target.width(), target.height())
          },
        ) && window.native().frame_with_shadows().is_ok_and(|actual| {
          (actual.width(), actual.height())
            == (target.width(), target.height())
        })
      });

  !other_stand_ins.contains(&window.id())
    && !is_live(&peek_styles(window, config))
    && !is_resized_already
}

/// The transition a column coming back from minimized enters by, as a new
/// one does by `window_open`: `window_minimize` reversed, behind the
/// switch (`STAND_INS_PLAN.md` task 5).
fn column_restoring(
  config: &UserConfig,
) -> Option<WindowTransitionParams> {
  let minimize_config = &config.value.animations.window_minimize;
  (wm_platform::isolation::current().peeks && minimize_config.enabled)
    .then(|| WindowTransitionParams::from_restore(minimize_config))
}

/// The windows of `windows` coming back from minimized this sync: not new
/// ones, which `new_ids` holds.
fn restored_columns(
  state: &WmState,
  windows: &[(WindowContainer, Rect)],
  new_ids: &HashSet<Uuid>,
) -> HashSet<Uuid> {
  windows
    .iter()
    .map(|(window, _)| window.id())
    .filter(|id| {
      !new_ids.contains(id) && state.pending_sync.is_window_restore(id)
    })
    .collect()
}

/// Turns round where they are the columns in `restored_ids` still
/// minimizing on their peeks, by `params`, their real windows restored
/// behind a cloak if still minimized. The strip then moves them on as any
/// moving column.
fn turn_round_restored(
  view: &mut PeekView,
  windows: &[(WindowContainer, Rect)],
  restored_ids: &HashSet<Uuid>,
  params: &WindowTransitionParams,
  now: Instant,
) {
  for (window, _) in windows {
    if restored_ids.contains(&window.id())
      && view.turn_round(&window.id(), params, true, now)
    {
      if let Ok(target) = target_rect(window) {
        restore_cloaked(&window.native(), &target);
      }
      view.raise_moving(&window.id());
    }
  }
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
/// it is shown now, or for a column in `new_ids` from its opening rect.
///
/// Not one leaving on its peek, which plays out where it was: a closing
/// one is out of the tree already, a minimizing one until the OS reports
/// it minimized.
fn column_moves(
  view: &PeekView,
  workspace: &Workspace,
  new_ids: &HashSet<Uuid>,
) -> Vec<(WindowContainer, Rect, Rect)> {
  let moves = tiling_windows(workspace)
    .into_iter()
    .filter(|window| !view.is_leaving(&window.id()))
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

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::moves_on_peek;

  fn column() -> Rect {
    Rect::from_xy(100, 50, 800, 600)
  }

  /// Without the switch, a column moves on its peek only with the strip,
  /// as before.
  #[test]
  fn without_the_switch_only_the_strip_moves_columns() {
    let moved = Rect::from_xy(900, 50, 800, 600);
    let resized = Rect::from_xy(100, 50, 400, 600);
    for to in [&moved, &resized] {
      assert!(moves_on_peek(
        &column(),
        to,
        true,
        None,
        None,
        false,
        false
      ));
      assert!(!moves_on_peek(
        &column(),
        to,
        false,
        None,
        None,
        true,
        false
      ));
    }
    assert!(!moves_on_peek(
      &column(),
      &column(),
      true,
      None,
      None,
      true,
      false
    ));
  }

  /// Behind the switch, a column that only moves goes on its peek alone,
  /// and one a peek draws stays on it; a resized one keeps the per-window
  /// path, as does a move below the threshold or one another stand-in
  /// draws.
  #[test]
  fn behind_the_switch_pure_column_moves_go_on_peeks() {
    let threshold = Some(10);
    let moved = Rect::from_xy(900, 50, 800, 600);
    let nudged = Rect::from_xy(105, 50, 800, 600);
    let resized = Rect::from_xy(100, 50, 400, 600);

    assert!(moves_on_peek(
      &column(),
      &moved,
      false,
      threshold,
      None,
      false,
      false
    ));
    assert!(!moves_on_peek(
      &column(),
      &moved,
      false,
      threshold,
      None,
      false,
      true
    ));
    assert!(!moves_on_peek(
      &column(),
      &nudged,
      false,
      threshold,
      None,
      false,
      false
    ));
    assert!(!moves_on_peek(
      &column(),
      &resized,
      false,
      threshold,
      None,
      false,
      false
    ));
    assert!(moves_on_peek(
      &column(),
      &resized,
      false,
      threshold,
      None,
      true,
      false
    ));
    assert!(moves_on_peek(
      &column(),
      &resized,
      true,
      threshold,
      None,
      false,
      true
    ));
    assert!(!moves_on_peek(
      &column(),
      &column(),
      true,
      threshold,
      None,
      true,
      false
    ));
  }

  /// With `stretch`, a column resized on screen past the threshold goes on
  /// its peek too, unless another stand-in draws it; a smaller resize, or
  /// one without `stretch`, doesn't.
  #[test]
  fn stretched_column_resizes_go_on_peeks() {
    let threshold = Some(10);
    let resized = Rect::from_xy(100, 50, 400, 600);
    let nudged = Rect::from_xy(100, 50, 795, 600);

    assert!(moves_on_peek(
      &column(),
      &resized,
      false,
      threshold,
      threshold,
      false,
      false
    ));
    assert!(!moves_on_peek(
      &column(),
      &resized,
      false,
      threshold,
      threshold,
      false,
      true
    ));
    assert!(!moves_on_peek(
      &column(),
      &nudged,
      false,
      threshold,
      threshold,
      false,
      false
    ));
    assert!(!moves_on_peek(
      &column(),
      &resized,
      false,
      None,
      None,
      false,
      false
    ));
  }
}
