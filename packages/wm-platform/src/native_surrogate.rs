use std::{
  rc::Rc,
  sync::OnceLock,
  time::{Duration, Instant},
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
      DestroyWindow, GetClassNameW, SetWindowPos, SET_WINDOW_POS_FLAGS,
      SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOSENDCHANGING,
      SWP_NOSIZE, SWP_NOZORDER, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    },
  },
};

use crate::{
  companion::{Companion, HandoffWait, COMPANION_OPACITY},
  dwm_thumbnail::{clip_rects, Look, Thumbnail},
  peek_surface::SurfaceWindow,
  platform_impl,
  window_class::{self, DeferredPositions},
  CornerStyle, PeekSurface, Rect,
};

pub(crate) const CLASS_NAME: PCWSTR = w!("GlazeWM_Surrogate");

/// How often a stand-in showing no companion looks for one shown since
/// (see [`NativeSurrogate::rechecks_companion`]). Time-based, as ticks run
/// at up to the fastest monitor's refresh rate, and each look past
/// `Companion::find`'s cache is an `EnumWindows` scan.
const COMPANION_RECHECK_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn ensure_class_registered() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    CLASS_NAME,
    window_class::default_wnd_proc,
  );
}

/// Whether `hwnd` is a surrogate window, of any animation kind.
pub(crate) fn is_surrogate(hwnd: HWND) -> bool {
  let mut buf = [0u16; 32];
  // SAFETY: `buf` outlives the call; a stale `hwnd` just returns 0.
  let len = unsafe { GetClassNameW(hwnd, &mut buf) };
  let name = &buf[..usize::try_from(len).unwrap_or(0)];

  // SAFETY: `CLASS_NAME` is a static, null-terminated literal.
  unsafe { CLASS_NAME.as_wide() == name }
}

/// Rounds the surrogate's corners like the window it stands in for, and
/// drops the 1px border Windows 11 draws around it.
///
/// DWM doesn't round popups by default, unlike app windows, so
/// `CornerStyle::Default` rounds here. With its border left on, the border
/// overlay's ring shows a second line inside it for every transition, as
/// it would on the real window (see `set_native_border_hidden`). Both are
/// no-ops before Windows 11.
pub(crate) fn style_like_window(hwnd: HWND, corner_style: CornerStyle) {
  let corner_style = match corner_style {
    CornerStyle::Default => CornerStyle::Rounded,
    style => style,
  };

  let window = platform_impl::NativeWindow::new(hwnd.0);
  let _ = window.set_corner_style(&corner_style);
  let _ = window.set_native_border_hidden(true);
}

/// `(rcSource, rcDestination)` sampling `width` × `height` of a window's
/// visible content onto the surrogate's top-left.
///
/// `rcSource` starts at `border_inset` so invisible-border pixels are
/// excluded; those pixels render as black in DWM thumbnails. When
/// `border_inset` is all-zero this is the full physical window.
pub(crate) fn content_rects(
  width: i32,
  height: i32,
  border_inset: RECT,
) -> (RECT, RECT) {
  let src = RECT {
    left: border_inset.left,
    top: border_inset.top,
    right: border_inset.left + width,
    bottom: border_inset.top + height,
  };
  let dst = RECT {
    left: 0,
    top: 0,
    right: width,
    bottom: height,
  };
  (src, dst)
}

/// `(rcSource, rcDestination)` drawing the part `shown` of a window placed
/// at `frame`, which is `size` at rest, into a surrogate whose top-left is
/// at `origin`. Both rects are in screen coordinates. `None` when nothing
/// of `frame` is shown.
///
/// `rcSource` starts past `border_inset`, like [`content_rects`]. Whatever
/// of `shown` lies outside the surrogate is cut by its bounds.
pub(crate) fn placed_rects(
  frame: &Rect,
  shown: &Rect,
  size: (i32, i32),
  border_inset: RECT,
  origin: (i32, i32),
) -> Option<(RECT, RECT)> {
  let local = |rect: &Rect| RECT {
    left: rect.left - origin.0,
    top: rect.top - origin.1,
    right: rect.right - origin.0,
    bottom: rect.bottom - origin.1,
  };
  let source = RECT {
    left: border_inset.left,
    top: border_inset.top,
    right: border_inset.left + size.0,
    bottom: border_inset.top + size.1,
  };

  clip_rects(source, local(frame), &local(shown))
}

/// Maps thumbnail rects given in the source window's coordinates onto
/// what the thumbnail actually samples: the window, or its companion.
fn sampled_rects(
  companion: Option<Companion>,
  src: RECT,
  dst: RECT,
) -> (RECT, RECT) {
  companion.map_or((src, dst), |companion| companion.map_rects(src, dst))
}

/// Registers a DWM thumbnail of `source_hwnd` -- or of its `companion`,
/// when it has one -- onto `dest_hwnd`, showing `look` (see
/// [`thumbnail_look`]), kept hidden if `held` (see `Thumbnail::set_held`).
fn register_thumbnail(
  dest_hwnd: HWND,
  source_hwnd: HWND,
  companion: Option<Companion>,
  look: Look,
  held: bool,
) -> Option<Thumbnail> {
  Thumbnail::register_held(
    dest_hwnd,
    companion.map_or(source_hwnd, Companion::hwnd),
    look,
    held,
  )
}

/// Whether a stand-in's thumbnail, if `is_registered`, sampling its
/// window's companion `kept` when it was made, can draw the window now
/// that its companion is `found`: only if it samples the same one.
fn keeps_window_thumbnail<T: PartialEq + Copy>(
  is_registered: bool,
  kept: Option<T>,
  found: Option<T>,
) -> bool {
  is_registered && kept == found
}

/// A shown thumbnail at `opacity` of `src` (in the source window's
/// coordinates) sampled into `dst`.
fn thumbnail_look(
  companion: Option<Companion>,
  src: RECT,
  dst: RECT,
  opacity: u8,
) -> Look {
  let (src, dst) = sampled_rects(companion, src, dst);
  Look {
    source: Some(src),
    dest: dst,
    opacity,
    visible: true,
  }
}

