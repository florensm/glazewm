//! Opt-in log of windows shown without their companion over them, e.g.
//! shades' recolored copy, which is seen as a flicker.
//!
//! Enabled by setting `GLAZEWM_FLICKER_LOG` to anything but `0`. Writes to
//! `~/.glzr/glazewm/flicker.log`. It reads window state only, never the
//! screen: windows are sampled every few milliseconds for a short while
//! after each WM input or window event, and at a slow rate otherwise. A
//! flicker shorter than the sampling interval can go unseen, and the
//! durations it reports are bounded by the samples around them.

use std::{
  collections::{HashMap, HashSet, VecDeque},
  fmt::Write as _,
  sync::OnceLock,
  time::{Duration, Instant},
};

use wm_common::InvokeCommand;
use wm_platform::{CompanionCoverage, PlatformEvent, WindowEvent};

use crate::{
  layout::is_hidden_in_layout,
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// `tracing` target the log is emitted under, routed to its own file by
/// `setup_logging`.
pub const LOG_TARGET: &str = "glazewm::flicker";

/// Sampling interval for a while after an input or window event, about
/// half a 60 Hz frame.
const FAST_INTERVAL: Duration = Duration::from_millis(8);

/// How long sampling stays fast after an input or window event.
const FAST_WINDOW: Duration = Duration::from_millis(750);

/// Sampling interval otherwise, which only catches long exposures.
const IDLE_INTERVAL: Duration = Duration::from_millis(250);

/// How long an exposure lasts before it is logged as it goes on, rather
/// than only once it ends.
const PERSISTENT_AFTER: Duration = Duration::from_secs(3);

/// Inputs and window events kept to explain an exposure.
const TRIGGER_HISTORY: usize = 12;

/// How far before an exposure's first sample its triggers are listed.
const TRIGGER_LOOKBACK: Duration = Duration::from_millis(500);

/// Longest window title written to the log, in characters.
const MAX_TITLE_CHARS: usize = 40;

/// Longest list of WM commands written to the log, in characters.
const MAX_COMMANDS_CHARS: usize = 80;

/// Whether the log is on, resolved once from `GLAZEWM_FLICKER_LOG`.
pub fn is_enabled() -> bool {
  static ENABLED: OnceLock<bool> = OnceLock::new();

  *ENABLED.get_or_init(|| {
    std::env::var("GLAZEWM_FLICKER_LOG").is_ok_and(|value| value != "0")
  })
}

/// What is seen of a window instead of its companion over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Exposure {
  /// The window is on screen, its companion not over it as it should be.
  Uncolored(CompanionCoverage),
  /// The window is hidden, its companion still on screen.
  Ghost,
}

impl Exposure {
  /// What a window's state, as sampled, exposes, if anything.
  ///
  /// `has_had_companion` keeps a companion that disappears with its window
  /// shown from passing for a window that never had one (e.g. an app
  /// shades leaves alone).
  fn of(
    is_on_screen: bool,
    coverage: CompanionCoverage,
    has_had_companion: bool,
    is_covered_by_wm: bool,
  ) -> Option<Self> {
    if is_covered_by_wm || !has_had_companion {
      return None;
    }

    if is_on_screen {
      (coverage != CompanionCoverage::Covers)
        .then_some(Self::Uncolored(coverage))
    } else {
      coverage.is_on_screen().then_some(Self::Ghost)
    }
  }

  fn describe(self) -> &'static str {
    match self {
      Self::Uncolored(CompanionCoverage::NoCompanion) => {
        "uncolored (companion gone)"
      }
      Self::Uncolored(CompanionCoverage::Cloaked) => {
        "uncolored (companion cloaked)"
      }
      Self::Uncolored(CompanionCoverage::Below) => {
        "uncolored (companion below window)"
      }
      Self::Uncolored(CompanionCoverage::Misplaced) => {
        "uncolored (companion off window frame)"
      }
      Self::Uncolored(CompanionCoverage::Covers) => "uncolored",
      Self::Ghost => "ghost (companion of hidden window on screen)",
    }
  }
}

