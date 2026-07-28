use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

/// What a `toggle_pip` command applies to.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipTarget {
  /// Only the subject window is minimized into a PIP tile.
  Window,
  /// Every window in the subject's workspace is minimized, each getting
  /// its own PIP tile in the dock rect.
  Workspace,
}

impl FromStr for PipTarget {
  type Err = anyhow::Error;

  /// Parses a string into a PIP target.
  ///
  /// Example:
  /// ```
  /// # use wm_common::PipTarget;
  /// # use std::str::FromStr;
  /// let target = PipTarget::from_str("window");
  /// assert_eq!(target.unwrap(), PipTarget::Window);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "window" => Ok(Self::Window),
      "workspace" => Ok(Self::Workspace),
      _ => bail!("Not a valid PIP target: {}", unparsed),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_valid_targets() {
    assert_eq!(PipTarget::from_str("window").unwrap(), PipTarget::Window);
    assert_eq!(
      PipTarget::from_str("workspace").unwrap(),
      PipTarget::Workspace
    );
  }

  #[test]
  fn rejects_invalid_target() {
    assert!(PipTarget::from_str("monitor").is_err());
  }
}
