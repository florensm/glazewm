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
//! the peeks go up first, one composited frame later the real windows are
//! cloaked, and all of them move in lock-step, their borders, backdrops
//! and tab bars with them. At the end, columns shown at rest get their
//! real window back, ones reaching onto another monitor keep their peek,
//! and the rest are dropped.
//!
//! A peek's border, backdrop and tab bar are cut at the monitor edge like
//! the peek itself, so nothing of a column shows on the monitor beside it.

use std::{
  collections::{HashMap, HashSet},
  time::Instant,
};

use uuid::Uuid;
use wm_common::{DisplayState, Layout, WindowState};
use wm_platform::{
  NativeWindowWindowsExt, Overlay, Point, Rect, StyleBlend,
  SurrogateBatch, WindowPeek, HWND,
};

use super::{
  stand_in::{moved_along, StandIn},
  state::WindowAnimationState,
};
use crate::{
  commands::general::{
    surrogate_effects_for, upsert_clipped_overlay,
    upsert_pinned_border_overlay,
  },
  layout::is_hidden_in_layout,
  models::{is_inactive_stack_child, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// What a peek is currently doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
  /// Shows a partly visible column at rest.
  Peek,
  /// Draws a moving column while its real window is cloaked.
  Moving { ends_shown: bool },
  /// Moved into place; waits for the real window to be uncloaked.
  HandingBack,
}

struct Entry {
  peek: WindowPeek,
  motion: WindowAnimationState,
  /// Where the peek was last placed, before clipping.
  rect: Rect,
  /// Monitor area the peek is clipped to.
  clip: Rect,
  opacity: u8,
  role: Role,
}

impl Entry {
  fn place(&mut self, rect: &Rect) {
    self.peek.place(rect, &self.clip, self.opacity);
    self.rect = rect.clone();
  }
}

/// Columns whose move ended on a tick.
#[derive(Default)]
pub struct Landed {
  /// Shown at rest, whose real windows are to be uncloaked by the next
  /// `platform_sync`.
  pub shown: Vec<Uuid>,
  /// Hidden at rest, whose real windows are to be put where they stay
  /// cloaked by the next `platform_sync`.
  pub hidden: Vec<Uuid>,
}

/// Peeks of the columns of displayed scrolling layouts.
#[derive(Default)]
pub struct ScrollView {
  entries: HashMap<Uuid, Entry>,
  /// Last target rect of every window in a displayed scrolling layout,
  /// i.e. where it is shown now and moves from on the next relayout.
  last_rects: HashMap<Uuid, Rect>,
  /// Windows already reported by `report_stuck_windows`.
  reported_stuck: HashSet<Uuid>,
}

impl ScrollView {
  /// Whether `window_id`'s real window is cloaked while a peek draws it.
  pub fn owns_window(&self, window_id: &Uuid) -> bool {
    self.owned_target(window_id).is_some()
  }

  /// Whether a peek draws `window_id`, moving or at rest, along with its
  /// overlays.
  pub fn draws_window(&self, window_id: &Uuid) -> bool {
    self.entries.contains_key(window_id)
  }

  /// The peek drawing `window_id`, which its overlays sit behind.
  pub fn peek_hwnd(&self, window_id: &Uuid) -> Option<HWND> {
    self.entries.get(window_id).map(|entry| entry.peek.hwnd())
  }

  /// Where the peek drawing `window_id` has `rect`, a rect of the layout
  /// that moves with the window, e.g. its stack's.
  pub fn stand_in(
    &self,
    window_id: &Uuid,
    rect: &Rect,
  ) -> Option<StandIn> {
    let entry = self.entries.get(window_id)?;

    Some(StandIn {
      rect: moved_along(rect, &entry.motion.target_rect, &entry.rect),
      clip: Some(entry.clip.clone()),
      anchor: Some(entry.peek.hwnd()),
    })
  }

  /// Where `window_id`'s peek is headed while its real window is cloaked.
  fn owned_target(&self, window_id: &Uuid) -> Option<&Rect> {
    self
      .entries
      .get(window_id)
      .filter(|entry| entry.role != Role::Peek)
      .map(|entry| &entry.motion.target_rect)
  }

  /// Whether `window_id` is moving, i.e. its real window stays frozen.
  pub fn is_moving(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| matches!(entry.role, Role::Moving { .. }))
  }

  /// Whether anything still needs animation ticks.
  pub fn is_active(&self) -> bool {
    self.entries.values().any(|entry| entry.role != Role::Peek)
  }

  /// Advances moving columns to `now`, returning the ones that landed.
  pub fn tick(&mut self, now: Instant, clock_origin: Instant) -> Landed {
    let mut landed = Landed::default();

    for (id, entry) in &mut self.entries {
      let Role::Moving { ends_shown } = entry.role else {
        continue;
      };

      entry.motion.start_clock(clock_origin);
      let (rect, _) = entry.motion.current_state_at(now);
      entry.place(&rect);

      if entry.motion.is_complete() {
        entry.role = if ends_shown {
          landed.shown.push(*id);
          Role::HandingBack
        } else {
          landed.hidden.push(*id);
          Role::Peek
        };
      }
    }

    // Columns that moved out of view have nothing left to show.
    self.entries.retain(|_, entry| {
      entry.role != Role::Peek || entry.peek.visible_rect().is_some()
    });

    landed
  }

  /// Takes out the peeks of windows that `is_known` no longer knows, i.e.
  /// that left the displayed scrolling layouts, and forgets their rects.
  fn take_departed(
    &mut self,
    is_known: impl Fn(&Uuid) -> bool,
  ) -> Vec<(Uuid, Entry)> {
    self.last_rects.retain(|id, _| is_known(id));
    self.entries.extract_if(|id, _| !is_known(id)).collect()
  }

  /// Drops `window_id`'s peek once its real window has been uncloaked.
  ///
  /// Called right after the uncloak so both land in the same composited
  /// frame.
  pub fn release(&mut self, window_id: &Uuid) {
    if self
      .entries
      .get(window_id)
      .is_some_and(|entry| entry.role == Role::HandingBack)
    {
      self.entries.remove(window_id);
    }
  }

  /// The window whose peek is at `point`, if any.
  pub fn window_at(&self, point: &Point) -> Option<Uuid> {
    self.entries.iter().find_map(|(id, entry)| {
      entry
        .peek
        .visible_rect()
        .filter(|rect| rect.contains_point(point))
        .map(|_| *id)
    })
  }

  /// Draws `window` with a peek in `role`, going from `from` to `target`.
  ///
  /// Returns whether a new peek now covers a still-visible real window,
  /// so it has to be composited before that window is cloaked.
  #[allow(clippy::too_many_arguments)]
  fn upsert(
    &mut self,
    window: &WindowContainer,
    from: &Rect,
    target: Rect,
    clip: &Rect,
    role: Role,
    config: &UserConfig,
    is_focused: bool,
  ) -> bool {
    let scroll_config = &config.value.animations.scroll;
    let is_moving = matches!(role, Role::Moving { .. });
    let in_flight = self
      .entries
      .get(&window.id())
      .filter(|entry| matches!(entry.role, Role::Moving { .. }));

    // A column already moving carries on at its speed toward the new
    // target, so quick presses in a row glide rather than restart.
    let motion = match in_flight {
      Some(entry) if is_moving => entry.motion.retargeted(
        target,
        scroll_config.duration_ms,
        scroll_config.easing.clone(),
        Instant::now(),
      ),
      _ => WindowAnimationState::new_movement(
        from.clone(),
        target,
        if is_moving {
          scroll_config.duration_ms
        } else {
          0
        },
        scroll_config.easing.clone(),
      ),
    };
    // The window's own transparency and corners, as it is focused or not.
    let (opacity, corner_style, _, _) =
      surrogate_effects_for(is_focused, config);
    if !is_moving {
      motion.start_clock(Instant::now());
    }

    if let Some(entry) = self.entries.get_mut(&window.id()) {
      entry.motion = motion;
      entry.clip = clip.clone();
      entry.opacity = opacity;
      entry.role = role;
      let rect = entry.motion.current_rect();
      entry.place(&rect);
      return false;
    }

    // Sized like the window itself, which a collapsed `from` (an opening
    // column) isn't.
    let Ok(peek) = WindowPeek::new(
      &window.native(),
      &motion.target_rect,
      &corner_style,
    )
    .inspect_err(|err| {
      tracing::warn!("Failed to create a peek of {window}: {err}");
    }) else {
      return false;
    };

    let rect = motion.current_rect();
    let mut entry = Entry {
      peek,
      motion,
      rect: rect.clone(),
      clip: clip.clone(),
      opacity,
      role,
    };
    entry.place(&rect);
    self.entries.insert(window.id(), entry);

    is_moving && !window.native().is_cloaked().unwrap_or(true)
  }
}

