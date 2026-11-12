use std::{
  collections::HashSet,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, PoisonError,
  },
  time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{platform_impl, NativeWindow, Point, Rect, WindowId};

/// Share of a window's width (and height), in percent, that each side zone
/// takes when picking the edges a resize grabs; the middle gets the rest.
/// `AltSnap`'s default.
const SIDE_ZONE_PERCENT: i32 = 38;

/// How long after the cursor is warped away from the desktop's edge moves
/// measured from where it was are told apart from real ones.
const WARP_SETTLE: Duration = Duration::from_millis(50);

/// Key that has to be held for a press to grab a window.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum GrabModifier {
  #[default]
  Alt,
  Win,
}

/// Edges a grab-and-resize drags, picked from where on the window it was
/// pressed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ResizeEdges {
  pub left: bool,
  pub right: bool,
  pub top: bool,
  pub bottom: bool,
}

impl ResizeEdges {
  /// Edges nearest to `point` on `rect`, as `AltSnap` picks them: the
  /// window is cut into a 3x3 grid whose side zones are
  /// [`SIDE_ZONE_PERCENT`] wide, a corner zone grabs its corner and a
  /// side zone its side. The middle zone grabs the side nearest to the
  /// point, relative to the window's size.
  #[must_use]
  pub fn at(rect: &Rect, point: &Point) -> Self {
    let width = rect.width().max(1);
    let height = rect.height().max(1);
    let x = (point.x - rect.left).clamp(0, width);
    let y = (point.y - rect.top).clamp(0, height);

    // -1 for the near side, 1 for the far one, 0 for the middle.
    let zone = |offset: i32, length: i32| match offset * 100 {
      share if share < length * SIDE_ZONE_PERCENT => -1,
      share if share >= length * (100 - SIDE_ZONE_PERCENT) => 1,
      _ => 0,
    };

    let (zone_x, zone_y) = (zone(x, width), zone(y, height));

    if zone_x == 0 && zone_y == 0 {
      // Distances to each side as a share of the window's size, compared
      // without dividing: `a / width < b / height` as `a * height < b *
      // width`.
      let horizontal = x.min(width - x) * height;
      let vertical = y.min(height - y) * width;

      return if horizontal <= vertical {
        Self {
          left: x * 2 <= width,
          right: x * 2 > width,
          ..Self::default()
        }
      } else {
        Self {
          top: y * 2 <= height,
          bottom: y * 2 > height,
          ..Self::default()
        }
      };
    }

    Self {
      left: zone_x < 0,
      right: zone_x > 0,
      top: zone_y < 0,
      bottom: zone_y > 0,
    }
  }
}

/// How far from the desktop's edge a resize keeps the cursor it holds
/// still.
const WARP_MARGIN: i32 = 48;

/// Where to hold the cursor still for a resize of `edges` pressed at
/// `point`, if not there: [`WARP_MARGIN`] in from the desktop's outer
/// edge, for each edge whose way out it is within that of.
///
/// The cursor can't go past the desktop's edge, so a move from there
/// towards it reports less, or nothing, of how far the mouse went. Edges
/// between monitors are no such limit. `is_on_desktop` is whether a point
/// is on any monitor.
pub(crate) fn warp_target(
  point: &Point,
  edges: ResizeEdges,
  is_on_desktop: impl Fn(&Point) -> bool,
) -> Option<Point> {
  let mut target = point.clone();

  let mut keep_in = |is_dragged: bool, dx: i32, dy: i32| {
    let outward = Point {
      x: point.x + dx * WARP_MARGIN,
      y: point.y + dy * WARP_MARGIN,
    };

    if is_dragged && !is_on_desktop(&outward) {
      target.x -= dx * WARP_MARGIN;
      target.y -= dy * WARP_MARGIN;
    }
  };

  keep_in(edges.left, -1, 0);
  keep_in(edges.right, 1, 0);
  keep_in(edges.top, 0, -1);
  keep_in(edges.bottom, 0, 1);

  (target != *point && is_on_desktop(&target)).then_some(target)
}

