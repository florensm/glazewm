//! Copies of a config that differ only in a few settings: how workspaces
//! switch, for `switch_modes.ps1`, or whether borders and backdrops are
//! drawn, for `compare_branches.ps1`.
//!
//! Written next to the config's main file, so the relative paths it holds
//! still resolve, under a prefix only these files use. The original files
//! are only read.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use serde_yaml::{Mapping, Value};

/// Prefix of every file this module writes.
const PREFIX: &str = "glazewm-bench-";

/// The styles [`switch_overrides`] takes, with the `workspace_switch`
/// settings each sets. The subtle ones get the short duration they are
/// meant for.
const STYLES: &[(&str, &str)] = &[
  ("off", "enabled: false"),
  ("cut", "enabled: true\nstyle: cut"),
  ("drift", "enabled: true\nstyle: drift\nduration_ms: 150"),
  (
    "soft_zoom",
    "enabled: true\nstyle: soft_zoom\nduration_ms: 150",
  ),
  ("fade", "enabled: true\nstyle: fade"),
  ("zoom", "enabled: true\nstyle: zoom"),
  ("slide", "enabled: true\nstyle: slide"),
];

/// Effect sets every config has.
const EFFECT_SETS: &[&str] = &["focused_window", "other_windows"];

/// Effect set a config may have; overridden only where it is set, as
/// setting it would change which effects urgent windows take.
const URGENT_SET: &str = "urgent_window";

/// A setting a copy sets, wherever the original set it or not.
pub struct Override {
  /// Keys from the document's root.
  path: Vec<String>,
  value: Value,
}

impl Override {
  fn new(path: &[&str], value: Value) -> Self {
    Self {
      path: path.iter().map(ToString::to_string).collect(),
      value,
    }
  }
}

/// How a copy draws an effect: as the original config does, or forced on
/// or off for every window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
  Config,
  On,
  Off,
}

impl Effect {
  pub fn parse(value: &str) -> anyhow::Result<Self> {
    match value {
      "config" => Ok(Self::Config),
      "on" => Ok(Self::On),
      "off" => Ok(Self::Off),
      _ => bail!(
        "Unknown effect setting '{value}'. Options: config, on, off."
      ),
    }
  }
}

/// The config's main file: `explicit`, else `GLAZEWM_CONFIG_PATH`, else
/// the default, as the WM itself picks it.
pub fn user_main_config(
  explicit: Option<&str>,
) -> anyhow::Result<PathBuf> {
  if let Some(path) = explicit {
    return Ok(PathBuf::from(path));
  }

  if let Ok(path) = std::env::var("GLAZEWM_CONFIG_PATH") {
    return Ok(PathBuf::from(path));
  }

  Ok(
    std::env::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/config.yaml"),
  )
}

/// The override that switches workspaces as `style` says.
pub fn switch_overrides(style: &str) -> anyhow::Result<Vec<Override>> {
  let settings = STYLES
    .iter()
    .find(|(name, _)| *name == style)
    .map(|(_, settings)| *settings)
    .with_context(|| {
      format!(
        "Unknown style '{style}'. Options: {}.",
        STYLES
          .iter()
          .map(|(name, _)| *name)
          .collect::<Vec<_>>()
          .join(", ")
      )
    })?;

  Ok(vec![Override::new(
    &["animations", "workspace_switch"],
    serde_yaml::from_str(settings)?,
  )])
}

/// The overrides that draw `borders` and `backdrops` as asked, for the
/// config at `main`, with minimize animated so `perf_bench`'s `minimize`
/// scenario has frames to measure.
pub fn effect_overrides(
  main: &Path,
  borders: Effect,
  backdrops: Effect,
) -> anyhow::Result<Vec<Override>> {
  let has_urgent = documents(main)?.iter().any(|(_, document)| {
    lookup(document, &["window_effects", URGENT_SET]).is_some()
  });
  let sets = EFFECT_SETS
    .iter()
    .copied()
    .chain(has_urgent.then_some(URGENT_SET));

  let mut overrides = vec![Override::new(
    &["animations", "window_minimize", "enabled"],
    Value::Bool(true),
  )];
  for set in sets {
    for (effect, setting) in [("border", borders), ("backdrop", backdrops)]
    {
      let enabled = match setting {
        Effect::Config => continue,
        Effect::On => true,
        Effect::Off => false,
      };
      overrides.push(Override::new(
        &["window_effects", set, effect, "enabled"],
        Value::Bool(enabled),
      ));
    }
  }

  Ok(overrides)
}

