use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use windows::{
  core::PWSTR,
  Win32::{
    Foundation::{CloseHandle, HWND, RECT},
    Graphics::{
      Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
      Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC,
        DeleteObject, GetDC, GetPixel, ReleaseDC, SelectObject, HGDIOBJ,
        SRCCOPY,
      },
    },
    System::Threading::{
      OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
      PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{
      GetWindowRect, GetWindowThreadProcessId, IsWindow, SetWindowPos,
      SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE,
      SWP_NOSENDCHANGING, SWP_NOZORDER,
    },
  },
};

/// Pixels inset from the content edge when sampling the backdrop color.
///
/// Matches `EDGE_SAMPLE_INSET_PX` in `native_surrogate.rs` so both the
/// sampled backdrop and the (fallback) edge-extension thumbnails read from
/// the same source strip.
const EDGE_SAMPLE_INSET: i32 = 4;

/// Upper bound on how long [`ResizeSession::pre_commit`] waits for the real
/// window to confirm it actually reached `target_rect` before giving up and
/// uncloaking anyway.
///
/// `pre_commit` used to skip `SWP_ASYNCWINDOWPOS` entirely, as a correctness
/// guarantee -- but that meant a plain `SetWindowPos` blocked the calling
/// thread (the WM's single async main loop) until the target process's
/// message queue processed the resize, with no upper bound at all. Apps with
/// a busy main thread (Outlook, observed taking up to ~284ms; Electron/
/// Chromium-based apps to a lesser degree) stalled all mouse/keybinding/IPC
/// handling for that whole duration. `pre_commit` now issues the move
/// asynchronously and polls (throttled, see `commit_poll_parity`) for it to land
/// across subsequent ticks instead -- this bounds the rare case where an app
/// never confirms (hung, or genuinely this slow) without reintroducing an
/// unbounded main-loop stall. Generous relative to every real duration
/// observed so far so it essentially never fires for a merely-slow (not
/// hung) app.
const COMMIT_CONFIRM_MAX_WAIT: Duration = Duration::from_millis(500);

/// Best-effort process name (e.g. `"outlook"`) owning `hwnd`, for labeling
/// [`COMMIT_CONFIRM_MAX_WAIT`]'s warning -- `None` on any failure, in which
/// case the caller falls back to logging the raw `hwnd`. Only ever called
/// once the wait has already timed out, so the extra `OpenProcess`/
/// `QueryFullProcessImageNameW` cost never lands on the common (fast) path.
fn process_name_for_warning(hwnd: HWND) -> Option<String> {
  let mut process_id = 0u32;
  // SAFETY: `hwnd` is a valid window handle for the lifetime of this call.
  unsafe {
    GetWindowThreadProcessId(hwnd, Some(&raw mut process_id));
  }

  // SAFETY: `process_id` was just populated above.
  let process_handle = unsafe {
    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
  }
  .ok()?;

  let mut buffer = [0u16; 256];
  let mut length = u32::try_from(buffer.len()).ok()?;

  // SAFETY: `process_handle` is valid; `buffer`/`length` are stack-allocated
  // and live for the duration of the call.
  let query_res = unsafe {
    QueryFullProcessImageNameW(
      process_handle,
      PROCESS_NAME_WIN32,
      PWSTR(buffer.as_mut_ptr()),
      &raw mut length,
    )
  };
  // SAFETY: `process_handle` is a valid, open handle.
  let _ = unsafe { CloseHandle(process_handle) };
  query_res.ok()?;

  let exe_path = String::from_utf16_lossy(&buffer[..length as usize]);
  exe_path
    .rsplit('\\')
    .next()
    .map(|file_name| {
      file_name.split('.').next().unwrap_or(file_name).to_string()
    })
}

use crate::{
  native_surrogate::to_logical, BlurOverlayParams, BorderOverlayParams,
  Color, CornerStyle, NativeSurrogate, Rect, SurrogateBatch,
};

/// Options for [`ResizeSession::begin`].
pub struct SessionOptions {
  /// DWM thumbnail opacity (0–255) from the window-effects config.
  pub effect_opacity: u8,
  /// Precomputed backdrop color for the surrogate, bypassing the on-screen
  /// edge sample.
  ///
  /// Sampling performs two GPU→CPU `BitBlt` readbacks per session, which
  /// multiplies into a visible hitch when a relayout begins sessions for
  /// many windows at once. Callers that cache the color per window (see
  /// `AnimationManager`) pass it here; `None` samples fresh.
  pub edge_color: Option<Color>,
  /// Whether the surrogate is visible immediately after creation.
  pub initially_visible: bool,
  /// Corner style to apply to the surrogate, matching the real window's
  /// configured style so the surrogate is visually consistent during the
  /// animation.
  pub corner_style: CornerStyle,
  /// When `true`, the surrogate is placed at the top of the non-topmost
  /// Z-order (`HWND_TOP`) so it appears above any co-active close surrogates.
  /// Pass `false` for close surrogates, which should remain below resize and
  /// open surrogates that fill the vacated space.
  pub place_at_top: bool,
  /// Tint/blur-amount/corner-radius/opacity/saturation for the acrylic
  /// blur overlay tracking this session, or `None` when blur-behind isn't
  /// configured. Snapshotted rather than applied directly to the
  /// surrogate -- the actual blur comes from the external
  /// `NativeBlurOverlay` tracker in `AnimationManager`/`platform_sync`,
  /// which needs these values even after the window is detached from the
  /// layout tree (close animations).
  pub blur_overlay: Option<BlurOverlayParams>,
  /// Color/width/corner-radius/opacity for the border overlay tracking this
  /// session, or `None` when the border effect isn't configured. Same
  /// snapshot rationale as `blur_overlay`.
  pub border_overlay: Option<BorderOverlayParams>,
}

