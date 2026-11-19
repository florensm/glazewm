//! Tiles under a fullscreen window stay where they are on screen.
//!
//! A tile going fullscreen leaves the tiling tree, so its siblings are
//! laid out again to fill its place. They are behind it, though: moving
//! them is work nobody sees, except mid-transition, where they visibly
//! reflow around the growing window, and again in reverse as it comes
//! back. So their redraws are held while the fullscreen window is on top
//! of its workspace, and let through once it no longer is, however that
//! ends (toggled back, closed, moved away, focus moving off it). Toggled
//! back to tiling, the layout is the one they are still drawn at, so they
//! never move at all.

use std::collections::HashSet;

use uuid::Uuid;
use wm_common::{DisplayState, WindowState};

use crate::{
  layout::scrolling,
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// Tiles whose redraws are held while a fullscreen window covers them.
#[derive(Debug, Default)]
pub struct CoveredTiles {
  held: HashSet<Uuid>,
}

impl CoveredTiles {
  /// Whether `window_id`'s redraws are held.
  pub fn contains(&self, window_id: &Uuid) -> bool {
    self.held.contains(window_id)
  }
}

/// Holds this sync's redraws of covered tiles, and lets through those of
/// tiles no longer covered. Runs before anything reads the redraw queue.
pub fn sync_covered_tiles(state: &mut WmState) {
  let held = std::mem::take(&mut state.covered_tiles.held);
  for id in held {
    match state.window_by_id(id) {
      Some(window) if is_covered(state, &window) => {
        state.covered_tiles.held.insert(id);
      }
      Some(window) => {
        state.pending_sync.queue_container_to_redraw(window);
      }
      None => {}
    }
  }

  let queued = state.windows_to_redraw();
  let (newly_held, redrawn): (Vec<_>, Vec<_>) = queued
    .into_iter()
    .partition(|window| is_covered(state, window));

  if newly_held.is_empty() {
    return;
  }

  tracing::debug!(
    "Holding {} redraws under a fullscreen window.",
    newly_held.len()
  );

  state
    .covered_tiles
    .held
    .extend(newly_held.iter().map(CommonGetters::id));
  state.pending_sync.replace_containers_to_redraw(redrawn);
}

/// Whether `window` is a tile shown, at rest, under the fullscreen window
/// focused on its workspace, which every window there is stacked below.
///
/// Not a column of a scrolling layout, whose strip the scroll view moves
/// as a whole, nor while the overview is shown, whose previews follow
/// windows to where they are laid out.
fn is_covered(state: &WmState, window: &WindowContainer) -> bool {
  if window.state() != WindowState::Tiling
    || window.display_state() != DisplayState::Shown
    || state.overview.is_shown()
    || !state.window_target_positions.contains_key(&window.id())
    || state.animation_manager.stands_in_for(&window.id())
    || scrolling::column_of(&window.as_container()).is_some()
  {
    return false;
  }

  let Some(workspace) = window.workspace() else {
    return false;
  };

  workspace.is_displayed()
    && workspace
      .descendant_focus_order()
      .find_map(|container| container.as_window_container().ok())
      .is_some_and(|top| matches!(top.state(), WindowState::Fullscreen(_)))
}

#[cfg(test)]
mod tests {
  use wm_common::{FullscreenStateConfig, WindowState};
  use wm_platform::Rect;

  use super::sync_covered_tiles;
  use crate::{
    commands::container::set_focused_descendant,
    models::{Monitor, NonTilingWindow, TilingWindow, Workspace},
    traits::CommonGetters,
    wm_state::WmState,
  };

  /// Two tiles and a fullscreen window, focused, on one workspace. Both
  /// tiles were drawn before.
  fn setup() -> (WmState, [TilingWindow; 2], NonTilingWindow) {
    let tiles = [TilingWindow::mock().call(), TilingWindow::mock().call()];
    let fullscreen = NonTilingWindow::mock()
      .state(WindowState::Fullscreen(FullscreenStateConfig::default()))
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(tiles.iter().cloned().map(Into::into).collect())
      .non_tiling_windows(vec![fullscreen.clone()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);

    set_focused_descendant(&fullscreen.clone().into(), None);
    for tile in &tiles {
      state
        .window_target_positions
        .insert(tile.id(), Rect::from_xy(0, 0, 100, 100));
    }

    (state, tiles, fullscreen)
  }

  fn queued_ids(state: &WmState) -> Vec<uuid::Uuid> {
    let mut ids = state
      .windows_to_redraw()
      .iter()
      .map(CommonGetters::id)
      .collect::<Vec<_>>();
    ids.sort();
    ids
  }

  #[test]
  fn tiles_under_a_focused_fullscreen_window_are_held() {
    let (mut state, tiles, fullscreen) = setup();
    let workspace = fullscreen.workspace().expect("workspace");
    state.pending_sync.queue_container_to_redraw(workspace);

    sync_covered_tiles(&mut state);

    assert_eq!(queued_ids(&state), vec![fullscreen.id()]);
    assert!(tiles.iter().all(|t| state.covered_tiles.contains(&t.id())));
  }

  #[test]
  fn held_tiles_are_redrawn_once_focus_leaves_the_fullscreen_window() {
    let (mut state, tiles, _) = setup();
    state.pending_sync.queue_containers_to_redraw(tiles.clone());
    sync_covered_tiles(&mut state);
    state.pending_sync.clear();

    set_focused_descendant(&tiles[0].clone().into(), None);
    sync_covered_tiles(&mut state);

    let mut expected =
      tiles.iter().map(CommonGetters::id).collect::<Vec<_>>();
    expected.sort();
    assert_eq!(queued_ids(&state), expected);
    assert!(!state.covered_tiles.contains(&tiles[0].id()));
  }

  #[test]
  fn held_tiles_stay_held_with_nothing_else_queued() {
    let (mut state, tiles, _) = setup();
    state.pending_sync.queue_containers_to_redraw(tiles.clone());
    sync_covered_tiles(&mut state);
    state.pending_sync.clear();

    sync_covered_tiles(&mut state);

    assert!(queued_ids(&state).is_empty());
    assert!(tiles.iter().all(|t| state.covered_tiles.contains(&t.id())));
  }

  #[test]
  fn a_tile_never_drawn_is_not_held() {
    let (mut state, tiles, _) = setup();
    state.window_target_positions.remove(&tiles[1].id());
    state.pending_sync.queue_containers_to_redraw(tiles.clone());

    sync_covered_tiles(&mut state);

    assert_eq!(queued_ids(&state), vec![tiles[1].id()]);
  }
}
