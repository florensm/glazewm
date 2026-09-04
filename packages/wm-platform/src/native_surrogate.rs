use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::{
      DwmExtendFrameIntoClientArea, DwmRegisterThumbnail, DwmSetWindowAttribute,
      DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
      DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWMWCP_ROUND,
      DWMWCP_ROUNDSMALL, DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY,
      DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE, DWM_TNP_SOURCECLIENTAREAONLY,
      DWM_TNP_VISIBLE,
    },
    UI::WindowsAndMessaging::{
      BeginDeferWindowPos, CreateWindowExW, DeferWindowPos, DestroyWindow,
      EndDeferWindowPos, SetWindowPos, SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE,
      SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER,
      SWP_SHOWWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
      WS_POPUP,
    },
  },
};

use crate::{window_class, Color, CornerStyle, Rect};
use crate::platform_impl::swca::{
  ACCENT_ENABLE_ACRYLICBLURBEHIND, ACCENT_ENABLE_GRADIENT, apply_swca_accent,
};

fn ensure_class_registered() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_Surrogate"),
    window_class::default_wnd_proc,
  );
}

/// Applies the DWM corner preference matching `corner_style` to `hwnd`.
///
/// `WS_POPUP | WS_EX_TOOLWINDOW` windows are not rounded by DWM by default —
/// unlike normal app windows, which are rounded on Windows 11. Explicitly
/// setting the corner preference on the surrogate keeps it visually consistent
/// with the real managed window it overlays.
///
/// `CornerStyle::Default` maps to `DWMWCP_ROUND` rather than `DWMWCP_DEFAULT`
/// because DWM's heuristic default for popup/tool windows is no rounding,
/// while GlazeWM-managed app windows default to rounded on Windows 11.
///
/// This is a no-op on Windows 10, where `DwmSetWindowAttribute` silently
/// returns an error for unknown attributes.
fn apply_corner_preference(hwnd: HWND, corner_style: &CornerStyle) {
  let pref = match corner_style {
    CornerStyle::Default | CornerStyle::Rounded => DWMWCP_ROUND,
    CornerStyle::Square => DWMWCP_DONOTROUND,
    CornerStyle::SmallRounded => DWMWCP_ROUNDSMALL,
  };
  // SAFETY: `hwnd` is a valid window handle. `pref` is a stack-allocated i32.
  unsafe {
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_WINDOW_CORNER_PREFERENCE,
      std::ptr::from_ref(&pref.0).cast(),
      std::mem::size_of::<i32>() as u32,
    );
  }
}

/// Applies a solid-color backdrop to `hwnd` via the undocumented
/// `SetWindowCompositionAttribute` API (Windows 10 1607+).
///
/// When `color` is `None`, no accent is applied — DWM's default transparent
/// backing store is used so the border-extension area around the DWM thumbnail
/// is genuinely see-through.
///
/// This is a no-op when the API is unavailable (pre-Windows 10 1607).
pub(crate) fn apply_backdrop(hwnd: HWND, color: Option<&Color>) {
  let Some(c) = color else {
    return;
  };

  // The undocumented `gradient_color` field uses ABGR byte order:
  // alpha in the high byte, then blue, green, red in the low bytes.
  let abgr = (u32::from(c.a) << 24)
    | (u32::from(c.b) << 16)
    | (u32::from(c.g) << 8)
    | u32::from(c.r);

  apply_swca_accent(hwnd, ACCENT_ENABLE_GRADIENT, 0, abgr);
}

