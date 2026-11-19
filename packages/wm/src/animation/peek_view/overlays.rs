//! What a peek carries along with its window: copies of its ring,
//! backdrop and stack tab bar, handed over to and back from the real ones.

use uuid::Uuid;
use wm_common::EffectSet;
use wm_platform::{
  perf::{self, Stage},
  BackdropOverlayParams, BorderOverlayParams, NativeBackdropOverlay,
  NativeBorderOverlay, NativeWindow, Overlay, Rect, RectDelta, StyleBlend,
  SurrogateBatch, HWND,
};

use super::{motions::hides_ring_for_fullscreen, Entry, Role};
use crate::{
  commands::general::{
    border_hidden_for_fullscreen, focus_blend, overlay_entry,
    surrogate_effects_for, upsert_clipped_overlay,
    upsert_pinned_border_overlay,
  },
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// How far a pinned peek has taken over its window's border ring or
/// backdrop, which it hands over as a workspace switch does: covered
/// where it is by the copy, hidden a composited frame later (the ring
/// parked where its window is headed), then shown again where the window
/// lands, under the copy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Handover {
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

/// How far `window`'s ring window reaches past its visible frame, focused
/// or not (see `BorderOverlayParams::extent`).
pub(super) fn ring_reach(
  window: &WindowContainer,
  config: &UserConfig,
) -> i32 {
  #[allow(clippy::cast_possible_truncation)]
  [true, false]
    .into_iter()
    .filter_map(|is_focused| {
      surrogate_effects_for(window.effect_set(is_focused), config)
        .2
        .border
    })
    .map(|border| border.extent().ceil() as i32)
    .max()
    .unwrap_or(0)
}

/// `window`'s ring overlay and its corner reach (see
/// `NativeBorderOverlay::corner_reach`), if it has one already.
pub(super) fn existing_ring(
  state: &WmState,
  window: &WindowContainer,
) -> Option<(HWND, i32)> {
  state
    .window_overlays
    .get(&window.id())
    .and_then(|overlays| overlays.border.as_ref())
    .map(|overlay| (overlay.hwnd(), overlay.corner_reach()))
}

/// Covers the rings and backdrops of windows whose peeks
/// [`move_on_peek`](super::motions::move_on_peek) put up this pass with
/// their copies, for the pass's shared flush to composite, as
/// `sync_scroll_view` does before its own flush.
pub fn cover_moved_overlays(state: &mut WmState, config: &UserConfig) {
  track_overlays(state, config, SurrogateBatch::new());
  restack_joining_overlays(state);
}

/// Puts the shown overlays of the tiles that joined a `PeekSurface` this
/// pass, with their windows above it, directly behind it (see
/// `PeekView::joining_tenants`). It went below the lowest of their windows
/// (see `WindowPeek::hosted`), maybe under the overlays of higher ones,
/// which would cover their copies there once their windows are cloaked,
/// until it is raised.
pub(in crate::animation) fn restack_joining_overlays(state: &mut WmState) {
  let _scope = perf::scope(Stage::OverlayRestack);
  for (window_id, anchor) in
    state.animation_manager.peeks.joining_tenants()
  {
    let Some(overlays) = state.window_overlays.get_mut(&window_id) else {
      continue;
    };

    // Backdrop first, so the border ends up between it and the surface.
    let backdrop = overlays
      .backdrop
      .as_mut()
      .filter(|overlay| overlay.is_visible())
      .map(|overlay| overlay.sync_z_order(anchor, false));
    let border = overlays
      .border
      .as_mut()
      .filter(|overlay| overlay.is_visible())
      .map(|overlay| overlay.sync_z_order(anchor, false));
    for result in [backdrop, border].into_iter().flatten() {
      if let Err(err) = result {
        tracing::warn!("Failed to restack {window_id}'s overlays: {err}");
      }
    }
  }
}

/// Hides the rings and backdrops [`cover_moved_overlays`] covered, now
/// that a flush has composited their copies: hidden on the first tick of
/// the move instead, a frame could show them still at rest beside their
/// moved copies.
pub fn hand_over_moved_overlays(state: &mut WmState, config: &UserConfig) {
  state.animation_manager.peeks.note_composited();
  track_overlays(state, config, SurrogateBatch::new());
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

  let peeks = &state.animation_manager.peeks;
  let tracked = peeks
    .entries
    .iter()
    .filter(|(id, _)| !peeks.standing_by.contains(id))
    .map(|(id, entry)| {
      // Only while some of the window is on screen.
      let frame = entry
        .peek
        .visible_rect()
        .and(entry.peek.frame_rect())
        .cloned();
      let is_moving = entry.role != Role::Peek;
      let anchor = overlay_anchor(state, *id, entry);
      // A leaving window keeps the look it left with.
      let kept_set = entry.departing.as_ref().map(|left| left.effect_set);
      let clip = entry.clip.clone();
      (*id, frame, clip, is_moving, anchor, kept_set)
    })
    .collect::<Vec<_>>();

  for (id, frame, clip, is_moving, anchor, kept_set) in tracked {
    let set = kept_set
      .unwrap_or_else(|| state.effect_set(id, focused_id == Some(id)));
    let (backdrop, border) = tracked_blends(state, id, set, config);

    // Read once for both, as each read is a round trip to DWM.
    let read = read_frame(state, id);
    let carries_backdrop =
      carry_backdrop(state, id, backdrop, read.as_ref());

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

    carry_tab_bar(state, id, focused_id == Some(id), config);

    if carry_ring(state, id, border, read.as_ref()) {
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
  // Its ring copies may have been registered or cut anew.
  state.animation_manager.peeks.restack_crossings();
}

/// The backdrop and ring `id`'s peek draws in `set`, blended across a
/// focus fade as steady state draws them. The peek fades its copies
/// itself (see `WindowPeek::place_scaled`).
fn tracked_blends(
  state: &WmState,
  id: Uuid,
  set: EffectSet,
  config: &UserConfig,
) -> (
  Option<StyleBlend<BackdropOverlayParams>>,
  Option<StyleBlend<BorderOverlayParams>>,
) {
  let backdrop =
    focus_blend::<NativeBackdropOverlay>(state, id, set, config);
  let border = focus_blend::<NativeBorderOverlay>(state, id, set, config)
    .filter(|_| !hides_ring_for_fullscreen(state, id, config));
  (backdrop, border)
}

/// The window the overlays of `id`, drawn by `entry`, sit behind: its
/// peek, or its real window once that is back on screen under a releasing
/// peek, where they stay once the peek goes.
fn overlay_anchor(state: &WmState, id: Uuid, entry: &Entry) -> HWND {
  if matches!(entry.role, Role::Releasing { .. }) {
    state
      .window_by_id(id)
      .map_or_else(|| entry.peek.hwnd(), |window| window.native().hwnd())
  } else {
    entry.peek.hwnd()
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
pub(super) fn carry_ring(
  state: &mut WmState,
  id: Uuid,
  blend: Option<StyleBlend<BorderOverlayParams>>,
  read: Option<&FrameRead>,
) -> bool {
  let _scope = perf::scope(Stage::PeekRing);
  let Some(read) = read else {
    return false;
  };
  // A leaving window's ring stays where it was, without a tab bar.
  let window = state.window_by_id(id).filter(|_| !read.is_leaving);
  // Shown where it was, or cloaked where its peek is headed.
  let (Some(bounds), Some(frame)) =
    (read.bounds.clone(), read.frame.clone())
  else {
    return false;
  };
  let native = read.native.clone();

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

  // A ring parked at a frame read off DWM that disagrees with the window's
  // own rect shows offset from its copy's window: logged to tell it from a
  // ring resized by a focus fade (below).
  if let Some(window) = &window {
    let expected =
      visible_frame(&bounds, &window.native_properties().shadow_borders);
    if expected != frame {
      tracing::debug!(
        "Ring of {id} parked at frame {frame:?}, off its window's rect \
         {bounds:?} (frame {expected:?} by its shadow borders)."
      );
    }
  }

  // Framed with its stack's tab bar, like the real window.
  let frame = landing_frame(state, id, frame);
  let ring_frame = window.as_ref().map_or_else(
    || frame.clone(),
    |window| crate::tab_bars::window_with_tab_bar(window, &frame),
  );
  let anchor = native.hwnd();

  let prep_scope = perf::scope(Stage::RingPrep);
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
  let outer_before = overlay.outer_rect();
  overlay.apply_blend(blend);
  drop(prep_scope);
  // The ring's window is sized for all of its window's styles, so only a
  // new reserve (a reload) re-places it and resizes its visual, by
  // separate routes, which may reach DWM in different frames.
  if overlay.outer_rect() != outer_before {
    tracing::debug!(
      "Ring of {id} resized by its style from {outer_before:?} to {:?}, \
       parked: {}.",
      overlay.outer_rect(),
      overlay.is_parked(),
    );
  }

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
      let _scope = perf::scope(Stage::RingPark);
      overlay.set_rect(&ring_frame, anchor);
      Handover::Revealed
    }
    Handover::Revealed if is_landing => Handover::Revealed,
    // A ring on screen is covered where it is first; one that isn't is
    // concealed at once.
    Handover::None | Handover::Revealed => {
      let is_shown = overlay.is_visible();
      let park_scope = perf::scope(Stage::RingPark);
      overlay.park(&ring_frame, anchor, !is_shown);
      drop(park_scope);

      let ring_rect = overlay.outer_rect();
      let copy_scope = perf::scope(Stage::RingCopy);
      if !entry.carry_ring(
        overlay.hwnd(),
        &ring_rect,
        &bounds,
        corner_reach,
      ) {
        return false;
      }
      drop(copy_scope);

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
      let park_scope = perf::scope(Stage::RingPark);
      overlay.conceal();
      drop(park_scope);
      let _scope = perf::scope(Stage::RingCopy);
      for (peek, _) in entry.peeks_mut() {
        peek.cut_ring(corner_reach);
      }
      Handover::Parked
    }
    Handover::Covered | Handover::Parked => {
      let park_scope = perf::scope(Stage::RingPark);
      overlay.park(&ring_frame, anchor, true);
      drop(park_scope);
      let ring_rect = overlay.outer_rect();
      let _scope = perf::scope(Stage::RingCopy);
      entry.carry_ring(overlay.hwnd(), &ring_rect, &bounds, corner_reach);
      Handover::Parked
    }
  };

  true
}

/// What a hosted peek does with the copy of its stack's tab bar this
/// frame, given how far it had taken the bar over (`handover`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarStep {
  /// Draws it, registered if it isn't, where the bar goes now.
  Carry,
  /// Leaves it as it is.
  Keep,
  /// Stops drawing it.
  Drop,
}

/// How far a hosted peek's copy of its stack's tab bar takes over from the
/// bar, handed over as its ring is (see [`Handover`]), and what it does
/// with it this frame: the bar is cloaked while [`conceals_bar`], so the
/// copy shows it alone, and uncloaked a composited frame before the copy
/// goes.
///
/// `wanted`: the peek may draw the bar (see [`carry_tab_bar`]). Landing,
/// the bar is shown again under the copy, which stays until the peek goes.
/// No longer wanted otherwise (its window no longer the active tab, or the
/// bar hidden), a copy never composited goes at once; one the bar was
/// cloaked for is revealed first, and goes a tick later.
fn bar_handover(
  handover: Handover,
  wanted: bool,
  is_landing: bool,
) -> (Handover, BarStep) {
  let step = if wanted {
    BarStep::Carry
  } else {
    BarStep::Keep
  };

  match handover {
    Handover::None if is_landing || !wanted => {
      (Handover::None, BarStep::Keep)
    }
    Handover::None => (Handover::Covering, BarStep::Carry),
    Handover::Covering | Handover::Covered | Handover::Parked
      if is_landing =>
    {
      (Handover::Revealed, step)
    }
    Handover::Revealed if is_landing => (Handover::Revealed, step),
    Handover::Covering | Handover::Covered if !wanted => {
      (Handover::None, BarStep::Drop)
    }
    Handover::Parked if !wanted => (Handover::Revealed, BarStep::Keep),
    Handover::Revealed if !wanted => (Handover::None, BarStep::Drop),
    Handover::Covering | Handover::Revealed => {
      (Handover::Covering, BarStep::Carry)
    }
    Handover::Covered | Handover::Parked => {
      (Handover::Parked, BarStep::Carry)
    }
  }
}

/// Whether a tab bar copied by a peek in `handover` is cloaked, shown by
/// the copy alone.
pub(super) fn conceals_bar(handover: Handover) -> bool {
  handover == Handover::Parked
}

/// Keeps `id`'s hosted peek drawing its stack's tab bar beside the window,
/// over the other tiles on its surface as the window is, while the window
/// is that stack's active tab and moves, lands or rests with no transit,
/// and hands the bar over to the copy a step at a time (see
/// [`bar_handover`]).
///
/// A peek of its own draws none: the bar goes behind it (see
/// `tab_bars::shown_tab_frame`), as it always has. So does a window
/// entering or leaving, which its bar doesn't go along with.
fn carry_tab_bar(
  state: &mut WmState,
  id: Uuid,
  is_focused: bool,
  config: &UserConfig,
) {
  let Some(entry) = state.animation_manager.peeks.entries.get(&id) else {
    return;
  };
  let is_landing =
    matches!(entry.role, Role::HandingBack | Role::Releasing { .. });
  let may_draw = entry.peek.is_hosted()
    && entry.transit.is_none()
    && entry.departing.is_none()
    && !matches!(entry.role, Role::Leaving(_));
  let target = entry.motion.target_rect.clone();

  let bar = may_draw.then(|| state.window_by_id(id)).flatten().and_then(
    |window| {
      let stack = window.parent()?.as_stack().cloned()?;
      let bar = state
        .tab_bars
        .get(&stack.id())
        .filter(|bar| bar.is_shown())?;
      let place = crate::tab_bars::tab_bar_place(
        &window, &target, is_focused, config,
      )?;
      Some((bar.hwnd(), place))
    },
  );

  let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id)
  else {
    return;
  };
  let (handover, step) =
    bar_handover(entry.bar, bar.is_some(), is_landing);
  entry.bar = match (step, bar) {
    (BarStep::Carry, Some((source, place))) => {
      if entry.peek.carry_tab_bar(source, place) {
        handover
      } else {
        Handover::None
      }
    }
    (BarStep::Drop, _) => {
      entry.peek.drop_tab_bar();
      handover
    }
    _ => handover,
  };
}

/// Where `id`'s ring and backdrop go back to as its peek lands, its real
/// window's visible frame being `frame`: there, unless the window was
/// sent its size only as it lands (see
/// [`Stretch::deferred`](super::stretch::Stretch::deferred)), whose frame
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
pub(super) fn visible_frame(
  rect: &Rect,
  shadow_borders: &RectDelta,
) -> Rect {
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
#[allow(clippy::too_many_lines)]
fn carry_backdrop(
  state: &mut WmState,
  id: Uuid,
  blend: Option<StyleBlend<BackdropOverlayParams>>,
  read: Option<&FrameRead>,
) -> bool {
  let _scope = perf::scope(Stage::PeekBackdrop);
  let carries = state
    .animation_manager
    .peeks
    .entries
    .get(&id)
    .is_some_and(|entry| entry.peek.carries_backdrop());
  let Some(read) = read.filter(|_| carries) else {
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
  let Some(frame) = read.frame.clone() else {
    return false;
  };
  let frame = if read.is_leaving {
    frame
  } else {
    landing_frame(state, id, frame)
  };
  let anchor = read.native.hwnd();

  let Some(entry) = state.animation_manager.peeks.entries.get_mut(&id)
  else {
    return false;
  };
  let is_landing =
    matches!(entry.role, Role::HandingBack | Role::Releasing { .. });
  let copy_scope = perf::scope(Stage::BackdropCopy);
  for (peek, _) in entry.peeks_mut() {
    peek.show_backdrop();
    peek.show_style_blend(&blend);
  }
  drop(copy_scope);

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
    let _scope = perf::scope(Stage::BackdropPark);
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
          let _scope = perf::scope(Stage::BackdropPark);
          overlay.hide();
        }
        Handover::Parked
      }
    }
  };

  true
}

