//! Repeatable animation benchmark for the frame profiler.
//!
//! Drives a running WM (started with `GLAZEWM_PERF=1`) over IPC through a
//! fixed, reversible scenario, then summarises the profiler reports the
//! bursts produced in `~/.glzr/glazewm/perf.log` as medians.
//!
//! Every step goes back and forth (grow then shrink, left then right), so
//! the layout ends where it started and runs stay comparable. Triggers go
//! over one IPC connection rather than a CLI process per step, which would
//! add seconds of spawn and antivirus-scan time between bursts.
//!
//! With `--dwm`, also reports what DWM actually displayed per
//! burst, through the `PresentMon` service; see `dwm.rs`.
//!
//! # Example usage
//!
//! ```text
//! cargo run -p wm-cli --release --example perf_bench -- \
//!   --scenario resize --target chrome --bursts 10 --label baseline
//! ```
//!
//! The `switch` scenario also needs `--other-workspace <name>`, a second
//! workspace on the target's monitor. Every scenario reports how long each
//! command took to be answered, which with animations off is the whole
//! switch, as no profiler report is written without animation frames.
//!
//! `--load <n>` queues `n` IPC queries as each animation starts, so the
//! main loop has a backlog to catch up on between frames, as it would
//! with busy apps. It is what `GLAZEWM_ISOLATE=peeksfirst` is for.

mod bench_config;
mod dwm;
mod layout;

use std::{
  collections::HashMap,
  path::PathBuf,
  time::{Duration, Instant},
};

use anyhow::{bail, Context};
use uuid::Uuid;
use wm_common::{ClientResponseData, ContainerDto, WindowState};
use wm_ipc_client::IpcClient;

/// Reversible scenarios, each a pair of steps applied alternately.
///
/// A step is one or more `;`-separated commands. Commands target the
/// pinned window (its workspace for [`WORKSPACE_SCENARIO`]), except
/// `wm-*` commands and [`SWITCH_SCENARIO`]'s, which take no subject. In
/// the latter, `{home}` stands for the pinned window's workspace and
/// `{other}` for `--other-workspace`.
const SCENARIOS: &[(&str, &str, &str)] = &[
  // Every window on the workspace moves and resizes in both dimensions.
  // Toggling a workspace's direction queues no redraw on its own.
  (
    WORKSPACE_SCENARIO,
    "toggle-tiling-direction; wm-redraw",
    "toggle-tiling-direction; wm-redraw",
  ),
  // Target grows while its neighbour shrinks, then the reverse.
  ("resize", "resize --width +25%", "resize --width -25%"),
  // Target leaves the tiling layout so its neighbours grow, then returns
  // so they shrink -- the close/open layout change without spawning.
  ("float", "toggle-floating --centered", "toggle-tiling"),
  // Target swaps places with its neighbour, then swaps back.
  ("move", "move --direction left", "move --direction right"),
  // Target minimizes, then restores: the close/open animations without
  // spawning a window. Only animates with `window_minimize` on.
  ("minimize", "toggle-minimized", "toggle-minimized"),
  // The workspace switches away from the target's, then back.
  (
    SWITCH_SCENARIO,
    "focus --workspace {other}",
    "focus --workspace {home}",
  ),
];

/// The scenario switching between the pinned window's workspace and
/// `--other-workspace` on the same monitor.
const SWITCH_SCENARIO: &str = "switch";

/// The scenario whose commands target the pinned window's workspace.
///
/// Toggling a window's own direction only wraps it in a split container,
/// which changes nothing on screen.
const WORKSPACE_SCENARIO: &str = "relayout";

