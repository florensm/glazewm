//! Workspace layouts.
//!
//! Every layout works on the same container tree; the workspace's
//! `Layout` decides where new windows join it and how rects are computed
//! from it.

mod geometry;
mod split;

use anyhow::Context;
pub use geometry::tiling_rect;
use wm_common::{Layout, WindowState};

use crate::{
  models::Container, traits::CommonGetters, user_config::UserConfig,
  wm_state::WmState,
};

/// Gets where to attach a new window, as (parent, index), following the
/// focused workspace's layout.
pub fn insertion_target(
  window_state: &WindowState,
  joins_focused_stack: bool,
  state: &WmState,
  _config: &UserConfig,
) -> anyhow::Result<(Container, usize)> {
  let workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No focused workspace.")?;

  match workspace.layout() {
    Layout::Split | Layout::Bsp | Layout::Dwindle | Layout::Scrolling => {
      split::insertion_target(window_state, joins_focused_stack, state)
    }
  }
}
