use std::{
  collections::HashSet,
  sync::{Arc, Mutex, PoisonError},
  time::{Duration, Instant},
};

use tokio::sync::mpsc;

use crate::{platform_impl, NativeWindow, Point, Rect, WindowId};

/// How long a [`MotionGuard`] holds without being renewed. Renewed every
/// animation frame, so it only lapses if those stop, and then clicks
/// reach their windows again rather than being kept for good.
const MOTION_GUARD_TTL: Duration = Duration::from_millis(250);

/// A click [`ClickGuard`] kept from where it landed.
#[derive(Clone, Debug)]
pub enum GuardedClick {
  /// A press on a guarded window.
  Window(NativeWindow),

  /// A press at this point on something drawn moving (see
  /// [`ClickGuard::guard_motion`]).
  Motion(Point),
}

/// Where windows are drawn moving by stand-ins, whose real windows are
/// hidden or elsewhere meanwhile.
#[derive(Debug, Default)]
struct MotionGuard {
  /// On-screen areas of the stand-ins.
  areas: Vec<Rect>,

  /// Areas of live windows over them, whose clicks are theirs.
  exempt: Vec<Rect>,

  /// When the guard was last renewed.
  renewed: Option<Instant>,
}

impl MotionGuard {
  fn is_guarded(&self, point: &Point, now: Instant) -> bool {
    self.renewed.is_some_and(|renewed| {
      now.duration_since(renewed) < MOTION_GUARD_TTL
    }) && self.areas.iter().any(|area| area.contains_point(point))
      && !self.exempt.iter().any(|area| area.contains_point(point))
  }
}

/// Keeps clicks from chosen windows and reports them instead, so that a
/// click on one of them can e.g. only focus it rather than also press
/// whatever is under the cursor.
///
/// A press is kept from a window, along with its release, only if the
/// window is the one under the cursor, i.e. not covered there. Presses on
/// windows drawn moving are kept too (see [`ClickGuard::guard_motion`]).
pub struct ClickGuard {
  /// Windows whose clicks are kept from them.
  guarded: Arc<Mutex<HashSet<WindowId>>>,

  /// Areas drawn moving, whose clicks are kept from what is below.
  motion: Arc<Mutex<MotionGuard>>,

  /// Sender of the clicks kept.
  click_tx: mpsc::UnboundedSender<GuardedClick>,

  /// Hook for the clicks, only while anything is guarded, so the mouse
  /// isn't hooked for nothing.
  hook: Option<platform_impl::ClickHook>,
}

impl ClickGuard {
  /// Creates an instance of `ClickGuard` that guards nothing yet.
  ///
  /// Each press kept is sent through `click_tx`.
  #[must_use]
  pub fn new(click_tx: mpsc::UnboundedSender<GuardedClick>) -> Self {
    Self {
      guarded: Arc::default(),
      motion: Arc::default(),
      click_tx,
      hook: None,
    }
  }

  /// Guards `windows` instead of the ones guarded so far.
  pub fn guard(&mut self, windows: impl IntoIterator<Item = WindowId>) {
    let windows = windows.into_iter().collect::<HashSet<_>>();

    {
      let mut guarded =
        self.guarded.lock().unwrap_or_else(PoisonError::into_inner);

      if *guarded == windows {
        return;
      }

      *guarded = windows;
    }

    if let Err(err) = self.sync_hook() {
      tracing::warn!("Failed to hook mouse clicks: {err}");
    }
  }

  /// Keeps presses within `areas`, but not `exempt`, from whatever is
  /// below, for the stand-ins there that draw windows moving. Stand-ins
  /// are click-through, and the real windows hidden, so the press would
  /// otherwise land on the desktop or a window behind.
  ///
  /// Lapses unless renewed within [`MOTION_GUARD_TTL`], so it has to be
  /// called every frame the stand-ins move, and with no `areas` once
  /// they stop.
  pub fn guard_motion(&mut self, areas: Vec<Rect>, exempt: Vec<Rect>) {
    let had_areas = {
      let mut motion =
        self.motion.lock().unwrap_or_else(PoisonError::into_inner);
      let had_areas = !motion.areas.is_empty();

      motion.renewed = (!areas.is_empty()).then(Instant::now);
      motion.areas = areas;
      motion.exempt = exempt;
      had_areas
    };

    if had_areas != self.is_guarding_motion() {
      if let Err(err) = self.sync_hook() {
        tracing::warn!("Failed to hook mouse clicks: {err}");
      }
    }
  }

  fn is_guarding_motion(&self) -> bool {
    !self
      .motion
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .areas
      .is_empty()
  }

  /// Hooks the mouse while anything is guarded, and unhooks it
  /// otherwise.
  fn sync_hook(&mut self) -> crate::Result<()> {
    let is_guarding = self.is_guarding_motion()
      || !self
        .guarded
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_empty();

    match (&mut self.hook, is_guarding) {
      (None, true) => {
        let guarded = self.guarded.clone();
        let motion = self.motion.clone();
        let click_tx = self.click_tx.clone();

        self.hook =
          Some(platform_impl::ClickHook::new(move |root, point| {
            // What is drawn moving there first: the real window under
            // the cursor may be one cloaked at where it is headed.
            let is_motion = motion
              .lock()
              .unwrap_or_else(PoisonError::into_inner)
              .is_guarded(&point, Instant::now());

            let guarded_window =
              root.map(|hwnd| WindowId(hwnd.0)).filter(|id| {
                guarded
                  .lock()
                  .unwrap_or_else(PoisonError::into_inner)
                  .contains(id)
              });

            let click = match (is_motion, guarded_window) {
              (true, _) => GuardedClick::Motion(point),
              (false, Some(id)) => GuardedClick::Window(
                platform_impl::NativeWindow::new(id.0).into(),
              ),
              (false, None) => return false,
            };

            let _ = click_tx.send(click);
            true
          })?);
      }
      (Some(hook), false) => {
        hook.terminate()?;
        self.hook = None;
      }
      _ => {}
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn guard() -> MotionGuard {
    MotionGuard {
      areas: vec![Rect::from_xy(0, 0, 100, 100)],
      exempt: vec![Rect::from_xy(50, 50, 20, 20)],
      renewed: Some(Instant::now()),
    }
  }

  #[test]
  fn guards_areas_but_not_exempt_ones() {
    let now = Instant::now();
    let guard = guard();

    assert!(guard.is_guarded(&Point { x: 10, y: 10 }, now));
    assert!(!guard.is_guarded(&Point { x: 60, y: 60 }, now));
    assert!(!guard.is_guarded(&Point { x: 200, y: 10 }, now));
  }

  #[test]
  fn lapses_unless_renewed() {
    let guard = guard();
    let later = Instant::now() + MOTION_GUARD_TTL;

    assert!(!guard.is_guarded(&Point { x: 10, y: 10 }, later));
  }
}