/// Writes a copy of the config at `main` with `overrides` set, its files
/// named after `tag`, and returns the path of its main file.
///
/// A setting set in two files is an error, so each overridden setting is
/// removed from every file that sets it, which is then copied, and set in
/// the main file's copy. A config without `include` is one file.
pub fn write(
  main: &Path,
  tag: &str,
  overrides: &[Override],
) -> anyhow::Result<PathBuf> {
  if tag.is_empty() || !tag.chars().all(|c| c.is_ascii_alphanumeric()) {
    bail!("A config copy's tag must be letters and digits, not '{tag}'.");
  }

  let dir = main
    .parent()
    .context("Config path has no parent directory.")?;
  let mut documents = documents(main)?.into_iter();
  let (_, mut main_document) =
    documents.next().context("No main config file.")?;

  let mut includes = Vec::new();
  for (index, (entry, mut document)) in documents.enumerate() {
    let mut changed = false;
    for setting in overrides {
      changed |= remove(&mut document, &setting.path);
    }

    if changed {
      let name = format!("{PREFIX}{tag}-{index}.yaml");
      write_mapping(&dir.join(&name), &document)?;
      includes.push(Value::String(name));
    } else {
      includes.push(Value::String(entry));
    }
  }

  for setting in overrides {
    set(&mut main_document, &setting.path, setting.value.clone());
  }
  if !includes.is_empty() {
    main_document.insert("include".into(), Value::Sequence(includes));
  }

  let main_path = dir.join(format!("{PREFIX}{tag}-config.yaml"));
  write_mapping(&main_path, &main_document)?;
  Ok(main_path)
}

/// Removes every file [`write`] wrote next to `main`, whatever its tag.
pub fn clean(main: &Path) -> anyhow::Result<()> {
  let dir = main
    .parent()
    .context("Config path has no parent directory.")?;

  for entry in std::fs::read_dir(dir)? {
    let path = entry?.path();
    let is_ours = path
      .file_name()
      .and_then(|name| name.to_str())
      .is_some_and(|name| {
        name.starts_with(PREFIX)
          && Path::new(name).extension().is_some_and(|ext| ext == "yaml")
      });

    if is_ours {
      std::fs::remove_file(&path)?;
    }
  }

  Ok(())
}

/// The main file at `main`, then each file it includes, as `(include
/// entry, document)`; the main file's entry is its path.
fn documents(main: &Path) -> anyhow::Result<Vec<(String, Mapping)>> {
  let dir = main
    .parent()
    .context("Config path has no parent directory.")?;
  let main_document = read_mapping(main)?;

  let mut documents = Vec::new();
  if let Some(includes) = main_document.get("include") {
    let Value::Sequence(entries) = includes else {
      bail!("`include` must be a list of file paths.");
    };

    for entry in entries {
      let Value::String(entry) = entry else {
        bail!("`include` must be a list of file paths.");
      };
      documents.push((entry.clone(), read_mapping(&dir.join(entry))?));
    }
  }

  documents.insert(0, (main.display().to_string(), main_document));
  Ok(documents)
}

/// The value at `path` in `document`, if set.
fn lookup<'a>(document: &'a Mapping, path: &[&str]) -> Option<&'a Value> {
  let (last, parents) = path.split_last()?;
  let mut mapping = document;
  for key in parents {
    mapping = mapping.get(*key)?.as_mapping()?;
  }
  mapping.get(*last)
}

/// Removes the value at `path` from `document`, returning whether it was
/// set.
fn remove(document: &mut Mapping, path: &[String]) -> bool {
  let Some((last, parents)) = path.split_last() else {
    return false;
  };
  let mut mapping = document;
  for key in parents {
    let Some(child) = mapping
      .get_mut(key.as_str())
      .and_then(Value::as_mapping_mut)
    else {
      return false;
    };
    mapping = child;
  }
  mapping.remove(last.as_str()).is_some()
}

