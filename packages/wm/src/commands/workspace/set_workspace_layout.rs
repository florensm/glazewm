use anyhow::Context;
use wm_common::{WmEvent, WorkspaceLayout};

use crate::{
  models::Container,
  traits::{CommonGetters, TilingDirectionGetters},
  wm_state::WmState,
};

/// Sets the layout for the direction container that owns the given container.
///
/// Finds the nearest direction container (workspace or split container) and
/// applies the new layout, then queues a redraw of all affected windows.
/// Emits a `LayoutChanged` event on success.
pub fn set_workspace_layout(
  container: Container,
  layout: WorkspaceLayout,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let direction_container = container
    .direction_container()
    .context("No direction container.")?;

  direction_container.set_layout(layout.clone());

  // Redraw all tiling windows in the affected container.
  for window in direction_container
    .descendants()
    .filter_map(|c| c.as_tiling_window().cloned())
  {
    state.pending_sync.queue_container_to_redraw(window);
  }

  state.emit_event(WmEvent::LayoutChanged {
    direction_container: direction_container.to_dto()?,
    new_layout: layout,
  });

  Ok(())
}