/// Tracks a single window's resize/move animation and manages its surrogate
/// overlay.
///
/// On `WmState` drop, [`commit`] is called on all active sessions so no window
/// is left at an intermediate position after a crash or forced exit.
///
/// [`commit`]: ResizeSession::commit
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct ResizeSession {
  /// Raw handle to the real app window. Stored as `isize` to avoid `Send`
  /// issues with windows-rs handle types. Set to `0` by `pre_commit` when
  /// the window has been destroyed.
  hwnd: isize,
  /// Final target rect for the real window (physical, including invisible
  /// border).
  target_rect: Rect,
  /// Surrogate overlay; `None` if creation failed.
  surrogate: Option<NativeSurrogate>,
  /// Invisible border insets (left, top, right, bottom) of the source window
  /// in physical pixels. Applied when converting physical rects to the logical
  /// (visible-content) rects that the surrogate is sized to.
  border_inset: RECT,
  /// DWM thumbnail opacity (0–255) from the window-effects config.
  ///
  /// Used as the surrogate opacity when the animation has no per-frame fade
  /// component, so the thumbnail matches the real window's `SetLayeredWindowAttributes`
  /// opacity throughout the move/resize.
  pub effect_opacity: u8,
  /// Backdrop color applied to the surrogate, either passed in via
  /// [`SessionOptions`] or freshly sampled at session start.
  ///
  /// Exposed via [`edge_color`] so callers can cache it per window and skip
  /// the two-`BitBlt` screen sample on subsequent sessions.
  ///
  /// [`edge_color`]: ResizeSession::edge_color
  edge_color: Option<Color>,
  /// `true` while every target this session has been given matches the
  /// source dimensions — the session only moves the real window, never
  /// resizes it.
  ///
  /// A pure move needs no `WM_NCCALCSIZE`/full repaint, so repositions may
  /// omit `SWP_FRAMECHANGED`. Cleared permanently by the first redirect
  /// that changes the target dimensions.
  is_move_only: bool,
  /// `true` when no dimension shrinks (target >= source in both width and
  /// height). Curtain-reveal mode.
  ///
  /// Growing sessions use a curtain-reveal: the cloaked window is
  /// pre-positioned at the target so DWM captures correctly-sized content,
  /// and `sync_registration` upgrades the thumbnail to target dims once the
  /// window's actual geometry catches up. Mixed/shrinking sessions use
  /// clip/wipe: thumbnail at source dimensions, real window stays at source
  /// until `maybe_handoff`/`pre_commit`.
  is_growing: bool,
  /// When `true`, each frame animates the DWM thumbnail `rcDestination`
  /// toward/away from the surrogate center instead of repositioning the
  /// surrogate window. Used for zoom-in (open) and zoom-out (close) effects.
  pub zoom: bool,
  /// Last progress value passed to [`update_zoom_fade`] (0.0 = zero-size,
  /// 1.0 = full surrogate). Defaults to `1.0` (fully shown) so non-`zoom`
  /// sessions -- which never call `update_zoom_fade` -- read as unscaled.
  ///
  /// The blur overlay tracks this session's surrogate at a fixed full-size
  /// `current_rect` throughout a zoom animation (see its doc comment) since
  /// only the thumbnail's `rcDestination`, not the surrogate window itself,
  /// actually animates in size -- so without this, the overlay would appear
  /// at full opacity from the very first frame while the window it's
  /// supposedly attached to is still a speck at the center. Scaling the
  /// overlay's own opacity by this value in [`blur_overlay_params`] instead
  /// fades it in/out in lockstep with the thumbnail's zoom.
  ///
  /// [`update_zoom_fade`]: ResizeSession::update_zoom_fade
  /// [`blur_overlay_params`]: ResizeSession::blur_overlay_params
  zoom_progress: f32,
  /// `true` once the real window has been repositioned at the current
  /// `target_rect` (at session start for growing curtain-reveals, or
  /// mid-animation via [`maybe_handoff`]). Reset whenever a redirect changes
  /// the target.
  ///
  /// [`maybe_handoff`]: ResizeSession::maybe_handoff
  handoff_done: bool,
  /// Toggled on each `sync_registration` call that reaches the
  /// `GetWindowRect` poll; the poll only actually runs on alternating
  /// calls, halving its per-tick syscall cost during the handoff tail. A
  /// window's actual resize landing one tick later than detected is
  /// imperceptible, but the syscall itself is real per-session per-tick
  /// cost that stacks up with concurrent handoffs — worse on high-refresh
  /// monitors where ticks fire 2-3x as often as at 60 Hz.
  poll_parity: bool,
  /// Same throttling idea as `poll_parity`, for `pre_commit`'s own
  /// `GetWindowRect` poll -- kept separate rather than shared, since the two
  /// polls run in different (though normally non-overlapping) phases of a
  /// session's life and sharing the flag would make each one's cadence
  /// depend on how often the other happens to run too.
  commit_poll_parity: bool,
  /// `true` once the session has successfully cloaked its source window.
  ///
  /// Used by `platform_sync` to skip the per-tick `DwmGetWindowAttribute`
  /// round-trip on steady-state animation frames — the query only needs to
  /// fire on the first `Frozen` frame (where the cloak state is unknown)
  /// and after the session is torn down.
  session_cloaked: bool,
  /// Thumbnail content dims that need to be applied on the next animation
  /// tick via `DwmUpdateThumbnailProperties`.
  ///
  /// Set by `update_target` instead of calling `update_thumbnail_dims`
  /// immediately, so the DWM cross-process call is deferred from keypress
  /// time (latency-sensitive) to vsync tick time (where there is budget).
  /// Consumed by `defer_update` before `sync_registration` so the short-
  /// circuit check still works on the same tick.
  pending_thumbnail_dims: Option<(i32, i32)>,
  /// Tint/blur-amount/corner-radius/opacity/saturation for the acrylic
  /// blur overlay tracking this session, snapshotted from
  /// `SessionOptions::blur_overlay`. See [`blur_overlay_params`].
  ///
  /// [`blur_overlay_params`]: ResizeSession::blur_overlay_params
  blur_overlay: Option<BlurOverlayParams>,
  /// Color/width/corner-radius/opacity for the border overlay tracking this
  /// session, snapshotted from `SessionOptions::border_overlay`. See
  /// [`border_overlay_params`].
  ///
  /// [`border_overlay_params`]: ResizeSession::border_overlay_params
  border_overlay: Option<BorderOverlayParams>,
  /// Live logical (border-deflated) on-screen rect for the current frame,
  /// mirroring `WorkspaceSurrogate::current_rect`. `None` while there's
  /// nothing to show (no surrogate, or fully clipped off-screen). Ignored
  /// for zoom sessions -- see [`current_rect`].
  ///
  /// [`current_rect`]: ResizeSession::current_rect
  current_rect: Option<Rect>,
  /// `true` once [`pre_commit`] has confirmed (or given up waiting for, see
  /// [`COMMIT_CONFIRM_MAX_WAIT`]) the real window landing at `target_rect`.
  /// The caller must not uncloak/fade the surrogate until this is `true`.
  /// Reset to `false` by [`update_target`] on a redirect, since that
  /// invalidates whatever commit was in flight for the old target.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  /// [`update_target`]: ResizeSession::update_target
  commit_confirmed: bool,
  /// When `pre_commit`'s asynchronous move was issued, or `None` before the
  /// first `pre_commit` call for the current target. See
  /// [`COMMIT_CONFIRM_MAX_WAIT`].
  commit_started_at: Option<Instant>,
}

