//! Each window's border and backdrop overlays: their looks, and keeping
//! them shown, placed and stacked behind their window.

use tracing::{debug, warn};
use wm_common::{EffectSet, WindowState};
use wm_platform::{
  perf::{self, Stage},
  BackdropOverlayParams, BorderOverlayParams, CornerStyle,
  NativeBackdropOverlay, NativeBorderOverlay, Overlay, OverlayParams,
  Rect, StyleBlend, SurrogateBatch, HWND,
};

use super::focus_blend;
use crate::{
  layout::is_hidden_in_layout,
  models::{Container, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  window_overlays::{OverlayPlacement, WindowOverlays},
  wm_state::WmState,
};

/// Creates or updates `window_id`'s overlay of kind `O`, applying `params`
/// and moving it to `rect`.
///
/// For callers with a definite up-to-date rect, e.g. from a surrogate
/// they're following, so no conditional re-query is needed -- unlike
/// [`sync_overlays`] below, which only re-queries `frame()` (a
/// cross-process call) when the window is actually being redrawn this
/// tick.
fn upsert_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  params: O::Params,
  rect: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  upsert_blended_overlay::<O>(
    overlays,
    window_id,
    StyleBlend::settled(params),
    rect,
    anchor,
    batch,
  );
}

/// Shows `window_id`'s overlays with `params` at `placement`, tracking an
/// animation's surrogate, or hides them while the surrogate has nowhere
/// to be. Leaves an overlay alone that `params` has no params for.
///
/// Takes the overlay map directly (rather than `&mut WmState`) so callers
/// still borrowing other `WmState` fields, e.g. the peek they track,
/// can pass `&mut state.window_overlays`.
pub(crate) fn track_overlays(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  params: OverlayParams,
  placement: Option<&OverlayPlacement>,
  batch: &mut SurrogateBatch,
) {
  track_overlay::<NativeBackdropOverlay>(
    overlays,
    window_id,
    params.backdrop,
    placement.map(|placement| (placement.anchor, &placement.rect)),
    batch,
  );
  track_overlay::<NativeBorderOverlay>(
    overlays,
    window_id,
    params.border,
    placement.map(|placement| (placement.anchor, &placement.border_rect)),
    batch,
  );
}

/// [`track_overlays`] for the overlay of kind `O`.
fn track_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  params: Option<O::Params>,
  placement: Option<(HWND, &Rect)>,
  batch: &mut SurrogateBatch,
) {
  let Some(params) = params else {
    return;
  };

  match placement {
    Some((anchor, rect)) => {
      upsert_overlay::<O>(
        overlays, window_id, params, rect, anchor, batch,
      );
    }
    None => {
      if let Some(overlay) = overlay_mut::<O>(overlays, window_id) {
        overlay.hide();
      }
    }
  }
}

/// `window_id`'s overlay of kind `O`, if it has one.
pub(super) fn overlay_mut<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
) -> Option<&mut O> {
  overlays
    .get_mut(&window_id)
    .and_then(|overlays| O::slot(overlays).as_mut())
}

/// [`upsert_overlay`] for a window cut off at the edges of `clip`, e.g. a
/// scrolling layout's column reaching onto another monitor.
pub(crate) fn upsert_clipped_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  blend: StyleBlend<O::Params>,
  rect: &Rect,
  clip: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  if let Some(overlay) =
    overlay_entry::<O>(overlays, window_id, blend.from, rect, anchor)
  {
    overlay.apply_blend(blend);
    overlay.defer_clipped_rect(batch, rect, clip, anchor);
  }
}

/// [`upsert_overlay`] for a style mid-transition.
fn upsert_blended_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  blend: StyleBlend<O::Params>,
  rect: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  if let Some(overlay) =
    overlay_entry::<O>(overlays, window_id, blend.from, rect, anchor)
  {
    overlay.apply_blend(blend);
    overlay.defer_rect(batch, rect, anchor);
  }
}

/// Moves `window`'s backdrop and border overlays to `frame` in one
/// `DeferWindowPos` commit, for a move the WM did not make itself (an
/// interactive drag, or the app moving its own window).
///
/// [`sync_overlays`] only re-queries the rect of windows it is redrawing
/// this tick, so it never observes such a move. It also misses a move of
/// its own that landed late: `set_frame` is asynchronous, and an app whose
/// UI thread is busy applies it after `sync_overlays` read the old rect.
pub(crate) fn follow_window_frame(
  window: &WindowContainer,
  frame: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) {
  let mut batch = SurrogateBatch::new();
  defer_window_frame(window, frame, state, config, &mut batch);
  batch.commit();
}

