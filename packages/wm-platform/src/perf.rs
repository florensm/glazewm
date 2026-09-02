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
  cmp::Reverse,
  collections::VecDeque,
  fmt::Write,
  sync::{
    atomic::{AtomicU64, AtomicU32, Ordering},
    Mutex, OnceLock,
  },
  time::{Duration, Instant},
};

/// Fallback frame budget, used until [`set_frame_budget`] reports the real
/// one.
///
/// One 60 Hz frame period -- the most pessimistic common refresh rate, so an
/// uncalibrated report over-counts slow frames rather than hiding them.
const DEFAULT_FRAME_BUDGET: Duration = Duration::from_micros(16_667);

/// The monitor's frame period, as last reported by [`set_frame_budget`].
///
/// A frame whose `Tick` exceeds this has missed its own vblank. Stored in
/// microseconds so it can live in an atomic; the budget is refresh-rate
/// dependent, and hard-coding one made "slow" meaningless on any monitor
/// that did not happen to match it (a 14 ms constant chosen for 144 Hz
/// flagged every perfectly on-budget frame on a 60 Hz panel).
static FRAME_BUDGET_US: AtomicU64 =
  AtomicU64::new(DEFAULT_FRAME_BUDGET.as_micros() as u64);

/// Sets the frame budget slow frames are counted against, from the frame
/// period of the monitor the animation is actually being paced by.
///
/// Cheap enough to call every time the pacing monitor changes; a zero or
/// absurd period is ignored so a bad reading cannot disable the counter.
pub fn set_frame_budget(period: Duration) {
  let micros = period.as_micros();
  if (1_000..=100_000).contains(&micros) {
    if let Ok(micros) = u64::try_from(micros) {
      FRAME_BUDGET_US.store(micros, Ordering::Relaxed);
    }
  }
}

/// Returns the frame budget slow frames are counted against.
fn frame_budget() -> Duration {
  Duration::from_micros(FRAME_BUDGET_US.load(Ordering::Relaxed))
}

/// Frames after which an in-progress session reports and resets on its own.
///
/// Guards against a session that never reaches an idle point (e.g. a
/// continuously-animating window) silently accumulating forever without
/// ever logging anything.
const AUTO_REPORT_FRAMES: u32 = 600;

/// Maximum number of distinct `(process, sync)` pairs attributed in the
/// `rd_apply` breakdown before the rest are lumped into one overflow row.
///
/// Bounds the table on a machine with many managed applications; the
/// interesting case is always a handful of slow apps, so a cap this size
/// never hides the culprit.
const APPLY_SAMPLE_LIMIT: usize = 24;

/// Maximum queued-event timestamps held per [`EventKind`].
///
/// The queues pair one-to-one with each listener's channel, so they only
/// grow if a producer outruns the WM's main loop -- which is exactly the
/// starvation being measured. The cap keeps a runaway producer (e.g. mouse
/// moves while the loop is blocked) from growing without bound; overflow is
/// counted so the report can say the numbers are incomplete.
const EVENT_QUEUE_LIMIT: usize = 1024;

/// A class of platform event whose queue wait is measured.
///
/// Each variant maps to exactly one listener channel with a single producer
/// and a single consumer, which is what makes the FIFO timestamp pairing in
/// [`mark_event_queued`]/[`record_event_dequeued`] sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
  /// A matched keybinding, queued from the low-level keyboard hook.
  Keybinding,
  /// A mouse move or button event, queued from the low-level mouse hook.
  Mouse,
  /// A window event, queued from the win-event hook.
  Window,
  /// A display-settings-changed event.
  Display,
}

impl EventKind {
  /// Every event kind, in report order.
  const ALL: [EventKind; 4] = [
    EventKind::Keybinding,
    EventKind::Mouse,
    EventKind::Window,
    EventKind::Display,
  ];

  /// Number of distinct event kinds, i.e. the width of the accumulators.
  const COUNT: usize = Self::ALL.len();

