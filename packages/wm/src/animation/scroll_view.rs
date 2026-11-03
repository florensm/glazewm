//! What a scrolling layout shows of its columns besides the real windows.
//!
//! At rest, columns show their real windows, also where they run off a
//! monitor edge with no other monitor behind it. Only a column that would
//! reach onto another monitor is cloaked, and its visible part is shown
//! by a `WindowPeek`: a click-through thumbnail clipped to the monitor.
//!
//! While the strip moves (a scroll, or columns opening, closing or
//! resizing across the monitor edge), every moving column is drawn by a
//! peek and its real window is cloaked, like the workspace-switch slide:
//! the peeks go up first, too faint to see, one composited frame later
//! they take over from the real windows as those are cloaked, and all of
//! them move in lock-step, their borders, backdrops and tab bars with
//! them. At the end, columns shown at rest get their real window back,
//! ones reaching onto another monitor keep their peek, and the rest are
//! dropped. Both swaps hide a semi-transparent real window by its alpha
//! first (see `WindowPeek::take_over`), so no frame shows a window twice
//! or not at all.
//!
//! As for a workspace switch, a peek spans its monitor and never moves:
//! only its thumbnail does, along with copies of the window's border ring,
//! shadow and backdrop, in the same DWM updates. A window whose backdrop
//! can't be copied (rounded, or drifting with parallax) has its peek move
//! with it instead (see `WindowPeek`).
//!
//! A peek's border, backdrop and tab bar are cut at the monitor edge like
//! the peek itself, so nothing of a column shows on the monitor beside it.

use std::{
  collections::{HashMap, HashSet},
  time::Instant,
};

use uuid::Uuid;
use wm_common::{DisplayState, Layout, WindowState};
use wm_platform::{
  BackdropOverlayParams, BorderOverlayParams, NativeBackdropOverlay,
  NativeBorderOverlay, NativeWindow, Overlay, PeekStyle, Point, Rect,
  StyleBlend, SurrogateBatch, WindowPeek, HWND,
};

