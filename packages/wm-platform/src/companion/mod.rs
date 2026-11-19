pub(crate) mod cover;

use std::{
  sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, PoisonError,
  },
  time::{Duration, Instant},
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    UI::WindowsAndMessaging::{
      EnumWindows, GetPropW, GetWindow, GetWindowRect, IsWindow,
      IsWindowVisible, SetWindowPos, GW_HWNDNEXT, SWP_ASYNCWINDOWPOS,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE,
    },
  },
};

use crate::{platform_impl::NativeWindow, window_class, Rect};

/// Window property another app sets on its own overlay window to declare
/// it the companion of the window it draws over (the value is that
/// window's handle), e.g. shades' recolored copy of a window.
///
/// The contract: the companion covers exactly the window's DWM extended
/// frame bounds from above it in the z-order, and while the window is
/// cloaked the companion stays shown but cloaked, so DWM keeps rendering
/// it for thumbnails. Animations then show the companion in the window's
/// place, and hand the window back once the companion covers it again.
const COMPANION_PROPERTY: PCWSTR = w!("shades.CompanionOf");

/// How long a scan for companions is reused. A workspace switch creates a
/// surrogate per window in one tick, which then costs one `EnumWindows`
/// instead of one each; a companion appearing within this window is missed
/// until a stand-in looks again (see
/// `NativeSurrogate::rechecks_companion`).
const SNAPSHOT_TTL: Duration = Duration::from_millis(50);

/// Every `(window, companion)` pair from a scan, when it ran, and the
/// [`SHOWN_GENERATION`] it saw.
struct Snapshot {
  taken: Instant,
  generation: u64,
  pairs: Vec<(isize, isize)>,
}

static SNAPSHOT: Mutex<Option<Snapshot>> = Mutex::new(None);

/// Bumped each time a window carrying [`COMPANION_PROPERTY`] is shown (see
/// [`note_shown`]): a snapshot taken before then may miss a companion.
/// One hidden or destroyed since is caught by each lookup's own check.
static SHOWN_GENERATION: AtomicU64 = AtomicU64::new(0);

/// How a lookup may reuse the last scan for companions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reuse {
  /// Only within [`SNAPSHOT_TTL`], e.g. a stand-in's periodic recheck,
  /// which also catches a companion whose property was set after it was
  /// shown.
  Fresh,
  /// For as long as no companion was shown since, e.g. a ready peek's
  /// motion start, the scan kept by [`Companion::refresh_at_rest`].
  Kept,
}

/// Whether a scan taken `age` ago, seeing `generation`, is reused by a
/// lookup allowing `reuse`, `current` being [`SHOWN_GENERATION`] now.
fn reuses_snapshot(
  reuse: Reuse,
  age: Duration,
  generation: u64,
  current: u64,
) -> bool {
  generation == current && (reuse == Reuse::Kept || age < SNAPSHOT_TTL)
}

/// Notes that `hwnd`, another app's window, was just shown: a companion
/// among them makes the next lookup scan again. Called from the window
/// event hook, so it reads one property and nothing else.
pub(crate) fn note_shown(hwnd: HWND) {
  if companion_of(hwnd).is_some() {
    SHOWN_GENERATION.fetch_add(1, Ordering::AcqRel);
  }
}

/// Most windows [`Companion::covers_window`] walks down past, bounding a
/// walk of a z-order that changes under it.
const MAX_Z_ORDER_WALK: usize = 4096;

/// Upper bound on how long a stand-in keeps covering a window after it is
/// uncloaked, for a companion owner that never puts the companion back on
/// screen.
///
/// Generous: a loaded machine can take well over 200 ms to put it back,
/// and the stand-in waiting shows the (cloaked) companion live, so the
/// wait itself is never seen. One that can't end well stops at once (see
/// [`CompanionState`]).
const HANDOFF_MAX_WAIT: Duration = Duration::from_secs(1);

/// How long a companion has to cover its window, call after call, before
/// a stand-in hands the window back to it (see [`HandoffWait::awaits`]).
const HANDOFF_SETTLE: Duration = Duration::from_millis(50);

/// Thumbnail opacity of a stand-in showing a companion, whatever the
/// window's own alpha: the companion covers the window opaquely at rest,
/// so drawn at the window's alpha it would look lighter for the whole
/// animation, and the hand-back would show the real window (as it really
/// is) through the stand-in.
pub(crate) const COMPANION_OPACITY: u8 = u8::MAX;

