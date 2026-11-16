//! Side-by-side tables of two builds' runs, from the summary rows
//! `compare_branches.ps1` collects.
//!
//! A row's label is `<build> <config> #<round>`; rows are grouped by
//! scenario and config, and each build's rounds summarised as a median.

use std::{fmt::Write as _, io::Write as _, path::Path};

use anyhow::{bail, Context};

/// A summary row's cell for a metric no report had.
pub const ABSENT_CELL: &str = "-";

/// A table cell for a metric no run of a build had: a stage that build
/// lacks, or one that never ran.
const ABSENT: &str = "\u{2014}";

/// Columns of the per-scenario table, those of the header that has them.
const HEADLINE: &[&str] = &[
  "input->first frame ms",
  "frame p90 ms",
  "interval p90 ms",
  "interval p99 ms",
  "peek_new ms (outside frames)",
  "restack ms (outside frames)",
  "dwm gpu busy p90 ms",
  "dwm missed vblanks/burst",
];

/// One summary row.
struct Run<'a> {
  build: &'a str,
  config: String,
  scenario: &'a str,
  cells: Vec<Option<f64>>,
}

/// Markdown tables of `rows` (a `summary-header` line and rows of
/// `print_summary`), build `this` beside build `other`.
pub fn tables(
  rows: &str,
  this: &str,
  other: &str,
) -> anyhow::Result<String> {
  let mut header = None;
  let mut runs = Vec::new();

  // Windows PowerShell's `-Encoding UTF8` starts a file with a BOM.
  let rows = rows.trim_start_matches('\u{feff}');
  for line in rows.lines().map(str::trim) {
    let Some(inner) = line
      .strip_prefix('|')
      .and_then(|line| line.strip_suffix('|'))
    else {
      continue;
    };
    let cells = inner.split('|').map(str::trim).collect::<Vec<_>>();

    match cells.as_slice() {
      ["label", ..] => header = Some(cells),
      [first, ..] if first.starts_with("---") => {}
      [label, scenario, values @ ..] => {
        let Some((build, config)) = split_label(label) else {
          continue;
        };
        runs.push(Run {
          build,
          config,
          scenario,
          cells: values
            .iter()
            .map(|cell| cell.parse::<f64>().ok())
            .collect(),
        });
      }
      _ => {}
    }
  }

  let header = header.context("The rows have no header line.")?;
  let Some(columns) = header.get(2..) else {
    bail!("The header has no metric columns.");
  };

  let mut text = String::new();
  for scenario in first_seen(runs.iter().map(|run| run.scenario)) {
    let configs = first_seen(
      runs
        .iter()
        .filter(|run| run.scenario == scenario)
        .map(|run| run.config.as_str()),
    );
    let summaries = configs
      .iter()
      .map(|config| {
        let of = |build: &str| {
          summarise(runs.iter().filter(|run| {
            run.scenario == scenario
              && run.config == *config
              && run.build == build
          }))
        };
        (*config, of(this), of(other))
      })
      .collect::<Vec<_>>();

    let headline = HEADLINE
      .iter()
      .filter_map(|name| {
        columns
          .iter()
          .position(|column| column == name)
          .map(|i| (*name, i))
      })
      .collect::<Vec<_>>();

    // Writing into a `String` is infallible, so the results are discarded.
    let _ = write!(
      text,
      "\n## {scenario}\n\nCells `{this} / {other}`, medians over rounds; \
       {ABSENT}: in no report of that build (no such stage, or it never \
       ran).\n\n| config | runs |"
    );
    for (name, _) in &headline {
      let _ = write!(text, " {name} |");
    }
    let _ = writeln!(text, "\n|{}", " --- |".repeat(headline.len() + 2));
    for (config, ours, theirs) in &summaries {
      let _ = write!(text, "| {config} | {} / {} |", ours.0, theirs.0);
      for (_, index) in &headline {
        let ours = cell(&ours.1, *index);
        let theirs = cell(&theirs.1, *index);
        let _ = write!(text, " {ours} / {theirs} |");
      }
      text.push('\n');
    }

    for (config, ours, theirs) in &summaries {
      let _ = write!(
        text,
        "\n### {scenario}, {config}\n\n| metric | {this} | {other} |\n\
         | --- | --- | --- |\n"
      );
      for (index, name) in columns.iter().enumerate() {
        let ours = cell(&ours.1, index);
        let theirs = cell(&theirs.1, index);
        let _ = writeln!(text, "| {name} | {ours} | {theirs} |");
      }
    }
  }

  Ok(text)
}

