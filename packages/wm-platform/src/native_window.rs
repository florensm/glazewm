use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    SET_WINDOW_POS_FLAGS, SWP_NOCOPYBITS, WINDOW_EX_STYLE, WINDOW_STYLE,
  },
};

use crate::{
  platform_impl, CornerStyle, Delta, OpacityValue, Rect, RectDelta,
};

/// Unique identifier of a window.
///
/// Can be obtained with `window.id()`. Wraps the window's `HWND`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub isize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowZOrder {
  Normal,
  AfterWindow(WindowId),
  Top,
  TopMost,
}

/// `SWP_NOCOPYBITS` when moving a window from `current` to `target`
/// changes its size, or when `current` is unknown.
///
/// Without it, the OS copies the old client area into the resized window
/// and asks the app to paint only the uncovered part. Many apps then keep
/// showing their content at the old size with the rest black, until a
/// resize by hand forces a full repaint. A pure move keeps the old pixels,
/// which are still correct, and spares the app a repaint.
#[must_use]
pub fn discard_bits_if_resized(
  current: Option<&Rect>,
  target: &Rect,
) -> SET_WINDOW_POS_FLAGS {
  let keeps_size = current.is_some_and(|current| {
    current.width() == target.width()
      && current.height() == target.height()
  });

  if keeps_size {
    SET_WINDOW_POS_FLAGS(0)
  } else {
    SWP_NOCOPYBITS
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  pub(crate) inner: platform_impl::NativeWindow,
}

impl NativeWindow {
  /// Gets the unique identifier for this window.
  #[must_use]
  pub fn id(&self) -> WindowId {
    self.inner.id()
  }

  /// Gets the window's title.
  ///
  /// Note that empty strings are valid window titles.
  #[must_use]
  pub fn title(&self) -> String {
    self.inner.title()
  }

  pub fn process_name(&self) -> crate::Result<String> {
    self.inner.process_name()
  }

  /// Gets a rectangle of the window's size and position.
  ///
  /// Includes the window's shadow borders.
  pub fn frame(&self) -> crate::Result<Rect> {
    let _scope = crate::perf::scope(crate::perf::Stage::NativeFrame);
    self.inner.frame()
  }

  /// Gets the window's position as (x, y) coordinates.
  pub fn position(&self) -> crate::Result<(f64, f64)> {
    self.inner.position()
  }

  /// Gets the window's size as (width, height).
  pub fn size(&self) -> crate::Result<(f64, f64)> {
    self.inner.size()
  }

  /// Whether the window is still valid.
  ///
  /// Returns `true` if the underlying window is still alive.
  #[must_use]
  pub fn is_valid(&self) -> bool {
    self.inner.is_valid()
  }

  /// Whether the window is actually visible.
  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.inner.is_visible()
  }

  /// Whether the window is minimized.
  #[must_use]
  pub fn is_minimized(&self) -> bool {
    self.inner.is_minimized()
  }

  /// Whether the window is maximized.
  #[must_use]
  pub fn is_maximized(&self) -> bool {
    self.inner.is_maximized()
  }

  /// Whether the window can be resized.
  #[must_use]
  pub fn is_resizable(&self) -> bool {
    self.inner.is_resizable()
  }

  /// Whether the window is the OS's desktop window.
  #[must_use]
  pub fn is_desktop_window(&self) -> bool {
    self.inner.is_desktop_window()
  }

  /// Repositions and resizes the window to the specified rectangle.
  ///
  /// Automatically adjusts the `rect` prior to calling [`SetWindowPos`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) to include the window's shadow borders. To set the window's position directly, use [`Self::set_window_pos`].
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    self.inner.set_frame(rect)
  }

  /// Resizes the window to the specified size.
  pub fn resize(&self, width: i32, height: i32) -> crate::Result<()> {
    self.inner.resize(width, height)
  }

  /// Repositions the window to the specified position.
  pub fn reposition(&self, x: i32, y: i32) -> crate::Result<()> {
    self.inner.reposition(x, y)
  }

  pub fn minimize(&self) -> crate::Result<()> {
    self.inner.minimize()
  }

  pub fn maximize(&self) -> crate::Result<()> {
    self.inner.maximize()
  }

  /// Sets focus to the window and raises it to the top of the z-order.
  pub fn focus(&self) -> crate::Result<()> {
    self.inner.focus()
  }

  /// Closes the window.
  ///
  /// This sends a `WM_CLOSE` message to the window.
  pub fn close(&self) -> crate::Result<()> {
    self.inner.close()
  }

  /// Creates a [`NativeWindow`] from a window handle.
  #[must_use]
  pub fn from_handle(handle: isize) -> Self {
    platform_impl::NativeWindow::new(handle).into()
  }

  /// Gets the window handle.
  #[must_use]
  pub fn hwnd(&self) -> HWND {
    self.inner.hwnd()
  }

  /// Gets the class name of the window.
  pub fn class_name(&self) -> crate::Result<String> {
    self.inner.class_name()
  }

  /// Gets the window's frame, including the window's shadow borders.
  pub fn frame_with_shadows(&self) -> crate::Result<Rect> {
    self.inner.frame_with_shadows()
  }

  /// Gets the delta between the window's frame and the window's border.
  /// This represents the size of a window's shadow borders.
  pub fn shadow_borders(&self) -> crate::Result<RectDelta> {
    self.inner.shadow_borders()
  }

  /// Smallest width and height the window's app lets it be resized to,
  /// shadow borders included, as it answers `WM_GETMINMAXINFO`.
  ///
  /// Asks the app, waiting a moment at most; `None` if it doesn't answer
  /// in time.
  #[must_use]
  pub fn min_window_size(&self) -> Option<(i32, i32)> {
    self.inner.min_window_size()
  }

  /// Whether the window has an owner window.
  #[must_use]
  pub fn has_owner_window(&self) -> bool {
    self.inner.has_owner_window()
  }

  /// ID of the window's owner window, if it has one.
  #[must_use]
  pub fn owner_window_id(&self) -> Option<WindowId> {
    self.inner.owner_window_id()
  }

  /// Whether the window is shown, but cloaked along with its cloaked owner
  /// window, e.g. while the WM animates the owner.
  #[must_use]
  pub fn is_cloaked_by_owner(&self) -> bool {
    self.inner.is_cloaked_by_owner()
  }

  /// Presses `keys` together (e.g. `ctrl+p`) in the window, if it is the
  /// foreground window.
  ///
  /// Returns `false` without sending anything if it isn't, since key
  /// presses always go to the foreground window.
  pub fn press_keys(&self, keys: &[crate::Key]) -> crate::Result<bool> {
    self.inner.press_keys(keys)
  }

  /// Whether the window's parent is the desktop, rather than another
  /// window it was embedded into (e.g. via `SetParent`).
  #[must_use]
  pub fn is_top_level(&self) -> bool {
    self.inner.is_top_level()
  }

  /// ID of the process that owns the window.
  #[must_use]
  pub fn process_id(&self) -> u32 {
    self.inner.process_id()
  }

  /// Whether the window accepts mouse and keyboard input, i.e. it hasn't
  /// been disabled with `EnableWindow`.
  #[must_use]
  pub fn is_enabled(&self) -> bool {
    self.inner.is_enabled()
  }

  /// Whether the window is disabled behind a shown, enabled popup it owns,
  /// i.e. a modal dialog such as a "Save changes?" prompt.
  #[must_use]
  pub fn has_modal_prompt(&self) -> bool {
    self.inner.has_modal_prompt()
  }

  /// Whether the window's app has stopped responding, as judged by the OS
  /// (no messages processed for about 5 seconds).
  #[must_use]
  pub fn is_hung(&self) -> bool {
    self.inner.is_hung()
  }

  /// Re-enables input to a window its app disabled, without blocking:
  /// `EnableWindow` waits for the app to handle `WM_ENABLE`, so it is
  /// called from a worker thread.
  pub fn enable_async(&self) {
    self.inner.enable_async();
  }

  /// Whether the window has the given window style flag(s) set.
  #[must_use]
  pub fn has_window_style(&self, style: WINDOW_STYLE) -> bool {
    self.inner.has_window_style(style)
  }

  /// Whether the window has the given extended window style flag(s) set.
  #[must_use]
  pub fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool {
    self.inner.has_window_style_ex(style)
  }

  /// Thin wrapper around [`SetWindowPos`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos).
  pub fn set_window_pos(
    &self,
    z_order: &WindowZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()> {
    self.inner.set_window_pos(z_order, rect, flags)
  }

  /// Shows the window asynchronously.
  ///
  /// NOTE: Cloaked windows do not get shown until uncloaked.
  pub fn show(&self) -> crate::Result<()> {
    self.inner.show()
  }

  /// Hides the window asynchronously.
  pub fn hide(&self) -> crate::Result<()> {
    self.inner.hide()
  }

  /// Restores the window (unminimizes and unmaximizes).
  ///
  /// If `outer_frame` is provided, the window will be restored to the
  /// specified position. This avoids flickering compared to restoring
  /// and then repositioning the window.
  pub fn restore(&self, outer_frame: Option<&Rect>) -> crate::Result<()> {
    self.inner.restore(outer_frame)
  }

  /// Cloaks or uncloaks the window.
  pub fn set_cloaked(&self, cloaked: bool) -> crate::Result<()> {
    self.inner.set_cloaked(cloaked)
  }

  /// Cloaks or uncloaks the window, or hides or shows it if it can't be
  /// cloaked.
  ///
  /// Only app windows can be cloaked, not e.g. the windows an app's main
  /// window owns, which `set_cloaked` would leave on screen. Unlike a
  /// cloaked window, a hidden one is not rendered for thumbnails.
  pub fn set_cloaked_or_hidden(&self, hidden: bool) -> crate::Result<()> {
    self.inner.set_cloaked_or_hidden(hidden)
  }

  /// Whether the shell has a view of the window to cloak it by. Without
  /// one, `set_cloaked_or_hidden` hides the window instead.
  ///
  /// `false` as well if the shell can't be asked.
  #[must_use]
  pub fn can_be_cloaked(&self) -> bool {
    self.inner.can_be_cloaked()
  }

  /// Returns whether the window is currently cloaked by DWM.
  ///
  /// Cloaking hides the window visually while keeping `WS_VISIBLE` set.
  /// The WM uses this for workspace hiding and surrogate animations.
  pub fn is_cloaked(&self) -> crate::Result<bool> {
    self.inner.is_cloaked()
  }

  /// Whether the window currently sits in the always-on-top band.
  #[must_use]
  pub fn is_topmost(&self) -> bool {
    crate::window_class::is_topmost(self.inner.hwnd())
  }

  /// Stops the window's taskbar button from flashing.
  ///
  /// Has no effect if the window isn't flashing.
  pub fn stop_flashing(&self) {
    self.inner.stop_flashing();
  }

  /// Marks the window as fullscreen.
  ///
  /// Causes the native Windows taskbar to be moved to the bottom of the
  /// z-order when this window is active.
  pub fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.inner.mark_fullscreen(fullscreen)
  }

  /// Adds or removes the window from the native taskbar.
  ///
  /// Cloaked windows are normally always shown in the taskbar, but can be
  /// manually toggled. Hidden windows (`SW_HIDE`) can never be shown in
  /// the taskbar.
  pub fn set_taskbar_visibility(
    &self,
    visible: bool,
  ) -> crate::Result<()> {
    self.inner.set_taskbar_visibility(visible)
  }

  /// Adds the given extended window style flag(s) to the window.
  pub fn add_window_style_ex(&self, style: WINDOW_EX_STYLE) {
    self.inner.add_window_style_ex(style);
  }

  /// Sets the window's z-order.
  pub fn set_z_order(&self, z_order: &WindowZOrder) -> crate::Result<()> {
    self.inner.set_z_order(z_order)
  }

  /// Sets the visibility of the window's title bar.
  pub fn set_title_bar_visibility(
    &self,
    visible: bool,
  ) -> crate::Result<()> {
    self.inner.set_title_bar_visibility(visible)
  }

  /// Hides or restores the window's own OS-drawn border, so it doesn't
  /// show as a second line inside a [`NativeBorderOverlay`] ring.
  ///
  /// Only does anything on Windows 11 (build 22000) and later -- earlier
  /// builds draw no such border and return `Err`.
  ///
  /// [`NativeBorderOverlay`]: crate::NativeBorderOverlay
  pub fn set_native_border_hidden(
    &self,
    hidden: bool,
  ) -> crate::Result<()> {
    self.inner.set_native_border_hidden(hidden)
  }

  /// Sets the corner style of the window.
  pub fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()> {
    self.inner.set_corner_style(corner_style)
  }

  /// Sets the transparency of the window.
  pub fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()> {
    self.inner.set_transparency(opacity_value)
  }

  /// Adjusts the window's transparency by a relative delta.
  pub fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()> {
    self.inner.adjust_transparency(opacity_delta)
  }

  /// Re-applies the window's current layered-window alpha, if any, without
  /// changing its value.
  ///
  /// `SWP_FRAMECHANGED` (issued whenever a window is resized) forces a
  /// non-client-area recalculation that can make DWM briefly composite the
  /// window at full opacity before the existing `LWA_ALPHA` value is
  /// reasserted — a one-frame flash to solid on every
  /// resize/move/workspace- switch landing for any window using the
  /// `transparency` effect. Calling this immediately after such a
  /// `SetWindowPos` forces DWM to recomposite with the correct alpha
  /// right away, closing that gap. No-op if the window isn't currently
  /// layered.
  pub fn reassert_transparency(&self) -> crate::Result<()> {
    self.inner.reassert_transparency()
  }

  /// The window's layered-window alpha, or `None` when it has none, e.g.
  /// when the `transparency` effect was never applied to it.
  pub fn layered_alpha(&self) -> crate::Result<Option<u8>> {
    self.inner.layered_alpha()
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.inner.id() == other.inner.id()
  }
}