/// What a grab does with the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesture {
  /// Moves it with the cursor, as a drag by its title bar does.
  Move,

  /// Drags these of its edges by the mouse's movement, while the cursor
  /// stays where it was pressed (see [`GrabEvent::Resized`]).
  Resize(ResizeEdges),
}

/// How a grab ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrabEnd {
  /// Its button was released.
  Released,

  /// Escape was pressed; a move puts its window back where it started.
  Cancelled,

  /// Its button was released where the hook couldn't see it: the input
  /// desktop switched (e.g. to a UAC prompt's, or the lock screen), or
  /// the button was pressed again.
  Lost,
}

#[derive(Clone, Debug)]
pub enum GrabEvent {
  /// A press on `window` was taken from it and began `gesture`.
  Started {
    window: NativeWindow,
    gesture: Gesture,
  },

  /// The mouse moved during a resize: read how far in all with
  /// [`GrabListener::take_resize_delta`]. Not sent again until it is
  /// read, so a slow reader only ever sees the newest total.
  Resized,

  /// The grab of `window` ended. A move's last reposition has landed by
  /// then. Also sent for a grab ended early by
  /// [`GrabListener::end_gesture`], once its button is released.
  Ended { window: NativeWindow, end: GrabEnd },
}

/// What [`GrabListener`] grabs with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrabSettings {
  pub modifier: GrabModifier,

  /// Whether the right button resizes; otherwise its presses pass.
  pub resizes: bool,
}

/// Takes presses of the left (move) or right (resize) mouse button made
/// with a modifier held on chosen windows, along with their releases, and
/// reports the gestures they make instead.
///
/// The mouse is only hooked while configured. Presses that start nothing
/// reach their windows untouched, and so does everything while disarmed.
pub struct GrabListener {
  shared: Arc<GrabShared>,
  event_rx: mpsc::UnboundedReceiver<GrabEvent>,
  hook: Option<platform_impl::GrabHook>,

  /// Whether the hook is only kept for the release of a press it took,
  /// since unhooking first would hand the app an orphan release.
  is_draining: bool,
}

impl GrabListener {
  /// Creates an instance of `GrabListener` that hooks nothing yet.
  #[must_use]
  pub fn new() -> Self {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    Self {
      shared: Arc::new(GrabShared {
        armed: AtomicBool::new(false),
        settings: Mutex::new(None),
        windows: Mutex::default(),
        tracker: Mutex::default(),
        is_held: AtomicBool::new(false),
        event_tx,
      }),
      event_rx,
      hook: None,
      is_draining: false,
    }
  }

  /// Grabs with `settings`, or stops grabbing with `None`, hooking or
  /// unhooking the mouse to match.
  pub fn configure(&mut self, settings: Option<GrabSettings>) {
    let is_on = settings.is_some();
    *self
      .shared
      .settings
      .lock()
      .unwrap_or_else(PoisonError::into_inner) = settings;

    if is_on {
      self.is_draining = false;

      if self.hook.is_none() {
        match platform_impl::GrabHook::new(self.shared.clone()) {
          Ok(hook) => self.hook = Some(hook),
          Err(err) => tracing::warn!("Failed to hook mouse grabs: {err}"),
        }
      }
    } else {
      self.end_gesture();
      self.is_draining = self.hook.is_some();
      self.unhook_if_drained();
    }
  }

  /// Lets presses start grabs, or not (e.g. while the WM is paused).
  pub fn arm(&self, armed: bool) {
    self.shared.armed.store(armed, Ordering::Relaxed);
  }

  /// Grabs presses on `windows` instead of the ones so far.
  pub fn set_windows(&self, windows: impl IntoIterator<Item = WindowId>) {
    let windows = windows.into_iter().collect::<HashSet<_>>();
    let mut current = self
      .shared
      .windows
      .lock()
      .unwrap_or_else(PoisonError::into_inner);

    if *current != windows {
      *current = windows;
    }
  }

  /// Whether the mouse is hooked for grabs.
  #[must_use]
  pub fn is_hooked(&self) -> bool {
    self.hook.is_some() && !self.is_draining
  }

