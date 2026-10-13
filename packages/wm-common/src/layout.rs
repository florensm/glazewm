use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

/// How a workspace arranges its tiling windows.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
  /// Manual i3-style splits: new windows tile next to the focused one.
  #[default]
  Split,
  /// Each new window splits the focused window along its longer side.
  Bsp,
  /// Each new window splits the most recent one, spiralling inwards.
  Dwindle,
  /// Columns on a strip that scrolls horizontally past the monitor.
  Scrolling,
}

impl FromStr for Layout {
  type Err = anyhow::Error;

  /// Parses a layout name.
  ///
  /// Example:
  /// ```
  /// # use wm_common::Layout;
  /// # use std::str::FromStr;
  /// assert_eq!(Layout::from_str("bsp").unwrap(), Layout::Bsp);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "split" => Ok(Self::Split),
      "bsp" => Ok(Self::Bsp),
      "dwindle" => Ok(Self::Dwindle),
      "scrolling" => Ok(Self::Scrolling),
      _ => bail!("Not a valid layout: {}", unparsed),
    }
  }
}