/// Report metrics summarised as medians, keyed by `section:stage:column`.
const METRICS: &[(&str, &str)] = &[
  ("frames", "frames"),
  ("tick:per_frame", "tick ms/frame"),
  ("dist:p90", "frame p90 ms"),
  ("dist:hitches_2x", "frames >2x budget"),
  ("interval:p50", "interval p50 ms"),
  ("interval:p90", "interval p90 ms"),
  ("interval:p99", "interval p99 ms"),
  // Like every column, a median over bursts, not the longest seen.
  ("interval:max", "interval max ms (median burst)"),
  ("tree:peek_frame:per_frame", "peek_frame ms/frame"),
  (
    "tree:session_overlays:per_frame",
    "session_overlays ms/frame",
  ),
  ("cross:batch_commit:per_frame", "batch_commit ms/frame"),
  ("cross:ovl_region:per_frame", "ovl_region ms/frame"),
  ("cross:dwm_flush:calls", "dwm_flush calls (in frames)"),
  ("outside:dwm_flush:total", "dwm_flush ms (outside frames)"),
  ("outside:cloak:total", "cloak ms (outside frames)"),
  (
    "outside:session_begin:total",
    "session_begin ms (outside frames)",
  ),
  ("latency:to anim start:p50", "input->anim start ms"),
  ("latency:to first frame:p50", "input->first frame ms"),
];

struct Args {
  scenario: String,
  target: String,
  bursts: usize,
  settle: Duration,
  label: String,
  /// Whether to capture DWM-side frame timing.
  dwm: bool,
  /// Workspace the switch scenario alternates with the target's.
  other_workspace: Option<String>,
  /// IPC queries queued as each animation starts.
  load: usize,
}

/// The query `--load` queues. It walks and serializes the whole tree, so
/// each costs the main loop a measurable slice of a frame.
const LOAD_QUERY: &str = "query workspaces";

/// Per-burst DWM summaries, as `(label, values)` for the summary table.
type DwmColumns = Vec<(&'static str, Vec<f64>)>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  if run_helper().await? {
    return Ok(());
  }

  let mut args = parse_args()?;
  let (_, forward, back) = SCENARIOS
    .iter()
    .find(|(name, ..)| *name == args.scenario)
    .with_context(|| {
      format!(
        "Unknown scenario '{}'. Options: {}.",
        args.scenario,
        SCENARIOS
          .iter()
          .map(|(name, ..)| *name)
          .collect::<Vec<_>>()
          .join(", ")
      )
    })?;

  let log_path = perf_log_path()?;
  let mut client = IpcClient::connect().await?;
  let (subject, forward, back) = match args.scenario.as_str() {
    SWITCH_SCENARIO => {
      let (home, other) =
        switch_workspaces(&mut client, args.other_workspace.as_deref())
          .await?;
      let fill = |step: &str| {
        step.replace("{home}", &home).replace("{other}", &other)
      };
      args.target = format!("{home}<->{other}");
      (None, fill(forward), fill(back))
    }
    WORKSPACE_SCENARIO => {
      let window = find_tiled_window(&mut client, &args.target).await?;
      let workspace = find_workspace_of(&mut client, window).await?;
      (Some(workspace), (*forward).to_string(), (*back).to_string())
    }
    _ => {
      let window = find_tiled_window(&mut client, &args.target).await?;
      (Some(window), (*forward).to_string(), (*back).to_string())
    }
  };

  let start_len = std::fs::metadata(&log_path).map_or(0, |m| m.len());
  let started = Instant::now();

  let mut capture = args.dwm.then(dwm::DwmCapture::start).transpose()?;
  let mut triggers = Vec::with_capacity(args.bursts + 1);
  let mut command_ms = Vec::with_capacity(args.bursts);

  for burst in 0..args.bursts {
    let step = if burst % 2 == 0 { &forward } else { &back };
    triggers.push(dwm::now_qpc());

    // The WM answers once the command and its sync have run, so this
    // spans everything done before the first animation frame: the whole
    // switch with animations off.
    let sent = Instant::now();
    send_step(&mut client, subject, step).await?;
    command_ms.push(sent.elapsed().as_secs_f64() * 1000.0);

    send_load(&mut client, args.load).await?;

    tokio::time::sleep(args.settle).await;

    // Drained every burst so the service's buffer never overflows.
    if let Some(capture) = capture.as_mut() {
      capture.drain()?;
    }
  }
  triggers.push(dwm::now_qpc());

  // Leave the layout as it was found.
  if args.bursts % 2 == 1 {
    send_step(&mut client, subject, &back).await?;
    tokio::time::sleep(args.settle).await;
  }

  let log = std::fs::read(&log_path)
    .with_context(|| format!("Failed to read {}.", log_path.display()))?;
  let new_text = String::from_utf8_lossy(
    log
      .get(usize::try_from(start_len).unwrap_or(usize::MAX)..)
      .unwrap_or_default(),
  );
  let reports = parse_reports(&strip_ansi(&new_text));

  // Expected with animations off, where no frame runs.
  if reports.is_empty() {
    println!(
      "No profiler reports in {}: no animation frame ran, or the WM isn't \
       running with GLAZEWM_PERF=1. Only command timings follow.",
      log_path.display()
    );
  }

  let dwm_columns = capture.map_or_else(DwmColumns::new, |capture| {
    dwm_columns(&capture, &triggers, &reports)
  });

  print_summary(
    &args,
    &command_ms,
    &reports,
    &dwm_columns,
    started.elapsed(),
  );
  Ok(())
}

