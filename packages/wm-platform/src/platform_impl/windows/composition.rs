//! `Windows.UI.Composition` based blur pipeline for the overlay-backed
//! backdrop styles that render through a visual tree.
//!
//! Replaces SWCA's fixed-intensity `ACCENT_ENABLE_ACRYLICBLURBEHIND` with a
//! host-backdrop brush fed through a hand-implemented Gaussian-blur effect
//! graph, giving a continuously adjustable blur radius, plus a
//! `CompositionRoundedRectangleGeometry` clip for a continuous corner
//! radius -- neither of which SWCA/Mica can provide. Validated in
//! `packages/composition-blur-spike` before this port; see that crate's
//! module docs for the concrete API findings this implementation relies on.
//!
//! # Threading
//!
//! A `Compositor` must be created on a thread that owns a dispatcher queue,
//! and (confirmed empirically in the spike, not just per docs) that thread
//! must keep pumping messages for async composition work -- e.g. the
//! effect factory's shader-graph compile -- to ever complete. The
//! wallpaper backdrop's D2D/WIC device stack (see `graphics_device`) is
//! thread-affine besides, and lives on this same thread for that reason.
//! `wm`'s main loop drives everything through `tokio::select!`/
//! `rt.block_on`, which never pumps Win32 messages, so the entire
//! composition pipeline (the `Compositor` itself, and every per-overlay
//! visual-tree build, which touches the same async-sensitive effect
//! factory) is constructed on a dedicated, self-pumping OS thread obtained
//! via `DispatcherQueueController::CreateOnDedicatedThread`.
//!
//! Once created, `Compositor` and every composition object handed back to
//! callers are documented `WinRT` "agile" objects (`windows-rs` applies
//! `unsafe impl Send + Sync` to each of them) -- so the frequent per-tick
//! property updates (`set_rect`, `set_tint`, `set_blur_amount`,
//! `set_corner_radius`, `set_opacity`, `set_saturation`) call directly
//! into them from the caller's thread with no cross-thread marshaling,
//! keeping the hot path exactly as cheap as the SWCA path it replaces.

use std::{
  cell::{Cell, RefCell},
  sync::{mpsc, OnceLock},
  time::Duration,
};

use windows::{
  core::{implement, ComInterface, GUID, HSTRING, PCWSTR},
  Foundation::{
    Numerics::{Vector2, Vector3},
    PropertyValue,
  },
  Graphics::Effects::{
    IGraphicsEffect, IGraphicsEffect_Impl, IGraphicsEffectSource,
    IGraphicsEffectSource_Impl,
  },
  System::{DispatcherQueue, DispatcherQueueController, DispatcherQueueHandler},
  UI::{
    Color,
    Composition::{
      CompositionBackdropBrush, CompositionColorBrush, CompositionEffectBrush,
      CompositionEffectSourceParameter, CompositionRoundedRectangleGeometry,
      CompositionMappingMode, CompositionRadialGradientBrush,
      CompositionSpriteShape, CompositionSurfaceBrush, Compositor,
      ContainerVisual, Desktop::DesktopWindowTarget, ShapeVisual,
      SpriteVisual,
    },
  },
  Win32::{
    Foundation::{E_INVALIDARG, HWND},
    System::WinRT::{
      Composition::ICompositorDesktopInterop,
      Graphics::Direct2D::{
        IGraphicsEffectD2D1Interop, IGraphicsEffectD2D1Interop_Impl,
        GRAPHICS_EFFECT_PROPERTY_MAPPING,
        GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT,
      },
    },
  },
};

use super::wallpaper_surface::{self, BakeKnobs};
use crate::{BackdropStyle, BlurOverlayParams, BorderOverlayParams, Rect};

/// `CLSID_D2D1GaussianBlur`, the built-in D2D1 Gaussian-blur effect.
const CLSID_D2D1_GAUSSIAN_BLUR: GUID =
  GUID::from_u128(0x1feb_6d69_2fe6_4ac9_8c58_1d7f_93e7_a6a5);

/// `D2D1_GAUSSIANBLUR_OPTIMIZATION_PERFORMANCE`. Trades some blur-kernel
/// accuracy for a cheaper separable-pass approximation, vs. the `BALANCED`
/// mode this used previously. A downsample-then-upscale approach (rendering
/// the blur at reduced resolution) was also tried for a bigger win, but
/// caused an intermittent `AppHangB1` under real use (confirmed via Windows
/// Event Viewer) that couldn't be pinned down with diagnostic tracing in
/// the time available -- reverted. This constant swap alone is a much
/// smaller, lower-risk change: a static effect-graph parameter evaluated
/// once at construction/rebuild time, not a per-frame property mutation.
const D2D1_GAUSSIANBLUR_OPTIMIZATION_PERFORMANCE: u32 = 2;
/// `D2D1_BORDER_MODE_SOFT`.
const D2D1_BORDER_MODE_SOFT: u32 = 0;

/// `CLSID_D2D1Saturation`, the built-in D2D1 saturation-adjustment effect.
/// Value matches `windows::Win32::Graphics::Direct2D::CLSID_D2D1Saturation`
/// (re-declared as a local `const` so it sits next to
/// `CLSID_D2D1_GAUSSIAN_BLUR` and follows this module's naming convention).
const CLSID_D2D1_SATURATION: GUID =
  GUID::from_u128(0x5cb2_d9cf_327d_459f_a0ce_40c0_b208_6bf7);

