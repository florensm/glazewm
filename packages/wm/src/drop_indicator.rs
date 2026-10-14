//! Highlight of where a dragged window will land, shown while it's
//! dragged.

use anyhow::Context;
use wm_common::ActiveDragOperation;
use wm_platform::{NativeDropIndicator, NativeWindowWindowsExt, Rect};

use crate::{
  layout::window_drop::{tab_bar_at, window_drop},
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Shows where `window` lands if dropped now, or hides the indicator when
/// it would not land anywhere in the tiling layout.
pub fn sync_drop_indicator(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) {
  let preview =
    drop_preview(window, state, config).unwrap_or_else(|err| {
      tracing::warn!("Failed to get drop target: {err:?}");
      None
    });

  let Some((rect, viewport)) = preview else {
    hide_drop_indicator(state);
    return;
  };

  // Styled once per drag rather than on every move, since resolving a
  // color from a palette file reads the file.
  let is_revealing = !state
    .drop_indicator
    .as_ref()
    .is_some_and(NativeDropIndicator::is_visible);

  if is_revealing {
    let effects = &config.value.window_effects.focused_window;
    let color = config
      .value
      .window_behavior
      .drop_indicator
      .color
      .as_ref()
      .unwrap_or(&effects.border.color)
      .resolve();
    let corner_radius = effects.window_corner_radius_px();

    match &mut state.drop_indicator {
      Some(indicator) => indicator.set_style(color, corner_radius),
      None => {
        match NativeDropIndicator::create(&viewport, color, corner_radius)
        {
          Ok(indicator) => state.drop_indicator = Some(indicator),
          Err(err) => {
            tracing::warn!("Failed to create drop indicator: {err}");
            return;
          }
        }
      }
    }
  }

  if let Some(indicator) = &mut state.drop_indicator {
    indicator.show(&rect, &viewport, window.native().hwnd());
  }
}

/// Hides the drop indicator, e.g. once the drag has ended.
pub fn hide_drop_indicator(state: &mut WmState) {
  if let Some(indicator) = &mut state.drop_indicator {
    indicator.hide();
  }
}

/// Where `window` lands if dropped now, and the rect of the monitor that
/// is on.
fn drop_preview(
  window: &WindowContainer,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<Option<(Rect, Rect)>> {
  // Over the drag shelf, the window goes to a workspace, not a spot.
  if !config.value.window_behavior.drop_indicator.enabled
    || state.overview.is_drag_shelf_open()
  {
    return Ok(None);
  }

  let WindowContainer::NonTilingWindow(window) = window else {
    return Ok(None);
  };

  let Some(drag) = window.active_drag() else {
    return Ok(None);
  };

  if !matches!(drag.operation, Some(ActiveDragOperation::Move)) {
    return Ok(None);
  }

  let cursor = state.dispatcher.cursor_position()?;
  let own_stack = window
    .parent()
    .and_then(|parent| parent.as_stack().cloned());

  // In the drop's own order: a tab bar takes any dragged window, and only
  // a window floated for the drag goes back into tiling.
  let (rect, monitor) =
    if let Some(stack) = tab_bar_at(&cursor, own_stack.as_ref(), state) {
      (stack.to_rect()?, stack.monitor())
    } else if drag.is_from_floating {
      return Ok(None);
    } else {
      let Some(drop) = window_drop(window, &cursor, state)? else {
        return Ok(None);
      };

      (
        drop.indicator_rect()?,
        drop.workspace().and_then(|workspace| workspace.monitor()),
      )
    };

  let viewport = monitor.context("No monitor.")?.to_rect()?;

  Ok(rect.intersection(&viewport).map(|rect| (rect, viewport)))
}