/// Summarises DWM timing per burst, each burst spanning from its trigger
/// to the next.
fn dwm_columns(
  capture: &dwm::DwmCapture,
  triggers: &[u64],
  reports: &[HashMap<String, f64>],
) -> DwmColumns {
  // The WM ticks once per vblank, so its interval is the refresh period.
  let Some(period_ms) = median(
    reports
      .iter()
      .filter_map(|report| report.get("interval:p50").copied())
      .collect(),
  ) else {
    return DwmColumns::new();
  };

  let stats: Vec<dwm::BurstStats> = triggers
    .windows(2)
    .filter_map(|span| capture.burst_stats(span[0], span[1], period_ms))
    .collect();

  vec![
    (
      "dwm missed vblanks/burst",
      stats.iter().map(|s| s.missed_vblanks).collect(),
    ),
    (
      "dwm display interval p90 ms",
      stats.iter().map(|s| s.interval_p90_ms).collect(),
    ),
    (
      "dwm gpu busy p90 ms",
      stats.iter().map(|s| s.gpu_busy_p90_ms).collect(),
    ),
  ]
}

fn parse_args() -> anyhow::Result<Args> {
  let mut args = Args {
    scenario: "resize".into(),
    target: String::new(),
    bursts: 10,
    settle: Duration::from_millis(1200),
    label: "run".into(),
    dwm: false,
    other_workspace: None,
    load: 0,
  };

  let mut iter = std::env::args().skip(1);
  while let Some(flag) = iter.next() {
    if flag == "--dwm" {
      args.dwm = true;
      continue;
    }

    let value = iter
      .next()
      .with_context(|| format!("Missing value for '{flag}'."))?;
    match flag.as_str() {
      "--scenario" => args.scenario = value,
      "--target" => args.target = value,
      "--bursts" => args.bursts = value.parse()?,
      "--settle-ms" => args.settle = Duration::from_millis(value.parse()?),
      "--label" => args.label = value,
      "--other-workspace" => args.other_workspace = Some(value),
      "--load" => args.load = value.parse()?,
      _ => bail!("Unknown flag '{flag}'."),
    }
  }

  if args.target.is_empty() && args.scenario != SWITCH_SCENARIO {
    bail!("--target <process name> is required, e.g. --target chrome.");
  }

  Ok(args)
}

fn perf_log_path() -> anyhow::Result<PathBuf> {
  Ok(
    std::env::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/perf.log"),
  )
}

/// Returns the ID of a tiled window owned by `process`.
///
/// Pinning the target matters: a heavy browser and a light app cost very
/// differently to animate, so picking "any window" would compare
/// different workloads between runs.
async fn find_tiled_window(
  client: &mut IpcClient,
  process: &str,
) -> anyhow::Result<Uuid> {
  let message = "query windows";
  client.send(message).await?;
  let response = client
    .client_response(message)
    .await
    .context("No response to window query.")?;

  let Some(ClientResponseData::Windows(data)) = response.data else {
    bail!("Unexpected response to window query.");
  };

  data
    .windows
    .iter()
    .find_map(|container| match container {
      ContainerDto::Window(window)
        if window.state == WindowState::Tiling
          && window.process_name.eq_ignore_ascii_case(process) =>
      {
        Some(window.id)
      }
      _ => None,
    })
    .with_context(|| format!("No tiled window of process '{process}'."))
}

