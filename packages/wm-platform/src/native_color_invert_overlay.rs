use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    UI::{
      Magnification::{
        MagGetWindowSource, MagInitialize, MagSetColorEffect,
        MagSetWindowFilterList, MagSetWindowSource, MAGCOLOREFFECT,
        MW_FILTERMODE_EXCLUDE, WC_MAGNIFIER,
      },
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindow,
        KillTimer, RegisterClassW, SetTimer, SetWindowPos, ShowWindow,
        GW_CHILD, GW_HWNDNEXT, GW_HWNDPREV, HWND_TOP, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER,
        SWP_SHOWWINDOW, SW_HIDE, WINDOW_EX_STYLE, WNDCLASSW, WS_CHILD,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
      },
    },
  },
};

use crate::{Dispatcher, NativeWindow, NativeWindowWindowsExt, Rect};

/// The default hue-rotation angle (in degrees) used when a window rule
/// doesn't specify one. See [`invert_hue_rotate_matrix`].
pub const DEFAULT_HUE_ROTATE_DEGREES: f32 = 180.0;

/// Builds an invert + hue-rotation color matrix for the given angle (in
/// degrees), in `MAGCOLOREFFECT`'s documented layout (same as
/// `System.Drawing.Imaging.ColorMatrix`: a 5x5 matrix applied as `output =
/// input_row_vector * matrix`, where the input row is `[r, g, b, a, 1]`
/// and the last matrix row is the translation row).
///
/// Composing a plain invert (`channel' = 1 - channel`) with a hue rotation
/// is the same trick as the CSS `filter: invert(1) hue-rotate(<angle>)`
/// combo used by dark-mode browser extensions: at the default 180°, it
/// maps whites near-black and keeps hues like blue recognizable, instead
/// of the harsh, washed-out look of a flat RGB negative -- but the same
/// rotation that keeps blues blue also pushes skin tones/photos toward
/// orange. There's no way to fix one without affecting the other with a
/// single global angle (this is a screen-capture-based filter with no
/// awareness of *which* pixels are a photo vs. UI chrome), so the angle is
/// tunable via `set-color-invert --hue-rotate=<degrees>` rather than
/// baked in, letting it be dialed in per-app.
///
/// Derived from the SVG `feColorMatrix type="hueRotate"` formula composed
/// with the invert step. Checked by hand (and by the unit tests below,
/// at the default angle) that black<->white round-trip and mid-gray (0.5)
/// is a fixed point -- true at every angle, since hue rotation is
/// grayscale-preserving by construction.
#[must_use]
pub fn invert_hue_rotate_matrix(degrees: f32) -> [f32; 25] {
  let radians = degrees.to_radians();
  let (s, c) = radians.sin_cos();

  // SVG `feColorMatrix type="hueRotate"` coefficients: `a[row][col]` is
  // the hue-rotation-only contribution of output channel `row` from input
  // channel `col` (row-sums are always 1, which is what makes the
  // transform grayscale-preserving).
  let a = [
    [
      0.213 + c * 0.787 - s * 0.213,
      0.715 - c * 0.715 - s * 0.715,
      0.072 - c * 0.072 + s * 0.928,
    ],
    [
      0.213 - c * 0.213 + s * 0.143,
      0.715 + c * 0.285 + s * 0.140,
      0.072 - c * 0.072 - s * 0.283,
    ],
    [
      0.213 - c * 0.213 - s * 0.787,
      0.715 - c * 0.715 + s * 0.715,
      0.072 + c * 0.928 + s * 0.072,
    ],
  ];

  // Composing with the invert step (`channel' = 1 - channel`) before the
  // hue rotation collapses to: `output_row = -a[row][0]*R - a[row][1]*G -
  // a[row][2]*B + (row sum of a[row])`, and each row of `a` always sums to
  // 1, so the translation term is always exactly `1.0`.
  #[rustfmt::skip]
  let matrix = [
    -a[0][0], -a[1][0], -a[2][0], 0.0, 0.0,
    -a[0][1], -a[1][1], -a[2][1], 0.0, 0.0,
    -a[0][2], -a[1][2], -a[2][2], 0.0, 0.0,
     0.0,      0.0,      0.0,     1.0, 0.0,
     1.0,      1.0,      1.0,     0.0, 1.0,
  ];

  matrix
}

