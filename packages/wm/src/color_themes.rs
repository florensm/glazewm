use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  time::SystemTime,
};

use wm_common::ColorThemesConfig;
use wm_platform::ColorTheme;

/// File name of the color themes config, next to the main config.
const FILE_NAME: &str = "color-themes.yaml";

/// Themes from `color-themes.yaml`, re-read whenever the file changes.
///
/// A missing file means no themes, without an error. An invalid file is
/// reported and the last valid themes are kept, so a half-saved edit never
/// strips a window of its theme.
#[derive(Debug, Default)]
pub struct ColorThemes {
  path: PathBuf,

  /// Modification time of the file as last read; `None` when it didn't
  /// exist.
  modified: Option<SystemTime>,

  themes: HashMap<String, ColorTheme>,
}

impl ColorThemes {
  /// Loads the themes next to the main config at `config_path`.
  pub fn new(config_path: &Path) -> Self {
    let mut themes = Self {
      path: config_path.with_file_name(FILE_NAME),
      modified: None,
      themes: HashMap::new(),
    };

    themes.reload_if_changed();
    themes
  }

  pub fn get(&self, name: &str) -> Option<&ColorTheme> {
    self.themes.get(name)
  }

  /// Re-reads the file if its modification time changed since the last
  /// read. Returns whether the themes changed.
  pub fn reload_if_changed(&mut self) -> bool {
    let modified = fs::metadata(&self.path)
      .and_then(|metadata| metadata.modified())
      .ok();

    if modified == self.modified {
      return false;
    }

    self.modified = modified;

    if modified.is_none() {
      if self.themes.is_empty() {
        return false;
      }

      tracing::info!(
        "Color themes file removed, clearing themes: {}",
        self.path.display()
      );
      self.themes.clear();
      return true;
    }

    match Self::read(&self.path) {
      Ok(themes) => {
        tracing::info!(
          "Loaded {} color theme(s) from {}.",
          themes.len(),
          self.path.display()
        );
        let changed = themes != self.themes;
        self.themes = themes;
        changed
      }
      Err(err) => {
        tracing::warn!(
          "Invalid color themes file {}, keeping previous themes: {err:#}",
          self.path.display()
        );
        false
      }
    }
  }

  fn read(path: &Path) -> anyhow::Result<HashMap<String, ColorTheme>> {
    let content = fs::read_to_string(path)?;
    parse(&content)
  }
}

fn parse(content: &str) -> anyhow::Result<HashMap<String, ColorTheme>> {
  // An empty (or comment-only) file is valid and defines no themes.
  let config: Option<ColorThemesConfig> = serde_yaml::from_str(content)?;
  config.unwrap_or_default().compile()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_example_themes() {
    let themes = parse(
      r##"
themes:
  winter:
    background: "#1e1e1e"
    foreground: "#d4d4d4"
    saturation_threshold: 0.15
    overrides:
      - { from: "#fff3b0", to: "#5a4a00", tolerance: 10 }
      - { from: "#0078d4", to: "#4aa3ff" }
  overrides_only:
    overrides:
      - { from: "#ffffff", to: "#000000", tolerance: 1 }
"##,
    )
    .expect("valid themes");

    assert_eq!(themes.len(), 2);
    assert!(themes.contains_key("winter"));
    assert!(themes.contains_key("overrides_only"));
  }

  #[test]
  fn sample_file_is_valid() {
    let themes = parse(include_str!(
      "../../../resources/assets/sample-color-themes.yaml"
    ))
    .expect("sample color themes should parse");

    assert!(themes.contains_key("winter"));
  }

  #[test]
  fn empty_file_has_no_themes() {
    assert!(parse("").expect("valid").is_empty());
    assert!(parse("# nothing yet\n").expect("valid").is_empty());
  }

  #[test]
  fn rejects_invalid_themes() {
    // Half a ramp.
    assert!(parse("themes: { a: { background: '#000000' } }").is_err());
    // Out-of-range threshold.
    assert!(parse("themes: { a: { saturation_threshold: 2 } }").is_err());
    // Typo'd key.
    assert!(parse("themes: { a: { backgroud: '#000000' } }").is_err());
    // Malformed color.
    assert!(parse(
      "themes: { a: { overrides: [{ from: '#fff', to: '#000000' }] } }"
    )
    .is_err());
  }

  #[test]
  fn invalid_edit_keeps_previous_themes() {
    let dir = std::env::temp_dir()
      .join(format!("glazewm_color_themes_test_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let config_path = dir.join("config.yaml");
    let themes_path = dir.join(FILE_NAME);
    let _ = fs::remove_file(&themes_path);

    let mut themes = ColorThemes::new(&config_path);
    assert!(themes.get("winter").is_none());

    fs::write(
      &themes_path,
      "themes: { winter: { background: '#1e1e1e', foreground: '#d4d4d4' } }",
    )
    .expect("write themes");
    themes.modified = None;
    assert!(themes.reload_if_changed());
    assert!(themes.get("winter").is_some());

    fs::write(&themes_path, "themes: { winter: { background: 3 } }")
      .expect("write themes");
    themes.modified = None;
    assert!(!themes.reload_if_changed());
    assert!(themes.get("winter").is_some());

    fs::remove_file(&themes_path).expect("remove themes");
    assert!(themes.reload_if_changed());
    assert!(themes.get("winter").is_none());

    let _ = fs::remove_dir_all(&dir);
  }
}
