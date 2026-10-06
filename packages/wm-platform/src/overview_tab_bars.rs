//! Stacks' tab bars on the overview's cards.
//!
//! Each is a copy of the stack's real bar, drawn by the same painter at
//! the bar's real size and shown as a thumbnail under its stack's window.
//! The zoom on open and close therefore starts and ends exactly on the
//! real bar.

use windows::Win32::Foundation::HWND;

use crate::{
  overview_layout::{to_card, CardMetrics, RectF},
  overview_thumbnails::Picture,
  tab_bar_paint::{paint_tab_bar, TabBarView},
  tab_layout::{TabHit, TabLayout},
  CornerRadii, Rect, TabFrame,
};

/// A stack's tab bar on a card.
pub(crate) struct CardTabBar {
  pub frame: TabFrame,
  layout: TabLayout,

  /// Where the bar's window goes on its card, at scale 1.
  pub rect: RectF,

  pub picture: Picture,

  /// What `picture` shows, to skip redrawing it unchanged.
  drawn: Option<(TabFrame, TabHit, u64)>,
}

impl CardTabBar {
  /// The bar `frame` on the card of a workspace covering `area`, drawn
  /// into `picture`.
  pub fn new(
    frame: TabFrame,
    metrics: &CardMetrics,
    area: &RectF,
    picture: Picture,
  ) -> Self {
    Self {
      layout: TabLayout::for_frame(&frame),
      rect: to_card(metrics, area, &RectF::from_rect(&frame.outer_rect)),
      frame,
      picture,
      drawn: None,
    }
  }

  /// Takes on `frame`, keeping the picture drawn so far if it still shows
  /// it.
  pub fn update(
    &mut self,
    frame: TabFrame,
    metrics: &CardMetrics,
    area: &RectF,
  ) {
    if frame != self.frame {
      self.layout = TabLayout::for_frame(&frame);
    }
    self.rect =
      to_card(metrics, area, &RectF::from_rect(&frame.outer_rect));
    self.frame = frame;
  }

  /// The bar itself on the card, without its strip under the window.
  pub fn bar_rect(&self) -> RectF {
    self.to_card(&self.frame.rect)
  }

  /// Where the card is cut out for the bar, with the bar's corners.
  #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
  pub fn hole(&self) -> (RectF, CornerRadii) {
    let scale = self.scale();
    let radii = self.frame.style.strip_radii;
    let scaled = |radius: i32| (radius as f32 * scale).round() as i32;

    (
      self.rect,
      CornerRadii {
        top_left: scaled(radii.top_left),
        top_right: scaled(radii.top_right),
        bottom_right: scaled(radii.bottom_right),
        bottom_left: scaled(radii.bottom_left),
      },
    )
  }

  /// Index of the tab of window `hwnd`.
  pub fn tab_index(&self, hwnd: isize) -> Option<usize> {
    self.frame.tabs.iter().position(|tab| tab.hwnd == hwnd)
  }

  /// Draws the bar with `hover` highlighting a tab, unless it already
  /// shows that. `icons` counts the window icons arrived so far, so tabs
  /// pick up new ones; `notify` is posted when one arrives.
  pub fn draw(&mut self, hover: TabHit, icons: u64, notify: (HWND, u32)) {
    let key = (self.frame.clone(), hover, icons);
    if self.drawn.as_ref() == Some(&key) {
      return;
    }

    let order = (0..self.frame.tabs.len()).collect::<Vec<_>>();
    let close_visible = vec![false; order.len()];
    let view = TabBarView {
      layout: &self.layout,
      order: &order,
      hover,
      dragged: None,
      close_visible: &close_visible,
    };

    let outer = &self.frame.outer_rect;
    let frame = &self.frame;
    self
      .picture
      .draw((outer.width(), outer.height()), |surface| {
        paint_tab_bar(surface, frame, &view, notify);
      });
    self.drawn = Some(key);
  }

  /// The window of the tab at the card point (`x`, `y`), or of the shown
  /// tab on the bar's empty part. `None` off the bar.
  pub fn hit(&self, x: f32, y: f32) -> Option<isize> {
    if !self.rect.contains(x, y) {
      return None;
    }

    let scale = self.scale();

    #[allow(clippy::cast_possible_truncation)]
    let (bar_x, bar_y) = (
      ((x - self.rect.x) / scale).floor() as i32,
      ((y - self.rect.y) / scale).floor() as i32,
    );

    let index = match self.layout.hit_test(bar_x, bar_y, |_| false) {
      TabHit::Tab(index) | TabHit::Close(index) => index,
      TabHit::Empty => self.frame.active_index,
    };
    self.frame.tabs.get(index).map(|tab| tab.hwnd)
  }

  /// Card units per pixel of the real bar.
  #[allow(clippy::cast_precision_loss)]
  fn scale(&self) -> f32 {
    self.rect.w / self.frame.outer_rect.width().max(1) as f32
  }

  /// `rect` (screen coordinates) on the card, at scale 1.
  #[allow(clippy::cast_precision_loss)]
  fn to_card(&self, rect: &Rect) -> RectF {
    let outer = &self.frame.outer_rect;
    let scale = self.scale();

    RectF::new(
      self.rect.x + (rect.left - outer.left) as f32 * scale,
      self.rect.y + (rect.top - outer.top) as f32 * scale,
      rect.width() as f32 * scale,
      rect.height() as f32 * scale,
    )
  }
}
