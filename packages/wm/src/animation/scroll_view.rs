//! What a scrolling layout shows of its columns besides the real windows.
//!
//! A real window is only shown while its column is fully on screen, so it
//! never reaches onto a neighbouring monitor. Everything else is drawn by
//! `WindowPeek`s, click-through thumbnails clipped to the monitor:
//!
//! - At rest, a partly visible column shows its visible part as a peek.
//! - While the strip moves (a scroll, or columns opening, closing or
//!   resizing across the monitor edge), every moving column is drawn by a
//!   peek and its real window is cloaked, like the workspace-switch slide:
//!   the peeks go up first, one composited frame later the real windows
//!   are cloaked, and all of them move in lock-step. At the end, columns
//!   that are fully on screen get their real window back, partly visible
//!   ones keep their peek, and the rest are dropped.

use std::{collections::HashMap, time::Instant};

use uuid::Uuid;
use wm_common::{Layout, WindowState};
use wm_platform::{NativeWindowWindowsExt, Point, Rect, WindowPeek};

use super::state::WindowAnimationState;
use crate::{
  commands::general::surrogate_effects_for,
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
  /// Monitor area the peek is clipped to.
  clip: Rect,
  opacity: u8,
  role: Role,
}

impl Entry {
  fn place(&mut self, rect: &Rect) {
    self.peek.place(rect, &self.clip, self.opacity);
  }
}

/// Peeks of the columns of displayed scrolling layouts.
#[derive(Default)]
pub struct ScrollView {
  entries: HashMap<Uuid, Entry>,
  /// Last target rect of every window in a displayed scrolling layout,
  /// i.e. where it is shown now and moves from on the next relayout.
  last_rects: HashMap<Uuid, Rect>,
}

impl ScrollView {
  /// Whether `window_id`'s real window is cloaked while a peek draws it.
  pub fn owns_window(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| entry.role != Role::Peek)
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

  /// Advances moving columns to `now`.
  ///
  /// Returns the windows that finished moving and are fully on screen,
  /// whose real windows are to be uncloaked by the next `platform_sync`.
  pub fn tick(
    &mut self,
    now: Instant,
    clock_origin: Instant,
  ) -> Vec<Uuid> {
    let mut handed_back = Vec::new();

    for (id, entry) in &mut self.entries {
      let Role::Moving { ends_shown } = entry.role else {
        continue;
      };

      entry.motion.start_clock(clock_origin);
      let (rect, _) = entry.motion.current_state_at(now);
      entry.place(&rect);

      if entry.motion.is_complete() {
        entry.role = if ends_shown {
          handed_back.push(*id);
          Role::HandingBack
        } else {
          Role::Peek
        };
      }
    }

    // Columns that moved out of view have nothing left to show.
    self.entries.retain(|_, entry| {
      entry.role != Role::Peek || entry.peek.visible_rect().is_some()
    });

    handed_back
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

  let view = &mut state.animation_manager.scroll_view;

  // Windows that left the displayed scrolling layouts lose their peeks;
  // their real windows are handled like any other window's.
  let is_known = |id: &Uuid| windows.iter().any(|(w, _)| w.id() == *id);
  view.entries.retain(|id, _| is_known(id));
  view.last_rects.retain(|id, _| is_known(id));

  let (opacity, corner_style, _, _) = surrogate_effects_for(false, config);
  let move_config = &config.value.animations.window_move;
  let mut needs_flush = false;
  let mut started_moving = Vec::new();

  for (workspace, clip) in &workspaces {
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

    // The strip moves as one when any column crosses the monitor edge
    // or is already drawn by a peek; moves that stay on screen are left
    // to the regular per-window animations.
    let crosses_edge = |rect: &Rect| !clip.contains_rect(rect);
    let strip_moves = move_config.enabled
      && moves.iter().any(|(window, from, target)| {
        from != target
          && (crosses_edge(from)
            || crosses_edge(target)
            || view.entries.contains_key(&window.id()))
      });

    for (window, from, target) in moves {
      view.last_rects.insert(window.id(), target.clone());

      let ends_shown = !is_hidden_in_layout(&window);
      let is_on_screen = |rect: &Rect| rect.intersection_area(clip) > 0;

      // Mid-move or mid-hand-back with nowhere new to go: leave it be.
      if from == target
        && view
          .entries
          .get(&window.id())
          .is_some_and(|entry| entry.role != Role::Peek)
      {
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

      if !is_on_screen(&from) && !is_on_screen(&target) {
        view.entries.remove(&window.id());
        continue;
      }

      let is_moving = matches!(role, Role::Moving { .. });
      let motion = WindowAnimationState::new_movement(
        from.clone(),
        target,
        if is_moving {
          move_config.duration_ms
        } else {
          0
        },
        move_config.easing.clone(),
      );
      if !is_moving {
        motion.start_clock(Instant::now());
      }

      if let Some(entry) = view.entries.get_mut(&window.id()) {
        entry.motion = motion;
        entry.clip = clip.clone();
        entry.role = role;
        let rect = entry.motion.current_rect();
        entry.place(&rect);
      } else {
        let Ok(peek) =
          WindowPeek::new(&window.native(), &from, &corner_style)
            .inspect_err(|err| {
              tracing::warn!("Failed to create a peek of {window}: {err}");
            })
        else {
          continue;
        };

        let mut entry = Entry {
          peek,
          motion,
          clip: clip.clone(),
          opacity,
          role,
        };
        let rect = entry.motion.current_rect();
        entry.place(&rect);

        // A peek covering a still-visible real window has to be
        // composited before that window is cloaked.
        needs_flush |=
          is_moving && !window.native().is_cloaked().unwrap_or(true);
        view.entries.insert(window.id(), entry);
      }

      if is_moving {
        started_moving.push(window.id());
      }
    }
  }

  if needs_flush {
    wm_platform::dwm_flush();
  }

  // The scroll view drives these windows now, not their own animations.
  for id in &started_moving {
    state.animation_manager.remove_animation(id);
  }

  if state.animation_manager.scroll_view.is_active() {
    state.animation_manager.ensure_timer_running();
  }
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
