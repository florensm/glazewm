use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, StackContainerDto, TabBarPosition,
  TilingDirection,
};
use wm_platform::{LengthValue, Rect};

use crate::{
  impl_common_getters, impl_container_debug,
  impl_position_getters_as_resizable, impl_tiling_size_getters,
  models::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters,
  },
};

#[derive(Clone)]
pub struct StackContainer(Rc<RefCell<StackContainerInner>>);

struct StackContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  gaps_config: GapsConfig,
  tab_bar_height: LengthValue,
  tab_bar_position: TabBarPosition,
  /// Optional user-assigned name for targeting via `move-to-stack
  /// --name`.
  name: Option<String>,
}

impl StackContainer {
  /// Creates a new `StackContainer` with default sizing and no children.
  pub fn new(
    gaps_config: GapsConfig,
    tab_bar_height: LengthValue,
    tab_bar_position: TabBarPosition,
  ) -> Self {
    let stack = StackContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      gaps_config,
      tab_bar_height,
      tab_bar_position,
      name: None,
    };

    Self(Rc::new(RefCell::new(stack)))
  }

  /// Returns the tab bar position (top or bottom).
  pub fn tab_bar_position(&self) -> TabBarPosition {
    self.0.borrow().tab_bar_position.clone()
  }

  /// Returns the user-assigned name of this stack, if any.
  pub fn name(&self) -> Option<String> {
    self.0.borrow().name.clone()
  }

  /// Sets the user-assigned name of this stack.
  pub fn set_name(&self, name: String) {
    self.0.borrow_mut().name = Some(name);
  }

  /// Returns the tab bar height in pixels, scaled for the monitor's DPI.
  ///
  /// Returns `0` when the tab bar is disabled or the monitor cannot be
  /// determined.
  pub fn tab_bar_height_px(&self) -> i32 {
    let inner = self.0.borrow();
    let scale_with_dpi = inner.gaps_config.scale_with_dpi;
    let height_lv = inner.tab_bar_height.clone();
    drop(inner);

    let scale_factor = if scale_with_dpi {
      self
        .monitor()
        .map_or(1.0, |m| m.native_properties().scale_factor)
    } else {
      1.0
    };

    height_lv.to_px(0, Some(scale_factor))
  }

  /// Whether the stack should be flattened once it has `child_count`
  /// children.
  ///
  /// Named stacks are targets for `move-to-stack`, so they are kept with a
  /// single window and only removed once empty.
  pub fn is_redundant_with(&self, child_count: usize) -> bool {
    match self.name() {
      Some(_) => child_count == 0,
      None => child_count <= 1,
    }
  }

  /// Returns the active tab, i.e. the most recently focused child.
  pub fn active_child(&self) -> Option<Container> {
    self.child_focus_order().next()
  }

  /// Converts this `StackContainer` to a `ContainerDto` for IPC and debug
  /// logging.
  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;
    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    Ok(ContainerDto::Stack(StackContainerDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      has_focus: self.has_focus(None),
      tiling_size: self.tiling_size(),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
    }))
  }
}

/// Whether `container` is a stack child other than its stack's active tab.
///
/// Inactive tabs are kept hidden; only the active tab is displayed.
pub fn is_inactive_stack_child(container: &impl CommonGetters) -> bool {
  container
    .parent()
    .and_then(|parent| parent.as_stack().cloned())
    .and_then(|stack| stack.active_child())
    .is_some_and(|active| active.id() != container.id())
}

impl_container_debug!(StackContainer);
impl_common_getters!(StackContainer);
impl_tiling_size_getters!(StackContainer);
impl_position_getters_as_resizable!(StackContainer);

#[cfg(test)]
mod tests {
  use crate::{
    commands::container::{
      detach_container, set_focused_descendant, wrap_in_stack_container,
    },
    models::{
      is_inactive_stack_child, StackContainer, TilingWindow, Workspace,
    },
    traits::{CommonGetters, TilingSizeGetters},
  };

  fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
  }

  #[test]
  fn wrap_takes_the_window_slot_and_size() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();

    let stack = StackContainer::mock().call();
    wrap_in_stack_container(
      &stack,
      &workspace.clone().into(),
      &[second.clone().into()],
    )
    .unwrap();

    assert_eq!(stack.index(), 1);
    assert!(approx_eq(stack.tiling_size(), 0.5));
    assert!(approx_eq(second.tiling_size(), 1.0));
    assert_eq!(second.parent().unwrap().id(), stack.id());
    assert!(approx_eq(first.tiling_size(), 0.5));
  }

  #[test]
  fn only_the_active_tab_is_shown() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let _workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();

    set_focused_descendant(&second.clone().into(), None);

    assert_eq!(stack.active_child().unwrap().id(), second.id());
    assert!(is_inactive_stack_child(&first));
    assert!(!is_inactive_stack_child(&second));
    assert!(!is_inactive_stack_child(&stack));
  }

  #[test]
  fn unnamed_stack_is_flattened_at_one_window() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();

    detach_container(second.into()).unwrap();

    assert!(stack.is_detached());
    assert_eq!(first.parent().unwrap().id(), workspace.id());
    assert!(approx_eq(first.tiling_size(), 1.0));
  }

  #[test]
  fn named_stack_survives_until_empty() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .name("tickets".to_string())
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![stack.clone().into()])
      .call();

    detach_container(second.into()).unwrap();

    assert!(!stack.is_detached());
    assert_eq!(first.parent().unwrap().id(), stack.id());

    detach_container(first.into()).unwrap();

    assert!(stack.is_detached());
    assert_eq!(workspace.child_count(), 0);
  }
}
