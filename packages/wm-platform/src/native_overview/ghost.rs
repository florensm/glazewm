//! The window being dragged or carried: a small picture of it that
//! follows the cursor, or waits beside the special workspace's dock, on
//! whichever screen it is over. Once let go of, it flies into the card it
//! lands on and fades. It stretches with its speed, and can leave fading
//! copies of its outline behind.

use std::collections::VecDeque;

use windows::Win32::Foundation::RECT;

use super::Screen;
use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{draw_ghost, GHOST_SIZE},
  overview_effects::stretch,
  overview_layout::{cover_crop, RectF, Spring, Tile},
  overview_thumbnails::{to_opacity, to_win32, Picture, Preview},
  OverviewEffects, OverviewStyle,
};

/// How long it takes to catch up with the cursor: short enough to keep
/// up, long enough to trail it a little.
const FOLLOW_MS: f32 = 110.0;

/// How long it takes to settle beside the dock while carried.
const PARK_MS: (f32, f32) = (280.0, 340.0);

/// How long it takes to fly into the card it lands on, and to fade
/// meanwhile.
const LAND_MS: f32 = 240.0;
const LAND_FADE_MS: f32 = 300.0;

/// How small it starts out as it is lifted, and ends up landing, relative
/// to its full size.
const LIFT_FROM: f32 = 0.6;

/// Frames between one copy of its trail and the next, and how strongly
/// the nearest shows.
const TRAIL_SPACING: usize = 2;
const TRAIL_OPACITY: f32 = 0.45;

/// The window being dragged or carried.
pub(super) struct Ghost {
  pub(super) hwnd: isize,

  /// Where its center is, in screen coordinates, so it carries on from
  /// one screen to the next.
  x: Spring,
  y: Spring,

  /// Its size relative to `GHOST_SIZE`, and how strongly it shows.
  scale: Spring,
  alpha: Spring,

  /// Whether it was let go of and is flying into its card.
  is_landing: bool,

  /// How much it stretches with its speed, from 0 to 1.
  squash: f32,

  /// Copies of its outline trailing it, bottom to top, and where it was
  /// over the last frames, in screen coordinates, latest first.
  trail: Vec<Thumbnail>,
  trail_length: usize,
  history: VecDeque<RectF>,

  preview: Preview,
  picture: Picture,

  /// Registration of the screen's thumbnails it was registered after, on
  /// the same screen: it shows above them.
  host: Option<u64>,

  /// Title and size its picture is drawn with.
  drawn: Option<(String, (i32, i32))>,
}

