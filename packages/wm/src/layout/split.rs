use anyhow::Context;
use wm_common::WindowState;

use super::Insertion;
use crate::{models::Container, traits::CommonGetters, wm_state::WmState};

/// Gets where to insert a new window in the container tree.
///
/// Rules:
/// - For non-tiling windows: Always append to the workspace.
/// - For tiling windows:
///   1. Try to insert after the focused tiling window (or its parent
///      stack) if one exists.
///   2. If a non-tiling window is focused, try to insert after the first
///      tiling window found.
///   3. If no tiling windows exist, append to the workspace.
///
/// New windows are never inserted into an existing `StackContainer`
/// automatically, unless configured. Stacking is otherwise an explicit
/// user action, e.g. via `toggle-stack` or `stack-absorb-neighbor`.
pub(super) fn insertion_target(
  window_state: &WindowState,
  joins_focused_stack: bool,
  state: &WmState,
) -> anyhow::Result<Insertion> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  // For tiling windows, try to find a suitable tiling window to insert
  // next to.
  if *window_state == WindowState::Tiling {
    let sibling = match focused_container {
      Container::TilingWindow(_) => Some(focused_container),
      _ => focused_workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window),
    };

    if let Some(sibling) = sibling {
      let parent = sibling.parent().context("No parent.")?;

      // Tile next to the sibling's stack rather than inside it, unless
      // configured otherwise.
      if parent.as_stack().is_some() && !joins_focused_stack {
        return Ok(Insertion::At {
          index: parent.index() + 1,
          parent: parent.parent().context("No parent.")?,
        });
      }

      return Ok(Insertion::At {
        index: sibling.index() + 1,
        parent,
      });
    }
  }

  // Default to appending to workspace.
  Ok(Insertion::At {
    index: focused_workspace.child_count(),
    parent: focused_workspace.into(),
  })
}