/// Registers a DWM thumbnail of `source_hwnd` onto `dest_hwnd`.
///
/// `logical_width` and `logical_height` are the visible content dimensions
/// of the source window (physical size minus invisible border). `border_inset`
/// gives the per-side border widths in the source window's coordinate space.
///
/// `rcSource` is set to the visible content area of the source window
/// (offset by `border_inset`). `rcDestination` fills the surrogate at
/// `{0, 0, logical_width, logical_height}` — callers are expected to have
/// already sized the surrogate to the logical rect. When `border_inset` is
/// all-zero the behaviour is identical to passing the full physical dimensions.
///
/// Returns the opaque thumbnail handle, or `None` if registration fails
/// (e.g. same-window, invalid handle). The caller is responsible for
/// calling [`DwmUnregisterThumbnail`] when done.
fn register_thumbnail(
  dest_hwnd: HWND,
  source_hwnd: HWND,
  logical_width: i32,
  logical_height: i32,
  border_inset: RECT,
  initial_opacity: u8,
) -> Option<isize> {
  // SAFETY: Both handles are valid top-level windows.
  let thumbnail =
    unsafe { DwmRegisterThumbnail(dest_hwnd, source_hwnd).ok()? };

  // `rcSource` starts at the border inset so invisible-border pixels are
  // excluded; those pixels render as black in DWM thumbnails. `rcDestination`
  // fills the whole (logical-sized) surrogate from (0, 0).
  let src_rect = RECT {
    left: border_inset.left,
    top: border_inset.top,
    right: border_inset.left + logical_width,
    bottom: border_inset.top + logical_height,
  };
  let dst_rect = RECT {
    left: 0,
    top: 0,
    right: logical_width,
    bottom: logical_height,
  };

  let props = DWM_THUMBNAIL_PROPERTIES {
    dwFlags: DWM_TNP_RECTDESTINATION
      | DWM_TNP_RECTSOURCE
      | DWM_TNP_OPACITY
      | DWM_TNP_VISIBLE
      | DWM_TNP_SOURCECLIENTAREAONLY,
    rcDestination: dst_rect,
    rcSource: src_rect,
    opacity: initial_opacity,
    fVisible: true.into(),
    fSourceClientAreaOnly: false.into(),
    ..Default::default()
  };

  // SAFETY: `thumbnail` is a valid handle returned by
  // `DwmRegisterThumbnail`.
  if unsafe { DwmUpdateThumbnailProperties(thumbnail, &raw const props) }
    .is_err()
  {
    // SAFETY: Same handle; unregister on failure.
    unsafe {
      let _ = DwmUnregisterThumbnail(thumbnail);
    };
    return None;
  }

  Some(thumbnail)
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
/// When the transaction cannot be created or fails mid-way, [`commit`] falls
/// back to individual `SetWindowPos` calls so no reposition is lost.
///
/// [`commit`]: SurrogateBatch::commit
///
/// # Platform-specific
///
/// Only available on Windows.
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
  /// process owns (used by [`NativeBlurOverlay::defer_rect`] to fold the
  /// acrylic overlay's reposition into the same transaction as the
  /// surrogates/real windows already batched this tick).
  ///
  /// [`commit`]: SurrogateBatch::commit
  /// [`NativeBlurOverlay::defer_rect`]: crate::NativeBlurOverlay::defer_rect
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
  /// `SWP_NOSENDCHANGING` is deliberately excluded: `DeferWindowPos` rejects
  /// it with `ERROR_INVALID_PARAMETER` even though `SetWindowPos` accepts it
  /// (and the documentation lists it for both). Including it made every
  /// batch fail on its very first entry, so the whole transaction was dead
  /// code that silently fell through to [`commit_individually`] — and, far
  /// worse, abandoned the transaction's `HDWP`, leaking one USER object per
  /// animation frame.
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

    // SAFETY: All handles refer to surrogate windows owned by this process;
    // a stale handle only causes the transaction to fail, which is handled
    // by the fallback below.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let deferred = unsafe {
      let Ok(mut hdwp) = BeginDeferWindowPos(self.entries.len() as i32)
      else {
        return Self::commit_individually(
          &self.entries,
          Self::individual_flags(),
        );
      };

      let mut failed = false;
      for (hwnd, rect) in &self.entries {
        match DeferWindowPos(
          hdwp,
          HWND(*hwnd),
          HWND(0),
          rect.x(),
          rect.y(),
          rect.width(),
          rect.height(),
          Self::deferred_flags(),
        ) {
          Ok(next) => hdwp = next,
          // The transaction (including prior entries) is invalidated on
          // failure; redo everything individually below.
          Err(_) => {
            failed = true;
            break;
          }
        }
      }

      // Always end the transaction, including after a failed
      // `DeferWindowPos`. `BeginDeferWindowPos` allocates an `HDWP`, which
      // is a USER object released only by `EndDeferWindowPos`; abandoning
      // the handle — as `DeferWindowPos`'s documentation suggests — leaks
      // exactly one USER object per abandoned transaction, permanently and
      // with no recovery short of process exit. Ending a partially built
      // transaction is safe and applies whichever entries were accepted
      // before the failure; `commit_individually` re-applies all of them
      // anyway, so the final positions are identical either way.
      let ended = EndDeferWindowPos(hdwp).is_ok();
      !failed && ended
    };

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

