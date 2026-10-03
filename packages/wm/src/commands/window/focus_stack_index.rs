use super::activate_stack_child;
use crate::{models::Container, traits::CommonGetters, wm_state::WmState};

/// Focuses the tab at `index` (zero-based) of a stack.
///
/// `subject` is either the stack itself (e.g. from a tab click) or a
/// window inside it. No-op if no stack is found or `index` is out of
/// bounds.
pub fn focus_stack_index(
  subject: &Container,
  index: usize,
  state: &mut WmState,
) {
  let Some(stack) = subject.as_stack().cloned().or_else(|| {
    subject
      .parent()
      .and_then(|parent| parent.as_stack().cloned())
  }) else {
    return;
  };

  if let Some(child) = stack.children().get(index) {
    activate_stack_child(&stack, child, state);
  }
}
