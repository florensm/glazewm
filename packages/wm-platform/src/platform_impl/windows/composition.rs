//! `Windows.UI.Composition` based pipeline for the overlay-backed backdrop
//! and border, both of which render through a visual tree.
//!
//! The backdrop is a crop of a per-monitor wallpaper image blurred once
//! ahead of time (see `wallpaper_surface`), composited under a tint layer
//! and clipped by a `CompositionRoundedRectangleGeometry` for a continuous
//! corner radius, which SWCA/Mica cannot provide.
//!
//! # Threading
//!
//! A `Compositor` must be created on a thread that owns a dispatcher
//! queue, and (confirmed empirically in the spike, not just per docs) that
//! thread must keep pumping messages for async composition work to ever
//! complete. The wallpaper backdrop's D2D/WIC device stack (see
//! `graphics_device`) is thread-affine besides, and lives on this same
//! thread for that reason. `wm`'s main loop drives everything through
//! `tokio::select!`/ `rt.block_on`, which never pumps Win32 messages, so
//! the entire composition pipeline (the `Compositor` itself, and every
//! per-overlay visual-tree build) is constructed on a dedicated,
//! self-pumping OS thread obtained via
//! `DispatcherQueueController::CreateOnDedicatedThread`.
//!
//! Once created, `Compositor` and every composition object handed back to
//! callers are documented `WinRT` "agile" objects (`windows-rs` applies
//! `unsafe impl Send + Sync` to each of them) -- so the frequent per-tick
//! property updates (`set_rect`, `set_tint`, `set_blur_amount`,
//! `set_corner_radius`, `set_opacity`, `set_saturation`) call directly
//! into them from the caller's thread with no cross-thread marshaling,
//! keeping the per-tick hot path cheap.

use std::{
  cell::{Cell, RefCell},
  sync::{mpsc, OnceLock},
  time::Duration,
};

use windows::{
  core::{h, ComInterface, HSTRING},
  Foundation::Numerics::{Vector2, Vector3},
  System::{
    DispatcherQueue, DispatcherQueueController, DispatcherQueueHandler,
  },
  Win32::{
    Foundation::HWND,
    System::WinRT::Composition::ICompositorDesktopInterop,
  },
  UI::{
    Color,
    Composition::{
      CompositionBrush, CompositionColorBrush,
      CompositionColorGradientStop,
      CompositionColorGradientStopCollection,
      CompositionLinearGradientBrush, CompositionMappingMode,
      CompositionPropertySet, CompositionRadialGradientBrush,
      CompositionRoundedRectangleGeometry, CompositionSpriteShape,
      CompositionSurfaceBrush, Compositor, ContainerVisual,
      Desktop::DesktopWindowTarget, InsetClip, KeyFrameAnimation,
      ScalarKeyFrameAnimation, ShapeVisual, SpriteVisual,
      Vector2KeyFrameAnimation,
    },
  },
};

use super::wallpaper_surface::{self, BakeKnobs};
use crate::{
  particles::{BurstPlan, ParticleStyle, PATH_KEYS},
  BackdropOverlayParams, BorderGlow, BorderOverlayParams, Rect,
};

/// The dedicated, self-pumping composition thread and its `Compositor`.
struct CompositionThread {
  /// Kept alive for the process's lifetime: dropping this tears down the
  /// dedicated thread and its dispatcher queue, which would stall every
  /// composition object's async completions (see the module docs).
  _controller: DispatcherQueueController,
  queue: DispatcherQueue,
  compositor: Compositor,
}

/// Lazily initializes the composition thread on first use, caching failure
/// too (as `None`) so later overlay creations don't retry an unavailable
/// pipeline on every call. On failure no overlay is created at all.
fn composition_thread() -> Option<&'static CompositionThread> {
  static COMPOSITION_THREAD: OnceLock<Option<CompositionThread>> =
    OnceLock::new();

  COMPOSITION_THREAD
    .get_or_init(|| match init_composition_thread() {
      Ok(thread) => Some(thread),
      Err(err) => {
        tracing::warn!(
          "Composition unavailable, skipping the window backdrop: {err}"
        );
        None
      }
    })
    .as_ref()
}

fn init_composition_thread() -> crate::Result<CompositionThread> {
  let controller = DispatcherQueueController::CreateOnDedicatedThread()?;
  let queue = controller.DispatcherQueue()?;
  let compositor = run_on_composition_thread(&queue, Compositor::new)?;

  Ok(CompositionThread {
    _controller: controller,
    queue,
    compositor,
  })
}

/// Runs `f` on the composition thread with that thread's `Compositor` and
/// dispatcher queue, bringing the pipeline up on first use.
///
/// Every entry point into the pipeline needs the same three steps --
/// resolve the thread, clone its agile handles into the closure, dispatch
/// -- so they live here rather than being repeated per visual type.
pub(crate) fn with_composition_thread<T, F>(f: F) -> crate::Result<T>
where
  T: Send + 'static,
  F: FnOnce(Compositor, DispatcherQueue) -> windows::core::Result<T>
    + Send
    + 'static,
{
  let thread = composition_thread().ok_or_else(|| {
    crate::Error::Platform("Composition pipeline unavailable.".to_string())
  })?;

  let compositor = thread.compositor.clone();
  let queue = thread.queue.clone();

  run_on_composition_thread(&thread.queue, move || f(compositor, queue))
}

/// Queues `f` on the composition thread and returns immediately.
///
/// [`run_on_composition_thread`] waits for a result on the WM's own
/// thread, which is right when the caller needs the value -- building a
/// visual tree, say. It is wrong for work whose only effect is on screen a
/// frame or two later, because the wait lands on the main loop: swapping
/// overlays to a different baked surface would block once per window per
/// focus change, felt as the focus ring and backdrop lagging behind the
/// keystroke.
///
/// Nothing observes the result, so failures are logged where they happen
/// rather than returned.
fn dispatch_on_composition_thread<F>(
  queue: &DispatcherQueue,
  f: F,
) -> crate::Result<()>
where
  F: FnOnce() + Send + 'static,
{
  let mut slot = Some(f);

  let handler = DispatcherQueueHandler::new(move || {
    if let Some(f) = slot.take() {
      f();
    }
    Ok(())
  });

  queue.TryEnqueue(&handler)?;
  Ok(())
}

/// Runs `f` on the composition thread via its dispatcher queue and blocks
/// the calling thread for the result.
///
/// Used for the one-time, async-sensitive construction calls
/// (`Compositor::new`, and per-overlay visual-tree building) -- see the
/// module docs for why these specifically must run there.
fn run_on_composition_thread<T, F>(
  queue: &DispatcherQueue,
  f: F,
) -> crate::Result<T>
where
  T: Send + 'static,
  F: FnOnce() -> windows::core::Result<T> + Send + 'static,
{
  let (tx, rx) = mpsc::channel();
  let mut slot = Some(f);

  let handler = DispatcherQueueHandler::new(move || {
    if let Some(f) = slot.take() {
      let _ = tx.send(f());
    }
    Ok(())
  });

  queue.TryEnqueue(&handler)?;

  let result = rx.recv_timeout(Duration::from_secs(5))?;
  Ok(result?)
}

/// Converts our `crate::Color` into a `windows::UI::Color` for Composition
/// brushes.
fn to_ui_color(color: crate::Color) -> Color {
  Color {
    A: color.a,
    B: color.b,
    G: color.g,
    R: color.r,
  }
}

/// One of an overlay's two blur layers: a sprite painting a crop of a
/// monitor's pre-blurred, opaque wallpaper surface, plus the state it
/// keeps to stay live.
///
/// The image is blurred once, ahead of time (see `wallpaper_surface`), so
/// nothing here samples or blurs per frame; what it does have to do is
/// follow the window across monitors.
struct BackdropLayer {
  sprite: SpriteVisual,
  brush: CompositionSurfaceBrush,

  /// Bounds of the monitor whose baked surface `brush` currently points
  /// at. Tracked so the common case -- a window moving within one display
  /// -- is a single offset write, with no monitor lookup and no cache
  /// probe.
  monitor: Rect,

  /// `wallpaper_surface`'s generation counter as of the last bind. A
  /// mismatch means the desktop wallpaper or the display layout changed
  /// under us; see `sync_backdrop`.
  generation: u64,

  /// What the surface `brush` points at was baked with.
  knobs: BakeKnobs,

  /// Last opacity written to `sprite`.
  opacity: f32,
}

impl BackdropLayer {
  /// Must be called on the composition thread.
  fn create(
    compositor: &Compositor,
    rect: &Rect,
    knobs: BakeKnobs,
    parallax: f32,
    opacity: f32,
  ) -> windows::core::Result<Self> {
    let (brush, monitor) =
      wallpaper_surface::crop_brush(compositor, rect, knobs, parallax)?;

    let sprite = compositor.CreateSpriteVisual()?;
    sprite.SetBrush(&brush)?;
    sprite.SetRelativeSizeAdjustment(FILL_PARENT)?;
    sprite.SetOpacity(opacity)?;

    Ok(Self {
      sprite,
      brush,
      monitor,
      generation: wallpaper_surface::generation(),
      knobs,
      opacity,
    })
  }

  /// Points the layer at the surface baked with `knobs`, baking it if no
  /// cached one matches.
  ///
  /// Queued, not awaited: the new surface is on screen a frame or so
  /// later, which beats blocking the main loop on a bake.
  fn rebind(
    &mut self,
    compositor: &Compositor,
    queue: &DispatcherQueue,
    knobs: BakeKnobs,
  ) -> crate::Result<()> {
    let compositor = compositor.clone();
    let brush = self.brush.clone();
    let monitor = self.monitor.clone();

    dispatch_on_composition_thread(queue, move || {
      if let Err(err) =
        wallpaper_surface::rebind(&compositor, &brush, &monitor, knobs)
      {
        tracing::warn!("Wallpaper backdrop re-bake failed: {err}.");
      }
    })?;

    self.knobs = knobs;
    Ok(())
  }

