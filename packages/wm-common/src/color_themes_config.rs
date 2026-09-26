use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use wm_platform::{
  Color, ColorFilter, ColorFilterOptions, ColorOverride, ColorTheme,
  ColorThemeOptions, ElementTreatment, RampStop, UiElementKind,
};

/// Element value that keeps an element's original colors.
const ORIGINAL: &str = "original";

/// Longest `extends` chain, which also catches cycles.
const MAX_EXTENDS_DEPTH: usize = 16;

/// Contents of `color-themes.yaml`, kept separate from the main config so
/// it can be hot-reloaded on its own.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorThemesConfig {
  /// Named color lists that themes' `palette` can refer to.
  #[serde(default)]
  pub palettes: HashMap<String, Vec<Color>>,

  #[serde(default)]
  pub themes: HashMap<String, ColorThemeConfig>,
}

/// One theme as written. Every field is optional so `extends` can tell
/// what a theme sets itself from what it inherits.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorThemeConfig {
  /// Theme whose settings this one starts from.
  pub extends: Option<String>,

  /// Color white maps to; shorthand for a two-stop `ramp`, set together
  /// with `foreground`.
  pub background: Option<Color>,

  /// Color black maps to.
  pub foreground: Option<Color>,

  /// Gray ramp stops, instead of `background`/`foreground`.
  pub ramp: Option<Vec<RampStopConfig>>,

  pub saturation_threshold: Option<f32>,
  pub accent_lightness: Option<f32>,
  pub saturation: Option<f32>,
  pub vibrance: Option<f32>,
  pub hue_shift: Option<f32>,
  pub palette: Option<PaletteConfig>,
  pub palette_strength: Option<f32>,
  pub palette_lightness: Option<f32>,
  pub brightness: Option<f32>,
  pub contrast: Option<f32>,
  pub min_contrast: Option<f32>,
  pub warmth: Option<f32>,
  pub overrides: Option<Vec<ColorOverrideConfig>>,
  pub detect_colors: Option<bool>,
  pub skip_if_dark: Option<bool>,
  /// Per UI element kind, `original` or the name of a theme whose colors
  /// it takes instead.
  pub elements: Option<BTreeMap<UiElementKind, String>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RampStopConfig {
  pub from: Color,
  pub to: Color,
}

/// A palette by name (from `palettes`), or its colors inline.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum PaletteConfig {
  Named(String),
  Colors(Vec<Color>),
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

fn default_tolerance() -> f32 {
  5.0
}

impl ColorThemesConfig {
  /// Validates every theme, failing on the first invalid one.
  pub fn compile(&self) -> anyhow::Result<HashMap<String, ColorTheme>> {
    if self.themes.contains_key(ORIGINAL) {
      anyhow::bail!("`{ORIGINAL}` is reserved and can't name a theme.");
    }

    self
      .themes
      .keys()
      .map(|name| {
        self
          .compile_theme(name)
          .map(|theme| (name.clone(), theme))
          .map_err(|err| anyhow::anyhow!("Color theme '{name}': {err}"))
      })
      .collect()
  }

  fn compile_theme(&self, name: &str) -> anyhow::Result<ColorTheme> {
    let theme = self.resolve(name)?;
    let filter = self.compile_filter(&theme)?;
    let elements = theme
      .elements
      .iter()
      .flatten()
      .map(|(kind, value)| {
        let treatment = if value == ORIGINAL {
          ElementTreatment::Original
        } else {
          ElementTreatment::Filter(
            self
              .resolve(value)
              .and_then(|other| self.compile_filter(&other))
              .map_err(|err| {
                anyhow::anyhow!("Element theme '{value}': {err}")
              })?,
          )
        };

        Ok((*kind, treatment))
      })
      .collect::<anyhow::Result<_>>()?;

    Ok(ColorTheme::new(ColorThemeOptions {
      filter,
      elements,
      detect_colors: theme.detect_colors.unwrap_or(false),
      skip_if_dark: theme.skip_if_dark.unwrap_or(false),
    })?)
  }

  /// `name`'s settings with its `extends` chain merged in.
  fn resolve(&self, name: &str) -> anyhow::Result<ColorThemeConfig> {
    let mut chain = Vec::new();
    let mut next = Some(name);

    while let Some(current) = next {
      if chain.len() >= MAX_EXTENDS_DEPTH {
        anyhow::bail!(
          "`extends` chain is circular or deeper than {MAX_EXTENDS_DEPTH}."
        );
      }

      let theme = self
        .themes
        .get(current)
        .ok_or_else(|| anyhow::anyhow!("Unknown theme '{current}'."))?;

      chain.push(theme);
      next = theme.extends.as_deref();
    }

    Ok(
      chain
        .into_iter()
        .rev()
        .fold(ColorThemeConfig::default(), |parent, child| {
          child.clone().merged_over(parent)
        }),
    )
  }

