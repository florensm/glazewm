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
      let weight = ramp_weight(saturation, c.saturation_threshold);

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

  /// Maps the center of a 3×3 neighborhood (row-major, straight-alpha
  /// sRGB) through the theme, keeping anti-aliased edges intact.
  ///
  /// An edge pixel is a mix of the colors on either side of it, so it is
  /// re-mixed from those colors' *themed* values at the same coverage,
  /// rather than themed as a color of its own. Coverage is per channel
  /// where the row averages out gray, which is what subpixel (ClearType)
  /// text looks like; elsewhere a single coverage is used, so a colored
  /// pixel between black and white isn't misread as a fringe.
  #[must_use]
  pub fn apply_neighborhood(&self, pixels: &[[f32; 3]; 9]) -> [f32; 3] {
    let center = pixels[4];
    let themed_center = self.apply(center);

    let mut dark = center;
    let mut light = center;
    let mut dark_lightness = srgb_to_oklab(center)[0];
    let mut light_lightness = dark_lightness;

    for pixel in pixels {
      let lightness = srgb_to_oklab(*pixel)[0];

      if lightness < dark_lightness {
        dark = *pixel;
        dark_lightness = lightness;
      }

      if lightness > light_lightness {
        light = *pixel;
        light_lightness = lightness;
      }
    }

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

    // Subpixel fringes cancel out across the row; real color doesn't.
    let row_average =
      lerp3(lerp3(pixels[3], pixels[5], 0.5), center, 1.0 / 3.0);
    let row_lab = srgb_to_oklab(row_average);
    let neutral = ramp_weight(
      (row_lab[1].hypot(row_lab[2]) / MAX_CHROMA).min(1.0),
      self.constants.saturation_threshold,
    );

    let mut error_sq = 0.0;

    for c in 0..3 {
      let own = if valid[c] { coverage[c] } else { mean_coverage };
      coverage[c] = lerp(mean_coverage, own, neutral);

      let reconstructed = lerp(light[c], dark[c], coverage[c]);
      error_sq +=
        (center[c] - reconstructed) * (center[c] - reconstructed);
    }

    let fit =
      1.0 - smoothstep(MIX_ERROR_START, MIX_ERROR_FULL, error_sq.sqrt());
    let themed_dark = self.apply(dark);
    let themed_light = self.apply(light);

    let remixed = [
      lerp(themed_light[0], themed_dark[0], coverage[0]),
      lerp(themed_light[1], themed_dark[1], coverage[1]),
      lerp(themed_light[2], themed_dark[2], coverage[2]),
    ];

    lerp3(themed_center, remixed, edge * fit)
  }
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
  fn threshold_zero_disables_the_ramp() {
    let theme = ColorTheme::new(
      Some((color("#1e1e1e"), color("#d4d4d4"))),
      0.0,
      &[],
    )
    .expect("valid theme");

    assert_close(theme.apply(rgb("#ffffff")), rgb("#ffffff"));
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

  /// A neighborhood with `center` in the middle, `left` in the left
  /// column and `right` in the right one.
  fn edge_block(
    left: [f32; 3],
    center: [f32; 3],
    right: [f32; 3],
  ) -> [[f32; 3]; 9] {
    [
      left, center, right, left, center, right, left, center, right,
    ]
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
        theme.apply_neighborhood(&[rgb(hex); 9]),
        theme.apply(rgb(hex)),
      );
    }
  }

  #[test]
  fn grayscale_antialiasing_keeps_its_coverage() {
    let theme = winter();
    let (black, white) = (rgb("#000000"), rgb("#ffffff"));

    // A 25%-covered text edge re-mixes the themed text and page colors at
    // that same 25%, rather than landing wherever the ramp puts `#bfbfbf`.
    let edge = [0.75; 3];
    let out = theme.apply_neighborhood(&edge_block(black, edge, white));

    assert_close(out, lerp_rgb(rgb("#1e1e1e"), rgb("#d4d4d4"), [0.25; 3]));
  }

  #[test]
  fn subpixel_fringes_keep_their_per_channel_coverage() {
    let theme = winter();
    let (black, white) = (rgb("#000000"), rgb("#ffffff"));

    // ClearType: a blue fringe between a black stroke and an orange one,
    // which together average out gray across the row.
    let blue = [0.2, 0.6, 1.0];
    let orange = [1.0, 0.6, 0.2];
    let out = theme.apply_neighborhood(&[
      black, black, white, //
      black, blue, orange, //
      black, black, black,
    ]);

    // Each channel keeps its own coverage in the new colors.
    assert_close(
      out,
      lerp_rgb(rgb("#1e1e1e"), rgb("#d4d4d4"), [0.8, 0.4, 0.0]),
    );
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
      lerp_rgb(rgb("#1e1e1e"), rgb("#4aa3ff"), [0.5; 3]),
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
