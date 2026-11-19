//! Stand-ins drawing windows whose real windows are cloaked.
//!
//! `PeekView` keeps an entry per window drawn by a peek and moves it on
//! each tick. Its parts: `motions` puts windows on peeks as the layout
//! changes, `surfaces` hosts tiles' peeks and stacks them, `ready`
//! keeps landed peeks for the next motion, `overlays` carries rings,
//! backdrops and tab bars along, and `stretch` draws resizes.

mod motions;
mod overlays;
mod ready;
mod stretch;
mod surfaces;

use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use motions::held_motion;
pub use motions::{
  can_hold, changes_state_on_peek, drop_strays, enter_on_peek,
  finish_leaving, floating_peek_raise, follow_duration,
  fullscreen_on_peek, fullscreen_peek_raise, hold_on_peek, hold_refusal,
  leave_on_peek, move_on_peek, resizes_on_peeks, restore_on_peek,
  retarget_resize, settle_switch_peeks, state_change_peek_raise,
  state_change_resize, PeekRaise,
};
pub(super) use motions::{
  has_other_stand_in, is_peek_resize, is_pure_move, restore_cloaked,
};
pub(super) use overlays::restack_joining_overlays;
use overlays::{carried_styles, conceals_bar, Handover, PeekStyles};
pub use overlays::{
  cover_moved_overlays, hand_over_moved_overlays, track_overlays,
};
pub use ready::prewarm_ready;
use ready::{
  kept_since_ready, note_ready_take, reuse_entry, Ready, ReadyFit,
};
pub(super) use stretch::note_fill_inputs;
use stretch::{
  is_same_size, resizes_at_start, restretched, sampled_size, Stretch,
};
pub use surfaces::keep_leaving_above;
pub(crate) use surfaces::surface_clips;
use surfaces::{
  keeps_rest_hosted, lanes_meet, rank, rests, Lane, Level, Stacking,
  Tenancy,
};
pub(super) use surfaces::{out_of_order, Host};
use uuid::Uuid;
use wm_common::{EasingFunction, EffectSet, WindowTransitionParams};
use wm_platform::{
  perf::{self, Stage},
  NativeWindow, PeekSurface, Point, Rect, WindowPeek, HWND,
};

use super::{
  stand_in::{moved_along, StandIn},
  state::WindowAnimationState,
  transit::{Direction, Transit},
};
use crate::{
  animation::AwayTransition,
  commands::general::surrogate_effects_for,
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
};

/// How long a window a grab let go of may take to report its last size
/// before its peek lands all the same (see [`Hold::Released`]).
const HOLD_RELEASE_WAIT: Duration = Duration::from_millis(150);

/// How long a window filled on its peek by a keyboard resize may take to
/// report its size once its motion is done before the peek lands all the
/// same (see [`Hold::Released`]).
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
  /// How far a hosted peek has taken over its stack's tab bar (see
  /// `bar_handover`).
  bar: Handover,
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
  /// Where its peek draws on its `PeekSurface`, if it is hosted on one.
  tenancy: Option<Tenancy>,
  /// For a peek reclaimed from a ready one, the registration order its
  /// thumbnails were all below then (see
  /// `wm_platform::next_thumbnail_order`): one with none registered since
  /// has been composited, held, so its cloak needs no flush (see
  /// [`PeekView::needs_flush`]).
  ready_mark: Option<u64>,
}

/// A second pinned peek of a tile moving onto another monitor, cut at the
/// monitor it comes from (see `tile_clips`), drawn
/// as the entry's own peek is.
///
/// Two peeks rather than one spanning both monitors: each carries the
/// backdrop of its own monitor, and nothing is drawn between them or on a
/// third. It never takes over or hands back the real window, and goes as
/// the motion lands, on the entry's own peek's monitor.
///
/// Hosted on its monitor's surface of the entry's layer whenever the
/// entry's own peek is hosted (see [`PeekView::add_trail`]): a second
/// tenancy of the tile, with its rank and lane, restacked among that
/// surface's tenants.
struct Trail {
  peek: WindowPeek,
  clip: Rect,
}