/// Hand-implemented D2D1 effect description with exactly one
/// runtime-adjustable scalar property (at index 0), plus any additional
/// fixed (non-adjustable) properties a given effect's D2D1 schema requires
/// but this code never tunes.
///
/// `Compositor::CreateEffectFactory` takes an `IGraphicsEffect` describing
/// an effect graph. `Win2D`'s convenience effect types (`GaussianBlurEffect`,
/// etc.) require the `Win2D` winmd, which `windows-rs`'s metadata-driven
/// binding generator cannot consume -- so this hand-implements the
/// `IGraphicsEffectD2D1Interop` COM shape directly against D2D1's built-in
/// effects, the same approach Microsoft's own
/// `Windows.UI.Composition-Win32-Samples` uses in C++. One instance of this
/// type is used per built-in effect ([`CLSID_D2D1_GAUSSIAN_BLUR`],
/// [`CLSID_D2D1_SATURATION`]) chained in `build_effect_brush`.
#[implement(IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectD2D1Interop)]
struct D2d1ScalarEffect {
  effect_id: GUID,
  source: IGraphicsEffectSource,
  /// Name of the single runtime-adjustable scalar property, matched
  /// against the dotted/bare property name in `GetNamedPropertyMapping`
  /// (see its doc comment for why both forms are checked).
  property_name: &'static str,
  /// Initial value baked into the effect graph at factory creation.
  /// Runtime adjustment rebuilds the whole brush rather than mutating this
  /// in place -- see `BlurVisual::set_blur_amount`'s doc comment for why.
  initial_value: f32,
  /// Additional fixed `u32` properties required by the effect's D2D1
  /// schema, in index order starting at index 1 (index 0 is always
  /// `initial_value`). Empty for saturation; Gaussian blur needs
  /// `[D2D1_GAUSSIANBLUR_OPTIMIZATION_PERFORMANCE, D2D1_BORDER_MODE_SOFT]` --
  /// `CreateEffectFactory` validates the description against D2D1's
  /// registered schema for the effect and fails with `E_INVALIDARG` unless
  /// all of them are present, even though only the scalar is
  /// runtime-adjustable here.
  extra_properties: &'static [u32],
  name: RefCell<HSTRING>,
}

impl D2d1ScalarEffect {
  fn new(
    effect_id: GUID,
    effect_name: &str,
    source: IGraphicsEffectSource,
    property_name: &'static str,
    initial_value: f32,
    extra_properties: &'static [u32],
  ) -> Self {
    Self {
      effect_id,
      source,
      property_name,
      initial_value,
      extra_properties,
      name: RefCell::new(HSTRING::from(effect_name)),
    }
  }
}

impl IGraphicsEffectSource_Impl for D2d1ScalarEffect {}

impl IGraphicsEffect_Impl for D2d1ScalarEffect {
  fn Name(&self) -> windows::core::Result<HSTRING> {
    Ok(self.name.borrow().clone())
  }

  fn SetName(&self, name: &HSTRING) -> windows::core::Result<()> {
    *self.name.borrow_mut() = name.clone();
    Ok(())
  }
}

impl IGraphicsEffectD2D1Interop_Impl for D2d1ScalarEffect {
  fn GetEffectId(&self) -> windows::core::Result<GUID> {
    Ok(self.effect_id)
  }

  fn GetNamedPropertyMapping(
    &self,
    name: &PCWSTR,
    index: *mut u32,
    mapping: *mut GRAPHICS_EFFECT_PROPERTY_MAPPING,
  ) -> windows::core::Result<()> {
    // SAFETY: `name` is a valid, null-terminated wide string for the
    // duration of this call, per the WinRT effect-description contract.
    let name = unsafe { name.to_string() }.unwrap_or_default();

    // Observed empirically (not documented): the composition engine calls
    // this with the *dotted* `"<Name>.<property>"` path, not the bare
    // property name alone. Match on the segment after the last `.` so this
    // works regardless of which convention is actually in play (both
    // dotted and bare names have been seen recommended in different
    // Microsoft samples).
    let property_name = name.rsplit('.').next().unwrap_or(&name);

    if property_name == self.property_name {
      // SAFETY: `index`/`mapping` are valid out-parameters supplied by the
      // composition engine for this call.
      unsafe {
        *index = 0;
        *mapping = GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT;
      }
      Ok(())
    } else {
      Err(windows::core::Error::from(E_INVALIDARG))
    }
  }