/// What a [`HandoffWait`] finds on a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompanionState {
  /// The companion or its window was destroyed.
  Gone,
  /// The window is hidden or cloaked, e.g. moved off screen again.
  WindowHidden,
  /// The window is shown, and the companion `covers` it or not.
  WindowShown { covers: bool },
}

impl CompanionState {
  /// The state as seen on screen: a companion with another window still
  /// over it doesn't cover its window yet.
  fn obstructed_if(self, is_obstructed: bool) -> Self {
    match self {
      Self::WindowShown { covers } => Self::WindowShown {
        covers: covers && !is_obstructed,
      },
      state => state,
    }
  }
}

/// Waits out a companion's return over its window when a stand-in showing
/// it hands the window back.
///
/// Once the real window is uncloaked, the companion's owner only learns of
/// it from a window event and shows the companion some time later, as it
/// restacks the companion over a window raised past it. Until then the
/// window shows as it really is (e.g. without shades' colors), so the
/// stand-in should keep covering it.
#[derive(Debug, Default)]
pub(crate) struct HandoffWait {
  /// When the companion was first found not covering its window.
  started_at: Option<Instant>,
  /// Since when the companion has covered its window, call after call.
  covered_since: Option<Instant>,
  /// Whether the window was hidden on the last call.
  hidden_last: bool,
}

impl HandoffWait {
  /// Starts waiting anew, for a stand-in beginning to hand its window
  /// back: a wait left from an earlier hand-back would have long timed
  /// out, and handed the window back at once.
  pub(crate) fn restart(&mut self) {
    *self = Self::default();
  }

  /// Whether to keep covering the window: `companion` is shown by the
  /// stand-in and hasn't been over the window for [`HANDOFF_SETTLE`] on
  /// every call since, and the wait hasn't timed out or become pointless.
  ///
  /// Called once a tick. Handed the window as soon as it covered it, the
  /// companion was often moved again by its owner right after (shades
  /// restacks its overlay around a window's stand-ins, several times as a
  /// workspace switch ends), and anything raising the window then showed
  /// it uncolored. Covering it longer costs nothing: the stand-in draws
  /// the companion itself.
  pub(crate) fn awaits(&mut self, companion: Option<Companion>) -> bool {
    self.awaits_unobstructed(companion, &[])
  }

  /// [`awaits`](Self::awaits), also while any of `obstructions`, e.g. the
  /// companion of a window hidden as this one was shown, is still on
  /// screen above `companion`.
  pub(crate) fn awaits_unobstructed(
    &mut self,
    companion: Option<Companion>,
    obstructions: &[Companion],
  ) -> bool {
    companion.is_some_and(|companion| {
      let is_obstructed = obstructions.iter().any(|obstruction| {
        obstruction.is_on_screen()
          && is_above(obstruction.hwnd(), companion.hwnd())
      });
      let state = companion.state().obstructed_if(is_obstructed);
      self.awaits_state(state, Instant::now())
    })
  }

  /// [`awaits`](Self::awaits), for a companion found in `state` at `now`.
  ///
  /// A hidden window only ends the wait when found hidden on two calls in
  /// a row: the first call can come right after its uncloak.
  fn awaits_state(&mut self, state: CompanionState, now: Instant) -> bool {
    let is_hidden = state == CompanionState::WindowHidden;
    let hidden_before =
      std::mem::replace(&mut self.hidden_last, is_hidden);

    let covers = match state {
      CompanionState::Gone => return false,
      CompanionState::WindowHidden if hidden_before => return false,
      CompanionState::WindowHidden => false,
      CompanionState::WindowShown { covers } => covers,
    };

    if covers {
      let since = *self.covered_since.get_or_insert(now);
      return now.duration_since(since) < HANDOFF_SETTLE;
    }
    self.covered_since = None;

    let started = *self.started_at.get_or_insert(now);
    now.duration_since(started) < HANDOFF_MAX_WAIT
  }
}

/// A companion window (see [`COMPANION_PROPERTY`]) of a managed window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Companion {
  hwnd: isize,

  /// The window the companion draws over.
  source: isize,

  /// Where the companion's origin lies in the window's own coordinates,
  /// i.e. the window's invisible left and top resize borders.
  origin: (i32, i32),
}