/// Converts a physical `Rect` to logical by subtracting the invisible border
/// inset on each side.
pub(crate) fn to_logical(rect: &Rect, inset: &RECT) -> Rect {
  Rect::from_ltrb(
    rect.left + inset.left,
    rect.top + inset.top,
    rect.right - inset.right,
    rect.bottom - inset.bottom,
  )
}

/// Lightweight overlay window used during move/resize animations.
///
/// At animation start the overlay is placed over the real app window at the
/// source rect. A DWM thumbnail of the real window is rendered on top,
/// registered at the source dimensions — never larger than the window's
/// current content, since an oversampled `rcSource` renders as a transparent
/// hole. For shrinking animations the surrogate clips the thumbnail edge as
/// it shrinks — a wipe effect with no distortion. For growing animations the
/// registration is upgraded to the target dimensions (via
/// [`update_thumbnail_dims`]) once the real window has actually resized,
/// progressively revealing the new content — a curtain-reveal effect.
///
/// Wherever the animated rect extends past the registered content (growing
/// sessions before the real window's resize lands, or the grown axis of a
/// mixed resize), the exposed area is filled by a solid-color backdrop
/// (sampled from the window's trailing edge at animation start) so the rect
/// reads as one continuous surface instead of exposing the desktop behind it.
///
/// [`update_thumbnail_dims`]: NativeSurrogate::update_thumbnail_dims
///
/// GlazeWM cloaks the real window while the overlay is active.
///
/// Per-frame cost is one [`SetWindowPos`] call (plus one
/// `DwmUpdateThumbnailProperties` when the thumbnail handle is valid). No
/// GDI allocations occur.
///
/// When the animation finishes the real window is uncloaked and this
/// surrogate is dropped, which unregisters the thumbnail and destroys the
/// overlay window.
///
/// # Platform-specific
///
/// Only available on Windows. Acrylic requires Windows 10 1803+; on older
/// versions the backdrop degrades gracefully (no blur, thumbnail still
/// shown).
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,
  /// DWM thumbnail handle, or `0` if registration failed.
  thumbnail: isize,
  /// Logical (visible-content) dimensions the main thumbnail samples.
  /// Updated by [`reregister_thumbnail`] when the registration size changes.
  ///
  /// [`reregister_thumbnail`]: NativeSurrogate::reregister_thumbnail
  content_size: (i32, i32),
  /// Invisible border insets of the source window, in physical pixels.
  border_inset: RECT,
  /// Cached visibility state; guards against redundant `ShowWindow` calls.
  is_visible: bool,
  /// Last opacity applied to the DWM thumbnail via `DWM_TNP_OPACITY`; used to
  /// skip redundant calls when opacity has not changed between frames.
  last_opacity: u8,
  /// Last rect passed to `SetWindowPos` via `reposition`; used to skip
  /// redundant calls when the position and size have not changed.
  last_rect: Option<Rect>,
}