impl ResizeSession {
  /// Creates a resize session with a DWM surrogate overlay.
  ///
  /// The thumbnail is always registered at source dims — never larger than
  /// the real window's current content, since an oversampled `rcSource`
  /// renders as a transparent hole that bleeds the desktop through the
  /// surrogate. Growing sessions pre-position the cloaked window at the
  /// target (curtain-reveal); `sync_registration` upgrades the registration
  /// to target dims once the window's actual geometry catches up, and the
  /// animated area beyond the source content shows the sampled backdrop
  /// color until then. When surrogate creation fails the animation falls
  /// back to direct repositioning.
  pub fn begin(
    hwnd: HWND,
    source_rect: &Rect,
    target_rect: &Rect,
    options: SessionOptions,
  ) -> crate::Result<Self> {
    let border_inset = compute_border_inset(hwnd);

    let is_growing = target_rect.width() >= source_rect.width()
      && target_rect.height() >= source_rect.height();
    let is_move_only = target_rect.width() == source_rect.width()
      && target_rect.height() == source_rect.height();

    let effect_opacity = options.effect_opacity;

    // Sample the dominant background color near the trailing content edge
    // to use as the surrogate's solid backdrop. The backdrop fills any gap
    // between the animated rect and the registered thumbnail area (mixed
    // resizes) with a uniform color that blends into the app's own background.
    //
    // Skipped entirely when `options.blur_overlay` is `Some`: a live acrylic
    // overlay is tracked behind this surrogate for the whole session (see
    // `platform_sync`'s post-flush loop), and `apply_backdrop` paints the
    // *entire* surrogate window with this color via `ACCENT_ENABLE_GRADIENT`,
    // not just the gap. A fully opaque sample hides the tracked overlay
    // outright; blending it at `effect_opacity` (the same ratio
    // `transparency.opacity` applies everywhere else) was tried and still
    // read as solid at typical (high) opacity values, since a
    // mostly-opaque fill still dominates the blend visually. Leaving
    // `surrogate_color` `None` keeps the surrogate's own background fully
    // transparent instead (see `NativeSurrogate::create`), so the tracked
    // overlay's live blur/tint/saturation is what actually shows through the
    // gap, at the cost of the gap no longer sampling the app's own color.
    //
    // Otherwise skipped when the caller supplies a cached color — the sample
    // costs two GPU→CPU `BitBlt` readbacks, which stack up when a relayout
    // begins many sessions in the same keypress. Falls back to transparent
    // (no backdrop) when sampling fails.
    let edge_color = if options.blur_overlay.is_some() {
      None
    } else {
      options.edge_color.or_else(|| {
        let logical_src = to_logical(source_rect, &border_inset);
        sample_edge_color(
          logical_src.x(),
          logical_src.y(),
          logical_src.width(),
          logical_src.height(),
        )
      })
    };

    let insert_after = if options.place_at_top { HWND(0) } else { hwnd };
    // Thumbnail registered at source dims for all directions (see doc
    // comment): the window is only source-sized at this point, and
    // registering larger would oversample.
    let surrogate = match NativeSurrogate::create(
      hwnd,
      source_rect,
      source_rect,
      edge_color.as_ref(),
      effect_opacity,
      options.initially_visible,
      border_inset,
      &options.corner_style,
      insert_after,
    ) {
      Ok(s) => Some(s),
      Err(err) => {
        tracing::warn!(
          "Failed to create surrogate: {err}. Falling back to direct \
           animation."
        );
        None
      }
    };

    Ok(Self {
      hwnd: hwnd.0,
      target_rect: target_rect.clone(),
      surrogate,
      border_inset,
      effect_opacity,
      edge_color,
      is_move_only,
      is_growing,
      zoom: false,
      zoom_progress: 1.0,
      handoff_done: is_growing,
      poll_parity: false,
      commit_poll_parity: false,
      session_cloaked: false,
      pending_thumbnail_dims: None,
      blur_overlay: options.blur_overlay,
      border_overlay: options.border_overlay,
      current_rect: None,
      commit_confirmed: false,
      commit_started_at: None,
    })
  }

  /// Returns the tint/blur-amount/corner-radius/opacity/saturation for the
  /// acrylic blur-overlay tracker in `AnimationManager`, or `None` when
  /// blur-behind isn't configured for this window.
  ///
  /// For `zoom` sessions, `opacity` is scaled by `zoom_progress` so the
  /// overlay fades in/out alongside the thumbnail's own zoom instead of
  /// sitting at full opacity for the whole animation -- see
  /// `zoom_progress`'s doc comment.
  #[must_use]
  pub fn blur_overlay_params(&self) -> Option<BlurOverlayParams> {
    let mut params = self.blur_overlay?;
    if self.zoom {
      params.opacity *= self.zoom_progress;
    }
    Some(params)
  }

  /// Returns the color/width/corner-radius/opacity for the border-overlay
  /// tracker in `AnimationManager`, or `None` when the border effect isn't
  /// configured for this window. Mirrors [`blur_overlay_params`] exactly,
  /// including the `zoom_progress` opacity scaling.
  ///
  /// [`blur_overlay_params`]: ResizeSession::blur_overlay_params
  #[must_use]
  pub fn border_overlay_params(&self) -> Option<BorderOverlayParams> {
    let mut params = self.border_overlay?;
    if self.zoom {
      params.opacity *= self.zoom_progress;
    }
    Some(params)
  }

  /// `HWND` of this session's surrogate, or `None` when surrogate creation
  /// failed.
  ///
  /// Used as the acrylic blur-overlay tracker's z-order anchor while a
  /// session is active: the surrogate is what's actually visible on screen
  /// (the real window is cloaked for the duration), so the overlay must sit
  /// directly behind *it*, not the (hidden) real window.
  #[must_use]
  pub fn surrogate_hwnd(&self) -> Option<HWND> {
    self.surrogate.as_ref().map(NativeSurrogate::hwnd)
  }

  /// Live on-screen rect (logical, border-deflated) for the acrylic-overlay
  /// tracker to follow this tick, or `None` when there's nothing to show.
  ///
  /// Zoom sessions never move their surrogate window -- only the DWM
  /// thumbnail's `rcDestination` animates within a surrogate fixed at
  /// `target_rect` for the whole animation (see [`update_zoom_fade`]) -- so
  /// this always reports the fixed target rect for them; no per-frame
  /// storage is needed.
  ///
  /// [`update_zoom_fade`]: ResizeSession::update_zoom_fade
  #[must_use]
  pub fn current_rect(&self) -> Option<Rect> {
    if self.zoom {
      Some(to_logical(&self.target_rect, &self.border_inset))
    } else {
      self.current_rect.clone()
    }
  }

  /// Returns the final target rect for the real window (physical, including
  /// invisible border).
  #[must_use]
  pub fn target_rect(&self) -> &Rect {
    &self.target_rect
  }

  /// Returns `true` when the cloaked real window should be pre-positioned at
  /// the target rect immediately after cloaking.
  ///
  /// Required for growing curtain-reveal sessions so DWM captures
  /// correctly-sized content before the surrogate begins expanding.
  pub fn needs_preposition(&self) -> bool {
    self.is_growing
  }