/// Pixels of the thumbnail's trailing edges the gap fill reaches under.
///
/// A window's visible frame ends in the border DWM draws on it, which the
/// WM hides: in a thumbnail those rows are see-through. Met exactly at the
/// thumbnail's edge, the fill left them showing the backdrop, as a line
/// between content and fill for as long as the fill showed.
const FILL_UNDERLAP_PX: i32 = 2;

/// Where the gap fill starts, from the surrogate's top-left, under a
/// thumbnail covering `covered` (see [`FILL_UNDERLAP_PX`]).
pub(crate) fn fill_start(covered: (i32, i32)) -> (i32, i32) {
  (
    (covered.0 - FILL_UNDERLAP_PX).max(0),
    (covered.1 - FILL_UNDERLAP_PX).max(0),
  )
}

/// Collects surrogate repositions for one animation frame and applies them
/// atomically in a single `DeferWindowPos` transaction.
///
/// Sequential per-surrogate `SetWindowPos` calls can straddle a DWM
/// composition boundary, letting adjacent windows' edges desync for one
/// frame during a multi-window relayout. Batching all repositions into one
/// transaction guarantees every surrogate lands in the same composition
/// frame.
///
/// When the transaction cannot be created or fails mid-way, [`commit`]
/// falls back to individual `SetWindowPos` calls so no reposition is lost.
///
/// [`commit`]: SurrogateBatch::commit
#[derive(Default)]
pub struct SurrogateBatch {
  /// Queued repositions as `(surrogate hwnd, logical target rect)` pairs.
  entries: Vec<(isize, Rect)>,
}

impl SurrogateBatch {
  /// Creates an empty batch.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Whether any repositions have been queued.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Queues a reposition; applied on [`commit`].
  ///
  /// Not surrogate-specific -- `hwnd` can be any top-level window this
  /// process owns (used by [`NativeBackdropOverlay::defer_rect`] to fold
  /// the backdrop overlay's reposition into the same transaction as the
  /// surrogates/real windows already batched this tick).
  ///
  /// [`commit`]: SurrogateBatch::commit
  /// [`NativeBackdropOverlay::defer_rect`]:
  ///     crate::NativeBackdropOverlay::defer_rect
  pub(crate) fn push(&mut self, hwnd: isize, rect: Rect) {
    self.entries.push((hwnd, rect));
  }

  /// `SetWindowPos` flags shared by both commit paths.
  ///
  /// Notably *without* `SWP_NOSENDCHANGING` — see [`deferred_flags`] and
  /// [`individual_flags`].
  ///
  /// [`deferred_flags`]: SurrogateBatch::deferred_flags
  /// [`individual_flags`]: SurrogateBatch::individual_flags
  fn base_flags() -> SET_WINDOW_POS_FLAGS {
    SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOZORDER
  }

  /// Flags passed to `DeferWindowPos`.
  ///
  /// `SWP_NOSENDCHANGING` is deliberately excluded: `DeferWindowPos`
  /// rejects it with `ERROR_INVALID_PARAMETER` even though
  /// `SetWindowPos` accepts it (and the documentation lists it for
  /// both). Including it made every batch fail on its very first entry,
  /// so the whole transaction was dead code that silently fell through
  /// to [`commit_individually`] — and, far worse, abandoned the
  /// transaction's `HDWP`, leaking one USER object per animation frame.
  ///
  /// The batched windows are all overlay windows this process owns, whose
  /// window procedure is `DefWindowProcW`, so the `WM_WINDOWPOSCHANGING`
  /// message this flag would have suppressed costs a same-thread dispatch
  /// into the default handler and nothing else.
  ///
  /// [`commit_individually`]: SurrogateBatch::commit_individually
  fn deferred_flags() -> SET_WINDOW_POS_FLAGS {
    Self::base_flags()
  }

  /// Flags passed to the per-window `SetWindowPos` fallback.
  ///
  /// `SetWindowPos` does honor `SWP_NOSENDCHANGING`, so the fallback keeps
  /// skipping the `WM_WINDOWPOSCHANGING` round-trip.
  fn individual_flags() -> SET_WINDOW_POS_FLAGS {
    Self::base_flags() | SWP_NOSENDCHANGING
  }

  /// Applies all queued repositions in one `DeferWindowPos` transaction.
  ///
  /// Falls back to individual `SetWindowPos` calls when the transaction
  /// fails (e.g. a surrogate window was destroyed mid-frame).
  pub fn commit(self) {
    if self.entries.is_empty() {
      return;
    }

    let _scope = crate::perf::scope(crate::perf::Stage::BatchCommit);

    // A failed transaction still applies the entries before the failure;
    // the fallback re-applies all of them, so positions end up the same.
    let deferred = DeferredPositions::begin(self.entries.len())
      .is_some_and(|mut transaction| {
        self.entries.iter().all(|(hwnd, rect)| {
          transaction.defer(
            HWND(*hwnd),
            HWND(0),
            rect,
            Self::deferred_flags(),
          )
        }) && transaction.end()
      });

    if !deferred {
      Self::commit_individually(&self.entries, Self::individual_flags());
    }
  }

  /// Fallback: applies each queued reposition with its own `SetWindowPos`
  /// call.
  fn commit_individually(
    entries: &[(isize, Rect)],
    flags: SET_WINDOW_POS_FLAGS,
  ) {
    for (hwnd, rect) in entries {
      // SAFETY: See `commit` — failures for stale handles are ignored.
      unsafe {
        let _ = SetWindowPos(
          HWND(*hwnd),
          HWND(0),
          rect.x(),
          rect.y(),
          rect.width(),
          rect.height(),
          flags,
        );
      }
    }
  }
}

