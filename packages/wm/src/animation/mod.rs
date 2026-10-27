#[cfg(target_os = "windows")]
pub mod curve;
pub mod engine;
pub mod manager;
#[cfg(target_os = "windows")]
pub mod scene_switch;
#[cfg(target_os = "windows")]
pub mod scroll_view;
#[cfg(target_os = "windows")]
pub mod stand_in;
pub mod state;

pub use manager::{AnimationManager, AnimationPositionResult};
#[cfg(target_os = "windows")]
pub(crate) use manager::{AwayTransition, WorkspaceSwitchEntry};
