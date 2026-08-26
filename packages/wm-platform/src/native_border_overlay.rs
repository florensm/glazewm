use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND},
    Graphics::{
      Dwm::DwmExtendFrameIntoClientArea,
      Gdi::{
        CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject,
        HGDIOBJ, RGN_DIFF, SetWindowRgn,
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
  window_class, BorderOverlayParams, Rect, SurrogateBatch,
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

  let ex_style = if composition {
    WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_NOREDIRECTIONBITMAP
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

/// Restricts `hwnd`'s window region to a "picture frame": the full
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

    // `bRedraw = TRUE`: the overlay's own content can change independently
    // of its rect (color/opacity updates), so the newly (dis)covered area
    // must be repainted, unlike the static-snapshot iris overlay.
    SetWindowRgn(hwnd, outer_rgn, BOOL(1));
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
/// width) on every side, and positioned directly behind an `anchor`
/// window in z-order -- same pairing mechanism [`NativeBlurOverlay`] uses.
/// The
/// overlay renders a solid-colored, rounded-rect-clipped sheet across its
/// whole (outset) area (see [`BorderVisual`]'s doc comment for why -- no
/// stroke-shape API is available in this crate's bound
/// `Windows.UI.Composition` surface), but a `SetWindowRgn` "picture frame"
/// region (outer bounds minus `window_rect`) excludes the center from the
/// window's own shape, so only the ring band is ever actually painted --
/// this holds regardless of the tracked window's own opacity or its
/// z-order relative to other overlays (e.g. the acrylic backdrop).
///
/// Renders via a `Windows.UI.Composition` pipeline when available, falling
/// back to a `SetWindowCompositionAttribute` solid-color accent otherwise.
/// In the fallback, `corner_radius` is a no-op (the OS gives no continuous
/// corner-radius knob for SWCA) but `color`/`opacity` keep working.
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

  /// `Some` when the Composition pipeline is active for this overlay;
  /// `None` when running the SWCA fallback.
  composition: Option<BorderVisual>,

  /// `(width, height, inner_radius)` of the hole-punch region last applied
  /// via [`apply_hole_region`], used to skip redundant `SetWindowRgn`
  /// calls when a reposition doesn't actually change the overlay's shape
  /// -- e.g. a pure translation during a workspace-switch slide. Distinct
  /// from `rect`/`is_visible`'s no-op check above: that one skips the
  /// whole `set_rect`/`defer_rect` call including `SetWindowPos`, this one
  /// only skips the (comparatively expensive) region recompute when the
  /// position moved but the shape didn't.
  hole_shape: (i32, i32, i32),
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

    let (hwnd, composition) =
      if let Some((hwnd, visual)) = try_create_composition(&outer, params) {
        (hwnd, Some(visual))
      } else {
        let hwnd = create_window(&outer, false)?;
        extend_glass_sheet(hwnd);
        apply_backdrop(hwnd, Some(&crate::Color::from_abgr(params.color)));
        (hwnd, None)
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

    #[allow(clippy::cast_possible_truncation)]
    let outset = params.width.round() as i32;
    let hole_shape =
      (outer.width(), outer.height(), inner_hole_radius(&params));
    apply_hole_region(hwnd, (hole_shape.0, hole_shape.1), outset, hole_shape.2);

    Ok(Self {
      hwnd: hwnd.0,
      params,
      rect: window_rect.clone(),
      anchor: anchor.0,
      is_visible: true,
      composition,
      hole_shape,
    })
  }

  /// Returns the `HWND` for this overlay.
  fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Re-applies the hole-punch region for `outer` if its shape (size or
  /// inner radius) actually changed since the last application -- skipped
  /// on a pure reposition, since `SetWindowRgn` is comparatively expensive
  /// to call on every animation tick.
  fn refresh_hole(&mut self, outer: &Rect) {
    #[allow(clippy::cast_possible_truncation)]
    let outset = self.params.width.round() as i32;
    let shape = (outer.width(), outer.height(), inner_hole_radius(&self.params));

    if shape == self.hole_shape {
      return;
    }

    apply_hole_region(self.hwnd(), (shape.0, shape.1), outset, shape.2);
    self.hole_shape = shape;
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

    if let Some(composition) = &self.composition {
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
    if !self.is_visible || self.anchor != anchor.0 {
      self.set_rect(window_rect, anchor);
      return;
    }

    if &self.rect == window_rect {
      return;
    }

    let outer = outer_rect(window_rect, self.params.width);
    batch.push(self.hwnd, outer.clone());

    if let Some(composition) = &self.composition {
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
  pub fn set_color(&mut self, color: u32) {
    if self.params.color == color {
      return;
    }
    self.params.color = color;

    match &self.composition {
      Some(composition) => {
        if let Err(e) = composition.set_color(color) {
          tracing::warn!("Border overlay composition color update failed: {e}.");
        }
      }
      None => {
        apply_backdrop(self.hwnd(), Some(&crate::Color::from_abgr(color)));
      }
    }
  }

  /// Updates the border width. Since width determines the overlay's own
  /// outset size (not just a composition property), this repositions/
  /// resizes the window immediately at the last-applied `rect`/`anchor`
  /// rather than deferring to the next `set_rect` call.
  #[allow(clippy::float_cmp)]
  pub fn set_width(&mut self, width: f32) {
    if self.params.width == width {
      return;
    }
    self.params.width = width;
    let anchor = HWND(self.anchor);
    let rect = self.rect.clone();
    self.is_visible = false; // force set_rect through despite unchanged rect.
    self.set_rect(&rect, anchor);
  }

  /// Updates the ring's corner radius; re-applies only when the value
  /// changes. No-op on the composition clip when running the SWCA
  /// fallback (no such knob exists), but the hole-punch is re-applied
  /// regardless, since it must stay concentric with `value` either way.
  #[allow(clippy::float_cmp)]
  pub fn set_corner_radius(&mut self, value: f32) {
    if self.params.corner_radius == value {
      return;
    }
    self.params.corner_radius = value;

    if let Some(composition) = &self.composition {
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

    if let Some(composition) = &self.composition {
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

  /// Hides the overlay without destroying it.
  pub fn hide(&mut self) {
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
    // window it's rooted to.
    self.composition.take();

    // SAFETY: `self.hwnd()` is a valid window handle and `Drop` is called
    // at most once.
    unsafe {
      let _ = DestroyWindow(self.hwnd());
    }
  }
}
