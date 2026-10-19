use wm_platform::{NativeWindow, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{NativeWindowWindowsExt, RectDelta};

#[derive(Clone, derive_more::Debug)]
pub struct NativeWindowProperties {
  /// Kept out of `Debug`, and so out of logs: it can hold private text.
  #[debug(skip)]
  pub title: String,
  #[cfg(target_os = "windows")]
  pub class_name: String,
  pub process_name: String,
  pub frame: Rect,
  pub is_minimized: bool,
  pub is_maximized: bool,
  pub is_resizable: bool,
  #[cfg(target_os = "windows")]
  pub shadow_borders: RectDelta,

  /// Narrowest its app lets the window be, shadow borders included, or 0
  /// if unknown. Only measured for a window being managed (see
  /// `measure_min_width`).
  pub min_width: i32,
}

impl NativeWindowProperties {
  /// Asks the window's app for the narrowest the window can be.
  ///
  /// Kept out of `try_from`, which runs for every window shown: the
  /// question waits on the window's own thread, which a busy app can take
  /// a while to answer.
  // LINT: Minimum sizes are only asked for on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(clippy::unused_self))]
  pub fn measure_min_width(&mut self, native_window: &NativeWindow) {
    #[cfg(target_os = "windows")]
    {
      self.min_width = native_window
        .min_window_size()
        .map_or(0, |(width, _)| width);
    }
    #[cfg(not(target_os = "windows"))]
    let _ = native_window;
  }
}

impl TryFrom<&NativeWindow> for NativeWindowProperties {
  type Error = anyhow::Error;

  fn try_from(native_window: &NativeWindow) -> Result<Self, Self::Error> {
    Ok(Self {
      title: native_window.title()?,
      #[cfg(target_os = "windows")]
      class_name: native_window.class_name()?,
      process_name: native_window.process_name()?,
      frame: native_window.frame()?,
      is_minimized: native_window.is_minimized()?,
      is_maximized: native_window.is_maximized()?,
      is_resizable: native_window.is_resizable()?,
      #[cfg(target_os = "windows")]
      shadow_borders: native_window.shadow_borders()?,
      min_width: 0,
    })
  }
}
