use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
  UI::WindowsAndMessaging::{
    GetWindowRect, IsWindow, SetWindowPos, SWP_ASYNCWINDOWPOS,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOSENDCHANGING, SWP_NOZORDER,
  },
};

use crate::{
  native_surrogate::to_logical, Color, NativeSurrogate, Rect,
  SurrogateBatch,
};

/// Options for [`ResizeSession::begin`].
pub struct SessionOptions<'a> {
  /// Backdrop color for mixed (one axis grows, one shrinks) animations.
  pub surrogate_color: Option<&'a Color>,
  /// DWM thumbnail opacity (0–255) from the window-effects config.
  pub effect_opacity: u8,
  /// Whether the surrogate is visible immediately after creation.
  pub initially_visible: bool,
  /// When `true`, the thumbnail is scaled to fill the animated rect each frame.
  pub stretch: bool,
  /// When `true`, growing sessions sample old (source-sized) content for
  /// `GROW_REVEAL_GRACE` while the app repaints at the new size.
  pub paint_grace: bool,
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
  /// `true` when no dimension shrinks (target >= source in both width and
  /// height). Curtain-reveal mode.
  ///
  /// Growing sessions use a curtain-reveal: thumbnail registered at target
  /// dimensions; cloaked window pre-positioned asynchronously so DWM captures
  /// correctly-sized content. Mixed/shrinking sessions use clip/wipe: thumbnail
  /// at source dimensions, real window stays at source until `pre_commit`.
  is_growing: bool,
  /// When `true`, each frame animates the DWM thumbnail `rcDestination`
  /// toward/away from the surrogate center instead of repositioning the
  /// surrogate window. Used for zoom-in (open) and zoom-out (close) effects.
  pub zoom: bool,
  /// When `true`, the DWM thumbnail is scaled each frame to fill the whole
  /// animated rect (`ResizeContentMode::Stretch`). The thumbnail stays
  /// registered at source dimensions for the session's lifetime — no
  /// re-registration on redirects, no backdrop exposure, and no
  /// pre-positioning of the real window before the mid-animation handoff.
  stretch: bool,
  /// `true` once a shrinking/mixed reveal session has handed the real
  /// window off to its target rect mid-animation (see [`maybe_handoff`]).
  /// The remaining frames scale the thumbnail like stretch mode, since the
  /// target-sized content can no longer fill the (still larger) animated
  /// rect at its natural size.
  ///
  /// [`maybe_handoff`]: ResizeSession::maybe_handoff
  tail_stretch: bool,
  /// `true` once the real window has been repositioned at the current
  /// `target_rect` (at session start for growing curtain-reveals, or
  /// mid-animation via [`maybe_handoff`]). Reset whenever a redirect changes
  /// the target.
  ///
  /// [`maybe_handoff`]: ResizeSession::maybe_handoff
  handoff_done: bool,
  /// Logical (visible-content) dimensions of the content the thumbnail
  /// currently samples. Matches the thumbnail registration size; updated
  /// when the real window is repositioned mid-session.
  content_size: (i32, i32),
  /// `true` while the thumbnail is registered at source dims during the paint
  /// grace period. Cleared by [`begin_reveal`] once the app has painted.
  ///
  /// [`begin_reveal`]: ResizeSession::begin_reveal
  reveal_pending: bool,
}

