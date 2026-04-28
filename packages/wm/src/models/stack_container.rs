use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{ContainerDto, GapsConfig, StackContainerDto, TilingDirection};
use wm_platform::Rect;

use crate::{
  impl_common_getters, impl_container_debug,
  impl_position_getters_as_resizable, impl_tiling_size_getters,
  models::{Container, DirectionContainer, TilingContainer, WindowContainer},
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, TilingSizeGetters,
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
}

impl StackContainer {
  /// Creates a new `StackContainer` with default sizing and no children.
  pub fn new(gaps_config: GapsConfig) -> Self {
    let stack = StackContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      gaps_config,
    };

    Self(Rc::new(RefCell::new(stack)))
  }

  /// Converts this `StackContainer` to a `ContainerDto` for IPC and debug logging.
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

impl_container_debug!(StackContainer);
impl_common_getters!(StackContainer);
impl_tiling_size_getters!(StackContainer);
impl_position_getters_as_resizable!(StackContainer);
