//! Geometry of a stack's tab bar: where each tab, icon, title and close
//! button goes, and what a point in the bar hits.
//!
//! All values are physical pixels relative to the bar window's top-left
//! corner.

/// What the user asked for through the tab bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabAction {
  /// Show the tab at this index.
  Activate(usize),
  /// Close the tab's window.
  Close(usize),
  /// Move a tab to another position.
  Move { from: usize, to: usize },
  /// Take the tab's window out of the stack.
  Detach(usize),
  /// Take the tab's window out of the stack as a floating window.
  Float(usize),
  /// Show the next (or previous) tab.
  Cycle { prev: bool },
}

/// A rectangle in tab bar coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TabRect {
  pub left: i32,
  pub top: i32,
  pub right: i32,
  pub bottom: i32,
}

impl TabRect {
  #[must_use]
  pub fn width(&self) -> i32 {
    self.right - self.left
  }

  #[must_use]
  pub fn height(&self) -> i32 {
    self.bottom - self.top
  }

  /// This rect moved down by `dy`.
  #[must_use]
  pub fn offset_y(self, dy: i32) -> Self {
    Self {
      top: self.top + dy,
      bottom: self.bottom + dy,
      ..self
    }
  }

  #[must_use]
  pub fn contains(&self, x: i32, y: i32) -> bool {
    x >= self.left && x < self.right && y >= self.top && y < self.bottom
  }
}

impl From<TabRect> for crate::Rect {
  fn from(rect: TabRect) -> Self {
    Self::from_ltrb(rect.left, rect.top, rect.right, rect.bottom)
  }
}

/// Inputs of a tab bar layout.
#[derive(Clone, Copy, Debug)]
pub struct TabLayoutParams {
  /// Where the tabs start in the bar's window, which can reach past them.
  pub top: i32,
  pub width: i32,
  pub height: i32,
  pub tab_count: usize,
  pub active_index: usize,

  /// Narrowest a tab gets before the bar scrolls.
  pub min_tab_width: i32,

  /// Widest a tab gets; 0 lets tabs share the whole bar.
  pub max_tab_width: i32,

  /// Below this width, tabs only show their icon.
  pub icon_only_width: i32,

  /// Below this width, tabs never show a close button.
  pub close_min_width: i32,

  /// Whether tabs have an icon.
  pub show_icons: bool,
}

/// Geometry of one tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabSlot {
  /// The tab's full cell, used for hit-testing.
  pub cell: TabRect,

  /// The tab's highlight, inset from its cell.
  pub pill: TabRect,

  /// Icon square, if the tab has an icon.
  pub icon: Option<TabRect>,

  /// Title area; empty when the tab is icon-only.
  pub text: TabRect,

  /// Close button square, when the tab is wide enough for one.
  pub close: Option<TabRect>,
}

/// What a point in the tab bar hits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabHit {
  Tab(usize),
  Close(usize),
  Empty,
}

/// Laid-out tab bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabLayout {
  pub slots: Vec<TabSlot>,

  /// Width of every tab cell.
  pub tab_width: i32,

  /// How far the tabs are scrolled left, so the active tab stays in view
  /// when they don't all fit.
  pub scroll: i32,
}

impl TabLayout {
  /// Lays out the tabs of the bar `frame` describes.
  #[must_use]
  pub fn for_frame(frame: &crate::TabFrame) -> Self {
    let scale = |px: f32| {
      #[allow(clippy::cast_possible_truncation)]
      let scaled = (px * frame.style.scale_factor).round() as i32;
      scaled
    };

    Self::new(&TabLayoutParams {
      top: frame.rect.top - frame.outer_rect.top,
      width: frame.rect.width(),
      height: frame.rect.height(),
      tab_count: frame.tabs.len(),
      active_index: frame.active_index,
      min_tab_width: frame.style.min_tab_width,
      max_tab_width: frame.style.max_tab_width,
      icon_only_width: scale(60.0),
      close_min_width: scale(80.0),
      show_icons: frame.style.show_icons,
    })
  }