/// An input or window event the WM handled.
#[derive(Debug)]
struct Trigger {
  at: Instant,
  label: String,
  /// Times it repeated in a row, e.g. location changes of a drag.
  count: u32,
}

/// A window's exposure, from its first sample on.
#[derive(Debug)]
struct Episode {
  exposure: Exposure,
  /// The last sample before the first one that saw it, when the window
  /// was still fine (or not yet sampled).
  ok_before: Option<Instant>,
  first_seen: Instant,
  last_seen: Instant,
  samples: u32,
  /// The window and what led up to the exposure, taken at its start.
  context: String,
  is_reported_persistent: bool,
}

/// What [`Tracker::track`] found to log.
#[derive(Debug, PartialEq, Eq)]
enum Report {
  /// An exposure ended, or changed kind, at `ended`.
  Ended {
    exposure: Exposure,
    /// Bounds on how long it lasted, from the samples around it.
    min: Duration,
    max: Duration,
    samples: u32,
    is_persistent: bool,
    context: String,
  },
  /// An exposure is still going after [`PERSISTENT_AFTER`].
  Persistent { exposure: Exposure, context: String },
}

/// See the module docs.
#[derive(Debug, Default)]
pub struct FlickerLog {
  triggers: VecDeque<Trigger>,
  fast_until: Option<Instant>,
  /// Windows a companion was ever seen for.
  with_companion: HashSet<isize>,
  tracker: Tracker,
}

/// The exposures of each window, from sample to sample.
#[derive(Debug, Default)]
struct Tracker {
  last_sample: Option<Instant>,
  episodes: HashMap<isize, Episode>,
}

impl FlickerLog {
  /// A log if [`is_enabled`].
  pub fn new() -> Option<Self> {
    is_enabled().then(|| {
      tracing::info!(
        target: LOG_TARGET,
        "Flicker log on: sampling every {FAST_INTERVAL:?} for \
         {FAST_WINDOW:?} after each input or window event, every \
         {IDLE_INTERVAL:?} otherwise."
      );
      Self::default()
    })
  }

  /// Notes a platform event, if it is one that can expose a window. Mouse
  /// moves are left out, as there is one per pixel, and keybindings are
  /// noted as the commands they run (see `note_commands`).
  pub fn note_event(&mut self, event: &PlatformEvent) {
    let label = match event {
      PlatformEvent::Mouse(_) | PlatformEvent::Keybinding(_) => return,
      PlatformEvent::DisplaySettingsChanged => {
        "display settings changed".to_string()
      }
      PlatformEvent::Window(event) => window_event_label(event),
    };

    self.note(label, Instant::now());
  }

  /// Notes WM commands run, e.g. from a keybinding or IPC.
  pub fn note_commands(&mut self, commands: &[InvokeCommand]) {
    let label = commands
      .iter()
      .map(|command| format!("{command:?}"))
      .collect::<Vec<_>>()
      .join(", ")
      .chars()
      .take(MAX_COMMANDS_CHARS)
      .collect();

    self.note(label, Instant::now());
  }

  /// Notes a WM input or event as a possible cause of the exposures that
  /// follow, and samples fast for a while.
  pub fn note(&mut self, label: String, now: Instant) {
    self.fast_until = Some(now + FAST_WINDOW);

    if let Some(last) = self.triggers.back_mut() {
      if last.label == label {
        last.at = now;
        last.count += 1;
        return;
      }
    }

    if self.triggers.len() == TRIGGER_HISTORY {
      self.triggers.pop_front();
    }
    self.triggers.push_back(Trigger {
      at: now,
      label,
      count: 1,
    });
  }

