use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// How a workspace arranges its tiling windows.
#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  Deserialize,
  PartialEq,
  Eq,
  Serialize,
  ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
  /// Manual i3-style splits: new windows tile next to the focused one.
  #[default]
  Split,
  /// Each new window halves the focused window along its longer side, as
  /// in Hyprland.
  Dwindle,
  /// Columns on a strip that scrolls horizontally past the monitor.
  Scrolling,
}