/// Returns the ID of the workspace containing `window`.
async fn find_workspace_of(
  client: &mut IpcClient,
  window: Uuid,
) -> anyhow::Result<Uuid> {
  layout::query_workspaces(client)
    .await?
    .iter()
    .find(|workspace| {
      layout::windows_in(&workspace.children)
        .iter()
        .any(|dto| dto.id == window)
    })
    .map(|workspace| workspace.id)
    .context("Target window is on no workspace.")
}

/// Names of the workspace the switch scenario starts on, the focused one,
/// and the one it switches to on the same monitor.
///
/// That is `other` if given, else one there that has windows. Failing
/// that, every second tiled window of the focused workspace is moved to
/// the next workspace to make one.
async fn switch_workspaces(
  client: &mut IpcClient,
  other: Option<&str>,
) -> anyhow::Result<(String, String)> {
  let workspaces = layout::query_workspaces(client).await?;
  let home = workspaces
    .iter()
    .find(|workspace| workspace.has_focus)
    .context("No workspace has focus.")?;

  if let Some(other) = other {
    if other == home.name {
      bail!("--other-workspace must differ from the focused one.");
    }
    return Ok((home.name.clone(), other.to_string()));
  }

  if let Some(other) = workspaces.iter().find(|workspace| {
    workspace.parent_id == home.parent_id
      && !workspace.is_displayed
      && !layout::windows_in(&workspace.children).is_empty()
  }) {
    return Ok((home.name.clone(), other.name.clone()));
  }

  let moved: Vec<Uuid> = layout::tiled_windows(home)
    .iter()
    .skip(1)
    .step_by(2)
    .map(|window| window.id)
    .collect();
  let Some(first_moved) = moved.first().copied() else {
    bail!(
      "Nothing to switch to: open two tiled windows on the focused \
       workspace, or pass --other-workspace."
    );
  };

  for id in &moved {
    let message = format!("command --id {id} move --next-workspace");
    layout::send_command(client, &message).await?;
  }

  let other = layout::query_workspaces(client)
    .await?
    .into_iter()
    .find(|workspace| {
      layout::windows_in(&workspace.children)
        .iter()
        .any(|window| window.id == first_moved)
    })
    .context("The moved windows are on no workspace.")?;

  println!(
    "Moved {} window(s) to workspace '{}' to switch to.",
    moved.len(),
    other.name
  );
  Ok((home.name.clone(), other.name))
}

/// Runs a helper the `*_modes.ps1` scripts call around the benchmark, if
/// the arguments name one, returning whether one ran:
///
/// - `save-layout <file>` / `restore-layout <file>`: see [`layout`].
/// - `write-switch-config <style> [config]`: prints the main file of a
///   copy of the config that switches workspaces in `style`.
/// - `clean-switch-config [config]`: removes those copies.
/// - `summary-header`: prints the header of the table rows
///   [`print_summary`] ends with, without `--dwm`'s columns.
async fn run_helper() -> anyhow::Result<bool> {
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  let arg = |index: usize| {
    args
      .get(index)
      .map(String::as_str)
      .with_context(|| format!("'{}' is missing an argument.", args[0]))
  };

  match args.first().map(String::as_str) {
    Some("save-layout") => {
      let mut client = IpcClient::connect().await?;
      layout::save(&mut client, std::path::Path::new(arg(1)?)).await?;
    }
    Some("restore-layout") => {
      let mut client = IpcClient::connect().await?;
      layout::restore(&mut client, std::path::Path::new(arg(1)?)).await?;
    }
    Some("write-switch-config") => {
      let user_main =
        bench_config::user_main_config(args.get(2).map(String::as_str))?;
      let path = bench_config::write(&user_main, arg(1)?)?;
      println!("{}", path.display());
    }
    Some("clean-switch-config") => {
      let user_main =
        bench_config::user_main_config(args.get(1).map(String::as_str))?;
      bench_config::clean(&user_main)?;
    }
    Some("summary-header") => print_summary_header(),
    _ => return Ok(false),
  }

  Ok(true)
}

