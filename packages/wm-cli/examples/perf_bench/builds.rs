//! The builds `compare_branches.ps1` runs beside this checkout, read from
//! its `-Against` value.
//!
//! The value lists builds separated by `;`, each a git ref, optionally
//! followed by `:` and its `GLAZEWM_ISOLATE` modes, separated by `,`:
//! `origin/tryout/stand-ins:nopeeks,default,morepeeks;origin/main`. A mode
//! is `default` (the variable unset) or words joined by `+`, all set at
//! once (`morepeeks+peeksfirst`). Git refs can't hold `:`, `;` or spaces.

use anyhow::{bail, Context};

/// Label of this checkout's build, which no other build may take.
pub const THIS: &str = "this";

/// Mode that runs a build with `GLAZEWM_ISOLATE` unset.
const DEFAULT_MODE: &str = "default";

/// One build and mode to run.
#[derive(Debug, PartialEq, Eq)]
pub struct Build {
  /// Its column in the tables: the ref's last path segment, then
  /// `:<mode>` unless the mode is `default`.
  pub label: String,
  pub git_ref: String,
  /// `GLAZEWM_ISOLATE`'s value, or `None` to leave it unset.
  pub isolate: Option<String>,
}

/// The builds of `specs`, each a `-Against` value or part of one, in the
/// order given.
pub fn parse(specs: &[&str]) -> anyhow::Result<Vec<Build>> {
  let mut builds: Vec<Build> = Vec::new();

  for spec in specs
    .iter()
    .flat_map(|spec| spec.split(';'))
    .map(str::trim)
    .filter(|spec| !spec.is_empty())
  {
    let (git_ref, modes) = match spec.split_once(':') {
      Some((git_ref, modes)) => (git_ref.trim(), Some(modes)),
      None => (spec, None),
    };
    if git_ref.is_empty() || git_ref.contains(char::is_whitespace) {
      bail!("'{spec}' doesn't start with a git ref.");
    }

    let name = git_ref.rsplit('/').next().unwrap_or(git_ref);
    let modes = match modes {
      Some(modes) => modes
        .split(',')
        .map(|mode| parse_mode(mode.trim()))
        .collect::<anyhow::Result<Vec<_>>>()
        .with_context(|| format!("Invalid modes in '{spec}'."))?,
      None => vec![None],
    };

    for words in modes {
      let label = match &words {
        Some(words) => format!("{name}:{}", words.join("+")),
        None => name.to_string(),
      };
      if label.starts_with('#') || label.contains('|') {
        bail!("'{git_ref}' can't label a table column.");
      }
      if label == THIS || builds.iter().any(|build| build.label == label) {
        bail!(
          "Two builds would both be labelled '{label}'. List each ref and \
           mode once, and refs that share a last path segment by another \
           name (e.g. a commit hash)."
        );
      }

      builds.push(Build {
        label,
        git_ref: git_ref.to_string(),
        isolate: words.map(|words| words.join(",")),
      });
    }
  }

  if builds.is_empty() {
    bail!("No build to compare against.");
  }

  Ok(builds)
}

/// The `GLAZEWM_ISOLATE` words of a mode, lowercase, or `None` for
/// `default`.
fn parse_mode(mode: &str) -> anyhow::Result<Option<Vec<String>>> {
  if mode.eq_ignore_ascii_case(DEFAULT_MODE) {
    return Ok(None);
  }

  let words = mode
    .split('+')
    .map(|word| word.trim().to_ascii_lowercase())
    .collect::<Vec<_>>();
  if words.iter().any(|word| {
    word.is_empty() || !word.chars().all(|c| c.is_ascii_alphanumeric())
  }) {
    bail!(
      "A mode is '{DEFAULT_MODE}' or `GLAZEWM_ISOLATE` words joined by '+', \
       not '{mode}'."
    );
  }

  Ok(Some(words))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn build(label: &str, git_ref: &str, isolate: Option<&str>) -> Build {
    Build {
      label: label.to_string(),
      git_ref: git_ref.to_string(),
      isolate: isolate.map(ToString::to_string),
    }
  }

  #[test]
  fn reads_refs_with_and_without_modes() {
    assert_eq!(
      parse(&[
        "origin/tryout/stand-ins:nopeeks, Default ,morepeeks; origin/main"
      ])
      .unwrap(),
      vec![
        build(
          "stand-ins:nopeeks",
          "origin/tryout/stand-ins",
          Some("nopeeks")
        ),
        build("stand-ins", "origin/tryout/stand-ins", None),
        build(
          "stand-ins:morepeeks",
          "origin/tryout/stand-ins",
          Some("morepeeks")
        ),
        build("main", "origin/main", None),
      ]
    );
  }

  /// Builds may come as separate values too, as an in-process call passes
  /// an array.
  #[test]
  fn reads_several_values() {
    assert_eq!(
      parse(&["65a60de:nopeeks", "origin/main;"]).unwrap(),
      vec![
        build("65a60de:nopeeks", "65a60de", Some("nopeeks")),
        build("main", "origin/main", None),
      ]
    );
  }

  #[test]
  fn joins_words_of_one_mode() {
    assert_eq!(
      parse(&["stand-ins:MorePeeks+peeksfirst"]).unwrap(),
      vec![build(
        "stand-ins:morepeeks+peeksfirst",
        "stand-ins",
        Some("morepeeks,peeksfirst")
      )]
    );
  }

  #[test]
  fn refuses_labels_twice() {
    assert!(parse(&["origin/main;main"]).is_err());
    assert!(parse(&["x:nopeeks,nopeeks"]).is_err());
    assert!(parse(&["origin/this"]).is_err());
    assert!(parse(&["x:nopeeks;x:morepeeks"]).is_ok());
  }

  #[test]
  fn refuses_malformed_values() {
    assert!(parse(&[""]).is_err());
    assert!(parse(&[" ; "]).is_err());
    assert!(parse(&[":nopeeks"]).is_err());
    assert!(parse(&["x:"]).is_err());
    assert!(parse(&["x:nopeeks,,default"]).is_err());
    assert!(parse(&["x:no peeks"]).is_err());
    assert!(parse(&["x:nopeeks+"]).is_err());
    assert!(parse(&["x:nopeeks=1"]).is_err());
  }
}
