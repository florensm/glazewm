use windows::Win32::Graphics::Gdi::HMONITOR;

use crate::{platform_impl, Rect};

/// Unique identifier for a display.
///
/// Can be obtained with `display.id()`. Wraps the display's `HMONITOR`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayId(pub isize);

/// Unique identifier for a display device.
///
/// Can be obtained with `display_device.id()`. Holds the device's
/// hardware ID, falling back to its adapter name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayDeviceId(pub String);

/// Represents a logical display space where windows can be placed.
///
/// This corresponds to a Win32 "display monitor", each with a monitor
/// handle (`HMONITOR`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
  pub(crate) inner: platform_impl::Display,
}

impl Display {
  /// Gets the unique identifier for this display.
  #[must_use]
  pub fn id(&self) -> DisplayId {
    self.inner.id()
  }

  /// Gets the display name.
  pub fn name(&self) -> crate::Result<String> {
    self.inner.name()
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> crate::Result<Rect> {
    self.inner.bounds()
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_area(&self) -> crate::Result<Rect> {
    self.inner.working_area()
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> crate::Result<f32> {
    self.inner.scale_factor()
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> crate::Result<u32> {
    self.inner.dpi()
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> crate::Result<bool> {
    self.inner.is_primary()
  }

  /// Gets the main device (the first active one) for this display.
  pub fn main_device(&self) -> crate::Result<DisplayDevice> {
    self.inner.main_device()
  }

  /// Gets the monitor handle.
  #[must_use]
  pub fn hmonitor(&self) -> HMONITOR {
    self.inner.hmonitor()
  }
}

/// Represents a display device (physical or virtual).
///
/// This is typically a physical display device, such as a monitor or
/// built-in laptop screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDevice {
  pub(crate) inner: platform_impl::DisplayDevice,
}

impl DisplayDevice {
  /// Gets the unique identifier for this display device.
  #[must_use]
  pub fn id(&self) -> DisplayDeviceId {
    self.inner.id()
  }

  /// Gets the device path.
  ///
  /// This can be an empty string for virtual display devices.
  #[must_use]
  pub fn device_path(&self) -> Option<String> {
    self.inner.device_path.clone()
  }

  /// Gets the hardware ID from the device path.
  ///
  /// # Example usage
  ///
  /// ```ignore
  /// device.device_path(); // "\\?\DISPLAY#DEL40A3#5&1234abcd&0&UID256#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}"
  /// device.hardware_id(); // Some("DEL40A3")
  /// ```
  #[must_use]
  pub fn hardware_id(&self) -> Option<String> {
    self.inner.hardware_id()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::EventLoop;

  #[test]
  fn test_nearest_display() {
    let (event_loop, dispatcher) = EventLoop::new().unwrap();

    let thread = std::thread::spawn(move || {
      let display = platform_impl::nearest_display(
        // Assumes that there is at least one window currently visible.
        &dispatcher.visible_windows().unwrap()[0],
        &dispatcher,
      );
      dispatcher.stop_event_loop().unwrap();
      display
    });

    event_loop.run().unwrap();
    let display = thread.join().unwrap();
    assert!(display.bounds().is_ok());
  }
}
