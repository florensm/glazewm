pub mod engine;
pub mod manager;
pub mod state;

#[cfg(target_os = "windows")]
pub use manager::ColorThemePlacement;
pub use manager::{AnimationManager, AnimationPositionResult};