/// Sends each `;`-separated command of a scenario step, on `subject`
/// unless it is `None` or the command is a `wm-*` one.
async fn send_step(
  client: &mut IpcClient,
  subject: Option<Uuid>,
  step: &str,
) -> anyhow::Result<()> {
  for command in step.split(';').map(str::trim) {
    let message = match subject {
      Some(subject) if !command.starts_with("wm-") => {
        format!("command --id {subject} {command}")
      }
      _ => format!("command {command}"),
    };
    layout::send_command(client, &message).await?;
  }

  Ok(())
}

/// Queues `count` [`LOAD_QUERY`]s at once, then waits for every answer.
///
/// All are sent before any answer is read, so they wait in the WM's queue
/// together rather than one at a time.
async fn send_load(
  client: &mut IpcClient,
  count: usize,
) -> anyhow::Result<()> {
  for _ in 0..count {
    client.send(LOAD_QUERY).await?;
  }

  for _ in 0..count {
    client
      .client_response(LOAD_QUERY)
      .await
      .context("No response to load query.")?;
  }

  Ok(())
}

/// Removes ANSI color escapes, which the log writer may emit.
fn strip_ansi(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut chars = text.chars();
  while let Some(c) = chars.next() {
    if c == '\u{1b}' {
      for c in chars.by_ref() {
        if c.is_ascii_alphabetic() {
          break;
        }
      }
    } else {
      out.push(c);
    }
  }
  out
}

