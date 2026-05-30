use ambassador::delegatable_trait;
use wm_platform::Rect;

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
}

/// Implements the `PositionGetters` trait for tiling containers that can
/// be resized (i.e. `StackContainer`).
///
/// Delegates to the parent direction container's layout to compute the
/// container's rect from the shared `tiling_size` proportions.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_getters_as_resizable {
  ($struct_name:ident) => {
    impl PositionGetters for $struct_name {
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a layout.")?;

        let parent_rect = parent.to_rect()?;
        let (h_gap, v_gap) = self.inner_gaps()?;

        let tiling_children: Vec<TilingContainer> =
          parent.tiling_children().collect();
        let my_index = tiling_children
          .iter()
          .position(|c| c.id() == self.id())
          .context(
            "Container not found among parent's tiling children.",
          )?;

        let sizes: Vec<f32> = tiling_children
          .iter()
          .map(TilingSizeGetters::tiling_size)
          .collect();

        parent
          .layout()
          .compute_rects(parent_rect, &sizes, h_gap, v_gap)
          .into_iter()
          .nth(my_index)
          .context("Rect index out of range for container.")
      }
    }
  };
}
