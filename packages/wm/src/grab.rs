//! Grab gestures: moving and resizing windows by dragging anywhere on them
//! with a modifier held (`general.grab_and_move`, see
//! `docs/grab-and-move.md`).
//!
//! A move runs as a drag by the title bar does: the platform moves the
//! window with the cursor, and its moves arrive as for any interactive
//! drag. A resize is laid out by the WM on every move of the mouse, its
//! neighbours along with it, unanimated. Windows that only change rect
//! are placed by a `WindowPlacer`, each window on its own, rather than by
//! `platform_sync`; the rest of the layout's changes go through it.
//!
//! Either way, the WM places the border and backdrop overlays of the
//! windows it moves where they are going as it hands their rects over,
//! rather than once each window reports its move: the window's app paces
//! its move, and the overlays would trail it by a frame or more.

use std::collections::{HashMap, HashSet};

use wm_common::{
  ActiveDragOperation, DisplayState, GrabDraw, Layout, WindowState,
};
use wm_platform::{
  Gesture, GrabEnd, GrabEvent, GrabListener, GrabSettings, NativeWindow,
  Point, Rect, RectDelta, ResizeEdges, SurrogateBatch, WindowId,
  WindowPlacer,
};

use crate::{
  animation::manager::{movement_timing, Motion},
  commands::{
    container::set_focused_descendant,
    general::{defer_window_frame, follow_window_frame, note_size_change},
    window::{MIN_FLOATING_HEIGHT, MIN_FLOATING_WIDTH},
  },
  events::{end_window_drag, handle_window_moved_or_resized},
  layout::{
    edge_resize::{drag_and_hold, dragged_rect, SizeSnapshot},
    is_hidden_in_layout,
    scrolling::{column_of, real_window_rect, sync_view, ColumnHold},
  },
  models::{Container, WindowContainer, Workspace},
  tab_bars::{sync_tab_bars, Restack},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// What [`GrabListener`] grabs with under `config`, if anything.
pub fn settings(config: &UserConfig) -> Option<GrabSettings> {
  let grab = &config.value.general.grab_and_move;

  grab.enabled.then_some(GrabSettings {
    modifier: grab.modifier,
    resizes: grab.resize,
  })
}

#[derive(Debug, Default)]
pub struct GrabState {
  grab: Option<Grab>,

  /// Whether each managed window is one the WM lacks the rights to move,
  /// asked once per window.
  out_of_reach: HashMap<WindowId, bool>,

  /// Whether another window took the foreground during the grab, e.g. a
  /// UAC prompt, which the grab's release may be made on.
  is_focus_lost: bool,

  /// Places the windows a resize only changes the rect of.
  placer: WindowPlacer,
}

#[derive(Debug)]
struct Grab {
  native: NativeWindow,

  /// The window's rect as the grab began.
  start: Rect,

  resize: Option<Resize>,
}

#[derive(Debug)]
struct Resize {
  edges: ResizeEdges,

  /// The tiling sizes of the window's workspace as the resize began,
  /// which every step resizes from.
  snapshot: Option<(Workspace, SizeSnapshot)>,

  /// Where the window's column is held on screen, and where it started.
  hold: Option<(ColumnHold, ColumnHold)>,

  /// Movement the last step applied.
  applied: (i32, i32),
}

/// Handles `event` from `listener`.
pub fn handle_grab_event(
  event: GrabEvent,
  listener: &GrabListener,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match event {
    GrabEvent::Started { window, gesture } => {
      start(window, gesture, listener, state, config)
    }
    GrabEvent::Resized => match listener.take_resize_delta() {
      Some(delta) => step(delta, state, config),
      // Ended since: its end brings the final total.
      None => Ok(()),
    },
    GrabEvent::Moved => {
      // Ended since: its end follows where the window landed.
      if let Some(position) = listener.take_move_position() {
        follow_move(&position, state, config);
      }
      Ok(())
    }
    GrabEvent::Ended {
      window,
      end,
      resize_total,
    } => {
      let is_current = state
        .grab
        .grab
        .as_ref()
        .is_some_and(|grab| grab.native.id() == window.id());

      if !is_current {
        return Ok(());
      }

      finish(end, resize_total, state, config)
    }
  }
}

/// Lays out the resize under way for the mouse's newest movement `delta`.
fn step(
  delta: (i32, i32),
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(mut grab) = state.grab.grab.take() else {
    return Ok(());
  };

  let result = match state.window_from_native(&grab.native) {
    Some(window) if grab.resize.is_some() => {
      apply_resize(&mut grab, &window, delta, state, config).map(|_| ())
    }
    _ => Ok(()),
  };

  state.grab.grab = Some(grab);
  result
}

/// Places the overlays of the window a move is under way on where its
/// top-left corner goes to `position`, as its mover moves it there.
fn follow_move(
  position: &Point,
  state: &mut WmState,
  config: &UserConfig,
) {
  let window = state
    .grab
    .grab
    .as_ref()
    .filter(|grab| grab.resize.is_none())
    .and_then(|grab| state.window_from_native(&grab.native))
    .filter(|window| window.active_drag().is_some());

  if let Some(window) = window {
    let properties = window.native_properties();
    let frame =
      moved_frame(&properties.frame, &properties.shadow_borders, position);
    follow_window_frame(&window, &frame, state, config);
  }
}

/// Whether the move under way places the overlays of `window`, whose
/// frame went from `old_frame` to `frame`: a translation, which its mover
/// made or is overtaking. The moved window's events then don't place
/// them, as a late one would put them back where a newer step left.
///
/// A change in size is the app's own (e.g. rescaling for a monitor's DPI),
/// which the overlays follow, and for which the shadow borders the moves
/// convert by are read anew.
pub fn places_move_overlays(
  window: &WindowContainer,
  old_frame: &Rect,
  frame: &Rect,
  state: &WmState,
) -> bool {
  let is_moved = state.grab.grab.as_ref().is_some_and(|grab| {
    grab.resize.is_none() && grab.native.id() == window.native().id()
  });

  if !is_moved {
    return false;
  }

  if old_frame.width() == frame.width()
    && old_frame.height() == frame.height()
  {
    return true;
  }

  // Bound by `let` so the `native()` borrow ends before the update.
  let shadow_borders = window.native().shadow_borders();
  if let Ok(shadow_borders) = shadow_borders {
    window.update_native_properties(|properties| {
      properties.shadow_borders = shadow_borders;
    });
  }

  false
}

/// Whether a resize under way on `window`'s workspace draws it on a peek
/// it holds (`grab_and_move.draw: "peeks"`, see
/// `peek_view::hold_on_peek`): a window the peek can draw, already held or
/// to be put on one by the redraw of its first step.
pub fn holds_on_peek(
  window: &WindowContainer,
  state: &WmState,
  config: &UserConfig,
) -> bool {
  let is_resizing_here = state.grab.grab.as_ref().is_some_and(|grab| {
    grab.resize.is_some()
      && state
        .window_from_native(&grab.native)
        .and_then(|grabbed| grabbed.workspace())
        .zip(window.workspace())
        .is_some_and(|(grabbed, workspace)| grabbed.id() == workspace.id())
  });

  config.value.general.grab_and_move.draw == GrabDraw::Peeks
    && is_resizing_here
    && (state.animation_manager.peeks.is_held(&window.id())
      || crate::animation::peek_view::can_hold(state, window, config))
}

impl GrabState {
  /// The smallest size, per axis, the resize under way sent `window` that
  /// it may not have applied yet.
  pub fn unlanded(&self, window: WindowId) -> Option<(i32, i32)> {
    self.placer.unlanded(window)
  }
}

/// Whether a resize's placer may not have placed `window` at its newest
/// rect yet, whose overlays are already there: an event of the window's
/// then shows an older rect, which they mustn't go back to.
pub fn is_placing(window: &WindowContainer, state: &WmState) -> bool {
  state.grab.placer.is_placing(window.native().id())
}

/// Notes that `native` took the foreground, which ends a grab of another
/// window at the next `sync_listener`.
///
/// Only if it still has the foreground: an event from before the grab
/// focused its window, handled after the grab began, is no such loss.
pub fn note_focused(state: &mut WmState, native: &NativeWindow) {
  let is_other = state.grab.grab.as_ref().is_some_and(|grab| {
    grab.native.id() != native.id()
      && state.dispatcher.focused_window().id() == native.id()
  });

  if is_other {
    state.grab.is_focus_lost = true;
  }
}

/// Hands `listener` the windows it may grab, and ends a gesture whose
/// window can no longer take it, e.g. gone, or with the WM paused.
pub fn sync_listener(
  listener: &GrabListener,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if !listener.is_hooked() {
    if state.grab.grab.is_some() {
      return end_early(listener, state, config);
    }
    return Ok(());
  }

  if let Some(grab) = &state.grab.grab {
    let is_lost = state.is_paused
      || state.grab.is_focus_lost
      || state.window_from_native(&grab.native).is_none_or(|window| {
        matches!(
          window.state(),
          WindowState::Fullscreen(_) | WindowState::Minimized
        ) || window
          .workspace()
          .is_none_or(|workspace| !workspace.is_displayed())
      });

    if is_lost {
      end_early(listener, state, config)?;
    }

    return Ok(());
  }

  let windows = if state.is_paused || state.overview.is_shown() {
    Vec::new()
  } else {
    state.windows()
  };

  let ids = windows.iter().map(|window| window.native().id());
  let ids = ids.collect::<HashSet<_>>();
  state.grab.out_of_reach.retain(|id, _| ids.contains(id));

  let grabbable = windows
    .iter()
    .filter(|window| is_grabbable(window, state))
    .map(|window| window.native().id())
    .collect::<Vec<_>>();

  let out_of_reach = &mut state.grab.out_of_reach;
  listener.set_windows(grabbable.into_iter().filter(|id| {
    !*out_of_reach
      .entry(*id)
      .or_insert_with(|| NativeWindow::from_handle(id.0).is_out_of_reach())
  }));

  Ok(())
}

/// Whether a press on `window` may grab it: a window shown where it is
/// drawn, which the mouse can place, unlike a fullscreen one.
fn is_grabbable(window: &WindowContainer, state: &WmState) -> bool {
  matches!(
    window.state(),
    WindowState::Tiling | WindowState::Floating(_)
  ) && window.display_state() == DisplayState::Shown
    && window
      .workspace()
      .is_some_and(|workspace| workspace.is_displayed())
    && !is_hidden_in_layout(window)
    && !state.animation_manager.owns_window_position(&window.id())
    && !state.animation_manager.peeks.owns_window(&window.id())
}

fn start(
  native: NativeWindow,
  gesture: Gesture,
  listener: &GrabListener,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let window = state
    .window_from_native(&native)
    .filter(|window| !state.is_paused && is_grabbable(window, state));

  let (Some(window), None) = (window, &state.grab.grab) else {
    listener.end_gesture();
    return Ok(());
  };

  tracing::info!("Window grabbed: {window}");

  let start = window.to_rect()?;

  let resize = match gesture {
    Gesture::Move => None,
    Gesture::Resize(edges) => Some(Resize::new(&window, edges)?),
  };

  // As a click on it would. Its column is held where it is: focusing it
  // would scroll one partly off screen into view, from under the cursor.
  if let Some((column, workspace)) = column_of(&window.as_container()) {
    ColumnHold::at(&column, &workspace)?.keep(&workspace);
  }
  set_focused_descendant(&window.as_container(), None);
  state
    .pending_sync
    .queue_focus_change()
    .suppress_animations();

  if resize.is_none() {
    handle_window_moved_or_resized(&native, true, false, state, config)?;

    // A move from the start, measured from where the window is now: the
    // frame cached from its last event may be stale, which would take
    // the drag for a resize that never floats the window.
    if let Some(window) = state.window_from_native(&native) {
      if let Some(drag) = window.active_drag() {
        window.set_active_drag(Some(wm_common::ActiveDrag {
          operation: Some(ActiveDragOperation::Move),
          initial_position: window.native_properties().frame,
          ..drag
        }));
      }
    }
  }

  state.grab.grab = Some(Grab {
    native,
    start,
    resize,
  });
  state.grab.is_focus_lost = false;

  Ok(())
}

impl Resize {
  fn new(
    window: &WindowContainer,
    edges: ResizeEdges,
  ) -> anyhow::Result<Self> {
    let snapshot = match window {
      WindowContainer::TilingWindow(_) => window.workspace().map(|ws| {
        let snapshot = SizeSnapshot::of(&ws);
        (ws, snapshot)
      }),
      WindowContainer::NonTilingWindow(_) => None,
    };

    let hold = column_of(&window.as_container())
      .map(|(column, workspace)| ColumnHold::at(&column, &workspace))
      .transpose()?
      .map(|hold| (hold, hold));

    Ok(Self {
      edges,
      snapshot,
      hold,
      applied: (0, 0),
    })
  }
}

/// Resizes `window`, grabbed by `grab`, for the mouse's movement `delta`
/// in all, and places the windows whose rect changed (see
/// [`place_moved_windows`]).
///
/// Returns the containers holding every window the resize can move, for
/// its end to redraw.
fn apply_resize(
  grab: &mut Grab,
  window: &WindowContainer,
  delta: (i32, i32),
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Vec<Container>> {
  let Some(resize) = &mut grab.resize else {
    return Ok(Vec::new());
  };

  resize.applied = delta;

  let redraw: Vec<Container> = match window {
    WindowContainer::TilingWindow(tiling) => {
      let Some((workspace, snapshot)) = &resize.snapshot else {
        return Ok(Vec::new());
      };

      // A window opened or closed since: its tiling sizes are no longer
      // the ones it resizes from, so it stays as it is.
      if !snapshot.matches(workspace) {
        return Ok(Vec::new());
      }

      snapshot.restore();
      let hold = resize.hold.as_mut().map(|(hold, _)| hold);
      drag_and_hold(tiling, resize.edges, delta, hold)?;

      // Scrolled now rather than by the sync, so that the windows' rects
      // below are where the sync puts them.
      if workspace.layout() == Layout::Scrolling {
        sync_view(workspace)?;
      }

      // All of them, as against where they were last placed: a step can
      // put back a window an earlier one moved, and a scroll move any.
      let workspace: Container = workspace.clone().into();
      place_moved_windows(std::slice::from_ref(&workspace), state, config);
      vec![workspace]
    }
    WindowContainer::NonTilingWindow(floating) => {
      if !matches!(floating.state(), WindowState::Floating(_)) {
        return Ok(Vec::new());
      }

      // Never below what the app allows, nor grown by a resize that
      // started smaller than the usual minimum.
      let min_width = MIN_FLOATING_WIDTH
        .min(grab.start.width())
        .max(floating.native_properties().min_width);
      let min_height = MIN_FLOATING_HEIGHT.min(grab.start.height());

      let rect = dragged_rect(
        &grab.start,
        resize.edges,
        delta,
        min_width,
        min_height,
      );
      floating.set_floating_placement(rect);
      floating.set_has_custom_floating_placement(true);

      // A floating stack resizes as one.
      let moved = vec![window
        .parent()
        .filter(|parent| parent.as_stack().is_some())
        .unwrap_or_else(|| window.as_container())];

      place_moved_windows(&moved, state, config);
      moved
    }
  };

  state.pending_sync.suppress_animations();
  Ok(redraw)
}

/// Places the windows of `containers` whose rect changed, each on its
/// own, at once: the grabbed one doesn't wait on its neighbours, nor on
/// the main loop's next sync.
///
/// Only windows that stay shown in place take this path; one that shows,
/// hides, or is drawn by an animation or a peek is redrawn by
/// `platform_sync` as usual. A stack's hidden tabs are left to the
/// gesture's end. Their overlays are placed here in one commit, and tab
/// bars synced; a window's own events correct its overlays once it has
/// landed (see [`is_placing`]), e.g. at a size its app insists on.
fn place_moved_windows(
  containers: &[Container],
  state: &mut WmState,
  config: &UserConfig,
) {
  let mut seen = HashSet::new();
  let mut has_stack = false;
  let mut overlays = SurrogateBatch::new();

  let windows = containers
    .iter()
    .flat_map(CommonGetters::self_and_descendants)
    .filter_map(|container| container.as_window_container().ok())
    .filter(|window| seen.insert(window.id()))
    .collect::<Vec<_>>();

  for window in windows {
    let Ok(target) = window.to_rect().and_then(|rect| {
      window
        .total_border_delta()
        .map(|delta| rect.apply_delta(&delta, None))
    }) else {
      continue;
    };

    if state.window_target_positions.get(&window.id()) == Some(&target) {
      continue;
    }

    let shows = window.workspace().is_some_and(|ws| ws.is_displayed())
      && !is_hidden_in_layout(&window);

    if is_placeable(&window, shows, &target, state, config) {
      note_size_change(&window, &target);
      state
        .grab
        .placer
        .place(window.native().id(), target.clone());

      // Held on a peek, which draws its overlays: the peek jumps to its
      // new tile, drawn there on the next frame.
      if state.animation_manager.peeks.is_held(&window.id()) {
        let now = state.animation_manager.frame_now();
        let (_, easing) = movement_timing(Motion::Resize, true, config);
        state.animation_manager.peeks.retarget_tile(
          &window.id(),
          target.clone(),
          0,
          easing,
          true,
          now,
        );
      } else if !state.animation_manager.has_active_surrogate(&window.id())
      {
        // As `follow_own_move` has them follow a window's own move.
        let frame =
          frame_at(&target, &window.native_properties().shadow_borders);
        defer_window_frame(&window, &frame, state, config, &mut overlays);
      }

      state.window_target_positions.insert(window.id(), target);
      has_stack |= window
        .parent()
        .is_some_and(|parent| parent.as_stack().is_some());
    } else if shows || window.display_state() != DisplayState::Hidden {
      state.grab.placer.forget(window.native().id());
      state.pending_sync.queue_container_to_redraw(window);
    }
  }

  overlays.commit();

  if has_stack {
    sync_tab_bars(state, config, Restack::None);
  }
}

/// The visible frame, as `frame()` reads it, of a window placed at `rect`,
/// which takes in its invisible `shadow_borders`.
fn frame_at(rect: &Rect, shadow_borders: &RectDelta) -> Rect {
  rect.apply_delta(&shadow_borders.inverse(), None)
}

/// [`frame_at`] for a window shown at `frame` moved for its top-left
/// corner, shadow borders included, to be at `position`.
fn moved_frame(
  frame: &Rect,
  shadow_borders: &RectDelta,
  position: &Point,
) -> Rect {
  let rect = frame
    .apply_delta(shadow_borders, None)
    .translate_to_coordinates(position.x, position.y);

  frame_at(&rect, shadow_borders)
}

/// Whether `window`, which `shows` in the layout, can be placed at
/// `target` by the placer: shown there already, at a rect of its own, and
/// with nothing else drawing it but a peek the resize holds. One the
/// resize is to hold is left to the redraw, which puts it on its peek.
fn is_placeable(
  window: &WindowContainer,
  shows: bool,
  target: &Rect,
  state: &WmState,
  config: &UserConfig,
) -> bool {
  let id = window.id();
  let is_held = state.animation_manager.peeks.is_held(&id);

  if !is_held && holds_on_peek(window, state, config) {
    return false;
  }

  shows
    && window.display_state() == DisplayState::Shown
    && matches!(
      window.state(),
      WindowState::Tiling | WindowState::Floating(_)
    )
    && window.active_drag().is_none()
    && !window.has_pending_dpi_adjustment()
    && !state.animation_manager.owns_window_position(&id)
    && (is_held || !state.animation_manager.peeks.owns_window(&id))
    && real_window_rect(&window.as_container(), target) == *target
}

/// Ends the grab under way as `end` says, a resize at the mouse's final
/// movement `delta`, or else at the last one applied.
fn finish(
  end: GrabEnd,
  delta: Option<(i32, i32)>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(mut grab) = state.grab.grab.take() else {
    return Ok(());
  };

  tracing::info!("Window grab ended: {end:?}");

  let Some(resize) = &mut grab.resize else {
    // A cancelled move put its window back: it drops where it started.
    let drop_point =
      (end == GrabEnd::Cancelled).then(|| grab.start.center_point());

    return end_window_drag(
      &grab.native,
      drop_point.as_ref(),
      state,
      config,
    );
  };

  let Some(window) = state.window_from_native(&grab.native) else {
    state.grab.placer.finish();
    state.animation_manager.peeks.release_holds();
    return Ok(());
  };

  let is_cancelled = end == GrabEnd::Cancelled;
  let start_hold = resize.hold.map(|(_, start)| start);
  let delta = if is_cancelled {
    (0, 0)
  } else {
    delta.unwrap_or(resize.applied)
  };

  let redraw = apply_resize(&mut grab, &window, delta, state, config)?;

  if is_cancelled {
    // Back where it started, its column too, even partly off screen.
    if let (Some(hold), Some((_, workspace))) =
      (start_hold, column_of(&window.as_container()))
    {
      hold.keep(&workspace);
      sync_view(&workspace)?;
      place_moved_windows(&[workspace.into()], state, config);
    }
  } else {
    crate::particles::note_resize(state, &window.as_container());
  }

  // The placer's last rects are the final ones, which the sync of every
  // window the resize can have moved sends too: whichever lands last,
  // each ends where the layout says, with its overlays and tab bar. Peeks
  // it held land once their windows have reached them.
  state.grab.placer.finish();
  state.animation_manager.peeks.release_holds();
  for container in redraw {
    state.pending_sync.queue_container_to_redraw(container);
  }

  Ok(())
}

/// Ends the grab under way as the WM, not the mouse, ends it, keeping
/// what it did so far.
fn end_early(
  listener: &GrabListener,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  listener.end_gesture();
  finish(GrabEnd::Released, None, state, config)
}

#[cfg(test)]
mod tests {
  use std::mem::ManuallyDrop;

  use wm_common::{
    ActiveDrag, ActiveDragOperation, DisplayState, GrabDraw, ParsedConfig,
    WindowState,
  };
  use wm_platform::{
    Gesture, GrabEnd, GrabListener, LengthValue, NativeWindow, Point,
    Rect, RectDelta, ResizeEdges,
  };

  use super::{
    finish, frame_at, holds_on_peek, is_grabbable, moved_frame,
    places_move_overlays, start, step,
  };
  use crate::{
    events::float_for_drag,
    models::{
      Monitor, NonTilingWindow, TilingWindow, WindowContainer, Workspace,
    },
    test_utils::{mock_config, mock_inner_gaps},
    traits::{
      CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
    },
    user_config::UserConfig,
    wm_state::WmState,
  };

  const RIGHT: ResizeEdges = ResizeEdges {
    left: false,
    right: true,
    top: false,
    bottom: false,
  };

  /// A split workspace of three windows told apart by their handles,
  /// never dropped, which would restore them via Win32 calls.
  fn setup() -> (ManuallyDrop<WmState>, [TilingWindow; 3]) {
    let windows = [1, 2, 3].map(|handle| {
      TilingWindow::mock()
        .gaps_config(mock_inner_gaps())
        .native(NativeWindow::from_handle(handle))
        .call()
    });
    let workspace = Workspace::mock()
      .gaps_config(mock_inner_gaps())
      .tiling_containers(
        windows.iter().map(|w| w.clone().into()).collect(),
      )
      .call();
    let mut state =
      ManuallyDrop::new(WmState::mock(vec![Monitor::mock()
        .workspaces(vec![workspace])
        .call()]));

    // Where a sync put them.
    for window in &windows {
      state
        .window_target_positions
        .insert(window.id(), target(window));
    }

    (state, windows)
  }

  fn target(window: &TilingWindow) -> wm_platform::Rect {
    window
      .to_rect()
      .unwrap()
      .apply_delta(&window.total_border_delta().unwrap(), None)
  }

  /// Grabs `window` to resize its right edge, from a clean sync.
  fn grab_right_edge(window: &TilingWindow, state: &mut WmState) {
    let native = window.native().clone();
    let gesture = Gesture::Resize(RIGHT);
    start(native, gesture, &GrabListener::new(), state, &mock_config())
      .unwrap();
    state.pending_sync.clear();
  }

  /// Takes a step for the mouse moved `dx` right in all.
  fn step_to(dx: i32, state: &mut WmState) {
    step((dx, 0), state, &mock_config()).unwrap();
  }

  fn is_queued(window: &TilingWindow, state: &WmState) -> bool {
    state
      .pending_sync
      .containers_to_redraw()
      .contains_key(&window.id())
  }

  fn is_placed(window: &TilingWindow, state: &WmState) -> bool {
    state.window_target_positions.get(&window.id())
      == Some(&target(window))
  }

  #[test]
  fn a_step_places_the_window_and_its_neighbour_at_once() {
    let (mut state, [left, middle, right]) = setup();
    let before = target(&left);
    grab_right_edge(&middle, &mut state);

    step_to(100, &mut state);

    assert!(middle.tiling_size() > right.tiling_size());
    assert!((left.tiling_size() - 1. / 3.).abs() < 1e-4);
    assert!(state.pending_sync.animations_suppressed());

    // Placed directly, not left to the next sync.
    assert!(is_placed(&middle, &state) && is_placed(&right, &state));
    assert!(!is_queued(&middle, &state) && !is_queued(&right, &state));
    assert_eq!(
      state.window_target_positions.get(&left.id()),
      Some(&before)
    );
  }

  /// Drawn on peeks, a resize still places at once the windows a peek
  /// can't draw (here, mocks with no real window to read), and holds
  /// nothing.
  #[test]
  fn on_peeks_a_window_no_peek_can_draw_is_placed_live() {
    let (mut state, [_, middle, right]) = setup();
    let mut parsed = ParsedConfig::default();
    parsed.general.grab_and_move.draw = GrabDraw::Peeks;
    let config = UserConfig::from_parsed(parsed);
    grab_right_edge(&middle, &mut state);

    for window in [&middle, &right] {
      assert!(!holds_on_peek(&window.clone().into(), &state, &config));
    }
    step((100, 0), &mut state, &config).unwrap();

    assert!(is_placed(&middle, &state) && is_placed(&right, &state));
    assert!(!is_queued(&middle, &state) && !is_queued(&right, &state));
    assert!(!state.animation_manager.peeks.is_held(&middle.id()));
  }

  /// Drawn live, the default, no window is ever held on a peek.
  #[test]
  fn live_resizes_hold_nothing() {
    let (mut state, [_, middle, _]) = setup();
    grab_right_edge(&middle, &mut state);

    assert!(!holds_on_peek(
      &middle.clone().into(),
      &state,
      &mock_config()
    ));
  }

  #[test]
  fn a_window_not_shown_in_place_is_left_to_the_sync() {
    let (mut state, [_, middle, right]) = setup();
    right.set_display_state(DisplayState::Showing);
    grab_right_edge(&middle, &mut state);

    step_to(100, &mut state);

    assert!(is_placed(&middle, &state) && !is_queued(&middle, &state));
    assert!(is_queued(&right, &state));
  }

  #[test]
  fn the_end_of_a_resize_syncs_every_window() {
    let (mut state, [left, middle, right]) = setup();
    grab_right_edge(&middle, &mut state);
    step_to(100, &mut state);

    finish(
      GrabEnd::Released,
      Some((120, 0)),
      &mut state,
      &mock_config(),
    )
    .unwrap();

    for window in [&left, &middle, &right] {
      assert!(is_placed(window, &state));
    }
    let workspace = middle.workspace().unwrap();
    assert!(state
      .pending_sync
      .containers_to_redraw()
      .contains_key(&workspace.id()));
    assert!(state.grab.grab.is_none());
  }

  #[test]
  fn a_cancelled_resize_puts_its_sizes_back() {
    let (mut state, [left, middle, right]) = setup();
    grab_right_edge(&middle, &mut state);
    step_to(200, &mut state);

    finish(GrabEnd::Cancelled, None, &mut state, &mock_config()).unwrap();

    for window in [&left, &middle, &right] {
      assert!((window.tiling_size() - 1. / 3.).abs() < 1e-4);
      assert!(is_placed(window, &state));
    }
    assert!(state.grab.grab.is_none());
  }

  /// Ends a move the way a release of one crashed on (a `RefCell` borrowed
  /// twice). A mock's frame can't be queried, so the frame read that
  /// crashed is skipped, and the float the move's first events would do is
  /// done directly.
  #[test]
  fn a_cancelled_move_tiles_its_floated_window_back() {
    let (mut state, [left, middle, _]) = setup();
    let config = mock_config();
    let native = middle.native().clone();
    start(
      native.clone(),
      Gesture::Move,
      &GrabListener::new(),
      &mut state,
      &config,
    )
    .unwrap();
    assert!(state.grab.grab.is_some());

    middle.set_active_drag(Some(ActiveDrag {
      operation: Some(ActiveDragOperation::Move),
      is_from_floating: false,
      initial_position: target(&middle),
    }));
    float_for_drag(&middle.clone().into(), false, &mut state, &config)
      .unwrap();
    let floated = state.window_from_native(&native).unwrap();
    assert!(matches!(floated, WindowContainer::NonTilingWindow(_)));
    assert!(floated.active_drag().is_some());

    finish(GrabEnd::Cancelled, None, &mut state, &config).unwrap();

    let dropped = state.window_from_native(&native).unwrap();
    assert!(matches!(dropped, WindowContainer::TilingWindow(_)));
    assert!(dropped.active_drag().is_none());
    let workspace = left.workspace().unwrap();
    assert_eq!(dropped.workspace().map(|w| w.id()), Some(workspace.id()));
    assert_eq!(workspace.tiling_children().count(), 3);
    assert!(state.grab.grab.is_none());
  }

  #[test]
  fn fullscreen_and_hidden_windows_are_not_grabbed() {
    let (mut state, [left, ..]) = setup();
    assert!(is_grabbable(&left.clone().into(), &state));

    let fullscreen = NonTilingWindow::mock()
      .state(WindowState::Fullscreen(
        mock_config()
          .value
          .window_behavior
          .state_defaults
          .fullscreen,
      ))
      .call();
    crate::commands::container::attach_container(
      &fullscreen.clone().into(),
      &left.workspace().unwrap().into(),
      None,
    )
    .unwrap();
    assert!(!is_grabbable(&fullscreen.into(), &state));

    left.set_display_state(DisplayState::Hidden);
    assert!(!is_grabbable(&left.clone().into(), &state));

    // Nor started on, should one get through.
    grab_right_edge(&left, &mut state);
    assert!(state.grab.grab.is_none());
  }

  /// Invisible borders as Windows 10 and 11 give a resizable window.
  fn shadow_borders() -> RectDelta {
    RectDelta::new(
      LengthValue::from_px(7),
      LengthValue::from_px(0),
      LengthValue::from_px(7),
      LengthValue::from_px(7),
    )
  }

  #[test]
  fn overlays_go_by_the_frame_a_placed_window_shows() {
    let (_state, [window, ..]) = setup();
    let placed = Rect::from_xy(93, 200, 514, 407);

    assert_eq!(
      frame_at(&placed, &shadow_borders()),
      Rect::from_xy(100, 200, 500, 400)
    );

    // As steady state has it: the layout rect plus the configured border
    // delta, without the shadow borders the placed rect takes in.
    window.update_native_properties(|properties| {
      properties.shadow_borders = shadow_borders();
    });
    window.set_border_delta(RectDelta::new(
      LengthValue::from_px(-2),
      LengthValue::from_px(-2),
      LengthValue::from_px(-2),
      LengthValue::from_px(-2),
    ));
    assert_eq!(
      frame_at(&target(&window), &shadow_borders()),
      window
        .to_rect()
        .unwrap()
        .apply_delta(&window.border_delta(), None)
    );
  }

  #[test]
  fn a_moved_frame_keeps_its_size_and_shadows() {
    let frame = Rect::from_xy(100, 200, 500, 400);

    // The window's own top-left corner, shadow borders included.
    let position = Point { x: 293, y: 50 };

    assert_eq!(
      moved_frame(&frame, &shadow_borders(), &position),
      Rect::from_xy(300, 50, 500, 400)
    );
  }

  #[test]
  fn a_move_places_the_overlays_of_its_translations_only() {
    let (mut state, [left, middle, _]) = setup();
    let frame = Rect::from_xy(100, 200, 500, 400);
    let moved = Rect::from_xy(160, 180, 500, 400);
    let resized = Rect::from_xy(160, 180, 600, 450);

    let middle = middle.into();
    assert!(!places_move_overlays(&middle, &frame, &moved, &state));

    start(
      NativeWindow::from_handle(2),
      Gesture::Move,
      &GrabListener::new(),
      &mut state,
      &mock_config(),
    )
    .unwrap();

    assert!(places_move_overlays(&middle, &frame, &moved, &state));
    assert!(!places_move_overlays(&middle, &frame, &resized, &state));
    assert!(!places_move_overlays(&left.into(), &frame, &moved, &state));
  }

  #[test]
  fn a_resize_leaves_the_overlays_to_the_windows_events() {
    let (mut state, [_, middle, _]) = setup();
    let frame = Rect::from_xy(100, 200, 500, 400);
    let moved = Rect::from_xy(160, 180, 500, 400);
    grab_right_edge(&middle, &mut state);

    assert!(!places_move_overlays(
      &middle.into(),
      &frame,
      &moved,
      &state
    ));
  }
}
