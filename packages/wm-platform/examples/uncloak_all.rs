//! Recovery tool: un-cloaks every top-level window that GlazeWM left hidden.
//!
//! GlazeWM hides windows on inactive workspaces with
//! `DwmSetWindowAttribute(DWMWA_CLOAK)`, which survives the process. Normally
//! `wm-watcher` reverses that when the WM dies unexpectedly, but it can only
//! restore windows the WM had reported as managed -- so a WM killed while its
//! watcher was also killed leaves those windows alive, running, and invisible,
//! with nothing left that knows about them.
//!
//! This walks every top-level window and clears the cloak on the ones that
//! look like real application windows.
//!
//! # Example usage
//!
//! ```text
//! cargo run -p wm-platform --release --example uncloak_all
//! ```
//!
//! # Platform-specific
//!
//! - Windows: does the work described above.
//! - macOS: no cloaking exists, so this is a no-op.

fn main() {
  #[cfg(target_os = "windows")]
  windows_impl::run();

  #[cfg(not(target_os = "windows"))]
  println!("Nothing to do: window cloaking is Windows-only.");
}

#[cfg(target_os = "windows")]
mod windows_impl {
  use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::WindowsAndMessaging::EnumWindows,
  };
  use wm_platform::{NativeWindow, NativeWindowWindowsExt};

  /// Collects every top-level window handle.
  fn all_handles() -> Vec<isize> {
    let mut handles: Vec<isize> = Vec::new();

    extern "system" fn collect(handle: HWND, data: LPARAM) -> BOOL {
      let handles = data.0 as *mut Vec<isize>;
      // SAFETY: `data` is the `Vec<isize>` passed in by the `EnumWindows`
      // call below, which outlives the enumeration.
      unsafe { (*handles).push(handle.0) };
      true.into()
    }

    // SAFETY: `collect` matches the expected callback signature and
    // `handles` outlives the call.
    let _ = unsafe {
      EnumWindows(
        Some(collect),
        LPARAM(std::ptr::from_mut(&mut handles) as _),
      )
    };

    handles
  }

  /// Un-cloaks every cloaked window that carries a title.
  ///
  /// The title check is the filter that keeps this from revealing the
  /// invisible helper windows many applications keep cloaked on purpose;
  /// those are overwhelmingly untitled.
  pub fn run() {
    let mut restored = 0;

    for handle in all_handles() {
      let window = NativeWindow::from_handle(handle);

      if !window.is_cloaked().unwrap_or(false) {
        continue;
      }

      let Ok(title) = window.title() else {
        continue;
      };

      if title.trim().is_empty() {
        continue;
      }

      match window.set_cloaked(false) {
        Ok(()) => {
          restored += 1;
          println!("restored: {title}");
        }
        Err(err) => println!("failed on {title}: {err}"),
      }
    }

    println!("\n{restored} window(s) un-cloaked.");
  }
}