impl NativeSurrogate {
  /// Creates a surrogate overlay and positions it above `source_hwnd`.
  ///
  /// The overlay is shown without activating it. A DWM thumbnail of
  /// `source_hwnd` is registered and the surrogate window starts at
  /// `source_rect`. When `surrogate_color` is `Some`, the backdrop is a
  /// solid-color fill; when `None`, the backdrop is fully transparent so only
  /// the DWM thumbnail is visible.
  ///
  /// `thumbnail_rect` controls the DWM thumbnail registration size. It must
  /// not exceed the source window's actual dimensions — an oversampled
  /// `rcSource` renders as a transparent hole that exposes whatever is
  /// behind the surrogate. Resize sessions pass `source_rect` and upgrade
  /// the registration later via [`update_thumbnail_dims`] as the real window
  /// resizes; workspace surrogates pass the window's screen rect (the
  /// surrogate itself spans the whole viewport).
  ///
  /// When `initially_visible` is `false`, the surrogate window is created
  /// hidden; the caller must call [`set_visible`] to reveal it. Pass
  /// `true` for surrogate types that must appear immediately (e.g.
  /// resize sessions). Workspace-switch surrogates pass `false` to avoid
  /// a one-frame flash before the caller explicitly shows the window.
  ///
  /// `border_inset` shrinks the surrogate from the physical rect to the
  /// logical (visible-content) rect, preventing the surrogate from occupying
  /// the configured window gap. Pass `RECT::default()` to keep the full
  /// physical size (workspace-switch surrogates).
  ///
  /// `corner_style` controls the DWM corner-rounding applied to the surrogate.
  /// Because `WS_POPUP | WS_EX_TOOLWINDOW` windows are not rounded by DWM by
  /// default, pass the real window's configured style so the surrogate matches
  /// visually. `CornerStyle::Default` maps to rounded (the Windows 11 app-window
  /// default).
  ///
  /// `insert_after` is the `hWndInsertAfter` argument for the initial
  /// `SetWindowPos` Z-order placement. Pass `HWND(0)` (`HWND_TOP`) to place
  /// the surrogate at the top of the non-topmost Z-order so it appears above
  /// any simultaneously active surrogates (e.g. close overlays). Pass
  /// `source_hwnd` to place immediately below the source window.
  ///
  /// Returns an error if window creation fails.
  ///
  /// [`set_visible`]: NativeSurrogate::set_visible
  /// [`update_thumbnail_dims`]: NativeSurrogate::update_thumbnail_dims
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    thumbnail_rect: &Rect,
    surrogate_color: Option<&Color>,
    opacity: u8,
    initially_visible: bool,
    border_inset: RECT,
    corner_style: &CornerStyle,
    insert_after: HWND,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    // Surrogate window is sized to the logical source rect (does not occupy
    // the window gap). Thumbnail dimensions come from `thumbnail_rect` and
    // may differ (e.g. target rect for growing animations).
    let logical_src = to_logical(source_rect, &border_inset);
    let logical_thumb = to_logical(thumbnail_rect, &border_inset);

    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP,
        logical_src.x(),
        logical_src.y(),
        logical_src.width(),
        logical_src.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create surrogate window.".to_string(),
      ));
    }

    // Extend the DWM glass sheet over the entire client area so that regions
    // not covered by the DWM thumbnail are transparent rather than opaque
    // black (which is the GDI default for a `WS_POPUP` with a null background
    // brush). The thumbnail is composited on top of this transparent sheet, so
    // only the thumbnail area shows content; everything else is see-through.
    {
      use windows::Win32::UI::Controls::MARGINS;
      let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
      };
      // SAFETY: `hwnd` is a valid window handle. `margins` is stack-allocated
      // and live for the duration of this call.
      unsafe {
        let _ = DwmExtendFrameIntoClientArea(hwnd, &raw const margins);
      }
    }

    apply_backdrop(hwnd, surrogate_color);
    apply_corner_preference(hwnd, corner_style);

    // Register the DWM thumbnail at `thumbnail_rect` dimensions. For shrinking
    // animations this equals `source_rect` so the thumbnail fills the whole
    // surrogate at start (wipe/clip effect). For growing animations this equals
    // the target rect so the surrogate progressively reveals the real window's
    // final content as it expands (curtain-reveal).
    //
    // `opacity` is baked into the initial registration so the first rendered
    // frame shows the correct transparency without a separate
    // `DwmUpdateThumbnailProperties` call.
    //
    // Failure is non-fatal: the surrogate still shows its backdrop color if
    // configured.
    let thumbnail = register_thumbnail(
      hwnd,
      source_hwnd,
      logical_thumb.width(),
      logical_thumb.height(),
      border_inset,
      opacity,
    )
    .unwrap_or(0);

    // Constructed before the final `SetWindowPos` call (rather than after) so
    // that if it fails, `?`'s early return drops `this` — running `Drop`'s
    // thumbnail-unregister and window-destroy cleanup — instead of leaking
    // the overlay window and DWM thumbnail.
    let this = Self {
      hwnd: hwnd.0,
      thumbnail,
      content_size: (logical_thumb.width(), logical_thumb.height()),
      border_inset,
      is_visible: initially_visible,
      last_opacity: opacity,
      last_rect: None,
    };

    // Set the initial Z-order position and optionally show the surrogate.
    // `insert_after` is caller-controlled: resize/open surrogates pass
    // `HWND(0)` (HWND_TOP) so they appear above any co-active close surrogate;
    // close and workspace surrogates pass `source_hwnd` to sit just below it.
    //
    // SAFETY: Both handles are valid.
    let show_flag = if initially_visible {
      SWP_SHOWWINDOW
    } else {
      SET_WINDOW_POS_FLAGS::default()
    };
    unsafe {
      SetWindowPos(
        hwnd,
        insert_after,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | show_flag,
      )
    }?;

    Ok(this)
  }

  /// Reconfigures this already-existing surrogate window (and its DWM
  /// thumbnail registration, when still valid) to track a fresh resize/move
  /// session on `source_hwnd`, instead of creating a brand new surrogate
  /// window and re-registering a thumbnail from scratch.
  ///
  /// Used to reuse a warm surrogate (see `AnimationManager::warm_surrogates`)
  /// for a follow-up resize of the same window shortly after its previous
  /// session ended -- skips the `CreateWindowExW`/`DwmRegisterThumbnail` cost
  /// entirely in the common case, since neither this surrogate window nor
  /// the source window was destroyed in between, so the existing thumbnail
  /// handle is still valid and only needs its rects/opacity updated. Falls
  /// back to registering fresh only if the prior registration never
  /// succeeded (`self.thumbnail == 0`).
  ///
  /// Parameters otherwise mirror [`create`]'s.
  ///
  /// [`create`]: NativeSurrogate::create
  pub fn revive(
    &mut self,
    source_hwnd: HWND,
    source_rect: &Rect,
    thumbnail_rect: &Rect,
    surrogate_color: Option<&Color>,
    opacity: u8,
    initially_visible: bool,
    border_inset: RECT,
    corner_style: &CornerStyle,
    insert_after: HWND,
  ) -> crate::Result<()> {
    apply_backdrop(self.hwnd(), surrogate_color);
    apply_corner_preference(self.hwnd(), corner_style);
    self.border_inset = border_inset;

    let logical_src = to_logical(source_rect, &border_inset);
    let logical_thumb = to_logical(thumbnail_rect, &border_inset);

    let show_flag = if initially_visible {
      SWP_SHOWWINDOW
    } else {
      SET_WINDOW_POS_FLAGS::default()
    };
    // SAFETY: `self.hwnd()` is a valid window handle owned by this
    // surrogate for its whole lifetime.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        insert_after,
        logical_src.x(),
        logical_src.y(),
        logical_src.width(),
        logical_src.height(),
        SWP_NOACTIVATE | show_flag,
      )
    }?;
    self.is_visible = initially_visible;
    self.last_rect = None;

    if self.thumbnail == 0 {
      self.thumbnail = register_thumbnail(
        self.hwnd(),
        source_hwnd,
        logical_thumb.width(),
        logical_thumb.height(),
        border_inset,
        opacity,
      )
      .unwrap_or(0);
    } else {
      // Single combined update (rects + opacity + visible), mirroring
      // `register_thumbnail`'s initial setup -- bypasses
      // `set_thumbnail_rects`/`set_window_opacity`'s unchanged-value skips,
      // since a revived surrogate must always apply fresh values regardless
      // of what its last session happened to leave behind.
      let src_rect = RECT {
        left: border_inset.left,
        top: border_inset.top,
        right: border_inset.left + logical_thumb.width(),
        bottom: border_inset.top + logical_thumb.height(),
      };
      let dst_rect = RECT {
        left: 0,
        top: 0,
        right: logical_thumb.width(),
        bottom: logical_thumb.height(),
      };
      let props = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_RECTDESTINATION
          | DWM_TNP_RECTSOURCE
          | DWM_TNP_OPACITY
          | DWM_TNP_VISIBLE
          | DWM_TNP_SOURCECLIENTAREAONLY,
        rcDestination: dst_rect,
        rcSource: src_rect,
        opacity,
        fVisible: true.into(),
        fSourceClientAreaOnly: false.into(),
        ..Default::default()
      };
      // SAFETY: `self.thumbnail` is a valid handle (checked non-zero above).
      unsafe {
        let _ =
          DwmUpdateThumbnailProperties(self.thumbnail, &raw const props);
      }
    }
    self.last_opacity = opacity;
    self.content_size = (logical_thumb.width(), logical_thumb.height());

    Ok(())
  }

  /// Returns the raw handle of the surrogate overlay window.
  pub fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Returns `true` when a DWM thumbnail was successfully registered.
  ///
  /// `DwmRegisterThumbnail` fails for elevated or UWP source windows.
  /// Callers use this to decide whether to freeze the real window behind
  /// the surrogate or fall back to direct repositioning.
  pub fn has_thumbnail(&self) -> bool {
    self.thumbnail != 0
  }

  /// Returns the logical dimensions the main thumbnail currently samples.
  #[must_use]
  pub fn content_size(&self) -> (i32, i32) {
    self.content_size
  }

  /// Shows or hides the surrogate overlay window without activating it.
  ///
  /// No-op when the window is already in the requested state.
  pub fn set_visible(&mut self, visible: bool) {
    if self.is_visible == visible {
      return;
    }
    self.is_visible = visible;
    use windows::Win32::UI::WindowsAndMessaging::{
      ShowWindow, SW_HIDE, SW_SHOWNOACTIVATE,
    };
    // SAFETY: `HWND(self.hwnd)` is valid until `drop`.
    unsafe {
      ShowWindow(
        HWND(self.hwnd),
        if visible { SW_SHOWNOACTIVATE } else { SW_HIDE },
      );
    }
  }

  /// Repositions the surrogate overlay to `rect` without touching the DWM
  /// thumbnail properties.
  ///
  /// Use this when the thumbnail is managed separately (e.g. workspace-switch
  /// slide animations that update `rcSource`/`rcDestination` independently).
  /// No-op when `rect` matches the last applied position.
  pub fn reposition(&mut self, rect: &Rect) -> crate::Result<()> {
    if self.last_rect.as_ref() == Some(rect) {
      return Ok(());
    }
    // SAFETY: `HWND(self.hwnd)` is the overlay created in `create` and remains
    // valid until `drop`. `SWP_NOZORDER` makes `hWndInsertAfter` irrelevant.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING | SWP_NOZORDER,
      )
    }?;
    self.last_rect = Some(rect.clone());
    Ok(())
  }

  /// Sets the DWM thumbnail visibility flag without changing any other
  /// thumbnail properties.
  ///
  /// No-op when no thumbnail was registered.
  pub fn set_thumbnail_visible(&self, visible: bool) {
    if self.thumbnail == 0 {
      return;
    }
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_VISIBLE,
      fVisible: visible.into(),
      ..Default::default()
    };
    // SAFETY: `self.thumbnail` is a valid handle. `props` is stack-allocated.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.thumbnail, &raw const props);
    }
  }

  /// Sets the DWM thumbnail opacity via `DWM_TNP_OPACITY`.
  ///
  /// `opacity` ranges from 0 (fully transparent) to 255 (fully opaque). The
  /// SWCA acrylic backdrop is unaffected — only the thumbnail content fades.
  /// No-op when `opacity` matches the last applied value or when no thumbnail
  /// is registered.
  pub fn set_window_opacity(&mut self, opacity: u8) {
    if opacity == self.last_opacity {
      return;
    }
    self.last_opacity = opacity;
    if self.thumbnail == 0 {
      return;
    }
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_OPACITY,
      opacity,
      ..Default::default()
    };
    // SAFETY: `self.thumbnail` is a valid handle. `props` is stack-allocated.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.thumbnail, &raw const props);
    }
  }

  /// Updates the DWM thumbnail source and destination rects in a single call.
  ///
  /// `rc_src` is the source-window-local rect to sample from; `rc_dst` is the
  /// surrogate-local rect to render into. Always forces `fVisible = true` and
  /// `fSourceClientAreaOnly = false`. Opacity is not set here; callers must
  /// follow with [`set_window_opacity`] each frame. No-op when no thumbnail was
  /// registered.
  ///
  /// [`set_window_opacity`]: NativeSurrogate::set_window_opacity
  pub fn set_thumbnail_rects(&self, rc_src: RECT, rc_dst: RECT) {
    if self.thumbnail == 0 {
      return;
    }
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTSOURCE
        | DWM_TNP_RECTDESTINATION
        | DWM_TNP_VISIBLE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcSource: rc_src,
      rcDestination: rc_dst,
      fVisible: true.into(),
      fSourceClientAreaOnly: false.into(),
      ..Default::default()
    };
    // SAFETY: `self.thumbnail` is a valid handle. `props` is stack-allocated.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.thumbnail, &raw const props);
    }
  }

  /// Updates the DWM thumbnail source and destination dimensions in a single
  /// `DwmUpdateThumbnailProperties` call.
  ///
  /// Cheaper than [`reregister_thumbnail`] for cases where the sampled area
  /// changes but the source window is unchanged. Avoids the three-call
  /// un-register / re-register / update-properties round-trip, which is paid
  /// on every keypress during a key-held resize.
  ///
  /// Falls back to a full [`reregister_thumbnail`] if the update fails (e.g.
  /// the thumbnail handle has become stale). No-op when no thumbnail was
  /// registered.
  ///
  /// [`reregister_thumbnail`]: NativeSurrogate::reregister_thumbnail
  pub fn update_thumbnail_dims(
    &mut self,
    source_hwnd: HWND,
    logical_width: i32,
    logical_height: i32,
    border_inset: RECT,
  ) {
    if self.thumbnail == 0 {
      return;
    }
    let src_rect = RECT {
      left: border_inset.left,
      top: border_inset.top,
      right: border_inset.left + logical_width,
      bottom: border_inset.top + logical_height,
    };
    let dst_rect = RECT {
      left: 0,
      top: 0,
      right: logical_width,
      bottom: logical_height,
    };
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTDESTINATION
        | DWM_TNP_RECTSOURCE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcDestination: dst_rect,
      rcSource: src_rect,
      fSourceClientAreaOnly: false.into(),
      ..Default::default()
    };
    // SAFETY: `self.thumbnail` is a valid handle. `props` is stack-allocated.
    if unsafe { DwmUpdateThumbnailProperties(self.thumbnail, &raw const props) }
      .is_err()
    {
      // Stale handle — fall back to a full re-registration.
      self.reregister_thumbnail(
        source_hwnd,
        logical_width,
        logical_height,
        border_inset,
      );
      return;
    }
    self.content_size = (logical_width, logical_height);
    self.border_inset = border_inset;
    self.last_rect = None;
  }

  /// Unregisters the current DWM thumbnail and registers a new one at
  /// `logical_width` × `logical_height`.
  ///
  /// Prefer [`update_thumbnail_dims`] where possible — the unregister →
  /// register window here can straddle a DWM composition, blanking the
  /// surrogate to backdrop-only for a frame. This full re-registration is
  /// the fallback for stale thumbnail handles.
  ///
  /// [`update_thumbnail_dims`]: NativeSurrogate::update_thumbnail_dims
  pub fn reregister_thumbnail(
    &mut self,
    source_hwnd: HWND,
    logical_width: i32,
    logical_height: i32,
    border_inset: RECT,
  ) {
    // SAFETY: `self.thumbnail` is a valid handle (or 0). Unregistering before
    // re-registering prevents a duplicate thumbnail on the same destination.
    if self.thumbnail != 0 {
      unsafe {
        let _ = DwmUnregisterThumbnail(self.thumbnail);
      }
      self.thumbnail = 0;
    }
    self.thumbnail = register_thumbnail(
      HWND(self.hwnd),
      source_hwnd,
      logical_width,
      logical_height,
      border_inset,
      self.last_opacity,
    )
    .unwrap_or(0);
    self.content_size = (logical_width, logical_height);
    self.border_inset = border_inset;
    // Force the next reposition call through even if the rect is unchanged,
    // ensuring the surrogate is repositioned after a thumbnail size change.
    self.last_rect = None;
  }

  /// Moves and resizes the surrogate overlay to `rect` and sets the whole-window
  /// opacity to `opacity` (0 = fully transparent, 255 = opaque).
  pub fn update(&mut self, rect: &Rect, opacity: u8) -> crate::Result<()> {
    self.reposition(rect)?;
    self.set_window_opacity(opacity);
    Ok(())
  }

  /// Queues a reposition to `rect` into `batch` instead of issuing an
  /// immediate `SetWindowPos`.
  ///
  /// All surrogates queued into the same [`SurrogateBatch`] are repositioned
  /// atomically when the batch is committed, so adjacent windows' edges move
  /// in the same DWM composition frame. No-op when `rect` matches the last
  /// applied position.
  pub fn defer_reposition(
    &mut self,
    batch: &mut SurrogateBatch,
    rect: &Rect,
  ) {
    if self.last_rect.as_ref() == Some(rect) {
      return;
    }
    batch.push(self.hwnd, rect.clone());
    self.last_rect = Some(rect.clone());
  }

  /// Applies SWCA acrylic blur-behind directly to this surrogate window.
  ///
  /// Replaces the DWM glass backdrop (extended via `DwmExtendFrameIntoClientArea`)
  /// with an acrylic blur layer. The DWM thumbnail is composited on top at the
  /// current opacity. Call once after creation; the effect persists for the
  /// lifetime of the surrogate.
  ///
  /// This is a no-op when SWCA is unavailable (pre-Windows 10 1607).
  pub fn apply_swca(&self, tint: u32) {
    apply_swca_accent(
      HWND(self.hwnd),
      ACCENT_ENABLE_ACRYLICBLURBEHIND,
      0,
      tint,
    );
  }
}

