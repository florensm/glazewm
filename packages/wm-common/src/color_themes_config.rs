use std::collections::HashMap;

use serde::Deserialize;
use wm_platform::{Color, ColorOverride, ColorTheme};

/// Contents of `color-themes.yaml`, kept separate from the main config so
/// it can be hot-reloaded on its own.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorThemesConfig {
  #[serde(default)]
  pub themes: HashMap<String, ColorThemeConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorThemeConfig {
  /// Color white maps to. Set together with `foreground`, or neither.
  pub background: Option<Color>,

  /// Color black maps to.
  pub foreground: Option<Color>,

  /// Pixels more saturated than this (0.0-1.0) skip the gray ramp.
  #[serde(default = "default_saturation_threshold")]
  pub saturation_threshold: f32,

  #[serde(default)]
  pub overrides: Vec<ColorOverrideConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorOverrideConfig {
  pub from: Color,
  pub to: Color,

  /// OKLab distance times 100 (~2 is barely noticeable).
  #[serde(default = "default_tolerance")]
  pub tolerance: f32,
}

fn default_saturation_threshold() -> f32 {
  0.15
}

fn default_tolerance() -> f32 {
  5.0
}

impl ColorThemesConfig {
  /// Validates every theme, failing on the first invalid one.
  pub fn compile(&self) -> anyhow::Result<HashMap<String, ColorTheme>> {
    self
      .themes
      .iter()
      .map(|(name, theme)| {
        theme
          .compile()
          .map(|compiled| (name.clone(), compiled))
          .map_err(|err| anyhow::anyhow!("Color theme '{name}': {err}"))
      })
      .collect()
  }
}

impl ColorThemeConfig {
  fn compile(&self) -> anyhow::Result<ColorTheme> {
    let ramp = match (self.background, self.foreground) {
      (Some(background), Some(foreground)) => {
        Some((background, foreground))
      }
      (None, None) => None,
      _ => anyhow::bail!(
        "`background` and `foreground` must be set together."
      ),
    };

    let overrides = self
      .overrides
      .iter()
      .map(|o| ColorOverride {
        from: o.from,
        to: o.to,
        tolerance: o.tolerance,
      })
      .collect::<Vec<_>>();

    Ok(ColorTheme::new(
      ramp,
      self.saturation_threshold,
      &overrides,
    )?)
  }
}
