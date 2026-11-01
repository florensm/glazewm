use std::collections::HashMap;

use tracing::{debug, warn};
use uuid::Uuid;
use wm_platform::{NativeWindow, NativeWindowWindowsExt, SystemBackdrop};

use crate::{
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
};

/// DWM materials given to managed windows by the `system_backdrop`
/// effect, keyed by managed-window UUID.
///
/// A window is only tracked once DWM accepts its material, so a window on
/// an OS build without system backdrops keeps the `transparency` and
/// `backdrop` effects instead.
#[derive(Default)]
pub struct SystemBackdrops {
  applied: HashMap<Uuid, AppliedSystemBackdrop>,
}

struct AppliedSystemBackdrop {
  handle: isize,
  material: SystemBackdrop,
  extend_frame: bool,

  /// The window's material before this one, put back on release so an
  /// app that sets its own isn't left with ours.
  original: SystemBackdrop,
}

impl SystemBackdrops {
  /// Whether DWM paints a material behind the window, in place of its
  /// `transparency` and `backdrop` effects.
  pub fn contains(&self, window_id: &Uuid) -> bool {
    self.applied.contains_key(window_id)
  }

  /// Gives `window` the material `config` assigns it, or restores its
  /// original one if it no longer has one.
  pub fn sync(&mut self, window: &WindowContainer, config: &UserConfig) {
    let wanted = config.system_backdrop_for(&window.native_properties());

    let Some((material, extend_frame)) = wanted else {
      self.release(&window.id());
      return;
    };

    let native = window.native();

    if let Some(applied) = self.applied.get_mut(&window.id()) {
      if applied.material != material {
        if let Err(err) = native.set_system_backdrop(material) {
          warn!("Failed to change system backdrop: {err}.");
        } else {
          applied.material = material;
        }
      }

      if applied.extend_frame != extend_frame {
        match native.set_frame_extended(extend_frame) {
          Ok(()) => applied.extend_frame = extend_frame,
          Err(err) => warn!("Failed to change frame extension: {err}."),
        }
      }

      return;
    }

    // Fails on builds before 22H2, which have no system backdrops.
    let original = match native.system_backdrop() {
      Ok(original) => original,
      Err(err) => {
        debug!("System backdrop unsupported for {}: {err}.", window.id());
        return;
      }
    };

    if let Err(err) = native.set_system_backdrop(material) {
      debug!("Failed to set system backdrop on {}: {err}.", window.id());
      return;
    }

    let extend_frame = extend_frame
      && native
        .set_frame_extended(true)
        .inspect_err(|err| warn!("Failed to extend frame: {err}."))
        .is_ok();

    self.applied.insert(
      window.id(),
      AppliedSystemBackdrop {
        handle: native.hwnd().0,
        material,
        extend_frame,
        original,
      },
    );
  }

  /// Gives every managed window the material `config` assigns it.
  pub fn sync_all(
    &mut self,
    windows: &[WindowContainer],
    config: &UserConfig,
  ) {
    self.applied.retain(|id, applied| {
      let is_managed = windows.iter().any(|window| window.id() == *id);
      if !is_managed {
        applied.restore();
      }
      is_managed
    });

    for window in windows {
      self.sync(window, config);
    }
  }

  /// Puts back the window's original material. Errors are ignored, as
  /// the window is often already destroyed.
  pub fn release(&mut self, window_id: &Uuid) {
    if let Some(applied) = self.applied.remove(window_id) {
      applied.restore();
    }
  }

  /// Puts back every window's original material, for when the WM exits.
  pub fn release_all(&mut self) {
    for (_, applied) in self.applied.drain() {
      applied.restore();
    }
  }
}

impl AppliedSystemBackdrop {
  fn restore(&self) {
    let native = NativeWindow::from_handle(self.handle);
    let _ = native.set_system_backdrop(self.original);

    if self.extend_frame {
      let _ = native.set_frame_extended(false);
    }
  }
}
