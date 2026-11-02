use anyhow::Context;
use wm_platform::{MouseButton, MouseEvent, Point};

use crate::{
  commands::container::set_focused_descendant,
  layout::scrolling::is_focusable_by_hover,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_mouse_move(
  event: &MouseEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Ignore mouse move events if the WM is paused. The mouse listener
  // should anyways be disabled when the WM is paused, but this is just in
  // case any events slipped through while disabling.
  if state.is_paused {
    return Ok(());
  }

  if let MouseEvent::ButtonUp {
    button: MouseButton::Left,
    position,
    ..
  } = event
  {
    // A click on the overview is its own: a peek behind it focused here
    // would fight its pick, e.g. scrolling or switching back mid-zoom.
    if !state.overview.is_shown() {
      focus_clicked_peek(position, state);
    }
    return Ok(());
  }

  if let MouseEvent::Move {
    pressed_buttons,
    position,
  } = event
  {
    // Ignore event if left/right-click is down. Otherwise, this causes
    // focus to jitter when a window is being resized by its drag
    // handles. Also ignore if the OS focused window isn't the same as
    // the WM's focused window, and below, while the focused app has a
    // menu or dropdown open: focusing another window would close it, or
    // bury a popup behind that window.
    if pressed_buttons.contains(&MouseButton::Left)
      || pressed_buttons.contains(&MouseButton::Right)
      || !state.is_focus_synced
      || !config.value.general.focus_follows_cursor
    {
      return Ok(());
    }

    // The cursor is over the overview while it is on screen, zooming
    // into its pick included.
    if state.overview.is_shown() {
      return Ok(());
    }

    let window_under_cursor = state
      .dispatcher
      .window_from_point(position)?
      .and_then(|native| state.window_from_native(&native));

    // Set focus to whichever window is currently under the cursor.
    if let Some(window) = window_under_cursor {
      let focused_container =
        state.focused_container().context("No focused container.")?;

      if focused_container.id() != window.id()
        && is_focusable_by_hover(&window)?
        && !state.dispatcher.is_foreground_interacting()
      {
        set_focused_descendant(&window.as_container(), None);
        state.pending_sync.queue_focus_change();
      }
    } else {
      // Focus the monitor if no window is under the cursor.
      let cursor_monitor = state
        .monitor_at_point(position)
        .context("No monitor under cursor.")?;

      let focused_monitor = state
        .focused_container()
        .context("No focused container.")?
        .monitor()
        .context("Focused container has no monitor.")?;

      // Avoid setting focus to the same monitor.
      if cursor_monitor.id() != focused_monitor.id()
        && !state.dispatcher.is_foreground_interacting()
      {
        set_focused_descendant(&cursor_monitor.as_container(), None);
        state.pending_sync.queue_focus_change();
      }
    }
  }

  Ok(())
}

/// Focuses the scrolling layout's column whose peek was clicked at
/// `position`, which scrolls it into view. Peeks are click-through, so
/// the click itself lands on whatever is below.
fn focus_clicked_peek(position: &Point, state: &mut WmState) {
  let is_dragging = state
    .windows()
    .iter()
    .any(|window| window.active_drag().is_some());

  if let Some(window) = state
    .animation_manager
    .scroll_view
    .window_at(position)
    .filter(|_| !is_dragging)
    .and_then(|id| state.container_by_id(id))
  {
    set_focused_descendant(&window, None);
    state.pending_sync.queue_focus_change();
  }
}
