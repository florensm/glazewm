//! What a scrolling layout shows of its columns besides the real windows.
//!
//! Columns show their real windows, also where they run off a monitor
//! edge with no other monitor behind it. Only a column that would reach
//! onto another monitor is cloaked, and its visible part is shown by a
//! `WindowPeek`: a click-through thumbnail clipped to the monitor, with
//! its border, backdrop and tab bar cut there too, so nothing of a column
//! shows on the monitor beside it.
//!
//! Columns moving play on the scene covering their monitor (see
//! `scene`), beneath which they land at once.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;
use wm_common::{DisplayState, Layout, WindowState};
use wm_platform::{
  NativeWindowWindowsExt, Overlay, Point, Rect, WindowBatch, WindowPeek,
  HWND,
};

use crate::{
  commands::general::{effects_for, upsert_clipped_overlay},
  layout::is_hidden_in_layout,
  models::{is_inactive_stack_child, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

struct Entry {
  peek: WindowPeek,
  /// Monitor area the peek is clipped to.
  clip: Rect,
}

/// Peeks of the columns of displayed scrolling layouts.
#[derive(Default)]
pub struct ScrollView {
  entries: HashMap<Uuid, Entry>,
  /// Windows already reported by `report_stuck_windows`.
  reported_stuck: HashSet<Uuid>,
}

impl ScrollView {
  /// Whether a peek draws `window_id`, along with its overlays.
  pub fn draws_window(&self, window_id: &Uuid) -> bool {
    self.entries.contains_key(window_id)
  }

  /// The monitor area the peek drawing `window_id` is cut to, and the
  /// peek's own window, which what goes with it (e.g. its stack's tab bar)
  /// sits behind.
  pub fn peek_of(&self, window_id: &Uuid) -> Option<(Rect, HWND)> {
    let entry = self.entries.get(window_id)?;
    Some((entry.clip.clone(), entry.peek.hwnd()))
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
}

/// Updates the peeks of the columns of displayed scrolling layouts.
///
/// Runs before `platform_sync` repositions anything, so a column's peek is
/// on screen before its real window is cloaked.
pub fn sync_scroll_view(state: &mut WmState, config: &UserConfig) {
  let windows = displayed_scrolling_workspaces(state)
    .into_iter()
    .flat_map(|(workspace, clip)| {
      tiling_windows(&workspace)
        .into_iter()
        .map(move |window| (window, clip.clone()))
    })
    .collect::<Vec<_>>();
  let focused_id = state.focused_container().map(|focused| focused.id());

  let view = &mut state.animation_manager.scroll_view;

  // Peeks of windows that left the displayed scrolling layouts, or that
  // their real window shows now, stay up until any that take over are on
  // screen, e.g. that of a stack's newly active tab, so the column never
  // goes blank.
  let mut gone = view
    .entries
    .extract_if(|id, _| !windows.iter().any(|(w, _)| w.id() == *id))
    .collect::<Vec<_>>();

  let mut adds_peek = false;
  let mut batch = WindowBatch::new();

  for (window, clip) in &windows {
    let Ok(target) = target_rect(window) else {
      continue;
    };

    // At rest on screen, the real window shows it.
    let is_peeked =
      is_hidden_in_layout(window) && target.intersection_area(clip) > 0;
    if !is_peeked {
      if let Some(entry) = view.entries.remove(&window.id()) {
        gone.push((window.id(), entry));
      }
      continue;
    }

    // The window's own transparency and corners, as it is focused or not.
    let (opacity, corner_style, _, _) =
      effects_for(focused_id == Some(window.id()), config);

    if let Some(entry) = view.entries.get_mut(&window.id()) {
      entry.clip = clip.clone();
      entry.peek.place(&target, clip, opacity, &mut batch);
      continue;
    }

    match WindowPeek::new(&window.native(), &target, &corner_style) {
      Ok(mut peek) => {
        peek.place(&target, clip, opacity, &mut batch);
        view.entries.insert(
          window.id(),
          Entry {
            peek,
            clip: clip.clone(),
          },
        );
        adds_peek = true;
      }
      Err(err) => {
        tracing::warn!("Failed to create a peek of {window}: {err}");
      }
    }
  }

  let replaces_peek = adds_peek
    && gone
      .iter()
      .any(|(_, entry)| entry.peek.visible_rect().is_some());
  if replaces_peek {
    wm_platform::dwm_flush();
  }
  drop(gone);

  track_overlays(state, config, batch);
}

/// Logs windows of displayed scrolling layouts that should be shown but
/// whose real window is still cloaked, once each, for diagnosing windows
/// that disappear.
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

    tracing::warn!(
      "Scrolling layout window stuck cloaked: {window}, rect {:?}, peeked \
       {}.",
      target_rect(&window).ok(),
      view.entries.contains_key(&window.id()),
    );
  }
}

/// Keeps the border and backdrop overlays of the windows the scroll view
/// draws on their peeks, cut at the monitor edge like the peeks are.
/// `sync_overlays` leaves these windows to it.
///
/// Commits their moves in `batch`, along with the peeks' moves it already
/// holds, so a peek and its overlays land in the same frame.
pub fn track_overlays(
  state: &mut WmState,
  config: &UserConfig,
  mut batch: WindowBatch,
) {
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
      (*id, frame, entry.clip.clone(), entry.peek.hwnd())
    })
    .collect::<Vec<(Uuid, Option<Rect>, Rect, HWND)>>();

  for (id, frame, clip, anchor) in tracked {
    let (_, _, backdrop, border) =
      effects_for(focused_id == Some(id), config);

    match (backdrop, &frame) {
      (Some(params), Some(frame)) => upsert_clipped_overlay(
        &mut state.backdrop_overlays,
        id,
        params,
        frame,
        &clip,
        anchor,
        &mut batch,
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
      (Some(params), Some(frame)) => upsert_clipped_overlay(
        &mut state.border_overlays,
        id,
        params,
        &frame,
        &clip,
        anchor,
        &mut batch,
      ),
      _ => {
        if let Some(overlay) = state.border_overlays.get_mut(&id) {
          overlay.hide();
        }
      }
    }
  }

  batch.commit();
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

/// Where the layout puts the window.
fn target_rect(window: &WindowContainer) -> anyhow::Result<Rect> {
  Ok(
    window
      .to_rect()?
      .apply_delta(&window.total_border_delta()?, None),
  )
}