  /// Dense index of this kind into the accumulator arrays.
  const fn index(self) -> usize {
    self as usize
  }

  /// Short human-readable name used in the report.
  const fn label(self) -> &'static str {
    match self {
      EventKind::Keybinding => "keybinding",
      EventKind::Mouse => "mouse",
      EventKind::Window => "window",
      EventKind::Display => "display",
    }
  }
}

/// Enqueue timestamps awaiting their matching dequeue, one queue per
/// [`EventKind`].
///
/// Shared across threads because events are queued on their listener's hook
/// thread and consumed on the WM thread. Held behind a `Mutex` rather than a
/// lock-free structure deliberately: the critical section is a single
/// push/pop, and the producers include a low-level keyboard hook where a
/// long stall would delay system-wide input.
static EVENT_QUEUES: [Mutex<VecDeque<Instant>>; EventKind::COUNT] =
  [const { Mutex::new(VecDeque::new()) }; EventKind::COUNT];

/// Timestamps dropped because a queue hit [`EVENT_QUEUE_LIMIT`], per kind.
///
/// Non-zero means the FIFO pairing has slipped and the reported waits for
/// that kind understate reality, so the report says so instead of quietly
/// printing wrong numbers.
static EVENT_QUEUE_OVERFLOW: [AtomicU32; EventKind::COUNT] =
  [const { AtomicU32::new(0) }; EventKind::COUNT];

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
  /// `AnimationManager::start_animation_if_needed`: this frame's animation
  /// math plus queueing the surrogate update.
  AnimStep,
  /// `redraw_containers`' `Frozen` arm, past the first-frame cloak.
  RedrawFrozen,
  /// `redraw_containers`' `Apply` arm, i.e. repositioning a real window.
  RedrawApply,
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
  /// `SurrogateBatch::commit` -- the `DeferWindowPos` transaction that
  /// actually moves every surrogate and overlay window queued this frame.
  BatchCommit,
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
  /// The border overlay's `SetWindowRgn` hole, including building the two
  /// GDI regions it combines.
  OverlayRegion,
  /// The border overlay's `Windows.UI.Composition` visual-tree resize.
  OverlayVisual,
}

