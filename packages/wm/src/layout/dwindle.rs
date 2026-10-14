//! Dwindle (as in Hyprland): every new window halves the focused window's
//! slot along its longer side, building a binary tree. The new window
//! takes the half the cursor is in, or the right/bottom one when the
//! cursor is elsewhere.

use std::collections::HashMap;

use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::{Point, Rect};

use super::Insertion;
use crate::{
  commands::container::{attach_container, wrap_in_split_container},
  models::{Container, SplitContainer, TilingContainer, Workspace},
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// The focused tiling window (as its stack, if stacked), or the most
/// recently focused one when a non-tiling window has focus.
pub(super) fn focused_target(
  workspace: &Workspace,
  state: &WmState,
) -> Option<TilingContainer> {
  let window = state
    .focused_container()
    .filter(Container::is_tiling_window)
    .filter(|focused| {
      focused
        .workspace()
        .is_some_and(|w| w.id() == workspace.id())
    })
    .or_else(|| {
      workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window)
    })?;

  // A stacked window is split as its whole stack.
  match window
    .parent()
    .and_then(|parent| parent.as_stack().cloned())
  {
    Some(stack) => Some(stack.into()),
    None => window.as_tiling_container().ok(),
  }
}

/// Direction that halves `rect` along its longer side.
fn split_direction(rect: &Rect) -> TilingDirection {
  if rect.width() >= rect.height() {
    TilingDirection::Horizontal
  } else {
    TilingDirection::Vertical
  }
}

impl Insertion {
  /// Insertion that splits `target`, or appends to `workspace` when it
  /// has no tiling windows yet. `cursor` picks the half the new window
  /// takes.
  pub(super) fn split(
    target: Option<TilingContainer>,
    workspace: &Workspace,
    cursor: Option<Point>,
  ) -> anyhow::Result<Self> {
    let Some(target) = target else {
      return Ok(Self::At {
        parent: workspace.clone().into(),
        index: workspace.child_count(),
      });
    };

    let rect = target.to_rect()?;
    let direction = split_direction(&rect);
    let center = rect.center_point();

    let is_before = cursor
      .filter(|cursor| rect.contains_point(cursor))
      .is_some_and(|cursor| match direction {
        TilingDirection::Horizontal => cursor.x < center.x,
        TilingDirection::Vertical => cursor.y < center.y,
      });

    Ok(Self::Split {
      target,
      direction,
      is_before,
    })
  }
}

