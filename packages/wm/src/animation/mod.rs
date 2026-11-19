//! Window animations, drawn by stand-ins while the real windows are
//! cloaked.
//!
//! Terms used throughout:
//!
//! - *Stand-in*: anything drawing a window in place of the real one.
//! - *Peek* (`WindowPeek`): a live thumbnail of one window, cut to a clip.
//!   Moves, opens, closes and resizes play on peeks (`peek_view`), as does
//!   the visible part of a scrolling layout's column that would reach onto
//!   another monitor (`scroll_view`).
//! - *Peek surface* (`PeekSurface`): one window per monitor that moving
//!   tiles' peeks draw on together.
//! - *Surrogate* (`WorkspaceSurrogate`): a window's stand-in in a
//!   workspace switch (`manager::workspace_switch`). `NativeSurrogate` is
//!   the thumbnail window both peeks and surrogates draw with.
//! - *Copies*: a stand-in's copies of its window's ring, backdrop and tab
//!   bar, which cover the real ones while it moves.
//! - *Hand-over* and *hand-back*: a stand-in taking over drawing a window,
//!   and giving the real window back as it lands.
//! - *Companion*: another app's window drawn over a window (e.g. shades'
//!   recolored copy), which a stand-in waits for at the hand-back.

pub mod engine;
pub mod focus_fade;
pub mod manager;
pub mod motion_clicks;
pub mod peek_view;
pub mod scroll_view;
pub mod stand_in;
pub mod state;
pub mod transit;

pub(crate) use manager::{
  restack_switch_tenants, switch_crossing, AwayTransition, HandBack,
  MakingWay, WorkspaceSwitchEntry,
};
pub use manager::{AnimationManager, AnimationPositionResult};