  /// Lays out the tabs of a bar.
  #[must_use]
  pub fn new(params: &TabLayoutParams) -> Self {
    let Ok(count) = i32::try_from(params.tab_count) else {
      return Self::empty();
    };

    if count == 0 || params.width <= 0 || params.height <= 0 {
      return Self::empty();
    }

    let fair_width = params.width / count;
    let tab_width = if params.max_tab_width > 0 {
      fair_width.min(params.max_tab_width)
    } else {
      fair_width
    }
    .max(params.min_tab_width.max(1));

    let content_width = tab_width * count;
    let max_scroll = (content_width - params.width).max(0);
    let active = i32::try_from(params.active_index).unwrap_or(0);

    // Scroll just far enough that the active tab is fully visible.
    let active_right = (active + 1) * tab_width;
    let scroll = (active_right - params.width).clamp(0, max_scroll);

    let slots = (0..count)
      .map(|index| {
        Self::slot(params, index * tab_width - scroll, tab_width)
      })
      .collect();

    Self {
      slots,
      tab_width,
      scroll,
    }
  }

  fn empty() -> Self {
    Self {
      slots: Vec::new(),
      tab_width: 0,
      scroll: 0,
    }
  }

  /// Geometry of the tab whose cell starts at `left`.
  ///
  /// Proportions follow the bar height, so they scale with DPI: tabs are
  /// pills inset by about a sixth of the height, with an icon of ~57% of
  /// the height.
  fn slot(params: &TabLayoutParams, left: i32, width: i32) -> TabSlot {
    let height = params.height;
    let h_inset = (height / 8).max(2);
    let v_inset = (height / 8).max(2);
    let pad = (height / 5).max(4);

    let cell = TabRect {
      left,
      top: 0,
      right: left + width,
      bottom: height,
    };

    let pill = TabRect {
      left: left + h_inset,
      top: v_inset,
      right: (left + width - h_inset).max(left + h_inset + 1),
      bottom: (height - v_inset).max(v_inset + 1),
    };

    #[allow(clippy::cast_possible_truncation)]
    let icon_size = ((f64::from(height) * 0.57).round() as i32).max(10);
    let icon_top = (height - icon_size) / 2;

    let is_icon_only = width < params.icon_only_width;

    let icon = params.show_icons.then(|| {
      let icon_left = if is_icon_only {
        left + (width - icon_size) / 2
      } else {
        pill.left + pad
      };

      TabRect {
        left: icon_left,
        top: icon_top,
        right: icon_left + icon_size,
        bottom: icon_top + icon_size,
      }
    });

    let close_size = (height / 2).max(8);
    let close =
      (width >= params.close_min_width && !is_icon_only).then(|| {
        let close_top = (height - close_size) / 2;

        TabRect {
          left: pill.right - pad / 2 - close_size,
          top: close_top,
          right: pill.right - pad / 2,
          bottom: close_top + close_size,
        }
      });

    let text_left =
      icon.map_or(pill.left + pad, |icon| icon.right + pad / 2);
    let text_right =
      close.map_or(pill.right - pad, |close| close.left - 2);

    let text = if is_icon_only {
      TabRect::default()
    } else {
      TabRect {
        left: text_left,
        top: 0,
        right: text_right.max(text_left),
        bottom: height,
      }
    };

    let top = params.top;

    TabSlot {
      cell: cell.offset_y(top),
      pill: pill.offset_y(top),
      icon: icon.map(|icon| icon.offset_y(top)),
      text: if is_icon_only {
        text
      } else {
        text.offset_y(top)
      },
      close: close.map(|close| close.offset_y(top)),
    }
  }

  /// What the point (`x`, `y`) hits. `close_visible` says which tabs
  /// currently show their close button.
  #[must_use]
  pub fn hit_test(
    &self,
    x: i32,
    y: i32,
    close_visible: impl Fn(usize) -> bool,
  ) -> TabHit {
    let Some(index) =
      self.slots.iter().position(|slot| slot.cell.contains(x, y))
    else {
      return TabHit::Empty;
    };

    let on_close = self.slots[index]
      .close
      .is_some_and(|close| close.contains(x, y));

    if on_close && close_visible(index) {
      TabHit::Close(index)
    } else {
      TabHit::Tab(index)
    }
  }

