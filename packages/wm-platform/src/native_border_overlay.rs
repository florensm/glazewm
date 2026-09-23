use windows::Win32::{
  Foundation::{BOOL, HWND},
  Graphics::Gdi::{
    CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject,
    SetWindowRgn, HGDIOBJ, HRGN, RGN_DIFF,
  },
};

use crate::{
  overlay_window::{Overlay, OverlayKind, OverlayWindow},
  platform_impl::composition::BorderVisual,
  BorderOverlayParams, Color, Rect, SurrogateBatch,
};

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
/// The region buys nothing visually -- composition strokes the ring
/// directly and leaves the interior unpainted. It is what stops the
/// overlay from answering point queries over the window it outlines.
/// `WS_EX_TRANSPARENT` already excludes it from ordinary mouse routing,
/// but `WindowFromPoint` does not
/// honour that flag, and it does honour the region; without one, anything
/// resolving "the window under the cursor" that way finds a
/// window-plus-gap-sized overlay belonging to a thread that never answers.
fn apply_hole_region(
  hwnd: HWND,
  outer_size: (i32, i32),
  outset: i32,
  inner_radius: i32,
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

    // No repaint: the composition visual tree has no GDI redirection
    // surface (`WS_EX_NOREDIRECTIONBITMAP`) and repaints itself.
    SetWindowRgn(hwnd, outer_rgn, BOOL(0));
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

/// A persistent overlay window that renders a colored border ring around a
/// paired managed window -- a self-drawn stand-in for the OS's
/// `DWMWA_BORDER_COLOR`, which isn't carried along by DWM thumbnails
/// (surrogates) and so vanishes during window-open/close, resize, and
/// workspace-switch transitions.
///
/// Sized to `window_rect` outset by `params.width` (the configured border
/// width) on every side, and positioned directly behind an `anchor` window
/// in z-order -- same pairing mechanism [`NativeBackdropOverlay`] uses.
///
/// Renders through a `Windows.UI.Composition` pipeline, which strokes a
/// rounded rectangle directly ([`BorderVisual`]) and leaves the interior
/// unpainted. A `SetWindowRgn` "picture frame" region additionally
/// excludes the centre from the window's own shape, so the overlay stays
/// out of `WindowFromPoint`. Nothing relies on the tracked window
/// occluding a fill, so the ring holds regardless of that window's own
/// opacity or its z-order relative to other overlays (e.g. the backdrop).
///
/// [`NativeBackdropOverlay`]: crate::NativeBackdropOverlay
/// [`BorderVisual`]: crate::platform_impl::composition::BorderVisual
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeBorderOverlay {
  /// The overlay's visual tree. Declared before `window`: fields drop in
  /// declaration order, and the tree must go before the `HWND` it is
  /// rooted to.
  composition: BorderVisual,

  window: OverlayWindow,

  /// Current color/width/corner-radius/opacity.
  params: BorderOverlayParams,

  /// Last *window* rect (not outset) applied, used to skip redundant
  /// `SetWindowPos` calls when the tracked window hasn't actually moved.
  rect: Rect,

  /// `(width, height, inner_radius)` of the picture-frame region last
  /// applied, or `None` when the window currently has none -- before the
  /// first application, or for as long as it is pinned.
  ///
  /// Skips redundant `SetWindowRgn` calls when a reposition doesn't
  /// change the overlay's shape, e.g. a pure translation. Distinct from
  /// `rect`/`is_visible`'s no-op check: that one skips the whole
  /// `set_rect`/`defer_rect` call including `SetWindowPos`, this one only
  /// skips the (comparatively expensive) region recompute when the
  /// position moved but the shape didn't.
  hole_shape: Option<(i32, i32, i32)>,

  /// Monitor viewport the overlay window is currently pinned to for a
  /// workspace-switch slide, or `None` in the normal window-tracking
  /// mode. See [`pin_or_slide`].
  ///
  /// [`pin_or_slide`]: NativeBorderOverlay::pin_or_slide
  pinned: Option<Rect>,
}

impl NativeBorderOverlay {
  /// Re-applies the picture-frame window region for `outer` if its
  /// shape (size or inner radius) actually changed since the last
  /// application -- skipped on a pure reposition, since `SetWindowRgn` is
  /// comparatively expensive to call on every animation tick.
  ///
  /// Applied so the overlay stays out of `WindowFromPoint`. See
  /// [`apply_hole_region`].
  fn refresh_hole(&mut self, outer: &Rect) {
    // A pinned overlay is viewport-sized with its ring drawn at an offset
    // inside it, so `outer` doesn't describe its window at all.
    // `clear_pin` restores the region on the way out.
    if self.pinned.is_some() {
      return;
    }

    #[allow(clippy::cast_possible_truncation)]
    let outset = self.params.width.round() as i32;
    let shape = (
      outer.width(),
      outer.height(),
      inner_hole_radius(&self.params),
    );

    if self.hole_shape == Some(shape) {
      return;
    }

    let _scope = crate::perf::scope(crate::perf::Stage::OverlayRegion);

    apply_hole_region(
      self.window.hwnd(),
      (shape.0, shape.1),
      outset,
      shape.2,
    );
    self.hole_shape = Some(shape);
  }

  /// Drops the window region, leaving the overlay shaped by its bounds
  /// alone. No-op when it has none.
  fn clear_region(&mut self) {
    if self.hole_shape.take().is_none() {
      return;
    }

    // SAFETY: The overlay's `HWND` is valid for the lifetime of `self`. A
    // null `HRGN` clears the region rather than setting one, so there is
    // nothing to free.
    unsafe {
      SetWindowRgn(self.window.hwnd(), HRGN(0), BOOL(0));
    }
  }

  /// Resizes the ring to `outer`, and with `reset_offset` also returns it
  /// to its window's own origin.
  ///
  /// Only a call that reveals a hidden overlay passes `reset_offset`: a
  /// pin leaves the offset at its last slid value, and the reveal is the
  /// first moment the window is back to tracking its own rect.
  fn apply_ring_rect(&self, outer: &Rect, reset_offset: bool) {
    let composition = &self.composition;

    if let Err(e) = composition.set_rect(outer) {
      tracing::warn!("Border overlay composition resize failed: {e}.");
    }

    if reset_offset {
      if let Err(e) = composition.set_offset(0, 0) {
        tracing::warn!(
          "Border overlay composition offset reset failed: {e}."
        );
      }
    }
  }

  /// Repositions and resizes the overlay to track `window_rect` (outset by
  /// the current border width), keeping it directly behind `anchor`, and
  /// ensures it's shown.
  ///
  /// No-op if neither `window_rect` nor `anchor` changed and the overlay
  /// is already visible.
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
    let was_pinned = self.pinned.is_some();
    self.clear_pin();

    if self.window.is_placed_behind(anchor) && &self.rect == window_rect {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);

    // Leaving the pin has to shrink the `HWND` and re-zero the ring's
    // offset together, and those commit on independent schedules: Win32
    // geometry reaches DWM by itself, the composition tree through the
    // compositor. Either can land first, so one of the two mixed frames is
    // always reachable, and the bad one -- a still-viewport-sized window
    // holding a ring already back at offset (0, 0) -- draws the ring in
    // the monitor's top-left corner. No call order rules it out (resetting
    // after the `SetWindowPos` was tried, and the offset still won the
    // race), so take the overlay out of composition instead: a hidden
    // overlay composites nothing, whichever side has committed.
    if was_pinned {
      self.window.hide();
    }

    // A hidden overlay gets its ring in place *before* the reveal, so the
    // `SetWindowPos` below -- which carries `SWP_SHOWWINDOW` and the new
    // geometry in one window-state update DWM cannot split -- can only
    // ever show a ring already matching it. A visible overlay is merely
    // moving, and is resized after the window so a pure translation costs
    // no ring rebuild.
    let revealing = !self.window.is_visible();
    if revealing {
      self.apply_ring_rect(&outer, true);
    }

    if let Err(err) = self.window.place(&outer, anchor) {
      tracing::warn!("{err}");
      return;
    }

    if !revealing {
      self.apply_ring_rect(&outer, false);
    }

    self.refresh_hole(&outer);

    self.rect = window_rect.clone();
  }

  /// Updates the ring's color; re-applies only when the value changes.
  pub fn set_color(&mut self, color: Color) {
    if self.params.color == color {
      return;
    }
    self.params.color = color;

    if let Err(e) = self.composition.set_color(color) {
      tracing::warn!(
        "Border overlay composition color update failed: {e}."
      );
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

    if let Err(e) = self.composition.set_width(width) {
      tracing::warn!(
        "Border overlay composition width update failed: {e}."
      );
    }

    let anchor = self.window.anchor();
    let rect = self.rect.clone();
    self.window.mark_stale();
    self.set_rect(&rect, anchor);
  }

  /// Updates the ring's corner radius; re-applies only when the value
  /// changes. The hole-punch radius must stay concentric with `value`, so
  /// this refreshes the window region too.
  #[allow(clippy::float_cmp)]
  pub fn set_corner_radius(&mut self, value: f32) {
    if self.params.corner_radius == value {
      return;
    }
    self.params.corner_radius = value;

    if let Err(e) = self.composition.set_corner_radius(value) {
      tracing::warn!(
        "Border overlay composition corner-radius update failed: {e}."
      );
    }

    let outer = outer_rect(&self.rect, self.params.width);
    self.refresh_hole(&outer);
  }

  /// Updates the overlay's opacity; re-applies only when the value
  /// changes.
  #[allow(clippy::float_cmp)]
  pub fn set_opacity(&mut self, value: f32) {
    if self.params.opacity == value {
      return;
    }
    self.params.opacity = value;

    if let Err(e) = self.composition.set_opacity(value) {
      tracing::warn!(
        "Border overlay composition opacity update failed: {e}."
      );
    }
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
    if self.pinned.is_none() {
      // Growing to the viewport and offsetting the ring inside it is the
      // same two-sided change `set_rect` makes on the way out, and carries
      // the same hazard: composition and Win32 geometry commit on
      // independent schedules, so DWM can catch a frame pairing the
      // viewport-sized window with a ring still at offset (0, 0) -- drawn
      // in the monitor's top-left corner rather than on its window.
      // Ordering the two doesn't rule it out, since either can land first,
      // so the overlay sits the change out hidden: it composites nothing
      // until the `SetWindowPos` below reveals it, and that one call
      // carries `SWP_SHOWWINDOW` and the viewport geometry together.
      self.window.hide();

      // The window is about to become viewport-sized with its ring drawn
      // at an offset inside it, so a frame region cut for the window's own
      // rect would clip that ring away. Dropped for the duration of the
      // switch; `clear_pin` puts it back. The overlay is point-query
      // visible in the meantime, which is the same few hundred ms in which
      // the real windows are cloaked behind surrogates anyway.
      self.clear_region();

      // Force the ring's geometry and offset through: the window is about
      // to change size underneath it, so nothing about the previous state
      // still applies.
      self.pinned = Some(viewport.clone());
      self.rect = Rect::from_ltrb(0, 0, 0, 0);
      self.slide(window_rect);

      if let Err(err) = self.window.place(viewport, anchor) {
        tracing::warn!("Border overlay viewport pin failed: {err}");

        // The ring is offset for a viewport that never arrived. Dropping
        // the pin leaves the overlay hidden, so the next `set_rect`
        // rebuilds the geometry and resets the offset.
        self.pinned = None;
        return false;
      }

      return true;
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

    let composition = &self.composition;
    let _scope = crate::perf::scope(crate::perf::Stage::OverlayVisual);
    let outer = outer_rect(window_rect, self.params.width);

    if self.rect.width() != window_rect.width()
      || self.rect.height() != window_rect.height()
    {
      if let Err(e) = composition.set_rect(&outer) {
        tracing::warn!("Border overlay composition resize failed: {e}.");
      }
    }

    if let Err(e) = composition
      .set_offset(outer.x() - viewport.x(), outer.y() - viewport.y())
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
  ///
  /// Deliberately leaves the composition offset untouched: re-zeroing it
  /// is only safe while the overlay composites nothing, so `set_rect` does
  /// it hidden, between taking the overlay out of composition and the
  /// `SetWindowPos` that reveals it at the window's own rect. `hide` needs
  /// no reset at all, since a hidden overlay composites nothing regardless
  /// of its stale offset.
  fn clear_pin(&mut self) {
    if self.pinned.take().is_none() {
      return;
    }

    self.window.mark_stale();
  }
}

impl Overlay for NativeBorderOverlay {
  type Params = BorderOverlayParams;

  /// There is no non-composition path, matching the backdrop.
  fn create(
    window_rect: &Rect,
    params: BorderOverlayParams,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let outer = outer_rect(window_rect, params.width);

    let mut window =
      OverlayWindow::create(OverlayKind::Border, &outer, anchor)?;
    let composition = BorderVisual::create(window.hwnd(), &outer, params)?;

    if let Err(err) = window.place(&outer, anchor) {
      tracing::warn!("{err}");
    }

    let mut overlay = Self {
      composition,
      window,
      params,
      rect: window_rect.clone(),
      hole_shape: None,
      pinned: None,
    };
    overlay.refresh_hole(&outer);

    Ok(overlay)
  }

  fn apply(&mut self, params: BorderOverlayParams) {
    self.set_color(params.color);
    self.set_width(params.width);
    self.set_corner_radius(params.corner_radius);
    self.set_opacity(params.opacity);
  }

  fn defer_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    window_rect: &Rect,
    anchor: HWND,
  ) {
    // A pinned overlay has to be unpinned by `set_rect` first.
    if !self.window.is_placed_behind(anchor) || self.pinned.is_some() {
      self.set_rect(window_rect, anchor);
      return;
    }

    if &self.rect == window_rect {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);
    batch.push(self.window.hwnd().0, outer.clone());

    {
      let _scope = crate::perf::scope(crate::perf::Stage::OverlayVisual);
      if let Err(e) = self.composition.set_rect(&outer) {
        tracing::warn!("Border overlay composition resize failed: {e}.");
      }
    }

    self.refresh_hole(&outer);

    self.rect = window_rect.clone();
  }

  fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()> {
    self.window.sync_z_order(anchor, force)
  }

  fn is_visible(&self) -> bool {
    self.window.is_visible()
  }

  fn hide(&mut self) {
    // Unpinned so a window sliding back into view is re-pinned (and so
    // re-shown), rather than having its ring moved in a hidden window.
    self.clear_pin();
    self.window.hide();
  }
}
