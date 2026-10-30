//! The window being dragged or carried: a small picture of it that
//! follows the cursor, or waits beside the special workspace's dock, on
//! whichever screen it is over.

use super::Screen;
use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{draw_ghost, GHOST_SIZE},
  overview_layout::{cover_crop, RectF, Spring, Tile},
  overview_thumbnails::{Picture, Preview},
  OverviewStyle,
};

/// The window being dragged or carried.
pub(super) struct Ghost {
  pub(super) hwnd: isize,

  /// Where it is, in screen coordinates, so it carries on from one screen
  /// to the next.
  x: Spring,
  y: Spring,

  preview: Preview,
  picture: Picture,

  /// Registration of the screen's thumbnails it was registered after, on
  /// the same screen: it shows above them.
  host: Option<u64>,

  /// Title and size its picture is drawn with.
  drawn: Option<(String, (i32, i32))>,
}

impl Ghost {
  /// A ghost of window `hwnd`, starting out in the middle of `screen`, so
  /// a lifted window visibly drops to where it waits.
  pub(super) fn new(hwnd: isize, screen: &Screen) -> Option<Self> {
    let preview = Preview::new(
      hwnd,
      Tile {
        rect: RectF::default(),
        crop: RectF::default(),
      },
    );
    let (width, height) = size(screen);
    let (view_width, view_height) = screen.metrics.view;
    // Bouncing as it drops into place beside the dock.
    let bounce = screen.style().map_or(0.0, |style| style.bounce);
    let mut x = Spring::new(0.0, 280.0, 0.25).with_bounce(bounce);
    let mut y = Spring::new(0.0, 340.0, 0.25).with_bounce(bounce);
    x.snap(screen.area.x + (view_width - width) / 2.0);
    y.snap(screen.area.y + (view_height - height) / 2.0);

    Some(Self {
      hwnd,
      x,
      y,
      preview,
      picture: Picture::new()?,
      host: None,
      drawn: None,
    })
  }

  /// Shows it on `screen`, registered anew if it moved there, or the
  /// screen registered its own thumbnails over it.
  pub(super) fn host_on(&mut self, screen: &Screen) {
    if self.host == Some(screen.registration) {
      return;
    }

    let scale_factor = screen.metrics.scale_factor;
    let (width, height) = size(screen);
    let inset = scale_factor * 2.0 * 2.0;
    let frame = self.preview.frame;

    #[allow(clippy::cast_precision_loss)]
    let source = (
      (frame.right - frame.left) as f32,
      (frame.bottom - frame.top) as f32,
    );
    self.preview.tile.crop =
      cover_crop(source, (width - inset, height - inset));

    self.preview.register(screen.hwnd);
    self.picture.thumbnail =
      Thumbnail::register(screen.hwnd, self.picture.window);
    self.host = Some(screen.registration);
  }

  /// Centers it on the cursor at screen point `position`.
  pub(super) fn follow(&mut self, position: (f32, f32), screen: &Screen) {
    let (width, height) = size(screen);
    self.x.snap(position.0 - width / 2.0);
    self.y.snap(position.1 - height / 2.0);
  }

  /// Parks it while carried: beside the dock it can be dropped on, or
  /// else in the corner above the hints.
  pub(super) fn park(&mut self, screen: &Screen) {
    let px = |length: f32| length * screen.metrics.scale_factor;
    let (width, height) = size(screen);
    let (view_width, view_height) = screen.metrics.view;

    let (x, y) = match screen.dock {
      Some(dock) => {
        (dock.x - width - px(32.0), dock.y + (dock.h - height) / 2.0)
      }
      None => (
        view_width - width - px(40.0),
        screen.search_y.target - height - px(16.0),
      ),
    };
    self.x.target = screen.area.x + x;
    self.y.target = screen.area.y + y.min(view_height - height);
  }

  /// Draws its picture, titled `title`, for a screen at `scale_factor`,
  /// unless it shows that already.
  pub(super) fn draw(
    &mut self,
    title: &str,
    scale_factor: f32,
    style: &OverviewStyle,
  ) {
    #[allow(clippy::cast_possible_truncation)]
    let size = (
      (GHOST_SIZE.0 * scale_factor).ceil() as i32,
      (GHOST_SIZE.1 * scale_factor).ceil() as i32,
    );
    if self
      .drawn
      .as_ref()
      .is_some_and(|drawn| drawn.0 == title && drawn.1 == size)
    {
      return;
    }

    self.picture.draw(size, |surface| {
      draw_ghost(surface, title, scale_factor, style);
    });
    self.drawn = Some((title.to_string(), size));
  }

  /// Forgets what its picture shows, so it is drawn anew.
  pub(super) fn forget_drawn(&mut self) {
    self.drawn = None;
  }

  /// Advances it by `dt` seconds. Returns whether it still moves.
  pub(super) fn step(&mut self, dt: f32) -> bool {
    let is_moving_x = self.x.step(dt);
    self.y.step(dt) || is_moving_x
  }

  /// Places it where it is now on `screen`.
  pub(super) fn place(&mut self, screen: &Screen) {
    let scale_factor = screen.metrics.scale_factor;
    let (width, height) = size(screen);
    let rect = RectF::new(
      self.x.value - screen.area.x,
      self.y.value - screen.area.y,
      width,
      height,
    );
    let view = screen.view();

    self
      .preview
      .place(&rect.inset(scale_factor * 2.0), &view, 0.9);
    self.picture.place(&rect, &view, 1.0);
  }
}

/// Its size on `screen`.
fn size(screen: &Screen) -> (f32, f32) {
  let scale_factor = screen.metrics.scale_factor;
  (GHOST_SIZE.0 * scale_factor, GHOST_SIZE.1 * scale_factor)
}