/// [`follow_window_frame`] into `batch`, e.g. for overlays of several
/// windows to land in one commit.
pub(crate) fn defer_window_frame(
  window: &WindowContainer,
  frame: &Rect,
  state: &mut WmState,
  config: &UserConfig,
  batch: &mut SurrogateBatch,
) {
  let is_focused = state
    .focused_container()
    .is_some_and(|container| container.id() == window.id());

  follow_frame::<NativeBackdropOverlay>(
    window, frame, is_focused, state, config, batch,
  );
  follow_frame::<NativeBorderOverlay>(
    window, frame, is_focused, state, config, batch,
  );
}

fn follow_frame<O: SyncableOverlay>(
  window: &WindowContainer,
  frame: &Rect,
  is_focused: bool,
  state: &mut WmState,
  config: &UserConfig,
  batch: &mut SurrogateBatch,
) {
  let blend = focus_blend::<O>(
    state,
    window.id(),
    window.effect_set(is_focused),
    config,
  )
  .filter(|_| !O::suppressed_for(window, config));

  if let Some(blend) = blend {
    upsert_blended_overlay::<O>(
      &mut state.window_overlays,
      window.id(),
      blend,
      &O::overlay_rect(window, frame),
      overlay_z_anchor(window),
      batch,
    );
  }
}

/// Puts every shown overlay and tab bar back directly behind its window,
/// e.g. after another process restacked windows.
///
/// `sync_overlays` only re-asserts z-order for windows the WM restacked
/// itself, so an app raising or lowering its own window (e.g. a tabbing
/// app keeping its host above its tabs' apps) would otherwise leave them
/// in the old slot. Settled ones cost a few USER32 reads.
pub(crate) fn resync_overlays_and_tab_bars(state: &mut WmState) {
  // Backdrop first, then border, matching `platform_sync`: each goes
  // directly behind the window, so the border ends up between the two.
  resync_overlay_z_order::<NativeBackdropOverlay>(state);
  resync_overlay_z_order::<NativeBorderOverlay>(state);

  for bar in state.tab_bars.values() {
    bar.keep_behind_anchor();
  }

  // E.g. an app raising a neighbour of a close or minimize still playing.
  crate::animation::peek_view::keep_leaving_above(state);
}

/// Puts shown overlays of kind `O` back directly behind their windows.
fn resync_overlay_z_order<O: SyncableOverlay>(state: &mut WmState) {
  for window in state.windows() {
    // Animated windows' overlays are owned by their animation driver.
    if state.animation_manager.has_active_surrogate(&window.id()) {
      continue;
    }

    // A peek's overlays stay behind the peek.
    let anchor = state
      .animation_manager
      .peeks
      .peek_hwnd(&window.id())
      .unwrap_or_else(|| overlay_z_anchor(&window));

    let Some(overlay) =
      overlay_mut::<O>(&mut state.window_overlays, window.id())
    else {
      continue;
    };

    if !overlay.is_visible() {
      continue;
    }

    if let Err(err) = overlay.sync_z_order(anchor, false) {
      debug!(
        "{} overlay z-order resync failed for {}: {err}.",
        O::LABEL,
        window.id()
      );
    }
  }
}

/// Creates or updates `window_id`'s border overlay for one frame of a
/// workspace-switch slide, keeping it pinned to `viewport` and sliding its
/// ring to `rect` (the window's *unclipped* rect for this frame), cut at
/// `panel` (see `WorkspaceSurrogate::panel`).
///
/// Unlike [`upsert_overlay`] this issues no `SetWindowPos` per frame
/// and takes no [`SurrogateBatch`]: after the initial pin, a frame costs a
/// single composition property write. See
/// `NativeBorderOverlay::pin_or_slide` for why the window is pinned rather
/// than moved.
pub(crate) fn upsert_pinned_border_overlay(
  overlays: &mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  blend: StyleBlend<BorderOverlayParams>,
  rect: &Rect,
  viewport: &Rect,
  panel: &Rect,
  anchor: HWND,
) {
  let Some(overlay) = overlay_entry::<NativeBorderOverlay>(
    overlays, window_id, blend.from, rect, anchor,
  ) else {
    return;
  };

  // Applied before the pin: on an unpinned overlay a width change re-runs
  // `set_rect` internally, and the pin below then takes over.
  overlay.apply_blend(blend);

  // A pin that failed to apply sits the transition out.
  if !overlay.pin_or_slide(viewport, rect, panel, anchor) {
    overlay.hide();
  }
}

