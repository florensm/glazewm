//! The DWM thumbnails the window overview is made of: live previews of
//! windows, and pictures drawn into cloaked windows of its own.

use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND, POINT, RECT},
    Graphics::Dwm::{
      DwmRegisterThumbnail, DwmSetWindowAttribute, DwmUnregisterThumbnail,
      DwmUpdateThumbnailProperties, DWMWA_CLOAK, DWM_THUMBNAIL_PROPERTIES,
      DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
      DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, ShowWindow, SW_SHOWNOACTIVATE,
      WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  companion::Companion,
  overview_layout::{RectF, Tile},
  surface::Surface,
  window_class, OverviewOverlay, OverviewOverlayKind, OverviewWindow,
};

/// Shown for a window whose frame can't be read, e.g. one being
/// destroyed.
pub(crate) const FALLBACK_FRAME: RECT = RECT {
  left: 0,
  top: 0,
  right: 1600,
  bottom: 1000,
};

/// Creates a cloaked layered window to draw a picture into: DWM keeps
/// composing it, so the overview can show it as a thumbnail.
fn create_picture_window() -> Option<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_OverviewPicture"),
    window_class::default_wnd_proc,
  );

  // SAFETY: The class is registered above, and the attribute matches its
  // documented `BOOL` size.
  unsafe {
    let hwnd = CreateWindowExW(
      WS_EX_LAYERED
        | WS_EX_TRANSPARENT
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE,
      w!("GlazeWM_OverviewPicture"),
      w!(""),
      WS_POPUP,
      0,
      0,
      1,
      1,
      None,
      None,
      None,
      None,
    );

    if hwnd.0 == 0 {
      return None;
    }

    let cloak = BOOL::from(true);
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_CLOAK,
      std::ptr::from_ref(&cloak).cast(),
      u32::try_from(std::mem::size_of::<BOOL>()).unwrap_or(4),
    );
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    Some(hwnd)
  }
}

/// A DWM thumbnail of some window on the overview.
pub(crate) struct Thumbnail {
  handle: isize,

  /// Last `(source, destination, opacity)` applied, to skip unchanged
  /// updates on every frame.
  placed: Option<(RECT, RECT, u8)>,
}

impl Thumbnail {
  /// Registers a thumbnail of `source` on `dest`, hidden until placed.
  /// Thumbnails stack in the order they are registered.
  pub fn register(dest: HWND, source: HWND) -> Option<Self> {
    // SAFETY: Both are top-level windows; registration fails otherwise.
    let handle = unsafe { DwmRegisterThumbnail(dest, source) }.ok()?;

    let mut thumbnail = Self {
      handle,
      placed: Some((RECT::default(), RECT::default(), 0)),
    };
    thumbnail.hide();
    Some(thumbnail)
  }

  /// Shows `source` (in the source window's coordinates) at `dest`, cut
  /// to the part of `dest` inside `clip`.
  pub fn place(
    &mut self,
    source: RECT,
    dest: RECT,
    clip: &RECT,
    opacity: u8,
  ) {
    let Some((source, dest)) = clip_rects(source, dest, clip) else {
      self.hide();
      return;
    };

    if self.placed == Some((source, dest, opacity)) {
      return;
    }
    self.placed = Some((source, dest, opacity));

    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTDESTINATION
        | DWM_TNP_RECTSOURCE
        | DWM_TNP_OPACITY
        | DWM_TNP_VISIBLE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcDestination: dest,
      rcSource: source,
      opacity,
      fVisible: BOOL::from(opacity > 0 && dest.right > dest.left),
      fSourceClientAreaOnly: false.into(),
    };

    // SAFETY: `self.handle` is a registered thumbnail and `props` outlives
    // the call.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.handle, &raw const props);
    }
  }

  pub fn hide(&mut self) {
    if self.placed.take().is_none() {
      return;
    }

    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_VISIBLE,
      fVisible: false.into(),
      ..Default::default()
    };

    // SAFETY: As in `place`.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.handle, &raw const props);
    }
  }
}

impl Drop for Thumbnail {
  fn drop(&mut self) {
    // SAFETY: Registered in `register` and unregistered once.
    unsafe {
      let _ = DwmUnregisterThumbnail(self.handle);
    }
  }
}

/// A picture drawn into a cloaked window, shown as a thumbnail.
pub(crate) struct Picture {
  pub window: HWND,
  surface: Option<Surface>,
  pub thumbnail: Option<Thumbnail>,
}

impl Picture {
  pub fn new() -> Option<Self> {
    Some(Self {
      window: create_picture_window()?,
      surface: None,
      thumbnail: None,
    })
  }

  /// Size of what was drawn last.
  pub fn size(&self) -> (i32, i32) {
    self
      .surface
      .as_ref()
      .map_or((0, 0), |surface| (surface.width, surface.height))
  }