  /// Index a dragged tab would move to when released at `x`.
  #[must_use]
  pub fn drop_index(&self, x: i32) -> usize {
    if self.slots.is_empty() || self.tab_width <= 0 {
      return 0;
    }

    let index = (x + self.scroll).div_euclid(self.tab_width);
    usize::try_from(index)
      .unwrap_or(0)
      .min(self.slots.len() - 1)
  }
}

#[cfg(test)]
mod tests {
  use super::{TabHit, TabLayout, TabLayoutParams};

  fn params(width: i32, tab_count: usize) -> TabLayoutParams {
    TabLayoutParams {
      top: 0,
      width,
      height: 28,
      tab_count,
      active_index: 0,
      min_tab_width: 48,
      max_tab_width: 0,
      icon_only_width: 60,
      close_min_width: 80,
      show_icons: true,
    }
  }

  #[test]
  fn tabs_share_the_bar_evenly() {
    let layout = TabLayout::new(&params(900, 3));

    assert_eq!(layout.tab_width, 300);
    assert_eq!(layout.scroll, 0);
    assert_eq!(layout.slots[2].cell.left, 600);
    assert_eq!(layout.slots[2].cell.right, 900);
  }

  #[test]
  fn max_width_caps_tabs() {
    let layout = TabLayout::new(&TabLayoutParams {
      max_tab_width: 200,
      ..params(900, 2)
    });

    assert_eq!(layout.tab_width, 200);
    assert_eq!(layout.slots[1].cell.right, 400);
  }

  #[test]
  fn overflowing_tabs_scroll_to_the_active_one() {
    let layout = TabLayout::new(&TabLayoutParams {
      active_index: 9,
      ..params(200, 10)
    });

    assert_eq!(layout.tab_width, 48);
    assert_eq!(layout.scroll, 10 * 48 - 200);
    assert_eq!(layout.slots[9].cell.right, 200);
  }

  #[test]
  fn narrow_tabs_are_icon_only_without_close_button() {
    let layout = TabLayout::new(&params(100, 2));
    let slot = layout.slots[0];

    assert!(slot.close.is_none());
    assert_eq!(slot.text.width(), 0);
    assert!(slot.icon.is_some());
  }

  #[test]
  fn title_sits_between_icon_and_close_button() {
    let layout = TabLayout::new(&params(600, 2));
    let slot = layout.slots[0];
    let (icon, close) = (slot.icon.unwrap(), slot.close.unwrap());

    assert!(slot.text.left >= icon.right);
    assert!(slot.text.right <= close.left);
    assert!(close.right <= slot.pill.right);
  }

  #[test]
  fn hit_test_finds_tabs_and_close_buttons() {
    let layout = TabLayout::new(&params(600, 2));
    let close = layout.slots[1].close.unwrap();

    assert_eq!(layout.hit_test(10, 14, |_| true), TabHit::Tab(0));
    assert_eq!(
      layout.hit_test(close.left + 1, close.top + 1, |_| true),
      TabHit::Close(1)
    );
    assert_eq!(
      layout.hit_test(close.left + 1, close.top + 1, |_| false),
      TabHit::Tab(1)
    );
    assert_eq!(layout.hit_test(700, 14, |_| true), TabHit::Empty);
  }

  #[test]
  fn drop_index_is_clamped() {
    let layout = TabLayout::new(&params(600, 3));

    assert_eq!(layout.drop_index(-50), 0);
    assert_eq!(layout.drop_index(250), 1);
    assert_eq!(layout.drop_index(5000), 2);
  }

  #[test]
  fn offset_layout_moves_tabs_and_hits() {
    let flat = TabLayout::new(&params(900, 3));
    let offset = TabLayout::new(&TabLayoutParams {
      top: 10,
      ..params(900, 3)
    });

    assert_eq!(offset.slots[1].pill, flat.slots[1].pill.offset_y(10));
    assert_eq!(offset.hit_test(450, 5, |_| false), TabHit::Empty);
    assert_eq!(offset.hit_test(450, 15, |_| false), TabHit::Tab(1));
  }
}
