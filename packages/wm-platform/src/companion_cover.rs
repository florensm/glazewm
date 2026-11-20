use std::time::{Duration, Instant};

use windows::Win32::{
  Foundation::{HWND, RECT},
  UI::WindowsAndMessaging::IsIconic,
};

use crate::{
  companion::{self, Companion, COMPANION_OPACITY},
  native_surrogate::{content_rects, to_logical},
  platform_impl,
  window_edges::compute_border_inset,
  CornerStyle, NativeSurrogate, NativeWindow, Rect,
};

/// Upper bound on how long a cover waits for its window to land where it
/// covers it, e.g. for an app that doesn't take the move or takes
/// another size.
///
/// A posted move normally lands within a few milliseconds. Meanwhile the
/// cover shows the companion where the window goes.
const LANDING_MAX_WAIT: Duration = Duration::from_millis(250);

/// A click-through, opaque copy of a window's companion (e.g. shades'
/// recolored copy of it), kept over the window as it is uncloaked, moved
/// or resized without an animation, until the companion covers it again.
///
/// Uncovered, the window shows as it really is (e.g. white) until the
/// companion's owner notices the uncloak or move, and the companion of a
/// window hidden in its place at the same time (e.g. the previous tab of a
/// stack) stays on screen over it until its owner cloaks that one.
pub struct CompanionCover {
  surrogate: NativeSurrogate,
  window: HWND,
  /// Where the cover is, in the window's `GetWindowRect` space.
  rect: Rect,
  border_inset: RECT,
  /// When the cover was put at `rect`.
  placed_at: Instant,
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
      rect: rect.clone(),
      border_inset,
      placed_at: Instant::now(),
      obstructions,
    };
    cover.cover_window();
    cover.surrogate.set_visible(true);

    Some(cover)
  }

  /// Moves the cover to `rect`, where its window is being moved, and
  /// waits anew for the companion to cover it there.
  pub fn retarget(&mut self, rect: &Rect) {
    if *rect == self.rect {
      return;
    }

    if let Err(err) = self
      .surrogate
      .reposition(&to_logical(rect, &self.border_inset))
    {
      tracing::warn!("Failed to move a companion cover: {err}");
    }

    self.rect = rect.clone();
    self.placed_at = Instant::now();
    self.surrogate.begin_hand_back();
    self.refresh_thumbnail();
    self.cover_window();
  }

  /// Maps the cover's content anew onto the companion as it is now, e.g.
  /// once its owner has resized it to the window's new size.
  fn refresh_thumbnail(&mut self) {
    let logical = to_logical(&self.rect, &self.border_inset);
    let (src, dst) =
      content_rects(logical.width(), logical.height(), self.border_inset);
    self.surrogate.set_thumbnail_rects(src, dst);
  }

  /// Whether the window is where the cover is, or the wait for it to get
  /// there is over (see [`LANDING_MAX_WAIT`]).
  fn has_landed(&self) -> bool {
    self.placed_at.elapsed() >= LANDING_MAX_WAIT
      || platform_impl::NativeWindow::new(self.window.0)
        .frame_with_shadows()
        .is_ok_and(|frame| frame == self.rect)
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

  /// Whether to keep covering the window: it hasn't landed where the
  /// cover is yet, or its companion isn't back over it there, or an
  /// obstruction still shows above that, and the wait hasn't timed out or
  /// become pointless (see `HandoffWait`). Restacks the cover first (see
  /// [`cover_window`](Self::cover_window)).
  ///
  /// A minimized window ends the wait: its companion is never put back
  /// over it.
  pub fn awaits_companion(&mut self) -> bool {
    // SAFETY: A stale handle just returns false.
    if unsafe { IsIconic(self.window) }.as_bool() {
      return false;
    }

    self.cover_window();
    self.refresh_thumbnail();

    // Checked before the companion: one still over the window where it
    // was would pass for covering it.
    if !self.has_landed() {
      return true;
    }

    self
      .surrogate
      .awaits_unobstructed_companion(&self.obstructions)
  }
}