  /// Keeps the layer showing the part of the desktop under its window at
  /// `frame`, from the wallpaper of the monitor the overlay window at
  /// `window` is on. The two differ while the overlay window is cut
  /// short, and only `window` is certain to be on the monitor it shows.
  ///
  /// Re-binds to another monitor's baked surface only when the overlay has
  /// actually crossed onto one -- checked arithmetically against the
  /// cached bounds first, so the per-tick case during an animation costs
  /// one property write and no system calls.
  fn sync_crop(
    &mut self,
    compositor: &Compositor,
    queue: &DispatcherQueue,
    frame: &Rect,
    window: &Rect,
    parallax: f32,
  ) -> crate::Result<()> {
    let current = wallpaper_surface::generation();

    // The generation check has to force a re-bind even when the overlay
    // has not moved: the monitor it sits on is unchanged, but the image
    // baked for that monitor is no longer the one the desktop is showing.
    if self.generation != current
      || !self.monitor.contains_point(&window.center_point())
    {
      self.monitor = wallpaper_surface::monitor_bounds(window);
      self.generation = current;
      self.rebind(compositor, queue, self.knobs)?;
    }

    wallpaper_surface::set_crop(
      &self.brush,
      frame,
      &self.monitor,
      parallax,
    );
    Ok(())
  }

  fn set_opacity(&mut self, value: f32) -> crate::Result<()> {
    #[allow(clippy::float_cmp)]
    if self.opacity == value {
      return Ok(());
    }

    self.sprite.SetOpacity(value)?;
    self.opacity = value;
    Ok(())
  }
}

/// A live `Windows.UI.Composition` visual tree providing an overlay's
/// rendering: two blur layers (see [`BackdropLayer`]) with a tint layer
/// and a vignette composited on top, all clipped to a continuous rounded
/// rectangle.
pub(crate) struct BackdropVisual {
  /// Binds the visual tree to the overlay's `HWND`. Kept alive but never
  /// touched again -- dropping it would unbind composition from the
  /// window.
  _target: DesktopWindowTarget,

  /// Retained (rather than just used during `create`) so the knob setters
  /// can rebuild whatever their layers need rebuilt.
  compositor: Compositor,
  queue: DispatcherQueue,

  /// Placed and sized at `frame` within the overlay window, cut to
  /// `shown` by `cut`, and carrying the overlay's opacity.
  root: ContainerVisual,
  cut: InsetClip,

  /// Fills `root` and holds the layers under the rounded clip.
  body: ContainerVisual,

  /// Two blur layers, so a change of baked knobs can crossfade between
  /// two finished surfaces: the knobs are baked into the image, so there
  /// is no in-between image to animate through, and re-baking per frame
  /// would cost a full-monitor render each time.
  ///
  /// Outside a crossfade one layer is opaque and the other transparent.
  /// The transparent one keeps its surface, so flipping back to the
  /// previous knobs -- e.g. focus returning to a window -- needs no bake
  /// and no re-bind, and lands in the same frame as the tint.
  layers: [BackdropLayer; 2],

  /// Index of the layer composited above the other. A crossfade fades the
  /// upper layer in over an opaque lower one; fading two opaque layers
  /// against each other would let the desktop show through mid-way.
  top: usize,

  tint_brush: CompositionColorBrush,

  /// Darkens the overlay toward its own edges.
  ///
  /// A visual rather than a stage in the wallpaper bake, because the bake
  /// is shared by every window on the monitor: baked in, the falloff
  /// anchors to the screen, so a window at the edge gets a uniformly
  /// dark crop and one in the middle gets the bright centre. Here it is
  /// measured from each window's own rect, which is what a vignette
  /// means.
  ///
  /// Its brush is built once at full strength, and the strength is the
  /// sprite's opacity, so changing it is a property write rather than a
  /// brush rebuild.
  vignette_sprite: SpriteVisual,
  rounded_geometry: CompositionRoundedRectangleGeometry,

  /// How far the wallpaper crop follows the window. Not a baked knob: it
  /// selects a different region of an already-baked surface rather than
  /// changing what was baked, so a change costs one property write.
  parallax: f32,

  /// Last layout applied; see [`set_layout`](Self::set_layout).
  frame: Rect,
  shown: Rect,
  window: Rect,
}

/// Pixels of `frame` outside `shown` on each side, as `InsetClip` insets.
fn cut_insets(frame: &Rect, shown: &Rect) -> [i32; 4] {
  [
    shown.left - frame.left,
    shown.top - frame.top,
    frame.right - shown.right,
    frame.bottom - shown.bottom,
  ]
}

impl BackdropVisual {
  /// Builds a new visual tree for `hwnd`, sized to `rect`, and roots it.
  ///
  /// Runs on the dedicated composition thread (see the module docs); the
  /// returned `BackdropVisual`'s composition objects are agile and can be
  /// mutated from any thread afterwards.
  pub(crate) fn create(
    hwnd: HWND,
    rect: &Rect,
    params: BackdropOverlayParams,
  ) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;
    let rect = rect.clone();

    with_composition_thread(move |compositor, queue| {
      build_visual_tree(&compositor, &queue, HWND(hwnd_raw), &rect, params)
    })
  }

  /// Lays the tree out for a window at `frame`, cut to its part `shown`,
  /// in an overlay window at `window` (which `shown` contains). Does not
  /// reposition the `HWND` itself -- callers still issue their own
  /// `SetWindowPos`.
  ///
  /// The visuals keep the window's own size and shape however little of
  /// it the overlay window shows. Composition and `SetWindowPos` (often
  /// batched to the end of an animation frame) reach DWM on independent
  /// schedules, so sized to the overlay window instead, a window sliding
  /// past an edge it is cut at had its clip shrink a frame before or after
  /// the window did, flashing a strip with no backdrop at that edge. This
  /// way such a slide changes nothing here but the crop, and where the cut
  /// edge is the window's own origin, the offset.
  pub(crate) fn set_layout(
    &mut self,
    frame: &Rect,
    shown: &Rect,
    window: &Rect,
  ) -> crate::Result<()> {
    let offset = |frame: &Rect, window: &Rect| {
      (frame.x() - window.x(), frame.y() - window.y())
    };
    let size = |rect: &Rect| (rect.width(), rect.height());

    let (x, y) = offset(frame, window);
    if (x, y) != offset(&self.frame, &self.window) {
      self.root.SetOffset(Vector3 {
        X: pixels_to_dips(x),
        Y: pixels_to_dips(y),
        Z: 0.0,
      })?;
    }

    let (width, height) = size(frame);
    if (width, height) != size(&self.frame) {
      let dips = Vector2 {
        X: pixels_to_dips(width),
        Y: pixels_to_dips(height),
      };
      self.root.SetSize(dips)?;
      self.rounded_geometry.SetSize(dips)?;
    }

    let insets = cut_insets(frame, shown);
    if insets != cut_insets(&self.frame, &self.shown) {
      let [left, top, right, bottom] = insets;
      self.cut.SetLeftInset(pixels_to_dips(left))?;
      self.cut.SetTopInset(pixels_to_dips(top))?;
      self.cut.SetRightInset(pixels_to_dips(right))?;
      self.cut.SetBottomInset(pixels_to_dips(bottom))?;
    }

    let moved = &self.frame != frame || &self.window != window;
    self.frame = frame.clone();
    self.shown = shown.clone();
    self.window = window.clone();

    if moved {
      self.sync_crop()?;
    }
    Ok(())
  }

  /// Keeps both layers showing the part of the desktop the overlay now
  /// covers.
  ///
  /// The transparent layer is kept current too: it is what a focus change
  /// flips to, and a stale crop would show for the frame it flips.
  fn sync_crop(&mut self) -> crate::Result<()> {
    for layer in &mut self.layers {
      layer.sync_crop(
        &self.compositor,
        &self.queue,
        &self.frame,
        &self.window,
        self.parallax,
      )?;
    }
    Ok(())
  }

  /// Re-binds the wallpaper backdrop when the desktop it was baked from
  /// has changed, and does nothing otherwise.
  ///
  /// Called on every sync tick, so the no-change path is deliberately one
  /// relaxed atomic load and a comparison per layer -- no shell query, no
  /// filesystem stat, and no composition property write.
  pub(crate) fn sync_backdrop(&mut self) -> crate::Result<()> {
    // Throttled internally to one shell query every couple of seconds, so
    // calling it from every overlay on every tick is fine.
    wallpaper_surface::poll_for_changes();

    let current = wallpaper_surface::generation();
    if self.layers.iter().all(|layer| layer.generation == current) {
      return Ok(());
    }

    self.sync_crop()
  }

  /// Updates the tint layer's color.
  pub(crate) fn set_tint(&self, tint: crate::Color) -> crate::Result<()> {
    self.tint_brush.SetColor(to_ui_color(tint))?;
    Ok(())
  }

  /// Shows the blur baked with `from`'s knobs crossfading `t` of the way
  /// into the one baked with `to`'s.
  ///
  /// Settled (`t >= 1.0`, or the two equal) this shows a single layer,
  /// and is the steady-state path too. Each layer keeps whatever it was
  /// last bound to, so a knob set either layer already holds costs only
  /// opacity writes; anything else is baked, or fetched from the surface
  /// cache, asynchronously.
  pub(crate) fn set_bake_blend(
    &mut self,
    from: BakeKnobs,
    to: BakeKnobs,
    t: f32,
  ) -> crate::Result<()> {
    if t >= 1.0 || from == to {
      return self.show_single(if t >= 1.0 { to } else { from });
    }
    if t <= 0.0 {
      return self.show_single(from);
    }

    let base = match (self.layer_with(from), self.layer_with(to)) {
      (Some(base), _) => base,
      // Keep `to` where it already is and re-bind only the other layer.
      (None, Some(fading)) => 1 - fading,
      (None, None) => self.most_visible(),
    };
    let fading = 1 - base;

    if self.layers[base].knobs != from {
      self.layers[base].rebind(&self.compositor, &self.queue, from)?;
    }
    if self.layers[fading].knobs != to {
      self.layers[fading].rebind(&self.compositor, &self.queue, to)?;
    }

    if self.top != fading {
      self.raise(fading)?;
    }

    self.layers[base].set_opacity(1.0)?;
    self.layers[fading].set_opacity(t)
  }

  /// Shows only the layer holding `knobs`, re-binding the visible layer in
  /// place when neither holds them (e.g. after a config reload).
  fn show_single(&mut self, knobs: BakeKnobs) -> crate::Result<()> {
    let shown = if let Some(index) = self.layer_with(knobs) {
      index
    } else {
      let index = self.most_visible();
      self.layers[index].rebind(&self.compositor, &self.queue, knobs)?;
      index
    };

    self.layers[shown].set_opacity(1.0)?;
    self.layers[1 - shown].set_opacity(0.0)
  }

  /// Index of a layer bound to `knobs`, preferring the more visible one.
  fn layer_with(&self, knobs: BakeKnobs) -> Option<usize> {
    let visible = self.most_visible();
    [visible, 1 - visible]
      .into_iter()
      .find(|&index| self.layers[index].knobs == knobs)
  }

  fn most_visible(&self) -> usize {
    usize::from(self.layers[1].opacity > self.layers[0].opacity)
  }

  /// Moves layer `index` directly above the other one, beneath the tint.
  fn raise(&mut self, index: usize) -> crate::Result<()> {
    let children = self.body.Children()?;
    let raised = &self.layers[index].sprite;

    children.Remove(raised)?;
    children.InsertAbove(raised, &self.layers[1 - index].sprite)?;

    self.top = index;
    Ok(())
  }

  /// Updates the vignette's strength, from `0.0` (off) to `1.0`.
  pub(crate) fn set_vignette(&self, value: f32) -> crate::Result<()> {
    self.vignette_sprite.SetOpacity(value.clamp(0.0, 1.0))?;
    Ok(())
  }

  /// Updates how far the crop follows the window, re-applying it so the
  /// change shows without waiting for the window to move.
  pub(crate) fn set_parallax(&mut self, value: f32) {
    self.parallax = value;

    for layer in &self.layers {
      wallpaper_surface::set_crop(
        &layer.brush,
        &self.frame,
        &layer.monitor,
        value,
      );
    }
  }

  /// Updates the clip's corner radius.
  pub(crate) fn set_corner_radius(&self, value: f32) -> crate::Result<()> {
    self
      .rounded_geometry
      .SetCornerRadius(Vector2 { X: value, Y: value })?;
    Ok(())
  }

  /// Updates the overlay's own opacity. `root` sits above every layer, so
  /// this fades the whole composited overlay (blur + tint together) as
  /// one unit -- a plain `Visual` property, so it never needs a brush
  /// rebuild.
  pub(crate) fn set_opacity(&self, value: f32) -> crate::Result<()> {
    self.root.SetOpacity(value)?;
    Ok(())
  }
}

