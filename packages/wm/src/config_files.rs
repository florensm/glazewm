use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use serde_yaml::{Mapping, Value};
use wm_common::ParsedConfig;

/// Key of the main config file that lists the files read along with it.
const INCLUDE_KEY: &str = "include";

/// A config file's path and contents.
#[derive(Debug)]
pub struct ConfigFile {
  pub path: PathBuf,
  pub contents: String,
}

/// A user config read from one or more files.
#[derive(Debug)]
pub struct LoadedConfig {
  pub value: ParsedConfig,

  /// The main file's contents, or with `include`, the merged config as
  /// YAML.
  pub value_str: String,
}

/// Reads the config at `main_path`, along with every file it lists under
/// `include`.
pub fn load(main_path: &Path) -> anyhow::Result<LoadedConfig> {
  let main_file = read_file(main_path)?;
  let main_document = parse_document(&main_file)?;
  let include_paths = include_paths(&main_document, main_path)?;

  let mut files = vec![main_file];
  let mut documents = vec![main_document];
  for path in include_paths {
    let file = read_file(&path)?;
    documents.push(parse_document(&file)?);
    files.push(file);
  }

  merge(&files, documents)
}

/// [`merge`] for files not parsed yet.
#[cfg(test)]
pub fn combine(files: &[ConfigFile]) -> anyhow::Result<LoadedConfig> {
  let documents = files
    .iter()
    .map(parse_document)
    .collect::<anyhow::Result<Vec<_>>>()?;

  merge(files, documents)
}

/// Merges config files, parsed into `documents`, one each, into one
/// config. The first file is the main one, the only one that may set
/// `include`.
///
/// Sections are merged key by key, so one can be split across files. A
/// setting set in more than one file is an error naming both.
fn merge(
  files: &[ConfigFile],
  documents: Vec<Mapping>,
) -> anyhow::Result<LoadedConfig> {
  let main_dir = files
    .first()
    .context("No config file to read.")?
    .path
    .parent()
    .map_or_else(PathBuf::new, Path::to_path_buf);

  let mut merged = Mapping::new();
  let mut origins = HashMap::new();

  for (index, (file, mut document)) in
    files.iter().zip(documents).enumerate()
  {
    let name = display_name(&file.path, &main_dir);

    if document.remove(INCLUDE_KEY).is_some() && index != 0 {
      bail!(
        "`{INCLUDE_KEY}` is only read from the main config file, but \
         {name} sets it."
      );
    }

    merge_into(&mut merged, document, &name, "", &mut origins)?;
  }

  // A lone file is parsed from its text, so that errors point at a line.
  if let [file] = files {
    let value = serde_yaml::from_str(&file.contents).map_err(|err| {
      anyhow::anyhow!(
        "Invalid config in {}: {err}",
        display_name(&file.path, &main_dir)
      )
    })?;

    return Ok(LoadedConfig {
      value,
      value_str: file.contents.clone(),
    });
  }

  let value = serde_yaml::from_value(Value::Mapping(merged.clone()))
    .map_err(|err| locate_error(files, &main_dir, &err))?;

  Ok(LoadedConfig {
    value,
    value_str: serde_yaml::to_string(&merged)?,
  })
}

/// Paths listed under the main config's `include`, resolved against its
/// directory.
pub fn include_paths(
  main_document: &Mapping,
  main_path: &Path,
) -> anyhow::Result<Vec<PathBuf>> {
  let Some(include) = main_document.get(INCLUDE_KEY) else {
    return Ok(Vec::new());
  };

  let entries = match include {
    Value::Null => return Ok(Vec::new()),
    Value::Sequence(entries) => entries,
    _ => bail!("`{INCLUDE_KEY}` must be a list of file paths."),
  };

  let main_dir = main_path.parent().unwrap_or_else(|| Path::new(""));
  let mut paths: Vec<PathBuf> = Vec::new();

  for entry in entries {
    let Value::String(entry) = entry else {
      bail!("`{INCLUDE_KEY}` must be a list of file paths.");
    };

    let path = main_dir.join(entry);

    if path == main_path {
      bail!("The main config file can't include itself.");
    }

    if paths.contains(&path) {
      bail!("`{entry}` is listed more than once under `{INCLUDE_KEY}`.");
    }

    paths.push(path);
  }

  Ok(paths)
}

fn read_file(path: &Path) -> anyhow::Result<ConfigFile> {
  let contents = fs::read_to_string(path).with_context(|| {
    format!("Unable to read config file {}.", path.display())
  })?;

  Ok(ConfigFile {
    path: path.to_path_buf(),
    contents,
  })
}

