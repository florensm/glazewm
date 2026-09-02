//! Opt-in frame profiler for the window-manager thread.
//!
//! The WM runs every relayout, animation frame, keybinding, and IPC message
//! on a single thread, so any blocking call made there directly becomes
//! input lag. This module measures where that thread's time actually goes
//! during an animation, without costing anything when disabled.
//!
//! Enabled by setting the `GLAZEWM_PERF` environment variable to anything
//! other than `0`. When disabled every entry point is a single cached
//! boolean load and the timers never call `Instant::now`.
//!
//! # Example usage
//!
//! ```no_run
//! use wm_platform::perf::{self, Stage};
//!
//! perf::begin_frame();
//! {
//!   let _scope = perf::scope(Stage::PlatformSync);
//!   // ...work being measured...
//! }
//! perf::end_frame();
//! perf::report("animations idle");
//! ```

use std::{
  cell::RefCell,
  fmt::Write,
  sync::OnceLock,
  time::{Duration, Instant},
};

/// A frame slower than this is counted as a dropped frame in the report.
///
/// Roughly two 144 Hz frame periods: past this, a tick has certainly missed
/// its own vblank and pushed the next one late as well.
const SLOW_FRAME_THRESHOLD: Duration = Duration::from_millis(14);

/// Frames after which an in-progress session reports and resets on its own.
///
/// Guards against a session that never reaches an idle point (e.g. a
/// continuously-animating window) silently accumulating forever without
/// ever logging anything.
const AUTO_REPORT_FRAMES: u32 = 600;

/// A measurable segment of the WM thread's per-frame work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
  /// The whole animation tick (`AnimationManager::update_internal`).
  Tick,
  /// `platform_sync`, including everything nested below it.
  PlatformSync,
  /// `redraw_containers`, including both stages below.
  Redraw,
  /// `redraw_containers`' setup before its per-window loop: expanding the
  /// redraw set, focus-order sorting, and the workspace-switch pre-pass.
  RedrawPrep,
  /// `redraw_containers`' per-window loop itself.
  RedrawLoop,
  /// Acrylic blur overlay sync pass.
  BlurSync,
  /// Border overlay sync pass.
  BorderSync,
  /// `DwmFlush` -- blocks until the next DWM composition frame.
  DwmFlush,
  /// `ResizeSession::begin`/`begin_reusing_surrogate`.
  SessionBegin,
  /// Cloaking the real window and pre-positioning it at its target.
  Cloak,
  /// `AnimationManager::flush_surrogate_updates`.
  SurrogateFlush,
  /// `platform_sync`'s post-flush loop tracking each live resize session's
  /// blur/border overlay onto its surrogate.
  SessionOverlays,
  /// `update_internal`'s post-`platform_sync` session/workspace cleanup and
  /// surrogate fade-out tail.
  Cleanup,
  /// `ResizeSession::pre_commit`.
  PreCommit,
  /// `NativeWindow::frame` -- a `DwmGetWindowAttribute` round-trip.
  NativeFrame,
}

impl Stage {
  /// Every stage, in report order.
  const ALL: [Stage; 15] = [
    Stage::Tick,
    Stage::PlatformSync,
    Stage::Redraw,
    Stage::RedrawPrep,
    Stage::RedrawLoop,
    Stage::BlurSync,
    Stage::BorderSync,
    Stage::DwmFlush,
    Stage::SessionBegin,
    Stage::Cloak,
    Stage::SurrogateFlush,
    Stage::SessionOverlays,
    Stage::Cleanup,
    Stage::PreCommit,
    Stage::NativeFrame,
  ];

  /// Number of distinct stages, i.e. the width of the accumulator arrays.
  const COUNT: usize = Self::ALL.len();

  /// Dense index of this stage into the accumulator arrays.
  const fn index(self) -> usize {
    self as usize
  }

