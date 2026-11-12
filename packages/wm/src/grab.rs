//! Grab gestures: moving and resizing windows by dragging anywhere on them
//! with a modifier held (`general.grab_and_move`, see
//! `docs/grab-and-move.md`).
//!
//! A move runs as a drag by the title bar does: the platform moves the
//! window with the cursor, and its moves arrive as for any interactive
//! drag. A resize is laid out by the WM as the mouse moves, its
//! neighbours along with it, every step unanimated.

use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{DisplayState, Layout, WindowState};
use wm_platform::{
  Gesture, GrabEnd, GrabEvent, GrabListener, GrabSettings, NativeWindow,
  Rect, ResizeEdges, WindowId,
};

use crate::{
  commands::{
    container::set_focused_descendant,
    window::{MIN_FLOATING_HEIGHT, MIN_FLOATING_WIDTH},
  },
  events::{end_window_drag, handle_window_moved_or_resized},
  layout::{
    edge_resize::{drag_and_hold, dragged_rect, SizeSnapshot},
    is_hidden_in_layout,
    scrolling::{column_of, sync_view, ColumnHold},
  },
  models::{Container, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Shortest time between two steps of a resize, which each lay out its
/// windows anew.
const STEP_INTERVAL: Duration = Duration::from_millis(8);

/// Longest a window is waited on to land at the size a step gave it
/// before it is given a newer one.
///
/// Repositions are asynchronous, so a slow app would otherwise queue up a
/// size per step and play them all back, lagging ever further behind.
const LANDING_TIMEOUT: Duration = Duration::from_millis(100);

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

  /// Newest movement of the mouse, in all, that no step has applied yet.
  pending: Option<(i32, i32)>,

  /// Movement the last step applied.
  applied: (i32, i32),

  last_step: Option<Instant>,

  /// Windows given a new rect that they haven't reported landing at yet,
  /// and when.
  in_flight: HashMap<Uuid, Instant>,

  /// Whether the last step left a window at its old rect, being still in
  /// flight.
  has_held_back: bool,
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
    GrabEvent::Resized => {
      let delta = listener.take_resize_delta();

      if let Some(resize) = resize_mut(state) {
        resize.pending = Some(delta);

        let is_due = resize
          .last_step
          .is_none_or(|at| at.elapsed() >= STEP_INTERVAL);
        if is_due {
          return step(state);
        }
      }

      Ok(())
    }
    GrabEvent::Ended { window, end } => {
      let is_current = state
        .grab
        .grab
        .as_ref()
        .is_some_and(|grab| grab.native.id() == window.id());

      if !is_current {
        return Ok(());
      }

      if let Some(resize) = resize_mut(state) {
        resize.pending = Some(listener.take_resize_delta());
      }

      finish(end, state, config)
    }
  }
}

/// When the resize under way takes its next step, if it has one to take.
pub fn step_deadline(state: &WmState) -> Option<Instant> {
  let resize = state.grab.grab.as_ref()?.resize.as_ref()?;

  (resize.pending.is_some() || resize.has_held_back).then(|| {
    resize
      .last_step
      .map_or_else(Instant::now, |at| at + STEP_INTERVAL)
  })
}

/// Lays out the resize under way for the mouse's newest movement.
pub fn step(state: &mut WmState) -> anyhow::Result<()> {
  let Some(mut grab) = state.grab.grab.take() else {
    return Ok(());
  };

  let delta = grab.resize.as_mut().map(|resize| {
    resize.last_step = Some(Instant::now());
    resize.pending.take().unwrap_or(resize.applied)
  });

  let result = match (delta, state.window_from_native(&grab.native)) {
    (Some(delta), Some(window)) => {
      apply_resize(&mut grab, &window, delta, true, state)
    }
    _ => Ok(()),
  };

  state.grab.grab = Some(grab);
  result
}

/// Notes that `native` moved, which lands it for the resize under way.
pub fn note_moved(state: &mut WmState, native: &NativeWindow) {
  if resize_mut(state).is_none_or(|resize| resize.in_flight.is_empty()) {
    return;
  }

  let Some(id) =
    state.window_from_native(native).map(|window| window.id())
  else {
    return;
  };

  if let Some(resize) = resize_mut(state) {
    resize.in_flight.remove(&id);
  }
}

/// Notes that `native` took the foreground, which ends a grab of another
/// window at the next `sync_listener`.
pub fn note_focused(state: &mut WmState, native: &NativeWindow) {
  let is_other = state
    .grab
    .grab
    .as_ref()
    .is_some_and(|grab| grab.native.id() != native.id());

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
        )
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
      pending: None,
      applied: (0, 0),
      last_step: None,
      in_flight: HashMap::new(),
      has_held_back: false,
    })
  }
}