/// Returns `window_id`'s tracked overlay, creating it at `rect` with
/// `params` when absent. `None` when creation failed, which is logged.
pub(crate) fn overlay_entry<'a, O: SyncableOverlay>(
  overlays: &'a mut std::collections::HashMap<uuid::Uuid, WindowOverlays>,
  window_id: uuid::Uuid,
  params: O::Params,
  rect: &Rect,
  anchor: HWND,
) -> Option<&'a mut O> {
  let slot = O::slot(overlays.entry(window_id).or_default());

  match slot {
    Some(overlay) => Some(overlay),
    None => match O::create(rect, params, anchor) {
      Ok(overlay) => {
        debug!("{} overlay created for {window_id}.", O::LABEL);
        Some(slot.insert(overlay))
      }
      Err(err) => {
        debug!(
          "{} overlay creation failed for {window_id}: {err}.",
          O::LABEL
        );
        None
      }
    },
  }
}

/// Resolves the effects a surrogate needs to stand in for a window:
/// Alpha of a window taking `set`, from its transparency.
fn window_alpha(set: EffectSet, config: &UserConfig) -> u8 {
  let transparency =
    &config.value.window_effects.for_set(set).transparency;
  if transparency.enabled {
    transparency.opacity.to_alpha()
  } else {
    u8::MAX
  }
}

/// `(effect_opacity, corner_style, overlay_params)` of a stand-in drawn
/// in `set`'s style.
pub(crate) fn surrogate_effects_for(
  set: EffectSet,
  config: &UserConfig,
) -> (u8, CornerStyle, OverlayParams) {
  (
    window_alpha(set, config),
    config
      .value
      .window_effects
      .for_set(set)
      .resolved_corner_style(),
    overlay_params_for(set, config),
  )
}

/// Resolves a window's backdrop and border overlay params from its
/// effect set.
pub(crate) fn overlay_params_for(
  set: EffectSet,
  config: &UserConfig,
) -> OverlayParams {
  OverlayParams {
    backdrop: backdrop_overlay_params_for(set, config),
    border: border_overlay_params_for(set, config),
  }
}

/// Whether `window`'s border ring is suppressed because it is fullscreen
/// and `window_effects.hide_border_on_fullscreen` is set.
///
/// Applies to the static overlay path and to the surrogate that stands in
/// for the window mid-animation, so a fullscreen transition doesn't
/// animate a ring that the window won't have once it lands.
pub(crate) fn border_hidden_for_fullscreen(
  window: &WindowContainer,
  config: &UserConfig,
) -> bool {
  config.value.window_effects.hide_border_on_fullscreen
    && matches!(window.state(), WindowState::Fullscreen(_))
}

/// Resolves the border overlay params of a window taking `set`, or `None`
/// when the border effect isn't enabled for it. Mirrors
/// [`backdrop_overlay_params_for`].
pub(crate) fn border_overlay_params_for(
  set: EffectSet,
  config: &UserConfig,
) -> Option<BorderOverlayParams> {
  let effect_cfg = config.value.window_effects.for_set(set);

  let target_color = effect_cfg.border.abgr_color()?;

  // Mirrors `corner_style` (falling back to `CornerStyle::Default` when
  // disabled) so the overlay's outer radius lines up concentrically with
  // the real managed window's own DWM-rendered corners.
  let corner_radius = effect_cfg.window_corner_radius_px();

  // Sized for every style the window may take, so a focus change only
  // redraws the ring inside its window.
  Some(BorderOverlayParams {
    reserve: config
      .value
      .window_effects
      .border_reserve(&config.value.gaps),
    ..effect_cfg.border.to_overlay_params(
      target_color,
      corner_radius,
      config.value.window_effects.border_inset(),
      &config.value.gaps,
    )
  })
}