/// Converts a physical `Rect` to logical by subtracting the invisible
/// border inset on each side.
pub(crate) fn to_logical(rect: &Rect, inset: &RECT) -> Rect {
  Rect::from_ltrb(
    rect.left + inset.left,
    rect.top + inset.top,
    rect.right - inset.right,
    rect.bottom - inset.bottom,
  )
}

/// A click-through window showing a DWM thumbnail of a real window, which
/// the WM cloaks meanwhile: what `WindowPeek`, `WorkspaceSurrogate` and
/// `CompanionCover` draw with.
///
/// Dropping it unregisters the thumbnail and destroys its window.
///
/// A hosted one (see [`create_hosted`]) draws on a [`PeekSurface`]'s
/// window instead of one of its own, which it can't move, show or hide:
/// each method that would says what it does instead.
///
/// [`create_hosted`]: NativeSurrogate::create_hosted
pub struct NativeSurrogate {
  /// The window it draws on.
  host: Host,
  /// DWM thumbnail, or `None` if registration failed.
  thumbnail: Option<Thumbnail>,
  /// The source window's companion, which the thumbnail samples instead
  /// of the window when present (e.g. a recolored copy of it).
  companion: Option<Companion>,
  /// See [`NativeSurrogate::awaits_companion`].
  companion_wait: HandoffWait,
  /// When the surrogate last looked for a companion. See
  /// [`COMPANION_RECHECK_INTERVAL`].
  companion_checked_at: Instant,
  /// Cached visibility state; guards against redundant `ShowWindow`
  /// calls.
  is_visible: bool,
  /// Last opacity applied to the DWM thumbnail via `DWM_TNP_OPACITY`;
  /// used to skip redundant calls when opacity has not changed between
  /// frames.
  last_opacity: u8,
  /// Last rect passed to `SetWindowPos` via `reposition`; used to skip
  /// redundant calls when the position and size have not changed.
  last_rect: Option<Rect>,
  /// Whether it is kept, hidden, as a ready peek of its surface (see
  /// [`retire`](NativeSurrogate::retire)).
  is_ready: bool,
}

/// The window a [`NativeSurrogate`] draws on.
enum Host {
  /// Its own, destroyed with it.
  Own(isize),
  /// A [`PeekSurface`]'s, shared with other surrogates.
  Shared(Rc<SurfaceWindow>),
}

impl NativeSurrogate {
  /// Creates a surrogate overlay and positions it above `source_hwnd`.
  ///
  /// The overlay is shown without activating it. A DWM thumbnail of
  /// `source_hwnd` is registered and the surrogate window starts at
  /// `source_rect`. Everything the thumbnail does not cover is
  /// transparent.
  ///
  /// `thumbnail_rect` controls the DWM thumbnail registration size. It
  /// must not exceed the source window's actual dimensions — an
  /// oversampled `rcSource` renders as a transparent hole that exposes
  /// whatever is behind the surrogate. Pinned surrogates pass the
  /// window's screen rect (the surrogate itself spans the whole
  /// viewport).
  ///
  /// The surrogate window is created hidden; the caller calls
  /// [`set_visible`] to reveal it.
  ///
  /// `border_inset` shrinks the surrogate from the physical rect to the
  /// logical (visible-content) rect, preventing the surrogate from
  /// occupying the configured window gap. Pass `RECT::default()` to keep
  /// the full physical size (workspace-switch surrogates).
  ///
  /// `corner_style` controls the DWM corner-rounding applied to the
  /// surrogate. Because `WS_POPUP | WS_EX_TOOLWINDOW` windows are not
  /// rounded by DWM by default, pass the real window's configured style
  /// so the surrogate matches visually. `CornerStyle::Default` maps to
  /// rounded (the Windows 11 app-window default).
  ///
  /// `insert_after` is the `hWndInsertAfter` argument for the initial
  /// `SetWindowPos` Z-order placement. Pass `HWND(0)` (`HWND_TOP`) to
  /// place the surrogate at the top of the non-topmost Z-order so it
  /// appears above any simultaneously active surrogates (e.g. close
  /// overlays). Pass `source_hwnd` to place immediately below the source
  /// window.
  ///
  /// Returns an error if window creation fails.
  ///
  /// [`set_visible`]: NativeSurrogate::set_visible
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    thumbnail_rect: &Rect,
    opacity: u8,
    border_inset: RECT,
    corner_style: &CornerStyle,
    insert_after: HWND,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    // Surrogate window is sized to the logical source rect (does not
    // occupy the window gap). Thumbnail dimensions come from
    // `thumbnail_rect` and may differ (a pinned one spans its clip).
    let logical_src = to_logical(source_rect, &border_inset);
    let logical_thumb = to_logical(thumbnail_rect, &border_inset);

    let window_scope =
      crate::perf::scope(crate::perf::Stage::SurrogateWindow);
    // `WS_EX_NOREDIRECTIONBITMAP` leaves the window with no GDI surface at
    // all, so everything outside the thumbnail is transparent.
    let hwnd = window_class::create_popup(
      CLASS_NAME,
      WS_EX_NOACTIVATE
        | WS_EX_TOOLWINDOW
        | WS_EX_TRANSPARENT
        | WS_EX_NOREDIRECTIONBITMAP,
      &logical_src,
      "surrogate",
    )?;

    style_like_window(hwnd, *corner_style);
    drop(window_scope);

    // Register the DWM thumbnail at `thumbnail_rect` dimensions.
    //
    // `opacity` is baked into the initial registration so the first
    // rendered frame shows the correct transparency without a separate
    // `DwmUpdateThumbnailProperties` call.
    //
    // Failure is non-fatal: the surrogate is then left without a
    // thumbnail (see `has_thumbnail`).
    let thumb_scope =
      crate::perf::scope(crate::perf::Stage::SurrogateThumb);
    let companion = Companion::find(source_hwnd);
    let (src, dst) = content_rects(
      logical_thumb.width(),
      logical_thumb.height(),
      border_inset,
    );
    let look = thumbnail_look(companion, src, dst, opacity);
    let thumbnail =
      register_thumbnail(hwnd, source_hwnd, companion, look, false);
    drop(thumb_scope);