  /// Returns `true` while the session has never been asked to change the
  /// real window's dimensions — every target so far matched the source size.
  ///
  /// Pure moves need no `WM_NCCALCSIZE`/full repaint, so callers may omit
  /// `SWP_FRAMECHANGED` when repositioning the window.
  #[must_use]
  pub fn is_move_only(&self) -> bool {
    self.is_move_only
  }

  /// Returns the backdrop color in use by this session's surrogate, if any.
  ///
  /// Callers cache this per window so subsequent sessions can skip the
  /// two-`BitBlt` screen sample via [`SessionOptions::edge_color`].
  #[must_use]
  pub fn edge_color(&self) -> Option<&Color> {
    self.edge_color.as_ref()
  }

  /// Returns `true` when this session has already cloaked the source window.
  ///
  /// Used by `platform_sync` to skip the per-tick `DwmGetWindowAttribute`
  /// query on steady-state `Frozen` frames where the cloak state is known.
  pub fn is_session_cloaked(&self) -> bool {
    self.session_cloaked
  }

  /// Marks the session's source window as cloaked.
  ///
  /// Called by `platform_sync` after `set_cloaked(true)` succeeds so that
  /// subsequent `Frozen` ticks skip the `DwmGetWindowAttribute` round-trip.
  pub fn mark_session_cloaked(&mut self) {
    self.session_cloaked = true;
  }

  /// Whether a surrogate overlay with a valid DWM thumbnail is active.
  ///
  /// Returns `false` when surrogate creation failed, or when the surrogate
  /// window exists but thumbnail registration failed (e.g. elevated/UWP
  /// source windows). Callers use this to decide whether to freeze the real
  /// window behind the surrogate or fall back to direct repositioning.
  pub fn has_surrogate(&self) -> bool {
    self.surrogate.as_ref().map_or(false, |s| s.has_thumbnail())
  }

  /// Makes the surrogate visible.
  ///
  /// Used after creating the surrogate with `initially_visible = false` to
  /// reveal it once the real window has been cloaked.
  pub fn show(&mut self) {
    if let Some(ref mut surrogate) = self.surrogate {
      surrogate.set_visible(true);
    }
  }

  /// Sets the surrogate overlay's whole-window opacity.
  ///
  /// Used to fade the surrogate out over the uncloaked real window at
  /// animation completion, softening the teardown swap.
  pub fn fade_overlay(&mut self, opacity: u8) {
    if let Some(ref mut surrogate) = self.surrogate {
      surrogate.set_window_opacity(opacity);
    }
  }

  /// Animates the DWM thumbnail `rcDestination` toward/away from center.
  ///
  /// `progress` is the eased animation progress (0.0 = zero-size, 1.0 = full
  /// surrogate). Used for zoom-in (open) and zoom-out (close) effects. The
  /// surrogate window itself stays fixed; only the thumbnail rect animates.
  pub fn update_zoom_fade(&mut self, progress: f32, opacity: u8) {
    self.zoom_progress = progress;

    let Some(ref mut surrogate) = self.surrogate else {
      return;
    };
    let logical = to_logical(&self.target_rect, &self.border_inset);
    let w = logical.width();
    let h = logical.height();
    let half_w = (w as f32 / 2.0 * progress).round() as i32;
    let half_h = (h as f32 / 2.0 * progress).round() as i32;
    if half_w <= 0 || half_h <= 0 {
      surrogate.set_visible(false);
    } else {
      let cx = w / 2;
      let cy = h / 2;
      // `rcSource` samples from the real window's coordinate space, so it
      // must carry the same `border_inset` offset as `register_thumbnail`/
      // `update_thumbnail_dims` -- otherwise every zoom-fade frame samples a
      // few pixels up-left of the true content area. `rcDestination` is
      // destination-space within the surrogate's own client area (already
      // sized to the logical rect), so it needs no such offset.
      surrogate.set_thumbnail_rects(
        RECT {
          left: self.border_inset.left,
          top: self.border_inset.top,
          right: self.border_inset.left + w,
          bottom: self.border_inset.top + h,
        },
        RECT {
          left: cx - half_w,
          top: cy - half_h,
          right: cx + half_w,
          bottom: cy + half_h,
        },
      );
      surrogate.set_visible(true);
    }
    surrogate.set_window_opacity(opacity);
  }

  /// Hands the real (cloaked) window off to its final target rect
  /// mid-animation.
  ///
  /// Resizing the real window at the very end of the animation makes the
  /// app's content reflow in a single frame while everything is at rest — a
  /// visible jump. Calling this while a slice of the animation remains moves
  /// that reflow into the motion, where it is far less noticeable, and gives
  /// the app time to repaint before the uncloak.
  ///
  /// The thumbnail registration is downsized to the per-axis minimum of its
  /// current dims and the target dims — never larger than the window before
  /// or after the (asynchronous) resize, so DWM always has real content to
  /// sample and no transparent hole exposes the desktop. Edge-extension
  /// thumbnails cover the remainder of the animated rect. Once the window's
  /// actual geometry reaches the target, [`sync_registration`] re-registers
  /// at exact target dims. `pre_commit` issues a final synchronous move at
  /// completion as the correctness guarantee.
  ///
  /// No-op for zoom sessions (close animations must never move the real
  /// window — their target rect may be off-screen) and when the current
  /// target has already been handed off.
  ///
  /// [`sync_registration`]: ResizeSession::sync_registration
  pub fn maybe_handoff(&mut self) {
    if self.handoff_done || self.zoom || self.hwnd == 0 {
      return;
    }
    self.handoff_done = true;

    // SAFETY: The window is cloaked while a surrogate session is active, so
    // this reposition is invisible. `SWP_NOZORDER` makes `hWndInsertAfter`
    // irrelevant.
    unsafe {
      let _ = SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        self.target_rect.x(),
        self.target_rect.y(),
        self.target_rect.width(),
        self.target_rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER
          | SWP_ASYNCWINDOWPOS | SWP_FRAMECHANGED,
      );
    }

