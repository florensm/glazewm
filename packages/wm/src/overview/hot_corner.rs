//! Opening the overview by resting the cursor in a corner of a monitor.

use std::time::{Duration, Instant};

use wm_common::HotCorner;
use wm_platform::{MouseButton, MouseEvent, Point, Rect};

use super::{to_layout_mode, toggle_overview};
use crate::{user_config::UserConfig, wm_state::WmState};

/// Where the cursor is with respect to the hot corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CornerVisit {
  /// Out of it.
  Away,

  /// In it, and opening the overview at the instant given if it stays.
  Resting(Instant),

  /// In it, the overview opened or not to be opened on this visit.
  Spent,
}

/// Pixels from a monitor's corner, either way, that count as in it.
const HOT_CORNER_SIZE: i32 = 2;

/// Notes the cursor moving into or out of the hot corner of a monitor,
/// opening the overview once it rests there for `hot_corner.delay_ms`.
pub fn check_hot_corner(
  event: &MouseEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let MouseEvent::Move { position, .. } = event else {
    return Ok(());
  };

  let corner = config.value.overview.hot_corner.corner;
  let is_in = corner != HotCorner::None
    && !state.is_paused
    && state.monitors().iter().any(|monitor| {
      is_in_corner(&monitor.native_properties().bounds, corner, position)
    });

  let visit = &mut state.overview.hot_corner;
  match (*visit, is_in) {
    (_, false) => *visit = CornerVisit::Away,
    (CornerVisit::Away, true) => {
      let delay = u64::from(config.value.overview.hot_corner.delay_ms);
      *visit = CornerVisit::Resting(
        Instant::now() + Duration::from_millis(delay),
      );
    }
    _ => {}
  }

  process_hot_corner(state, config)
}

/// Opens the overview once the cursor has rested in the hot corner long
/// enough, unless a mouse button is held, e.g. dragging a window there.
pub fn process_hot_corner(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let CornerVisit::Resting(due) = state.overview.hot_corner else {
    return Ok(());
  };
  if Instant::now() < due {
    return Ok(());
  }
  state.overview.hot_corner = CornerVisit::Spent;

  let is_button_held = [MouseButton::Left, MouseButton::Right]
    .iter()
    .any(|button| state.dispatcher.is_mouse_down(button));
  if is_button_held || state.overview.is_shown() {
    return Ok(());
  }

  let layout = to_layout_mode(config.value.overview.layout);
  toggle_overview(state, config, layout)
}

/// Whether `position` is in `corner` of `bounds`.
fn is_in_corner(
  bounds: &Rect,
  corner: HotCorner,
  position: &Point,
) -> bool {
  let near = |edge: i32, at: i32| (at - edge).abs() < HOT_CORNER_SIZE;
  let (left, top) = (bounds.left, bounds.top);
  let (right, bottom) = (bounds.right - 1, bounds.bottom - 1);

  // Not `Rect::contains_point`, which counts `bounds.right` and `bottom`
  // as in: those are the first pixels of a neighbouring monitor.
  (left..=right).contains(&position.x)
    && (top..=bottom).contains(&position.y)
    && match corner {
      HotCorner::None => false,
      HotCorner::TopLeft => {
        near(left, position.x) && near(top, position.y)
      }
      HotCorner::TopRight => {
        near(right, position.x) && near(top, position.y)
      }
      HotCorner::BottomLeft => {
        near(left, position.x) && near(bottom, position.y)
      }
      HotCorner::BottomRight => {
        near(right, position.x) && near(bottom, position.y)
      }
    }
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  #[test]
  fn hot_corners_are_the_monitors_own_corners() {
    use wm_common::HotCorner;
    use wm_platform::Point;

    use super::is_in_corner;

    let bounds = Rect::from_xy(1920, 0, 1920, 1080);
    let at = |x, y| Point { x, y };

    assert!(is_in_corner(&bounds, HotCorner::TopLeft, &at(1920, 0)));
    assert!(is_in_corner(&bounds, HotCorner::TopLeft, &at(1921, 1)));
    assert!(!is_in_corner(&bounds, HotCorner::TopLeft, &at(1925, 0)));
    assert!(!is_in_corner(&bounds, HotCorner::TopLeft, &at(1919, 0)));
    assert!(is_in_corner(
      &bounds,
      HotCorner::BottomRight,
      &at(3839, 1079)
    ));
    assert!(!is_in_corner(
      &bounds,
      HotCorner::BottomRight,
      &at(3840, 1079)
    ));
    assert!(!is_in_corner(&bounds, HotCorner::None, &at(1920, 0)));
  }
}
