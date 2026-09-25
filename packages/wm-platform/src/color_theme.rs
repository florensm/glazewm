//! Per-window color themes: the color math, as a CPU reference for the
//! pixel shader in `shaders/color_theme.hlsl`.
//!
//! The shader is a line-for-line port of
//! [`ColorTheme::apply_neighborhood`] (and the [`ColorTheme::apply`] it
//! builds on) and reads the exact [`ThemeConstants`] this module builds,
//! so the unit tests here cover what the GPU computes. Any change to one
//! must be mirrored in the other.

use crate::Color;

/// Maximum number of color overrides per theme; sizes the shader's
/// constant buffer.
pub const MAX_COLOR_OVERRIDES: usize = 16;

/// OKLab chroma treated as fully saturated, roughly that of pure sRGB
/// blue (the most chromatic sRGB primary).
const MAX_CHROMA: f32 = 0.32;

/// OKLab lightness over which a color goes from judged by the configured
/// saturation threshold to judged as a pale tint (see [`tint_threshold`]).
const TINT_LIGHTNESS_START: f32 = 0.8;
const TINT_LIGHTNESS_FULL: f32 = 0.9;

/// How much a pale tint's saturation threshold is scaled up, and the
/// saturation it's capped at so vivid light colors (yellow, amber, light
/// green) still keep their color.
const TINT_THRESHOLD_SCALE: f32 = 2.5;
const TINT_THRESHOLD_MAX: f32 = 0.4;

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

/// Pixels in the window [`ColorTheme::apply_neighborhood`] reads: 5 wide,
/// 3 tall.
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

/// Coverage gamma for edges the theme turns light-on-dark, which read
/// thinner than the same coverage dark-on-light.
const INVERTED_TEXT_GAMMA: f32 = 1.4;

/// OKLab lightness by which the themed ink must exceed the themed paper
/// for [`INVERTED_TEXT_GAMMA`] to fully apply.
const INVERSION_FULL: f32 = 0.1;

/// OKLab distance from a page color over which a pixel inside an image
/// goes from page (themed) to image content (kept). Tight: a photo's own
/// near-white must stay, the page's flat color matches almost exactly.
const IMAGE_PAPER_MATCH_START: f32 = 0.01;
const IMAGE_PAPER_MATCH_FULL: f32 = 0.03;

/// Fewest non-page pixels for an image to be judged a picture at all.
const PICTURE_MIN_CONTENT_PIXELS: usize = 16;

/// sRGB distance from the page -> ink line beyond which a pixel is one no
/// single ink explains.
const PICTURE_OFF_LINE: f32 = 0.1;

/// Share of an image's non-page pixels no single ink explains over which
/// it is a picture. Anti-aliased icons stay near 0; photos are far above.
const PICTURE_MIN_UNEXPLAINED: f32 = 0.2;

/// Override tolerances are configured as OKLab distance times 100, so they
/// read like the familiar CIE ΔE scale (~2 is barely noticeable).
const TOLERANCE_SCALE: f32 = 100.0;

/// An explicit color mapping applied on top of the gray ramp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorOverride {
  pub from: Color,
  pub to: Color,

  /// OKLab distance times 100 within which pixels blend towards `to`.
  pub tolerance: f32,
}

/// A validated color theme, ready to upload to the GPU.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTheme {
  constants: ThemeConstants,
}

/// Constant buffer layout shared with `cbuffer Theme` in the shader.
///
/// Colors are pre-converted to OKLab so neither side converts them per
/// pixel.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ThemeConstants {
  /// OKLab color white maps to.
  background: [f32; 4],

  /// OKLab color black maps to.
  foreground: [f32; 4],

  saturation_threshold: f32,
  override_count: u32,

  /// Non-zero when the gray ramp is enabled.
  ramp_enabled: u32,
  _padding: u32,

  /// Per override: `from` OKLab with the tolerance (as an OKLab distance)
  /// in `w`, then `to` OKLab.
  overrides: [[f32; 4]; MAX_COLOR_OVERRIDES * 2],
}