/// Resizes `window`, grabbed by `grab`, for the mouse's movement `delta`
/// in all, and queues the windows that moved for redraw: with `gate`, only
/// those not still in flight.
fn apply_resize(
  grab: &mut Grab,
  window: &WindowContainer,
  delta: (i32, i32),
  gate: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(resize) = &mut grab.resize else {
    return Ok(());
  };

  resize.applied = delta;

  let redraw: Vec<Container> = match window {
    WindowContainer::TilingWindow(tiling) => {
      let Some((workspace, snapshot)) = &resize.snapshot else {
        return Ok(());
      };

      // A window opened or closed since: its tiling sizes are no longer
      // the ones it resizes from, so it stays as it is.
      if !snapshot.matches(workspace) {
        return Ok(());
      }

      snapshot.restore();
      let hold = resize.hold.as_mut().map(|(hold, _)| hold);
      let redraw = drag_and_hold(tiling, resize.edges, delta, hold)?;

      // Scrolled now rather than by the sync, so that the windows' rects
      // below are where the sync puts them.
      if workspace.layout() == Layout::Scrolling {
        sync_view(workspace)?;
      }

      redraw
    }
    WindowContainer::NonTilingWindow(floating) => {
      if !matches!(floating.state(), WindowState::Floating(_)) {
        return Ok(());
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
      vec![window
        .parent()
        .filter(|parent| parent.as_stack().is_some())
        .unwrap_or_else(|| window.as_container())]
    }
  };

  queue_moved_windows(&redraw, resize, gate, state);
  state.pending_sync.suppress_animations();

  Ok(())
}

/// Queues the windows of `containers` whose rect changed for redraw:
/// with `gate`, only those not still in flight.
fn queue_moved_windows(
  containers: &[Container],
  resize: &mut Resize,
  gate: bool,
  state: &mut WmState,
) {
  let now = Instant::now();
  let mut seen = HashSet::new();
  resize.has_held_back = false;

  let windows = containers
    .iter()
    .flat_map(CommonGetters::self_and_descendants)
    .filter_map(|container| container.as_window_container().ok())
    .filter(|window| seen.insert(window.id()));

  for window in windows {
    let target = window.to_rect().and_then(|rect| {
      window
        .total_border_delta()
        .map(|delta| rect.apply_delta(&delta, None))
    });

    let is_unchanged = target.is_ok_and(|target| {
      state.window_target_positions.get(&window.id()) == Some(&target)
    });

    if is_unchanged {
      continue;
    }

    let is_in_flight = resize
      .in_flight
      .get(&window.id())
      .is_some_and(|at| now.duration_since(*at) < LANDING_TIMEOUT);

    if gate && is_in_flight {
      resize.has_held_back = true;
      continue;
    }

    resize.in_flight.insert(window.id(), now);
    state.pending_sync.queue_container_to_redraw(window);
  }
}

/// Ends the grab under way as `end` says.
fn finish(
  end: GrabEnd,
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
    return Ok(());
  };

  let start_hold = resize.hold.map(|(_, start)| start);
  let delta = if end == GrabEnd::Cancelled {
    (0, 0)
  } else {
    resize.pending.take().unwrap_or(resize.applied)
  };

  apply_resize(&mut grab, &window, delta, false, state)?;

  if end == GrabEnd::Cancelled {
    // Back where it started, its column too, even partly off screen.
    if let (Some(hold), Some((_, workspace))) =
      (start_hold, column_of(&window.as_container()))
    {
      hold.keep(&workspace);
    }
  } else {
    crate::particles::note_resize(state, &window.as_container());
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
  finish(GrabEnd::Released, state, config)
}

fn resize_mut(state: &mut WmState) -> Option<&mut Resize> {
  state.grab.grab.as_mut()?.resize.as_mut()
}

#[cfg(test)]
mod tests {
  use std::mem::ManuallyDrop;

  use wm_common::WindowState;
  use wm_platform::{
    Gesture, GrabEnd, GrabListener, NativeWindow, ResizeEdges,
  };

  use super::{
    finish, is_grabbable, note_moved, start, step, step_deadline,
  };
  use crate::{
    models::{Monitor, NonTilingWindow, TilingWindow, Workspace},
    test_utils::{mock_config, mock_inner_gaps},
    traits::{
      CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
    },
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
      let target = window.to_rect().unwrap();
      state.window_target_positions.insert(window.id(), target);
    }

    (state, windows)
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
    super::resize_mut(state).unwrap().pending = Some((dx, 0));
    step(state).unwrap();
  }

  fn is_queued(window: &TilingWindow, state: &WmState) -> bool {
    state
      .pending_sync
      .containers_to_redraw()
      .contains_key(&window.id())
  }

  #[test]
  fn a_step_resizes_the_neighbour_at_once_without_animations() {
    let (mut state, [left, middle, right]) = setup();
    grab_right_edge(&middle, &mut state);

    step_to(100, &mut state);

    assert!(middle.tiling_size() > right.tiling_size());
    assert!((left.tiling_size() - 1. / 3.).abs() < 1e-4);
    assert!(state.pending_sync.animations_suppressed());
    assert!(is_queued(&middle, &state) && is_queued(&right, &state));
    assert!(!is_queued(&left, &state));
  }

  #[test]
  fn a_window_still_in_flight_waits_for_its_landing() {
    let (mut state, [_, middle, right]) = setup();
    grab_right_edge(&middle, &mut state);
    step_to(100, &mut state);
    for window in [&middle, &right] {
      // Sent: as the sync records it.
      let target = window.to_rect().unwrap();
      state.window_target_positions.insert(window.id(), target);
    }
    state.pending_sync.clear();

    // Neither has landed: both are held back, and another step is due.
    step_to(150, &mut state);
    assert!(!is_queued(&middle, &state) && !is_queued(&right, &state));
    assert!(step_deadline(&state).is_some());

    note_moved(&mut state, &right.native());
    step(&mut state).unwrap();
    assert!(is_queued(&right, &state));
    assert!(!is_queued(&middle, &state));
  }

  #[test]
  fn a_cancelled_resize_puts_its_sizes_back() {
    let (mut state, [left, middle, right]) = setup();
    grab_right_edge(&middle, &mut state);
    step_to(200, &mut state);

    finish(GrabEnd::Cancelled, &mut state, &mock_config()).unwrap();

    for window in [&left, &middle, &right] {
      assert!((window.tiling_size() - 1. / 3.).abs() < 1e-4);
    }
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

    left.set_display_state(wm_common::DisplayState::Hidden);
    assert!(!is_grabbable(&left.clone().into(), &state));

    // Nor started on, should one get through.
    grab_right_edge(&left, &mut state);
    assert!(state.grab.grab.is_none());
  }
}
