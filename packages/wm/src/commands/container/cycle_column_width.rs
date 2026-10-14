use super::resize_tiling_container;
use crate::{
  layout::scrolling::{column_of, column_size, MIN_COLUMN_SIZE},
  models::Container,
  traits::TilingSizeGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// How much wider than a column a preset has to be to count as the next
/// one, so float noise can't keep a column on its current preset.
const PRESET_TOLERANCE: f32 = 0.001;

/// Sets the width of the column holding `container` to the next of
/// `layouts.scrolling.preset_column_widths`, from narrowest to widest and
/// back around. A column between presets goes to the next wider one.
///
/// Does nothing outside a scrolling layout.
pub fn cycle_column_width(
  container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some((column, workspace)) = column_of(container) else {
    return Ok(());
  };

  // Clamped as `resize_tiling_container` would, so a preset past the
  // limits can't leave the column stuck short of it.
  let mut sizes = config
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
  sizes.sort_by(f32::total_cmp);

  let current = column.tiling_size();
  let Some(size) = sizes
    .iter()
    .find(|size| **size > current + PRESET_TOLERANCE)
    .or(sizes.first())
  else {
    return Ok(());
  };

  resize_tiling_container(&column, *size);
  state.pending_sync.queue_container_to_redraw(workspace);

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::Layout;
  use wm_platform::LengthValue;

  use super::cycle_column_width;
  use crate::{
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
    state: &mut WmState,
    config: &UserConfig,
  ) {
    cycle_column_width(&window.clone().into(), state, config).unwrap();
  }

  fn assert_size(window: &TilingWindow, size: f32) {
    let actual = window.tiling_size();
    assert!((actual - size).abs() < 1e-4, "{actual} != {size}");
  }

  #[test]
  fn cycles_through_the_presets_and_wraps_around() {
    let (mut state, window) = setup(0.5);
    let config = mock_config();

    cycle(&window, &mut state, &config);
    assert_size(&window, 2. / 3.);
    cycle(&window, &mut state, &config);
    assert_size(&window, 1. / 3.);
    cycle(&window, &mut state, &config);
    assert_size(&window, 0.5);
  }

  #[test]
  fn a_column_between_presets_goes_to_the_next_wider_one() {
    let (mut state, window) = setup(0.4);
    let config = mock_config();

    cycle(&window, &mut state, &config);
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

    cycle(&window, &mut state, &config);
    assert_eq!(
      window.to_rect().unwrap().width(),
      window.workspace().unwrap().to_rect().unwrap().width()
    );

    cycle(&window, &mut state, &config);
    assert_eq!(window.to_rect().unwrap().width(), 400);
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

    cycle(&window, &mut state, &mock_config());
    assert_size(&window, 1.);
  }
}
