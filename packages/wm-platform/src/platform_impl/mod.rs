#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;

pub(crate) use platform::*;

#[cfg(not(target_os = "windows"))]
compile_error!("The platform you're compiling for is not supported.");
