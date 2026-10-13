//! BSP and dwindle: every new window halves an existing window's slot,
//! along that slot's longer side, building a binary tree.
//!
//! They differ only in which window gets split: BSP splits the focused
//! one, dwindle the last one in the tree, which spirals new windows
//! towards the bottom right.

use std::collections::HashMap;

use anyhow::Context;
use wm_common::TilingDirection;

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

  unit_of(&window)
}

/// The last tiling window in tree order (as its stack, if stacked).
pub(super) fn last_target(
  workspace: &Workspace,
) -> Option<TilingContainer> {
  let mut container: Container = workspace.clone().into();

  while let Some(child) = container.tiling_children().last() {
    let is_leaf = !matches!(child, TilingContainer::Split(_));
    container = child.into();

    if is_leaf {
      return unit_of(&container);
    }
  }

  None
}

/// A stacked window is split as its whole stack.
fn unit_of(window: &Container) -> Option<TilingContainer> {
  match window
    .parent()
    .and_then(|parent| parent.as_stack().cloned())
  {
    Some(stack) => Some(stack.into()),
    None => window.as_tiling_container().ok(),
  }
}

/// Direction that halves `target` along its longer side.
pub(super) fn split_direction(
  target: &TilingContainer,
) -> anyhow::Result<TilingDirection> {
  let rect = target.to_rect()?;

  Ok(if rect.width() >= rect.height() {
    TilingDirection::Horizontal
  } else {
    TilingDirection::Vertical
  })
}

/// Attaches `child` so that it and `target` share `target`'s old slot.
///
/// Splits nested in the same direction as their parent get flattened
/// elsewhere (`flatten_child_split_containers`), so a split along the
/// parent's own direction is a sibling taking half of `target`'s size
/// rather than a new split container.
pub(super) fn attach_split(
  child: &Container,
  target: &TilingContainer,
  direction: &TilingDirection,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = target
    .parent()
    .and_then(|parent| parent.as_direction_container().ok())
    .context("Split target has no direction container.")?;

  let index = target.index() + 1;

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

  attach_container(child, &split.into(), Some(1))
}

impl Insertion {
  /// Insertion that splits `target`, or appends to `workspace` when it
  /// has no tiling windows yet.
  pub(super) fn split(
    target: Option<TilingContainer>,
    workspace: &Workspace,
  ) -> anyhow::Result<Self> {
    match target {
      Some(target) => Ok(Self::Split {
        direction: split_direction(&target)?,
        target,
      }),
      None => Ok(Self::At {
        parent: workspace.clone().into(),
        index: workspace.child_count(),
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, TilingDirection, WindowState};

  use crate::{
    layout::{attach, insertion_target},
    models::{
      Container, Monitor, SplitContainer, TilingWindow, Workspace,
    },
    traits::{
      CommonGetters, PositionGetters, TilingDirectionGetters,
      TilingSizeGetters,
    },
    user_config::UserConfig,
    wm_state::WmState,
  };

  fn config() -> UserConfig {
    UserConfig::from_parsed(wm_common::ParsedConfig::default())
  }

  /// A landscape workspace with `layout` and one window, focused.
  fn setup(layout: Layout) -> (WmState, Workspace, TilingWindow) {
    let first = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(layout)
      .tiling_containers(vec![first.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);
    crate::commands::container::set_focused_descendant(
      &first.clone().into(),
      None,
    );

    (state, workspace, first)
  }

  /// Opens a tiling window the way `manage_window` places one.
  fn open(state: &WmState) -> TilingWindow {
    let window = TilingWindow::mock().call();
    let insertion =
      insertion_target(&WindowState::Tiling, false, state).unwrap();
    attach(&window.clone().into(), &insertion, &config()).unwrap();
    window
  }

  fn focus(window: &TilingWindow) {
    crate::commands::container::set_focused_descendant(
      &window.clone().into(),
      None,
    );
  }

  fn sizes_sum_to_one(parent: &Container) -> bool {
    let sum = parent
      .tiling_children()
      .map(|child| child.tiling_size())
      .sum::<f32>();
    (sum - 1.).abs() < 1e-4
  }

  #[test]
  fn bsp_splits_the_focused_window_along_its_longer_side() {
    let (state, workspace, first) = setup(Layout::Bsp);

    // Landscape workspace: side by side.
    let second = open(&state);
    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(workspace.tiling_children().count(), 2);

    // Each half is portrait: the focused one is split top/bottom.
    focus(&first);
    let third = open(&state);
    let split = third.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(split.children()[0].id(), first.id());
    assert_eq!(second.parent().unwrap().id(), workspace.id());

    let (first_rect, third_rect) =
      (first.to_rect().unwrap(), third.to_rect().unwrap());
    assert_eq!(first_rect.x(), third_rect.x());
    assert!(third_rect.y() > first_rect.y());
  }

  #[test]
  fn bsp_halves_a_slot_without_nesting_a_same_direction_split() {
    let a = TilingWindow::mock().call();
    let b = TilingWindow::mock().call();
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![a.clone().into(), b.clone().into()])
      .call();
    let c = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .layout(Layout::Bsp)
      .tiling_containers(vec![column.clone().into(), c.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);

    // `c` is the right half (portrait), splitting it top/bottom nests a
    // vertical split; `a` is a landscape quarter, split side by side.
    focus(&a);
    let d = open(&state);
    let inner = d.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(inner.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(inner.parent().unwrap().id(), column.id());
    assert!(sizes_sum_to_one(&column.clone().into()));
    assert!((b.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn dwindle_spirals_from_the_last_window_whatever_has_focus() {
    let (state, workspace, first) = setup(Layout::Dwindle);

    let second = open(&state);
    focus(&first);
    let third = open(&state);
    focus(&first);
    let fourth = open(&state);

    // H[first, V[second, H[third, fourth]]]
    assert_eq!(first.parent().unwrap().id(), workspace.id());
    let right = second.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(right.tiling_direction(), TilingDirection::Vertical);
    let corner = third.parent().unwrap().as_split().cloned().unwrap();
    assert_eq!(corner.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(corner.parent().unwrap().id(), right.id());
    assert_eq!(fourth.parent().unwrap().id(), corner.id());

    let (third_rect, fourth_rect) =
      (third.to_rect().unwrap(), fourth.to_rect().unwrap());
    assert!(fourth_rect.x() > third_rect.x());
    assert_eq!(fourth_rect.y(), third_rect.y());
  }

  #[test]
  fn same_direction_split_halves_the_target_and_keeps_the_rest() {
    let (state, workspace, first) = setup(Layout::Bsp);
    let second = open(&state);
    first.set_tiling_size(0.7);
    second.set_tiling_size(0.3);

    // `first` is wider than tall, so it splits side by side, which is
    // the workspace's own direction.
    focus(&first);
    let third = open(&state);

    assert_eq!(third.parent().unwrap().id(), workspace.id());
    assert_eq!(third.index(), first.index() + 1);
    assert!((first.tiling_size() - 0.35).abs() < 1e-4);
    assert!((third.tiling_size() - 0.35).abs() < 1e-4);
    assert!((second.tiling_size() - 0.3).abs() < 1e-4);
  }
}
