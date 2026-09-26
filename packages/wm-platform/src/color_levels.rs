//! Measuring a window's own paper and ink colors from a sparse sample of
//! its pixels, for themes with `detect_colors` or `skip_if_dark`.

use crate::color_theme::{srgb_to_oklab, SourceLevels};

/// Histogram resolution for finding the paper's lightness.
const HISTOGRAM_BINS: usize = 32;

/// Fewer samples than this (e.g. a tiny popup) aren't worth measuring.
const MIN_SAMPLES: usize = 64;

/// OKLab lightness distance from the paper beyond which a pixel counts as
/// ink rather than a shade of the background.
const MIN_INK_CONTRAST: f32 = 0.3;

/// Share of samples that must be ink for it to be measured; below it the
/// window has too little text to tell, and black (or white) ink is
/// assumed.
const MIN_INK_SHARE: f32 = 0.002;

/// Percentile of ink candidates, from the far end, taken as the ink: past
/// the anti-aliased edges, but robust to a few outliers.
const INK_PERCENTILE: f32 = 0.1;

/// Paper lightness below which a light window turns dark, and above which
/// a dark one turns light again; apart, so a window near the line doesn't
/// flicker between the two.
const DARK_ENTER: f32 = 0.4;
const DARK_EXIT: f32 = 0.55;

/// Changes in measured lightness smaller than these are ignored, so
/// scrolling past an image doesn't shift the colors.
const PAPER_HYSTERESIS: f32 = 0.03;
const INK_HYSTERESIS: f32 = 0.05;

/// Estimates paper and ink from straight-alpha sRGB samples of a window.
///
/// The paper is the most common lightness. The ink is the far end of the
/// pixels clearly darker (or, on dark paper, lighter) than it.
#[must_use]
pub(crate) fn estimate_levels(
  samples: &[[f32; 3]],
) -> Option<SourceLevels> {
  if samples.len() < MIN_SAMPLES {
    return None;
  }

  let lightness = samples
    .iter()
    .map(|sample| srgb_to_oklab(*sample)[0].clamp(0.0, 1.0))
    .collect::<Vec<_>>();

  #[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
  )]
  let bin = |value: f32| {
    ((value * HISTOGRAM_BINS as f32) as usize).min(HISTOGRAM_BINS - 1)
  };

  let mut histogram = [0usize; HISTOGRAM_BINS];
  for value in &lightness {
    histogram[bin(*value)] += 1;
  }

  let paper_bin = (0..HISTOGRAM_BINS)
    .max_by_key(|index| histogram[*index])
    .unwrap_or(0);

  #[allow(clippy::cast_precision_loss)]
  let paper = lightness
    .iter()
    .filter(|value| bin(**value) == paper_bin)
    .sum::<f32>()
    / histogram[paper_bin].max(1) as f32;

  let paper_is_light = paper >= 0.5;
  let mut ink_candidates = lightness
    .iter()
    .copied()
    .filter(|value| {
      if paper_is_light {
        *value < paper - MIN_INK_CONTRAST
      } else {
        *value > paper + MIN_INK_CONTRAST
      }
    })
    .collect::<Vec<_>>();

  #[allow(clippy::cast_precision_loss)]
  let enough_ink = ink_candidates.len() as f32
    >= (samples.len() as f32 * MIN_INK_SHARE).max(3.0);

  let ink = if enough_ink {
    // Sorted from the far end towards the paper.
    ink_candidates.sort_by(|a, b| {
      if paper_is_light {
        a.total_cmp(b)
      } else {
        b.total_cmp(a)
      }
    });

    #[allow(
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss,
      clippy::cast_precision_loss
    )]
    let index = (ink_candidates.len() as f32 * INK_PERCENTILE) as usize;
    ink_candidates[index.min(ink_candidates.len() - 1)]
  } else if paper_is_light {
    0.0
  } else {
    1.0
  };

  Some(SourceLevels { paper, ink })
}

/// Smooths successive [`estimate_levels`] results for one window.
#[derive(Clone, Debug, Default)]
pub(crate) struct LevelsTracker {
  levels: Option<SourceLevels>,
  is_dark: bool,
}

