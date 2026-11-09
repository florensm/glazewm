//! Copies of the user's config that differ only in how workspaces switch,
//! for `switch_modes.ps1` to compare the styles with.
//!
//! Written next to the user's main config, so the relative paths it holds
//! still resolve, under a prefix only these files use. The user's own
//! files are only read.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use serde_yaml::{Mapping, Value};

/// Prefix of every file this module writes.
const PREFIX: &str = "glazewm-bench-";

/// The styles [`write`] takes, with the `workspace_switch` settings each
/// sets. The subtle ones get the short duration they are meant for.
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

/// The user's main config file: `explicit`, else `GLAZEWM_CONFIG_PATH`,
/// else the default, as the WM itself picks it.
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

/// Writes a copy of the config at `user_main` that switches workspaces as
/// `style` says, returning the path of its main file.
///
/// A setting set in two files is an error, so every copied file that sets
/// `animations.workspace_switch` is copied without it, and one more file
/// with the style's settings is included.
pub fn write(user_main: &Path, style: &str) -> anyhow::Result<PathBuf> {
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

  let dir = user_main
    .parent()
    .context("Config path has no parent directory.")?;
  let mut main = read_mapping(user_main)?;
  remove_switch(&mut main);

  let mut includes = Vec::new();
  if let Some(Value::Sequence(entries)) = main.get("include") {
    for (index, entry) in entries.iter().enumerate() {
      let Value::String(entry) = entry else {
        bail!("`include` must be a list of file paths.");
      };

      let mut document = read_mapping(&dir.join(entry))?;
      if remove_switch(&mut document) {
        let name = format!("{PREFIX}{index}.yaml");
        write_mapping(&dir.join(&name), &document)?;
        includes.push(Value::String(name));
      } else {
        includes.push(Value::String(entry.clone()));
      }
    }
  }

  let switch_name = format!("{PREFIX}switch.yaml");
  let switch: Value = serde_yaml::from_str(settings)?;
  let mut animations = Mapping::new();
  animations.insert("workspace_switch".into(), switch);
  let mut switch_file = Mapping::new();
  switch_file.insert("animations".into(), Value::Mapping(animations));
  write_mapping(&dir.join(&switch_name), &switch_file)?;

  includes.push(Value::String(switch_name));
  main.insert("include".into(), Value::Sequence(includes));

  let main_path = dir.join(format!("{PREFIX}config.yaml"));
  write_mapping(&main_path, &main)?;
  Ok(main_path)
}

/// Removes every file [`write`] wrote next to `user_main`.
pub fn clean(user_main: &Path) -> anyhow::Result<()> {
  let dir = user_main
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

/// Removes `animations.workspace_switch`, returning whether it was set.
fn remove_switch(document: &mut Mapping) -> bool {
  document
    .get_mut("animations")
    .and_then(Value::as_mapping_mut)
    .and_then(|animations| animations.remove("workspace_switch"))
    .is_some()
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

  /// The switch settings move out of the file that set them into one of
  /// their own; the user's files are left as they were.
  #[test]
  fn replaces_switch_settings_across_files() {
    let dir = std::env::temp_dir()
      .join(format!("{PREFIX}test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let main = dir.join("config.yaml");
    let user_main =
      "include: ['animations.yaml']\ngaps:\n  inner_gap: '10px'\n";
    let user_animations =
      "animations:\n  workspace_switch:\n    style: 'slide'\n  window_move:\n    enabled: true\n";
    std::fs::write(&main, user_main).unwrap();
    std::fs::write(dir.join("animations.yaml"), user_animations).unwrap();

    let bench_main = write(&main, "drift").unwrap();

    let bench = read_mapping(&bench_main).unwrap();
    let includes = bench.get("include").unwrap().as_sequence().unwrap();
    assert_eq!(includes.len(), 2);

    let copied =
      read_mapping(&dir.join(format!("{PREFIX}0.yaml"))).unwrap();
    let animations =
      copied.get("animations").unwrap().as_mapping().unwrap();
    assert!(animations.get("workspace_switch").is_none());
    assert!(animations.get("window_move").is_some());

    let switch = Value::Mapping(
      read_mapping(&dir.join(format!("{PREFIX}switch.yaml"))).unwrap(),
    );
    let style = &switch["animations"]["workspace_switch"]["style"];
    assert_eq!(style.as_str(), Some("drift"));

    assert_eq!(std::fs::read_to_string(&main).unwrap(), user_main);

    clean(&main).unwrap();
    let left: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
    assert_eq!(left.len(), 2);
    std::fs::remove_dir_all(&dir).unwrap();
  }
}
