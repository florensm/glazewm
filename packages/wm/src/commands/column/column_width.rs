use wm_platform::LengthValue;

use super::presets::{nearest_preset, step_preset};
use crate::{
  commands::container::resize_tiling_container,
  layout::scrolling::{
    column_of, column_size, width_at, Strip, ViewRequest, MIN_COLUMN_SIZE,
  },
  models::Container,
  traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Sets the width of the column holding `container` to the next of
/// `layouts.scrolling.preset_column_widths`, or with `forwards` unset the
/// previous one, wrapping around.
///
/// A column last set by this goes on from that preset. Any other column
/// goes to the first preset wider than it (or, backwards, the last one
/// narrower).
///
/// Does nothing outside a scrolling layout.
pub fn cycle_column_width(
  container: &Container,
  forwards: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some((column, workspace)) = column_of(container) else {
    return Ok(());
  };

  // Clamped as `resize_tiling_container` would, so a preset past the
  // limits can't leave the column stuck short of it.
  let sizes = config
    .value
    .layouts
    .scrolling
    .preset_column_widths
    .iter()
    .map(|width| {
      column_size(&column, &workspace, width)
        .map(|size| size.clamp(MIN_COLUMN_SIZE, 1.))
    })
    .collect::<anyhow::Result<Vec<_>>>()?;

  if sizes.is_empty() {
    return Ok(());
  }

  let current = column.tiling_size();
  let preset =
    workspace
      .strip_view()
      .preset_width(column.id(), current, sizes.len());
  let index = if let Some(index) = preset {
    step_preset(index, sizes.len(), forwards)
  } else {
    let width = |size: f32| width_at(&column, &workspace, size);
    let widths = sizes
      .iter()
      .map(|size| width(*size))
      .collect::<Result<Vec<_>, _>>()?;

    nearest_preset(&widths, width(current)?, 1, forwards)
  };

  resize_tiling_container(&column, sizes[index]);
  workspace.update_strip_view(|view| {
    view.set_preset_width(column.id(), index, column.tiling_size());
  });
  state.pending_sync.queue_container_to_redraw(workspace);

  Ok(())
}

/// Makes the column holding `container` as wide as the workspace, or back
/// to the width it had before.
///
/// Does nothing outside a scrolling layout.
pub fn maximize_column(container: &Container, state: &mut WmState) {
  let Some((column, workspace)) = column_of(container) else {
    return;
  };

  let restored = workspace
    .update_strip_view(|view| view.take_size_before_maximize(column.id()));

  match restored {
    Some(size) if (column.tiling_size() - 1.).abs() < 1e-6 => {
      resize_tiling_container(&column, size);
    }
    _ => {
      workspace.update_strip_view(|view| {
        view.set_size_before_maximize(column.id(), column.tiling_size());
      });
      resize_tiling_container(&column, 1.);
    }
  }

  state.pending_sync.queue_container_to_redraw(workspace);
}

/// Widens the column holding `container` to fill the room the columns
/// fully on screen leave, then scrolls to show them all from the left
/// edge.
///
/// A column alone on screen is maximized instead, as is one kept
/// centered. Does nothing outside a scrolling layout, for a column not
/// fully on screen, or when there is no room left.
pub fn expand_column_to_available_width(
  container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some((column, workspace)) = column_of(container) else {
    return Ok(());
  };

  if (column.tiling_size() - 1.).abs() < 1e-6 {
    return Ok(());
  }

  let strip = Strip::new(&workspace)?;
  if workspace.scrolling_config().keeps_centered(strip.len()) {
    maximize_column(container, state);
    return Ok(());
  }

  let Some(index) = strip.index_of(column.id()) else {
    return Ok(());
  };
  let Some(visible) = strip
    .fully_visible(workspace.scroll_offset())
    .filter(|visible| visible.contains(&index))
  else {
    return Ok(());
  };

  let (gap, _) = column.inner_gaps()?;
  let taken = visible
    .clone()
    .map(|other| strip.widths[other] + gap)
    .sum::<i32>();
  let room = strip.view_width + gap - taken;

  if room <= 0 {
    return Ok(());
  }

  if visible == (index..=index) {
    maximize_column(container, state);
    return Ok(());
  }

  let width = LengthValue::from_px(strip.widths[index] + room);
  let size = column_size(&column, &workspace, &width)?;
  resize_tiling_container(&column, size.clamp(MIN_COLUMN_SIZE, 1.));

  workspace.request_view(ViewRequest::PlaceActive {
    x: strip.start(index) - strip.start(*visible.start()),
  });
  state.pending_sync.queue_container_to_redraw(workspace);

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::Layout;
  use wm_platform::{LengthUnit, LengthValue};

  use super::{
    cycle_column_width, expand_column_to_available_width, maximize_column,
  };
  use crate::{
    commands::container::set_focused_descendant,
    layout::scrolling::sync_view,
    models::{Monitor, TilingWindow, Workspace},
    test_utils::{mock_config, MOCK_MONITOR_WIDTH},
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// A scrolling workspace with one column of `size`.
  fn setup(size: f32) -> (WmState, TilingWindow) {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![window.clone().into()])
      .call();
    window.set_tiling_size(size);
    let monitor = Monitor::mock().workspaces(vec![workspace]).call();

    (WmState::mock(vec![monitor]), window)
  }

  fn cycle(
    window: &TilingWindow,
    forwards: bool,
    state: &mut WmState,
    config: &UserConfig,
  ) {
    cycle_column_width(&window.clone().into(), forwards, state, config)
      .unwrap();
  }

  fn assert_size(window: &TilingWindow, size: f32) {
    let actual = window.tiling_size();
    assert!((actual - size).abs() < 1e-4, "{actual} != {size}");
  }

  #[test]
  fn cycles_through_the_presets_and_wraps_around() {
    let (mut state, window) = setup(0.5);
    let config = mock_config();

    cycle(&window, true, &mut state, &config);
    assert_size(&window, 2. / 3.);
    cycle(&window, true, &mut state, &config);
    assert_size(&window, 1. / 3.);
    cycle(&window, true, &mut state, &config);
    assert_size(&window, 0.5);
  }

  #[test]
  fn cycles_backwards_too() {
    let (mut state, window) = setup(0.5);
    let config = mock_config();

    cycle(&window, false, &mut state, &config);
    assert_size(&window, 1. / 3.);
    cycle(&window, false, &mut state, &config);
    assert_size(&window, 2. / 3.);
  }

  #[test]
  fn a_column_between_presets_goes_to_the_next_wider_one() {
    let (mut state, window) = setup(0.4);
    let config = mock_config();

    cycle(&window, true, &mut state, &config);
    assert_size(&window, 0.5);
  }

  #[test]
  fn presets_in_pixels_and_out_of_order_work_too() {
    let (mut state, window) = setup(0.5);
    let mut config = mock_config();
    config.value.layouts.scrolling.preset_column_widths = vec![
      LengthValue::from_px(MOCK_MONITOR_WIDTH),
      LengthValue::from_px(400),
    ];

    cycle(&window, true, &mut state, &config);
    assert_eq!(
      window.to_rect().unwrap().width(),
      window.workspace().unwrap().to_rect().unwrap().width()
    );

    cycle(&window, true, &mut state, &config);
    assert_eq!(window.to_rect().unwrap().width(), 400);
  }

  #[test]
  fn presets_reloaded_to_fewer_go_on_by_size() {
    let (mut state, window) = setup(0.5);
    let mut config = mock_config();
    let percent = |amount| LengthValue {
      amount,
      unit: LengthUnit::Percentage,
    };
    config.value.layouts.scrolling.preset_column_widths =
      vec![percent(0.25), percent(0.5), percent(0.75), percent(1.)];

    cycle(&window, true, &mut state, &config);
    cycle(&window, true, &mut state, &config);
    assert_size(&window, 1.);

    // The last preset it was set to is no more.
    config.value.layouts.scrolling.preset_column_widths =
      vec![percent(0.5), percent(1.)];
    cycle(&window, false, &mut state, &config);
    assert_size(&window, 0.5);
  }

  #[test]
  fn does_nothing_outside_a_scrolling_layout() {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![window.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace])
      .call()]);

    cycle(&window, true, &mut state, &mock_config());
    assert_size(&window, 1.);
  }

  #[test]
  fn maximizing_toggles_full_width_and_back() {
    let (mut state, window) = setup(0.4);

    maximize_column(&window.clone().into(), &mut state);
    assert_size(&window, 1.);
    maximize_column(&window.clone().into(), &mut state);
    assert_size(&window, 0.4);
  }

  #[test]
  fn expanding_fills_the_room_left_on_screen() {
    let (first, second) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    first.set_tiling_size(0.3);
    second.set_tiling_size(0.3);
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let mut state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);
    set_focused_descendant(&second.clone().into(), None);
    sync_view(&workspace).unwrap();

    expand_column_to_available_width(&second.clone().into(), &mut state)
      .unwrap();
    sync_view(&workspace).unwrap();

    let (rect, first_rect, second_rect) = (
      workspace.to_rect().unwrap(),
      first.to_rect().unwrap(),
      second.to_rect().unwrap(),
    );
    assert_eq!(first_rect.x(), rect.x());
    assert_eq!(
      second_rect.x() + second_rect.width(),
      rect.x() + rect.width()
    );
  }
}