/// Which of an entry's peeks: its own, or its trail's (see [`Trail`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
  Own,
  Trail,
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

  /// Its peeks on surfaces, with their surfaces' windows: its own while it
  /// has a tenancy, and its trail's if that is hosted too.
  fn hosted_parts(&self) -> impl Iterator<Item = (Part, HWND)> + '_ {
    let own = self.tenancy.as_ref().map(|_| (Part::Own, self.peek.hwnd()));
    let trail = self.hosted_trail().map(|peek| (Part::Trail, peek.hwnd()));
    own.into_iter().chain(trail)
  }

  /// Its trail's peek, if that is a tenant of a surface.
  fn hosted_trail(&self) -> Option<&WindowPeek> {
    self
      .trail
      .as_ref()
      .map(|trail| &trail.peek)
      .filter(|peek| self.tenancy.is_some() && peek.is_hosted())
  }

  /// Whether any of its peeks is a tenant of the surface `surface`.
  fn is_on(&self, surface: HWND) -> bool {
    self.hosted_parts().any(|(_, hwnd)| hwnd == surface)
  }

  fn part(&self, part: Part) -> Option<&WindowPeek> {
    match part {
      Part::Own => Some(&self.peek),
      Part::Trail => self.trail.as_ref().map(|trail| &trail.peek),
    }
  }

  fn part_mut(&mut self, part: Part) -> Option<&mut WindowPeek> {
    match part {
      Part::Own => Some(&mut self.peek),
      Part::Trail => self.trail.as_mut().map(|trail| &mut trail.peek),
    }
  }

  /// Whether its tile and `other`'s, both on a surface, may overlap from
  /// now on (see [`lanes_meet`]): one that landed only where it did.
  ///
  /// One sliding in or out is drawn off its motion's rect, by a track of
  /// its own, so is never on one clock with another. A held motion (an
  /// entering or leaving tile's) never overshoots, whatever its easing.
  fn meets(&self, other: &Self) -> bool {
    let is_moving =
      |entry: &Self| matches!(entry.role, Role::Moving { .. });
    let (Some(a), Some(b)) = (self.lane(), other.lane()) else {
      return false;
    };

    let on_clock = |entry: &Self| {
      is_moving(entry)
        && entry
          .transit
          .as_ref()
          .is_none_or(|transit| transit.style.is_stationary())
    };
    let same_clock = on_clock(self)
      && on_clock(other)
      && self.motion.shares_clock(&other.motion);
    let overshoots = [self, other].into_iter().any(|entry| {
      is_moving(entry)
        && entry.motion.start_rect != entry.motion.target_rect
        && entry.motion.easing.can_overshoot()
    });
    lanes_meet(&a, &b, same_clock, overshoots)
  }

  /// The lane its tile draws along from now on, on a surface: its
  /// tenancy's while it moves, with what its transit reaches while it
  /// enters or leaves (see [`Transit::reach`]), and where it landed once
  /// it has.
  fn lane(&self) -> Option<Lane> {
    let tenancy = self.tenancy.as_ref()?;

    Some(match (self.role, &self.transit) {
      (Role::Moving { .. } | Role::Leaving(_), Some(transit)) => {
        let reach = |rect: &Rect| transit.reach(rect);
        tenancy.reach.lane(
          &reach(&self.motion.start_rect),
          &reach(&self.motion.target_rect),
        )
      }
      (Role::Moving { .. } | Role::Leaving(_), None) => {
        tenancy.lane.clone()
      }
      _ => tenancy.lane.landed(),
    })
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
  fn draw(&mut self, rect: &Rect, now: Instant) {
    if self.transit.is_none() && self.draw_stretched(rect, now) {
      self.rect = rect.clone();
      return;
    }

    let frame = self
      .transit
      .as_ref()
      .map(|transit| transit.frame_at(rect, now));
    let opacity = self.opacity;

    for (peek, clip) in self.peeks_mut() {
      match &frame {
        None => peek.place(rect, clip, opacity),
        Some(Some((frame, frac))) => {
          peek.place_scaled(rect, frame, clip, opacity, *frac);
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
  fn draw_stretched(&mut self, rect: &Rect, now: Instant) -> bool {
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
          let _scope = perf::scope(Stage::PeekFill);
          peek.carry_fill(color);
        }
        peek.place_filled(sampled, rect, clip, self.opacity);
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
      );
      drawn.get_or_insert(cover);
    }
    if let Some(cover) = drawn {
      stretch.cover = cover;
    }
    true
  }
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
/// it is at `now` and started then, so it doesn't jump, else `fresh`,
/// from all the way in.
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

/// How long the transit a tile enters by runs on, `left` of it to go,
/// once the tile's motion is retargeted to last `motion_ms` from now:
/// stretched to end with the motion, so the window zooms (or slides) in
/// once into its new tile rather than reaching its old one first and then
/// growing into the new one, a second zoom. `None` keeps it as it is: it
/// ends with the motion or after it, or `is_spring`, which sets its own
/// duration.
fn retimed_transit_ms(
  left: Duration,
  motion_ms: u32,
  is_spring: bool,
) -> Option<u32> {
  (!is_spring && u128::from(motion_ms) > left.as_millis())
    .then_some(motion_ms)
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

/// Whether a tile's trail, `hosted` or not, draws on for a tick once the
/// tile has landed (see [`PeekView::tick`]), so no frame shows neither it
/// nor what takes over from it. One of its own goes at once, as it always
/// has.
fn keeps_trail_drawing(hosted: bool) -> bool {
  hosted
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
  /// sampled again.
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
  /// Not on a peek, so it goes at once too: one already drawing the
  /// window in a role that doesn't turn round (handing back, releasing, a
  /// partly visible column's at rest), no monitor or frame, or one that
  /// couldn't be made.
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
  /// Columns whose resting peeks stand by under a workspace switch's
  /// surrogates (see `WindowPeek::stand_by`).
  standing_by: HashSet<Uuid>,
  /// Resting peeks of columns sliding out with a workspace switch, kept
  /// at rest until their surrogates take over.
  switching_out: HashMap<Uuid, Entry>,
  /// Peeks of columns that `upsert` replaced with a peek elsewhere (on
  /// another surface or a window of their own), still drawing until the
  /// scroll view's flush has composited what replaced them (see
  /// [`drop_replaced`](Self::drop_replaced)).
  replaced: Vec<WindowPeek>,
  /// Peeks of landing tiles that `move_tile` replaced with one on another
  /// surface, and hosted trails of tiles just landed (see
  /// [`keeps_trail_drawing`]), still drawing until the next tick, by when
  /// what replaced them has been composited.
  replaced_tiles: Vec<WindowPeek>,
  /// The next [`Rank::order`](surfaces::Rank::order).
  next_rank: u64,
  /// Raise passes closed so far (see
  /// [`Rank::pass`](surfaces::Rank::pass)).
  raise_passes: u64,
  /// Tiles' peeks kept for their next motions, by window (see
  /// [`Ready`]). Before `surfaces`, which they are on.
  ready: HashMap<Uuid, Ready>,
  /// When [`prewarm_ready`] is next due, if it is.
  prewarm_at: Option<Instant>,
  /// Windows [`prewarm_ready`] could make no ready peek of since it was
  /// last scheduled.
  prewarm_refused: HashSet<Uuid>,
  /// One per clip that tiles were drawn on, kept
  /// between motions. Last, so the peeks hosted on them go first.
  surfaces: Vec<PeekSurface>,
  /// One per monitor rect that windows shown in the always-on-top band
  /// were drawn on (see [`Layer::TopBand`](surfaces::Layer::TopBand)),
  /// kept as `surfaces` are.
  band_surfaces: Vec<PeekSurface>,
  /// One per working area that workspace switches drew on (see
  /// [`switch_surface`](Self::switch_surface)). Their tenants live in
  /// the switch, and hold their windows.
  switch_surfaces: Vec<PeekSurface>,
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

  /// Whether `window_id`'s peek draws its stack's tab bar in the bar's
  /// place, composited, so the bar itself is cloaked meanwhile (see
  /// `tab_bars::conceal_copied_bars`).
  pub fn conceals_tab_bar(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| conceals_bar(entry.bar))
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

  /// Whether `window_id`'s cloak waits for a `DwmFlush` first, so its
  /// peek's thumbnails are composited before its window goes: unless it
  /// is [`is_composited`](Self::is_composited) already, or a ready peek's
  /// that registered nothing since it was reclaimed (see
  /// [`kept_since_ready`]), whose thumbnails DWM has held, populated,
  /// since prewarm or its last landing (proposal 1). A window without
  /// such a peek always waits.
  pub fn needs_flush(&self, window_id: &Uuid) -> bool {
    !self.is_composited(window_id)
      && !self.entries.get(window_id).is_some_and(|entry| {
        matches!(entry.role, Role::Moving { .. })
          && entry.trail.is_none()
          && kept_since_ready(
            entry.ready_mark,
            entry.peek.stacking().map(|(_, last)| last),
          )
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
    // Never raised: its surface goes above its window as it lands.
    if let Some(tenancy) = entry.tenancy.as_mut().filter(|_| is_held) {
      tenancy.never_raised = true;
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

  /// Whether anything still needs animation ticks: a replaced tile's peek
  /// too, which only a tick drops, even once its entry is gone.
  pub fn is_active(&self) -> bool {
    !self.replaced_tiles.is_empty()
      || self.entries.values().any(|entry| entry.role != Role::Peek)
  }

  /// Advances moving columns to `now`, returning the ones that landed.
  /// Their moves are queued into `batch`, for [`track_overlays`] to commit
  /// along with their overlays'.
  #[allow(clippy::too_many_lines)]
  pub fn tick(&mut self, now: Instant, clock_origin: Instant) -> Landed {
    let mut landed = Landed::default();

    // A later composition than the one copies were put over rings and
    // backdrops in, and peeks put up in place of others.
    self.note_composited();
    self.replaced_tiles.clear();

    for (id, entry) in &mut self.entries {
      match entry.role {
        Role::Moving { ends_shown } => {
          // Its real window was cloaked a tick ago at least, so needs its
          // alpha no more. One not taken over from, e.g. as it wasn't
          // redrawn, is drawn all the same.
          let draw_scope = perf::scope(Stage::PeekDraw);
          for (peek, _) in entry.peeks_mut() {
            let alpha_scope = perf::scope(Stage::PeekDrawAlpha);
            peek.restore_alpha();
            drop(alpha_scope);
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
          let entry_scope = perf::scope(Stage::PeekDrawEntry);
          entry.draw(&rect, now);
          drop(entry_scope);
          drop(draw_scope);

          if entry.may_land(now) {
            // Landed on its own peek's monitor, it has nothing left on
            // another but what reaches past its tile, e.g. its ring at
            // the edge. A hosted trail draws that on until the next tick,
            // by when the real ring revealed under the copies has been
            // composited. Its window's alpha goes back now, as a dropped
            // one's would, before the hand-back sets it aside again.
            if let Some(mut trail) = entry.trail.take() {
              if keeps_trail_drawing(trail.peek.is_hosted()) {
                trail.peek.restore_alpha();
                self.replaced_tiles.push(trail.peek);
              }
            }
            if entry.stretch.as_ref().is_some_and(|stretch| stretch.filled)
            {
              landed.filled.push(*id);
            }
            entry.role = if ends_shown {
              // Raised while its real window is still cloaked, so the
              // uncloak never shows the window over the peek.
              entry.peek.begin_hand_back();
              // A column's surface is not raised over the tiles it draws,
              // nor a held tile's, and one raised in a band may be under
              // its window.
              if entry.is_column
                || entry
                  .tenancy
                  .as_ref()
                  .is_some_and(Tenancy::lifts_at_landing)
              {
                entry.peek.stay_over_window();
              }
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
          let draw_scope = perf::scope(Stage::PeekDraw);
          match entry.departing.as_mut() {
            Some(departing) if departing.awaits_cloak => {
              entry.peek.take_over(&departing.native);
              let cloak_scope = perf::scope(Stage::TakeOverCloak);
              match departing.native.set_cloaked(true) {
                Ok(()) => {
                  tracing::debug!("Cloaked leaving {id} under its peek.");
                }
                Err(err) => {
                  tracing::warn!("Failed to cloak leaving {id}: {err}");
                }
              }
              drop(cloak_scope);
              departing.awaits_cloak = false;
              // Drawn over its cloaked window from now on, never raised.
              entry.peek.hold_place();
            }
            departing => {
              let _scope = perf::scope(Stage::PeekDrawAlpha);
              // A closing one's only once it reads cloaked: back while
              // the cloak hasn't landed, a semi-transparent window would
              // show at its full size beside its shrinking peek. A
              // minimizing one isn't cloaked (the OS minimizes it).
              let is_closing =
                leaves_as(entry.role, AwayTransition::Close);
              let is_cloaked = || {
                departing.is_none_or(|departing| {
                  departing.native.is_cloaked().unwrap_or(true)
                })
              };
              if !is_closing || !entry.peek.hides_alpha() || is_cloaked() {
                entry.peek.restore_alpha();
              } else {
                tracing::debug!(
                  "Alpha of leaving {id} kept hidden: not cloaked yet."
                );
              }
            }
          }
          for (peek, _) in entry.peeks_mut() {
            peek.show_window();
            peek.refresh_companion();
          }
          if let Some(transit) = &entry.transit {
            transit.clock.start_clock(clock_origin);
          }
          let (rect, _) = entry.motion.current_state_at(now);
          let entry_scope = perf::scope(Stage::PeekDrawEntry);
          entry.draw(&rect, now);
          drop(entry_scope);
          drop(draw_scope);
        }
        Role::Peek | Role::HandingBack => {}
      }
    }

    for id in &landed.released {
      if let Some(entry) = self.entries.remove(id) {
        self.keep_ready(*id, entry, now);
      }
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
    // Their neighbours may rest on peeks that went ready before.
    if !landed.left.is_empty() {
      self.schedule_prewarm(now);
    }

    // Columns that moved out of view have nothing left to show.
    self.entries.retain(|_, entry| {
      entry.role != Role::Peek || entry.peek.visible_rect().is_some()
    });
    // Its draws may have registered a tenant's fill or companion.
    self.restack_crossings();
    // The last moving tenant may have landed or gone.
    self.settle_resting_surfaces();

    landed
  }

  /// Notes that the copies put over real rings and backdrops have been
  /// composited since, so those can go (see [`Handover::Covered`]).
  pub(super) fn note_composited(&mut self) {
    for entry in self.entries.values_mut() {
      entry.composited = true;
      for handover in
        [&mut entry.ring, &mut entry.backdrop, &mut entry.bar]
      {
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

  /// Has `window_id`'s resting peek stand by from now on, drawn too faint
  /// to see, as a workspace switch's surrogate draws it (see
  /// `WindowPeek::stand_by`). Its overlays are left to the switch.
  pub(super) fn stand_by(&mut self, window_id: Uuid) {
    self.standing_by.insert(window_id);
    if let Some(entry) = self.entries.get_mut(&window_id) {
      entry.peek.stand_by();
    }
  }

  /// Has `window_id`'s peek draw its window again after
  /// [`stand_by`](Self::stand_by). No-op for one not standing by.
  pub fn end_stand_by(&mut self, window_id: &Uuid) {
    if !self.standing_by.remove(window_id) {
      return;
    }
    if let Some(entry) = self.entries.get_mut(window_id) {
      entry.peek.end_stand_by();
    }
  }

  /// Keeps `entry`, the resting peek of a column sliding out with a
  /// workspace switch, drawn where it is until its surrogate takes over
  /// (see [`drop_switched_out`](Self::drop_switched_out)).
  pub(super) fn keep_switching_out(
    &mut self,
    window_id: Uuid,
    entry: Entry,
  ) {
    self.standing_by.remove(&window_id);
    self.switching_out.insert(window_id, entry);
  }

  /// Whether `window_id`'s resting peek is kept for its surrogate to take
  /// over from (see [`keep_switching_out`](Self::keep_switching_out)).
  pub fn is_switching_out(&self, window_id: &Uuid) -> bool {
    self.switching_out.contains_key(window_id)
  }

  /// Drops `window_id`'s kept resting peek, once its surrogate is drawn.
  pub fn drop_switched_out(&mut self, window_id: &Uuid) {
    self.switching_out.remove(window_id);
  }

  /// Takes out the peeks of windows entering a tiling layout that
  /// `shown` no longer has on screen: unmanaged (`None`), or not shown
  /// (`Some(false)`). Columns are left to
  /// [`take_departed`](Self::take_departed).
  pub(super) fn take_strays(
    &mut self,
    shown: impl Fn(&Uuid) -> Option<bool>,
  ) -> Vec<Uuid> {
    let strays = self
      .entries
      .extract_if(|id, entry| {
        is_stray(entry.is_column, entry.role, shown(id))
      })
      .map(|(id, _)| id)
      .collect::<Vec<_>>();
    // A resting column may be left alone on a surface they shared, with no
    // tick to come to settle it.
    if !strays.is_empty() {
      self.settle_resting_surfaces();
    }
    strays
  }

  /// Drops `window_id`'s peek, if it is entering a tiling layout, and its
  /// ready one, as its window is unmanaged.
  pub fn forget_entering(&mut self, window_id: &Uuid) {
    self.ready.remove(window_id);
    if self.entries.get(window_id).is_some_and(|entry| {
      !entry.is_column && !matches!(entry.role, Role::Leaving(_))
    }) {
      self.entries.remove(window_id);
      self.settle_resting_surfaces();
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
      self.settle_resting_surfaces();
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
        .filter(|entry| entry.role == Role::Peek)
        .filter(|_| !self.standing_by.contains(id))?
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
  /// A new peek is drawn on the `PeekSurface` of `clip` if its role keeps
  /// it there (see
  /// [`keeps_rest_hosted`]), the column's ready peek there if it has one
  /// and its window shows; a surface made for it drops those left unused
  /// off `monitor_clips`. A hosted peek no longer drawn so (another clip,
  /// a surface dropped, or not to be hosted any more) is replaced by a new
  /// one, its motion, transit and stretch carried over; the old one draws
  /// on until the flush that follows (see
  /// [`drop_replaced`](Self::drop_replaced)), so no frame shows neither.
  /// One of its own stays so.
  ///
  /// Returns whether a new peek now covers a still-visible real window,
  /// or replaces one, so it has to be composited before that window is
  /// cloaked or the old one goes.
  #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
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
    monitor_clips: &[Rect],
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
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
    let (opacity, _, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    let styles = carried_styles(window, config);
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

    let id = window.id();
    let host =
      keeps_rest_hosted(rests(role), self.rests_on_surface(clip, &id));
    let moves_on = self.moves_on_in_place(&id, clip, host);

    if let Some(entry) = self.entries.get_mut(&id).filter(|_| moves_on) {
      if opening.is_some() || entry.transit.is_some() {
        tracing::debug!(
          "Column {window} moved on in place to {:?}: its transit carries \
           on, none started again.",
          motion.target_rect,
        );
      }
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
      entry
        .peek
        .carry_styles(&styles.backdrops, styles.ring_width);
      if covers_shown_window {
        entry.composited = false;
      }
      entry.hold = if held { Hold::Held } else { Hold::Free };
      if self.standing_by.contains(&window.id()) {
        entry.peek.stand_by();
      }
      if let Some(tenancy) = &mut entry.tenancy {
        tenancy.lane = tenancy
          .reach
          .lane(&entry.motion.start_rect, &entry.motion.target_rect);
      }
      let (rect, _) = entry.motion.current_state_at(now);
      entry.draw(&rect, now);
      return covers_shown_window;
    }

    // Drawn as a column, its window has no ready peek of a tile's left,
    // and one on a surface only for a window still shown, which a ready
    // peek rejoins its surface by.
    let ready = if host {
      self.take_ready(
        &id,
        &ReadyFit {
          source: window.native().hwnd(),
          clip: clip.clone(),
        },
      )
    } else {
      self.ready.remove(&id);
      None
    };
    // Put on top as its own peek would be: a focused or opening column's
    // went up just below its window, above its neighbours'.
    let stacking = if is_focused || opening.is_some() {
      Stacking::Raised
    } else {
      Stacking::InPass
    };
    let tenancy = if host {
      self.tenancy(
        window,
        &motion,
        clip,
        Host::Tiles,
        stacking,
        config,
        monitor_clips,
      )
    } else {
      None
    };
    let surface = tenancy
      .as_ref()
      .and_then(|(index, _)| self.surfaces.get(*index));
    let ready = ready.filter(|peek| {
      surface.is_some_and(|surface| surface.hwnd() == peek.hwnd())
        && !window.native().is_cloaked().unwrap_or(true)
    });
    let made = match ready {
      Some(peek) => Some(reuse_entry(
        peek, window, motion, clip, opacity, &styles, role,
      )),
      None => {
        new_entry(window, motion, clip, opacity, &styles, role, surface)
      }
    };
    let Some((mut entry, covers_shown_window)) = made else {
      return false;
    };

    let old = self.entries.remove(&id).map(|old| {
      self.replaced.push(old.peek);
      (old.transit, old.stretch, old.motion.target_rect)
    });
    let replaces = old.is_some();
    if let Some((transit, stretch, headed)) = old {
      // Carried on as it was drawn, as if moved on in place.
      entry.transit = kept_transit(transit, is_moving);
      let resizes = stretch
        .as_ref()
        .filter(|_| is_moving)
        .map(Stretch::resize)
        .or(resizes)
        .filter(|_| entry.transit.is_none());
      entry.stretch = restretched(
        stretch,
        resizes,
        deferred,
        actual,
        &headed,
        &entry.motion.target_rect,
        now,
      );
    } else {
      entry.transit = opening.filter(|_| is_moving);
      entry.stretch =
        resizes.map(|resize| Stretch::at(actual(), resize, deferred));
    }
    if held {
      entry.hold = Hold::Held;
    }
    if self.standing_by.contains(&window.id()) {
      entry.peek.stand_by();
    }
    entry.tenancy = tenancy.map(|(_, tenancy)| tenancy);
    let rect = if replaces {
      entry.motion.current_state_at(now).0
    } else {
      entry.rect.clone()
    };
    entry.draw(&rect, now);
    self.entries.insert(id, entry);

    covers_shown_window || replaces
  }

  /// Whether `window_id`'s entry, if any, is moved on in place by
  /// [`upsert`](Self::upsert) as a column cut at `clip`, to be hosted or
  /// not (`host`): one of its own always, and a column's on a surface
  /// still kept that spans `clip`, if it is to stay hosted. A tile's on a
  /// surface takes no column's role (see [`Tenancy`]).
  fn moves_on_in_place(
    &self,
    window_id: &Uuid,
    clip: &Rect,
    host: bool,
  ) -> bool {
    self.entries.get(window_id).is_some_and(|entry| {
      !entry.peek.is_hosted()
        || (host
          && entry.is_column
          && entry.clip == *clip
          && self
            .surfaces
            .iter()
            .any(|surface| surface.hwnd() == entry.peek.hwnd()))
    })
  }

  /// Whether a column other than `window_id`'s rests, or is to, on the
  /// surface of `clip` (see [`keeps_rest_hosted`]). One kept for a
  /// switch's surrogate does until that takes over later in the sync: a
  /// column joining meanwhile would move the surface it draws on at full
  /// (see [`keep_switching_out`](Self::keep_switching_out)).
  fn rests_on_surface(&self, clip: &Rect, window_id: &Uuid) -> bool {
    let Some(surface) =
      self.surfaces.iter().find(|surface| surface.clip() == clip)
    else {
      return false;
    };

    self
      .entries
      .iter()
      .chain(&self.switching_out)
      .any(|(id, entry)| {
        id != window_id
          && entry.tenancy.is_some()
          && entry.peek.hwnd() == surface.hwnd()
          && rests(entry.role)
      })
  }

  /// Drops the peeks [`upsert`](Self::upsert) replaced, once what replaced
  /// them has been composited.
  pub(super) fn drop_replaced(&mut self) {
    self.replaced.clear();
  }

  /// Draws `window`, just opened or restored at `target`, entering on a
  /// pinned peek by `params`' style, opacity and timing, until it is
  /// handed back.
  ///
  /// With `host`, it is drawn on that `PeekSurface` of `clip`, ranked over
  /// every tile its pass and earlier ones moved (see
  /// [`Rank`](surfaces::Rank)), or over them for one raised in its band;
  /// a surface made for it drops those left unused off `monitor_clips`.
  ///
  /// Returns `false`, creating nothing, for a window already drawn, or a
  /// transition with nothing to show (see [`Transit::opening`]).
  #[allow(clippy::too_many_arguments)]
  pub(super) fn enter(
    &mut self,
    window: &WindowContainer,
    target: &Rect,
    clip: &Rect,
    params: &WindowTransitionParams,
    host: Option<Host>,
    monitor_clips: &[Rect],
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
  ) -> bool {
    let styles = carried_styles(window, config);
    if !self.can_enter(window) {
      tracing::debug!(
        "Opening of {window} not started again: its peek draws it."
      );
      return false;
    }
    let Some(transit) = Transit::opening(params) else {
      return false;
    };
    tracing::debug!(
      "Opening of {window} started at {target:?} ({:?}, {} ms).",
      params.style,
      params.duration_ms,
    );

    let (opacity, _, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    // Held at its tile: only the transit moves it in.
    let motion = WindowAnimationState::new_movement(
      target.clone(),
      target.clone(),
      params.duration_ms,
      params.easing.clone(),
    );

    self.ready.remove(&window.id());
    let tenancy = host.and_then(|host| {
      self.tenancy(
        window,
        &motion,
        clip,
        host,
        Stacking::Transit,
        config,
        monitor_clips,
      )
    });
    let surface = self.hosting(host, tenancy.as_ref());
    let Some((mut entry, _)) = new_entry(
      window,
      motion,
      clip,
      opacity,
      &styles,
      Role::Moving { ends_shown: true },
      surface,
    ) else {
      return false;
    };

    entry.transit = Some(transit);
    entry.is_column = false;
    entry.tenancy = tenancy.map(|(_, tenancy)| tenancy);
    let is_hosted = entry.tenancy.is_some();
    entry.draw(target, now);
    self.entries.insert(window.id(), entry);
    // Registered last, it is over every tile it ranks above; a kept
    // registration of a tile ranked over it may not be.
    if is_hosted {
      self.restack_crossings();
    }

    true
  }

  /// Whether [`enter`](Self::enter) can draw `window`: not one already
  /// drawn.
  fn can_enter(&self, window: &WindowContainer) -> bool {
    !self.draws_window(&window.id())
  }

  /// Draws `window`, a tile moving from `from` to `target`, on a peek
  /// over `duration_ms` and `easing`, until it is handed back. A peek
  /// still handing it back after an earlier move or entry takes it on
  /// again from where it landed, as [`upsert`](Self::upsert) does for a
  /// column. With `stretch`, a change of size scales the window (see
  /// [`Stretch`]); without, it keeps the same size.
  ///
  /// With `host`, a new peek is drawn on that `PeekSurface` of `clip` if
  /// that takes it (see [`tenancy`](Self::tenancy)), the tile's ready
  /// peek there if it has one (see [`Ready`]); a surface made for it
  /// drops those left unused off `monitor_clips`. One landing keeps where
  /// it is drawn, unless it is on a surface that the motion isn't to be
  /// drawn on: a new one replaces it then, the old one drawing until the
  /// next tick, so no frame shows neither.
  ///
  /// `ring`: the window's ring overlay and its corner reach, when it has
  /// one already (see [`existing_ring`](overlays::existing_ring)). A new
  /// hosted peek then registers its ring copy with its window and
  /// backdrop (see `WindowPeek::reserve_ring`), for
  /// [`carry_ring`](overlays::carry_ring) to take as it covers the ring,
  /// so the tile's thumbnails stack together and
  /// [`restack_crossings`](Self::restack_crossings) finds nothing to
  /// re-register for it.
  ///
  /// Returns `false`, creating nothing, for a window already drawn
  /// otherwise (moving, leaving, or as a column), or one DWM can't draw.
  #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
  pub(super) fn move_tile(
    &mut self,
    window: &WindowContainer,
    from: &Rect,
    target: Rect,
    clip: &Rect,
    duration_ms: u32,
    easing: EasingFunction,
    resize: Option<PeekResize>,
    host: Option<Host>,
    ring: Option<(HWND, i32)>,
    monitor_clips: &[Rect],
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
  ) -> bool {
    let id = window.id();
    let styles = carried_styles(window, config);
    let resizes = resize.filter(|_| !is_same_size(from, &target));
    let deferred = !resizes_at_start(from, &target);
    let landing = match self.entries.get(&id) {
      None => false,
      Some(entry)
        if !entry.is_column
          && matches!(
            entry.role,
            Role::HandingBack | Role::Releasing { .. }
          ) =>
      {
        !entry.peek.is_hosted()
          || host.is_some_and(|host| {
            self
              .surface_at(host.layer(), clip)
              .is_some_and(|surface| surface.hwnd() == entry.peek.hwnd())
          })
      }
      Some(entry) => {
        if entry.transit.is_some() {
          tracing::debug!(
            "Move of {window} left to its transit, which carries on."
          );
        }
        return false;
      }
    };
    let (opacity, _, _) =
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
      entry
        .peek
        .carry_styles(&styles.backdrops, styles.ring_width);
      if covers_shown_window {
        entry.composited = false;
      }
      // Raised by this command, it goes on top as its own peek would.
      let stacking = host
        .map_or(Stacking::Raised, |host| host.stacking(Stacking::Raised));
      self.keep_leaving_on_top();
      let Some(entry) = self.entries.get_mut(&id) else {
        return false;
      };
      let rank = rank(stacking, self.raise_passes, self.next_rank);
      if let Some(tenancy) = &mut entry.tenancy {
        tenancy.lane = tenancy.reach.lane(from, &entry.motion.target_rect);
        tenancy.rank = rank;
        tenancy.never_raised = false;
        tenancy.in_band = matches!(host, Some(Host::InBand(_)));
        self.next_rank += 1;
      }
      let place_scope = perf::scope(Stage::PeekPlace);
      entry.draw(from, now);
      drop(place_scope);
      // Its registrations kept, it may be under tiles now ranked below it.
      self.restack_crossings();
      return true;
    }

    // Any ready peek goes unless this motion takes it.
    let wanted = ReadyFit {
      source: window.native().hwnd(),
      clip: clip.clone(),
    };
    let kept = self.ready.get(&id).map(|ready| ready.fit.clone());
    let ready = self.take_ready(&id, &wanted);
    let tenancy = host.and_then(|host| {
      self.tenancy(
        window,
        &motion,
        clip,
        host,
        Stacking::Raised,
        config,
        monitor_clips,
      )
    });
    let surface = self.hosting(host, tenancy.as_ref());
    let ready = ready.filter(|peek| {
      surface.is_some_and(|surface| surface.hwnd() == peek.hwnd())
    });
    if host.is_some() {
      note_ready_take(window, kept.as_ref(), &wanted, ready.is_some());
    }
    let made = match ready {
      Some(peek) => Some(reuse_entry(
        peek, window, motion, clip, opacity, &styles, role,
      )),
      None => {
        new_entry(window, motion, clip, opacity, &styles, role, surface)
      }
    };
    let Some((mut entry, _)) = made else {
      return false;
    };
    // Nothing to draw it with (e.g. an elevated window): it snaps into
    // place.
    if !entry.peek.has_thumbnail() {
      tracing::debug!("No peek of {window} to move on.");
      return false;
    }

    entry.is_column = false;
    entry.tenancy = tenancy.map(|(_, tenancy)| tenancy);
    entry.stretch =
      resizes.map(|resize| Stretch::at(actual(), resize, deferred));
    // Cut as `carry_ring` cuts it: in cells while the window's size
    // changes on its peek.
    if let Some((source, corner_reach)) = ring {
      let corner_reach = entry.stretch.is_some().then_some(corner_reach);
      entry.peek.reserve_ring(source, corner_reach);
    }
    let place_scope = perf::scope(Stage::PeekPlace);

    entry.draw(from, now);
    drop(place_scope);
    // A landing one it replaces, on another surface, draws until this one
    // has been composited.
    if let Some(replaced) = self.entries.insert(id, entry) {
      self.replaced_tiles.push(replaced.peek);
    }
    // A ready peek's kept registrations may be under tiles ranked below
    // it; a new one's are over every other.
    self.restack_crossings();

    true
  }

  /// Has `window`'s peek, just put up by [`move_tile`](Self::move_tile)
  /// on the monitor it moves onto, drawn on `clip`, the monitor it comes
  /// from, too (see [`Trail`]). Returns whether it is: if not, the peek
  /// is dropped, as it would cut the window at its monitor's edge.
  ///
  /// With `host`, and the own peek hosted, it is drawn on that layer's
  /// surface of `clip`, on `ready`, the tile's ready peek, if that is on
  /// it; a surface made for it drops those left unused off
  /// `monitor_clips`. It shares the own peek's tenancy: one rank and lane
  /// for both parts, each restacked among its own surface's tenants.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_trail(
    &mut self,
    window: &WindowContainer,
    clip: &Rect,
    host: Option<Host>,
    ready: Option<WindowPeek>,
    monitor_clips: &[Rect],
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
  ) -> bool {
    let id = window.id();
    let Some((motion, role, is_hosted)) =
      self.entries.get(&id).map(|entry| {
        (entry.motion.clone(), entry.role, entry.tenancy.is_some())
      })
    else {
      return false;
    };

    // One that can't be made leaves the trail on a window of its own.
    let layer = host.filter(|_| is_hosted).map(Host::layer);
    let index = layer
      .and_then(|layer| self.surface_index(layer, clip, monitor_clips));
    let surface = layer
      .zip(index)
      .and_then(|(layer, index)| self.layer_surfaces(layer).get(index));
    let ready = ready.filter(|peek| {
      surface.is_some_and(|surface| surface.hwnd() == peek.hwnd())
    });

    let styles = carried_styles(window, config);
    let (opacity, _, _) =
      surrogate_effects_for(window.effect_set(is_focused), config);
    let made = match ready {
      Some(peek) => Some(reuse_entry(
        peek, window, motion, clip, opacity, &styles, role,
      )),
      None => {
        new_entry(window, motion, clip, opacity, &styles, role, surface)
      }
    };
    let Some((trail, _)) = made else {
      self.entries.remove(&id);
      return false;
    };
    let Some(entry) = self.entries.get_mut(&id) else {
      return false;
    };

    let restacks = trail.peek.is_hosted();
    entry.trail = Some(Trail {
      peek: trail.peek,
      clip: clip.clone(),
    });
    let rect = entry.rect.clone();
    entry.draw(&rect, now);
    // A ready peek's kept registrations may be under tiles ranked below
    // it; a new one's are over every other.
    if restacks {
      self.restack_crossings();
    }
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
  ///
  /// On a surface, it ranks as a restored tile, put on top as it comes
  /// back (see [`Rank`](surfaces::Rank)), so it is stacked over the tiles
  /// it may meet.
  pub(super) fn turn_round(
    &mut self,
    window_id: &Uuid,
    params: &WindowTransitionParams,
    is_column: bool,
    now: Instant,
  ) -> bool {
    let is_hosted = match self.entries.get(window_id) {
      Some(entry)
        if leaves_as(entry.role, AwayTransition::Minimize)
          && entry.transit.is_some() =>
      {
        entry.tenancy.is_some()
      }
      _ => return false,
    };
    let rank = is_hosted.then(|| self.take_rank(Stacking::Transit));
    let Some(entry) = self.entries.get_mut(window_id) else {
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
    if let Some((tenancy, rank)) = entry.tenancy.as_mut().zip(rank) {
      tenancy.lane = tenancy.reach.lane(&current, &current);
      tenancy.rank = rank;
    }
    if is_hosted {
      self.restack_crossings();
    }

    true
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
    // An entering one zooms or slides on into its new tile, never anew.
    if let Some(transit) = entry
      .transit
      .as_mut()
      .filter(|transit| transit.direction == Direction::In)
    {
      let motion_ms =
        u32::try_from(motion.duration.as_millis()).unwrap_or(u32::MAX);
      let retimed = retimed_transit_ms(
        transit.clock.remaining_at(now),
        motion_ms,
        matches!(transit.clock.easing, EasingFunction::Spring(_)),
      );
      tracing::debug!(
        "Opening of {window_id} retargeted to {:?}: transit {}.",
        motion.target_rect,
        retimed.map_or_else(
          || "carries on".to_string(),
          |ms| format!("stretched to end with it in {ms} ms"),
        ),
      );
      if let Some(ms) = retimed {
        *transit = transit.retimed(ms, now);
      }
    }
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
    let is_hosted = entry.tenancy.as_mut().is_some_and(|tenancy| {
      tenancy.lane = tenancy
        .reach
        .lane(&entry.motion.start_rect, &entry.motion.target_rect);
      true
    });
    if is_hosted {
      self.restack_crossings();
    }
    true
  }

  /// Draws `window`, at `rect`, leaving by `params`' style, opacity and
  /// timing on a pinned peek, until [`tick`](Self::tick) reports it
  /// [`Landed::left`]. One already entering on its peek turns round where
  /// it is.
  ///
  /// A new peek goes up directly below its real window and is never
  /// raised: a closing window is drawn below its neighbours. Under a
  /// real window still shown, it is warmed up until the window is cloaked
  /// by the first tick, as it takes over.
  ///
  /// With `host`, it is drawn on the `PeekSurface` of `clip` instead,
  /// ranked over every tile there that moves (see
  /// [`Rank`](surfaces::Rank)), which a surface made for it drops those
  /// left unused off `monitor_clips` for. One already drawn stays where
  /// it is, re-ranked so unless it entered.
  #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
  fn leave(
    &mut self,
    window: &WindowContainer,
    rect: &Rect,
    clip: &Rect,
    transition: AwayTransition,
    params: &WindowTransitionParams,
    host: Option<Host>,
    monitor_clips: &[Rect],
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
  ) -> Leave {
    let id = window.id();
    // A new peek is its ready one if that fits; none is kept after.
    let wanted = ReadyFit {
      source: window.native().hwnd(),
      clip: clip.clone(),
    };
    let kept = self.ready.get(&id).map(|ready| ready.fit.clone());
    let ready = self.take_ready(&id, &wanted);
    let role = self.entries.get(&id).map(|entry| entry.role);
    if role.is_some_and(|role| matches!(role, Role::Leaving(_))) {
      return Leave::Leaving;
    }

    let styles = carried_styles(window, config);
    if role.is_some_and(|role| !matches!(role, Role::Moving { .. })) {
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
      let (opacity, _, _) = surrogate_effects_for(effect_set, config);
      let motion = held_motion(rect.clone(), params);
      let tenancy = host.and_then(|host| {
        self.tenancy(
          window,
          &motion,
          clip,
          host,
          Stacking::Transit,
          config,
          monitor_clips,
        )
      });
      let surface = self.hosting(host, tenancy.as_ref());
      let ready = ready.filter(|peek| {
        surface.is_some_and(|surface| surface.hwnd() == peek.hwnd())
      });
      if host.is_some() {
        note_ready_take(window, kept.as_ref(), &wanted, ready.is_some());
      }
      let role = Role::Leaving(transition);
      let made = match ready {
        Some(peek) => Some(reuse_entry(
          peek, window, motion, clip, opacity, &styles, role,
        )),
        None => {
          new_entry(window, motion, clip, opacity, &styles, role, surface)
        }
      };
      let Some((mut entry, _)) = made else {
        return Leave::Unsupported;
      };
      entry.tenancy = tenancy.map(|(_, tenancy)| tenancy);
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
    entry.draw(&current, now);
    // A tile that was moving goes over its neighbours as it starts to
    // leave, as a new leaving one does.
    let reranks = entry
      .tenancy
      .as_ref()
      .is_some_and(|tenancy| tenancy.rank.level != Level::Transit);
    if reranks {
      let rank = self.take_rank(Stacking::Transit);
      if let Some(tenancy) =
        self.entries.get_mut(&id).and_then(|e| e.tenancy.as_mut())
      {
        tenancy.rank = rank;
      }
      self.restack_crossings();
    }

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
    // Drawn where its window was from now on, never raised.
    peek.hold_place();
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

/// A hidden peek of `window` for `motion` in `role`, carrying copies of
/// its backdrop as `styles` say, and whether it covers a still-shown real
/// window, so it has to be composited before that window is cloaked.
/// `None` when the peek can't be created.
///
/// Drawn on `surface` if given, which then takes it (see
/// [`PeekView::tenancy`]), else on a window of its own.
#[allow(clippy::too_many_arguments)]
fn new_entry(
  window: &WindowContainer,
  motion: WindowAnimationState,
  clip: &Rect,
  opacity: u8,
  styles: &PeekStyles,
  role: Role,
  surface: Option<&PeekSurface>,
) -> Option<(Entry, bool)> {
  let _scope = perf::scope(Stage::PeekNew);

  // Sized like the window itself, which a collapsed `from` (an opening
  // column) isn't.
  let peek = match surface {
    // A window not known to be cloaked is taken to show, which keeps the
    // surface under it.
    Some(surface) => WindowPeek::hosted(
      &window.native(),
      &motion.target_rect,
      surface,
      !window.native().is_cloaked().unwrap_or(false),
    ),
    None => WindowPeek::new(&window.native(), &motion.target_rect, clip)
      .inspect_err(|err| {
        tracing::warn!("Failed to create a peek of {window}: {err}");
      })
      .ok()?,
  };

  Some(entry_on(peek, window, motion, clip, opacity, styles, role))
}

/// The entry of `peek`, just made or reclaimed for `window`, for `motion`
/// in `role`, carrying copies of its backdrop as `styles` say, and
/// whether it covers a still-shown real window (see [`new_entry`]).
fn entry_on(
  mut peek: WindowPeek,
  window: &WindowContainer,
  motion: WindowAnimationState,
  clip: &Rect,
  opacity: u8,
  styles: &PeekStyles,
  role: Role,
) -> (Entry, bool) {
  let is_moving = matches!(role, Role::Moving { .. } | Role::Leaving(_));

  // Before the ring, which goes on top. Shown in its style by
  // `track_overlays`, before anything is composited.
  let styles_scope = perf::scope(Stage::PeekStyles);
  peek.carry_styles(&styles.backdrops, styles.ring_width);
  drop(styles_scope);

  // A moving column's or a leaving window's real window may still show
  // until the peek takes over from it as it is cloaked; one at rest is
  // already cloaked.
  let covers_shown_window =
    is_moving && !window.native().is_cloaked().unwrap_or(true);
  if covers_shown_window {
    let _scope = perf::scope(Stage::PeekWarm);
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
    bar: Handover::None,
    composited: false,
    transit: None,
    is_column: true,
    departing: None,
    stretch: None,
    hold: Hold::Free,
    trail: None,
    tenancy: None,
    ready_mark: None,
  };

  (entry, covers_shown_window)
}

#[cfg(test)]
mod tests {
  use std::{
    collections::HashMap,
    time::{Duration, Instant},
  };

  use uuid::Uuid;
  use wm_common::{
    EasingFunction, WindowTransitionParams, WindowTransitionStyle,
  };
  use wm_platform::Rect;

  use super::{
    follows_tile, has_landed, has_left, is_departed, is_stray,
    keeps_trail_drawing, kept_transit, lands_on_trail, leaves_as,
    leaving_transit, motions::follow_duration, move_clips,
    retargeted_motion, retimed_transit_ms, returning_transit, take_left,
    Hold, Role, FILL_LANDING_WAIT, HOLD_RELEASE_WAIT,
  };
  use crate::animation::{
    state::WindowAnimationState,
    transit::{Direction, Transit},
    AwayTransition,
  };

  /// A hosted trail draws a tick past the landing; one of its own goes at
  /// once.
  #[test]
  fn only_a_hosted_trail_outlives_its_landing() {
    assert!(keeps_trail_drawing(true));
    assert!(!keeps_trail_drawing(false));
  }

  /// Retargeted for longer than it has left, an opening's transit is
  /// stretched to end with the motion; never cut short, nor a spring's.
  #[test]
  fn an_opening_transit_ends_with_its_retargeted_motion() {
    let left = Duration::from_millis(80);
    assert_eq!(retimed_transit_ms(left, 200, false), Some(200));
    assert_eq!(retimed_transit_ms(left, 80, false), None);
    assert_eq!(retimed_transit_ms(left, 50, false), None);
    assert_eq!(retimed_transit_ms(left, 200, true), None);
  }

  fn linear() -> EasingFunction {
    EasingFunction::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0)
  }

  pub(super) fn tile() -> Rect {
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
      away_scale: 0.0,
    }
  }

  pub(super) fn transit(duration_ms: u32) -> Transit {
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
  /// on a peek alone; a resize, or a move that isn't animated, doesn't.
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

  /// Done moving, a filled window sent its size waits for it at most
  /// `FILL_LANDING_WAIT`.
  #[test]
  fn a_filled_landing_waits_for_its_windows_size() {
    let now = Instant::now();
    let waiting = Hold::Released(now, FILL_LANDING_WAIT);

    assert!(!waiting.allows_landing(false, now + HOLD_RELEASE_WAIT));
    assert!(waiting.allows_landing(true, now));
    assert!(waiting.allows_landing(false, now + FILL_LANDING_WAIT));
  }
}