impl LevelsTracker {
  /// Folds in a new estimate. Returns whether [`Self::levels`] or
  /// [`Self::is_dark`] changed.
  pub fn update(&mut self, estimate: SourceLevels) -> bool {
    let was_dark = self.is_dark;

    if self.is_dark {
      self.is_dark = estimate.paper <= DARK_EXIT;
    } else {
      self.is_dark = estimate.paper < DARK_ENTER;
    }

    let levels_changed = self.levels.is_none_or(|current| {
      (current.paper - estimate.paper).abs() > PAPER_HYSTERESIS
        || (current.ink - estimate.ink).abs() > INK_HYSTERESIS
    });

    if levels_changed {
      self.levels = Some(estimate);
    }

    levels_changed || was_dark != self.is_dark
  }

  /// The measured levels, or black on white until the first estimate.
  #[must_use]
  pub fn levels(&self) -> SourceLevels {
    self.levels.unwrap_or_default()
  }

  #[must_use]
  pub fn is_dark(&self) -> bool {
    self.is_dark
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn gray(value: f32) -> [f32; 3] {
    [value; 3]
  }

  /// A window of `paper` with `ink_share` of it covered by `ink`, plus
  /// some anti-aliased pixels between the two.
  fn window(paper: f32, ink: f32, ink_share: f32) -> Vec<[f32; 3]> {
    #[allow(
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss,
      clippy::cast_precision_loss
    )]
    let ink_count = (1000.0 * ink_share) as usize;

    let mut samples = vec![gray(paper); 1000 - ink_count * 2];
    samples.extend(vec![gray(ink); ink_count]);
    samples.extend(
      (0..ink_count)
        .map(|index| {
          #[allow(clippy::cast_precision_loss)]
          let t = index as f32 / ink_count as f32;
          gray(paper + (ink - paper) * t)
        })
        .collect::<Vec<_>>(),
    );
    samples
  }

  fn lightness(value: f32) -> f32 {
    srgb_to_oklab(gray(value))[0]
  }

  #[test]
  fn finds_off_white_paper_and_gray_ink() {
    let levels =
      estimate_levels(&window(0.95, 0.2, 0.05)).expect("enough samples");

    assert!((levels.paper - lightness(0.95)).abs() < 0.01, "{levels:?}");
    assert!((levels.ink - lightness(0.2)).abs() < 0.02, "{levels:?}");
  }

  #[test]
  fn finds_light_ink_on_dark_paper() {
    let levels =
      estimate_levels(&window(0.12, 0.85, 0.05)).expect("enough samples");

    assert!((levels.paper - lightness(0.12)).abs() < 0.01, "{levels:?}");
    assert!((levels.ink - lightness(0.85)).abs() < 0.02, "{levels:?}");
  }

  #[test]
  fn assumes_black_ink_without_text() {
    let levels =
      estimate_levels(&vec![gray(0.97); 500]).expect("enough samples");

    assert!(levels.ink.abs() < f32::EPSILON);
  }

  #[test]
  fn skips_tiny_samples() {
    assert!(estimate_levels(&[gray(1.0); 10]).is_none());
  }

  #[test]
  fn tracker_ignores_small_changes_and_switches_dark_with_hysteresis() {
    let mut tracker = LevelsTracker::default();
    let light = SourceLevels {
      paper: 0.95,
      ink: 0.1,
    };

    assert!(tracker.update(light));
    assert!(!tracker.update(SourceLevels {
      paper: 0.94,
      ink: 0.12,
    }));
    assert!(!tracker.is_dark());

    // Between the thresholds, a light window stays light...
    let middle = SourceLevels {
      paper: 0.45,
      ink: 0.1,
    };
    tracker.update(middle);
    assert!(!tracker.is_dark());

    // ...and a dark one stays dark.
    assert!(tracker.update(SourceLevels {
      paper: 0.2,
      ink: 0.9,
    }));
    assert!(tracker.is_dark());
    tracker.update(middle);
    assert!(tracker.is_dark());

    assert!(tracker.update(light));
    assert!(!tracker.is_dark());
  }
}
