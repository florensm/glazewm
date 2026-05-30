use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use wm_platform::Rect;

use crate::TilingDirection;

/// Algorithm used to position the direct tiling children of a container.
///
/// Applied by both `Workspace` and `SplitContainer` via `LayoutGetters`.
/// Adding a new layout only requires adding a variant here and implementing
/// `compute_rects`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ValueEnum)]
#[clap(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceLayout {
  /// Proportional splits left-to-right. Children are sized according to
  /// their `tiling_size` fraction.
  SplitHorizontal,
  /// Proportional splits top-to-bottom. Children are sized according to
  /// their `tiling_size` fraction.
  SplitVertical,
  /// Equal-width columns laid out left-to-right.
  Columns,
  /// Equal-height rows laid out top-to-bottom.
  Rows,
  /// Rectangular grid with as-square-as-possible cells.
  Grid,
  /// First child occupies the left portion (ratio controlled by its
  /// `tiling_size`); remaining children are stacked vertically on the right.
  VerticalStack,
  /// First child occupies the top portion (ratio controlled by its
  /// `tiling_size`); remaining children are stacked horizontally at the
  /// bottom.
  HorizontalStack,
}

impl WorkspaceLayout {
  /// Returns the primary tiling axis used for directional focus/move
  /// navigation.
  #[must_use]
  pub fn primary_direction(&self) -> TilingDirection {
    match self {
      Self::SplitHorizontal
      | Self::Columns
      | Self::Grid
      | Self::VerticalStack => TilingDirection::Horizontal,
      Self::SplitVertical
      | Self::Rows
      | Self::HorizontalStack => TilingDirection::Vertical,
    }
  }

  /// Returns whether the layout is a `Split` variant.
  ///
  /// Used by flatten logic to avoid collapsing containers in non-split
  /// layouts.
  #[must_use]
  pub fn is_split(&self) -> bool {
    matches!(self, Self::SplitHorizontal | Self::SplitVertical)
  }

  /// Whether this layout respects `tiling_size` for resize operations.
  ///
  /// Returns `false` for layouts that distribute space equally, making
  /// manual resize a no-op.
  #[must_use]
  pub fn supports_resize(&self) -> bool {
    matches!(
      self,
      Self::SplitHorizontal
        | Self::SplitVertical
        | Self::VerticalStack
        | Self::HorizontalStack
    )
  }

  /// Returns the inverse of a split layout, or `self` for non-split layouts.
  #[must_use]
  pub fn inverse(&self) -> Self {
    match self {
      Self::SplitHorizontal => Self::SplitVertical,
      Self::SplitVertical => Self::SplitHorizontal,
      other => other.clone(),
    }
  }

