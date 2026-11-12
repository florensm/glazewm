//! Stand-ins drawing windows whose real windows are cloaked.

use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{
  EasingFunction, EffectSet, ResizeStyle, WindowResizeConfig, WindowState,
  WindowTransitionParams,
};
use wm_platform::{
  perf::{self, Stage},
  BackdropOverlayParams, BorderOverlayParams, Color, CornerStyle,
  NativeBackdropOverlay, NativeBorderOverlay, NativeWindow, Overlay,
  Point, Rect, RectDelta, StyleBlend, SurrogateBatch, WindowPeek, HWND,
};

use super::{
  stand_in::{moved_along, StandIn},
  state::WindowAnimationState,
  transit::{transit_frame, Direction, Transit},
};
use crate::{
  animation::{
    manager::{movement_timing, Motion},
    AnimationManager, AwayTransition,
  },
  commands::general::{
    border_hidden_for_fullscreen, focus_blend, overlay_entry,
    surrogate_effects_for, upsert_clipped_overlay,
    upsert_pinned_border_overlay,
  },
  layout::is_hidden_in_layout,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// How long a stretched window takes to ease back to the scale of its
/// rect over its target once it has grown (see [`Stretch`]): a few frames,
/// short of `window_resize`'s 150 ms.
const STRETCH_SETTLE: Duration = Duration::from_millis(80);

/// How long a window a grab let go of may take to report its last size
/// before its peek lands all the same (see [`Hold::Released`]).
const HOLD_RELEASE_WAIT: Duration = Duration::from_millis(150);

/// How long a window filled on its peek by a keyboard resize may take to
/// report its size once its motion is done before the peek lands all the
/// same, as `ResizeSession::pre_commit` waits (see [`Hold::Released`]).
const FILL_LANDING_WAIT: Duration = Duration::from_millis(500);

/// Whether an entry is kept from landing while its window resizes behind
/// it: by a grab (see [`PeekView::hold`]), or waiting for a filled
/// window's last size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Hold {
  #[default]
  Free,
  /// Drawn where the grab puts it, never landing.
  Held,
  /// Let go of, or done moving, at the instant given: lands once its
  /// window has been read at its target's size, so the hand-back doesn't
  /// swap a filled tile for a window still short of it, or after the
  /// wait given.
  Released(Instant, Duration),
}

impl Hold {
  /// Whether an entry held so may land at `now`, its window having
  /// `reached` its target's size or not.
  fn allows_landing(self, reached: bool, now: Instant) -> bool {
    match self {
      Self::Free => true,
      Self::Held => false,
      Self::Released(at, wait) => {
        reached || now.saturating_duration_since(at) >= wait
      }
    }
  }
}

/// How a window whose size changes on its peek is drawn
/// (`window_resize.style`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeekResize {
  /// Scaled (see [`Stretch`]).
  Stretch,
  /// 1:1, cut at its tile, the rest of the tile in its edge color (see
  /// `WindowPeek::place_filled`).
  Fill,
}

/// Whether a window going from `from` to `target`, drawn filled, is sent
/// its new size as its motion starts: only one growing on both axes,
/// whose new content then arrives under the cut. One that shrinks on
/// either axis keeps its size until it lands, cut at its tile, as
/// `ResizeSession::needs_preposition` has it.
fn resizes_at_start(from: &Rect, target: &Rect) -> bool {
  target.width() >= from.width() && target.height() >= from.height()
}

/// What a peek is currently doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
  /// Shows a partly visible column at rest.
  Peek,
  /// Draws a moving column while its real window is cloaked.
  Moving { ends_shown: bool },
  /// Moved into place, raised above its still-cloaked real window; waits
  /// for that window to be uncloaked.
  HandingBack,
  /// Covers its just-uncloaked real window until it can hand the window
  /// back (see `WindowPeek::hand_back`): at once, or once any companion
  /// of it is on screen too (e.g. shades' recolored copy, shown by its
  /// app only once it notices the uncloak). `settled` is set on the
  /// first tick after that; the peek, still drawing the ring and
  /// backdrop over the real ones, goes on the tick after, a later
  /// composition.
  Releasing { settled: bool },
  /// Draws a window closing or minimizing, its real window cloaked or
  /// minimized, until its transit ends. Never handed back.
  Leaving(AwayTransition),
}

/// How far a pinned peek has taken over its window's border ring or
/// backdrop, which it hands over as a workspace switch does: covered
/// where it is by the copy, hidden a composited frame later (the ring
/// parked where its window is headed), then shown again where the window
/// lands, under the copy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Handover {
  /// The peek draws no copy of it.
  #[default]
  None,
  /// The copy is drawn, the real one still shown where it was.
  Covering,
  /// The copy has been composited, so the real one can go.
  Covered,
  /// Hidden, shown only by the copy.
  Parked,
  /// Shown again where its window landed, under the copy.
  Revealed,
}

pub(super) struct Entry {
  pub(super) peek: WindowPeek,
  pub(super) motion: WindowAnimationState,
  /// Where the peek was last placed, before clipping.
  rect: Rect,
  /// Monitor area the peek is clipped to.
  clip: Rect,
  opacity: u8,
  pub(super) role: Role,
  ring: Handover,
  backdrop: Handover,
  /// Whether the peek has been composited since it went up over its
  /// still-shown real window, so that window can be cloaked without
  /// waiting for another composition (see [`PeekView::is_composited`]).
  composited: bool,
  /// How far the window has entered, for one opening on its peek (see
  /// [`PeekView::enter`]). Its motion then only follows its tile.
  transit: Option<Transit>,
  /// Whether it is a scrolling layout's column, which
  /// [`PeekView::take_departed`] takes once it leaves the layout.
  is_column: bool,
  /// What the overlays of a window leaving read, which a closing one,
  /// out of the tree, has nowhere else (see [`Role::Leaving`]).
  departing: Option<Departing>,
  /// How its window is scaled, for one whose size changes on its peek
  /// with `window_resize.style: "stretch"`.
  stretch: Option<Stretch>,
  /// Whether a grab holds it, which draws it filled rather than
  /// stretched (see `WindowPeek::place_filled`).
  hold: Hold,
  /// Its peek on the monitor it moves from, for a tile moving onto
  /// another one.
  trail: Option<Trail>,
}

