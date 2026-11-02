use anyhow::Context;
use wm_common::{
  try_warn, ActiveDrag, ActiveDragOperation, DisplayState,
  FloatingStateConfig, FullscreenStateConfig, WindowState,
};
use wm_platform::{NativeWindow, Rect};

use crate::{
  commands::{
    container::{flatten_split_container, move_container_within_tree},
    general::follow_window_frame,
    window::update_window_state,
  },
  events::handle_window_moved_or_resized_end,
  layout::scrolling::column_of,
  models::{
    is_inactive_stack_child, Container, Monitor, NonTilingWindow,
    WindowContainer,
  },
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

#[allow(clippy::too_many_lines)]
pub fn handle_window_moved_or_resized(
  native_window: &NativeWindow,
  is_interactive_start: bool,
  is_interactive_end: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    let old_frame_position = window.native_properties().frame;
    let frame_position = try_warn!(window.native().frame());

    window.update_native_properties(|properties| {
      properties.frame = frame_position.clone();
    });

    // Only a stack's active window drives the stack's state. The hidden
    // tabs are moved along with it without being e.g. maximized, so their
    // own events would undo the state.
    if is_inactive_stack_child(&window) {
      return Ok(());
    }

    // Handle windows that are actively being dragged.
    if !state.is_paused && window.active_drag().is_some() {
      // An interactive drag moves the window through the OS, and the
      // window is dequeued from redraw for the gesture, so `platform_sync`
      // never tracks it.
      follow_window_frame(&window, &frame_position, state, config);

      // The drag operation has ended on `EVENT_SYSTEM_MOVESIZEEND`.
      if is_interactive_end {
        return handle_window_moved_or_resized_end(&window, state, config);
      }

      update_drag_state(&window, &frame_position, state, config)?;

      // Looked up again, since floating it for the drag replaces it.
      if let Some(window) = state.window_from_native(native_window) {
        crate::drop_indicator::sync_drop_indicator(&window, state, config);
      }

      follow_drag_with_tab_bar(native_window, state, config);

      return Ok(());
    }

    // A resize/move/open animation session owns this window's position for
    // the duration of its `ResizeSession` and the fade-out tail that
    // follows it -- the real window is frozen and cloaked behind a
    // surrogate, repositioned only once via
    // `ResizeSession::maybe_handoff`. That handoff's `SetWindowPos` uses
    // `SWP_NOSENDCHANGING`, which suppresses `WM_WINDOWPOSCHANGING` but
    // not the location-change notification this handler is driven by.
    // Without this guard, that "invisible" handoff reposition gets
    // fully reprocessed below as if it were an independent move --
    // state transitions, floating-position updates, redraw queueing --
    // fighting the animation still in flight on the same window. Most
    // visible as a flicker on a window that's only being repositioned
    // as a side effect of a sibling's drag/relayout (its own animated
    // move, not an interactive drag, so it doesn't hit the
    // `active_drag` branch above).
    if state.animation_manager.owns_window_position(&window.id()) {
      return Ok(());
    }

    let old_is_maximized = window.native_properties().is_maximized;
    let is_maximized = try_warn!(window.native().is_maximized());

    // Ignore duplicate move/resize events. Window position changes can
    // trigger multiple events. For example, restoring from maximized can
    // trigger as many as 4 identical events on Windows.
    if old_frame_position == frame_position
      && old_is_maximized == is_maximized
      && !is_interactive_start
    {
      return Ok(());
    }

    window.update_native_properties(|properties| {
      properties.is_maximized = is_maximized;
    });

    // If the window is not maximized, update its cached shadow borders.
    // Maximized windows temporarily have 0 shadow borders, in which case
    // we should use its previous value for redraws.
    let shadow_borders = try_warn!(window.native().shadow_borders());
    if !is_maximized {
      window.update_native_properties(|properties| {
        properties.shadow_borders = shadow_borders;
      });
    }

    let is_minimized = try_warn!(window.native().is_minimized());

    // Ignore events for minimized windows. Let them be handled by the
    // `PlatformEvent::WindowMinimized` event handler instead.
    if is_minimized {
      return Ok(());
    }

    // Detect whether the window is starting to be interactively moved or
    // resized by the user (e.g. via the window's drag handles). Drag
    // events can be valid for all window states apart from minimized.
    let is_drag_start = !state.is_paused
      && is_interactive_start
      && !matches!(window.state(), WindowState::Minimized);

    if is_drag_start {
      tracing::info!("Window started dragging: {window}");

      window.set_active_drag(Some(ActiveDrag {
        operation: None,
        is_from_floating: matches!(
          window.state(),
          WindowState::Floating(_)
        ),
        initial_position: old_frame_position.clone(),
      }));

      update_drag_state(&window, &frame_position, state, config)?;

      return Ok(());
    }

    let nearest_monitor = state
      .nearest_monitor(&window.native())
      .context("No nearest monitor.")?;

    let should_fullscreen = {
      let workspace = nearest_monitor
        .displayed_workspace()
        .context("No workspace.")?;

      let should_fullscreen = window.should_fullscreen(&workspace)?;

      match window.state() {
        // Override the fullscreen check for when an app self-exits
        // fullscreen (e.g. Chrome via F11) and restores its window to
        // a position that exactly covers the workspace rect.
        WindowState::Fullscreen(fullscreen)
          if !fullscreen.maximized && should_fullscreen =>
        {
          let workspace_rect = workspace.max_workspace_rect()?;

          let old_frame = old_frame_position
            .apply_delta(&window.border_delta().inverse(), None);
          let new_frame = frame_position
            .apply_delta(&window.border_delta().inverse(), None);

          let old_exceeded =
            old_frame.inset(1).contains_rect(&workspace_rect);
          let new_exceeds =
            new_frame.inset(1).contains_rect(&workspace_rect);

          // The window should no longer be fullscreen if the old frame
          // exceeded the workspace bounds (app was in OS fullscreen), but
          // the new frame no longer does. Configs with 0px outer gaps
          // always use the `should_fullscreen` check, since the old frame
          // will never exceed the workspace bounds.
          if old_exceeded && !new_exceeds {
            false
          } else {
            should_fullscreen
          }
        }
        _ => should_fullscreen,
      }
    };

    // Handle a window being maximized or entering fullscreen.
    if is_maximized || should_fullscreen {
      let is_same_state = is_maximized
        && matches!(
          window.state(),
          WindowState::Fullscreen(FullscreenStateConfig {
            maximized: true,
            ..
          })
        )
        || should_fullscreen
          && matches!(
            window.state(),
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: false,
              ..
            })
          );

      // Ignore if there's no state change.
      if is_same_state {
        return Ok(());
      }

      let fullscreen_state = if let WindowState::Fullscreen(
        fullscreen_state,
      ) = window.state()
      {
        fullscreen_state
      } else {
        config
          .value
          .window_behavior
          .state_defaults
          .fullscreen
          .clone()
      };

      let window = update_window_state(
        window.clone(),
        WindowState::Fullscreen(FullscreenStateConfig {
          maximized: is_maximized,
          ..fullscreen_state
        }),
        state,
        config,
      )?;

      if is_maximized {
        // Dequeue the window from redraw if it's maximized, since the
        // window is already in the correct state.
        state
          .pending_sync
          .dequeue_container_from_redraw(window.clone());
      }

      // TODO: Handle a fullscreen window being moved from one monitor to
      // another.

      return Ok(());
    }

    match window.state() {
      WindowState::Fullscreen(_) => {
        // Window is no longer maximized/fullscreen and should be restored.
        tracing::info!("Restoring window from fullscreen: {window}");

        update_window_state(
          window.clone(),
          window.toggled_state(window.state(), config),
          state,
          config,
        )?;
      }
      WindowState::Floating(_) => {
        follow_own_move(&window, &frame_position, state, config);

        if let WindowContainer::NonTilingWindow(window) = window {
          update_floating_window_position(
            &window,
            frame_position,
            &nearest_monitor,
            state,
          )?;
        }
      }
      _ => {
        follow_own_move(&window, &frame_position, state, config);
        make_room_for_min_width(&window, &frame_position, state);
      }
    }
  }

  Ok(())
}

