//! Workspace layouts.
//!
//! Every layout works on the same container tree; the workspace's
//! `Layout` decides where new windows join it and how rects are computed
//! from it.

pub mod column_drop;
mod dwindle;
mod geometry;
pub mod scrolling;
mod split;

use anyhow::Context;
pub use geometry::tiling_rect;
use wm_common::{Layout, TilingDirection, WindowState};

use crate::{
  commands::container::attach_container,
  models::{
    is_inactive_stack_child, Container, TilingContainer, Workspace,
  },
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Where a new window joins the tree.
#[derive(Clone, Debug)]
pub enum Insertion {
  /// At `index` among `parent`'s children.
  At { parent: Container, index: usize },
  /// Sharing `target`'s slot, split along `direction`, first if
  /// `is_before`.
  Split {
    target: TilingContainer,
    direction: TilingDirection,
    is_before: bool,
  },
}

impl Insertion {
  pub fn workspace(&self) -> Option<Workspace> {
    match self {
      Self::At { parent, .. } => parent.workspace(),
      Self::Split { target, .. } => target.workspace(),
    }
  }

  /// The stack the window joins as a tab, if any.
  pub fn stack(&self) -> Option<crate::models::StackContainer> {
    match self {
      Self::At { parent, .. } => parent.as_stack().cloned(),
      Self::Split { .. } => None,
    }
  }
}

/// Gets where a new window goes, following the focused workspace's
/// layout.
///
/// Only tiling windows follow the layout; others are appended to the
/// workspace, and a window joining the focused stack goes in it.
pub fn insertion_target(
  window_state: &WindowState,
  joins_focused_stack: bool,
  state: &WmState,
) -> anyhow::Result<Insertion> {
  let workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No focused workspace.")?;

  let split = split_insertion(window_state, joins_focused_stack, state)?;

  if *window_state != WindowState::Tiling || split.stack().is_some() {
    return Ok(split);
  }

  match workspace.layout() {
    Layout::Split => Ok(split),
    Layout::Scrolling => Ok(scrolling::insertion_target(&workspace)),
    Layout::Dwindle => Insertion::split(
      dwindle::focused_target(&workspace, state),
      &workspace,
      state.dispatcher.cursor_position().ok(),
    ),
  }
}

fn split_insertion(
  window_state: &WindowState,
  joins_focused_stack: bool,
  state: &WmState,
) -> anyhow::Result<Insertion> {
  let (parent, index) =
    split::insertion_target(window_state, joins_focused_stack, state)?;

  Ok(Insertion::At { parent, index })
}

/// Whether the layout keeps `window` hidden although its workspace is
/// displayed: a stack's background tab, or a column scrolled off screen.
pub fn is_hidden_in_layout(window: &impl CommonGetters) -> bool {
  is_inactive_stack_child(window)
    || !scrolling::is_on_screen(&window.as_container())
}

/// Attaches a detached `child` where `insertion` says.
pub fn attach(
  child: &Container,
  insertion: &Insertion,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match insertion {
    Insertion::At { parent, index } => {
      attach_container(child, parent, Some(*index))
    }
    Insertion::Split {
      target,
      direction,
      is_before,
    } => {
      dwindle::attach_split(child, target, direction, *is_before, config)
    }
  }
}