  /// Short human-readable name used in the report.
  const fn label(self) -> &'static str {
    match self {
      Stage::Tick => "tick",
      Stage::PlatformSync => "platform_sync",
      Stage::Redraw => "redraw",
      Stage::RedrawPrep => "  redraw_prep",
      Stage::RedrawLoop => "  redraw_loop",
      Stage::BlurSync => "blur_sync",
      Stage::BorderSync => "border_sync",
      Stage::DwmFlush => "dwm_flush",
      Stage::SessionBegin => "session_begin",
      Stage::Cloak => "cloak",
      Stage::SurrogateFlush => "surrogate_flush",
      Stage::SessionOverlays => "session_overlays",
      Stage::Cleanup => "cleanup",
      Stage::PreCommit => "pre_commit",
      Stage::NativeFrame => "native_frame",
    }
  }
}

/// `tracing` target the profiler's reports are emitted under.
///
/// Distinct from the WM's own targets so a subscriber can route reports to
/// their own sink without picking up unrelated `INFO` events -- see
/// `setup_logging` in the `wm` crate.
pub const LOG_TARGET: &str = "glazewm::perf";

/// Whether profiling is on, resolved once from `GLAZEWM_PERF`.
static ENABLED: OnceLock<bool> = OnceLock::new();

/// Returns whether the profiler is enabled for this process.
///
/// Reads `GLAZEWM_PERF` on first call and caches the result, so toggling
/// the variable at runtime has no effect.
#[must_use]
pub fn is_enabled() -> bool {
  *ENABLED.get_or_init(|| {
    std::env::var("GLAZEWM_PERF").is_ok_and(|value| value != "0")
  })
}

/// Per-stage totals for the frame in progress, plus the session totals they
/// roll up into.
#[derive(Default)]
struct Profiler {
  /// Time accumulated in each stage during the frame in progress.
  frame_total: [Duration; Stage::COUNT],
  /// Calls to each stage during the frame in progress.
  frame_calls: [u32; Stage::COUNT],
  /// Time accumulated in each stage across the whole session.
  total: [Duration; Stage::COUNT],
  /// Calls to each stage across the whole session.
  calls: [u32; Stage::COUNT],
  /// Worst single frame's time in each stage across the session.
  worst_frame: [Duration; Stage::COUNT],
  /// Frames completed in the session.
  frames: u32,
  /// Frames whose `Tick` exceeded [`SLOW_FRAME_THRESHOLD`].
  slow_frames: u32,
  /// Highest simultaneously-animating window count seen this session.
  peak_windows: usize,
  /// When the session's first frame began.
  started_at: Option<Instant>,
}

thread_local! {
  /// Per-thread profiler state.
  ///
  /// Only the WM thread ever calls [`report`], so samples taken on other
  /// threads (e.g. the animation tick thread's own pacing `DwmFlush`)
  /// accumulate into their own inert instance and never pollute the
  /// report.
  static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::default());
}

/// An in-flight stage measurement, recorded on drop.
///
/// Created by [`scope`]. Holds no state when profiling is disabled.
pub struct Scope {
  /// The stage being measured.
  stage: Stage,
  /// When the scope was opened, or `None` when profiling is disabled.
  start: Option<Instant>,
}

impl Drop for Scope {
  /// Accumulates the elapsed time into the current frame's totals.
  fn drop(&mut self) {
    let Some(start) = self.start else {
      return;
    };
    let elapsed = start.elapsed();
    let index = self.stage.index();

    PROFILER.with(|profiler| {
      if let Ok(mut profiler) = profiler.try_borrow_mut() {
        profiler.frame_total[index] += elapsed;
        profiler.frame_calls[index] += 1;
      }
    });
  }
}

/// Starts measuring `stage`, until the returned [`Scope`] is dropped.
///
/// Nested and repeated scopes of the same stage are summed, so an outer
/// stage legitimately overlaps the inner stages it contains.
#[must_use]
pub fn scope(stage: Stage) -> Scope {
  Scope {
    stage,
    start: is_enabled().then(Instant::now),
  }
}