/// Parses profiler reports into `section:stage:column` -> value maps.
///
/// Only reports for an animation burst are kept; the profiler's periodic
/// idle reports contain no frames worth comparing.
fn parse_reports(text: &str) -> Vec<HashMap<String, f64>> {
  let mut reports = Vec::new();
  let mut current: Option<HashMap<String, f64>> = None;
  let mut section = "tree";

  for line in text.lines() {
    if let Some(header) = line.split("perf [").nth(1) {
      reports.extend(current.take());
      let mut report = HashMap::new();
      if let Some(frames) = header
        .split("]: ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|frames| frames.parse().ok())
      {
        report.insert("frames".to_string(), frames);
      }
      current = Some(report);
      section = "tree";
      continue;
    }

    let Some(report) = current.as_mut() else {
      continue;
    };
    let trimmed = line.trim();

    if trimmed.starts_with("--") {
      section = if trimmed.contains("several parents") {
        "cross"
      } else if trimmed.contains("outside frames") {
        "outside"
      } else if trimmed.contains("frame time distribution") {
        "dist"
      } else if trimmed.contains("motion latency") {
        "latency"
      } else {
        "other"
      };
      continue;
    }

    let values = trimmed
      .split_whitespace()
      .filter_map(|token| token.trim_end_matches("ms").parse::<f64>().ok())
      .collect::<Vec<_>>();
    let name = trimmed
      .split_whitespace()
      .take_while(|token| {
        token.trim_end_matches("ms").parse::<f64>().is_err()
      })
      .collect::<Vec<_>>()
      .join(" ");

    match section {
      "tree" | "cross" if values.len() == 4 => {
        let key = if name == "tick" {
          "tick".to_string()
        } else {
          format!("{section}:{name}")
        };
        report.insert(format!("{key}:calls"), values[0]);
        report.insert(format!("{key}:total"), values[1]);
        report.insert(format!("{key}:per_frame"), values[2]);
      }
      "outside" if values.len() == 2 => {
        report.insert(format!("outside:{name}:total"), values[1]);
      }
      "dist" if name.is_empty() && values.len() == 4 => {
        report.insert("dist:p50".into(), values[0]);
        report.insert("dist:p90".into(), values[1]);
      }
      "dist" if name == "frame interval" && values.len() >= 4 => {
        report.insert("interval:p50".into(), values[0]);
        report.insert("interval:p90".into(), values[1]);
        report.insert("interval:p99".into(), values[2]);
        report.insert("interval:max".into(), values[3]);
      }
      "dist" if name == "hitches:" && values.len() == 3 => {
        report.insert("dist:hitches_2x".into(), values[0]);
      }
      "latency" if values.len() == 3 => {
        report.insert(format!("latency:{name}:p50"), values[1]);
      }
      _ => {}
    }
  }

  reports.extend(current);
  reports
    .into_iter()
    .filter(|report| report.contains_key("tick:per_frame"))
    .collect()
}

fn median(values: Vec<f64>) -> Option<f64> {
  percentile(values, 0.5)
}

/// The value at fraction `q` (0.0 to 1.0) of `values`, sorted.
#[allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]
fn percentile(mut values: Vec<f64>, q: f64) -> Option<f64> {
  if values.is_empty() {
    return None;
  }
  values.sort_by(f64::total_cmp);
  let index = ((values.len() - 1) as f64 * q).round() as usize;
  values.get(index).copied()
}

/// Column names of the row [`print_summary`] ends with, as a Markdown
/// table header.
fn print_summary_header() {
  let mut names =
    vec!["label", "scenario", "command p50 ms", "command p90 ms"];
  names.extend(METRICS.iter().map(|(_, label)| *label));
  println!("| {} |", names.join(" | "));
  println!("|{}", " --- |".repeat(names.len()));
}

fn print_summary(
  args: &Args,
  command_ms: &[f64],
  reports: &[HashMap<String, f64>],
  dwm_columns: &DwmColumns,
  elapsed: Duration,
) {
  println!(
    "label={} scenario={} target={} bursts={} load={} reports={} \
     wall={:.1}s",
    args.label,
    args.scenario,
    args.target,
    args.bursts,
    args.load,
    reports.len(),
    elapsed.as_secs_f64(),
  );

  let mut cells = vec![args.label.clone(), args.scenario.clone()];
  for (label, q) in [("command p50 ms", 0.5), ("command p90 ms", 0.9)] {
    let value = percentile(command_ms.to_vec(), q).unwrap_or(0.0);
    println!("  {label:<36}{value:>9.2}");
    cells.push(format!("{value:.2}"));
  }

  for (key, label) in METRICS {
    let value = median(
      reports
        .iter()
        .map(|report| report.get(*key).copied().unwrap_or(0.0))
        .collect(),
    );
    let cell =
      value.map_or_else(|| "-".to_string(), |v| format!("{v:.2}"));
    println!("  {label:<36}{cell:>9}");
    cells.push(cell);
  }

  for (label, values) in dwm_columns {
    let value = median(values.clone()).unwrap_or(0.0);
    println!("  {label:<36}{value:>9.2}");
    cells.push(format!("{value:.2}"));
  }

  println!("| {} |", cells.join(" | "));
}

#[cfg(test)]
mod tests {
  use super::parse_reports;

  #[test]
  fn reads_peek_frames_intervals_and_hitches() {
    let text = "\
perf [idle]: 30 frames in 500.0ms, 2 slow (>6.9ms)
  stage                 calls      total  per-frame      worst
  tick                     30     60.0ms     2.00ms     9.00ms
  peek_frame               30     45.0ms     1.50ms     4.00ms
  -- frame time distribution (budget 6.9ms) --
  frame time                  p50        p90        p99        max
                           1.80ms     3.10ms     8.50ms     9.00ms
  hitches: 3 frame(s) >2x budget, 1 >4x budget, out of 30
  frame interval           6.94ms     7.40ms    13.80ms    14.10ms  (~144 fps)
";
    let reports = parse_reports(text);
    assert_eq!(reports.len(), 1);
    let report = &reports[0];
    assert_eq!(report.get("frames"), Some(&30.0));
    assert_eq!(report.get("tree:peek_frame:per_frame"), Some(&1.5));
    assert_eq!(report.get("dist:p90"), Some(&3.1));
    assert_eq!(report.get("dist:hitches_2x"), Some(&3.0));
    assert_eq!(report.get("interval:p50"), Some(&6.94));
    assert_eq!(report.get("interval:p90"), Some(&7.4));
    assert_eq!(report.get("interval:p99"), Some(&13.8));
    assert_eq!(report.get("interval:max"), Some(&14.1));
  }
}
