pub mod engine;
pub mod focus_fade;
pub mod manager;
pub mod peek_view;
pub mod scroll_view;
pub mod stand_in;
pub mod state;
// Unused until task 3 of `STAND_INS_PLAN.md` draws opens with it.
#[allow(dead_code)]
pub mod transit;

pub use manager::{AnimationManager, AnimationPositionResult};
pub(crate) use manager::{AwayTransition, HandBack, WorkspaceSwitchEntry};