  fn GetPropertyCount(&self) -> windows::core::Result<u32> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(1 + self.extra_properties.len() as u32)
  }

  fn GetProperty(
    &self,
    index: u32,
  ) -> windows::core::Result<windows::Foundation::IPropertyValue> {
    if index == 0 {
      return PropertyValue::CreateSingle(self.initial_value)?.cast();
    }

    match self.extra_properties.get((index - 1) as usize) {
      Some(&value) => PropertyValue::CreateUInt32(value)?.cast(),
      None => Err(windows::core::Error::from(E_INVALIDARG)),
    }
  }

  fn GetSource(
    &self,
    index: u32,
  ) -> windows::core::Result<IGraphicsEffectSource> {
    if index == 0 {
      Ok(self.source.clone())
    } else {
      Err(windows::core::Error::from(E_INVALIDARG))
    }
  }

  fn GetSourceCount(&self) -> windows::core::Result<u32> {
    Ok(1)
  }
}

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
/// pipeline on every call. Falls back to the SWCA path on failure.
fn composition_thread() -> Option<&'static CompositionThread> {
  static COMPOSITION_THREAD: OnceLock<Option<CompositionThread>> =
    OnceLock::new();

  COMPOSITION_THREAD
    .get_or_init(|| match init_composition_thread() {
      Ok(thread) => Some(thread),
      Err(err) => {
        tracing::warn!(
          "Composition-based acrylic blur unavailable, falling back to \
           SWCA: {err}"
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
/// Every entry point into the pipeline needs the same three steps -- resolve
/// the thread, clone its agile handles into the closure, dispatch -- so they
/// live here rather than being repeated per visual type.
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

/// Runs `f` on the composition thread via its dispatcher queue and blocks
/// the calling thread for the result. Used for the one-time, async-sensitive
/// construction calls (`Compositor::new`, and per-overlay visual-tree
/// building, which touches the effect factory) -- see the module docs for
/// why these specifically must run there.
/// Queues `f` on the composition thread and returns immediately.
///
/// The blocking sibling below waits for a result on the WM's own thread,
/// which is right when the caller needs the value -- building a visual tree,
/// say. It is wrong for work whose only effect is on screen a frame or two
/// later, because the wait lands on the main loop: swapping an overlay to a
/// different baked surface used to block once per window per focus change,
/// and once per window *at once* on a workspace switch, which is felt as the
/// focus ring and backdrop lagging behind the keystroke.
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
  Color { A: color.a, B: color.b, G: color.g, R: color.r }
}

/// What paints the overlay's lower (blur) layer, and the state each source
/// needs to keep to stay live.
///
/// The two are not variations on one pipeline: acrylic samples the desktop
/// and blurs it every frame through a D2D effect graph, while the wallpaper
/// backdrop is a plain brush over an image blurred once, ahead of time (see
/// `wallpaper_surface`). Only the former has an effect graph to rebuild when
/// a knob changes; only the latter has to follow the window across monitors.
enum Backdrop {
  /// Live host-backdrop brush fed through the Gaussian/saturation graph.
  Acrylic {
    /// The graph's source. Retained (rather than just used during `create`)
    /// so `set_blur_amount`/`set_saturation` can rebuild the effect brush
    /// around it -- see `set_blur_amount`'s doc comment for why a rebuild,
    /// not an in-place property update, is used.
    host_backdrop: CompositionBackdropBrush,
    effect_brush: CompositionEffectBrush,
  },

  /// A flat colour fill. No sampling, no blur, no baked surface: at an
  /// opaque `tint` this is one opaque visual and nothing else, which is why
  /// it is the cheapest style rather than merely a cheap one.
  ///
  /// Carries no state -- the fill is `tint`, which `set_tint` already
  /// applies to the sprite's brush.
  Solid,

  /// A crop of the monitor's pre-blurred, opaque wallpaper surface.
  Wallpaper {
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
  },
}

/// A live `Windows.UI.Composition` visual tree providing an overlay's
/// rendering: a blur layer (see [`Backdrop`]) with a tint layer composited
/// on top, both clipped to a continuous rounded rectangle.
pub(crate) struct BlurVisual {
  /// Binds the visual tree to the overlay's `HWND`. Kept alive but never
  /// touched again -- dropping it would unbind composition from the window.
  _target: DesktopWindowTarget,

  /// Retained (rather than just used during `create`) so the knob setters
  /// can rebuild whatever their [`Backdrop`] needs rebuilt.
  compositor: Compositor,
  queue: DispatcherQueue,
  backdrop: Backdrop,
  root: ContainerVisual,
  blur_sprite: SpriteVisual,
  tint_sprite: SpriteVisual,

  tint_brush: CompositionColorBrush,

  /// Darkens the overlay toward its own edges.
  ///
  /// A visual rather than a stage in the wallpaper bake, because the bake is
  /// shared by every window on the monitor: baked in, the falloff anchors to
  /// the screen, so a window at the edge gets a uniformly dark crop and one
  /// in the middle gets the bright centre. Here it is measured from each
  /// window's own rect, which is what a vignette means.
  ///
  /// `MappingMode::Relative` expresses the gradient in fractions of the
  /// sprite, so a resize needs no update to the brush at all -- only the
  /// sprite itself is resized, alongside the others in `set_rect`.
  vignette_brush: CompositionRadialGradientBrush,
  vignette_sprite: SpriteVisual,
  rounded_geometry: CompositionRoundedRectangleGeometry,

  /// Stands in for the window content a mid-animation surrogate has not
  /// captured yet, in the two strips its DWM thumbnail does not reach.
  ///
  /// Two sprites rather than one because the uncovered area is an L: the
  /// thumbnail is anchored top-left, so what is left over is a strip down
  /// the right and a strip along the bottom. A single sprite would have to
  /// cover the thumbnail as well, and being composited *under* a
  /// part-transparent thumbnail it would tint the content too -- the whole
  /// window would read as solid, which is the bug this replaces.
  ///
  /// Painted here rather than on the surrogate because the surrogate can
  /// only ask for a solid backdrop through SWCA, and an SWCA accent
  /// renders opaque whatever alpha it is given. A sprite takes a real
  /// opacity, so the fill can match the window's own `transparency` and
  /// sit over the backdrop the way the settled window does.
  gap_brush: CompositionColorBrush,
  gap_right: SpriteVisual,
  gap_bottom: SpriteVisual,

  /// Everything baked into the wallpaper image. Kept whole so any one
  /// setter can re-render using the others' current values -- acrylic reads
  /// only `blur_amount`/`saturation` from it, since the remaining knobs are
  /// unreachable through `CreateEffectFactory`.
  knobs: BakeKnobs,

  /// How far the wallpaper crop follows the window. Not part of `knobs`:
  /// it selects a different region of an already-baked surface rather than
  /// changing what was baked, so a change costs one property write.
  parallax: f32,
}

impl BlurVisual {
  /// Builds a new visual tree for `hwnd`, sized to `rect`, and roots it.
  ///
  /// Runs on the dedicated composition thread (see the module docs); the
  /// returned `BlurVisual`'s composition objects are agile and can be
  /// mutated from any thread afterwards.
  pub(crate) fn create(
    hwnd: HWND,
    rect: &Rect,
    params: BlurOverlayParams,
  ) -> crate::Result<Self> {
    let hwnd_raw = hwnd.0;
    let rect = rect.clone();

    with_composition_thread(move |compositor, queue| {
      build_visual_tree(&compositor, &queue, HWND(hwnd_raw), &rect, params)
    })
  }

  /// Resizes the visual tree's clip and both child visuals to match `rect`.
  /// Does not reposition the `HWND` itself -- callers still issue their own
  /// `SetWindowPos`, exactly as with the SWCA path.
  ///
  /// Must resize `root`/`blur_sprite`/`tint_sprite` in addition to the clip
  /// geometry -- they're independently-sized visuals set once in
  /// `build_visual_tree` and never otherwise touched, so leaving them out
  /// here left them pinned at their creation-time size while only the clip
  /// grew, showing blur/tint over just the original area and nothing over
  /// the rest whenever the overlay's `HWND` was resized after creation.
  pub(crate) fn set_rect(&mut self, rect: &Rect) -> crate::Result<()> {
    let size = Vector2 {
      X: pixels_to_dips(rect.width()),
      Y: pixels_to_dips(rect.height()),
    };
    self.root.SetSize(size)?;
    self.blur_sprite.SetSize(size)?;
    self.tint_sprite.SetSize(size)?;
    self.vignette_sprite.SetSize(size)?;
    self.rounded_geometry.SetSize(size)?;

    self.sync_crop(rect)?;
    Ok(())
  }

  /// Keeps the wallpaper backdrop showing the part of the desktop the
  /// overlay now covers. No-op for acrylic, which samples live and so needs
  /// no notion of where it is.
  ///
  /// Re-binds to another monitor's baked surface only when the overlay has
  /// actually crossed onto one -- checked arithmetically against the cached
  /// bounds first, so the per-tick case during an animation costs one
  /// property write and no system calls.
  fn sync_crop(&mut self, rect: &Rect) -> crate::Result<()> {
    let knobs = self.knobs;
    let parallax = self.parallax;
    let compositor = self.compositor.clone();
    let queue = self.queue.clone();
    let current = wallpaper_surface::generation();

    let Backdrop::Wallpaper { brush, monitor, generation } =
      &mut self.backdrop
    else {
      return Ok(());
    };

    // The generation check has to force a re-bind even when the overlay has
    // not moved: the monitor it sits on is unchanged, but the image baked
    // for that monitor is no longer the one the desktop is showing.
    if *generation != current || !monitor.contains_point(&rect.center_point())
    {
      let bounds = wallpaper_surface::monitor_bounds(rect);
      let rebound = brush.clone();
      let target = bounds.clone();

      // Queued, not awaited: this runs from the per-tick sync path, and the
      // new crop being on screen a frame later is invisible next to blocking
      // the main loop until it is.
      dispatch_on_composition_thread(&queue, move || {
        if let Err(err) =
          wallpaper_surface::rebind(&compositor, &rebound, &target, knobs)
        {
          tracing::warn!("Wallpaper backdrop re-bind failed: {err}.");
        }
      })?;

      *monitor = bounds;
      *generation = current;
    }

    wallpaper_surface::set_crop(brush, rect, monitor, parallax);
    Ok(())
  }

  /// Re-binds the wallpaper backdrop when the desktop it was baked from has
  /// changed, and does nothing otherwise.
  ///
  /// Called on every sync tick, so the no-change path is deliberately one
  /// relaxed atomic load and a discriminant check -- no shell query, no
  /// filesystem stat, and no composition property write. Acrylic samples
  /// live and has nothing to go stale.
  pub(crate) fn sync_backdrop(&mut self, rect: &Rect) -> crate::Result<()> {
    let Backdrop::Wallpaper { generation, .. } = &self.backdrop else {
      return Ok(());
    };

    // Throttled internally to one shell query every couple of seconds, so
    // calling it from every overlay on every tick is fine.
    wallpaper_surface::poll_for_changes();

    if *generation == wallpaper_surface::generation() {
      return Ok(());
    }

    self.sync_crop(rect)
  }

  /// Re-bakes the wallpaper surface at the given knobs and points this
  /// overlay's brush at the result. No-op for acrylic.
  ///
  /// Only reached on a config reload: `blur_amount` and `saturation` are
  /// baked into the image rather than evaluated per frame, which is the
  /// whole reason the style is cheap, so changing either means rendering a
  /// new one.
  fn rebake(&self, knobs: BakeKnobs) -> crate::Result<()> {
    let Backdrop::Wallpaper { brush, monitor, .. } = &self.backdrop else {
      return Ok(());
    };

    let compositor = self.compositor.clone();
    let brush = brush.clone();
    let monitor = monitor.clone();

    dispatch_on_composition_thread(&self.queue, move || {
      if let Err(err) =
        wallpaper_surface::rebind(&compositor, &brush, &monitor, knobs)
      {
        tracing::warn!("Wallpaper backdrop re-bake failed: {err}.");
      }
    })
  }

  /// Updates the tint layer's color; no-op unless the value changed.
  /// Paints `color` at `opacity` over the two strips of this overlay that
  /// the surrogate's thumbnail does not cover, and clears them when there
  /// is nothing uncovered.
  ///
  /// `covered` is the thumbnail's size, `full` the overlay's; both in
  /// physical pixels, both anchored top-left, which is where DWM draws the
  /// thumbnail. Passing a `covered` at least as large as `full` on both
  /// axes hides the fill, which is the steady state for a pure move or a
  /// shrink.
  pub(crate) fn set_gap_fill(
    &self,
    color: Option<crate::Color>,
    opacity: f32,
    covered: (i32, i32),
    full: (i32, i32),
  ) -> crate::Result<()> {
    let right_w = (full.0 - covered.0).max(0);
    let bottom_h = (full.1 - covered.1).max(0);
    let Some(color) = color.filter(|_| right_w > 0 || bottom_h > 0) else {
      self.gap_right.SetSize(Vector2 { X: 0.0, Y: 0.0 })?;
      self.gap_bottom.SetSize(Vector2 { X: 0.0, Y: 0.0 })?;
      return Ok(());
    };

    self.gap_brush.SetColor(to_ui_color(color))?;

    // The right strip takes the full height and the bottom strip only the
    // covered width, so the two meet without overlapping -- overlapping
    // would double-composite the corner and show it darker than the rest.
    self.gap_right.SetOffset(Vector3 {
      X: pixels_to_dips(covered.0),
      Y: 0.0,
      Z: 0.0,
    })?;
    self.gap_right.SetSize(Vector2 {
      X: pixels_to_dips(right_w),
      Y: pixels_to_dips(full.1),
    })?;
    self.gap_right.SetOpacity(opacity)?;

    self.gap_bottom.SetOffset(Vector3 {
      X: 0.0,
      Y: pixels_to_dips(covered.1),
      Z: 0.0,
    })?;
    self.gap_bottom.SetSize(Vector2 {
      X: pixels_to_dips(covered.0.min(full.0)),
      Y: pixels_to_dips(bottom_h),
    })?;
    self.gap_bottom.SetOpacity(opacity)?;

    Ok(())
  }

  pub(crate) fn set_tint(&self, tint: crate::Color) -> crate::Result<()> {
    self.tint_brush.SetColor(to_ui_color(tint))?;

    // For `Solid` the tint *is* the backdrop, so it has to reach the fill
    // sprite too. Painting both leaves the colour composited over itself,
    // which is a no-op at any alpha.
    if matches!(self.backdrop, Backdrop::Solid) {
      let fill =
        self.compositor.CreateColorBrushWithColor(to_ui_color(tint))?;
      self.blur_sprite.SetBrush(&fill)?;
    }

    Ok(())
  }

  /// Updates the live blur radius.
  ///
  /// For acrylic this rebuilds the effect brush; the rest of this comment
  /// is about why a rebuild rather than an in-place property update. The
  /// wallpaper backdrop has no live graph to update at all and re-bakes its
  /// surface instead (see `rebake`).
  ///
  /// The plan's original design mutated the existing brush in place via
  /// `effect_brush.Properties().InsertScalar("Blur.BlurAmount", value)`
  /// (the effect graph's `Name`-prefixed named-property system), matching
  /// the pattern shown for a *static* setup in Microsoft's own samples.
  /// Confirmed by actually running the real integration (not just the
  /// spike's automated checks) that this path reliably fails with
  /// `E_INVALIDARG` here, regardless of whether `GetNamedPropertyMapping`
  /// is queried with `"BlurAmount"` or `"Blur.BlurAmount"` -- consistent
  /// with `InsertScalar` needing the property to have been registered via
  /// `Compositor::CreateEffectFactoryWithProperties`'s `animatableProperties`
  /// list (which requires an `IIterable<HSTRING>`, not constructible from a
  /// `Vec` in this `windows-rs` version without hand-implementing the
  /// `WinRT` iterator interfaces) rather than the plain `CreateEffectFactory`
  /// this code uses. Rebuilding the brush instead reuses only the
  /// `GetProperty`-based initial-value path, which is confirmed working
  /// (overlays visibly render blur from their initial `blur_amount`).
  pub(crate) fn set_blur_amount(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.blur_amount = value;
    self.reapply_knobs(knobs)
  }

  /// Re-renders the blur layer at the given knob values, however this
  /// overlay's [`Backdrop`] produces it.
  fn reapply_knobs(&mut self, knobs: BakeKnobs) -> crate::Result<()> {
    self.knobs = knobs;

    match &self.backdrop {
      Backdrop::Acrylic { host_backdrop, .. } => {
        let compositor = self.compositor.clone();
        let host_backdrop = host_backdrop.clone();
        let (blur_amount, saturation) =
          (knobs.blur_amount, knobs.saturation);

        let effect_brush = run_on_composition_thread(&self.queue, move || {
          build_effect_brush(
            &compositor,
            &host_backdrop,
            blur_amount,
            saturation,
          )
        })?;

        self.blur_sprite.SetBrush(&effect_brush)?;

        if let Backdrop::Acrylic { effect_brush: current, .. } =
          &mut self.backdrop
        {
          *current = effect_brush;
        }

        Ok(())
      }
      Backdrop::Wallpaper { .. } => self.rebake(knobs),
      Backdrop::Solid => Ok(()),
    }
  }

  /// Updates the live saturation. Both knobs feed one render -- acrylic's
  /// effect graph or the wallpaper bake -- so either setter re-runs it
  /// using the other's current stored value.
  pub(crate) fn set_saturation(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.saturation = value;
    self.reapply_knobs(knobs)
  }

  /// Updates the exposure baked into the wallpaper image. No-op for
  /// acrylic, whose effect factory renders `Exposure` as a pass-through.
  pub(crate) fn set_exposure(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.exposure = value;
    self.rebake_only(knobs)
  }

  /// Updates the contrast baked into the wallpaper image. Wallpaper only,
  /// same reason as [`set_exposure`].
  ///
  /// [`set_exposure`]: BlurVisual::set_exposure
  pub(crate) fn set_contrast(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.contrast = value;
    self.rebake_only(knobs)
  }

  /// Updates the highlight recovery baked into the wallpaper image.
  /// Wallpaper only, same reason as [`set_exposure`].
  ///
  /// [`set_exposure`]: BlurVisual::set_exposure
  pub(crate) fn set_highlights(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.highlights = value;
    self.rebake_only(knobs)
  }

  /// Updates the shadow lift baked into the wallpaper image. Wallpaper
  /// only, same reason as [`set_exposure`].
  ///
  /// [`set_exposure`]: BlurVisual::set_exposure
  pub(crate) fn set_shadows(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.shadows = value;
    self.rebake_only(knobs)
  }

  /// Updates the vignette.
  ///
  /// Unlike the other grading knobs this touches no baked image, so it
  /// applies to every style and costs one brush rebuild -- no re-render of
  /// anything, and nothing per frame.
  pub(crate) fn set_vignette(&mut self, value: f32) -> crate::Result<()> {
    let brush = build_vignette_brush(&self.compositor, value)?;
    self.vignette_sprite.SetBrush(&brush)?;
    self.vignette_brush = brush;
    Ok(())
  }

  /// Updates the grain baked into the wallpaper image. Wallpaper only, same
  /// reason as [`set_exposure`].
  ///
  /// [`set_exposure`]: BlurVisual::set_exposure
  pub(crate) fn set_grain(&mut self, value: f32) -> crate::Result<()> {
    let mut knobs = self.knobs;
    knobs.grain = value;
    self.rebake_only(knobs)
  }

  /// Updates how far the crop follows the window, re-applying it at
  /// `rect` so the change shows without waiting for the window to move.
  pub(crate) fn set_parallax(&mut self, value: f32, rect: &Rect) {
    self.parallax = value;

    if let Backdrop::Wallpaper { brush, monitor, .. } = &self.backdrop {
      wallpaper_surface::set_crop(brush, rect, monitor, value);
    }
  }

  /// Stores `knobs` and re-bakes, without acrylic's brush rebuild.
  ///
  /// For the four knobs acrylic cannot express at all, so that setting one
  /// on an acrylic overlay does nothing rather than pointlessly rebuilding
  /// an effect graph that would render identically.
  fn rebake_only(&mut self, knobs: BakeKnobs) -> crate::Result<()> {
    self.knobs = knobs;

    match &self.backdrop {
      Backdrop::Acrylic { .. } | Backdrop::Solid => Ok(()),
      Backdrop::Wallpaper { .. } => self.rebake(knobs),
    }
  }

  /// Updates the clip's corner radius.
  pub(crate) fn set_corner_radius(&self, value: f32) -> crate::Result<()> {
    self
      .rounded_geometry
      .SetCornerRadius(Vector2 { X: value, Y: value })?;
    Ok(())
  }

  /// Updates the overlay's own opacity. `root` sits above both
  /// `blur_sprite` and `tint_sprite`, so this fades the whole composited
  /// overlay (blur + tint together) as one unit -- a plain `Visual`
  /// property, not an effect-graph one, so unlike `set_blur_amount` this
  /// never needs a brush rebuild.
  pub(crate) fn set_opacity(&self, value: f32) -> crate::Result<()> {
    self.root.SetOpacity(value)?;
    Ok(())
  }
}

/// Builds the radial gradient that darkens an overlay toward its edges.
///
/// Transparent across the middle and reaching `strength` alpha at the
/// corners. The ellipse is deliberately larger than the sprite
/// (`radius > 0.5` in relative units) so the darkest point falls outside the
/// visible area: a gradient that reached full strength exactly at the edge
/// puts its steepest part on screen and reads as a ring rather than shading.
///
/// A `strength` of zero still builds a brush, fully transparent. Skipping
/// the visual entirely would mean rebuilding the tree when the knob is first
/// raised, and a transparent visual costs DWM nothing to composite.
fn build_vignette_brush(
  compositor: &Compositor,
  strength: f32,
) -> windows::core::Result<CompositionRadialGradientBrush> {
  let brush = compositor.CreateRadialGradientBrush()?;

  // Relative to the sprite, so resizing the overlay needs no update here.
  brush.SetMappingMode(CompositionMappingMode::Relative)?;
  brush.SetEllipseCenter(Vector2 { X: 0.5, Y: 0.5 })?;
  brush.SetEllipseRadius(Vector2 { X: 0.75, Y: 0.75 })?;

  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  let alpha = (strength.clamp(0.0, 1.0) * 255.0).round() as u8;

  let clear = Color { A: 0, R: 0, G: 0, B: 0 };
  let dark = Color { A: alpha, R: 0, G: 0, B: 0 };

  let stops = brush.ColorStops()?;
  stops.Append(&compositor.CreateColorGradientStopWithOffsetAndColor(0.0, clear)?)?;
  stops.Append(&compositor.CreateColorGradientStopWithOffsetAndColor(0.45, clear)?)?;
  stops.Append(&compositor.CreateColorGradientStopWithOffsetAndColor(1.0, dark)?)?;

  Ok(brush)
}

/// `DesktopWindowTarget` sizes composition visuals 1:1 against the HWND's
/// actual client pixel size (no DPI virtualization layer here, unlike
/// XAML/UWP) -- so this is a passthrough today. Named/kept separate from a
/// bare cast so a future DPI-aware sizing adjustment has a single call site.
#[allow(clippy::cast_precision_loss, clippy::unnecessary_wraps)]
fn pixels_to_dips(pixels: i32) -> f32 {
  pixels as f32
}

/// Builds a fresh effect brush (`host_backdrop` -> Gaussian blur ->
/// saturation -> brush) at the given knob values. Split out from
/// `build_visual_tree` so `BlurVisual::set_blur_amount`/`set_saturation`
/// can call it again on demand -- see `set_blur_amount`'s doc comment for
/// why a rebuild, not an in-place property update, is used.
///
/// Both stages are described in one `IGraphicsEffect` graph passed to a
/// single `CreateEffectFactory` call, producing one brush -- not two
/// independently-chained brushes. `SetSourceParameter("Source", ..)` binds
/// `host_backdrop` to the *inner* (blur) node's named parameter; the outer
/// (saturation) node's own source is the blur node's `IGraphicsEffectSource`
/// directly (an internal graph edge via `GetSource`, not a named
/// parameter), and the composition engine resolves the "Source" name
/// lookup through to it regardless of nesting depth.
///
/// `exposure`, `vignette`, and `grain` were tried and dropped.
/// `Windows.UI.Composition`'s `CreateEffectFactory` accepts a curated
/// subset of D2D1 built-in effects at *construction* time, but `Exposure`
/// (`CLSID_D2D1Exposure`) was a no-op at render time despite constructing
/// without error and being byte-identical in its wrapper to `Saturation`,
/// which renders correctly -- observed with no visible change even at
/// extreme values well outside the documented `-2..2` range, so this
/// isn't a subtlety-of-effect issue, just an unsupported effect that
/// silently degrades to pass-through instead of failing loudly.
/// `CLSID_D2D1Vignette` plus the `Turbulence`/`Composite` combination
/// grain needed both failed outright with `E_INVALIDARG` ("Unsupported
/// effect type") at construction, confirmed via bisection. `HueRotation`
/// was also tried and dropped -- `CreateEffectFactory` accepted it fine,
/// but with no persistent tint the live desktop content behind the
/// overlay had too little color for a rotation to visibly do anything.
fn build_effect_brush(
  compositor: &Compositor,
  host_backdrop: &CompositionBackdropBrush,
  blur_amount: f32,
  saturation: f32,
) -> windows::core::Result<CompositionEffectBrush> {
  let source_param =
    CompositionEffectSourceParameter::Create(&HSTRING::from("Source"))?;
  let blur_effect: IGraphicsEffectSource = D2d1ScalarEffect::new(
    CLSID_D2D1_GAUSSIAN_BLUR,
    "Blur",
    source_param.cast()?,
    "BlurAmount",
    blur_amount,
    &[D2D1_GAUSSIANBLUR_OPTIMIZATION_PERFORMANCE, D2D1_BORDER_MODE_SOFT],
  )
  .into();
  let saturation_effect: IGraphicsEffect = D2d1ScalarEffect::new(
    CLSID_D2D1_SATURATION,
    "Saturation",
    blur_effect,
    "Saturation",
    saturation,
    &[],
  )
  .into();
  let effect_factory = compositor.CreateEffectFactory(&saturation_effect)?;
  let effect_brush = effect_factory.CreateBrush()?;
  effect_brush.SetSourceParameter(&HSTRING::from("Source"), host_backdrop)?;
  Ok(effect_brush)
}

/// Builds the full visual tree: a `ContainerVisual` rooting a blur sprite
/// (whichever [`Backdrop`] `params.style` selects) and a tint sprite (flat
/// color) stacked above it, both clipped by a shared rounded rectangle
/// geometry.
fn build_visual_tree(
  compositor: &Compositor,
  queue: &DispatcherQueue,
  hwnd: HWND,
  rect: &Rect,
  params: BlurOverlayParams,
) -> windows::core::Result<BlurVisual> {
  // SAFETY: `hwnd` is a valid, already-created top-level window.
  let target = unsafe {
    compositor
      .cast::<ICompositorDesktopInterop>()?
      .CreateDesktopWindowTarget(hwnd, false)?
  };

  let width = pixels_to_dips(rect.width());
  let height = pixels_to_dips(rect.height());
  let size = Vector2 { X: width, Y: height };

  let rounded_geometry = compositor.CreateRoundedRectangleGeometry()?;
  rounded_geometry.SetSize(size)?;
  rounded_geometry.SetCornerRadius(Vector2 {
    X: params.corner_radius,
    Y: params.corner_radius,
  })?;
  let clip = compositor.CreateGeometricClipWithGeometry(&rounded_geometry)?;

  let blur_sprite = compositor.CreateSpriteVisual()?;
  blur_sprite.SetSize(size)?;

  let backdrop = if params.style == BackdropStyle::Solid {
    let fill = compositor.CreateColorBrushWithColor(to_ui_color(params.tint))?;
    blur_sprite.SetBrush(&fill)?;
    Backdrop::Solid
  } else if params.style == BackdropStyle::Wallpaper {
    let (brush, monitor) =
      wallpaper_surface::crop_brush(compositor, rect, params)?;

    blur_sprite.SetBrush(&brush)?;
    Backdrop::Wallpaper {
      brush,
      monitor,
      generation: wallpaper_surface::generation(),
    }
  } else {
    let host_backdrop = compositor.CreateHostBackdropBrush()?;
    let effect_brush = build_effect_brush(
      compositor,
      &host_backdrop,
      params.blur_amount,
      params.saturation,
    )?;

    blur_sprite.SetBrush(&effect_brush)?;
    Backdrop::Acrylic { host_backdrop, effect_brush }
  };

  let tint_brush =
    compositor.CreateColorBrushWithColor(to_ui_color(params.tint))?;
  let tint_sprite = compositor.CreateSpriteVisual()?;
  tint_sprite.SetBrush(&tint_brush)?;
  tint_sprite.SetSize(size)?;

  let vignette_brush = build_vignette_brush(compositor, params.vignette)?;
  let vignette_sprite = compositor.CreateSpriteVisual()?;
  vignette_sprite.SetBrush(&vignette_brush)?;
  vignette_sprite.SetSize(size)?;

  // Zero-sized until a resize session actually uncovers something; see the
  // field docs. Topmost so the fill reads as window content sitting on the
  // backdrop, not as another layer of backdrop.
  let gap_brush =
    compositor.CreateColorBrushWithColor(to_ui_color(params.tint))?;
  let gap_right = compositor.CreateSpriteVisual()?;
  gap_right.SetBrush(&gap_brush)?;
  gap_right.SetSize(Vector2 { X: 0.0, Y: 0.0 })?;
  let gap_bottom = compositor.CreateSpriteVisual()?;
  gap_bottom.SetBrush(&gap_brush)?;
  gap_bottom.SetSize(Vector2 { X: 0.0, Y: 0.0 })?;

  let root = compositor.CreateContainerVisual()?;
  root.SetSize(size)?;
  root.SetClip(&clip)?;
  root.SetOpacity(params.opacity)?;
  root.Children()?.InsertAtTop(&blur_sprite)?;
  root.Children()?.InsertAtTop(&tint_sprite)?;
  root.Children()?.InsertAtTop(&vignette_sprite)?;
  root.Children()?.InsertAtTop(&gap_right)?;
  root.Children()?.InsertAtTop(&gap_bottom)?;

  target.SetRoot(&root)?;

  Ok(BlurVisual {
    _target: target,
    compositor: compositor.clone(),
    queue: queue.clone(),
    backdrop,
    root,
    blur_sprite,
    tint_sprite,
    tint_brush,
    vignette_brush,
    vignette_sprite,
    gap_brush,
    gap_right,
    gap_bottom,
    rounded_geometry,
    knobs: params.into(),
    parallax: params.parallax,
  })
}


/// A live `Windows.UI.Composition` visual tree providing a border overlay's
/// rendering: a single rounded rectangle *stroked* with a solid color, so
/// only the ring band is ever painted and the interior stays fully
/// transparent. Considerably lighter than [`BlurVisual`] -- no effect
/// graph, no live backdrop sampling, just one stroked shape.
///
/// `NativeBorderOverlay` sizes and positions the overlay's `HWND` to the
/// tracked window's rect *outset* by the configured border width, directly
/// behind the real window in z-order (see its `anchor` field doc, same
/// mechanism [`BlurVisual`]'s pairing already relies on). The stroke is
/// that border width thick and its geometry is inset by half of it, so the
/// ring's outer edge lands exactly on the overlay's outer rect and its
/// inner edge exactly on the tracked window's own rect.
///
/// This replaces an earlier fill-plus-hole-punch design, whose
/// `SetWindowRgn` region rebuild cost ~3.2ms per frame across a
/// five-window resize burst -- the largest single border-attributable cost
/// in that profile -- and whose `CreateRoundRectRgn` hole only
/// approximated the inner curve. A stroked shape needs no window region at
/// all, and rounds the ring's inner *and* outer corners exactly. An
/// earlier version of this comment claimed `Windows.UI.Composition`
/// exposed no stroke-shape API in this crate's bound surface; that was
/// wrong -- `Compositor::CreateShapeVisual`,
/// `CreateSpriteShapeWithGeometry` and `ShapeVisual::Shapes` are all bound
/// in `windows` 0.52.
///
/// The SWCA fallback path has no equivalent, so `NativeBorderOverlay`
/// keeps the region punch there and only there.
pub(crate) struct BorderVisual {
  /// Binds the visual tree to the overlay's `HWND`. Kept alive but never
  /// touched again -- dropping it would unbind composition from the window.
  _target: DesktopWindowTarget,

  /// Root of the tree, holding the single stroked shape. A `ShapeVisual`
  /// derives `ContainerVisual`, so this doubles as the size/opacity knob
  /// the previous design needed a separate `ContainerVisual` for.
  root: ShapeVisual,
  shape: CompositionSpriteShape,
  stroke_brush: CompositionColorBrush,
  geometry: CompositionRoundedRectangleGeometry,

  /// Last-applied ring inputs, so any one of `set_rect`/`set_width`/
  /// `set_corner_radius` can recompute the derived geometry (which depends
  /// on all three) from the other two's current values.
  ring: Cell<Ring>,
}

/// The inputs a stroked ring's geometry is derived from: the overlay's own
/// (already-outset) size in pixels, the border width the stroke is drawn
/// at, and the ring's *outer* corner radius.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ring {
  size: Vector2,
  width: f32,
  corner_radius: f32,
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
/// Returns `(offset, size, corner_radius)` for the geometry, each clamped
/// so a width exceeding the overlay's own size (or its corner radius)
/// degenerates gracefully instead of producing a negative extent, which
/// composition rejects.
fn ring_geometry(ring: Ring) -> (Vector2, Vector2, f32) {
  let width = ring.width.max(0.0);
  let inset = width / 2.0;

  let offset = Vector2 { X: inset, Y: inset };
  let size = Vector2 {
    X: (ring.size.X - width).max(0.0),
    Y: (ring.size.Y - width).max(0.0),
  };
  let corner_radius = (ring.corner_radius - inset).max(0.0);

  (offset, size, corner_radius)
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

  /// Re-derives and applies the stroke thickness and geometry for `ring`,
  /// storing it as the new baseline for the next partial update.
  fn apply_ring(&self, ring: Ring) -> windows::core::Result<()> {
    let (offset, size, corner_radius) = ring_geometry(ring);

    self.root.SetSize(ring.size)?;
    self.shape.SetStrokeThickness(ring.width.max(0.0))?;
    self.geometry.SetOffset(offset)?;
    self.geometry.SetSize(size)?;
    self
      .geometry
      .SetCornerRadius(Vector2 { X: corner_radius, Y: corner_radius })?;

    self.ring.set(ring);
    Ok(())
  }

  /// Resizes the ring to match `rect`. Does not reposition the `HWND`
  /// itself -- callers still issue their own `SetWindowPos`.
  pub(crate) fn set_rect(&self, rect: &Rect) -> crate::Result<()> {
    let size = Vector2 {
      X: pixels_to_dips(rect.width()),
      Y: pixels_to_dips(rect.height()),
    };

    Ok(self.apply_ring(Ring { size, ..self.ring.get() })?)
  }

  /// Updates the ring's color.
  pub(crate) fn set_color(&self, color: crate::Color) -> crate::Result<()> {
    self.stroke_brush.SetColor(to_ui_color(color))?;
    Ok(())
  }

  /// Updates the border width, i.e. the stroke's thickness.
  ///
  /// The overlay's `HWND` is outset by this same width, so callers must
  /// resize the window (and hence call [`set_rect`]) to match -- this only
  /// updates the band drawn inside it.
  ///
  /// [`set_rect`]: BorderVisual::set_rect
  pub(crate) fn set_width(&self, width: f32) -> crate::Result<()> {
    Ok(self.apply_ring(Ring { width, ..self.ring.get() })?)
  }

  /// Updates the ring's outer corner radius.
  pub(crate) fn set_corner_radius(&self, value: f32) -> crate::Result<()> {
    Ok(self.apply_ring(Ring { corner_radius: value, ..self.ring.get() })?)
  }

  /// Updates the overlay's own opacity.
  pub(crate) fn set_opacity(&self, value: f32) -> crate::Result<()> {
    self.root.SetOpacity(value)?;
    Ok(())
  }

  /// Translates the ring within its `HWND`, in pixels relative to the
  /// window's top-left.
  ///
  /// Used by the workspace-switch slide, where the overlay window stays
  /// pinned to the monitor viewport and only its content moves: one
  /// property write per frame instead of a `SetWindowPos` plus a geometry
  /// rebuild. Content driven outside the window's bounds is clipped by the
  /// `DesktopWindowTarget`, so a ring sliding off the monitor is cut at the
  /// edge rather than spilling onto the neighbouring one.
  pub(crate) fn set_offset(&self, x: i32, y: i32) -> crate::Result<()> {
    self.root.SetOffset(Vector3 {
      X: pixels_to_dips(x),
      Y: pixels_to_dips(y),
      Z: 0.0,
    })?;
    Ok(())
  }
}

/// Builds the full visual tree: a `ShapeVisual` rooting a single
/// `CompositionSpriteShape` that strokes a rounded rectangle in the border
/// color. No fill brush is set, so the shape's interior stays transparent
/// and the tracked window shows through with no window region, mask, or
/// reliance on that window occluding a fill.
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

  let stroke_brush =
    compositor.CreateColorBrushWithColor(to_ui_color(params.color))?;
  let shape = compositor.CreateSpriteShapeWithGeometry(&geometry)?;
  shape.SetStrokeBrush(&stroke_brush)?;

  let root = compositor.CreateShapeVisual()?;
  root.SetOpacity(params.opacity)?;
  root.Shapes()?.Append(&shape)?;

  target.SetRoot(&root)?;

  let visual = BorderVisual {
    _target: target,
    root,
    shape,
    stroke_brush,
    geometry,
    ring: Cell::new(Ring {
      size: Vector2 { X: 0.0, Y: 0.0 },
      width: params.width,
      corner_radius: params.corner_radius,
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
  })?;

  Ok(visual)
}

#[cfg(test)]
mod tests {
  use windows::Foundation::Numerics::Vector2;

  use super::{ring_geometry, Ring};

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
    });

    assert_eq!(offset, Vector2 { X: 0.0, Y: 0.0 });
    assert_eq!(size, Vector2 { X: 100.0, Y: 50.0 });
  }
}