/// Builds the radial gradient that darkens an overlay toward its edges, at
/// full strength -- the vignette sprite's opacity scales it down.
///
/// Transparent across the middle and opaque black at the corners. The
/// ellipse is deliberately larger than the sprite (`radius > 0.5` in
/// relative units) so the darkest point falls outside the visible area: a
/// gradient that reached full strength exactly at the edge puts its
/// steepest part on screen and reads as a ring rather than shading.
fn build_vignette_brush(
  compositor: &Compositor,
) -> windows::core::Result<CompositionRadialGradientBrush> {
  let brush = compositor.CreateRadialGradientBrush()?;

  // Relative to the sprite, so resizing the overlay needs no update here.
  brush.SetMappingMode(CompositionMappingMode::Relative)?;
  brush.SetEllipseCenter(Vector2 { X: 0.5, Y: 0.5 })?;
  brush.SetEllipseRadius(Vector2 { X: 0.75, Y: 0.75 })?;

  let clear = Color {
    A: 0,
    R: 0,
    G: 0,
    B: 0,
  };
  let dark = Color {
    A: u8::MAX,
    R: 0,
    G: 0,
    B: 0,
  };

  let stops = brush.ColorStops()?;
  stops.Append(
    &compositor.CreateColorGradientStopWithOffsetAndColor(0.0, clear)?,
  )?;
  stops.Append(
    &compositor.CreateColorGradientStopWithOffsetAndColor(0.45, clear)?,
  )?;
  stops.Append(
    &compositor.CreateColorGradientStopWithOffsetAndColor(1.0, dark)?,
  )?;

  Ok(brush)
}

/// `DesktopWindowTarget` sizes composition visuals 1:1 against the HWND's
/// actual client pixel size (no DPI virtualization layer here, unlike
/// XAML/UWP) -- so this is a passthrough today. Named/kept separate from a
/// bare cast so a future DPI-aware sizing adjustment has a single call
/// site.
#[allow(clippy::cast_precision_loss, clippy::unnecessary_wraps)]
fn pixels_to_dips(pixels: i32) -> f32 {
  pixels as f32
}

/// `RelativeSizeAdjustment` making a visual track its parent's size (or,
/// for a target's root, the `HWND`'s) with no explicit size writes.
const FILL_PARENT: Vector2 = Vector2 { X: 1.0, Y: 1.0 };

/// Builds the full visual tree: a root sized to `rect`, filled by a body
/// holding the two [`BackdropLayer`] sprites, a tint sprite (flat color)
/// and the vignette stacked above them, all clipped by a shared rounded
/// rectangle geometry. The root's inset clip starts out cutting nothing.
fn build_visual_tree(
  compositor: &Compositor,
  queue: &DispatcherQueue,
  hwnd: HWND,
  rect: &Rect,
  params: BackdropOverlayParams,
) -> windows::core::Result<BackdropVisual> {
  // SAFETY: `hwnd` is a valid, already-created top-level window.
  let target = unsafe {
    compositor
      .cast::<ICompositorDesktopInterop>()?
      .CreateDesktopWindowTarget(hwnd, false)?
  };

  let rounded_geometry = compositor.CreateRoundedRectangleGeometry()?;
  rounded_geometry.SetSize(Vector2 {
    X: pixels_to_dips(rect.width()),
    Y: pixels_to_dips(rect.height()),
  })?;
  rounded_geometry.SetCornerRadius(Vector2 {
    X: params.corner_radius,
    Y: params.corner_radius,
  })?;
  let clip =
    compositor.CreateGeometricClipWithGeometry(&rounded_geometry)?;

  // Both layers start on the same surface: the second costs a brush and a
  // sprite, not a bake, until a crossfade first binds it elsewhere.
  let knobs = BakeKnobs::from(params);
  let layers = [
    BackdropLayer::create(compositor, rect, knobs, params.parallax, 1.0)?,
    BackdropLayer::create(compositor, rect, knobs, params.parallax, 0.0)?,
  ];

  let tint_brush =
    compositor.CreateColorBrushWithColor(to_ui_color(params.tint))?;
  let tint_sprite = compositor.CreateSpriteVisual()?;
  tint_sprite.SetBrush(&tint_brush)?;

  let vignette_sprite = compositor.CreateSpriteVisual()?;
  vignette_sprite.SetBrush(&build_vignette_brush(compositor)?)?;
  vignette_sprite.SetOpacity(params.vignette.clamp(0.0, 1.0))?;

  let body = compositor.CreateContainerVisual()?;
  body.SetRelativeSizeAdjustment(FILL_PARENT)?;
  body.SetClip(&clip)?;

  let children = body.Children()?;
  for layer in &layers {
    children.InsertAtTop(&layer.sprite)?;
  }
  for sprite in [&tint_sprite, &vignette_sprite] {
    sprite.SetRelativeSizeAdjustment(FILL_PARENT)?;
    children.InsertAtTop(sprite)?;
  }

  let cut = compositor.CreateInsetClip()?;
  let root = compositor.CreateContainerVisual()?;
  root.SetSize(Vector2 {
    X: pixels_to_dips(rect.width()),
    Y: pixels_to_dips(rect.height()),
  })?;
  root.SetClip(&cut)?;
  root.SetOpacity(params.opacity)?;
  root.Children()?.InsertAtTop(&body)?;

  target.SetRoot(&root)?;

  Ok(BackdropVisual {
    _target: target,
    compositor: compositor.clone(),
    queue: queue.clone(),
    root,
    cut,
    body,
    layers,
    top: 1,
    tint_brush,
    vignette_sprite,
    rounded_geometry,
    parallax: params.parallax,
    frame: rect.clone(),
    shown: rect.clone(),
    window: rect.clone(),
  })
}

/// A surrogate's gap fill: a solid color painted over the part of the
/// surrogate its DWM thumbnail does not cover, standing in for window
/// content the thumbnail has not caught up to yet mid-resize.
///
/// Rooted on the surrogate's own `HWND`, underneath the thumbnail (DWM
/// draws thumbnails above a non-topmost `DesktopWindowTarget`), so it
/// moves with the surrogate atomically and never tints the content above
/// it.
///
/// The uncovered area is an L -- the thumbnail is anchored top-left -- so
/// it is two sprites: a full-height strip right of the covered width, and
/// a bottom strip under it only as wide as the covered width, so the two
/// never overlap and double-composite the corner. Both size themselves
/// relative to the window, offset by the covered size: a per-frame resize
/// of the surrogate needs no composition write at all, and a strip whose
/// relative size goes negative (the surrogate narrower than the thumbnail)
/// simply renders nothing.
pub(crate) struct SurrogateFill {
  /// Binds the visual tree to the surrogate's `HWND`. Kept alive but
  /// never touched again -- dropping it would unbind composition from
  /// the window.
  _target: DesktopWindowTarget,
  root: ContainerVisual,
  brush: CompositionColorBrush,
  right: SpriteVisual,
  bottom: SpriteVisual,
}

impl SurrogateFill {
  /// Builds a hidden fill rooted on `hwnd`, which must have been created
  /// with `WS_EX_NOREDIRECTIONBITMAP`.
  pub(crate) fn create(hwnd: HWND) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;

    with_composition_thread(move |compositor, _| {
      // SAFETY: `hwnd` is a valid, already-created top-level window.
      let target = unsafe {
        compositor
          .cast::<ICompositorDesktopInterop>()?
          .CreateDesktopWindowTarget(HWND(hwnd_raw), false)?
      };

      let brush = compositor.CreateColorBrush()?;
      let right = compositor.CreateSpriteVisual()?;
      right.SetBrush(&brush)?;
      right.SetRelativeSizeAdjustment(FILL_PARENT)?;
      let bottom = compositor.CreateSpriteVisual()?;
      bottom.SetBrush(&brush)?;
      bottom.SetRelativeSizeAdjustment(Vector2 { X: 0.0, Y: 1.0 })?;

      let root = compositor.CreateContainerVisual()?;
      root.SetRelativeSizeAdjustment(FILL_PARENT)?;
      root.SetIsVisible(false)?;
      root.Children()?.InsertAtTop(&right)?;
      root.Children()?.InsertAtTop(&bottom)?;
      target.SetRoot(&root)?;

      Ok(Self {
        _target: target,
        root,
        brush,
        right,
        bottom,
      })
    })
  }

  /// Shows the fill in `color`, or hides it when `None`.
  pub(crate) fn set_color(
    &self,
    color: Option<crate::Color>,
  ) -> crate::Result<()> {
    if let Some(color) = color {
      self.brush.SetColor(to_ui_color(color))?;
    }
    self.root.SetIsVisible(color.is_some())?;
    Ok(())
  }

  /// Sets the thumbnail's covered size, in physical pixels from the
  /// surrogate's top-left.
  pub(crate) fn set_covered(
    &self,
    covered: (i32, i32),
  ) -> crate::Result<()> {
    let (width, height) =
      (pixels_to_dips(covered.0), pixels_to_dips(covered.1));

    self.right.SetOffset(Vector3 {
      X: width,
      Y: 0.0,
      Z: 0.0,
    })?;
    self.right.SetSize(Vector2 { X: -width, Y: 0.0 })?;
    self.bottom.SetOffset(Vector3 {
      X: 0.0,
      Y: height,
      Z: 0.0,
    })?;
    self.bottom.SetSize(Vector2 {
      X: width,
      Y: -height,
    })?;
    Ok(())
  }

  /// Sets the fill's opacity, which tracks the thumbnail's so the fill
  /// fades with the content it stands in for.
  pub(crate) fn set_opacity(&self, opacity: f32) -> crate::Result<()> {
    self.root.SetOpacity(opacity)?;
    Ok(())
  }
}