use super::{
  stand_in::{moved_along, StandIn},
  state::WindowAnimationState,
};
use crate::{
  commands::general::{
    overlay_entry, surrogate_effects_for, upsert_clipped_overlay,
    upsert_pinned_border_overlay,
  },
  layout::is_hidden_in_layout,
  models::{is_inactive_stack_child, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// What a peek is currently doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
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

struct Entry {
  peek: WindowPeek,
  motion: WindowAnimationState,
  /// Where the peek was last placed, before clipping.
  rect: Rect,
  /// Monitor area the peek is clipped to.
  clip: Rect,
  opacity: u8,
  role: Role,
  ring: Handover,
  backdrop: Handover,
}

impl Entry {
  fn place(&mut self, rect: &Rect, batch: &mut SurrogateBatch) {
    self.peek.place(rect, &self.clip, self.opacity, batch);
    self.rect = rect.clone();
  }
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
pub struct ScrollView {
  entries: HashMap<Uuid, Entry>,
  /// Last target rect of every window in a displayed scrolling layout,
  /// i.e. where it is shown now and moves from on the next relayout.
  last_rects: HashMap<Uuid, Rect>,
  /// Windows already reported by `report_stuck_windows`.
  reported_stuck: HashSet<Uuid>,
}

impl ScrollView {
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
  fn owned_target(&self, window_id: &Uuid) -> Option<&Rect> {
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
  fn is_releasing(&self, window_id: &Uuid) -> bool {
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

  /// Whether `window_id` is moving, i.e. its real window stays frozen.
  pub fn is_moving(&self, window_id: &Uuid) -> bool {
    self
      .entries
      .get(window_id)
      .is_some_and(|entry| matches!(entry.role, Role::Moving { .. }))
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
          // Its real window was cloaked a tick ago at least, taking its
          // own shadow along, so needs its alpha no more. One not taken
          // over from, e.g. as it wasn't redrawn, is drawn all the same.
          entry.peek.restore_alpha();
          entry.peek.show_window();
          entry.peek.cast_shadow(true);
          entry.motion.start_clock(clock_origin);
          let (rect, _) = entry.motion.current_state_at(now);
          entry.place(&rect, batch);

          if entry.motion.is_complete() {
            entry.role = if ends_shown {
              // Raised while its real window is still cloaked, so the
              // uncloak never shows the window over the peek.
              entry.peek.cover_window();
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
  fn note_composited(&mut self) {
    for entry in self.entries.values_mut() {
      for handover in [&mut entry.ring, &mut entry.backdrop] {
        if *handover == Handover::Covering {
          *handover = Handover::Covered;
        }
      }
    }
  }

  /// Takes out the peeks of windows that `is_known` no longer knows, i.e.
  /// that left the displayed scrolling layouts, and forgets their rects.
  fn take_departed(
    &mut self,
    is_known: impl Fn(&Uuid) -> bool,
  ) -> Vec<(Uuid, Entry)> {
    self.last_rects.retain(|id, _| is_known(id));
    self.entries.extract_if(|id, _| !is_known(id)).collect()
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
  /// A move of a peek already shown is queued into `batch`.
  ///
  /// Returns whether a new peek now covers a still-visible real window,
  /// so it has to be composited before that window is cloaked.
  #[allow(clippy::too_many_arguments)]
  fn upsert(
    &mut self,
    window: &WindowContainer,
    from: &Rect,
    target: Rect,
    clip: &Rect,
    role: Role,
    config: &UserConfig,
    is_focused: bool,
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
      Some(entry) if is_moving => entry.motion.retargeted(
        target,
        scroll_config.duration_ms,
        scroll_config.easing.clone(),
        Instant::now(),
      ),
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
    let [focused_style, unfocused_style] = styles;
    let style = if is_focused {
      focused_style
    } else {
      unfocused_style
    };
    if !is_moving {
      motion.start_clock(Instant::now());
    }

    if let Some(entry) = self.entries.get_mut(&window.id()) {
      // Whether its real window is cloaked, so shows no shadow of its own:
      // all but one being released, which a new move cloaks again first.
      let is_cloaked = !matches!(entry.role, Role::Releasing { .. });

      entry.motion = motion;
      entry.clip = clip.clone();
      entry.opacity = opacity;
      entry.role = role;
      // Copies of both styles are drawn up front, so a focus change only
      // swaps which show: one registered mid-move left frames without it.
      entry.peek.carry_styles(&window.native(), &styles);
      entry.peek.show_style(&style);
      entry.peek.cast_shadow(is_cloaked);
      let rect = entry.motion.current_rect();
      entry.place(&rect, batch);
      return covers_shown_window;
    }

    // A window with a backdrop that can't be copied along in either focus
    // state has a peek that moves with it.
    let is_live = styles.iter().any(|style| {
      style.backdrop.is_some_and(|params| !params.is_copyable())
    });

    // Sized like the window itself, which a collapsed `from` (an opening
    // column) isn't.
    let Ok(mut peek) = WindowPeek::new(
      &window.native(),
      &motion.target_rect,
      clip,
      &corner_style,
      is_live,
    )
    .inspect_err(|err| {
      tracing::warn!("Failed to create a peek of {window}: {err}");
    }) else {
      return false;
    };

    // Before the ring, which goes on top.
    peek.carry_styles(&window.native(), &styles);
    peek.show_style(&style);

    // A moving column's real window may still show, with its own shadow,
    // until the peek takes over from it as it is cloaked; one at rest is
    // already cloaked.
    let covers_shown_window =
      is_moving && !window.native().is_cloaked().unwrap_or(true);
    if covers_shown_window {
      peek.warm_up();
    }
    peek.cast_shadow(!is_moving);

    let rect = motion.current_rect();
    let mut entry = Entry {
      peek,
      motion,
      rect: rect.clone(),
      clip: clip.clone(),
      opacity,
      role,
      ring: Handover::None,
      backdrop: Handover::None,
    };
    entry.place(&rect, batch);
    self.entries.insert(window.id(), entry);

    covers_shown_window
  }
}

/// Starts moving the columns of displayed scrolling layouts whose rects
/// changed, and updates the peeks of the ones at rest. Columns land at
/// once unless the sync `animates`.
///
/// Runs before `platform_sync` repositions anything: moving windows are
/// then frozen (cloaked and pre-positioned at their target) behind peeks
/// that are already on screen.
pub fn sync_scroll_view(
  state: &mut WmState,
  config: &UserConfig,
  animates: bool,
) {
  let workspaces = displayed_scrolling_workspaces(state);
  let windows = workspaces
    .iter()
    .flat_map(|(workspace, clip)| {
      tiling_windows(workspace)
        .into_iter()
        .map(move |window| (window, clip.clone()))
    })
    .collect::<Vec<_>>();
  let focused_id = state.focused_container().map(|focused| focused.id());

  // Windows the WM hasn't placed yet, i.e. just opened.
  let new_ids = windows
    .iter()
    .map(|(window, _)| window.id())
    .filter(|id| !state.window_target_positions.contains_key(id))
    .collect::<HashSet<_>>();

  let view = &mut state.animation_manager.scroll_view;

  // Windows that left the displayed scrolling layouts lose their peeks;
  // their real windows are handled like any other window's. One the
  // scroll view held cloaked is redrawn, which shows or hides it for good.
  //
  // Their peeks stay up until any that take over are on screen, e.g. that
  // of a stack's newly active tab, so the column never goes blank.
  let gone =
    view.take_departed(|id| windows.iter().any(|(w, _)| w.id() == *id));

  let scroll_config = &config.value.animations.scroll;
  let mut needs_flush = false;
  let mut adds_peek = false;
  let mut started_moving = Vec::new();
  let mut batch = SurrogateBatch::new();

  for (workspace, clip) in &workspaces {
    let moves = column_moves(view, workspace, &new_ids);

    // The strip moves as one when any column crosses the monitor edge,
    // is already drawn by a peek, or opens; moves that stay on screen are
    // left to the regular per-window animations.
    let crosses_edge = |rect: &Rect| !clip.contains_rect(rect);
    let strip_moves = scroll_config.enabled
      && animates
      && moves.iter().any(|(window, from, target)| {
        from != target
          && (crosses_edge(from)
            || crosses_edge(target)
            || view.entries.contains_key(&window.id())
            || new_ids.contains(&window.id()))
      });

    for (window, from, target) in moves {
      view.last_rects.insert(window.id(), target.clone());

      let ends_shown = !is_hidden_in_layout(&window);
      let is_on_screen = |rect: &Rect| rect.intersection_area(clip) > 0;

      // Mid-move or mid-hand-back with nowhere new to go: leave it be.
      // Retargeting it anyway would restart its move every sync.
      if view.owned_target(&window.id()) == Some(&target) {
        continue;
      }

      let role = if strip_moves && from != target {
        Role::Moving { ends_shown }
      } else if ends_shown {
        // At rest on screen: the real window shows it, once a peek still
        // covering it after a move lets go.
        if !view.is_releasing(&window.id()) {
          view.entries.remove(&window.id());
        }
        continue;
      } else if is_on_screen(&target) {
        Role::Peek
      } else {
        view.entries.remove(&window.id());
        continue;
      };

      // Nothing to draw. A window held cloaked keeps its peek though, to
      // be handed back at the end of its move: dropping it would leave
      // the real window cloaked, and its cloak event would then look
      // like the app hiding it.
      if !is_on_screen(&from)
        && !is_on_screen(&target)
        && !view.owns_window(&window.id())
      {
        view.entries.remove(&window.id());
        continue;
      }

      let is_moving = matches!(role, Role::Moving { .. });
      adds_peek |= !view.entries.contains_key(&window.id());
      needs_flush |= view.upsert(
        &window,
        &from,
        target,
        clip,
        role,
        config,
        focused_id == Some(window.id()),
        &mut batch,
      );

      if is_moving {
        started_moving.push(window.id());
      }
    }
  }

  let replaces_peek = adds_peek
    && gone
      .iter()
      .any(|(_, entry)| entry.peek.visible_rect().is_some());

  flush_overlays(state, config, batch, needs_flush || replaces_peek);

  for (id, entry) in gone {
    if let Some(window) =
      state.window_by_id(id).filter(|_| entry.role != Role::Peek)
    {
      state.pending_sync.queue_container_to_redraw(window);
    }
  }

  // The scroll view drives these windows now, not their own animations.
  for id in &started_moving {
    state.animation_manager.remove_animation(id);
  }

  if state.animation_manager.scroll_view.is_active() {
    state.animation_manager.ensure_timer_running();
  }
}

/// Tracks the overlays of the windows the scroll view draws (see
/// [`track_overlays`]) and, if `flushes`, has DWM composite them along
/// with the peeks.
///
/// Tracked before the flush, so it composites the copies drawn over the
/// rings and backdrops handed over too. Those then go at once, before
/// anything moves: hidden on the first tick of the move instead, a frame
/// could show them still at rest beside their moved copies.
fn flush_overlays(
  state: &mut WmState,
  config: &UserConfig,
  batch: SurrogateBatch,
  flushes: bool,
) {
  track_overlays(state, config, batch);

  if flushes {
    wm_platform::dwm_flush();
    state.animation_manager.scroll_view.note_composited();
    track_overlays(state, config, SurrogateBatch::new());
  }
}

/// Logs windows of displayed scrolling layouts that should be shown but
/// whose real window is still cloaked, once each, for diagnosing windows
/// that disappear. Only meaningful while nothing animates.
pub fn report_stuck_windows(state: &mut WmState) {
  let stuck = displayed_scrolling_workspaces(state)
    .iter()
    .flat_map(|(workspace, _)| tiling_windows(workspace))
    .filter(|window| {
      window.display_state() == DisplayState::Shown
        && !is_hidden_in_layout(window)
        && window.native().is_cloaked().unwrap_or(false)
    })
    .collect::<Vec<_>>();

  let view = &mut state.animation_manager.scroll_view;
  let stuck_ids =
    stuck.iter().map(CommonGetters::id).collect::<HashSet<_>>();
  view.reported_stuck.retain(|id| stuck_ids.contains(id));

  for window in stuck {
    if !view.reported_stuck.insert(window.id()) {
      continue;
    }

    let role = view.entries.get(&window.id()).map(|entry| entry.role);
    tracing::warn!(
      "Scrolling layout window stuck cloaked: {window}, rect {:?}, last \
       rect {:?}, scroll view role {role:?}.",
      target_rect(&window).ok(),
      view.last_rects.get(&window.id()),
    );
  }
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
    .scroll_view
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
    let (_, _, overlays) = surrogate_effects_for(
      state.effect_set(id, focused_id == Some(id)),
      config,
    );

    let carries_backdrop =
      !is_live && carry_backdrop(state, id, overlays.backdrop);

    match (overlays.backdrop, &frame) {
      _ if carries_backdrop => {}
      (Some(params), Some(frame)) => {
        upsert_clipped_overlay::<NativeBackdropOverlay>(
          &mut state.window_overlays,
          id,
          params,
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

    if !is_live && carry_ring(state, id, overlays.border) {
      continue;
    }

    // Framed with its stack's tab bar, like the real window.
    let border_frame = frame
      .map(|frame| crate::tab_bars::with_tab_bar_of(state, id, &frame));

    match (overlays.border, border_frame) {
      (Some(params), Some(frame)) if is_moving => {
        upsert_pinned_border_overlay(
          &mut state.window_overlays,
          id,
          StyleBlend::settled(params),
          &frame,
          &clip,
          &clip,
          anchor,
        );
      }
      (Some(params), Some(frame)) => {
        upsert_clipped_overlay::<NativeBorderOverlay>(
          &mut state.window_overlays,
          id,
          params,
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
  params: Option<BorderOverlayParams>,
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

  if let Some(entry) =
    state.animation_manager.scroll_view.entries.get_mut(&id)
  {
    entry.peek.follow_window(&bounds, &frame);
  }

  let Some(params) = params else {
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
    params,
    &ring_frame,
    anchor,
  ) else {
    return false;
  };
  overlay.apply_blend(StyleBlend::settled(params));

  let Some(entry) =
    state.animation_manager.scroll_view.entries.get_mut(&id)
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
  params: Option<BackdropOverlayParams>,
) -> bool {
  let carries = state
    .animation_manager
    .scroll_view
    .entries
    .get(&id)
    .is_some_and(|entry| entry.peek.carries_backdrop());
  let Some(window) = state.window_by_id(id).filter(|_| carries) else {
    return false;
  };

  let Some(params) = params else {
    if let Some(overlay) = state
      .window_overlays
      .get_mut(&id)
      .and_then(|overlays| overlays.backdrop.as_mut())
    {
      overlay.hide();
    }
    if let Some(entry) =
      state.animation_manager.scroll_view.entries.get_mut(&id)
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

  let Some(entry) =
    state.animation_manager.scroll_view.entries.get_mut(&id)
  else {
    return false;
  };
  let is_landing =
    matches!(entry.role, Role::HandingBack | Role::Releasing { .. });
  entry.peek.show_backdrop();

  let handover = entry.backdrop;
  entry.backdrop = if is_landing
    && matches!(
      handover,
      Handover::Covering | Handover::Covered | Handover::Parked
    ) {
    let Some(overlay) = overlay_entry::<NativeBackdropOverlay>(
      &mut state.window_overlays,
      id,
      params,
      &frame,
      anchor,
    ) else {
      return false;
    };
    overlay.apply_blend(StyleBlend::settled(params));
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
      overlay.apply_blend(StyleBlend::settled(params));
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

/// How `window` is drawn focused and not, which its peek carries copies
/// for.
fn peek_styles(
  window: &WindowContainer,
  config: &UserConfig,
) -> [PeekStyle; 2] {
  [true, false].map(|is_focused| {
    let (_, corner_style, overlays) =
      surrogate_effects_for(window.effect_set(is_focused), config);

    PeekStyle {
      is_active: is_focused,
      corner_style,
      backdrop: overlays.backdrop,
    }
  })
}

/// Where each tiling window of `workspace` moves from and to: from where
/// it is shown now, or for a new column from its opening rect.
fn column_moves(
  view: &ScrollView,
  workspace: &Workspace,
  new_ids: &HashSet<Uuid>,
) -> Vec<(WindowContainer, Rect, Rect)> {
  let moves = tiling_windows(workspace)
    .into_iter()
    .filter_map(|window| {
      let target = target_rect(&window).ok()?;
      let from = view
        .entries
        .get(&window.id())
        .map(|entry| entry.motion.current_rect())
        .or_else(|| view.last_rects.get(&window.id()).cloned())
        .unwrap_or_else(|| target.clone());

      Some((window, from, target))
    })
    .collect::<Vec<_>>();

  // A new column opens in place, widening as its neighbours make room,
  // rather than sliding in over them.
  moves
    .iter()
    .map(|(window, from, target)| {
      let from = if new_ids.contains(&window.id()) {
        opening_rect(target, &moves, new_ids)
      } else {
        from.clone()
      };
      (window.clone(), from, target.clone())
    })
    .collect()
}

/// Where a new column opens from: collapsed to no width at its place in
/// the strip as it was before, so it widens as its neighbours make room.
fn opening_rect(
  target: &Rect,
  moves: &[(WindowContainer, Rect, Rect)],
  new_ids: &HashSet<Uuid>,
) -> Rect {
  let existing = moves
    .iter()
    .filter(|(window, _, _)| !new_ids.contains(&window.id()));

  // How far the strip scrolls: what a column left of the new one moves
  // by, or one right of it minus the room it makes.
  let scroll = existing
    .clone()
    .find(|(_, _, to)| to.x() < target.x())
    .map(|(_, from, to)| from.x() - to.x())
    .or_else(|| {
      existing
        .clone()
        .find(|(_, _, to)| to.x() > target.x())
        .map(|(_, from, to)| from.x() - to.x() + target.width())
    })
    .unwrap_or(0);

  Rect::from_xy(target.x() + scroll, target.y(), 0, target.height())
}

/// Displayed scrolling workspaces, with the monitor area their peeks are
/// clipped to.
fn displayed_scrolling_workspaces(
  state: &WmState,
) -> Vec<(Workspace, Rect)> {
  state
    .workspaces()
    .into_iter()
    .filter(|workspace| {
      workspace.layout() == Layout::Scrolling && workspace.is_displayed()
    })
    .filter_map(|workspace| {
      let clip = workspace.monitor()?.native_properties().working_area;
      Some((workspace, clip))
    })
    .collect()
}

/// The workspace's tiling windows that are shown when on screen.
fn tiling_windows(workspace: &Workspace) -> Vec<WindowContainer> {
  workspace
    .descendants()
    .filter_map(|container| container.as_window_container().ok())
    .filter(|window| {
      window.state() == WindowState::Tiling
        && !is_inactive_stack_child(window)
    })
    .collect()
}

/// Where `platform_sync` puts the real window.
fn target_rect(window: &WindowContainer) -> anyhow::Result<Rect> {
  Ok(
    window
      .to_rect()?
      .apply_delta(&window.total_border_delta()?, None),
  )
}
