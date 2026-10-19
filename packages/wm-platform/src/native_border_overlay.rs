use windows::Win32::{
  Foundation::{BOOL, HWND},
  Graphics::Gdi::{
    CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject,
    SetWindowRgn, HGDIOBJ, HRGN, RGN_AND, RGN_DIFF,
  },
};

use crate::{
  native_surrogate::is_surrogate,
  overlay_window::{self, Overlay, OverlayKind, OverlayWindow},
  platform_impl::composition::BorderVisual,
  BorderOverlayParams, Color, Rect, StyleBlend, SurrogateBatch,
};

/// Computes the overlay's own rect: `window_rect` outset by `width` on
/// every side.
fn outer_rect(window_rect: &Rect, width: f32) -> Rect {
  #[allow(clippy::cast_possible_truncation)]
  window_rect.inset(-(width.round() as i32))
}

/// Pixels of `outer` lying past each edge of `area` (left, top, right,
/// bottom).
fn insets_outside(outer: &Rect, area: &Rect) -> [i32; 4] {
  [
    (area.left - outer.left).max(0),
    (area.top - outer.top).max(0),
    (outer.right - area.right).max(0),
    (outer.bottom - area.bottom).max(0),
  ]
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
/// For the same reason, the region is cut by `cut` (left, top, right,
/// bottom) like the ring is.
fn apply_hole_region(
  hwnd: HWND,
  outer_size: (i32, i32),
  outset: i32,
  inner_radius: i32,
  cut: [i32; 4],
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

    if cut != [0; 4] {
      let [left, top, right, bottom] = cut;
      let kept_rgn = CreateRectRgn(left, top, w - right, h - bottom);
      if kept_rgn.0 != 0 {
        CombineRgn(outer_rgn, outer_rgn, kept_rgn, RGN_AND);
        let _ = DeleteObject(HGDIOBJ(kept_rgn.0));
      }
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

  /// `(width, height, inner_radius, cut)` of the picture-frame region
  /// last applied, or `None` when the window currently has none -- before
  /// the first application, or for as long as it is behind a surrogate
  /// without `bounds`.
  ///
  /// Skips redundant `SetWindowRgn` calls when a reposition doesn't
  /// change the overlay's shape, e.g. a pure translation. Distinct from
  /// `rect`/`is_visible`'s no-op check: that one skips the whole
  /// `set_rect`/`defer_rect` call including `SetWindowPos`, this one only
  /// skips the (comparatively expensive) region recompute when the
  /// position moved but the shape didn't.
  hole_shape: Option<(i32, i32, i32, [i32; 4])>,

  /// Area the ring is cut to while it tracks its window, e.g. the monitor
  /// of a scrolling layout's column that reaches onto another one.
  bounds: Option<Rect>,

  /// Pixels of the ring's outer rect clipped away on each side (left,
  /// top, right, bottom), cutting it at its `bounds`.
  clip_insets: [i32; 4],

  /// The ring a workspace-switch slide draws instead, kept for the next
  /// slide once one ends. See [`pin_or_slide`].
  ///
  /// [`pin_or_slide`]: NativeBorderOverlay::pin_or_slide
  slide: Option<SlideRing>,

  /// Whether `slide` is shown in place of this ring.
  is_sliding: bool,

  /// Anchor last classified by [`behind_surrogate`], and whether it is a
  /// surrogate, so the class lookup runs once per anchor change.
  ///
  /// [`behind_surrogate`]: NativeBorderOverlay::behind_surrogate
  classified_anchor: (isize, bool),
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
    // Behind a surrogate the outlined window is cloaked and the surrogate,
    // equally unanswering, already covers the hole for point queries, so
    // the region changes nothing there. Reshaping it on every frame of a
    // resize was most of the per-frame overlay cost, so the animation runs
    // without one; the first placement behind anything else restores it.
    // A cut ring keeps it, since past its bounds no surrogate covers it.
    if self.behind_surrogate() && self.bounds.is_none() {
      self.clear_region();
      return;
    }

    #[allow(clippy::cast_possible_truncation)]
    let outset = self.params.width.round() as i32;
    let shape = (
      outer.width(),
      outer.height(),
      inner_hole_radius(&self.params),
      self.bounds_insets(outer),
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
      shape.3,
    );
    self.hole_shape = Some(shape);
  }

  /// Whether the overlay is currently anchored behind a surrogate, i.e.
  /// tracks a window mid-animation.
  fn behind_surrogate(&mut self) -> bool {
    let anchor = self.window.anchor();

    if self.classified_anchor.0 != anchor.0 {
      self.classified_anchor = (anchor.0, is_surrogate(anchor));
    }

    self.classified_anchor.1
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

  /// Resizes the ring to `outer`, cut at its `bounds`.
  fn apply_ring_rect(&mut self, outer: &Rect) {
    if let Err(e) = self.composition.set_rect(outer) {
      tracing::warn!("Border overlay composition resize failed: {e}.");
    }

    self.set_clip_insets(self.bounds_insets(outer));
  }

  /// The overlay's own visual tree and, once a slide has made one, the
  /// slide ring's, which knob changes keep in step.
  fn visuals(&self) -> impl Iterator<Item = &BorderVisual> {
    std::iter::once(&self.composition)
      .chain(self.slide.as_ref().map(|slide| &slide.composition))
  }

  /// Pixels of the ring `outer` past its `bounds` on each side.
  fn bounds_insets(&self, outer: &Rect) -> [i32; 4] {
    self
      .bounds
      .as_ref()
      .map_or([0; 4], |bounds| insets_outside(outer, bounds))
  }

  /// Cuts the ring at the edges of `bounds` while it tracks its window, or
  /// lifts the cut with `None`. A change forces the next placement
  /// through, which applies it.
  fn set_bounds(&mut self, bounds: Option<&Rect>) {
    if self.bounds.as_ref() == bounds {
      return;
    }

    self.bounds = bounds.cloned();
    self.window.mark_stale();
  }

  /// Queues tracking `window_rect` into `batch`, keeping the ring cut at
  /// its `bounds`. See [`Overlay::defer_rect`].
  fn defer_track(
    &mut self,
    batch: &mut SurrogateBatch,
    window_rect: &Rect,
    anchor: HWND,
  ) {
    // A slide is ended by `track`, outside the batch.
    if !self.window.is_placed_behind(anchor) || self.is_sliding {
      self.track(window_rect, anchor);
      return;
    }

    if &self.rect == window_rect {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);
    batch.push(self.window.hwnd().0, outer.clone());

    {
      let _scope = crate::perf::scope(crate::perf::Stage::OverlayVisual);
      self.apply_ring_rect(&outer);
    }

    self.refresh_hole(&outer);

    self.rect = window_rect.clone();
  }

  /// Clips the ring's outer rect by `insets`; re-applies only when they
  /// change.
  fn set_clip_insets(&mut self, insets: [i32; 4]) {
    if self.clip_insets == insets {
      return;
    }

    if let Err(e) = self.composition.set_clip_insets(insets) {
      tracing::warn!("Border overlay composition clip failed: {e}.");
      return;
    }

    self.clip_insets = insets;
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
    self.set_bounds(None);
    self.track(window_rect, anchor);
  }

  /// [`set_rect`](Self::set_rect), keeping the ring cut at its `bounds`.
  fn track(&mut self, window_rect: &Rect, anchor: HWND) {
    if self.is_sliding {
      self.end_slide(window_rect, anchor);
      return;
    }

    if self.window.is_placed_behind(anchor) && &self.rect == window_rect {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);

    // A hidden overlay gets its ring in place *before* the reveal, so the
    // `SetWindowPos` below -- which carries `SWP_SHOWWINDOW` and the new
    // geometry in one window-state update DWM cannot split -- can only
    // ever show a ring already matching it. A visible overlay is merely
    // moving, and is resized after the window so a pure translation costs
    // no ring rebuild.
    let revealing = !self.window.is_visible();
    if revealing {
      self.apply_ring_rect(&outer);
    }

    if let Err(err) = self.window.place(&outer, anchor) {
      tracing::warn!("{err}");
      return;
    }

    if !revealing {
      self.apply_ring_rect(&outer);
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

    for visual in self.visuals() {
      if let Err(e) = visual.set_color(color) {
        tracing::warn!(
          "Border overlay composition color update failed: {e}."
        );
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
  ///
  /// While sliding, the slide ring's viewport-sized window already has
  /// room for any width, so only that ring is rebuilt, and this one is
  /// laid out anew as the slide ends.
  #[allow(clippy::float_cmp)]
  pub fn set_width(&mut self, width: f32) {
    if self.params.width == width {
      return;
    }
    self.params.width = width;

    for visual in self.visuals() {
      if let Err(e) = visual.set_width(width) {
        tracing::warn!(
          "Border overlay composition width update failed: {e}."
        );
      }
    }

    if let Some(slide) = &mut self.slide {
      // The ring's size and offset follow the outset, so an unmoved window
      // is re-slid rather than skipped, now or by the next slide.
      let window_rect =
        std::mem::replace(&mut slide.rect, Rect::from_ltrb(0, 0, 0, 0));
      if self.is_sliding {
        let panel = slide.panel.clone();
        slide.slide(&window_rect, &panel, width);
      }
    }

    if self.is_sliding {
      return;
    }

    let anchor = self.window.anchor();
    let rect = self.rect.clone();
    self.window.mark_stale();
    self.track(&rect, anchor);
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

    for visual in self.visuals() {
      if let Err(e) = visual.set_corner_radius(value) {
        tracing::warn!(
          "Border overlay composition corner-radius update failed: {e}."
        );
      }
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

    for visual in self.visuals() {
      if let Err(e) = visual.set_opacity(value) {
        tracing::warn!(
          "Border overlay composition opacity update failed: {e}."
        );
      }
    }
  }

  /// Draws the ring for `window_rect` with the slide ring, pinned to
  /// `viewport` and cut at `panel`, for one frame of a workspace-switch
  /// slide. Returns `false` when the slide ring couldn't be made.
  ///
  /// The border is a separate `HWND` and DWM thumbnails are captured with
  /// `DWM_TNP_SOURCECLIENTAREAONLY`, so a surrogate can never carry the
  /// ring with it. Moving the overlay window itself every frame would cost
  /// a `SetWindowPos` and a geometry rebuild per window per frame, and
  /// would need the monitor clip applied by hand. The slide ring's window
  /// instead stays still and only its content slides: one property write
  /// per frame, with the clip falling out of composition rendering nothing
  /// outside the target.
  ///
  /// It is shown in place of this ring from the frame after the first,
  /// once its layout has reached DWM. Any ordinary
  /// [`set_rect`]/[`defer_rect`] swaps this ring back in.
  ///
  /// [`set_rect`]: NativeBorderOverlay::set_rect
  /// [`defer_rect`]: NativeBorderOverlay::defer_rect
  pub fn pin_or_slide(
    &mut self,
    viewport: &Rect,
    window_rect: &Rect,
    panel: &Rect,
    anchor: HWND,
  ) -> bool {
    // A slide on another monitor needs a window covering that one.
    if self
      .slide
      .as_ref()
      .is_some_and(|slide| &slide.viewport != viewport)
    {
      self.slide = None;
      self.is_sliding = false;
    }

    let slide = match &mut self.slide {
      Some(slide) => slide,
      slot @ None => {
        match SlideRing::create(viewport, self.params, anchor) {
          Ok(slide) => slot.insert(slide),
          Err(err) => {
            tracing::warn!("Border overlay slide ring failed: {err}");
            return false;
          }
        }
      }
    };

    let is_laid_out = slide.is_laid_out;
    slide.slide(window_rect, panel, self.params.width);
    slide.is_laid_out = true;

    if !self.is_sliding && is_laid_out {
      overlay_window::swap(
        &mut slide.window,
        viewport,
        anchor,
        &mut self.window,
      );
      self.is_sliding = true;
    }

    true
  }

  /// Swaps this ring back in for the slide ring, at `window_rect` behind
  /// `anchor`.
  ///
  /// This ring sat hidden since the slide began at the rect its window had
  /// then, which is where a window lands again unless the layout changed
  /// meanwhile, so the swap shows a ring already in place.
  fn end_slide(&mut self, window_rect: &Rect, anchor: HWND) {
    let outer = outer_rect(window_rect, self.params.width);
    self.apply_ring_rect(&outer);

    match &mut self.slide {
      Some(slide) => {
        overlay_window::swap(
          &mut self.window,
          &outer,
          anchor,
          &mut slide.window,
        );
        slide.is_laid_out = false;
      }
      None => {
        if let Err(err) = self.window.place(&outer, anchor) {
          tracing::warn!("{err}");
        }
      }
    }
    self.is_sliding = false;

    self.refresh_hole(&outer);
    self.rect = window_rect.clone();
  }
}

/// The ring a workspace-switch slide draws in place of its overlay's own:
/// a window of its own pinned to the monitor, with the ring slid inside
/// it by a composition offset.
///
/// Its own window, so the overlay's never has to leave a pin. That took
/// shrinking the window and re-zeroing the ring's offset together, which
/// reach DWM on independent schedules, so the ring was hidden through the
/// change and blinked out as every switch ended. The two windows swap in
/// one `DeferWindowPos` transaction instead.
struct SlideRing {
  /// Declared before `window`, which it is rooted to.
  composition: BorderVisual,
  window: OverlayWindow,

  /// Monitor viewport `window` covers.
  viewport: Rect,

  /// Window rect the ring was last slid to.
  rect: Rect,

  /// Panel the ring was last cut at: its workspace's monitor as moved by
  /// the slide.
  panel: Rect,

  /// Pixels of the ring's outer rect cut away on each side (left, top,
  /// right, bottom) at `panel`.
  clip_insets: [i32; 4],

  /// Whether the ring has been laid out for the slide under way, so its
  /// window may be shown.
  is_laid_out: bool,
}

impl SlideRing {
  /// Creates the ring's window, hidden, over `viewport`.
  fn create(
    viewport: &Rect,
    params: BorderOverlayParams,
    anchor: HWND,
  ) -> crate::Result<Self> {
    let window =
      OverlayWindow::create(OverlayKind::Border, viewport, anchor)?;
    let composition =
      BorderVisual::create(window.hwnd(), viewport, params)?;

    Ok(Self {
      composition,
      window,
      viewport: viewport.clone(),
      rect: Rect::from_ltrb(0, 0, 0, 0),
      panel: viewport.clone(),
      clip_insets: [0; 4],
      is_laid_out: false,
    })
  }

  /// Moves the ring to `window_rect` with a border `width` wide, cut at
  /// `panel`.
  ///
  /// The ring's geometry is only rebuilt when the window's *size* changes
  /// (a zooming slide); a pure translation costs a single property write.
  fn slide(&mut self, window_rect: &Rect, panel: &Rect, width: f32) {
    if &self.rect == window_rect && &self.panel == panel {
      return;
    }

    let _scope = crate::perf::scope(crate::perf::Stage::OverlayVisual);
    let outer = outer_rect(window_rect, width);
    let composition = &self.composition;

    let insets = insets_outside(&outer, panel);
    if insets != self.clip_insets {
      match composition.set_clip_insets(insets) {
        Ok(()) => self.clip_insets = insets,
        Err(e) => {
          tracing::warn!("Border overlay composition clip failed: {e}.");
        }
      }
    }

    if self.rect.width() != window_rect.width()
      || self.rect.height() != window_rect.height()
    {
      if let Err(e) = composition.set_rect(&outer) {
        tracing::warn!("Border overlay composition resize failed: {e}.");
      }
    }

    if let Err(e) = composition.set_offset(
      outer.x() - self.viewport.x(),
      outer.y() - self.viewport.y(),
    ) {
      tracing::warn!("Border overlay composition offset failed: {e}.");
      return;
    }

    self.panel = panel.clone();
    self.rect = window_rect.clone();
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
      bounds: None,
      clip_insets: [0; 4],
      slide: None,
      is_sliding: false,
      classified_anchor: (0, false),
    };
    overlay.refresh_hole(&outer);

    Ok(overlay)
  }

  fn apply_blend(&mut self, blend: StyleBlend<BorderOverlayParams>) {
    let params = blend.from.lerp(&blend.to, blend.t);
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
    self.set_bounds(None);
    self.defer_track(batch, window_rect, anchor);
  }

  /// Cuts the ring itself, rather than outlining the part of the window
  /// inside `clip`, so it ends at the edge like the window does.
  fn defer_clipped_rect(
    &mut self,
    batch: &mut SurrogateBatch,
    window_rect: &Rect,
    clip: &Rect,
    anchor: HWND,
  ) {
    let outer = outer_rect(window_rect, self.params.width);
    if outer.intersection(clip).is_none() {
      self.hide();
      return;
    }

    self.set_bounds(Some(clip));
    self.defer_track(batch, window_rect, anchor);
  }

  fn sync_z_order(
    &mut self,
    anchor: HWND,
    force: bool,
  ) -> crate::Result<()> {
    match &mut self.slide {
      Some(slide) if self.is_sliding => {
        slide.window.sync_z_order(anchor, force)
      }
      _ => self.window.sync_z_order(anchor, force),
    }
  }

  fn is_visible(&self) -> bool {
    self.is_sliding || self.window.is_visible()
  }

  fn hide(&mut self) {
    // The slide is ended too, so a window sliding back into view shows
    // its ring again through a fresh swap.
    if let Some(slide) = &mut self.slide {
      slide.window.hide();
      slide.is_laid_out = false;
    }
    self.is_sliding = false;
    self.window.hide();
  }

  fn hwnd(&self) -> HWND {
    self.window.hwnd()
  }

  fn bounds(window_rect: &Rect, params: &BorderOverlayParams) -> Rect {
    outer_rect(window_rect, params.width)
  }
}