/// A live `Windows.UI.Composition` visual tree providing a border
/// overlay's rendering: a rounded rectangle *stroked* with a solid color,
/// so only the ring band is ever painted and the interior stays fully
/// transparent, plus a hairline along its inner edge and the bands of its
/// glow. Considerably lighter than [`BackdropVisual`] -- no effect graph,
/// no live backdrop sampling, just stroked shapes.
///
/// `NativeBorderOverlay` sizes and positions the overlay's `HWND` to the
/// tracked window's rect *outset* by the configured border width and glow
/// reach, directly behind the real window in z-order (same
/// `OverlayWindow` pairing as [`BackdropVisual`]'s). The stroke is that
/// border width thick and its geometry is inset by half of it past the
/// glow, so the ring's inner edge lands exactly on the tracked window's
/// own rect.
///
/// A stroke rounds the ring's inner *and* outer corners exactly, which a
/// fill clipped by a `CreateRoundRectRgn` hole only approximated. The
/// window region `NativeBorderOverlay` still sets is for hit-testing only
/// (see its `apply_hole_region`), and is skipped on pure translations.
pub(crate) struct BorderVisual {
  /// Binds the visual tree to the overlay's `HWND`. Kept alive but never
  /// touched again -- dropping it would unbind composition from the
  /// window.
  _target: DesktopWindowTarget,

  /// Root of the tree, holding the single stroked shape. A `ShapeVisual`
  /// derives `ContainerVisual`, so this doubles as the size/opacity knob
  /// the previous design needed a separate `ContainerVisual` for.
  root: ShapeVisual,
  shape: CompositionSpriteShape,
  /// Runs from the ring's top edge, in its lit color, down to
  /// [`HIGHLIGHT_FALLOFF`] of its height, in its color, which it keeps
  /// below.
  stroke_brush: CompositionLinearGradientBrush,
  /// The lit and the plain color's stops, in that order.
  stroke_stops: [CompositionColorGradientStop; 2],
  geometry: CompositionRoundedRectangleGeometry,
  /// A hairline along the ring's inner edge (see
  /// `BorderOverlayParams::rim`), over the ring.
  rim: CompositionSpriteShape,
  rim_brush: CompositionColorBrush,
  rim_geometry: CompositionRoundedRectangleGeometry,
  /// The glow's bands, outward from the ring (see `BorderGlow`). Made as
  /// a glow first needs them and kept, drawing nothing, past its reach.
  glow: RefCell<Vec<GlowBand>>,
  /// Cuts the ring short of its own edges. Zero except while the ring
  /// slides with a workspace switch.
  clip: InsetClip,

  /// Last-applied ring inputs, so any one of `set_rect`/`set_widths`/
  /// `set_corner_radius` can recompute the derived geometry (which
  /// depends on all of them) from the others' current values.
  ring: Cell<Ring>,
}

/// One pixel-wide band of a border's glow: a stroke around the ring, as
/// many pixels out as its index.
struct GlowBand {
  shape: CompositionSpriteShape,
  brush: CompositionColorBrush,
  geometry: CompositionRoundedRectangleGeometry,
}

impl GlowBand {
  /// Creates `count` bands, drawing nothing until laid out and colored.
  fn create_all(
    compositor: &Compositor,
    count: usize,
  ) -> windows::core::Result<Vec<Self>> {
    (0..count)
      .map(|_| {
        let geometry = compositor.CreateRoundedRectangleGeometry()?;
        let brush = compositor.CreateColorBrushWithColor(Color {
          A: 0,
          R: 0,
          G: 0,
          B: 0,
        })?;
        let shape = compositor.CreateSpriteShapeWithGeometry(&geometry)?;
        shape.SetStrokeBrush(&brush)?;
        shape.SetStrokeThickness(0.0)?;

        Ok(Self {
          shape,
          brush,
          geometry,
        })
      })
      .collect()
  }
}

/// How far down a ring, as a fraction of its height, its highlight fades
/// out (see `BorderOverlayParams::highlight`).
const HIGHLIGHT_FALLOFF: f32 = 0.35;

/// Thickness of the hairline along a ring's inner edge, or of the whole
/// ring if thinner (see `BorderOverlayParams::rim`).
const RIM_WIDTH: f32 = 1.0;

/// The inputs a stroked ring's geometry is derived from: the overlay's own
/// (already-outset) size in pixels, the border width the stroke is drawn
/// at, the ring's *outer* corner radius, and how far its glow reaches
/// past it, which the ring is inset by.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ring {
  size: Vector2,
  width: f32,
  corner_radius: f32,
  glow_reach: f32,
}

/// The rounded rectangle a stroke follows -- its offset, size and corner
/// radius -- inset by `inset` from an overlay of `size`, clamped so it
/// never goes negative, which composition rejects.
fn inset_path(
  size: Vector2,
  inset: f32,
  corner_radius: f32,
) -> (Vector2, Vector2, f32) {
  (
    Vector2 { X: inset, Y: inset },
    Vector2 {
      X: (size.X - 2.0 * inset).max(0.0),
      Y: (size.Y - 2.0 * inset).max(0.0),
    },
    corner_radius.max(0.0),
  )
}

/// Applies `path` (see [`inset_path`]) to `geometry`.
fn set_path(
  geometry: &CompositionRoundedRectangleGeometry,
  (offset, size, corner_radius): (Vector2, Vector2, f32),
) -> windows::core::Result<()> {
  geometry.SetOffset(offset)?;
  geometry.SetSize(size)?;
  geometry.SetCornerRadius(Vector2 {
    X: corner_radius,
    Y: corner_radius,
  })
}

/// Derives a [`Ring`]'s centerline geometry, i.e. the rounded rectangle a
/// stroke of `ring.width` must follow for the resulting band to span
/// exactly from the overlay's outer rect inwards to the tracked window's
/// own rect.
///
/// A composition stroke straddles its geometry, half of it on either side,
/// so the path is inset by half the width and its radius shrunk by the
/// same amount -- leaving the band's outer edge at `ring.corner_radius`
/// and its inner edge at `ring.corner_radius - ring.width`, both exact
/// curves rather than the old hole punch's `CreateRoundRectRgn`
/// approximation.
///
/// Returns `(offset, size, corner_radius)` for the geometry, inset by the
/// glow's reach too, each clamped so a width exceeding the overlay's own
/// size (or its corner radius) degenerates gracefully instead of
/// producing a negative extent, which composition rejects.
fn ring_geometry(ring: Ring) -> (Vector2, Vector2, f32) {
  let half = ring.width.max(0.0) / 2.0;

  inset_path(
    ring.size,
    ring.glow_reach.max(0.0) + half,
    ring.corner_radius - half,
  )
}

/// The path and thickness of the hairline along `ring`'s inner edge: the
/// innermost [`RIM_WIDTH`] of the ring's band.
fn rim_geometry(ring: Ring) -> ((Vector2, Vector2, f32), f32) {
  let width = ring.width.max(0.0);
  let thickness = RIM_WIDTH.min(width);
  // From the ring's outer edge to the hairline's centerline.
  let depth = width - thickness / 2.0;

  (
    inset_path(
      ring.size,
      ring.glow_reach.max(0.0) + depth,
      ring.corner_radius - depth,
    ),
    thickness,
  )
}

/// The path of the glow's `band`-th band out from `ring`, concentric with
/// it, so a square ring's glow rounds off at the corners like any glow.
#[allow(clippy::cast_precision_loss)]
fn glow_band_geometry(ring: Ring, band: usize) -> (Vector2, Vector2, f32) {
  let distance = band as f32 + 0.5;

  inset_path(
    ring.size,
    ring.glow_reach - distance,
    ring.corner_radius + distance,
  )
}

/// Number of bands a glow reaching `reach` past its ring is drawn in.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn glow_bands(reach: f32) -> usize {
  reach.round().max(0.0) as usize
}

impl BorderVisual {
  /// Builds a new visual tree for `hwnd`, sized to `rect` (the overlay's
  /// own, already-outset rect -- see the type doc), and roots it.
  ///
  /// Runs on the dedicated composition thread (see the module docs); the
  /// returned `BorderVisual`'s composition objects are agile and can be
  /// mutated from any thread afterwards.
  pub(crate) fn create(
    hwnd: HWND,
    rect: &Rect,
    params: BorderOverlayParams,
  ) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;
    let rect = rect.clone();

