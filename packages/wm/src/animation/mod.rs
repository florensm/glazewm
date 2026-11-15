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
  WorkspaceSwitchEntry,
};
pub use manager::{AnimationManager, AnimationPositionResult};
