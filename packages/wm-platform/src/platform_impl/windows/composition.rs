//! `Windows.UI.Composition` based acrylic blur pipeline.
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
//! effect factory's shader-graph compile -- to ever complete.
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
  Foundation::{Numerics::Vector2, PropertyValue},
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
      CompositionSpriteShape, Compositor, ContainerVisual,
      Desktop::DesktopWindowTarget, ShapeVisual, SpriteVisual,
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

use crate::{BlurOverlayParams, BorderOverlayParams, Rect};

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

/// Runs `f` on the composition thread via its dispatcher queue and blocks
/// the calling thread for the result. Used for the one-time, async-sensitive
/// construction calls (`Compositor::new`, and per-overlay visual-tree
/// building, which touches the effect factory) -- see the module docs for
/// why these specifically must run there.
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

/// A live `Windows.UI.Composition` visual tree providing an acrylic blur
/// overlay's rendering: a live host-backdrop brush, blurred through a
/// Gaussian-blur effect graph, with a tint layer composited on top, both
/// clipped to a continuous rounded rectangle.
pub(crate) struct BlurVisual {
  /// Binds the visual tree to the overlay's `HWND`. Kept alive but never
  /// touched again -- dropping it would unbind composition from the window.
  _target: DesktopWindowTarget,

  /// Retained (rather than just used during `create`) so
  /// `set_blur_amount`/`set_saturation` can rebuild the effect brush --
  /// see `set_blur_amount`'s doc comment for why a rebuild, not an
  /// in-place property update, is used.
  compositor: Compositor,
  queue: DispatcherQueue,
  host_backdrop: CompositionBackdropBrush,
  root: ContainerVisual,
  blur_sprite: SpriteVisual,
  tint_sprite: SpriteVisual,

  effect_brush: CompositionEffectBrush,
  tint_brush: CompositionColorBrush,
  rounded_geometry: CompositionRoundedRectangleGeometry,

  /// Current blur amount, kept alongside `saturation` so either setter can
  /// rebuild the full effect graph using the other's current value.
  blur_amount: f32,
  /// Current saturation. See `blur_amount`.
  saturation: f32,
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
    let thread = composition_thread().ok_or_else(|| {
      crate::Error::Platform(
        "Composition pipeline unavailable.".to_string(),
      )
    })?;

    let compositor = thread.compositor.clone();
    let queue = thread.queue.clone();
    let hwnd_raw = hwnd.0;
    let rect = rect.clone();

    run_on_composition_thread(&thread.queue, move || {
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
  pub(crate) fn set_rect(&self, rect: &Rect) -> crate::Result<()> {
    let size = Vector2 {
      X: pixels_to_dips(rect.width()),
      Y: pixels_to_dips(rect.height()),
    };
    self.root.SetSize(size)?;
    self.blur_sprite.SetSize(size)?;
    self.tint_sprite.SetSize(size)?;
    self.rounded_geometry.SetSize(size)?;
    Ok(())
  }

  /// Updates the tint layer's color; no-op unless the value changed.
  pub(crate) fn set_tint(&self, tint: crate::Color) -> crate::Result<()> {
    self.tint_brush.SetColor(to_ui_color(tint))?;
    Ok(())
  }

  /// Updates the live blur radius by rebuilding the effect brush.
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
    let compositor = self.compositor.clone();
    let host_backdrop = self.host_backdrop.clone();
    let saturation = self.saturation;

    let effect_brush = run_on_composition_thread(&self.queue, move || {
      build_effect_brush(&compositor, &host_backdrop, value, saturation)
    })?;

    self.blur_sprite.SetBrush(&effect_brush)?;
    self.effect_brush = effect_brush;
    self.blur_amount = value;
    Ok(())
  }

  /// Updates the live saturation by rebuilding the effect brush. Same
  /// rebuild-not-mutate approach as `set_blur_amount`, for the same
  /// reason (`InsertScalar` on a named effect property is not confirmed
  /// working in this pipeline) -- both knobs share the one effect graph,
  /// so either setter rebuilds the whole thing using the other's current
  /// stored value.
  pub(crate) fn set_saturation(&mut self, value: f32) -> crate::Result<()> {
    let compositor = self.compositor.clone();
    let host_backdrop = self.host_backdrop.clone();
    let blur_amount = self.blur_amount;

    let effect_brush = run_on_composition_thread(&self.queue, move || {
      build_effect_brush(&compositor, &host_backdrop, blur_amount, value)
    })?;

    self.blur_sprite.SetBrush(&effect_brush)?;
    self.effect_brush = effect_brush;
    self.saturation = value;
    Ok(())
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
/// (host-backdrop brush through the Gaussian-blur/saturation effect graph)
/// and a tint sprite (flat color) stacked above it, both clipped by a
/// shared rounded rectangle geometry.
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

  let host_backdrop = compositor.CreateHostBackdropBrush()?;
  let effect_brush = build_effect_brush(
    compositor,
    &host_backdrop,
    params.blur_amount,
    params.saturation,
  )?;

  let blur_sprite = compositor.CreateSpriteVisual()?;
  blur_sprite.SetBrush(&effect_brush)?;
  blur_sprite.SetSize(size)?;

  let tint_brush =
    compositor.CreateColorBrushWithColor(to_ui_color(params.tint))?;
  let tint_sprite = compositor.CreateSpriteVisual()?;
  tint_sprite.SetBrush(&tint_brush)?;
  tint_sprite.SetSize(size)?;

  let root = compositor.CreateContainerVisual()?;
  root.SetSize(size)?;
  root.SetClip(&clip)?;
  root.SetOpacity(params.opacity)?;
  root.Children()?.InsertAtTop(&blur_sprite)?;
  root.Children()?.InsertAtTop(&tint_sprite)?;

  target.SetRoot(&root)?;

  Ok(BlurVisual {
    _target: target,
    compositor: compositor.clone(),
    queue: queue.clone(),
    host_backdrop,
    root,
    blur_sprite,
    tint_sprite,
    effect_brush,
    tint_brush,
    rounded_geometry,
    blur_amount: params.blur_amount,
    saturation: params.saturation,
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
    let thread = composition_thread().ok_or_else(|| {
      crate::Error::Platform(
        "Composition pipeline unavailable.".to_string(),
      )
    })?;

    let compositor = thread.compositor.clone();
    let hwnd_raw = hwnd.0;
    let rect = rect.clone();

    run_on_composition_thread(&thread.queue, move || {
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