/// Widens a scrolling layout's column for a window its app keeps wider
/// than the column, once the app reports a larger minimum width than it
/// did (e.g. after moving to a monitor of a higher DPI).
fn make_room_for_min_width(
  window: &WindowContainer,
  frame: &Rect,
  state: &mut WmState,
) {
  let Some((_, workspace)) = column_of(&window.as_container()) else {
    return;
  };

  // Mid-animation, the window is between the sizes it was given.
  if state
    .animation_manager
    .get_animation(&window.id())
    .is_some()
  {
    return;
  }

  let Some(target) = state.window_target_positions.get(&window.id())
  else {
    return;
  };

  // The frame is without the shadow borders the target includes.
  let shadows = &window.native_properties().shadow_borders;
  let width = frame.width()
    + shadows.left.to_px(0, None)
    + shadows.right.to_px(0, None);

  if width <= target.width() + 1
    || width <= window.native_properties().min_width
  {
    return;
  }

  let min_width = window
    .native()
    .min_window_size()
    .map_or(0, |(min_width, _)| min_width);

  if min_width > window.native_properties().min_width {
    tracing::info!("Making room for minimum width {min_width}: {window}");
    window.update_native_properties(|properties| {
      properties.min_width = min_width;
    });
    state.pending_sync.queue_container_to_redraw(workspace);
  }
}

