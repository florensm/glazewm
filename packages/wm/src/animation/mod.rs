pub mod engine;
pub mod manager;
#[cfg(target_os = "windows")]
pub mod scroll_view;
#[cfg(target_os = "windows")]
pub mod stand_in;
pub mod state;

#[cfg(target_os = "windows")]
pub(crate) use manager::WorkspaceSwitchEntry;
pub use manager::{AnimationManager, AnimationPositionResult};