/// A second pinned peek of a tile moving onto another monitor, cut at the
/// monitor it comes from (see [`tile_clips`]), drawn as the entry's own
/// peek is.
///
/// Two peeks rather than one spanning both monitors: each carries the
/// backdrop of its own monitor, and nothing is drawn between them or on a
/// third. It never takes over or hands back the real window, and goes as
/// the motion lands, on the entry's own peek's monitor.
struct Trail {
  peek: WindowPeek,
  clip: Rect,
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
/// over [`STRETCH_SETTLE`] rather than stepping, which was the old path's
/// zoom.
#[derive(Clone, Debug)]
struct Stretch {
  /// The real window's rect, as last read.
  actual: Option<Rect>,
  /// Whether the window has been seen at its target's size, after which
  /// it is sampled whole: before, no larger than the target, as a resize
  /// still on its way there may land before the frame is composited.
  reached: bool,
  /// The cover it was last drawn with, per axis.
  cover: (f32, f32),
  /// When its cover began to ease back to 1, and from what.
  settle: Option<(Instant, (f32, f32))>,
  /// Whether it is drawn filled rather than scaled (see [`PeekResize`]).
  filled: bool,
  /// Whether, filled, its window is sent its new size only as it lands
  /// (see [`resizes_at_start`]). Sampled as it is meanwhile.
  deferred: bool,
  /// The window's edge color, as last sampled, which it is filled with
  /// (see [`note_fill_inputs`]).
  edge_color: Option<Color>,
  /// The smallest size, per axis, the window was sent by a grab and may
  /// not have reported yet (see `WindowPlacer::unlanded`).
  in_flight: Option<(i32, i32)>,
}

impl Stretch {
  /// One for a window whose rect is `actual`, if it could be read, drawn
  /// as `resize` says, its new size sent only as it lands if `deferred`.
  fn at(actual: Option<Rect>, resize: PeekResize, deferred: bool) -> Self {
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

  fn resize(&self) -> PeekResize {
    if self.filled {
      PeekResize::Fill
    } else {
      PeekResize::Stretch
    }
  }

  /// Notes at `now` whether the window has reached the size of `target`,
  /// from when its cover settles.
  fn note_target(&mut self, target: &Rect, now: Instant) {
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
  fn cover_at(&self, now: Instant) -> (f32, f32) {
    self.settle.map_or((1.0, 1.0), |(since, from)| {
      let elapsed = now.saturating_duration_since(since);
      (settling(from.0, elapsed), settling(from.1, elapsed))
    })
  }

  /// Whether its scale has come to rest at `now`, so it can land. One
  /// short of its target isn't waited for.
  fn is_settled(&self, now: Instant) -> bool {
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

/// A leaving window as it was when it started to leave.
struct Departing {
  native: NativeWindow,
  effect_set: EffectSet,
  /// The window's rect with its shadows and its visible frame, read
  /// before it went: a minimized window's are its taskbar button's.
  bounds: Rect,
  frame: Rect,
  /// Whether its real window, still shown under the warmed-up peek, is
  /// yet to be cloaked as the peek takes over (see [`PeekView::leave`]).
  awaits_cloak: bool,
}

/// `peek`, cut at `clip`, and the peek of `trail` with its clip.
fn peeks_of<'a>(
  peek: &'a mut WindowPeek,
  clip: &'a Rect,
  trail: Option<&'a mut Trail>,
) -> impl Iterator<Item = (&'a mut WindowPeek, &'a Rect)> {
  std::iter::once((peek, clip))
    .chain(trail.map(|trail| (&mut trail.peek, &trail.clip)))
}

impl Entry {
  /// Its peeks with the clips they are cut at: its own, then its trail's.
  fn peeks_mut(
    &mut self,
  ) -> impl Iterator<Item = (&mut WindowPeek, &Rect)> {
    peeks_of(&mut self.peek, &self.clip, self.trail.as_mut())
  }

  /// Draws its window's ring along with it on each of its peeks (see
  /// `WindowPeek::carry_ring`). Returns whether its own peek does.
  fn carry_ring(
    &mut self,
    source: HWND,
    ring_rect: &Rect,
    window_rect: &Rect,
    corner_reach: Option<i32>,
  ) -> bool {
    if let Some(trail) = &mut self.trail {
      trail
        .peek
        .carry_ring(source, ring_rect, window_rect, corner_reach);
    }
    self
      .peek
      .carry_ring(source, ring_rect, window_rect, corner_reach)
  }

  /// Places the peek with its window at `rect`, zoomed, slid and faded by
  /// its transit at `now` if it has one, or scaled as it is if it
  /// stretches.
  fn draw(
    &mut self,
    rect: &Rect,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) {
    if self.transit.is_none() && self.draw_stretched(rect, now, batch) {
      self.rect = rect.clone();
      return;
    }

    let frame = self.transit.as_ref().map(|transit| {
      transit_frame(
        rect,
        &transit.style,
        transit.away_opacity,
        transit.amount_at(now),
      )
    });
    let opacity = self.opacity;

    for (peek, clip) in self.peeks_mut() {
      match &frame {
        None => peek.place(rect, clip, opacity, batch),
        Some(Some((frame, frac))) => {
          peek.place_scaled(rect, frame, clip, opacity, *frac, batch);
        }
        Some(None) => peek.hide(),
      }
    }
    self.rect = rect.clone();
  }

  /// Whether a moving entry may land at `now`: its motion and transit
  /// done, and a stretched window's scale at rest. A filled window sent
  /// its size first waits for it, once (see [`FILL_LANDING_WAIT`]), and a
  /// held one for its grab to let go of it (see [`Hold`]).
  fn may_land(&mut self, now: Instant) -> bool {
    let stretch = self.stretch.as_ref();
    let is_done = stretch.is_none_or(|stretch| stretch.is_settled(now))
      && has_landed(&self.motion, self.transit.as_ref());
    let reached = stretch.is_some_and(|stretch| stretch.reached);
    let awaits_size = stretch.is_some_and(|stretch| {
      stretch.filled && !stretch.deferred && !stretch.reached
    });

    if is_done && awaits_size && self.hold == Hold::Free {
      self.hold = Hold::Released(now, FILL_LANDING_WAIT);
    }
    if !is_done || !self.hold.allows_landing(reached, now) {
      return false;
    }

    self.hold = Hold::Free;
    true
  }

  /// Draws a stretching window at `rect` as it is at `now` (see
  /// [`Stretch`]). Returns `false`, drawing nothing, for one to be placed
  /// as it is: one that doesn't stretch or couldn't be read.
  fn draw_stretched(
    &mut self,
    rect: &Rect,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> bool {
    let Some(stretch) = &mut self.stretch else {
      return false;
    };
    let Some(actual) = &stretch.actual else {
      return false;
    };

    let peeks = peeks_of(&mut self.peek, &self.clip, self.trail.as_mut());

    if stretch.filled || self.hold != Hold::Free {
      // Sent its target and yet to reach it, the window may be resized to
      // it at any time, so is sampled no larger, nor larger than any size
      // a grab sent it and it may apply first.
      let target = &self.motion.target_rect;
      let in_flight = [
        (!stretch.reached && !stretch.deferred)
          .then(|| (target.width(), target.height())),
        stretch.in_flight,
      ];
      let sampled =
        sampled_size(actual, rect, in_flight.into_iter().flatten());
      for (peek, clip) in peeks {
        if let Some(color) = stretch.edge_color {
          peek.carry_fill(color);
        }
        peek.place_filled(sampled, rect, clip, self.opacity, batch);
      }
      return true;
    }

    // The cover doesn't depend on the clip: the same on every peek.
    let mut drawn = None;
    for (peek, clip) in peeks {
      let cover = peek.place_stretched(
        actual,
        &self.motion.target_rect,
        rect,
        !stretch.reached,
        stretch.cover_at(now),
        clip,
        self.opacity,
        batch,
      );
      drawn.get_or_insert(cover);
    }
    if let Some(cover) = drawn {
      stretch.cover = cover;
    }
    true
  }
}

/// The size of a window, drawn filled at `rect`, that is sure to be drawn
/// (see `WindowPeek::place_filled`): per axis, the smallest of its size as
/// last read, `actual`, the rect's, and those of sizes it was sent and is
/// yet to report, `in_flight`.
fn sampled_size(
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
pub(super) fn note_fill_inputs(state: &mut WmState) {
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
fn is_same_size(a: &Rect, b: &Rect) -> bool {
  a.width() == b.width() && a.height() == b.height()
}

/// The stretch an entry keeps as it is moved on at `now`, from heading
/// for `from` to `target`: none without a `resize`, else its own,
/// retargeted if its size changes (see [`Stretch::retarget`]), or a new
/// one for a window at `actual`, its new size sent as it lands if
/// `deferred`.
fn restretched(
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

/// Whether a moving entry has landed: its motion and, while it enters,
/// its transit too.
fn has_landed(
  motion: &WindowAnimationState,
  transit: Option<&Transit>,
) -> bool {
  motion.is_complete() && transit.is_none_or(Transit::is_complete)
}

/// Whether [`PeekView::take_departed`] takes an entry whose window
/// `is_known` to the displayed scrolling layouts or not: only a column's,
/// so one entering a tiling layout stays.
fn is_departed(is_column: bool, is_known: bool) -> bool {
  is_column && !is_known
}

/// Whether [`PeekView::take_strays`] takes an entry: one entering a
/// tiling layout, whose window is unmanaged (`shown` is `None`) or not
/// shown. A leaving window is neither shown nor a stray.
fn is_stray(is_column: bool, role: Role, shown: Option<bool>) -> bool {
  !is_column && !matches!(role, Role::Leaving(_)) && shown != Some(true)
}

/// Whether an entry in `role` has its window leaving by `transition`.
fn leaves_as(role: Role, transition: AwayTransition) -> bool {
  role == Role::Leaving(transition)
}

/// Whether a leaving entry is done: its transit has ended (its motion is
/// held where it was).
fn has_left(
  role: Role,
  motion: &WindowAnimationState,
  transit: Option<&Transit>,
) -> bool {
  matches!(role, Role::Leaving(_)) && has_landed(motion, transit)
}

/// Takes the entries `is_done` says have left out of `entries`, so each
/// is reported once.
fn take_left<E>(
  entries: &mut HashMap<Uuid, E>,
  is_done: impl Fn(&E) -> bool,
) -> Vec<(Uuid, E)> {
  entries.extract_if(|_, entry| is_done(entry)).collect()
}

/// The transit a window leaves by: an entering one's turned round where
/// it is at `now` and started then, so it doesn't jump
/// (`ANIMATION_REVIEW.md` 14), else `fresh`, from all the way in.
fn leaving_transit(
  entering: Option<&Transit>,
  fresh: Transit,
  params: &WindowTransitionParams,
  now: Instant,
) -> Transit {
  match entering.filter(|transit| {
    transit.direction == Direction::In && !transit.is_complete()
  }) {
    Some(entering) => {
      let reversed = entering.reversed(params, now);
      reversed.clock.start_clock(now);
      reversed
    }
    None => fresh,
  }
}

/// The transit a window still leaving comes back by, turned round where
/// it is at `now` and started then: it grows back from where it had
/// shrunk to, rather than starting over.
fn returning_transit(
  leaving: &Transit,
  params: &WindowTransitionParams,
  now: Instant,
) -> Transit {
  let returning = leaving.reversed(params, now);
  returning.clock.start_clock(now);
  returning
}

/// The transit an entry keeps as [`PeekView::upsert`] moves it on or
/// puts it at rest: an unfinished one, while it moves. Nothing ticks a
/// transit at rest, where the peek shows its window whole.
fn kept_transit(
  transit: Option<Transit>,
  is_moving: bool,
) -> Option<Transit> {
  transit.filter(|transit| is_moving && !transit.is_complete())
}

/// Whether an entry is a window moving in a tiling layout, entering or
/// not, which `platform_sync` retargets to its tile: not a column, which
/// [`PeekView::upsert`] retargets.
fn follows_tile(is_column: bool, is_moving: bool) -> bool {
  !is_column && is_moving
}

/// The clips of the peeks a tile moving from `from` onto the monitor at
/// `clip` is drawn on: `clip`, and, if it comes from another monitor,
/// `source`, that monitor's, for its trail (see [`Trail`]), if `crosses`.
/// `None` if it can't be drawn so: the peek would cut it at `clip`'s edge.
fn move_clips(
  from: &Rect,
  clip: Rect,
  source: Option<Rect>,
  crosses: bool,
) -> Option<(Rect, Option<Rect>)> {
  let center = from.center_point();
  if clip.contains_point(&center) {
    return Some((clip, None));
  }

  source
    .filter(|source| {
      crosses && *source != clip && source.contains_point(&center)
    })
    .map(|source| (clip, Some(source)))
}

/// Whether a tile drawn on a peek cut at `clip` and a trail cut at
/// `trail_clip`, headed for `target`, lands on its trail's monitor.
fn lands_on_trail(target: &Rect, clip: &Rect, trail_clip: &Rect) -> bool {
  let center = target.center_point();
  !clip.contains_point(&center) && trail_clip.contains_point(&center)
}

/// How long a window drawn moving or entering by its peek takes to follow
/// a new rect: `duration_ms`, unless it is floating and stays so, whose
/// moves the old path doesn't animate. That one jumps there; an entry it
/// plays plays on.
pub fn follow_duration(
  is_floating: bool,
  is_state_change: bool,
  duration_ms: u32,
) -> u32 {
  if is_floating && !is_state_change {
    0
  } else {
    duration_ms
  }
}

/// Where the peek of a floating window opening on it goes once its window
/// is cloaked (see [`floating_peek_raise`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeekRaise {
  /// The top of the non-topmost windows.
  Top,
  /// The top of the always-on-top band, with the floating windows shown
  /// on top.
  TopMost,
}

/// Where the peek of a floating window opening on it is raised, if at
/// all: the top of the band the window lands in. One `shown_on_top` lands
/// at the top of the always-on-top band (the hand-back's `HWND_TOPMOST`),
/// focused or not; another is raised only if focused, over the floating
/// windows restacked below it, or stays just below its window.
pub fn floating_peek_raise(
  shown_on_top: bool,
  is_focused: bool,
) -> Option<PeekRaise> {
  if shown_on_top {
    Some(PeekRaise::TopMost)
  } else {
    is_focused.then_some(PeekRaise::Top)
  }
}

/// Whether `window`, whose state changed this pass, moves on a pinned
/// peek as a tile does (`morepeeks`, `STAND_INS_PLAN.md` 10f): between
/// floating and tiling, not while the OS still drags it.
pub fn changes_state_on_peek(window: &WindowContainer) -> bool {
  wm_platform::isolation::current().more_peeks
    && window.active_drag().is_none()
    && crosses_floating(&window.state(), window.prev_state().as_ref())
}

/// Whether a window now in `state`, its state changed this pass, came
/// there across floating and tiling. `prev_state` is unreliable for a
/// window dropped after a drag (the state before the drag), so it only
/// rules out a window coming back from fullscreen or minimized.
fn crosses_floating(
  state: &WindowState,
  prev_state: Option<&WindowState>,
) -> bool {
  matches!(state, WindowState::Tiling | WindowState::Floating(_))
    && !matches!(
      prev_state,
      Some(WindowState::Fullscreen(_) | WindowState::Minimized)
    )
}

/// How a window crossing between floating and tiling on its peek draws a
/// change of size: in `window_resize`'s style, as the old path does.
pub fn state_change_resize(
  resize: &WindowResizeConfig,
) -> Option<PeekResize> {
  resize.enabled.then_some(match resize.style {
    ResizeStyle::Stretch => PeekResize::Stretch,
    ResizeStyle::Fill => PeekResize::Fill,
  })
}

/// Where the peek of a window crossing between floating and tiling goes
/// once its window is cloaked: one landing floating as one opening does
/// (see [`floating_peek_raise`]), a tile to the top of the other windows,
/// out of the always-on-top band.
pub fn state_change_peek_raise(
  lands_floating: bool,
  shown_on_top: bool,
  is_focused: bool,
) -> Option<PeekRaise> {
  if lands_floating {
    floating_peek_raise(shown_on_top, is_focused)
  } else {
    Some(PeekRaise::Top)
  }
}

/// Whether a window laid out at `target` after `from` only moves, and by
/// more than `threshold_px` as `should_start_new_animation` measures it:
/// below that, the old path doesn't animate it either.
pub(super) fn is_pure_move(
  from: &Rect,
  target: &Rect,
  threshold_px: u32,
) -> bool {
  let threshold = i32::try_from(threshold_px).unwrap_or(i32::MAX);
  let travel =
    (from.x() - target.x()).abs() + (from.y() - target.y()).abs();

  from.width() == target.width()
    && from.height() == target.height()
    && travel > threshold
}

/// Whether a window laid out at `target` after `from` changes size, by
/// more than `threshold_px` as `should_start_new_animation` measures a
/// resize: below that, the old path doesn't animate it either.
pub(super) fn is_peek_resize(
  from: &Rect,
  target: &Rect,
  threshold_px: u32,
) -> bool {
  let threshold = i32::try_from(threshold_px).unwrap_or(i32::MAX);
  let distance = (from.x() - target.x()).abs()
    + (from.y() - target.y()).abs()
    + (from.width() - target.width()).abs()
    + (from.height() - target.height()).abs();

  !is_same_size(from, target) && distance > threshold
}

/// How `window_resize` draws resizes on peeks, if it does: in its style,
/// enabled, with peeks on.
pub(super) fn resizes_on_peeks(config: &UserConfig) -> Option<PeekResize> {
  let resize = &config.value.animations.window_resize;
  if !wm_platform::isolation::current().peeks || !resize.enabled {
    return None;
  }

  Some(match resize.style {
    ResizeStyle::Stretch => PeekResize::Stretch,
    ResizeStyle::Fill => PeekResize::Fill,
  })
}

/// `motion` headed for `target` instead, at the speed it has at `now`
/// (see `WindowAnimationState::retargeted`), so a retarget mid-way glides
/// on rather than restarting. One already drawn carries on from where it
/// was at `now`: sampled at the wall clock and started on the next tick,
/// it repeated or skipped up to a frame of travel at every key press.
fn retargeted_motion(
  motion: &WindowAnimationState,
  target: Rect,
  duration_ms: u32,
  easing: EasingFunction,
  now: Instant,
) -> WindowAnimationState {
  let retargeted = motion.retargeted(target, duration_ms, easing, now);
  if motion.is_running() {
    retargeted.start_clock(now);
  }
  retargeted
}

/// Whether a peek with copies of `styles` has to move with its window: a
/// backdrop that can't be copied along in either focus state.
pub(super) fn is_live(styles: &[Option<BackdropOverlayParams>]) -> bool {
  styles
    .iter()
    .any(|style| style.is_some_and(|params| !params.is_copyable()))
}

/// Whether a window whose peek would carry copies of `styles` can move and
/// resize on it: on a pinned peek, or on a `Live` one under
/// `GLAZEWM_ISOLATE=morepeeks` (`STAND_INS_PLAN.md` 10f).
pub(super) fn can_move_on_peek(
  styles: &[Option<BackdropOverlayParams>],
) -> bool {
  allows_motion(
    is_live(styles),
    wm_platform::isolation::current().more_peeks,
  )
}

/// [`can_move_on_peek`] for a peek that is `live` or not.
fn allows_motion(live: bool, more_peeks: bool) -> bool {
  !live || more_peeks
}

/// Columns whose move ended on a tick.
#[derive(Default)]
pub struct Landed {
  /// Shown at rest, whose real windows are to be uncloaked by the next
  /// `platform_sync`.
  pub shown: Vec<Uuid>,
  /// Hidden at rest, whose real windows are to be put where they stay
  /// cloaked by the next `platform_sync`.
  pub hidden: Vec<Uuid>,
  /// Shown at rest, whose peeks were dropped now that their real window
  /// is on screen; their overlays go back to the real window.
  pub released: Vec<Uuid>,
  /// Done leaving, whose peeks were dropped (see [`finish_leaving`]).
  pub left: Vec<Left>,
  /// Landed after a resize drawn filled, whose edge color is to be
  /// sampled again, as after a `fill` session.
  pub filled: Vec<Uuid>,
}

/// A window whose peek has finished taking it off the screen.
pub struct Left {
  pub id: Uuid,
  pub transition: AwayTransition,
  pub window: NativeWindow,
}

/// How [`PeekView::leave`] takes a window off the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leave {
  /// On its peek, until its transit ends.
  Leaving,
  /// Not at all, so it goes at once: a transition with nothing to show,
  /// or a window DWM can't draw (e.g. an elevated one).
  Skipped,
  /// Not on a peek: a live one, or a peek already drawing the window in
  /// a role that doesn't turn round (e.g. a partly visible column's).
  Unsupported,
}

/// Peeks of the columns of displayed scrolling layouts.
#[derive(Default)]
pub struct PeekView {
  pub(super) entries: HashMap<Uuid, Entry>,
  /// Last target rect of every window in a displayed scrolling layout,
  /// i.e. where it is shown now and moves from on the next relayout.
  pub(super) last_rects: HashMap<Uuid, Rect>,
  /// Windows already reported by `report_stuck_windows`.
  pub(super) reported_stuck: HashSet<Uuid>,
}

impl PeekView {
  /// Whether `window_id`'s real window is cloaked while a peek draws it.
  pub fn owns_window(&self, window_id: &Uuid) -> bool {
    self.owned_target(window_id).is_some()
  }

  /// Whether a peek draws `window_id`, moving or at rest, along with its
  /// overlays.
  pub fn draws_window(&self, window_id: &Uuid) -> bool {
    self.entries.contains_key(window_id)
  }

  /// The peek drawing `window_id`, which its overlays sit behind. `None`
  /// while it only covers its real window on the way out, as they then
  /// sit behind the real window.
  pub fn peek_hwnd(&self, window_id: &Uuid) -> Option<HWND> {
    self.stands_in(window_id).map(|entry| entry.peek.hwnd())
  }

  /// Where the peek drawing `window_id` has `rect`, a rect of the layout
  /// that moves with the window, e.g. its stack's.
  pub fn stand_in(
    &self,
    window_id: &Uuid,
    rect: &Rect,
  ) -> Option<StandIn> {
    let entry = self.stands_in(window_id)?;

    Some(StandIn {
      rect: moved_along(rect, &entry.motion.target_rect, &entry.rect),
      clip: Some(entry.clip.clone()),
      anchor: Some(entry.peek.hwnd()),
    })
  }

  /// Where `window_id`'s peek is headed while its real window is cloaked.
  pub(super) fn owned_target(&self, window_id: &Uuid) -> Option<&Rect> {
    self
      .entries
      .get(window_id)
      .filter(|entry| {
        matches!(entry.role, Role::Moving { .. } | Role::HandingBack)
      })
      .map(|entry| &entry.motion.target_rect)
  }

  /// The entry of a peek standing in for `window_id`'s real window, i.e.
  /// not one on its way out.
  fn stands_in(&self, window_id: &Uuid) -> Option<&Entry> {
    self
      .entries
      .get(window_id)
      .filter(|entry| !matches!(entry.role, Role::Releasing { .. }))
  }

  /// Whether `window_id`'s peek still covers its uncloaked real window.
  pub(super) fn is_releasing(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| matches!(entry.role, Role::Releasing { .. }))
  }

  /// Whether `window_id`'s landed peek is about to hand its real window
  /// back, as that is uncloaked.
  pub fn is_handing_back(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| entry.role == Role::HandingBack)
  }

  /// How long until `window_id`'s column lands at `now`, while it moves.
  pub fn time_to_land(
    &self,
    window_id: &Uuid,
    now: Instant,
  ) -> Option<Duration> {
    self
      .entries
      .get(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
      .map(|entry| {
        let motion = entry.motion.remaining_at(now);
        entry.transit.as_ref().map_or(motion, |transit| {
          motion.max(transit.clock.remaining_at(now))
        })
      })
  }

  /// Whether `window_id` is moving, i.e. its real window stays frozen.
  pub fn is_moving(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| matches!(entry.role, Role::Moving { .. }))
  }

  /// Whether `window_id` is moving and its peek has been composited since
  /// it went up over the still-shown real window, so that can be cloaked
  /// at once. True once `sync_scroll_view`'s flush, or a tick, has passed.
  pub fn is_composited(&self, window_id: &Uuid) -> bool {
    self.entries.get(window_id).is_some_and(|entry| {
      matches!(entry.role, Role::Moving { .. }) && entry.composited
    })
  }

  /// Holds `id`'s peek where a grab draws it, never landing, or lets go
  /// of it, after which it lands as [`Hold::Released`] says. Letting go of
  /// one not held changes nothing.
  pub(crate) fn hold(&mut self, id: Uuid, is_held: bool) {
    let Some(entry) = self.entries.get_mut(&id) else {
      return;
    };

    entry.hold = match (is_held, entry.hold) {
      (true, _) => Hold::Held,
      (false, Hold::Held) => {
        Hold::Released(Instant::now(), HOLD_RELEASE_WAIT)
      }
      (false, hold) => hold,
    };
    // Its window is sent each size the grab gives it.
    if let Some(stretch) = entry.stretch.as_mut().filter(|_| is_held) {
      stretch.deferred = false;
    }
  }

  /// Whether `id`'s window, filled on its peek, is to be sent its new
  /// size only as its peek lands (see [`resizes_at_start`]).
  pub fn defers_resize(&self, id: &Uuid) -> bool {
    self
      .entries
      .get(id)
      .and_then(|entry| entry.stretch.as_ref())
      .is_some_and(|stretch| stretch.deferred)
  }

  /// Lets go of every peek a grab holds (see [`hold`](Self::hold)).
  pub(crate) fn release_holds(&mut self) {
    let now = Instant::now();
    for entry in self.entries.values_mut() {
      if entry.hold == Hold::Held {
        entry.hold = Hold::Released(now, HOLD_RELEASE_WAIT);
      }
    }
  }

  /// Whether a grab holds `id`'s peek.
  pub(crate) fn is_held(&self, id: &Uuid) -> bool {
    self
      .entries
      .get(id)
      .is_some_and(|entry| entry.hold == Hold::Held)
  }

  /// Whether anything still needs animation ticks.
  pub fn is_active(&self) -> bool {
    self.entries.values().any(|entry| entry.role != Role::Peek)
  }

  /// Advances moving columns to `now`, returning the ones that landed.
  /// Their moves are queued into `batch`, for [`track_overlays`] to commit
  /// along with their overlays'.
  pub fn tick(
    &mut self,
    now: Instant,
    clock_origin: Instant,
    batch: &mut SurrogateBatch,
  ) -> Landed {
    let mut landed = Landed::default();

    // A later composition than the one copies were put over rings and
    // backdrops in.
    self.note_composited();

    for (id, entry) in &mut self.entries {
      match entry.role {
        Role::Moving { ends_shown } => {
          // Its real window was cloaked a tick ago at least, so needs its
          // alpha no more. One not taken over from, e.g. as it wasn't
          // redrawn, is drawn all the same.
          for (peek, _) in entry.peeks_mut() {
            peek.restore_alpha();
            peek.show_window();
            peek.refresh_companion();
          }
          entry.motion.start_clock(clock_origin);
          if let Some(transit) = &entry.transit {
            transit.clock.start_clock(clock_origin);
          }
          if let Some(stretch) = &mut entry.stretch {
            stretch.note_target(&entry.motion.target_rect, now);
          }
          let (rect, _) = entry.motion.current_state_at(now);
          entry.draw(&rect, now, batch);

          if entry.may_land(now) {
            // Landed on its own peek's monitor, it has nothing left on
            // another.
            entry.trail = None;
            if entry.stretch.as_ref().is_some_and(|stretch| stretch.filled)
            {
              landed.filled.push(*id);
            }
            entry.role = if ends_shown {
              // Raised while its real window is still cloaked, so the
              // uncloak never shows the window over the peek.
              entry.peek.begin_hand_back();
              landed.shown.push(*id);
              Role::HandingBack
            } else {
              landed.hidden.push(*id);
              Role::Peek
            };
          }
        }
        Role::Releasing { ref mut settled } => {
          if !entry.peek.awaits_companion() {
            if *settled {
              landed.released.push(*id);
            } else {
              entry.peek.hand_back();
              *settled = true;
            }
          }
        }
        Role::Leaving(_) => {
          // Cloaked as its peek takes over, a tick after the peek went up
          // under it; given its alpha back a tick later, as a moving one.
          match entry.departing.as_mut() {
            Some(departing) if departing.awaits_cloak => {
              entry.peek.take_over(&departing.native);
              if let Err(err) = departing.native.set_cloaked(true) {
                tracing::warn!("Failed to cloak leaving {id}: {err}");
              }
              departing.awaits_cloak = false;
            }
            _ => entry.peek.restore_alpha(),
          }
          for (peek, _) in entry.peeks_mut() {
            peek.show_window();
            peek.refresh_companion();
          }
          if let Some(transit) = &entry.transit {
            transit.clock.start_clock(clock_origin);
          }
          let (rect, _) = entry.motion.current_state_at(now);
          entry.draw(&rect, now, batch);
        }
        Role::Peek | Role::HandingBack => {}
      }
    }

    for id in &landed.released {
      self.entries.remove(id);
    }

    landed.left = take_left(&mut self.entries, |entry| {
      has_left(entry.role, &entry.motion, entry.transit.as_ref())
    })
    .into_iter()
    .filter_map(|(id, entry)| {
      let Role::Leaving(transition) = entry.role else {
        return None;
      };
      entry.departing.map(|departing| Left {
        id,
        transition,
        window: departing.native,
      })
    })
    .collect();

    // Columns that moved out of view have nothing left to show.
    self.entries.retain(|_, entry| {
      entry.role != Role::Peek || entry.peek.visible_rect().is_some()
    });

    landed
  }

  /// Notes that the copies put over real rings and backdrops have been
  /// composited since, so those can go (see [`Handover::Covered`]).
  pub(super) fn note_composited(&mut self) {
    for entry in self.entries.values_mut() {
      entry.composited = true;
      for handover in [&mut entry.ring, &mut entry.backdrop] {
        if *handover == Handover::Covering {
          *handover = Handover::Covered;
        }
      }
    }
  }

  /// Takes out the peeks of columns that `is_known` no longer knows, i.e.
  /// that left the displayed scrolling layouts, and forgets their rects.
  pub(super) fn take_departed(
    &mut self,
    is_known: impl Fn(&Uuid) -> bool,
  ) -> Vec<(Uuid, Entry)> {
    self.last_rects.retain(|id, _| is_known(id));
    self
      .entries
      .extract_if(|id, entry| is_departed(entry.is_column, is_known(id)))
      .collect()
  }

  /// Takes out the peeks of windows entering a tiling layout that
  /// `shown` no longer has on screen: unmanaged (`None`), or not shown
  /// (`Some(false)`). Columns are left to
  /// [`take_departed`](Self::take_departed).
  pub(super) fn take_strays(
    &mut self,
    shown: impl Fn(&Uuid) -> Option<bool>,
  ) -> Vec<Uuid> {
    self
      .entries
      .extract_if(|id, entry| {
        is_stray(entry.is_column, entry.role, shown(id))
      })
      .map(|(id, _)| id)
      .collect()
  }

  /// Drops `window_id`'s peek, if it is entering a tiling layout, as its
  /// window is unmanaged.
  pub fn forget_entering(&mut self, window_id: &Uuid) {
    if self.entries.get(window_id).is_some_and(|entry| {
      !entry.is_column && !matches!(entry.role, Role::Leaving(_))
    }) {
      self.entries.remove(window_id);
    }
  }

  /// Whether any peek draws a window entering a tiling layout.
  fn has_entering(&self) -> bool {
    self.entries.values().any(|entry| {
      !entry.is_column && !matches!(entry.role, Role::Leaving(_))
    })
  }

  /// Whether `window_id`'s peek is taking it off the screen (see
  /// [`leave`](Self::leave)).
  pub fn is_leaving(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| matches!(entry.role, Role::Leaving(_)))
  }

  /// Whether `window_id`'s peek is taking it off the screen by
  /// `transition`.
  pub(crate) fn leaves_by(
    &self,
    window_id: &Uuid,
    transition: AwayTransition,
  ) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| leaves_as(entry.role, transition))
  }