  /// How far the mouse has moved in all since the resize began, or zero
  /// outside a resize.
  #[must_use]
  pub fn take_resize_delta(&self) -> (i32, i32) {
    self.shared.tracker().take_delta()
  }

  /// Ends the gesture under way, if any, without reporting it, e.g. as its
  /// window goes away. The press's release is still taken from the app.
  pub fn end_gesture(&self) {
    self.shared.tracker().end_live();

    if let Some(hook) = &self.hook {
      hook.end_gesture();
    }
  }

  /// Returns the next grab event. Never yields while nothing is hooked.
  pub async fn next_event(&mut self) -> Option<GrabEvent> {
    let event = self.event_rx.recv().await;

    if let Some(GrabEvent::Ended { .. }) = event {
      self.unhook_if_drained();
    }

    event
  }

  fn unhook_if_drained(&mut self) {
    if !self.is_draining || self.shared.is_held.load(Ordering::Relaxed) {
      return;
    }

    if let Some(mut hook) = self.hook.take() {
      hook.terminate();
    }

    self.is_draining = false;
  }
}

impl Default for GrabListener {
  fn default() -> Self {
    Self::new()
  }
}

/// State the hook's threads share with [`GrabListener`].
pub(crate) struct GrabShared {
  armed: AtomicBool,
  settings: Mutex<Option<GrabSettings>>,
  windows: Mutex<HashSet<WindowId>>,
  tracker: Mutex<Tracker>,

  /// Whether a press is held, so mouse moves cost one atomic load in the
  /// hook while none is.
  pub(crate) is_held: AtomicBool,

  pub(crate) event_tx: mpsc::UnboundedSender<GrabEvent>,
}

impl GrabShared {
  pub(crate) fn tracker(&self) -> std::sync::MutexGuard<'_, Tracker> {
    self.tracker.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// Handles a press of `button` at `point` on the window `window_at`
  /// finds there (with its rect), returning that window and the gesture
  /// the press starts, if any, which takes the press.
  pub(crate) fn press(
    &self,
    button: GrabButton,
    point: &Point,
    window_at: impl FnOnce() -> Option<(WindowId, Rect)>,
    is_modifier_down: impl FnOnce(GrabModifier) -> bool,
  ) -> Option<(WindowId, Gesture)> {
    if !self.armed.load(Ordering::Relaxed) {
      return None;
    }

    let settings = self
      .settings
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .clone()?;

    if !is_modifier_down(settings.modifier) {
      return None;
    }

    let (id, rect) = window_at().filter(|(id, _)| {
      self
        .windows
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .contains(id)
    })?;

    let gesture =
      self.tracker().press(button, point, id, &rect, &settings)?;
    self.is_held.store(true, Ordering::Relaxed);

    Some((id, gesture))
  }
}

/// Mouse buttons a grab is made with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GrabButton {
  Left,
  Right,
}

/// A press taken from its window, until its release.
#[derive(Clone, Debug, PartialEq)]
struct Held {
  button: GrabButton,
  window: WindowId,

  /// The gesture, until it ends, which can be before the release.
  live: Option<Live>,
}

/// A gesture under way.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Live {
  Move {
    /// From the window's top-left corner to the cursor.
    offset: Point,
    /// Where the window's top-left corner started.
    start: Point,
  },
  Resize {
    total: (i32, i32),
    /// Whether `total` changed since last read.
    is_unread: bool,
    warp: Option<Warp>,
  },
}

/// The cursor moved away from the desktop's edge at the start of a resize.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Warp {
  from: Point,
  to: Point,
  at: Instant,
}

/// What a mouse move during a held press does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MoveOutcome {
  /// The cursor moves as usual.
  Pass,
  /// The cursor moves, and the window is moved for its top-left corner to
  /// be here.
  MoveWindow(WindowId, Point),
  /// The cursor stays: a resize measures the move instead. `notify` is
  /// whether its reader has to be told.
  Measure { notify: bool },
}