  /// When the next sample is due.
  pub fn next_sample(&self) -> Instant {
    let Some(last) = self.tracker.last_sample else {
      return Instant::now();
    };

    // An open exposure is followed closely, so its end is timed.
    let is_fast = !self.tracker.episodes.is_empty()
      || self.fast_until.is_some_and(|until| last < until);

    last
      + if is_fast {
        FAST_INTERVAL
      } else {
        IDLE_INTERVAL
      }
  }

  /// Samples every managed window, if a sample is due, and logs the
  /// exposures that ended or went on too long.
  pub fn sample_if_due(&mut self, state: &WmState) {
    let now = Instant::now();
    if now < self.next_sample() {
      return;
    }

    // The overview draws windows itself while open.
    if state.overview.is_shown() {
      self.tracker.last_sample = Some(now);
      return;
    }

    let windows = state.windows();
    let sampled = windows
      .iter()
      .map(|window| window.native().hwnd().0)
      .collect::<HashSet<_>>();

    for window in &windows {
      let native = window.native();
      let hwnd = native.hwnd().0;
      let coverage = wm_platform::companion_coverage(&native);

      if coverage != CompanionCoverage::NoCompanion {
        self.with_companion.insert(hwnd);
      }

      let is_on_screen = native.is_visible()
        && !native.is_minimized()
        && !native.is_cloaked().unwrap_or(true);
      let is_covered_by_wm =
        state.animation_manager.stands_in_for(&window.id())
          || state.animation_manager.has_companion_cover(&window.id());

      let exposure = Exposure::of(
        is_on_screen,
        coverage,
        self.with_companion.contains(&hwnd),
        is_covered_by_wm,
      );

      let triggers = &self.triggers;
      let report = self
        .tracker
        .track(hwnd, exposure, now, || describe(window, triggers, now));
      if let Some(report) = report {
        log_report(&report);
      }
    }

    // Windows no longer managed end their exposures unseen.
    self
      .tracker
      .episodes
      .retain(|hwnd, _| sampled.contains(hwnd));
    self.with_companion.retain(|hwnd| sampled.contains(hwnd));
    self.tracker.last_sample = Some(now);
  }
}

impl Tracker {
  /// Updates the episode of `hwnd` with what a sample at `now` saw of it,
  /// returning what to log. `context` is called only when an exposure
  /// starts.
  fn track(
    &mut self,
    hwnd: isize,
    exposure: Option<Exposure>,
    now: Instant,
    context: impl FnOnce() -> String,
  ) -> Option<Report> {
    let ended = match (self.episodes.get_mut(&hwnd), exposure) {
      (Some(episode), Some(exposure)) if episode.exposure == exposure => {
        episode.last_seen = now;
        episode.samples += 1;

        let is_persistent =
          now.duration_since(episode.first_seen) >= PERSISTENT_AFTER;
        if is_persistent && !episode.is_reported_persistent {
          episode.is_reported_persistent = true;
          return Some(Report::Persistent {
            exposure,
            context: episode.context.clone(),
          });
        }
        return None;
      }
      (Some(_), _) => self.episodes.remove(&hwnd),
      (None, _) => None,
    };

    if let Some(exposure) = exposure {
      self.episodes.insert(
        hwnd,
        Episode {
          exposure,
          ok_before: self.last_sample,
          first_seen: now,
          last_seen: now,
          samples: 1,
          context: context(),
          is_reported_persistent: false,
        },
      );
    }

    ended.map(|episode| Report::Ended {
      exposure: episode.exposure,
      min: episode.last_seen.duration_since(episode.first_seen),
      max: now
        .duration_since(episode.ok_before.unwrap_or(episode.first_seen)),
      samples: episode.samples,
      is_persistent: episode.is_reported_persistent,
      context: episode.context,
    })
  }
}