  fn compile_filter(
    &self,
    theme: &ColorThemeConfig,
  ) -> anyhow::Result<ColorFilter> {
    let ramp = match (&theme.ramp, theme.background, theme.foreground) {
      (Some(ramp), None, None) => ramp
        .iter()
        .map(|stop| RampStop {
          from: stop.from,
          to: stop.to,
        })
        .collect(),
      (None, Some(background), Some(foreground)) => vec![
        RampStop {
          from: WHITE,
          to: background,
        },
        RampStop {
          from: BLACK,
          to: foreground,
        },
      ],
      (None, None, None) => Vec::new(),
      (Some(_), _, _) => anyhow::bail!(
        "Set either `ramp` or `background`/`foreground`, not both."
      ),
      _ => anyhow::bail!(
        "`background` and `foreground` must be set together."
      ),
    };

    let palette = match &theme.palette {
      None => Vec::new(),
      Some(PaletteConfig::Colors(colors)) => colors.clone(),
      Some(PaletteConfig::Named(name)) => self
        .palettes
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unknown palette '{name}'."))?,
    };

    let defaults = ColorFilterOptions::default();

    Ok(ColorFilter::new(&ColorFilterOptions {
      ramp,
      saturation_threshold: theme
        .saturation_threshold
        .unwrap_or(defaults.saturation_threshold),
      accent_lightness: theme
        .accent_lightness
        .unwrap_or(defaults.accent_lightness),
      saturation: theme.saturation.unwrap_or(defaults.saturation),
      vibrance: theme.vibrance.unwrap_or(defaults.vibrance),
      hue_shift: theme.hue_shift.unwrap_or(defaults.hue_shift),
      palette,
      palette_strength: theme
        .palette_strength
        .unwrap_or(defaults.palette_strength),
      palette_lightness: theme
        .palette_lightness
        .unwrap_or(defaults.palette_lightness),
      brightness: theme.brightness.unwrap_or(defaults.brightness),
      contrast: theme.contrast.unwrap_or(defaults.contrast),
      min_contrast: theme.min_contrast.unwrap_or(defaults.min_contrast),
      warmth: theme.warmth.unwrap_or(defaults.warmth),
      overrides: theme
        .overrides
        .iter()
        .flatten()
        .map(|o| ColorOverride {
          from: o.from,
          to: o.to,
          tolerance: o.tolerance,
        })
        .collect(),
    })?)
  }
}

impl ColorThemeConfig {
  /// `self`'s settings, falling back to `parent`'s for any it leaves
  /// unset. The ramp (`ramp` or `background`/`foreground`) is taken as a
  /// whole, so a child can switch between the two forms.
  fn merged_over(self, parent: Self) -> Self {
    let has_ramp = self.ramp.is_some()
      || self.background.is_some()
      || self.foreground.is_some();
    let (ramp, background, foreground) = if has_ramp {
      (self.ramp, self.background, self.foreground)
    } else {
      (parent.ramp, parent.background, parent.foreground)
    };

    Self {
      extends: None,
      background,
      foreground,
      ramp,
      saturation_threshold: self
        .saturation_threshold
        .or(parent.saturation_threshold),
      accent_lightness: self.accent_lightness.or(parent.accent_lightness),
      saturation: self.saturation.or(parent.saturation),
      vibrance: self.vibrance.or(parent.vibrance),
      hue_shift: self.hue_shift.or(parent.hue_shift),
      palette: self.palette.or(parent.palette),
      palette_strength: self.palette_strength.or(parent.palette_strength),
      palette_lightness: self
        .palette_lightness
        .or(parent.palette_lightness),
      brightness: self.brightness.or(parent.brightness),
      contrast: self.contrast.or(parent.contrast),
      min_contrast: self.min_contrast.or(parent.min_contrast),
      warmth: self.warmth.or(parent.warmth),
      overrides: self.overrides.or(parent.overrides),
      detect_colors: self.detect_colors.or(parent.detect_colors),
      skip_if_dark: self.skip_if_dark.or(parent.skip_if_dark),
      elements: match (parent.elements, self.elements) {
        (Some(mut parent), Some(child)) => {
          parent.extend(child);
          Some(parent)
        }
        (parent, child) => child.or(parent),
      },
    }
  }
}

const WHITE: Color = Color {
  r: 255,
  g: 255,
  b: 255,
  a: 255,
};

const BLACK: Color = Color {
  r: 0,
  g: 0,
  b: 0,
  a: 255,
};
