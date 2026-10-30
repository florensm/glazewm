//! Borders around the windows of the activities' scenes whose own border
//! isn't on screen to show, e.g. on a workspace not displayed, whose
//! border overlays the WM hides. Each is one small ring stretched to any
//! size, so moving or zooming a window redraws nothing.

use windows::Win32::Foundation::HWND;

use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{fade, SliceLook},
  overview_layout::RectF,
  overview_overlays::NineSlice,
  BorderOverlayParams,
};

/// Most windows of scenes shown at once with a border drawn for them.
const MAX_BORDERS: usize = 32;

/// A border to draw around a scene window.
pub(super) struct SceneBorder {
  /// Where the window shows.
  pub rect: RectF,

  /// How the WM draws its border, in physical pixels on screen.
  pub params: BorderOverlayParams,

  /// How far the window is scaled from its size on screen.
  pub scale: f32,

  pub opacity: f32,
}

/// Rings for the scenes' windows that need one.
#[derive(Default)]
pub(super) struct SceneBorders {
  rings: Vec<NineSlice>,
}

impl SceneBorders {
  /// Registers on `host`, above everything registered so far, room for
  /// as many borders as `count`, keeping the pictures of those it had.
  pub fn register(&mut self, host: HWND, count: usize) {
    let count = count.min(MAX_BORDERS);
    self.rings.truncate(count);
    while self.rings.len() < count {
      let Some(ring) = NineSlice::new() else {
        break;
      };
      self.rings.push(ring);
    }

    for ring in &mut self.rings {
      ring.register(host);
    }
  }

  /// Hands their thumbnails over to `retired`, still registered.
  pub fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    for ring in &mut self.rings {
      ring.retire(retired);
    }
  }

  /// Shows `borders`, cut to `clip`, and hides the rings left over.
  #[allow(clippy::cast_possible_truncation)]
  pub fn place(&mut self, borders: &[SceneBorder], clip: &RectF) {
    let mut rings = self.rings.iter_mut();

    for border in borders {
      let Some(ring) = rings.next() else {
        break;
      };
      let params = &border.params;

      // As the WM draws it: around the window, its outer edge rounded by
      // the corner radius. Drawn at full size, and scaled with the
      // window.
      ring.draw(SliceLook::ring(
        params.corner_radius.round() as i32,
        params.width.round().max(1.0) as i32,
        fade(params.color, params.opacity),
      ));
      let outset = params.width.round() * border.scale;
      ring.place(
        &border.rect.inset(-outset),
        border.scale,
        clip,
        border.opacity,
      );
    }

    for ring in rings {
      ring.hide();
    }
  }
}
