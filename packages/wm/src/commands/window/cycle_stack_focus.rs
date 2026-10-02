use crate::{
  commands::container::set_focused_descendant,
  models::{Container, StackContainer},
  traits::CommonGetters,
  wm_state::WmState,
};

/// Focuses the next (or previous) tab of the stack containing
/// `focused_container`, wrapping around at either end.
///
/// No-op if the container is not in a stack.
pub fn cycle_stack_focus(
  focused_container: &Container,
  prev: bool,
  state: &mut WmState,
) {
  let Some(stack) = focused_container
    .parent()
    .and_then(|parent| parent.as_stack().cloned())
  else {
    return;
  };

  let children = stack.children();
  let child_count = children.len();

  let Some(active_index) = stack
    .active_child()
    .and_then(|active| children.iter().position(|c| *c == active))
  else {
    return;
  };

  let next_index = if prev {
    (active_index + child_count - 1) % child_count
  } else {
    (active_index + 1) % child_count
  };

  activate_stack_child(&stack, &children[next_index], state);
}

/// Makes `child` the active tab of `stack` and queues the tabs for redraw
/// so that only the active one is shown.
pub(crate) fn activate_stack_child(
  stack: &StackContainer,
  child: &Container,
  state: &mut WmState,
) {
  set_focused_descendant(child, None);

  state
    .pending_sync
    .queue_containers_to_redraw(stack.tiling_children());
}
