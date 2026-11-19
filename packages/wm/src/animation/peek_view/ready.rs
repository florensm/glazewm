//! Ready peeks: a tile's peek kept hosted after it lands, so its next
//! motion starts without making one.

use std::time::{Duration, Instant};

use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{
  perf::{self, Stage},
  Rect, WindowPeek, HWND,
};

use super::{
  entry_on,
  motions::is_shown_tile,
  overlays::{carried_styles, existing_ring, PeekStyles},
  surfaces::{surface_clips, Layer},
  Entry, PeekView, Role,
};
use crate::{
  animation::state::WindowAnimationState,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// How long the WM is left idle, after a sync or a landing, before tiles'
/// peeks are made ready (see [`prewarm_ready`]): past a burst of
/// commands, so none waits on the work.
const PREWARM_DELAY: Duration = Duration::from_millis(150);

/// A tile's peek hosted on a `PeekSurface`, kept for the tile's next
/// motion with every registration it has, held so it draws nothing (see
/// `WindowPeek::set_ready`): a landed one, or one made ready while the
/// tile rests (see [`prewarm_ready`]). Kept while its window is shown.
/// Apart from `PeekView::entries`, so no query over those sees it.
pub(super) struct Ready {
  peek: WindowPeek,
  pub(super) fit: ReadyFit,
}

/// What a [`Ready`] peek was made for, which a new motion must ask for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadyFit {
  /// The real window it shows.
  pub(super) source: HWND,
  /// Its surface's clip.
  pub(super) clip: Rect,
}

/// Whether a landed entry's peek is kept ready: a tile's or a column's,
/// hosted on a surface, drawn on no other peek.
fn keeps_ready(is_hosted: bool, has_trail: bool) -> bool {
  is_hosted && !has_trail
}

/// Whether a ready peek made for `fit` can draw a motion asking for
/// `wanted`. Its surface must take the motion too (see
/// [`PeekView::tenancy`]).
fn takes_ready(fit: &ReadyFit, wanted: &ReadyFit) -> bool {
  fit == wanted
}

/// Why a tile's motion on a surface started on a new peek, not its ready
/// one (see [`ready_miss`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadyMiss {
  /// None was kept for the window.
  NoneKept,
  /// One was kept for another handle of it or another clip.
  OtherFit,
  /// One fit, but the motion is drawn on another surface, or on none.
  OtherSurface,
}

/// Why a motion asking for `wanted` did not start on the ready peek kept
/// for `kept`, if any; `None` if it did (`taken`).
fn ready_miss(
  kept: Option<&ReadyFit>,
  wanted: &ReadyFit,
  taken: bool,
) -> Option<ReadyMiss> {
  if taken {
    return None;
  }

  Some(match kept {
    None => ReadyMiss::NoneKept,
    Some(kept) if !takes_ready(kept, wanted) => ReadyMiss::OtherFit,
    Some(_) => ReadyMiss::OtherSurface,
  })
}

/// Counts whether `window`'s motion on a surface started on its ready peek
/// (see `perf::note_ready_peek`), and logs why not if it didn't.
pub(super) fn note_ready_take(
  window: &WindowContainer,
  kept: Option<&ReadyFit>,
  wanted: &ReadyFit,
  taken: bool,
) {
  let miss = ready_miss(kept, wanted, taken);
  perf::note_ready_peek(miss.is_none());
  if let Some(miss) = miss {
    tracing::debug!("No ready peek for the motion of {window}: {miss:?}.");
  }
}

/// Whether a tile at rest gets a ready peek made (see [`prewarm_ready`]):
/// shown, tiled outside a scrolling layout, as its moves are drawn on its
/// monitor's surface, and drawn by no peek or other stand-in now.
// Independent conditions, each keeping a tile from a ready peek.
#[allow(clippy::fn_params_excessive_bools)]
fn wants_ready(
  is_shown: bool,
  is_tile: bool,
  is_column: bool,
  is_drawn: bool,
) -> bool {
  is_shown && is_tile && !is_column && !is_drawn
}

/// Whether a `PeekSurface` that `is_unused` (no tenant, no ready peek) at
/// `clip` is dropped, as a new one is made or at a purge point: once its
/// clip is none of `monitor_clips` (none at all for a reload, pause or
/// exit). Never one in use, so its tenants always go before it.
pub(super) fn is_stale_surface(
  is_unused: bool,
  clip: &Rect,
  monitor_clips: &[Rect],
) -> bool {
  is_unused && !monitor_clips.contains(clip)
}

