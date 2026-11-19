//! Windows drawn scaled on their peeks as their size changes, their apps
//! resized behind them.

use std::time::{Duration, Instant};

use wm_platform::{Color, Rect};

use super::{Hold, PeekResize};
use crate::{traits::WindowGetters, wm_state::WmState};

/// How long a stretched window takes to ease back to the scale of its
/// rect over its target once it has grown (see [`Stretch`]): a few frames,
/// short of `window_resize`'s 150 ms.
const STRETCH_SETTLE: Duration = Duration::from_millis(80);

/// Whether a window going from `from` to `target`, drawn filled, is sent
/// its new size as its motion starts: only one growing on both axes,
/// whose new content then arrives under the cut. One that shrinks on
/// either axis keeps its size until it lands, cut at its tile, so its
/// content never falls short of the drawn frame.
pub(super) fn resizes_at_start(from: &Rect, target: &Rect) -> bool {
  target.width() >= from.width() && target.height() >= from.height()
}

/// A window drawn scaled as its size changes, its app resized behind its
/// cloak as the motion starts.
///
/// Its picture always covers its animated rect, sampled from no more of
/// the window than it is sure to have. Its scale is the rect over its
/// target, which ends 1:1 and doesn't change as the app's content
/// arrives, times a cover of 1 or more (see
/// `WindowPeek::place_stretched`): what it takes a window yet to grow to
/// reach across the rect. Once it has grown, the cover eases back to 1
/// over [`STRETCH_SETTLE`] rather than stepping, which shows as a zoom at
/// landing.
#[derive(Clone, Debug)]
pub(super) struct Stretch {
  /// The real window's rect, as last read.
  pub(super) actual: Option<Rect>,
  /// Whether the window has been seen at its target's size, after which
  /// it is sampled whole: before, no larger than the target, as a resize
  /// still on its way there may land before the frame is composited.
  pub(super) reached: bool,
  /// The cover it was last drawn with, per axis.
  pub(super) cover: (f32, f32),
  /// When its cover began to ease back to 1, and from what.
  settle: Option<(Instant, (f32, f32))>,
  /// Whether it is drawn filled rather than scaled (see [`PeekResize`]).
  pub(super) filled: bool,
  /// Whether, filled, its window is sent its new size only as it lands
  /// (see [`resizes_at_start`]). Sampled as it is meanwhile.
  pub(super) deferred: bool,
  /// The window's edge color, as last sampled, which it is filled with
  /// (see [`note_fill_inputs`]).
  pub(super) edge_color: Option<Color>,
  /// The smallest size, per axis, the window was sent by a grab and may
  /// not have reported yet (see `WindowPlacer::unlanded`).
  pub(super) in_flight: Option<(i32, i32)>,
}

impl Stretch {
  /// One for a window whose rect is `actual`, if it could be read, drawn
  /// as `resize` says, its new size sent only as it lands if `deferred`.
  pub(super) fn at(
    actual: Option<Rect>,
    resize: PeekResize,
    deferred: bool,
  ) -> Self {
    let filled = resize == PeekResize::Fill;

    Self {
      actual,
      reached: false,
      cover: (1.0, 1.0),
      settle: None,
      filled,
      deferred: filled && deferred,
      edge_color: None,
      in_flight: None,
    }
  }

  pub(super) fn resize(&self) -> PeekResize {
    if self.filled {
      PeekResize::Fill
    } else {
      PeekResize::Stretch
    }
  }

  /// Notes at `now` whether the window has reached the size of `target`,
  /// from when its cover settles.
  pub(super) fn note_target(&mut self, target: &Rect, now: Instant) {
    if !self.reached
      && self
        .actual
        .as_ref()
        .is_some_and(|actual| is_same_size(actual, target))
    {
      self.reached = true;
      self.settle = Some((now, self.cover));
    }
  }

  /// Heads it for `target` instead of `from` at `now`: yet to reach it,
  /// its cover settling from what keeps the scale it was last drawn at.
  #[allow(clippy::cast_precision_loss)]
  fn retarget(&mut self, from: &Rect, target: &Rect, now: Instant) {
    let kept = |cover: f32, from: i32, to: i32| {
      if from > 0 {
        (cover * to as f32 / from as f32).max(1.0)
      } else {
        1.0
      }
    };
    let cover = self.cover;

    self.reached = false;
    self.settle = Some((
      now,
      (
        kept(cover.0, from.width(), target.width()),
        kept(cover.1, from.height(), target.height()),
      ),
    ));
  }