impl Companion {
  /// The companion of `window`, if one is shown (possibly cloaked).
  pub(crate) fn find(window: HWND) -> Option<Self> {
    Self::lookup(window, Reuse::Fresh)
  }

  /// [`find`](Self::find), reusing the last scan for as long as no
  /// companion was shown since, however old: for a ready peek's motion
  /// start, which then does no `EnumWindows`. The scan is kept current
  /// by [`refresh_at_rest`](Self::refresh_at_rest).
  ///
  /// A companion whose property is set only after it was shown is missed
  /// until the next rest or a stand-in's own recheck (see
  /// `NativeSurrogate::rechecks_companion`).
  pub(crate) fn find_kept(window: HWND) -> Option<Self> {
    Self::lookup(window, Reuse::Kept)
  }

  /// Scans for companions anew, unless the last scan is younger than
  /// [`SNAPSHOT_TTL`] and none was shown since, for the lookups of motions
  /// to come (see [`find_kept`](Self::find_kept)). Run at rest.
  pub(crate) fn refresh_at_rest() {
    let mut snapshot =
      SNAPSHOT.lock().unwrap_or_else(PoisonError::into_inner);
    current_snapshot(&mut snapshot, Reuse::Fresh);
  }

  fn lookup(window: HWND, reuse: Reuse) -> Option<Self> {
    let mut snapshot =
      SNAPSHOT.lock().unwrap_or_else(PoisonError::into_inner);

    let hwnd = current_snapshot(&mut snapshot, reuse)
      .pairs
      .iter()
      .find(|(source, _)| *source == window.0)
      .map(|(_, companion)| HWND(*companion))
      .filter(|companion| companion_of(*companion) == Some(window))
      .filter(|companion| {
        // SAFETY: A stale handle just returns false.
        unsafe { IsWindowVisible(*companion) }.as_bool()
      })?;
    drop(snapshot);

    Some(Self {
      hwnd: hwnd.0,
      source: window.0,
      origin: frame_origin(window)?,
    })
  }

  pub(crate) fn hwnd(self) -> HWND {
    HWND(self.hwnd)
  }

  /// The window the companion draws over.
  pub(crate) fn source(self) -> HWND {
    HWND(self.source)
  }

  /// Where the companion and its window are, for a [`HandoffWait`].
  fn state(self) -> CompanionState {
    let source = HWND(self.source);

    // SAFETY: `IsWindow` and `IsWindowVisible` take any handle; a stale
    // one just returns false.
    let (exists, is_shown) = unsafe {
      (
        IsWindow(self.hwnd()).as_bool() && IsWindow(source).as_bool(),
        IsWindowVisible(source).as_bool(),
      )
    };

    if !exists {
      CompanionState::Gone
    } else if !is_shown
      || NativeWindow::new(self.source).is_cloaked().unwrap_or(false)
    {
      CompanionState::WindowHidden
    } else {
      CompanionState::WindowShown {
        covers: self.coverage() == CompanionCoverage::Covers,
      }
    }
  }

  /// Where the companion is relative to its window.
  ///
  /// Only [`CompanionCoverage::Covers`] has it over the window as its
  /// owner draws it at rest: one shown above the window, but still where
  /// the window was before a move or resize, leaves part of the window
  /// uncolored, or colors where the window no longer is.
  fn coverage(self) -> CompanionCoverage {
    let source = HWND(self.source);

    if !self.is_on_screen() {
      return CompanionCoverage::Cloaked;
    }

    if !is_above(self.hwnd(), source) {
      return CompanionCoverage::Below;
    }

    let frame =
      window_class::bounds_and_frame(source).map(|(_, frame)| frame);
    let is_over_frame = self
      .bounds()
      .zip(frame)
      .is_some_and(|(bounds, frame)| is_over_frame(&bounds, frame));

    if is_over_frame {
      CompanionCoverage::Covers
    } else {
      CompanionCoverage::Misplaced
    }
  }

  /// Whether the companion is shown and not cloaked.
  pub(crate) fn is_on_screen(self) -> bool {
    // SAFETY: A stale handle just returns false.
    let is_visible = unsafe { IsWindowVisible(self.hwnd()) }.as_bool();
    is_visible
      && !NativeWindow::new(self.hwnd).is_cloaked().unwrap_or(false)
  }