  /// Redraws the picture at `size` with `draw`.
  pub fn draw(
    &mut self,
    size: (i32, i32),
    draw: impl FnOnce(&mut Surface),
  ) {
    if self.size() != size {
      self.surface = Surface::new(size.0, size.1);
    }

    let Some(surface) = &mut self.surface else {
      return;
    };
    draw(surface);

    if let Err(err) = surface.show_on(self.window, POINT::default()) {
      tracing::warn!("Failed to draw an overview picture: {err}");
    }
  }

  /// Shows the whole picture at `dest` on the overview, cut to `clip`.
  pub fn place(&mut self, dest: &RectF, clip: &RectF, opacity: f32) {
    let (width, height) = self.size();
    if let Some(thumbnail) = &mut self.thumbnail {
      thumbnail.place(
        RECT {
          right: width,
          bottom: height,
          ..Default::default()
        },
        to_win32(dest),
        &to_win32(clip),
        to_opacity(opacity),
      );
    }
  }
}

impl Drop for Picture {
  fn drop(&mut self) {
    self.thumbnail = None;

    // SAFETY: Created in `new` on this thread and destroyed once.
    unsafe {
      let _ = DestroyWindow(self.window);
    }
  }
}

/// A window's live preview on a card.
pub(crate) struct Preview {
  pub hwnd: isize,
  pub tile: Tile,

  /// Visible frame of the window, in its own coordinates.
  pub frame: RECT,

  /// What the thumbnail samples instead of the window, if anything.
  companion: Option<Companion>,

  pub thumbnail: Option<Thumbnail>,
  pub opacity: f32,

  /// Opacity of the window itself, from its `transparency` effect, which
  /// its overlays don't share.
  pub alpha: f32,

  /// Its window's backdrop and border, bottom to top, under it.
  overlays: Vec<OverlayPreview>,

  /// Of a column scrolled out of view: there is no on-screen window for
  /// the zoom to start or end on, so it fades with the zoom instead.
  pub is_scrolled_off: bool,

  /// Where its window is on screen, which its tile shows.
  pub screen_rect: RectF,
}

/// One of a window's overlays, shown around its preview as it sits around
/// the window on screen.
struct OverlayPreview {
  overlay: OverviewOverlay,
  thumbnail: Option<Thumbnail>,
}

impl Preview {
  pub fn new(hwnd: isize, tile: Tile) -> Self {
    Self {
      hwnd,
      tile,
      frame: visible_frame(HWND(hwnd)).unwrap_or(FALLBACK_FRAME),
      companion: Companion::find(HWND(hwnd)),
      thumbnail: None,
      opacity: 1.0,
      alpha: 1.0,
      overlays: Vec::new(),
      is_scrolled_off: false,
      screen_rect: RectF::default(),
    }
  }

  /// Takes on how `window` looks: its opacity and overlays. Returns
  /// whether its overlays changed, which calls for registering the
  /// thumbnails anew.
  pub fn restyle(&mut self, window: &OverviewWindow) -> bool {
    self.alpha = f32::from(window.opacity) / f32::from(u8::MAX);

    let is_changed = self
      .overlays
      .iter()
      .map(|overlay| &overlay.overlay)
      .ne(&window.overlays);

    if is_changed {
      self.overlays = window
        .overlays
        .iter()
        .map(|overlay| OverlayPreview {
          overlay: *overlay,
          thumbnail: None,
        })
        .collect();
    }
    is_changed
  }

  /// Whether its window has a border of its own.
  pub fn has_border(&self) -> bool {
    self
      .overlays
      .iter()
      .any(|overlay| overlay.overlay.kind == OverviewOverlayKind::Border)
  }

  /// Measures its window's frame again, e.g. after it was resized.
  pub fn refresh_frame(&mut self) {
    self.frame = visible_frame(HWND(self.hwnd)).unwrap_or(self.frame);
  }

  /// Its opacity on a card zoomed `openness` open.
  pub fn zoomed_opacity(&self, openness: f32) -> f32 {
    if self.is_scrolled_off {
      self.opacity * openness
    } else {
      self.opacity
    }
  }

  /// Registers its overlays, then itself, on `host`, above everything
  /// registered there so far.
  pub fn register(&mut self, host: HWND) {
    for overlay in &mut self.overlays {
      overlay.thumbnail =
        Thumbnail::register(host, HWND(overlay.overlay.hwnd));
    }

    let source = self.companion.map_or(HWND(self.hwnd), Companion::hwnd);
    self.thumbnail = Thumbnail::register(host, source);
  }

