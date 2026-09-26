//! Per-window color themes: the color math, as a CPU reference for the
//! pixel shader in `shaders/color_theme.hlsl`.
//!
//! The shader is a line-for-line port of
//! [`ColorFilter::apply_neighborhood`] (and the [`ColorFilter::apply`] it
//! builds on) and reads the exact [`FilterConstants`] this module builds,
//! so the unit tests here cover what the GPU computes. Any change to one
//! must be mirrored in the other.

use std::sync::Arc;

use crate::Color;

/// Maximum number of color overrides per filter; sizes the shader's
/// constant buffer.
pub const MAX_COLOR_OVERRIDES: usize = 16;

/// Maximum number of gray ramp stops per filter.
pub const MAX_RAMP_STOPS: usize = 8;

/// Maximum number of (non-gray) palette colors per filter.
pub const MAX_PALETTE_COLORS: usize = 16;

/// Maximum number of distinct filters a theme can assign to UI elements,
/// on top of its own.
pub const MAX_ELEMENT_FILTERS: usize = 3;

/// Filters one window renders with: the theme's own, then its element
/// filters.
#[cfg(any(target_os = "windows", test))]
pub(crate) const MAX_FILTER_SLOTS: usize = 1 + MAX_ELEMENT_FILTERS;

/// Element slot that shows the captured pixels unchanged.
#[cfg(any(target_os = "windows", test))]
pub(crate) const SLOT_ORIGINAL: u32 = u32::MAX;

/// Maximum number of UI element regions per window; sizes the shader's
/// constant buffer.
#[cfg(any(target_os = "windows", test))]
pub(crate) const MAX_REGIONS: usize = 64;

/// OKLab chroma treated as fully saturated, roughly that of pure sRGB
/// blue (the most chromatic sRGB primary).
const MAX_CHROMA: f32 = 0.32;

/// OKLab distance between the darkest and lightest neighbor over which a
/// pixel goes from flat to fully treated as an anti-aliased edge.
const EDGE_START: f32 = 0.02;
const EDGE_FULL: f32 = 0.06;

/// sRGB error, reconstructing a pixel as a mix of its darkest and lightest
/// neighbor, over which that reconstruction goes from trusted to ignored
/// (e.g. a third color meeting the edge).
const MIX_ERROR_START: f32 = 0.03;
const MIX_ERROR_FULL: f32 = 0.1;

/// Channel span below which a channel can't tell the two colors apart and
/// borrows the other channels' coverage instead.
const MIN_CHANNEL_SPAN: f32 = 0.02;

/// Pixels in the window [`ColorFilter::apply_neighborhood`] reads: 5
/// wide, 3 tall.
pub const NEIGHBORHOOD_SIZE: usize = 15;

/// Largest coverage step between adjacent channels over which a pixel
/// goes from plausible subpixel text to real color. Measured `ClearType`
/// fringes step up to ~0.45; a vivid color between black and white steps
/// ~0.85.
const SUBPIXEL_STEP_START: f32 = 0.55;
const SUBPIXEL_STEP_FULL: f32 = 0.75;

/// sRGB range within which a window counts as flat and skips edge
/// handling; small enough to sit well below [`EDGE_START`].
const FLAT_RANGE: f32 = 0.01;

/// OKLab chroma below which a pixel has no meaningful hue.
const HUE_MIN_CHROMA: f32 = 0.03;

/// Chroma-weighted hue agreement (0-1) over which a window goes from
/// mixed-hue `ClearType` fringes to consistently colored content.
const HUE_AGREEMENT_START: f32 = 0.6;
const HUE_AGREEMENT_FULL: f32 = 0.85;

/// Coverage gamma for edges the theme turns light-on-dark, which read
/// thinner than the same coverage dark-on-light.
const INVERTED_TEXT_GAMMA: f32 = 1.4;

/// OKLab lightness by which the themed ink must exceed the themed paper
/// for [`INVERTED_TEXT_GAMMA`] to fully apply.
const INVERSION_FULL: f32 = 0.1;

/// Override tolerances are configured as OKLab distance times 100, so they
/// read like the familiar CIE ΔE scale (~2 is barely noticeable).
const TOLERANCE_SCALE: f32 = 100.0;

/// OKLab lightness over which a color goes from judged by the configured
/// saturation threshold to judged as a pale tint (see [`tint_threshold`]).
const TINT_LIGHTNESS_START: f32 = 0.8;
const TINT_LIGHTNESS_FULL: f32 = 0.9;

/// How much a pale tint's saturation threshold is scaled up, and the
/// saturation it's capped at so vivid light colors (yellow, amber, light
/// green) still keep their color.
const TINT_THRESHOLD_SCALE: f32 = 2.5;
const TINT_THRESHOLD_MAX: f32 = 0.4;

/// How far the window's most ink-like pixel reaches towards the estimated
/// ink over which it goes from a fringe to the ink itself.
const SOLID_INK_START: f32 = 0.75;
const SOLID_INK_FULL: f32 = 0.9;

/// Ratio of the least to the most covered channel, summed over the
/// window, over which an ink estimate goes from colored to neutral.
/// Measured: WPF `ClearType` on gray text 0.43+, on `#1976d2` links 0.37-.
const INK_BALANCE_COLORED: f32 = 0.38;
const INK_BALANCE_NEUTRAL: f32 = 0.5;

/// sRGB distance outside the gamut at which an ink estimate is no longer
/// taken as a real color.
const INK_OVERSHOOT_FULL: f32 = 0.1;

/// Largest channel gap between the paper pixel and the window's channel
/// extremes over which the paper goes from those extremes to the pixel.
const PAPER_ENVELOPE_START: f32 = 0.2;
const PAPER_ENVELOPE_FULL: f32 = 0.35;

/// Hue agreement against neutral below which the paper side is taken to
/// be a fringe, fully at `START`. Stricter than [`HUE_AGREEMENT_START`]:
/// colored paper with `ClearType` text on it agrees ~0.7.
const PAPER_FRINGE_AGREEMENT_START: f32 = 0.3;
const PAPER_FRINGE_AGREEMENT_FULL: f32 = 0.5;

/// sRGB distance between an override's `from` and the page below which
/// it isn't tried as ink: too close to tell coverage apart.
const KNOWN_INK_MIN_CONTRAST: f32 = 0.1;

/// Share of the way from an override's `from` to white (or black) the
/// page must reach in every channel to be taken as a page.
const KNOWN_INK_PAGE_REACH: f32 = 0.5;

/// sRGB amount any pixel lies past an override's `from`, over which the
/// window goes from that ink on the page to something else.
const KNOWN_INK_FIT_START: f32 = 0.03;
const KNOWN_INK_FIT_FULL: f32 = 0.08;

/// Most ink coverage in the window over which it goes from no evidence
/// of the ink to enough; keeps faint page-like colors from passing as
/// a trace of it.
const KNOWN_INK_EVIDENCE_START: f32 = 0.3;
const KNOWN_INK_EVIDENCE_FULL: f32 = 0.6;

/// OKLab chroma over which a color goes from untouched to fully snapped to
/// the palette, so near-grays and anti-aliasing don't pick up a hue.
const PALETTE_CHROMA_START: f32 = 0.02;
const PALETTE_CHROMA_FULL: f32 = 0.06;

/// Palette colors below this OKLab chroma are grays, which the ramp
/// handles; they are left out of hue snapping.
const PALETTE_MIN_CHROMA: f32 = 0.05;

/// How sharply palette snapping prefers the nearest hue: a palette color
/// 25° away weighs ~e^-1 of an exact match. Soft, so gradients between two
/// palette hues blend instead of banding.
const PALETTE_SHARPNESS: f32 = 10.7;

/// Smallest source lightness span between paper and ink that measured
/// levels are applied over; closer levels leave lightness as is.
const MIN_LEVELS_SPAN: f32 = 0.05;

/// Binary search steps when pulling an out-of-gamut color's chroma in;
/// leaves an error below 1/255.
const GAMUT_STEPS: u32 = 8;

/// An explicit color mapping applied on top of everything else.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorOverride {
  pub from: Color,
  pub to: Color,

  /// OKLab distance times 100 within which pixels blend towards `to`.
  pub tolerance: f32,
}

/// A gray ramp stop: grays as light as `from` become `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RampStop {
  pub from: Color,
  pub to: Color,
}

/// Settings of a [`ColorFilter`]. [`Default`] is the identity.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorFilterOptions {
  /// Gray ramp, by source lightness; empty disables it.
  pub ramp: Vec<RampStop>,

  /// Pixels more saturated than this (0-1) skip the ramp.
  pub saturation_threshold: f32,

  /// How far (0-1) colors skipping the ramp still take on its lightness,
  /// which keeps colored text readable when the ramp flips light and
  /// dark.
  pub accent_lightness: f32,

  /// Chroma multiplier; 0 is grayscale.
  pub saturation: f32,

  /// Chroma boost (-1-1) weighted towards dull colors.
  pub vibrance: f32,

  /// Hue rotation in degrees.
  pub hue_shift: f32,

  /// Colors whose hues colored pixels snap to; grays are ignored.
  pub palette: Vec<Color>,

  /// How far (0-1) colors move to the palette.
  pub palette_strength: f32,

  /// How far (0-1) snapped colors also take on the palette's lightness.
  pub palette_lightness: f32,

  /// Lightness multiplier.
  pub brightness: f32,

  /// Lightness contrast around mid-gray.
  pub contrast: f32,

  /// Minimum OKLab lightness difference (0-1) to the themed background
  /// that colors keep, capped at the difference they had originally.
  pub min_contrast: f32,

  /// Color temperature shift, from cool (-1) to warm (1).
  pub warmth: f32,

  pub overrides: Vec<ColorOverride>,
}

impl Default for ColorFilterOptions {
  fn default() -> Self {
    Self {
      ramp: Vec::new(),
      saturation_threshold: 0.15,
      accent_lightness: 0.0,
      saturation: 1.0,
      vibrance: 0.0,
      hue_shift: 0.0,
      palette: Vec::new(),
      palette_strength: 1.0,
      palette_lightness: 0.0,
      brightness: 1.0,
      contrast: 1.0,
      min_contrast: 0.0,
      warmth: 0.0,
      overrides: Vec::new(),
    }
  }
}

/// A validated per-pixel color transform, ready to upload to the GPU.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorFilter {
  /// Shared, since themes are cloned into every overlay.
  constants: Arc<FilterConstants>,
}

/// Constant buffer layout shared with `struct Filter` in the shader. Only
/// `float4`/`uint4` members, so HLSL packing matches `repr(C)`.
///
/// Colors are pre-converted to OKLab so neither side converts them per
/// pixel.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FilterConstants {
  /// Ramp stop, palette and override counts.
  counts: [u32; 4],

  /// Saturation threshold, accent lightness, saturation, vibrance.
  tone: [f32; 4],

  /// Hue shift cosine and sine, palette strength, palette lightness.
  hue: [f32; 4],

  /// Brightness, contrast, minimum contrast, and the lightness the
  /// source's paper ends up at (what `min_contrast` measures against).
  lightness: [f32; 4],

  /// Linear RGB gains for warmth; `w` is non-zero when enabled.
  warmth: [f32; 4],

  /// Per stop: `to` OKLab, source lightness in `w`; sorted by it.
  ramp: [[f32; 4]; MAX_RAMP_STOPS],

  /// Per color: OKLab, chroma in `w`.
  palette: [[f32; 4]; MAX_PALETTE_COLORS],

  /// Per override: `from` OKLab with the tolerance (as an OKLab distance)
  /// in `w`, then `to` OKLab.
  overrides: [[f32; 4]; MAX_COLOR_OVERRIDES * 2],

  /// Per override: `from` in sRGB, tried as the ink of anti-aliased edges
  /// (see [`ColorFilter::known_ink_remix`]).
  override_inks: [[f32; 4]; MAX_COLOR_OVERRIDES],
}

impl FilterConstants {
  const IDENTITY: Self = Self {
    counts: [0; 4],
    tone: [0.0, 0.0, 1.0, 0.0],
    hue: [1.0, 0.0, 0.0, 0.0],
    lightness: [1.0, 1.0, 0.0, 1.0],
    warmth: [1.0, 1.0, 1.0, 0.0],
    ramp: [[0.0; 4]; MAX_RAMP_STOPS],
    palette: [[0.0; 4]; MAX_PALETTE_COLORS],
    overrides: [[0.0; 4]; MAX_COLOR_OVERRIDES * 2],
    override_inks: [[0.0; 4]; MAX_COLOR_OVERRIDES],
  };