  /// Where the companion is on screen.
  pub(crate) fn bounds(self) -> Option<Rect> {
    let mut bounds = RECT::default();

    // SAFETY: `bounds` outlives the call; a stale handle just fails.
    unsafe { GetWindowRect(self.hwnd(), &raw mut bounds) }.ok()?;
    Some(Rect::from_ltrb(
      bounds.left,
      bounds.top,
      bounds.right,
      bounds.bottom,
    ))
  }

  /// Maps a thumbnail's `(rcSource, rcDestination)`, with the source in
  /// the window's coordinates, onto the companion.
  pub(crate) fn map_rects(self, src: RECT, dst: RECT) -> (RECT, RECT) {
    let mut bounds = RECT::default();

    // SAFETY: `bounds` outlives the call; a stale handle just fails.
    if unsafe { GetWindowRect(self.hwnd(), &raw mut bounds) }.is_err() {
      return (src, dst);
    }

    map_rects(
      src,
      dst,
      self.origin,
      (bounds.right - bounds.left, bounds.bottom - bounds.top),
    )
  }
}

/// How far a companion may sit off its window's frame, per edge, and
/// still count as covering it (rounding at fractional DPI scales).
const COVERAGE_TOLERANCE: i32 = 1;

/// Where a window's companion is relative to the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionCoverage {
  /// No companion is shown for the window, cloaked or not.
  NoCompanion,
  /// On screen, above the window and over its frame.
  Covers,
  /// Shown but cloaked.
  Cloaked,
  /// On screen, but below the window.
  Below,
  /// On screen above the window, but not over its frame (e.g. lagging a
  /// move).
  Misplaced,
}

impl CompanionCoverage {
  /// Whether the companion is drawn on screen.
  #[must_use]
  pub fn is_on_screen(self) -> bool {
    matches!(self, Self::Covers | Self::Below | Self::Misplaced)
  }
}

/// Where `window`'s companion is relative to it (see
/// [`CompanionCoverage`]).
///
/// Shares the companion scan's snapshot, so it costs at most one
/// `EnumWindows` per [`SNAPSHOT_TTL`] across calls.
pub(crate) fn coverage(window: HWND) -> CompanionCoverage {
  Companion::find(window)
    .map_or(CompanionCoverage::NoCompanion, Companion::coverage)
}

/// Puts `window` back directly below its companion if it was raised past
/// it, in the same band, and the companion is on screen. Returns whether
/// it did.
///
/// Only `window` is moved, never the companion: its owner owns its place.
/// Posted, so a hung app doesn't block the WM.
pub(crate) fn keep_below(window: HWND) -> crate::Result<bool> {
  let Some(companion) = Companion::find(window) else {
    return Ok(false);
  };
  let companion_hwnd = companion.hwnd();

  // Inserted after a window of the other band, `window` would change band.
  if !companion.is_on_screen()
    || !is_above(window, companion_hwnd)
    || window_class::is_topmost(window)
      != window_class::is_topmost(companion_hwnd)
  {
    return Ok(false);
  }

  // SAFETY: Both handles are valid or fail the call; the flags leave
  // position, size, visibility and activation untouched.
  unsafe {
    SetWindowPos(
      window,
      companion_hwnd,
      0,
      0,
      0,
      0,
      SWP_NOACTIVATE
        | SWP_NOMOVE
        | SWP_NOSIZE
        | SWP_NOSENDCHANGING
        | SWP_ASYNCWINDOWPOS,
    )
  }?;

  Ok(true)
}

/// Whether a companion at `bounds` lies over a window whose DWM extended
/// frame bounds are `frame`, as the companion contract has it, within
/// [`COVERAGE_TOLERANCE`].
fn is_over_frame(bounds: &Rect, frame: RECT) -> bool {
  (bounds.left - frame.left).abs() <= COVERAGE_TOLERANCE
    && (bounds.top - frame.top).abs() <= COVERAGE_TOLERANCE
    && (bounds.right - frame.right).abs() <= COVERAGE_TOLERANCE
    && (bounds.bottom - frame.bottom).abs() <= COVERAGE_TOLERANCE
}