/// Timer ID used to periodically force the magnifier to re-capture its
/// source region, and how often it fires.
///
/// `MagSetWindowSource` sets *which* screen region a `WC_MAGNIFIER` control
/// mirrors, but DWM can stop compositing (and the magnifier stops
/// re-capturing) a static, click-through layered window it doesn't think
/// needs continuous updates -- content changes *inside* the target window
/// (e.g. the app re-rendering its own UI) then never show up until
/// something else forces a recomposite, e.g. the overlay actually moving.
/// Re-applying the same source rect on a short timer forces a fresh
/// capture regardless. 10 times a second is frequent enough that content
/// changes feel live without meaningfully adding to the overlay's cost.
const REFRESH_TIMER_ID: usize = 1;
const REFRESH_INTERVAL_MS: u32 = 100;

/// `TIMERPROC` callback for [`REFRESH_TIMER_ID`].
///
/// Re-reads and re-applies the magnifier's own current source rect (rather
/// than needing access to the owning [`NativeColorInvertOverlay`]'s
/// state), purely to force a fresh capture -- the rect itself doesn't
/// change here.
unsafe extern "system" fn refresh_timer_proc(
  hwnd: HWND,
  _msg: u32,
  _id: usize,
  _time: u32,
) {
  // SAFETY: `hwnd` is the overlay host, valid for the lifetime of its
  // timer; `GW_CHILD` finds the `WC_MAGNIFIER` control created as its only
  // child.
  let magnifier = unsafe { GetWindow(hwnd, GW_CHILD) };
  if magnifier.0 == 0 {
    return;
  }

  let mut rect = RECT::default();

  // SAFETY: `magnifier` is a valid `WC_MAGNIFIER` control; `rect` is a
  // valid, properly sized out-parameter.
  let got_source = unsafe { MagGetWindowSource(magnifier, &raw mut rect) };

  if got_source.as_bool() {
    // SAFETY: `magnifier` is a valid `WC_MAGNIFIER` control.
    unsafe {
      let _ = MagSetWindowSource(magnifier, rect);
    }
  }
}

/// Ensures the overlay host window class is registered exactly once per
/// process.
static COLOR_INVERT_OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

/// Ensures `MagInitialize` is called exactly once per process, before the
/// first `WC_MAGNIFIER` control is created.
static MAG_INIT: OnceLock<()> = OnceLock::new();

/// Default window procedure for the overlay host class.
unsafe extern "system" fn default_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: All parameters are forwarded unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_class_registered() {
  COLOR_INVERT_OVERLAY_CLASS.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_ColorInvertOverlay"),
      lpfnWndProc: Some(default_wnd_proc),
      ..Default::default()
    };

    // SAFETY: `wnd_class` is properly initialized with a static class name
    // and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

fn ensure_magnification_initialized() {
  MAG_INIT.get_or_init(|| {
    // SAFETY: `MagInitialize` has no preconditions; safe to call once per
    // process before creating any `WC_MAGNIFIER` control.
    let initialized = unsafe { MagInitialize() };

    if !initialized.as_bool() {
      tracing::warn!(
        "MagInitialize failed; color invert overlays may not render."
      );
    }
  });
}

