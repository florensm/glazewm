use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::LengthUnit;

use super::{
  presets::{nearest_preset, step_preset},
  row_of, ColumnPlace,
};
use crate::{
  commands::container::resize_tiling_container,
  models::{Container, TilingContainer},
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters, MIN_TILING_SIZE,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// The row of a column split into rows that holds `window`, and the split.
fn split_row(
  window: &Container,
) -> Option<(TilingContainer, TilingContainer)> {
  let place = ColumnPlace::of(window)?;

  let is_split = matches!(
    &place.column,
    TilingContainer::Split(split)
      if split.tiling_direction() == TilingDirection::Vertical
        && split.tiling_children().nth(1).is_some()
  );

  is_split
    .then(|| row_of(window, &place.column))
    .flatten()
    .map(|row| (row, place.column))
}

/// Sets the height of the row of its column that `window` is in to the
/// next of `layouts.scrolling.preset_window_heights`, or with `forwards`
/// unset the previous one, wrapping around. The other rows share what is
/// left.
///
/// Does nothing outside a scrolling layout's column of several rows.
pub fn cycle_window_height(
  window: &Container,
  forwards: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some((row, column)) = split_row(window) else {
    return Ok(());
  };

  let workspace = column.workspace().context("No workspace.")?;
  let column_height = column.to_rect()?.height();
  let (_, gap) = row.inner_gaps()?;

  // The rows share the column's height less the gaps between them, and
  // each keeps some of it.
  #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
  let rows = column.tiling_children().count() as i32;
  let available = (column_height - gap * (rows - 1)).max(1);
  #[allow(clippy::cast_precision_loss)]
  let max_size =
    (1. - MIN_TILING_SIZE * (rows - 1) as f32).max(MIN_TILING_SIZE);

  // Presets are of the workspace height, sized like column widths so
  // that ones adding up to 100% fill it, gaps included.
  #[allow(clippy::cast_precision_loss)]
  let sizes = config
    .value
    .layouts
    .scrolling
    .preset_window_heights
    .iter()
    .map(|preset| {
      let height = match preset.unit {
        LengthUnit::Percentage => {
          (column_height + gap) as f32 * preset.amount - gap as f32
        }
        LengthUnit::Pixel => preset.amount,
      };
      (height / available as f32).clamp(MIN_TILING_SIZE, max_size)
    })
    .collect::<Vec<_>>();

  if sizes.is_empty() {
    return Ok(());
  }

  let current = row.tiling_size();
  let preset =
    workspace
      .strip_view()
      .preset_height(row.id(), current, sizes.len());
  let index = if let Some(index) = preset {
    step_preset(index, sizes.len(), forwards)
  } else {
    // A pixel's worth of the column.
    #[allow(clippy::cast_precision_loss)]
    let leeway = 1. / available as f32;
    nearest_preset(&sizes, current, leeway, forwards)
  };

  resize_tiling_container(&row, sizes[index]);
  workspace.update_strip_view(|view| {
    view.set_preset_height(row.id(), index, row.tiling_size());
  });
  state.pending_sync.queue_container_to_redraw(column);

  Ok(())
}

/// Gives the row of its column that `window` is in an equal share of the
/// column's height again, the other rows sharing the rest as before.
///
/// Does nothing outside a scrolling layout's column of several rows.
pub fn reset_window_height(window: &Container, state: &mut WmState) {
  let Some((row, column)) = split_row(window) else {
    return;
  };

  #[allow(clippy::cast_precision_loss)]
  let share = 1. / column.tiling_children().count() as f32;
  resize_tiling_container(&row, share);
  state.pending_sync.queue_container_to_redraw(column);
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};

  use super::{cycle_window_height, reset_window_height};
  use crate::{
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    test_utils::mock_config,
    traits::{PositionGetters, TilingSizeGetters},
    wm_state::WmState,
  };

  fn setup() -> (WmState, Workspace, TilingWindow, TilingWindow) {
    let (top, bottom) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.into()])
      .call();
    let state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);

    (state, workspace, top, bottom)
  }

  fn height_share(window: &TilingWindow, workspace: &Workspace) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let share = window.to_rect().unwrap().height() as f32
      / workspace.to_rect().unwrap().height() as f32;
    share
  }

  #[test]
  fn cycles_a_window_through_the_preset_heights() {
    let (mut state, workspace, top, bottom) = setup();
    let config = mock_config();

    // From half the column, the next preset is two thirds.
    cycle_window_height(&top.clone().into(), true, &mut state, &config)
      .unwrap();
    assert!((height_share(&top, &workspace) - 2. / 3.).abs() < 0.01);
    assert!((height_share(&bottom, &workspace) - 1. / 3.).abs() < 0.01);

    cycle_window_height(&top.clone().into(), true, &mut state, &config)
      .unwrap();
    assert!((height_share(&top, &workspace) - 1. / 3.).abs() < 0.01);

    reset_window_height(&top.clone().into(), &mut state);
    assert!((top.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn a_lone_window_has_no_height_to_set() {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![window.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);
    let size = window.tiling_size();

    cycle_window_height(
      &window.clone().into(),
      true,
      &mut state,
      &mock_config(),
    )
    .unwrap();
    assert!((window.tiling_size() - size).abs() < 1e-6);
  }
}
