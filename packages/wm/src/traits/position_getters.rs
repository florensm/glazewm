use ambassador::delegatable_trait;
use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::Rect;

use crate::traits::{
  CommonGetters, TilingDirectionGetters, TilingSizeGetters,
};

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
}

/// Rect of a tiling container, from its share of its parent's rect.
pub fn tiling_rect<T>(container: &T) -> anyhow::Result<Rect>
where
  T: TilingSizeGetters,
{
  let parent = container
    .parent()
    .and_then(|parent| parent.as_direction_container().ok())
    .context("Parent does not have a tiling direction.")?;

  let parent_rect = parent.to_rect()?;

  let (horizontal_gap, vertical_gap) = container.inner_gaps()?;
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
        - inner_gap * container.tiling_siblings().count() as i32;

      let height =
        (container.tiling_size() * available_height as f32) as i32;

      (parent_rect.width(), height)
    }
    TilingDirection::Horizontal => {
      let available_width = parent_rect.width()
        - inner_gap * container.tiling_siblings().count() as i32;

      let width =
        (available_width as f32 * container.tiling_size()).round() as i32;

      (width, parent_rect.height())
    }
  };

  let (x, y) = {
    let mut prev_siblings = container
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