    // Constructed before the final `SetWindowPos` call (rather than after)
    // so that if it fails, `?`'s early return drops `this` — running
    // `Drop`'s thumbnail-unregister and window-destroy cleanup —
    // instead of leaking the overlay window and DWM thumbnail.
    let this = Self {
      host: Host::Own(hwnd.0),
      thumbnail,
      companion,
      companion_wait: HandoffWait::default(),
      companion_checked_at: Instant::now(),
      is_visible: false,
      last_opacity: opacity,
      last_rect: None,
      is_ready: false,
    };

    // Set the initial Z-order position.
    // `insert_after` is caller-controlled: peeks, workspace surrogates and
    // companion covers pass `source_hwnd` to sit just below it.
    //
    // SAFETY: Both handles are valid.
    unsafe {
      SetWindowPos(
        hwnd,
        insert_after,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      )
    }?;

    Ok(this)
  }

  /// Creates a hidden surrogate drawing `source_hwnd` on `surface`, as
  /// [`create`](Self::create) does one spanning the surface's clip with
  /// no border inset and square corners, just below
  /// `source_hwnd`.
  ///
  /// The surface goes by `source_hwnd`, which `is_shown` or is cloaked, as
  /// `SurfaceWindow::admit` decides. A failed registration leaves it
  /// without a thumbnail, as for `create`.
  #[must_use]
  pub fn create_hosted(
    surface: &PeekSurface,
    source_hwnd: HWND,
    thumbnail_rect: &Rect,
    opacity: u8,
    is_shown: bool,
  ) -> Self {
    let this =
      Self::registered_on(surface, source_hwnd, thumbnail_rect, opacity);
    if let Host::Shared(window) = &this.host {
      window.admit(source_hwnd, is_shown);
    }
    this
  }

  /// Creates a surrogate drawing `source_hwnd` on `surface` as
  /// [`create_hosted`](Self::create_hosted) does, kept from the start as a
  /// ready peek of it (see [`retire`](Self::retire)): the surface is not
  /// moved, and takes it as a tenant only at [`reclaim`](Self::reclaim).
  #[must_use]
  pub fn create_ready(
    surface: &PeekSurface,
    source_hwnd: HWND,
    thumbnail_rect: &Rect,
  ) -> Self {
    let mut this =
      Self::registered_on(surface, source_hwnd, thumbnail_rect, u8::MAX);
    if let Host::Shared(window) = &this.host {
      window.add_ready();
    }
    this.is_ready = true;
    this
  }

  /// A hidden surrogate drawing `source_hwnd` on `surface`, its thumbnail
  /// registered held, neither a tenant nor a ready peek of it yet.
  fn registered_on(
    surface: &PeekSurface,
    source_hwnd: HWND,
    thumbnail_rect: &Rect,
    opacity: u8,
  ) -> Self {
    let window = Rc::clone(surface.window());

    // No window to create: `sg_window` has nothing to time here.
    let thumb_scope =
      crate::perf::scope(crate::perf::Stage::SurrogateThumb);
    let companion = Companion::find(source_hwnd);
    let (src, dst) = content_rects(
      thumbnail_rect.width(),
      thumbnail_rect.height(),
      RECT::default(),
    );
    let look = thumbnail_look(companion, src, dst, opacity);
    let thumbnail = register_thumbnail(
      window.hwnd(),
      source_hwnd,
      companion,
      look,
      true,
    );
    drop(thumb_scope);

    Self {
      host: Host::Shared(window),
      thumbnail,
      companion,
      companion_wait: HandoffWait::default(),
      companion_checked_at: Instant::now(),
      is_visible: false,
      last_opacity: opacity,
      last_rect: None,
      is_ready: false,
    }
  }

  /// Keeps a hosted surrogate, hidden, as a ready peek of its surface,
  /// which then takes new tenants as if it were gone (see
  /// `SurfaceWindow::retire`), until [`reclaim`](Self::reclaim). None for
  /// one of its own or one shown.
  pub(crate) fn retire(&mut self) {
    let Host::Shared(window) = &self.host else {
      return;
    };
    if self.is_visible || self.is_ready {
      return;
    }

    window.retire();
    self.is_ready = true;
  }

  /// Whether it is kept as a ready peek of its surface (see
  /// [`retire`](Self::retire)).
  pub(crate) fn is_ready(&self) -> bool {
    self.is_ready
  }

  /// Has a retired surrogate draw `source_hwnd` again as
  /// [`create_hosted`](Self::create_hosted) would anew with
  /// `thumbnail_rect` and `opacity`, back on its surface, which goes just
  /// below `source_hwnd` as `SurfaceWindow::admit` decides: the same
  /// fields, and the same thumbnail unless that would sample another
  /// companion now (see [`keeps_window_thumbnail`]). Returns whether the
  /// thumbnail was registered anew, i.e. stacks above everything on the
  /// surface.
  ///
  /// Looks up the companion in the scan kept at rest (see
  /// `Companion::find_kept`), so a motion start does no `EnumWindows`.
  pub(crate) fn reclaim(
    &mut self,
    source_hwnd: HWND,
    thumbnail_rect: &Rect,
    opacity: u8,
  ) -> bool {
    let Host::Shared(window) = &self.host else {
      return false;
    };
    let window = Rc::clone(window);

    let thumb_scope =
      crate::perf::scope(crate::perf::Stage::SurrogateThumb);
    let companion = Companion::find_kept(source_hwnd);
    let (src, dst) = content_rects(
      thumbnail_rect.width(),
      thumbnail_rect.height(),
      RECT::default(),
    );
    let look = thumbnail_look(companion, src, dst, opacity);
    let keeps = keeps_window_thumbnail(
      self.thumbnail.is_some(),
      self.companion,
      companion,
    );
    match self.thumbnail.as_mut().filter(|_| keeps) {
      Some(thumbnail) => {
        let _ = thumbnail.set(look);
      }
      None => {
        self.thumbnail = register_thumbnail(
          window.hwnd(),
          source_hwnd,
          companion,
          look,
          true,
        );
      }
    }
    drop(thumb_scope);

    self.companion = companion;
    self.companion_wait = HandoffWait::default();
    self.companion_checked_at = Instant::now();
    self.last_opacity = opacity;
    self.last_rect = None;

    if self.is_ready {
      self.is_ready = false;
      window.rejoin(source_hwnd);
    }
    !keeps
  }

  /// Its thumbnail's registration order (see `Thumbnail::order`).
  pub(crate) fn thumbnail_order(&self) -> Option<u64> {
    self.thumbnail.as_ref().map(Thumbnail::order)
  }

  /// A second thumbnail of what it draws of `source_hwnd`, registered held
  /// over everything on its window, for
  /// [`put_thumbnail`](Self::put_thumbnail). `None` without a thumbnail or
  /// if registration failed.
  pub(crate) fn thumbnail_held_anew(
    &self,
    source_hwnd: HWND,
  ) -> Option<Thumbnail> {
    let source = self.companion.map_or(source_hwnd, Companion::hwnd);
    self.thumbnail.as_ref()?.held_anew(self.hwnd(), source)
  }

  /// Draws `thumbnail`, from
  /// [`thumbnail_held_anew`](Self::thumbnail_held_anew), in place of its
  /// own: let go first, so no composition shows neither.
  pub(crate) fn put_thumbnail(&mut self, mut thumbnail: Thumbnail) {
    thumbnail.set_held(self.is_held());
    self.thumbnail = Some(thumbnail);
  }

  /// Whether it draws on a [`PeekSurface`] (see
  /// [`create_hosted`](Self::create_hosted)).
  #[must_use]
  pub fn is_hosted(&self) -> bool {
    matches!(self.host, Host::Shared(_))
  }

  /// Keeps a hosted one's surface from going down from now on, as its
  /// source window is cloaked under it (see `SurfaceWindow::hold_place`).
  /// None for one of its own.
  pub(crate) fn hold_place(&self) {
    if let Host::Shared(window) = &self.host {
      window.hold_place();
    }
  }

  /// Puts a hosted one's surface just above `window` unless it is above it
  /// already (see `SurfaceWindow::keep_above`). None for one of its own.
  pub(crate) fn lift_surface_above(&self, window: HWND) {
    if let Host::Shared(surface) = &self.host {
      surface.keep_above(window);
    }
  }

  /// Whether its thumbnails are to be kept hidden: hosted and hidden.
  fn is_held(&self) -> bool {
    self.is_hosted() && !self.is_visible
  }

  /// Returns the raw handle of the window it draws on: its own, or its
  /// surface's.
  #[must_use]
  pub fn hwnd(&self) -> HWND {
    match &self.host {
      Host::Own(hwnd) => HWND(*hwnd),
      Host::Shared(window) => window.hwnd(),
    }
  }

  /// Returns `true` when a DWM thumbnail was successfully registered.
  ///
  /// `DwmRegisterThumbnail` fails for elevated or UWP source windows.
  /// Callers use this to decide whether to freeze the real window behind
  /// the surrogate or fall back to direct repositioning.
  #[must_use]
  pub fn has_thumbnail(&self) -> bool {
    self.thumbnail.is_some()
  }

  /// Whether to keep covering the uncloaked source window: the thumbnail
  /// samples its companion, which is not back over it yet (see
  /// [`HandoffWait::awaits`]).
  pub(crate) fn awaits_companion(&mut self) -> bool {
    self.companion_wait.awaits(self.companion)
  }

  /// [`awaits_companion`](Self::awaits_companion), also while any of
  /// `obstructions` shows over the companion (see
  /// [`HandoffWait::awaits_unobstructed`]).
  pub(crate) fn awaits_unobstructed_companion(
    &mut self,
    obstructions: &[Companion],
  ) -> bool {
    self
      .companion_wait
      .awaits_unobstructed(self.companion, obstructions)
  }

  /// Starts a fresh companion wait (see [`HandoffWait::restart`]), as the
  /// stand-in begins handing its window back.
  pub(crate) fn begin_hand_back(&mut self) {
    self.companion_wait.restart();
  }

  /// Whether the thumbnail samples the source window's companion rather
  /// than the window itself.
  #[must_use]
  pub(crate) fn shows_companion(&self) -> bool {
    self.companion.is_some()
  }

  /// Opacity to draw the thumbnail at for a window whose own alpha is
  /// `window_alpha`: [`COMPANION_OPACITY`] while it samples the window's
  /// companion.
  #[must_use]
  pub(crate) fn stand_in_opacity(&self, window_alpha: u8) -> u8 {
    if self.shows_companion() {
      COMPANION_OPACITY
    } else {
      window_alpha
    }
  }

  /// Whether it is time to look for a companion shown since the
  /// surrogate last did (see [`adopt_companion`](Self::adopt_companion)):
  /// at most every [`COMPANION_RECHECK_INTERVAL`], and never once it has
  /// one.
  #[must_use]
  pub(crate) fn rechecks_companion(&self) -> bool {
    self.companion.is_none()
      && self.companion_checked_at.elapsed() >= COMPANION_RECHECK_INTERVAL
  }

  /// Switches the thumbnail from `source_hwnd` to its companion, drawn at
  /// `opacity`, if it has none yet and one is shown now. Returns whether
  /// it did. A hidden thumbnail, e.g. one handed back, is left as it is.
  ///
  /// The companion's thumbnail is registered before the window's goes, so
  /// it stacks above it and no composition shows neither, but also above
  /// any thumbnail registered since the window's. It keeps what the
  /// window's showed, mapped onto the companion.
  pub(crate) fn adopt_companion(
    &mut self,
    source_hwnd: HWND,
    opacity: u8,
  ) -> bool {
    if self.companion.is_some() {
      return false;
    }
    let Some(shown) = self
      .thumbnail
      .as_ref()
      .map(|thumbnail| *thumbnail.look())
      .filter(|look| look.visible)
    else {
      return false;
    };

    self.companion_checked_at = Instant::now();
    let Some(companion) = Companion::find(source_hwnd) else {
      return false;
    };

    // Every look this type registers has a source rect, which DWM then
    // keeps (see `Look::source`).
    let Some(src) = shown.source else {
      return false;
    };
    let look = thumbnail_look(Some(companion), src, shown.dest, opacity);
    let Some(thumbnail) = register_thumbnail(
      self.hwnd(),
      source_hwnd,
      Some(companion),
      look,
      self.is_held(),
    ) else {
      return false;
    };

    self.thumbnail = Some(thumbnail);
    self.companion = Some(companion);
    self.companion_wait = HandoffWait::default();
    self.last_opacity = opacity;

    // Until now its visible thumbnail drew the window as it really is
    // (e.g. without shades' colors), for up to a recheck interval.
    tracing::info!(
      target: crate::FLICKER_LOG_TARGET,
      "stand-in of {:#x} drew it uncolored until it found its companion \
       mid-animation (rechecked every {}ms)",
      source_hwnd.0,
      COMPANION_RECHECK_INTERVAL.as_millis(),
    );

    true
  }

  /// Restacks the surrogate directly above `window`, so it covers the
  /// window once uncloaked instead of sitting behind it.
  ///
  /// None for a hosted one: its surface is raised over every tile it
  /// draws, and moved down to one it would take others under them.
  pub fn raise_above(&self, window: HWND) -> crate::Result<()> {
    let Host::Own(hwnd) = self.host else {
      return Ok(());
    };
    window_class::place_above(HWND(hwnd), window)
  }

  /// Restacks the surrogate at the top of the non-topmost windows, as
  /// [`create`](Self::create) places it given `HWND(0)` (`HWND_TOP`). A
  /// hosted one restacks its surface, with every surrogate on it.
  pub fn raise_to_top(&self) -> crate::Result<()> {
    let hwnd = match &self.host {
      Host::Own(hwnd) => HWND(*hwnd),
      Host::Shared(window) => return window.raise_to_top(),
    };

    // SAFETY: `hwnd` is valid until `drop`; the flags leave position,
    // size and activation untouched.
    unsafe {
      SetWindowPos(
        hwnd,
        HWND(0),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      )
    }?;
    Ok(())
  }

  /// Restacks the surrogate at the top of the always-on-top band if
  /// `topmost`, else at the top of the other windows. A hosted one
  /// restacks its surface, with every surrogate on it.
  pub fn raise_to_top_of_band(&self, topmost: bool) -> crate::Result<()> {
    match &self.host {
      Host::Own(hwnd) => {
        window_class::raise_to_top_of_band(HWND(*hwnd), topmost)
      }
      Host::Shared(window) => window.raise_to_top_of_band(topmost),
    }
  }

  /// Puts the surrogate directly above `anchor`, unless it already is
  /// anywhere above it, so it keeps covering a window raised past it.
  /// None for a hosted one, as for [`raise_above`](Self::raise_above).
  pub(crate) fn keep_above(&self, anchor: HWND) {
    if self.is_hosted() || crate::companion::is_above(self.hwnd(), anchor)
    {
      return;
    }
    if let Err(err) = self.raise_above(anchor) {
      tracing::warn!(
        "Failed to raise a surrogate above its window: {err}"
      );
    }
  }

  /// Shows or hides the surrogate overlay window without activating it.
  /// A hosted one shows or hides its thumbnail, kept hidden meanwhile
  /// whatever it is set to show, and its surface follows its surrogates
  /// (see `SurfaceWindow::note_drawing`).
  ///
  /// No-op when the window is already in the requested state.
  pub fn set_visible(&mut self, visible: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{
      ShowWindow, SW_HIDE, SW_SHOWNOACTIVATE,
    };

    if self.is_visible == visible {
      return;
    }
    self.is_visible = visible;
    match &self.host {
      // SAFETY: `hwnd` is valid until `drop`.
      Host::Own(hwnd) => unsafe {
        ShowWindow(
          HWND(*hwnd),
          if visible { SW_SHOWNOACTIVATE } else { SW_HIDE },
        );
      },
      Host::Shared(window) => {
        if let Some(thumbnail) = &mut self.thumbnail {
          thumbnail.set_held(!visible);
        }
        window.note_drawing(visible);
      }
    }
  }

  /// Repositions the surrogate overlay to `rect` without touching the DWM
  /// thumbnail properties.
  ///
  /// Use this when the thumbnail is managed separately (e.g.
  /// workspace-switch slide animations that update
  /// `rcSource`/`rcDestination` independently). No-op when `rect`
  /// matches the last applied position, and for a hosted one, whose
  /// surface spans its clip.
  pub fn reposition(&mut self, rect: &Rect) -> crate::Result<()> {
    let Host::Own(hwnd) = self.host else {
      return Ok(());
    };
    if self.last_rect.as_ref() == Some(rect) {
      return Ok(());
    }
    // SAFETY: `hwnd` is the overlay created in `create` and remains valid
    // until `drop`. `SWP_NOZORDER` makes `hWndInsertAfter` irrelevant.
    unsafe {
      SetWindowPos(
        HWND(hwnd),
        HWND(0),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE
          | SWP_NOCOPYBITS
          | SWP_NOSENDCHANGING
          | SWP_NOZORDER,
      )
    }?;
    self.last_rect = Some(rect.clone());
    Ok(())
  }

  /// Hides the DWM thumbnail, leaving the surrogate window shown.
  pub fn hide_thumbnail(&mut self) {
    if let Some(thumbnail) = &mut self.thumbnail {
      thumbnail.hide();
    }
  }

  /// Sets the DWM thumbnail opacity via `DWM_TNP_OPACITY`.
  ///
  /// `opacity` ranges from 0 (fully transparent) to 255 (fully opaque).
  /// No-op when `opacity` matches the last applied value.
  pub fn set_window_opacity(&mut self, opacity: u8) {
    if opacity == self.last_opacity {
      return;
    }
    self.last_opacity = opacity;
    if let Some(thumbnail) = &mut self.thumbnail {
      let _ = thumbnail.set_opacity(opacity);
    }
  }

  /// Shows `rc_src` (source-window-local) at `rc_dst` (surrogate-local),
  /// keeping the opacity set by [`set_window_opacity`]. No-op when no
  /// thumbnail was registered.
  ///
  /// [`set_window_opacity`]: NativeSurrogate::set_window_opacity
  pub fn set_thumbnail_rects(&mut self, rc_src: RECT, rc_dst: RECT) {
    if self.thumbnail.is_none() {
      return;
    }
    let (rc_src, rc_dst) = sampled_rects(self.companion, rc_src, rc_dst);
    if let Some(thumbnail) = &mut self.thumbnail {
      let _ = thumbnail.show(rc_src, rc_dst);
    }
  }

  /// Registers the thumbnail of `source_hwnd` anew as it is, so it stacks
  /// above thumbnails registered on the surrogate since (see
  /// `Thumbnail::register`).
  pub(crate) fn raise_thumbnail(&mut self, source_hwnd: HWND) {
    let Some(look) =
      self.thumbnail.as_ref().map(|thumbnail| *thumbnail.look())
    else {
      return;
    };

    // Unregistered first, as two would draw the window twice.
    self.thumbnail = None;
    self.thumbnail = register_thumbnail(
      self.hwnd(),
      source_hwnd,
      self.companion,
      look,
      self.is_held(),
    );
  }
}