/// What a release of a held press ends.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ReleaseOutcome {
  /// Not the held press's button.
  Pass,
  /// The held press's release, taken along with it, ending `live` on
  /// `window` if it was still under way.
  Swallow {
    window: WindowId,
    live: Option<Live>,
  },
}

/// Pairs the presses the hook takes with their releases and tracks the
/// gesture between them, apart from any Win32 call.
#[derive(Debug, Default)]
pub(crate) struct Tracker {
  held: Option<Held>,
}

impl Tracker {
  fn press(
    &mut self,
    button: GrabButton,
    point: &Point,
    window: WindowId,
    rect: &Rect,
    settings: &GrabSettings,
  ) -> Option<Gesture> {
    if self.held.is_some() {
      return None;
    }

    let (gesture, live) = match button {
      GrabButton::Left => (
        Gesture::Move,
        Live::Move {
          offset: Point {
            x: point.x - rect.left,
            y: point.y - rect.top,
          },
          start: Point {
            x: rect.left,
            y: rect.top,
          },
        },
      ),
      GrabButton::Right if settings.resizes => (
        Gesture::Resize(ResizeEdges::at(rect, point)),
        Live::Resize {
          total: (0, 0),
          is_unread: false,
          warp: None,
        },
      ),
      GrabButton::Right => return None,
    };

    self.held = Some(Held {
      button,
      window,
      live: Some(live),
    });

    Some(gesture)
  }

  /// Handles a move of the cursor to `point` from `cursor`, where it is.
  pub(crate) fn moved(
    &mut self,
    point: &Point,
    cursor: &Point,
    now: Instant,
  ) -> MoveOutcome {
    let Some(held) = &mut self.held else {
      return MoveOutcome::Pass;
    };

    match &mut held.live {
      None => MoveOutcome::Pass,
      Some(Live::Move { offset, .. }) => MoveOutcome::MoveWindow(
        held.window,
        Point {
          x: point.x - offset.x,
          y: point.y - offset.y,
        },
      ),
      Some(Live::Resize {
        total,
        is_unread,
        warp,
      }) => {
        // A move measured from where the cursor was before it was warped
        // arrives after the warp: nearer to where it was than to where
        // it is. Dropped, or it would count the warp's distance.
        let is_stale = warp.as_ref().is_some_and(|warp| {
          now.duration_since(warp.at) < WARP_SETTLE
            && point.distance_between(&warp.from)
              < point.distance_between(&warp.to)
        });

        if is_stale {
          return MoveOutcome::Measure { notify: false };
        }

        let (dx, dy) = (point.x - cursor.x, point.y - cursor.y);
        if dx == 0 && dy == 0 {
          return MoveOutcome::Measure { notify: false };
        }

        *total = (total.0 + dx, total.1 + dy);
        let notify = !*is_unread;
        *is_unread = true;
        MoveOutcome::Measure { notify }
      }
    }
  }

  /// Handles a release of `button`.
  pub(crate) fn released(&mut self, button: GrabButton) -> ReleaseOutcome {
    match self.held.take() {
      Some(held) if held.button == button => ReleaseOutcome::Swallow {
        window: held.window,
        live: held.live,
      },
      held => {
        self.held = held;
        ReleaseOutcome::Pass
      }
    }
  }

  /// Ends the gesture under way, keeping its press held. Returns its
  /// window and the gesture.
  pub(crate) fn end_live(&mut self) -> Option<(WindowId, Live)> {
    let held = self.held.as_mut()?;
    held.live.take().map(|live| (held.window, live))
  }

  /// Forgets the held press, whose release went unseen.
  pub(crate) fn drop_held(&mut self) -> Option<(WindowId, Option<Live>)> {
    self.held.take().map(|held| (held.window, held.live))
  }

  #[cfg(test)]
  fn is_live(&self) -> bool {
    self.held.as_ref().is_some_and(|held| held.live.is_some())
  }

  /// Whether a press of `button` is held.
  pub(crate) fn holds(&self, button: GrabButton) -> bool {
    self.held.as_ref().is_some_and(|held| held.button == button)
  }

