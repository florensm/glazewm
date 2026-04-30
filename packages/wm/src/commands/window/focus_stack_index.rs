use crate::{
  commands::container::set_focused_descendant,
  models::Container,
  traits::CommonGetters,
  wm_state::WmState,
};

/// Focuses the stack child at `index`.
///
/// When `subject` is a window inside a stack, its parent stack is used.
/// When `subject` is a `StackContainer` directly (e.g. via tab click), it
/// is used as-is. No-op when no stack can be resolved or `index` is out
/// of bounds.
pub fn focus_stack_index(
  subject: &Container,
  index: usize,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Subject may itself be a stack (e.g. when called from tab click).
  let stack = if let Some(s) = subject.as_stack() {
    s.clone()
  } else {
    let parent = match subject.parent() {
      Some(p) => p,
      None => return Ok(()),
    };
    match parent.as_stack() {
      Some(s) => s.clone(),
      None => return Ok(()),
    }
  };

  let children = stack.children();
  if index >= children.len() {
    return Ok(());
  }

  set_focused_descendant(&children[index], None);

  for child in stack.tiling_children() {
    state.pending_sync.queue_container_to_redraw(child);
  }

  Ok(())
}
