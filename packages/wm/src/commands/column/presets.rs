//! Stepping through the preset sizes of a scrolling layout's columns and
//! rows.

use std::ops::Add;

/// Index of the preset after `preset` among `count`, or with `forwards`
/// unset the one before, wrapping around.
pub(crate) fn step_preset(
  preset: usize,
  count: usize,
  forwards: bool,
) -> usize {
  if forwards {
    (preset + 1) % count.max(1)
  } else {
    preset.checked_sub(1).unwrap_or(count.saturating_sub(1))
  }
}

/// Index of the first of `sizes` bigger than `current` by more than
/// `leeway`, or with `forwards` unset the last one smaller by more than
/// it, wrapping around to the other end.
///
/// The leeway keeps rounding from holding a container on the preset it
/// already has.
pub(crate) fn nearest_preset<T>(
  sizes: &[T],
  current: T,
  leeway: T,
  forwards: bool,
) -> usize
where
  T: Add<Output = T> + PartialOrd + Copy,
{
  if forwards {
    sizes
      .iter()
      .position(|size| current + leeway < *size)
      .unwrap_or(0)
  } else {
    sizes
      .iter()
      .rposition(|size| *size + leeway < current)
      .unwrap_or(sizes.len().saturating_sub(1))
  }
}

#[cfg(test)]
mod tests {
  use super::{nearest_preset, step_preset};

  #[test]
  fn steps_wrap_around_both_ends() {
    assert_eq!(step_preset(0, 3, true), 1);
    assert_eq!(step_preset(2, 3, true), 0);
    assert_eq!(step_preset(1, 3, false), 0);
    assert_eq!(step_preset(0, 3, false), 2);
  }

  #[test]
  fn nearest_skips_presets_within_the_leeway() {
    let widths = [400, 600, 800];

    assert_eq!(nearest_preset(&widths, 600, 1, true), 2);
    assert_eq!(nearest_preset(&widths, 600, 1, false), 0);
    assert_eq!(nearest_preset(&widths, 599, 1, true), 2);
    assert_eq!(nearest_preset(&widths, 500, 1, true), 1);
  }

  #[test]
  fn nearest_wraps_past_the_last_preset() {
    let sizes = [0.25, 0.5];

    assert_eq!(nearest_preset(&sizes, 0.75, 0.01, true), 0);
    assert_eq!(nearest_preset(&sizes, 0.1, 0.01, false), 1);
  }
}