/// Appends `text` to the file at `path`, creating it if needed.
pub fn append(path: &Path, text: &str) -> anyhow::Result<()> {
  std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
    .and_then(|mut file| file.write_all(text.as_bytes()))
    .with_context(|| format!("Failed to write {}.", path.display()))
}

/// `(build, config)` of a `<build> <config> #<round>` label.
fn split_label(label: &str) -> Option<(&str, String)> {
  let mut words = label.split_whitespace().collect::<Vec<_>>();
  if words.last().is_some_and(|word| word.starts_with('#')) {
    words.pop();
  }
  let (build, config) = words.split_first()?;
  Some((build, config.join(" ")))
}

/// Distinct `values`, in the order first seen.
fn first_seen<'a>(values: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
  let mut seen = Vec::new();
  for value in values {
    if !seen.contains(&value) {
      seen.push(value);
    }
  }
  seen
}

/// The number of `runs`, and each column's median over them: a run
/// without a value counts as 0, unless none has one.
fn summarise<'a>(
  runs: impl Iterator<Item = &'a Run<'a>>,
) -> (usize, Vec<Option<f64>>) {
  let runs = runs.collect::<Vec<_>>();
  let width = runs.iter().map(|run| run.cells.len()).max().unwrap_or(0);

  let medians = (0..width)
    .map(|index| {
      let values = runs
        .iter()
        .map(|run| run.cells.get(index).copied().flatten())
        .collect::<Vec<_>>();
      if values.iter().all(Option::is_none) {
        return None;
      }
      let mut values = values
        .into_iter()
        .map(|value| value.unwrap_or(0.0))
        .collect::<Vec<_>>();
      values.sort_by(f64::total_cmp);
      Some(values[values.len() / 2])
    })
    .collect();

  (runs.len(), medians)
}

fn cell(values: &[Option<f64>], index: usize) -> String {
  values
    .get(index)
    .copied()
    .flatten()
    .map_or_else(|| ABSENT.to_string(), |value| format!("{value:.2}"))
}

#[cfg(test)]
mod tests {
  use super::*;

  const ROWS: &str = "\
| label | scenario | frame p90 ms | restack ms (outside frames) |
| --- | --- | --- | --- |
| this cpu0 bd-on bo-on #1 | move | 2.00 | 1.00 |
| main cpu0 bd-on bo-on #1 | move | 3.00 | - |
| this cpu0 bd-on bo-on #2 | move | 4.00 | - |
| main cpu0 bd-on bo-on #2 | move | 5.00 | - |
";

  /// Each build's rounds are summarised side by side; a stage one build
  /// never reported shows as absent, not as 0.
  #[test]
  fn sets_builds_side_by_side() {
    let text = tables(ROWS, "this", "main").unwrap();

    assert!(text.contains("## move"));
    assert!(text.contains("| cpu0 bd-on bo-on | 2 / 2 | 4.00 / 5.00 |"));
    assert!(text.contains(&format!(
      "| restack ms (outside frames) | 1.00 | {ABSENT} |"
    )));
  }

  #[test]
  fn reads_rows_after_a_bom() {
    let text = tables(&format!("\u{feff}{ROWS}"), "this", "main").unwrap();

    assert!(text.contains("## move"));
  }

  #[test]
  fn splits_labels() {
    assert_eq!(
      split_label("this cpu8 bd-off bo-on #3"),
      Some(("this", "cpu8 bd-off bo-on".to_string()))
    );
    assert_eq!(split_label("main #1"), Some(("main", String::new())));
  }
}