/// Resolves the backdrop overlay params of a window taking `set`, or
/// `None` when backdrop isn't configured or enabled for it.
///
/// Shared by [`sync_overlays`] (the per-tick static path) and the
/// interactive-drag tracker in `handle_window_moved_or_resized`, which
/// needs the same params to keep the overlay live while the OS drags the
/// window outside of `GlazeWM`'s own redraw pipeline.
fn backdrop_overlay_params_for(
  set: EffectSet,
  config: &UserConfig,
) -> Option<BackdropOverlayParams> {
  let effect_cfg = config.value.window_effects.for_set(set);

  let tint = effect_cfg.backdrop.overlay_tint()?;

  // Mirrors `corner_style` (falling back to `CornerStyle::Default` when
  // disabled, same as `apply_corner_effect`) so the overlay's rounded clip
  // lines up with the real managed window's own DWM-rendered corners
  // sitting on top of it, rather than being an independently configured
  // radius that can mismatch what's actually on screen.
  let corner_radius = effect_cfg.window_corner_radius_px();

  Some(effect_cfg.backdrop.to_overlay_params(tint, corner_radius))
}

/// Whether the window should be shown above all other windows, i.e. in the
/// always-on-top band.
pub(super) fn is_shown_on_top(window: &WindowContainer) -> bool {
  match window.state() {
    WindowState::Floating(config) => config.shown_on_top,
    WindowState::Fullscreen(config) => config.shown_on_top,
    _ => false,
  }
}

/// Resolves the z-order anchor to keep `window`'s overlays pinned directly
/// behind it (see [`NativeBackdropOverlay`]'s doc comment).
///
/// Always the window's own handle, including for "always on top" windows:
/// `HWND_TOPMOST` is an insert-after *pseudo-handle* meaning "top of the
/// band", which would put the overlay above the very window it belongs
/// behind, and never equals a real handle in the "already in place?"
/// check.
///
/// Band membership is handled separately, by `window_class::match_z_band`,
/// which the overlays call before positioning themselves.
pub(crate) fn overlay_z_anchor(window: &WindowContainer) -> HWND {
  window.native().hwnd()
}

/// The WM-side half of an [`Overlay`]: where its instances live in
/// `WmState` and how its params are resolved from the user config, so
/// [`sync_overlays`] drives the backdrop and border through one path.
pub(crate) trait SyncableOverlay: Overlay {
  /// Label used in this overlay kind's debug log messages.
  const LABEL: &'static str;

  /// Profiler stage this overlay kind's [`sync_overlays`] pass reports as.
  const PERF_STAGE: Stage;

  /// This overlay kind's slot among a window's overlays.
  fn slot(overlays: &mut WindowOverlays) -> &mut Option<Self>;

  /// This overlay kind's overlay among a window's overlays, if it has one.
  fn get(overlays: &WindowOverlays) -> Option<&Self>;

  /// Resolves the params of a window taking `set`, or `None` when this
  /// effect isn't enabled for it. Depends only on its arguments, so
  /// [`sync_overlays`] resolves it once per set, not per window.
  fn params_for(
    set: EffectSet,
    config: &UserConfig,
  ) -> Option<Self::Params>;

  /// Whether this effect is suppressed for one specific window despite
  /// being configured for its effect set. A suppressed overlay is hidden
  /// rather than destroyed.
  fn suppressed_for(
    _window: &WindowContainer,
    _config: &UserConfig,
  ) -> bool {
    false
  }

  /// `params` at zero opacity: what a transition fades from or to when
  /// the effect is off on one side of it.
  fn transparent(params: Self::Params) -> Self::Params;

  /// Whether this effect, off in `from`'s style, shows in full at once
  /// rather than fading in as a window takes `to`'s.
  fn shows_at_once(
    _from: EffectSet,
    _to: EffectSet,
    _config: &UserConfig,
  ) -> bool {
    false
  }

  /// The rect this overlay tracks for `window` shown at `frame`.
  fn overlay_rect(_window: &WindowContainer, frame: &Rect) -> Rect {
    frame.clone()
  }
}

impl SyncableOverlay for NativeBackdropOverlay {
  const LABEL: &'static str = "Backdrop";
  const PERF_STAGE: Stage = Stage::BackdropSync;

  fn slot(overlays: &mut WindowOverlays) -> &mut Option<Self> {
    &mut overlays.backdrop
  }

  fn get(overlays: &WindowOverlays) -> Option<&Self> {
    overlays.backdrop.as_ref()
  }

  fn params_for(
    set: EffectSet,
    config: &UserConfig,
  ) -> Option<Self::Params> {
    backdrop_overlay_params_for(set, config)
  }

  fn transparent(params: Self::Params) -> Self::Params {
    BackdropOverlayParams {
      opacity: 0.0,
      ..params
    }
  }

