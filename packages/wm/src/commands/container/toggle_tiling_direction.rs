use anyhow::Context;
use wm_common::{Layout, TilingDirection, WmEvent};

use super::{flatten_split_container, wrap_in_split_container};
use crate::{
  models::{Container, DirectionContainer, SplitContainer, TilingWindow},
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Does nothing in a scrolling layout, whose columns always run left to
/// right and the windows in a column top to bottom.
pub fn toggle_tiling_direction(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if container
    .workspace()
    .is_some_and(|workspace| workspace.layout() == Layout::Scrolling)
  {
    return Ok(());
  }

  let direction_container = match container {
    Container::TilingWindow(tiling_window) => {
      toggle_window_direction(tiling_window, config)
    }
    Container::Workspace(workspace) => {
      workspace
        .set_tiling_direction(workspace.tiling_direction().inverse());

      Ok(workspace.into())
    }
    // Can only toggle tiling direction from a tiling window or workspace.
    _ => return Ok(()),
  }?;

  state.emit_event(WmEvent::TilingDirectionChanged {
    direction_container: direction_container.to_dto()?,
    new_tiling_direction: direction_container.tiling_direction(),
  });

  Ok(())
}

fn toggle_window_direction(
  tiling_window: TilingWindow,
  config: &UserConfig,
) -> anyhow::Result<DirectionContainer> {
  let parent = tiling_window
    .direction_container()
    .context("No direction container.")?;

  // If the window is an only child, then either change the tiling
  // direction of its parent workspace or flatten its parent split
  // container.
  if tiling_window.tiling_siblings().count() == 0 {
    return match parent {
      DirectionContainer::Workspace(workspace) => {
        workspace
          .set_tiling_direction(workspace.tiling_direction().inverse());

        Ok(workspace.into())
      }
      DirectionContainer::Split(split_container) => {
        flatten_split_container(split_container.clone())?;

        tiling_window
          .direction_container()
          .context("No direction container.")
      }
    };
  }

  // Create a new split container to wrap the window.
  let split_container = SplitContainer::new(
    parent.tiling_direction().inverse(),
    config.value.gaps.clone(),
  );

  wrap_in_split_container(
    &split_container,
    &parent.into(),
    &[tiling_window.into()],
  )?;

  Ok(split_container.into())
}

pub fn set_tiling_direction(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
  tiling_direction: &TilingDirection,
) -> anyhow::Result<()> {
  let direction_container = container
    .direction_container()
    .context("No direction container.")?;

  if direction_container.tiling_direction() == *tiling_direction {
    Ok(())
  } else {
    toggle_tiling_direction(container, state, config)
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};

  use super::{set_tiling_direction, toggle_tiling_direction};
  use crate::{
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::{child_ids, mock_config},
    traits::CommonGetters,
    wm_state::WmState,
  };

  #[test]
  fn does_nothing_in_a_scrolling_layout() {
    let (single, top, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![
        single.clone().into(),
        column.clone().into(),
      ])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    let config = mock_config();

    toggle_tiling_direction(single.clone().into(), &mut state, &config)
      .unwrap();
    toggle_tiling_direction(top.clone().into(), &mut state, &config)
      .unwrap();
    set_tiling_direction(
      bottom.clone().into(),
      &mut state,
      &config,
      &TilingDirection::Horizontal,
    )
    .unwrap();

    assert_eq!(child_ids(&workspace), vec![single.id(), column.id()]);
    assert_eq!(child_ids(&column), vec![top.id(), bottom.id()]);
  }
}
