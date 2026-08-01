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
  cell::RefCell,
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
      Compositor, ContainerVisual, Desktop::DesktopWindowTarget, SpriteVisual,
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

use crate::{BlurOverlayParams, Rect};

/// `CLSID_D2D1GaussianBlur`, the built-in D2D1 Gaussian-blur effect.
const CLSID_D2D1_GAUSSIAN_BLUR: GUID =
  GUID::from_u128(0x1feb_6d69_2fe6_4ac9_8c58_1d7f_93e7_a6a5);

/// D2D1 Gaussian-blur effect property indices. `CreateEffectFactory`
/// validates the effect description against D2D1's registered schema for
/// this built-in effect and fails with `E_INVALIDARG` unless all three are
/// present, even though only `STANDARD_DEVIATION` is runtime-adjustable
/// here (see `GetNamedPropertyMapping`).
const PROP_STANDARD_DEVIATION: u32 = 0;
const PROP_OPTIMIZATION: u32 = 1;
const PROP_BORDER_MODE: u32 = 2;

/// `D2D1_GAUSSIANBLUR_OPTIMIZATION_BALANCED`.
const D2D1_GAUSSIANBLUR_OPTIMIZATION_BALANCED: u32 = 1;
/// `D2D1_BORDER_MODE_SOFT`.
const D2D1_BORDER_MODE_SOFT: u32 = 0;

/// Hand-implemented D2D1 Gaussian-blur effect description.
///
/// `Compositor::CreateEffectFactory` takes an `IGraphicsEffect` describing
/// an effect graph. `Win2D`'s `GaussianBlurEffect` convenience type
/// requires the `Win2D` winmd, which `windows-rs`'s metadata-driven binding
/// generator cannot consume -- so this hand-implements the
/// `IGraphicsEffectD2D1Interop` COM shape directly against the D2D1
/// built-in Gaussian-blur effect, the same approach Microsoft's own
/// `Windows.UI.Composition-Win32-Samples` uses in C++.
#[implement(IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectD2D1Interop)]
struct GaussianBlurEffect {
  source: IGraphicsEffectSource,
  /// Initial blur std-deviation baked into the effect graph at factory
  /// creation. Runtime adjustment after that happens via the resulting
  /// brush's `Properties().InsertScalar("Blur.BlurAmount", ..)`, not this
  /// field -- see `GetNamedPropertyMapping`.
  initial_amount: f32,
  name: RefCell<HSTRING>,
}

impl GaussianBlurEffect {
  fn new(source: IGraphicsEffectSource, initial_amount: f32) -> Self {
    Self {
      source,
      initial_amount,
      name: RefCell::new(HSTRING::from("Blur")),
    }
  }
}

impl IGraphicsEffectSource_Impl for GaussianBlurEffect {}

impl IGraphicsEffect_Impl for GaussianBlurEffect {
  fn Name(&self) -> windows::core::Result<HSTRING> {
    Ok(self.name.borrow().clone())
  }

  fn SetName(&self, name: &HSTRING) -> windows::core::Result<()> {
    *self.name.borrow_mut() = name.clone();
    Ok(())
  }
}

impl IGraphicsEffectD2D1Interop_Impl for GaussianBlurEffect {
  fn GetEffectId(&self) -> windows::core::Result<GUID> {
    Ok(CLSID_D2D1_GAUSSIAN_BLUR)
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
    // this with the *dotted* `"Blur.BlurAmount"` path -- `Blur` being this
    // effect's `Name` -- not the bare property name alone. Match on the
    // segment after the last `.` so this works regardless of which
    // convention is actually in play (both dotted and bare names have been
    // seen recommended in different Microsoft samples).
    let property_name = name.rsplit('.').next().unwrap_or(&name);

    if property_name == "BlurAmount" {
      // SAFETY: `index`/`mapping` are valid out-parameters supplied by the
      // composition engine for this call.
      unsafe {
        *index = PROP_STANDARD_DEVIATION;
        *mapping = GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT;
      }
      Ok(())
    } else {
      Err(windows::core::Error::from(E_INVALIDARG))
    }
  }

  fn GetPropertyCount(&self) -> windows::core::Result<u32> {
    Ok(3)
  }