  /// Gray ramp, chroma and hue, palette, then brightness and contrast.
  /// `t` is the source lightness normalized between ink and paper.
  fn tone_map(&self, lab: [f32; 3], t: f32) -> [f32; 3] {
    let mut result = lab;

    if self.counts[0] > 0 {
      let chroma = lab[1].hypot(lab[2]);
      let saturation = (chroma / MAX_CHROMA).min(1.0);
      let threshold = tint_threshold(self.tone[0], lab[0]);
      let weight = ramp_weight(saturation, threshold);

      // The source's own chroma is carried over so faintly tinted grays
      // keep their tint.
      let stop = self.ramp_at(t);
      let ramped = [stop[0], stop[1] + lab[1], stop[2] + lab[2]];

      // Colors keep their hue, and take on as much of the ramp's
      // lightness as configured.
      let accent = [lerp(lab[0], stop[0], self.tone[1]), lab[1], lab[2]];
      result = lerp3(accent, ramped, weight);
    }

    // Chroma, with vibrance boosting dull colors more than vivid ones.
    let saturation = (result[1].hypot(result[2]) / MAX_CHROMA).min(1.0);
    let scale = self.tone[2]
      * (1.0 + self.tone[3] * (1.0 - saturation) * (1.0 - saturation));
    let a = result[1] * scale;
    let b = result[2] * scale;
    result[1] = a * self.hue[0] - b * self.hue[1];
    result[2] = a * self.hue[1] + b * self.hue[0];

    if self.counts[1] > 0 {
      result = self.snap_to_palette(result);
    }

    result[0] =
      ((result[0] - 0.5) * self.lightness[1] + 0.5) * self.lightness[0];
    result
  }

  /// OKLab color of the gray ramp at source lightness `t`, clamped to its
  /// first and last stop.
  fn ramp_at(&self, t: f32) -> [f32; 3] {
    let count = self.counts[0] as usize;
    let mut result = xyz(self.ramp[0]);

    for index in 1..count {
      let low = self.ramp[index - 1];
      let high = self.ramp[index];

      if t > low[3] {
        let progress = ((t - low[3]) / (high[3] - low[3])).min(1.0);
        result = lerp3(xyz(low), xyz(high), progress);
      }
    }

    result
  }

  /// Pulls a color towards the palette colors nearest in hue, weighted
  /// softly by hue distance.
  fn snap_to_palette(&self, lab: [f32; 3]) -> [f32; 3] {
    let chroma = lab[1].hypot(lab[2]);

    if chroma < 1e-4 {
      return lab;
    }

    let direction = [lab[1] / chroma, lab[2] / chroma];
    let mut weight_sum = 0.0;
    let mut direction_sum = [0.0_f32; 2];
    let mut chroma_sum = 0.0;
    let mut lightness_sum = 0.0;

    for entry in &self.palette[..self.counts[1] as usize] {
      let entry_direction = [entry[1] / entry[3], entry[2] / entry[3]];
      let similarity = direction[0] * entry_direction[0]
        + direction[1] * entry_direction[1];
      let weight = ((similarity - 1.0) * PALETTE_SHARPNESS).exp();

      weight_sum += weight;
      direction_sum[0] += entry_direction[0] * weight;
      direction_sum[1] += entry_direction[1] * weight;
      chroma_sum += entry[3] * weight;
      lightness_sum += entry[0] * weight;
    }

    let length = direction_sum[0].hypot(direction_sum[1]);
    let target_direction = if length > 1e-4 {
      [direction_sum[0] / length, direction_sum[1] / length]
    } else {
      direction
    };
    let target_chroma = chroma_sum / weight_sum;

    let amount = self.hue[2]
      * smoothstep(PALETTE_CHROMA_START, PALETTE_CHROMA_FULL, chroma);

    [
      lerp(lab[0], lightness_sum / weight_sum, amount * self.hue[3]),
      lerp(lab[1], target_direction[0] * target_chroma, amount),
      lerp(lab[2], target_direction[1] * target_chroma, amount),
    ]
  }

  /// Keeps `lab` at least `min_contrast` in lightness from the themed
  /// paper, capped at `source_contrast`, its distance from the source's
  /// paper: colors the app drew close to its background (panels, subtle
  /// borders) stay close.
  fn keep_contrast(
    &self,
    lab: [f32; 3],
    source_contrast: f32,
  ) -> [f32; 3] {
    let target = self.lightness[2].min(source_contrast);
    let paper = self.lightness[3];
    let difference = lab[0] - paper;

    if target <= 0.0 || difference.abs() >= target {
      return lab;
    }

    // Keep the side of the paper the color is on, unless there's no room
    // there.
    let mut side = if difference.abs() > 1e-4 {
      difference.signum()
    } else if paper < 0.5 {
      1.0
    } else {
      -1.0
    };

    if !(0.0..=1.0).contains(&(paper + side * target)) {
      side = -side;
    }

    [(paper + side * target).clamp(0.0, 1.0), lab[1], lab[2]]
  }

  /// Applies warmth in linear RGB, then pulls the color into the sRGB
  /// gamut.
  fn warm(&self, lab: [f32; 3]) -> [f32; 3] {
    let gains = self.warmth;
    let mut lab = lab;

    if gains[3] != 0.0 {
      let linear = oklab_to_linear(lab);
      lab = linear_to_oklab([
        linear[0] * gains[0],
        linear[1] * gains[1],
        linear[2] * gains[2],
      ]);
    }

    gamut_clip(lab)
  }
}

/// OKLab lightness of the source window's paper (its main background) and
/// ink (its text), which the gray ramp treats as white and black.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceLevels {
  pub paper: f32,
  pub ink: f32,
}

impl Default for SourceLevels {
  /// Black text on white.
  fn default() -> Self {
    Self {
      paper: 1.0,
      ink: 0.0,
    }
  }
}

impl SourceLevels {
  /// Where `lightness` sits between ink (0) and paper (1).
  fn normalize(self, lightness: f32) -> f32 {
    let span = self.paper - self.ink;

    if span.abs() < MIN_LEVELS_SPAN {
      return lightness.clamp(0.0, 1.0);
    }

    ((lightness - self.ink) / span).clamp(0.0, 1.0)
  }
}

impl ColorFilter {
  /// Validates and compiles a filter.
  pub fn new(options: &ColorFilterOptions) -> crate::Result<Self> {
    validate_ranges(options)?;

    let stops = ramp_constants(&options.ramp)?;
    let palette = palette_constants(&options.palette)?;
    let overrides = override_constants(&options.overrides)?;

    let mut constants = FilterConstants::IDENTITY;
    constants.ramp[..stops.len()].copy_from_slice(&stops);
    constants.palette[..palette.len()].copy_from_slice(&palette);
    constants.overrides[..overrides.len()].copy_from_slice(&overrides);

    for (ink, color_override) in
      constants.override_inks.iter_mut().zip(&options.overrides)
    {
      *ink = with_w(to_rgb(color_override.from), 0.0);
    }

    // All bounded by the checks above.
    #[allow(clippy::cast_possible_truncation)]
    {
      constants.counts = [
        stops.len() as u32,
        palette.len() as u32,
        (overrides.len() / 2) as u32,
        0,
      ];
    }

    constants.tone = [
      options.saturation_threshold,
      options.accent_lightness,
      options.saturation,
      options.vibrance,
    ];

    let (sin, cos) = options.hue_shift.to_radians().sin_cos();
    constants.hue = [
      cos,
      sin,
      options.palette_strength,
      options.palette_lightness,
    ];

    let warmth = options.warmth;
    constants.warmth = if warmth > 0.0 {
      [1.0, 1.0 - 0.18 * warmth, 1.0 - 0.45 * warmth, 1.0]
    } else if warmth < 0.0 {
      [1.0 + 0.25 * warmth, 1.0 + 0.1 * warmth, 1.0, 1.0]
    } else {
      [1.0, 1.0, 1.0, 0.0]
    };

    constants.lightness = [options.brightness, options.contrast, 0.0, 0.0];

    // Where the source's paper lands, which `min_contrast` measures
    // against.
    let paper = constants.warm(constants.tone_map([1.0, 0.0, 0.0], 1.0));
    constants.lightness[2] = options.min_contrast;
    constants.lightness[3] = paper[0];

    Ok(Self {
      constants: Arc::new(constants),
    })
  }

  #[cfg(test)]
  fn constants(&self) -> &FilterConstants {
    &self.constants
  }

  /// Maps one straight-alpha sRGB color (components in `0.0..=1.0`)
  /// through the filter, for a window with the given `levels`.
  #[must_use]
  pub fn apply(&self, srgb: [f32; 3], levels: SourceLevels) -> [f32; 3] {
    let c = &*self.constants;
    let lab = srgb_to_oklab(srgb);
    let mut result = c.tone_map(lab, levels.normalize(lab[0]));

    result = c.keep_contrast(result, (lab[0] - levels.paper).abs());
    result = c.warm(result);

    let mut best_weight = 0.0;
    let mut best_to = [0.0; 3];

    // Matched against the original color, so overrides name the colors
    // the app actually draws.
    for index in 0..c.counts[2] as usize {
      let from = c.overrides[index * 2];
      let to = c.overrides[index * 2 + 1];
      let weight = override_weight(distance(lab, xyz(from)), from[3]);

      if weight > best_weight {
        best_weight = weight;
        best_to = xyz(to);
      }
    }

    result = lerp3(result, best_to, best_weight);
    oklab_to_srgb(result).map(|channel| channel.clamp(0.0, 1.0))
  }

  /// [`apply`](Self::apply) for a single opaque color, keeping its alpha.
  #[must_use]
  pub fn apply_color(&self, color: Color, levels: SourceLevels) -> Color {
    let [r, g, b] = self.apply(to_rgb(color), levels).map(to_byte);
    Color {
      r,
      g,
      b,
      a: color.a,
    }
  }

