//! The animation timer thread, and the instants frames are drawn for.

use std::{
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

use wm_platform::{perf, DxgiVsyncWaiter, FrameClock};

use super::AnimationManager;

/// Fraction of the monitor's vblank period to lead the animation clock by.
///
/// After `IDXGIOutput::WaitForVBlank` returns, the surrogate update
/// written this tick is not composited by DWM until the *next* vblank — up
/// to one full frame period later. Advancing the animation clock by a
/// fraction of that period makes the computed position align with where it
/// will be when DWM actually presents it, rather than systematically
/// lagging behind.
///
/// Expressed as a fraction of the per-monitor frame period read live from
/// the installed `DxgiVsyncWaiter`, so the compensation scales correctly
/// across monitors with different refresh rates (e.g. 60 Hz + 175 Hz).
/// `0.5` leads by half a frame: a balance between under-compensating
/// (motion trails the cursor) and over-compensating (motion arrives
/// early). Tune here if the slide feels laggy (increase toward `1.0`) or
/// rushed (decrease toward `0.0`).
const VSYNC_LEAD_FRACTION: f32 = 0.5;

/// How long animations may go without a frame before the watchdog runs
/// one itself and replaces the timer thread.
const FRAME_STALL_TIMEOUT: Duration = Duration::from_millis(250);

/// Replaced timer threads that may be left blocked at once. See
/// [`AnimationManager::replace_stalled_timer`].
const MAX_STALLED_TIMERS: usize = 3;

/// The animation timer thread, with the flag that retires it.
pub(super) struct TimerThread {
  pub(super) handle: std::thread::JoinHandle<()>,

  /// Makes the thread exit once its current wait returns. Set when the
  /// watchdog replaces a thread stuck in a vsync wait.
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

/// The instants an animation tick's frame is evaluated at, read once per
/// frame.
#[derive(Clone, Copy, Debug)]
pub(super) struct FrameTimes {
  /// See [`AnimationManager::frame_now`].
  pub(super) now: Instant,
  /// One frame period before `now`, where clocks started this frame
  /// begin, so its first frame already shows motion.
  pub(super) clock_origin: Instant,
}

/// The wake a tick being drawn at `now` came from: the timer thread's
/// `last_wake`, if it came since the frame before began at
/// `last_frame_at`. An older one was a tick drawn already, or one before
/// the ticking phase began (`ensure_timer_running` resets
/// `last_frame_at`), whose first tick is sent at once: then `now`.
fn tick_wake(
  last_wake: Option<Instant>,
  last_frame_at: Instant,
  now: Instant,
) -> Instant {
  last_wake
    .filter(|wake| *wake > last_frame_at && *wake <= now)
    .unwrap_or(now)
}

impl AnimationManager {
  /// Starts a ticking phase of the persistent animation timer thread.
  ///
  /// The timer thread is spawned once (on the first animation) and parked
  /// between animations rather than re-created each time, so the cost of
  /// thread creation and priority/MMCSS setup never lands on the
  /// input-to-first-frame latency path. This call wakes the parked thread
  /// (or spawns it the first time) when there are active animations and
  /// the thread is not already ticking.
  ///
  /// The thread uses a two-tier vsync strategy on Windows:
  ///
  /// 1. **`IDXGIOutput::WaitForVBlank`** — when a `DxgiVsyncWaiter` is
  ///    installed, waits for the animation monitor's specific vblank
  ///    signal. Per-monitor and reliable at any Hz.
  /// 2. **`DwmFlush`** — fallback when no waiter is installed, aligning to
  ///    the primary monitor's composition cycle.
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
  /// The old thread is most likely blocked in `WaitForVBlank` on an output
  /// that went stale, which can block indefinitely. It is left to exit
  /// whenever that wait returns, and the vsync waiter is dropped so the
  /// new thread paces itself with `DwmFlush` instead.
  ///
  /// At most [`MAX_STALLED_TIMERS`] replaced threads are left blocked at
  /// once: past that, replacements are stalling too (e.g. `DwmFlush` with
  /// the displays off), and more threads would only pile up behind them.
  /// The watchdog keeps running frames itself until one recovers.
  pub fn replace_stalled_timer(&self) {
    if self.stalled_timers.load(Ordering::Relaxed) >= MAX_STALLED_TIMERS {
      tracing::debug!("Animation timer still stalled; not replacing it.");
      return;
    }

    tracing::warn!(
      "Animation timer stalled for over {FRAME_STALL_TIMEOUT:?}; \
       replacing it."
    );

    self.clear_waiter();

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
    let vsync_waiter = self.animation_timer_vsync.clone();
    let frame_clock = self.frame_clock.clone();

    // Spawn a real OS thread (not a Tokio task) so blocking vsync waits do
    // not stall the async runtime.
    let result = std::thread::Builder::new()
      .name("glazewm-anim-tick".into())
      .spawn(move || {
        let _stalled_guard = stalled_guard;

        // Elevate scheduling priority once for the thread's whole
        // lifetime. MMCSS "DisplayPostProcessing" gives
        // near-real-time guarantees beyond
        // THREAD_PRIORITY_HIGHEST, matching the scheduling class used by
        // DWM and video renderers. Falls back gracefully to
        // THREAD_PRIORITY_HIGHEST if avrt.dll is unavailable. The
        // persistent thread keeps the registration across
        // animations rather than re-acquiring it per switch; while
        // parked it consumes no CPU.
        wm_platform::set_thread_priority_highest();
        let _mmcss = wm_platform::try_set_thread_mmcss();

        let should_exit = || {
          shutdown.load(Ordering::Relaxed)
            || thread_retired.load(Ordering::Relaxed)
        };

        loop {
          // Idle: park until a ticking phase begins (or shutdown). A
          // `while` loop re-checks the gate to absorb spurious
          // wake-ups and any buffered unpark token from a
          // just-ended phase.
          while !running.load(Ordering::Relaxed) {
            if should_exit() {
              return;
            }
            std::thread::park();
          }
          if should_exit() {
            return;
          }

          // Send an immediate tick so the first animation frame begins
          // without waiting for the next vblank. Without this the
          // surrogate is frozen at its start position for up to
          // one full frame period (~16 ms at 60 Hz, ~5.7 ms at
          // 175 Hz) before any movement begins.
          if tx.send(()).is_err() {
            return;
          }

          // Ticking phase: drive frames until the gate clears.
          while running.load(Ordering::Relaxed) {
            // Per-monitor `IDXGIOutput::WaitForVBlank`. Cloned under the
            // lock so the wait runs without holding it: cleanup can clear
            // the waiter without blocking on a wait in progress.
            let waiter = vsync_waiter
              .lock()
              .expect("animation mutex poisoned")
              .clone();

            // `DwmFlush` when no waiter is installed, or its wait failed.
            // Its return is as periodic as a vblank, so it feeds the
            // clock the same way.
            if !waiter.is_some_and(|waiter| waiter.wait()) {
              wm_platform::dwm_flush();
            }

            frame_clock
              .lock()
              .unwrap_or_else(std::sync::PoisonError::into_inner)
              .note_vblank(Instant::now());

            // A retired thread's wait can return long after it was
            // replaced; its tick would only duplicate the new thread's.
            if should_exit() || tx.send(()).is_err() {
              return;
            }
          }
          // Gate cleared: loop back to park until the next animation.
        }
        // `_mmcss` is dropped on return, reverting the MMCSS registration.
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

  /// The instant this frame is evaluated at: the latest vblank plus a
  /// fraction of the frame period (see [`VSYNC_LEAD_FRACTION`]), or `None`
  /// while the timer thread hasn't woken for one lately.
  pub(super) fn predictive_vsync_now(&self) -> Option<Instant> {
    let clock = self.clock();
    let vblank = clock.vblank_at(Instant::now())?;

    Some(vblank + clock.period().mul_f32(1.0 - VSYNC_LEAD_FRACTION))
  }

  /// The pacing monitor's frame period, or zero before a vsync waiter is
  /// installed.
  fn frame_period(&self) -> Duration {
    let has_waiter = self
      .animation_timer_vsync
      .lock()
      .is_ok_and(|waiter| waiter.is_some());

    if has_waiter {
      self.clock().period()
    } else {
      Duration::ZERO
    }
  }

  fn clock(&self) -> std::sync::MutexGuard<'_, FrameClock> {
    self
      .frame_clock
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
  }

  /// Drops the vsync waiter, so the timer thread paces itself with
  /// `DwmFlush` until an animation installs one.
  pub(super) fn clear_waiter(&self) {
    *self
      .animation_timer_vsync
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    self.clock().reset(FrameClock::DEFAULT_PERIOD);
  }

  /// The time a frame drawn now is evaluated at, read afresh: for
  /// sampling a movement outside a redraw pass where it was last drawn.
  pub(crate) fn frame_time(&self) -> Instant {
    self.predictive_vsync_now().unwrap_or_else(Instant::now)
  }

  /// Starts a new redraw pass, so the next [`frame_now`] takes a fresh
  /// reading.
  ///
  /// [`frame_now`]: AnimationManager::frame_now
  pub fn begin_redraw_pass(&mut self) {
    self.pass_now = None;
  }

  /// The timestamp this redraw pass evaluates every window at: the
  /// predictive vsync instant on Windows when one is available, else
  /// wall-clock now -- read once per pass and shared.
  ///
  /// Shared because a window's animation clock starts at its first
  /// evaluation. Read per window, the reading lands after that window's
  /// stand-in setup (peek creation, preposition) and the ones before it,
  /// so windows relaid out together started up to a frame
  /// apart: measured 11ms between a shrinking window and the growing
  /// neighbour started after it, which the steep start of the easing
  /// curve turned into a ~190px hole between them on the first frame.
  pub(crate) fn frame_now(&mut self) -> Instant {
    if let Some(now) = self.pass_now {
      return now;
    }

    let now = self.frame_time();

    self.pass_now = Some(now);
    now
  }

  /// Starts an animation tick's redraw pass and reads the instants its
  /// frame is evaluated at.
  pub(super) fn begin_frame_times(&mut self) -> FrameTimes {
    self.begin_redraw_pass();
    let now = self.frame_now();
    let clock_origin = now.checked_sub(self.frame_period()).unwrap_or(now);

    FrameTimes { now, clock_origin }
  }

  /// When the timer thread woke for the tick about to be drawn, for the
  /// profiler: its latest wake, if that came since the last frame began,
  /// else now, as a ticking phase's first tick is sent without a wait.
  pub(super) fn tick_woke_at(&self) -> Instant {
    tick_wake(
      self.clock().last_wake(),
      self.last_frame_at.get(),
      Instant::now(),
    )
  }

  /// Paces animation ticks against the monitor with `monitor_handle`,
  /// i.e. the one the animating window is on. Leaves a workspace switch's
  /// monitor in place while it plays.
  pub(crate) fn ensure_waiter_for(&self, monitor_handle: isize) {
    if self.workspace_switch.is_none() {
      self.install_waiter(monitor_handle);
    }
  }

  /// Installs the vsync waiter of the monitor with `monitor_handle`,
  /// unless it is the one installed.
  ///
  /// Always switches, even to a slower refresh rate: keeping a faster
  /// waiter paces a 60 Hz monitor's animations against the wrong vblank,
  /// which reads as stutter specific to that monitor.
  pub(super) fn install_waiter(&self, monitor_handle: isize) {
    let mut installed = self
      .animation_timer_vsync
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);

    if installed
      .as_ref()
      .is_some_and(|waiter| waiter.monitor_handle() == monitor_handle)
    {
      return;
    }

    match DxgiVsyncWaiter::for_monitor(monitor_handle) {
      Ok(waiter) => {
        let period = Duration::from_micros(waiter.frame_period_us());

        // Slow frames are counted against the frame period of whichever
        // monitor is actually pacing the animation, so the profiler stays
        // meaningful across a 60 Hz panel and a 175 Hz one alike.
        perf::set_frame_budget(period);
        tracing::debug!(
          monitor = monitor_handle,
          period_us = waiter.frame_period_us(),
          "vsync waiter switched"
        );

        *installed = Some(waiter);
        self.clock().reset(period);
      }
      Err(err) => {
        tracing::warn!(?err, "failed to create vsync waiter for monitor");
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use tokio::sync::mpsc;

  use super::*;

  /// The profiler times a peek frame from the wake its tick came from,
  /// not from one already drawn or from before the ticking phase.
  #[test]
  fn a_tick_is_timed_from_its_own_wake() {
    let t0 = Instant::now();
    let at = |ms| t0 + Duration::from_millis(ms);

    assert_eq!(tick_wake(Some(at(10)), at(5), at(12)), at(10));
    assert_eq!(tick_wake(Some(at(4)), at(5), at(12)), at(12));
    assert_eq!(tick_wake(None, at(5), at(12)), at(12));
    assert_eq!(tick_wake(Some(at(13)), at(5), at(12)), at(12));
  }

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