    with_composition_thread(move |compositor, _| {
      build_border_visual_tree(&compositor, HWND(hwnd_raw), &rect, params)
    })
  }

  /// Re-derives and applies the stroke thicknesses and geometry for
  /// `ring`, storing it as the new baseline for the next partial update.
  ///
  /// Lays out the glow bands made so far, leaving any more it reaches to
  /// [`grow_glow`](Self::grow_glow).
  fn apply_ring(&self, ring: Ring) -> windows::core::Result<()> {
    let reach = ring.glow_reach.max(0.0);

    self.root.SetSize(ring.size)?;
    self
      .stroke_brush
      .SetStartPoint(Vector2 { X: 0.0, Y: reach })?;
    self.stroke_brush.SetEndPoint(Vector2 {
      X: 0.0,
      Y: reach + (ring.size.Y - 2.0 * reach).max(0.0) * HIGHLIGHT_FALLOFF,
    })?;
    self.shape.SetStrokeThickness(ring.width.max(0.0))?;
    set_path(&self.geometry, ring_geometry(ring))?;

    let (rim_path, rim_thickness) = rim_geometry(ring);
    self.rim.SetStrokeThickness(rim_thickness)?;
    set_path(&self.rim_geometry, rim_path)?;

    // Bands past the reach are only touched as it shrinks past them.
    let (bands, drawn) = (
      glow_bands(ring.glow_reach),
      glow_bands(self.ring.get().glow_reach),
    );
    for (index, band) in self.glow.borrow().iter().enumerate() {
      if index < bands {
        band.shape.SetStrokeThickness(1.0)?;
        set_path(&band.geometry, glow_band_geometry(ring, index))?;
      } else if index < drawn {
        band.shape.SetStrokeThickness(0.0)?;
      }
    }

    self.ring.set(ring);
    Ok(())
  }

  /// Makes sure the glow has `bands` bands to draw, making the missing
  /// ones on the composition thread. They are kept, so this waits on that
  /// thread at most as often as a window's glow first reaches further.
  fn grow_glow(&self, bands: usize) -> crate::Result<()> {
    let missing = bands.saturating_sub(self.glow.borrow().len());
    if missing == 0 {
      return Ok(());
    }

    let made = with_composition_thread(move |compositor, _| {
      GlowBand::create_all(&compositor, missing)
    })?;

    let shapes = self.root.Shapes()?;
    for band in &made {
      shapes.Append(&band.shape)?;
    }
    self.glow.borrow_mut().extend(made);
    Ok(())
  }

  /// Colors the glow's bands, outward from the ring, `color` at each of
  /// `opacities`, leaving those past them transparent.
  fn color_glow(
    &self,
    color: crate::Color,
    opacities: &[f32],
  ) -> windows::core::Result<()> {
    for (index, band) in self.glow.borrow().iter().enumerate() {
      let opacity = opacities.get(index).copied().unwrap_or(0.0);
      #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
      let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;

      band
        .brush
        .SetColor(to_ui_color(crate::Color { a: alpha, ..color }))?;
    }
    Ok(())
  }

  /// Colors the glow `color`, each band at its one of `opacities` (see
  /// `BorderGlow::band_opacities`).
  pub(crate) fn set_glow(
    &self,
    color: crate::Color,
    opacities: &[f32],
  ) -> crate::Result<()> {
    self.grow_glow(opacities.len())?;
    Ok(self.color_glow(color, opacities)?)
  }

  /// Resizes the ring to match `rect`. Does not reposition the `HWND`
  /// itself -- callers still issue their own `SetWindowPos`.
  pub(crate) fn set_rect(&self, rect: &Rect) -> crate::Result<()> {
    let size = Vector2 {
      X: pixels_to_dips(rect.width()),
      Y: pixels_to_dips(rect.height()),
    };

    Ok(self.apply_ring(Ring {
      size,
      ..self.ring.get()
    })?)
  }

  /// Updates the ring's color, `lit` at its top edge, and its inner
  /// edge's hairline `rim` color.
  pub(crate) fn set_colors(
    &self,
    color: crate::Color,
    lit: crate::Color,
    rim: crate::Color,
  ) -> crate::Result<()> {
    let [lit_stop, stop] = &self.stroke_stops;
    lit_stop.SetColor(to_ui_color(lit))?;
    stop.SetColor(to_ui_color(color))?;
    self.rim_brush.SetColor(to_ui_color(rim))?;
    Ok(())
  }

  /// Updates the border width, i.e. the stroke's thickness, and how far
  /// the glow reaches past it.
  ///
  /// The overlay's `HWND` is outset by both, so callers must resize the
  /// window (and hence call [`set_rect`]) to match -- this only updates
  /// what is drawn inside it.
  ///
  /// [`set_rect`]: BorderVisual::set_rect
  pub(crate) fn set_widths(
    &self,
    width: f32,
    glow_reach: f32,
  ) -> crate::Result<()> {
    self.grow_glow(glow_bands(glow_reach))?;

    Ok(self.apply_ring(Ring {
      width,
      glow_reach,
      ..self.ring.get()
    })?)
  }

  /// Updates the ring's outer corner radius.
  pub(crate) fn set_corner_radius(&self, value: f32) -> crate::Result<()> {
    Ok(self.apply_ring(Ring {
      corner_radius: value,
      ..self.ring.get()
    })?)
  }

  /// Updates the overlay's own opacity.
  pub(crate) fn set_opacity(&self, value: f32) -> crate::Result<()> {
    self.root.SetOpacity(value)?;
    Ok(())
  }

  /// Hides the ring's outer `left`, `top`, `right` and `bottom` pixels.
  pub(crate) fn set_clip_insets(
    &self,
    [left, top, right, bottom]: [i32; 4],
  ) -> crate::Result<()> {
    self.clip.SetLeftInset(pixels_to_dips(left))?;
    self.clip.SetTopInset(pixels_to_dips(top))?;
    self.clip.SetRightInset(pixels_to_dips(right))?;
    self.clip.SetBottomInset(pixels_to_dips(bottom))?;
    Ok(())
  }

  /// Translates the ring within its `HWND`, in pixels relative to the
  /// window's top-left.
  ///
  /// Used by the workspace-switch slide, where the overlay window stays
  /// pinned to the monitor viewport and only its content moves: one
  /// property write per frame instead of a `SetWindowPos` plus a geometry
  /// rebuild. Content driven outside the window's bounds is clipped by the
  /// `DesktopWindowTarget`, so a ring sliding off the monitor is cut at
  /// the edge rather than spilling onto the neighbouring one.
  pub(crate) fn set_offset(&self, x: i32, y: i32) -> crate::Result<()> {
    self.root.SetOffset(Vector3 {
      X: pixels_to_dips(x),
      Y: pixels_to_dips(y),
      Z: 0.0,
    })?;
    Ok(())
  }
}

/// Builds the full visual tree: a `ShapeVisual` rooting a
/// `CompositionSpriteShape` that strokes a rounded rectangle in the border
/// color, the hairline over its inner edge and the glow's bands around it.
/// No fill brush is set, so the shapes' interiors stay transparent and the
/// tracked window shows through with no window region, mask, or reliance
/// on that window occluding a fill.
fn build_border_visual_tree(
  compositor: &Compositor,
  hwnd: HWND,
  rect: &Rect,
  params: BorderOverlayParams,
) -> windows::core::Result<BorderVisual> {
  // SAFETY: `hwnd` is a valid, already-created top-level window.
  let target = unsafe {
    compositor
      .cast::<ICompositorDesktopInterop>()?
      .CreateDesktopWindowTarget(hwnd, false)?
  };

  let geometry = compositor.CreateRoundedRectangleGeometry()?;

  // In the shape's own pixels, as the end point is set from the ring's
  // height (see `BorderVisual::apply_ring`).
  let stroke_brush = compositor.CreateLinearGradientBrush()?;
  stroke_brush.SetMappingMode(CompositionMappingMode::Absolute)?;
  let stroke_stops = [
    compositor.CreateColorGradientStopWithOffsetAndColor(
      0.0,
      to_ui_color(params.lit_color()),
    )?,
    compositor.CreateColorGradientStopWithOffsetAndColor(
      1.0,
      to_ui_color(params.color),
    )?,
  ];
  let stops = stroke_brush.ColorStops()?;
  for stop in &stroke_stops {
    stops.Append(stop)?;
  }

  let shape = compositor.CreateSpriteShapeWithGeometry(&geometry)?;
  shape.SetStrokeBrush(&stroke_brush)?;

  let rim_geometry = compositor.CreateRoundedRectangleGeometry()?;
  let rim_brush = compositor
    .CreateColorBrushWithColor(to_ui_color(params.rim_color()))?;
  let rim = compositor.CreateSpriteShapeWithGeometry(&rim_geometry)?;
  rim.SetStrokeBrush(&rim_brush)?;

  // Made here for the glow it starts with, as `grow_glow` can't wait on
  // the composition thread from that very thread.
  let glow = GlowBand::create_all(compositor, params.glow.bands())?;

  let root = compositor.CreateShapeVisual()?;
  root.SetOpacity(params.opacity)?;
  let shapes = root.Shapes()?;
  shapes.Append(&shape)?;
  shapes.Append(&rim)?;
  for band in &glow {
    shapes.Append(&band.shape)?;
  }

  let clip = compositor.CreateInsetClip()?;
  root.SetClip(&clip)?;

  target.SetRoot(&root)?;

  let visual = BorderVisual {
    _target: target,
    root,
    shape,
    stroke_brush,
    stroke_stops,
    geometry,
    rim,
    rim_brush,
    rim_geometry,
    glow: RefCell::new(glow),
    clip,
    ring: Cell::new(Ring {
      size: Vector2 { X: 0.0, Y: 0.0 },
      width: params.width,
      corner_radius: params.corner_radius,
      glow_reach: 0.0,
    }),
  };

  // Sizes the root and derives the stroke geometry through the one place
  // that math lives, rather than duplicating it here.
  visual.apply_ring(Ring {
    size: Vector2 {
      X: pixels_to_dips(rect.width()),
      Y: pixels_to_dips(rect.height()),
    },
    width: params.width,
    corner_radius: params.corner_radius,
    glow_reach: params.glow.reach,
  })?;
  visual.color_glow(
    params.color,
    &BorderGlow::band_opacities(&params.glow, &params.glow, 1.0),
  )?;

  Ok(visual)
}

/// Thickness of a drop indicator's outline.
const DROP_INDICATOR_STROKE: f32 = 2.0;

/// A backdrop's vignette alone, for a copy of a window's backdrop (see
/// `VignetteSource`): the overlay's own brush, stretched over the whole of
/// its `HWND`, so the copy matches the overlay.
pub(crate) struct VignetteVisual {
  /// Binds the visual tree to the `HWND`. Kept alive but never touched
  /// again -- dropping it would unbind composition from the window.
  _target: DesktopWindowTarget,
  _root: SpriteVisual,
}

impl VignetteVisual {
  /// Builds the vignette at `strength`, from `0.0` to `1.0`, on `hwnd`,
  /// which must have been created with `WS_EX_NOREDIRECTIONBITMAP`.
  pub(crate) fn create(hwnd: HWND, strength: f32) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;