/// Creates the overlay's host window.
///
/// `WS_EX_TRANSPARENT` lets clicks and other input pass through to the
/// real window underneath, so the target stays fully interactive.
fn create_host_window(rect: &Rect) -> crate::Result<HWND> {
  ensure_class_registered();

  // SAFETY: The class is guaranteed registered by `ensure_class_registered`.
  // No parent HWND is needed.
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_LAYERED
        | WS_EX_TRANSPARENT
        | WS_EX_NOACTIVATE
        | WS_EX_TOOLWINDOW,
      w!("GlazeWM_ColorInvertOverlay"),
      w!(""),
      WS_POPUP,
      rect.x(),
      rect.y(),
      rect.width(),
      rect.height(),
      None,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create color invert overlay window.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// Creates the `WC_MAGNIFIER` child control that does the actual mirroring
/// and color-matrix work, filling `host`'s client area.
fn create_magnifier_control(host: HWND, rect: &Rect) -> crate::Result<HWND> {
  // SAFETY: `host` was just created by `create_host_window` and is a valid
  // parent for a child control.
  let hwnd = unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE::default(),
      WC_MAGNIFIER,
      w!(""),
      WS_CHILD | WS_VISIBLE,
      0,
      0,
      rect.width(),
      rect.height(),
      host,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create magnifier control.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// Applies the [`invert_hue_rotate_matrix`] for `hue_rotate_degrees` to
/// `magnifier`.
fn apply_color_effect(magnifier: HWND, hue_rotate_degrees: f32) {
  let mut effect = MAGCOLOREFFECT {
    transform: invert_hue_rotate_matrix(hue_rotate_degrees),
  };

  // SAFETY: `magnifier` is a valid `WC_MAGNIFIER` control; `effect` is a
  // fully initialized `MAGCOLOREFFECT`.
  unsafe {
    let _ = MagSetColorEffect(magnifier, &raw mut effect);
  }
}

/// Excludes `host` from what `magnifier` mirrors, so the lens never
/// captures its own overlay if the two ever visually coincide.
fn exclude_self_from_capture(magnifier: HWND, host: HWND) {
  let mut exclude = [host];

  // SAFETY: `magnifier` is valid; `exclude` is a properly sized buffer of
  // one valid `HWND`.
  unsafe {
    let _ = MagSetWindowFilterList(
      magnifier,
      MW_FILTERMODE_EXCLUDE,
      1,
      exclude.as_mut_ptr(),
    );
  }
}

/// Sets the screen-space source rect that `magnifier` mirrors.
fn set_source_rect(magnifier: HWND, rect: &Rect) {
  let source = RECT {
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
  };

  // SAFETY: `magnifier` is a valid `WC_MAGNIFIER` control.
  unsafe {
    let _ = MagSetWindowSource(magnifier, source);
  }
}

/// Resolves the z-order anchor to keep the overlay directly *above*
/// `target`.
///
/// `SetWindowPos`'s `hWndInsertAfter` places a window immediately *below*
/// whichever handle is passed -- there's no "insert before" parameter.
/// Sitting directly above `target` therefore means anchoring on whatever
/// currently sits directly above `target` itself (its `GW_HWNDPREV`), or
/// `HWND_TOP` if `target` is already the topmost window (`GW_HWNDPREV`
/// returns null in that case).
fn invert_overlay_z_anchor(target: HWND) -> HWND {
  // SAFETY: `target` is a valid window handle owned by another process;
  // `GetWindow` is safe to call on any valid handle.
  let prev = unsafe { GetWindow(target, GW_HWNDPREV) };

  if prev.0 == 0 {
    HWND_TOP
  } else {
    prev
  }
}

/// A persistent overlay window that inverts the colors of a paired managed
/// window, live.
///
/// Positioned directly *above* a `target` window in z-order and kept
/// pixel-aligned with its frame. Contains a `WC_MAGNIFIER` child control
/// that mirrors whatever is on screen directly beneath it and applies
/// [`invert_hue_rotate_matrix`], producing a hue-preserving "smart invert"
/// instead of a flat RGB negative.
///
/// The overlay window itself is `WS_EX_TRANSPARENT`, so clicks and other
/// input pass through to `target` underneath -- it stays fully
/// interactive.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeColorInvertOverlay {
  /// Raw window handle of the overlay host, stored as `isize` so that
  /// `NativeColorInvertOverlay` is `Send` even though `HWND` is not.
  hwnd: isize,

  /// Raw window handle of the child `WC_MAGNIFIER` control.
  magnifier_hwnd: isize,

  /// Last rect applied via [`set_rect`], used to skip redundant
  /// `SetWindowPos`/`MagSetWindowSource` calls when the overlay hasn't
  /// actually moved.
  ///
  /// [`set_rect`]: NativeColorInvertOverlay::set_rect
  rect: Rect,

  /// `HWND` of the window this overlay tracks, as raw `isize`. Tracked so
  /// [`set_rect`]/[`sync_z_order`] can skip a redundant `SetWindowPos`
  /// when the target hasn't changed.
  ///
  /// [`set_rect`]: NativeColorInvertOverlay::set_rect
  /// [`sync_z_order`]: NativeColorInvertOverlay::sync_z_order
  target: isize,

  /// Whether the overlay window is currently shown.
  ///
  /// Tracked explicitly (rather than inferred from a change in `rect`) so
  /// that a caller re-showing the overlay after [`hide`] with an unchanged
  /// rect still issues the `SetWindowPos` needed to reapply
  /// `SWP_SHOWWINDOW`.
  ///
  /// [`hide`]: NativeColorInvertOverlay::hide
  is_visible: bool,

  /// Hue-rotation angle (in degrees) last applied via [`apply_color_effect`],
  /// used to skip a redundant `MagSetColorEffect` call in [`set_hue_rotate`]
  /// when the value hasn't changed.
  ///
  /// [`set_hue_rotate`]: NativeColorInvertOverlay::set_hue_rotate
  hue_rotate_degrees: f32,

  /// Dispatcher for the event-loop thread that owns this overlay's
  /// windows.
  ///
  /// `WmState`'s own command/event processing runs on a separate worker
  /// thread from the one that pumps Win32 messages (`EventLoop::run`).
  /// `HWND`s are thread-affine for anything message-pump-dependent --
  /// window messages, including `WM_TIMER`, are only ever delivered on the
  /// thread that *created* the window -- so [`create`] and [`Drop`] use
  /// this to force creation and teardown onto that thread. Without it,
  /// [`REFRESH_TIMER_ID`] would silently never fire: the timer would still
  /// be registered, but nothing would ever pump the message loop that
  /// delivers it.
  ///
  /// `SetWindowPos`/`MagSetWindowSource` (used by [`set_rect`]/
  /// [`sync_z_order`]) don't have this requirement -- they take effect
  /// directly rather than via message delivery -- so those stay plain
  /// cross-thread calls.
  ///
  /// [`create`]: NativeColorInvertOverlay::create
  /// [`set_rect`]: NativeColorInvertOverlay::set_rect
  /// [`sync_z_order`]: NativeColorInvertOverlay::sync_z_order
  dispatcher: Dispatcher,
}

/// Creates the host + magnifier child windows and applies all one-time
/// setup. Must run on the event-loop thread (see the `dispatcher` field
/// doc on [`NativeColorInvertOverlay`]) -- callers dispatch this via
/// [`Dispatcher::dispatch_sync`].
fn create_overlay_windows(
  rect: &Rect,
  target_hwnd_raw: isize,
  hue_rotate_degrees: f32,
) -> crate::Result<(isize, isize)> {
  ensure_magnification_initialized();

  let target_hwnd = HWND(target_hwnd_raw);
  let host = create_host_window(rect)?;

  let magnifier = match create_magnifier_control(host, rect) {
    Ok(magnifier) => magnifier,
    Err(err) => {
      // SAFETY: `host` was just created above and not yet handed to a
      // caller; safe to destroy immediately on this failure path.
      unsafe {
        let _ = DestroyWindow(host);
      }
      return Err(err);
    }
  };

  apply_color_effect(magnifier, hue_rotate_degrees);
  exclude_self_from_capture(magnifier, host);
  set_source_rect(magnifier, rect);

  let anchor = invert_overlay_z_anchor(target_hwnd);

  // SAFETY: `host` is a valid window just created above.
  if let Err(e) = unsafe {
    SetWindowPos(
      host,
      anchor,
      rect.x(),
      rect.y(),
      rect.width(),
      rect.height(),
      SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
    )
  } {
    tracing::warn!(
      "Color invert overlay SetWindowPos failed on create: {e}."
    );
  }

  // SAFETY: `host` is a valid window just created above; `refresh_timer_proc`
  // matches the `TIMERPROC` signature.
  unsafe {
    SetTimer(
      host,
      REFRESH_TIMER_ID,
      REFRESH_INTERVAL_MS,
      Some(refresh_timer_proc),
    );
  }

  Ok((host.0, magnifier.0))
}

impl NativeColorInvertOverlay {
  /// Creates a new color invert overlay sized and positioned to `rect`,
  /// shown immediately, positioned directly above `target` in z-order.
  ///
  /// Window creation happens on `dispatcher`'s event-loop thread (see the
  /// `dispatcher` field doc), not the calling thread.
  pub fn create(
    rect: &Rect,
    target: &NativeWindow,
    hue_rotate_degrees: f32,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let target_hwnd_raw = target.hwnd().0;
    let rect_for_thread = rect.clone();

    let (host_raw, magnifier_raw) = dispatcher
      .dispatch_sync(move || {
        create_overlay_windows(
          &rect_for_thread,
          target_hwnd_raw,
          hue_rotate_degrees,
        )
      })??;

    Ok(Self {
      hwnd: host_raw,
      magnifier_hwnd: magnifier_raw,
      rect: rect.clone(),
      target: target_hwnd_raw,
      is_visible: true,
      hue_rotate_degrees,
      dispatcher: dispatcher.clone(),
    })
  }

  /// Returns the `HWND` of the overlay host window.
  fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Returns the `HWND` of the child `WC_MAGNIFIER` control.
  fn magnifier_hwnd(&self) -> HWND {
    HWND(self.magnifier_hwnd)
  }

  /// Returns whether the overlay window is currently shown.
  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.is_visible
  }

  /// Repositions and resizes the overlay to match `rect`, keeping it
  /// directly above `target`, and ensures it's shown.
  ///
  /// No-op if neither `rect` nor `target` changed and the overlay is
  /// already visible, to avoid redundant `SetWindowPos`/`MagSetWindowSource`
  /// calls on every sync tick for overlays that haven't actually moved.
  pub fn set_rect(&mut self, rect: &Rect, target: &NativeWindow) {
    let target_hwnd = target.hwnd();

    if self.is_visible && &self.rect == rect && self.target == target_hwnd.0
    {
      return;
    }

    let anchor = invert_overlay_z_anchor(target_hwnd);

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    if let Err(e) = unsafe {
      SetWindowPos(
        self.hwnd(),
        anchor,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    } {
      tracing::warn!("Color invert overlay SetWindowPos failed: {e}.");
      return;
    }

    // SAFETY: `self.magnifier_hwnd()` is a valid window handle for the
    // lifetime of this struct. `SWP_NOZORDER` is required since no
    // meaningful z-order anchor exists for a lone child control.
    unsafe {
      let _ = SetWindowPos(
        self.magnifier_hwnd(),
        None,
        0,
        0,
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
      );
    }

    set_source_rect(self.magnifier_hwnd(), rect);

    self.rect = rect.clone();
    self.target = target_hwnd.0;
    self.is_visible = true;
  }

  /// Corrects z-order drift by re-positioning the overlay directly above
  /// `target` if it isn't already there, without touching its rect.
  ///
  /// Cheap to call unconditionally every sync tick: `GetWindow`/`GW_HWNDNEXT`
  /// is a same-process, no-op-fast check, so this only issues a real
  /// `SetWindowPos` when the overlay actually needs to move (e.g. an
  /// unrelated window was brought to the foreground and landed between the
  /// overlay and its target).
  pub fn sync_z_order(&mut self, target: &NativeWindow) -> crate::Result<()> {
    let target_hwnd = target.hwnd();

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    let next = unsafe { GetWindow(self.hwnd(), GW_HWNDNEXT) };
    if next == target_hwnd {
      self.target = target_hwnd.0;
      return Ok(());
    }

    let anchor = invert_overlay_z_anchor(target_hwnd);

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        anchor,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOMOVE | SWP_NOSIZE,
      )
    }?;

    self.target = target_hwnd.0;
    Ok(())
  }

  /// Updates the hue-rotation angle (in degrees); re-applies only when the
  /// value changes.
  ///
  /// Compares the raw `f32` for exact equality -- the value only ever
  /// changes when a caller passes a genuinely different, command-resolved
  /// number, not through any arithmetic that could introduce drift.
  #[allow(clippy::float_cmp)]
  pub fn set_hue_rotate(&mut self, hue_rotate_degrees: f32) {
    if self.hue_rotate_degrees == hue_rotate_degrees {
      return;
    }

    self.hue_rotate_degrees = hue_rotate_degrees;
    apply_color_effect(self.magnifier_hwnd(), hue_rotate_degrees);
  }

  /// Hides the overlay without destroying it.
  pub fn hide(&mut self) {
    self.is_visible = false;
    // SAFETY: `self.hwnd()` is a valid window handle.
    unsafe {
      let _ = ShowWindow(self.hwnd(), SW_HIDE);
    }
  }
}