impl ResizeSession {
  /// Creates a resize session with a DWM surrogate overlay.
  ///
  /// Growing sessions (no dimension shrinks) use curtain-reveal: thumbnail at
  /// target dims, cloaked window pre-positioned so DWM captures new content.
  /// Shrinking/mixed sessions use clip/wipe: thumbnail at source dims. Stretch
  /// mode scales the thumbnail to fit every frame. When surrogate creation
  /// fails the animation falls back to direct window repositioning.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  pub fn begin(
    hwnd: HWND,
    source_rect: &Rect,
    target_rect: &Rect,
    options: SessionOptions<'_>,
  ) -> crate::Result<Self> {
    let border_inset = compute_border_inset(hwnd);

    let is_growing = target_rect.width() >= source_rect.width()
      && target_rect.height() >= source_rect.height();

    // Paint-grace: register at source dims while the app repaints at new size.
    let reveal_pending =
      is_growing && !options.stretch && options.paint_grace;

    // Growing non-grace: thumbnail at target dims (curtain-reveal).
    // Shrinking/mixed, stretch, or paint-grace: thumbnail at source dims.
    let thumbnail_rect =
      if is_growing && !options.stretch && !reveal_pending {
        target_rect
      } else {
        source_rect
      };

    // Mixed (one axis grows, one shrinks): backdrop prevents desktop bleed-
    // through. Not needed for stretch (full coverage) or growing (no gap).
    let is_mixed = !is_growing
      && (target_rect.width() > source_rect.width()
        || target_rect.height() > source_rect.height());
    let backdrop_color = if is_mixed && !options.stretch {
      options.surrogate_color
    } else {
      None
    };

    let surrogate = match NativeSurrogate::create(
      hwnd,
      source_rect,
      thumbnail_rect,
      backdrop_color,
      options.effect_opacity,
      options.initially_visible,
      border_inset,
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

    let logical_content = to_logical(thumbnail_rect, &border_inset);

    Ok(Self {
      hwnd: hwnd.0,
      target_rect: target_rect.clone(),
      surrogate,
      border_inset,
      effect_opacity: options.effect_opacity,
      is_growing,
      zoom: false,
      stretch: options.stretch,
      tail_stretch: false,
      handoff_done: is_growing && !options.stretch,
      content_size: (logical_content.width(), logical_content.height()),
      reveal_pending,
    })
  }

  /// Returns the final target rect for the real window (physical, including
  /// invisible border).
  #[must_use]
  pub fn target_rect(&self) -> &Rect {
    &self.target_rect
  }

  /// Returns `true` when the cloaked real window should be asynchronously
  /// pre-positioned at the target rect immediately after cloaking.
  ///
  /// This is required for growing curtain-reveal sessions so DWM captures
  /// correctly-sized content. Stretch sessions never pre-position: the
  /// thumbnail is sampled at source dimensions for the whole animation, so
  /// resizing the real window mid-animation would corrupt the sampled
  /// content.
  pub fn needs_preposition(&self) -> bool {
    self.is_growing && !self.stretch
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

  /// Animates the DWM thumbnail `rcDestination` toward/away from center.
  ///
  /// `progress` is the eased animation progress (0.0 = zero-size, 1.0 = full
  /// surrogate). Used for zoom-in (open) and zoom-out (close) effects. The
  /// surrogate window itself stays fixed; only the thumbnail rect animates.
  pub fn update_zoom_fade(&mut self, progress: f32, opacity: u8) {
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
      surrogate.set_thumbnail_rects(
        RECT { left: 0, top: 0, right: w, bottom: h },
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

  /// Whether per-frame thumbnail scaling is active.
  ///
  /// True during stretch mode, the post-handoff tail, or the paint-grace
  /// period when the thumbnail is still registered at source dims.
  fn scales_content(&self) -> bool {
    self.stretch || self.tail_stretch || self.reveal_pending
  }

  /// Switches the thumbnail from source dims to target dims after the paint
  /// grace period, triggering the curtain-reveal of freshly painted content.
  ///
  /// No-op when not in a pending-reveal state or the window has been destroyed.
  pub fn begin_reveal(&mut self) {
    if !self.reveal_pending || self.hwnd == 0 {
      return;
    }
    let logical = to_logical(&self.target_rect, &self.border_inset);
    if let Some(surrogate) = &mut self.surrogate {
      surrogate.reregister_thumbnail(
        HWND(self.hwnd),
        logical.width(),
        logical.height(),
        self.border_inset,
      );
    }
    self.content_size = (logical.width(), logical.height());
    self.reveal_pending = false;
  }

  /// Computes the DWM thumbnail `rcSource`/`rcDestination` pair that scales
  /// the full sampled content to a `dst_width` × `dst_height` destination.
  fn stretch_rects(&self, dst_width: i32, dst_height: i32) -> (RECT, RECT) {
    let (src_w, src_h) = self.content_size;
    (
      RECT {
        left: self.border_inset.left,
        top: self.border_inset.top,
        right: self.border_inset.left + src_w,
        bottom: self.border_inset.top + src_h,
      },
      RECT {
        left: 0,
        top: 0,
        right: dst_width,
        bottom: dst_height,
      },
    )
  }

  /// Hands the real (cloaked) window off to its final target rect
  /// mid-animation.
  ///
  /// The classic end-of-resize flash comes from resizing the real window at
  /// the very end of the animation: the app's first repaint at its new size
  /// (often a white/background-colored frame) lands right as the window is
  /// uncloaked. Calling this while a comfortable slice of the animation
  /// remains lets the app repaint at its final size while still hidden
  /// behind the surrogate, so the uncloak reveals settled content.
  ///
  /// The reposition is posted asynchronously; `pre_commit` issues a final
  /// synchronous move at completion as the correctness guarantee. After the
  /// handoff the thumbnail samples target-sized content, which can no
  /// longer fill a still-larger animated rect at natural size — reveal
  /// sessions therefore switch to scaled rendering (`tail_stretch`) for the
  /// remaining frames.
  ///
  /// No-op for zoom sessions (close animations must never move the real
  /// window — their target rect is off-screen) and when the current target
  /// has already been handed off.
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

    let logical_target = to_logical(&self.target_rect, &self.border_inset);
    self.content_size =
      (logical_target.width(), logical_target.height());
    if !self.stretch {
      self.tail_stretch = true;
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
    let logical = to_logical(current_rect, &self.border_inset);
    let stretch_rects = self
      .scales_content()
      .then(|| self.stretch_rects(logical.width(), logical.height()));

    if let Some(surrogate) = &mut self.surrogate {
      if let Some((rc_src, rc_dst)) = stretch_rects {
        surrogate.set_thumbnail_rects(rc_src, rc_dst);
      }
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
    let logical = to_logical(current_rect, &self.border_inset);
    let stretch_rects = self
      .scales_content()
      .then(|| self.stretch_rects(logical.width(), logical.height()));

    if let Some(surrogate) = &mut self.surrogate {
      if let Some((rc_src, rc_dst)) = stretch_rects {
        surrogate.set_thumbnail_rects(rc_src, rc_dst);
      }
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
      return;
    }

    let src_left = vis_left - logical.x();
    let src_top = vis_top - logical.y();
    let constrained_w = vis_right - vis_left;
    let constrained_h = vis_bottom - vis_top;

    surrogate.set_thumbnail_rects(
      RECT { left: src_left, top: src_top, right: src_left + constrained_w, bottom: src_top + constrained_h },
      RECT { left: 0, top: 0, right: constrained_w, bottom: constrained_h },
    );

    let constrained = Rect::from_xy(vis_left, vis_top, constrained_w, constrained_h);
    if let Err(err) = surrogate.reposition(&constrained) {
      tracing::warn!("Surrogate clipped update failed: {err}.");
    }
    surrogate.set_window_opacity(opacity);
    surrogate.set_visible(true);
  }

  /// Redirects the session to a new target rect while the surrogate is still
  /// active.
  ///
  /// `current_rect` is the current animated position (used to recompute the
  /// grow/shrink direction for the new `start → new_target` span). When the
  /// direction changes, the DWM thumbnail is re-registered at the appropriate
  /// dimensions so the curtain-reveal or clip/wipe renders correctly:
  ///
  /// - Shrinking → growing: registers thumbnail at `new_target` dimensions and
  ///   sends a synchronous `SetWindowPos` to pre-position the cloaked real
  ///   window at the new target so DWM captures the correctly-sized content.
  /// - Growing → shrinking: registers thumbnail at `current_rect` dimensions
  ///   so the clip/wipe effect starts from the correct boundary.
  /// - Same direction: growing updates position and thumbnail; shrinking only
  ///   stores the new target.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  pub fn update_target(&mut self, current_rect: &Rect, new_target: &Rect) {
    let new_is_growing = new_target.width() >= current_rect.width()
      && new_target.height() >= current_rect.height();
    let direction_changed = new_is_growing != self.is_growing;

    self.is_growing = new_is_growing;
    self.target_rect = new_target.clone();

    // Stretch mode never re-registers the thumbnail on a redirect; only the
    // handoff scheduling resets so the window is repositioned at the new
    // target near the end of the redirected animation.
    if self.hwnd == 0 || self.stretch {
      self.handoff_done = false;
      return;
    }

    if new_is_growing {
      // Pre-position the cloaked real window at the new target so DWM captures
      // correctly-sized content.
      //
      // SAFETY: Window is cloaked during an active Frozen animation.
      unsafe {
        let _ = SetWindowPos(
          HWND(self.hwnd),
          HWND(0),
          new_target.x(),
          new_target.y(),
          new_target.width(),
          new_target.height(),
          SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
        );
      }
      // Defer thumbnail re-registration to `begin_reveal` so the curtain-reveal
      // shows freshly painted content rather than blank target-sized frames.
      self.reveal_pending = true;
      self.handoff_done = true;
      self.tail_stretch = false;
    } else if direction_changed {
      // Was growing, now shrinking: register thumbnail at current dims so the
      // clip/wipe starts from the correct boundary.
      let logical = to_logical(current_rect, &self.border_inset);
      if let Some(surrogate) = &mut self.surrogate {
        surrogate.reregister_thumbnail(
          HWND(self.hwnd),
          logical.width(),
          logical.height(),
          self.border_inset,
        );
      }
      self.content_size = (logical.width(), logical.height());
      self.reveal_pending = false;
      self.handoff_done = false;
      self.tail_stretch = false;
    } else {
      // Still shrinking: reset handoff so the window is repositioned near the
      // end of the redirected animation. `tail_stretch` is preserved — scaled
      // rendering stays correct until `maybe_handoff` updates `content_size`.
      self.handoff_done = false;
    }
  }

  /// Snaps the surrogate to the final target rect and synchronously
  /// pre-positions the real window, in preparation for `platform_sync` to
  /// uncloak it.
  ///
  /// Checks `IsWindow` and nullifies the stored handle if the window has been
  /// destroyed mid-animation, so that [`commit`] skips the `SetWindowPos`
  /// call.
  ///
  /// The synchronous `SetWindowPos` here ensures the real window is at
  /// `target_rect` before `set_cloaked(false)` fires, even when the
  /// `SWP_ASYNCWINDOWPOS` call from [`begin`] or [`update_target`] has not
  /// yet been processed by the target window's message queue.
  ///
  /// [`commit`]: ResizeSession::commit
  /// [`begin`]: ResizeSession::begin
  /// [`update_target`]: ResizeSession::update_target
  pub fn pre_commit(&mut self) {
    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(HWND(self.hwnd)).as_bool() } {
      self.hwnd = 0;
      return;
    }

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
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
      );
    }

    let logical = to_logical(&self.target_rect, &self.border_inset);
    let stretch_rects = self
      .scales_content()
      .then(|| self.stretch_rects(logical.width(), logical.height()));

    if let Some(surrogate) = &mut self.surrogate {
      if let Some((rc_src, rc_dst)) = stretch_rects {
        surrogate.set_thumbnail_rects(rc_src, rc_dst);
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

/// Computes the invisible border insets of `hwnd` in physical pixels.
///
/// Windows adds a transparent resize border (~7 px on left, right, bottom;
/// none on top) outside the visible window frame. Compares `GetWindowRect`
/// with `DWMWA_EXTENDED_FRAME_BOUNDS` to obtain per-side inset values.
///
/// Returns a zeroed `RECT` if either API call fails.
fn compute_border_inset(hwnd: HWND) -> RECT {
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
