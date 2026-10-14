use wm_common::{ColumnDisplay, TilingDirection};

use super::{unwrap_single_row, ColumnPlace};
use crate::{
  commands::{
    container::{
      flatten_stack_container, move_container_within_tree,
      set_focused_descendant, wrap_in_split_container,
      wrap_in_stack_container,
    },
    window::{join_stack, new_stack},
  },
  models::{Container, SplitContainer, TilingContainer},
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Shows the windows of the column holding `container` as `display` says,
/// or switches between the two when `display` is `None`: stacked one
/// above the other, or as the tabs of a stack, top to bottom.
///
/// Does nothing outside a scrolling layout, or for a column of a single
/// window, which has nothing to show either way.
pub fn set_column_display(
  container: &Container,
  display: Option<ColumnDisplay>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(place) = ColumnPlace::of(container) else {
    return Ok(());
  };

  let current = match &place.column {
    TilingContainer::Stack(_) => ColumnDisplay::Tabbed,
    _ => ColumnDisplay::Normal,
  };

  let display = display.unwrap_or(match current {
    ColumnDisplay::Normal => ColumnDisplay::Tabbed,
    ColumnDisplay::Tabbed => ColumnDisplay::Normal,
  });

  if display == current {
    return Ok(());
  }

  let focused = state.focused_container();

  match &place.column {
    TilingContainer::Stack(stack) => {
      let split = SplitContainer::new(
        TilingDirection::Vertical,
        config.value.gaps.clone(),
      );
      wrap_in_split_container(
        &split,
        &place.workspace.clone().into(),
        std::slice::from_ref(&place.column),
      )?;

      // Every tab but the last becomes a row above what is left of the
      // stack, which then is its last tab.
      let windows = stack.windows();
      for (index, window) in windows.iter().rev().skip(1).rev().enumerate()
      {
        move_container_within_tree(
          &window.clone().into(),
          &split.clone().into(),
          index,
          state,
        )?;
      }

      if !stack.is_detached() {
        flatten_stack_container(stack.clone())?;
      }
    }
    TilingContainer::Split(split)
      if split.tiling_direction() == TilingDirection::Vertical =>
    {
      let windows = split
        .descendants()
        .filter_map(|descendant| descendant.as_tiling_window().cloned())
        .collect::<Vec<_>>();

      let Some(first) = windows.first() else {
        return Ok(());
      };

      let stack = new_stack(config);
      let first_parent = first
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Window has no parent."))?;
      wrap_in_stack_container(
        &stack,
        &first_parent,
        &[first.clone().into()],
      )?;

      for window in windows.iter().skip(1) {
        join_stack(
          window.clone().into(),
          &stack,
          stack.child_count(),
          state,
          config,
        )?;
      }

      // The stack, alone in the column's split now, is the column.
      unwrap_single_row(&place.column)?;
    }
    _ => return Ok(()),
  }

  if let Some(focused) = focused.filter(|focused| !focused.is_detached()) {
    set_focused_descendant(&focused, None);
  }

  state
    .pending_sync
    .queue_container_to_redraw(place.workspace)
    .queue_focus_change()
    .queue_tab_bar_update();

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{ColumnDisplay, Layout, TilingDirection};

  use super::set_column_display;
  use crate::{
    commands::container::set_focused_descendant,
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::{child_ids, mock_config},
    traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  #[test]
  fn switches_a_column_between_rows_and_tabs() {
    let (top, middle, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![
        top.clone().into(),
        middle.clone().into(),
        bottom.clone().into(),
      ])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.clone().into()])
      .call();
    column.set_tiling_size(0.6);
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&middle.clone().into(), None);
    let config = mock_config();

    set_column_display(&middle.clone().into(), None, &mut state, &config)
      .unwrap();

    let stack = middle.parent().unwrap();
    assert!(stack.as_stack().is_some());
    assert_eq!(child_ids(&workspace), vec![stack.id()]);
    assert_eq!(
      child_ids(&stack),
      vec![top.id(), middle.id(), bottom.id()]
    );
    let width = stack.as_tiling_container().unwrap().tiling_size();
    assert!((width - 0.6).abs() < 1e-4, "keeps its width");
    assert_eq!(
      state.focused_container().map(|focused| focused.id()),
      Some(middle.id())
    );

    set_column_display(
      &middle.clone().into(),
      Some(ColumnDisplay::Normal),
      &mut state,
      &config,
    )
    .unwrap();

    let split = middle.parent().unwrap();
    let split = split.as_split().unwrap();
    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(child_ids(split), vec![top.id(), middle.id(), bottom.id()]);
    assert!((split.tiling_size() - 0.6).abs() < 1e-4);
  }

  #[test]
  fn a_lone_window_stays_as_it_is() {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![window.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);

    set_column_display(
      &window.clone().into(),
      None,
      &mut state,
      &mock_config(),
    )
    .unwrap();
    assert_eq!(child_ids(&workspace), vec![window.id()]);
  }
}
