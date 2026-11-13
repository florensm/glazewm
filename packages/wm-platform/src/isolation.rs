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
//! - `fillpeeks`: still accepted, changes nothing: `window_resize`'s
//!   `fill` style is drawn on peeks by default since `STAND_INS_PLAN.md`
//!   10e-3 (`nopeeks` draws it with `ResizeSession` again).
//! - `morepeeks`: draw on peeks too what `ResizeSession` still draws with
//!   peeks on: a fullscreen toggle, floating <-> tiling and floating moves
//!   by command, a tile moving to another monitor, and windows whose
//!   backdrop can't be copied (`STAND_INS_PLAN.md` 10f).
//! - `asynchandback`: a window its peek hands back is moved
//!   asynchronously, not in step with the peek.
//! - `keepoverlays`: as `origin/main`, a window taken over by a new
//!   `ResizeSession` keeps its overlays where they are until the session
//!   loop moves them behind the surrogate, rather than having them hidden
//!   for the take-over's flush and shown again after it.
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
  pub more_peeks: bool,
  pub async_hand_back: bool,
  pub keep_overlays: bool,
}

impl Default for Isolation {
  fn default() -> Self {
    Self {
      no_backdrop: false,
      no_ring: false,
      live: false,
      peeks: true,
      peeks_first: false,
      more_peeks: false,
      async_hand_back: false,
      keep_overlays: false,
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
        // Defaults now, still accepted.
        "peeks" | "fillpeeks" => {}
        "nopeeks" => isolation.peeks = false,
        "peeksfirst" => isolation.peeks_first = true,
        "morepeeks" => isolation.more_peeks = true,
        "asynchandback" => isolation.async_hand_back = true,
        "keepoverlays" => isolation.keep_overlays = true,
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
        more_peeks: false,
        async_hand_back: false,
        keep_overlays: false,
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
  fn more_peeks_is_off_unless_given() {
    assert!(!Isolation::default().more_peeks);
    assert!(Isolation::parse("MorePeeks").more_peeks);
  }

  #[test]
  fn async_hand_back_is_off_unless_given() {
    assert!(!Isolation::default().async_hand_back);
    assert_eq!(
      Isolation::parse("AsyncHandBack"),
      Isolation {
        async_hand_back: true,
        ..Isolation::default()
      }
    );
    assert_eq!(Isolation::parse("async_hand_back"), Isolation::default());
  }

  #[test]
  fn fill_peeks_is_still_accepted() {
    assert_eq!(Isolation::parse("fillpeeks"), Isolation::default());
  }

  #[test]
  fn keep_overlays_is_off_unless_given() {
    assert!(!Isolation::default().keep_overlays);
    assert!(Isolation::parse("nopeeks,KeepOverlays").keep_overlays);
  }
}