  fn GetProperty(
    &self,
    index: u32,
  ) -> windows::core::Result<windows::Foundation::IPropertyValue> {
    match index {
      PROP_STANDARD_DEVIATION => {
        PropertyValue::CreateSingle(self.initial_amount)?.cast()
      }
      PROP_OPTIMIZATION => {
        PropertyValue::CreateUInt32(D2D1_GAUSSIANBLUR_OPTIMIZATION_BALANCED)?
          .cast()
      }
      PROP_BORDER_MODE => {
        PropertyValue::CreateUInt32(D2D1_BORDER_MODE_SOFT)?.cast()
      }
      _ => Err(windows::core::Error::from(E_INVALIDARG)),
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

/// `CLSID_D2D1Saturation`, the built-in D2D1 saturation-adjustment effect.
/// Value matches `windows::Win32::Graphics::Direct2D::CLSID_D2D1Saturation`
/// (re-declared as a local `const` so it sits next to
/// `CLSID_D2D1_GAUSSIAN_BLUR` and follows this module's naming convention).
const CLSID_D2D1_SATURATION: GUID =
  GUID::from_u128(0x5cb2_d9cf_327d_459f_a0ce_40c0_b208_6bf7);

/// D2D1 saturation effect's one property index
/// (`D2D1_SATURATION_PROP_SATURATION`).
const PROP_SATURATION: u32 = 0;

/// Hand-implemented D2D1 saturation effect description, chained after
/// [`GaussianBlurEffect`] in the same effect graph -- see
/// `build_effect_brush`. Mirrors `GaussianBlurEffect`'s shape; the only
/// structural difference is `source` here is the blur effect's own
/// `IGraphicsEffectSource` (an internal graph edge via `GetSource`), not a
/// `CompositionEffectSourceParameter` (an external named binding) -- only
/// the innermost node needs a named parameter for `SetSourceParameter` to
/// bind the live host-backdrop brush to.
#[implement(IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectD2D1Interop)]
struct SaturationEffect {
  source: IGraphicsEffectSource,
  /// Initial saturation baked into the effect graph at factory creation.
  /// Like `GaussianBlurEffect::initial_amount`, runtime adjustment
  /// rebuilds the whole brush rather than mutating this in place -- see
  /// `BlurVisual::set_saturation`.
  initial_saturation: f32,
  name: RefCell<HSTRING>,
}

impl SaturationEffect {
  fn new(source: IGraphicsEffectSource, initial_saturation: f32) -> Self {
    Self {
      source,
      initial_saturation,
      name: RefCell::new(HSTRING::from("Saturation")),
    }
  }
}

impl IGraphicsEffectSource_Impl for SaturationEffect {}

impl IGraphicsEffect_Impl for SaturationEffect {
  fn Name(&self) -> windows::core::Result<HSTRING> {
    Ok(self.name.borrow().clone())
  }

  fn SetName(&self, name: &HSTRING) -> windows::core::Result<()> {
    *self.name.borrow_mut() = name.clone();
    Ok(())
  }
}

impl IGraphicsEffectD2D1Interop_Impl for SaturationEffect {
  fn GetEffectId(&self) -> windows::core::Result<GUID> {
    Ok(CLSID_D2D1_SATURATION)
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
    let property_name = name.rsplit('.').next().unwrap_or(&name);

    if property_name == "Saturation" {
      // SAFETY: `index`/`mapping` are valid out-parameters supplied by the
      // composition engine for this call.
      unsafe {
        *index = PROP_SATURATION;
        *mapping = GRAPHICS_EFFECT_PROPERTY_MAPPING_DIRECT;
      }
      Ok(())
    } else {
      Err(windows::core::Error::from(E_INVALIDARG))
    }
  }

  fn GetPropertyCount(&self) -> windows::core::Result<u32> {
    Ok(1)
  }

  fn GetProperty(
    &self,
    index: u32,
  ) -> windows::core::Result<windows::Foundation::IPropertyValue> {
    match index {
      PROP_SATURATION => {
        PropertyValue::CreateSingle(self.initial_saturation)?.cast()
      }
      _ => Err(windows::core::Error::from(E_INVALIDARG)),
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

/// Unpacks an ABGR-packed tint (see `BackdropEffectConfig::acrylic_tint`)
/// into a `windows::UI::Color`.
#[allow(clippy::cast_possible_truncation)]
fn unpack_abgr_tint(tint: u32) -> Color {
  Color {
    A: (tint >> 24) as u8,
    B: (tint >> 16) as u8,
    G: (tint >> 8) as u8,
    R: tint as u8,
  }
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

  /// Current blur amount, kept alongside `saturation` so either
  /// `set_blur_amount` or `set_saturation` can rebuild the full two-stage
  /// effect graph using the other's current value.
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

  /// Updates the tint layer's color; no-op unless the ABGR value changed.
  pub(crate) fn set_tint(&self, tint: u32) -> crate::Result<()> {
    self.tint_brush.SetColor(unpack_abgr_tint(tint))?;
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
  /// working in this pipeline) -- both knobs share the one two-stage
  /// graph, so either setter rebuilds the whole thing using the other's
  /// current stored value.
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

/// Builds a fresh two-stage effect brush (`host_backdrop` -> Gaussian blur
/// -> saturation -> brush) at the given blur amount and saturation. Split
/// out from `build_visual_tree` so `BlurVisual::set_blur_amount`/
/// `set_saturation` can call it again on demand -- see `set_blur_amount`'s
/// doc comment for why a rebuild, not an in-place property update, is
/// used.
///
/// Both stages are described in one `IGraphicsEffect` graph passed to a
/// single `CreateEffectFactory` call, producing one brush -- not two
/// independently-chained brushes. `SetSourceParameter("Source", ..)` binds
/// `host_backdrop` to the *inner* (blur) node's named parameter; the
/// saturation node's own source is that blur node's
/// `IGraphicsEffectSource` directly (an internal graph edge via
/// `GetSource`, not a named parameter), and the composition engine
/// resolves the "Source" name lookup through to it regardless of nesting
/// depth.
fn build_effect_brush(
  compositor: &Compositor,
  host_backdrop: &CompositionBackdropBrush,
  blur_amount: f32,
  saturation: f32,
) -> windows::core::Result<CompositionEffectBrush> {
  let source_param =
    CompositionEffectSourceParameter::Create(&HSTRING::from("Source"))?;
  let blur_effect: IGraphicsEffectSource =
    GaussianBlurEffect::new(source_param.cast()?, blur_amount).into();
  let saturation_effect: IGraphicsEffect =
    SaturationEffect::new(blur_effect, saturation).into();
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
    compositor.CreateColorBrushWithColor(unpack_abgr_tint(params.tint))?;
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