  /// Drops `window_id`'s peek if it is leaving, ending its animation with
  /// nothing more done: no `WM_CLOSE`, and its real window left as it is.
  pub fn forget_leaving(&mut self, window_id: &Uuid) {
    if self.is_leaving(window_id) {
      self.entries.remove(window_id);
    }
  }

  /// Drops every leaving peek, as [`forget_leaving`](Self::forget_leaving)
  /// does.
  pub fn forget_all_leaving(&mut self) {
    self
      .entries
      .retain(|_, entry| !matches!(entry.role, Role::Leaving(_)));
  }

  /// Has `window_id`'s moving peek take over from its real window, which
  /// is about to be cloaked (see `WindowPeek::take_over`).
  pub fn take_over(&mut self, window_id: &Uuid, window: &NativeWindow) {
    if let Some(entry) = self
      .entries
      .get_mut(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
    {
      // Drawn before the real window's alpha goes, as its own peek is.
      if let Some(trail) = &mut entry.trail {
        trail.peek.show_window();
      }
      entry.peek.take_over(window);
    }
  }

  /// Readies `window_id`'s landed peek to hand its real window back, as
  /// that is about to be uncloaked (see `WindowPeek::prepare_hand_back`).
  pub fn prepare_hand_back(
    &mut self,
    window_id: &Uuid,
    window: &NativeWindow,
  ) {
    if let Some(entry) = self
      .entries
      .get_mut(window_id)
      .filter(|entry| entry.role == Role::HandingBack)
    {
      entry.peek.prepare_hand_back(window);
    }
  }

  /// Hands `window_id`'s real window, just uncloaked, back from its peek,
  /// which goes once that is surely on screen (see [`Role::Releasing`]).
  pub fn release(&mut self, window_id: &Uuid) {
    if let Some(entry) = self
      .entries
      .get_mut(window_id)
      .filter(|entry| entry.role == Role::HandingBack)
    {
      entry.role = Role::Releasing { settled: false };
      if !entry.peek.awaits_companion() {
        entry.peek.hand_back();
      }
    }
  }

  /// The window whose peek at rest is at `point`, if any. Moving ones
  /// are clicked through `ClickGuard`'s motion guard instead (see
  /// [`Self::moving_rects`]).
  pub fn window_at(&self, point: &Point) -> Option<Uuid> {
    self.entries.iter().find_map(|(id, entry)| {
      Some(entry)
        .filter(|entry| entry.role == Role::Peek)?
        .peek
        .visible_rect()
        .filter(|rect| rect.contains_point(point))
        .map(|_| *id)
    })
  }

  /// On-screen rects of the peeks drawing moving windows, whose real
  /// windows are cloaked meanwhile, so a click there reaches nothing.
  pub fn moving_rects(&self) -> impl Iterator<Item = (Uuid, &Rect)> {
    self
      .entries
      .iter()
      .filter(|(_, entry)| {
        matches!(entry.role, Role::Moving { .. } | Role::HandingBack)
      })
      .flat_map(|(id, entry)| {
        std::iter::once(&entry.peek)
          .chain(entry.trail.as_ref().map(|trail| &trail.peek))
          .filter_map(|peek| peek.visible_rect().map(|rect| (*id, rect)))
      })
  }

  /// Draws `window` with a peek in `role`, going from `from` to `target`.
  /// A move of a peek already shown is queued into `batch`, and one
  /// already moving carries on from where it is at `now`, the time frames
  /// are evaluated at.
  ///
  /// A peek it creates moving and pinned enters by `opening`, if given
  /// (see [`Transit`]); one already drawn keeps its own.
  ///
  /// Returns whether a new peek now covers a still-visible real window,
  /// so it has to be composited before that window is cloaked.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn upsert(
    &mut self,
    window: &WindowContainer,
    from: &Rect,
    target: Rect,
    clip: &Rect,
    role: Role,
    opening: Option<Transit>,
    resize: Option<PeekResize>,
    held: bool,
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> bool {
    let scroll_config = &config.value.animations.scroll;
    let is_moving = matches!(role, Role::Moving { .. });
    // Held by a grab, it jumps to where each step puts it, drawn filled
    // (see [`Hold`]).
    let duration_ms = if held { 0 } else { scroll_config.duration_ms };
    let resize = if held { Some(PeekResize::Fill) } else { resize };
    let in_flight = self
      .entries
      .get(&window.id())
      .filter(|entry| matches!(entry.role, Role::Moving { .. }));
    // Its real window is back on screen, so a new move has to cloak it
    // again behind the peek.
    let covers_shown_window = is_moving && self.is_releasing(&window.id());

    // A column already moving carries on at its speed toward the new
    // target, so quick presses in a row glide rather than restart.
    let motion = match in_flight {
      Some(entry) if is_moving => retargeted_motion(
        &entry.motion,
        target,
        duration_ms,
        scroll_config.easing.clone(),
        now,
      ),
      _ => WindowAnimationState::new_movement(
        from.clone(),
        target,
        if is_moving { duration_ms } else { 0 },
        scroll_config.easing.clone(),
      ),
    };
    // The window's own transparency, corners and backdrop, as it is
    // focused or not.
    let (opacity, corner_style, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    let styles = peek_styles(window, config);
    if !is_moving {
      motion.start_clock(Instant::now());
    }
    // Resized on its peek while its size changes on the way, but not
    // entering: a new column widens from nothing, showing more of itself.
    let resizes = resize.filter(|_| {
      is_moving
        && opening.is_none()
        && !is_same_size(&motion.start_rect, &motion.target_rect)
    });
    let deferred =
      !held && !resizes_at_start(&motion.start_rect, &motion.target_rect);

    let actual = || window.native().frame_with_shadows().ok();

    if let Some(entry) = self.entries.get_mut(&window.id()) {
      let headed = std::mem::replace(&mut entry.motion, motion);
      entry.clip = clip.clone();
      entry.opacity = opacity;
      entry.role = role;
      entry.is_column = true;
      entry.transit = kept_transit(entry.transit.take(), is_moving);
      // One resizing already goes on so while it moves: drawn another
      // way from here, its scale would jump.
      let resizes = entry
        .stretch
        .as_ref()
        .filter(|_| is_moving)
        .map(Stretch::resize)
        .or(resizes)
        .filter(|_| entry.transit.is_none());
      entry.stretch = restretched(
        entry.stretch.take(),
        resizes,
        deferred,
        actual,
        &headed.target_rect,
        &entry.motion.target_rect,
        now,
      );
      // Copies of both styles are drawn up front, so a focus change only
      // crossfades them (see `track_overlays`): one registered mid-move
      // left frames without it.
      entry.peek.carry_styles(&styles);
      if covers_shown_window {
        entry.composited = false;
      }
      entry.hold = if held { Hold::Held } else { Hold::Free };
      let (rect, _) = entry.motion.current_state_at(now);
      entry.draw(&rect, now, batch);
      return covers_shown_window;
    }

    let Some((mut entry, covers_shown_window)) = new_entry(
      window,
      motion,
      clip,
      opacity,
      corner_style,
      &styles,
      role,
    ) else {
      return false;
    };

    // `place_scaled` draws pinned peeks only.
    entry.transit = opening.filter(|_| is_moving && !entry.peek.is_live());
    entry.stretch =
      resizes.map(|resize| Stretch::at(actual(), resize, deferred));
    if held {
      entry.hold = Hold::Held;
    }
    let rect = entry.rect.clone();
    entry.draw(&rect, now, batch);
    self.entries.insert(window.id(), entry);

    covers_shown_window
  }

  /// Draws `window`, just opened or restored at `target`, entering on a
  /// pinned peek by `params`' style, opacity and timing, until it is
  /// handed back.
  ///
  /// Returns `false`, creating nothing, for a window already drawn, one
  /// whose peek would be `Live`, or a transition with nothing to show (see
  /// [`Transit::opening`]).
  #[allow(clippy::too_many_arguments)]
  pub(super) fn enter(
    &mut self,
    window: &WindowContainer,
    target: &Rect,
    clip: &Rect,
    params: &WindowTransitionParams,
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> bool {
    let styles = peek_styles(window, config);
    let Some(transit) =
      Transit::opening(params).filter(|_| self.can_enter(window, &styles))
    else {
      return false;
    };

    let (opacity, corner_style, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    // Held at its tile: only the transit moves it in.
    let motion = WindowAnimationState::new_movement(
      target.clone(),
      target.clone(),
      params.duration_ms,
      params.easing.clone(),
    );

    let Some((mut entry, _)) = new_entry(
      window,
      motion,
      clip,
      opacity,
      corner_style,
      &styles,
      Role::Moving { ends_shown: true },
    ) else {
      return false;
    };

    entry.transit = Some(transit);
    entry.is_column = false;
    entry.draw(target, now, batch);
    self.entries.insert(window.id(), entry);

    true
  }

  /// Whether [`enter`](Self::enter) can draw `window`, whose peek would
  /// carry copies of `styles`: not one already drawn, nor one whose peek
  /// would be `Live`.
  fn can_enter(
    &self,
    window: &WindowContainer,
    styles: &[Option<BackdropOverlayParams>],
  ) -> bool {
    !is_live(styles) && !self.draws_window(&window.id())
  }

  /// Draws `window`, a tile moving from `from` to `target`, on a peek
  /// over `duration_ms` and `easing`, until it is handed back. A peek
  /// still handing it back after an earlier move or entry takes it on
  /// again from where it landed, as [`upsert`](Self::upsert) does for a
  /// column. With `stretch`, a change of size scales the window (see
  /// [`Stretch`]); without, it keeps the same size.
  ///
  /// Returns `false`, creating nothing, for a window already drawn
  /// otherwise (moving, leaving, or as a column), one whose peek can't
  /// move it (see [`can_move_on_peek`]), or one DWM can't draw.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn move_tile(
    &mut self,
    window: &WindowContainer,
    from: &Rect,
    target: Rect,
    clip: &Rect,
    duration_ms: u32,
    easing: EasingFunction,
    resize: Option<PeekResize>,
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> bool {
    let id = window.id();
    let styles = peek_styles(window, config);
    let resizes = resize.filter(|_| !is_same_size(from, &target));
    let deferred = !resizes_at_start(from, &target);
    let landing = match self.entries.get(&id) {
      None => false,
      Some(entry)
        if !entry.is_column
          && allows_motion(
            entry.peek.is_live(),
            wm_platform::isolation::current().more_peeks,
          )
          && matches!(
            entry.role,
            Role::HandingBack | Role::Releasing { .. }
          ) =>
      {
        true
      }
      Some(_) => return false,
    };
    if !can_move_on_peek(&styles) {
      return false;
    }

    let (opacity, corner_style, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    let motion = WindowAnimationState::new_movement(
      from.clone(),
      target,
      duration_ms,
      easing,
    );
    let role = Role::Moving { ends_shown: true };
    let actual = || window.native().frame_with_shadows().ok();

    if landing {
      // A releasing peek's real window is back on screen, so it has to be
      // taken over from and cloaked again.
      let covers_shown_window = self.is_releasing(&id);
      let Some(entry) = self.entries.get_mut(&id) else {
        return false;
      };
      let headed = std::mem::replace(&mut entry.motion, motion);
      entry.clip = clip.clone();
      entry.opacity = opacity;
      entry.role = role;
      entry.transit = None;
      entry.hold = Hold::Free;
      entry.stretch = restretched(
        entry.stretch.take(),
        resizes,
        deferred,
        actual,
        &headed.target_rect,
        &entry.motion.target_rect,
        now,
      );
      entry.peek.carry_styles(&styles);
      if covers_shown_window {
        entry.composited = false;
      }
      entry.draw(from, now, batch);
      return true;
    }

    let Some((mut entry, _)) = new_entry(
      window,
      motion,
      clip,
      opacity,
      corner_style,
      &styles,
      role,
    ) else {
      return false;
    };
    // Nothing to draw it with: the old path snaps it into place.
    if !entry.peek.has_thumbnail() {
      tracing::debug!("No peek of {window} to move on.");
      return false;
    }

    entry.is_column = false;
    entry.stretch =
      resizes.map(|resize| Stretch::at(actual(), resize, deferred));
    entry.draw(from, now, batch);
    self.entries.insert(id, entry);

    true
  }

  /// Has `window`'s peek, just put up by [`move_tile`](Self::move_tile)
  /// on the monitor it moves onto, drawn on `clip`, the monitor it comes
  /// from, too (see [`Trail`]). Returns whether it is: if not, the peek
  /// is dropped, as it would cut the window at its monitor's edge.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_trail(
    &mut self,
    window: &WindowContainer,
    clip: &Rect,
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> bool {
    let id = window.id();
    let Some(entry) = self.entries.get_mut(&id) else {
      return false;
    };

    let styles = peek_styles(window, config);
    let (opacity, corner_style, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    let Some((trail, _)) = new_entry(
      window,
      entry.motion.clone(),
      clip,
      opacity,
      corner_style,
      &styles,
      entry.role,
    ) else {
      self.entries.remove(&id);
      return false;
    };

    entry.trail = Some(Trail {
      peek: trail.peek,
      clip: clip.clone(),
    });
    let rect = entry.rect.clone();
    entry.draw(&rect, now, batch);
    true
  }

  /// Whether `window_id` moves onto another monitor on two peeks (see
  /// [`Trail`]).
  pub fn has_trail(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| entry.trail.is_some())
  }

  /// Turns `window_id`'s peek, taking it off the screen by a minimize,
  /// round where it is at `now`: it comes back in by `params`, held
  /// where it is until retargeted, and is handed back as an entering one
  /// is. Its real window is read from the tree again. Returns whether it
  /// was leaving so.
  pub(super) fn turn_round(
    &mut self,
    window_id: &Uuid,
    params: &WindowTransitionParams,
    is_column: bool,
    now: Instant,
  ) -> bool {
    let Some(entry) = self
      .entries
      .get_mut(window_id)
      .filter(|entry| leaves_as(entry.role, AwayTransition::Minimize))
    else {
      return false;
    };
    let Some(leaving) = &entry.transit else {
      return false;
    };

    let (current, _) = entry.motion.current_state_at(now);
    entry.transit = Some(returning_transit(leaving, params, now));
    entry.motion = WindowAnimationState::new_movement(
      current.clone(),
      current.clone(),
      params.duration_ms,
      params.easing.clone(),
    );
    entry.motion.start_clock(now);
    entry.role = Role::Moving { ends_shown: true };
    entry.is_column = is_column;
    entry.departing = None;
    entry.stretch = None;

    true
  }

  /// Puts `window_id`'s moving peek on top, as the old path's stand-ins
  /// of a move, open or restore are (`place_at_top`). A peek goes up just
  /// below its real window: a restored one was at the bottom of the
  /// z-order, under the windows it comes back over, and a moved one may be
  /// under windows or stand-ins it passes. Not one leaving, which stays
  /// below its neighbours (task 4).
  pub fn raise_moving(&self, window_id: &Uuid) {
    if let Some(entry) = self
      .entries
      .get(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
    {
      entry.peek.raise_to_top();
      if let Some(trail) = &entry.trail {
        trail.peek.raise_to_top();
      }
    }
  }

  /// Puts `window_id`'s moving peek at the top of the band `raise` names
  /// (see [`floating_peek_raise`]).
  pub fn raise_in_band(&self, window_id: &Uuid, raise: PeekRaise) {
    if let Some(entry) = self
      .entries
      .get(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
    {
      entry.peek.raise_to_top_of_band(raise == PeekRaise::TopMost);
    }
  }

  /// Whether `window_id` moves in a tiling layout on its peek, entering
  /// (see [`enter`](Self::enter)) or not (see
  /// [`move_tile`](Self::move_tile)).
  pub fn follows_tile(&self, window_id: &Uuid) -> bool {
    self.entries.get(window_id).is_some_and(|entry| {
      follows_tile(
        entry.is_column,
        matches!(entry.role, Role::Moving { .. }),
      )
    })
  }

  /// Heads `window_id`'s peek, moving in a tiling layout, for its new
  /// tile at `target`, at the speed it has at `now`, over `duration_ms`
  /// and `easing`. A transit it enters by carries on untouched. With
  /// `stretch`, a window that doesn't enter is scaled as its size changes
  /// (see [`Stretch`]).
  ///
  /// Returns whether it was retargeted.
  pub fn retarget_tile(
    &mut self,
    window_id: &Uuid,
    target: Rect,
    duration_ms: u32,
    easing: EasingFunction,
    resize: Option<PeekResize>,
    now: Instant,
  ) -> bool {
    // A grab's steps put a column it holds too.
    if !self.follows_tile(window_id) && !self.is_held(window_id) {
      return false;
    }

    let Some(entry) = self
      .entries
      .get_mut(window_id)
      .filter(|entry| entry.motion.target_rect != target)
    else {
      return false;
    };

    let motion =
      retargeted_motion(&entry.motion, target, duration_ms, easing, now);
    let headed = std::mem::replace(&mut entry.motion, motion);
    // Headed back to the monitor it came from, it lands on its trail's
    // peek, which becomes its own.
    if let Some(trail) = entry.trail.as_mut().filter(|trail| {
      lands_on_trail(&entry.motion.target_rect, &entry.clip, &trail.clip)
    }) {
      std::mem::swap(&mut entry.peek, &mut trail.peek);
      std::mem::swap(&mut entry.clip, &mut trail.clip);
    }
    // One resizing already goes on so: drawn another way from here, its
    // scale would jump.
    let resizes =
      entry.stretch.as_ref().map(Stretch::resize).or_else(|| {
        resize.filter(|_| {
          entry.transit.is_none()
            && !is_same_size(
              &entry.motion.start_rect,
              &entry.motion.target_rect,
            )
        })
      });
    let deferred = entry.hold != Hold::Held
      && !resizes_at_start(&headed.target_rect, &entry.motion.target_rect);
    // A new motion waits anew for its window's size, unless a grab holds
    // it.
    if entry.hold != Hold::Held {
      entry.hold = Hold::Free;
    }
    // One that only moved so far is the size of the tile it headed for.
    entry.stretch = restretched(
      entry.stretch.take(),
      resizes,
      deferred,
      || Some(headed.target_rect.clone()),
      &headed.target_rect,
      &entry.motion.target_rect,
      now,
    );
    true
  }

  /// Draws `window`, at `rect`, leaving by `params`' style, opacity and
  /// timing on a pinned peek, until [`tick`](Self::tick) reports it
  /// [`Landed::left`]. One already entering on its peek turns round where
  /// it is.
  ///
  /// A new peek goes up directly below its real window and is never
  /// raised: a closing window is drawn below its neighbours (owner's
  /// decision, `STAND_INS_PLAN.md` task 4). Under a real window still
  /// shown, it is warmed up until the window is cloaked by the first
  /// tick, as it takes over.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn leave(
    &mut self,
    window: &WindowContainer,
    rect: &Rect,
    clip: &Rect,
    transition: AwayTransition,
    params: &WindowTransitionParams,
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> Leave {
    let id = window.id();
    let role = self.entries.get(&id).map(|entry| entry.role);
    if role.is_some_and(|role| matches!(role, Role::Leaving(_))) {
      return Leave::Leaving;
    }

    let styles = peek_styles(window, config);
    if is_live(&styles)
      || role.is_some_and(|role| !matches!(role, Role::Moving { .. }))
    {
      return Leave::Unsupported;
    }

    let native = window.native().clone();
    let (Ok(bounds), Ok(frame)) =
      (native.frame_with_shadows(), native.frame())
    else {
      return Leave::Unsupported;
    };
    let Some(fresh) = Transit::leaving(params) else {
      return Leave::Skipped;
    };
    let effect_set = window.effect_set(is_focused);

    if role.is_none() {
      let (opacity, corner_style, _) =
        surrogate_effects_for(effect_set, config);
      let Some((entry, _)) = new_entry(
        window,
        held_motion(rect.clone(), params),
        clip,
        opacity,
        corner_style,
        &styles,
        Role::Leaving(transition),
      ) else {
        return Leave::Unsupported;
      };
      // No thumbnail, nothing to play (`5f39127`): dropped here.
      if !entry.peek.has_thumbnail() {
        tracing::debug!("No peek of {window} to leave on.");
        return Leave::Skipped;
      }
      self.entries.insert(id, entry);
    }

    let Some(entry) = self
      .entries
      .get_mut(&id)
      .filter(|entry| entry.peek.has_thumbnail())
    else {
      return Leave::Skipped;
    };

    let (current, _) = entry.motion.current_state_at(now);
    entry.transit =
      Some(leaving_transit(entry.transit.as_ref(), fresh, params, now));
    entry.motion = held_motion(current.clone(), params);
    entry.motion.start_clock(now);
    entry.role = Role::Leaving(transition);
    entry.is_column = false;
    entry.stretch = None;
    entry.departing = Some(Departing {
      awaits_cloak: !native.is_cloaked().unwrap_or(true),
      native,
      effect_set,
      bounds,
      frame,
    });
    entry.draw(&current, now, batch);

    Leave::Leaving
  }

  /// Has `window_id`'s leaving peek, warmed up under its still-shown real
  /// window, take over from it now, as the caller is about to minimize
  /// it (see `WindowPeek::take_over`). Returns whether it was waiting to.
  fn take_over_leaving(&mut self, window_id: &Uuid) -> bool {
    let Some((peek, departing)) =
      self.entries.get_mut(window_id).and_then(|entry| {
        let departing = entry.departing.as_mut()?;
        departing
          .awaits_cloak
          .then_some((&mut entry.peek, departing))
      })
    else {
      return false;
    };

    peek.take_over(&departing.native);
    departing.awaits_cloak = false;
    true
  }

  /// Whether `window_id`'s leaving peek waits to take over from its real
  /// window, still shown.
  fn awaits_take_over(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .and_then(|entry| entry.departing.as_ref())
      .is_some_and(|departing| departing.awaits_cloak)
  }
}

/// A motion holding a window at `rect`, for one leaving: only its transit
/// moves it.
fn held_motion(
  rect: Rect,
  params: &WindowTransitionParams,
) -> WindowAnimationState {
  WindowAnimationState::new_movement(
    rect.clone(),
    rect,
    0,
    params.easing.clone(),
  )
}

/// Drops the peeks of windows entering a tiling layout that are no longer
/// shown: closed, moved to a hidden workspace or switched away, minimized
/// or now a hidden tab. No hand-back: their real windows are queued for
/// redraw, which hides or shows them for good, as `sync_scroll_view` does
/// for departed columns. Free while no window enters.
pub fn drop_strays(state: &mut WmState) {
  if !state.animation_manager.peeks.has_entering() {
    return;
  }

  let shown = |id: &Uuid| {
    let window = state.window_by_id(*id)?;
    let manager = &state.animation_manager;
    Some(
      window
        .workspace()
        .is_some_and(|workspace| workspace.is_displayed())
        && !is_hidden_in_layout(&window)
        && window.state() != WindowState::Minimized
        && !manager.has_close_animation(id)
        && !manager.has_minimize_animation(id),
    )
  };
  let shown = state
    .animation_manager
    .peeks
    .entries
    .keys()
    .map(|id| (*id, shown(id)))
    .collect::<HashMap<_, _>>();

  let strays = state
    .animation_manager
    .peeks
    .take_strays(|id| shown.get(id).copied().flatten());

  for id in strays {
    tracing::debug!("Dropped the peek of {id}, no longer shown.");
    if let Some(window) = state.window_by_id(id) {
      state.pending_sync.queue_container_to_redraw(window);
    }
  }
}

/// Has `window`, just opened at `target`, enter on a pinned peek by
/// `params` (see [`PeekView::enter`]), on ticks paced by its monitor's
/// vblanks as a scroll is. Returns whether it does.
///
/// Clipped to the whole monitor, as the old path's slides are.
pub fn enter_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  target: &Rect,
  params: &WindowTransitionParams,
  config: &UserConfig,
  is_focused: bool,
) -> bool {
  let Some((clip, handle)) = window.monitor().and_then(|monitor| {
    let clip = monitor.to_rect().ok()?;
    Some((clip, monitor.native_properties().handle))
  }) else {
    return false;
  };

  let now = state.animation_manager.frame_time();
  let mut batch = SurrogateBatch::new();
  let entered = state.animation_manager.peeks.enter(
    window, target, &clip, params, config, is_focused, now, &mut batch,
  );
  batch.commit();

  if entered {
    perf::note_animation_start(false);
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  entered
}

/// Has `window`, a tile shown at `from` and laid out at `target`, move
/// there on a peek if it only moves (see [`is_pure_move`]) or, if
/// its relayout `stretches` (see [`stretches_relayout`]), resizes too (see
/// [`is_peek_resize`]), on `window_move`'s timing or, when it resizes or
/// `cycle_has_resize`, `window_resize`'s, as the old path would
/// (`movement_timing`). Returns whether it does.
///
/// Its window is still shown: the redraw that freezes it queues the
/// take-over into the pass's shared flush (`commit_pending_cloaks`), with
/// [`cover_moved_overlays`] before it and [`hand_over_moved_overlays`]
/// after. That flush also sends the window to its target, size and all,
/// behind its cloak.
///
/// Not a window another stand-in draws, which keeps it, nor one moving
/// onto another monitor unless [`tile_clips`] draws it on two peeks.
#[allow(clippy::too_many_arguments)]
pub fn move_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  from: &Rect,
  target: &Rect,
  cycle_has_resize: bool,
  resizes: Option<PeekResize>,
  config: &UserConfig,
  is_focused: bool,
) -> bool {
  let animations = &config.value.animations;
  let is_resize = !is_same_size(from, target);
  let qualifies = if is_resize {
    resizes.is_some()
      && is_peek_resize(
        from,
        target,
        animations.window_resize.threshold_px,
      )
  } else {
    is_pure_move(from, target, animations.window_move.threshold_px)
  };
  if !qualifies
    || has_other_stand_in(&state.animation_manager, &window.id())
  {
    return false;
  }

  let Some(handle) = window
    .monitor()
    .map(|monitor| monitor.native_properties().handle)
  else {
    return false;
  };
  let Some((clip, trail_clip)) = tile_clips(state, window, from, config)
  else {
    return false;
  };

  let motion = if is_resize {
    Motion::Resize
  } else {
    Motion::Move
  };
  let (duration_ms, easing) =
    movement_timing(motion, cycle_has_resize, config);
  let now = state.animation_manager.frame_time();
  let mut batch = SurrogateBatch::new();
  let peeks = &mut state.animation_manager.peeks;
  let moved = peeks.move_tile(
    window,
    from,
    target.clone(),
    &clip,
    duration_ms,
    easing,
    resizes,
    config,
    is_focused,
    now,
    &mut batch,
  ) && trail_clip.as_ref().is_none_or(|trail_clip| {
    peeks
      .add_trail(window, trail_clip, config, is_focused, now, &mut batch)
  });
  batch.commit();

  if moved {
    perf::note_animation_start(is_resize);
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  moved
}

/// How a WM fullscreen toggle is drawn on peeks, if it is: enabled, with
/// peeks on, under `GLAZEWM_ISOLATE=morepeeks`, in `window_fullscreen`'s
/// own style, as the old path draws it.
fn fullscreen_on_peeks(config: &UserConfig) -> Option<PeekResize> {
  let fullscreen = &config.value.animations.window_fullscreen;
  let isolation = wm_platform::isolation::current();
  (isolation.peeks && isolation.more_peeks && fullscreen.enabled)
    .then_some(peek_resize(fullscreen.style))
}

/// How a window drawn in `style` is drawn on its peek.
fn peek_resize(style: ResizeStyle) -> PeekResize {
  match style {
    ResizeStyle::Stretch => PeekResize::Stretch,
    ResizeStyle::Fill => PeekResize::Fill,
  }
}

/// How `retarget_tile` draws a resize of a peek retargeted by `motion`:
/// a fullscreen toggle as [`fullscreen_on_peeks`] says, if it does, and
/// anything else as the relayout's `resizes`.
pub fn retarget_resize(
  motion: Motion,
  resizes: Option<PeekResize>,
  config: &UserConfig,
) -> Option<PeekResize> {
  match motion {
    Motion::Fullscreen => fullscreen_on_peeks(config).or(resizes),
    _ => resizes,
  }
}

/// Whether a window now in `state`, after `prev`, went from a tile to
/// fullscreen or back. A floating one, or one only changing how it is
/// fullscreen, isn't.
fn toggles_tile_fullscreen(
  state: &WindowState,
  prev: Option<&WindowState>,
) -> bool {
  matches!(
    (state, prev),
    (WindowState::Fullscreen(_), Some(WindowState::Tiling))
      | (WindowState::Tiling, Some(WindowState::Fullscreen(_)))
  )
}

/// Where the peek of a window toggled in or out of fullscreen, now in
/// `state` after `prev`, is raised beyond `raise_moving`: to the top of
/// the always-on-top band if the window is shown on top at either end,
/// so it plays in the band its cloaked window is in and is handed back
/// there.
pub fn fullscreen_peek_raise(
  state: &WindowState,
  prev: Option<&WindowState>,
) -> Option<PeekRaise> {
  let on_top = |state: &WindowState| matches!(state, WindowState::Fullscreen(config) if config.shown_on_top);

  (on_top(state) || prev.is_some_and(on_top)).then_some(PeekRaise::TopMost)
}

/// Puts `window`, which a WM command toggled in or out of fullscreen
/// from `from` to `target`, on a pinned peek, on `window_fullscreen`'s
/// timing, as [`move_on_peek`] puts a move (see [`fullscreen_on_peeks`]).
/// Returns whether it does.
///
/// Only a tile going fullscreen or back, from inside its monitor, and not
/// one another stand-in draws. A window driving itself fullscreen never
/// gets here: cloaking an exclusive-fullscreen game breaks it.
pub fn fullscreen_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  from: &Rect,
  target: &Rect,
  config: &UserConfig,
  is_focused: bool,
) -> bool {
  let Some(resize) = fullscreen_on_peeks(config) else {
    return false;
  };
  if from == target
    || !toggles_tile_fullscreen(
      &window.state(),
      window.prev_state().as_ref(),
    )
    || has_other_stand_in(&state.animation_manager, &window.id())
  {
    return false;
  }

  let Some((clip, handle)) = window.monitor().and_then(|monitor| {
    let clip = monitor.to_rect().ok()?;
    Some((clip, monitor.native_properties().handle))
  }) else {
    return false;
  };
  if !clip.contains_point(&from.center_point()) {
    return false;
  }

  let (duration_ms, easing) =
    movement_timing(Motion::Fullscreen, false, config);
  let now = state.animation_manager.frame_time();
  let mut batch = SurrogateBatch::new();
  let moved = state.animation_manager.peeks.move_tile(
    window,
    from,
    target.clone(),
    &clip,
    duration_ms,
    easing,
    Some(resize),
    config,
    is_focused,
    now,
    &mut batch,
  );
  batch.commit();

  if moved {
    perf::note_animation_start(!is_same_size(from, target));
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  moved
}

/// Whether `id`'s ring is left off its peek as its window is fullscreen,
/// as `border_hidden_for_fullscreen` leaves it off the window, under
/// `morepeeks` (see [`fullscreen_on_peek`]).
fn hides_ring_for_fullscreen(
  state: &WmState,
  id: Uuid,
  config: &UserConfig,
) -> bool {
  wm_platform::isolation::current().more_peeks
    && state
      .window_by_id(id)
      .is_some_and(|window| border_hidden_for_fullscreen(&window, config))
}

/// Whether a grab resize can draw `window` on a peek it holds (see
/// [`hold_on_peek`], and `sync_scroll_view` for a column): a tile shown
/// in its layout, its real window on its monitor unless it is a column,
/// on a peek that never moves, and with no other stand-in. A stack's
/// window is left out, its tab bar being a window of its own.
pub fn can_hold(
  state: &WmState,
  window: &WindowContainer,
  config: &UserConfig,
) -> bool {
  hold_refusal(state, window, config).is_none()
}

/// Why a grab resize can't draw `window` on a peek it holds (see
/// [`can_hold`]), or `None` if it can.
pub fn hold_refusal(
  state: &WmState,
  window: &WindowContainer,
  config: &UserConfig,
) -> Option<&'static str> {
  let is_shown = window
    .workspace()
    .is_some_and(|workspace| workspace.is_displayed())
    && !is_hidden_in_layout(window)
    && window.display_state() == wm_common::DisplayState::Shown;
  // A column's peek is cut at its monitor by the scroll view, which also
  // keeps the real window of one partly shown on it; one off screen is
  // drawn once a step brings it into view. A tile's real window must be on
  // its monitor, or the peek would cut it at that monitor's edge.
  let is_column =
    crate::layout::scrolling::column_of(&window.as_container()).is_some();
  let fits_monitor = is_column
    || window
      .monitor()
      .and_then(|monitor| monitor.to_rect().ok())
      .zip(window.native().frame_with_shadows().ok())
      .is_some_and(|(clip, rect)| {
        clip.contains_point(&rect.center_point())
      });

  if window.state() != WindowState::Tiling {
    Some("not tiling")
  } else if !is_shown {
    Some("not shown")
  } else if !fits_monitor {
    Some("not on its monitor")
  } else if window
    .parent()
    .is_some_and(|parent| parent.as_stack().is_some())
  {
    Some("in a stack")
  } else if has_other_stand_in(&state.animation_manager, &window.id()) {
    Some("drawn by another animation")
  } else if is_live(&peek_styles(window, config)) {
    Some("its backdrop can't be copied (rounded or parallax)")
  } else {
    None
  }
}

/// Puts `window` on a peek a grab resize holds where it puts it, from
/// `from` at `target`: drawn filled, its ring in cells, never landing
/// until the grab lets go of it (see [`Hold`]). Its real window is taken
/// over from and cloaked by the redraw that called this, as for a move on
/// a peek, and then resized behind it. Returns whether it is.
pub fn hold_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  from: &Rect,
  target: &Rect,
  config: &UserConfig,
  is_focused: bool,
) -> bool {
  let Some((clip, handle)) = window.monitor().and_then(|monitor| {
    let clip = monitor.to_rect().ok()?;
    Some((clip, monitor.native_properties().handle))
  }) else {
    return false;
  };

  let (_, easing) = movement_timing(Motion::Resize, true, config);
  let now = state.animation_manager.frame_time();
  let mut batch = SurrogateBatch::new();
  let peeks = &mut state.animation_manager.peeks;
  let moved = peeks.move_tile(
    window,
    from,
    target.clone(),
    &clip,
    0,
    easing,
    Some(PeekResize::Fill),
    config,
    is_focused,
    now,
    &mut batch,
  );
  if moved {
    peeks.hold(window.id(), true);
  } else {
    tracing::info!(
      target: crate::grab::GRAB_LOG_TARGET,
      "{window}: live, its peek couldn't be put up."
    );
  }
  batch.commit();

  if moved {
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  moved
}

/// How a relayout redrawing `windows` draws its resizes on peeks (see
/// [`resizes_on_peeks`]), if it does: only if every tile it moves or
/// resizes can go on a peek as [`move_on_peek`] puts it there.
///
/// One that can't would resize on the old path, in another style beside
/// the rest: the whole relayout keeps the old path's resizes then. So it
/// does when a resized window is at its new size already, which an
/// interactive resize leaves it at as its drag ends: that one stays
/// exactly as it was.
///
/// Columns are left to `sync_scroll_view`, and windows that open, are not
/// shown, or stay floating to the redraw itself. One crossing between
/// floating and tiling goes on a peek only as [`changes_state_on_peek`]
/// allows.
pub fn resizes_relayout(
  state: &WmState,
  windows: &[WindowContainer],
  config: &UserConfig,
) -> Option<PeekResize> {
  let resize = resizes_on_peeks(config)?;

  let animations = &config.value.animations;
  windows
    .iter()
    .all(|window| {
      let id = window.id();
      let (Some(from), Ok(target)) = (
        state.window_target_positions.get(&id),
        window.to_rect().and_then(|rect| {
          Ok(rect.apply_delta(&window.total_border_delta()?, None))
        }),
      ) else {
        return true;
      };
      let is_state_change = state.pending_sync.is_window_state_change(&id);
      let is_shown = window
        .workspace()
        .is_some_and(|workspace| workspace.is_displayed())
        && !is_hidden_in_layout(window)
        && window.state() != WindowState::Minimized;
      let is_resize = !is_same_size(from, &target);
      let animates = if is_resize {
        is_peek_resize(
          from,
          &target,
          animations.window_resize.threshold_px,
        )
      } else {
        animations.window_move.enabled
          && is_pure_move(
            from,
            &target,
            animations.window_move.threshold_px,
          )
      };

      if !is_shown
        || !animates
        || crate::layout::scrolling::column_of(&window.as_container())
          .is_some()
        || (matches!(window.state(), WindowState::Floating(_))
          && !is_state_change)
      {
        return true;
      }

      let fits_monitor = tile_clips(state, window, from, config).is_some();
      let is_resized_already = is_resize
        && !state.animation_manager.peeks.is_moving(&id)
        && window
          .native()
          .frame_with_shadows()
          .is_ok_and(|actual| is_same_size(&actual, &target));

      let moves_as_tile = if is_state_change {
        changes_state_on_peek(window)
      } else {
        window.state() == WindowState::Tiling
      };

      moves_as_tile
        && window.display_state() == wm_common::DisplayState::Shown
        && !has_other_stand_in(&state.animation_manager, &id)
        && can_move_on_peek(&peek_styles(window, config))
        && fits_monitor
        && !is_resized_already
    })
    .then_some(resize)
}

/// The clips of the peeks [`move_on_peek`] draws `window`, a tile moving
/// from `from`, on (see [`move_clips`]). With `morepeeks`, one moving onto
/// another monitor is drawn on two, unless a peek draws it already (one
/// handing it back is on its own monitor), it is in a stack, whose tab
/// bar is cut at one monitor, or its peek is live.
fn tile_clips(
  state: &WmState,
  window: &WindowContainer,
  from: &Rect,
  config: &UserConfig,
) -> Option<(Rect, Option<Rect>)> {
  let clip = window.monitor()?.to_rect().ok()?;
  // A live peek's real backdrop overlay can follow one peek, not two.
  let crosses = wm_platform::isolation::current().more_peeks
    && !state.animation_manager.peeks.draws_window(&window.id())
    && window
      .parent()
      .is_none_or(|parent| parent.as_stack().is_none())
    && !is_live(&peek_styles(window, config));
  let source = crosses
    .then(|| state.monitor_at_point(&from.center_point()))
    .flatten()
    .and_then(|monitor| monitor.to_rect().ok());

  move_clips(from, clip, source, crosses)
}

/// Whether a stand-in other than a peek draws `window_id` or hands it
/// back: its own animation, a move's surrogate until it has faded out, or
/// a workspace switch's. A move then stays with it.
pub(super) fn has_other_stand_in(
  manager: &AnimationManager,
  window_id: &Uuid,
) -> bool {
  manager.get_animation(window_id).is_some()
    || manager.owns_window_position(window_id)
    || (manager.has_active_surrogate(window_id)
      && !manager.peeks.draws_window(window_id))
}

/// Covers the rings and backdrops of windows whose peeks [`move_on_peek`]
/// put up this pass with their copies, for the pass's shared flush to
/// composite, as `sync_scroll_view` does before its own flush.
pub fn cover_moved_overlays(state: &mut WmState, config: &UserConfig) {
  track_overlays(state, config, SurrogateBatch::new());
}

/// Hides the rings and backdrops [`cover_moved_overlays`] covered, now
/// that a flush has composited their copies: hidden on the first tick of
/// the move instead, a frame could show them still at rest beside their
/// moved copies.
pub fn hand_over_moved_overlays(state: &mut WmState, config: &UserConfig) {
  state.animation_manager.peeks.note_composited();
  track_overlays(state, config, SurrogateBatch::new());
}

/// Has `window`, coming back from minimized to `target`, enter on a
/// pinned peek by `params`, as [`enter_on_peek`] has a new one. A minimize
/// still playing on its peek turns round where it is instead. Returns
/// whether it does either.
///
/// A window still natively minimized (`toggle-minimized`) is restored at
/// `target` behind a cloak first (see [`restore_cloaked`]).
pub fn restore_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  target: &Rect,
  params: &WindowTransitionParams,
  config: &UserConfig,
  is_focused: bool,
) -> bool {
  let id = window.id();
  let now = state.animation_manager.frame_time();

  if state
    .animation_manager
    .peeks
    .turn_round(&id, params, false, now)
  {
    restore_cloaked(&window.native(), target);
    state.animation_manager.peeks.raise_moving(&id);
    perf::note_animation_start(false);
    state.animation_manager.ensure_timer_running();
    return true;
  }

  let enters = Transit::opening(params).is_some()
    && state
      .animation_manager
      .peeks
      .can_enter(window, &peek_styles(window, config));
  if !enters {
    return false;
  }

  restore_cloaked(&window.native(), target);
  let entered =
    enter_on_peek(state, window, target, params, config, is_focused);
  state.animation_manager.peeks.raise_moving(&id);
  entered
}

/// Restores `window`, if it is still natively minimized, at `rect` behind
/// a cloak, so that no frame shows it before its peek does
/// (`ANIMATION_REVIEW.md` 10). One that can't be cloaked is left
/// minimized, to be restored as it is handed back.
pub(super) fn restore_cloaked(window: &NativeWindow, rect: &Rect) {
  if !window.is_minimized() {
    return;
  }

  if let Err(err) = window.set_cloaked(true) {
    tracing::warn!("Failed to cloak a minimized window to restore: {err}");
    return;
  }
  if let Err(err) = window.restore(Some(rect)) {
    tracing::warn!("Failed to restore a cloaked window: {err}");
  }
}

/// Takes `window`, at `rect`, off the screen on a pinned peek by `params`
/// (see [`PeekView::leave`]), on ticks paced by its monitor's vblanks.
/// Clipped to the whole monitor, as an open on a peek is.
///
/// A minimize's peek takes over before this returns: the caller then
/// minimizes the window, which leaves no thumbnail to warm up from, so a
/// frame is composited here first, as `commit_pending_cloaks` does. A
/// close's waits for the next tick, which cloaks the window: a closing
/// window is out of the tree, so no cloak of the following sync's covers
/// it.
#[allow(clippy::too_many_arguments)]
pub fn leave_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  rect: &Rect,
  transition: AwayTransition,
  params: &WindowTransitionParams,
  config: &UserConfig,
  is_focused: bool,
) -> Leave {
  let Some((clip, handle)) = window.monitor().and_then(|monitor| {
    let clip = monitor.to_rect().ok()?;
    Some((clip, monitor.native_properties().handle))
  }) else {
    return Leave::Unsupported;
  };

  let now = state.animation_manager.frame_time();
  let mut batch = SurrogateBatch::new();
  let leave = state.animation_manager.peeks.leave(
    window, rect, &clip, transition, params, config, is_focused, now,
    &mut batch,
  );
  if leave != Leave::Leaving {
    batch.commit();
    return leave;
  }

  let id = window.id();
  state.animation_manager.drop_companion_cover(&id);
  track_overlays(state, config, batch);

  if transition == AwayTransition::Minimize
    && state.animation_manager.peeks.awaits_take_over(&id)
  {
    wm_platform::dwm_flush();
    state.animation_manager.peeks.note_composited();
    track_overlays(state, config, SurrogateBatch::new());
    state.animation_manager.peeks.take_over_leaving(&id);
  }

  perf::note_animation_start(false);
  state.animation_manager.ensure_waiter_for(handle);
  state.animation_manager.ensure_timer_running();

  Leave::Leaving
}

/// Ends the animations of windows done leaving, as `update_internal` ends
/// a close or minimize surrogate's: their overlays go, and a closing one
/// is sent `WM_CLOSE`.
pub fn finish_leaving(state: &mut WmState, left: Vec<Left>) {
  for Left {
    id,
    transition,
    window,
  } in left
  {
    state.window_overlays.remove(&id);

    match transition {
      AwayTransition::Close => {
        if let Err(err) = window.close() {
          tracing::warn!(
            "Failed to send WM_CLOSE for window {id}: {err}."
          );
        }
        state.closing_windows.mark_close_sent(id, Instant::now());
      }
      AwayTransition::Minimize => {
        tracing::debug!("Minimize on its peek complete for {id}.");
      }
    }
  }
}

/// A hidden peek of `window` for `motion` in `role`, carrying copies of
/// its backdrop `styles`, and whether it covers a still-shown real window,
/// so it has to be composited before that window is cloaked. `None` when
/// the peek can't be created.
fn new_entry(
  window: &WindowContainer,
  motion: WindowAnimationState,
  clip: &Rect,
  opacity: u8,
  corner_style: CornerStyle,
  styles: &[Option<BackdropOverlayParams>; 2],
  role: Role,
) -> Option<(Entry, bool)> {
  let _scope = perf::scope(Stage::PeekNew);
  let is_moving = matches!(role, Role::Moving { .. } | Role::Leaving(_));

  // Sized like the window itself, which a collapsed `from` (an opening
  // column) isn't. A window with a backdrop that can't be copied along has
  // a peek that moves with it.
  let mut peek = WindowPeek::new(
    &window.native(),
    &motion.target_rect,
    clip,
    &corner_style,
    is_live(styles),
  )
  .inspect_err(|err| {
    tracing::warn!("Failed to create a peek of {window}: {err}");
  })
  .ok()?;

  // Before the ring, which goes on top. Shown in its style by
  // `track_overlays`, before anything is composited.
  peek.carry_styles(styles);

  // A moving column's or a leaving window's real window may still show
  // until the peek takes over from it as it is cloaked; one at rest is
  // already cloaked.
  let covers_shown_window =
    is_moving && !window.native().is_cloaked().unwrap_or(true);
  if covers_shown_window {
    peek.warm_up();
  }

  let rect = motion.current_rect();
  let entry = Entry {
    peek,
    motion,
    rect,
    clip: clip.clone(),
    opacity,
    role,
    ring: Handover::None,
    backdrop: Handover::None,
    composited: false,
    transit: None,
    is_column: true,
    departing: None,
    stretch: None,
    hold: Hold::Free,
    trail: None,
  };

  Some((entry, covers_shown_window))
}
/// Keeps the border and backdrop overlays of the windows the scroll view
/// draws on their peeks, cut at the monitor edge like the peeks are.
/// `sync_overlays` leaves these windows to it.
///
/// Commits their moves in `batch`, along with the peeks' moves it already
/// holds, so a peek and its overlays land in the same frame.
///
/// A peek that never moves carries copies of the ring and backdrop (see
/// [`Handover`]). One that moves with its window has its border
/// pinned to the monitor instead and slid within it, as for a workspace
/// switch, so the ring and where it is cut can't land on screen a frame
/// apart.
pub fn track_overlays(
  state: &mut WmState,
  config: &UserConfig,
  mut batch: SurrogateBatch,
) {
  let focused_id = state.focused_container().map(|focused| focused.id());

  let tracked = state
    .animation_manager
    .peeks
    .entries
    .iter()
    .map(|(id, entry)| {
      // Only while some of the window is on screen.
      let frame = entry
        .peek
        .visible_rect()
        .and(entry.peek.frame_rect())
        .cloned();
      let is_moving = entry.role != Role::Peek;
      let anchor = overlay_anchor(state, *id, entry);
      let is_live = entry.peek.is_live();
      // A leaving window keeps the look it left with.
      let kept_set = entry.departing.as_ref().map(|left| left.effect_set);
      let clip = entry.clip.clone();
      (*id, frame, clip, is_moving, is_live, anchor, kept_set)
    })
    .collect::<Vec<_>>();

  for (id, frame, clip, is_moving, is_live, anchor, kept_set) in tracked {
    // Blended across a focus fade, as steady state draws it.
    let set = kept_set
      .unwrap_or_else(|| state.effect_set(id, focused_id == Some(id)));
    let backdrop =
      focus_blend::<NativeBackdropOverlay>(state, id, set, config);
    let border =
      focus_blend::<NativeBorderOverlay>(state, id, set, config)
        .filter(|_| !hides_ring_for_fullscreen(state, id, config));

    if is_live {
      note_live_size(state, id);
    }
    let carries_backdrop = !is_live && carry_backdrop(state, id, backdrop);

    match (backdrop, &frame) {
      _ if carries_backdrop => {}
      (Some(blend), Some(frame)) => {
        upsert_clipped_overlay::<NativeBackdropOverlay>(
          &mut state.window_overlays,
          id,
          blend,
          frame,
          &clip,
          anchor,
          &mut batch,
        );
      }
      _ => {
        if let Some(overlay) = state
          .window_overlays
          .get_mut(&id)
          .and_then(|overlays| overlays.backdrop.as_mut())
        {
          overlay.hide();
        }
      }
    }

    if !is_live && carry_ring(state, id, border) {
      continue;
    }

    // Framed with its stack's tab bar, like the real window.
    let border_frame = frame
      .map(|frame| crate::tab_bars::with_tab_bar_of(state, id, &frame));

    match (border, border_frame) {
      (Some(blend), Some(frame)) if is_moving => {
        upsert_pinned_border_overlay(
          &mut state.window_overlays,
          id,
          blend,
          &frame,
          &clip,
          &clip,
          anchor,
        );
      }
      (Some(blend), Some(frame)) => {
        upsert_clipped_overlay::<NativeBorderOverlay>(
          &mut state.window_overlays,
          id,
          blend,
          &frame,
          &clip,
          anchor,
          &mut batch,
        );
      }
      _ => {
        if let Some(overlay) = state
          .window_overlays
          .get_mut(&id)
          .and_then(|overlays| overlays.border.as_mut())
        {
          overlay.hide();
        }
      }
    }
  }

  batch.commit();
}

/// The window the overlays of `id`, drawn by `entry`, sit behind: its
/// peek, or its real window once that is back on screen under a releasing
/// peek, where they stay once the peek goes. A landed live tile's go there
/// as its peek leaves the top for just above the window (see
/// `WindowPeek::begin_hand_back`), before the window is uncloaked.
fn overlay_anchor(state: &WmState, id: Uuid, entry: &Entry) -> HWND {
  let behind_window = match entry.role {
    Role::Releasing { .. } => true,
    Role::HandingBack => entry.peek.is_live() && !entry.is_column,
    _ => false,
  };

  if behind_window {
    state
      .window_by_id(id)
      .map_or_else(|| entry.peek.hwnd(), |window| window.native().hwnd())
  } else {
    entry.peek.hwnd()
  }
}

/// Notes the real window's rect for `id`'s live peek, if it resizes on it
/// (see [`Stretch::actual`]), as [`carry_ring`] does for a pinned one.
fn note_live_size(state: &mut WmState, id: Uuid) {
  let resizes = state
    .animation_manager
    .peeks
    .entries
    .get(&id)
    .is_some_and(|entry| entry.stretch.is_some());
  if !resizes {
    return;
  }

  let Some(bounds) = state
    .window_by_id(id)
    .and_then(|window| window.native().frame_with_shadows().ok())
  else {
    return;
  };
  if let Some(stretch) = state
    .animation_manager
    .peeks
    .entries
    .get_mut(&id)
    .and_then(|entry| entry.stretch.as_mut())
  {
    stretch.actual = Some(bounds);
  }
}

/// Puts the backdrop and border of `window_id`, moving on a live peek
/// just raised, back directly behind it: a live peek's are real overlays,
/// not copies drawn in it.
pub fn restack_live_overlays(state: &mut WmState, window_id: &Uuid) {
  let Some(anchor) = state
    .animation_manager
    .peeks
    .entries
    .get(window_id)
    .filter(|entry| {
      entry.peek.is_live() && matches!(entry.role, Role::Moving { .. })
    })
    .map(|entry| entry.peek.hwnd())
  else {
    return;
  };
  let Some(overlays) = state.window_overlays.get_mut(window_id) else {
    return;
  };

  // Backdrop first, so the border ends up between it and the peek.
  let backdrop = overlays
    .backdrop
    .as_mut()
    .filter(|overlay| overlay.is_visible())
    .map(|overlay| overlay.sync_z_order(anchor, true));
  let border = overlays
    .border
    .as_mut()
    .filter(|overlay| overlay.is_visible())
    .map(|overlay| overlay.sync_z_order(anchor, true));
  for result in [backdrop, border].into_iter().flatten() {
    if let Err(err) = result {
      tracing::warn!("Failed to restack {window_id}'s overlays: {err}");
    }
  }
}

/// Keeps `id`'s pinned peek drawing its window's ring, with `params` or
/// not at all, and shadow around where the real window is, and hands the
/// ring over to its copy a step at a time (see [`Handover`]). Returns
/// whether it did: not when the copy failed.
///
/// Both follow the real window's own rect and frame, as steady state
/// places its ring (see `sync_overlays`), rather than where the layout
/// puts it: an app that keeps its window a little off that rect would
/// otherwise have its ring shift as it scrolls, and back once it lands.
#[allow(clippy::too_many_lines)]
fn carry_ring(
  state: &mut WmState,
  id: Uuid,
  blend: Option<StyleBlend<BorderOverlayParams>>,
) -> bool {
  let leaving = leaving_window(state, id);
  let window = state.window_by_id(id).filter(|_| leaving.is_none());
  let (native, bounds, frame) = match (leaving, &window) {
    // A leaving window's ring stays where it was, without a tab bar.
    (Some(leaving), _) => leaving,
    (None, Some(window)) => {
      // Shown where it was, or cloaked where its peek is headed.
      let native = window.native().clone();
      let (Ok(bounds), Ok(frame)) =
        (native.frame_with_shadows(), native.frame())
      else {
        return false;
      };
      (native, bounds, frame)
    }
    (None, None) => return false,
  };

  if let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id) {
    for (peek, _) in entry.peeks_mut() {
      peek.follow_window(&bounds, &frame);
    }
    if let Some(stretch) = &mut entry.stretch {
      stretch.actual = Some(bounds.clone());
    }
  }

  let Some(blend) = blend else {
    // Hidden, which also ends its parking.
    if let Some(overlay) = state
      .window_overlays
      .get_mut(&id)
      .and_then(|overlays| overlays.border.as_mut())
    {
      overlay.hide();
    }
    return true;
  };

  // Framed with its stack's tab bar, like the real window.
  let frame = landing_frame(state, id, frame);
  let ring_frame = window.as_ref().map_or_else(
    || frame.clone(),
    |window| crate::tab_bars::window_with_tab_bar(window, &frame),
  );
  let anchor = native.hwnd();

  if !create_concealed_ring(state, id, &ring_frame, blend.from, anchor) {
    return false;
  }

  let Some(overlay) = overlay_entry::<NativeBorderOverlay>(
    &mut state.window_overlays,
    id,
    blend.from,
    &ring_frame,
    anchor,
  ) else {
    return false;
  };
  overlay.apply_blend(blend);

  let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id)
  else {
    return false;
  };
  let is_landing =
    matches!(entry.role, Role::HandingBack | Role::Releasing { .. });
  // A window whose size changes on its peek keeps its ring's width and
  // corners, cut into cells.
  let corner_reach =
    entry.stretch.is_some().then(|| overlay.corner_reach());

  entry.ring = match entry.ring {
    // Shown where the window landed, under the copy, before the peek
    // that draws it goes.
    Handover::Covering | Handover::Covered | Handover::Parked
      if is_landing =>
    {
      overlay.set_rect(&ring_frame, anchor);
      Handover::Revealed
    }
    Handover::Revealed if is_landing => Handover::Revealed,
    // A ring on screen is covered where it is first; one that isn't is
    // concealed at once.
    Handover::None | Handover::Revealed => {
      let is_shown = overlay.is_visible();
      overlay.park(&ring_frame, anchor, !is_shown);

      let ring_rect = overlay.outer_rect();
      if !entry.carry_ring(
        overlay.hwnd(),
        &ring_rect,
        &bounds,
        corner_reach,
      ) {
        return false;
      }

      if is_shown {
        Handover::Covering
      } else {
        Handover::Parked
      }
    }
    Handover::Covering => Handover::Covering,
    // A resizing window's ring stays where it was carried from, unless
    // the ring itself changed: re-parked at the window's frame, its window
    // and visual would be resized by their own routes while its cells are
    // drawn from them.
    Handover::Covered | Handover::Parked
      if corner_reach.is_some()
        && entry.peek.carried_ring_rect()
          == Some(&overlay.outer_rect()) =>
    {
      overlay.conceal();
      for (peek, _) in entry.peeks_mut() {
        peek.cut_ring(corner_reach);
      }
      Handover::Parked
    }
    Handover::Covered | Handover::Parked => {
      overlay.park(&ring_frame, anchor, true);
      let ring_rect = overlay.outer_rect();
      entry.carry_ring(overlay.hwnd(), &ring_rect, &bounds, corner_reach);
      Handover::Parked
    }
  };

  true
}

/// Where `id`'s ring and backdrop go back to as its peek lands, its real
/// window's visible frame being `frame`: there, unless the window was
/// sent its size only as it lands (see [`Stretch::deferred`]), whose frame
/// is still the old one. Theirs is then the frame it lands at, or they
/// would be shown at the old one until the window has its new size.
fn landing_frame(state: &WmState, id: Uuid, frame: Rect) -> Rect {
  let target = state
    .animation_manager
    .peeks
    .entries
    .get(&id)
    .filter(|entry| {
      matches!(entry.role, Role::HandingBack | Role::Releasing { .. })
        && entry
          .stretch
          .as_ref()
          .is_some_and(|stretch| stretch.deferred)
    })
    .map(|entry| entry.motion.target_rect.clone());

  match (target, state.window_by_id(id)) {
    (Some(target), Some(window)) => {
      visible_frame(&target, &window.native_properties().shadow_borders)
    }
    _ => frame,
  }
}

/// The visible frame of a window placed at `rect`, which takes in its
/// invisible `shadow_borders`.
fn visible_frame(rect: &Rect, shadow_borders: &RectDelta) -> Rect {
  rect.apply_delta(&shadow_borders.inverse(), None)
}

/// Whether a ring made for an entry in `role` is made parked and
/// concealed: one entering or leaving, by a transit. Made shown, it was
/// handed over as a ring on screen, drawn whole at the window's rect for a
/// frame around its zoomed, slid or faded copy.
fn conceals_new_ring(role: Role, has_transit: bool) -> bool {
  has_transit && matches!(role, Role::Moving { .. } | Role::Leaving(_))
}

/// Makes `id` a ring parked and concealed at `ring_frame`, behind
/// `anchor`, if it has none and [`conceals_new_ring`] says so. Returns
/// `false` only when that failed.
fn create_concealed_ring(
  state: &mut WmState,
  id: Uuid,
  ring_frame: &Rect,
  params: BorderOverlayParams,
  anchor: HWND,
) -> bool {
  let conceals = state
    .animation_manager
    .peeks
    .entries
    .get(&id)
    .is_some_and(|entry| {
      conceals_new_ring(entry.role, entry.transit.is_some())
    });
  let slot = &mut state.window_overlays.entry(id).or_default().border;
  if slot.is_some() || !conceals {
    return true;
  }

  match NativeBorderOverlay::create_parked(
    ring_frame, params, anchor, true,
  ) {
    Ok(overlay) => {
      *slot = Some(overlay);
      true
    }
    Err(err) => {
      tracing::debug!("Border overlay creation failed for {id}: {err}.");
      false
    }
  }
}

/// Hands `id`'s backdrop, drawn with `params` or not at all, over to the
/// copy its pinned peek carries, as its ring is (see [`Handover`]): the
/// copy is shown under the overlay, which goes a composited frame later,
/// and the overlay comes back where the window lands, under the copy.
/// Returns whether it did: not for a peek carrying no backdrop.
fn carry_backdrop(
  state: &mut WmState,
  id: Uuid,
  blend: Option<StyleBlend<BackdropOverlayParams>>,
) -> bool {
  let carries = state
    .animation_manager
    .peeks
    .entries
    .get(&id)
    .is_some_and(|entry| entry.peek.carries_backdrop());
  let leaving = leaving_window(state, id);
  if !carries || (leaving.is_none() && state.window_by_id(id).is_none()) {
    return false;
  }

  let Some(blend) = blend else {
    if let Some(overlay) = state
      .window_overlays
      .get_mut(&id)
      .and_then(|overlays| overlays.backdrop.as_mut())
    {
      overlay.hide();
    }
    if let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id)
    {
      for (peek, _) in entry.peeks_mut() {
        peek.hide_backdrop();
      }
      entry.backdrop = Handover::None;
    }
    return true;
  };

  // Where steady state puts it (see `sync_overlays`): over the real
  // window's own frame, where it shows before a scroll and waits cloaked
  // during one. A leaving window's stays where it was.
  let (native, frame) = if let Some((native, _, frame)) = leaving {
    (native, frame)
  } else {
    let Some(native) =
      state.window_by_id(id).map(|window| window.native().clone())
    else {
      return false;
    };
    let Ok(frame) = native.frame() else {
      return false;
    };
    (native, landing_frame(state, id, frame))
  };
  let anchor = native.hwnd();

  let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id)
  else {
    return false;
  };
  let is_landing =
    matches!(entry.role, Role::HandingBack | Role::Releasing { .. });
  for (peek, _) in entry.peeks_mut() {
    peek.show_backdrop();
    peek.show_style_blend(&blend);
  }

