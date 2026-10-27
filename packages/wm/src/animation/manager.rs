#[cfg(target_os = "windows")]
use std::collections::HashMap;
use std::{
  cell::Cell,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
  },
  time::{Duration, Instant},
};

use tokio::sync::mpsc;
#[cfg(target_os = "windows")]
use uuid::Uuid;
use wm_platform::perf::{self, Stage};

#[cfg(target_os = "windows")]
use crate::commands::general::resync_settling_overlays;
use crate::{
  commands::general::platform_sync, user_config::UserConfig,
  wm_state::WmState,
};

/// How long a window's overlays keep being re-asserted behind it after its
/// own z-order last changed. See
/// `AnimationManager::settle_overlay_z_order`.
///
/// Covers the asynchronous move itself, `set_z_order`'s 10ms retry, and
/// apps that briefly re-assert their own z-order in response; measured
/// cases land within a few frames.
#[cfg(target_os = "windows")]
const OVERLAY_Z_SETTLE: Duration = Duration::from_millis(200);

/// Upper bound on how long a window's overlays keep settling, however
/// often the settle is extended -- so a window that never reaches its
/// intended band (an app that refuses `HWND_TOPMOST`) cannot keep the
/// animation timer ticking forever.
#[cfg(target_os = "windows")]
const OVERLAY_Z_SETTLE_MAX: Duration = Duration::from_secs(2);

/// Deadlines for one window in `AnimationManager::overlay_z_settle`.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug)]
struct SettleWindow {
  /// When re-checking stops, unless extended.
  until: Instant,
  /// When re-checking stops regardless; see [`OVERLAY_Z_SETTLE_MAX`].
  cap: Instant,
}

/// How long animations may go without a frame before the watchdog runs
/// one itself and replaces the timer thread.
const FRAME_STALL_TIMEOUT: Duration = Duration::from_millis(250);

/// Replaced timer threads that may be left blocked at once. See
/// [`AnimationManager::replace_stalled_timer`].
const MAX_STALLED_TIMERS: usize = 3;

/// The animation timer thread, with the flag that retires it.
struct TimerThread {
  handle: std::thread::JoinHandle<()>,

  /// Makes the thread exit once its current wait returns. Set when the
  /// watchdog replaces a thread stuck in a composition wait.
  retired: Arc<AtomicBool>,
}

/// Counts a retired timer thread out of `stalled_timers` once it exits.
struct StalledTimerGuard {
  retired: Arc<AtomicBool>,
  stalled_timers: Arc<AtomicUsize>,
}

impl Drop for StalledTimerGuard {
  fn drop(&mut self) {
    if self.retired.load(Ordering::Relaxed) {
      self.stalled_timers.fetch_sub(1, Ordering::Relaxed);
    }
  }
}

/// Stores `next` in `result` if it is the first error; later errors are
/// only logged.
fn keep_first_error(
  result: &mut anyhow::Result<()>,
  next: anyhow::Result<()>,
) {
  if let Err(err) = next {
    if result.is_ok() {
      *result = Err(err);
    } else {
      tracing::error!("{err:?}");
    }
  }
}

/// Drives what animates over time: the scenes playing animations, and the
/// overlays settling behind windows whose z-order changed.
pub struct AnimationManager {
  /// Sender for animation tick events.
  animation_tick_tx: mpsc::UnboundedSender<()>,
  /// Whether the animation timer thread is currently ticking (vs.
  /// parked).
  ///
  /// Acts as the gate for the persistent timer thread: set `true` to
  /// start a ticking phase, `false` to send it back to parking.
  animation_timer_running: Arc<AtomicBool>,
  /// Handle to the persistent animation timer thread.
  ///
  /// Spawned lazily on the first animation and kept for the process
  /// lifetime, parking between animations rather than being re-spawned
  /// each time.
  timer_thread: Mutex<Option<TimerThread>>,
  /// Signals the persistent timer thread to exit. Set on drop.
  timer_shutdown: Arc<AtomicBool>,
  /// When the last frame ran, or ticking last started. Read by the
  /// watchdog; see [`watchdog_deadline`].
  ///
  /// [`watchdog_deadline`]: AnimationManager::watchdog_deadline
  last_frame_at: Cell<Instant>,
  /// Timer threads replaced by the watchdog that are still blocked in
  /// their wait. See [`replace_stalled_timer`].
  ///
  /// [`replace_stalled_timer`]: AnimationManager::replace_stalled_timer
  stalled_timers: Arc<AtomicUsize>,
  /// Windows whose own z-order was changed recently, mapped to when their
  /// overlays stop being re-checked. See [`settle_overlay_z_order`].
  ///
  /// [`settle_overlay_z_order`]: AnimationManager::settle_overlay_z_order
  #[cfg(target_os = "windows")]
  overlay_z_settle: HashMap<Uuid, SettleWindow>,
  /// Peeks drawing scrolling layouts' columns.
  #[cfg(target_os = "windows")]
  pub scroll_view: super::scroll_view::ScrollView,
  /// Scenes playing animations over their monitors.
  #[cfg(target_os = "windows")]
  pub scenes: super::scene::Scenes,
}