/// Whether a peek reclaimed from a ready one when the next registration
/// order was `ready_mark` has registered nothing since: its last
/// registration, `last`, is below the mark. One not reclaimed, or with no
/// thumbnail, isn't kept so.
pub(super) fn kept_since_ready(
  ready_mark: Option<u64>,
  last: Option<u64>,
) -> bool {
  ready_mark.zip(last).is_some_and(|(mark, last)| last < mark)
}

/// Drops the ready peeks of windows no longer shown (see
/// [`is_shown_tile`]). Free while none is kept.
pub(super) fn drop_stray_ready(state: &mut WmState) {
  if state.animation_manager.peeks.ready.is_empty() {
    return;
  }

  let strays = state
    .animation_manager
    .peeks
    .ready
    .keys()
    .filter(|id| is_shown_tile(state, id) != Some(true))
    .copied()
    .collect::<Vec<_>>();
  let peeks = &mut state.animation_manager.peeks;
  for id in strays {
    peeks.ready.remove(&id);
  }
}

/// Makes one tile at rest a ready peek, or tops one up with its ring copy
/// cut for a resize, so its next motion registers nothing (see
/// [`Ready`]); scheduled again at once while another wants it, so the
/// event loop, which checks input first, gets in between. Run only while
/// nothing animates, the WM not paused (see
/// [`schedule_prewarm`](PeekView::schedule_prewarm)).
///
/// Scans for companion windows first, unless that was just done (see
/// `wm_platform::refresh_companions_at_rest`).
///
/// A ready peek made for another window handle or clip is replaced. Its
/// registrations go on top of everything on its surface, which a motion
/// crossing a tile ranked over it restacks (see
/// [`PeekView::restack_crossings`]).
pub fn prewarm_ready(state: &mut WmState, config: &UserConfig) {
  state.animation_manager.peeks.prewarm_at = None;
  if state.is_paused {
    return;
  }
  // Tried again once whatever draws now is done.
  if state.animation_manager.has_active_animations()
    || state.overview.is_open()
  {
    state
      .animation_manager
      .peeks
      .schedule_prewarm(Instant::now());
    return;
  }

  // At rest, so a ready peek's motion start finds companions without a
  // scan (see `NativeSurrogate::reclaim`).
  wm_platform::refresh_companions_at_rest();
  drop_stray_ready(state);
  let monitor_clips = surface_clips(state);
  let mut wanting = state.windows().into_iter().filter(|window| {
    let id = window.id();
    wants_ready(
      is_shown_tile(state, &id) == Some(true),
      window.state() == WindowState::Tiling,
      crate::layout::scrolling::column_of(&window.as_container())
        .is_some(),
      state.animation_manager.peeks.draws_window(&id)
        || state.animation_manager.has_active_surrogate(&id),
    ) && !state.animation_manager.peeks.prewarm_refused.contains(&id)
  });

  let peeks = &state.animation_manager.peeks;
  let next = wanting.find_map(|window| {
    let clip = window.monitor()?.to_rect().ok()?;
    let fit = ReadyFit {
      source: window.native().hwnd(),
      clip,
    };
    let ring = existing_ring(state, &window)
      .map(|(source, corner_reach)| (source, Some(corner_reach)));
    let kept = peeks.ready.get(&window.id());
    let is_whole = kept.is_some_and(|ready| {
      takes_ready(&ready.fit, &fit)
        && ring.is_none_or(|(source, reach)| {
          ready.peek.keeps_ready_ring(source, reach)
        })
    });
    (!is_whole).then_some((window, fit, ring))
  });
  let more = wanting.next().is_some();
  let Some((window, fit, ring)) = next else {
    return;
  };

  let styles = carried_styles(&window, config);
  let rest = window.to_rect().ok();
  let peeks = &mut state.animation_manager.peeks;
  let id = window.id();
  match peeks.ready.get_mut(&id) {
    Some(ready) if takes_ready(&ready.fit, &fit) => {
      let topped = ring.is_none_or(|(source, reach)| {
        ready.peek.ready_ring(source, reach)
      });
      tracing::debug!("Topped up the ready peek of {window}: {topped}.");
      if !topped {
        peeks.prewarm_refused.insert(id);
      }
    }
    _ => {
      peeks.ready.remove(&id);
      let surface = rest.as_ref().and_then(|rest| {
        let index =
          peeks.surface_index(Layer::Tiles, &fit.clip, &monitor_clips)?;
        Some((rest, peeks.surfaces.get(index)?))
      });
      let peek = surface.map(|(rest, surface)| {
        WindowPeek::ready(
          &window.native(),
          rest,
          surface,
          &styles.backdrops,
          styles.ring_width,
          ring,
        )
      });
      match peek.filter(WindowPeek::has_thumbnail) {
        Some(peek) => {
          tracing::debug!("Made a ready peek of {window}.");
          peeks.ready.insert(id, Ready { peek, fit });
        }
        // Not tried again before the next schedule (e.g. an elevated
        // window, which DWM refuses a thumbnail of).
        None => {
          peeks.prewarm_refused.insert(id);
        }
      }
    }
  }

  if more {
    peeks.prewarm_at = Some(Instant::now());
  }
}