/// Sets `value` at `path` in `document`, making the mappings on the way.
fn set(document: &mut Mapping, path: &[String], value: Value) {
  let Some((last, parents)) = path.split_last() else {
    return;
  };
  let mut mapping = document;
  for key in parents {
    let entry = mapping
      .entry(Value::String(key.clone()))
      .or_insert_with(|| Value::Mapping(Mapping::new()));
    if !entry.is_mapping() {
      *entry = Value::Mapping(Mapping::new());
    }
    let Value::Mapping(child) = entry else {
      return;
    };
    mapping = child;
  }
  mapping.insert(Value::String(last.clone()), value);
}

fn read_mapping(path: &Path) -> anyhow::Result<Mapping> {
  let text = std::fs::read_to_string(path)
    .with_context(|| format!("Failed to read {}.", path.display()))?;

  Ok(serde_yaml::from_str::<Option<Mapping>>(&text)?.unwrap_or_default())
}

fn write_mapping(path: &Path, mapping: &Mapping) -> anyhow::Result<()> {
  std::fs::write(path, serde_yaml::to_string(mapping)?)
    .with_context(|| format!("Failed to write {}.", path.display()))
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A fresh directory for one test's files.
  fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
      .join(format!("{PREFIX}test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  /// The switch settings move out of the file that set them into the main
  /// file's copy; the user's files are left as they were.
  #[test]
  fn replaces_switch_settings_across_files() {
    let dir = test_dir("switch");
    let main = dir.join("config.yaml");
    let user_main =
      "include: ['animations.yaml']\ngaps:\n  inner_gap: '10px'\n";
    let user_animations =
      "animations:\n  workspace_switch:\n    style: 'slide'\n  window_move:\n    enabled: true\n";
    std::fs::write(&main, user_main).unwrap();
    std::fs::write(dir.join("animations.yaml"), user_animations).unwrap();

    let overrides = switch_overrides("drift").unwrap();
    let bench_main = write(&main, "switch", &overrides).unwrap();

    let bench = read_mapping(&bench_main).unwrap();
    let includes = bench.get("include").unwrap().as_sequence().unwrap();
    assert_eq!(includes.len(), 1);
    let style =
      lookup(&bench, &["animations", "workspace_switch", "style"]);
    assert_eq!(style.and_then(Value::as_str), Some("drift"));

    let copied =
      read_mapping(&dir.join(format!("{PREFIX}switch-0.yaml"))).unwrap();
    assert!(lookup(&copied, &["animations", "workspace_switch"]).is_none());
    assert!(lookup(&copied, &["animations", "window_move"]).is_some());

    assert_eq!(std::fs::read_to_string(&main).unwrap(), user_main);

    clean(&main).unwrap();
    let left: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
    assert_eq!(left.len(), 2);
    std::fs::remove_dir_all(&dir).unwrap();
  }

  /// Turning borders off sets only their `enabled`, in every effect set
  /// the config has; backdrops left to the config keep its settings.
  #[test]
  fn turns_borders_off_in_a_single_file() {
    let dir = test_dir("effects");
    let main = dir.join("config.yaml");
    std::fs::write(
      &main,
      "window_effects:\n  focused_window:\n    border:\n      enabled: true\n      width: '3px'\n    backdrop:\n      enabled: true\n",
    )
    .unwrap();

    let overrides =
      effect_overrides(&main, Effect::Off, Effect::Config).unwrap();
    let bench =
      read_mapping(&write(&main, "this", &overrides).unwrap()).unwrap();

    assert!(bench.get("include").is_none());
    for set in EFFECT_SETS {
      let enabled =
        lookup(&bench, &["window_effects", set, "border", "enabled"]);
      assert_eq!(enabled.and_then(Value::as_bool), Some(false));
    }
    let width = lookup(
      &bench,
      &["window_effects", "focused_window", "border", "width"],
    );
    assert_eq!(width.and_then(Value::as_str), Some("3px"));
    let backdrop = lookup(
      &bench,
      &["window_effects", "focused_window", "backdrop", "enabled"],
    );
    assert_eq!(backdrop.and_then(Value::as_bool), Some(true));
    assert!(lookup(&bench, &["window_effects", URGENT_SET]).is_none());
    let minimize =
      lookup(&bench, &["animations", "window_minimize", "enabled"]);
    assert_eq!(minimize.and_then(Value::as_bool), Some(true));

    std::fs::remove_dir_all(&dir).unwrap();
  }
}
