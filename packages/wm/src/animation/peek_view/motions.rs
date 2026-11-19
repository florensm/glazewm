//! Putting windows on peeks as the layout changes: opening, moving,
//! fullscreen, grab resizes, restoring, closing and minimizing.

use std::{
  collections::{HashMap, HashSet},
  time::Instant,
};

use uuid::Uuid;
use wm_common::{
  ResizeStyle, WindowResizeConfig, WindowState, WindowTransitionParams,
};
use wm_platform::{perf, NativeWindow, Rect, SurrogateBatch};

use super::{
  move_clips,
  overlays::{existing_ring, track_overlays},
  ready::drop_stray_ready,
  stretch::is_same_size,
  surfaces::{
    crossing_host, has_held_neighbour, host_for, hosts_hold,
    hosts_transit, surface_monitor_clips, Host, Layer,
  },
  Leave, Left, PeekResize,
};
use crate::{
  animation::{
    manager::{movement_timing, relayout_timing, MakingWay, Motion},
    state::WindowAnimationState,
    transit::Transit,
    AnimationManager, AwayTransition,
  },
  commands::general::border_hidden_for_fullscreen,
  layout::is_hidden_in_layout,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// How long a window drawn moving or entering by its peek takes to follow
/// a new rect: `duration_ms`, unless it is floating and stays so, whose
/// moves are not animated (a command's move can't be told from an app's
/// or a drag's). That one jumps there; an entry it plays plays on.
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
/// peek as a tile does: between floating and
/// tiling, not while the OS still drags it.
pub fn changes_state_on_peek(window: &WindowContainer) -> bool {
  window.active_drag().is_none()
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
/// change of size: in `window_resize`'s style, as a tile's resize is.
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
/// more than `threshold_px`: below that, it isn't animated.
pub(in crate::animation) fn is_pure_move(
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
/// more than `threshold_px`: below that, it isn't animated.
pub(in crate::animation) fn is_peek_resize(
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
/// enabled.
pub fn resizes_on_peeks(config: &UserConfig) -> Option<PeekResize> {
  let resize = &config.value.animations.window_resize;
  resize.enabled.then(|| peek_resize(resize.style))
}

/// A motion holding a window at `rect`, for one leaving: only its transit
/// moves it.
pub(super) fn held_motion(
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
  drop_stray_ready(state);
  if !state.animation_manager.peeks.has_entering() {
    return;
  }

  let shown = |id: &Uuid| is_shown_tile(state, id);
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

/// Whether `id`'s window is on screen in its layout: `None` once it is
/// unmanaged, `Some(false)` while it is closed, moved to a hidden
/// workspace or switched away, minimized or a hidden tab.
pub(super) fn is_shown_tile(state: &WmState, id: &Uuid) -> Option<bool> {
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
}

/// Has the peeks standing by for a workspace switch that no switch owes a
/// hand-over any more (none played, or it ended another way) draw their
/// windows again, and drops the resting peeks kept for surrogates that
/// took over by now or never will (see
/// [`PeekView::keep_switching_out`](super::PeekView::keep_switching_out)).
///
/// Run once `redraw_containers` has started or skipped the switch, so no
/// peek stays faint, or kept, past the sync that decides it.
pub fn settle_switch_peeks(state: &mut WmState, config: &UserConfig) {
  let manager = &state.animation_manager;
  let owed = manager
    .peeks
    .standing_by
    .iter()
    .filter(|id| manager.awaits_peek_hand_back(id))
    .copied()
    .collect::<HashSet<_>>();

  let view = &mut state.animation_manager.peeks;
  view.switching_out.clear();
  let released = view
    .standing_by
    .iter()
    .filter(|id| !owed.contains(id))
    .copied()
    .collect::<Vec<_>>();
  for id in &released {
    view.end_stand_by(id);
  }

  if !released.is_empty() {
    track_overlays(state, config, SurrogateBatch::new());
  }
}

/// Has `window`, just opened at `target`, enter on a peek,
/// by `params` (see [`PeekView::enter`](super::PeekView::enter)), on ticks
/// paced by its monitor's vblanks as a scroll is. Returns whether it does.
///
/// Clipped to the whole monitor, not to its tile. Drawn on one of that
/// monitor's `PeekSurface`s if [`hosts_transit`] says so, given how its
/// pass `raise`s it in a band (see [`floating_peek_raise`]).
pub fn enter_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  target: &Rect,
  params: &WindowTransitionParams,
  config: &UserConfig,
  is_focused: bool,
  raise: Option<PeekRaise>,
) -> bool {
  let Some((clip, handle)) = window.monitor().and_then(|monitor| {
    let clip = monitor.to_rect().ok()?;
    Some((clip, monitor.native_properties().handle))
  }) else {
    return false;
  };

  let host =
    hosts_transit(window, true, raise, &state.animation_manager.peeks);
  let monitor_clips = surface_monitor_clips(state, host, &clip);
  let now = state.animation_manager.frame_time();
  let entered = state.animation_manager.peeks.enter(
    window,
    target,
    &clip,
    params,
    host,
    &monitor_clips,
    config,
    is_focused,
    now,
  );

  if entered {
    perf::note_animation_start(false);
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  entered
}

/// Has `window`, a tile shown at `from` and laid out at `target`, move
/// there on a peek if it only moves (see [`is_pure_move`]) or, given
/// `resizes` (see [`resizes_on_peeks`]), resizes too (see
/// [`is_peek_resize`]), on `window_move`'s timing or, when it resizes or
/// `cycle_has_resize`, `window_resize`'s (`movement_timing`). Returns
/// whether it does.
///
/// Its window is still shown: the redraw that freezes it queues the
/// take-over into the pass's shared flush (`commit_pending_cloaks`), with
/// [`cover_moved_overlays`](super::overlays::cover_moved_overlays) before
/// it and [`hand_over_moved_overlays`](super::overlays::hand_over_moved_overlays)
/// after. That flush also sends the window to its target, size and all,
/// behind its cloak.
///
/// Not a window another stand-in draws, which keeps it, nor one moving
/// onto another monitor unless [`tile_clips`] draws it on two peeks:
/// on each monitor's surface (see [`move_host`]). Each tile of a
/// relayout is taken or refused on its own; a refused one snaps.
///
/// One crossing between floating and tiling (see
/// [`changes_state_on_peek`]) is drawn on the surface of the band its pass
/// `raise`s it in, if any (see [`state_change_peek_raise`], [`host_for`]).
#[allow(clippy::too_many_arguments)]
pub fn move_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  from: &Rect,
  target: &Rect,
  cycle_has_resize: bool,
  making_way: Option<&MakingWay>,
  resizes: Option<PeekResize>,
  config: &UserConfig,
  is_focused: bool,
  raise: Option<PeekRaise>,
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
      && !is_resized_already(
        state.animation_manager.peeks.is_moving(&window.id()),
        || window.native().frame_with_shadows().ok(),
        target,
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
  let Some((clip, trail_clip)) = tile_clips(state, window, from) else {
    return false;
  };

  let motion = if is_resize {
    Motion::Resize
  } else {
    Motion::Move
  };
  let (duration_ms, easing) =
    relayout_timing(motion, cycle_has_resize, making_way, config);
  let host = move_host(state, window, &clip, trail_clip.as_ref(), raise);
  let mut monitor_clips = surface_monitor_clips(state, host, &clip);
  if let Some(trail_clip) =
    trail_clip.as_ref().filter(|_| monitor_clips.is_empty())
  {
    monitor_clips = surface_monitor_clips(state, host, trail_clip);
  }
  let now = state.animation_manager.frame_time();
  let ring = existing_ring(state, window);
  let peeks = &mut state.animation_manager.peeks;
  // Taken before `move_tile` drops it: a tile moving back onto the monitor
  // it just landed on finds its ready peek there (see `Ready`).
  let trail_ready = trail_clip
    .as_ref()
    .filter(|_| host.is_some())
    .and_then(|trail_clip| peeks.take_ready_at(window, trail_clip));
  let moved = peeks.move_tile(
    window,
    from,
    target.clone(),
    &clip,
    duration_ms,
    easing,
    resizes,
    host,
    ring,
    &monitor_clips,
    config,
    is_focused,
    now,
  ) && trail_clip.as_ref().is_none_or(|trail_clip| {
    peeks.add_trail(
      window,
      trail_clip,
      host,
      trail_ready,
      &monitor_clips,
      config,
      is_focused,
      now,
    )
  });

  if moved {
    perf::note_animation_start(is_resize);
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  moved
}

/// Where [`move_on_peek`] draws `window`, cut at `clip` and, crossing
/// monitors, at `trail_clip` too (see [`crossing_host`]), its pass
/// raising it in a band as `raise` says.
fn move_host(
  state: &WmState,
  window: &WindowContainer,
  clip: &Rect,
  trail_clip: Option<&Rect>,
  raise: Option<PeekRaise>,
) -> Option<Host> {
  let peeks = &state.animation_manager.peeks;
  let host = host_for(
    !state.pending_sync.is_window_state_change(&window.id()),
    raise,
  )
  // Its band raise would top the tiles' surface over held tiles, as an
  // entering one's would (see `hosts_transit`).
  .filter(|host| {
    *host != Host::InBand(Layer::Tiles)
      || !has_held_neighbour(window, peeks)
  });

  match trail_clip {
    Some(trail_clip) => crossing_host(
      host,
      peeks.holds_on_surface(clip) || peeks.holds_on_surface(trail_clip),
    ),
    None => host,
  }
}

/// How a WM fullscreen toggle is drawn on peeks, if it is: enabled, in
/// `window_fullscreen`'s own style.
fn fullscreen_on_peeks(config: &UserConfig) -> Option<PeekResize> {
  let fullscreen = &config.value.animations.window_fullscreen;
  fullscreen.enabled.then_some(peek_resize(fullscreen.style))
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
///
/// Drawn among the tiles as a resize is, or on the
/// always-on-top band's surface if its pass `raise`s it there (see
/// [`fullscreen_peek_raise`], [`host_for`]).
pub fn fullscreen_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  from: &Rect,
  target: &Rect,
  config: &UserConfig,
  is_focused: bool,
  raise: Option<PeekRaise>,
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
  let host = host_for(true, raise);
  let monitor_clips = surface_monitor_clips(state, host, &clip);
  let now = state.animation_manager.frame_time();
  let ring = existing_ring(state, window);
  let moved = state.animation_manager.peeks.move_tile(
    window,
    from,
    target.clone(),
    &clip,
    duration_ms,
    easing,
    Some(resize),
    host,
    ring,
    &monitor_clips,
    config,
    is_focused,
    now,
  );

  if moved {
    perf::note_animation_start(!is_same_size(from, target));
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  moved
}

/// Whether `id`'s ring is left off its peek as its window is fullscreen,
/// as `border_hidden_for_fullscreen` leaves it off the window (see
/// [`fullscreen_on_peek`]).
pub(super) fn hides_ring_for_fullscreen(
  state: &WmState,
  id: Uuid,
  config: &UserConfig,
) -> bool {
  state
    .window_by_id(id)
    .is_some_and(|window| border_hidden_for_fullscreen(&window, config))
}

/// Whether a grab resize can draw `window` on a peek it holds (see
/// [`hold_on_peek`], and `sync_scroll_view` for a column): a tile shown
/// in its layout, its real window on its monitor unless it is a column,
/// on a peek that never moves, and with no other stand-in. A stack's
/// window is left out, its tab bar being a window of its own.
pub fn can_hold(state: &WmState, window: &WindowContainer) -> bool {
  hold_refusal(state, window).is_none()
}

/// Why a grab resize can't draw `window` on a peek it holds (see
/// [`can_hold`]), or `None` if it can.
pub fn hold_refusal(
  state: &WmState,
  window: &WindowContainer,
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
  } else {
    None
  }
}

/// Puts `window` on a peek a grab resize holds where it puts it, from
/// `from` at `target`: drawn filled, its ring in cells, never landing
/// until the grab lets go of it (see [`Hold`](super::Hold)). Its real
/// window is taken over from and cloaked by the redraw that called this,
/// as for a move on a peek, and then resized behind it. Returns whether it
/// is.
///
/// Drawn on its monitor's `PeekSurface` as [`hosts_hold`] says, which is
/// then never raised for it: it stays where the tile's admission put it.
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
  let host =
    hosts_hold(state.animation_manager.peeks.is_surface_topped(&clip))
      .then_some(Host::Tiles);
  let monitor_clips = surface_monitor_clips(state, host, &clip);
  let now = state.animation_manager.frame_time();
  let ring = existing_ring(state, window);
  let peeks = &mut state.animation_manager.peeks;
  let moved = peeks.move_tile(
    window,
    from,
    target.clone(),
    &clip,
    0,
    easing,
    Some(PeekResize::Fill),
    host,
    ring,
    &monitor_clips,
    config,
    is_focused,
    now,
  );
  if moved {
    peeks.hold(window.id(), true);
  } else {
    tracing::debug!("{window}: live, its peek couldn't be put up.");
  }

  if moved {
    state.animation_manager.ensure_waiter_for(handle);
    state.animation_manager.ensure_timer_running();
  }

  moved
}

/// Whether a window laid out at `target`, its frame read by `actual`, is
/// at that size already, as an interactive resize leaves it as its drag
/// ends: drawn resizing on a peek, it would shrink back to its old size
/// first, so it snaps. Not one `is_moving` on a peek, whose filled or
/// stretched peek may have sent its window that size already: retargeted,
/// it stays on its peek.
fn is_resized_already(
  is_moving: bool,
  actual: impl FnOnce() -> Option<Rect>,
  target: &Rect,
) -> bool {
  !is_moving
    && actual().is_some_and(|actual| is_same_size(&actual, target))
}

/// The clips of the peeks [`move_on_peek`] draws `window`, a tile moving
/// from `from`, on (see [`move_clips`]). One moving onto another monitor
/// is drawn on two, unless a peek draws it already (one handing it back is
/// on its own monitor), it is in a stack, whose tab bar's window is cut at
/// one peek's monitor (a hosted copy of it draws only what that window
/// holds).
fn tile_clips(
  state: &WmState,
  window: &WindowContainer,
  from: &Rect,
) -> Option<(Rect, Option<Rect>)> {
  let clip = window.monitor()?.to_rect().ok()?;
  let crosses = !state.animation_manager.peeks.draws_window(&window.id())
    && window
      .parent()
      .is_none_or(|parent| parent.as_stack().is_none());
  let source = crosses
    .then(|| state.monitor_at_point(&from.center_point()))
    .flatten()
    .and_then(|monitor| monitor.to_rect().ok());

  move_clips(from, clip, source, crosses)
}

/// Whether a stand-in other than a peek draws `window_id` or hands it
/// back: a workspace switch's. A move then stays with it.
pub(in crate::animation) fn has_other_stand_in(
  manager: &AnimationManager,
  window_id: &Uuid,
) -> bool {
  manager.has_active_surrogate(window_id)
    && !manager.peeks.draws_window(window_id)
}

/// Has `window`, coming back from minimized to `target`, enter on a peek
/// by `params`, as [`enter_on_peek`] has a new one. A minimize
/// still playing on its peek turns round where it is instead, on its
/// surface or on its own window as it was drawn. Returns whether it does
/// either.
///
/// A window still natively minimized (`toggle-minimized`) is restored at
/// `target` behind a cloak first (see [`restore_cloaked`]). A tile is
/// raised as it comes back; a floating one enters as one opening does
/// (see [`floating_peek_raise`]): raised in its band by its pass's
/// `raise`, if any, else left just below its window.
pub fn restore_on_peek(
  state: &mut WmState,
  window: &WindowContainer,
  target: &Rect,
  params: &WindowTransitionParams,
  config: &UserConfig,
  is_focused: bool,
  raise: Option<PeekRaise>,
) -> bool {
  let id = window.id();
  let now = state.animation_manager.frame_time();
  let raises_as_tile = !matches!(window.state(), WindowState::Floating(_));

  if state
    .animation_manager
    .peeks
    .turn_round(&id, params, false, now)
  {
    restore_cloaked(&window.native(), target);
    if raises_as_tile {
      state.animation_manager.peeks.raise_moving(&id);
    }
    perf::note_animation_start(false);
    state.animation_manager.ensure_timer_running();
    return true;
  }

  let enters = Transit::opening(params).is_some()
    && state.animation_manager.peeks.can_enter(window);
  if !enters {
    if state.animation_manager.peeks.draws_window(&id) {
      tracing::debug!(
        "Restore of {window} not started: its peek draws it, which carries \
         on."
      );
    }
    return false;
  }

  restore_cloaked(&window.native(), target);
  // A tile is raised as one, in no band of its own.
  let raise = raise.filter(|_| !raises_as_tile);
  let entered = enter_on_peek(
    state, window, target, params, config, is_focused, raise,
  );
  if raises_as_tile {
    state.animation_manager.peeks.raise_moving(&id);
  }
  entered
}

/// Restores `window`, if it is still natively minimized, at `rect` behind
/// a cloak, so that no frame shows it before its peek does. One that can't
/// be cloaked is left minimized, to be restored as it is handed back.
pub(in crate::animation) fn restore_cloaked(
  window: &NativeWindow,
  rect: &Rect,
) {
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

/// Takes `window`, at `rect`, off the screen on a peek, by
/// `params` (see [`PeekView::leave`](super::PeekView::leave)), on ticks
/// paced by its monitor's vblanks. Clipped to the whole monitor, as an
/// open on a peek is, and drawn on its `PeekSurface` if [`hosts_transit`]
/// says so.
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

  // Never raised: only a tile goes on a surface.
  let host =
    hosts_transit(window, false, None, &state.animation_manager.peeks);
  let monitor_clips = surface_monitor_clips(state, host, &clip);
  let now = state.animation_manager.frame_time();
  let leave = state.animation_manager.peeks.leave(
    window,
    rect,
    &clip,
    transition,
    params,
    host,
    &monitor_clips,
    config,
    is_focused,
    now,
  );
  if leave != Leave::Leaving {
    return leave;
  }

  let id = window.id();
  state.animation_manager.drop_companion_cover(&id);
  track_overlays(state, config, SurrogateBatch::new());

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
        tracing::debug!(
          "Close on its peek complete for {id}; sending WM_CLOSE (cloaked: \
           {:?}).",
          window.is_cloaked().ok()
        );
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

#[cfg(test)]
mod tests {

  use wm_common::{
    FloatingStateConfig, FullscreenStateConfig, ResizeStyle,
    WindowResizeConfig, WindowState,
  };
  use wm_platform::Rect;

  use super::{
    crosses_floating, floating_peek_raise, fullscreen_peek_raise,
    is_peek_resize, is_pure_move, is_resized_already, peek_resize,
    state_change_peek_raise, state_change_resize, toggles_tile_fullscreen,
    PeekRaise,
  };
  use crate::animation::peek_view::{tests::tile, PeekResize};

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

  #[test]
  fn only_a_tile_not_moving_is_resized_already() {
    let target = Rect::from_xy(0, 0, 800, 600);
    let at_size = Rect::from_xy(40, 40, 800, 600);
    let other_size = Rect::from_xy(0, 0, 640, 600);

    assert!(is_resized_already(false, || Some(at_size.clone()), &target));
    assert!(!is_resized_already(true, || Some(at_size.clone()), &target));
    assert!(!is_resized_already(false, || Some(other_size), &target));
    assert!(!is_resized_already(false, || None, &target));
  }
}
