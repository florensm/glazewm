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
  /// Area the bar is cut off at, like its window, e.g. the monitor of a
  /// scrolling layout's column.
  pub clip: Option<Rect>,
  pub tabs: Vec<TabInfo>,
  pub active_index: usize,
  /// The window drawing the stack's active tab. The bar is kept directly
  /// behind it in z-order, so it is covered by whatever covers it.
  pub anchor: isize,
  pub style: TabBarStyle,
}

impl TabFrame {
  /// The part of `outer_rect` inside `clip`, or `None` when no part is.
  #[must_use]
  pub fn visible_rect(&self) -> Option<Rect> {
    match &self.clip {
      Some(clip) => self.outer_rect.intersection(clip),
      None => Some(self.outer_rect.clone()),
    }
  }

  /// Whether `other` draws the same picture, only elsewhere on screen or
  /// cut off differently.
  #[must_use]
  pub fn draws_like(&self, other: &TabFrame) -> bool {
    // `rect` within `outer_rect`, which the picture is the size of.
    let local = |frame: &TabFrame| {
      Rect::from_xy(
        frame.rect.x() - frame.outer_rect.x(),
        frame.rect.y() - frame.outer_rect.y(),
        frame.rect.width(),
        frame.rect.height(),
      )
    };

    local(self) == local(other)
      && self.outer_rect.width() == other.outer_rect.width()
      && self.outer_rect.height() == other.outer_rect.height()
      && self.tabs == other.tabs
      && self.active_index == other.active_index
      && self.style == other.style
  }
}

#[cfg(test)]
mod tests {
  use super::{CornerRadii, TabBarStyle, TabCloseMode, TabFrame};
  use crate::{Color, Rect};

  fn frame(clip: Option<Rect>) -> TabFrame {
    let color = Color {
      r: 0,
      g: 0,
      b: 0,
      a: u8::MAX,
    };

    TabFrame {
      rect: Rect::from_ltrb(0, 0, 300, 28),
      outer_rect: Rect::from_ltrb(0, 0, 300, 36),
      clip,
      tabs: Vec::new(),
      active_index: 0,
      anchor: 0,
      style: TabBarStyle {
        background: color,
        active_background: color,
        hover_background: color,
        inactive_background: color,
        urgent_background: color,
        text: color,
        inactive_text: color,
        font_family: String::new(),
        font_size: 12,
        corner_radius: 0,
        strip_radii: CornerRadii::default(),
        min_tab_width: 0,
        max_tab_width: 0,
        show_icons: false,
        show_numbers: false,
        close_button: TabCloseMode::Never,
        scale_factor: 1.0,
      },
    }
  }

  #[test]
  fn moved_or_cut_off_bar_draws_alike() {
    let bar = frame(None);

    let moved = TabFrame {
      rect: Rect::from_ltrb(-500, 40, -200, 68),
      outer_rect: Rect::from_ltrb(-500, 40, -200, 76),
      clip: Some(Rect::from_ltrb(-300, 0, 0, 500)),
      anchor: 1,
      ..frame(None)
    };
    assert!(bar.draws_like(&moved));

    let resized = TabFrame {
      rect: Rect::from_ltrb(0, 0, 200, 28),
      outer_rect: Rect::from_ltrb(0, 0, 200, 36),
      ..frame(None)
    };
    assert!(!bar.draws_like(&resized));

    let switched = TabFrame {
      active_index: 1,
      ..frame(None)
    };
    assert!(!bar.draws_like(&switched));
  }

  #[test]
  fn bar_is_cut_off_at_its_clip() {
    assert_eq!(
      frame(None).visible_rect(),
      Some(Rect::from_ltrb(0, 0, 300, 36))
    );
    assert_eq!(
      frame(Some(Rect::from_ltrb(200, -50, 1000, 500))).visible_rect(),
      Some(Rect::from_ltrb(200, 0, 300, 36))
    );
    assert_eq!(
      frame(Some(Rect::from_ltrb(300, 0, 1000, 500))).visible_rect(),
      None
    );
  }
}