  let handover = entry.backdrop;
  entry.backdrop = if is_landing
    && matches!(
      handover,
      Handover::Covering | Handover::Covered | Handover::Parked
    ) {
    let Some(overlay) = overlay_entry::<NativeBackdropOverlay>(
      &mut state.window_overlays,
      id,
      blend.from,
      &frame,
      anchor,
    ) else {
      return false;
    };
    overlay.apply_blend(blend);
    overlay.set_rect(&frame, anchor);
    Handover::Revealed
  } else {
    // Only an overlay that exists is handed over: one created now would
    // show at once, over the copy.
    let mut overlay = state
      .window_overlays
      .get_mut(&id)
      .and_then(|overlays| overlays.backdrop.as_mut());
    if let Some(overlay) = &mut overlay {
      overlay.apply_blend(blend);
    }
    let is_shown =
      overlay.as_ref().is_some_and(|overlay| overlay.is_visible());

    match handover {
      Handover::Revealed if is_landing => Handover::Revealed,
      Handover::None | Handover::Revealed => {
        if is_shown {
          Handover::Covering
        } else {
          Handover::Parked
        }
      }
      Handover::Covering => Handover::Covering,
      Handover::Covered | Handover::Parked => {
        if let Some(overlay) = overlay.filter(|_| is_shown) {
          overlay.hide();
        }
        Handover::Parked
      }
    }
  };