  /// Hands its thumbnails over to `retired`, still registered.
  pub fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    retired.extend(self.thumbnail.take());
    for overlay in &mut self.overlays {
      retired.extend(overlay.thumbnail.take());
    }
  }

  pub fn hide(&mut self) {
    let overlays = self
      .overlays
      .iter_mut()
      .filter_map(|overlay| overlay.thumbnail.as_mut());

    for thumbnail in self.thumbnail.iter_mut().chain(overlays) {
      thumbnail.hide();
    }
  }

  /// The part of the window its tile shows.
  #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
  fn source(&self) -> RECT {
    let (width, height) = (
      (self.frame.right - self.frame.left) as f32,
      (self.frame.bottom - self.frame.top) as f32,
    );
    let crop = self.tile.crop;

    RECT {
      left: self.frame.left + (crop.x * width).round() as i32,
      top: self.frame.top + (crop.y * height).round() as i32,
      right: self.frame.left + (crop.right() * width).round() as i32,
      bottom: self.frame.top + (crop.bottom() * height).round() as i32,
    }
  }

  /// Shows the preview at `dest` on the overview, with its overlays
  /// around it, cut to `clip`.
  pub fn place(&mut self, dest: &RectF, clip: &RectF, opacity: f32) {
    let clip_rect = to_win32(clip);

    for index in 0..self.overlays.len() {
      let mapped = self.overlay_rect(&self.overlays[index].overlay, dest);
      let overlay = &mut self.overlays[index];
      let Some(thumbnail) = &mut overlay.thumbnail else {
        continue;
      };

      match mapped {
        Some((source, mapped)) => thumbnail.place(
          source,
          to_win32(&mapped),
          &clip_rect,
          to_opacity(opacity),
        ),
        None => thumbnail.hide(),
      }
    }

    let source = self.source();
    let (source, dest) = match self.companion {
      Some(companion) => companion.map_rects(source, to_win32(dest)),
      None => (source, to_win32(dest)),
    };

    if let Some(thumbnail) = &mut self.thumbnail {
      thumbnail.place(
        source,
        dest,
        &clip_rect,
        to_opacity(opacity * self.alpha),
      );
    }
  }

  /// `dest` grown to take in its overlays shown around it, e.g. the hole
  /// its card leaves for it.
  pub fn bounds(&self, dest: &RectF) -> RectF {
    self
      .overlays
      .iter()
      .filter_map(|overlay| self.overlay_rect(&overlay.overlay, dest))
      .fold(*dest, |bounds, (_, rect)| bounds.union(&rect))
  }

  /// Where `overlay` shows with the window shown at `dest`, and the part
  /// of the overlay's window to show there.
  fn overlay_rect(
    &self,
    overlay: &OverviewOverlay,
    dest: &RectF,
  ) -> Option<(RECT, RectF)> {
    around(&self.screen_rect, &self.tile.crop, overlay.outset, dest)
  }
}

/// Where an overlay reaching `outset` (left, top, right, bottom) past a
/// window at `screen` shows, the part `crop` of that window showing at
/// `dest`: scaled and offset as the window is. Also returns the part of
/// the overlay's window to show there: all of it.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn around(
  screen: &RectF,
  crop: &RectF,
  outset: [i32; 4],
  dest: &RectF,
) -> Option<(RECT, RectF)> {
  // The part of the window `dest` shows, on screen.
  let shown = RectF::new(
    screen.x + crop.x * screen.w,
    screen.y + crop.y * screen.h,
    crop.w * screen.w,
    crop.h * screen.h,
  );
  if shown.w <= 0.0 || shown.h <= 0.0 {
    return None;
  }

  let [left, top, right, bottom] = outset.map(|px| px as f32);
  let on_screen = RectF::new(
    screen.x - left,
    screen.y - top,
    screen.w + left + right,
    screen.h + top + bottom,
  );
  let (scale_x, scale_y) = (dest.w / shown.w, dest.h / shown.h);

  let source = RECT {
    right: on_screen.w.round() as i32,
    bottom: on_screen.h.round() as i32,
    ..Default::default()
  };
  let rect = RectF::new(
    dest.x + (on_screen.x - shown.x) * scale_x,
    dest.y + (on_screen.y - shown.y) * scale_y,
    on_screen.w * scale_x,
    on_screen.h * scale_y,
  );

  Some((source, rect))
}

/// `hwnd`'s visible frame, without its invisible resize borders, in the
/// window's own coordinates.
pub(crate) fn visible_frame(hwnd: HWND) -> Option<RECT> {
  let (bounds, frame) = window_class::bounds_and_frame(hwnd)?;

  let visible = RECT {
    left: frame.left - bounds.left,
    top: frame.top - bounds.top,
    right: frame.right - bounds.left,
    bottom: frame.bottom - bounds.top,
  };

  (visible.right > visible.left && visible.bottom > visible.top)
    .then_some(visible)
}