impl Drop for AnimationManager {
  /// Signals the persistent timer thread to exit.
  ///
  /// Sets the shutdown flag, clears the ticking gate, and unparks the
  /// thread so it observes the shutdown and returns. The handle is
  /// dropped without joining so shutdown never blocks on a thread
  /// mid-wait.
  fn drop(&mut self) {
    self.timer_shutdown.store(true, Ordering::Relaxed);
    self.animation_timer_running.store(false, Ordering::Relaxed);
    if let Some(timer) = self
      .timer_thread
      .lock()
      .expect("animation mutex poisoned")
      .take()
    {
      timer.handle.thread().unpark();
    }
  }
}

impl AnimationManager {
  /// Creates a new `AnimationManager`.
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      timer_thread: Mutex::new(None),
      timer_shutdown: Arc::new(AtomicBool::new(false)),
      last_frame_at: Cell::new(Instant::now()),
      stalled_timers: Arc::new(AtomicUsize::new(0)),
      #[cfg(target_os = "windows")]
      overlay_z_settle: HashMap::new(),
      #[cfg(target_os = "windows")]
      scroll_view: super::scroll_view::ScrollView::default(),
      #[cfg(target_os = "windows")]
      scenes: super::scene::Scenes::default(),
    }
  }

  /// Whether anything still needs animation ticks.
  #[cfg(target_os = "windows")]
  pub fn has_active_animations(&self) -> bool {
    self.scenes.is_active() || !self.overlay_z_settle.is_empty()
  }

  /// Whether anything still needs animation ticks: nothing animates off
  /// Windows.
  #[cfg(not(target_os = "windows"))]
  #[allow(clippy::unused_self)]
  pub fn has_active_animations(&self) -> bool {
    false
  }

  /// Keeps re-asserting the overlays of `window_ids` behind their windows
  /// on every tick for [`OVERLAY_Z_SETTLE`], after their own z-order was
  /// changed.
  ///
  /// `set_z_order` moves the real window with `SWP_ASYNCWINDOWPOS` (plus a
  /// delayed retry), so it lands only after the same pass has already put
  /// the overlays behind the window's *old* position. A move that then
  /// takes the window down past them -- `AfterWindow` behind the focused
  /// window, or leaving the topmost band when a `shown_on_top` floating
  /// window is tiled again -- leaves its own opaque backdrop on top of it,
  /// and nothing else looks again. There is no completion signal to wait
  /// on, so the overlays are re-checked until the move has surely landed.
  #[cfg(target_os = "windows")]
  pub fn settle_overlay_z_order(
    &mut self,
    window_ids: impl IntoIterator<Item = Uuid>,
  ) {
    let now = Instant::now();
    let settle = SettleWindow {
      until: now + OVERLAY_Z_SETTLE,
      cap: now + OVERLAY_Z_SETTLE_MAX,
    };
    self
      .overlay_z_settle
      .extend(window_ids.into_iter().map(|id| (id, settle)));

    if !self.overlay_z_settle.is_empty() {
      self.ensure_timer_running();
    }
  }

  /// Pushes `window_id`'s settle deadline out by another
  /// [`OVERLAY_Z_SETTLE`] from now, within its cap, for a window whose
  /// band change has not landed yet.
  ///
  /// Returns `true` only on the call that first clamps the deadline to the
  /// cap, so callers can report a settle that never finished once.
  #[cfg(target_os = "windows")]
  pub fn extend_overlay_z_settle(&mut self, window_id: &Uuid) -> bool {
    let Some(settle) = self.overlay_z_settle.get_mut(window_id) else {
      return false;
    };

    let was_capped = settle.until >= settle.cap;
    settle.until = (Instant::now() + OVERLAY_Z_SETTLE).min(settle.cap);
    !was_capped && settle.until >= settle.cap
  }

  /// Windows whose overlays are still settling, dropping expired ones.
  #[cfg(target_os = "windows")]
  pub fn settling_overlay_windows(&mut self) -> Vec<Uuid> {
    let now = Instant::now();
    self.overlay_z_settle.retain(|_, settle| settle.until > now);
    self.overlay_z_settle.keys().copied().collect()
  }

  /// Starts a ticking phase of the persistent animation timer thread.
  ///
  /// The timer thread is spawned once (on the first animation) and parked
  /// between animations rather than re-created each time. This call wakes
  /// the parked thread (or spawns it the first time) when there are active
  /// animations and the thread is not already ticking.
  ///
  /// The thread paces itself with `DwmFlush`, ticking once per
  /// composition; on non-Windows, where that is a no-op, it sleeps for a
  /// 60 fps frame instead.
  pub fn ensure_timer_running(&self) {
    if !self.has_active_animations() {
      return;
    }

    // Idempotent: if the thread is already ticking, there is nothing to
    // do. The swap also claims the idle -> ticking transition so only
    // one caller wakes the thread.
    if self.animation_timer_running.swap(true, Ordering::Relaxed) {
      return;
    }

    self.last_frame_at.set(Instant::now());

    let mut guard =
      self.timer_thread.lock().expect("animation mutex poisoned");
    match guard.as_ref() {
      // Wake the parked thread to begin a new ticking phase.
      Some(timer) => timer.handle.thread().unpark(),
      // First animation: spawn the persistent thread.
      None => {
        if let Some(timer) = self.spawn_timer_thread() {
          *guard = Some(timer);
        }
      }
    }
  }

  /// When the watchdog should step in if no frame has run by then, or
  /// `None` while nothing is animating.
  pub fn watchdog_deadline(&self) -> Option<Instant> {
    self
      .has_active_animations()
      .then(|| self.last_frame_at.get() + FRAME_STALL_TIMEOUT)
  }

  /// Replaces a timer thread that stopped ticking with a new one.
  ///
  /// The old thread is most likely blocked in a composition wait, which
  /// can block indefinitely with the displays off. It is left to exit
  /// whenever that wait returns.
  ///
  /// At most [`MAX_STALLED_TIMERS`] replaced threads are left blocked at
  /// once: past that, replacements are stalling too, and more threads
  /// would only pile up behind them. The watchdog keeps running frames
  /// itself until one recovers.
  pub fn replace_stalled_timer(&self) {
    if self.stalled_timers.load(Ordering::Relaxed) >= MAX_STALLED_TIMERS {
      tracing::debug!("Animation timer still stalled; not replacing it.");
      return;
    }

    tracing::warn!(
      "Animation timer stalled for over {FRAME_STALL_TIMEOUT:?}; \
       replacing it."
    );

    if let Some(timer) = self
      .timer_thread
      .lock()
      .expect("animation mutex poisoned")
      .take()
    {
      self.stalled_timers.fetch_add(1, Ordering::Relaxed);
      timer.retired.store(true, Ordering::Relaxed);
      timer.handle.thread().unpark();
    }

    self.animation_timer_running.store(false, Ordering::Relaxed);
    self.ensure_timer_running();
  }

  /// Spawns the persistent animation timer thread.
  ///
  /// Returns the join handle, or `None` if the OS thread could not be
  /// created. On failure the ticking gate is reset so a later
  /// [`ensure_timer_running`] call retries the spawn.
  ///
  /// [`ensure_timer_running`]: AnimationManager::ensure_timer_running
  fn spawn_timer_thread(&self) -> Option<TimerThread> {
    let tx = self.animation_tick_tx.clone();
    let running = self.animation_timer_running.clone();
    let shutdown = self.timer_shutdown.clone();
    let retired = Arc::new(AtomicBool::new(false));
    let thread_retired = retired.clone();
    let stalled_guard = StalledTimerGuard {
      retired: retired.clone(),
      stalled_timers: self.stalled_timers.clone(),
    };

    // A real OS thread (not a Tokio task), so its blocking waits do not
    // stall the async runtime.
    let result = std::thread::Builder::new()
      .name("glazewm-anim-tick".into())
      .spawn(move || {
        let _stalled_guard = stalled_guard;

        // Raised once for the thread's whole lifetime, so ticks land on
        // the composition they wait for.
        wm_platform::set_thread_priority_highest();
        #[cfg(target_os = "windows")]
        let _mmcss = wm_platform::try_set_thread_mmcss();

        let should_exit = || {
          shutdown.load(Ordering::Relaxed)
            || thread_retired.load(Ordering::Relaxed)
        };

        loop {
          // Idle: park until a ticking phase begins (or shutdown). A
          // `while` loop re-checks the gate to absorb spurious wake-ups
          // and any buffered unpark token from a just-ended phase.
          while !running.load(Ordering::Relaxed) {
            if should_exit() {
              return;
            }
            std::thread::park();
          }
          if should_exit() {
            return;
          }

          // An immediate tick, so the first frame doesn't wait for a
          // composition.
          if tx.send(()).is_err() {
            return;
          }

          // Ticking phase: drive frames until the gate clears.
          while running.load(Ordering::Relaxed) {
            wm_platform::dwm_flush();
            #[cfg(not(target_os = "windows"))]
            std::thread::sleep(Duration::from_micros(16_667));

            // A retired thread's wait can return long after it was
            // replaced; its tick would only duplicate the new thread's.
            if should_exit() || tx.send(()).is_err() {
              return;
            }
          }
        }
      });

    match result {
      Ok(handle) => Some(TimerThread { handle, retired }),
      Err(err) => {
        tracing::warn!("Failed to spawn animation tick thread: {err}.");
        // Reset the gate so a later call retries the spawn.
        self.animation_timer_running.store(false, Ordering::Relaxed);
        None
      }
    }
  }

  /// Runs one animation frame. Accessed through `WmState` to avoid a
  /// double borrow.
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    state.animation_manager.last_frame_at.set(Instant::now());

    perf::begin_frame();
    let tick_scope = perf::scope(Stage::Tick);

    // A failed sync must not end the frame early; the first error is
    // returned once the frame is done.
    let mut frame_result = Ok(());

    // What has played on scenes goes, and so do scenes with nothing left
    // moving.
    #[cfg(target_os = "windows")]
    super::scene::tick(state);

    if state.pending_sync.has_changes() {
      keep_first_error(&mut frame_result, platform_sync(state, config));
    }

    #[cfg(target_os = "windows")]
    resync_settling_overlays(state);

    // Close the frame before deciding whether to report: `report` only
    // emits once at least one full frame has been rolled up.
    drop(tick_scope);
    perf::end_frame();

    // Keep the timer running while animations are active; park it
    // otherwise.
    if state.animation_manager.has_active_animations() {
      state.animation_manager.ensure_timer_running();
    } else {
      // Last frame of this animation burst: emit the accumulated timings.
      perf::report("animations idle");

      state
        .animation_manager
        .animation_timer_running
        .store(false, Ordering::Relaxed);
    }

    frame_result
  }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
  use super::*;

  #[test]
  fn stalled_guard_counts_only_retired_threads() {
    let stalled_timers = Arc::new(AtomicUsize::new(1));

    for (is_retired, expected) in [(false, 1), (true, 0)] {
      drop(StalledTimerGuard {
        retired: Arc::new(AtomicBool::new(is_retired)),
        stalled_timers: stalled_timers.clone(),
      });
      assert_eq!(stalled_timers.load(Ordering::Relaxed), expected);
    }
  }

  #[test]
  fn no_replacement_while_a_replaced_timer_is_stalled() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let manager = AnimationManager::new(tick_tx);
    manager
      .stalled_timers
      .store(MAX_STALLED_TIMERS, Ordering::Relaxed);
    manager
      .animation_timer_running
      .store(true, Ordering::Relaxed);

    manager.replace_stalled_timer();

    // The running gate is left as it was, so no new thread was started.
    assert!(manager.animation_timer_running.load(Ordering::Relaxed));
    assert_eq!(
      manager.stalled_timers.load(Ordering::Relaxed),
      MAX_STALLED_TIMERS
    );
  }
}