    let logical = to_logical(&self.target_rect, &self.border_inset);
    if let Some(surrogate) = &mut self.surrogate {
      let (cur_w, cur_h) = surrogate.content_size();
      let safe_w = cur_w.min(logical.width());
      let safe_h = cur_h.min(logical.height());
      if (cur_w, cur_h) != (safe_w, safe_h) && safe_w > 0 && safe_h > 0 {
        // Single-call dims update rather than a full re-registration: the
        // unregister → register window of `reregister_thumbnail` can straddle
        // a DWM composition, blanking the surrogate to backdrop-only for a
        // frame.
        surrogate.update_thumbnail_dims(
          HWND(self.hwnd),
          safe_w,
          safe_h,
          self.border_inset,
        );
      }
    }
  }

  /// Converges the thumbnail registration toward the target dims as the
  /// window's actual geometry catches up with the handoff reposition.
  ///
  /// After [`maybe_handoff`] the registration is capped at the per-axis
  /// minimum of old and target dims, leaving the grown axis of a mixed
  /// resize edge-extended. Once `GetWindowRect` confirms the window has
  /// reached the target size, re-registering at exact target dims is safe
  /// and reveals the full new content. Cheap no-op outside the handoff tail.
  ///
  /// [`maybe_handoff`]: ResizeSession::maybe_handoff
  fn sync_registration(&mut self) {
    if !self.handoff_done || self.hwnd == 0 {
      return;
    }
    let target_logical = to_logical(&self.target_rect, &self.border_inset);
    let target_dims = (target_logical.width(), target_logical.height());
    let Some(surrogate) = &mut self.surrogate else {
      return;
    };
    if surrogate.content_size() == target_dims {
      return;
    }

    // Throttle the poll itself to every other tick that reaches this
    // point -- see the field doc comment on `poll_parity` for why.
    self.poll_parity = !self.poll_parity;
    if !self.poll_parity {
      return;
    }

    let mut window = RECT::default();
    // SAFETY: `HWND(self.hwnd)` was verified live at session start; a stale
    // handle only fails the call.
    if unsafe {
      GetWindowRect(HWND(self.hwnd), std::ptr::from_mut(&mut window).cast())
    }
    .is_err()
    {
      return;
    }

    let actual = to_logical(
      &Rect::from_ltrb(window.left, window.top, window.right, window.bottom),
      &self.border_inset,
    );
    if (actual.width(), actual.height()) == target_dims {
      // Use the fast single-call path: the thumbnail handle is still valid
      // so there is no need to unregister and re-register — only rcSource
      // and rcDestination need updating. Falls back to reregister internally
      // if the handle has gone stale.
      surrogate.update_thumbnail_dims(
        HWND(self.hwnd),
        target_dims.0,
        target_dims.1,
        self.border_inset,
      );
    }
  }

  /// Updates the surrogate to the current animation frame position and opacity.
  ///
  /// `current_rect` is the physical animated rect; it is converted to the
  /// logical rect before being applied to the surrogate window.
  ///
  /// `opacity` maps to the DWM thumbnail opacity (0 = transparent, 255 =
  /// opaque). Pass `255` for resize animations where no fade is needed.
  pub fn update(&mut self, current_rect: &Rect, opacity: u8) {
    self.sync_registration();
    let logical = to_logical(current_rect, &self.border_inset);
    self.current_rect = Some(logical.clone());
    if let Some(surrogate) = &mut self.surrogate {
      if let Err(err) = surrogate.update(&logical, opacity) {
        tracing::warn!("Surrogate update failed: {err}.");
      }
    }
  }

  /// Like [`update`], but queues the surrogate reposition into `batch` so
  /// all surrogates in the same animation tick move atomically in one
  /// `DeferWindowPos` transaction.
  ///
  /// Thumbnail and opacity updates are applied immediately — they are DWM
  /// state changes that cannot be deferred, and only become visible at the
  /// next composition alongside the batched repositions.
  ///
  /// [`update`]: ResizeSession::update
  pub fn defer_update(
    &mut self,
    batch: &mut SurrogateBatch,
    current_rect: &Rect,
    opacity: u8,
  ) {
    // Apply deferred thumbnail dims before `sync_registration` so the
    // short-circuit check (`content_size == target_dims`) succeeds on this
    // tick rather than falling back to `GetWindowRect`.
    if let Some((w, h)) = self.pending_thumbnail_dims.take() {
      if let Some(surrogate) = &mut self.surrogate {
        surrogate.update_thumbnail_dims(HWND(self.hwnd), w, h, self.border_inset);
      }
    }
    self.sync_registration();
    let logical = to_logical(current_rect, &self.border_inset);
    self.current_rect = Some(logical.clone());
    if let Some(surrogate) = &mut self.surrogate {
      surrogate.defer_reposition(batch, &logical);
      surrogate.set_window_opacity(opacity);
    }
  }

  /// Updates the surrogate, clamping its visible area to `monitor_rect`.
  ///
  /// When `current_rect` extends outside `monitor_rect`, the surrogate is
  /// constrained to the intersection and the DWM thumbnail `rcSource` and
  /// `rcDestination` are adjusted to show only the visible slice — matching
  /// the approach used by `WorkspaceSurrogate`. Hides the surrogate when
  /// the rect is fully off-screen.
  pub fn update_clipped(
    &mut self,
    current_rect: &Rect,
    monitor_rect: &Rect,
    opacity: u8,
  ) {
    let Some(surrogate) = &mut self.surrogate else {
      self.current_rect = None;
      return;
    };

    let logical = to_logical(current_rect, &self.border_inset);

    let vis_left = logical.x().max(monitor_rect.x());
    let vis_top = logical.y().max(monitor_rect.y());
    let vis_right = (logical.x() + logical.width())
      .min(monitor_rect.x() + monitor_rect.width());
    let vis_bottom = (logical.y() + logical.height())
      .min(monitor_rect.y() + monitor_rect.height());

    if vis_left >= vis_right || vis_top >= vis_bottom {
      surrogate.set_visible(false);
      self.current_rect = None;
      return;
    }

    let src_left = vis_left - logical.x();
    let src_top = vis_top - logical.y();
    let constrained_w = vis_right - vis_left;
    let constrained_h = vis_bottom - vis_top;

    // `rcSource` samples from the real window's coordinate space, so it must
    // carry the same `border_inset` offset as `register_thumbnail`/
    // `update_thumbnail_dims` (see `update_zoom_fade`'s comment) --
    // `src_left`/`src_top` above are the visible slice's offset within the
    // *logical* rect, not yet shifted into the window's physical/source space.
    surrogate.set_thumbnail_rects(
      RECT {
        left: self.border_inset.left + src_left,
        top: self.border_inset.top + src_top,
        right: self.border_inset.left + src_left + constrained_w,
        bottom: self.border_inset.top + src_top + constrained_h,
      },
      RECT { left: 0, top: 0, right: constrained_w, bottom: constrained_h },
    );

    let constrained = Rect::from_xy(vis_left, vis_top, constrained_w, constrained_h);
    if let Err(err) = surrogate.reposition(&constrained) {
      tracing::warn!("Surrogate clipped update failed: {err}.");
    }
    surrogate.set_window_opacity(opacity);
    surrogate.set_visible(true);
    self.current_rect = Some(constrained);
  }

  /// Redirects the session to a new target rect while the surrogate is still
  /// active.
  ///
  /// `current_rect` is the current animated position (used to recompute the
  /// grow/shrink direction for the new `start → new_target` span). When the
  /// direction changes, the DWM thumbnail is re-registered at the appropriate
  /// dimensions so the curtain-reveal or clip/wipe renders correctly:
  ///
  /// - Shrinking → growing: sends an asynchronous `SetWindowPos` to
  ///   pre-position the cloaked real window at the new target so DWM captures
  ///   the correctly-sized content; `sync_registration` upgrades the
  ///   thumbnail once the resize lands.
  /// - Growing → shrinking: updates the thumbnail to `current_rect`
  ///   dimensions (capped at the window's actual dims) so the clip/wipe
  ///   effect starts from the correct boundary.
  /// - Same direction: growing updates position; shrinking only stores the
  ///   new target.
  ///
  /// The thumbnail registration is never enlarged here — an oversampled
  /// `rcSource` renders as a transparent hole while the asynchronous resize
  /// is still in the target app's message queue, bleeding the desktop
  /// through the surrogate on every key-repeat.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  pub fn update_target(&mut self, current_rect: &Rect, new_target: &Rect) {
    let new_is_growing = new_target.width() >= current_rect.width()
      && new_target.height() >= current_rect.height();
    let direction_changed = new_is_growing != self.is_growing;
    let prev_target = to_logical(&self.target_rect, &self.border_inset);
    let prev_target_dims = (prev_target.width(), prev_target.height());

    // A redirect that changes the target dims means the window will be
    // resized at least once — the session permanently stops qualifying for
    // the `SWP_FRAMECHANGED`-free pure-move path.
    self.is_move_only = self.is_move_only
      && new_target.width() == self.target_rect.width()
      && new_target.height() == self.target_rect.height();

    self.is_growing = new_is_growing;
    self.target_rect = new_target.clone();

    // A commit in flight (or already confirmed) was for the old target --
    // this window isn't done animating after all, so `pre_commit` needs to
    // run its full first-call logic again once this new target completes.
    self.commit_confirmed = false;
    self.commit_started_at = None;

    if self.hwnd == 0 {
      return;
    }

    if new_is_growing {
      // Gate on the target dims actually changing — pure-move redirects need
      // neither the reposition nor a thumbnail update, saving an unnecessary
      // WM_NCCALCSIZE in the target app per keypress.
      let logical = to_logical(new_target, &self.border_inset);
      let new_dims = (logical.width(), logical.height());
      let dims_changed = new_dims != prev_target_dims;

      if dims_changed {
        // Pre-position the cloaked real window at the new target so DWM
        // captures correctly-sized content for the curtain-reveal.
        // `SWP_FRAMECHANGED` triggers `WM_NCCALCSIZE` to recalculate the
        // client area for the new size.
        //
        // For pure-move redirects (dims unchanged) both the `SetWindowPos`
        // and the thumbnail update are skipped entirely: the window's content
        // doesn't change, and `pre_commit` issues a synchronous move to the
        // final position just before uncloak. Skipping N_neighbors ×
        // 11-keypresses/sec of async cross-process IPC posts is meaningful for
        // heavy source windows (e.g. browsers with video) and reduces
        // contention on the target process's message queue.
        //
        // SAFETY: Window is cloaked during an active animation.
        unsafe {
          let _ = SetWindowPos(
            HWND(self.hwnd),
            HWND(0),
            new_target.x(),
            new_target.y(),
            new_target.width(),
            new_target.height(),
            SWP_NOACTIVATE
              | SWP_NOSENDCHANGING
              | SWP_NOZORDER
              | SWP_ASYNCWINDOWPOS
              | SWP_FRAMECHANGED,
          );
        }
        // Cap the registration at the per-axis minimum of its current dims
        // and the new target — the reposition above is asynchronous, so the
        // window cannot supply more content than it already has. A redirect
        // below the current registration is downsized on the next animation
        // tick (deferred so the DWM cross-process call fires at vsync time,
        // where there is budget, rather than on every keypress);
        // `sync_registration` upgrades to exact target dims once the resize
        // lands. `defer_update` consumes the pending dims before
        // `sync_registration` so the short-circuit check still works on the
        // same tick.
        if let Some(surrogate) = &self.surrogate {
          let (reg_w, reg_h) = surrogate.content_size();
          let capped = (reg_w.min(new_dims.0), reg_h.min(new_dims.1));
          self.pending_thumbnail_dims =
            if capped == (reg_w, reg_h) { None } else { Some(capped) };
        }
      }
      self.handoff_done = true;
    } else if direction_changed {
      // Was growing, now shrinking: update the thumbnail to the current
      // animated dims so the clip/wipe starts from the correct boundary,
      // capped at the window's actual dims — the earlier asynchronous grow
      // may not have been processed yet. Drop any dims still queued from the
      // grow phase so they don't overwrite this update on the next tick.
      self.pending_thumbnail_dims = None;
      if let Some(surrogate) = &mut self.surrogate {
        let logical = to_logical(current_rect, &self.border_inset);

        let mut window = RECT::default();
        // SAFETY: A stale handle only fails the call.
        let actual = if unsafe {
          GetWindowRect(HWND(self.hwnd), std::ptr::from_mut(&mut window).cast())
        }
        .is_ok()
        {
          to_logical(
            &Rect::from_ltrb(window.left, window.top, window.right, window.bottom),
            &self.border_inset,
          )
        } else {
          logical.clone()
        };

        let safe_w = logical.width().min(actual.width());
        let safe_h = logical.height().min(actual.height());
        if safe_w > 0 && safe_h > 0 {
          surrogate.update_thumbnail_dims(
            HWND(self.hwnd),
            safe_w,
            safe_h,
            self.border_inset,
          );
        }
      }
      self.handoff_done = false;
    } else {
      // Still shrinking: just store the new target; the thumbnail keeps its
      // current registration. Reset the handoff so the window is repositioned
      // near the end of the redirected animation.
      self.handoff_done = false;
    }
  }

  /// Drives this session's final move toward `target_rect`, returning
  /// `true` once it's safe for the caller to uncloak the real window and
  /// start fading the surrogate out.
  ///
  /// Safe to call every tick once the window's animation has completed:
  ///
  /// - First call: if the real window is already at `target_rect`
  ///   (`maybe_handoff`/the initial async preposition normally got it there
  ///   already), finishes immediately. Otherwise issues an *asynchronous*
  ///   move (`SWP_ASYNCWINDOWPOS`) and returns `false` -- unlike a plain
  ///   `SetWindowPos`, this never blocks the calling thread on the target
  ///   process's message queue, however slow it is.
  /// - Later calls: polls (throttled, see `commit_poll_parity`'s doc
  ///   comment) for
  ///   the real window to have actually reached `target_rect`, finishing
  ///   once it has. If [`COMMIT_CONFIRM_MAX_WAIT`] elapses without that
  ///   happening (a hung app, or one that's genuinely this slow), gives up
  ///   and finishes anyway -- seeing the caller uncloak a window that's not
  ///   quite at its final rect yet is a much smaller cost than the
  ///   unbounded main-loop stall this replaced.
  ///
  /// Checks `IsWindow` and nullifies the stored handle if the window has
  /// been destroyed mid-animation, in which case there is nothing left to
  /// wait for and this returns `true` immediately.
  pub fn pre_commit(&mut self) -> bool {
    if self.commit_confirmed {
      return true;
    }

    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(HWND(self.hwnd)).as_bool() } {
      self.hwnd = 0;
      self.finish_commit();
      return true;
    }

    let Some(started) = self.commit_started_at else {
      // First call since this target was set. Skip the move entirely when
      // the real window is already there -- `maybe_handoff` (shrinking
      // sessions) and the initial async preposition (growing sessions)
      // normally get it there well before the animation completes, so this
      // is the common case.
      //
      // SAFETY: `HWND(self.hwnd)` is valid (verified above).
      let mut current = RECT::default();
      let at_target = unsafe {
        GetWindowRect(HWND(self.hwnd), std::ptr::from_mut(&mut current).cast())
          .is_ok()
      } && Rect::from_ltrb(
        current.left,
        current.top,
        current.right,
        current.bottom,
      ) == self.target_rect;

      if at_target {
        self.finish_commit();
        return true;
      }

      self.commit_started_at = Some(Instant::now());
      // SAFETY: `HWND(self.hwnd)` is valid (verified above). `SWP_NOZORDER`
      // makes `hWndInsertAfter` irrelevant.
      unsafe {
        let _ = SetWindowPos(
          HWND(self.hwnd),
          HWND(0),
          self.target_rect.x(),
          self.target_rect.y(),
          self.target_rect.width(),
          self.target_rect.height(),
          SWP_NOACTIVATE
            | SWP_NOSENDCHANGING
            | SWP_NOZORDER
            | SWP_ASYNCWINDOWPOS,
        );
      }
      return false;
    };

    if started.elapsed() > COMMIT_CONFIRM_MAX_WAIT {
      let process = process_name_for_warning(HWND(self.hwnd))
        .unwrap_or_else(|| format!("hwnd {:#x}", self.hwnd));
      tracing::warn!(
        "ResizeSession commit for {process} did not land within \
         {COMMIT_CONFIRM_MAX_WAIT:?} -- uncloaking anyway (see \
         COMMIT_CONFIRM_MAX_WAIT's doc comment)."
      );
      self.finish_commit();
      return true;
    }

    // Throttle the poll itself to every other call reaching this point --
    // see `commit_poll_parity`'s doc comment.
    self.commit_poll_parity = !self.commit_poll_parity;
    if !self.commit_poll_parity {
      return false;
    }

    // SAFETY: `HWND(self.hwnd)` is valid (verified above).
    let mut current = RECT::default();
    let at_target = unsafe {
      GetWindowRect(HWND(self.hwnd), std::ptr::from_mut(&mut current).cast())
        .is_ok()
    } && Rect::from_ltrb(
      current.left,
      current.top,
      current.right,
      current.bottom,
    ) == self.target_rect;

    if at_target {
      self.finish_commit();
      true
    } else {
      false
    }
  }

  /// Finalizes the surrogate for teardown once the real window's placement
  /// is confirmed (or [`COMMIT_CONFIRM_MAX_WAIT`] forces it through).
  ///
  /// Flushes any pending thumbnail dims, then syncs the thumbnail to the
  /// target's exact content size and repositions the surrogate one last
  /// time, so it becomes a pixel-aligned 1:1 mirror of the resized window
  /// before the caller uncloaks it and starts the fade.
  fn finish_commit(&mut self) {
    self.commit_confirmed = true;

    if let Some((w, h)) = self.pending_thumbnail_dims.take() {
      if let Some(surrogate) = &mut self.surrogate {
        surrogate.update_thumbnail_dims(HWND(self.hwnd), w, h, self.border_inset);
      }
    }
    let logical = to_logical(&self.target_rect, &self.border_inset);
    self.current_rect = Some(logical.clone());
    if let Some(surrogate) = &mut self.surrogate {
      // The real window was just resized to the target above, but the live
      // DWM thumbnail still maps the old content dimensions — for the 1–2
      // frames until teardown it would sample a window that no longer
      // matches its registration, producing a visible scale glitch. Update
      // to target dims so the surrogate becomes a pixel-aligned 1:1 mirror
      // of the resized window and the teardown swap is seamless. Single-call
      // update rather than a full re-registration, which can blank the
      // surrogate for a composition frame.
      if surrogate.content_size() != (logical.width(), logical.height()) {
        surrogate.update_thumbnail_dims(
          HWND(self.hwnd),
          logical.width(),
          logical.height(),
          self.border_inset,
        );
      }
      if let Err(err) = surrogate.update(&logical, self.effect_opacity) {
        tracing::warn!("Surrogate pre-commit update failed: {err}.");
      }
    }
  }

  /// Moves the real window to its final target rect and destroys the
  /// surrogate.
  ///
  /// Intended as a cleanup path (e.g. on `WmState::Drop`) to prevent windows
  /// from being left at intermediate animation positions after a crash or
  /// forced exit. Checks `IsWindow` before calling `SetWindowPos` to handle
  /// windows destroyed mid-animation.
  ///
  /// For normal animation completion, `platform_sync` calls
  /// `reposition_window` which handles the full `SetWindowPos` path
  /// including maximize/restore handling; this method is a best-effort
  /// fallback only.
  pub fn commit(mut self) -> crate::Result<()> {
    // Destroy the surrogate before moving the real window so the overlay
    // never outlives the final position update.
    drop(self.surrogate.take());

    if self.hwnd == 0 {
      return Ok(());
    }

    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(HWND(self.hwnd)).as_bool() } {
      return Ok(());
    }

    // SAFETY: `HWND(self.hwnd)` is valid (verified above). With
    // `SWP_NOZORDER` set, `hWndInsertAfter` (`HWND(0)`) is ignored per
    // the Win32 documentation.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        self.target_rect.x(),
        self.target_rect.y(),
        self.target_rect.width(),
        self.target_rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
      )
    }?;

    Ok(())
  }
}