/// [`new_entry`](super::new_entry) on `peek`, a ready peek of `window`
/// (see [`Ready`]), drawn on the surface it is on, which takes it (see
/// [`PeekView::tenancy`]).
pub(super) fn reuse_entry(
  mut peek: WindowPeek,
  window: &WindowContainer,
  motion: WindowAnimationState,
  clip: &Rect,
  opacity: u8,
  styles: &PeekStyles,
  role: Role,
) -> (Entry, bool) {
  let _scope = perf::scope(Stage::PeekNew);
  let mark = wm_platform::next_thumbnail_order();
  peek.reclaim(
    &window.native(),
    &motion.target_rect,
    &styles.backdrops,
    styles.ring_width,
  );

  let (mut entry, covers_shown_window) =
    entry_on(peek, window, motion, clip, opacity, styles, role);
  entry.ready_mark = Some(mark);
  (entry, covers_shown_window)
}

impl PeekView {
  /// Keeps `entry`'s peek, just released, ready for `id`'s next motion if
  /// it is a tile's on a surface (see [`Ready`]); drops it otherwise.
  /// Tops up the ready peeks once the WM has been idle a while after
  /// `now` (see [`prewarm_ready`]).
  pub(super) fn keep_ready(
    &mut self,
    id: Uuid,
    entry: Entry,
    now: Instant,
  ) {
    self.schedule_prewarm(now);
    if !keeps_ready(entry.peek.is_hosted(), entry.trail.is_some()) {
      return;
    }

    let mut peek = entry.peek;
    let fit = ReadyFit {
      source: peek.source(),
      clip: entry.clip,
    };
    peek.set_ready();
    tracing::debug!("Kept the landed peek of {id} ready.");
    self.ready.insert(id, Ready { peek, fit });
  }

  /// Takes `id`'s ready peek if it was made for `wanted`. Any other goes.
  pub(super) fn take_ready(
    &mut self,
    id: &Uuid,
    wanted: &ReadyFit,
  ) -> Option<WindowPeek> {
    self
      .ready
      .remove(id)
      .filter(|ready| takes_ready(&ready.fit, wanted))
      .map(|ready| ready.peek)
  }

  /// Takes `window`'s ready peek only if it was made for it cut at `clip`,
  /// leaving any other kept.
  pub(super) fn take_ready_at(
    &mut self,
    window: &WindowContainer,
    clip: &Rect,
  ) -> Option<WindowPeek> {
    let id = window.id();
    let wanted = ReadyFit {
      source: window.native().hwnd(),
      clip: clip.clone(),
    };
    let fits = self
      .ready
      .get(&id)
      .is_some_and(|ready| takes_ready(&ready.fit, &wanted));
    fits
      .then(|| self.ready.remove(&id))
      .flatten()
      .map(|ready| ready.peek)
  }

  /// Has [`prewarm_ready`] run once the WM has been idle for
  /// [`PREWARM_DELAY`] after `now`, unless something draws then.
  pub fn schedule_prewarm(&mut self, now: Instant) {
    self.prewarm_at = Some(now + PREWARM_DELAY);
    self.prewarm_refused.clear();
  }

  /// When [`prewarm_ready`] is due: `None` while none is scheduled.
  pub fn prewarm_deadline(&self) -> Option<Instant> {
    self.prewarm_at
  }
}

#[cfg(test)]
mod tests {

