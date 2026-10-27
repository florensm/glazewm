use windows::Win32::{
  Foundation::HWND,
  Graphics::Dwm::{
    DwmRegisterThumbnail, DwmUnregisterThumbnail,
    DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES,
  },
};

/// A registered DWM thumbnail, unregistered on drop, which must come
/// before its destination window is destroyed.
pub(crate) struct DwmThumbnail(isize);

impl DwmThumbnail {
  /// Registers a thumbnail of `source` on `dest`.
  pub(crate) fn register(dest: HWND, source: HWND) -> Option<Self> {
    // SAFETY: Registration fails for anything but two valid top-level
    // windows.
    unsafe { DwmRegisterThumbnail(dest, source) }.ok().map(Self)
  }

  pub(crate) fn update(
    &self,
    props: &DWM_THUMBNAIL_PROPERTIES,
  ) -> windows::core::Result<()> {
    // SAFETY: `self.0` stays registered until drop, and `props` outlives
    // the call.
    unsafe { DwmUpdateThumbnailProperties(self.0, props) }
  }
}

impl Drop for DwmThumbnail {
  fn drop(&mut self) {
    // SAFETY: Registered in `register` and unregistered only here.
    unsafe {
      let _ = DwmUnregisterThumbnail(self.0);
    }
  }
}