impl ColorTheme {
  /// Validates and compiles a theme.
  ///
  /// `ramp` is `(background, foreground)`: white maps to the former,
  /// black to the latter. Alpha channels are ignored.
  pub fn new(
    ramp: Option<(Color, Color)>,
    saturation_threshold: f32,
    overrides: &[ColorOverride],
  ) -> crate::Result<Self> {
    if !(0.0..=1.0).contains(&saturation_threshold) {
      return Err(crate::Error::Platform(format!(
        "Saturation threshold {saturation_threshold} must be between 0 and 1."
      )));
    }

    if overrides.len() > MAX_COLOR_OVERRIDES {
      return Err(crate::Error::Platform(format!(
        "A color theme supports at most {MAX_COLOR_OVERRIDES} overrides, got {}.",
        overrides.len()
      )));
    }

    let mut constants = ThemeConstants {
      background: [0.0; 4],
      foreground: [0.0; 4],
      saturation_threshold,
      override_count: 0,
      ramp_enabled: 0,
      _padding: 0,
      overrides: [[0.0; 4]; MAX_COLOR_OVERRIDES * 2],
    };

    if let Some((background, foreground)) = ramp {
      constants.background =
        with_w(srgb_to_oklab(to_rgb(background)), 0.0);
      constants.foreground =
        with_w(srgb_to_oklab(to_rgb(foreground)), 0.0);
      constants.ramp_enabled = 1;
    }

    for (index, color_override) in overrides.iter().enumerate() {
      if !color_override.tolerance.is_finite()
        || color_override.tolerance < 0.0
      {
        return Err(crate::Error::Platform(format!(
          "Override tolerance {} must be a non-negative number.",
          color_override.tolerance
        )));
      }

      constants.overrides[index * 2] = with_w(
        srgb_to_oklab(to_rgb(color_override.from)),
        color_override.tolerance / TOLERANCE_SCALE,
      );
      constants.overrides[index * 2 + 1] =
        with_w(srgb_to_oklab(to_rgb(color_override.to)), 0.0);
    }

    // Bounded by `MAX_COLOR_OVERRIDES` above.
    #[allow(clippy::cast_possible_truncation)]
    {
      constants.override_count = overrides.len() as u32;
    }

    Ok(Self { constants })
  }

  pub(crate) fn constants(&self) -> &ThemeConstants {
    &self.constants
  }

