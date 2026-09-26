use std::collections::HashMap;

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
  pub elements: Option<ElementsConfig>,
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

/// Per UI element kind, `original` or the name of a theme whose colors it
/// takes instead.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElementsConfig {
  pub edit: Option<String>,
  pub document: Option<String>,
  pub button: Option<String>,
  pub hyperlink: Option<String>,
  pub check_box: Option<String>,
  pub radio_button: Option<String>,
  pub combo_box: Option<String>,
  pub list_item: Option<String>,
  pub tree_item: Option<String>,
  pub tab_item: Option<String>,
  pub menu_item: Option<String>,
  pub data_item: Option<String>,
  pub header: Option<String>,
  pub tool_bar: Option<String>,
  pub status_bar: Option<String>,
  pub title_bar: Option<String>,
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
    let elements = theme.elements.clone().unwrap_or_default();

    let treatments = elements
      .entries()
      .into_iter()
      .filter_map(|(kind, value)| value.map(|value| (kind, value)))
      .map(|(kind, value)| {
        let treatment = if value == ORIGINAL {
          ElementTreatment::Original
        } else {
          let other = self.resolve(value).map_err(|err| {
            anyhow::anyhow!("Element theme '{value}': {err}")
          })?;
          ElementTreatment::Filter(self.compile_filter(&other).map_err(
            |err| anyhow::anyhow!("Element theme '{value}': {err}"),
          )?)
        };

        Ok((kind, treatment))
      })
      .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(ColorTheme::new(ColorThemeOptions {
      filter,
      elements: treatments,
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
          from: white(),
          to: background,
        },
        RampStop {
          from: black(),
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
      elements: match (self.elements, parent.elements) {
        (Some(child), Some(parent)) => Some(child.merged_over(parent)),
        (child, parent) => child.or(parent),
      },
    }
  }
}

impl ElementsConfig {
  fn merged_over(self, parent: Self) -> Self {
    Self {
      edit: self.edit.or(parent.edit),
      document: self.document.or(parent.document),
      button: self.button.or(parent.button),
      hyperlink: self.hyperlink.or(parent.hyperlink),
      check_box: self.check_box.or(parent.check_box),
      radio_button: self.radio_button.or(parent.radio_button),
      combo_box: self.combo_box.or(parent.combo_box),
      list_item: self.list_item.or(parent.list_item),
      tree_item: self.tree_item.or(parent.tree_item),
      tab_item: self.tab_item.or(parent.tab_item),
      menu_item: self.menu_item.or(parent.menu_item),
      data_item: self.data_item.or(parent.data_item),
      header: self.header.or(parent.header),
      tool_bar: self.tool_bar.or(parent.tool_bar),
      status_bar: self.status_bar.or(parent.status_bar),
      title_bar: self.title_bar.or(parent.title_bar),
    }
  }

  fn entries(&self) -> [(UiElementKind, Option<&str>); 16] {
    [
      (UiElementKind::Edit, self.edit.as_deref()),
      (UiElementKind::Document, self.document.as_deref()),
      (UiElementKind::Button, self.button.as_deref()),
      (UiElementKind::Hyperlink, self.hyperlink.as_deref()),
      (UiElementKind::CheckBox, self.check_box.as_deref()),
      (UiElementKind::RadioButton, self.radio_button.as_deref()),
      (UiElementKind::ComboBox, self.combo_box.as_deref()),
      (UiElementKind::ListItem, self.list_item.as_deref()),
      (UiElementKind::TreeItem, self.tree_item.as_deref()),
      (UiElementKind::TabItem, self.tab_item.as_deref()),
      (UiElementKind::MenuItem, self.menu_item.as_deref()),
      (UiElementKind::DataItem, self.data_item.as_deref()),
      (UiElementKind::Header, self.header.as_deref()),
      (UiElementKind::ToolBar, self.tool_bar.as_deref()),
      (UiElementKind::StatusBar, self.status_bar.as_deref()),
      (UiElementKind::TitleBar, self.title_bar.as_deref()),
    ]
  }
}

fn white() -> Color {
  Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
  }
}

fn black() -> Color {
  Color {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
  }
}
