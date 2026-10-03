use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, NewTabPosition, StackContainerDto,
  TabBarPosition, WindowState,
};
use wm_platform::{LengthValue, Rect};

use crate::{
  impl_common_getters, impl_container_debug, impl_tiling_size_getters,
  models::{
    Container, DirectionContainer, InsertionTarget, TilingContainer,
    WindowContainer,
  },
  traits::{
    tiling_rect, CommonGetters, PositionGetters, TilingSizeGetters,
    WindowGetters,
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
  /// Where the stack goes back to in the tiling layout once its windows
  /// tile again.
  insertion_target: Option<InsertionTarget>,
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
      insertion_target: None,
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

  /// Index among the tabs for a window added to the stack.
  pub fn new_tab_index(&self, position: NewTabPosition) -> usize {
    match position {
      NewTabPosition::End => self.child_count(),
      NewTabPosition::AfterActive => self
        .active_child()
        .map_or(self.child_count(), |active| active.index() + 1),
    }
  }

  /// The stack's windows, in tab order.
  pub fn windows(&self) -> Vec<WindowContainer> {
    self
      .children()
      .into_iter()
      .filter_map(|child| child.as_window_container().ok())
      .collect()
  }

  /// Whether the stack is part of the tiling layout.
  ///
  /// All windows of a stack share one state, so a stack of floating,
  /// fullscreen or minimized windows floats, fullscreens or minimizes as
  /// a whole.
  pub fn is_tiling(&self) -> bool {
    !self
      .0
      .borrow()
      .children
      .iter()
      .any(|child| matches!(child, Container::NonTilingWindow(_)))
  }

  /// The state shared by the stack's windows.
  pub fn state(&self) -> WindowState {
    self
      .windows()
      .first()
      .map_or(WindowState::Tiling, WindowGetters::state)
  }

  /// Whether the tab bar is drawn, which it isn't for a fullscreen or
  /// minimized stack.
  pub fn shows_tab_bar(&self) -> bool {
    matches!(self.state(), WindowState::Tiling | WindowState::Floating(_))
  }

  pub fn insertion_target(&self) -> Option<InsertionTarget> {
    self.0.borrow().insertion_target.clone()
  }

  pub fn set_insertion_target(
    &self,
    insertion_target: Option<InsertionTarget>,
  ) {
    self.0.borrow_mut().insertion_target = insertion_target;
  }

  /// The part of `stack_rect` left to the windows next to the tab bar.
  pub fn content_rect(&self, stack_rect: &Rect) -> Rect {
    let height = self.tab_bar_height_px();
    if height <= 0 || !self.shows_tab_bar() {
      return stack_rect.clone();
    }

    match self.tab_bar_position() {
      TabBarPosition::Top => Rect::from_ltrb(
        stack_rect.left,
        stack_rect.top + height,
        stack_rect.right,
        stack_rect.bottom,
      ),
      TabBarPosition::Bottom => Rect::from_ltrb(
        stack_rect.left,
        stack_rect.top,
        stack_rect.right,
        stack_rect.bottom - height,
      ),
    }
  }

  /// Inverse of `content_rect`: the stack rect around windows at
  /// `content_rect`.
  pub fn outer_rect(&self, content_rect: &Rect) -> Rect {
    let height = self.tab_bar_height_px();
    if height <= 0 || !self.shows_tab_bar() {
      return content_rect.clone();
    }

    match self.tab_bar_position() {
      TabBarPosition::Top => Rect::from_ltrb(
        content_rect.left,
        content_rect.top - height,
        content_rect.right,
        content_rect.bottom,
      ),
      TabBarPosition::Bottom => Rect::from_ltrb(
        content_rect.left,
        content_rect.top,
        content_rect.right,
        content_rect.bottom + height,
      ),
    }
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

/// The other windows of the stack `container` is in, one of which is
/// shown once `container` leaves.
///
/// Taken before `container` leaves, since that can flatten the stack and
/// empty it.
pub fn other_stack_tabs(
  container: &impl CommonGetters,
) -> Vec<WindowContainer> {
  container
    .parent()
    .and_then(|parent| parent.as_stack().map(StackContainer::windows))
    .unwrap_or_default()
    .into_iter()
    .filter(|tab| tab.id() != container.id())
    .collect()
}

impl_container_debug!(StackContainer);
impl_common_getters!(StackContainer);
impl_tiling_size_getters!(StackContainer);

impl PositionGetters for StackContainer {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    if self.is_tiling() {
      return tiling_rect(self);
    }

    // A non-tiling stack is wherever its windows are.
    let active = self
      .active_child()
      .and_then(|child| child.as_window_container().ok())
      .context("Stack has no active window.")?;

    // Follow the window live while it's dragged, rather than its
    // placement from before the drag.
    let content_rect = if active.active_drag().is_some() {
      active.native_properties().frame
    } else {
      active.to_rect()?
    };

    Ok(self.outer_rect(&content_rect))
  }
}

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

  #[test]
  fn closing_a_tab_keeps_the_layout() {
    let left = TilingWindow::mock().tiling_size(0.3).call();
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    stack.set_tiling_size(0.7);
    let _workspace = Workspace::mock()
      .tiling_containers(vec![left.clone().into(), stack.clone().into()])
      .call();
    left.set_tiling_size(0.3);
    stack.set_tiling_size(0.7);

    detach_container(second.into()).unwrap();

    assert!(stack.is_detached());
    assert!(approx_eq(first.tiling_size(), 0.7));
    assert!(approx_eq(left.tiling_size(), 0.3));
  }
}