  /// Maps one straight-alpha sRGB color (components in `0.0..=1.0`)
  /// through the theme.
  #[must_use]
  pub fn apply(&self, srgb: [f32; 3]) -> [f32; 3] {
    let c = &self.constants;
    let lab = srgb_to_oklab(srgb);
    let mut result = lab;

    if c.ramp_enabled != 0 {
      let chroma = lab[1].hypot(lab[2]);
      let saturation = (chroma / MAX_CHROMA).min(1.0);
      let threshold = tint_threshold(c.saturation_threshold, lab[0]);
      let weight = ramp_weight(saturation, threshold);

      // Lightness picks the spot on the ramp; the source's own chroma is
      // carried over so faintly tinted grays keep their tint.
      let t = lab[0].clamp(0.0, 1.0);
      let ramped = [
        lerp(c.foreground[0], c.background[0], t),
        lerp(c.foreground[1], c.background[1], t) + lab[1],
        lerp(c.foreground[2], c.background[2], t) + lab[2],
      ];

      result = lerp3(lab, ramped, weight);
    }

    let mut best_weight = 0.0;
    let mut best_to = [0.0; 3];

    for index in 0..c.override_count as usize {
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
  pub fn apply_color(&self, color: Color) -> Color {
    let [r, g, b] = self.apply(to_rgb(color)).map(to_byte);
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
  ) -> [f32; 3] {
    let center = pixels[NEIGHBORHOOD_SIZE / 2];
    let themed_center = self.apply(center);

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
    let themed_dark = self.apply(dark);
    let themed_light = self.apply(light);

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
    let unexplained =
      lerp3(themed_center, self.apply(neutral(center)), edge * fringes);
    lerp3(unexplained, remixed, edge * fit)
  }

  /// Maps the center of a 5×3 neighborhood inside an image through the
  /// theme: its own colors are kept, and only pixels matching one of
  /// `papers` (the page colors sampled around the image) are themed.
  ///
  /// Images are reported as rectangles, while their content is often
  /// round or rounded; the page showing through their corners is themed
  /// with the rest of the page. Where the image's edge is anti-aliased
  /// into the page, the pixel is re-mixed from the themed page and the
  /// kept image color at the same coverage.
  #[must_use]
  pub fn apply_image_neighborhood(
    &self,
    pixels: &[[f32; 3]; NEIGHBORHOOD_SIZE],
    papers: &[[f32; 3]],
  ) -> [f32; 3] {
    let center = pixels[NEIGHBORHOOD_SIZE / 2];
    let themed_center = self.apply_image(center, papers);

    let mut paper = center;
    let mut paper_weight = image_paper_weight(center, papers);

    for pixel in pixels {
      let weight = image_paper_weight(*pixel, papers);

      if weight > paper_weight {
        paper = *pixel;
        paper_weight = weight;
      }
    }

    if paper_weight <= 0.0 {
      return themed_center;
    }

    // The image side of the edge: the pixel least like the page.
    let mut content = paper;
    let mut content_distance = 0.0;

    for pixel in pixels {
      let gap = distance(*pixel, paper);

      if gap > content_distance {
        content = *pixel;
        content_distance = gap;
      }
    }

    if content_distance < MIN_CHANNEL_SPAN {
      return themed_center;
    }

    let mut along = 0.0;

    for c in 0..3 {
      along += (center[c] - paper[c]) * (content[c] - paper[c]);
    }

    let coverage =
      (along / (content_distance * content_distance)).clamp(0.0, 1.0);
    let fit = 1.0
      - smoothstep(
        MIX_ERROR_START,
        MIX_ERROR_FULL,
        distance(center, lerp3(paper, content, coverage)),
      );

    let remixed = lerp3(
      self.apply_image(paper, papers),
      self.apply_image(content, papers),
      coverage,
    );
    lerp3(themed_center, remixed, fit * paper_weight)
  }

  /// [`apply`](Self::apply) for a pixel inside an image: themed only as
  /// far as it matches one of `papers`.
  fn apply_image(&self, srgb: [f32; 3], papers: &[[f32; 3]]) -> [f32; 3] {
    lerp3(srgb, self.apply(srgb), image_paper_weight(srgb, papers))
  }
}

/// Whether an image's pixels are a picture to keep in its own colors,
/// rather than an icon to theme like text.
///
/// An icon is drawn in one ink over the page, so its pixels all lie
/// between one of `papers` and that ink; a picture has too many pixels
/// no single ink explains. Images that are mostly page count as icons.
#[must_use]
pub fn is_picture(pixels: &[[f32; 3]], papers: &[[f32; 3]]) -> bool {
  let content: Vec<[f32; 3]> = pixels
    .iter()
    .copied()
    .filter(|pixel| image_paper_weight(*pixel, papers) < 0.5)
    .collect();

  if content.len() < PICTURE_MIN_CONTENT_PIXELS {
    return false;
  }

  // The page the image sits on, and the ink farthest from it.
  let paper = papers.first().copied().unwrap_or([1.0; 3]);
  let ink = content.iter().copied().fold(paper, |farthest, pixel| {
    if distance(pixel, paper) > distance(farthest, paper) {
      pixel
    } else {
      farthest
    }
  });

  let span = distance(ink, paper);
  if span < MIN_CHANNEL_SPAN {
    return false;
  }

  let unexplained = content
    .iter()
    .filter(|pixel| {
      let mut along = 0.0;
      for c in 0..3 {
        along += (pixel[c] - paper[c]) * (ink[c] - paper[c]);
      }
      let coverage = (along / (span * span)).clamp(0.0, 1.0);
      distance(**pixel, lerp3(paper, ink, coverage)) > PICTURE_OFF_LINE
    })
    .count();

  #[allow(clippy::cast_precision_loss)]
  let unexplained_share = unexplained as f32 / content.len() as f32;
  unexplained_share > PICTURE_MIN_UNEXPLAINED
}

/// How closely `srgb` matches one of `papers`: 1 up to
/// [`IMAGE_PAPER_MATCH_START`], easing out to 0 at
/// [`IMAGE_PAPER_MATCH_FULL`].
fn image_paper_weight(srgb: [f32; 3], papers: &[[f32; 3]]) -> f32 {
  let lab = srgb_to_oklab(srgb);

  papers.iter().fold(0.0, |weight, paper| {
    let gap = distance(lab, srgb_to_oklab(*paper));
    weight.max(
      1.0
        - smoothstep(IMAGE_PAPER_MATCH_START, IMAGE_PAPER_MATCH_FULL, gap),
    )
  })
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
///
/// Names and constants follow that reference (and the shader) verbatim.
#[allow(clippy::excessive_precision, clippy::many_single_char_names)]
fn srgb_to_oklab(srgb: [f32; 3]) -> [f32; 3] {
  let [r, g, b] = srgb.map(srgb_to_linear);

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

/// OKLab to (unclamped) sRGB; inverse of [`srgb_to_oklab`].
#[allow(clippy::excessive_precision, clippy::many_single_char_names)]
fn oklab_to_srgb(lab: [f32; 3]) -> [f32; 3] {
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
  .map(linear_to_srgb)
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  const EPSILON: f32 = 2.0 / 255.0;

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

  fn winter() -> ColorTheme {
    ColorTheme::new(Some((color("#1e1e1e"), color("#d4d4d4"))), 0.15, &[])
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

    assert_close(theme.apply(rgb("#ffffff")), rgb("#1e1e1e"));
    assert_close(theme.apply(rgb("#000000")), rgb("#d4d4d4"));
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
      let [r, g, b] = theme.apply([v, v, v]);

      assert!(r <= previous + 1e-4, "not monotonic at {v}: {r}");
      assert!((background - 1e-3..=foreground + 1e-3).contains(&r));
      assert!((r - g).abs() < 1e-3 && (g - b).abs() < 1e-3);
      previous = r;
    }

    let mid = theme.apply([0.5, 0.5, 0.5])[0];
    assert!(mid > background + 0.1 && mid < foreground - 0.1);
  }

  #[test]
  fn apply_color_matches_apply_and_keeps_alpha() {
    let theme = winter();

    assert_eq!(theme.apply_color(color("#ffffff80")), color("#1e1e1e80"));
    assert_eq!(theme.apply_color(color("#000000")), color("#d4d4d4"));
    assert_eq!(theme.apply_color(color("#0078d4")), color("#0078d4"));
  }

  #[test]
  fn saturated_pixels_above_threshold_are_untouched() {
    let theme = winter();

    for hex in ["#0078d4", "#e81123", "#16c60c", "#fff100"] {
      assert_close(theme.apply(rgb(hex)), rgb(hex));
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
      let themed = srgb_to_oklab(theme.apply(rgb(hex)));
      assert!(themed[0] < 0.4, "{hex} stayed light: {themed:?}");
    }
  }

  #[test]
  fn vivid_light_colors_keep_their_color() {
    let theme = winter();

    for hex in ["#fff100", "#ffb900", "#90ee90"] {
      assert_close(theme.apply(rgb(hex)), rgb(hex));
    }
  }

  #[test]
  fn threshold_zero_disables_the_ramp() {
    let theme = ColorTheme::new(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.0,
      &[],
    )
    .expect("valid theme");

    assert_close(theme.apply(rgb("#ffffff")), rgb("#ffffff"));
    assert_close(theme.apply(rgb("#bee6fd")), rgb("#bee6fd"));
  }

  #[test]
  fn no_ramp_is_identity() {
    let theme = ColorTheme::new(None, 0.15, &[]).expect("valid theme");

    for hex in ["#ffffff", "#000000", "#0078d4", "#7f7f7f"] {
      assert_close(theme.apply(rgb(hex)), rgb(hex));
    }
  }

  #[test]
  fn override_replaces_exact_match_even_when_saturated() {
    let theme = ColorTheme::new(
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

    assert_close(theme.apply(rgb("#fff3b0")), rgb("#5a4a00"));
    assert_close(theme.apply(rgb("#0078d4")), rgb("#4aa3ff"));
  }

  #[test]
  fn override_falls_off_softly_within_tolerance() {
    let from = color("#fff3b0");
    let to = color("#5a4a00");
    let theme = ColorTheme::new(
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
      let out = srgb_to_oklab(theme.apply(oklab_to_srgb(source)));
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
    let theme = ColorTheme::new(
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

    assert_close(theme.apply(rgb("#ff1010")), rgb("#0000ff"));
  }

  #[test]
  fn rejects_invalid_values() {
    assert!(ColorTheme::new(None, 1.5, &[]).is_err());
    assert!(ColorTheme::new(None, -0.1, &[]).is_err());

    let bad_tolerance = ColorOverride {
      from: color("#000000"),
      to: color("#ffffff"),
      tolerance: -1.0,
    };
    assert!(ColorTheme::new(None, 0.1, &[bad_tolerance]).is_err());

    let many = vec![
      ColorOverride {
        from: color("#000000"),
        to: color("#ffffff"),
        tolerance: 1.0,
      };
      MAX_COLOR_OVERRIDES + 1
    ];
    assert!(ColorTheme::new(None, 0.1, &many).is_err());
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
        theme.apply_neighborhood(&[rgb(hex); NEIGHBORHOOD_SIZE]),
        theme.apply(rgb(hex)),
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
    let same_polarity = ColorTheme::new(
      Some((color("#fdf6e3"), color("#073642"))),
      0.15,
      &[],
    )
    .expect("valid theme");
    assert_close(
      same_polarity.apply_neighborhood(&edge_block(black, edge, white)),
      lerp_rgb(rgb("#fdf6e3"), rgb("#073642"), [0.25; 3]),
    );

    // Flipped to light-on-dark, the same edge is boosted.
    assert_close(
      winter().apply_neighborhood(&edge_block(black, edge, white)),
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
    let out =
      theme.apply_neighborhood(&rows([black, black, blue, orange, white]));

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
      let out = theme.apply_neighborhood(&rows(row));
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
    let out = theme.apply_neighborhood(&rows([
      white,
      px(255, 255, 186),
      px(106, 0, 106),
      px(186, 255, 255),
      white,
    ]));
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
    let out = winter().apply_neighborhood(&hex_rows([
      ["#ffffb5", "#630034", "#8ddada", "#8d3400", "#63b5ff"],
      ["#ffffb5", "#630034", "#343400", "#348dda", "#ffffff"],
      ["#ffffb5", "#630000", "#343434", "#0063b5", "#ffffff"],
    ]));

    assert_gray(out);
    assert!(out[0] > 0.4, "diagonal should read as light ink: {out:?}");
  }

  #[test]
  fn fringes_between_close_stems_become_grayscale() {
    // Measured between the "i" and "l" of a WPF "File": no paper shows.
    let row = ["#f0f0b1", "#6f214a", "#91d0f0", "#f0f0b1", "#6f214a"];
    assert_gray(winter().apply_neighborhood(&hex_rows([row, row, row])));

    // Measured between two stems of "Right-click", paper side a fringe.
    assert_gray(winter().apply_neighborhood(&hex_rows([
      ["#5e0032", "#86d0f3", "#f3f3ac", "#5e0032", "#86ac86"],
      ["#000032", "#86d0f3", "#f3f3ac", "#5e0000", "#003286"],
      ["#5e0032", "#86d0f3", "#f3f3ac", "#5e0032", "#86d0f3"],
    ])));
  }

  #[test]
  fn highlight_paper_between_letters_stays_highlighted() {
    let theme = ColorTheme::new(
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
    let out = theme.apply_neighborhood(&rows([
      paper,
      px(0, 0, 11),
      px(255, 243, 170),
      px(178, 91, 6),
      paper,
    ]));

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
    let out = theme.apply_neighborhood(&rows([
      paper,
      px(255, 238, 146),
      stem,
      px(160, 212, 176),
      paper,
    ]));

    // The stem reads as neutral ink, not as the dark red fringe.
    let spread = out.iter().fold(0.0_f32, |m, c| m.max(*c))
      - out.iter().fold(1.0_f32, |m, c| m.min(*c));
    assert!(spread < 0.05, "stem stayed colored: {out:?}");
    assert!(out[0] > 0.4, "stem should read as light ink: {out:?}");
  }

  #[test]
  fn thin_colored_text_keeps_its_color() {
    let theme = ColorTheme::new(
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
    let out = theme.apply_neighborhood(&rows([
      rgb("#ffffff"),
      [0.7, 0.68, 0.85],
      ink,
      [0.1, 0.68, 0.95],
      rgb("#ffffff"),
    ]));

    assert_close(out, rgb("#4aa3ff"));
  }

  #[test]
  fn hairline_colored_text_keeps_its_hue() {
    let theme = ColorTheme::new(
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
      let out = srgb_to_oklab(theme.apply_neighborhood(&rows(row)));
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
    let theme = ColorTheme::new(
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
    let out = srgb_to_oklab(theme.apply_neighborhood(&bar));
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
    let out = srgb_to_oklab(theme.apply_neighborhood(&stem));
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
    let out = srgb_to_oklab(theme.apply_neighborhood(&fringe));
    assert!(
      out[1].hypot(out[2]) < 0.03,
      "fringe took a color of its own: {out:?}"
    );
  }

  #[test]
  fn image_content_keeps_its_colors() {
    let theme = winter();
    let papers = [rgb("#fbfbfb")];

    // Skin, gray, and near-white photo pixels are not the page.
    for hex in ["#e0b89a", "#808080", "#2b2b2b", "#f0ece4"] {
      assert_close(
        theme.apply_image_neighborhood(
          &[rgb(hex); NEIGHBORHOOD_SIZE],
          &papers,
        ),
        rgb(hex),
      );
    }

    // Nor is an edge within the image.
    let (gray, dark) = (rgb("#808080"), rgb("#404040"));
    let mix = lerp3(gray, dark, 0.5);
    assert_close(
      theme
        .apply_image_neighborhood(&edge_block(gray, mix, dark), &papers),
      mix,
    );
  }

  #[test]
  fn page_inside_an_image_rect_is_themed() {
    let theme = winter();
    let paper = rgb("#fbfbfb");

    // The corner of a round avatar's bounding rect shows the page.
    assert_close(
      theme
        .apply_image_neighborhood(&[paper; NEIGHBORHOOD_SIZE], &[paper]),
      theme.apply(paper),
    );
  }

  #[test]
  fn round_image_edge_blends_into_the_themed_page() {
    let theme = winter();
    let (paper, skin) = (rgb("#fbfbfb"), rgb("#e0b89a"));
    let edge = lerp3(paper, skin, 0.5);

    // The anti-aliased rim of a circle-clipped photo: half page, half
    // face.
    assert_close(
      theme.apply_image_neighborhood(
        &edge_block(paper, edge, skin),
        &[paper],
      ),
      lerp3(theme.apply(paper), skin, 0.5),
    );
  }

  #[test]
  fn monochrome_icon_is_not_a_picture() {
    let (paper, ink) = (rgb("#fbfbfb"), rgb("#1f1f1f"));

    // A glyph: page, ink, and anti-aliased mixes of the two.
    let pixels: Vec<[f32; 3]> = (0..64)
      .map(|i| lerp3(paper, ink, f32::from(i % 9_u8) / 8.0))
      .collect();

    assert!(!is_picture(&pixels, &[paper]));
  }

  #[test]
  fn photo_is_a_picture() {
    let paper = rgb("#fbfbfb");
    let tones = ["#e0b89a", "#3c2415", "#b4875b", "#f0ece4", "#5d8fbf"];

    let pixels: Vec<[f32; 3]> =
      (0..64).map(|i| rgb(tones[i % tones.len()])).collect();

    assert!(is_picture(&pixels, &[paper]));
  }

  #[test]
  fn blank_image_is_not_a_picture() {
    let paper = rgb("#fbfbfb");

    // Not loaded yet: nothing but page.
    assert!(!is_picture(&[paper; 64], &[paper]));
  }

  #[test]
  fn colored_pixel_between_black_and_white_is_not_a_fringe() {
    let theme = winter();
    let red = rgb("#e81123");
    let out = theme.apply_neighborhood(&edge_block(
      rgb("#000000"),
      red,
      rgb("#ffffff"),
    ));

    // The row is far from gray, so this is real color, left untouched.
    assert_close(out, red);
  }

  #[test]
  fn colorful_image_edges_are_untouched() {
    let theme = winter();
    let (red, green) = (rgb("#e81123"), rgb("#16c60c"));
    let mix = lerp3(red, green, 0.5);

    assert_close(
      theme.apply_neighborhood(&edge_block(red, mix, green)),
      mix,
    );
  }

  #[test]
  fn button_edges_blend_between_themed_colors() {
    let theme = ColorTheme::new(
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
      theme.apply_neighborhood(&edge_block(blue, corner, white)),
      lerp_rgb(rgb("#1e1e1e"), rgb("#4aa3ff"), inverted(0.5)),
    );
  }

  #[test]
  fn constants_match_the_shader_layout() {
    // `cbuffer Theme`: two float4s, one packed register, then the
    // override float4 array.
    assert_eq!(
      std::mem::size_of::<ThemeConstants>(),
      16 * (3 + MAX_COLOR_OVERRIDES * 2)
    );
  }
}
