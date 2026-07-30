use std::{ffi::c_void, sync::OnceLock};

use windows::{
  Win32::{
    Foundation::HWND,
    System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
  },
  core::{s, w},
};

/// Accent state: solid-color fill, used for surrogate backdrops.
pub(crate) const ACCENT_ENABLE_GRADIENT: u32 = 1;

/// Accent state: acrylic blur-behind, the Win10 frosted-glass equivalent.
pub(crate) const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

/// Accent state: host backdrop -- samples live desktop content from behind
/// the window for a `Windows.UI.Composition` host-backdrop brush to pick
/// up, rather than blurring a solid color like `ACCENT_ENABLE_ACRYLICBLURBEHIND`.
/// The `gradient_color` field is unused for this accent state.
pub(crate) const ACCENT_ENABLE_HOSTBACKDROP: u32 = 5;

/// `WCA_ACCENT_POLICY` attribute index for
/// `SetWindowCompositionAttribute`.
const WCA_ACCENT_POLICY: u32 = 19;

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

/// Cached pointer to `SetWindowCompositionAttribute` from user32.dll.
static SET_WCA: OnceLock<Option<SetWindowCompositionAttributeFn>> =
  OnceLock::new();

/// Undocumented accent policy passed to `SetWindowCompositionAttribute`.
#[repr(C)]
struct AccentPolicy {
  accent_state: u32,
  accent_flags: u32,
  /// ABGR tint applied over the blurred backdrop.
  gradient_color: u32,
  animation_id: u32,
}

/// Descriptor for `SetWindowCompositionAttribute`.
#[repr(C)]
struct WindowCompositionAttribData {
  attrib: u32,
  pv_data: *mut c_void,
  cb_data: usize,
}

/// Retrieves the `SetWindowCompositionAttribute` function pointer from
/// user32.dll, caching it in a `OnceLock` for subsequent calls.
///
/// Returns `None` when the export is unavailable (pre-Windows 10 1607).
fn get_set_wca() -> Option<SetWindowCompositionAttributeFn> {
  *SET_WCA.get_or_init(|| {
    // SAFETY: user32.dll is always loaded in every Win32 process.
    // `GetModuleHandleW` does not increment the reference count.
    let module = unsafe { GetModuleHandleW(w!("user32.dll")).ok()? };

    // SAFETY: `module` is a valid handle. The ASCII string is
    // null-terminated via the `s!` macro.
    let proc = unsafe {
      GetProcAddress(module, s!("SetWindowCompositionAttribute"))
    }?;

    // SAFETY: `proc` is a valid export with the expected calling
    // convention and parameter layout.
    Some(unsafe {
      std::mem::transmute::<
        unsafe extern "system" fn() -> isize,
        SetWindowCompositionAttributeFn,
      >(proc)
    })
  })
}

/// Applies the given `accent_state` and `gradient_color` (ABGR) to `hwnd`
/// via the undocumented `SetWindowCompositionAttribute` API.
///
/// Returns `true` if the call succeeded, `false` if the API is unavailable
/// (pre-Windows 10 1607) or if the call itself failed.
pub(crate) fn apply_swca_accent(
  hwnd: HWND,
  accent_state: u32,
  gradient_color: u32,
) -> bool {
  let Some(set_wca) = get_set_wca() else {
    return false;
  };

  let mut policy = AccentPolicy {
    accent_state,
    accent_flags: 0,
    gradient_color,
    animation_id: 0,
  };

  let mut data = WindowCompositionAttribData {
    attrib: WCA_ACCENT_POLICY,
    pv_data: std::ptr::addr_of_mut!(policy).cast::<c_void>(),
    cb_data: std::mem::size_of::<AccentPolicy>(),
  };

  // SAFETY: `hwnd` is a valid window handle. `data` and `policy` are
  // stack-allocated and remain live for the duration of this call. The
  // struct layout matches the undocumented Win32 ABI for
  // `WCA_ACCENT_POLICY`.
  unsafe { set_wca(hwnd, std::ptr::addr_of_mut!(data)) != 0 }
}