  /// A window's transparency switches at once (see `start_focus_fades`),
  /// so one turning more see-through would show what is behind it
  /// unblurred while its backdrop faded in.
  fn shows_at_once(
    from: EffectSet,
    to: EffectSet,
    config: &UserConfig,
  ) -> bool {
    window_alpha(to, config) < window_alpha(from, config)
  }
}

impl SyncableOverlay for NativeBorderOverlay {
  const LABEL: &'static str = "Border";
  const PERF_STAGE: Stage = Stage::BorderSync;

  fn slot(overlays: &mut WindowOverlays) -> &mut Option<Self> {
    &mut overlays.border
  }

  fn get(overlays: &WindowOverlays) -> Option<&Self> {
    overlays.border.as_ref()
  }

  fn params_for(
    set: EffectSet,
    config: &UserConfig,
  ) -> Option<Self::Params> {
    border_overlay_params_for(set, config)
  }

  fn suppressed_for(
    window: &WindowContainer,
    config: &UserConfig,
  ) -> bool {
    border_hidden_for_fullscreen(window, config)
  }

  fn transparent(params: Self::Params) -> Self::Params {
    BorderOverlayParams {
      opacity: 0.0,
      ..params
    }
  }

  /// Takes in the tab bar of the stack `window` is the active tab of, so
  /// the bar is framed as part of its window.
  fn overlay_rect(window: &WindowContainer, frame: &Rect) -> Rect {
    crate::tab_bars::window_with_tab_bar(window, frame)
  }
}