    with_composition_thread(move |compositor, _| {
      // SAFETY: `hwnd` is a valid, already-created top-level window.
      let target = unsafe {
        compositor
          .cast::<ICompositorDesktopInterop>()?
          .CreateDesktopWindowTarget(HWND(hwnd_raw), false)?
      };

      let root = compositor.CreateSpriteVisual()?;
      root.SetRelativeSizeAdjustment(FILL_PARENT)?;
      root.SetBrush(&build_vignette_brush(&compositor)?)?;
      root.SetOpacity(strength.clamp(0.0, 1.0))?;
      target.SetRoot(&root)?;

      Ok(Self {
        _target: target,
        _root: root,
      })
    })
  }
}

/// Share of the outline color's alpha a drop indicator's fill gets.
const DROP_INDICATOR_FILL_ALPHA: f32 = 0.2;

/// Length of a drop indicator's glide from one target to the next.
const DROP_INDICATOR_GLIDE: Duration = Duration::from_millis(180);

/// Length of a drop indicator's fade in when it appears.
const DROP_INDICATOR_FADE_IN: Duration = Duration::from_millis(120);

/// A drop indicator's visual tree: one rounded rectangle, washed with its
/// color and outlined in it, inside a window covering the whole monitor.
///
/// The rectangle glides to each new target on the compositor, so the WM
/// sets where it goes once per target instead of moving it every frame.
pub(crate) struct DropIndicatorVisual {
  /// Binds the visual tree to the indicator's `HWND`. Kept alive but
  /// never touched again.
  _target: DesktopWindowTarget,

  root: ShapeVisual,
  geometry: CompositionRoundedRectangleGeometry,
  fill_brush: CompositionColorBrush,
  stroke_brush: CompositionColorBrush,

  /// The glides' destination, read by their expression key frames when
  /// they start.
  goal: CompositionPropertySet,

  /// Animations of the geometry's `Offset` and `Size` to `goal`. Built
  /// once and restarted per target: with no key frame at 0, each starts
  /// from wherever the previous one had got to.
  offset_glide: Vector2KeyFrameAnimation,
  size_glide: Vector2KeyFrameAnimation,

  fade_in: ScalarKeyFrameAnimation,
}

impl DropIndicatorVisual {
  /// Builds the visual tree for `hwnd`, which covers `viewport`, with the
  /// rectangle hidden until it is first placed.
  pub(crate) fn create(
    hwnd: HWND,
    viewport: &Rect,
    color: crate::Color,
    corner_radius: f32,
  ) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;
    let viewport = viewport.clone();

    with_composition_thread(move |compositor, _| {
      build_drop_indicator_tree(
        &compositor,
        HWND(hwnd_raw),
        &viewport,
        color,
        corner_radius,
      )
    })
  }

  /// Resizes the tree to a window now covering `viewport`.
  pub(crate) fn set_viewport(&self, viewport: &Rect) -> crate::Result<()> {
    self.root.SetSize(rect_size(viewport))?;
    Ok(())
  }

  /// Puts the rectangle at `rect`, relative to the window, at once.
  pub(crate) fn jump_to(&self, rect: &Rect) -> crate::Result<()> {
    let (offset, size) = drop_indicator_geometry(rect);

    self.geometry.StopAnimation(h!("Offset"))?;
    self.geometry.StopAnimation(h!("Size"))?;
    self.geometry.SetOffset(offset)?;
    self.geometry.SetSize(size)?;
    self.goal.InsertVector2(h!("Offset"), offset)?;
    self.goal.InsertVector2(h!("Size"), size)?;
    Ok(())
  }

  /// Glides the rectangle to `rect`, relative to the window, from wherever
  /// it is, including partway through a previous glide.
  pub(crate) fn glide_to(&self, rect: &Rect) -> crate::Result<()> {
    let (offset, size) = drop_indicator_geometry(rect);

    self.goal.InsertVector2(h!("Offset"), offset)?;
    self.goal.InsertVector2(h!("Size"), size)?;
    self
      .geometry
      .StartAnimation(h!("Offset"), &self.offset_glide)?;
    self.geometry.StartAnimation(h!("Size"), &self.size_glide)?;
    Ok(())
  }

  /// Updates the outline's color, and the fill's faint wash of it.
  pub(crate) fn set_color(
    &self,
    color: crate::Color,
  ) -> crate::Result<()> {
    self.stroke_brush.SetColor(to_ui_color(color))?;
    self
      .fill_brush
      .SetColor(to_ui_color(drop_indicator_fill(color)))?;
    Ok(())
  }

  /// Updates the rectangle's outer corner radius.
  pub(crate) fn set_corner_radius(&self, value: f32) -> crate::Result<()> {
    self
      .geometry
      .SetCornerRadius(drop_indicator_corner_radius(value))?;
    Ok(())
  }

  /// Fades the tree in from fully transparent.
  pub(crate) fn fade_in(&self) -> crate::Result<()> {
    self.root.StartAnimation(h!("Opacity"), &self.fade_in)?;
    Ok(())
  }

  /// Makes the tree fully transparent, so whatever it last showed can't
  /// flash up when its window is next shown.
  pub(crate) fn clear(&self) -> crate::Result<()> {
    self.root.StopAnimation(h!("Opacity"))?;
    self.root.SetOpacity(0.0)?;
    Ok(())
  }
}

/// The fill color of a drop indicator outlined in `color`.
fn drop_indicator_fill(color: crate::Color) -> crate::Color {
  #[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless
  )]
  let a = (color.a as f32 * DROP_INDICATOR_FILL_ALPHA).round() as u8;

  crate::Color { a, ..color }
}

/// Size of `rect` in DIPs.
fn rect_size(rect: &Rect) -> Vector2 {
  Vector2 {
    X: pixels_to_dips(rect.width()),
    Y: pixels_to_dips(rect.height()),
  }
}

/// Geometry corner radius giving the outline an outer radius of `value`.
fn drop_indicator_corner_radius(value: f32) -> Vector2 {
  let radius = (value - DROP_INDICATOR_STROKE / 2.0).max(0.0);
  Vector2 {
    X: radius,
    Y: radius,
  }
}

/// Geometry offset and size that outline `rect` exactly: the stroke
/// straddles its path, so the path is inset by half the stroke.
fn drop_indicator_geometry(rect: &Rect) -> (Vector2, Vector2) {
  let inset = DROP_INDICATOR_STROKE / 2.0;

  let offset = Vector2 {
    X: pixels_to_dips(rect.x()) + inset,
    Y: pixels_to_dips(rect.y()) + inset,
  };
  let size = rect_size(rect);
  let size = Vector2 {
    X: (size.X - DROP_INDICATOR_STROKE).max(0.0),
    Y: (size.Y - DROP_INDICATOR_STROKE).max(0.0),
  };

  (offset, size)
}

/// Builds a drop indicator's visual tree, transparent until faded in.
fn build_drop_indicator_tree(
  compositor: &Compositor,
  hwnd: HWND,
  viewport: &Rect,
  color: crate::Color,
  corner_radius: f32,
) -> windows::core::Result<DropIndicatorVisual> {
  // SAFETY: `hwnd` is a valid, already-created top-level window.
  let target = unsafe {
    compositor
      .cast::<ICompositorDesktopInterop>()?
      .CreateDesktopWindowTarget(hwnd, false)?
  };

  let geometry = compositor.CreateRoundedRectangleGeometry()?;
  geometry.SetCornerRadius(drop_indicator_corner_radius(corner_radius))?;
  let fill_brush = compositor
    .CreateColorBrushWithColor(to_ui_color(drop_indicator_fill(color)))?;
  let stroke_brush =
    compositor.CreateColorBrushWithColor(to_ui_color(color))?;

  let shape = compositor.CreateSpriteShapeWithGeometry(&geometry)?;
  shape.SetFillBrush(&fill_brush)?;
  shape.SetStrokeBrush(&stroke_brush)?;
  shape.SetStrokeThickness(DROP_INDICATOR_STROKE)?;

  let root = compositor.CreateShapeVisual()?;
  root.SetSize(rect_size(viewport))?;
  root.SetOpacity(0.0)?;
  root.Shapes()?.Append(&shape)?;
  target.SetRoot(&root)?;

  let goal = compositor.CreatePropertySet()?;
  goal.InsertVector2(h!("Offset"), Vector2::default())?;
  goal.InsertVector2(h!("Size"), Vector2::default())?;

  let easing = compositor.CreateCubicBezierEasingFunction(
    Vector2 { X: 0.2, Y: 0.9 },
    Vector2 { X: 0.3, Y: 1.0 },
  )?;

  let glide = |property: &HSTRING| {
    let animation = compositor.CreateVector2KeyFrameAnimation()?;
    animation.InsertExpressionKeyFrameWithEasingFunction(
      1.0, property, &easing,
    )?;
    animation.SetReferenceParameter(h!("goal"), &goal)?;
    animation.SetDuration(DROP_INDICATOR_GLIDE.into())?;
    windows::core::Result::Ok(animation)
  };
  let offset_glide = glide(h!("goal.Offset"))?;
  let size_glide = glide(h!("goal.Size"))?;

  let fade_in = compositor.CreateScalarKeyFrameAnimation()?;
  fade_in.InsertKeyFrame(0.0, 0.0)?;
  fade_in.InsertKeyFrame(1.0, 1.0)?;
  fade_in.SetDuration(DROP_INDICATOR_FADE_IN.into())?;

  Ok(DropIndicatorVisual {
    _target: target,
    root,
    geometry,
    fill_brush,
    stroke_brush,
    goal,
    offset_glide,
    size_glide,
    fade_in,
  })
}

/// A particle burst's visual tree: a sprite per particle, drawn in its
/// style and played along its planned path by the compositor as it fades,
/// turns and tumbles, with nothing left for the caller to drive.
pub(crate) struct ParticleVisual {
  /// Binds the visual tree to the burst's `HWND`. Kept alive but never
  /// touched again.
  _target: DesktopWindowTarget,
  _root: ContainerVisual,
}

impl ParticleVisual {
  /// Builds `plan`'s particles for `hwnd`, which covers the plan's
  /// bounds, and sets them off.
  pub(crate) fn create(
    hwnd: HWND,
    plan: &BurstPlan,
  ) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;
    let plan = plan.clone();

    with_composition_thread(move |compositor, _| {
      build_particle_tree(&compositor, HWND(hwnd_raw), &plan)
    })
  }
}

/// A piece of a particle's look: `brush` filling `size` (a multiple of
/// the particle's own), centered at `center` (a share of the particle's
/// size), turned `angle` degrees.
struct ParticlePart {
  brush: CompositionBrush,
  size: (f32, f32),
  center: (f32, f32),
  angle: f32,
}

