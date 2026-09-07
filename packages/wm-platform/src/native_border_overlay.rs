use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND},
    Graphics::{
      Dwm::DwmExtendFrameIntoClientArea,
      Gdi::{
        CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject,
        HGDIOBJ, HRGN, RGN_DIFF, SetWindowRgn,
      },
    },
    UI::{
      Controls::MARGINS,
      WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetWindow, SetWindowPos, ShowWindow,
        GW_HWNDPREV, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING,
        SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, WS_EX_NOACTIVATE,
        WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
        WS_POPUP,
      },
    },
  },
};

use crate::{
  native_surrogate::apply_backdrop, platform_impl::composition::BorderVisual,
  window_class, BorderOverlayParams, Color, Rect, SurrogateBatch,
};

fn ensure_class_registered() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_BorderOverlay"),
    window_class::default_wnd_proc,
  );
}

/// Creates the overlay's window, outset from `window_rect` by `width` on
/// every side.
///
/// `composition` selects `WS_EX_NOREDIRECTIONBITMAP`, which skips the GDI
/// redirection surface DWM would otherwise allocate -- correct for the
/// `Windows.UI.Composition` path, whose visual tree replaces that surface
/// entirely, but incompatible with the SWCA fallback, which composites into
/// it. Callers falling back from a failed Composition attempt must create a
/// *new* window with `composition: false` rather than reusing one created
/// with the flag set.
fn create_window(outer_rect: &Rect, composition: bool) -> crate::Result<HWND> {
  ensure_class_registered();

  // `WS_EX_TRANSPARENT` on both paths -- see the matching comment in
  // `native_blur_overlay::create_window`. The composition path used to omit
  // it, leaving the (window-outsetting) border overlay hit-testable and
  // therefore showing the busy cursor over every window's border and gap.
  let ex_style = if composition {
    WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT
      | WS_EX_NOREDIRECTIONBITMAP
  } else {
    WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT
  };

  // SAFETY: All parameters are valid. The class is guaranteed registered
  // by `ensure_class_registered`. No parent HWND is needed.
  let hwnd = unsafe {
    CreateWindowExW(
      ex_style,
      w!("GlazeWM_BorderOverlay"),
      w!(""),
      WS_POPUP,
      outer_rect.x(),
      outer_rect.y(),
      outer_rect.width(),
      outer_rect.height(),
      None,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create border overlay window.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// Attempts to build the `Windows.UI.Composition` pipeline for a freshly
/// created overlay window. On any failure, destroys `hwnd` (since it was
/// created with `WS_EX_NOREDIRECTIONBITMAP`, unusable for the SWCA
/// fallback) so the caller can create a fresh window for that path.
fn try_create_composition(
  outer_rect: &Rect,
  params: BorderOverlayParams,
) -> Option<(HWND, BorderVisual)> {
  let hwnd = match create_window(outer_rect, true) {
    Ok(hwnd) => hwnd,
    Err(err) => {
      tracing::warn!(
        "Border overlay composition window creation failed: {err}."
      );
      return None;
    }
  };

  match BorderVisual::create(hwnd, outer_rect, params) {
    Ok(visual) => Some((hwnd, visual)),
    Err(err) => {
      tracing::warn!(
        "Composition border pipeline unavailable, falling back to SWCA: \
         {err}."
      );
      // SAFETY: `hwnd` was just created above and not yet handed to a
      // caller; safe to destroy immediately on this failure path.
      unsafe {
        let _ = DestroyWindow(hwnd);
      }
      None
    }
  }
}

/// Extends the DWM glass sheet over the whole client area, needed by the
/// SWCA fallback so the ring window is transparent outside wherever
/// `apply_backdrop`'s accent tint paints -- the `Windows.UI.Composition`
/// path doesn't need this (`WS_EX_NOREDIRECTIONBITMAP` windows have no GDI
/// backing surface to begin with).
fn extend_glass_sheet(hwnd: HWND) {
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

/// Computes the overlay's own rect: `window_rect` outset by `width` on
/// every side.
fn outer_rect(window_rect: &Rect, width: f32) -> Rect {
  #[allow(clippy::cast_possible_truncation)]
  window_rect.inset(-(width.round() as i32))
}

/// Restricts `hwnd`'s window region to a "picture frame" -- the full
/// `outer_size` rect minus a rect inset by `outset` on every side (in
/// `hwnd`'s own local coordinates), rounded by `inner_radius` so the hole
/// stays concentric with the ring's own rounded outer edge -- a plain
/// rectangular hole would poke past the outer curve at higher radii,
/// showing a gap at the corner instead of a continuous ring. Cuts a real
/// hole for the tracked window to show through, instead of relying on
/// that window's own opacity to occlude the center -- the latter breaks
/// the moment the tracked window isn't fully opaque (e.g. `transparency`
/// enabled), since there'd be nothing left to hide the overlay's own
/// fill.
///
/// Both renderers need the region, for different reasons. SWCA paints a
/// solid accent sheet across the whole overlay and has no stroke primitive,
/// so the ring only exists once the centre is cut away. Composition strokes
/// the ring directly and leaves the interior unpainted, so there the region
/// buys nothing visually -- it is what stops the overlay from answering
/// point queries over the window it outlines. `WS_EX_TRANSPARENT` already
/// excludes it from ordinary mouse routing, but `WindowFromPoint` does not
/// honour that flag, and it does honour the region; without one, anything
/// resolving "the window under the cursor" that way finds a
/// window-plus-gap-sized overlay belonging to a thread that never answers.
///
/// `redraw` should be set only on the SWCA path -- see the call site.
fn apply_hole_region(
  hwnd: HWND,
  outer_size: (i32, i32),
  outset: i32,
  inner_radius: i32,
  redraw: bool,
) {
  let (w, h) = outer_size;

  // SAFETY: `w`/`h` are the overlay window's own (non-negative) client
  // dimensions. `CombineRgn` writes its result into `outer_rgn`;
  // `inner_rgn` is freed immediately after, and `outer_rgn`'s ownership
  // passes to `SetWindowRgn`, which frees it once no longer needed --
  // it must not be deleted here.
  unsafe {
    let outer_rgn = CreateRectRgn(0, 0, w, h);
    if outer_rgn.0 == 0 {
      return;
    }

    let inner_rgn = if inner_radius > 0 {
      CreateRoundRectRgn(
        outset,
        outset,
        w - outset,
        h - outset,
        inner_radius * 2,
        inner_radius * 2,
      )
    } else {
      CreateRectRgn(outset, outset, w - outset, h - outset)
    };
    if inner_rgn.0 != 0 {
      CombineRgn(outer_rgn, outer_rgn, inner_rgn, RGN_DIFF);
      let _ = DeleteObject(HGDIOBJ(inner_rgn.0));
    }

    SetWindowRgn(hwnd, outer_rgn, BOOL(i32::from(redraw)));
  }
}

/// The radius to round the hole-punch's inner edge to, so it stays
/// concentric with the ring's own outer `corner_radius` (itself outset by
/// `width` from the tracked window's edge). Clamped to zero (a square
/// hole) if `width` alone would already exceed the outer radius.
fn inner_hole_radius(params: &BorderOverlayParams) -> i32 {
  #[allow(clippy::cast_possible_truncation)]
  {
    (params.corner_radius - params.width).max(0.0).round() as i32
  }
}

/// How a [`NativeBorderOverlay`] actually paints its ring.
///
/// The two paths need genuinely different machinery, so this keeps the
/// window-region state confined to the one that needs it rather than
/// carrying a permanently-unused field on both.
enum BorderRenderer {
  /// The `Windows.UI.Composition` path: a stroked rounded rectangle whose
  /// interior is simply never painted.
  Composition(BorderVisual),

  /// The `SetWindowCompositionAttribute` fallback: a solid accent sheet
  /// across the whole overlay, whose centre only becomes a ring once
  /// [`apply_hole_region`] cuts it out.
  Swca,
}

/// A persistent overlay window that renders a colored border ring around a
/// paired managed window -- a self-drawn stand-in for the OS's
/// `DWMWA_BORDER_COLOR`, which isn't carried along by DWM thumbnails
/// (surrogates) and so vanishes during window-open/close, resize, and
/// workspace-switch transitions.
///
/// Sized to `window_rect` outset by `params.width` (the configured border
/// width) on every side, and positioned directly behind an `anchor`
/// window in z-order -- same pairing mechanism [`NativeBlurOverlay`] uses.
///
/// Renders via a `Windows.UI.Composition` pipeline when available, falling
/// back to a `SetWindowCompositionAttribute` solid-color accent otherwise.
/// In the fallback, `corner_radius` is a no-op for the fill itself (the OS
/// gives no continuous corner-radius knob for SWCA) but `color`/`opacity`
/// keep working.
///
/// The two paths carve the ring out very differently, which
/// [`BorderRenderer`] captures. Composition strokes a rounded rectangle
/// directly ([`BorderVisual`]), leaving the interior unpainted. SWCA has
/// no stroke primitive, so it paints a full sheet and a `SetWindowRgn`
/// "picture frame" region (outer bounds minus `window_rect`) excludes the
/// center from the window's own shape. Neither path relies on the tracked
/// window occluding a fill, so both hold regardless of that window's own
/// opacity or its z-order relative to other overlays (e.g. the acrylic
/// backdrop).
///
/// [`NativeBlurOverlay`]: crate::NativeBlurOverlay
/// [`BorderVisual`]: crate::platform_impl::composition::BorderVisual
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeBorderOverlay {
  /// Raw window handle stored as `isize` so that `NativeBorderOverlay` is
  /// `Send` even though `HWND` is not.
  hwnd: isize,

  /// Current color/width/corner-radius/opacity.
  params: BorderOverlayParams,

  /// Last *window* rect (not outset) applied via `set_rect`, used to skip
  /// redundant `SetWindowPos` calls when the tracked window hasn't actually
  /// moved.
  rect: Rect,

  /// `HWND` of the window this overlay is positioned directly behind (its
  /// z-order anchor), as raw `isize`. See `NativeBlurOverlay::anchor`'s doc
  /// comment for why anchoring directly behind the managed window (rather
  /// than e.g. the global `HWND_BOTTOM`) matters.
  anchor: isize,

  /// Whether the overlay window is currently shown. See
  /// `NativeBlurOverlay::is_visible`'s doc comment for why this is tracked
  /// explicitly rather than inferred from a rect change.
  is_visible: bool,

  /// Which of the two rendering paths this overlay is running, plus any
  /// state that path alone needs.
  renderer: BorderRenderer,

  /// `(width, height, inner_radius)` of the picture-frame region last
  /// applied, or `None` when the window currently has none -- before the
  /// first application, or for as long as it is pinned.
  ///
  /// Skips redundant `SetWindowRgn` calls when a reposition doesn't change
  /// the overlay's shape, e.g. a pure translation. Distinct from
  /// `rect`/`is_visible`'s no-op check: that one skips the whole
  /// `set_rect`/`defer_rect` call including `SetWindowPos`, this one only
  /// skips the (comparatively expensive) region recompute when the
  /// position moved but the shape didn't.
  hole_shape: Option<(i32, i32, i32)>,

  /// Monitor viewport the overlay window is currently pinned to for a
  /// workspace-switch slide, or `None` in the normal window-tracking mode.
  /// See [`pin_or_slide`].
  ///
  /// [`pin_or_slide`]: NativeBorderOverlay::pin_or_slide
  pinned: Option<Rect>,
}

impl NativeBorderOverlay {
  /// Creates a new border overlay tracking `window_rect`, with the given
  /// `params` (`corner_radius`/`opacity` are only honored when the
  /// Composition pipeline is available).
  ///
  /// The overlay is shown immediately, positioned directly behind `anchor`
  /// (see the `anchor` field doc) -- typically the `HWND` of the managed
  /// window it's tracking, or its surrogate's `HWND` while one is active.
  pub fn create(
    window_rect: &Rect,
    params: BorderOverlayParams,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let outer = outer_rect(window_rect, params.width);

    let (hwnd, renderer) =
      if let Some((hwnd, visual)) = try_create_composition(&outer, params) {
        (hwnd, BorderRenderer::Composition(visual))
      } else {
        let hwnd = create_window(&outer, false)?;
        extend_glass_sheet(hwnd);
        apply_backdrop(hwnd, Some(&params.color));

        (hwnd, BorderRenderer::Swca)
      };

    // SAFETY: `hwnd` is a valid window just created above.
    if let Err(e) = unsafe {
      SetWindowPos(
        hwnd,
        anchor,
        outer.x(),
        outer.y(),
        outer.width(),
        outer.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    } {
      tracing::warn!("Border overlay SetWindowPos failed on create: {e}.");
    }

    let mut overlay = Self {
      hwnd: hwnd.0,
      params,
      rect: window_rect.clone(),
      anchor: anchor.0,
      is_visible: true,
      renderer,
      hole_shape: None,
      pinned: None,
    };
    overlay.refresh_hole(&outer);

    Ok(overlay)
  }

  /// Returns the `HWND` for this overlay.
  fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Re-applies the picture-frame window region for `outer` if its
  /// shape (size or inner radius) actually changed since the last
  /// application -- skipped on a pure reposition, since `SetWindowRgn` is
  /// comparatively expensive to call on every animation tick.
  ///
  /// No-op on the Composition path, which strokes its ring and needs no
  /// window region at all.
  fn refresh_hole(&mut self, outer: &Rect) {
    // A pinned overlay is viewport-sized with its ring drawn at an offset
    // inside it, so `outer` doesn't describe its window at all. `clear_pin`
    // restores the region on the way out.
    if self.pinned.is_some() {
      return;
    }

    #[allow(clippy::cast_possible_truncation)]
    let outset = self.params.width.round() as i32;
    let shape = (outer.width(), outer.height(), inner_hole_radius(&self.params));

    if self.hole_shape == Some(shape) {
      return;
    }

    let _scope = crate::perf::scope(crate::perf::Stage::OverlayRegion);

    // `bRedraw` only on the SWCA path, whose accent brush composites into
    // the window's GDI redirection surface: the newly (dis)covered area
    // genuinely must be repainted there, since that content changes
    // independently of the rect (color/opacity updates). The Composition
    // path has no redirection surface (`WS_EX_NOREDIRECTIONBITMAP`) and
    // its visual tree repaints itself.
    let redraw = matches!(self.renderer, BorderRenderer::Swca);

    apply_hole_region(self.hwnd(), (shape.0, shape.1), outset, shape.2, redraw);
    self.hole_shape = Some(shape);
  }

  /// Drops the window region, leaving the overlay shaped by its bounds
  /// alone. No-op when it has none.
  fn clear_region(&mut self) {
    if self.hole_shape.take().is_none() {
      return;
    }

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct. A null `HRGN` clears the region rather than setting one,
    // so there is nothing to free.
    unsafe {
      SetWindowRgn(self.hwnd(), HRGN(0), BOOL(0));
    }
  }

  /// Returns whether the overlay window is currently shown.
  #[must_use]
  pub fn is_visible(&self) -> bool {
    self.is_visible
  }

  /// Repositions and resizes the overlay to track `window_rect` (outset by
  /// the current border width), keeping it directly behind `anchor`, and
  /// ensures it's shown.
  ///
  /// No-op if neither `window_rect` nor `anchor` changed and the overlay is
  /// already visible -- see `NativeBlurOverlay::set_rect`'s doc comment for
  /// why.
  ///
  /// Callers that only need to correct z-order drift should use
  /// [`sync_z_order`] instead.
  ///
  /// [`sync_z_order`]: NativeBorderOverlay::sync_z_order
  pub fn set_rect(&mut self, window_rect: &Rect, anchor: HWND) {
    // A pinned overlay's `HWND` covers the whole viewport and its ring is
    // positioned by a composition offset, so any normal reposition has to
    // undo both before its own geometry means anything. Doing it here
    // rather than only in `unpin` means no path can strand the pin.
    self.clear_pin();

    if self.is_visible && &self.rect == window_rect && self.anchor == anchor.0
    {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    if let Err(e) = unsafe {
      SetWindowPos(
        self.hwnd(),
        anchor,
        outer.x(),
        outer.y(),
        outer.width(),
        outer.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
      )
    } {
      tracing::warn!("Border overlay SetWindowPos failed: {e}.");
      return;
    }

    if let BorderRenderer::Composition(composition) = &self.renderer {
      if let Err(e) = composition.set_rect(&outer) {
        tracing::warn!("Border overlay composition resize failed: {e}.");
      }
    }

    self.refresh_hole(&outer);

    self.rect = window_rect.clone();
    self.anchor = anchor.0;
    self.is_visible = true;
  }

  /// Queues a reposition into `batch` instead of issuing an immediate
  /// `SetWindowPos` -- see `NativeBlurOverlay::defer_rect`'s doc comment
  /// for the batching rationale. Falls back to [`set_rect`] (immediate,
  /// unbatched) when the overlay isn't currently visible, or when `anchor`
  /// changed.
  ///
  /// [`set_rect`]: NativeBorderOverlay::set_rect
  pub fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    window_rect: &Rect,
    anchor: HWND,
  ) {
    if !self.is_visible || self.anchor != anchor.0 || self.pinned.is_some() {
      self.set_rect(window_rect, anchor);
      return;
    }

    if &self.rect == window_rect {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);
    batch.push(self.hwnd, outer.clone());

    if let BorderRenderer::Composition(composition) = &self.renderer {
      let _scope = crate::perf::scope(crate::perf::Stage::OverlayVisual);
      if let Err(e) = composition.set_rect(&outer) {
        tracing::warn!("Border overlay composition resize failed: {e}.");
      }
    }

    self.refresh_hole(&outer);

    self.rect = window_rect.clone();
  }

  /// Corrects z-order drift by re-positioning the overlay directly behind
  /// `anchor` if it isn't already there, without touching its rect. See
  /// `NativeBlurOverlay::sync_z_order`'s doc comment.
  pub fn sync_z_order(&mut self, anchor: HWND) -> crate::Result<()> {
    // `anchor` is always a real window handle, so the comparison below is
    // meaningful -- but the overlay has to be in the anchor's band first,
    // or the OS will refuse to leave it directly behind a topmost window.
    window_class::match_z_band(self.hwnd(), anchor);

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    let prev = unsafe { GetWindow(self.hwnd(), GW_HWNDPREV) };
    if prev == anchor {
      self.anchor = anchor.0;
      return Ok(());
    }

    // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
    // this struct.
    unsafe {
      SetWindowPos(
        self.hwnd(),
        anchor,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOMOVE | SWP_NOSIZE,
      )
    }?;

    self.anchor = anchor.0;
    Ok(())
  }

  /// Updates the ring's color; re-applies only when the value changes.
  pub fn set_color(&mut self, color: Color) {
    if self.params.color == color {
      return;
    }
    self.params.color = color;

    match &self.renderer {
      BorderRenderer::Composition(composition) => {
        if let Err(e) = composition.set_color(color) {
          tracing::warn!("Border overlay composition color update failed: {e}.");
        }
      }
      BorderRenderer::Swca => {
        apply_backdrop(self.hwnd(), Some(&color));
      }
    }
  }

  /// Updates the border width. Since width determines the overlay's own
  /// outset size (not just a composition property), this repositions/
  /// resizes the window immediately at the last-applied `rect`/`anchor`
  /// rather than deferring to the next `set_rect` call.
  ///
  /// On the Composition path the width is also the ring's stroke
  /// thickness, so the visual is updated first -- the `set_rect` below
  /// then re-derives the stroke's geometry from both at once.
  #[allow(clippy::float_cmp)]
  pub fn set_width(&mut self, width: f32) {
    if self.params.width == width {
      return;
    }
    self.params.width = width;

    if let BorderRenderer::Composition(composition) = &self.renderer {
      if let Err(e) = composition.set_width(width) {
        tracing::warn!("Border overlay composition width update failed: {e}.");
      }
    }

    let anchor = HWND(self.anchor);
    let rect = self.rect.clone();
    self.is_visible = false; // force set_rect through despite unchanged rect.
    self.set_rect(&rect, anchor);
  }

  /// Updates the ring's corner radius; re-applies only when the value
  /// changes. On the SWCA fallback the sheet itself has no radius knob,
  /// but its hole-punch does, and must stay concentric with `value`.
  #[allow(clippy::float_cmp)]
  pub fn set_corner_radius(&mut self, value: f32) {
    if self.params.corner_radius == value {
      return;
    }
    self.params.corner_radius = value;

    if let BorderRenderer::Composition(composition) = &self.renderer {
      if let Err(e) = composition.set_corner_radius(value) {
        tracing::warn!(
          "Border overlay composition corner-radius update failed: {e}."
        );
      }
    }

    let outer = outer_rect(&self.rect, self.params.width);
    self.refresh_hole(&outer);
  }

  /// Updates the overlay's opacity; re-applies only when the value
  /// changes. No-op when running the SWCA fallback (no such knob exists).
  #[allow(clippy::float_cmp)]
  pub fn set_opacity(&mut self, value: f32) {
    if self.params.opacity == value {
      return;
    }
    self.params.opacity = value;

    if let BorderRenderer::Composition(composition) = &self.renderer {
      if let Err(e) = composition.set_opacity(value) {
        tracing::warn!("Border overlay composition opacity update failed: {e}.");
      }
    }
  }

  /// Applies `params`, re-applying only whichever fields actually changed
  /// (each setter no-ops internally on an unchanged value).
  pub fn apply(&mut self, params: BorderOverlayParams) {
    self.set_color(params.color);
    self.set_width(params.width);
    self.set_corner_radius(params.corner_radius);
    self.set_opacity(params.opacity);
  }

  /// Pins the overlay window to `viewport` (when it isn't already) and
  /// draws its ring for `window_rect` as a composition offset within it,
  /// for one frame of a workspace-switch slide.
  ///
  /// The border is a separate `HWND` and DWM thumbnails are captured with
  /// `DWM_TNP_SOURCECLIENTAREAONLY`, so a surrogate can never carry the
  /// ring with it. Moving the overlay window itself every frame would cost
  /// a `SetWindowPos` and a geometry rebuild per window per frame, and
  /// would need the monitor clip applied by hand. Pinning instead leaves
  /// the window still and slides only its content: one property write per
  /// frame, with the clip falling out of composition rendering nothing
  /// outside the target.
  ///
  /// Returns `false` on the SWCA fallback, which has no composition tree to
  /// offset -- callers should hide the overlay for the transition there.
  ///
  /// Undone by any ordinary [`set_rect`]/[`defer_rect`].
  ///
  /// [`set_rect`]: NativeBorderOverlay::set_rect
  /// [`defer_rect`]: NativeBorderOverlay::defer_rect
  pub fn pin_or_slide(
    &mut self,
    viewport: &Rect,
    window_rect: &Rect,
    anchor: HWND,
  ) -> bool {
    if !matches!(self.renderer, BorderRenderer::Composition(_)) {
      return false;
    }

    if self.pinned.is_none() {
      // SAFETY: `self.hwnd()` is a valid window handle for the lifetime of
      // this struct.
      if let Err(e) = unsafe {
        SetWindowPos(
          self.hwnd(),
          anchor,
          viewport.x(),
          viewport.y(),
          viewport.width(),
          viewport.height(),
          SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
        )
      } {
        tracing::warn!("Border overlay viewport pin failed: {e}.");
        return false;
      }

      // The window is about to become viewport-sized with its ring drawn
      // at an offset inside it, so a frame region cut for the window's own
      // rect would clip that ring away. Dropped for the duration of the
      // switch; `clear_pin` puts it back. The overlay is point-query
      // visible in the meantime, which is the same few hundred ms in which
      // the real windows are cloaked behind surrogates anyway.
      self.clear_region();

      self.pinned = Some(viewport.clone());
      self.anchor = anchor.0;
      self.is_visible = true;

      // Force the ring's geometry and offset through: the window just
      // changed size underneath it, so nothing about the previous state
      // still applies.
      self.rect = Rect::from_ltrb(0, 0, 0, 0);
    }

    self.slide(window_rect);

    true
  }

  /// Moves the ring to `window_rect` while pinned, without touching the
  /// overlay window. No-op when not pinned, or when the rect is unchanged.
  ///
  /// The ring's geometry is only rebuilt when the window's *size* changes
  /// (a zooming slide); a pure translation costs a single property write.
  fn slide(&mut self, window_rect: &Rect) {
    let Some(viewport) = self.pinned.clone() else {
      return;
    };

    if &self.rect == window_rect {
      return;
    }

    let BorderRenderer::Composition(composition) = &self.renderer else {
      return;
    };

    let _scope = crate::perf::scope(crate::perf::Stage::OverlayVisual);
    let outer = outer_rect(window_rect, self.params.width);

    if self.rect.width() != window_rect.width()
      || self.rect.height() != window_rect.height()
    {
      if let Err(e) = composition.set_rect(&outer) {
        tracing::warn!("Border overlay composition resize failed: {e}.");
      }
    }

    if let Err(e) =
      composition.set_offset(outer.x() - viewport.x(), outer.y() - viewport.y())
    {
      tracing::warn!("Border overlay composition offset failed: {e}.");
      return;
    }

    self.rect = window_rect.clone();
  }

  /// Drops any viewport pin, returning the ring to its window's own
  /// origin. Leaves the overlay marked not-visible so that the caller's
  /// own reposition is not skipped by `set_rect`'s no-op guard -- while
  /// pinned, `self.rect` describes a ring drawn at an offset inside a
  /// viewport-sized window, which no longer matches the window itself.
  /// That reposition is also what restores the window region dropped at
  /// pin time.
  fn clear_pin(&mut self) {
    if self.pinned.take().is_none() {
      return;
    }

    if let BorderRenderer::Composition(composition) = &self.renderer {
      if let Err(e) = composition.set_offset(0, 0) {
        tracing::warn!("Border overlay composition offset reset failed: {e}.");
      }
    }

    self.is_visible = false;
  }

  /// Hides the overlay without destroying it.
  ///
  /// Drops any viewport pin, so that a window sliding back into view is
  /// re-pinned (and hence re-shown) rather than having its ring moved
  /// inside a still-hidden window.
  pub fn hide(&mut self) {
    self.clear_pin();
    self.is_visible = false;
    // SAFETY: `self.hwnd()` is a valid window handle.
    unsafe {
      let _ = ShowWindow(self.hwnd(), SW_HIDE);
    }
  }
}

impl Drop for NativeBorderOverlay {
  fn drop(&mut self) {
    // Drop the Composition visual tree (if any) before destroying the
    // window it's rooted to -- its `DesktopWindowTarget` is bound to that
    // `HWND`. Swapping in the fallback variant is only a way to move the
    // visual out from behind `&mut self`; nothing reads `renderer` again
    // after this.
    drop(std::mem::replace(&mut self.renderer, BorderRenderer::Swca));

    // SAFETY: `self.hwnd()` is a valid window handle and `Drop` is called
    // at most once.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}