  true
}

/// The real window, rect with shadows and frame that `id`'s peek kept as
/// it started to leave, if it is leaving (see [`Departing`]).
fn leaving_window(
  state: &WmState,
  id: Uuid,
) -> Option<(NativeWindow, Rect, Rect)> {
  let departing = state
    .animation_manager
    .peeks
    .entries
    .get(&id)?
    .departing
    .as_ref()?;

  Some((
    departing.native.clone(),
    departing.bounds.clone(),
    departing.frame.clone(),
  ))
}

/// The backdrops of `window` focused and not, which its peek carries
/// copies of.
pub(super) fn peek_styles(
  window: &WindowContainer,
  config: &UserConfig,
) -> [Option<BackdropOverlayParams>; 2] {
  [true, false].map(|is_focused| {
    surrogate_effects_for(window.effect_set(is_focused), config)
      .2
      .backdrop
  })
}

#[cfg(test)]
mod tests {
  use std::{
    collections::HashMap,
    time::{Duration, Instant},
  };

  use uuid::Uuid;
  use wm_common::{
    EasingFunction, FloatingStateConfig, FullscreenStateConfig,
    ResizeStyle, WindowResizeConfig, WindowState, WindowTransitionParams,
    WindowTransitionStyle,
  };
  use wm_platform::{LengthValue, Rect, RectDelta};