impl Drop for NativeSurrogate {
  fn drop(&mut self) {
    // SAFETY: All thumbnail handles and `self.hwnd` are valid handles
    // created by this type. Thumbnails must be unregistered before the
    // destination window is destroyed.
    unsafe {
      if self.thumbnail != 0 {
        let _ = DwmUnregisterThumbnail(self.thumbnail);
      }
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}

#[cfg(test)]
mod tests {
  use windows::{
    core::w,
    Win32::System::Threading::{
      GetCurrentProcess, GetGuiResources, GR_USEROBJECTS,
    },
  };

  use super::{
    ensure_class_registered, BeginDeferWindowPos, CreateWindowExW,
    DeferWindowPos, DestroyWindow, EndDeferWindowPos, Rect, SurrogateBatch,
    HWND, SWP_NOSENDCHANGING, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_POPUP,
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

  /// Returns this process's current USER-object count.
  fn user_objects() -> i64 {
    // SAFETY: The pseudo-handle returned by `GetCurrentProcess` is always
    // valid and needs no closing.
    i64::from(unsafe { GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS) })
  }

  /// Creates a hidden, off-screen surrogate-class popup window for use as a
  /// reposition target.
  ///
  /// `index` offsets the window's y position so several probe windows do
  /// not overlap exactly.
  ///
  /// Returns `None` when window creation fails (e.g. no interactive window
  /// station), letting the caller skip rather than fail spuriously.
  fn create_probe_window(index: i32) -> Option<HWND> {
    ensure_class_registered();

    // SAFETY: The class is registered above; all other arguments are valid.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP,
        -8000,
        100 + index * 10,
        400,
        300,
        None,
        None,
        None,
        None,
      )
    };

    (hwnd.0 != 0).then_some(hwnd)
  }

  /// `DeferWindowPos` rejects `SWP_NOSENDCHANGING`, so it must never appear
  /// in the batch path's flags -- while the `SetWindowPos` fallback, which
  /// does honor it, keeps it.
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

    // SAFETY: `hwnd` is a live window owned by this process; the `HDWP` is
    // ended on every path below.
    let accepted = unsafe {
      match BeginDeferWindowPos(1) {
        Err(_) => None,
        Ok(hdwp) => {
          let result = DeferWindowPos(
            hdwp,
            hwnd,
            HWND(0),
            -8000,
            120,
            400,
            300,
            SurrogateBatch::deferred_flags(),
          );
          let ok = result.is_ok();
          let _ = EndDeferWindowPos(result.unwrap_or(hdwp));
          Some(ok)
        }
      }
    };

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
  /// `BeginDeferWindowPos` allocates an `HDWP`, a USER object released only
  /// by `EndDeferWindowPos`. Abandoning it after a failed `DeferWindowPos`
  /// leaked one object per animation frame, exhausting the process's
  /// 10,000-object limit within a few hundred gestures.
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