/// Samples the dominant background color near the trailing content edge by
/// `BitBlt`-ing two narrow strips from the already-composited screen.
///
/// Samples 32 evenly-spaced pixels along the right-edge column and 32 along
/// the bottom-edge row, both at `EDGE_SAMPLE_INSET` px inward from the
/// content boundary (matching the edge-extension thumbnail source). Returns
/// `None` when GDI handle creation fails.
///
/// `content_screen_left` and `content_screen_top` are the screen coordinates
/// of the content area's top-left corner (physical rect left/top plus the
/// invisible border insets). Reading from the DWM-composited screen rather
/// than via `PrintWindow` avoids allocating a full-resolution bitmap and
/// forcing a GPU→CPU flush, which is proportional to window area and becomes
/// expensive on large displays.
fn sample_edge_color(
  content_screen_left: i32,
  content_screen_top: i32,
  content_w: i32,
  content_h: i32,
) -> Option<crate::Color> {
  if content_w <= EDGE_SAMPLE_INSET + 1 || content_h <= EDGE_SAMPLE_INSET + 1 {
    return None;
  }

  // Screen coordinates of the right-edge column and bottom-edge row.
  let screen_x = content_screen_left + content_w - EDGE_SAMPLE_INSET - 1;
  let screen_y = content_screen_top + content_h - EDGE_SAMPLE_INSET - 1;

  // Sample the middle half of each edge to avoid corners (rounded-corner
  // antialiasing) and the title-bar region.
  let y0 = content_screen_top + content_h / 4;
  let y1 = content_screen_top + (3 * content_h) / 4;
  let x0 = content_screen_left + content_w / 4;
  let x1 = content_screen_left + (3 * content_w) / 4;

  let strip_h = y1 - y0;
  let strip_w = x1 - x0;
  if strip_h <= 0 || strip_w <= 0 {
    return None;
  }

  // SAFETY: A null HWND argument to GetDC returns the screen DC.
  let hdc_screen = unsafe { GetDC(HWND(0)) };
  if hdc_screen.is_invalid() {
    return None;
  }

  // Right-edge strip: 1 px wide × strip_h px tall.
  // SAFETY: hdc_screen is a valid DC; dimensions are positive.
  let hdc_right = unsafe { CreateCompatibleDC(hdc_screen) };
  let hbm_right = unsafe { CreateCompatibleBitmap(hdc_screen, 1, strip_h) };

  if hdc_right.is_invalid() || hbm_right.is_invalid() {
    unsafe {
      if !hdc_right.is_invalid() {
        DeleteDC(hdc_right);
      }
      if !hbm_right.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(hbm_right.0));
      }
      ReleaseDC(HWND(0), hdc_screen);
    }
    return None;
  }

  // SAFETY: Both handles are valid.
  let old_right = unsafe { SelectObject(hdc_right, HGDIOBJ(hbm_right.0)) };
  // SAFETY: All DCs and dimensions are valid; coordinates are in screen space.
  unsafe { let _ = BitBlt(hdc_right, 0, 0, 1, strip_h, hdc_screen, screen_x, y0, SRCCOPY); }

  // Bottom-edge strip: strip_w px wide × 1 px tall.
  // SAFETY: hdc_screen is a valid DC; dimensions are positive.
  let hdc_bottom = unsafe { CreateCompatibleDC(hdc_screen) };
  let hbm_bottom = unsafe { CreateCompatibleBitmap(hdc_screen, strip_w, 1) };

  if hdc_bottom.is_invalid() || hbm_bottom.is_invalid() {
    unsafe {
      SelectObject(hdc_right, old_right);
      DeleteDC(hdc_right);
      let _ = DeleteObject(HGDIOBJ(hbm_right.0));
      if !hdc_bottom.is_invalid() {
        DeleteDC(hdc_bottom);
      }
      if !hbm_bottom.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(hbm_bottom.0));
      }
      ReleaseDC(HWND(0), hdc_screen);
    }
    return None;
  }

  let old_bottom = unsafe { SelectObject(hdc_bottom, HGDIOBJ(hbm_bottom.0)) };
  // SAFETY: All DCs and dimensions are valid; coordinates are in screen space.
  unsafe { let _ = BitBlt(hdc_bottom, 0, 0, strip_w, 1, hdc_screen, x0, screen_y, SRCCOPY); }

  // SAFETY: The screen DC is no longer needed; hdc_right and hdc_bottom are
  // independent allocations that outlive this scope.
  unsafe { ReleaseDC(HWND(0), hdc_screen) };

  const N: i32 = 32;
  let mut counts: HashMap<u32, u32> = HashMap::with_capacity(64);

  for i in 0..N {
    let sy = (strip_h - 1) * i / (N - 1);
    // SAFETY: hdc_right is a valid DC with hbm_right selected.
    let c = unsafe { GetPixel(hdc_right, 0, sy) };
    if c.0 != 0xFFFF_FFFF {
      *counts.entry(c.0 & 0x00FF_FFFF).or_insert(0) += 1;
    }
  }
  for i in 0..N {
    let sx = (strip_w - 1) * i / (N - 1);
    // SAFETY: hdc_bottom is a valid DC with hbm_bottom selected.
    let c = unsafe { GetPixel(hdc_bottom, sx, 0) };
    if c.0 != 0xFFFF_FFFF {
      *counts.entry(c.0 & 0x00FF_FFFF).or_insert(0) += 1;
    }
  }

  // SAFETY: Restore selections before freeing so GDI holds no references to
  // deleted objects.
  unsafe {
    SelectObject(hdc_right, old_right);
    DeleteDC(hdc_right);
    let _ = DeleteObject(HGDIOBJ(hbm_right.0));
    SelectObject(hdc_bottom, old_bottom);
    DeleteDC(hdc_bottom);
    let _ = DeleteObject(HGDIOBJ(hbm_bottom.0));
  }

  // COLORREF is 0x00BBGGRR.
  counts
    .into_iter()
    .max_by_key(|(_, n)| *n)
    .map(|(colorref, _)| crate::Color {
      r: (colorref & 0xFF) as u8,
      g: ((colorref >> 8) & 0xFF) as u8,
      b: ((colorref >> 16) & 0xFF) as u8,
      a: 255,
    })
}

