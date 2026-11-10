use windows::Win32::{
  Foundation::HWND, UI::WindowsAndMessaging::IsIconic,
};

use crate::{
  companion::{self, Companion, COMPANION_OPACITY},
  native_surrogate::to_logical,
  resize_session::compute_border_inset,
  CornerStyle, NativeSurrogate, NativeWindow, Rect,
};

/// A click-through, opaque copy of a window's companion (e.g. shades'
/// recolored copy of it), kept over the window as it is uncloaked without
/// an animation, until the companion covers it again.
///
/// Uncovered, the window shows as it really is (e.g. white) until the
/// companion's owner notices the uncloak, and the companion of a window
/// hidden in its place at the same time (e.g. the previous tab of a stack)
/// stays on screen over it until its owner cloaks that one.
pub struct CompanionCover {
  surrogate: NativeSurrogate,
  window: HWND,
  /// Companions of windows hidden as this one was shown, over the same
  /// place, which the cover stays above while they are on screen.
  obstructions: Vec<Companion>,
}

impl CompanionCover {
  /// Shows a cover of the cloaked `window` at `rect`, above it and above
  /// the companions of `hiding`, windows about to be hidden, that are on
  /// screen over the same place.
  ///
  /// `None`, creating no window, when `window` has no companion or its
  /// thumbnail can't be registered. The cover has to be composited once
  /// before `window` is uncloaked, so it is drawn as the uncloak lands.
  #[must_use]
  pub fn new(
    window: &NativeWindow,
    rect: &Rect,
    corner_style: &CornerStyle,
    hiding: &[HWND],
  ) -> Option<Self> {
    let hwnd = window.hwnd();
    Companion::find(hwnd)?;

    let border_inset = compute_border_inset(hwnd);
    let surrogate = NativeSurrogate::create(
      hwnd,
      rect,
      rect,
      COMPANION_OPACITY,
      border_inset,
      corner_style,
      hwnd,
      false,
    )
    .inspect_err(|err| {
      tracing::warn!("Failed to create a companion cover: {err}");
    })
    .ok()?;

    if !surrogate.has_thumbnail() || !surrogate.shows_companion() {
      return None;
    }

    let frame = to_logical(rect, &border_inset);
    let obstructions = hiding
      .iter()
      .filter_map(|hidden| Companion::find(*hidden))
      .filter(|obstruction| {
        obstruction.is_on_screen()
          && obstruction
            .bounds()
            .is_some_and(|bounds| bounds.intersection(&frame).is_some())
      })
      .collect();

    let mut cover = Self {
      surrogate,
      window: hwnd,
      obstructions,
    };
    cover.cover_window();
    cover.surrogate.set_visible(true);

    Some(cover)
  }

  /// Puts the cover back above its window and the obstructions still on
  /// screen, should something (e.g. focusing the window) have raised one
  /// past it.
  pub fn cover_window(&self) {
    let anchor = self
      .obstructions
      .iter()
      .filter(|obstruction| obstruction.is_on_screen())
      .fold(self.window, |anchor, obstruction| {
        if companion::is_above(obstruction.hwnd(), anchor) {
          obstruction.hwnd()
        } else {
          anchor
        }
      });

    self.surrogate.keep_above(anchor);
  }

  /// Whether to keep covering the window: its companion isn't back over
  /// it, or an obstruction still shows above that, and the wait hasn't
  /// timed out or become pointless (see `HandoffWait`). Restacks the
  /// cover first (see [`cover_window`](Self::cover_window)).
  ///
  /// A minimized window ends the wait: its companion is never put back
  /// over it.
  pub fn awaits_companion(&mut self) -> bool {
    // SAFETY: A stale handle just returns false.
    if unsafe { IsIconic(self.window) }.as_bool() {
      return false;
    }

    self.cover_window();
    self
      .surrogate
      .awaits_unobstructed_companion(&self.obstructions)
  }
}
