use serde::{Deserialize, Serialize};
use wm_platform::Rect;

/// How the WM shows a window on screen, for programs drawing live
/// previews of it (e.g. an overview). Rects are in physical pixels.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowStyleDto {
  /// Its visible frame where it rests, as laid out: where the user sees
  /// it, also on a workspace not displayed.
  pub frame: Rect,

  /// The whole window where it rests, invisible resize borders included.
  /// A DWM thumbnail of the window shows all of it, so cutting the
  /// thumbnail to `frame` shows what is on screen.
  pub bounds: Rect,

  /// Opacity its `transparency` effect gives it, from 0 to 255, which DWM
  /// thumbnails of it don't carry.
  pub opacity: u8,

  /// A window another program draws over this one, covering `frame`
  /// exactly (e.g. shades' recolored copy), shown in the window's place.
  pub companion_handle: Option<isize>,

  /// What the WM draws around and behind it, bottom to top, while the
  /// window shows on screen (the WM hides them otherwise).
  pub overlays: Vec<WindowOverlayDto>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowOverlayDto {
  pub handle: isize,
  pub kind: WindowOverlayKind,

  /// The overlay's whole window where it rests around `frame`; a DWM
  /// thumbnail of `handle` shows all of it.
  pub rect: Rect,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowOverlayKind {
  Backdrop,
  Border,
}
