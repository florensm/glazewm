use anyhow::Context;
use wm_common::{
  try_warn, ActiveDrag, ActiveDragOperation, FullscreenStateConfig,
  WindowState,
};
use wm_platform::{LengthValue, Point};

use crate::{
  commands::{
    container::{
      move_container_within_tree, set_focused_descendant,
      wrap_in_container,
    },
    window::{join_stack, set_window_size, update_window_state},
  },
  events::{float_for_drag, update_floating_window_position},
  layout::{
    column_drop::drop_into_column,
    scrolling::keep_right_edge,
    window_drop::{tab_bar_at, window_drop, window_drop_on, WindowDrop},
  },
  models::{
    Container, NonTilingWindow, SplitContainer, TilingWindow,
    WindowContainer, Workspace,
  },
  traits::{CommonGetters, TilingDirectionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window and attach a dragged
/// floating window.
///
/// TODO: Move this to a better location - maybe a new `active_drag_ext`
/// mod.
pub fn handle_window_moved_or_resized_end(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  #[cfg(target_os = "windows")]
  crate::drop_indicator::hide_drop_indicator(state);

  let Some(active_drag) = window.active_drag() else {
    return Ok(());
  };

  // The drag raised the window, so its stack's tab bar is put back behind
  // it wherever the stack ends up.
  if window
    .parent()
    .is_some_and(|parent| parent.as_stack().is_some())
  {
    state.pending_sync.queue_tab_bar_update();
  }

  match &window {
    WindowContainer::NonTilingWindow(window) => {
      let is_move =
        matches!(active_drag.operation, Some(ActiveDragOperation::Move));

      if is_move && drop_onto_tab_bar(window, state, config)? {
        return Ok(());
      }

      let is_maximized = try_warn!(window.native().is_maximized());

      window.update_native_properties(|properties| {
        properties.is_maximized = is_maximized;
      });

      let nearest_monitor = state
        .nearest_monitor(&window.native())
        .context("Failed to get workspace of nearest monitor.")?;

      let should_fullscreen = window.should_fullscreen(
        &nearest_monitor
          .displayed_workspace()
          .context("No workspace.")?,
      )?;

      if is_maximized || should_fullscreen {
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
          window.clone().into(),
          WindowState::Fullscreen(FullscreenStateConfig {
            maximized: is_maximized,
            ..fullscreen_state
          }),
          state,
          config,
        )?;

        window.set_active_drag(None);

        if is_maximized {
          // Dequeue the window from redraw if it's maximized, since the
          // window is already in the correct state.
          state
            .pending_sync
            .dequeue_container_from_redraw(window.clone());
        } else {
          // Force a redraw to snap the window to the monitor edges.
          // TODO: Skip redraw if it's already matches fullscreen frame.
          state.pending_sync.queue_container_to_redraw(window.clone());
        }

        return Ok(());
      }

      if active_drag.is_from_floating {
        update_floating_window_position(
          window,
          window.native_properties().frame,
          &nearest_monitor,
          state,
        )?;
        window.set_active_drag(None);
      } else {
        // Seed `window_target_positions` with the current drag position so
        // the snap-back animation starts from where the window actually
        // is, not its stale pre-drag tiling position.
        state
          .window_target_positions
          .insert(window.id(), window.native_properties().frame);

        // Window is a temporary floating window that should be
        // reverted back to tiling.
        let mouse_pos = state.dispatcher.cursor_position()?;
        let window =
          drop_as_tiling_window(window, &mouse_pos, state, config)?;
        window.set_active_drag(None);
      }
    }
    WindowContainer::TilingWindow(window) => {
      end_tiling_drag(window, &active_drag, state)?;
    }
  }

  Ok(())
}

/// Takes the size a tiling window was dragged to as its tiling size.
fn end_tiling_drag(
  window: &TilingWindow,
  active_drag: &ActiveDrag,
  state: &mut WmState,
) -> anyhow::Result<()> {
  tracing::info!(
    "Tiling window move/resize ended: {}",
    window.as_window_container()?
  );

  let frame = window.native_properties().frame;

  // A scrolling layout's column dragged by its left edge grows or shrinks
  // leftwards, as it was dragged.
  if active_drag.operation == Some(ActiveDragOperation::Resize)
    && frame.left != active_drag.initial_position.left
  {
    keep_right_edge(&window.clone().into())?;
  }

  // Update the window's size based on the new frame position. This means
  // we use the actual window dimensions as the source of truth.
  set_window_size(
    window.clone().into(),
    Some(LengthValue::from_px(frame.width())),
    Some(LengthValue::from_px(frame.height())),
    state,
  )?;

  window.set_active_drag(None);

  // Force a redraw of the window to snap it back to its original position.
  // This is necessary when:
  // - The window is the only tiling window in the workspace.
  // - The window is not past the movement threshold for transitioning to
  //   floating while being dragged.
  // - Resizing in a direction that doesn't change the window's tiling
  //   size.
  state.pending_sync.queue_container_to_redraw(window.clone());

  Ok(())
}

/// Handles transition from temporary floating window to tiling window on
/// drag end, with the cursor at `mouse_pos`.
fn drop_as_tiling_window(
  moved_window: &NonTilingWindow,
  mouse_pos: &Point,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  tracing::info!(
    "Tiling window drag ended: {}",
    moved_window.as_window_container()?
  );

  let drop = window_drop(moved_window, mouse_pos, state)?
    .context("Couldn't find workspace for window drop.")?;
  put_dropped_window(moved_window, drop, state, config)
}

/// Moves tiling `window` to where dragging it with the mouse to `point`
/// on `workspace` and letting go puts it, displayed or not: beside the
/// window there, into the column there, or into the workspace if empty.
/// A window of a stack brings its whole stack along.
pub fn place_tiling_window(
  window: &TilingWindow,
  workspace: &Workspace,
  point: &Point,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  float_for_drag(&window.clone().into(), false, state, config)?;

  let floating = state
    .container_by_id(window.id())
    .and_then(|container| NonTilingWindow::try_from(container).ok())
    .context("Window didn't float to be placed.")?;

  let drop = window_drop_on(&floating, workspace.clone(), point, state)?;
  put_dropped_window(&floating, drop, state, config)?;
  Ok(())
}

/// Ends `window`'s drag where it began, e.g. as the overview takes it
/// over: a window dragged out of tiling goes back into it there, and a
/// floating one back to where it was.
pub fn cancel_drag(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(active_drag) = window.active_drag() else {
    return Ok(());
  };
  let home = active_drag.initial_position;

  match window {
    WindowContainer::NonTilingWindow(window)
      if active_drag.is_from_floating =>
    {
      let monitor = state
        .nearest_monitor(&window.native())
        .context("No monitor to put the window back on.")?;
      update_floating_window_position(window, home, &monitor, state)?;
      window.set_active_drag(None);
      state.pending_sync.queue_container_to_redraw(window.clone());
    }
    WindowContainer::NonTilingWindow(window) => {
      let dropped = drop_as_tiling_window(
        window,
        &home.center_point(),
        state,
        config,
      )?;
      dropped.set_active_drag(None);
    }
    // Not floated yet, it never left its place.
    WindowContainer::TilingWindow(window) => {
      window.set_active_drag(None);
      state.pending_sync.queue_container_to_redraw(window.clone());
    }
  }

  Ok(())
}

/// Puts `moved_window`, floating while dragged, back into tiling where
/// `drop` says.
fn put_dropped_window(
  moved_window: &NonTilingWindow,
  drop: WindowDrop,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  // A window of a stack carries its whole stack.
  let moved_stack = moved_window
    .parent()
    .and_then(|parent| parent.as_stack().cloned());

  // The container that ends up where the window is dropped.
  let dropped = |window: &WindowContainer| -> Container {
    moved_stack
      .clone()
      .map_or_else(|| window.clone().into(), Into::into)
  };

  match drop {
    WindowDrop::Column { workspace, drop } => {
      let moved_window = update_window_state(
        moved_window.clone().into(),
        WindowState::Tiling,
        state,
        config,
      )?;

      drop_into_column(&dropped(&moved_window), &drop, state, config)?;
      state.pending_sync.queue_container_to_redraw(workspace);

      Ok(moved_window)
    }
    WindowDrop::Into(parent) => {
      let moved: Container = if let Some(stack) = &moved_stack {
        stack.set_insertion_target(None);
        stack.clone().into()
      } else {
        moved_window.set_insertion_target(None);
        moved_window.clone().into()
      };

      move_container_within_tree(&moved, &parent.into(), 0, state)?;

      update_window_state(
        moved_window.as_window_container()?,
        WindowState::Tiling,
        state,
        config,
      )
    }
    WindowDrop::Beside {
      parent,
      target,
      position,
      should_split,
    } => {
      let moved_window = update_window_state(
        moved_window.clone().into(),
        WindowState::Tiling,
        state,
        config,
      )?;

      let moved = dropped(&moved_window);

      if should_split {
        let split_container = SplitContainer::new(
          parent.tiling_direction().inverse(),
          config.value.gaps.clone(),
        );

        wrap_in_container(
          &split_container,
          &parent.clone().into(),
          &[target],
        )?;

        move_container_within_tree(
          &moved,
          &split_container.into(),
          usize::from(!position.is_before()),
          state,
        )?;
      } else {
        let target_index =
          target.index() + usize::from(!position.is_before());

        move_container_within_tree(
          &moved,
          &parent.clone().into(),
          target_index,
          state,
        )?;
      }

      state.pending_sync.queue_container_to_redraw(parent);

      Ok(moved_window)
    }
  }
}

/// Moves a dragged window into the stack whose tab bar it was dropped on.
/// A window of a stack brings its whole stack along.
///
/// Returns whether the window was dropped onto a tab bar.
fn drop_onto_tab_bar(
  window: &NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<bool> {
  let cursor = state.dispatcher.cursor_position()?;
  let own_stack = window.parent().and_then(|p| p.as_stack().cloned());

  let Some(target) = tab_bar_at(&cursor, own_stack.as_ref(), state) else {
    return Ok(false);
  };

  tracing::info!(
    "Window dropped onto a tab bar: {}",
    window.as_window_container()?
  );

  let moved = own_stack
    .map_or_else(|| vec![window.clone().into()], |stack| stack.windows());

  // Kept in their order, wherever they go among the tabs.
  let first_index =
    target.new_tab_index(config.value.stack.new_tab_position);

  for (offset, moved_window) in moved.into_iter().enumerate() {
    let is_dragged = moved_window.id() == window.id();
    moved_window.set_active_drag(None);

    let joined = join_stack(
      moved_window,
      &target,
      first_index + offset,
      state,
      config,
    )?;

    // A window taken in like this is never auto-stacked elsewhere.
    state.auto_stack.mark_settled(joined.native().id());

    if is_dragged {
      set_focused_descendant(&joined.into(), None);
    }
  }

  state
    .pending_sync
    .queue_containers_to_redraw(target.windows())
    .queue_focus_change();

  Ok(true)
}

#[cfg(test)]
mod tests {
  use std::mem::ManuallyDrop;

  use wm_common::{
    ActiveDrag, ActiveDragOperation, FloatingStateConfig, Layout,
    WindowState,
  };
  use wm_platform::{LengthValue, Point, Rect};

  use super::{
    cancel_drag, drop_as_tiling_window,
    handle_window_moved_or_resized_end, place_tiling_window,
  };
  #[cfg(target_os = "windows")]
  use crate::layout::window_drop::{window_drop, WindowDrop};
  use crate::{
    commands::{
      container::{attach_container, set_focused_descendant},
      window::{set_window_size, update_window_state},
    },
    events::handle_window_moved_or_resized::update_drag_state,
    layout::{is_hidden_in_layout, scrolling::sync_view},
    models::{
      Monitor, NonTilingWindow, StackContainer, TilingContainer,
      TilingWindow, WindowContainer, Workspace,
    },
    test_utils::{
      child_ids, mock_bounds, mock_config,
      mock_gapped_column as vertical_split, mock_gapped_window as window,
      mock_inner_gaps, mock_working_area, MOCK_MONITOR_WIDTH,
    },
    traits::{
      CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
    },
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// Default config, with the windows' gaps.
  fn config() -> UserConfig {
    let mut config = mock_config();
    config.value.gaps = mock_inner_gaps();
    config
  }

  /// A scrolling workspace of `columns`, each half the 1680px width.
  fn scrolling(columns: Vec<TilingContainer>) -> Workspace {
    Workspace::mock()
      .layout(Layout::Scrolling)
      .gaps_config(mock_inner_gaps())
      .tiling_containers(columns)
      .call()
  }

  /// Never dropped, which would restore its mock windows via Win32 calls.
  fn state(monitors: Vec<Monitor>) -> ManuallyDrop<WmState> {
    ManuallyDrop::new(WmState::mock(monitors))
  }

  /// A monitor displaying `workspace`, `x` pixels right of the origin.
  fn monitor(workspace: &Workspace, x: i32) -> Monitor {
    let offset =
      |rect: Rect| rect.translate_to_coordinates(rect.x() + x, rect.y());

    Monitor::mock()
      .bounds(offset(mock_bounds()))
      .working_area(offset(mock_working_area()))
      .workspaces(vec![workspace.clone()])
      .call()
  }

  /// Drags `window` far enough to float it, as an interactive move does.
  fn start_drag(
    window: &TilingWindow,
    state: &mut WmState,
    config: &UserConfig,
  ) -> NonTilingWindow {
    let frame = window.to_rect().unwrap();
    window.set_active_drag(Some(ActiveDrag {
      operation: None,
      is_from_floating: false,
      initial_position: frame.clone(),
    }));

    let moved = frame.translate_to_coordinates(frame.x() + 50, frame.y());
    update_drag_state(&window.clone().into(), &moved, state, config)
      .unwrap();

    state
      .container_by_id(window.id())
      .and_then(|container| NonTilingWindow::try_from(container).ok())
      .unwrap()
  }

  fn drop_at(
    window: &NonTilingWindow,
    x: i32,
    y: i32,
    state: &mut WmState,
    config: &UserConfig,
  ) -> WindowContainer {
    let dropped =
      drop_as_tiling_window(window, &Point { x, y }, state, config)
        .unwrap();
    dropped.set_active_drag(None);
    dropped
  }

  fn assert_size(container: &impl TilingSizeGetters, size: f32) {
    let actual = container.tiling_size();
    assert!((actual - size).abs() < 1e-3, "{actual} != {size}");
  }

  fn tiling(window: &WindowContainer) -> TilingContainer {
    window.as_tiling_container().unwrap()
  }

  #[test]
  fn ending_a_drag_on_a_column_keeps_its_width() {
    let (first, second, third) = (window(), window(), window());
    let workspace =
      scrolling(vec![first.clone().into(), second.into(), third.into()]);
    let mut state = state(vec![monitor(&workspace, 0)]);

    // A click on the title bar ends a drag with the window's own frame.
    for _ in 0..3 {
      let width = first.to_rect().unwrap().width();
      set_window_size(
        first.clone().into(),
        Some(LengthValue::from_px(width)),
        None,
        &mut state,
      )
      .unwrap();
    }

    assert_size(&first, 0.5);
    assert_eq!(first.to_rect().unwrap().width(), 830);

    set_window_size(
      first.clone().into(),
      Some(LengthValue::from_px(1680)),
      None,
      &mut state,
    )
    .unwrap();
    assert_size(&first, 1.);
  }

  #[test]
  fn a_column_dragged_to_the_strip_end_keeps_its_width() {
    let (first, moved, last) = (window(), window(), window());
    let workspace = scrolling(vec![
      first.clone().into(),
      moved.clone().into(),
      last.clone().into(),
    ]);
    first.set_tiling_size(0.3);
    moved.set_tiling_size(0.3);
    last.set_tiling_size(0.3);
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&moved.clone().into(), None);

    let floating = start_drag(&moved, &mut state, &config);
    assert_size(&first, 0.3);
    assert_size(&last, 0.3);

    // Past the strip's end, which the remaining columns leave at about
    // 60% of the workspace width.
    let dropped = drop_at(&floating, 1300, 500, &mut state, &config);

    assert_eq!(dropped.state(), WindowState::Tiling);
    assert_eq!(
      child_ids(&workspace),
      vec![first.id(), last.id(), moved.id()]
    );
    assert_size(&first, 0.3);
    assert_size(&last, 0.3);
    assert_size(&tiling(&dropped), 0.3);
  }

  #[test]
  fn a_row_dragged_onto_another_column_joins_it() {
    let (left, top, bottom) = (window(), window(), window());
    let column =
      vertical_split(vec![top.clone().into(), bottom.clone().into()]);
    let workspace =
      scrolling(vec![left.clone().into(), column.clone().into()]);
    left.set_tiling_size(0.4);
    column.set_tiling_size(0.7);
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&top.clone().into(), None);

    let floating = start_drag(&top, &mut state, &config);

    // The column left with a single row is that row again.
    assert!(column.is_detached());
    assert_eq!(child_ids(&workspace)[..2], [left.id(), bottom.id()]);
    assert_size(&bottom, 0.7);

    let dropped = drop_at(&floating, 100, 900, &mut state, &config);

    let new_column = left.parent().unwrap();
    assert_eq!(new_column.parent().unwrap().id(), workspace.id());
    assert_eq!(child_ids(&new_column), vec![left.id(), dropped.id()]);
    assert_size(&new_column.as_tiling_container().unwrap(), 0.4);
    assert_eq!(child_ids(&workspace), vec![new_column.id(), bottom.id()]);
    assert_size(&bottom, 0.7);
  }

  #[test]
  fn dragging_a_stacked_row_leaves_no_single_row_column() {
    let (left, tab, other_tab, bottom) =
      (window(), window(), window(), window());
    let stack = StackContainer::mock()
      .tiling_containers(vec![tab.clone().into(), other_tab.into()])
      .call();
    let column = vertical_split(vec![stack.into(), bottom.clone().into()]);
    let workspace = scrolling(vec![left.into(), column.clone().into()]);
    column.set_tiling_size(0.7);
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&tab.clone().into(), None);

    start_drag(&tab, &mut state, &config);

    assert!(column.is_detached());
    assert_eq!(bottom.parent().unwrap().id(), workspace.id());
    assert_size(&bottom, 0.7);
  }

  #[test]
  fn dragging_the_focused_column_keeps_the_strip_in_place() {
    let windows = [window(), window(), window(), window()];
    let workspace =
      scrolling(windows.iter().map(|w| w.clone().into()).collect());
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());

    set_focused_descendant(&windows[0].clone().into(), None);
    set_focused_descendant(&windows[3].clone().into(), None);
    sync_view(&workspace).unwrap();
    assert_eq!(workspace.scroll_offset(), 1700);

    start_drag(&windows[3], &mut state, &config);
    sync_view(&workspace).unwrap();

    // The column now in its place, left of the gap, stays where it was.
    assert_eq!(workspace.scroll_offset(), 1700);
    assert!(!is_hidden_in_layout(&windows[2]));
  }

  #[test]
  fn a_column_resized_by_its_left_edge_keeps_its_right_edge() {
    let (first, second, third) = (window(), window(), window());
    let workspace = scrolling(vec![
      first.clone().into(),
      second.clone().into(),
      third.clone().into(),
    ]);
    for column in [&first, &second, &third] {
      column.set_tiling_size(0.3);
    }
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());

    set_focused_descendant(&second.clone().into(), None);
    sync_view(&workspace).unwrap();

    // The window's left edge dragged 200px further left, which leaves
    // room on screen right of it.
    let initial = second.to_rect().unwrap();
    let resized = Rect::from_ltrb(
      initial.left - 200,
      initial.top,
      initial.right,
      initial.bottom,
    );
    second.set_active_drag(Some(ActiveDrag {
      operation: Some(ActiveDragOperation::Resize),
      is_from_floating: false,
      initial_position: initial.clone(),
    }));
    second.update_native_properties(|properties| {
      properties.frame = resized.clone();
    });

    handle_window_moved_or_resized_end(
      &second.clone().into(),
      &mut state,
      &config,
    )
    .unwrap();
    sync_view(&workspace).unwrap();

    let rect = second.to_rect().unwrap();
    assert_eq!(rect.right, initial.right);
    assert!((rect.left - resized.left).abs() <= 1, "{rect:?}");
  }

  #[test]
  fn a_window_partly_past_a_scrolling_workspace_stays_shown() {
    let workspace = scrolling(vec![window().into()]);
    let _state = state(vec![monitor(&workspace, 0)]);

    for window_state in [
      WindowState::Floating(FloatingStateConfig::default()),
      WindowState::Fullscreen(
        config().value.window_behavior.state_defaults.fullscreen,
      ),
    ] {
      let window = NonTilingWindow::mock()
        .state(window_state)
        .floating_placement(Rect::from_xy(-100, 100, 400, 300))
        .call();
      attach_container(
        &window.clone().into(),
        &workspace.clone().into(),
        None,
      )
      .unwrap();

      assert!(!is_hidden_in_layout(&window));
    }
  }

  #[test]
  fn a_floated_column_tiles_back_at_its_width() {
    let (first, moved, last) = (window(), window(), window());
    let workspace =
      scrolling(vec![first.into(), moved.clone().into(), last.into()]);
    moved.set_tiling_size(0.7);
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    let floating = WindowState::Floating(FloatingStateConfig::default());

    let moved =
      update_window_state(moved.into(), floating, &mut state, &config)
        .unwrap();
    attach_container(&window().into(), &workspace.clone().into(), None)
      .unwrap();

    let moved =
      update_window_state(moved, WindowState::Tiling, &mut state, &config)
        .unwrap();

    assert_eq!(moved.index(), 1);
    assert_size(&tiling(&moved), 0.7);
  }

  #[test]
  fn a_window_moves_between_split_and_scrolling_workspaces() {
    let (split_left, split_right) = (window(), window());
    let split = Workspace::mock()
      .name("2".into())
      .gaps_config(mock_inner_gaps())
      .tiling_containers(vec![
        split_left.clone().into(),
        split_right.clone().into(),
      ])
      .call();
    let column = window();
    let workspace = scrolling(vec![column.clone().into()]);
    column.set_tiling_size(0.6);
    let (mut state, config) = (
      state(vec![
        monitor(&workspace, 0),
        monitor(&split, MOCK_MONITOR_WIDTH),
      ]),
      config(),
    );

    // Into the scrolling workspace, as a new column past the strip's end.
    set_focused_descendant(&split_left.clone().into(), None);
    let floating = start_drag(&split_left, &mut state, &config);
    let dropped = drop_at(&floating, 1600, 500, &mut state, &config);

    assert_eq!(child_ids(&workspace), vec![column.id(), dropped.id()]);
    assert_size(&tiling(&dropped), 0.5);
    assert_size(&column, 0.6);
    assert_size(&split_right, 1.);

    // And back out, right of the other window.
    let dropped = dropped.as_container().as_tiling_window().cloned();
    let floating = start_drag(&dropped.unwrap(), &mut state, &config);
    let dropped = drop_at(
      &floating,
      MOCK_MONITOR_WIDTH + 1600,
      500,
      &mut state,
      &config,
    );

    assert_eq!(child_ids(&split), vec![split_right.id(), dropped.id()]);
    assert_size(&split_right, 0.5);
    assert_eq!(child_ids(&workspace), vec![column.id()]);
    assert_size(&column, 0.6);
  }
  #[test]
  fn a_cancelled_drag_puts_the_window_back_where_it_was() {
    let (moved, left, right) = (window(), window(), window());
    let workspace = Workspace::mock()
      .gaps_config(mock_inner_gaps())
      .tiling_containers(vec![
        moved.clone().into(),
        left.clone().into(),
        right.clone().into(),
      ])
      .call();
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&moved.clone().into(), None);

    let floating = start_drag(&moved, &mut state, &config);
    assert_eq!(child_ids(&workspace), vec![left.id(), right.id()]);

    cancel_drag(&floating.into(), &mut state, &config).unwrap();

    assert_eq!(
      child_ids(&workspace),
      vec![moved.id(), left.id(), right.id()]
    );
    let back = state.container_by_id(moved.id()).unwrap();
    assert!(back.as_tiling_window().is_some(), "tiling again");
    assert!(back.as_window_container().unwrap().active_drag().is_none());
  }

  #[test]
  fn placing_a_window_puts_it_where_dropping_it_there_would() {
    let (moved, left, right) = (window(), window(), window());
    let workspace = Workspace::mock()
      .gaps_config(mock_inner_gaps())
      .tiling_containers(vec![
        moved.clone().into(),
        left.clone().into(),
        right.clone().into(),
      ])
      .call();
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&moved.clone().into(), None);

    // Near the right edge of the right window.
    let rect = right.to_rect().unwrap();
    let point = Point {
      x: rect.right - 10,
      y: rect.center_point().y,
    };
    place_tiling_window(&moved, &workspace, &point, &mut state, &config)
      .unwrap();

    assert_eq!(
      child_ids(&workspace),
      vec![left.id(), right.id(), moved.id()]
    );
    let placed = state.container_by_id(moved.id()).unwrap();
    assert!(placed.as_tiling_window().is_some(), "tiling again");
  }

  #[cfg(target_os = "windows")]
  #[test]
  fn the_drop_indicator_shows_the_half_a_window_drops_beside() {
    let (left, right, moved) = (window(), window(), window());
    let workspace = Workspace::mock()
      .gaps_config(mock_inner_gaps())
      .tiling_containers(vec![
        left.clone().into(),
        right.clone().into(),
        moved.clone().into(),
      ])
      .call();
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&moved.clone().into(), None);

    let floating = start_drag(&moved, &mut state, &config);

    // Near the right edge of the right window.
    let rect = right.to_rect().unwrap();
    let point = Point {
      x: rect.right - 10,
      y: rect.center_point().y,
    };

    let drop = window_drop(&floating, &point, &state).unwrap().unwrap();
    assert!(matches!(drop, WindowDrop::Beside { .. }));
    assert_eq!(
      drop.indicator_rect().unwrap(),
      Rect::from_ltrb(
        rect.center_point().x,
        rect.top,
        rect.right,
        rect.bottom
      )
    );

    let dropped =
      drop_at(&floating, point.x, point.y, &mut state, &config);
    assert_eq!(
      child_ids(&workspace),
      vec![left.id(), right.id(), dropped.id()]
    );
  }

  #[cfg(target_os = "windows")]
  #[test]
  fn the_drop_indicator_shows_a_new_column_past_the_strip_end() {
    let (first, moved, last) = (window(), window(), window());
    let workspace = scrolling(vec![
      first.clone().into(),
      moved.clone().into(),
      last.clone().into(),
    ]);
    first.set_tiling_size(0.3);
    moved.set_tiling_size(0.3);
    last.set_tiling_size(0.3);
    let (mut state, config) =
      (state(vec![monitor(&workspace, 0)]), config());
    set_focused_descendant(&moved.clone().into(), None);

    let floating = start_drag(&moved, &mut state, &config);

    let point = Point { x: 1300, y: 500 };
    let drop = window_drop(&floating, &point, &state).unwrap().unwrap();
    let rect = last.to_rect().unwrap();

    assert!(matches!(drop, WindowDrop::Column { .. }));
    assert_eq!(
      drop.indicator_rect().unwrap(),
      Rect::from_ltrb(
        rect.center_point().x,
        rect.top,
        rect.right,
        rect.bottom
      )
    );

    drop_at(&floating, point.x, point.y, &mut state, &config);
    assert_eq!(
      child_ids(&workspace),
      vec![first.id(), last.id(), moved.id()]
    );
  }
}
