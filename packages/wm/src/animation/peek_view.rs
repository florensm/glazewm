//! Stand-ins drawing windows whose real windows are cloaked.

use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{EasingFunction, WindowState, WindowTransitionParams};
use wm_platform::{
  BackdropOverlayParams, BorderOverlayParams, CornerStyle,
  NativeBackdropOverlay, NativeBorderOverlay, NativeWindow, Overlay,
  Point, Rect, StyleBlend, SurrogateBatch, WindowPeek, HWND,
};

use super::{
  stand_in::{moved_along, StandIn},
  state::WindowAnimationState,
  transit::{transit_frame, Transit},
};
use crate::{
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

/// Whether [`PeekView::take_strays`] takes an entry: one not a column's,
/// whose window is unmanaged (`shown` is `None`) or not shown.
fn is_stray(is_column: bool, shown: Option<bool>) -> bool {
  !is_column && shown != Some(true)
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
        Role::Peek | Role::HandingBack => {}
      }
    }

    for id in &landed.released {
      self.entries.remove(id);
    }

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
      .extract_if(|id, entry| is_stray(entry.is_column, shown(id)))
      .map(|(id, _)| id)
      .collect()
  }

  /// Drops `window_id`'s peek, if it is entering a tiling layout, as its
  /// window is unmanaged.
  pub fn forget_entering(&mut self, window_id: &Uuid) {
    if self
      .entries
      .get(window_id)
      .is_some_and(|entry| !entry.is_column)
    {
      self.entries.remove(window_id);
    }
  }

  /// Whether any peek draws a window entering a tiling layout.
  fn has_entering(&self) -> bool {
    self.entries.values().any(|entry| !entry.is_column)
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

  /// The window whose peek is at `point`, if any.
  pub fn window_at(&self, point: &Point) -> Option<Uuid> {
    self.entries.iter().find_map(|(id, entry)| {
      entry
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
      // Copies of both styles are drawn up front, so a focus change only
      // crossfades them (see `track_overlays`): one registered mid-move
      // left frames without it.
      entry.peek.carry_styles(&styles);
      if covers_shown_window {
        entry.composited = false;
      }
      let (rect, _) = entry.motion.current_state_at(now);
      entry.place(&rect, batch);
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

    let rect = entry.rect.clone();
    entry.place(&rect, batch);
    self.entries.insert(window.id(), entry);

    covers_shown_window
  }

  /// Draws `window`, just opened at `target`, entering on a pinned peek
  /// by `params`' style, opacity and timing, until it is handed back.
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
    let Some(transit) = Transit::opening(params)
      .filter(|_| !is_live(&styles) && !self.draws_window(&window.id()))
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

  /// Whether `window_id` is entering a tiling layout on its peek (see
  /// [`enter`](Self::enter)).
  pub fn is_entering(&self, window_id: &Uuid) -> bool {
    self.entries.get(window_id).is_some_and(|entry| {
      entry.transit.is_some()
        && !entry.is_column
        && matches!(entry.role, Role::Moving { .. })
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
  let is_moving = matches!(role, Role::Moving { .. });

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

  // A moving column's real window may still show until the peek takes
  // over from it as it is cloaked; one at rest is already cloaked.
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
      (*id, frame, entry.clip.clone(), is_moving, is_live, anchor)
    })
    .collect::<Vec<(Uuid, Option<Rect>, Rect, bool, bool, HWND)>>();

  for (id, frame, clip, is_moving, is_live, anchor) in tracked {
    // Blended across a focus fade, as steady state draws it.
    let set = state.effect_set(id, focused_id == Some(id));
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
  let Some(window) = state.window_by_id(id) else {
    return false;
  };

  // Shown where it was, or cloaked where its peek is headed.
  let native = window.native();
  let (Ok(bounds), Ok(frame)) =
    (native.frame_with_shadows(), native.frame())
  else {
    return false;
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
  let ring_frame = crate::tab_bars::window_with_tab_bar(&window, &frame);
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
  let Some(window) = state.window_by_id(id).filter(|_| carries) else {
    return false;
  };

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
  // during one.
  let native = window.native();
  let Ok(frame) = native.frame() else {
    return false;
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
  use std::time::{Duration, Instant};

  use wm_common::{
    EasingFunction, WindowTransitionParams, WindowTransitionStyle,
  };
  use wm_platform::Rect;

  use super::{has_landed, is_departed, is_stray, retargeted_motion};
  use crate::animation::{
    state::WindowAnimationState,
    transit::{Direction, Transit},
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

  fn transit(duration_ms: u32) -> Transit {
    let params = WindowTransitionParams {
      duration_ms,
      easing: linear(),
      style: WindowTransitionStyle::Zoom,
      away_opacity: 0.0,
    };
    Transit::new(Direction::In, &params, 0.0)
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

  /// Only an entering window that is unmanaged or no longer shown is a
  /// stray; columns and entering windows still shown are left alone.
  #[test]
  fn strays_are_entering_windows_not_shown() {
    assert!(is_stray(false, None));
    assert!(is_stray(false, Some(false)));
    assert!(!is_stray(false, Some(true)));
    for shown in [None, Some(false), Some(true)] {
      assert!(!is_stray(true, shown), "{shown:?}");
    }
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