/// `hwnd`'s visible frame on screen.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn visible_frame_on_screen(hwnd: HWND) -> Option<RectF> {
  let (_, frame) = window_class::bounds_and_frame(hwnd)?;

  (frame.right > frame.left && frame.bottom > frame.top).then(|| {
    RectF::new(
      frame.left as f32,
      frame.top as f32,
      (frame.right - frame.left) as f32,
      (frame.bottom - frame.top) as f32,
    )
  })
}

/// `dest` cut to the part inside `clip`, and `source` cut by the same
/// share, or `None` when nothing of `dest` is inside.
///
/// DWM draws a thumbnail wherever its destination is, also past its host
/// window, so a card running off the overview would show on the monitor
/// beside it.
#[allow(clippy::cast_possible_truncation)]
fn clip_rects(
  source: RECT,
  dest: RECT,
  clip: &RECT,
) -> Option<(RECT, RECT)> {
  let visible = RECT {
    left: dest.left.max(clip.left),
    top: dest.top.max(clip.top),
    right: dest.right.min(clip.right),
    bottom: dest.bottom.min(clip.bottom),
  };

  if visible.right <= visible.left || visible.bottom <= visible.top {
    return None;
  }

  if visible == dest {
    return Some((source, dest));
  }

  let scale_x = f64::from(source.right - source.left)
    / f64::from(dest.right - dest.left);
  let scale_y = f64::from(source.bottom - source.top)
    / f64::from(dest.bottom - dest.top);
  let scaled =
    |pixels: i32, scale: f64| (f64::from(pixels) * scale).round() as i32;

  let source = RECT {
    left: source.left + scaled(visible.left - dest.left, scale_x),
    top: source.top + scaled(visible.top - dest.top, scale_y),
    right: source.right - scaled(dest.right - visible.right, scale_x),
    bottom: source.bottom - scaled(dest.bottom - visible.bottom, scale_y),
  };

  Some((source, visible))
}

pub(crate) fn to_win32(rect: &RectF) -> RECT {
  let rect = rect.to_rect();
  RECT {
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
  }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn to_opacity(opacity: f32) -> u8 {
  (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
  use windows::Win32::Foundation::RECT;

  use super::{around, clip_rects};
  use crate::overview_layout::RectF;

  fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
      left,
      top,
      right,
      bottom,
    }
  }

  #[test]
  fn inside_the_clip_is_left_as_is() {
    let clip = rect(0, 0, 1920, 1080);

    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(10, 10, 410, 310), &clip),
      Some((rect(0, 0, 800, 600), rect(10, 10, 410, 310)))
    );
  }

  #[test]
  fn past_the_clip_is_cut_with_its_source() {
    // Half-size: the 100 on-screen pixels past the right edge are 200 of
    // the source.
    let clip = rect(0, 0, 1920, 1080);

    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(1720, 0, 2120, 300), &clip),
      Some((rect(0, 0, 400, 600), rect(1720, 0, 1920, 300)))
    );
    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(-100, 900, 300, 1200), &clip),
      Some((rect(200, 0, 800, 360), rect(0, 900, 300, 1080)))
    );
  }

  #[test]
  fn wholly_outside_the_clip_shows_nothing() {
    let clip = rect(0, 0, 1920, 1080);

    assert_eq!(
      clip_rects(rect(0, 0, 800, 600), rect(1920, 0, 2320, 300), &clip),
      None
    );
  }

  #[test]
  fn an_overlay_shows_around_its_window_as_scaled() {
    let screen = RectF::new(100.0, 100.0, 800.0, 600.0);
    let whole = RectF::new(0.0, 0.0, 1.0, 1.0);
    let dest = RectF::new(10.0, 20.0, 200.0, 150.0);

    let (source, shown) =
      around(&screen, &whole, [10, 20, 10, 0], &dest).expect("shown");
    assert_eq!(source, rect(0, 0, 820, 620));
    assert_eq!(shown, RectF::new(7.5, 15.0, 205.0, 155.0));

    // A backdrop covers the window exactly.
    assert_eq!(
      around(&screen, &whole, [0; 4], &dest).map(|(_, shown)| shown),
      Some(dest)
    );
  }

  #[test]
  fn an_overlay_follows_the_part_of_its_window_shown() {
    // Its right half shown, twice as large.
    let screen = RectF::new(0.0, 0.0, 400.0, 200.0);
    let crop = RectF::new(0.5, 0.0, 0.5, 1.0);
    let dest = RectF::new(0.0, 0.0, 400.0, 400.0);

    let (_, shown) =
      around(&screen, &crop, [5, 5, 5, 5], &dest).expect("shown");
    assert_eq!(shown, RectF::new(-410.0, -10.0, 820.0, 420.0));

    let nothing = RectF::new(0.0, 0.0, 0.0, 1.0);
    assert_eq!(around(&screen, &nothing, [0; 4], &dest), None);
  }
}