/// Starts moving the columns of displayed scrolling layouts whose rects
/// changed, and updates the peeks of the ones at rest.
///
/// Runs before `platform_sync` repositions anything: moving windows are
/// then frozen (cloaked and pre-positioned at their target) behind peeks
/// that are already on screen.
pub fn sync_scroll_view(state: &mut WmState, config: &UserConfig) {
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

  let view = &mut state.animation_manager.scroll_view;

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

  for (workspace, clip) in &workspaces {
    let moves = column_moves(view, workspace, &new_ids);

    // The strip moves as one when any column crosses the monitor edge,
    // is already drawn by a peek, or opens; moves that stay on screen are
    // left to the regular per-window animations.
    let crosses_edge = |rect: &Rect| !clip.contains_rect(rect);
    let strip_moves = scroll_config.enabled
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
        // At rest on screen: the real window shows it.
        view.entries.remove(&window.id());
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

  if needs_flush || replaces_peek {
    wm_platform::dwm_flush();
  }

  for (id, entry) in gone {
    if let Some(window) = state
      .container_by_id(id)
      .and_then(|container| container.as_window_container().ok())
      .filter(|_| entry.role != Role::Peek)
    {
      state.pending_sync.queue_container_to_redraw(window);
    }
  }

  // The scroll view drives these windows now, not their own animations.
  for id in &started_moving {
    state.animation_manager.remove_animation(id);
  }

  track_overlays(state, config);

  if state.animation_manager.scroll_view.is_active() {
    state.animation_manager.ensure_timer_running();
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

  let view = &mut state.animation_manager.scroll_view;
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

/// Keeps the border and backdrop overlays of the windows the scroll view
/// draws on their peeks, cut at the monitor edge like the peeks are.
/// `sync_overlays` leaves these windows to it.
///
/// A moving peek's border is pinned to its monitor and slid within it, as
/// for a workspace switch, so the ring and where it is cut can't land on
/// screen a frame apart.
pub fn track_overlays(state: &mut WmState, config: &UserConfig) {
  let focused_id = state.focused_container().map(|focused| focused.id());

  let tracked = state
    .animation_manager
    .scroll_view
    .entries
    .iter()
    .map(|(id, entry)| {
      // Only while some of the window is on screen.
      let frame = entry
        .peek
        .visible_rect()
        .and(entry.peek.frame_rect())
        .cloned();
      let is_moving = entry.role != Role::Peek;
      (*id, frame, entry.clip.clone(), is_moving, entry.peek.hwnd())
    })
    .collect::<Vec<(Uuid, Option<Rect>, Rect, bool, HWND)>>();

  let mut backdrop_batch = SurrogateBatch::new();
  let mut border_batch = SurrogateBatch::new();

  for (id, frame, clip, is_moving, anchor) in tracked {
    let (_, _, backdrop, border) =
      surrogate_effects_for(focused_id == Some(id), config);

    match (backdrop, &frame) {
      (Some(params), Some(frame)) => upsert_clipped_overlay(
        &mut state.backdrop_overlays,
        id,
        params,
        frame,
        &clip,
        anchor,
        &mut backdrop_batch,
      ),
      _ => {
        if let Some(overlay) = state.backdrop_overlays.get_mut(&id) {
          overlay.hide();
        }
      }
    }

    // Framed with its stack's tab bar, like the real window.
    let border_frame = frame
      .map(|frame| crate::tab_bars::with_tab_bar_of(state, id, &frame));

    match (border, border_frame) {
      (Some(params), Some(frame)) if is_moving => {
        upsert_pinned_border_overlay(
          &mut state.border_overlays,
          id,
          StyleBlend::settled(params),
          &frame,
          &clip,
          &clip,
          anchor,
        );
      }
      (Some(params), Some(frame)) => upsert_clipped_overlay(
        &mut state.border_overlays,
        id,
        params,
        &frame,
        &clip,
        anchor,
        &mut border_batch,
      ),
      _ => {
        if let Some(overlay) = state.border_overlays.get_mut(&id) {
          overlay.hide();
        }
      }
    }
  }

  backdrop_batch.commit();
  border_batch.commit();
}

/// Where each tiling window of `workspace` moves from and to: from where
/// it is shown now, or for a new column from its opening rect.
fn column_moves(
  view: &ScrollView,
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