impl Stage {
  /// Every stage, in report order.
  const ALL: [Stage; 21] = [
    Stage::Tick,
    Stage::PlatformSync,
    Stage::Redraw,
    Stage::RedrawPrep,
    Stage::RedrawLoop,
    Stage::AnimStep,
    Stage::RedrawFrozen,
    Stage::RedrawApply,
    Stage::BlurSync,
    Stage::BorderSync,
    Stage::DwmFlush,
    Stage::SessionBegin,
    Stage::Cloak,
    Stage::SurrogateFlush,
    Stage::BatchCommit,
    Stage::SessionOverlays,
    Stage::Cleanup,
    Stage::PreCommit,
    Stage::NativeFrame,
    Stage::OverlayRegion,
    Stage::OverlayVisual,
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
      Stage::RedrawPrep => "redraw_prep",
      Stage::RedrawLoop => "redraw_loop",
      Stage::AnimStep => "anim_step",
      Stage::RedrawFrozen => "rd_frozen",
      Stage::RedrawApply => "rd_apply",
      Stage::BlurSync => "blur_sync",
      Stage::BorderSync => "border_sync",
      Stage::DwmFlush => "dwm_flush",
      Stage::SessionBegin => "session_begin",
      Stage::Cloak => "cloak",
      Stage::SurrogateFlush => "surrogate_flush",
      Stage::BatchCommit => "batch_commit",
      Stage::SessionOverlays => "session_overlays",
      Stage::Cleanup => "cleanup",
      Stage::PreCommit => "pre_commit",
      Stage::NativeFrame => "native_frame",
      Stage::OverlayRegion => "ovl_region",
      Stage::OverlayVisual => "ovl_visual",
    }
  }

  /// Indent depth in the report, for stages that genuinely nest inside the
  /// stage above them.
  ///
  /// Only meaningful for stages where [`is_cross_cutting`] is `false`.
  ///
  /// [`is_cross_cutting`]: Stage::is_cross_cutting
  const fn depth(self) -> usize {
    match self {
      Stage::Tick => 0,
      Stage::PlatformSync | Stage::Cleanup => 1,
      Stage::Redraw => 2,
      Stage::RedrawPrep
      | Stage::RedrawLoop
      | Stage::SurrogateFlush
      | Stage::SessionOverlays => 3,
      Stage::AnimStep | Stage::RedrawFrozen | Stage::RedrawApply => 4,
      _ => 0,
    }
  }

  /// Whether this stage is called from several places rather than nesting
  /// under one parent.
  ///
  /// These are reported separately: their time is already included in the
  /// tree above, but attributing them to any single parent would be wrong.
  /// `SurrogateBatch::commit`, for instance, runs from the surrogate flush,
  /// both overlay sync passes, the cloak commit and the cleanup tail -- an
  /// earlier version of this report indented it under `surrogate_flush`,
  /// where its total exceeded its supposed parent's.
  const fn is_cross_cutting(self) -> bool {
    matches!(
      self,
      Stage::BatchCommit
        | Stage::DwmFlush
        | Stage::SessionBegin
        | Stage::Cloak
        | Stage::PreCommit
        | Stage::NativeFrame
        | Stage::OverlayRegion
        | Stage::OverlayVisual
        | Stage::BlurSync
        | Stage::BorderSync
    )
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
  /// Frames whose `Tick` exceeded the monitor's frame budget.
  slow_frames: u32,
  /// Frame budget the session's slow frames were counted against.
  budget: Duration,
  /// Highest simultaneously-animating window count seen this session.
  peak_windows: usize,
  /// When the session's first frame began.
  started_at: Option<Instant>,
  /// Per-`(process, sync)` breakdown of the `RedrawApply` repositions.
  apply_samples: Vec<ApplySample>,
  /// Repositions that arrived after [`APPLY_SAMPLE_LIMIT`] distinct pairs
  /// were already tracked, collapsed into one bucket.
  apply_overflow: ApplySample,
  /// Queue wait accumulated per event kind.
  event_wait: [EventWait; EventKind::COUNT],
}

/// One row of the `rd_apply` breakdown: every reposition of a given
/// process's windows, split by whether the call was synchronous.
///
/// Split by `synchronous` because that is the whole point of the
/// measurement -- an async `SetWindowPos` returns immediately, while a
/// synchronous one blocks the WM thread on the target application's own
/// message pump.
#[derive(Default)]
struct ApplySample {
  /// Executable name of the window's process, empty for the overflow row.
  process: String,
  /// Whether `SWP_ASYNCWINDOWPOS` was omitted for these calls.
  synchronous: bool,
  /// Repositions recorded for this pair.
  calls: u32,
  /// Time spent in those repositions.
  total: Duration,
  /// Worst single reposition.
  worst: Duration,
}

impl ApplySample {
  /// Folds one reposition into this row.
  fn record(&mut self, elapsed: Duration) {
    self.calls += 1;
    self.total += elapsed;
    self.worst = self.worst.max(elapsed);
  }
}