  use wm_platform::{Rect, HWND};

  use super::{
    is_stale_surface, keeps_ready, kept_since_ready, ready_miss,
    takes_ready, wants_ready, ReadyFit, ReadyMiss,
  };

  /// Moving back onto the monitor it just landed on, a tile's ready peek
  /// there fits its trail, not the part on the monitor it moves onto.
  #[test]
  fn a_ready_peek_fits_the_part_on_its_monitor() {
    let left = Rect::from_xy(0, 0, 1920, 1080);
    let right = Rect::from_xy(1920, 0, 1920, 1080);
    let fit = |clip: &Rect| ReadyFit {
      source: HWND(1),
      clip: clip.clone(),
    };
    assert!(takes_ready(&fit(&right), &fit(&right)));
    assert!(!takes_ready(&fit(&right), &fit(&left)));
  }

  /// Only a hosted peek, drawn on no other, is kept ready.
  #[test]
  fn only_hosted_peeks_are_kept_ready() {
    assert!(keeps_ready(true, false));
    assert!(!keeps_ready(false, false));
    assert!(!keeps_ready(true, true));
  }

  /// A ready peek draws only its own window's motion on the same surface.
  #[test]
  fn a_ready_peek_is_taken_for_its_window_and_clip_only() {
    let fit = ReadyFit {
      source: HWND(7),
      clip: Rect::from_ltrb(0, 0, 1920, 1040),
    };
    let other_window = ReadyFit {
      source: HWND(8),
      ..fit.clone()
    };
    let other_clip = ReadyFit {
      clip: Rect::from_ltrb(1920, 0, 3840, 1040),
      ..fit.clone()
    };

    assert!(takes_ready(&fit, &fit));
    assert!(!takes_ready(&fit, &other_window));
    assert!(!takes_ready(&fit, &other_clip));
  }

  /// A reclaimed ready peek needs no flush until something is registered
  /// on it; any other peek does.
  #[test]
  fn a_ready_peek_with_nothing_registered_since_needs_no_flush() {
    assert!(kept_since_ready(Some(10), Some(9)));
    assert!(!kept_since_ready(Some(10), Some(10)));
    assert!(!kept_since_ready(None, Some(9)));
    assert!(!kept_since_ready(Some(10), None));
  }

  /// A motion that started on a new peek tells why: none kept, one kept
  /// for another handle or clip, or one its surface didn't take.
  #[test]
  fn a_ready_miss_tells_why() {
    let fit = ReadyFit {
      source: HWND(7),
      clip: Rect::from_ltrb(0, 0, 1920, 1040),
    };
    let other_clip = ReadyFit {
      clip: Rect::from_ltrb(1920, 0, 3840, 1040),
      ..fit.clone()
    };

    assert_eq!(ready_miss(Some(&fit), &fit, true), None);
    assert_eq!(ready_miss(None, &fit, false), Some(ReadyMiss::NoneKept));
    assert_eq!(
      ready_miss(Some(&other_clip), &fit, false),
      Some(ReadyMiss::OtherFit)
    );
    assert_eq!(
      ready_miss(Some(&fit), &fit, false),
      Some(ReadyMiss::OtherSurface)
    );
  }

  /// Only a shown tile outside a scrolling layout, drawn by nothing else,
  /// gets a ready peek made.
  #[test]
  fn only_a_resting_shown_tile_is_made_ready() {
    assert!(wants_ready(true, true, false, false));
    assert!(!wants_ready(false, true, false, false));
    assert!(!wants_ready(true, false, false, false));
    assert!(!wants_ready(true, true, true, false));
    assert!(!wants_ready(true, true, false, true));
  }

  /// Only an unused surface off every kept clip goes.
  #[test]
  fn only_unused_surfaces_off_the_monitors_go() {
    let monitor = Rect::from_ltrb(0, 0, 1920, 1080);
    let gone = Rect::from_ltrb(1920, 0, 3840, 1080);
    let monitors = [monitor.clone()];

    assert!(is_stale_surface(true, &gone, &monitors));
    assert!(!is_stale_surface(false, &gone, &monitors));
    assert!(!is_stale_surface(true, &monitor, &monitors));
    // A reload, pause or exit keeps no clip: only those in use stay.
    assert!(is_stale_surface(true, &monitor, &[]));
    assert!(!is_stale_surface(false, &monitor, &[]));
  }
}