  /// Maps the center of a 5×3 neighborhood (row-major, straight-alpha
  /// sRGB) through the theme, keeping anti-aliased edges intact. Five
  /// wide, because `ClearType` fringes reach two pixels from the ink.
  ///
  /// An edge pixel is a mix of the colors on either side of it, so it is
  /// re-mixed from those colors' *themed* values at the same coverage,
  /// rather than themed as a color of its own.
  ///
  /// Subpixel (`ClearType`) fringes are recognized as text by how gently
  /// coverage steps between adjacent channels, and re-mixed as grayscale:
  /// `ClearType` is tuned for dark-on-light, and replaying its color
  /// fringes light-on-dark makes them loud. A vivid pixel between black
  /// and white steps harder and is left as real color. Where the theme
  /// flips dark ink to light, coverage is boosted the way native text
  /// renderers do, since light-on-dark text otherwise reads thinner.
  /// Fringes the re-mix can't explain are themed by lightness alone.
  #[must_use]
  pub fn apply_neighborhood(
    &self,
    pixels: &[[f32; 3]; NEIGHBORHOOD_SIZE],
    levels: SourceLevels,
  ) -> [f32; 3] {
    let center = pixels[NEIGHBORHOOD_SIZE / 2];
    let themed_center = self.apply(center, levels);

    // Most of a window is flat, and the edge path below leaves flat areas
    // alone anyway: skip it on the cheap.
    let mut range = 0.0_f32;
    for pixel in pixels {
      for c in 0..3 {
        range = range.max((pixel[c] - center[c]).abs());
      }
    }

    if range < FLAT_RANGE {
      return themed_center;
    }

    // How consistent the window's hues are against neutral; opposing
    // `ClearType` fringes cancel out. Judged against neutral rather than
    // the paper: text too dense for the window to show any paper leaves
    // only fringes to pick it from.
    let neutral_agreement = hue_agreement(pixels, [1.0; 3], 0.0);
    let fringes = 1.0
      - smoothstep(
        HUE_AGREEMENT_START,
        HUE_AGREEMENT_FULL,
        neutral_agreement,
      );

    let (dark, light) = edge_colors(
      pixels,
      1.0
        - smoothstep(
          PAPER_FRINGE_AGREEMENT_START,
          PAPER_FRINGE_AGREEMENT_FULL,
          neutral_agreement,
        ),
    );

    let edge = smoothstep(
      EDGE_START,
      EDGE_FULL,
      distance(srgb_to_oklab(dark), srgb_to_oklab(light)),
    );

    if edge <= 0.0 {
      return themed_center;
    }

    // Per-channel coverage of `dark` over `light`, with channels too close
    // to call borrowing the mean of the others.
    let mut coverage = [0.0; 3];
    let mut valid = [false; 3];
    let mut coverage_sum = 0.0;
    let mut valid_count = 0.0;

    for c in 0..3 {
      let span = dark[c] - light[c];

      if span.abs() > MIN_CHANNEL_SPAN {
        coverage[c] = ((center[c] - light[c]) / span).clamp(0.0, 1.0);
        valid[c] = true;
        coverage_sum += coverage[c];
        valid_count += 1.0;
      }
    }

    let mean_coverage = if valid_count > 0.0 {
      coverage_sum / valid_count
    } else {
      0.0
    };

    for c in 0..3 {
      if !valid[c] {
        coverage[c] = mean_coverage;
      }
    }

    // The `ClearType` filter smears ink across neighboring subpixels, so a
    // fringe's adjacent channels differ only moderately; real color
    // between the same two colors jumps.
    let channel_step = (coverage[0] - coverage[1])
      .abs()
      .max((coverage[1] - coverage[2]).abs());
    let subpixel = 1.0
      - smoothstep(SUBPIXEL_STEP_START, SUBPIXEL_STEP_FULL, channel_step);

    // How well the pixel is explained as a mix of `dark` and `light`,
    // allowing per-channel coverage only for subpixel text.
    let mut error_sq = 0.0;

    for c in 0..3 {
      let reconstructed = lerp(
        light[c],
        dark[c],
        lerp(mean_coverage, coverage[c], subpixel),
      );
      error_sq +=
        (center[c] - reconstructed) * (center[c] - reconstructed);
    }

    let fit =
      1.0 - smoothstep(MIX_ERROR_START, MIX_ERROR_FULL, error_sq.sqrt());
    let themed_dark = self.apply(dark, levels);
    let themed_light = self.apply(light, levels);

    let inverted = smoothstep(
      0.0,
      INVERSION_FULL,
      srgb_to_oklab(themed_dark)[0] - srgb_to_oklab(themed_light)[0],
    );
    let coverage = lerp(
      mean_coverage,
      mean_coverage.powf(1.0 / INVERTED_TEXT_GAMMA),
      inverted,
    );

    let remixed = lerp3(themed_light, themed_dark, coverage);

    // A fringe the re-mix can't explain would otherwise keep its
    // `ClearType` color, being too saturated for the ramp. Its lightness
    // still says how much ink it holds, so it is themed as that gray.
    let unexplained = lerp3(
      themed_center,
      self.apply(neutral(center), levels),
      edge * fringes,
    );
    let estimated = lerp3(unexplained, remixed, edge * fit);

    let (known, known_weight) = self.known_ink_remix(pixels, levels);
    lerp3(estimated, known, known_weight)
  }

  /// Re-mixes the window's center as one of the overrides' `from` colors
  /// used as ink over the page, weighted by how well that explains the
  /// whole window.
  ///
  /// The estimate in [`apply_neighborhood`](Self::apply_neighborhood)
  /// has to guess the ink, and thin colored strokes never show it
  /// whole. An override's `from` is a color the user says the window
  /// uses, so where every pixel lies between it and the page channel by
  /// channel (as `ClearType` fringes do), the ink is known exactly.
  /// Black text, images, and other colors fall outside and are left to
  /// the estimate.
  fn known_ink_remix(
    &self,
    pixels: &[[f32; 3]; NEIGHBORHOOD_SIZE],
    levels: SourceLevels,
  ) -> ([f32; 3], f32) {
    let c = &*self.constants;
    let center = pixels[NEIGHBORHOOD_SIZE / 2];
    let mut best = ([0.0; 3], 0.0);

    for ink in &c.override_inks[..c.counts[2] as usize] {
      let ink = xyz(*ink);

      // The page is lighter than the ink in every channel, or darker in
      // every channel; each channel's extreme over the window is its
      // level, since ink only ever pulls a channel towards itself.
      for page_is_light in [true, false] {
        let mut paper = ink;
        for pixel in pixels {
          for ch in 0..3 {
            paper[ch] = if page_is_light {
              paper[ch].max(pixel[ch])
            } else {
              paper[ch].min(pixel[ch])
            };
          }
        }

        // A page reaches at least partway from the ink towards white (or
        // black) in every channel; else the window is something darker
        // (or lighter) than a page, e.g. an image, not ink on one.
        let reaches = (0..3).all(|ch| {
          if page_is_light {
            paper[ch] >= lerp(ink[ch], 1.0, KNOWN_INK_PAGE_REACH)
          } else {
            paper[ch] <= lerp(ink[ch], 0.0, KNOWN_INK_PAGE_REACH)
          }
        });

        if !reaches || distance(paper, ink) < KNOWN_INK_MIN_CONTRAST {
          continue;
        }

        // How far any pixel lies past the ink (the page side can't be
        // passed by construction), and how much ink the window holds.
        let mut violation = 0.0_f32;
        let mut most_ink = 0.0_f32;

        for pixel in pixels {
          for ch in 0..3 {
            // Where ink and page are the same level, any deviation from
            // it is something else. (`signum` alone would call `0.0`
            // positive, and the shader's `sign` calls it 0.)
            let span = ink[ch] - paper[ch];
            let past = if span == 0.0 {
              (pixel[ch] - ink[ch]).abs()
            } else {
              (pixel[ch] - ink[ch]) * span.signum()
            };
            violation = violation.max(past);
          }

          most_ink = most_ink.max(known_ink_coverage(*pixel, ink, paper));
        }

        let weight = (1.0
          - smoothstep(
            KNOWN_INK_FIT_START,
            KNOWN_INK_FIT_FULL,
            violation,
          ))
          * smoothstep(
            KNOWN_INK_EVIDENCE_START,
            KNOWN_INK_EVIDENCE_FULL,
            most_ink,
          );

        if weight > best.1 {
          let themed_ink = self.apply(ink, levels);
          let themed_paper = self.apply(paper, levels);
          let coverage = known_ink_coverage(center, ink, paper);

          let inverted = smoothstep(
            0.0,
            INVERSION_FULL,
            srgb_to_oklab(themed_ink)[0] - srgb_to_oklab(themed_paper)[0],
          );
          let coverage = lerp(
            coverage,
            coverage.powf(1.0 / INVERTED_TEXT_GAMMA),
            inverted,
          );

          best = (lerp3(themed_paper, themed_ink, coverage), weight);
        }
      }
    }

    best
  }
}

/// A kind of UI element, as reported by the app's accessibility tree,
/// that a theme can render differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiElementKind {
  Edit,
  Document,
  Button,
  Hyperlink,
  CheckBox,
  RadioButton,
  ComboBox,
  ListItem,
  TreeItem,
  TabItem,
  MenuItem,
  DataItem,
  Header,
  ToolBar,
  StatusBar,
  TitleBar,
}

impl UiElementKind {
  pub const ALL: [Self; 16] = [
    Self::Edit,
    Self::Document,
    Self::Button,
    Self::Hyperlink,
    Self::CheckBox,
    Self::RadioButton,
    Self::ComboBox,
    Self::ListItem,
    Self::TreeItem,
    Self::TabItem,
    Self::MenuItem,
    Self::DataItem,
    Self::Header,
    Self::ToolBar,
    Self::StatusBar,
    Self::TitleBar,
  ];
}

/// How a theme renders one kind of UI element.
#[derive(Clone, Debug, PartialEq)]
pub enum ElementTreatment {
  /// The app's own colors, unchanged.
  Original,

  /// A filter of its own instead of the theme's.
  Filter(ColorFilter),
}

/// A UI element's bounds within the captured frame, in physical pixels
/// (right and bottom exclusive).
#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ElementRect {
  pub kind: UiElementKind,
  pub ltrb: [i32; 4],
}

/// Settings of a [`ColorTheme`].
#[derive(Clone, Debug, PartialEq)]
pub struct ColorThemeOptions {
  pub filter: ColorFilter,

  /// Per element kind, how to render it instead of with `filter`.
  pub elements: Vec<(UiElementKind, ElementTreatment)>,

  /// Measure the window's own paper and ink colors instead of assuming
  /// black on white.
  pub detect_colors: bool,

  /// Leave the window unchanged while its paper is dark.
  pub skip_if_dark: bool,
}

/// Everything a themed window renders with: a [`ColorFilter`], per-element
/// treatments, and whether to measure the window's colors.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorTheme {
  options: ColorThemeOptions,

  /// Distinct element filters, in slot order after the theme's own.
  element_filters: Vec<ColorFilter>,
}

impl ColorTheme {
  /// Validates a theme.
  pub fn new(options: ColorThemeOptions) -> crate::Result<Self> {
    let mut element_filters: Vec<ColorFilter> = Vec::new();

    for (index, (kind, treatment)) in options.elements.iter().enumerate() {
      if options.elements[..index]
        .iter()
        .any(|(other, _)| other == kind)
      {
        return Err(crate::Error::Platform(format!(
          "UI element {kind:?} is configured twice."
        )));
      }

      if let ElementTreatment::Filter(filter) = treatment {
        if filter != &options.filter && !element_filters.contains(filter) {
          element_filters.push(filter.clone());
        }
      }
    }

    if element_filters.len() > MAX_ELEMENT_FILTERS {
      return Err(crate::Error::Platform(format!(
        "UI elements can use at most {MAX_ELEMENT_FILTERS} distinct \
         themes, got {}.",
        element_filters.len()
      )));
    }

    Ok(Self {
      options,
      element_filters,
    })
  }

  /// Maps `color` through the theme's own filter, as for black-on-white
  /// content; for fills, where no measured levels apply.
  #[must_use]
  pub fn apply_color(&self, color: Color) -> Color {
    self
      .options
      .filter
      .apply_color(color, SourceLevels::default())
  }

  #[must_use]
  pub fn filter(&self) -> &ColorFilter {
    &self.options.filter
  }

  #[must_use]
  pub fn detect_colors(&self) -> bool {
    self.options.detect_colors
  }

  #[must_use]
  pub fn skip_if_dark(&self) -> bool {
    self.options.skip_if_dark
  }

  /// Whether the window's pixels need to be measured.
  #[must_use]
  pub fn needs_analysis(&self) -> bool {
    self.options.detect_colors || self.options.skip_if_dark
  }

  /// Element kinds rendered differently, with the filter slot each uses
  /// (see [`Self::filter_constants`]), or [`SLOT_ORIGINAL`].
  #[cfg(any(target_os = "windows", test))]
  pub(crate) fn element_slots(&self) -> Vec<(UiElementKind, u32)> {
    self
      .options
      .elements
      .iter()
      .map(|(kind, treatment)| {
        let slot = match treatment {
          ElementTreatment::Original => SLOT_ORIGINAL,
          ElementTreatment::Filter(filter) => self
            .element_filters
            .iter()
            .position(|other| other == filter)
            // Bounded by `MAX_ELEMENT_FILTERS`.
            .map_or(0, |index| u32::try_from(index + 1).unwrap_or(0)),
        };

        (*kind, slot)
      })
      .collect()
  }

  /// The filter in each slot: the theme's own first, then its element
  /// filters. Unused slots are the identity.
  #[cfg(any(target_os = "windows", test))]
  pub(crate) fn filter_constants(
    &self,
  ) -> [FilterConstants; MAX_FILTER_SLOTS] {
    let mut slots = [FilterConstants::IDENTITY; MAX_FILTER_SLOTS];
    slots[0] = *self.options.filter.constants;

    for (slot, filter) in slots[1..].iter_mut().zip(&self.element_filters)
    {
      *slot = *filter.constants;
    }

    slots
  }
  /// The regions `elements` render with, as `(ltrb, slot)`: clipped to
  /// the `size` of the frame, largest first so nested elements win in the
  /// shader, and capped at [`MAX_REGIONS`] (dropping the smallest).
  #[cfg(any(target_os = "windows", test))]
  pub(crate) fn element_regions(
    &self,
    elements: &[ElementRect],
    size: (u32, u32),
  ) -> Vec<([i32; 4], u32)> {
    let slots = self.element_slots();
    let width = i32::try_from(size.0).unwrap_or(i32::MAX);
    let height = i32::try_from(size.1).unwrap_or(i32::MAX);

    let mut regions = elements
      .iter()
      .filter_map(|element| {
        let slot = slots
          .iter()
          .find(|(kind, _)| *kind == element.kind)
          .map(|(_, slot)| *slot)?;

        let [left, top, right, bottom] = element.ltrb;
        let clipped = [
          left.max(0),
          top.max(0),
          right.min(width),
          bottom.min(height),
        ];

        (clipped[0] < clipped[2] && clipped[1] < clipped[3])
          .then_some((clipped, slot))
      })
      .collect::<Vec<_>>();

    let area = |ltrb: &[i32; 4]| {
      i64::from(ltrb[2] - ltrb[0]) * i64::from(ltrb[3] - ltrb[1])
    };
    regions.sort_by_key(|(ltrb, _)| std::cmp::Reverse(area(ltrb)));
    regions.truncate(MAX_REGIONS);
    regions
  }
}

