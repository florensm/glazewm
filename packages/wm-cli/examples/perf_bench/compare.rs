//! Side-by-side tables of several builds' runs, from the summary rows
//! `compare_branches.ps1` collects.
//!
//! A row's label is `<build> <config> #<round>`; rows are grouped by
//! scenario and config, and each build's rounds summarised as a median,
//! one column per build.

use std::{fmt::Write as _, io::Write as _, path::Path};

use anyhow::{bail, Context};

/// A summary row's cell for a metric no report had.
pub const ABSENT_CELL: &str = "-";

/// A table cell for a metric no run of a build had: a stage that build
/// lacks, or one that never ran.
const ABSENT: &str = "\u{2014}";

/// Metrics of the per-scenario table, those of the header that has them.
const HEADLINE: &[&str] = &[
  "input->first frame ms",
  "frame p90 ms",
  "interval p90 ms",
  "interval p99 ms",
  "peek_new ms (outside frames)",
  "session_begin ms (outside frames)",
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

/// A build's runs of one scenario and config: how many, and each column's
/// median over them.
struct Summary {
  runs: usize,
  medians: Vec<Option<f64>>,
}

/// Markdown tables of `rows` (a `summary-header` line and rows of
/// `print_summary`), one column per build of `builds`, in that order.
pub fn tables(rows: &str, builds: &[&str]) -> anyhow::Result<String> {
  if builds.is_empty() {
    bail!("No builds to compare.");
  }

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

  let headline = HEADLINE
    .iter()
    .filter_map(|name| {
      columns
        .iter()
        .position(|column| column == name)
        .map(|i| (*name, i))
    })
    .collect::<Vec<_>>();

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
        let of_build = |build: &&str| {
          summarise(runs.iter().filter(|run| {
            run.scenario == scenario
              && run.config == *config
              && run.build == *build
          }))
        };
        (*config, builds.iter().map(of_build).collect::<Vec<_>>())
      })
      .collect::<Vec<_>>();

    // Writing into a `String` is infallible, so the results are discarded.
    let _ = write!(
      text,
      "\n## {scenario}\n\nMedians over rounds; {ABSENT}: in no report of \
       that build (no such stage, or it never ran).\n\n"
    );
    write_header(&mut text, &["config", "metric"], builds);
    for (config, summaries) in &summaries {
      write_row(&mut text, &[config, "runs"], summaries, |summary| {
        summary.runs.to_string()
      });
      for (name, index) in &headline {
        write_row(&mut text, &[config, name], summaries, |summary| {
          cell(&summary.medians, *index)
        });
      }
    }

    for (config, summaries) in &summaries {
      let _ = write!(text, "\n### {scenario}, {config}\n\n");
      write_header(&mut text, &["metric"], builds);
      write_row(&mut text, &["runs"], summaries, |summary| {
        summary.runs.to_string()
      });
      for (index, name) in columns.iter().enumerate() {
        write_row(&mut text, &[name], summaries, |summary| {
          cell(&summary.medians, index)
        });
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

/// Writes a table's header: the `lead` columns, then one per build.
fn write_header(text: &mut String, lead: &[&str], builds: &[&str]) {
  let names = lead.iter().chain(builds).copied().collect::<Vec<_>>();
  let _ = writeln!(text, "| {} |", names.join(" | "));
  let _ = writeln!(text, "|{}", " --- |".repeat(names.len()));
}

/// Writes a table row: the `lead` cells, then `cell` of each build's
/// summary.
fn write_row(
  text: &mut String,
  lead: &[&str],
  summaries: &[Summary],
  cell: impl Fn(&Summary) -> String,
) {
  let cells = lead
    .iter()
    .map(ToString::to_string)
    .chain(summaries.iter().map(cell))
    .collect::<Vec<_>>();
  let _ = writeln!(text, "| {} |", cells.join(" | "));
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
fn summarise<'a>(runs: impl Iterator<Item = &'a Run<'a>>) -> Summary {
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

  Summary {
    runs: runs.len(),
    medians,
  }
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
| stand-ins:nopeeks cpu0 bd-on bo-on #1 | move | 6.00 | - |
| this cpu0 bd-on bo-on #2 | move | 4.00 | - |
| main cpu0 bd-on bo-on #2 | move | 5.00 | - |
| stand-ins:nopeeks cpu0 bd-on bo-on #2 | move | 8.00 | - |
";

  /// Each build's rounds are summarised in a column of its own, in the
  /// order given; a stage one build never reported shows as absent, not
  /// as 0.
  #[test]
  fn sets_builds_side_by_side() {
    let text =
      tables(ROWS, &["this", "stand-ins:nopeeks", "main"]).unwrap();

    assert!(text.contains("## move"));
    assert!(text.contains(
      "| config | metric | this | stand-ins:nopeeks | main |\n\
       | --- | --- | --- | --- | --- |\n\
       | cpu0 bd-on bo-on | runs | 2 | 2 | 2 |\n\
       | cpu0 bd-on bo-on | frame p90 ms | 4.00 | 8.00 | 5.00 |\n"
    ));
    assert!(text.contains(&format!(
      "| restack ms (outside frames) | 1.00 | {ABSENT} | {ABSENT} |"
    )));
  }

  /// A build with no runs gets a column of absent cells, not an error.
  #[test]
  fn shows_a_build_without_runs_as_absent() {
    let text = tables(ROWS, &["this", "stand-ins"]).unwrap();

    assert!(text.contains("| runs | 2 | 0 |"));
    assert!(text.contains(&format!("| frame p90 ms | 4.00 | {ABSENT} |")));
  }

  #[test]
  fn needs_a_build() {
    assert!(tables(ROWS, &[]).is_err());
  }

  #[test]
  fn reads_rows_after_a_bom() {
    let text =
      tables(&format!("\u{feff}{ROWS}"), &["this", "main"]).unwrap();

    assert!(text.contains("## move"));
  }

  #[test]
  fn splits_labels() {
    assert_eq!(
      split_label("this cpu8 bd-off bo-on #3"),
      Some(("this", "cpu8 bd-off bo-on".to_string()))
    );
    assert_eq!(
      split_label("stand-ins:morepeeks cpu0 #1"),
      Some(("stand-ins:morepeeks", "cpu0".to_string()))
    );
    assert_eq!(split_label("main #1"), Some(("main", String::new())));
  }
}