/// Attaches `child` so that it and `target` share `target`'s old slot,
/// `child` first if `is_before`.
///
/// Splits nested in the same direction as their parent get flattened
/// elsewhere (`flatten_child_split_containers`), so a split along the
/// parent's own direction is a sibling taking half of `target`'s size
/// rather than a new split container.
pub(super) fn attach_split(
  child: &Container,
  target: &TilingContainer,
  direction: &TilingDirection,
  is_before: bool,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = target
    .parent()
    .and_then(|parent| parent.as_direction_container().ok())
    .context("Split target has no direction container.")?;

  let index = target.index() + usize::from(!is_before);

  if parent.tiling_direction() == *direction {
    let target_size = target.tiling_size();
    let sizes = parent
      .tiling_children()
      .map(|sibling| (sibling.id(), sibling.tiling_size()))
      .collect::<HashMap<_, _>>();

    attach_container(child, &parent.clone().into(), Some(index))?;

    for sibling in parent.tiling_children() {
      if let Some(size) = sizes.get(&sibling.id()) {
        sibling.set_tiling_size(*size);
      }
    }

    let child = child.as_tiling_container()?;
    child.set_tiling_size(target_size / 2.);
    target.set_tiling_size(target_size / 2.);

    return Ok(());
  }

  // An only child turns its parent instead of nesting a split in it.
  if target.tiling_siblings().count() == 0 {
    parent.set_tiling_direction(direction.clone());
    return attach_container(child, &parent.into(), Some(index));
  }

  let split =
    SplitContainer::new(direction.clone(), config.value.gaps.clone());
  wrap_in_split_container(
    &split,
    &parent.into(),
    std::slice::from_ref(target),
  )?;

  attach_container(child, &split.into(), Some(usize::from(!is_before)))
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection};
  use wm_platform::Point;

  use super::focused_target;
  use crate::{
    commands::container::set_focused_descendant,
    layout::{attach, Insertion},
    models::{
      Container, Monitor, SplitContainer, TilingWindow, Workspace,
    },
    test_utils::mock_config as config,
    traits::{
      CommonGetters, PositionGetters, TilingDirectionGetters,
      TilingSizeGetters,
    },
    wm_state::WmState,
  };

  /// A landscape dwindle workspace with one window, focused.
  fn setup() -> (WmState, Workspace, TilingWindow) {
    let first = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(Layout::Dwindle)
      .tiling_containers(vec![first.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);
    set_focused_descendant(&first.clone().into(), None);

    (state, workspace, first)
  }

  /// Opens a tiling window next to the focused one, with the cursor at
  /// `cursor`, and focuses it.
  fn open(
    state: &WmState,
    workspace: &Workspace,
    cursor: Option<Point>,
  ) -> TilingWindow {
    let window = TilingWindow::mock().call();
    let insertion = Insertion::split(
      focused_target(workspace, state),
      workspace,
      cursor,
    )
    .unwrap();
    attach(&window.clone().into(), &insertion, &config()).unwrap();
    set_focused_descendant(&window.clone().into(), None);
    window
  }

  fn sizes_sum_to_one(parent: &Container) -> bool {
    let sum = parent
      .tiling_children()
      .map(|child| child.tiling_size())
      .sum::<f32>();
    (sum - 1.).abs() < 1e-4
  }

  #[test]
  fn splits_the_focused_window_along_its_longer_side() {
    let (state, workspace, first) = setup();

    // Landscape workspace: side by side.
    let second = open(&state, &workspace, None);
    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(second.index(), first.index() + 1);

    // Each half is portrait: the focused one is split top/bottom.
    let third = open(&state, &workspace, None);
    let split = third.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(split.children()[0].id(), second.id());
    assert_eq!(first.parent().unwrap().id(), workspace.id());

    let (second_rect, third_rect) =
      (second.to_rect().unwrap(), third.to_rect().unwrap());
    assert_eq!(second_rect.x(), third_rect.x());
    assert!(third_rect.y() > second_rect.y());
  }

  #[test]
  fn new_window_takes_the_half_the_cursor_is_in() {
    let (state, workspace, first) = setup();
    let rect = first.to_rect().unwrap();

    let left_half = Point {
      x: rect.x() + 10,
      y: rect.y() + 10,
    };
    let second = open(&state, &workspace, Some(left_half));
    assert_eq!(second.index(), 0);
    assert_eq!(first.index(), 1);

    // With the cursor outside the focused window, it goes after it.
    let elsewhere = Point { x: -10_000, y: 0 };
    let third = open(&state, &workspace, Some(elsewhere));
    let split = third.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(split.children()[1].id(), third.id());
  }

  #[test]
  fn halves_a_slot_without_nesting_a_same_direction_split() {
    let a = TilingWindow::mock().call();
    let b = TilingWindow::mock().call();
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![a.clone().into(), b.clone().into()])
      .call();
    let c = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(Layout::Dwindle)
      .tiling_containers(vec![column.clone().into(), c.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);

    // `a` is a landscape quarter, so it splits side by side within its
    // vertical column.
    set_focused_descendant(&a.clone().into(), None);
    let d = open(&state, &workspace, None);
    let inner = d.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(inner.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(inner.parent().unwrap().id(), column.id());
    assert!(sizes_sum_to_one(&column.clone().into()));
    assert!((b.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn same_direction_split_halves_the_target_and_keeps_the_rest() {
    let (state, workspace, first) = setup();
    let second = open(&state, &workspace, None);
    first.set_tiling_size(0.7);
    second.set_tiling_size(0.3);

    // `first` is wider than tall, so it splits side by side, which is
    // the workspace's own direction.
    set_focused_descendant(&first.clone().into(), None);
    let third = open(&state, &workspace, None);

    assert_eq!(third.parent().unwrap().id(), workspace.id());
    assert_eq!(third.index(), first.index() + 1);
    assert!((first.tiling_size() - 0.35).abs() < 1e-4);
    assert!((third.tiling_size() - 0.35).abs() < 1e-4);
    assert!((second.tiling_size() - 0.3).abs() < 1e-4);
  }
}