impl ParticlePart {
  /// A part filling the whole particle.
  fn whole(brush: CompositionBrush) -> Self {
    Self {
      brush,
      size: (1.0, 1.0),
      center: (0.5, 0.5),
      angle: 0.0,
    }
  }
}

/// A brush fading through `stops` from its middle out to its rim,
/// stretched over whatever it fills.
fn radial_brush(
  compositor: &Compositor,
  stops: &[(f32, crate::Color)],
) -> windows::core::Result<CompositionBrush> {
  let brush = compositor.CreateRadialGradientBrush()?;
  brush.SetMappingMode(CompositionMappingMode::Relative)?;
  brush.SetEllipseCenter(Vector2 { X: 0.5, Y: 0.5 })?;
  brush.SetEllipseRadius(Vector2 { X: 0.5, Y: 0.5 })?;
  append_stops(compositor, &brush.ColorStops()?, stops)?;
  brush.cast()
}

/// A brush running through `stops` from its top left to its bottom right.
fn diagonal_brush(
  compositor: &Compositor,
  stops: &[(f32, crate::Color)],
) -> windows::core::Result<CompositionBrush> {
  let brush = compositor.CreateLinearGradientBrush()?;
  brush.SetMappingMode(CompositionMappingMode::Relative)?;
  brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
  brush.SetEndPoint(Vector2 { X: 1.0, Y: 1.0 })?;
  append_stops(compositor, &brush.ColorStops()?, stops)?;
  brush.cast()
}

fn append_stops(
  compositor: &Compositor,
  collection: &CompositionColorGradientStopCollection,
  stops: &[(f32, crate::Color)],
) -> windows::core::Result<()> {
  for (offset, color) in stops {
    collection.Append(
      &compositor.CreateColorGradientStopWithOffsetAndColor(
        *offset,
        to_ui_color(*color),
      )?,
    )?;
  }
  Ok(())
}

/// The parts a particle of `style` in `color` is drawn with; the first
/// fills the particle itself, the rest sit on it.
fn particle_parts(
  compositor: &Compositor,
  style: ParticleStyle,
  color: crate::Color,
) -> windows::core::Result<Vec<ParticlePart>> {
  const WHITE: crate::Color = crate::Color {
    r: u8::MAX,
    g: u8::MAX,
    b: u8::MAX,
    a: u8::MAX,
  };
  const BLACK: crate::Color = crate::Color {
    r: 0,
    g: 0,
    b: 0,
    a: u8::MAX,
  };

  let color = crate::Color {
    a: u8::MAX,
    ..color
  };
  let tint = |a: u8| crate::Color { a, ..color };
  let white = |a: u8| crate::Color { a, ..WHITE };
  // `color` heated toward white, as `heat` goes from 0 to 1.
  let hot = |heat: f32, a: u8| crate::Color {
    a,
    ..color.lerp(&WHITE, heat)
  };
  let radial = |stops: &[(f32, crate::Color)]| {
    radial_brush(compositor, stops).map(ParticlePart::whole)
  };

  Ok(match style {
    ParticleStyle::Sparks => vec![radial(&[
      (0.0, hot(0.7, u8::MAX)),
      (0.35, tint(220)),
      (1.0, tint(0)),
    ])?],
    ParticleStyle::Dust => {
      vec![radial(&[
        (0.0, tint(170)),
        (0.5, tint(90)),
        (1.0, tint(0)),
      ])?]
    }
    ParticleStyle::Embers => vec![radial(&[
      (0.0, hot(0.45, u8::MAX)),
      (0.3, tint(235)),
      (0.6, tint(90)),
      (1.0, tint(0)),
    ])?],
    // A flat fleck with a sheen across it, which catches the eye as it
    // turns and tumbles.
    ParticleStyle::Confetti => {
      vec![ParticlePart::whole(diagonal_brush(
        compositor,
        &[
          (0.0, hot(0.35, u8::MAX)),
          (0.45, color),
          (1.0, color.lerp(&BLACK, 0.2)),
        ],
      )?)]
    }
    // A white-hot core crossed by two long, thin rays.
    ParticleStyle::Glints => {
      let ray = radial_brush(
        compositor,
        &[(0.0, hot(0.85, u8::MAX)), (0.35, tint(160)), (1.0, tint(0))],
      )?;
      let ray = |angle: f32| ParticlePart {
        brush: ray.clone(),
        size: (2.6, 0.16),
        center: (0.5, 0.5),
        angle,
      };

      vec![
        radial(&[
          (0.0, white(u8::MAX)),
          (0.3, hot(0.6, 220)),
          (1.0, tint(0)),
        ])?,
        ray(0.0),
        ray(90.0),
      ]
    }
    // A thin ring around a faint film, with a highlight up and to the
    // left.
    ParticleStyle::Bubbles => vec![
      radial(&[
        (0.0, tint(18)),
        (0.7, tint(40)),
        (0.84, hot(0.3, 210)),
        (0.93, tint(110)),
        (1.0, tint(0)),
      ])?,
      ParticlePart {
        brush: radial_brush(
          compositor,
          &[(0.0, white(230)), (1.0, white(0))],
        )?,
        size: (0.28, 0.28),
        center: (0.32, 0.3),
        angle: 0.0,
      },
    ],
  })
}

/// Builds a particle burst's visual tree and starts every particle's
/// animations, each after its own delay, before which it is transparent.
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn build_particle_tree(
  compositor: &Compositor,
  hwnd: HWND,
  plan: &BurstPlan,
) -> windows::core::Result<ParticleVisual> {
  // SAFETY: `hwnd` is a valid, already-created top-level window.
  let target = unsafe {
    compositor
      .cast::<ICompositorDesktopInterop>()?
      .CreateDesktopWindowTarget(hwnd, false)?
  };

  let root = compositor.CreateContainerVisual()?;
  root.SetSize(rect_size(&plan.bounds))?;
  target.SetRoot(&root)?;

  // Every key is already where it should be at its time, so each runs
  // straight to the next.
  let linear = compositor.CreateLinearEasingFunction()?;
  let children = root.Children()?;
  let point = |(x, y): (f32, f32)| Vector3 { X: x, Y: y, Z: 0.0 };
  let scale = |(x, y): (f32, f32)| Vector3 { X: x, Y: y, Z: 1.0 };

  // Each look's parts, made once and shared by its particles.
  let mut looks =
    Vec::<(ParticleStyle, crate::Color, Vec<ParticlePart>)>::new();

  for particle in &plan.particles {
    let made = looks.iter().position(|(style, color, _)| {
      *style == particle.style && *color == particle.color
    });
    let index = if let Some(index) = made {
      index
    } else {
      let parts =
        particle_parts(compositor, particle.style, particle.color)?;
      looks.push((particle.style, particle.color, parts));
      looks.len() - 1
    };
    let Some((body, parts)) = looks[index].2.split_first() else {
      continue;
    };

    let (width, height) = particle.size;
    let scale_keys = particle.scale_keys();

    let sprite = compositor.CreateSpriteVisual()?;
    sprite.SetBrush(&body.brush)?;
    sprite.SetSize(Vector2 {
      X: width,
      Y: height,
    })?;
    sprite.SetAnchorPoint(Vector2 { X: 0.5, Y: 0.5 })?;
    sprite.SetCenterPoint(point((width / 2.0, height / 2.0)))?;
    sprite.SetOffset(point(particle.path[0]))?;
    sprite.SetRotationAngleInDegrees(particle.angle)?;
    if let Some((_, first)) = scale_keys.first() {
      sprite.SetScale(scale(*first))?;
    }
    sprite.SetOpacity(0.0)?;

    for part in parts {
      let size = Vector2 {
        X: width * part.size.0,
        Y: height * part.size.1,
      };
      let child = compositor.CreateSpriteVisual()?;
      child.SetBrush(&part.brush)?;
      child.SetSize(size)?;
      child.SetAnchorPoint(Vector2 { X: 0.5, Y: 0.5 })?;
      child.SetCenterPoint(point((size.X / 2.0, size.Y / 2.0)))?;
      child.SetOffset(point((
        width * part.center.0,
        height * part.center.1,
      )))?;
      child.SetRotationAngleInDegrees(part.angle)?;
      sprite.Children()?.InsertAtTop(&child)?;
    }
    children.InsertAtTop(&sprite)?;

    let path = compositor.CreateVector3KeyFrameAnimation()?;
    for (key, position) in particle.path.iter().enumerate() {
      path.InsertKeyFrameWithEasingFunction(
        key as f32 / (PATH_KEYS - 1) as f32,
        point(*position),
        &linear,
      )?;
    }

    let fade = compositor.CreateScalarKeyFrameAnimation()?;
    for (progress, opacity) in &particle.opacity {
      fade
        .InsertKeyFrameWithEasingFunction(*progress, *opacity, &linear)?;
    }

    let grow = compositor.CreateVector3KeyFrameAnimation()?;
    for (progress, value) in &scale_keys {
      grow.InsertKeyFrameWithEasingFunction(
        *progress,
        scale(*value),
        &linear,
      )?;
    }

    let start = |property: &HSTRING, animation: KeyFrameAnimation| {
      animation.SetDuration(particle.duration.into())?;
      animation.SetDelayTime(particle.delay.into())?;
      sprite.StartAnimation(property, &animation)
    };
    start(h!("Offset"), path.cast()?)?;
    start(h!("Opacity"), fade.cast()?)?;
    start(h!("Scale"), grow.cast()?)?;

    if particle.spin.abs() > f32::EPSILON {
      let turn = compositor.CreateScalarKeyFrameAnimation()?;
      turn.InsertKeyFrameWithEasingFunction(
        0.0,
        particle.angle,
        &linear,
      )?;
      turn.InsertKeyFrameWithEasingFunction(
        1.0,
        particle.angle + particle.spin,
        &linear,
      )?;
      start(h!("RotationAngleInDegrees"), turn.cast()?)?;
    }
  }

  Ok(ParticleVisual {
    _target: target,
    _root: root,
  })
}

/// The window overview's background: the monitor's wallpaper as it is,
/// with a blurred, tinted copy of it fading in on top.
///
/// The plain copy is what lets the overview cover the real windows from
/// its first frame without anything visibly changing: their previews sit
/// exactly over them, and around them is what the desktop shows anyway.
pub(crate) struct OverviewBackdrop {
  /// Binds the visual tree to the overview's `HWND`. Kept alive but never
  /// touched again -- dropping it would unbind composition from the
  /// window.
  _target: DesktopWindowTarget,
  compositor: Compositor,
  queue: DispatcherQueue,
  sharp: CompositionSurfaceBrush,
  blurred: CompositionSurfaceBrush,
  blurred_sprite: SpriteVisual,
  tint_brush: CompositionColorBrush,
  tint_sprite: SpriteVisual,