/// Moves the tab bar along with a stacked window being dragged, since the
/// drag doesn't go through `platform_sync`. The bar is kept directly
/// behind the window, which the drag raises above other windows.
fn follow_drag_with_tab_bar(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) {
  let stack_id = state
    .window_from_native(native_window)
    .and_then(|window| window.parent())
    .filter(|parent| parent.as_stack().is_some())
    .map(|stack| stack.id());

  if let Some(stack_id) = stack_id {
    crate::tab_bars::sync_tab_bars(
      state,
      config,
      crate::tab_bars::Restack::Stack(stack_id),
    );
  }
}

/// Keeps a shown window's overlays on it after a move nothing queued a
/// redraw for: the app moved its own window, or the WM's asynchronous
/// move landed after `platform_sync` had placed the overlays.
fn follow_own_move(
  window: &WindowContainer,
  frame: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) {
  let is_shown = window.display_state() == DisplayState::Shown
    && window.workspace().is_some_and(|ws| ws.is_displayed());

  if is_shown
    && !state.animation_manager.has_active_surrogate(&window.id())
  {
    follow_window_frame(window, frame, state, config);
  }
}

// TODO: Move to shared location. `handle_window_moved_or_resized_end.rs`
// also uses this.
pub fn update_floating_window_position(
  window: &NonTilingWindow,
  frame_position: Rect,
  nearest_monitor: &Monitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  tracing::info!(
    "Updating floating window position: {}",
    window.as_window_container()?
  );

  // Update state with the new location of the floating window.
  window.set_floating_placement(frame_position);
  window.set_has_custom_floating_placement(true);

  let monitor = window.monitor().context("No monitor.")?;

  // Update the window's workspace if it goes out of bounds of its
  // current workspace.
  if monitor.id() != nearest_monitor.id() {
    let updated_workspace = nearest_monitor
      .displayed_workspace()
      .context("Failed to get workspace of nearest monitor.")?;

    tracing::info!(
      "Floating window moved to new workspace: {updated_workspace}",
    );

    // A window of a floating stack takes its stack along.
    let moved: Container = if let Some(stack) =
      window.parent().filter(|parent| parent.as_stack().is_some())
    {
      stack
    } else {
      window.set_insertion_target(None);
      window.clone().into()
    };

    move_container_within_tree(
      &moved,
      &updated_workspace.clone().into(),
      updated_workspace.child_count(),
      state,
    )?;
  }

  Ok(())
}