  use super::{
    allows_motion, conceals_new_ring, crosses_floating,
    floating_peek_raise, follow_duration, follows_tile,
    fullscreen_peek_raise, has_landed, has_left, is_departed,
    is_peek_resize, is_pure_move, is_stray, kept_transit, lands_on_trail,
    leaves_as, leaving_transit, move_clips, peek_resize, resizes_at_start,
    restretched, retargeted_motion, returning_transit, sampled_size,
    settling, state_change_peek_raise, state_change_resize, take_left,
    toggles_tile_fullscreen, visible_frame, Hold, PeekRaise, PeekResize,
    Role, Stretch, FILL_LANDING_WAIT, HOLD_RELEASE_WAIT, STRETCH_SETTLE,
  };
  use crate::animation::{
    state::WindowAnimationState,
    transit::{Direction, Transit},
    AwayTransition,
  };

  #[test]
  fn floating_peeks_play_in_the_band_their_windows_land_in() {
    // Shown on top: over the other floating windows shown on top, focused
    // or not, as its window lands at the top of their band.
    assert_eq!(floating_peek_raise(true, true), Some(PeekRaise::TopMost));
    assert_eq!(floating_peek_raise(true, false), Some(PeekRaise::TopMost));
    // Otherwise under them: on top of the rest if focused, else where its
    // window is.
    assert_eq!(floating_peek_raise(false, true), Some(PeekRaise::Top));
    assert_eq!(floating_peek_raise(false, false), None);
  }