impl Drop for NativeSurrogate {
  fn drop(&mut self) {
    // The thumbnail must go before the window it is drawn on.
    self.thumbnail.take();

    match &self.host {
      // SAFETY: `hwnd` is the window created by this type.
      Host::Own(hwnd) => unsafe {
        let _ = DestroyWindow(HWND(*hwnd));
      },
      // The surface's window stays, held by `host` until it is dropped
      // after this.
      Host::Shared(window) => {
        if self.is_visible {
          window.note_drawing(false);
        }
        if self.is_ready {
          window.drop_ready();
        } else {
          window.release();
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use windows::Win32::System::Threading::{
    GetCurrentProcess, GetGuiResources, GR_USEROBJECTS,
  };

  use super::{
    ensure_class_registered, fill_start, is_surrogate,
    keeps_window_thumbnail, placed_rects, window_class, DeferredPositions,
    DestroyWindow, Rect, SurrogateBatch, CLASS_NAME, HWND, RECT,
    SWP_NOSENDCHANGING, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT,
  };

  /// Batch commits performed by the leak regression test.
  ///
  /// Comfortably larger than the number of USER objects any concurrently
  /// running test could plausibly create, so the regressed behaviour (one
  /// leaked object per commit) is unambiguous against the tolerance below.
  const LEAK_TEST_COMMITS: usize = 500;

  /// Slack allowed on the USER-object delta in the leak regression test.
  ///
  /// Tests share one process and `GetGuiResources` counts the whole
  /// process, so an unrelated test creating a window or event loop
  /// concurrently must not fail this. The regressed behaviour leaked
  /// [`LEAK_TEST_COMMITS`] objects, an order of magnitude above this.
  const LEAK_TEST_TOLERANCE: i64 = 32;

  fn ltrb(rect: RECT) -> (i32, i32, i32, i32) {
    (rect.left, rect.top, rect.right, rect.bottom)
  }

  /// A kept thumbnail draws its window again only sampling what it would
  /// sample made anew: the window, or the same companion.
  #[test]
  fn a_kept_thumbnail_is_kept_only_for_the_same_companion() {
    assert!(keeps_window_thumbnail::<u8>(true, None, None));
    assert!(keeps_window_thumbnail(true, Some(1), Some(1)));
    assert!(!keeps_window_thumbnail(true, None, Some(1)));
    assert!(!keeps_window_thumbnail(true, Some(1), None));
    assert!(!keeps_window_thumbnail(true, Some(1), Some(2)));
    assert!(!keeps_window_thumbnail::<u8>(false, None, None));
  }

  /// A window cut at its surrogate's far edge keeps the same thumbnail
  /// however far past it it slides, so only the surrogate's move changes
  /// per frame.
  #[test]
  fn placed_rects_leave_a_far_edge_cut_to_the_surrogate() {
    let inset = RECT {
      left: 7,
      top: 0,
      right: 7,
      bottom: 7,
    };

    // A 400 px window sliding off a monitor that ends at x = 1000.
    for left in [700, 750, 900] {
      let frame = Rect::from_xy(left, 100, 400, 300);
      let (src, dst) =
        placed_rects(&frame, &frame, (400, 300), inset, (left, 100))
          .expect("shown");

      assert_eq!(ltrb(src), (7, 0, 407, 300));
      assert_eq!(ltrb(dst), (0, 0, 400, 300));
    }
  }

  /// A window cut at its surrogate's origin is drawn offset past it,
  /// whole.
  #[test]
  fn placed_rects_offset_a_window_cut_at_the_surrogate_origin() {
    let frame = Rect::from_xy(-150, 100, 400, 300);
    let (src, dst) =
      placed_rects(&frame, &frame, (400, 300), RECT::default(), (0, 100))
        .expect("shown");

    assert_eq!(ltrb(src), (0, 0, 400, 300));
    assert_eq!(ltrb(dst), (-150, 0, 250, 300));
  }

  /// Only the shown part of a scaled window is sampled, mapped back to the
  /// window's size at rest.
  #[test]
  fn placed_rects_sample_only_the_shown_part() {
    let frame = Rect::from_xy(100, 100, 200, 150);
    let shown = Rect::from_ltrb(200, 100, 300, 250);
    let (src, dst) =
      placed_rects(&frame, &shown, (400, 300), RECT::default(), (0, 0))
        .expect("shown");

    assert_eq!(ltrb(src), (200, 0, 400, 300));
    assert_eq!(ltrb(dst), (200, 100, 300, 250));
  }

  /// The fill reaches under the thumbnail's trailing rows and columns,
  /// and never starts before the surrogate does.
  #[test]
  fn the_fill_starts_under_the_thumbnails_trailing_edges() {
    assert_eq!(fill_start((800, 600)), (798, 598));
    assert_eq!(fill_start((1, 0)), (0, 0));
  }

  /// Returns this process's current USER-object count.
  fn user_objects() -> i64 {
    // SAFETY: The pseudo-handle returned by `GetCurrentProcess` is always
    // valid and needs no closing.
    i64::from(unsafe {
      GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS)
    })
  }

  /// Creates a hidden, off-screen surrogate-class popup window for use as
  /// a reposition target.
  ///
  /// `index` offsets the window's y position so several probe windows do
  /// not overlap exactly.
  ///
  /// Returns `None` when window creation fails (e.g. no interactive window
  /// station), letting the caller skip rather than fail spuriously.
  fn create_probe_window(index: i32) -> Option<HWND> {
    ensure_class_registered();

    window_class::create_popup(
      CLASS_NAME,
      WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
      &Rect::from_xy(-8000, 100 + index * 10, 400, 300),
      "probe",
    )
    .ok()
  }

  #[test]
  fn is_surrogate_matches_only_the_surrogate_class() {
    assert!(!is_surrogate(HWND(0)));

    let Some(hwnd) = create_probe_window(3) else {
      return;
    };
    let matched = is_surrogate(hwnd);

    // SAFETY: `hwnd` was created above and not yet destroyed.
    unsafe {
      let _ = DestroyWindow(hwnd);
    }

    assert!(matched);
  }

  /// `DeferWindowPos` rejects `SWP_NOSENDCHANGING`, so it must never
  /// appear in the batch path's flags -- while the `SetWindowPos`
  /// fallback, which does honor it, keeps it.
  #[test]
  fn deferred_flags_exclude_no_send_changing() {
    assert_eq!(
      SurrogateBatch::deferred_flags().0 & SWP_NOSENDCHANGING.0,
      0,
      "`DeferWindowPos` fails with ERROR_INVALID_PARAMETER when passed \
       `SWP_NOSENDCHANGING`, which abandons the transaction's `HDWP`."
    );
    assert_ne!(
      SurrogateBatch::individual_flags().0 & SWP_NOSENDCHANGING.0,
      0,
      "The `SetWindowPos` fallback should still skip \
       `WM_WINDOWPOSCHANGING`."
    );
  }

  /// The OS itself must accept the batch path's flags -- this is what the
  /// regression was: a flag set the documentation permits but
  /// `DeferWindowPos` rejects at runtime, making every batch fall back.
  #[test]
  fn defer_window_pos_accepts_deferred_flags() {
    let Some(hwnd) = create_probe_window(0) else {
      return;
    };

    let accepted = DeferredPositions::begin(1).map(|mut transaction| {
      transaction.defer(
        hwnd,
        HWND(0),
        &Rect::from_xy(-8000, 120, 400, 300),
        SurrogateBatch::deferred_flags(),
      )
    });

    // SAFETY: `hwnd` was created above and not yet destroyed.
    unsafe {
      let _ = DestroyWindow(hwnd);
    }

    if let Some(accepted) = accepted {
      assert!(
        accepted,
        "`DeferWindowPos` rejected `SurrogateBatch::deferred_flags()`, so \
         every batch would silently fall back to individual \
         `SetWindowPos` calls."
      );
    }
  }

  /// Committing batches must not accumulate USER objects.
  ///
  /// `BeginDeferWindowPos` allocates an `HDWP`, a USER object released
  /// only by `EndDeferWindowPos`. Abandoning it after a failed
  /// `DeferWindowPos` leaked one object per animation frame, exhausting
  /// the process's 10,000-object limit within a few hundred gestures.
  #[test]
  fn commit_leaks_no_user_objects() {
    let Some(first) = create_probe_window(1) else {
      return;
    };
    let Some(second) = create_probe_window(2) else {
      // SAFETY: `first` was created above and not yet destroyed.
      unsafe {
        let _ = DestroyWindow(first);
      }
      return;
    };

    // Warm-up commit: the first reposition of a freshly created window can
    // allocate one-off state that would otherwise read as a delta.
    let mut warmup = SurrogateBatch::new();
    warmup.push(first.0, Rect::from_xy(-8000, 100, 400, 300));
    warmup.push(second.0, Rect::from_xy(-8000, 110, 400, 300));
    warmup.commit();

    let before = user_objects();

    for i in 0..LEAK_TEST_COMMITS {
      let offset = i32::from(i % 2 == 0);
      let mut batch = SurrogateBatch::new();
      batch.push(first.0, Rect::from_xy(-8000, 100 + offset, 400, 300));
      batch.push(second.0, Rect::from_xy(-8000, 110 + offset, 400, 300));
      batch.commit();
    }

    let after = user_objects();

    // SAFETY: Both handles were created above and not yet destroyed.
    unsafe {
      let _ = DestroyWindow(first);
      let _ = DestroyWindow(second);
    }

    assert!(
      after - before <= LEAK_TEST_TOLERANCE,
      "{LEAK_TEST_COMMITS} batch commits leaked {} USER objects \
       (before={before}, after={after}).",
      after - before
    );
  }
}
