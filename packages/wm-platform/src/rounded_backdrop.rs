//! `GLAZEWM_ROUNDED_BACKDROP`: a temporary switch for comparing, on
//! screen, ways of drawing a backdrop with rounded corners on a pinned
//! peek or surrogate, which can't draw it as one rectangular copy
//! (`ONE_SURFACE_PLAN.md`, "Rounded backdrops: as built"). Goes once one
//! is chosen.
//!
//! - `live` (also unset or unknown): the window moves on a live peek of
//!   its own, its real backdrop overlay following it, as before.
//! - `a`: copied as the monitor's backdrop is for a square one, cut into
//!   rows whose ends follow the corners' arcs (see `corner_bands`).
//!
//! A backdrop with `parallax` != 1 moves live whatever this says.
//!
//! Read once per process.

use std::sync::OnceLock;

/// How a backdrop with rounded corners is drawn while its window moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RoundedBackdrop {
  #[default]
  Live,
  Bands,
}

impl RoundedBackdrop {
  /// Whether a pinned copy can draw a rounded backdrop.
  pub(crate) fn is_copied(self) -> bool {
    self != Self::Live
  }

  /// Parses a `GLAZEWM_ROUNDED_BACKDROP` value, and whether it was one
  /// of the known ones.
  fn parse(value: &str) -> (Self, bool) {
    match value.trim().to_ascii_lowercase().as_str() {
      "a" => (Self::Bands, true),
      "live" | "" => (Self::Live, true),
      _ => (Self::Live, false),
    }
  }
}

/// How `GLAZEWM_ROUNDED_BACKDROP` says to draw a rounded backdrop, read
/// on first call.
pub(crate) fn current() -> RoundedBackdrop {
  static ROUNDED: OnceLock<RoundedBackdrop> = OnceLock::new();

  *ROUNDED.get_or_init(|| {
    let Ok(value) = std::env::var("GLAZEWM_ROUNDED_BACKDROP") else {
      return RoundedBackdrop::Live;
    };

    let (rounded, is_known) = RoundedBackdrop::parse(&value);
    if !is_known {
      tracing::warn!(
        "Unknown GLAZEWM_ROUNDED_BACKDROP value: {value}. Using `live`."
      );
    } else if rounded.is_copied() {
      tracing::warn!(
        "GLAZEWM_ROUNDED_BACKDROP changes how moving windows with \
         rounded backdrops are drawn: {rounded:?}."
      );
    }
    rounded
  })
}

#[cfg(test)]
mod tests {
  use super::RoundedBackdrop;

  #[test]
  fn values_are_read_in_any_case() {
    assert_eq!(
      RoundedBackdrop::parse("a"),
      (RoundedBackdrop::Bands, true)
    );
    assert_eq!(
      RoundedBackdrop::parse(" A "),
      (RoundedBackdrop::Bands, true)
    );
    assert_eq!(
      RoundedBackdrop::parse("Live"),
      (RoundedBackdrop::Live, true)
    );
    assert_eq!(RoundedBackdrop::parse(""), (RoundedBackdrop::Live, true));
  }

  #[test]
  fn unknown_values_stay_live() {
    assert_eq!(
      RoundedBackdrop::parse("bands"),
      (RoundedBackdrop::Live, false)
    );
    assert_eq!(
      RoundedBackdrop::parse("a,b"),
      (RoundedBackdrop::Live, false)
    );
    assert_eq!(RoundedBackdrop::default(), RoundedBackdrop::Live);
  }

  #[test]
  fn only_live_draws_no_copy() {
    assert!(!RoundedBackdrop::Live.is_copied());
    assert!(RoundedBackdrop::Bands.is_copied());
  }
}