  #[test]
  fn only_crossings_between_floating_and_tiling_go_on_peeks() {
    let floating = WindowState::Floating(FloatingStateConfig::default());
    let fullscreen =
      WindowState::Fullscreen(FullscreenStateConfig::default());

    assert!(crosses_floating(&floating, Some(&WindowState::Tiling)));
    assert!(crosses_floating(&WindowState::Tiling, Some(&floating)));
    // Dropped after a drag, its state before the drag.
    assert!(crosses_floating(
      &WindowState::Tiling,
      Some(&WindowState::Tiling)
    ));
    // Back from fullscreen or minimized: not this path.
    assert!(!crosses_floating(&WindowState::Tiling, Some(&fullscreen)));
    assert!(!crosses_floating(&floating, Some(&fullscreen)));
    assert!(!crosses_floating(&floating, Some(&WindowState::Minimized)));
    assert!(!crosses_floating(&fullscreen, Some(&floating)));
    assert!(!crosses_floating(&WindowState::Minimized, Some(&floating)));
  }

  #[test]
  fn a_state_change_resizes_in_window_resizes_style() {
    let resize = |enabled, style| WindowResizeConfig {
      enabled,
      style,
      ..WindowResizeConfig::default()
    };

    assert_eq!(
      state_change_resize(&resize(true, ResizeStyle::Fill)),
      Some(PeekResize::Fill)
    );
    assert_eq!(
      state_change_resize(&resize(true, ResizeStyle::Stretch)),
      Some(PeekResize::Stretch)
    );
    assert_eq!(
      state_change_resize(&resize(false, ResizeStyle::Fill)),
      None
    );
  }

  fn fullscreen(shown_on_top: bool) -> WindowState {
    WindowState::Fullscreen(FullscreenStateConfig {
      maximized: true,
      shown_on_top,
    })
  }

  #[test]
  fn only_tiles_toggled_in_or_out_of_fullscreen_go_on_peeks() {
    let tiling = WindowState::Tiling;
    assert!(toggles_tile_fullscreen(&fullscreen(false), Some(&tiling)));
    assert!(toggles_tile_fullscreen(&tiling, Some(&fullscreen(true))));
    // From or back to floating, or fullscreen in another way: the old
    // path.
    let floating = WindowState::Floating(FloatingStateConfig::default());
    assert!(!toggles_tile_fullscreen(
      &fullscreen(false),
      Some(&floating)
    ));
    assert!(!toggles_tile_fullscreen(
      &floating,
      Some(&fullscreen(false))
    ));
    assert!(!toggles_tile_fullscreen(
      &fullscreen(false),
      Some(&fullscreen(true))
    ));
    assert!(!toggles_tile_fullscreen(&fullscreen(false), None));
  }

  #[test]
  fn fullscreen_peeks_play_in_the_band_their_windows_are_in() {
    let tiling = WindowState::Tiling;
    assert_eq!(
      fullscreen_peek_raise(&fullscreen(true), Some(&tiling)),
      Some(PeekRaise::TopMost)
    );
    // Leaving the band only once handed back.
    assert_eq!(
      fullscreen_peek_raise(&tiling, Some(&fullscreen(true))),
      Some(PeekRaise::TopMost)
    );
    assert_eq!(
      fullscreen_peek_raise(&fullscreen(false), Some(&tiling)),
      None
    );
    assert_eq!(
      fullscreen_peek_raise(&tiling, Some(&fullscreen(false))),
      None
    );
  }

  #[test]
  fn state_change_peeks_play_in_the_band_their_windows_land_in() {
    // Landing floating: as a floating window opening.
    assert_eq!(
      state_change_peek_raise(true, true, false),
      Some(PeekRaise::TopMost)
    );
    assert_eq!(
      state_change_peek_raise(true, false, true),
      Some(PeekRaise::Top)
    );
    assert_eq!(state_change_peek_raise(true, false, false), None);
    // Landing tiled: the top of the other windows, focused or not.
    assert_eq!(
      state_change_peek_raise(false, true, false),
      Some(PeekRaise::Top)
    );
    assert_eq!(
      state_change_peek_raise(false, false, false),
      Some(PeekRaise::Top)
    );
  }

  #[test]
  fn fullscreen_peeks_are_drawn_in_their_own_style() {
    assert_eq!(peek_resize(ResizeStyle::Stretch), PeekResize::Stretch);
    assert_eq!(peek_resize(ResizeStyle::Fill), PeekResize::Fill);
  }

  fn linear() -> EasingFunction {
    EasingFunction::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0)
  }

  fn tile() -> Rect {
    Rect::from_ltrb(100, 50, 900, 650)
  }

  fn motion(duration_ms: u32) -> WindowAnimationState {
    WindowAnimationState::new_movement(
      tile(),
      tile(),
      duration_ms,
      linear(),
    )
  }

  fn params(duration_ms: u32) -> WindowTransitionParams {
    WindowTransitionParams {
      duration_ms,
      easing: linear(),
      style: WindowTransitionStyle::Zoom,
      away_opacity: 0.0,
    }
  }

  fn transit(duration_ms: u32) -> Transit {
    Transit::new(Direction::In, &params(duration_ms), 0.0)
  }

  /// An entering window lands once its motion and its transit both have.
  #[test]
  fn landing_needs_both_tracks() {
    let now = Instant::now();
    let (done, running) = (motion(0), motion(60_000));
    let (entered, entering) = (transit(0), transit(60_000));
    for clock in [&done, &running] {
      clock.start_clock(now);
    }
    for transit in [&entered, &entering] {
      transit.clock.start_clock(now);
    }

    assert!(has_landed(&done, None));
    assert!(!has_landed(&running, None));
    assert!(has_landed(&done, Some(&entered)));
    assert!(!has_landed(&done, Some(&entering)));
    assert!(!has_landed(&running, Some(&entered)));
  }

  /// A held peek never lands; let go of, it lands once its window has
  /// reached its target's size, or once it has waited long enough.
  #[test]
  fn a_held_peek_lands_once_let_go_and_its_window_reached() {
    let now = Instant::now();
    let released = Hold::Released(now, HOLD_RELEASE_WAIT);

    assert!(Hold::Free.allows_landing(false, now));
    assert!(!Hold::Held.allows_landing(true, now));
    assert!(!released.allows_landing(false, now));
    assert!(released.allows_landing(true, now));
    assert!(released.allows_landing(false, now + HOLD_RELEASE_WAIT));
  }

