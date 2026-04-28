use crate::{
  commands::container::set_focused_descendant,
  models::Container,
  traits::CommonGetters,
  wm_state::WmState,
};

/// Cycles focus to the next or previous window within the focused window's
/// parent `StackContainer`.
///
/// If the focused container is not in a stack, this is a no-op.
pub fn cycle_stack_focus(
  focused_container: &Container,
  prev: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = match focused_container.parent() {
    Some(p) => p,
    None => return Ok(()),
  };

  let stack = match parent.as_stack() {
    Some(s) => s.clone(),
    None => return Ok(()),
  };

  let children = stack.children();
  let child_count = children.len();

  if child_count == 0 {
    return Ok(());
  }

  // Find the position of the currently focused child (front of focus order).
  let focused_id = stack
    .borrow_child_focus_order()
    .front()
    .copied()
    .unwrap_or(focused_container.id());

  let current_index = children
    .iter()
    .position(|c| c.id() == focused_id)
    .unwrap_or(0);

  let next_index = if prev {
    (current_index + child_count - 1) % child_count
  } else {
    (current_index + 1) % child_count
  };

  let next_child = &children[next_index];

  set_focused_descendant(next_child, None);

  // Redraw all stack children so the inactive ones are hidden and the
  // newly focused one is shown.
  for child in stack.tiling_children() {
    state.pending_sync.queue_container_to_redraw(child);
  }

  Ok(())
}