/// The real window, rect with shadows and frame that `id`'s peek kept as
/// it started to leave, if it is leaving (see
/// [`Departing`](super::Departing)). Where `id`'s window is, read once a
/// pass for both [`carry_backdrop`] and [`carry_ring`]: a leaving window's
/// where it left, another's own rect and visible frame, each `None` if its
/// read failed.
pub(super) struct FrameRead {
  native: NativeWindow,
  bounds: Option<Rect>,
  frame: Option<Rect>,
  is_leaving: bool,
}

/// [`FrameRead`] of `id`'s window, `None` for one neither leaving nor
/// managed.
fn read_frame(state: &WmState, id: Uuid) -> Option<FrameRead> {
  if let Some((native, bounds, frame)) = leaving_window(state, id) {
    return Some(FrameRead {
      native,
      bounds: Some(bounds),
      frame: Some(frame),
      is_leaving: true,
    });
  }

  let native = state.window_by_id(id)?.native().clone();
  Some(FrameRead {
    bounds: native.frame_with_shadows().ok(),
    frame: native.frame().ok(),
    native,
    is_leaving: false,
  })
}

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

/// What a peek of a window carries copies of its backdrop as.
pub(super) struct PeekStyles {
  /// Its backdrops focused and not (see [`peek_styles`]).
  pub(super) backdrops: [Option<BackdropOverlayParams>; 2],
  /// The narrowest ring drawn over the edge of either, which hides the
  /// steps of rounded corners cut in rows, or `None` if one has no ring.
  pub(super) ring_width: Option<f32>,
}