/// Shifts `src` by `-origin` and clips it to `size`, clipping `dst`
/// proportionally, so the thumbnail never samples past the companion:
/// DWM renders an oversampled source as a transparent hole.
fn map_rects(
  src: RECT,
  dst: RECT,
  origin: (i32, i32),
  size: (i32, i32),
) -> (RECT, RECT) {
  let mut src = RECT {
    left: src.left - origin.0,
    top: src.top - origin.1,
    right: src.right - origin.0,
    bottom: src.bottom - origin.1,
  };
  let mut dst = dst;

  let src_width = src.right - src.left;
  let src_height = src.bottom - src.top;

  if src_width <= 0 || src_height <= 0 {
    return (src, dst);
  }

  let scale_x = f64::from(dst.right - dst.left) / f64::from(src_width);
  let scale_y = f64::from(dst.bottom - dst.top) / f64::from(src_height);

  #[allow(clippy::cast_possible_truncation)]
  let scaled =
    |pixels: i32, scale: f64| (f64::from(pixels) * scale).round() as i32;

  let clip_left = (-src.left).max(0);
  let clip_top = (-src.top).max(0);
  let clip_right = (src.right - size.0).max(0);
  let clip_bottom = (src.bottom - size.1).max(0);

  src.left += clip_left;
  src.top += clip_top;
  src.right -= clip_right;
  src.bottom -= clip_bottom;

  dst.left += scaled(clip_left, scale_x);
  dst.top += scaled(clip_top, scale_y);
  dst.right -= scaled(clip_right, scale_x);
  dst.bottom -= scaled(clip_bottom, scale_y);

  (src, dst)
}

/// The snapshot in `slot`, scanned anew first unless `reuse` allows the
/// one there (see [`reuses_snapshot`]).
fn current_snapshot(
  slot: &mut Option<Snapshot>,
  reuse: Reuse,
) -> &Snapshot {
  let now = Instant::now();
  let current = SHOWN_GENERATION.load(Ordering::Acquire);
  let reuses = slot.as_ref().is_some_and(|snapshot| {
    reuses_snapshot(
      reuse,
      now.duration_since(snapshot.taken),
      snapshot.generation,
      current,
    )
  });

  if !reuses {
    // Read before the scan, so a companion shown during it makes the next
    // lookup scan again.
    *slot = Some(Snapshot {
      taken: now,
      generation: current,
      pairs: scan(),
    });
  }

  slot.get_or_insert_with(|| Snapshot {
    taken: now,
    generation: current,
    pairs: Vec::new(),
  })
}

/// Whether `upper` is anywhere above `lower` in the z-order.
pub(crate) fn is_above(upper: HWND, lower: HWND) -> bool {
  let mut below = upper;

  for _ in 0..MAX_Z_ORDER_WALK {
    // SAFETY: A stale handle just makes `GetWindow` return `HWND(0)`.
    below = unsafe { GetWindow(below, GW_HWNDNEXT) };
    if below.0 == 0 {
      return false;
    }
    if below == lower {
      return true;
    }
  }

  false
}

/// Where `window`'s visible frame starts within its own rect: past its
/// invisible left and top resize borders.
fn frame_origin(window: HWND) -> Option<(i32, i32)> {
  let (bounds, frame) = window_class::bounds_and_frame(window)?;
  Some((frame.left - bounds.left, frame.top - bounds.top))
}

/// The window `hwnd` is a companion of, per its property.
fn companion_of(hwnd: HWND) -> Option<HWND> {
  // SAFETY: A stale handle or a missing property just returns 0.
  let value = unsafe { GetPropW(hwnd, COMPANION_PROPERTY) };
  (value.0 != 0).then_some(HWND(value.0))
}

