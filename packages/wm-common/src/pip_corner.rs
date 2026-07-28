use serde::{Deserialize, Serialize};

/// Monitor corner that a PIP tile is docked in.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipCorner {
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
}

impl Default for PipCorner {
  fn default() -> Self {
    Self::BottomRight
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_to_bottom_right() {
    assert_eq!(PipCorner::default(), PipCorner::BottomRight);
  }
}