  /// Computes the bounding rect for each direct tiling child.
  ///
  /// `tiling_sizes` must contain one entry per tiling child, in tree order.
  /// Returns one `Rect` per child in the same order.
  ///
  /// `h_gap` is the gap applied between horizontally adjacent children;
  /// `v_gap` is the gap applied between vertically adjacent children.
  #[must_use]
  #[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
  )]
  pub fn compute_rects(
    &self,
    parent_rect: Rect,
    tiling_sizes: &[f32],
    h_gap: i32,
    v_gap: i32,
  ) -> Vec<Rect> {
    let n = tiling_sizes.len();
    if n == 0 {
      return vec![];
    }

    match self {
      Self::SplitHorizontal => {
        // Each child gets a proportional share of the width.
        let available_w =
          parent_rect.width() - h_gap * (n as i32 - 1);
        let mut x = parent_rect.x();
        tiling_sizes
          .iter()
          .enumerate()
          .map(|(i, &size)| {
            let w = if i == n - 1 {
              // Last child takes remaining space to absorb rounding drift.
              parent_rect.x() + parent_rect.width() - x
            } else {
              (available_w as f32 * size).round() as i32
            };
            let rect = Rect::from_xy(
              x,
              parent_rect.y(),
              w,
              parent_rect.height(),
            );
            x += w + h_gap;
            rect
          })
          .collect()
      }
      Self::SplitVertical => {
        // Each child gets a proportional share of the height.
        let available_h =
          parent_rect.height() - v_gap * (n as i32 - 1);
        let mut y = parent_rect.y();
        tiling_sizes
          .iter()
          .enumerate()
          .map(|(i, &size)| {
            let h = if i == n - 1 {
              parent_rect.y() + parent_rect.height() - y
            } else {
              // Match original floor truncation for vertical splits.
              (size * available_h as f32) as i32
            };
            let rect = Rect::from_xy(
              parent_rect.x(),
              y,
              parent_rect.width(),
              h,
            );
            y += h + v_gap;
            rect
          })
          .collect()
      }
      Self::Columns => {
        // Equal-width columns.
        let available_w =
          parent_rect.width() - h_gap * (n as i32 - 1);
        let col_w = available_w / n as i32;
        let mut x = parent_rect.x();
        (0..n)
          .map(|i| {
            let w = if i == n - 1 {
              parent_rect.x() + parent_rect.width() - x
            } else {
              col_w
            };
            let rect = Rect::from_xy(
              x,
              parent_rect.y(),
              w,
              parent_rect.height(),
            );
            x += w + h_gap;
            rect
          })
          .collect()
      }
      Self::Rows => {
        // Equal-height rows.
        let available_h =
          parent_rect.height() - v_gap * (n as i32 - 1);
        let row_h = available_h / n as i32;
        let mut y = parent_rect.y();
        (0..n)
          .map(|i| {
            let h = if i == n - 1 {
              parent_rect.y() + parent_rect.height() - y
            } else {
              row_h
            };
            let rect = Rect::from_xy(
              parent_rect.x(),
              y,
              parent_rect.width(),
              h,
            );
            y += h + v_gap;
            rect
          })
          .collect()
      }
      Self::Grid => {
        // Arrange children in a roughly-square grid.
        let cols = (n as f64).sqrt().ceil() as usize;
        let rows = n.div_ceil(cols);
        let available_w =
          parent_rect.width() - h_gap * (cols as i32 - 1);
        let available_h =
          parent_rect.height() - v_gap * (rows as i32 - 1);
        let cell_w = available_w / cols as i32;
        let cell_h = available_h / rows as i32;

        (0..n)
          .map(|i| {
            let col = (i % cols) as i32;
            let row = (i / cols) as i32;
            let x = parent_rect.x() + col * (cell_w + h_gap);
            let y = parent_rect.y() + row * (cell_h + v_gap);
            let w = if col == cols as i32 - 1 {
              parent_rect.x() + parent_rect.width() - x
            } else {
              cell_w
            };
            let h = if row == rows as i32 - 1 {
              parent_rect.y() + parent_rect.height() - y
            } else {
              cell_h
            };
            Rect::from_xy(x, y, w, h)
          })
          .collect()
      }
      Self::VerticalStack => {
        // First child is the main area on the left; the rest stack
        // vertically on the right.
        if n == 1 {
          return vec![parent_rect];
        }

        let main_ratio = tiling_sizes[0];
        // One h_gap between the main area and the stack column.
        let available_w = parent_rect.width() - h_gap;
        let main_w =
          (available_w as f32 * main_ratio).round() as i32;
        let stack_w = available_w - main_w;
        let n_stack = n - 1;
        let stack_available_h =
          parent_rect.height() - v_gap * (n_stack as i32 - 1);
        let stack_child_h = stack_available_h / n_stack as i32;

        let mut rects = Vec::with_capacity(n);
        rects.push(Rect::from_xy(
          parent_rect.x(),
          parent_rect.y(),
          main_w,
          parent_rect.height(),
        ));

        let stack_x = parent_rect.x() + main_w + h_gap;
        let mut y = parent_rect.y();
        for i in 0..n_stack {
          let h = if i == n_stack - 1 {
            parent_rect.y() + parent_rect.height() - y
          } else {
            stack_child_h
          };
          rects.push(Rect::from_xy(stack_x, y, stack_w, h));
          y += h + v_gap;
        }

        rects
      }
      Self::HorizontalStack => {
        // First child is the main area on top; the rest stack
        // horizontally at the bottom.
        if n == 1 {
          return vec![parent_rect];
        }

        let main_ratio = tiling_sizes[0];
        // One v_gap between the main area and the stack row.
        let available_h = parent_rect.height() - v_gap;
        let main_h =
          (available_h as f32 * main_ratio).round() as i32;
        let stack_h = available_h - main_h;
        let n_stack = n - 1;
        let stack_available_w =
          parent_rect.width() - h_gap * (n_stack as i32 - 1);
        let stack_child_w = stack_available_w / n_stack as i32;

        let mut rects = Vec::with_capacity(n);
        rects.push(Rect::from_xy(
          parent_rect.x(),
          parent_rect.y(),
          parent_rect.width(),
          main_h,
        ));

        let stack_y = parent_rect.y() + main_h + v_gap;
        let mut x = parent_rect.x();
        for i in 0..n_stack {
          let w = if i == n_stack - 1 {
            parent_rect.x() + parent_rect.width() - x
          } else {
            stack_child_w
          };
          rects.push(Rect::from_xy(x, stack_y, w, stack_h));
          x += w + h_gap;
        }

        rects
      }
    }
  }
}