  /// Bounds of the monitor whose wallpaper both brushes show.
  monitor: Rect,

  /// `wallpaper_surface`'s generation counter as of the last bind.
  generation: u64,

  blur: f32,
}

/// Bake knobs of the overview's wallpaper copies: blurred by `blur`, and
/// otherwise as the desktop shows it.
fn overview_wallpaper_knobs(blur: f32) -> BakeKnobs {
  BakeKnobs {
    blur_amount: blur,
    saturation: 1.0,
    exposure: 0.0,
    contrast: 0.0,
    highlights: 0.0,
    shadows: 0.0,
    grain: 0.0,
  }
}

impl OverviewBackdrop {
  /// Builds the background for the overview `hwnd`, which covers `rect`
  /// and was created with `WS_EX_NOREDIRECTIONBITMAP`. Starts out showing
  /// the plain wallpaper.
  pub(crate) fn create(
    hwnd: HWND,
    rect: &Rect,
    blur: f32,
    tint: crate::Color,
  ) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;
    let rect = rect.clone();

    with_composition_thread(move |compositor, queue| {
      // SAFETY: `hwnd` is a valid, already-created top-level window.
      let target = unsafe {
        compositor
          .cast::<ICompositorDesktopInterop>()?
          .CreateDesktopWindowTarget(HWND(hwnd_raw), false)?
      };

      // Parallax 1: the image stays put against the desktop.
      let (sharp, monitor) = wallpaper_surface::crop_brush(
        &compositor,
        &rect,
        overview_wallpaper_knobs(0.0),
        1.0,
      )?;
      let (blurred, _) = wallpaper_surface::crop_brush(
        &compositor,
        &rect,
        overview_wallpaper_knobs(blur),
        1.0,
      )?;
      let tint_brush =
        compositor.CreateColorBrushWithColor(to_ui_color(tint))?;

      let sharp_sprite = compositor.CreateSpriteVisual()?;
      sharp_sprite.SetBrush(&sharp)?;
      let blurred_sprite = compositor.CreateSpriteVisual()?;
      blurred_sprite.SetBrush(&blurred)?;
      blurred_sprite.SetOpacity(0.0)?;
      let tint_sprite = compositor.CreateSpriteVisual()?;
      tint_sprite.SetBrush(&tint_brush)?;
      tint_sprite.SetOpacity(0.0)?;

      let root = compositor.CreateContainerVisual()?;
      root.SetRelativeSizeAdjustment(FILL_PARENT)?;
      for sprite in [&sharp_sprite, &blurred_sprite, &tint_sprite] {
        sprite.SetRelativeSizeAdjustment(FILL_PARENT)?;
        root.Children()?.InsertAtTop(sprite)?;
      }
      target.SetRoot(&root)?;

      Ok(Self {
        _target: target,
        compositor: compositor.clone(),
        queue: queue.clone(),
        sharp,
        blurred,
        blurred_sprite,
        tint_brush,
        tint_sprite,
        monitor,
        generation: wallpaper_surface::generation(),
        blur,
      })
    })
  }

  /// Fades the blurred, tinted copy in, from 0 (plain wallpaper) to 1.
  pub(crate) fn set_progress(&self, progress: f32) -> crate::Result<()> {
    let progress = progress.clamp(0.0, 1.0);
    self.blurred_sprite.SetOpacity(progress)?;
    self.tint_sprite.SetOpacity(progress)?;
    Ok(())
  }

  /// Shows the wallpaper under `rect` with the given blur and tint,
  /// re-baking only when the monitor, the wallpaper or the blur changed.
  #[allow(clippy::float_cmp)]
  pub(crate) fn update(
    &mut self,
    rect: &Rect,
    blur: f32,
    tint: crate::Color,
  ) -> crate::Result<()> {
    self.tint_brush.SetColor(to_ui_color(tint))?;
    wallpaper_surface::poll_for_changes();

    let current = wallpaper_surface::generation();
    let monitor = wallpaper_surface::monitor_bounds(rect);

    if current != self.generation
      || monitor != self.monitor
      || blur != self.blur
    {
      let compositor = self.compositor.clone();
      let (sharp, blurred) = (self.sharp.clone(), self.blurred.clone());
      let target = monitor.clone();

      // Queued, not awaited: the new image showing a frame later beats
      // blocking the overview until it is baked.
      dispatch_on_composition_thread(&self.queue, move || {
        let rebound = wallpaper_surface::rebind(
          &compositor,
          &sharp,
          &target,
          overview_wallpaper_knobs(0.0),
        )
        .and_then(|()| {
          wallpaper_surface::rebind(
            &compositor,
            &blurred,
            &target,
            overview_wallpaper_knobs(blur),
          )
        });

        if let Err(err) = rebound {
          tracing::warn!("Overview backdrop re-bind failed: {err}.");
        }
      })?;

      self.monitor = monitor;
      self.generation = current;
      self.blur = blur;
    }

    wallpaper_surface::set_crop(&self.sharp, rect, &self.monitor, 1.0);
    wallpaper_surface::set_crop(&self.blurred, rect, &self.monitor, 1.0);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use windows::Foundation::Numerics::Vector2;

  use super::{glow_band_geometry, rim_geometry, ring_geometry, Ring};

  /// A ring's stroke straddles its geometry, so the path sits half a width
  /// inside the overlay on every side and its radius shrinks by that same
  /// half -- putting the band's outer edge on the overlay's rect and its
  /// inner edge on the tracked window's rect.
  #[test]
  fn geometry_is_inset_by_half_the_stroke() {
    let (offset, size, corner_radius) = ring_geometry(Ring {
      size: Vector2 { X: 800.0, Y: 600.0 },
      width: 4.0,
      corner_radius: 10.0,
      glow_reach: 0.0,
    });

    assert_eq!(offset, Vector2 { X: 2.0, Y: 2.0 });
    assert_eq!(size, Vector2 { X: 796.0, Y: 596.0 });
    assert!((corner_radius - 8.0).abs() < f32::EPSILON);
  }

  /// A zero-width border collapses to a zero-thickness stroke over the
  /// overlay's full rect, with the corner radius left untouched.
  #[test]
  fn zero_width_leaves_geometry_at_full_size() {
    let (offset, size, corner_radius) = ring_geometry(Ring {
      size: Vector2 { X: 800.0, Y: 600.0 },
      width: 0.0,
      corner_radius: 10.0,
      glow_reach: 0.0,
    });

    assert_eq!(offset, Vector2 { X: 0.0, Y: 0.0 });
    assert_eq!(size, Vector2 { X: 800.0, Y: 600.0 });
    assert!((corner_radius - 10.0).abs() < f32::EPSILON);
  }

  /// A width wider than the overlay itself (or than its corner radius)
  /// clamps to zero rather than producing a negative extent, which
  /// composition rejects.
  #[test]
  fn oversized_width_clamps_instead_of_going_negative() {
    let (_, size, corner_radius) = ring_geometry(Ring {
      size: Vector2 { X: 20.0, Y: 10.0 },
      width: 40.0,
      corner_radius: 2.0,
      glow_reach: 0.0,
    });

    assert_eq!(size, Vector2 { X: 0.0, Y: 0.0 });
    assert!(corner_radius.abs() < f32::EPSILON);
  }

  /// A negative width (never configured, but cheap to defend against)
  /// behaves exactly like zero rather than insetting outwards.
  #[test]
  fn negative_width_behaves_like_zero() {
    let (offset, size, _) = ring_geometry(Ring {
      size: Vector2 { X: 100.0, Y: 50.0 },
      width: -8.0,
      corner_radius: 4.0,
      glow_reach: 0.0,
    });

    assert_eq!(offset, Vector2 { X: 0.0, Y: 0.0 });
    assert_eq!(size, Vector2 { X: 100.0, Y: 50.0 });
  }

  fn ring(width: f32, corner_radius: f32, glow_reach: f32) -> Ring {
    Ring {
      size: Vector2 { X: 800.0, Y: 600.0 },
      width,
      corner_radius,
      glow_reach,
    }
  }

  /// The glow takes the outer part of the overlay, so the ring sits inside
  /// it, still ending on the tracked window's rect.
  #[test]
  fn glow_insets_the_ring_by_its_reach() {
    let (offset, size, corner_radius) =
      ring_geometry(ring(4.0, 10.0, 6.0));

    assert_eq!(offset, Vector2 { X: 8.0, Y: 8.0 });
    assert_eq!(size, Vector2 { X: 784.0, Y: 584.0 });
    assert!((corner_radius - 8.0).abs() < f32::EPSILON);
  }

  /// The rim is the innermost pixel of the ring, or all of a thinner one.
  #[test]
  fn rim_follows_the_ring_inner_edge() {
    let ((offset, size, corner_radius), thickness) =
      rim_geometry(ring(4.0, 10.0, 6.0));

    assert!((thickness - 1.0).abs() < f32::EPSILON);
    assert_eq!(offset, Vector2 { X: 9.5, Y: 9.5 });
    assert_eq!(size, Vector2 { X: 781.0, Y: 581.0 });
    assert!((corner_radius - 6.5).abs() < f32::EPSILON);

    let (_, thin) = rim_geometry(ring(0.5, 10.0, 0.0));
    assert!((thin - 0.5).abs() < f32::EPSILON);
  }

  /// Bands run outward from the ring's outer edge, a pixel each, rounding
  /// off further out.
  #[test]
  fn glow_bands_run_outward_from_the_ring() {
    let (offset, size, corner_radius) =
      glow_band_geometry(ring(2.0, 2.0, 6.0), 0);
    assert_eq!(offset, Vector2 { X: 5.5, Y: 5.5 });
    assert_eq!(size, Vector2 { X: 789.0, Y: 589.0 });
    assert!((corner_radius - 2.5).abs() < f32::EPSILON);

    let (offset, _, corner_radius) =
      glow_band_geometry(ring(2.0, 2.0, 6.0), 5);
    assert_eq!(offset, Vector2 { X: 0.5, Y: 0.5 });
    assert!((corner_radius - 7.5).abs() < f32::EPSILON);
  }
}
