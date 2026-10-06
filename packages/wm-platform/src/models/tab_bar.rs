//! What a stack's tab bar shows and how it looks, as data: drawn by the
//! bar's own window and, for the overview, into its cards.

use crate::{Color, Rect};

/// Radius of each corner of a rounded rect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CornerRadii {
  pub top_left: i32,
  pub top_right: i32,
  pub bottom_right: i32,
  pub bottom_left: i32,
}

impl CornerRadii {
  /// The same `radius` on every corner.
  #[must_use]
  pub fn uniform(radius: i32) -> Self {
    Self {
      top_left: radius,
      top_right: radius,
      bottom_right: radius,
      bottom_left: radius,
    }
  }

  /// Each radius limited to half of the shorter side of a `width` x
  /// `height` rect.
  #[must_use]
  pub fn clamped(self, width: i32, height: i32) -> Self {
    let max = width.min(height) / 2;
    let clamp = |radius: i32| radius.clamp(0, max.max(0));

    Self {
      top_left: clamp(self.top_left),
      top_right: clamp(self.top_right),
      bottom_right: clamp(self.bottom_right),
      bottom_left: clamp(self.bottom_left),
    }
  }
}

/// A tab of the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabInfo {
  /// Title shown on the tab.
  pub title: String,

  /// Handle of the tab's window, for its icon.
  pub hwnd: isize,

  /// Whether the window requests attention, which highlights its tab.
  pub is_urgent: bool,
}

/// When tabs show a close button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabCloseMode {
  /// On the active and the hovered tab.
  Hover,
  Always,
  Never,
}

/// Look of a tab bar, in physical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct TabBarStyle {
  pub background: Color,
  pub active_background: Color,
  pub hover_background: Color,
  pub inactive_background: Color,
  pub urgent_background: Color,
  pub text: Color,
  pub inactive_text: Color,
  pub font_family: String,
  pub font_size: i32,
  /// Corner radius of the tab highlights.
  pub corner_radius: i32,
  /// Corners of the strip, e.g. square where it meets its window.
  pub strip_radii: CornerRadii,
  pub min_tab_width: i32,
  /// 0 lets tabs share the whole bar.
  pub max_tab_width: i32,
  pub show_icons: bool,
  pub show_numbers: bool,
  pub close_button: TabCloseMode,
  /// Scale factor of the bar's monitor, for the size thresholds.
  pub scale_factor: f32,
}

/// Everything shown by a tab bar, posted to its thread as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct TabFrame {
  /// Where the tabs are shown.
  pub rect: Rect,
  /// The bar's window: `rect`, plus strip reaching under the stack's
  /// window to fill in its rounded corners.
  pub outer_rect: Rect,
  pub tabs: Vec<TabInfo>,
  pub active_index: usize,
  /// The stack's active window. The bar is kept directly behind it in
  /// z-order, so it is covered by whatever covers the window.
  pub anchor: isize,
  pub style: TabBarStyle,
}