/// Mean coverage of `ink` over `paper` in `pixel`, across the channels
/// where the two differ.
fn known_ink_coverage(
  pixel: [f32; 3],
  ink: [f32; 3],
  paper: [f32; 3],
) -> f32 {
  let mut sum = 0.0;
  let mut count = 0.0;

  for ch in 0..3 {
    let span = paper[ch] - ink[ch];

    if span.abs() > MIN_CHANNEL_SPAN {
      sum += ((paper[ch] - pixel[ch]) / span).clamp(0.0, 1.0);
      count += 1.0;
    }
  }

  if count > 0.0 {
    sum / count
  } else {
    0.0
  }
}

/// The two colors the window's edge pixels are a mix of, as `(dark,
/// light)`: its darkest and lightest pixels, with the ink side corrected
/// by [`estimate_ink`].
///
/// Hairline text never fully covers a pixel, so the ink side can be a
/// fringe rather than the ink. Paper covers more of the window than ink,
/// so it is the side the mean lightness sits closer to. Between two close
/// stems no paper shows at all, so the paper side is neutralized as far
/// as `paper_fringes` says it is one.
fn edge_colors(
  pixels: &[[f32; 3]; NEIGHBORHOOD_SIZE],
  paper_fringes: f32,
) -> ([f32; 3], [f32; 3]) {
  let center = pixels[NEIGHBORHOOD_SIZE / 2];
  let mut dark = center;
  let mut light = center;
  let mut dark_lightness = srgb_to_oklab(center)[0];
  let mut light_lightness = dark_lightness;
  let mut lightness_sum = 0.0;

  for pixel in pixels {
    let lightness = srgb_to_oklab(*pixel)[0];
    lightness_sum += lightness;

    if lightness < dark_lightness {
      dark = *pixel;
      dark_lightness = lightness;
    }

    if lightness > light_lightness {
      light = *pixel;
      light_lightness = lightness;
    }
  }

  #[allow(clippy::cast_precision_loss)]
  let mean_lightness = lightness_sum / NEIGHBORHOOD_SIZE as f32;
  let paper_is_light =
    (light_lightness - mean_lightness) < (mean_lightness - dark_lightness);
  let (paper, extreme) = if paper_is_light {
    (light, 0.0)
  } else {
    (dark, 1.0)
  };

  // Anti-aliasing only moves channels from the paper towards the ink, so
  // each channel's paper level is its extreme over the window. That holds
  // where dense text leaves only fringes to pick the paper from; a window
  // spanning two unrelated colors lands far from any pixel instead.
  let mut envelope = paper;

  for pixel in pixels {
    for c in 0..3 {
      envelope[c] = if paper_is_light {
        envelope[c].max(pixel[c])
      } else {
        envelope[c].min(pixel[c])
      };
    }
  }

  let envelope_gap = (0..3)
    .map(|c| (envelope[c] - paper[c]).abs())
    .fold(0.0_f32, f32::max);
  let paper = lerp3(
    envelope,
    paper,
    smoothstep(PAPER_ENVELOPE_START, PAPER_ENVELOPE_FULL, envelope_gap),
  );

  let paper = lerp3(
    paper,
    [channel_extreme(paper, 1.0 - extreme); 3],
    paper_fringes,
  );

  let mixed_hues = 1.0
    - smoothstep(
      HUE_AGREEMENT_START,
      HUE_AGREEMENT_FULL,
      hue_agreement(pixels, paper, extreme),
    );

  if paper_is_light {
    (estimate_ink(pixels, dark, paper, 0.0, mixed_hues), paper)
  } else {
    (paper, estimate_ink(pixels, light, paper, 1.0, mixed_hues))
  }
}

/// `color`'s lightest channel for `extreme` 1, its darkest for 0.
fn channel_extreme(color: [f32; 3], extreme: f32) -> f32 {
  if extreme > 0.5 {
    color[0].max(color[1]).max(color[2])
  } else {
    color[0].min(color[1]).min(color[2])
  }
}

/// How consistently the window's pixels deviate in hue from a plain mix of
/// `paper` and neutral ink (`extreme`), chroma-weighted: near 1 for
/// colored text or shapes, low for `ClearType` fringes, which deviate in
/// opposing hues. Measured as deviations so the paper's own color, however
/// saturated, doesn't count.
fn hue_agreement(
  pixels: &[[f32; 3]; NEIGHBORHOOD_SIZE],
  paper: [f32; 3],
  extreme: f32,
) -> f32 {
  let paper_mean = (paper[0] + paper[1] + paper[2]) / 3.0;
  let span = extreme - paper_mean;

  let mut hue_sum = [0.0_f32; 2];
  let mut chroma_sum = 0.0;

  for pixel in pixels {
    let mean = (pixel[0] + pixel[1] + pixel[2]) / 3.0;
    let t = if span.abs() > MIN_CHANNEL_SPAN {
      ((mean - paper_mean) / span).clamp(0.0, 1.0)
    } else {
      0.0
    };

    let lab = srgb_to_oklab(*pixel);
    let model = srgb_to_oklab(lerp3(paper, [extreme; 3], t));
    let deviation = [lab[1] - model[1], lab[2] - model[2]];
    let chroma = deviation[0].hypot(deviation[1]);

    if chroma > HUE_MIN_CHROMA {
      hue_sum[0] += deviation[0];
      hue_sum[1] += deviation[1];
      chroma_sum += chroma;
    }
  }

  if chroma_sum > 0.0 {
    hue_sum[0].hypot(hue_sum[1]) / chroma_sum
  } else {
    1.0
  }
}

/// The ink the window's edge pixels are a mix of, against `paper`, when
/// `endpoint` (its most ink-like pixel) may only be a fringe of it.
///
/// Thin strokes never fully cover a pixel, so the endpoint is too light
/// and, under `ClearType`, the wrong hue. Anti-aliasing shifts coverage
/// between pixels without changing the total, so the window's summed
/// deviation from `paper` points along the ink's color; scaled until the
/// most-covered channel is full, it is the palest ink that explains every
/// pixel. Neutral ink's fringes aren't energy-balanced across channels, so
/// where the window looks like neutral ink, the endpoint's most-covered
/// channel is taken as a neutral ink level instead.
fn estimate_ink(
  pixels: &[[f32; 3]; NEIGHBORHOOD_SIZE],
  endpoint: [f32; 3],
  paper: [f32; 3],
  extreme: f32,
  mixed_hues: f32,
) -> [f32; 3] {
  let mut coverage = [0.0; 3];

  for c in 0..3 {
    let span = paper[c] - extreme;

    if span.abs() <= MIN_CHANNEL_SPAN {
      return endpoint;
    }

    coverage[c] = ((paper[c] - endpoint[c]) / span).clamp(0.0, 1.0);
  }

  let mut deviation_sum = [0.0_f32; 3];

  for pixel in pixels {
    for c in 0..3 {
      deviation_sum[c] += pixel[c] - paper[c];
    }
  }

  let mut full_scale = 0.0_f32;

  for pixel in pixels {
    for c in 0..3 {
      if deviation_sum[c].abs() > MIN_CHANNEL_SPAN {
        full_scale =
          full_scale.max((pixel[c] - paper[c]) / deviation_sum[c]);
      }
    }
  }

  let estimated =
    [0, 1, 2].map(|c| paper[c] + deviation_sum[c] * full_scale);

  // How far `endpoint` reaches along paper -> estimate. Where it is full
  // ink already, its own color is exact, while the estimate carries the
  // fringes' channel imbalance (enough to miss an override).
  let mut along = 0.0;
  let mut length_sq = 0.0;

  for c in 0..3 {
    along += (endpoint[c] - paper[c]) * (estimated[c] - paper[c]);
    length_sq += (estimated[c] - paper[c]) * (estimated[c] - paper[c]);
  }

  let reach = if length_sq > 0.0 {
    along / length_sq
  } else {
    1.0
  };
  let colored = lerp3(
    estimated,
    endpoint,
    smoothstep(SOLID_INK_START, SOLID_INK_FULL, reach),
  );

  let channel_step = (coverage[0] - coverage[1])
    .abs()
    .max((coverage[1] - coverage[2]).abs());
  let fringe = 1.0
    - smoothstep(SUBPIXEL_STEP_START, SUBPIXEL_STEP_FULL, channel_step);
  let neutral =
    lerp3(endpoint, [channel_extreme(endpoint, extreme); 3], fringe);

  // `ClearType` fringes of neutral ink disagree in hue, but so do those of
  // colored ink under WPF's strong filter. What sets colored ink apart is
  // a strongly unbalanced summed deviation that still lands on a real
  // color; neutral fringes only unbalance it mildly, or overshoot the
  // gamut when the window catches one side of a stroke.
  let mut ratio_min = f32::MAX;
  let mut ratio_max = 0.0_f32;
  let mut overshoot = 0.0_f32;

  for c in 0..3 {
    let ratio = deviation_sum[c] / (extreme - paper[c]);
    ratio_min = ratio_min.min(ratio);
    ratio_max = ratio_max.max(ratio);
    overshoot = overshoot.max(-estimated[c]).max(estimated[c] - 1.0);
  }

  let balance = if ratio_max > 0.0 {
    ratio_min / ratio_max
  } else {
    1.0
  };
  let colored_ink = (1.0
    - smoothstep(INK_BALANCE_COLORED, INK_BALANCE_NEUTRAL, balance))
    * (1.0 - smoothstep(0.0, INK_OVERSHOOT_FULL, overshoot));

  lerp3(colored, neutral, mixed_hues * (1.0 - colored_ink))
    .map(|c| c.clamp(0.0, 1.0))
}

/// The gray with `srgb`'s OKLab lightness.
fn neutral(srgb: [f32; 3]) -> [f32; 3] {
  let lightness = srgb_to_oklab(srgb)[0];
  oklab_to_srgb([lightness, 0.0, 0.0])
}

fn validate_ranges(options: &ColorFilterOptions) -> crate::Result<()> {
  for (name, value, range) in [
    (
      "saturation_threshold",
      options.saturation_threshold,
      0.0..=1.0,
    ),
    ("accent_lightness", options.accent_lightness, 0.0..=1.0),
    ("saturation", options.saturation, 0.0..=4.0),
    ("vibrance", options.vibrance, -1.0..=1.0),
    ("hue_shift", options.hue_shift, -360.0..=360.0),
    ("palette_strength", options.palette_strength, 0.0..=1.0),
    ("palette_lightness", options.palette_lightness, 0.0..=1.0),
    ("brightness", options.brightness, 0.0..=2.0),
    ("contrast", options.contrast, 0.0..=3.0),
    ("min_contrast", options.min_contrast, 0.0..=1.0),
    ("warmth", options.warmth, -1.0..=1.0),
  ] {
    if !range.contains(&value) {
      return Err(crate::Error::Platform(format!(
        "`{name}` {value} must be between {} and {}.",
        range.start(),
        range.end()
      )));
    }
  }

  Ok(())
}