/// Every shown top-level window carrying the companion property, as
/// `(window, companion)`.
fn scan() -> Vec<(isize, isize)> {
  unsafe extern "system" fn visit(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: `lparam` is the `Vec` passed to `EnumWindows` below, which
    // outlives the enumeration and is only accessed from this callback.
    let pairs = unsafe { &mut *(lparam.0 as *mut Vec<(isize, isize)>) };

    // SAFETY: `hwnd` comes from the enumeration.
    if unsafe { IsWindowVisible(hwnd) }.as_bool() {
      if let Some(source) = companion_of(hwnd) {
        pairs.push((source.0, hwnd.0));
      }
    }

    TRUE
  }

  let mut pairs: Vec<(isize, isize)> = Vec::new();

  // SAFETY: `visit` only dereferences `lparam` as the `Vec` above, which
  // lives until `EnumWindows` returns.
  let _ = unsafe {
    EnumWindows(
      Some(visit),
      LPARAM(std::ptr::from_mut(&mut pairs) as isize),
    )
  };

  pairs
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use windows::Win32::Foundation::RECT;

  use super::{
    is_over_frame, map_rects, reuses_snapshot, CompanionState,
    HandoffWait, Reuse, HANDOFF_MAX_WAIT, HANDOFF_SETTLE, SNAPSHOT_TTL,
  };
  use crate::Rect;

  #[test]
  fn companion_over_frame_allows_a_pixel_of_rounding_per_edge() {
    let frame = RECT {
      left: 100,
      top: 50,
      right: 900,
      bottom: 650,
    };

    assert!(is_over_frame(&Rect::from_ltrb(100, 50, 900, 650), frame));
    assert!(is_over_frame(&Rect::from_ltrb(101, 49, 899, 651), frame));
    // Where the window was before moving 20px right.
    assert!(!is_over_frame(&Rect::from_ltrb(80, 50, 880, 650), frame));
    // The size it had before growing.
    assert!(!is_over_frame(&Rect::from_ltrb(100, 50, 700, 650), frame));
  }

  #[test]
  fn a_kept_scan_is_reused_however_old_until_a_companion_shows() {
    let old = SNAPSHOT_TTL * 100;
    assert!(reuses_snapshot(Reuse::Kept, old, 3, 3));
    assert!(!reuses_snapshot(Reuse::Kept, old, 3, 4));
    assert!(!reuses_snapshot(Reuse::Kept, Duration::ZERO, 3, 4));
  }

  #[test]
  fn a_fresh_lookup_scans_again_past_its_ttl() {
    assert!(reuses_snapshot(Reuse::Fresh, Duration::ZERO, 3, 3));
    assert!(!reuses_snapshot(Reuse::Fresh, SNAPSHOT_TTL, 3, 3));
    assert!(!reuses_snapshot(Reuse::Fresh, Duration::ZERO, 2, 3));
  }

  const COVERING: CompanionState =
    CompanionState::WindowShown { covers: true };
  const NOT_COVERING: CompanionState =
    CompanionState::WindowShown { covers: false };

  /// Feeds `states` to a fresh wait a tick apart, returning what each
  /// call answered.
  fn answers(states: &[CompanionState]) -> Vec<bool> {
    let mut wait = HandoffWait::default();
    let start = Instant::now();
    (0u32..)
      .zip(states)
      .map(|(tick, state)| {
        let now = start + Duration::from_millis(16) * tick;
        wait.awaits_state(*state, now)
      })
      .collect()
  }

  /// Calls, 16ms apart, a companion covering from the first takes to be
  /// handed the window: the first at or past `HANDOFF_SETTLE`.
  fn calls_to_settle() -> usize {
    let settle = u64::try_from(HANDOFF_SETTLE.as_millis()).unwrap_or(0);
    usize::try_from(settle.div_ceil(16)).unwrap_or(0)
  }

  /// `states` followed by a companion covering long enough to settle.
  fn then_settled(mut states: Vec<CompanionState>) -> Vec<CompanionState> {
    states.extend(vec![COVERING; calls_to_settle() + 1]);
    states
  }

  /// `true` for every call but the last of `states`, which hands back.
  fn waits_until_last(states: &[CompanionState]) -> Vec<bool> {
    let mut answers = vec![true; states.len() - 1];
    answers.push(false);
    answers
  }

  #[test]
  fn hands_back_once_the_companion_covered_long_enough() {
    let states = then_settled(vec![NOT_COVERING]);
    assert_eq!(answers(&states), waits_until_last(&states));
  }

  #[test]
  fn waits_for_a_companion_covering_from_the_start() {
    let states = then_settled(Vec::new());
    assert_eq!(answers(&states), waits_until_last(&states));
  }

  #[test]
  fn waits_again_for_a_companion_that_stops_covering() {
    let states = then_settled(vec![COVERING, NOT_COVERING]);
    assert_eq!(answers(&states), waits_until_last(&states));
  }

  #[test]
  fn stops_at_once_for_a_companion_gone() {
    assert_eq!(answers(&[CompanionState::Gone]), [false]);
    assert_eq!(
      answers(&[NOT_COVERING, CompanionState::Gone]),
      [true, false]
    );
  }

  #[test]
  fn stops_for_a_window_hidden_again() {
    assert_eq!(
      answers(&[
        NOT_COVERING,
        CompanionState::WindowHidden,
        CompanionState::WindowHidden,
      ]),
      [true, true, false]
    );
  }

  #[test]
  fn waits_through_a_window_read_hidden_once() {
    let states =
      then_settled(vec![CompanionState::WindowHidden, NOT_COVERING]);
    assert_eq!(answers(&states), waits_until_last(&states));
  }

  #[test]
  fn restarted_wait_waits_again_after_timing_out() {
    let mut wait = HandoffWait::default();
    let start = Instant::now();
    assert!(wait.awaits_state(NOT_COVERING, start));
    assert!(!wait.awaits_state(NOT_COVERING, start + HANDOFF_MAX_WAIT));

    let later = start + HANDOFF_MAX_WAIT * 3;
    assert!(!wait.awaits_state(NOT_COVERING, later));
    wait.restart();
    assert!(wait.awaits_state(NOT_COVERING, later));
    assert!(wait.awaits_state(
      NOT_COVERING,
      later + HANDOFF_MAX_WAIT.saturating_sub(Duration::from_millis(1))
    ));
  }

  #[test]
  fn restarted_wait_waits_again_for_a_companion_covering() {
    let mut wait = HandoffWait::default();
    let start = Instant::now();
    assert!(wait.awaits_state(COVERING, start));
    assert!(!wait.awaits_state(COVERING, start + HANDOFF_SETTLE));

    let later = start + HANDOFF_SETTLE * 2;
    wait.restart();
    assert!(wait.awaits_state(COVERING, later));
    assert!(!wait.awaits_state(COVERING, later + HANDOFF_SETTLE));
  }

  #[test]
  fn waits_while_obstructed() {
    let obstructed = COVERING.obstructed_if(true);
    let states = then_settled(vec![obstructed, obstructed]);
    assert_eq!(answers(&states), waits_until_last(&states));
  }

  #[test]
  fn obstruction_leaves_hidden_and_gone_alone() {
    for state in [CompanionState::Gone, CompanionState::WindowHidden] {
      assert_eq!(state.obstructed_if(true), state);
    }
    assert_eq!(NOT_COVERING.obstructed_if(false), NOT_COVERING);
  }

  #[test]
  fn stops_waiting_for_an_obstruction_after_the_max_wait() {
    let mut wait = HandoffWait::default();
    let start = Instant::now();
    let obstructed = COVERING.obstructed_if(true);
    assert!(wait.awaits_state(obstructed, start));
    assert!(!wait.awaits_state(obstructed, start + HANDOFF_MAX_WAIT));
  }

  #[test]
  fn stops_waiting_after_the_max_wait() {
    let mut wait = HandoffWait::default();
    let start = Instant::now();
    assert!(wait.awaits_state(NOT_COVERING, start));
    assert!(wait.awaits_state(
      NOT_COVERING,
      start + HANDOFF_MAX_WAIT.saturating_sub(Duration::from_millis(1))
    ));
    assert!(!wait.awaits_state(NOT_COVERING, start + HANDOFF_MAX_WAIT));
  }

  fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
      left,
      top,
      right,
      bottom,
    }
  }

  #[test]
  fn shifts_source_past_invisible_border() {
    let (src, dst) = map_rects(
      rect(7, 0, 807, 600),
      rect(0, 0, 800, 600),
      (7, 0),
      (800, 600),
    );
    assert_eq!(src, rect(0, 0, 800, 600));
    assert_eq!(dst, rect(0, 0, 800, 600));
  }

  #[test]
  fn clips_source_outside_companion() {
    // Samples the invisible border left of the frame, and 100px past a
    // companion that hasn't grown yet.
    let (src, dst) = map_rects(
      rect(0, 0, 907, 600),
      rect(0, 0, 907, 600),
      (7, 0),
      (800, 600),
    );
    assert_eq!(src, rect(0, 0, 800, 600));
    assert_eq!(dst, rect(7, 0, 807, 600));
  }

  #[test]
  fn clips_destination_proportionally_when_scaled() {
    // Half-size zoom: 200 source pixels past the companion are 100 on
    // screen.
    let (src, dst) = map_rects(
      rect(0, 0, 1000, 600),
      rect(0, 0, 500, 300),
      (0, 0),
      (800, 600),
    );
    assert_eq!(src, rect(0, 0, 800, 600));
    assert_eq!(dst, rect(0, 0, 400, 300));
  }
}