/// Records the number of windows animating simultaneously this frame.
///
/// Reported as a peak, so a session's worst-case concurrency is visible
/// alongside its worst-case frame times.
pub fn note_window_count(count: usize) {
  if is_enabled() {
    record_window_count(count);
  }
}

/// [`note_window_count`] without the `GLAZEWM_PERF` gate.
///
/// The gate resolves once per process, so the recording logic is factored
/// out here to stay reachable from tests.
fn record_window_count(count: usize) {
  PROFILER.with(|profiler| {
    if let Ok(mut profiler) = profiler.try_borrow_mut() {
      profiler.peak_windows = profiler.peak_windows.max(count);
    }
  });
}

/// Marks the start of a frame, clearing the previous frame's per-stage
/// accumulators.
pub fn begin_frame() {
  if is_enabled() {
    start_frame();
  }
}

/// [`begin_frame`] without the `GLAZEWM_PERF` gate.
fn start_frame() {
  PROFILER.with(|profiler| {
    if let Ok(mut profiler) = profiler.try_borrow_mut() {
      profiler.frame_total = [Duration::ZERO; Stage::COUNT];
      profiler.frame_calls = [0; Stage::COUNT];
      profiler.started_at.get_or_insert_with(Instant::now);
    }
  });
}

/// Rolls the completed frame's per-stage accumulators into the session
/// totals.
///
/// Auto-reports every [`AUTO_REPORT_FRAMES`] frames so a session that never
/// idles still produces output.
pub fn end_frame() {
  if !is_enabled() {
    return;
  }

  if roll_up_frame() {
    report("frame cap reached");
  }
}

/// [`end_frame`] without the `GLAZEWM_PERF` gate.
///
/// Returns `true` once the session has reached [`AUTO_REPORT_FRAMES`], i.e.
/// when the caller should report and reset.
fn roll_up_frame() -> bool {
  PROFILER.with(|profiler| {
    let Ok(mut profiler) = profiler.try_borrow_mut() else {
      return false;
    };

    for index in 0..Stage::COUNT {
      let frame_total = profiler.frame_total[index];
      profiler.total[index] += frame_total;
      profiler.calls[index] += profiler.frame_calls[index];
      profiler.worst_frame[index] =
        profiler.worst_frame[index].max(frame_total);
    }

    profiler.frames += 1;
    if profiler.frame_total[Stage::Tick.index()] > SLOW_FRAME_THRESHOLD {
      profiler.slow_frames += 1;
    }

    profiler.frames >= AUTO_REPORT_FRAMES
  })
}

/// Logs the session's accumulated timings at `INFO` and resets them.
///
/// A no-op when profiling is disabled or no frames have been recorded, so
/// callers can invoke it unconditionally at any natural end-of-animation
/// boundary. `reason` labels what triggered the report.
pub fn report(reason: &str) {
  if !is_enabled() {
    return;
  }

  if let Some(report) = take_report(reason) {
    tracing::info!(target: LOG_TARGET, "{report}");
  }
}