  /// The least cover it is drawn with at `now`.
  pub(super) fn cover_at(&self, now: Instant) -> (f32, f32) {
    self.settle.map_or((1.0, 1.0), |(since, from)| {
      let elapsed = now.saturating_duration_since(since);
      (settling(from.0, elapsed), settling(from.1, elapsed))
    })
  }

  /// Whether its scale has come to rest at `now`, so it can land. One
  /// short of its target isn't waited for.
  pub(super) fn is_settled(&self, now: Instant) -> bool {
    !self.reached || self.cover_at(now) == (1.0, 1.0)
  }
}

/// A cover easing from `from` back to 1, `elapsed` after it began to:
/// fast at first, 1 from [`STRETCH_SETTLE`] on.
fn settling(from: f32, elapsed: Duration) -> f32 {
  let left =
    1.0 - (elapsed.as_secs_f32() / STRETCH_SETTLE.as_secs_f32()).min(1.0);

  1.0 + (from - 1.0) * left * left
}

/// The size of a window, drawn filled at `rect`, that is sure to be drawn
/// (see `WindowPeek::place_filled`): per axis, the smallest of its size as
/// last read, `actual`, the rect's, and those of sizes it was sent and is
/// yet to report, `in_flight`.
pub(super) fn sampled_size(
  actual: &Rect,
  rect: &Rect,
  in_flight: impl IntoIterator<Item = (i32, i32)>,
) -> (i32, i32) {
  [
    (actual.width(), actual.height()),
    (rect.width(), rect.height()),
  ]
  .into_iter()
  .chain(in_flight)
  .fold((i32::MAX, i32::MAX), |(width, height), size| {
    (width.min(size.0), height.min(size.1))
  })
}

/// Notes on the peek of each window drawn filled its edge color, as last
/// sampled, to fill with, and the sizes a grab sent it that it may not
/// have reported yet.
pub(in crate::animation) fn note_fill_inputs(state: &mut WmState) {
  let inputs = state
    .animation_manager
    .peeks
    .entries
    .iter()
    .filter(|(_, entry)| {
      entry
        .stretch
        .as_ref()
        .is_some_and(|stretch| stretch.filled || entry.hold != Hold::Free)
    })
    .filter_map(|(id, _)| {
      let native = state.window_by_id(*id)?.native().clone();
      let color =
        state.animation_manager.cached_edge_color(native.hwnd().0);
      Some((*id, color, state.grab.unlanded(native.id())))
    })
    .collect::<Vec<_>>();

  for (id, color, in_flight) in inputs {
    if let Some(stretch) = state
      .animation_manager
      .peeks
      .entries
      .get_mut(&id)
      .and_then(|entry| entry.stretch.as_mut())
    {
      stretch.edge_color = color.or(stretch.edge_color);
      stretch.in_flight = in_flight;
    }
  }
}

/// Whether `a` and `b` are the same size.
pub(super) fn is_same_size(a: &Rect, b: &Rect) -> bool {
  a.width() == b.width() && a.height() == b.height()
}

