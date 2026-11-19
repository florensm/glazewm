//! The window overview: what it shows and how it lays that out and moves
//! (`state`, `layout`, `scene`, `effects`), and how it is drawn (`native`
//! and the rest).
//!
//! `model` holds what the WM sends it and what it sends back.

mod chrome;
mod effects;
mod layout;
mod model;
mod native;
mod overlays;
mod scene;
mod state;
mod tab_bars;
mod thumbnails;

pub use model::*;
pub use native::NativeOverview;