/// Formats the session's accumulated timings and clears them, ungated by
/// `GLAZEWM_PERF`.
///
/// Returns `None` when no frames have been recorded, so a caller that
/// reports at every idle boundary emits nothing on the boundaries where
/// nothing ran.
fn take_report(reason: &str) -> Option<String> {
  let summary = PROFILER.with(|profiler| {
    let mut profiler = profiler.try_borrow_mut().ok()?;
    if profiler.frames == 0 {
      return None;
    }
    Some(std::mem::take(&mut *profiler))
  })?;

  let elapsed = summary
    .started_at
    .map_or(Duration::ZERO, |started_at| started_at.elapsed());
  let frames = f64::from(summary.frames);

  // Writing into a `String` is infallible, so the results are discarded.
  let mut lines = String::new();
  let _ = writeln!(
    lines,
    "perf [{reason}]: {} frames in {:.1}ms, {} slow (>{:.0}ms), peak {} \
     window(s) animating",
    summary.frames,
    elapsed.as_secs_f64() * 1000.0,
    summary.slow_frames,
    SLOW_FRAME_THRESHOLD.as_secs_f64() * 1000.0,
    summary.peak_windows,
  );
  let _ = writeln!(
    lines,
    "  {:<16}{:>7}{:>11}{:>11}{:>11}",
    "stage", "calls", "total", "per-frame", "worst",
  );

  for stage in Stage::ALL {
    let index = stage.index();
    if summary.calls[index] == 0 {
      continue;
    }

    let _ = writeln!(
      lines,
      "  {:<16}{:>7}{:>9.1}ms{:>9.2}ms{:>9.2}ms",
      stage.label(),
      summary.calls[index],
      summary.total[index].as_secs_f64() * 1000.0,
      summary.total[index].as_secs_f64() * 1000.0 / frames,
      summary.worst_frame[index].as_secs_f64() * 1000.0,
    );
  }

  Some(lines.trim_end().to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Opens an ungated scope, so the recording path is exercised regardless
  /// of `GLAZEWM_PERF` (whose value resolves once per process).
  fn forced_scope(stage: Stage) -> Scope {
    Scope {
      stage,
      start: Some(Instant::now()),
    }
  }

  #[test]
  fn stage_indices_are_dense_and_ordered() {
    for (expected, stage) in Stage::ALL.into_iter().enumerate() {
      assert_eq!(stage.index(), expected);
    }
  }

  #[test]
  fn disabled_entry_points_are_inert() {
    // `GLAZEWM_PERF` is unset under `cargo test`, so the public entry
    // points must record nothing and, critically, must not panic.
    assert!(!is_enabled());
    begin_frame();
    drop(scope(Stage::Tick));
    note_window_count(4);
    end_frame();
    report("test");

    PROFILER.with(|profiler| assert_eq!(profiler.borrow().frames, 0));
  }

  #[test]
  fn accumulates_stages_across_frames_then_resets() {
    // Runs on its own thread so the thread-local profiler is untouched by
    // the other tests in this module.
    std::thread::spawn(|| {
      start_frame();
      drop(forced_scope(Stage::Tick));
      drop(forced_scope(Stage::DwmFlush));
      drop(forced_scope(Stage::DwmFlush));
      record_window_count(3);
      assert!(!roll_up_frame());

      start_frame();
      drop(forced_scope(Stage::Tick));
      record_window_count(2);
      assert!(!roll_up_frame());

      PROFILER.with(|profiler| {
        let profiler = profiler.borrow();
        assert_eq!(profiler.frames, 2);
        // Per-frame accumulators are cleared by `start_frame`, so the two
        // `DwmFlush` calls land in the session totals exactly once.
        assert_eq!(profiler.calls[Stage::DwmFlush.index()], 2);
        assert_eq!(profiler.calls[Stage::Tick.index()], 2);
        assert_eq!(profiler.calls[Stage::Redraw.index()], 0);
        // A peak, not a last-write.
        assert_eq!(profiler.peak_windows, 3);
      });

      let report = take_report("unit test").expect("frames were recorded");
      assert!(report.starts_with("perf [unit test]: 2 frames in "));
      assert!(report.contains("peak 3 window(s) animating"));
      assert!(report.contains("dwm_flush"));
      // Stages that never ran are omitted from the table.
      assert!(!report.contains("border_sync"));

      // Reporting resets the session, so a second report has nothing to say.
      assert!(take_report("unit test").is_none());
      PROFILER.with(|profiler| assert_eq!(profiler.borrow().frames, 0));
    })
    .join()
    .expect("profiler test thread panicked");
  }
}
