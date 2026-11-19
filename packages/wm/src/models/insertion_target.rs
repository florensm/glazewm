use crate::{
  layout::scrolling::column_workspace,
  models::{Container, TilingContainer},
  traits::CommonGetters,
};

#[derive(Debug, Clone)]
pub struct InsertionTarget {
  pub target_parent: Container,
  pub target_index: usize,
  pub prev_tiling_size: f32,
  pub prev_sibling_count: usize,
}

impl InsertionTarget {
  /// Tiling size for `container`, put back into the tree, to take up the
  /// same share of its parent as before.
  ///
  /// The previous size is scaled by the change in sibling count (e.g. 0.5
  /// with 1 sibling becomes 0.5 * 2/3 with 2 siblings). A column keeps its
  /// width, which isn't shared with the other columns.
  #[allow(clippy::cast_precision_loss)]
  pub fn restored_size(&self, container: &TilingContainer) -> f32 {
    if column_workspace(container).is_some() {
      return self.prev_tiling_size;
    }

    let size_scale = (self.prev_sibling_count + 1) as f32
      / (container.tiling_siblings().count() + 1) as f32;

    self.prev_tiling_size * size_scale
  }
}