/// The window, where the WM has it, and the triggers shortly before
/// `now`.
fn describe(
  window: &WindowContainer,
  triggers: &VecDeque<Trigger>,
  now: Instant,
) -> String {
  let native = window.native();
  let title = native
    .title()
    .chars()
    .take(MAX_TITLE_CHARS)
    .collect::<String>();

  let mut text = format!(
    "{} {:#x} \"{title}\", {:?}, {:?}",
    native.process_name().unwrap_or_default(),
    native.hwnd().0,
    window.state(),
    window.display_state(),
  );

  if let Some(stack) = window.parent().and_then(|p| p.as_stack().cloned())
  {
    let tabs = stack.windows();
    let index = tabs.iter().position(|tab| tab.id() == window.id());
    let _ = write!(
      text,
      ", stack tab {}/{}",
      index.map_or(0, |index| index + 1),
      tabs.len()
    );
  }

  if is_hidden_in_layout(window) {
    text.push_str(", hidden in layout");
  }

  let recent = triggers
    .iter()
    .rev()
    .filter(|trigger| now.duration_since(trigger.at) <= TRIGGER_LOOKBACK)
    .map(|trigger| {
      let ago = now.duration_since(trigger.at).as_millis();
      if trigger.count > 1 {
        format!("-{ago}ms {} (x{})", trigger.label, trigger.count)
      } else {
        format!("-{ago}ms {}", trigger.label)
      }
    })
    .collect::<Vec<_>>();

  if recent.is_empty() {
    text.push_str("; no WM event before it");
  } else {
    let _ = write!(text, "; after: {}", recent.join(", "));
  }

  text
}

fn window_event_label(event: &WindowEvent) -> String {
  let (name, window) = match event {
    WindowEvent::Focused { window, .. } => ("focused", window),
    WindowEvent::Hidden { window, .. } => ("hidden", window),
    WindowEvent::MovedOrResized { window, .. } => ("moved", window),
    WindowEvent::Minimized { window, .. } => ("minimized", window),
    WindowEvent::MinimizeEnded { window, .. } => ("restored", window),
    WindowEvent::Shown { window, .. } => ("shown", window),
    WindowEvent::TitleChanged { window, .. } => ("title changed", window),
    WindowEvent::AttentionRequested { window, .. } => {
      ("attention", window)
    }
    WindowEvent::Reparented { window, .. } => ("reparented", window),
    WindowEvent::ZOrderChanged { .. } => {
      return "z-order changed".to_string()
    }
    WindowEvent::Destroyed { .. } => return "destroyed".to_string(),
  };

  format!("{name} {:#x}", window.hwnd().0)
}

