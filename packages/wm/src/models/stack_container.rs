use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{ContainerDto, GapsConfig, StackContainerDto, TilingDirection};
use wm_platform::{LengthValue, Rect};

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
  tab_bar_height: LengthValue,
  /// Optional user-assigned name for targeting via `move-to-stack --name`.
  name: Option<String>,
}

impl StackContainer {
  /// Creates a new `StackContainer` with default sizing and no children.
  pub fn new(gaps_config: GapsConfig, tab_bar_height: LengthValue) -> Self {
    let stack = StackContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      gaps_config,
      tab_bar_height,
      name: None,
    };

    Self(Rc::new(RefCell::new(stack)))
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
        .map(|m| m.native_properties().scale_factor)
        .unwrap_or(1.0)
    } else {
      1.0
    };

    height_lv.to_px(0, Some(scale_factor))
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