/// Parses a config file into its top-level mapping. An empty file is an
/// empty mapping.
pub fn parse_document(file: &ConfigFile) -> anyhow::Result<Mapping> {
  let value: Value =
    serde_yaml::from_str(&file.contents).with_context(|| {
      format!("Invalid YAML in {}.", file.path.display())
    })?;

  match value {
    Value::Mapping(mapping) => Ok(mapping),
    Value::Null => Ok(Mapping::new()),
    _ => bail!(
      "{} must contain config sections (e.g. `general:`).",
      file.path.display()
    ),
  }
}

/// Merges `source` into `target`, recording which file set each key.
///
/// `origins` maps a dotted key path to the file that set it; a key nested
/// in a mapping copied over whole is attributed to that mapping's file.
fn merge_into(
  target: &mut Mapping,
  source: Mapping,
  file_name: &str,
  prefix: &str,
  origins: &mut HashMap<String, String>,
) -> anyhow::Result<()> {
  for (key, value) in source {
    let key_path = match &key {
      Value::String(key) if prefix.is_empty() => key.clone(),
      Value::String(key) => format!("{prefix}.{key}"),
      _ => bail!(
        "{file_name} has a key that isn't a string under `{prefix}`."
      ),
    };

    match (target.get_mut(&key), value) {
      (Some(Value::Mapping(existing)), Value::Mapping(incoming)) => {
        merge_into(existing, incoming, file_name, &key_path, origins)?;
      }
      (Some(_), _) => {
        let other = origin_of(&key_path, origins);
        bail!(
          "`{key_path}` is set in both {other} and {file_name}. Keep it in \
           one of them."
        );
      }
      (None, value) => {
        origins.insert(key_path, file_name.to_string());
        target.insert(key, value);
      }
    }
  }

  Ok(())
}

/// File that set `key_path`, or the closest of its parents that has one.
fn origin_of(key_path: &str, origins: &HashMap<String, String>) -> String {
  let mut path = key_path;

  loop {
    if let Some(file_name) = origins.get(path) {
      return file_name.clone();
    }

    match path.rfind('.') {
      Some(index) => path = &path[..index],
      None => return "another file".to_string(),
    }
  }
}

/// Attributes a failed parse of the merged config to the file at fault.
///
/// Errors from the merged value carry no position, so each file is parsed
/// on its own to find one that fails with its line and column. Falls back
/// to the merged error when no file fails alone (e.g. a struct without
/// defaults split across files).
fn locate_error(
  files: &[ConfigFile],
  main_dir: &Path,
  merged_error: &serde_yaml::Error,
) -> anyhow::Error {
  for file in files {
    if let Err(err) = serde_yaml::from_str::<ParsedConfig>(&file.contents)
    {
      return anyhow::anyhow!(
        "Invalid config in {}: {err}",
        display_name(&file.path, main_dir)
      );
    }
  }

  anyhow::anyhow!("Invalid config: {merged_error}")
}