/// Ramp stops as `to` OKLab with the `from` lightness in `w`, sorted by
/// it.
fn ramp_constants(ramp: &[RampStop]) -> crate::Result<Vec<[f32; 4]>> {
  if ramp.len() == 1 || ramp.len() > MAX_RAMP_STOPS {
    return Err(crate::Error::Platform(format!(
      "A gray ramp needs 2 to {MAX_RAMP_STOPS} stops, got {}.",
      ramp.len()
    )));
  }

  let mut stops = ramp
    .iter()
    .map(|stop| {
      with_w(
        srgb_to_oklab(to_rgb(stop.to)),
        srgb_to_oklab(to_rgb(stop.from))[0],
      )
    })
    .collect::<Vec<_>>();
  stops.sort_by(|a, b| a[3].total_cmp(&b[3]));

  if stops.windows(2).any(|pair| pair[1][3] - pair[0][3] < 0.01) {
    return Err(crate::Error::Platform(
      "Gray ramp stops need distinct `from` lightness.".to_string(),
    ));
  }

  Ok(stops)
}

/// Non-gray palette colors as OKLab with their chroma in `w`.
fn palette_constants(palette: &[Color]) -> crate::Result<Vec<[f32; 4]>> {
  let entries = palette
    .iter()
    .map(|color| {
      let lab = srgb_to_oklab(to_rgb(*color));
      with_w(lab, lab[1].hypot(lab[2]))
    })
    .filter(|entry| entry[3] >= PALETTE_MIN_CHROMA)
    .collect::<Vec<_>>();

  if entries.len() > MAX_PALETTE_COLORS {
    return Err(crate::Error::Platform(format!(
      "A palette supports at most {MAX_PALETTE_COLORS} non-gray colors, got {}.",
      entries.len()
    )));
  }

  if !palette.is_empty() && entries.is_empty() {
    return Err(crate::Error::Platform(
      "A palette needs at least one non-gray color.".to_string(),
    ));
  }

  Ok(entries)
}

/// Per override, `from` OKLab with the tolerance in `w`, then `to`.
fn override_constants(
  overrides: &[ColorOverride],
) -> crate::Result<Vec<[f32; 4]>> {
  if overrides.len() > MAX_COLOR_OVERRIDES {
    return Err(crate::Error::Platform(format!(
      "A color theme supports at most {MAX_COLOR_OVERRIDES} overrides, got {}.",
      overrides.len()
    )));
  }

  let mut entries = Vec::with_capacity(overrides.len() * 2);

  for color_override in overrides {
    if !color_override.tolerance.is_finite()
      || color_override.tolerance < 0.0
    {
      return Err(crate::Error::Platform(format!(
        "Override tolerance {} must be a non-negative number.",
        color_override.tolerance
      )));
    }

    entries.push(with_w(
      srgb_to_oklab(to_rgb(color_override.from)),
      color_override.tolerance / TOLERANCE_SCALE,
    ));
    entries.push(with_w(srgb_to_oklab(to_rgb(color_override.to)), 0.0));
  }

  Ok(entries)
}

/// How much of the gray ramp applies at `saturation`: fully below half the
/// threshold, easing out to none at the threshold, so the edge of an image
/// doesn't band.
fn ramp_weight(saturation: f32, threshold: f32) -> f32 {
  if saturation >= threshold {
    0.0
  } else {
    1.0 - smoothstep(threshold * 0.5, threshold, saturation)
  }
}

/// Saturation threshold for a color of OKLab `lightness`: `threshold`,
/// raised for light colors.
///
/// Pale tints (hover and selection highlights, tinted panels) are surfaces
/// that text sits on. Leaving one light while the text on it turns light
/// makes that text unreadable, so they are ramped like grays.
fn tint_threshold(threshold: f32, lightness: f32) -> f32 {
  let tint = threshold
    .max((threshold * TINT_THRESHOLD_SCALE).min(TINT_THRESHOLD_MAX));
  lerp(
    threshold,
    tint,
    smoothstep(TINT_LIGHTNESS_START, TINT_LIGHTNESS_FULL, lightness),
  )
}

/// How much of an override applies at OKLab `distance` from its `from`
/// color: fully at an exact match, easing out to none at `tolerance`.
fn override_weight(distance: f32, tolerance: f32) -> f32 {
  if distance >= tolerance {
    0.0
  } else {
    1.0 - smoothstep(0.0, tolerance, distance)
  }
}

/// Clamps lightness and, if the color is still outside sRGB, reduces its
/// chroma until it fits, keeping lightness and hue.
fn gamut_clip(lab: [f32; 3]) -> [f32; 3] {
  let lab = [lab[0].clamp(0.0, 1.0), lab[1], lab[2]];

  if in_gamut(oklab_to_linear(lab)) {
    return lab;
  }

  let mut low = 0.0;
  let mut high = 1.0;

  for _ in 0..GAMUT_STEPS {
    let mid = f32::midpoint(low, high);

    if in_gamut(oklab_to_linear([lab[0], lab[1] * mid, lab[2] * mid])) {
      low = mid;
    } else {
      high = mid;
    }
  }

  [lab[0], lab[1] * low, lab[2] * low]
}

