use windows::Win32::{
  Foundation::HWND,
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_SYSTEMBACKDROP_TYPE},
};

use crate::platform_impl::swca::query_accent_state;

/// Default ABGR tint applied to a surrogate's live blur when the backdrop
/// was found via [`has_live_backdrop`] rather than GlazeWM's own
/// `blur_behind` config.
///
/// Neither `DWMWA_SYSTEMBACKDROP_TYPE` nor `WCA_ACCENT_POLICY` exposes the
/// source window's actual configured tint, so this is a deliberately
/// unobtrusive near-transparent default rather than a guess. Alpha must be
/// non-zero — some Windows builds render no blur at all for
/// `ACCENT_ENABLE_ACRYLICBLURBEHIND` at alpha 0.
pub(crate) const LIVE_BLUR_DEFAULT_TINT: u32 = 0x20FF_FFFF;

/// DWM thumbnail opacity cap used when a surrogate carries a probe-detected
/// live blur backdrop (as opposed to GlazeWM's own configured effect).
///
/// The live blur sits underneath the thumbnail in the same surrogate's
/// client area — at full thumbnail opacity the thumbnail would completely
/// hide it. GlazeWM doesn't control the probed window's own opacity config
/// (unlike its own `blur_behind` effect, which is only visible in
/// combination with a configured `transparency` effect), so this caps the
/// thumbnail opacity unconditionally to let the blur bleed through and avoid
/// regressing what the source (Windhawk, native Mica) was already showing.
/// An approximation, not a per-pixel reproduction — it washes out the whole
/// thumbnail evenly, not just the parts the source window made translucent.
pub(crate) const LIVE_BLUR_THUMBNAIL_OPACITY: u8 = 190;

/// Whether `hwnd` has a live DWM backdrop translucency effect applied.
///
/// Covers both the modern Windows 11 system-backdrop API
/// (`DWMWA_SYSTEMBACKDROP_TYPE`, used by Mica/Acrylic/MicaAlt apps and by
/// Windhawk's `translucent-windows` mod for its Acrylic/Mica/MicaAlt
/// presets) and the legacy undocumented Windows 10 accent-policy blur
/// (`WCA_ACCENT_POLICY`, used by that same mod's `AccentBlurBehind` preset,
/// which leaves `DWMWA_SYSTEMBACKDROP_TYPE` untouched at its default).
///
/// `DwmRegisterThumbnail` flattens either effect to an opaque bitmap, so
/// callers use this to decide whether an animation surrogate should carry a
/// live acrylic backdrop of its own rather than showing a suddenly-opaque
/// thumbnail.
pub fn has_live_backdrop(hwnd: HWND) -> bool {
  probe_system_backdrop(hwnd).is_some_and(is_system_backdrop_active)
    || query_accent_state(hwnd).is_some_and(is_accent_blur_active)
}

/// Whether a `DWM_SYSTEMBACKDROP_TYPE` value indicates an active backdrop.
///
/// `DWMSBT_AUTO` (0, the untouched default) and `DWMSBT_NONE` (1) are not
/// active backdrops; `DWMSBT_MAINWINDOW`/`TRANSIENTWINDOW`/`TABBEDWINDOW`
/// (2/3/4) are.
fn is_system_backdrop_active(value: i32) -> bool {
  matches!(value, 2..=4)
}

/// Whether a `WCA_ACCENT_POLICY` accent-state value indicates an active
/// blur.
///
/// `ACCENT_DISABLED` (0), `ACCENT_ENABLE_GRADIENT` (1), and
/// `ACCENT_ENABLE_TRANSPARENTGRADIENT` (2) are solid-fill states, not blur.
/// `ACCENT_ENABLE_BLURBEHIND` (3), `ACCENT_ENABLE_ACRYLICBLURBEHIND` (4),
/// and `ACCENT_ENABLE_HOSTBACKDROP` (5) are live blur states.
fn is_accent_blur_active(value: u32) -> bool {
  matches!(value, 3..=5)
}

/// Queries `hwnd`'s `DWMWA_SYSTEMBACKDROP_TYPE` via `DwmGetWindowAttribute`.
///
/// Returns `None` if the query fails (pre-Windows 11, or an invalid/elevated
/// handle under UIPI).
fn probe_system_backdrop(hwnd: HWND) -> Option<i32> {
  let mut value = 0i32;
  // SAFETY: `hwnd` is a valid window handle. `value` is a stack-allocated
  // `i32` live for the duration of the call, matching
  // `DWM_SYSTEMBACKDROP_TYPE`'s underlying representation.
  let ok = unsafe {
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_SYSTEMBACKDROP_TYPE,
      std::ptr::addr_of_mut!(value).cast(),
      std::mem::size_of::<i32>() as u32,
    )
  }
  .is_ok();
  ok.then_some(value)
}

#[cfg(test)]
mod tests {
  use super::{is_accent_blur_active, is_system_backdrop_active};

  #[test]
  fn system_backdrop_active_states() {
    assert!(!is_system_backdrop_active(0)); // DWMSBT_AUTO
    assert!(!is_system_backdrop_active(1)); // DWMSBT_NONE
    assert!(is_system_backdrop_active(2)); // DWMSBT_MAINWINDOW
    assert!(is_system_backdrop_active(3)); // DWMSBT_TRANSIENTWINDOW
    assert!(is_system_backdrop_active(4)); // DWMSBT_TABBEDWINDOW
    assert!(!is_system_backdrop_active(5));
  }

  #[test]
  fn accent_blur_active_states() {
    assert!(!is_accent_blur_active(0)); // ACCENT_DISABLED
    assert!(!is_accent_blur_active(1)); // ACCENT_ENABLE_GRADIENT
    assert!(!is_accent_blur_active(2)); // ACCENT_ENABLE_TRANSPARENTGRADIENT
    assert!(is_accent_blur_active(3)); // ACCENT_ENABLE_BLURBEHIND
    assert!(is_accent_blur_active(4)); // ACCENT_ENABLE_ACRYLICBLURBEHIND
    assert!(is_accent_blur_active(5)); // ACCENT_ENABLE_HOSTBACKDROP
    assert!(!is_accent_blur_active(6));
  }
}
