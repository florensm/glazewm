use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, SplitContainerDto, WorkspaceLayout,
};
use wm_platform::Rect;

use crate::{
  impl_common_getters, impl_container_debug,
  impl_tiling_direction_getters, impl_tiling_size_getters,
  models::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters,
  },
};

#[derive(Clone)]
pub struct SplitContainer(Rc<RefCell<SplitContainerInner>>);

struct SplitContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  layout: WorkspaceLayout,
  gaps_config: GapsConfig,
}

impl SplitContainer {
  pub fn new(
    layout: WorkspaceLayout,
    gaps_config: GapsConfig,
  ) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      layout,
      gaps_config,
    };

    Self(Rc::new(RefCell::new(split)))
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;
    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    Ok(ContainerDto::Split(SplitContainerDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      has_focus: self.has_focus(None),
      tiling_size: self.tiling_size(),
      layout: self.layout(),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
    }))
  }
}

impl_container_debug!(SplitContainer);
impl_common_getters!(SplitContainer);
impl_tiling_size_getters!(SplitContainer);
impl_tiling_direction_getters!(SplitContainer);

impl PositionGetters for SplitContainer {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    let parent = self.parent().context("No parent container.")?;

    // All children of a stack share the full stack rect.
    if let Some(stack) = parent.as_stack() {
      return stack.to_rect();
    }

    let parent = parent
      .as_direction_container()
      .context("Parent does not have a layout.")?;

    let parent_rect = parent.to_rect()?;
    let (h_gap, v_gap) = self.inner_gaps()?;

    let tiling_children: Vec<TilingContainer> =
      parent.tiling_children().collect();
    let my_index = tiling_children
      .iter()
      .position(|c| c.id() == self.id())
      .context("Split container not found among parent's tiling children.")?;

    let sizes: Vec<f32> =
      tiling_children.iter().map(TilingSizeGetters::tiling_size).collect();

    parent
      .layout()
      .compute_rects(parent_rect, &sizes, h_gap, v_gap)
      .into_iter()
      .nth(my_index)
      .context("Rect index out of range for split container.")
  }
}
