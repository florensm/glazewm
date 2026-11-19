pub mod engine;
pub mod focus_fade;
pub mod manager;
pub mod scroll_view;
pub mod stand_in;
pub mod state;

pub use manager::{AnimationManager, AnimationPositionResult};
pub(crate) use manager::{AwayTransition, HandBack, WorkspaceSwitchEntry};