/// Creates, repositions, and removes overlay windows of kind `O` so every
/// managed window with the matching effect enabled has an overlay tracking
/// its rect and z-order. Shared by backdrop and border overlays -- see
/// [`SyncableOverlay`].
///
/// Each overlay's appearance is re-resolved every tick (cheap: the setters
/// no-op internally unless the resolved value changed, e.g. on focus
/// change). The window's on-screen rect, however, is only re-queried via
/// `frame()` -- a cross-process `DwmGetWindowAttribute` call -- for
/// windows actually queued for redraw this tick (or on first creation).
/// `GlazeWM` owns all positioning of managed windows through that same
/// redraw queue, so a window that isn't in it this tick cannot have moved;
/// skipping the query avoids a round-trip per overlay per tick for windows
/// unrelated to whatever triggered this sync (e.g. other
/// monitors/workspaces during a workspace-switch animation elsewhere).
///
/// Windows with a workspace-switch surrogate are owned entirely by the
/// per-tick tracker in `AnimationManager::update_internal` for the whole
/// slide plus the `pending_ws_cleanup` grace ticks -- it slides the ring
/// of one whose surrogate carries no copy of it every tick, which this
/// function cannot do since it isn't invoked on most mid-slide ticks
/// (only when `platform_sync` itself runs). Leave these
/// overlays untouched here entirely: falling through to the hide logic
/// below would hide them out from under the tracker, since the outgoing
/// workspace's `!is_displayed()` flips true the instant the switch command
/// runs, well before the animation completes.
///
/// Otherwise, hides the static overlay while some other kind of stand-in
/// is active for this window without a live tracker of its own (e.g.
/// this effect not configured for it). When such a surrogate
/// is running, the real window is cloaked at its target rect; reading
/// `frame()` would return the target position while the surrogate is still
/// mid-animation, causing the overlay to jump ahead. Also hides for
/// minimized windows and windows on inactive workspaces; the entry stays
/// alive so it can be re-shown immediately when the window returns.
///
/// Every other configured window's overlay is resynced only when
/// `z_order_touched` names it (its own real z-order was actually touched
/// this cycle by `redraw_containers` or `sync_focus`) -- an overlay whose
/// anchor didn't move can't have drifted
/// (see `sync_z_order`'s doc comment), and unconditionally resyncing every
/// configured window on every tick would issue a `GetWindow` syscall per
/// window that's a guaranteed no-op in the common case.
#[allow(clippy::too_many_lines)]
pub(super) fn sync_overlays<O: SyncableOverlay>(
  state: &mut WmState,
  config: &UserConfig,
  focused_container: &Container,
  z_order_touched: &std::collections::HashSet<uuid::Uuid>,
) {
  let _scope = perf::scope(O::PERF_STAGE);

  let all_windows = state.windows();
  let mut wanted_ids = std::collections::HashSet::new();
  let mut batch = SurrogateBatch::new();

  // Overlays whose z-order needs re-asserting, applied *after* the batch
  // below is committed. Issuing a `SetWindowPos` mid-loop would reorder an
  // overlay while it still sits at its previous position -- the batch has
  // not moved it yet -- so DWM composites a frame of it re-stacked but
  // stale. Across a layout change that is every overlay flashing at once,
  // which is exactly the artifact this ordering avoids.
  let mut z_order_resyncs: Vec<(uuid::Uuid, HWND, bool)> = Vec::new();

  let surrogates_alive =
    state.animation_manager.is_workspace_switch_active();

  // `containers_to_redraw()` may hold an ancestor (e.g. a whole workspace
  // on a workspace switch) rather than each window individually --
  // mirror the same descendant expansion `redraw_containers` uses via
  // `windows_to_redraw` so a window whose ancestor was queued still
  // counts as redrawing.
  let redrawing_ids: std::collections::HashSet<_> = state
    .windows_to_redraw()
    .iter()
    .map(CommonGetters::id)
    .collect();

  // `params_for` depends only on the effect set, not on any particular
  // window -- resolved once per set here instead of once per window in
  // the loop below, so a config with a `{ file, key }` color source pays
  // its `fs::metadata` stat once per tick rather than once per tracked
  // window.
  let focused_params = O::params_for(EffectSet::Focused, config);
  let urgent_params = O::params_for(EffectSet::Urgent, config);
  let other_params = O::params_for(EffectSet::Other, config);

  // Whether the effect is configured for *any* set. When it is configured
  // for none it is off outright, and every overlay is destroyed below
  // rather than kept hidden forever.
  let effect_configured = focused_params.is_some()
    || urgent_params.is_some()
    || other_params.is_some();

  for window in &all_windows {
    let is_focused = window.id() == focused_container.id();

    // Every managed window keeps its overlay while the effect is
    // configured at all, even on the focus side that doesn't currently
    // want one, so a focus change hides and re-shows an existing
    // overlay instead of destroying and rebuilding it. A rebuild costs
    // `CreateWindowExW` plus a whole `Windows.UI.Composition` visual
    // tree -- visible as the window turning translucent a frame or
    // more before its backdrop catches up. A hidden overlay is not
    // composited, so the retained ones cost handles and memory rather
    // than frame time.
    if effect_configured {
      wanted_ids.insert(window.id());
    }

    if overlays_tracked_elsewhere(state, window.id()) {
      continue;
    }

    let set = window.effect_set(is_focused);
    let blend = if state
      .animation_manager
      .focus_mix(&window.id(), set, config)
      .is_some()
    {
      focus_blend::<O>(state, window.id(), set, config)
    } else {
      match set {
        EffectSet::Focused => focused_params,
        EffectSet::Urgent => urgent_params,
        EffectSet::Other => other_params,
      }
      .map(StyleBlend::settled)
    }
    .filter(|_| !O::suppressed_for(window, config));

    let Some(blend) = blend else {
      // Configured, but not for this window's effect set, or suppressed
      // for this particular window: hide rather than destroy.
      // Guarded on `is_visible` because `hide` issues a `ShowWindow`
      // unconditionally.
      if let Some(overlay) =
        overlay_mut::<O>(&mut state.window_overlays, window.id())
      {
        if overlay.is_visible() {
          overlay.hide();
        }
      }
      continue;
    };

    if overlays_hidden(state, window) {
      if let Some(overlay) =
        overlay_mut::<O>(&mut state.window_overlays, window.id())
      {
        overlay.hide();
      }
      continue;
    }

    let is_redrawing = redrawing_ids.contains(&window.id());
    let anchor = overlay_z_anchor(window);
    let slot =
      O::slot(state.window_overlays.entry(window.id()).or_default());

    if let Some(overlay) = slot {
      overlay.apply_blend(blend);

      // Always re-query and re-show when the overlay isn't currently
      // visible, even if this window isn't part of this tick's redraw --
      // otherwise an overlay hidden on some earlier tick (e.g. while a
      // surrogate was active) stays hidden indefinitely once
      // `should_hide` clears, unless this exact window happens
      // to be redrawn again for an unrelated reason (see
      // `is_visible` doc comment).
      let repositioned = is_redrawing || !overlay.is_visible();

      if repositioned {
        // Use the DWM extended frame bounds so the overlay sits flush
        // with the window's visible edge, clipping the invisible resize
        // border.
        match window.native().frame() {
          Ok(rect) => overlay.defer_rect(
            &mut batch,
            &O::overlay_rect(window, &rect),
            anchor,
          ),
          Err(err) => debug!(
            "{} overlay frame() query failed for {}: {err}.",
            O::LABEL,
            window.id()
          ),
        }
      }

      // Overlays that moved this tick, plus every overlay while any
      // surrogate is alive.
      //
      // `defer_rect` batches through `DeferWindowPos` with
      // `SWP_NOZORDER`, so it moves an overlay without restoring
      // where it sits in the stack -- that part still has to be
      // re-asserted. But an overlay nobody moved has nothing to
      // re-assert, and running the check on every overlay on
      // every tick was three USER32 calls per window per
      // frame in exchange for nothing.
      //
      // The surrogate case is not that. Surrogates are created at
      // `HWND_TOP`, which displaces *other* windows' overlays out of
      // their slot -- a window nobody moved can still need
      // putting back. So the check runs for everyone while any
      // surrogate is alive, which is exactly the window in which
      // a workspace switch is on screen, and for nobody once they are
      // gone.
      //
      // Neither covers a window whose z-order was changed *without*
      // being repositioned -- `redraw_containers`' bare `set_z_order`
      // and `sync_focus`' `SetForegroundWindow`. Those leave the
      // overlay behind with nothing in the overlay's own state to show
      // for it, which is what `z_order_touched` reports. They force the
      // re-assert rather than letting `sync_z_order` decide: the
      // window's move is issued with `SWP_ASYNCWINDOWPOS` and may
      // not have landed yet, so the `GW_HWNDPREV` check can still
      // see the stale (correct-looking) ordering and skip. See
      // `sync_z_order`'s doc comment.
      let forced = z_order_touched.contains(&window.id());

      if repositioned || surrogates_alive || forced {
        z_order_resyncs.push((window.id(), anchor, forced));
      }
    } else {
      let Ok(rect) = window.native().frame() else {
        continue;
      };

      match O::create(&O::overlay_rect(window, &rect), blend.from, anchor)
      {
        Ok(mut overlay) => {
          debug!("{} overlay created for {}.", O::LABEL, window.id());
          overlay.apply_blend(blend);
          *slot = Some(overlay);
        }
        Err(err) => {
          debug!(
            "{} overlay creation failed for {}: {err}.",
            O::LABEL,
            window.id()
          );
        }
      }
    }
  }

  // Destroy overlays for windows that are gone, or for every window once
  // the effect is switched off entirely -- `wanted_ids` is empty in that
  // case. A window that merely lost focus keeps its (hidden) overlay.
  // Entries left with neither overlay go too. A window leaving on a peek
  // keeps its overlays, which the peek copies, until it has left, out of
  // the tree as a closing one is.
  state.window_overlays.retain(|id, overlays| {
    let is_leaving =
      effect_configured && state.animation_manager.peeks.is_leaving(id);
    if !wanted_ids.contains(id) && !is_leaving {
      *O::slot(overlays) = None;
    }
    !overlays.is_empty()
  });

  // A second, still-open cause of "the window went blank" -- distinct
  // from the z-order drift the resync below repairs, and not addressed
  // here.
  //
  // The window and its overlay are moved by two separate calls -- the
  // window earlier in the tick by `redraw_containers`, the overlays
  // here. Normally both fall inside one composition interval and DWM
  // never sees them disagree, which is what `defer_rect`'s doc means by
  // "the same DWM composition frame as the window it's paired with".
  // Occasionally they straddle a compose, and DWM draws a frame where an
  // opaque overlay sits over a window that has already moved --
  // concluding the window is not visible, and caching that until
  // something makes it look again. Alt+Tab makes it look again, which is
  // why the window returns the instant the switcher opens without
  // anything being selected.
  //
  // A `dwm_flush()` here (gated on `!batch.is_empty()`, so a config with
  // no overlays keeps the tick it has today) would remove the straddle by
  // forcing one composed frame once every overlay has landed. It is not
  // applied because it blocks until DWM's next composition -- ~16.7ms
  // added to every relayout that moves an overlay -- which needs
  // measuring against the artifact's actual frequency first.
  batch.commit();

  // Now that every overlay is at its final position, put each back
  // directly behind its window. Ordering matters in one direction only:
  // a reposition that lands after a z-order fix is invisible, whereas a
  // z-order fix that lands before the reposition is a visible flash.
  for (id, anchor, forced) in z_order_resyncs {
    if let Some(overlay) = overlay_mut::<O>(&mut state.window_overlays, id)
    {
      if let Err(err) = overlay.sync_z_order(anchor, forced) {
        debug!(
          "{} overlay z-order sync failed for {id}: {err}.",
          O::LABEL
        );
      }
    }
  }
}

