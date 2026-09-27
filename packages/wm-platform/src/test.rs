#![feature(iterator_try_collect)]

#[macro_use]
extern crate libtest_mimic_collect;

#[cfg(target_os = "windows")]
mod color_levels;
mod color_theme;
mod dispatcher;
mod display;
mod error;
mod event_loop;
mod external_color_source;
mod keybinding_listener;
mod models;
mod mouse_listener;
#[cfg(target_os = "windows")]
mod native_surrogate;
mod native_window;
#[cfg(target_os = "windows")]
mod overlay_window;
pub mod perf;
mod platform_event;
mod platform_impl;
mod thread_bound;
#[cfg(target_os = "windows")]
mod window_class;
mod window_listener;

pub use color_theme::*;
pub use dispatcher::*;
pub use display::*;
pub use error::*;
pub use event_loop::*;
pub use external_color_source::*;
pub use keybinding_listener::*;
pub use models::*;
pub use mouse_listener::*;
#[cfg(target_os = "windows")]
pub use native_surrogate::SurrogateBatch;
pub use native_window::*;
pub use platform_event::*;
pub use thread_bound::*;
pub use window_listener::*;

pub fn main() {
  // Due to macOS requiring the main thread for some UI APIs, these
  // tests must execute on the main thread. Until this is natively
  // supported via cargo's test harness, we use `libtest_mimic_collect`.
  //
  // To run these tests, run `cargo test <...args> -- --test-threads=1`.
  //
  // Ref: https://github.com/rust-lang/rust/issues/104053
  libtest_mimic_collect::TestCollection::run();
}
