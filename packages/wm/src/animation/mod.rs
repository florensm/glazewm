pub mod engine;
pub mod focus_fade;
pub mod manager;
pub mod peek_view;
pub mod scroll_view;
pub mod stand_in;
pub mod state;
pub mod transit;

pub use manager::{AnimationManager, AnimationPositionResult, PeeksDrawn};
pub(crate) use manager::{AwayTransition, HandBack, WorkspaceSwitchEntry};