/// Computes the invisible border insets of `hwnd` in physical pixels.
///
/// Windows adds a transparent resize border (~7 px on left, right, bottom;
/// none on top) outside the visible window frame. Compares `GetWindowRect`
/// with `DWMWA_EXTENDED_FRAME_BOUNDS` to obtain per-side inset values.
///
/// Returns a zeroed `RECT` if either API call fails.
pub(crate) fn compute_border_inset(hwnd: HWND) -> RECT {
  let mut window = RECT::default();
  let mut frame = RECT::default();

  // SAFETY: `hwnd` is a valid window handle. Both output pointers are valid
  // stack-allocated `RECT`s live for the duration of the call.
  let ok = unsafe {
    GetWindowRect(hwnd, std::ptr::from_mut(&mut window).cast()).is_ok()
      && DwmGetWindowAttribute(
        hwnd,
        DWMWA_EXTENDED_FRAME_BOUNDS,
        std::ptr::addr_of_mut!(frame).cast(),
        std::mem::size_of::<RECT>() as u32,
      )
      .is_ok()
  };

  if ok {
    RECT {
      left: (frame.left - window.left).max(0),
      top: (frame.top - window.top).max(0),
      right: (window.right - frame.right).max(0),
      bottom: (window.bottom - frame.bottom).max(0),
    }
  } else {
    RECT::default()
  }
}