  /// Retargeted mid-entry, a window heads for its new tile from where it
  /// is, its transit carrying on as it was.
  #[test]
  fn retarget_keeps_the_entry_track() {
    let t0 = Instant::now();
    let now = t0 + Duration::from_millis(50);
    let entry_motion = motion(200);
    let entry_transit = transit(200);
    entry_motion.start_clock(t0);
    entry_transit.clock.start_clock(t0);
    let amount = entry_transit.amount_at(now);

    let new_tile = Rect::from_ltrb(100, 50, 500, 650);
    let retargeted = retargeted_motion(
      &entry_motion,
      new_tile.clone(),
      200,
      linear(),
      now,
    );

    assert_eq!(retargeted.target_rect, new_tile);
    assert_eq!(retargeted.current_state_at(now).0, tile());
    assert!(retargeted.is_running());
    assert!((entry_transit.amount_at(now) - amount).abs() < f32::EPSILON);
    assert!((amount - 0.25).abs() < 1e-3, "{amount}");
  }

  /// A pinned peek always moves its window; a live one only under
  /// `morepeeks`.
  #[test]
  fn live_peeks_move_only_under_morepeeks() {
    assert!(allows_motion(false, false));
    assert!(allows_motion(false, true));
    assert!(!allows_motion(true, false));
    assert!(allows_motion(true, true));
  }

  /// A column moved on mid-entry keeps entering where it was; one put at
  /// rest, or done entering, drops its transit.
  #[test]
  fn upsert_keeps_an_unfinished_entry_track() {
    let t0 = Instant::now();
    let now = t0 + Duration::from_millis(50);
    let entering = transit(200);
    entering.clock.start_clock(t0);
    let amount = entering.amount_at(now);

    let kept = kept_transit(Some(entering.clone()), true);
    assert!(kept.is_some_and(
      |kept| (kept.amount_at(now) - amount).abs() < f32::EPSILON
    ));
    assert!(kept_transit(Some(entering), false).is_none());

    let entered = transit(0);
    entered.clock.start_clock(t0);
    assert!(kept_transit(Some(entered), true).is_none());
    assert!(kept_transit(None, true).is_none());
  }

  /// A moving column is retargeted by the scroll, never as a window
  /// moving in a tiling layout; one at rest or landing by neither.
  #[test]
  fn columns_do_not_follow_tiles() {
    assert!(follows_tile(false, true));
    assert!(!follows_tile(true, true));
    assert!(!follows_tile(false, false));
  }

  /// A floating window moved mid-entry jumps to its new rect, as the old
  /// path's floating moves do, and lands once its entry has; one tiled or
  /// changing state follows on its timing.
  #[test]
  fn a_floating_entry_jumps_to_its_new_rect() {
    assert_eq!(follow_duration(true, false, 200), 0);
    assert_eq!(follow_duration(true, true, 200), 200);
    assert_eq!(follow_duration(false, false, 200), 200);

    let t0 = Instant::now();
    let now = t0 + Duration::from_millis(50);
    let entering = transit(200);
    entering.clock.start_clock(t0);
    for easing in [linear(), spring()] {
      let held = WindowAnimationState::new_movement(
        tile(),
        tile(),
        200,
        easing.clone(),
      );
      held.start_clock(t0);

      let moved = Rect::from_xy(400, 300, 800, 600);
      let jumped = retargeted_motion(
        &held,
        moved.clone(),
        follow_duration(true, false, 200),
        easing,
        now,
      );

      assert_eq!(jumped.current_state_at(now).0, moved);
      assert!(has_landed(&jumped, None));
      assert!(!has_landed(&jumped, Some(&entering)));
    }
  }

  /// Only a move that keeps the window's size, past the threshold, goes
  /// on a peek alone; a resize, or a move the old path wouldn't animate,
  /// doesn't.
  #[test]
  fn a_tile_from_another_monitor_is_drawn_on_two_peeks() {
    let left = Rect::from_ltrb(0, 0, 1920, 1080);
    let right = Rect::from_ltrb(1920, 0, 3840, 1080);
    let on_left = Rect::from_xy(100, 100, 800, 600);
    let on_right = Rect::from_xy(2020, 100, 800, 600);

    // On its own monitor: one peek, whether or not it may cross.
    assert_eq!(
      move_clips(&on_right, right.clone(), Some(right.clone()), true),
      Some((right.clone(), None))
    );
    assert_eq!(
      move_clips(&on_right, right.clone(), None, false),
      Some((right.clone(), None))
    );
    // From another: a trail on that one, only if it may cross.
    assert_eq!(
      move_clips(&on_left, right.clone(), Some(left.clone()), true),
      Some((right.clone(), Some(left.clone())))
    );
    assert_eq!(
      move_clips(&on_left, right.clone(), Some(left.clone()), false),
      None
    );
    // From no monitor known, or one that doesn't hold it: none.
    assert_eq!(move_clips(&on_left, right.clone(), None, true), None);
    assert_eq!(
      move_clips(&on_left, right.clone(), Some(right.clone()), true),
      None
    );
  }

  #[test]
  fn a_tile_headed_back_lands_on_its_trail() {
    let left = Rect::from_ltrb(0, 0, 1920, 1080);
    let right = Rect::from_ltrb(1920, 0, 3840, 1080);
    let on_left = Rect::from_xy(100, 100, 800, 600);
    let on_right = Rect::from_xy(2020, 100, 800, 600);
    let elsewhere = Rect::from_xy(100, 2000, 800, 600);

    assert!(lands_on_trail(&on_left, &right, &left));
    assert!(!lands_on_trail(&on_right, &right, &left));
    // Onto neither monitor, it stays as it is.
    assert!(!lands_on_trail(&elsewhere, &right, &left));
  }

  #[test]
  fn pure_moves_keep_their_size_past_the_threshold() {
    let moved = |dx, dy| Rect::from_xy(100 + dx, 50 + dy, 800, 600);
    assert!(is_pure_move(&tile(), &moved(800, 0), 10));
    assert!(is_pure_move(&tile(), &moved(-6, 5), 10));
    assert!(!is_pure_move(&tile(), &moved(6, 4), 10));
    assert!(!is_pure_move(&tile(), &tile(), 0));
    assert!(!is_pure_move(
      &tile(),
      &Rect::from_xy(900, 50, 400, 600),
      10
    ));
    assert!(!is_pure_move(&tile(), &Rect::from_xy(100, 50, 800, 601), 0));
  }

  fn spring() -> EasingFunction {
    EasingFunction::Spring(wm_common::SpringParams::default())
  }

  /// Pressed again mid-move (the same swap, a held key), a tile on its
  /// `window_move` timing and a column on the scroll's carry on from
  /// where they were drawn, never from rest: a spring at the speed it
  /// had, a curve with its start steepened toward it (which its second
  /// control point can make a little faster still).
  #[test]
  fn a_retargeted_move_keeps_its_speed() {
    let t0 = Instant::now();
    let at = |ms| t0 + Duration::from_millis(ms);
    let moved = |dx| Rect::from_xy(100 + dx, 50, 800, 600);
    #[allow(clippy::cast_precision_loss)]
    let speed = |motion: &WindowAnimationState, from, to| {
      let (from, to) = (at(from), at(to));
      (motion.current_state_at(to).0.x()
        - motion.current_state_at(from).0.x()) as f32
        / (to - from).as_secs_f32()
    };
    let curve = EasingFunction::CubicBezier(0.2, 0.0, 0.0, 1.0);

    for (duration_ms, easing) in [
      (200, curve.clone()),
      (300, curve),
      (200, spring()),
      (300, spring()),
    ] {
      let motion = WindowAnimationState::new_movement(
        moved(0),
        moved(800),
        duration_ms,
        easing.clone(),
      );
      motion.start_clock(t0);
      let drawn = motion.current_state_at(at(60)).0;
      let before = speed(&motion, 58, 60);

      let retargeted = retargeted_motion(
        &motion,
        moved(1_600),
        duration_ms,
        easing.clone(),
        at(60),
      );
      assert_eq!(retargeted.current_state_at(at(60)).0, drawn);
      let after = speed(&retargeted, 60, 62);
      if matches!(easing, EasingFunction::Spring(_)) {
        assert!(
          (after - before).abs() < before * 0.25,
          "{easing:?} over {duration_ms} ms: {before} then {after}"
        );
      }

      let restarted = WindowAnimationState::new_movement(
        drawn,
        moved(1_600),
        duration_ms,
        easing,
      );
      restarted.start_clock(at(60));
      assert!(speed(&restarted, 60, 62) < after);
    }
  }

  /// Only an entering window that is unmanaged or no longer shown is a
  /// stray; columns, entering windows still shown and leaving windows are
  /// left alone.
  #[test]
  fn strays_are_entering_windows_not_shown() {
    let entering = Role::Moving { ends_shown: true };
    assert!(is_stray(false, entering, None));
    assert!(is_stray(false, entering, Some(false)));
    assert!(!is_stray(false, entering, Some(true)));
    for shown in [None, Some(false), Some(true)] {
      assert!(!is_stray(true, entering, shown), "{shown:?}");
      for transition in [AwayTransition::Close, AwayTransition::Minimize] {
        let leaving = Role::Leaving(transition);
        assert!(!is_stray(false, leaving, shown), "{shown:?}");
      }
    }
  }

  /// A ring made for a window entering or leaving starts concealed, so
  /// only its copy shows it; one made for a column scrolling, a plain
  /// move or a landing window is made shown, as before.
  #[test]
  fn a_new_ring_is_concealed_while_its_window_transits() {
    let moving = Role::Moving { ends_shown: true };
    assert!(conceals_new_ring(moving, true));
    assert!(conceals_new_ring(
      Role::Leaving(AwayTransition::Close),
      true
    ));
    assert!(!conceals_new_ring(moving, false));
    for role in [
      Role::Peek,
      Role::HandingBack,
      Role::Releasing { settled: false },
    ] {
      assert!(!conceals_new_ring(role, true), "{role:?}");
    }
  }

  /// A leaving entry answers for its own transition only, which the
  /// close and minimize queries (and with them `has_active_surrogate`
  /// and `stands_in_for`) read; no other role does.
  #[test]
  fn leaving_entries_answer_for_their_transition() {
    let (close, minimize) =
      (AwayTransition::Close, AwayTransition::Minimize);
    assert!(leaves_as(Role::Leaving(close), close));
    assert!(!leaves_as(Role::Leaving(close), minimize));
    assert!(leaves_as(Role::Leaving(minimize), minimize));
    for role in [
      Role::Peek,
      Role::Moving { ends_shown: true },
      Role::HandingBack,
      Role::Releasing { settled: false },
    ] {
      assert!(!leaves_as(role, close) && !leaves_as(role, minimize));
    }
  }

  /// A leaving entry is taken out once its transit ends, so reported
  /// once; one still playing, and entries in other roles, stay.
  #[test]
  fn a_leaving_entry_is_reported_once() {
    let now = Instant::now();
    let held = motion(0);
    held.start_clock(now);
    let (left, leaving) = (transit(0), transit(60_000));
    for transit in [&left, &leaving] {
      transit.clock.start_clock(now);
    }
    let close = Role::Leaving(AwayTransition::Close);
    let (done, playing, moving) =
      (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let mut entries = HashMap::from([
      (done, (close, left.clone())),
      (playing, (close, leaving)),
      (moving, (Role::Moving { ends_shown: true }, left)),
    ]);
    let is_done = |(role, transit): &(Role, Transit)| {
      has_left(*role, &held, Some(transit))
    };

    let taken = take_left(&mut entries, is_done);
    assert_eq!(
      taken.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
      [done]
    );
    assert!(take_left(&mut entries, is_done).is_empty());
    assert!(
      entries.contains_key(&playing) && entries.contains_key(&moving)
    );
  }

  /// A window closed mid-entry leaves from the amount it had reached, and
  /// one at rest from all the way in.
  #[test]
  fn closed_mid_entry_leaves_from_where_it_is() {
    let t0 = Instant::now();
    let now = t0 + Duration::from_millis(50);
    let entering = transit(200);
    entering.clock.start_clock(t0);
    let amount = entering.amount_at(now);
    let fresh = || Transit::new(Direction::Out, &params(200), 1.0);

    let turned =
      leaving_transit(Some(&entering), fresh(), &params(200), now);
    assert_eq!(turned.direction, Direction::Out);
    assert!((turned.amount_at(now) - amount).abs() < 1e-3, "{amount}");
    assert!(turned.amount_at(now + Duration::from_millis(25)) < amount);

    let entered = transit(0);
    entered.clock.start_clock(t0);
    for at_rest in [None, Some(&entered)] {
      let left = leaving_transit(at_rest, fresh(), &params(200), now);
      assert!((left.from - 1.0).abs() < f32::EPSILON);
    }
  }

  /// A window restored mid-minimize comes back from the amount it had
  /// shrunk to, and minimized again on the way back turns round where it
  /// is once more: no jump either way.
  #[test]
  fn restored_mid_minimize_comes_back_from_where_it_is() {
    let t0 = Instant::now();
    let at = |ms| t0 + Duration::from_millis(ms);
    let leaving = Transit::new(Direction::Out, &params(200), 1.0);
    leaving.clock.start_clock(t0);
    let shrunk = leaving.amount_at(at(50));

    let returning = returning_transit(&leaving, &params(200), at(50));
    assert_eq!(returning.direction, Direction::In);
    assert!((returning.amount_at(at(50)) - shrunk).abs() < 1e-3);
    assert!(returning.amount_at(at(75)) > shrunk);

    let back = returning.amount_at(at(75));
    let fresh = Transit::new(Direction::Out, &params(200), 1.0);
    let again =
      leaving_transit(Some(&returning), fresh, &params(200), at(75));
    assert_eq!(again.direction, Direction::Out);
    assert!((again.amount_at(at(75)) - back).abs() < 1e-3, "{back}");
    assert!(again.amount_at(at(100)) < back);
  }

  /// Only columns leave with the scrolling layouts; a window entering a
  /// tiling layout keeps its peek.
  #[test]
  fn take_departed_leaves_other_entries() {
    assert!(is_departed(true, false));
    assert!(!is_departed(true, true));
    assert!(!is_departed(false, false));
    assert!(!is_departed(false, true));
  }

  /// Only a change of size past the threshold is a resize on a peek.
  #[test]
  fn peek_resizes_change_size_past_the_threshold() {
    let resized = |dw, dh| Rect::from_xy(100, 50, 800 + dw, 600 + dh);
    assert!(is_peek_resize(&tile(), &resized(-400, 0), 10));
    assert!(is_peek_resize(&tile(), &resized(6, 5), 10));
    assert!(!is_peek_resize(&tile(), &resized(6, 4), 10));
    assert!(!is_peek_resize(
      &tile(),
      &Rect::from_xy(900, 50, 800, 600),
      0
    ));
  }

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

  /// A window placed at a rect is framed inside its invisible borders.
  #[test]
  fn a_landing_frame_leaves_out_the_invisible_borders() {
    let borders = RectDelta::new(
      LengthValue::from_px(7),
      LengthValue::from_px(0),
      LengthValue::from_px(7),
      LengthValue::from_px(7),
    );

    assert_eq!(
      visible_frame(&Rect::from_ltrb(93, 100, 907, 707), &borders),
      Rect::from_ltrb(100, 100, 900, 700)
    );
  }

  /// Done moving, a filled window sent its size waits for it at most as
  /// long as a `fill` session does.
  #[test]
  fn a_filled_landing_waits_for_its_windows_size() {
    let now = Instant::now();
    let waiting = Hold::Released(now, FILL_LANDING_WAIT);

    assert!(!waiting.allows_landing(false, now + HOLD_RELEASE_WAIT));
    assert!(waiting.allows_landing(true, now));
    assert!(waiting.allows_landing(false, now + FILL_LANDING_WAIT));
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