impl Eq for NativeWindow {}

#[cfg(test)]
mod tests {
  use super::{
    discard_bits_if_resized, Rect, SET_WINDOW_POS_FLAGS, SWP_NOCOPYBITS,
  };

  #[test]
  fn discard_bits_if_resized_keeps_bits_on_a_pure_move() {
    let current = Rect::from_xy(0, 0, 800, 600);
    let target = Rect::from_xy(400, 200, 800, 600);

    assert_eq!(
      discard_bits_if_resized(Some(&current), &target),
      SET_WINDOW_POS_FLAGS(0)
    );
  }

  #[test]
  fn discard_bits_if_resized_discards_bits_on_a_resize() {
    let current = Rect::from_xy(0, 0, 800, 600);

    for target in [
      Rect::from_xy(0, 0, 1600, 600),
      Rect::from_xy(0, 0, 800, 300),
      Rect::from_xy(100, 100, 400, 900),
    ] {
      assert_eq!(
        discard_bits_if_resized(Some(&current), &target),
        SWP_NOCOPYBITS
      );
    }
  }

  #[test]
  fn discard_bits_if_resized_discards_bits_when_size_is_unknown() {
    let target = Rect::from_xy(0, 0, 800, 600);

    assert_eq!(discard_bits_if_resized(None, &target), SWP_NOCOPYBITS);
  }
}