fn in_gamut(linear: [f32; 3]) -> bool {
  linear.iter().all(|c| (-1e-4..=1.0 + 1e-4).contains(c))
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
  let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
  t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
  [
    lerp(a[0], b[0], t),
    lerp(a[1], b[1], t),
    lerp(a[2], b[2], t),
  ]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
  let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
  (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn xyz(v: [f32; 4]) -> [f32; 3] {
  [v[0], v[1], v[2]]
}

fn with_w(v: [f32; 3], w: f32) -> [f32; 4] {
  [v[0], v[1], v[2], w]
}

fn to_rgb(color: Color) -> [f32; 3] {
  [color.r, color.g, color.b].map(|channel| f32::from(channel) / 255.0)
}

/// Inverse of [`to_rgb`] for one channel.
// LINT: Clamped to `0.0..=255.0` first, so the cast can't truncate.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn to_byte(channel: f32) -> u8 {
  (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn srgb_to_linear(c: f32) -> f32 {
  if c <= 0.04045 {
    c / 12.92
  } else {
    ((c + 0.055) / 1.055).max(0.0).powf(2.4)
  }
}

fn linear_to_srgb(c: f32) -> f32 {
  if c <= 0.003_130_8 {
    c * 12.92
  } else {
    1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055
  }
}

/// sRGB to OKLab, per <https://bottosson.github.io/posts/oklab/>.
pub(crate) fn srgb_to_oklab(srgb: [f32; 3]) -> [f32; 3] {
  linear_to_oklab(srgb.map(srgb_to_linear))
}

/// OKLab to (unclamped) sRGB; inverse of [`srgb_to_oklab`].
fn oklab_to_srgb(lab: [f32; 3]) -> [f32; 3] {
  oklab_to_linear(lab).map(linear_to_srgb)
}

/// Names and constants follow the OKLab reference (and the shader)
/// verbatim.
#[allow(clippy::excessive_precision, clippy::many_single_char_names)]
fn linear_to_oklab(linear: [f32; 3]) -> [f32; 3] {
  let [r, g, b] = linear;

  let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
  let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
  let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;

  let [l, m, s] = [l, m, s].map(|v| v.max(0.0).powf(1.0 / 3.0));

  [
    0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
    1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
    0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
  ]
}

#[allow(clippy::excessive_precision, clippy::many_single_char_names)]
fn oklab_to_linear(lab: [f32; 3]) -> [f32; 3] {
  let [lightness, a, b] = lab;

  let l = lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
  let m = lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
  let s = lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

  let [l, m, s] = [l * l * l, m * m * m, s * s * s];

  [
    4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
    -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
    -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
  ]
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  const EPSILON: f32 = 2.0 / 255.0;

  const LEVELS: SourceLevels = SourceLevels {
    paper: 1.0,
    ink: 0.0,
  };

  /// A filter with just a `(background, foreground)` ramp and overrides.
  fn legacy(
    ramp: Option<(Color, Color)>,
    saturation_threshold: f32,
    overrides: &[ColorOverride],
  ) -> crate::Result<ColorFilter> {
    ColorFilter::new(&ColorFilterOptions {
      ramp: ramp.map_or_else(Vec::new, |(background, foreground)| {
        vec![
          RampStop {
            from: color("#ffffff"),
            to: background,
          },
          RampStop {
            from: color("#000000"),
            to: foreground,
          },
        ]
      }),
      saturation_threshold,
      overrides: overrides.to_vec(),
      ..ColorFilterOptions::default()
    })
  }

  fn color(hex: &str) -> Color {
    Color::from_str(hex).expect("valid test color")
  }

  fn rgb(hex: &str) -> [f32; 3] {
    to_rgb(color(hex))
  }

  fn assert_close(actual: [f32; 3], expected: [f32; 3]) {
    for (a, e) in actual.iter().zip(expected) {
      assert!(
        (a - e).abs() <= EPSILON,
        "expected {expected:?}, got {actual:?}"
      );
    }
  }

  fn winter() -> ColorFilter {
    legacy(Some((color("#1e1e1e"), color("#d4d4d4"))), 0.15, &[])
      .expect("valid theme")
  }

  #[test]
  fn oklab_round_trips() {
    for hex in ["#000000", "#ffffff", "#0078d4", "#fff3b0", "#808080"] {
      assert_close(oklab_to_srgb(srgb_to_oklab(rgb(hex))), rgb(hex));
    }
  }

  #[test]
  fn ramp_maps_white_to_background_and_black_to_foreground() {
    let theme = winter();

    assert_close(theme.apply(rgb("#ffffff"), LEVELS), rgb("#1e1e1e"));
    assert_close(theme.apply(rgb("#000000"), LEVELS), rgb("#d4d4d4"));
  }

  #[test]
  fn antialiased_grays_land_between_background_and_foreground() {
    let theme = winter();
    let background = rgb("#1e1e1e")[0];
    let foreground = rgb("#d4d4d4")[0];
    let mut previous = foreground;

    // Walking from black text towards the white page, every step must move
    // monotonically from `foreground` towards `background`, and stay gray.
    for step in 1..=16u8 {
      let v = f32::from(step) / 16.0;
      let [r, g, b] = theme.apply([v, v, v], LEVELS);

      assert!(r <= previous + 1e-4, "not monotonic at {v}: {r}");
      assert!((background - 1e-3..=foreground + 1e-3).contains(&r));
      assert!((r - g).abs() < 1e-3 && (g - b).abs() < 1e-3);
      previous = r;
    }

    let mid = theme.apply([0.5, 0.5, 0.5], LEVELS)[0];
    assert!(mid > background + 0.1 && mid < foreground - 0.1);
  }

  #[test]
  fn apply_color_matches_apply_and_keeps_alpha() {
    let theme = winter();

    assert_eq!(
      theme.apply_color(color("#ffffff80"), LEVELS),
      color("#1e1e1e80")
    );
    assert_eq!(
      theme.apply_color(color("#000000"), LEVELS),
      color("#d4d4d4")
    );
    assert_eq!(
      theme.apply_color(color("#0078d4"), LEVELS),
      color("#0078d4")
    );
  }

  #[test]
  fn saturated_pixels_above_threshold_are_untouched() {
    let theme = winter();

    for hex in ["#0078d4", "#e81123", "#16c60c", "#fff100"] {
      assert_close(theme.apply(rgb(hex), LEVELS), rgb(hex));
    }
  }

  #[test]
  fn saturation_threshold_eases_out() {
    let lab = srgb_to_oklab(rgb("#c8d2e6"));
    let saturation = lab[1].hypot(lab[2]) / MAX_CHROMA;

    // A threshold far above leaves the full ramp, one just above blends,
    // and one at or below leaves the pixel alone.
    assert!((ramp_weight(saturation, 1.0) - 1.0).abs() < 1e-6);
    let partial = ramp_weight(saturation, saturation * 1.2);
    assert!(partial > 0.0 && partial < 1.0);
    assert!(ramp_weight(saturation, saturation).abs() < 1e-6);
    assert!(ramp_weight(0.0, 0.0).abs() < 1e-6);
  }

  #[test]
  fn pale_tints_are_ramped_like_grays() {
    let theme = winter();

    // WPF hover, pressed, and selection backgrounds.
    for hex in ["#bee6fd", "#c4e5f6", "#cce8ff"] {
      let themed = srgb_to_oklab(theme.apply(rgb(hex), LEVELS));
      assert!(themed[0] < 0.4, "{hex} stayed light: {themed:?}");
    }
  }

  #[test]
  fn vivid_light_colors_keep_their_color() {
    let theme = winter();

    for hex in ["#fff100", "#ffb900", "#90ee90"] {
      assert_close(theme.apply(rgb(hex), LEVELS), rgb(hex));
    }
  }

  #[test]
  fn threshold_zero_disables_the_ramp() {
    let theme =
      legacy(Some((color("#1e1e1e"), color("#d4d4d4"))), 0.0, &[])
        .expect("valid theme");

    assert_close(theme.apply(rgb("#ffffff"), LEVELS), rgb("#ffffff"));
    assert_close(theme.apply(rgb("#bee6fd"), LEVELS), rgb("#bee6fd"));
  }

  #[test]
  fn no_ramp_is_identity() {
    let theme = legacy(None, 0.15, &[]).expect("valid theme");

    for hex in ["#ffffff", "#000000", "#0078d4", "#7f7f7f"] {
      assert_close(theme.apply(rgb(hex), LEVELS), rgb(hex));
    }
  }

  #[test]
  fn override_replaces_exact_match_even_when_saturated() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[
        ColorOverride {
          from: color("#fff3b0"),
          to: color("#5a4a00"),
          tolerance: 10.0,
        },
        ColorOverride {
          from: color("#0078d4"),
          to: color("#4aa3ff"),
          tolerance: 6.0,
        },
      ],
    )
    .expect("valid theme");

    assert_close(theme.apply(rgb("#fff3b0"), LEVELS), rgb("#5a4a00"));
    assert_close(theme.apply(rgb("#0078d4"), LEVELS), rgb("#4aa3ff"));
  }

  #[test]
  fn override_falls_off_softly_within_tolerance() {
    let from = color("#fff3b0");
    let to = color("#5a4a00");
    let theme = legacy(
      None,
      0.15,
      &[ColorOverride {
        from,
        to,
        tolerance: 10.0,
      }],
    )
    .expect("valid theme");

    let from_lab = srgb_to_oklab(to_rgb(from));
    let to_lab = srgb_to_oklab(to_rgb(to));

    // Nudge lightness away from `from` in OKLab and measure how far the
    // result moved towards `to`.
    let progress = |delta: f32| {
      let source = [from_lab[0] - delta, from_lab[1], from_lab[2]];
      let out = srgb_to_oklab(theme.apply(oklab_to_srgb(source), LEVELS));
      1.0 - distance(out, to_lab) / distance(source, to_lab)
    };

    let near = progress(0.02);
    let mid = progress(0.05);
    let edge = progress(0.095);

    assert!(near > mid && mid > edge, "{near} {mid} {edge}");
    assert!(near > 0.8 && near < 1.0);
    assert!(mid > 0.3 && mid < 0.7);
    assert!(edge > 0.0 && edge < 0.05);
    assert!(progress(0.12).abs() < 1e-3);
  }

  #[test]
  fn closest_override_wins() {
    let theme = legacy(
      None,
      0.15,
      &[
        ColorOverride {
          from: color("#ff0000"),
          to: color("#00ff00"),
          tolerance: 50.0,
        },
        ColorOverride {
          from: color("#ff1010"),
          to: color("#0000ff"),
          tolerance: 50.0,
        },
      ],
    )
    .expect("valid theme");

    assert_close(theme.apply(rgb("#ff1010"), LEVELS), rgb("#0000ff"));
  }

  #[test]
  fn rejects_invalid_values() {
    assert!(legacy(None, 1.5, &[]).is_err());
    assert!(legacy(None, -0.1, &[]).is_err());

    let bad_tolerance = ColorOverride {
      from: color("#000000"),
      to: color("#ffffff"),
      tolerance: -1.0,
    };
    assert!(legacy(None, 0.1, &[bad_tolerance]).is_err());

    let many = vec![
      ColorOverride {
        from: color("#000000"),
        to: color("#ffffff"),
        tolerance: 1.0,
      };
      MAX_COLOR_OVERRIDES + 1
    ];
    assert!(legacy(None, 0.1, &many).is_err());
  }

  /// A neighborhood repeating `row` on all three lines.
  fn rows(row: [[f32; 3]; 5]) -> [[f32; 3]; NEIGHBORHOOD_SIZE] {
    let mut pixels = [[0.0; 3]; NEIGHBORHOOD_SIZE];
    for (index, pixel) in pixels.iter_mut().enumerate() {
      *pixel = row[index % 5];
    }
    pixels
  }

  /// A neighborhood with `center` in the middle column, `left` left of
  /// it and `right` right of it.
  fn edge_block(
    left: [f32; 3],
    center: [f32; 3],
    right: [f32; 3],
  ) -> [[f32; 3]; NEIGHBORHOOD_SIZE] {
    rows([left, left, center, right, right])
  }

  fn lerp_rgb(
    from: [f32; 3],
    to: [f32; 3],
    coverage: [f32; 3],
  ) -> [f32; 3] {
    [0, 1, 2].map(|c| lerp(from[c], to[c], coverage[c]))
  }

  #[test]
  fn flat_neighborhood_matches_apply() {
    let theme = winter();

    for hex in ["#ffffff", "#7f7f7f", "#0078d4"] {
      assert_close(
        theme.apply_neighborhood(&[rgb(hex); NEIGHBORHOOD_SIZE], LEVELS),
        theme.apply(rgb(hex), LEVELS),
      );
    }
  }

  /// Coverage after the boost for edges the theme turns light-on-dark.
  fn inverted(coverage: f32) -> [f32; 3] {
    [coverage.powf(1.0 / INVERTED_TEXT_GAMMA); 3]
  }

  #[test]
  fn grayscale_antialiasing_keeps_its_coverage() {
    let (black, white) = (rgb("#000000"), rgb("#ffffff"));
    let edge = [0.75; 3];

    // A 25%-covered text edge re-mixes the themed text and page colors at
    // that same 25%, rather than landing wherever the ramp puts `#bfbfbf`.
    let same_polarity =
      legacy(Some((color("#fdf6e3"), color("#073642"))), 0.15, &[])
        .expect("valid theme");
    assert_close(
      same_polarity
        .apply_neighborhood(&edge_block(black, edge, white), LEVELS),
      lerp_rgb(rgb("#fdf6e3"), rgb("#073642"), [0.25; 3]),
    );

    // Flipped to light-on-dark, the same edge is boosted.
    assert_close(
      winter().apply_neighborhood(&edge_block(black, edge, white), LEVELS),
      lerp_rgb(rgb("#1e1e1e"), rgb("#d4d4d4"), inverted(0.25)),
    );
  }

  #[test]
  fn subpixel_fringes_become_grayscale() {
    let theme = winter();
    let (black, white) = (rgb("#000000"), rgb("#ffffff"));

    // ClearType: a blue fringe between a black stroke and an orange one,
    // which together average out gray across the row.
    let blue = [0.2, 0.6, 1.0];
    let orange = [1.0, 0.6, 0.2];
    let out = theme.apply_neighborhood(
      &rows([black, black, blue, orange, white]),
      LEVELS,
    );

    // Recognized as text rather than color, at its mean coverage.
    assert_close(
      out,
      lerp_rgb(rgb("#1e1e1e"), rgb("#d4d4d4"), inverted(0.4)),
    );
  }

  #[test]
  fn wpf_cleartype_fringes_become_grayscale() {
    let theme = winter();
    let px = |r: u8, g: u8, b: u8| [r, g, b].map(|c| f32::from(c) / 255.0);
    let (black, white) = (px(0, 0, 0), px(255, 255, 255));

    // Measured across the left edge of a WPF "T" stroke: page, yellow
    // fringe, red fringe, ink.
    let (yellow, red) = (px(255, 255, 186), px(106, 0, 0));
    for (center, row) in [
      (yellow, [white, white, yellow, red, black]),
      (red, [white, yellow, red, black, black]),
    ] {
      let out = theme.apply_neighborhood(&rows(row), LEVELS);
      let spread = out.iter().fold(0.0_f32, |m, c| m.max(*c))
        - out.iter().fold(1.0_f32, |m, c| m.min(*c));

      assert!(spread < 0.02, "fringe {center:?} stayed colored: {out:?}");
    }
  }

  #[test]
  fn wpf_hairline_without_full_ink_becomes_grayscale() {
    let theme = winter();
    let px = |r: u8, g: u8, b: u8| [r, g, b].map(|c| f32::from(c) / 255.0);
    let white = px(255, 255, 255);

    // Measured across a WPF hairline stem that never reaches full ink.
    let out = theme.apply_neighborhood(
      &rows([
        white,
        px(255, 255, 186),
        px(106, 0, 106),
        px(186, 255, 255),
        white,
      ]),
      LEVELS,
    );
    let spread = out.iter().fold(0.0_f32, |m, c| m.max(*c))
      - out.iter().fold(1.0_f32, |m, c| m.min(*c));

    assert!(spread < 0.02, "hairline stayed colored: {out:?}");
    assert!(out[0] > 0.5, "hairline should read as light ink: {out:?}");
  }

  /// A neighborhood from three rows of five hex colors.
  fn hex_rows(rows: [[&str; 5]; 3]) -> [[f32; 3]; NEIGHBORHOOD_SIZE] {
    let mut pixels = [[0.0; 3]; NEIGHBORHOOD_SIZE];
    for (index, pixel) in pixels.iter_mut().enumerate() {
      *pixel = rgb(rows[index / 5][index % 5]);
    }
    pixels
  }

  fn assert_gray(out: [f32; 3]) {
    let spread = out.iter().fold(0.0_f32, |m, c| m.max(*c))
      - out.iter().fold(1.0_f32, |m, c| m.min(*c));
    assert!(spread < 0.03, "fringe stayed colored: {out:?}");
  }

  #[test]
  fn dense_diagonal_fringes_become_grayscale() {
    // Measured on the diagonal of a WPF "k", where ink outweighs paper.
    let out = winter().apply_neighborhood(
      &hex_rows([
        ["#ffffb5", "#630034", "#8ddada", "#8d3400", "#63b5ff"],
        ["#ffffb5", "#630034", "#343400", "#348dda", "#ffffff"],
        ["#ffffb5", "#630000", "#343434", "#0063b5", "#ffffff"],
      ]),
      LEVELS,
    );

    assert_gray(out);
    assert!(out[0] > 0.4, "diagonal should read as light ink: {out:?}");
  }

  #[test]
  fn fringes_between_close_stems_become_grayscale() {
    // Measured between the "i" and "l" of a WPF "File": no paper shows.
    let row = ["#f0f0b1", "#6f214a", "#91d0f0", "#f0f0b1", "#6f214a"];
    assert_gray(
      winter().apply_neighborhood(&hex_rows([row, row, row]), LEVELS),
    );

    // Measured between two stems of "Right-click", paper side a fringe.
    assert_gray(winter().apply_neighborhood(
      &hex_rows([
        ["#5e0032", "#86d0f3", "#f3f3ac", "#5e0032", "#86ac86"],
        ["#000032", "#86d0f3", "#f3f3ac", "#5e0000", "#003286"],
        ["#5e0032", "#86d0f3", "#f3f3ac", "#5e0032", "#86d0f3"],
      ]),
      LEVELS,
    ));
  }

  #[test]
  fn highlight_paper_between_letters_stays_highlighted() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#fff3b0"),
        to: color("#5a4a00"),
        tolerance: 10.0,
      }],
    )
    .expect("valid theme");
    let px = |r: u8, g: u8, b: u8| [r, g, b].map(|c| f32::from(c) / 255.0);
    let paper = px(255, 243, 176);

    // Measured between two WPF letters on a `#fff3b0` highlight.
    let out = theme.apply_neighborhood(
      &rows([
        paper,
        px(0, 0, 11),
        px(255, 243, 170),
        px(178, 91, 6),
        paper,
      ]),
      LEVELS,
    );

    // Still (nearly) the themed highlight, not a neutral gray.
    assert!(
      distance(srgb_to_oklab(out), srgb_to_oklab(rgb("#5a4a00"))) < 0.05,
      "highlight paper lost its color: {out:?}"
    );
  }

  #[test]
  fn hairline_on_highlight_becomes_grayscale() {
    let theme = winter();
    let px = |r: u8, g: u8, b: u8| [r, g, b].map(|c| f32::from(c) / 255.0);
    let paper = px(255, 243, 176);

    // Measured across the "l" of "highlighted" on a `#fff3b0` highlight.
    let stem = px(140, 53, 49);
    let out = theme.apply_neighborhood(
      &rows([paper, px(255, 238, 146), stem, px(160, 212, 176), paper]),
      LEVELS,
    );

    // The stem reads as neutral ink, not as the dark red fringe.
    let spread = out.iter().fold(0.0_f32, |m, c| m.max(*c))
      - out.iter().fold(1.0_f32, |m, c| m.min(*c));
    assert!(spread < 0.05, "stem stayed colored: {out:?}");
    assert!(out[0] > 0.4, "stem should read as light ink: {out:?}");
  }

  #[test]
  fn thin_colored_text_keeps_its_color() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#0078d4"),
        to: color("#4aa3ff"),
        tolerance: 6.0,
      }],
    )
    .expect("valid theme");

    // A `#0078d4` link stroke with its own `ClearType` fringes.
    let ink = rgb("#0078d4");
    let out = theme.apply_neighborhood(
      &rows([
        rgb("#ffffff"),
        [0.7, 0.68, 0.85],
        ink,
        [0.1, 0.68, 0.95],
        rgb("#ffffff"),
      ]),
      LEVELS,
    );

    assert_close(out, rgb("#4aa3ff"));
  }

  #[test]
  fn hairline_colored_text_keeps_its_hue() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#0078d4"),
        to: color("#4aa3ff"),
        tolerance: 6.0,
      }],
    )
    .expect("valid theme");
    let px = |r: u8, g: u8, b: u8| [r, g, b].map(|c| f32::from(c) / 255.0);
    let (white, purple, cyan) =
      (px(255, 255, 255), px(162, 162, 219), px(57, 188, 245));
    let ink = srgb_to_oklab(rgb("#4aa3ff"));

    // A 1.2px `#0078d4` `ClearType` stroke: no pixel is fully covered, so
    // both of its pixels are fringes.
    for (center, row) in [
      (
        purple,
        [white, px(255, 254, 249), purple, cyan, px(241, 255, 255)],
      ),
      (
        cyan,
        [px(255, 254, 249), purple, cyan, px(241, 255, 255), white],
      ),
    ] {
      let out =
        srgb_to_oklab(theme.apply_neighborhood(&rows(row), LEVELS));
      let hue_error = (out[2].atan2(out[1]) - ink[2].atan2(ink[1])).abs();

      assert!(out[0] < 0.7, "fringe {center:?} stayed light: {out:?}");
      assert!(
        hue_error < 0.15,
        "fringe {center:?} lost the ink's hue: {out:?}"
      );
    }
  }

  #[test]
  fn wpf_link_keeps_its_color() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#1976d2"),
        to: color("#8ab4f8"),
        tolerance: 6.0,
      }],
    )
    .expect("valid theme");
    let hex = |rows: [[&str; 5]; 3]| {
      let mut pixels = [[0.0; 3]; NEIGHBORHOOD_SIZE];
      for (index, pixel) in pixels.iter_mut().enumerate() {
        *pixel = rgb(rows[index / 5][index % 5]);
      }
      pixels
    };
    let target = srgb_to_oklab(rgb("#8ab4f8"));

    // Measured on a WPF `#1976d2` link on `#fbfbfb`: the bar of an "F",
    // whose fully covered pixels must still hit the override.
    let bar = hex([
      ["#fbfbed", "#968ad6", "#96d9fb", "#fbfbfb", "#fbfbfb"],
      ["#fbfbed", "#968ad2", "#1976d2", "#1976d2", "#1976d6"],
      ["#fbfbed", "#968ad6", "#96d9fb", "#fbfbfb", "#fbfbfb"],
    ]);
    let out = srgb_to_oklab(theme.apply_neighborhood(&bar, LEVELS));
    assert!(
      distance(out, target) < 0.03,
      "ink missed the override: {out:?}"
    );

    // The stem of an "l", which never fully covers a pixel and whose
    // fringes disagree in hue: still the link color, not gray.
    let stem = hex([
      ["#96d9fb", "#fbd9df", "#478adf", "#d5fbfb", "#fbc2da"],
      ["#fbfbfb", "#fbd9df", "#478adf", "#d5fbfb", "#d5afd6"],
      ["#fbfbfb", "#fbd9df", "#478adf", "#d5fbfb", "#d5afd6"],
    ]);
    let out = srgb_to_oklab(theme.apply_neighborhood(&stem, LEVELS));
    let hue_error =
      (out[2].atan2(out[1]) - target[2].atan2(target[1])).abs();
    assert!(
      out[1].hypot(out[2]) > 0.05 && hue_error < 0.3,
      "stem lost the link color: {out:?}"
    );

    // The cyan fringe right of that stem, in a window too dense to show
    // any plain paper: still (nearly) the themed paper, not dark teal.
    let fringe = hex([
      ["#fbd9df", "#478adf", "#d5fbfb", "#fbc2da", "#1976da"],
      ["#fbd9df", "#478adf", "#d5fbfb", "#d5afd6", "#72c2fb"],
      ["#fbd9df", "#478adf", "#d5fbfb", "#d5afd6", "#72c2fb"],
    ]);
    let out = srgb_to_oklab(theme.apply_neighborhood(&fringe, LEVELS));
    assert!(
      out[1].hypot(out[2]) < 0.03,
      "fringe took a color of its own: {out:?}"
    );
  }

  #[test]
  fn override_ink_remixes_thin_colored_text_exactly() {
    let theme = winter_with_link();
    let (paper, ink) = (rgb("#fbfbfb"), rgb("#1976d2"));

    // Measured on a WPF `#1976d2` link: the "l" stem, which no pixel
    // covers fully, between `ClearType` fringes.
    let stem = hex_window([
      ["#96d9fb", "#fbd9df", "#478adf", "#d5fbfb", "#fbc2da"],
      ["#fbfbfb", "#fbd9df", "#478adf", "#d5fbfb", "#d5afd6"],
      ["#fbfbfb", "#fbd9df", "#478adf", "#d5fbfb", "#d5afd6"],
    ]);
    let coverage = known_ink_coverage(rgb("#478adf"), ink, paper);

    // The override's ink at the stem's coverage, boosted as light text.
    assert_close(
      theme.apply_neighborhood(&stem, LEVELS),
      lerp_rgb(
        theme.apply(paper, LEVELS),
        rgb("#8ab4f8"),
        [coverage.powf(1.0 / INVERTED_TEXT_GAMMA); 3],
      ),
    );
  }

  #[test]
  fn override_ink_leaves_other_content_alone() {
    let with_link = winter_with_link();
    let without = winter();

    // Black text, and dark hair in a photo, measured beside that link:
    // neither is `#1976d2` over the page.
    let (black, white) = (rgb("#000000"), rgb("#ffffff"));
    let text = edge_block(black, [0.5; 3], white);
    let hair = hex_window([
      ["#0e0805", "#0f0805", "#110904", "#170c06", "#3d2819"],
      ["#0c0704", "#0c0704", "#0e0905", "#0c0603", "#0e0805"],
      ["#150d09", "#120b06", "#100a06", "#0d0805", "#0a0704"],
    ]);

    for window in [text, hair] {
      assert_close(
        with_link.apply_neighborhood(&window, LEVELS),
        without.apply_neighborhood(&window, LEVELS),
      );
    }
  }

  /// `winter` plus a `#1976d2` -> `#8ab4f8` link override.
  fn winter_with_link() -> ColorFilter {
    legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#1976d2"),
        to: color("#8ab4f8"),
        tolerance: 20.0,
      }],
    )
    .expect("valid theme")
  }

  fn hex_window(rows: [[&str; 5]; 3]) -> [[f32; 3]; NEIGHBORHOOD_SIZE] {
    let mut pixels = [[0.0; 3]; NEIGHBORHOOD_SIZE];
    for (index, pixel) in pixels.iter_mut().enumerate() {
      *pixel = rgb(rows[index / 5][index % 5]);
    }
    pixels
  }

  #[test]
  fn colored_pixel_between_black_and_white_is_not_a_fringe() {
    let theme = winter();
    let red = rgb("#e81123");
    let out = theme.apply_neighborhood(
      &edge_block(rgb("#000000"), red, rgb("#ffffff")),
      LEVELS,
    );

    // The row is far from gray, so this is real color, left untouched.
    assert_close(out, red);
  }

  #[test]
  fn colorful_image_edges_are_untouched() {
    let theme = winter();
    let (red, green) = (rgb("#e81123"), rgb("#16c60c"));
    let mix = lerp3(red, green, 0.5);

    assert_close(
      theme.apply_neighborhood(&edge_block(red, mix, green), LEVELS),
      mix,
    );
  }

  #[test]
  fn button_edges_blend_between_themed_colors() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#0078d4"),
        to: color("#4aa3ff"),
        tolerance: 6.0,
      }],
    )
    .expect("valid theme");

    let (blue, white) = (rgb("#0078d4"), rgb("#ffffff"));
    let corner = lerp3(blue, white, 0.5);

    assert_close(
      theme.apply_neighborhood(&edge_block(blue, corner, white), LEVELS),
      lerp_rgb(rgb("#1e1e1e"), rgb("#4aa3ff"), inverted(0.5)),
    );
  }

  #[test]
  fn override_ink_ignores_colors_sharing_a_channel_with_it() {
    let theme = legacy(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.15,
      &[ColorOverride {
        from: color("#0078d4"),
        to: color("#4aa3ff"),
        tolerance: 6.0,
      }],
    )
    .expect("valid theme");
    let px = |r: f32, g: f32, b: f32| [r, g, b];
    let (dark_red, purple) =
      (px(0.416, 0.0, 0.0), px(0.605, 0.368, 0.659));

    // `#0078d4` has no red, like the darkest red level here; the purple
    // having some must rule the link out.
    let out = theme.apply_neighborhood(
      &rows([dark_red, dark_red, px(0.48, 0.214, 0.633), purple, purple]),
      LEVELS,
    );
    let link = rgb("#4aa3ff");

    assert!(
      distance(srgb_to_oklab(out), srgb_to_oklab(link)) > 0.2,
      "took the window for link text: {out:?}"
    );
  }

  #[test]
  fn constants_match_the_shader_layout() {
    // `struct Filter`: five float4s, then the ramp, palette, override and
    // override ink float4 arrays.
    assert_eq!(
      std::mem::size_of::<FilterConstants>(),
      16 * (5
        + MAX_RAMP_STOPS
        + MAX_PALETTE_COLORS
        + MAX_COLOR_OVERRIDES * 3)
    );
  }

  fn options() -> ColorFilterOptions {
    ColorFilterOptions::default()
  }

  fn stop(from: &str, to: &str) -> RampStop {
    RampStop {
      from: color(from),
      to: color(to),
    }
  }

  fn winter_options() -> ColorFilterOptions {
    ColorFilterOptions {
      ramp: vec![stop("#ffffff", "#1e1e1e"), stop("#000000", "#d4d4d4")],
      ..options()
    }
  }

  fn new_filter(options: &ColorFilterOptions) -> ColorFilter {
    ColorFilter::new(options).expect("valid filter")
  }

  fn lab(hex: &str) -> [f32; 3] {
    srgb_to_oklab(rgb(hex))
  }

  fn chroma(lab: [f32; 3]) -> f32 {
    lab[1].hypot(lab[2])
  }

  /// Hue difference in degrees.
  fn hue_difference(a: [f32; 3], b: [f32; 3]) -> f32 {
    let difference = (a[2].atan2(a[1]) - b[2].atan2(b[1])).to_degrees();
    (difference + 540.0).rem_euclid(360.0) - 180.0
  }

  #[test]
  fn default_options_are_the_identity() {
    let filter = new_filter(&options());

    for hex in ["#ffffff", "#000000", "#0078d4", "#7f7f7f", "#fff3b0"] {
      assert_close(filter.apply(rgb(hex), LEVELS), rgb(hex));
    }
  }

  #[test]
  fn multi_stop_ramp_maps_each_stop() {
    // Out of order on purpose: stops sort by source lightness.
    let filter = new_filter(&ColorFilterOptions {
      ramp: vec![
        stop("#000000", "#cdd6f4"),
        stop("#ffffff", "#1e1e2e"),
        stop("#f0f0f0", "#181825"),
      ],
      ..options()
    });

    assert_close(filter.apply(rgb("#ffffff"), LEVELS), rgb("#1e1e2e"));
    assert_close(filter.apply(rgb("#f0f0f0"), LEVELS), rgb("#181825"));
    assert_close(filter.apply(rgb("#000000"), LEVELS), rgb("#cdd6f4"));

    // Between two stops, lightness lands between their targets.
    let between = srgb_to_oklab(filter.apply(rgb("#f8f8f8"), LEVELS))[0];
    assert!(between > lab("#181825")[0] && between < lab("#1e1e2e")[0]);
  }

  #[test]
  fn rejects_invalid_ramps() {
    let single = ColorFilterOptions {
      ramp: vec![stop("#ffffff", "#000000")],
      ..options()
    };
    assert!(ColorFilter::new(&single).is_err());

    let duplicate = ColorFilterOptions {
      ramp: vec![stop("#ffffff", "#000000"), stop("#ffffff", "#111111")],
      ..options()
    };
    assert!(ColorFilter::new(&duplicate).is_err());
  }

  #[test]
  fn accent_lightness_flips_colored_text_but_keeps_its_hue() {
    let filter = new_filter(&ColorFilterOptions {
      accent_lightness: 1.0,
      ..winter_options()
    });

    // A dark blue link on white becomes a light blue on dark.
    let source = lab("#0000ee");
    let out = srgb_to_oklab(filter.apply(rgb("#0000ee"), LEVELS));

    // It lands where a gray of its lightness does: well clear of the
    // themed background.
    assert!(out[0] > source[0] + 0.1, "link stayed dark: {out:?}");
    assert!(out[0] - lab("#1e1e1e")[0] > 0.3, "link too faint: {out:?}");
    assert!(hue_difference(out, source).abs() < 5.0);
    assert!(chroma(out) > 0.1, "link lost its color: {out:?}");

    // Without it, the same color is left alone.
    assert_close(
      new_filter(&winter_options()).apply(rgb("#0000ee"), LEVELS),
      rgb("#0000ee"),
    );
  }

  #[test]
  fn saturation_zero_is_grayscale() {
    let filter = new_filter(&ColorFilterOptions {
      saturation: 0.0,
      ..options()
    });
    let [r, g, b] = filter.apply(rgb("#e81123"), LEVELS);

    assert!((r - g).abs() < 1e-3 && (g - b).abs() < 1e-3);
  }

  #[test]
  fn vibrance_boosts_dull_colors_more_than_vivid_ones() {
    let filter = new_filter(&ColorFilterOptions {
      vibrance: 0.8,
      ..options()
    });
    let boost = |hex: &str| {
      chroma(srgb_to_oklab(filter.apply(rgb(hex), LEVELS)))
        / chroma(lab(hex))
    };

    let dull = boost("#8a9aa8");
    let vivid = boost("#0078d4");
    assert!(dull > 1.3, "dull color barely changed: {dull}");
    // Vivid colors barely move (and may be pulled back into gamut).
    assert!(vivid < 1.05 && vivid > 0.97, "{vivid} vs {dull}");
  }

  #[test]
  fn hue_shift_rotates_hue() {
    let filter = new_filter(&ColorFilterOptions {
      hue_shift: 90.0,
      ..options()
    });
    let source = lab("#b07070");
    let out = srgb_to_oklab(filter.apply(rgb("#b07070"), LEVELS));

    assert!((hue_difference(out, source) - 90.0).abs() < 3.0);
    assert!((out[0] - source[0]).abs() < 0.01);
  }

  #[test]
  fn palette_snaps_hues_and_ignores_grays() {
    let filter = new_filter(&ColorFilterOptions {
      palette: vec![color("#89b4fa"), color("#f38ba8"), color("#1e1e2e")],
      ..options()
    });

    // Windows blue takes on the palette's blue hue and chroma, keeping its
    // own lightness.
    let out = srgb_to_oklab(filter.apply(rgb("#0078d4"), LEVELS));
    let blue = lab("#89b4fa");
    assert!(hue_difference(out, blue).abs() < 8.0, "{out:?}");
    assert!((chroma(out) - chroma(blue)).abs() < 0.02, "{out:?}");
    assert!((out[0] - lab("#0078d4")[0]).abs() < 0.01);

    // A red goes to the palette's pink, not its blue.
    let out = srgb_to_oklab(filter.apply(rgb("#e81123"), LEVELS));
    assert!(hue_difference(out, lab("#f38ba8")).abs() < 10.0, "{out:?}");

    // Grays have no hue to snap.
    assert_close(filter.apply(rgb("#808080"), LEVELS), rgb("#808080"));
  }

  #[test]
  fn palette_lightness_takes_on_the_palette_lightness() {
    let filter = new_filter(&ColorFilterOptions {
      palette: vec![color("#89b4fa")],
      palette_lightness: 1.0,
      ..options()
    });

    assert_close(filter.apply(rgb("#0050a0"), LEVELS), rgb("#89b4fa"));
  }

  #[test]
  fn gray_only_palette_is_rejected() {
    let gray = ColorFilterOptions {
      palette: vec![color("#1e1e2e"), color("#ffffff")],
      ..options()
    };
    assert!(ColorFilter::new(&gray).is_err());
  }

  #[test]
  fn brightness_and_contrast_scale_lightness() {
    let dim = new_filter(&ColorFilterOptions {
      brightness: 0.5,
      ..options()
    });
    let out = srgb_to_oklab(dim.apply(rgb("#ffffff"), LEVELS));
    assert!((out[0] - 0.5).abs() < 0.01);

    let contrast = new_filter(&ColorFilterOptions {
      contrast: 2.0,
      ..options()
    });
    let gray = lab("#999999")[0];
    let out = srgb_to_oklab(contrast.apply(rgb("#999999"), LEVELS))[0];
    assert!((out - ((gray - 0.5) * 2.0 + 0.5)).abs() < 0.01);
  }

  #[test]
  fn min_contrast_lifts_text_but_not_subtle_surfaces() {
    // A deliberately low-contrast ramp: text barely above the background.
    let options = ColorFilterOptions {
      ramp: vec![stop("#ffffff", "#1e1e1e"), stop("#000000", "#333333")],
      min_contrast: 0.4,
      ..options()
    };
    let filter = new_filter(&options);
    let paper = lab("#1e1e1e")[0];

    let text = srgb_to_oklab(filter.apply(rgb("#000000"), LEVELS))[0];
    assert!(text >= paper + 0.4 - 0.01, "text too faint: {text}");

    // The paper itself doesn't move.
    assert_close(filter.apply(rgb("#ffffff"), LEVELS), rgb("#1e1e1e"));

    // A panel the app drew close to its background keeps its small
    // difference instead of being pushed to full contrast.
    let panel = srgb_to_oklab(filter.apply(rgb("#f0f0f0"), LEVELS))[0];
    let source_difference = 1.0 - lab("#f0f0f0")[0];
    assert!((panel - paper).abs() <= source_difference + 0.01);
  }

  #[test]
  fn warmth_shifts_towards_red_or_blue() {
    let warm = new_filter(&ColorFilterOptions {
      warmth: 1.0,
      ..options()
    });
    let [r, _, b] = warm.apply(rgb("#ffffff"), LEVELS);
    assert!(r - b > 0.2, "not warm: {r} {b}");

    let cool = new_filter(&ColorFilterOptions {
      warmth: -1.0,
      ..options()
    });
    let [r, _, b] = cool.apply(rgb("#ffffff"), LEVELS);
    assert!(b - r > 0.1, "not cool: {r} {b}");
  }

  #[test]
  fn out_of_gamut_results_keep_their_hue() {
    let filter = new_filter(&ColorFilterOptions {
      saturation: 4.0,
      ..options()
    });
    let source = lab("#0078d4");
    let out = srgb_to_oklab(filter.apply(rgb("#0078d4"), LEVELS));

    // Clamping channels one by one would skew the hue instead.
    assert!(hue_difference(out, source).abs() < 3.0, "{out:?}");
    assert!((out[0] - source[0]).abs() < 0.02, "{out:?}");
  }

  #[test]
  fn source_levels_treat_measured_paper_as_white() {
    let filter = new_filter(&winter_options());
    let levels = SourceLevels {
      paper: lab("#f3f3f3")[0],
      ink: lab("#333333")[0],
    };

    assert_close(filter.apply(rgb("#f3f3f3"), levels), rgb("#1e1e1e"));
    assert_close(filter.apply(rgb("#333333"), levels), rgb("#d4d4d4"));

    // Dark paper with light ink maps the same way round.
    let dark = SourceLevels {
      paper: lab("#202020")[0],
      ink: lab("#e0e0e0")[0],
    };
    assert_close(filter.apply(rgb("#202020"), dark), rgb("#1e1e1e"));
    assert_close(filter.apply(rgb("#e0e0e0"), dark), rgb("#d4d4d4"));
  }

  #[test]
  fn rejects_out_of_range_options() {
    for invalid in [
      ColorFilterOptions {
        saturation: -1.0,
        ..options()
      },
      ColorFilterOptions {
        vibrance: 2.0,
        ..options()
      },
      ColorFilterOptions {
        warmth: f32::NAN,
        ..options()
      },
      ColorFilterOptions {
        min_contrast: 1.5,
        ..options()
      },
    ] {
      assert!(ColorFilter::new(&invalid).is_err(), "{invalid:?}");
    }
  }

  fn theme_with(
    elements: Vec<(UiElementKind, ElementTreatment)>,
  ) -> crate::Result<ColorTheme> {
    ColorTheme::new(ColorThemeOptions {
      filter: winter(),
      elements,
      detect_colors: false,
      skip_if_dark: false,
    })
  }

  fn tinted(hex: &str) -> ColorFilter {
    new_filter(&ColorFilterOptions {
      ramp: vec![stop("#ffffff", hex), stop("#000000", "#ffffff")],
      ..options()
    })
  }

  #[test]
  fn element_filters_share_slots() {
    let theme = theme_with(vec![
      (UiElementKind::Hyperlink, ElementTreatment::Original),
      (
        UiElementKind::Edit,
        ElementTreatment::Filter(tinted("#202040")),
      ),
      (
        UiElementKind::Button,
        ElementTreatment::Filter(tinted("#402020")),
      ),
      (
        UiElementKind::ComboBox,
        ElementTreatment::Filter(tinted("#202040")),
      ),
      (UiElementKind::TabItem, ElementTreatment::Filter(winter())),
    ])
    .expect("valid theme");

    assert_eq!(
      theme.element_slots(),
      vec![
        (UiElementKind::Hyperlink, SLOT_ORIGINAL),
        (UiElementKind::Edit, 1),
        (UiElementKind::Button, 2),
        (UiElementKind::ComboBox, 1),
        (UiElementKind::TabItem, 0),
      ]
    );

    let slots = theme.filter_constants();
    assert_eq!(slots[0], *winter().constants());
    assert_eq!(slots[1], *tinted("#202040").constants());
    assert_eq!(slots[3], FilterConstants::IDENTITY);
  }

  #[test]
  fn rejects_invalid_elements() {
    let too_many = (0..=MAX_ELEMENT_FILTERS)
      .map(|index| {
        let kind = [
          UiElementKind::Edit,
          UiElementKind::Button,
          UiElementKind::TabItem,
          UiElementKind::ListItem,
        ][index];
        (
          kind,
          ElementTreatment::Filter(tinted(
            ["#100000", "#200000", "#300000", "#400000"][index],
          )),
        )
      })
      .collect();
    assert!(theme_with(too_many).is_err());

    assert!(theme_with(vec![
      (UiElementKind::Button, ElementTreatment::Original),
      (UiElementKind::Button, ElementTreatment::Filter(winter())),
    ])
    .is_err());
  }

  #[test]
  fn element_regions_are_filtered_clipped_and_ordered() {
    let theme = ColorTheme::new(ColorThemeOptions {
      filter: winter(),
      elements: vec![
        (UiElementKind::Button, ElementTreatment::Original),
        (
          UiElementKind::Edit,
          ElementTreatment::Filter(tinted("#202040")),
        ),
      ],
      detect_colors: false,
      skip_if_dark: false,
    })
    .expect("valid theme");

    let element = |kind, ltrb| ElementRect { kind, ltrb };
    let regions = theme.element_regions(
      &[
        // Partly left of the frame.
        element(UiElementKind::Button, [-100, 50, 100, 250]),
        element(UiElementKind::Edit, [0, 0, 400, 30]),
        // Not configured.
        element(UiElementKind::ListItem, [0, 0, 50, 50]),
        // Entirely outside the frame.
        element(UiElementKind::Edit, [900, 900, 950, 950]),
      ],
      (800, 600),
    );

    assert_eq!(
      regions,
      vec![([0, 50, 100, 250], SLOT_ORIGINAL), ([0, 0, 400, 30], 1)]
    );
  }
}