/// The stretch an entry keeps as it is moved on at `now`, from heading
/// for `from` to `target`: none without a `resize`, else its own,
/// retargeted if its size changes (see [`Stretch::retarget`]), or a new
/// one for a window at `actual`, its new size sent as it lands if
/// `deferred`.
pub(super) fn restretched(
  stretch: Option<Stretch>,
  resize: Option<PeekResize>,
  deferred: bool,
  actual: impl FnOnce() -> Option<Rect>,
  from: &Rect,
  target: &Rect,
  now: Instant,
) -> Option<Stretch> {
  resize.map(|resize| match stretch {
    Some(mut kept) => {
      if !is_same_size(from, target) {
        kept.retarget(from, target, now);
      }
      kept
    }
    None => Stretch::at(actual(), resize, deferred),
  })
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use wm_platform::Rect;

  use super::{
    resizes_at_start, restretched, sampled_size, settling, Stretch,
    STRETCH_SETTLE,
  };
  use crate::animation::peek_view::{tests::tile, PeekResize};

  /// A cover eases back to 1 without a step: all there as it begins,
  /// most of the way early, exactly 1 from the settle time on.
  #[test]
  fn a_cover_settles_to_one_without_a_step() {
    assert_eq!(settling(1.5, Duration::ZERO), 1.5);
    let half = settling(1.5, STRETCH_SETTLE / 2);
    assert!(half > 1.0 && half < 1.25, "{half}");
    assert_eq!(settling(1.5, STRETCH_SETTLE), 1.0);
    assert_eq!(settling(1.5, STRETCH_SETTLE * 3), 1.0);
    assert_eq!(settling(1.0, Duration::ZERO), 1.0);
  }

  /// A window seen at its target's size settles from the cover it was
  /// last drawn with, and lands only once that is 1. One short of its
  /// target isn't waited for.
  #[test]
  fn a_stretch_settles_once_its_window_has_its_size() {
    let t0 = Instant::now();
    let target = Rect::from_xy(100, 50, 1200, 600);
    let mut stretch =
      Stretch::at(Some(tile()), PeekResize::Stretch, false);
    stretch.cover = (1.5, 1.0);

    stretch.note_target(&target, t0);
    assert!(!stretch.reached && stretch.is_settled(t0));
    assert_eq!(stretch.cover_at(t0), (1.0, 1.0));

    // Reached wherever it is: only its size counts.
    stretch.actual = Some(Rect::from_xy(500, 50, 1200, 600));
    stretch.note_target(&target, t0);
    assert!(stretch.reached);
    assert_eq!(stretch.cover_at(t0), (1.5, 1.0));
    assert!(!stretch.is_settled(t0));

    // Noted again later, it doesn't start over.
    let later = t0 + STRETCH_SETTLE;
    stretch.note_target(&target, later);
    assert_eq!(stretch.cover_at(later), (1.0, 1.0));
    assert!(stretch.is_settled(later));
  }

  /// A stretch kept as its window moves on keeps its last read. Headed
  /// for another size, it is yet to reach it, and keeps its scale: its
  /// cover times the new target over the old, or 1 where that is less.
  #[test]
  fn a_kept_stretch_keeps_its_scale_for_a_new_size() {
    let t0 = Instant::now();
    let old = Rect::from_xy(100, 50, 1000, 600);
    let read = || {
      let mut stretch =
        Stretch::at(Some(old.clone()), PeekResize::Stretch, false);
      stretch.note_target(&old, t0);
      stretch
    };
    let unread = || -> Option<Rect> { panic!("read again") };

    let moved = restretched(
      Some(read()),
      Some(PeekResize::Stretch),
      false,
      unread,
      &old,
      &Rect::from_xy(500, 50, 1000, 600),
      t0,
    );
    assert!(moved.is_some_and(|kept| kept.reached));

    let wider = Rect::from_xy(100, 50, 1200, 300);
    let resized = restretched(
      Some(read()),
      Some(PeekResize::Stretch),
      false,
      unread,
      &old,
      &wider,
      t0,
    );
    assert!(resized.is_some_and(|kept| {
      !kept.reached
        && kept.actual == Some(old.clone())
        && kept.cover_at(t0) == (1.2, 1.0)
    }));

    assert!(restretched(
      Some(read()),
      None,
      false,
      unread,
      &old,
      &wider,
      t0
    )
    .is_none());

    let fresh = restretched(
      None,
      Some(PeekResize::Fill),
      true,
      || Some(tile()),
      &old,
      &wider,
      t0,
    );
    assert!(fresh.is_some_and(|fresh| {
      fresh.actual == Some(tile())
        && !fresh.reached
        && fresh.filled
        && fresh.deferred
        && fresh.cover_at(t0) == (1.0, 1.0)
    }));
  }

  /// Filled, a window growing on both axes is sent its size as it starts;
  /// one shrinking on either keeps its size until it lands.
  #[test]
  fn a_filled_window_is_resized_at_the_start_only_to_grow() {
    let from = Rect::from_xy(0, 0, 800, 600);

    assert!(resizes_at_start(&from, &Rect::from_xy(0, 0, 900, 600)));
    assert!(resizes_at_start(&from, &Rect::from_xy(50, 0, 900, 700)));
    assert!(!resizes_at_start(&from, &Rect::from_xy(0, 0, 700, 600)));
    assert!(!resizes_at_start(&from, &Rect::from_xy(0, 0, 900, 500)));
  }

  /// Filled, a window samples per axis no more than it was read at, its
  /// rect, and a size it is yet to report.
  #[test]
  fn a_filled_window_samples_the_smallest_size_it_is_sure_to_have() {
    let actual = Rect::from_xy(0, 0, 1000, 600);
    let rect = Rect::from_xy(0, 0, 980, 700);

    assert_eq!(sampled_size(&actual, &rect, None), (980, 600));

    // Sent 950 wide, not yet read back: it may land before the frame
    // shows.
    assert_eq!(
      sampled_size(&actual, &rect, [(950, 800), (990, 650)]),
      (950, 600)
    );
  }
}