/// Updates the window operation based on changes in frame position.
///
/// This function determines whether a window is being moved or resized and
/// updates its operation state accordingly. If the window is being moved,
/// it's set to floating mode.
pub(super) fn update_drag_state(
  window: &WindowContainer,
  frame_position: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(active_drag) = window.active_drag() else {
    return Ok(());
  };

  // Ignore if the window position has not changed yet.
  if *frame_position == active_drag.initial_position {
    return Ok(());
  }

  // Determine the drag operation if not already set.
  let is_move = if let Some(operation) = active_drag.operation {
    matches!(operation, ActiveDragOperation::Move)
  } else {
    let is_move = *frame_position != active_drag.initial_position
      && frame_position.height() == active_drag.initial_position.height()
      && frame_position.width() == active_drag.initial_position.width();

    let operation = if is_move {
      ActiveDragOperation::Move
    } else {
      ActiveDragOperation::Resize
    };

    window.set_active_drag(Some(ActiveDrag {
      operation: Some(operation),
      ..active_drag.clone()
    }));

    is_move
  };

  // Transition window to be floating while it's being dragged, but only
  // after it has been moved at least 10px from its initial position. The
  // 10px threshold is to account for small movements that may be
  // accidental.
  if is_move && !matches!(window.state(), WindowState::Floating(_)) {
    let move_distance = frame_position
      .center_point()
      .distance_between(&active_drag.initial_position.center_point());

    // The OS immediately restores a maximized window while it's being
    // dragged, so we need to update state accordingly without a redraw.
    let is_maximized = matches!(
      window.state(),
      WindowState::Fullscreen(FullscreenStateConfig {
        maximized: true,
        ..
      })
    );

    if move_distance >= 10.0 || is_maximized {
      float_for_drag(window, is_maximized, state, config)?;
    }
  }

  Ok(())
}

/// Floats tiling `window` for it to be dragged around, as a move does
/// once past its threshold: it leaves its slot, which its siblings close
/// up, and is dropped back into tiling where it is let go of.
///
/// `is_maximized` is whether it was maximized, which the OS restores as
/// the drag starts.
pub(crate) fn float_for_drag(
  window: &WindowContainer,
  is_maximized: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // A window of a stack floats with its whole stack, which leaves the
  // stack's parent behind.
  let parent = window
    .parent()
    .and_then(|parent| match parent.as_stack() {
      Some(stack) => stack.parent(),
      None => Some(parent),
    })
    .context("No parent")?;

  let is_fullscreen =
    matches!(window.state(), WindowState::Fullscreen(_)) && !is_maximized;

  let window = update_window_state(
    window.clone(),
    WindowState::Floating(FloatingStateConfig {
      centered: false,
      ..config.value.window_behavior.state_defaults.floating
    }),
    state,
    config,
  )?;

  // `update_window_state` automatically adds the window for redraw,
  // which we don't want in this case. However, for fullscreen windows,
  // we do actually want it to be resized initially so that it's
  // easier to move around while dragging.
  if !is_fullscreen {
    state
      .pending_sync
      .dequeue_container_from_redraw(window.clone());
  }

  // Flatten the parent split container if it only contains the window.
  // TODO: Consider doing this to `move_container_within_tree`, so that
  // the behavior is consistent.
  if let Some(split_parent) = parent.as_split() {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent.clone())?;

      // Hacky fix to redraw siblings after flattening. The parent is
      // queued for redraw from the state change, which gets detached
      // on flatten.
      // TODO: Change `queue_containers_to_redraw` to iterate over its
      // descendant windows and store those instead.
      state
        .pending_sync
        .queue_containers_to_redraw(window.tiling_siblings());
    }
  }

  Ok(())
}
