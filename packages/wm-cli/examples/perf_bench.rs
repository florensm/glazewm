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
//! # Example usage
//!
//! ```text
//! cargo run -p wm-cli --release --example perf_bench -- \
//!   --scenario resize --target chrome --bursts 10 --label baseline
//! ```

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
/// `wm-*` commands, which take no subject.
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
];

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
  ("interval:p50", "interval p50 ms"),
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = parse_args()?;
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
  let window = find_tiled_window(&mut client, &args.target).await?;
  let target = if args.scenario == WORKSPACE_SCENARIO {
    find_workspace_of(&mut client, window).await?
  } else {
    window
  };

  let start_len = std::fs::metadata(&log_path).map_or(0, |m| m.len());
  let started = Instant::now();

  for burst in 0..args.bursts {
    let step = if burst % 2 == 0 { forward } else { back };
    send_step(&mut client, target, step).await?;
    tokio::time::sleep(args.settle).await;
  }

  // Leave the layout as it was found.
  if args.bursts % 2 == 1 {
    send_step(&mut client, target, back).await?;
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

  if reports.is_empty() {
    bail!(
      "No profiler reports in {}. Is the WM running with GLAZEWM_PERF=1?",
      log_path.display()
    );
  }

  print_summary(&args, &reports, started.elapsed());
  Ok(())
}

fn parse_args() -> anyhow::Result<Args> {
  let mut args = Args {
    scenario: "resize".into(),
    target: String::new(),
    bursts: 10,
    settle: Duration::from_millis(1200),
    label: "run".into(),
  };

  let mut iter = std::env::args().skip(1);
  while let Some(flag) = iter.next() {
    let value = iter
      .next()
      .with_context(|| format!("Missing value for '{flag}'."))?;
    match flag.as_str() {
      "--scenario" => args.scenario = value,
      "--target" => args.target = value,
      "--bursts" => args.bursts = value.parse()?,
      "--settle-ms" => args.settle = Duration::from_millis(value.parse()?),
      "--label" => args.label = value,
      _ => bail!("Unknown flag '{flag}'."),
    }
  }

  if args.target.is_empty() {
    bail!("--target <process name> is required, e.g. --target chrome.");
  }

  Ok(args)
}

fn perf_log_path() -> anyhow::Result<PathBuf> {
  Ok(
    home::home_dir()
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
  fn contains(children: &[ContainerDto], window: Uuid) -> bool {
    children.iter().any(|child| match child {
      ContainerDto::Window(dto) => dto.id == window,
      ContainerDto::Split(split) => contains(&split.children, window),
      _ => false,
    })
  }

  let message = "query workspaces";
  client.send(message).await?;
  let response = client
    .client_response(message)
    .await
    .context("No response to workspace query.")?;

  let Some(ClientResponseData::Workspaces(data)) = response.data else {
    bail!("Unexpected response to workspace query.");
  };

  data
    .workspaces
    .iter()
    .find_map(|container| match container {
      ContainerDto::Workspace(workspace)
        if contains(&workspace.children, window) =>
      {
        Some(workspace.id)
      }
      _ => None,
    })
    .context("Target window is on no workspace.")
}

/// Sends each `;`-separated command of a scenario step.
async fn send_step(
  client: &mut IpcClient,
  target: Uuid,
  step: &str,
) -> anyhow::Result<()> {
  for command in step.split(';').map(str::trim) {
    let message = if command.starts_with("wm-") {
      format!("command {command}")
    } else {
      format!("command --id {target} {command}")
    };
    send_command(client, &message).await?;
  }

  Ok(())
}

async fn send_command(
  client: &mut IpcClient,
  message: &str,
) -> anyhow::Result<()> {
  client.send(message).await?;
  let response = client
    .client_response(message)
    .await
    .context("No response to command.")?;

  if !response.success {
    bail!(
      "Command '{message}' failed: {}",
      response.error.unwrap_or_default()
    );
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

fn median(mut values: Vec<f64>) -> Option<f64> {
  if values.is_empty() {
    return None;
  }
  values.sort_by(f64::total_cmp);
  Some(values[values.len() / 2])
}

fn print_summary(
  args: &Args,
  reports: &[HashMap<String, f64>],
  elapsed: Duration,
) {
  println!(
    "label={} scenario={} target={} bursts={} reports={} wall={:.1}s",
    args.label,
    args.scenario,
    args.target,
    args.bursts,
    reports.len(),
    elapsed.as_secs_f64(),
  );

  let mut cells = vec![args.label.clone(), args.scenario.clone()];
  for (key, label) in METRICS {
    let value = median(
      reports
        .iter()
        .map(|report| report.get(*key).copied().unwrap_or(0.0))
        .collect(),
    )
    .unwrap_or(0.0);
    println!("  {label:<36}{value:>9.2}");
    cells.push(format!("{value:.2}"));
  }

  println!("| {} |", cells.join(" | "));
}