impl Default for WorkspaceLayout {
  fn default() -> Self {
    Self::SplitHorizontal
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect::from_xy(x, y, w, h)
  }

  #[test]
  fn split_h_two_equal_children() {
    let rects = WorkspaceLayout::SplitHorizontal.compute_rects(
      rect(0, 0, 1000, 600),
      &[0.5, 0.5],
      0,
      0,
    );
    assert_eq!(rects[0], rect(0, 0, 500, 600));
    assert_eq!(rects[1], rect(500, 0, 500, 600));
  }

  #[test]
  fn split_h_with_gap() {
    let rects = WorkspaceLayout::SplitHorizontal.compute_rects(
      rect(0, 0, 1010, 600),
      &[0.5, 0.5],
      10,
      0,
    );
    // available = 1010 - 10 = 1000; each gets 500
    assert_eq!(rects[0], rect(0, 0, 500, 600));
    assert_eq!(rects[1], rect(510, 0, 500, 600));
  }

  #[test]
  fn columns_equal_distribution() {
    let rects = WorkspaceLayout::Columns.compute_rects(
      rect(0, 0, 900, 600),
      &[1.0, 1.0, 1.0],
      0,
      0,
    );
    assert_eq!(rects[0], rect(0, 0, 300, 600));
    assert_eq!(rects[1], rect(300, 0, 300, 600));
    assert_eq!(rects[2], rect(600, 0, 300, 600));
  }

  #[test]
  fn grid_four_children() {
    let rects = WorkspaceLayout::Grid.compute_rects(
      rect(0, 0, 1000, 600),
      &[1.0, 1.0, 1.0, 1.0],
      0,
      0,
    );
    // 4 children → 2×2 grid
    assert_eq!(rects[0], rect(0, 0, 500, 300));
    assert_eq!(rects[1], rect(500, 0, 500, 300));
    assert_eq!(rects[2], rect(0, 300, 500, 300));
    assert_eq!(rects[3], rect(500, 300, 500, 300));
  }

  #[test]
  fn vertical_stack_main_and_two_stacked() {
    // sizes[0]=0.5 → main gets ~50% width; 2 stacked children share right
    let rects = WorkspaceLayout::VerticalStack.compute_rects(
      rect(0, 0, 1000, 600),
      &[0.5, 0.5, 0.5],
      0,
      0,
    );
    assert_eq!(rects[0], rect(0, 0, 500, 600)); // main
    assert_eq!(rects[1], rect(500, 0, 500, 300)); // stack[0]
    assert_eq!(rects[2], rect(500, 300, 500, 300)); // stack[1]
  }
}
