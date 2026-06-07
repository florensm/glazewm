use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, SplitContainerDto, TilingDirection,
};
use wm_platform::Rect;

use crate::{
  impl_common_getters, impl_container_debug, impl_tiling_direction_getters,
  impl_tiling_size_getters,
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
  tiling_direction: TilingDirection,
  gaps_config: GapsConfig,
}

impl SplitContainer {
  pub fn new(
    tiling_direction: TilingDirection,
    gaps_config: GapsConfig,
  ) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      tiling_direction,
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
      tiling_direction: self.tiling_direction(),
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

    // Otherwise, use the normal tiling position calculation (same as
    // `impl_position_getters_as_resizable!` macro).
    let parent = parent
      .as_direction_container()
      .context("Parent does not have a tiling direction.")?;

    let parent_rect = parent.to_rect()?;

    let (horizontal_gap, vertical_gap) = self.inner_gaps()?;
    let inner_gap = match parent.tiling_direction() {
      TilingDirection::Vertical => vertical_gap,
      TilingDirection::Horizontal => horizontal_gap,
    };

    #[allow(
      clippy::cast_precision_loss,
      clippy::cast_possible_truncation,
      clippy::cast_possible_wrap
    )]
    let (width, height) = match parent.tiling_direction() {
      TilingDirection::Vertical => {
        let available_height = parent_rect.height()
          - inner_gap * self.tiling_siblings().count() as i32;

        let height =
          (self.tiling_size() * available_height as f32) as i32;

        (parent_rect.width(), height)
      }
      TilingDirection::Horizontal => {
        let available_width = parent_rect.width()
          - inner_gap * self.tiling_siblings().count() as i32;

        let width =
          (available_width as f32 * self.tiling_size()).round() as i32;

        (width, parent_rect.height())
      }
    };

    let (x, y) = {
      let mut prev_siblings = self
        .prev_siblings()
        .filter_map(|sibling| sibling.as_tiling_container().ok());

      match prev_siblings.next() {
        None => (parent_rect.x(), parent_rect.y()),
        Some(sibling) => {
          let sibling_rect = sibling.to_rect()?;

          match parent.tiling_direction() {
            TilingDirection::Vertical => (
              parent_rect.x(),
              sibling_rect.y() + sibling_rect.height() + inner_gap,
            ),
            TilingDirection::Horizontal => (
              sibling_rect.x() + sibling_rect.width() + inner_gap,
              parent_rect.y(),
            ),
          }
        }
      }
    };

    Ok(Rect::from_xy(x, y, width, height))
  }
}
