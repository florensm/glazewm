//! `GLAZEWM_ISOLATE`: a switch for finding what draws an artifact on
//! moving windows in a live session, by leaving parts out one at a time,
//! and for comparing animations with the way they were drawn before.
//!
//! A comma-separated list of:
//!
//! - `nobackdrop`, `noring`: draw that copy of pinned surrogates and peeks
//!   (see `OverlayCopies`) at no opacity. Everything else, the handover of
//!   the real overlays included, runs as usual, so the window moves
//!   without it.
//! - `live`: copy no backdrop, so windows with one move in surrogates and
//!   peeks that move with them, as before backdrops were copied.
//! - `nopeeks`: draw open, close, minimize, restore and moves with
//!   `ResizeSession` again instead of pinned peeks, to compare the two or
//!   to fall back. `peeks` is still accepted and changes nothing.
//! - `peeksfirst`: on an animation tick with peeks to draw, draw their
//!   frame before the main loop's catch-up on queued events, then the rest
//!   of the tick, rather than all of it after the catch-up. With or
//!   without `nopeeks`: scrolling columns move on peeks either way.
//! - `fillpeeks`: draw resizes on peeks that `window_resize`'s `stretch`
//!   style draws stretched filled instead: the window 1:1, cut at its
//!   tile, the rest of the tile in its edge color.
//!
//! Read once per process.

use std::sync::OnceLock;

/// What `GLAZEWM_ISOLATE` leaves out or turns on.
// Independent switches, any combination of which is valid.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Isolation {
  pub no_backdrop: bool,
  pub no_ring: bool,
  pub live: bool,
  /// On unless `nopeeks` is given.
  pub peeks: bool,
  pub peeks_first: bool,
  pub fill_peeks: bool,
}

impl Default for Isolation {
  fn default() -> Self {
    Self {
      no_backdrop: false,
      no_ring: false,
      live: false,
      peeks: true,
      peeks_first: false,
      fill_peeks: false,
    }
  }
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
        "peeks" => {}
        "nopeeks" => isolation.peeks = false,
        "peeksfirst" => isolation.peeks_first = true,
        "fillpeeks" => isolation.fill_peeks = true,
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
    assert!(Isolation::default().peeks);
    assert!(!Isolation::default().peeks_first);
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
        peeks_first: false,
        fill_peeks: false,
      }
    );
    assert_eq!(Isolation::parse("peeks"), Isolation::default());
  }

  #[test]
  fn peeks_are_on_unless_turned_off() {
    assert_eq!(
      Isolation::parse("NoPeeks"),
      Isolation {
        peeks: false,
        ..Isolation::default()
      }
    );
    assert!(!Isolation::parse("peeks,nopeeks").peeks);
    assert!(Isolation::parse("noring").peeks);
  }

  #[test]
  fn peeks_first_is_its_own_word() {
    assert_eq!(
      Isolation::parse("PeeksFirst"),
      Isolation {
        peeks_first: true,
        ..Isolation::default()
      }
    );
    assert_eq!(
      Isolation::parse("peeks,peeksfirst"),
      Isolation {
        peeks: true,
        peeks_first: true,
        ..Isolation::default()
      }
    );
    assert_eq!(Isolation::parse("peeks_first"), Isolation::default());
  }

  #[test]
  fn fill_peeks_is_off_unless_given() {
    assert!(!Isolation::default().fill_peeks);
    assert_eq!(
      Isolation::parse("FillPeeks"),
      Isolation {
        fill_peeks: true,
        ..Isolation::default()
      }
    );
  }
}