fn log_report(report: &Report) {
  match report {
    Report::Ended {
      exposure,
      min,
      max,
      samples,
      is_persistent,
      context,
    } => tracing::info!(
      target: LOG_TARGET,
      "{}{} for {}-{}ms ({samples} sample{}): {context}",
      if *is_persistent { "persistent " } else { "" },
      exposure.describe(),
      min.as_millis(),
      max.as_millis(),
      if *samples == 1 { "" } else { "s" },
    ),
    Report::Persistent { exposure, context } => tracing::info!(
      target: LOG_TARGET,
      "{} for over {PERSISTENT_AFTER:?}, still going: {context}",
      exposure.describe(),
    ),
  }
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use wm_platform::CompanionCoverage;

  use super::{
    Exposure, FlickerLog, Report, FAST_INTERVAL, IDLE_INTERVAL,
    PERSISTENT_AFTER, TRIGGER_HISTORY,
  };

  const UNCOLORED: Exposure =
    Exposure::Uncolored(CompanionCoverage::Below);

  fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
  }

  /// Feeds `log` one sample of window `1` at `start + at`.
  fn sample(
    log: &mut FlickerLog,
    start: Instant,
    at: u64,
    exposure: Option<Exposure>,
  ) -> Option<Report> {
    let now = start + ms(at);
    let report = log.tracker.track(1, exposure, now, || "ctx".to_string());
    log.tracker.last_sample = Some(now);
    report
  }

  #[test]
  fn exposure_needs_a_companion_once_seen_and_no_wm_cover() {
    let gone = CompanionCoverage::NoCompanion;
    assert_eq!(Exposure::of(true, gone, false, false), None);
    assert_eq!(
      Exposure::of(true, gone, true, false),
      Some(Exposure::Uncolored(gone))
    );
    assert_eq!(
      Exposure::of(true, CompanionCoverage::Below, true, true),
      None
    );
    assert_eq!(
      Exposure::of(true, CompanionCoverage::Covers, true, false),
      None
    );
  }

  #[test]
  fn hidden_window_exposes_only_a_companion_on_screen() {
    assert_eq!(
      Exposure::of(false, CompanionCoverage::Cloaked, true, false),
      None
    );
    assert_eq!(
      Exposure::of(false, CompanionCoverage::Covers, true, false),
      Some(Exposure::Ghost)
    );
  }

  #[test]
  fn ended_exposure_is_bounded_by_the_samples_around_it() {
    let mut log = FlickerLog::default();
    let start = Instant::now();

    assert_eq!(sample(&mut log, start, 0, None), None);
    assert_eq!(sample(&mut log, start, 8, Some(UNCOLORED)), None);
    assert_eq!(sample(&mut log, start, 16, Some(UNCOLORED)), None);

    assert_eq!(
      sample(&mut log, start, 24, None),
      Some(Report::Ended {
        exposure: UNCOLORED,
        min: ms(8),
        max: ms(24),
        samples: 2,
        is_persistent: false,
        context: "ctx".to_string(),
      })
    );
    assert!(log.tracker.episodes.is_empty());
  }

  #[test]
  fn changed_exposure_ends_the_previous_one() {
    let mut log = FlickerLog::default();
    let start = Instant::now();

    sample(&mut log, start, 0, Some(UNCOLORED));
    let report = sample(&mut log, start, 8, Some(Exposure::Ghost));

    assert!(matches!(
      report,
      Some(Report::Ended {
        exposure: UNCOLORED,
        samples: 1,
        ..
      })
    ));
    assert_eq!(log.tracker.episodes[&1].exposure, Exposure::Ghost);
  }

  #[test]
  fn long_exposure_is_reported_once_while_it_goes_on() {
    let mut log = FlickerLog::default();
    let start = Instant::now();
    let late = u64::try_from(PERSISTENT_AFTER.as_millis()).unwrap_or(0);

    sample(&mut log, start, 0, Some(UNCOLORED));
    assert!(matches!(
      sample(&mut log, start, late, Some(UNCOLORED)),
      Some(Report::Persistent { .. })
    ));
    assert_eq!(sample(&mut log, start, late + 8, Some(UNCOLORED)), None);
    assert!(matches!(
      sample(&mut log, start, late + 16, None),
      Some(Report::Ended {
        is_persistent: true,
        ..
      })
    ));
  }

  #[test]
  fn sampling_is_fast_after_a_trigger_or_while_exposed() {
    let mut log = FlickerLog::default();
    let start = Instant::now();
    log.tracker.last_sample = Some(start);
    assert_eq!(log.next_sample(), start + IDLE_INTERVAL);

    log.note("focused 0x1".to_string(), start);
    assert_eq!(log.next_sample(), start + FAST_INTERVAL);

    let mut log = FlickerLog::default();
    sample(&mut log, start, 0, Some(UNCOLORED));
    assert_eq!(log.next_sample(), start + FAST_INTERVAL);
  }

  #[test]
  fn repeated_triggers_are_counted_and_history_is_bounded() {
    let mut log = FlickerLog::default();
    let start = Instant::now();

    log.note("moved 0x1".to_string(), start);
    log.note("moved 0x1".to_string(), start + ms(5));
    assert_eq!(log.triggers.len(), 1);
    assert_eq!(log.triggers[0].count, 2);

    for i in 0..TRIGGER_HISTORY * 2 {
      log.note(format!("focused {i:#x}"), start);
    }
    assert_eq!(log.triggers.len(), TRIGGER_HISTORY);
  }
}
