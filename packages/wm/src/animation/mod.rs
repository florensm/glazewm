pub mod engine;
pub mod manager;
pub mod state;

#[cfg(target_os = "windows")]
pub(crate) use manager::WorkspaceSwitchEntry;
pub use manager::{AnimationManager, AnimationPositionResult};
