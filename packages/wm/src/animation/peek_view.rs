//! Stand-ins drawing windows whose real windows are cloaked.

use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{
  EasingFunction, EffectSet, WindowState, WindowTransitionParams,
};
use wm_platform::{
  BackdropOverlayParams, BorderOverlayParams, CornerStyle,
  NativeBackdropOverlay, NativeBorderOverlay, NativeWindow, Overlay,
  Point, Rect, StyleBlend, SurrogateBatch, WindowPeek, HWND,
};

use super::{
  stand_in::{moved_along, StandIn},
  state::WindowAnimationState,
  transit::{transit_frame, Direction, Transit},
};
use crate::{
  animation::AwayTransition,
  commands::general::{
    focus_blend, overlay_entry, surrogate_effects_for,
    upsert_clipped_overlay, upsert_pinned_border_overlay,
  },
  layout::is_hidden_in_layout,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

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

impl Entry {
  fn place(&mut self, rect: &Rect, batch: &mut SurrogateBatch) {
    self.peek.place(rect, &self.clip, self.opacity, batch);
    self.rect = rect.clone();
  }

  /// Places the peek with its window at `rect`, zoomed, slid and faded by
  /// its transit at `now` if it has one.
  fn draw(
    &mut self,
    rect: &Rect,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) {
    let Some(transit) = &self.transit else {
      self.place(rect, batch);
      return;
    };

    match transit_frame(
      rect,
      &transit.style,
      transit.away_opacity,
      transit.amount_at(now),
    ) {
      Some((frame, frac)) => self.peek.place_scaled(
        rect,
        &frame,
        &self.clip,
        self.opacity,
        frac,
        batch,
      ),
      None => self.peek.hide(),
    }
    self.rect = rect.clone();
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

/// Whether an entry is a window entering a tiling layout, which
/// `platform_sync` retargets: not a column, which
/// [`PeekView::upsert`] retargets.
fn is_entering_tile(
  has_transit: bool,
  is_column: bool,
  is_moving: bool,
) -> bool {
  has_transit && !is_column && is_moving
}

/// `motion` headed for `target` instead, at the speed it has at `now`
/// (see `WindowAnimationState::retargeted`). One already drawn carries on
/// from where it was at `now`.
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
fn is_live(styles: &[Option<BackdropOverlayParams>]) -> bool {
  styles
    .iter()
    .any(|style| style.is_some_and(|params| !params.is_copyable()))
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
          entry.peek.restore_alpha();
          entry.peek.show_window();
          entry.peek.refresh_companion();
          entry.motion.start_clock(clock_origin);
          if let Some(transit) = &entry.transit {
            transit.clock.start_clock(clock_origin);
          }
          let (rect, _) = entry.motion.current_state_at(now);
          entry.draw(&rect, now, batch);

          if has_landed(&entry.motion, entry.transit.as_ref()) {
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
          entry.peek.show_window();
          entry.peek.refresh_companion();
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

  /// The window whose peek is at `point`, if any. Not one leaving, which
  /// is going and, closing, no longer managed.
  pub fn window_at(&self, point: &Point) -> Option<Uuid> {
    self.entries.iter().find_map(|(id, entry)| {
      Some(entry)
        .filter(|entry| !matches!(entry.role, Role::Leaving(_)))?
        .peek
        .visible_rect()
        .filter(|rect| rect.contains_point(point))
        .map(|_| *id)
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
    config: &UserConfig,
    is_focused: bool,
    now: Instant,
    batch: &mut SurrogateBatch,
  ) -> bool {
    let scroll_config = &config.value.animations.scroll;
    let is_moving = matches!(role, Role::Moving { .. });
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
      Some(entry) if is_moving => {
        let motion = entry.motion.retargeted(
          target,
          scroll_config.duration_ms,
          scroll_config.easing.clone(),
          now,
        );
        // Sampled at the wall clock and started on the next tick, it
        // repeated or skipped up to a frame of travel at every key press.
        if entry.motion.is_running() {
          motion.start_clock(now);
        }
        motion
      }
      _ => WindowAnimationState::new_movement(
        from.clone(),
        target,
        if is_moving {
          scroll_config.duration_ms
        } else {
          0
        },
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

    if let Some(entry) = self.entries.get_mut(&window.id()) {
      entry.motion = motion;
      entry.clip = clip.clone();
      entry.opacity = opacity;
      entry.role = role;
      entry.is_column = true;
      entry.transit = kept_transit(entry.transit.take(), is_moving);
      // Copies of both styles are drawn up front, so a focus change only
      // crossfades them (see `track_overlays`): one registered mid-move
      // left frames without it.
      entry.peek.carry_styles(&styles);
      if covers_shown_window {
        entry.composited = false;
      }
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

    true
  }

  /// Puts `window_id`'s peek on top, as the old path's stand-ins of an
  /// open or restore are (`place_at_top`): a restored window was at the
  /// bottom of the z-order, under the windows it comes back over, and its
  /// peek went up just below it. Not one leaving, which stays below its
  /// neighbours (task 4).
  pub(super) fn raise_restored(&self, window_id: &Uuid) {
    if let Some(entry) = self
      .entries
      .get(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
    {
      entry.peek.raise_to_top();
    }
  }

  /// Whether `window_id` is entering a tiling layout on its peek (see
  /// [`enter`](Self::enter)).
  pub fn is_entering(&self, window_id: &Uuid) -> bool {
    self.entries.get(window_id).is_some_and(|entry| {
      is_entering_tile(
        entry.transit.is_some(),
        entry.is_column,
        matches!(entry.role, Role::Moving { .. }),
      )
    })
  }

  /// Heads `window_id`'s peek, entering a tiling layout, for its new tile
  /// at `target`, at the speed it has at `now`, over `duration_ms` and
  /// `easing`. Its transit carries on untouched.
  pub fn retarget_entering(
    &mut self,
    window_id: &Uuid,
    target: Rect,
    duration_ms: u32,
    easing: EasingFunction,
    now: Instant,
  ) {
    if !self.is_entering(window_id) {
      return;
    }

    if let Some(entry) = self
      .entries
      .get_mut(window_id)
      .filter(|entry| entry.motion.target_rect != target)
    {
      entry.motion =
        retargeted_motion(&entry.motion, target, duration_ms, easing, now);
    }
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
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  entered
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
    state.animation_manager.peeks.raise_restored(&id);
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
  state.animation_manager.peeks.raise_restored(&id);
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
      // A releasing peek covers its real window, back on screen and with
      // the overlays behind it, which stay there once the peek goes.
      let anchor = match entry.role {
        Role::Releasing { .. } => state.window_by_id(*id).map_or_else(
          || entry.peek.hwnd(),
          |window| window.native().hwnd(),
        ),
        _ => entry.peek.hwnd(),
      };
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
      focus_blend::<NativeBorderOverlay>(state, id, set, config);

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

/// Keeps `id`'s pinned peek drawing its window's ring, with `params` or
/// not at all, and shadow around where the real window is, and hands the
/// ring over to its copy a step at a time (see [`Handover`]). Returns
/// whether it did: not when the copy failed.
///
/// Both follow the real window's own rect and frame, as steady state
/// places its ring (see `sync_overlays`), rather than where the layout
/// puts it: an app that keeps its window a little off that rect would
/// otherwise have its ring shift as it scrolls, and back once it lands.
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
    entry.peek.follow_window(&bounds, &frame);
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
  let ring_frame = window.as_ref().map_or_else(
    || frame.clone(),
    |window| crate::tab_bars::window_with_tab_bar(window, &frame),
  );
  let anchor = native.hwnd();

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
      if !entry.peek.carry_ring(overlay.hwnd(), &ring_rect, &bounds) {
        return false;
      }

      if is_shown {
        Handover::Covering
      } else {
        Handover::Parked
      }
    }
    Handover::Covering => Handover::Covering,
    Handover::Covered | Handover::Parked => {
      overlay.park(&ring_frame, anchor, true);
      let ring_rect = overlay.outer_rect();
      entry.peek.carry_ring(overlay.hwnd(), &ring_rect, &bounds);
      Handover::Parked
    }
  };

  true
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
      entry.peek.hide_backdrop();
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
    (native, frame)
  };
  let anchor = native.hwnd();

  let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id)
  else {
    return false;
  };
  let is_landing =
    matches!(entry.role, Role::HandingBack | Role::Releasing { .. });
  entry.peek.show_backdrop();
  entry.peek.show_style_blend(&blend);

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
fn peek_styles(
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
    EasingFunction, WindowTransitionParams, WindowTransitionStyle,
  };
  use wm_platform::Rect;

  use super::{
    has_landed, has_left, is_departed, is_entering_tile, is_stray,
    kept_transit, leaves_as, leaving_transit, retargeted_motion,
    returning_transit, take_left, Role,
  };
  use crate::animation::{
    state::WindowAnimationState,
    transit::{Direction, Transit},
    AwayTransition,
  };

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

  /// A column entering is retargeted by the scroll, never as a window
  /// entering a tiling layout.
  #[test]
  fn columns_are_not_entering_tiles() {
    assert!(is_entering_tile(true, false, true));
    assert!(!is_entering_tile(true, true, true));
    assert!(!is_entering_tile(false, false, true));
    assert!(!is_entering_tile(true, false, false));
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
}