/// Whether an animation tracker draws `window_id`'s overlays, which
/// `sync_overlays` then leaves to it -- including fading one out whose
/// effect is off on the window's new focus side, which steady state would
/// otherwise hide on the first tick.
pub(super) fn overlays_tracked_elsewhere(
  state: &WmState,
  window_id: uuid::Uuid,
) -> bool {
  let manager = &state.animation_manager;

  manager.has_live_ws_surrogate(&window_id)
    || manager.peeks.draws_window(&window_id)
}

/// Whether `window`'s overlays are hidden in steady state: behind a
/// surrogate, on a workspace not shown, or with the window not shown.
///
/// Minimized windows show nothing, so their overlays are hidden rather
/// than left to follow them off screen: an overlay is only re-placed when
/// its window is redrawn, which one its app minimizes itself (e.g.
/// `AnyDesk`'s helper window) needn't be, so it stays at its last rect,
/// seen through the gaps between windows.
pub(super) fn overlays_hidden(
  state: &WmState,
  window: &WindowContainer,
) -> bool {
  state.animation_manager.has_active_surrogate(&window.id())
    || !window.workspace().is_some_and(|ws| ws.is_displayed())
    || window.state() == WindowState::Minimized
    || is_hidden_in_layout(window)
}

/// Puts every overlay and tab bar back behind its window once per
/// animation tick while any window is still settling after a z-order
/// change. See `AnimationManager::settle_overlay_z_order`.
///
/// Every one rather than only the settling windows': a window's z-order
/// change also moves the windows it owns, e.g. an app's dialogs rise with
/// its main window, leaving their overlays and tab bars under it.
///
/// `sync_z_order` re-matches the topmost band before re-stacking, which is
/// what repairs a band change that landed after the pass that stacked the
/// overlay.
///
/// The settle is extended rather than left to run out while it cannot
/// finish yet: while the window still has a surrogate (the animation code
/// anchors its overlays itself, so they are skipped here -- and they need
/// re-checking once it is gone, since the tail anchors without matching
/// bands), and while the window's own band change has not landed. Letting
/// either expire left a floating window's border in the normal band, where
/// the next tiled window to be raised covered it.
pub(crate) fn resync_settling_overlays(state: &mut WmState) {
  let settling = state.animation_manager.settling_overlay_windows();
  if settling.is_empty() {
    return;
  }

  for id in settling {
    if state.animation_manager.has_active_surrogate(&id) {
      if state.animation_manager.extend_overlay_z_settle(&id) {
        warn!("Overlay z-settle for {id} capped: surrogate still active.");
      }
      continue;
    }

    let Some(window) = state.window_by_id(id) else {
      continue;
    };

    let is_topmost = window.native().is_topmost();
    if is_topmost != is_shown_on_top(&window)
      && state.animation_manager.extend_overlay_z_settle(&id)
    {
      warn!(
        "Overlay z-settle for {id} ({}) capped: topmost={is_topmost}, \
         expected {}.",
        window.native_properties().process_name,
        !is_topmost
      );
    }
  }

  resync_overlays_and_tab_bars(state);
}

#[cfg(test)]
mod tests {
  use wm_common::{EffectSet, ParsedConfig};
  use wm_platform::{NativeBackdropOverlay, OpacityValue};

  use super::SyncableOverlay;
  use crate::user_config::UserConfig;

  #[test]
  fn backdrops_show_at_once_behind_windows_turning_see_through() {
    let mut parsed = ParsedConfig::default();
    let focused = &mut parsed.window_effects.focused_window.transparency;
    focused.enabled = true;
    focused.opacity = OpacityValue::from_alpha(204);
    parsed.window_effects.other_windows.transparency.enabled = false;
    let config = UserConfig::from_parsed(parsed);

    let shows_at_once =
      |from, to| NativeBackdropOverlay::shows_at_once(from, to, &config);
    assert!(shows_at_once(EffectSet::Other, EffectSet::Focused));
    assert!(!shows_at_once(EffectSet::Focused, EffectSet::Other));
    assert!(!shows_at_once(EffectSet::Other, EffectSet::Other));
  }
}
