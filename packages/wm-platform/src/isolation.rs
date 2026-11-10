//! `GLAZEWM_ISOLATE`: a switch for finding what draws an artifact on
//! moving windows in a live session, by leaving parts out one at a time,
//! and for trying animations not on by default yet.
//!
//! A comma-separated list of:
//!
//! - `nobackdrop`, `noring`: draw that copy of pinned surrogates and peeks
//!   (see `OverlayCopies`) at no opacity. Everything else, the handover of
//!   the real overlays included, runs as usual, so the window moves
//!   without it.
//! - `live`: copy no backdrop, so windows with one move in surrogates and
//!   peeks that move with them, as before backdrops were copied.
//! - `peeks`: draw open, close, minimize, restore and moves with pinned
//!   peeks instead of `ResizeSession`, for the kinds already moved over.
//!
//! Unset, nothing changes. Read once per process.

use std::sync::OnceLock;

/// What `GLAZEWM_ISOLATE` leaves out or turns on.
// Independent switches, any combination of which is valid.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Isolation {
  pub no_backdrop: bool,
  pub no_ring: bool,
  pub live: bool,
  pub peeks: bool,
}

impl Isolation {
  /// Parses a `GLAZEWM_ISOLATE` value, warning about unknown words.
  fn parse(value: &str) -> Self {
    let mut isolation = Self::default();

    for word in value
      .split(|c: char| c == ',' || c.is_whitespace())
      .filter(|word| !word.is_empty())
    {
      match word.to_ascii_lowercase().as_str() {
        "nobackdrop" => isolation.no_backdrop = true,
        "noring" => isolation.no_ring = true,
        "live" => isolation.live = true,
        "peeks" => isolation.peeks = true,
        _ => tracing::warn!("Unknown GLAZEWM_ISOLATE flag: {word}."),
      }
    }

    isolation
  }
}

/// What `GLAZEWM_ISOLATE` leaves out or turns on, read on first call.
pub fn current() -> Isolation {
  static ISOLATION: OnceLock<Isolation> = OnceLock::new();

  *ISOLATION.get_or_init(|| {
    std::env::var("GLAZEWM_ISOLATE")
      .map_or_default(|value| Isolation::parse(&value))
  })
}

#[cfg(test)]
mod tests {
  use super::Isolation;

  #[test]
  fn empty_leaves_nothing_out() {
    assert_eq!(Isolation::parse(""), Isolation::default());
    assert!(!Isolation::default().peeks);
  }

  #[test]
  fn flags_are_read_from_a_list() {
    assert_eq!(
      Isolation::parse("nobackdrop, Live,,bogus noring PEEKS"),
      Isolation {
        no_backdrop: true,
        no_ring: true,
        live: true,
        peeks: true,
      }
    );
    assert_eq!(
      Isolation::parse("peeks"),
      Isolation {
        peeks: true,
        ..Isolation::default()
      }
    );
  }
}