/// `path` relative to the main config's directory when inside it.
fn display_name(path: &Path, main_dir: &Path) -> String {
  path
    .strip_prefix(main_dir)
    .unwrap_or(path)
    .display()
    .to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn file(name: &str, contents: &str) -> ConfigFile {
    ConfigFile {
      path: PathBuf::from("cfg").join(name),
      contents: contents.to_string(),
    }
  }

  #[test]
  fn single_file_keeps_its_contents() {
    let contents = "# Comment.\ngeneral:\n  focus_follows_cursor: true\n";
    let loaded = combine(&[file("config.yaml", contents)]).unwrap();

    assert_eq!(loaded.value_str, contents);
    assert!(loaded.value.general.focus_follows_cursor);
  }

  /// Merged files are deserialized from a `Value`, which nothing may
  /// borrow strings from.
  #[test]
  fn parses_keybindings_from_an_included_file() {
    let loaded = combine(&[
      file("config.yaml", "include: [a.yaml]\n"),
      file(
        "a.yaml",
        "keybindings:\n  - commands: [close]\n    bindings: [h, alt+q]\n",
      ),
    ])
    .unwrap();

    assert_eq!(loaded.value.keybindings[0].bindings.len(), 2);
  }

  #[test]
  fn merges_sections_from_several_files() {
    let loaded = combine(&[
      file(
        "config.yaml",
        "include: [a.yaml]\ngaps:\n  inner_gap: 7px\n",
      ),
      file("a.yaml", "special_workspace:\n  name: scratch\n"),
    ])
    .unwrap();

    assert_eq!(loaded.value.special_workspace.name, "scratch");
    assert!(loaded.value_str.contains("inner_gap"));
    assert!(!loaded.value_str.contains("include"));
  }

  #[test]
  fn merges_a_section_split_across_files() {
    let loaded = combine(&[
      file("config.yaml", "general:\n  focus_follows_cursor: true\n"),
      file("a.yaml", "general:\n  dynamic_workspaces: true\n"),
    ])
    .unwrap();

    assert!(loaded.value.general.focus_follows_cursor);
    assert!(loaded.value.general.dynamic_workspaces);
  }

  #[test]
  fn rejects_a_setting_set_twice() {
    let err = combine(&[
      file("config.yaml", "general:\n  dynamic_workspaces: true\n"),
      file("a.yaml", "general:\n  dynamic_workspaces: false\n"),
    ])
    .unwrap_err()
    .to_string();

    assert!(err.contains("`general.dynamic_workspaces`"), "{err}");
    assert!(
      err.contains("config.yaml") && err.contains("a.yaml"),
      "{err}"
    );
  }

  #[test]
  fn rejects_a_section_set_whole_and_in_part() {
    let err = combine(&[
      file("config.yaml", "keybindings: []\n"),
      file("a.yaml", "keybindings:\n  - commands: [close]\n"),
    ])
    .unwrap_err()
    .to_string();

    assert!(err.contains("`keybindings`"), "{err}");
  }

  #[test]
  fn names_the_file_of_a_conflict_nested_in_a_copied_section() {
    let err = combine(&[
      file(
        "config.yaml",
        "general:\n  cursor_jump:\n    enabled: true\n",
      ),
      file("a.yaml", "general:\n  cursor_jump:\n    enabled: false\n"),
    ])
    .unwrap_err()
    .to_string();

    assert!(err.contains("both config.yaml and a.yaml"), "{err}");
  }

  #[test]
  fn only_the_main_file_may_include() {
    let err = combine(&[
      file("config.yaml", "include: [a.yaml]\n"),
      file("a.yaml", "include: [b.yaml]\n"),
    ])
    .unwrap_err()
    .to_string();

    assert!(err.contains("a.yaml sets it"), "{err}");
  }

  #[test]
  fn reports_the_file_of_an_invalid_value() {
    let err = combine(&[
      file("config.yaml", "general:\n  focus_follows_cursor: true\n"),
      file(
        "a.yaml",
        "gaps:\n  inner_gap: 7px\n\ngeneral:\n  default_layout: 3\n",
      ),
    ])
    .unwrap_err()
    .to_string();

    assert!(err.contains("a.yaml"), "{err}");
    assert!(err.contains("line 5"), "{err}");
  }

  #[test]
  fn empty_files_are_empty_sections() {
    let loaded = combine(&[
      file("config.yaml", "include: [a.yaml]\n"),
      file("a.yaml", ""),
    ])
    .unwrap();

    assert_eq!(loaded.value.special_workspace.name, "special");
  }

  #[test]
  fn resolves_includes_next_to_the_main_file() {
    let main_path = Path::new("cfg/config.yaml");
    let document = parse_document(&file(
      "config.yaml",
      "include: [a.yaml, sub/b.yaml]\n",
    ))
    .unwrap();

    assert_eq!(
      include_paths(&document, main_path).unwrap(),
      [PathBuf::from("cfg/a.yaml"), PathBuf::from("cfg/sub/b.yaml")]
    );
  }

  #[test]
  fn rejects_invalid_includes() {
    let main_path = Path::new("cfg/config.yaml");

    for (contents, expected) in [
      ("include: a.yaml\n", "must be a list"),
      ("include: [3]\n", "must be a list"),
      ("include: [a.yaml, a.yaml]\n", "more than once"),
      ("include: [config.yaml]\n", "include itself"),
    ] {
      let document =
        parse_document(&file("config.yaml", contents)).unwrap();
      let err =
        include_paths(&document, main_path).unwrap_err().to_string();

      assert!(err.contains(expected), "{contents}: {err}");
    }
  }

  #[test]
  fn rejects_a_file_that_is_not_a_mapping() {
    let err = combine(&[file("config.yaml", "- general\n")])
      .unwrap_err()
      .to_string();

    assert!(err.contains("must contain config sections"), "{err}");
  }
}