  pub(crate) fn is_resizing(&self) -> bool {
    matches!(
      self.held.as_ref().and_then(|held| held.live.as_ref()),
      Some(Live::Resize { .. })
    )
  }

  /// Notes that the cursor was warped `from` a point `to` another as the
  /// resize began.
  pub(crate) fn warped(&mut self, from: Point, to: Point, now: Instant) {
    if let Some(Held {
      live: Some(Live::Resize { warp, .. }),
      ..
    }) = &mut self.held
    {
      *warp = Some(Warp { from, to, at: now });
    }
  }

  fn take_delta(&mut self) -> (i32, i32) {
    match self.held.as_mut().and_then(|held| held.live.as_mut()) {
      Some(Live::Resize {
        total, is_unread, ..
      }) => {
        *is_unread = false;
        *total
      }
      _ => (0, 0),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn window_rect() -> Rect {
    Rect::from_xy(100, 100, 1000, 500)
  }

  fn edges_at(x: i32, y: i32) -> ResizeEdges {
    ResizeEdges::at(&window_rect(), &Point { x, y })
  }

  fn settings() -> GrabSettings {
    GrabSettings {
      modifier: GrabModifier::Alt,
      resizes: true,
    }
  }

  fn press(tracker: &mut Tracker, button: GrabButton) -> Option<Gesture> {
    tracker.press(
      button,
      &Point { x: 150, y: 150 },
      WindowId(1),
      &window_rect(),
      &settings(),
    )
  }

  #[test]
  fn corners_and_sides_grab_their_edges() {
    let top_left = edges_at(110, 110);
    assert!(top_left.left && top_left.top);
    assert!(!top_left.right && !top_left.bottom);

    let bottom_right = edges_at(1090, 590);
    assert!(bottom_right.right && bottom_right.bottom);

    // The right side zone, halfway down.
    assert_eq!(
      edges_at(1000, 350),
      ResizeEdges {
        right: true,
        ..ResizeEdges::default()
      }
    );

    // The top side zone, halfway across.
    assert_eq!(
      edges_at(600, 120),
      ResizeEdges {
        top: true,
        ..ResizeEdges::default()
      }
    );
  }

  #[test]
  fn the_middle_grabs_the_nearest_side_for_the_window_size() {
    // 45% across and 50% down: nearer to the left, relative to size.
    assert_eq!(
      edges_at(550, 350),
      ResizeEdges {
        left: true,
        ..ResizeEdges::default()
      }
    );

    // 50% across and 41% down: nearer to the top.
    assert_eq!(
      edges_at(600, 305),
      ResizeEdges {
        top: true,
        ..ResizeEdges::default()
      }
    );
  }

  #[test]
  fn the_cursor_is_held_in_from_the_desktop_edge_it_resizes_towards() {
    // Two monitors side by side, 1680 wide each.
    let desktop = Rect::from_xy(0, 0, 3360, 1050);
    let is_on_desktop = |point: &Point| desktop.contains_point(point);
    let right = ResizeEdges {
      right: true,
      ..ResizeEdges::default()
    };
    let top_left = ResizeEdges {
      left: true,
      top: true,
      ..ResizeEdges::default()
    };

    // At the outer right edge, held in from it.
    assert_eq!(
      warp_target(&Point { x: 3359, y: 500 }, right, is_on_desktop),
      Some(Point { x: 3311, y: 500 })
    );

    // At the edge between the monitors, left where it is.
    assert_eq!(
      warp_target(&Point { x: 1679, y: 500 }, right, is_on_desktop),
      None
    );

    // Only the edges being dragged count.
    assert_eq!(
      warp_target(&Point { x: 3359, y: 5 }, top_left, is_on_desktop),
      Some(Point { x: 3359, y: 53 })
    );
  }

  #[test]
  fn a_press_outside_the_rect_grabs_the_nearest_corner() {
    let edges = edges_at(0, 2000);
    assert!(edges.left && edges.bottom);
  }

  #[test]
  fn a_press_and_its_release_are_taken_together() {
    let mut tracker = Tracker::default();

    assert_eq!(press(&mut tracker, GrabButton::Left), Some(Gesture::Move));

    // Another press while held starts nothing, and its release isn't the
    // held one's.
    assert_eq!(press(&mut tracker, GrabButton::Right), None);
    assert_eq!(tracker.released(GrabButton::Right), ReleaseOutcome::Pass);

    assert!(matches!(
      tracker.released(GrabButton::Left),
      ReleaseOutcome::Swallow {
        window: WindowId(1),
        live: Some(Live::Move { .. }),
      }
    ));
    assert_eq!(tracker.released(GrabButton::Left), ReleaseOutcome::Pass);
  }

  #[test]
  fn a_gesture_ended_early_still_takes_its_release() {
    let mut tracker = Tracker::default();
    press(&mut tracker, GrabButton::Right);

    assert!(tracker.end_live().is_some());
    assert!(!tracker.is_live());

    let cursor = Point { x: 150, y: 150 };
    assert_eq!(
      tracker.moved(&Point { x: 160, y: 150 }, &cursor, Instant::now()),
      MoveOutcome::Pass
    );
    assert_eq!(
      tracker.released(GrabButton::Right),
      ReleaseOutcome::Swallow {
        window: WindowId(1),
        live: None,
      }
    );
  }

  #[test]
  fn right_presses_pass_unless_resizing_is_on() {
    let mut tracker = Tracker::default();
    let settings = GrabSettings {
      resizes: false,
      ..settings()
    };

    let gesture = tracker.press(
      GrabButton::Right,
      &Point { x: 150, y: 150 },
      WindowId(1),
      &window_rect(),
      &settings,
    );

    assert_eq!(gesture, None);
    assert_eq!(tracker.released(GrabButton::Right), ReleaseOutcome::Pass);
  }

  #[test]
  fn a_move_keeps_the_cursor_where_it_grabbed_the_window() {
    let mut tracker = Tracker::default();
    press(&mut tracker, GrabButton::Left);

    let outcome = tracker.moved(
      &Point { x: 400, y: 260 },
      &Point { x: 150, y: 150 },
      Instant::now(),
    );

    assert_eq!(
      outcome,
      MoveOutcome::MoveWindow(WindowId(1), Point { x: 350, y: 210 })
    );
  }

  #[test]
  fn a_resize_adds_up_moves_and_notifies_once_until_read() {
    let mut tracker = Tracker::default();
    press(&mut tracker, GrabButton::Right);
    let cursor = Point { x: 150, y: 150 };
    let now = Instant::now();

    assert_eq!(
      tracker.moved(&Point { x: 160, y: 145 }, &cursor, now),
      MoveOutcome::Measure { notify: true }
    );
    assert_eq!(
      tracker.moved(&Point { x: 155, y: 150 }, &cursor, now),
      MoveOutcome::Measure { notify: false }
    );
    assert_eq!(tracker.take_delta(), (15, -5));

    assert_eq!(
      tracker.moved(&Point { x: 140, y: 150 }, &cursor, now),
      MoveOutcome::Measure { notify: true }
    );
    assert_eq!(tracker.take_delta(), (5, -5));
  }

  #[test]
  fn moves_from_before_a_warp_are_dropped() {
    let mut tracker = Tracker::default();
    press(&mut tracker, GrabButton::Right);
    let (from, to) =
      (Point { x: 1679, y: 500 }, Point { x: 1631, y: 500 });
    let now = Instant::now();
    tracker.warped(from.clone(), to.clone(), now);

    // Measured from where the cursor was, though it is already warped.
    tracker.moved(&Point { x: 1679, y: 502 }, &to, now);
    assert_eq!(tracker.take_delta(), (0, 0));

    tracker.moved(&Point { x: 1640, y: 500 }, &to, now);
    assert_eq!(tracker.take_delta(), (9, 0));

    // Once settled, every move counts.
    let later = now + WARP_SETTLE;
    tracker.moved(&Point { x: 1679, y: 500 }, &to, later);
    assert_eq!(tracker.take_delta(), (57, 0));
  }
}