impl Drop for NativeColorInvertOverlay {
  fn drop(&mut self) {
    let host_raw = self.hwnd;
    let magnifier_raw = self.magnifier_hwnd;

    // Teardown must happen on the same thread the windows were created on
    // (see the `dispatcher` field doc). `dispatch_async` rather than
    // `dispatch_sync` since `Drop` has nothing to wait for, and the event
    // loop thread may itself already be mid-shutdown by the time this
    // runs -- either way, the OS reclaims the HWNDs when the process
    // exits, so a lost dispatch here during full shutdown is harmless.
    let _ = self.dispatcher.dispatch_async(move || {
      let host = HWND(host_raw);
      let magnifier = HWND(magnifier_raw);

      // SAFETY: `host`/`magnifier` are valid window handles and `Drop` is
      // called at most once. The magnifier control is destroyed
      // explicitly first (rather than relying on it being torn down
      // implicitly as a child of the host) so its `Mag*` state tears down
      // deterministically. The refresh timer is killed before either
      // window is destroyed so it can't fire against a handle
      // mid-teardown.
      unsafe {
        let _ = KillTimer(host, REFRESH_TIMER_ID);
        let _ = DestroyWindow(magnifier);
        let _ = DestroyWindow(host);
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::{invert_hue_rotate_matrix, DEFAULT_HUE_ROTATE_DEGREES};

  /// Applies `invert_hue_rotate_matrix(degrees)` to an `[r, g, b]` triple,
  /// using the same `[row-vector] * matrix` convention `MAGCOLOREFFECT`
  /// documents (input row is `[r, g, b, a, 1]`; alpha is irrelevant to the
  /// RGB output columns, so it's fixed at `0.0`).
  fn apply(rgb: [f32; 3], degrees: f32) -> [f32; 3] {
    let m = invert_hue_rotate_matrix(degrees);
    let input = [rgb[0], rgb[1], rgb[2], 0.0_f32, 1.0_f32];

    std::array::from_fn(|col| {
      (0..5).map(|row| input[row] * m[row * 5 + col]).sum()
    })
  }

  #[test]
  fn inverts_black_and_white_at_default_angle() {
    let white = apply([1.0, 1.0, 1.0], DEFAULT_HUE_ROTATE_DEGREES);
    for channel in white {
      assert!(
        channel.abs() < 1e-4,
        "white should map near black, got {white:?}"
      );
    }

    let black = apply([0.0, 0.0, 0.0], DEFAULT_HUE_ROTATE_DEGREES);
    for channel in black {
      assert!(
        (channel - 1.0).abs() < 1e-4,
        "black should map near white, got {black:?}"
      );
    }
  }

  #[test]
  fn mid_gray_is_a_fixed_point_at_any_angle() {
    // Hue rotation is grayscale-preserving by construction (each row of
    // the hue-rotate-only matrix sums to 1), so this should hold
    // regardless of the chosen angle, not just the default.
    for degrees in [0.0, 45.0, 90.0, 150.0, 180.0, 220.0, 300.0] {
      let gray = apply([0.5, 0.5, 0.5], degrees);
      for channel in gray {
        assert!(
          (channel - 0.5).abs() < 1e-4,
          "mid-gray should be unchanged at {degrees} degrees, got {gray:?}"
        );
      }
    }
  }

  #[test]
  fn zero_degrees_is_a_plain_invert() {
    // At a 0-degree hue rotation, the composed transform is just a plain
    // RGB negative (hue rotation by 0 is the identity).
    let inverted = apply([0.2, 0.6, 0.9], 0.0);
    let expected = [0.8, 0.4, 0.1];

    for (actual, expected) in inverted.iter().zip(expected) {
      assert!(
        (actual - expected).abs() < 1e-4,
        "expected plain invert at 0 degrees, got {inverted:?}"
      );
    }
  }
}