impl Ghost {
  /// A ghost of window `hwnd`, growing in at screen point `at` (where it
  /// is grabbed with the mouse), or else in the middle of `screen`, so a
  /// lifted window visibly drops to where it waits.
  pub(super) fn new(
    hwnd: isize,
    at: Option<(f32, f32)>,
    screen: &Screen,
  ) -> Option<Self> {
    let preview = Preview::new(
      hwnd,
      Tile {
        rect: RectF::default(),
        crop: RectF::default(),
      },
    );
    let (view_width, view_height) = screen.metrics.view;
    let (x, y) = at.unwrap_or((
      screen.area.x + view_width / 2.0,
      screen.area.y + view_height / 2.0,
    ));
    // Popping up and bouncing as it is lifted and dropped into place.
    let bounce = screen.style().map_or(0.0, |style| style.bounce);
    let effects = screen
      .style()
      .map_or(OverviewEffects::NONE, |style| style.effects);
    let trail_length = effects.trail.min(OverviewEffects::MAX_TRAIL);
    let mut scale = Spring::new(LIFT_FROM, PARK_MS.0, 0.001)
      .with_bounce(bounce.max(0.3));
    scale.target = 1.0;

    Some(Self {
      hwnd,
      x: Spring::new(x, PARK_MS.0, 0.25).with_bounce(bounce),
      y: Spring::new(y, PARK_MS.1, 0.25).with_bounce(bounce),
      scale,
      alpha: Spring::new(1.0, LAND_FADE_MS, 0.002),
      is_landing: false,
      squash: effects.squash,
      trail: Vec::new(),
      trail_length: usize::try_from(trail_length).unwrap_or_default(),
      history: VecDeque::new(),
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

    // Under the ghost itself.
    self.trail = (0..self.trail_length)
      .filter_map(|_| {
        Thumbnail::register(screen.hwnd, self.picture.window)
      })
      .collect();
    self.preview.register(screen.hwnd);
    self.picture.thumbnail =
      Thumbnail::register(screen.hwnd, self.picture.window);
    self.host = Some(screen.registration);
  }

  /// Has it trail the cursor at screen point `position`.
  pub(super) fn follow(&mut self, position: (f32, f32)) {
    self.x.retune(FOLLOW_MS);
    self.y.retune(FOLLOW_MS);
    self.x.target = position.0;
    self.y.target = position.1;
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
    self.x.retune(PARK_MS.0);
    self.y.retune(PARK_MS.1);
    self.x.target = screen.area.x + x + width / 2.0;
    self.y.target =
      screen.area.y + y.min(view_height - height) + height / 2.0;
  }

  /// Flies it into `card`, in screen coordinates, shrinking and fading
  /// as it goes, once let go of.
  pub(super) fn land(&mut self, card: &RectF, screen: &Screen) {
    let (width, height) = size(screen);
    let (cx, cy) = card.center();
    let fit = (card.w / width).min(card.h / height);

    self.x.retune(LAND_MS);
    self.y.retune(LAND_MS);
    self.scale.retune(LAND_MS);
    self.x.target = cx;
    self.y.target = cy;
    self.scale.target = (fit * LIFT_FROM).min(LIFT_FROM);
    self.alpha.target = 0.0;
    self.is_landing = true;
  }

  /// Whether it has landed and faded out, and can go.
  pub(super) fn has_landed(&self) -> bool {
    self.is_landing && self.alpha.is_settled()
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
    [&mut self.x, &mut self.y, &mut self.scale, &mut self.alpha]
      .into_iter()
      .fold(false, |is_moving, spring| spring.step(dt) | is_moving)
  }

  /// Places it where it is now on `screen`.
  pub(super) fn place(&mut self, screen: &Screen) {
    let scale_factor = screen.metrics.scale_factor;
    let (width, height) = size(screen);
    let scale = self.scale.value.max(0.0);
    let (across, down) =
      stretch((self.x.velocity(), self.y.velocity()), self.squash);
    let (width, height) = (width * scale * across, height * scale * down);
    let rect = RectF::new(
      self.x.value - screen.area.x - width / 2.0,
      self.y.value - screen.area.y - height / 2.0,
      width,
      height,
    );
    let view = screen.view();
    let alpha = self.alpha.value.clamp(0.0, 1.0);

    self
      .place_trail(&rect.translate(screen.area.x, screen.area.y), screen);

    self.preview.place(
      &rect.inset(scale_factor * 2.0 * scale),
      &view,
      0.9 * alpha,
    );
    self.picture.place(&rect, &view, alpha);
  }
}

impl Ghost {
  /// Places the copies of its outline where it was over the last frames,
  /// fainter the further back, now that it is at `rect` (in screen
  /// coordinates).
  #[allow(clippy::cast_precision_loss)]
  fn place_trail(&mut self, rect: &RectF, screen: &Screen) {
    if self.trail.is_empty() {
      return;
    }

    self.history.push_front(*rect);
    self.history.truncate(self.trail.len() * TRAIL_SPACING + 1);

    let (width, height) = self.picture.size();
    let source = RECT {
      right: width,
      bottom: height,
      ..Default::default()
    };
    let clip = to_win32(&screen.view());
    let count = self.trail.len();
    let alpha = self.alpha.value.clamp(0.0, 1.0);

    // Bottom to top: the furthest back first.
    for (index, thumbnail) in self.trail.iter_mut().enumerate() {
      let back = count - index;
      let shown = self.history.get(back * TRAIL_SPACING).filter(|past| {
        // Still, it would only thicken the outline it sits under.
        (past.x - rect.x).abs() + (past.y - rect.y).abs() > 1.0
      });

      match shown {
        Some(past) => {
          let fade = 1.0 - back as f32 / (count + 1) as f32;
          thumbnail.place(
            source,
            to_win32(&past.translate(-screen.area.x, -screen.area.y)),
            &clip,
            to_opacity(TRAIL_OPACITY * fade * alpha),
          );
        }
        None => thumbnail.hide(),
      }
    }
  }
}

/// Its size on `screen`.
fn size(screen: &Screen) -> (f32, f32) {
  let scale_factor = screen.metrics.scale_factor;
  (GHOST_SIZE.0 * scale_factor, GHOST_SIZE.1 * scale_factor)
}