/// [`PeekStyles`] of `window`.
pub(super) fn carried_styles(
  window: &WindowContainer,
  config: &UserConfig,
) -> PeekStyles {
  // The ring over each backdrop drawn, `None` for one without.
  let rings = [true, false].into_iter().filter_map(|is_focused| {
    let params =
      surrogate_effects_for(window.effect_set(is_focused), config).2;
    params.backdrop.map(|_| {
      params
        .border
        .filter(|_| !border_hidden_for_fullscreen(window, config))
        .map(|border| border.width)
    })
  });

  PeekStyles {
    backdrops: peek_styles(window, config),
    ring_width: narrowest_ring(rings),
  }
}

/// The narrowest of `rings`, or `None` if any is (no ring drawn).
fn narrowest_ring(
  rings: impl IntoIterator<Item = Option<f32>>,
) -> Option<f32> {
  rings
    .into_iter()
    .try_fold(None, |narrowest: Option<f32>, width| {
      width.map(|width| Some(narrowest.map_or(width, |n| n.min(width))))
    })
    .flatten()
}

/// The backdrops of `window` focused and not, which its peek carries
/// copies of.
pub(in crate::animation) fn peek_styles(
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

  use wm_platform::{LengthValue, Rect, RectDelta};

  use super::{conceals_new_ring, visible_frame};
  use crate::animation::{peek_view::Role, AwayTransition};

  /// A hosted peek covers its stack's bar, cloaks it once composited, and
  /// shows it again before the copy goes: as its window lands, or a
  /// composited tick ahead once the window no longer has the bar.
  #[test]
  fn a_tab_bar_is_handed_over_as_a_ring_is() {
    use super::{bar_handover, conceals_bar, BarStep, Handover};

    let start = bar_handover(Handover::None, true, false);
    assert_eq!(start, (Handover::Covering, BarStep::Carry));
    assert!(!conceals_bar(start.0));
    // `note_composited` turns `Covering` to `Covered`.
    let parked = bar_handover(Handover::Covered, true, false);
    assert_eq!(parked, (Handover::Parked, BarStep::Carry));
    assert!(conceals_bar(parked.0));

    let landed = bar_handover(Handover::Parked, true, true);
    assert_eq!(landed, (Handover::Revealed, BarStep::Carry));
    assert!(!conceals_bar(landed.0));
    assert_eq!(
      bar_handover(Handover::Revealed, false, true),
      (Handover::Revealed, BarStep::Keep)
    );
    // Moved on in place after landing: covered again, the bar shown.
    assert_eq!(
      bar_handover(Handover::Revealed, true, false),
      (Handover::Covering, BarStep::Carry)
    );

    let lost = bar_handover(Handover::Parked, false, false);
    assert_eq!(lost, (Handover::Revealed, BarStep::Keep));
    assert_eq!(
      bar_handover(lost.0, false, false),
      (Handover::None, BarStep::Drop)
    );
    assert_eq!(
      bar_handover(Handover::Covering, false, false),
      (Handover::None, BarStep::Drop)
    );
    assert_eq!(
      bar_handover(Handover::None, true, true),
      (Handover::None, BarStep::Keep)
    );
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

  #[test]
  fn rounded_rows_are_cut_for_the_narrowest_ring() {
    assert_eq!(super::narrowest_ring([Some(3.0), Some(2.0)]), Some(2.0));
    assert_eq!(super::narrowest_ring([Some(3.0), None]), None);
    assert_eq!(super::narrowest_ring([Some(4.0)]), Some(4.0));
    assert_eq!(super::narrowest_ring([]), None);
  }
}
