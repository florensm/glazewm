//! One line per workspace switch in `~/.glzr/glazewm/switch.log`, saying
//! how long it took and where the time went.
//!
//! Always on, unlike the profiler (`GLAZEWM_PERF`), which is too much for
//! everyday use: a switch costs a few clock reads and one line. Times are
//! from the key press, or from when the WM took up the command if it came
//! some other way.

use std::time::{Duration, Instant};

/// `tracing` target of the switch lines, which go to their own file.
pub const SWITCH_LOG_TARGET: &str = "glazewm::switch";

/// The switch being timed, if any, and the key press behind the command
/// being handled.
#[derive(Debug, Default)]
pub struct SwitchLog {
  trace: Option<SwitchTrace>,
  pressed: Option<Instant>,
}

#[derive(Debug)]
struct SwitchTrace {
  /// The configured style, or `off`.
  style: String,
  incoming: usize,
  outgoing: usize,
  /// The key press, else when the switch was taken up.
  origin: Instant,
  /// From the key press to when the switch was taken up.
  queued: Option<Duration>,
  /// Shell operations made before the switch, see `shell_calls`.
  shell_before: (u32, Duration),
  /// When the switch's own sync ended: the whole switch without an
  /// animation, else the setup before its first frame.
  synced: Option<Duration>,
  first_frame: Option<Duration>,
  /// When the animation ended, with the new workspace drawn in full.
  swapped: Option<Duration>,
  frames: u32,
  last_frame: Option<Instant>,
  worst_gap: Duration,
}

impl SwitchLog {
  /// Notes when the keys of the keybinding about to be handled were
  /// pressed, or clears it with `None` once handled.
  pub fn note_press(&mut self, pressed: Option<Instant>) {
    self.pressed = pressed;
  }

  /// Starts timing a switch, ending one still being timed as cut short.
  pub fn begin(
    &mut self,
    style: String,
    incoming: usize,
    outgoing: usize,
  ) {
    if self.trace.is_some() {
      self.finish("cut short");
    }

    let now = Instant::now();
    let origin = self.pressed.filter(|pressed| *pressed <= now);

    self.trace = Some(SwitchTrace {
      style,
      incoming,
      outgoing,
      origin: origin.unwrap_or(now),
      queued: origin.map(|pressed| now - pressed),
      shell_before: wm_platform::shell_calls(),
      synced: None,
      first_frame: None,
      swapped: None,
      frames: 0,
      last_frame: None,
      worst_gap: Duration::ZERO,
    });
  }

  /// Notes the end of a sync, the switch's own being the first; without
  /// an animation to follow, that ends it.
  pub fn note_synced(&mut self, is_animating: bool) {
    let Some(trace) = &mut self.trace else {
      return;
    };

    if trace.synced.is_none() {
      trace.synced = Some(trace.origin.elapsed());
      if !is_animating {
        self.finish("done");
      }
    }
  }

  /// Notes a frame of the switch's animation.
  pub fn note_frame(&mut self) {
    let Some(trace) = &mut self.trace else {
      return;
    };

    let now = Instant::now();
    trace.first_frame.get_or_insert(now - trace.origin);
    if let Some(last) = trace.last_frame {
      trace.worst_gap = trace.worst_gap.max(now - last);
    }
    trace.last_frame = Some(now);
    trace.frames += 1;
  }

  /// Notes the end of the switch's animation.
  pub fn note_swapped(&mut self) {
    if let Some(trace) = &mut self.trace {
      trace.swapped.get_or_insert(trace.origin.elapsed());
    }
  }

  /// Ends the switch once its real windows are back, writing its line.
  /// One handed back before the switch's animation ended was an earlier
  /// switch's, still being torn down.
  pub fn note_handed_back(&mut self) {
    if self
      .trace
      .as_ref()
      .is_some_and(|trace| trace.swapped.is_some())
    {
      self.finish("done");
    }
  }

  fn finish(&mut self, outcome: &str) {
    let Some(trace) = self.trace.take() else {
      return;
    };

    let (calls, total) = wm_platform::shell_calls();
    let shell_calls = calls.saturating_sub(trace.shell_before.0);
    let shell_time = total.saturating_sub(trace.shell_before.1);

    tracing::info!(
      target: SWITCH_LOG_TARGET,
      "{outcome}: style={} in={} out={} queued={} synced={} \
       first_frame={} swapped={} total={} explorer={shell_calls} \
       calls/{} frames={} worst_frame_gap={}",
      trace.style,
      trace.incoming,
      trace.outgoing,
      ms(trace.queued),
      ms(trace.synced),
      ms(trace.first_frame),
      ms(trace.swapped),
      ms(Some(trace.origin.elapsed())),
      ms(Some(shell_time)),
      trace.frames,
      ms((trace.frames > 1).then_some(trace.worst_gap)),
    );
  }
}

/// `duration` in milliseconds, or `-` without one.
fn ms(duration: Option<Duration>) -> String {
  duration.map_or_else(
    || "-".to_string(),
    |duration| format!("{:.1}ms", duration.as_secs_f64() * 1000.0),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A switch without an animation ends with its sync; one with an
  /// animation only once its own windows are handed back.
  #[test]
  fn ends_with_its_sync_or_its_own_hand_back() {
    let mut log = SwitchLog::default();

    log.begin("off".to_string(), 2, 1);
    log.note_synced(false);
    assert!(log.trace.is_none());

    log.begin("cut".to_string(), 2, 1);
    log.note_synced(true);
    log.note_handed_back();
    assert!(log.trace.is_some(), "an earlier switch's hand-back");

    log.note_frame();
    log.note_swapped();
    log.note_handed_back();
    assert!(log.trace.is_none());
  }

  /// A switch started while another is timed ends that one.
  #[test]
  fn new_switch_cuts_the_last_one_short() {
    let mut log = SwitchLog::default();
    log.note_press(Some(Instant::now()));

    log.begin("drift".to_string(), 1, 1);
    log.note_synced(true);
    log.begin("drift".to_string(), 1, 1);

    let trace = log.trace.as_ref().expect("new switch is timed");
    assert!(trace.synced.is_none());
    assert!(trace.queued.is_some());
  }
}
