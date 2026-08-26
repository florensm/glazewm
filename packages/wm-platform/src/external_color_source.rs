use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  str::FromStr,
  sync::{Mutex, OnceLock},
  time::SystemTime,
};

use crate::Color;

type CacheKey = (PathBuf, String);

static COLOR_FILE_CACHE: OnceLock<Mutex<HashMap<CacheKey, (SystemTime, Color)>>> =
  OnceLock::new();

/// Reads a named CSS custom property from an external generated palette
/// file -- e.g. matugen, pywal, or YASB's `yasb_colors.css` -- for the
/// `color: { file, key }` border config source. `key` is matched verbatim,
/// including its leading `--` (e.g. `"--yasb-accent"`).
///
/// Cached per `(path, key)`, keyed on the file's mtime: the file is only
/// re-read/re-parsed when its mtime changes since the last call, since this
/// runs on the same per-tick overlay sync hot path as
/// [`crate::system_accent_color`] (an mtime `stat()` every call is cheap;
/// re-parsing file content every call would not be).
///
/// Recognizes `#rrggbb`/`#rrggbbaa` hex and `rgb(r, g, b)` functional
/// notation -- not a general CSS parser, just this one property-value
/// pattern, since that covers every generator this was built against
/// (matugen leans hex, YASB emits `rgb()`).
///
/// # Errors
///
/// Returns an error if the file can't be read, `key` isn't found in it, or
/// the associated value isn't a recognized color format.
pub fn color_from_file(path: &Path, key: &str) -> crate::Result<Color> {
  let metadata = std::fs::metadata(path)?;
  let modified = metadata.modified()?;

  let cache = COLOR_FILE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
  let cache_key = (path.to_path_buf(), key.to_string());

  // A poisoned lock means an earlier caller panicked while holding it,
  // which never happens in this function's small, panic-free critical
  // section, so `expect` here is effectively infallible.
  let mut guard = cache.lock().expect("color-file cache mutex poisoned");

  if let Some((cached_mtime, color)) = guard.get(&cache_key) {
    if *cached_mtime == modified {
      return Ok(*color);
    }
  }

  let content = std::fs::read_to_string(path)?;
  let color = parse_css_custom_property(&content, key)?;
  guard.insert(cache_key, (modified, color));
  Ok(color)
}

/// Finds `--key: value;` in `content` and parses `value` as a color.
fn parse_css_custom_property(
  content: &str,
  key: &str,
) -> crate::Result<Color> {
  let needle = format!("{key}:");
  let start = content.find(&needle).ok_or_else(|| {
    crate::Error::Platform(format!(
      "Color source key '{key}' not found in file."
    ))
  })?;

  let after_colon = &content[start + needle.len()..];
  let end = after_colon.find(';').ok_or_else(|| {
    crate::Error::Platform(format!(
      "Color source key '{key}' has no terminating ';' in file."
    ))
  })?;

  let value = after_colon[..end].trim();
  parse_color_value(value).ok_or_else(|| {
    crate::Error::Platform(format!(
      "Color source key '{key}' has an unrecognized value '{value}' -- \
       expected '#rrggbb'/'#rrggbbaa' hex or 'rgb(r, g, b)'."
    ))
  })
}

/// Parses a color value in `#rrggbb`/`#rrggbbaa` hex or `rgb(r, g, b)`
/// functional notation.
fn parse_color_value(value: &str) -> Option<Color> {
  if value.starts_with('#') {
    return Color::from_str(value).ok();
  }

  let inner =
    value.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')'))?;

  let mut parts = inner.split(',').map(|p| p.trim().parse::<u8>());
  let r = parts.next()?.ok()?;
  let g = parts.next()?.ok()?;
  let b = parts.next()?.ok()?;
  if parts.next().is_some() {
    return None;
  }

  Some(Color { r, g, b, a: 255 })
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use super::*;

  /// Writes `content` to a uniquely-named temp file and returns its path;
  /// the file is left for the OS temp-dir cleanup rather than deleted
  /// inline, matching this being a short-lived unit test.
  fn temp_css_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir()
      .join(format!("glazewm_external_color_source_test_{name}.css"));
    let mut file = std::fs::File::create(&path)
      .expect("failed to create temp css file for test");
    file
      .write_all(content.as_bytes())
      .expect("failed to write temp css file for test");
    path
  }

  #[test]
  fn parses_hex_value() {
    let path = temp_css_file(
      "hex",
      ":root {\n  --accent: #ab6219;\n  --other: #000000;\n}\n",
    );

    let color = color_from_file(&path, "--accent").unwrap();
    assert_eq!(color, Color { r: 0xab, g: 0x62, b: 0x19, a: 255 });
  }

  #[test]
  fn parses_rgb_value() {
    let path = temp_css_file(
      "rgb",
      ":root {\n  --yasb-accent: rgb(171, 98, 25);\n}\n",
    );

    let color = color_from_file(&path, "--yasb-accent").unwrap();
    assert_eq!(color, Color { r: 171, g: 98, b: 25, a: 255 });
  }

  #[test]
  fn does_not_match_a_key_that_is_a_suffix_of_another() {
    // "--accent" must not match inside "--yasb-accent-dark1: ...".
    let path = temp_css_file(
      "suffix",
      ":root {\n  --yasb-accent-dark1: rgb(1, 2, 3);\n  --accent: rgb(9, 9, 9);\n}\n",
    );

    let color = color_from_file(&path, "--accent").unwrap();
    assert_eq!(color, Color { r: 9, g: 9, b: 9, a: 255 });
  }

  #[test]
  fn errors_when_key_not_found() {
    let path = temp_css_file("missing", ":root {\n  --other: #ffffff;\n}\n");

    let err = color_from_file(&path, "--accent").unwrap_err();
    assert!(err.to_string().contains("not found"));
  }

  #[test]
  fn errors_on_malformed_value() {
    let path =
      temp_css_file("malformed", ":root {\n  --accent: not-a-color;\n}\n");

    let err = color_from_file(&path, "--accent").unwrap_err();
    assert!(err.to_string().contains("unrecognized value"));
  }

  #[test]
  fn errors_when_file_missing() {
    let path = std::env::temp_dir()
      .join("glazewm_external_color_source_test_does_not_exist.css");

    assert!(color_from_file(&path, "--accent").is_err());
  }
}