/// Time events of one [`EventKind`] spent queued before the main loop
/// serviced them.
#[derive(Clone, Copy, Default)]
struct EventWait {
  /// Events dequeued during the session.
  count: u32,
  /// Total time those events spent waiting.
  total: Duration,
  /// Worst single wait.
  worst: Duration,
  /// Dequeues that found no matching enqueue timestamp.
  ///
  /// Should always be zero; a non-zero value means the FIFO pairing is
  /// broken and the other numbers in the row cannot be trusted.
  unpaired: u32,
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

/// An in-flight real-window reposition, attributed to a process on drop.
///
/// Created by [`apply_scope`]. Holds no state when profiling is disabled.
pub struct ApplyScope {
  /// Executable name of the repositioned window's process.
  process: Option<String>,
  /// Whether `SWP_ASYNCWINDOWPOS` was omitted for this call.
  synchronous: bool,
  /// When the scope was opened, or `None` when profiling is disabled.
  start: Option<Instant>,
}

impl Drop for ApplyScope {
  /// Accumulates the elapsed time into the session's `rd_apply` breakdown.
  fn drop(&mut self) {
    let (Some(start), Some(process)) = (self.start, self.process.take())
    else {
      return;
    };

    record_apply(process, self.synchronous, start.elapsed());
  }
}

/// Measures one real-window reposition and attributes it to the owning
/// process, until the returned [`ApplyScope`] is dropped.
///
/// `process` is only invoked when profiling is enabled, so callers can pass
/// a closure that clones the window's cached process name without paying for
/// it in a normal build.
///
/// `synchronous` records whether `SWP_ASYNCWINDOWPOS` was omitted -- only
/// synchronous calls can block on the target application's message pump, so
/// the report keeps the two apart.
///
/// # Example usage
///
/// ```no_run
/// use wm_platform::perf;
///
/// let _scope = perf::apply_scope(|| "explorer.exe".to_string(), true);
/// // ...the `SetWindowPos` being measured...
/// ```
#[must_use]
pub fn apply_scope<F>(process: F, synchronous: bool) -> ApplyScope
where
  F: FnOnce() -> String,
{
  if is_enabled() {
    ApplyScope {
      process: Some(process()),
      synchronous,
      start: Some(Instant::now()),
    }
  } else {
    ApplyScope {
      process: None,
      synchronous,
      start: None,
    }
  }
}

/// Folds one attributed reposition into the session's breakdown, ungated by
/// `GLAZEWM_PERF`.
fn record_apply(process: String, synchronous: bool, elapsed: Duration) {
  PROFILER.with(|profiler| {
    let Ok(mut profiler) = profiler.try_borrow_mut() else {
      return;
    };

    if let Some(sample) = profiler
      .apply_samples
      .iter_mut()
      .find(|s| s.synchronous == synchronous && s.process == process)
    {
      sample.record(elapsed);
    } else if profiler.apply_samples.len() < APPLY_SAMPLE_LIMIT {
      let mut sample = ApplySample {
        process,
        synchronous,
        ..ApplySample::default()
      };
      sample.record(elapsed);
      profiler.apply_samples.push(sample);
    } else {
      profiler.apply_overflow.record(elapsed);
    }
  });
}

/// Timestamps an event as it is pushed onto its listener's channel.
///
/// Called from the producing hook thread. Must be paired one-to-one with a
/// [`record_event_dequeued`] of the same kind on the consuming thread; the
/// queues are FIFO, so the pairing recovers each event's own wait without
/// having to widen the channel's item type.
pub fn mark_event_queued(kind: EventKind) {
  if is_enabled() {
    queue_event(kind);
  }
}

/// [`mark_event_queued`] without the `GLAZEWM_PERF` gate.
fn queue_event(kind: EventKind) {
  let index = kind.index();
  if let Ok(mut queue) = EVENT_QUEUES[index].lock() {
    if queue.len() >= EVENT_QUEUE_LIMIT {
      queue.pop_front();
      EVENT_QUEUE_OVERFLOW[index].fetch_add(1, Ordering::Relaxed);
    }
    queue.push_back(Instant::now());
  }
}

/// Records how long the event just taken off `kind`'s channel spent queued.
///
/// Called from the WM's main loop, immediately after the event is received.
/// This is the only way to see whether the main loop's `biased` select is
/// starving input while animation ticks saturate the thread: our own
/// handling time shows up in the stage tree, but time an event spends
/// *waiting to be looked at* does not.
pub fn record_event_dequeued(kind: EventKind) {
  if is_enabled() {
    dequeue_event(kind);
  }
}

/// [`record_event_dequeued`] without the `GLAZEWM_PERF` gate.
fn dequeue_event(kind: EventKind) {
  let index = kind.index();
  let queued_at = EVENT_QUEUES[index]
    .lock()
    .ok()
    .and_then(|mut queue| queue.pop_front());

  PROFILER.with(|profiler| {
    if let Ok(mut profiler) = profiler.try_borrow_mut() {
      let wait = &mut profiler.event_wait[index];
      match queued_at {
        Some(queued_at) => {
          let elapsed = queued_at.elapsed();
          wait.count += 1;
          wait.total += elapsed;
          wait.worst = wait.worst.max(elapsed);
        }
        None => wait.unpaired += 1,
      }
    }
  });
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
    let budget = frame_budget();
    profiler.budget = budget;
    if profiler.frame_total[Stage::Tick.index()] > budget {
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
    "perf [{reason}]: {} frames in {:.1}ms, {} slow (>{:.1}ms), peak {} \
     window(s) animating",
    summary.frames,
    elapsed.as_secs_f64() * 1000.0,
    summary.slow_frames,
    summary.budget.as_secs_f64() * 1000.0,
    summary.peak_windows,
  );
  let _ = writeln!(
    lines,
    "  {:<20}{:>7}{:>11}{:>11}{:>11}",
    "stage", "calls", "total", "per-frame", "worst",
  );

  // Nesting tree first: each stage's time includes the stages indented
  // under it, so a parent is never the sum of its children.
  fn row(
    lines: &mut String,
    summary: &Profiler,
    frames: f64,
    stage: Stage,
    indent: usize,
  ) {
    let index = stage.index();
    if summary.calls[index] == 0 {
      return;
    }
    let name = format!("{:indent$}{}", "", stage.label(), indent = indent);
    // Writing into a `String` is infallible, so the result is discarded.
    let _ = writeln!(
      lines,
      "  {:<20}{:>7}{:>9.1}ms{:>9.2}ms{:>9.2}ms",
      name,
      summary.calls[index],
      summary.total[index].as_secs_f64() * 1000.0,
      summary.total[index].as_secs_f64() * 1000.0 / frames,
      summary.worst_frame[index].as_secs_f64() * 1000.0,
    );
  }

  for stage in Stage::ALL {
    if !stage.is_cross_cutting() {
      row(&mut lines, &summary, frames, stage, stage.depth() * 2);
    }
  }

  // Cross-cutting stages are called from several parents, so they get their
  // own section rather than a misleading indent. Their time is already
  // counted inside the tree above.
  if Stage::ALL
    .into_iter()
    .any(|s| s.is_cross_cutting() && summary.calls[s.index()] > 0)
  {
    let _ = writeln!(lines, "  -- called from several parents, already counted above --");
    for stage in Stage::ALL {
      if stage.is_cross_cutting() {
        row(&mut lines, &summary, frames, stage, 2);
      }
    }
  }

  write_apply_breakdown(&mut lines, &summary);
  write_event_waits(&mut lines, &summary);

  Some(lines.trim_end().to_string())
}

/// Appends the per-process `rd_apply` breakdown to the report.
///
/// A no-op when no reposition was attributed during the session.
fn write_apply_breakdown(lines: &mut String, summary: &Profiler) {
  // Which windows the `rd_apply` time actually went to. A synchronous
  // reposition blocks the WM thread on the target application's message
  // pump, so a single slow app can dominate the frame; the stage tree alone
  // cannot show that.
  if !summary.apply_samples.is_empty() {
    let _ = writeln!(
      lines,
      "  -- rd_apply by process (sync = blocked on that app's message pump)        --"
    );
    let _ = writeln!(
      lines,
      "  {:<20}{:>7}{:>11}{:>11}{:>11}",
      "process", "calls", "total", "per-call", "worst",
    );

    let mut samples = summary.apply_samples.iter().collect::<Vec<_>>();
    samples.sort_by_key(|sample| Reverse(sample.total));

    for sample in samples {
      apply_row(
        lines,
        &format!(
          "{} [{}]",
          sample.process,
          if sample.synchronous { "sync" } else { "async" }
        ),
        sample,
      );
    }

    if summary.apply_overflow.calls > 0 {
      apply_row(lines, "(other processes)", &summary.apply_overflow);
    }
  }
}

/// Appends the per-kind event queue-wait section to the report.
///
/// A no-op when no event was dequeued during the session.
fn write_event_waits(lines: &mut String, summary: &Profiler) {
  // How long events sat in their channel before the main loop looked at
  // them. Our handling time is already in the tree above; this is the part
  // that is invisible there, and the only direct evidence of whether the
  // `biased` select starves input during an animation.
  let event_rows = EventKind::ALL
    .into_iter()
    .filter(|kind| {
      let wait = summary.event_wait[kind.index()];
      wait.count > 0 || wait.unpaired > 0
    })
    .collect::<Vec<_>>();

  if !event_rows.is_empty() {
    let _ =
      writeln!(lines, "  -- event queue wait (main-loop starvation) --");
    let _ = writeln!(
      lines,
      "  {:<20}{:>7}{:>11}{:>11}{:>11}",
      "event", "count", "total", "mean", "worst",
    );

    for kind in event_rows {
      let wait = summary.event_wait[kind.index()];
      let dropped = EVENT_QUEUE_OVERFLOW[kind.index()].swap(0, Ordering::Relaxed);
      let mean = wait
        .total
        .checked_div(wait.count.max(1))
        .unwrap_or(Duration::ZERO);

      // Writing into a `String` is infallible, so the result is discarded.
      let _ = writeln!(
        lines,
        "  {:<20}{:>7}{:>9.1}ms{:>9.2}ms{:>9.2}ms{}",
        kind.label(),
        wait.count,
        wait.total.as_secs_f64() * 1000.0,
        mean.as_secs_f64() * 1000.0,
        wait.worst.as_secs_f64() * 1000.0,
        suspect_suffix(wait.unpaired, dropped),
      );
    }
  }
}

/// Writes one row of the `rd_apply` breakdown.
///
/// Averages over calls rather than frames: a reposition either happens for a
/// given window this frame or it does not, so a per-frame average of a
/// per-window cost would be meaningless.
fn apply_row(lines: &mut String, name: &str, sample: &ApplySample) {
  let per_call = sample
    .total
    .checked_div(sample.calls.max(1))
    .unwrap_or(Duration::ZERO);

  // Writing into a `String` is infallible, so the result is discarded.
  let _ = writeln!(
    lines,
    "  {:<20}{:>7}{:>9.1}ms{:>9.2}ms{:>9.2}ms",
    name,
    sample.calls,
    sample.total.as_secs_f64() * 1000.0,
    per_call.as_secs_f64() * 1000.0,
    sample.worst.as_secs_f64() * 1000.0,
  );
}

/// Returns a trailing warning for an event row whose timestamp pairing
/// slipped, or an empty string when the row is trustworthy.
///
/// Kept explicit in the report because a silently-skewed latency number is
/// worse than no number at all.
fn suspect_suffix(unpaired: u32, dropped: u32) -> String {
  match (unpaired, dropped) {
    (0, 0) => String::new(),
    (unpaired, 0) => format!("  ({unpaired} unpaired -- SUSPECT)"),
    (0, dropped) => format!("  ({dropped} dropped -- SUSPECT)"),
    (unpaired, dropped) => {
      format!("  ({unpaired} unpaired, {dropped} dropped -- SUSPECT)")
    }
  }
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
    drop(apply_scope(|| unreachable!("label built while disabled"), true));
    note_window_count(4);
    mark_event_queued(EventKind::Window);
    record_event_dequeued(EventKind::Window);
    end_frame();
    report("test");

    PROFILER.with(|profiler| {
      let profiler = profiler.borrow();
      assert_eq!(profiler.frames, 0);
      assert!(profiler.apply_samples.is_empty());
      assert_eq!(profiler.event_wait[EventKind::Window.index()].count, 0);
    });
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

  #[test]
  fn attributes_repositions_per_process_and_sync_flag() {
    std::thread::spawn(|| {
      start_frame();
      record_apply("explorer.exe".to_string(), true, ms(10));
      record_apply("explorer.exe".to_string(), true, ms(30));
      // Same process, different call flavour -- must not merge with the
      // synchronous row, since only that one blocks the WM thread.
      record_apply("explorer.exe".to_string(), false, ms(1));
      record_apply("outlook.exe".to_string(), true, ms(5));
      assert!(!roll_up_frame());

      PROFILER.with(|profiler| {
        let profiler = profiler.borrow();
        // Three rows, not two: the async explorer call is kept apart from
        // the synchronous ones, which are the only blocking kind.
        assert_eq!(profiler.apply_samples.len(), 3);

        let sync_explorer = profiler
          .apply_samples
          .iter()
          .find(|s| s.process == "explorer.exe" && s.synchronous)
          .expect("synchronous explorer row");
        assert_eq!(sync_explorer.calls, 2);
        assert_eq!(sync_explorer.total, ms(40));
        assert_eq!(sync_explorer.worst, ms(30));
      });

      let report = take_report("unit test").expect("frames were recorded");
      assert!(report.contains("rd_apply by process"));
      // Sorted by total, so the 40ms synchronous explorer row leads.
      let first_row = report
        .lines()
        .find(|line| line.contains(".exe ["))
        .expect("at least one attributed row");
      assert!(first_row.contains("explorer.exe [sync]"), "{first_row}");
      assert!(report.contains("explorer.exe [async]"));
      assert!(report.contains("outlook.exe [sync]"));
    })
    .join()
    .expect("profiler test thread panicked");
  }

  #[test]
  fn collapses_repositions_past_the_sample_limit() {
    std::thread::spawn(|| {
      start_frame();
      for index in 0..=APPLY_SAMPLE_LIMIT {
        record_apply(format!("app{index}.exe"), true, ms(1));
      }
      assert!(!roll_up_frame());

      PROFILER.with(|profiler| {
        let profiler = profiler.borrow();
        assert_eq!(profiler.apply_samples.len(), APPLY_SAMPLE_LIMIT);
        assert_eq!(profiler.apply_overflow.calls, 1);
      });

      let report = take_report("unit test").expect("frames were recorded");
      assert!(report.contains("(other processes)"));
    })
    .join()
    .expect("profiler test thread panicked");
  }

  #[test]
  fn pairs_event_queue_timestamps_fifo() {
    // Uses `Display` events, which no other test touches, because the
    // enqueue queues are process-global rather than thread-local.
    std::thread::spawn(|| {
      start_frame();
      queue_event(EventKind::Display);
      queue_event(EventKind::Display);
      dequeue_event(EventKind::Display);
      dequeue_event(EventKind::Display);
      // A third dequeue has nothing to pair with and must be flagged rather
      // than silently reported as a zero-length wait.
      dequeue_event(EventKind::Display);
      assert!(!roll_up_frame());

      PROFILER.with(|profiler| {
        let wait = profiler.borrow().event_wait[EventKind::Display.index()];
        assert_eq!(wait.count, 2);
        assert_eq!(wait.unpaired, 1);
      });

      let report = take_report("unit test").expect("frames were recorded");
      assert!(report.contains("event queue wait"));
      assert!(report.contains("1 unpaired -- SUSPECT"), "{report}");
    })
    .join()
    .expect("profiler test thread panicked");
  }

  /// Shorthand for a whole number of milliseconds.
  fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
  }
}
