use anyhow::Context;
use wm_common::{FloatingStateConfig, WindowState};
use wm_platform::Rect;

use super::{keep_tab_bar_on_screen, update_stack_state};
use crate::{
  commands::container::{
    attach_container, detach_container, move_container_within_tree,
    set_focused_descendant, wrap_in_stack_container,
  },
  models::{
    other_stack_tabs, InsertionTarget, NonTilingWindow, StackContainer,
    WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves `window` into `stack` at `index`, giving it the stack's state: a
/// floating window joins a floating stack as is, and joins a tiling stack
/// as a tiling window.
///
/// A minimized stack is restored first. Returns the window, which is a
/// new container if its state changed between tiling and non-tiling.
pub fn join_stack(
  window: WindowContainer,
  stack: &StackContainer,
  index: usize,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  if window
    .parent()
    .is_some_and(|parent| parent.id() == stack.id())
  {
    return Ok(window);
  }

  if stack.state() == WindowState::Minimized {
    update_stack_state(
      stack,
      restored_state(stack, config),
      state,
      config,
    )?;
  }

  let had_focus = window.has_focus(None);
  let old_workspace = window.workspace();
  let old_tabs = other_stack_tabs(&window);
  let template = stack.windows().into_iter().next();

  let joined: WindowContainer = match (window, stack.is_tiling()) {
    (window @ WindowContainer::TilingWindow(_), true)
    | (window @ WindowContainer::NonTilingWindow(_), false) => {
      move_container_within_tree(
        &window.clone().into(),
        &stack.clone().into(),
        index,
        state,
      )?;

      window
    }
    (WindowContainer::TilingWindow(window), false) => {
      detach_container(window.clone().into())?;
      let non_tiling = window.to_non_tiling(stack.state(), None);
      attach_container(
        &non_tiling.clone().into(),
        &stack.clone().into(),
        Some(index),
      )?;

      non_tiling.into()
    }
    (WindowContainer::NonTilingWindow(window), true) => {
      detach_container(window.clone().into())?;
      let tiling = window.to_tiling(config.value.gaps.clone());
      attach_container(
        &tiling.clone().into(),
        &stack.clone().into(),
        Some(index),
      )?;

      tiling.into()
    }
  };

  // Match the other windows of a non-tiling stack.
  if let (WindowContainer::NonTilingWindow(window), Some(template)) =
    (&joined, &template)
  {
    window.set_state(template.state());
    window.set_prev_state(
      template.prev_state().unwrap_or(WindowState::Tiling),
    );
    window.set_own_floating_placement(template.floating_placement());
  }

  if had_focus {
    set_focused_descendant(&joined.clone().into(), None);
  }

  state
    .pending_sync
    .queue_containers_to_redraw(stack.windows())
    .queue_containers_to_redraw(old_tabs);

  if let Some(parent) = stack.parent() {
    state
      .pending_sync
      .queue_containers_to_redraw(parent.tiling_children());
  }

  if let Some(old_workspace) = old_workspace {
    state
      .pending_sync
      .queue_containers_to_redraw(old_workspace.tiling_children())
      .queue_workspace_to_reorder(old_workspace);
  }

  Ok(joined)
}

/// Puts `window`, which isn't in a stack, into the new, empty `stack` in
/// its place.
pub fn wrap_window_in_stack(
  window: &WindowContainer,
  stack: &StackContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window.parent().context("No parent.")?;

  match window {
    WindowContainer::TilingWindow(window) => {
      wrap_in_stack_container(stack, &parent, &[window.clone().into()])?;
    }
    WindowContainer::NonTilingWindow(window) => {
      let index = window.index();
      let had_focus = window.has_focus(None);

      // The stack is only attached once it holds the window, so it never
      // takes part in the tiling layout.
      detach_container(window.clone().into())?;
      attach_container(
        &window.clone().into(),
        &stack.clone().into(),
        None,
      )?;
      attach_container(&stack.clone().into(), &parent, Some(index))?;

      if had_focus {
        set_focused_descendant(&window.clone().into(), None);
      }

      if let Some(workspace) = window.workspace() {
        window.set_floating_placement(keep_tab_bar_on_screen(
          stack,
          &workspace,
          window.floating_placement(),
        ));
      }
    }
  }

  state
    .pending_sync
    .queue_containers_to_redraw(stack.windows());

  Ok(())
}

/// Takes `window` out of its stack as a floating window, placed at the
/// cursor if `at_cursor`, otherwise slightly offset from the stack.
///
/// A window taken out of a tiling stack goes back into it when tiled
/// again (e.g. with `toggle-floating`).
pub fn float_out_of_stack(
  window: &WindowContainer,
  at_cursor: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(stack) = window.parent().and_then(|p| p.as_stack().cloned())
  else {
    return Ok(());
  };

  let workspace = window.workspace().context("No workspace.")?;
  let floating_state = WindowState::Floating(FloatingStateConfig {
    centered: false,
    ..config.value.window_behavior.state_defaults.floating
  });

  let placement = {
    let size = window.floating_placement();
    let cursor = at_cursor
      .then(|| state.dispatcher.cursor_position().ok())
      .flatten();

    match cursor {
      // Grab the window where the tab was let go, near its top edge.
      Some(cursor) => size.translate_to_coordinates(
        cursor.x - size.width() / 2,
        cursor.y - stack.tab_bar_height_px() / 2,
      ),
      None if stack.is_tiling() => {
        size.translate_to_center(&workspace.to_rect()?)
      }
      None => {
        let offset = stack.tab_bar_height_px().max(24);
        size.translate_to_coordinates(size.x() + offset, size.y() + offset)
      }
    }
  };

  // A window taken out of a stack is never auto-stacked again.
  state.auto_stack.mark_settled(window.native().id());

  let other_tabs = other_stack_tabs(window);

  let non_tiling: NonTilingWindow = match window {
    WindowContainer::TilingWindow(window) => {
      let insertion_target = InsertionTarget {
        target_parent: stack.clone().into(),
        target_index: window.index(),
        prev_tiling_size: window.tiling_size(),
        prev_sibling_count: window.tiling_siblings().count(),
      };

      detach_container(window.clone().into())?;
      window.to_non_tiling(floating_state, Some(insertion_target))
    }
    WindowContainer::NonTilingWindow(window) => {
      detach_container(window.clone().into())?;

      if !window.state().is_same_state(&floating_state) {
        window.set_prev_state(window.state());
      }

      window.set_state(floating_state);
      window.set_insertion_target(None);
      window.clone()
    }
  };

  attach_container(
    &non_tiling.clone().into(),
    &workspace.clone().into(),
    None,
  )?;

  non_tiling
    .set_floating_placement(clamp_to(&placement, &workspace.to_rect()?));
  set_focused_descendant(&non_tiling.clone().into(), None);

  state
    .pending_sync
    .mark_window_state_change(non_tiling.id())
    .queue_container_to_redraw(non_tiling)
    .queue_containers_to_redraw(other_tabs)
    .queue_containers_to_redraw(workspace.tiling_children())
    .queue_workspace_to_reorder(workspace)
    .queue_focus_change();

  Ok(())
}

/// State a minimized stack goes back to.
pub(crate) fn restored_state(
  stack: &StackContainer,
  config: &UserConfig,
) -> WindowState {
  stack
    .windows()
    .first()
    .and_then(WindowGetters::prev_state)
    .filter(|state| *state != WindowState::Minimized)
    .unwrap_or_else(|| WindowState::default_from_config(&config.value))
}

/// Moves `rect` into `bounds` as far as it fits.
fn clamp_to(rect: &Rect, bounds: &Rect) -> Rect {
  let x = rect.x().min(bounds.right - rect.width()).max(bounds.left);
  let y = rect.y().min(bounds.bottom - rect.height()).max(bounds.top);
  rect.translate_to_coordinates(x, y)
}
