#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

use anyhow::Context;
use tracing::debug;
#[cfg(target_os = "windows")]
use tracing::warn;
use wm_common::{
  CursorJumpTrigger, DisplayState, HideCorner, HideMethod, WindowState,
  WmEvent,
};
#[cfg(target_os = "windows")]
use wm_common::{
  OverlayTracking, WindowEffectConfig, WindowTransitionParams,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{
  perf::{self, Stage},
  Rect, WindowZOrder,
};
#[cfg(target_os = "windows")]
use wm_platform::{
  BackdropOverlayParams, BorderOverlayParams, CornerStyle,
  NativeBackdropOverlay, NativeBorderOverlay, OpacityValue, Overlay,
  StyleBlend, SurrogateBatch, HWND,
};

use crate::{
  animation::AnimationPositionResult,
  layout::is_hidden_in_layout,
  models::{Container, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};
#[cfg(target_os = "windows")]
use crate::{
  commands::window::unmanage_if_embedded, models::is_inactive_stack_child,
};

/// Whether this sync cycle skips animations: when asked to (e.g. display
/// setting changes, or a pick in the overview, whose zoom is the
/// transition), and while the overview is on screen. Its previews follow
/// the windows to where they land, and an animation stands surrogates in
/// for them, hiding their borders and backdrops meanwhile.
fn animations_suppressed(state: &WmState) -> bool {
  let is_suppressed = state.pending_sync.animations_suppressed();
  #[cfg(target_os = "windows")]
  let is_suppressed = is_suppressed || state.overview.is_shown();
  is_suppressed
}

pub fn platform_sync(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let _scope = perf::scope(Stage::PlatformSync);

  #[cfg(target_os = "windows")]
  unmanage_embedded_windows(state)?;

  // A focus change or relayout can scroll a scrolling layout, which then
  // moves the rest of its columns too. That includes focus the OS moved
  // itself, e.g. a click on a column partly off screen, which only shows
  // as a focused-effect update.
  if state.pending_sync.needs_focus_update()
    || state.pending_sync.needs_focused_effect_update()
    || !state.pending_sync.containers_to_redraw().is_empty()
  {
    crate::layout::scrolling::sync_scroll_offsets(state)?;
  }

  // What the sync animates plays on scenes covering their monitors before
  // anything changes beneath them, which then happens at once.
  #[cfg(target_os = "windows")]
  let mut plan = crate::animation::scene::plan(state, config);
  #[cfg(target_os = "windows")]
  crate::animation::scene::cover(state, config, &mut plan);

  let synced = sync_to_state(state, config);

  #[cfg(target_os = "windows")]
  crate::animation::scene::play(state, config, &plan, synced.is_ok());

  synced
}

/// Brings the windows, overlays and tab bars on screen in line with the
/// state, as queued in `pending_sync`.
fn sync_to_state(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  // Columns of scrolling layouts are handed to the scroll view before
  // anything is repositioned, so their peeks are on screen before their
  // real windows are cloaked.
  #[cfg(target_os = "windows")]
  if state.pending_sync.needs_focus_update()
    || state.pending_sync.needs_focused_effect_update()
    || !state.pending_sync.containers_to_redraw().is_empty()
  {
    crate::animation::scroll_view::sync_scroll_view(
      state,
      config,
      !animations_suppressed(state),
    );
  }

  // Windows whose own z-order was changed this tick, and whose overlays
  // therefore have to be put back behind them. Collected rather than
  // inferred: the two producers below (`redraw_containers`' bare
  // `set_z_order` for windows it is *not* repositioning, and `sync_focus`'
  // `SetForegroundWindow`) both move a window without moving its overlay
  // and without marking it for redraw, so `sync_overlays` has no other way
  // to know the pairing was broken.
  //
  // Windows repositioned this pass. Their move carries an async z-order
  // too, so their overlays settle alongside `z_order_touched`'s.
  #[cfg(target_os = "windows")]
  let repositioned: Vec<uuid::Uuid> = state
    .windows_to_redraw()
    .iter()
    .map(CommonGetters::id)
    .collect();

  // LINT: only read on Windows, where the overlays exist.
  #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
  let mut z_order_touched: std::collections::HashSet<uuid::Uuid> =
    if !state.pending_sync.containers_to_redraw().is_empty()
      || !state.pending_sync.workspaces_to_reorder().is_empty()
    {
      redraw_containers(&focused_container, state, config)?
    } else {
      std::collections::HashSet::new()
    };

  // Focus is synced after `redraw_containers` so that the workspace-switch
  // animation is already set up when `sync_focus` runs. This lets the
  // deferral check in `sync_focus` correctly suppress
  // `SetForegroundWindow` during the slide (the animation manager
  // re-queues focus after it completes), preventing the OS from
  // asynchronously uncloaking the incoming focused window mid-animation.
  if state.pending_sync.needs_focus_update() {
    // `SetForegroundWindow` raises the window independently of any
    // `set_z_order` call above, so its overlay needs the same repair.
    if let Some(focused_id) = sync_focus(&focused_container, state)? {
      z_order_touched.insert(focused_id);
    }
  }

  if state.pending_sync.needs_cursor_jump()
    && config.value.general.cursor_jump.enabled
  {
    jump_cursor(focused_container.clone(), state, config)?;
  }

  if state.pending_sync.needs_cursor_move() {
    super::move_cursor_to_active_window(state)?;
  }

  if state.pending_sync.needs_focused_effect_update()
    || state.pending_sync.needs_all_effects_update()
  {
    let all_windows = state.pending_sync.needs_all_effects_update();
    apply_focus_effects(state, &focused_container, all_windows, config);
  }

  // A surrogate created this cycle with `place_at_top: true` may have
  // displaced any window's overlay (of either kind), not just its own --
  // fall back to resyncing every non-redrawing window's z-order this one
  // tick instead of only `z_order_touched`'s narrower set (see the field's
  // doc comment on `AnimationManager`). Read and cleared once here so both
  // overlay kinds see it: clearing it per `sync_overlays` call let
  // whichever ran first consume it and starve the other.
  #[cfg(target_os = "windows")]
  let full_z_order_resync = std::mem::take(
    &mut state.animation_manager.backdrop_overlay_z_order_dirty,
  );

  // Sync backdrop and border overlays every tick so they track window
  // position through moves, resizes, and workspace changes. See
  // `sync_overlays`.
  #[cfg(target_os = "windows")]
  sync_overlays::<NativeBackdropOverlay>(
    state,
    config,
    &focused_container,
    full_z_order_resync,
    &z_order_touched,
  );
  #[cfg(target_os = "windows")]
  sync_overlays::<NativeBorderOverlay>(
    state,
    config,
    &focused_container,
    full_z_order_resync,
    &z_order_touched,
  );

  // The overlays above were stacked against where each window is *now*;
  // its own async move may not have landed yet.
  #[cfg(target_os = "windows")]
  state.animation_manager.settle_overlay_z_order(
    repositioned.into_iter().chain(z_order_touched),
  );

  #[cfg(target_os = "windows")]
  let restack = if state.pending_sync.needs_tab_bar_update() {
    crate::tab_bars::Restack::All
  } else {
    crate::tab_bars::Restack::None
  };
  #[cfg(target_os = "windows")]
  crate::tab_bars::sync_tab_bars(state, config, restack);

  // After focus is synced, so the overview never hides while it is still
  // the foreground window.
  #[cfg(target_os = "windows")]
  crate::overview::sync_overview(state, config);

  #[cfg(target_os = "windows")]
  if !state.animation_manager.has_active_animations() {
    crate::animation::scroll_view::report_stuck_windows(state);
  }

  state.pending_sync.clear();

  Ok(())
}

/// Applies the focused-window effects to `focused_container` and the
/// unfocused ones to the window that previously had them (or to every
/// other window when `all_windows` is set).
pub(crate) fn apply_focus_effects(
  state: &mut WmState,
  focused_container: &Container,
  all_windows: bool,
  config: &UserConfig,
) {
  // Keep reference to the previous window that had focus effects
  // applied.
  let prev_effects_window = state.prev_effects_window.clone();

  if let Ok(window) = focused_container.as_window_container() {
    apply_window_effects(&window, true, config);
    state.prev_effects_window = Some(window.clone());
  } else {
    state.prev_effects_window = None;
  }

  // Get windows that should have the unfocused border applied to them.
  // For the sake of performance, we only update the border of the
  // previously focused window. If the `reset_window_effects` flag is
  // passed, the unfocused border is applied to all unfocused windows.
  let unfocused_windows = if all_windows {
    state.windows()
  } else {
    prev_effects_window.into_iter().collect()
  }
  .into_iter()
  .filter(|window| window.id() != focused_container.id());

  for window in unfocused_windows {
    apply_window_effects(&window, false, config);
  }

  // Re-apply animation-driven opacity for the focused window if an
  // opacity focus animation is running. `apply_window_effects` above
  // may have reset the transparency to the config value; overriding it
  // here ensures the animated opacity is visible on the first frame.
  #[cfg(target_os = "windows")]
  if let Ok(window) = focused_container.as_window_container() {
    if let Some(anim) = state.animation_manager.get_animation(&window.id())
    {
      if let (_, Some(opacity)) = anim.current_state() {
        debug!(
          "Overriding transparency for {} with in-progress animation \
           opacity: alpha={}.",
          window.id(),
          opacity.to_alpha()
        );
        let _ = window.native().set_transparency(&opacity);
      }
    }
  }
}

/// Syncs OS input focus to `focused_container`.
///
/// Returns the ID of the window `SetForegroundWindow` was actually called
/// on, if any -- this raises the window's real z-order independently of
/// [`redraw_containers`]'s own `set_z_order` calls, so callers use it to
/// know which windows' backdrop overlays may need a z-order resync (see
/// [`sync_overlays`]'s `z_order_touched` handling).
fn sync_focus(
  focused_container: &Container,
  state: &mut WmState,
) -> anyhow::Result<Option<uuid::Uuid>> {
  // The overview holds the foreground while open, and gives it back when
  // it closes.
  #[cfg(target_os = "windows")]
  if state.overview.is_open() {
    return Ok(None);
  }

  let native_window = focused_container.as_window_container().ok();

  // Defer `SetForegroundWindow` while the focused window is covered by an
  // active surrogate (a resize, or a scrolling layout's moving column).
  // The OS may asynchronously remove the DWM cloak when a window becomes
  // the foreground window, causing the slow `IApplicationView::SetCloak`
  // path to fire on the next animation tick and blocking the frame loop.
  // `AnimationManager::update_internal` re-queues the focus change once
  // the animation completes and the window is uncloaked.
  #[cfg(target_os = "windows")]
  if let Some(window) = &native_window {
    let has_resize_session = state
      .animation_manager
      .resize_sessions
      .contains_key(&window.id());
    if has_resize_session
      || state
        .animation_manager
        .scroll_view
        .owns_window(&window.id())
    {
      return Ok(None);
    }
  }

  // Sets focus to the appropriate target:
  // - If the container is a window, focuses that window.
  // - If the container is a workspace, "resets" focus by focusing the
  //   desktop window.
  //
  // In either case, a `PlatformEvent::WindowFocused` event is subsequently
  // triggered.
  let focused_window_id = native_window.as_ref().map(CommonGetters::id);
  let result = if let Some(window) = native_window {
    tracing::info!("Setting focus to window: {window}");
    window.native().focus()
  } else {
    tracing::info!("Setting focus to the desktop window.");
    state.dispatcher.reset_focus()
  };

  if let Err(err) = result {
    tracing::warn!("Failed to set focus: {}", err);
  }

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container.to_dto()?,
  });

  Ok(focused_window_id)
}

/// Finds windows that should be brought to the top of their workspace's
/// z-order.
///
/// Windows are brought to front if they match the focused window's state
/// (floating/tiling) and any of these conditions are met:
///  * Focus has changed to a different window.
///  * Focused window's state has changed (e.g. tiling -> floating).
///  * Focused window has moved to a different workspace.
fn windows_to_bring_to_front(
  focused_container: &Container,
  state: &WmState,
) -> anyhow::Result<Vec<WindowContainer>> {
  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  // Add focused workspace if there's been a focus change.
  // `HashSet::insert` returns `false` for an id already seen, so this
  // dedupes lazily.
  let mut seen_workspaces = std::collections::HashSet::new();
  let workspaces_to_reorder = state
    .pending_sync
    .workspaces_to_reorder()
    .iter()
    .chain(
      state
        .pending_sync
        .needs_focus_update()
        .then_some(&focused_workspace),
    )
    .filter(|workspace| seen_workspaces.insert(workspace.id()));

  // Bring forward windows that match the focused state. Only do this for
  // tiling/floating windows, and not for a stack's hidden tabs:
  // `set_z_order` shows the window it moves, which would bring back one
  // that couldn't be cloaked.
  let windows_to_bring_to_front = workspaces_to_reorder
    .flat_map(|workspace| {
      let focused_descendant = workspace
        .descendant_focus_order()
        .next()
        .and_then(|container| container.as_window_container().ok());

      match focused_descendant {
        Some(focused_descendant) => workspace
          .descendants()
          .filter_map(|descendant| descendant.as_window_container().ok())
          .filter(|window| {
            let is_floating_or_tiling = matches!(
              window.state(),
              WindowState::Floating(_) | WindowState::Tiling
            );

            is_floating_or_tiling
              && !is_hidden_in_layout(window)
              && window.state().is_same_state(&focused_descendant.state())
          })
          .collect(),
        None => vec![],
      }
    })
    .collect::<Vec<_>>();

  Ok(windows_to_bring_to_front)
}

/// A window queued by [`redraw_containers`] to be cloaked once this pass's
/// shared `DwmFlush` barrier has run.
///
/// Carries everything the cloak-and-preposition step needs, so the step
/// can run after the redraw loop has released its borrow on
/// `windows_to_update`.
#[cfg(target_os = "windows")]
struct PendingCloak {
  /// The window to cloak.
  window: WindowContainer,
  /// Z-order the window was resolved to this pass.
  z_order: WindowZOrder,
  /// Rect to pre-position the cloaked window at.
  target_rect: Rect,
}

/// Cloaks and pre-positions every window [`redraw_containers`] queued this
/// pass, behind one shared `DwmFlush`.
///
/// `backdrop_batch`/`border_batch` hold the overlay repositions that must
/// be visible in the flushed frame; they are committed first so the flush
/// covers them too. See `pending_cloaks`' declaration in
/// [`redraw_containers`] for why the flush is hoisted out of the
/// per-window loop.
#[cfg(target_os = "windows")]
fn commit_pending_cloaks(
  state: &mut WmState,
  pending: Vec<PendingCloak>,
  backdrop_batch: SurrogateBatch,
  border_batch: SurrogateBatch,
) {
  use wm_platform::{
    discard_bits_if_resized, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOSENDCHANGING, SWP_NOZORDER,
  };

  if pending.is_empty() {
    return;
  }

  let _scope = perf::scope(Stage::Cloak);

  backdrop_batch.commit();
  border_batch.commit();

  // Flush before cloaking so DWM renders one frame with every queued
  // surrogate (and its now-correctly-anchored overlays) visible and its
  // thumbnail populated while the real windows are still visible. Without
  // this, thumbnail content may not be ready for the first composition
  // after the cloak, producing a blank frame at animation start.
  wm_platform::dwm_flush();

  for PendingCloak {
    window,
    z_order,
    target_rect,
  } in pending
  {
    let _ = window.native().set_cloaked(true);

    // Pre-position the cloaked window at its target rect so it appears
    // there when uncloaked at animation end. Posted asynchronously — the
    // animation duration (~300 ms) is far longer than any app's
    // message-queue processing time.
    //
    // Growing resize sessions (both dimensions grow) pre-position so DWM
    // captures the correctly-sized content during the curtain-reveal.
    // Mixed and shrinking sessions use the clip/wipe approach
    // (thumbnail at source), and stretch sessions sample source-sized
    // content for the whole animation — both leave the window at
    // source until `pre_commit`.
    //
    // The thumbnail stays registered at source dims until
    // `sync_registration` confirms the resize landed, so a slow-to-respond
    // app costs at most a few frames of backdrop fill in the newly
    // revealed area — never a mis-sized capture. `pre_commit` issues a
    // final synchronous move at animation end as a correctness
    // guarantee.
    let session_flags = state
      .animation_manager
      .resize_sessions
      .get(&window.id())
      .map(|session| {
        (session.needs_preposition(), session.is_move_only())
      });

    let swp_flags = match session_flags {
      // No resize session (e.g. a workspace-switch frozen window): always
      // pre-position, with a frame change.
      None => Some(
        SWP_NOZORDER
          | SWP_FRAMECHANGED
          | SWP_NOACTIVATE
          | SWP_NOSENDCHANGING
          | SWP_ASYNCWINDOWPOS,
      ),
      Some((true, is_move_only)) => {
        let mut flags = SWP_NOZORDER
          | SWP_NOACTIVATE
          | SWP_NOSENDCHANGING
          | SWP_ASYNCWINDOWPOS;
        // `SWP_FRAMECHANGED` forces `WM_NCCALCSIZE` plus a full repaint in
        // the target app; a pure move needs neither, so multi-window
        // relayouts skip that per-window repaint burst for windows that
        // only change position.
        if !is_move_only {
          flags |= SWP_FRAMECHANGED;
        }
        Some(flags)
      }
      Some((false, _)) => None,
    };

    if let Some(swp_flags) = swp_flags {
      let current = window.native().frame_with_shadows().ok();
      let _ = window.native().set_window_pos(
        &z_order,
        &target_rect,
        swp_flags
          | discard_bits_if_resized(current.as_ref(), &target_rect),
      );
    }

    // Mark the session cloaked so subsequent Frozen ticks skip the
    // per-tick `DwmGetWindowAttribute` query.
    if let Some(session) = state
      .animation_manager
      .resize_sessions
      .get_mut(&window.id())
    {
      session.mark_session_cloaked();
    }
  }
}

/// Returns the IDs of windows whose z-order this pass changed *without*
/// repositioning them -- see `z_order_touched`'s declaration below. A
/// window this pass also redraws is not included: its overlay is already
/// resynced on the strength of having moved.
#[allow(clippy::too_many_lines)]
/// Unmanages windows queued for redraw that another app has since
/// embedded into one of its own windows.
///
/// The reparent and hide events that normally unmanage them race with the
/// window's show event when a tabbing app embeds a window right after it
/// opens. Left managed, each workspace switch would cloak the embedded
/// window and move it in its parent's client coordinates.
#[cfg(target_os = "windows")]
fn unmanage_embedded_windows(state: &mut WmState) -> anyhow::Result<()> {
  for window in state.windows_to_redraw() {
    unmanage_if_embedded(window, state)?;
  }

  Ok(())
}

#[allow(clippy::too_many_lines)]
fn redraw_containers(
  focused_container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<std::collections::HashSet<uuid::Uuid>> {
  let _scope = perf::scope(Stage::Redraw);
  let prep_scope = perf::scope(Stage::RedrawPrep);

  state.animation_manager.begin_redraw_pass();

  // Windows given a bare `set_z_order` below, which moves them in the
  // stack while leaving their overlays where they were. Reported to
  // `platform_sync` so `sync_overlays` can put the overlays back.
  //
  // LINT: only populated on Windows, where `set_z_order` is called.
  #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
  let mut z_order_touched = std::collections::HashSet::new();

  let windows_to_redraw = state.windows_to_redraw();
  let windows_to_bring_to_front =
    windows_to_bring_to_front(focused_container, state)?;

  let windows_to_update = {
    let mut seen_windows = std::collections::HashSet::new();
    let mut windows = windows_to_redraw
      .iter()
      .chain(&windows_to_bring_to_front)
      .filter(|window| seen_windows.insert(window.id()))
      .collect::<Vec<_>>();

    // Sorting a single window is a no-op, and building the focus-order
    // index below is an O(total windows) tree walk that runs on every
    // redraw pass while any animation is active -- skip it unless there
    // is actually more than one window to order relative to each other.
    if windows.len() > 1 {
      // Indexed once per pass instead of linearly scanned per window
      // below: that scan made sorting O(windows_to_update x total
      // windows), which showed up as input lag during relayouts that
      // moved several windows on systems with many windows open.
      let focus_order_index: std::collections::HashMap<uuid::Uuid, usize> =
        state
          .root_container
          .descendant_focus_order()
          .enumerate()
          .map(|(i, container)| (container.id(), i))
          .collect();

      // Sort by focus order, with windows that will be shown last so that
      // the reversed iteration below shows them before hiding the others.
      // Hiding first leaves a blank frame, e.g. when switching stack tabs.
      windows.sort_by_key(|window| {
        let will_show = !is_hidden_in_layout(*window)
          && window.workspace().is_some_and(|w| w.is_displayed());

        (will_show, focus_order_index.get(&window.id()).copied())
      });
    }

    windows
  };

  // In-flight animations of redrawn windows are cancelled below so their
  // windows snap to their target rect.
  let suppress_animations = animations_suppressed(state);

  // Get monitors by their optimal hide corner.
  let monitors_by_hide_corner = state.monitors_by_hide_corner();

  // Whether any window in this redraw cycle changes size. Pure
  // translations in the same cycle then share the `window_resize` timing
  // so all edges stay in lock-step during the relayout (see
  // `AnimationManager::start_animation_if_needed`).
  // Set when any window's animation-completion redraw actually issues a
  // `SWP_FRAMECHANGED` reposition on a visible window this pass (see the
  // `already_positioned` check below) -- `reassert_transparency` inside
  // `reposition_window` is only a best-effort fix-up for that flash (it
  // races DWM's own compositor thread), so a single flush after the whole
  // loop deterministically forces one composited frame with the corrected
  // alpha, closing the race for every such window in one blocking call
  // instead of leaving it to chance (or serializing a flush per window).
  #[cfg(target_os = "windows")]
  let mut needs_transparency_flush = false;

  // Windows entering their first `Frozen` frame this pass, deferred out of
  // the loop below so the whole relayout pays one `DwmFlush` instead of
  // one per window.
  //
  // Each window's surrogate must be visible and its overlays anchored for
  // one composited frame *before* the real window is cloaked, otherwise
  // the cloak can land on a frame where the surrogate's thumbnail is not
  // yet populated -- a blank flash at animation start. That barrier is a
  // `DwmFlush`, which blocks until DWM's next composition (~16.7 ms at
  // 60 Hz, ~5.7 ms at 175 Hz). Issued inline it multiplied by the number
  // of windows in the relayout, so a five-window resize stalled the WM's
  // single thread -- which also serves keybindings, mouse events and IPC
  // -- for five whole frames before the animation's first frame even
  // rendered. The barrier is per-composition, not per-window, so one
  // flush after the loop satisfies every queued window at once.
  #[cfg(target_os = "windows")]
  let mut pending_cloaks: Vec<PendingCloak> = Vec::new();
  #[cfg(target_os = "windows")]
  let mut cloak_backdrop_batch = SurrogateBatch::new();
  #[cfg(target_os = "windows")]
  let mut cloak_border_batch = SurrogateBatch::new();

  let cycle_has_resize = windows_to_update.iter().any(|window| {
    let target_rect = window.to_rect().and_then(|rect| {
      window
        .total_border_delta()
        .map(|delta| rect.apply_delta(&delta, None))
    });

    match (target_rect, state.window_target_positions.get(&window.id())) {
      (Ok(target_rect), Some(prev)) => {
        prev.width() != target_rect.width()
          || prev.height() != target_rect.height()
      }
      _ => false,
    }
  });

  drop(prep_scope);
  let loop_scope = perf::scope(Stage::RedrawLoop);

  for window in windows_to_update.iter().rev() {
    let should_bring_to_front = windows_to_bring_to_front.contains(window);

    let workspace =
      window.workspace().context("Window has no workspace.")?;

    let monitor = window.monitor().context("No monitor.")?;
    let hide_corner = monitors_by_hide_corner
      .iter()
      .find(|(m, _)| m.id() == monitor.id())
      .map(|(_, hide_corner)| hide_corner)
      .context("Monitor not found in hide corner map.")?;

    let z_order = match window.state() {
      _ if is_shown_on_top(window) => WindowZOrder::TopMost,
      _ if should_bring_to_front => {
        let focused_descendant = workspace
          .descendant_focus_order()
          .next()
          .and_then(|container| container.as_window_container().ok());

        if let Some(focused_descendant) = focused_descendant {
          if window.id() == focused_descendant.id() {
            WindowZOrder::Normal
          } else {
            WindowZOrder::AfterWindow(focused_descendant.native().id())
          }
        } else {
          WindowZOrder::Normal
        }
      }
      _ => WindowZOrder::Normal,
    };

    // Set the z-order of the window.
    //
    // NOTE: macOS doesn't have a robust public API for setting the z-order
    // of a window. See `NativeWindow::raise` for more details.
    #[cfg(target_os = "windows")]
    if should_bring_to_front && !windows_to_redraw.contains(window) {
      tracing::info!("Updating window z-order: {window}");
      if let Err(err) = window.native().set_z_order(&z_order) {
        tracing::warn!("Failed to set window z-order: {}", err);
      }

      // Recorded even when the call above failed or was skipped as
      // already-correct: `set_z_order` reports neither, and re-asserting
      // an overlay that turns out not to have drifted is one no-op
      // `SetWindowPos`, whereas missing one that did leaves the window
      // covered by its own backdrop until something else reorders it.
      z_order_touched.insert(window.id());
    }

    // Skip updating the window's position if it only required a z-order
    // change.
    if !windows_to_redraw.contains(window) {
      continue;
    }

    // Capture display state before transition to detect opening windows
    let previous_display_state = window.display_state();

    // Transition display state depending on whether window will be
    // shown or hidden. Only the active tab of a stack is shown.
    let should_show =
      workspace.is_displayed() && !is_hidden_in_layout(*window);
    let new_display_state =
      match (previous_display_state.clone(), should_show) {
        (DisplayState::Hidden | DisplayState::Hiding, true) => {
          DisplayState::Showing
        }
        (DisplayState::Shown | DisplayState::Showing, false) => {
          DisplayState::Hiding
        }
        _ => previous_display_state.clone(),
      };
    window.set_display_state(new_display_state);

    let target_rect = window
      .to_rect()?
      .apply_delta(&window.total_border_delta()?, None);

    // Where the real window goes, which differs for a column a peek
    // shows.
    let real_rect = crate::layout::scrolling::real_window_rect(
      &window.as_container(),
      &target_rect,
    );

    let is_visible = matches!(
      window.display_state(),
      DisplayState::Showing | DisplayState::Shown
    );

    // Get the previous target position before updating.
    let previous_target =
      state.window_target_positions.get(&window.id()).cloned();

    // Always record the latest target position.
    state
      .window_target_positions
      .insert(window.id(), target_rect.clone());

    // Floating windows are not animated in general, but we allow a single
    // `window_move` animation when the window just crossed the
    // tiling/floating boundary so the transition is smooth rather than
    // a teleport.
    let is_floating = matches!(window.state(), WindowState::Floating(_));

    // Fullscreen windows are not animated: cloaking the real window (or
    // covering it with a surrogate) kicks exclusive-fullscreen games out
    // of fullscreen, reverting their resolution mode-set and
    // re-triggering a display-settings-changed relayout in a loop.
    //
    // The one exception is a fullscreen transition the *WM* initiated, via
    // `set-fullscreen`/`toggle-fullscreen`. A game driving itself
    // fullscreen is detected in `handle_window_moved_or_resized` and
    // routed through `update_window_state`, so it sets
    // `is_state_change` just like a command does -- only
    // `is_wm_fullscreen_toggle` separates the two. Display-change
    // cycles remain covered by `suppress_animations` regardless.
    let is_fullscreen =
      matches!(window.state(), WindowState::Fullscreen(_));
    let is_state_change =
      state.pending_sync.is_window_state_change(&window.id());
    let is_wm_fullscreen_toggle =
      state.pending_sync.is_wm_fullscreen_toggle(&window.id());

    // Columns of a scrolling layout moving across the monitor edge are
    // drawn by the scroll view until they come to rest, so focus events
    // meanwhile don't uncloak the real window early.
    #[cfg(target_os = "windows")]
    let is_frozen =
      state.animation_manager.scroll_view.is_moving(&window.id());
    #[cfg(not(target_os = "windows"))]
    let is_frozen = false;

    // A window is resizing when its dimensions change (vs. a pure
    // translation).
    let is_resize = previous_target.as_ref().is_some_and(|prev| {
      prev.width() != target_rect.width()
        || prev.height() != target_rect.height()
    });

    // Columns of a scrolling layout move and resize as `scroll` says.
    let is_column =
      crate::layout::scrolling::column_of(&window.as_container())
        .is_some();

    let anim_enabled = if is_column {
      config.value.animations.scroll.enabled
    } else if is_resize {
      config.value.animations.window_resize.enabled
    } else {
      config.value.animations.window_move.enabled
    };

    // Compute effect opacity and corner style unconditionally — needed for
    // both the movement surrogate path and the fade-in path.
    #[cfg(target_os = "windows")]
    let (effect_opacity, corner_style, backdrop_overlay, border_overlay) = {
      let effect_cfg = if window.id() == focused_container.id() {
        &config.value.window_effects.focused_window
      } else {
        &config.value.window_effects.other_windows
      };
      let opacity = effect_cfg.transparency.alpha();
      let style = effect_cfg.resolved_corner_style();
      // Snapshotted onto the `ResizeSession` (rather than re-read live
      // from config at tracking time) since the close animation's
      // direct-drive loop runs after the window is detached from the
      // container tree, where `effect_cfg` can no longer be
      // recomputed.
      let corner_radius = effect_cfg.window_corner_radius_px();
      // Decided here rather than in the tracking loop so the session is
      // built knowing it has no live overlay. `ResizeSession::begin` skips
      // the sampled backdrop color whenever `backdrop_overlay` is `Some`,
      // on the grounds that the tracked backdrop will show through the gap
      // between the animated rect and the thumbnail -- so an overlay that
      // is declared but never tracked leaves that gap fully transparent,
      // which reads as a hole straight through to the desktop. Passing
      // `None` restores the sampled, app-matched fill instead.
      let is_tracked = match config.value.animations.overlay_tracking {
        OverlayTracking::All => true,
        OverlayTracking::FocusedOnly => {
          window.id() == focused_container.id()
        }
        OverlayTracking::None => false,
      };

      let backdrop_overlay = effect_cfg
        .backdrop
        .overlay_tint()
        .filter(|_| is_tracked)
        .map(|tint| {
          effect_cfg.backdrop.to_overlay_params(tint, corner_radius)
        });
      let border_overlay = effect_cfg
        .border
        .abgr_color()
        .filter(|_| is_tracked)
        .filter(|_| !border_hidden_for_fullscreen(window, config))
        .map(|color| {
          effect_cfg.border.to_overlay_params(color, corner_radius)
        });
      (opacity, style, backdrop_overlay, border_overlay)
    };

    // A newly appearing tiling window plays `window_open`.
    // `previous_target.is_none()` is true only on the first
    // `platform_sync` call for a window, so an open starts exactly once.
    #[cfg(target_os = "windows")]
    let entry_transition = (is_visible
      && !is_fullscreen
      && !is_frozen
      && !suppress_animations
      && previous_target.is_none()
      && !is_floating
      && config.value.animations.window_open.enabled)
      .then(|| {
        WindowTransitionParams::from_open(
          &config.value.animations.window_open,
        )
      });

    #[cfg(target_os = "windows")]
    if let Some(params) = entry_transition {
      let monitor_rect = monitor.to_rect()?;
      let native_ref = window.native();
      state.animation_manager.start_open_animation(
        window.id(),
        &target_rect,
        monitor_rect,
        effect_opacity,
        corner_style,
        backdrop_overlay,
        border_overlay,
        &params,
        &native_ref,
      );
    }

    // A slide-in animation creates a `ResizeSession` and animation entry,
    // making the window eligible for the `Frozen`/`Apply` animation paths
    // even when `window_move` animations are disabled.
    #[cfg(target_os = "windows")]
    let has_slide_in = state
      .animation_manager
      .resize_sessions
      .contains_key(&window.id())
      && state
        .animation_manager
        .get_animation(&window.id())
        .is_some_and(|a| !a.is_complete());
    #[cfg(not(target_os = "windows"))]
    let has_slide_in = false;

    // Columns a moving peek draws stay on the animation path regardless
    // of suppression -- the scroll view uncloaks them as they land.
    // Fullscreen windows and suppressed cycles otherwise always take the
    // non-animated path, which also cancels any in-flight animation (and
    // its surrogate) via `remove_animation` below. So do hidden windows,
    // e.g. a stack's inactive tabs: a surrogate would show them while
    // they are cloaked.
    let should_use_animations = is_frozen
      || (is_visible
        && (!is_fullscreen || is_wm_fullscreen_toggle || has_slide_in)
        && !suppress_animations
        && ((anim_enabled && (!is_floating || is_state_change))
          || has_slide_in));

    // Determine the rect to use for this frame.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    let anim_scope = perf::scope(Stage::AnimStep);
    // LINT: Animated opacity is only applied on Windows.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    let (position_result, anim_opacity) = if should_use_animations {
      // A moving peek handles all visuals until the column lands --
      // freeze the real window.
      #[cfg(target_os = "windows")]
      let native_ref = window.native();
      #[cfg(target_os = "windows")]
      if is_frozen {
        (AnimationPositionResult::Frozen, None)
      } else {
        state.animation_manager.start_animation_if_needed(
          window.id(),
          is_resize,
          cycle_has_resize,
          is_column,
          target_rect.clone(),
          previous_target,
          &native_ref,
          effect_opacity,
          corner_style,
          backdrop_overlay,
          border_overlay,
          config,
        )
      }
      #[cfg(not(target_os = "windows"))]
      state.animation_manager.start_animation_if_needed(
        window.id(),
        is_resize,
        cycle_has_resize,
        is_column,
        target_rect.clone(),
        previous_target,
        &window.native(),
        u8::MAX,
        config,
      )
    } else {
      // Animations are skipped for this window. Cancel any in-progress
      // animation and its surrogate so subsequent ticks don't re-cloak it.
      state.animation_manager.remove_animation(&window.id());
      (AnimationPositionResult::Apply(real_rect.clone()), None)
    };

    drop(anim_scope);

    debug!("Updating window position: {window}");

    match position_result {
      AnimationPositionResult::Frozen => {
        let _scope = perf::scope(Stage::RedrawFrozen);

        // A surrogate overlay is covering this window. On the first frame,
        // cloak the real window (so only the surrogate is visible) and
        // synchronously pre-position it at its target rect. Both
        // operations are skipped on subsequent frames: they are
        // idempotent, and repeating a blocking `SetWindowPos`
        // cross-process every tick stalls the animation loop on
        // slow apps and delays keybinding processing.
        //
        // `handle_window_hidden` is guarded against unmanaging cloaked
        // windows so cloaking is safe. If something unclocks the
        // window mid-animation the next tick will re-cloak and
        // re-position it.
        //
        // For `ResizeSession`-backed animations, `pre_commit` also calls
        // `SetWindowPos` synchronously just before the surrogate drops,
        // guaranteeing the window is at `target_rect` when uncloaked.
        // Skip the per-tick `DwmGetWindowAttribute(DWMWA_CLOAKED)`
        // round-trip for resize-session windows whose cloak state
        // is already known — the check only fires on the first
        // `Frozen` frame and after session
        // teardown. Workspace-switch frozen windows (no resize session)
        // retain the full per-tick guard as a safety net.
        #[cfg(target_os = "windows")]
        let already_cloaked_by_session = state
          .animation_manager
          .resize_sessions
          .get(&window.id())
          .is_some_and(wm_platform::ResizeSession::is_session_cloaked);
        #[cfg(target_os = "windows")]
        if !already_cloaked_by_session
          && !window.native().is_cloaked().unwrap_or(false)
        {
          let _scope = perf::scope(Stage::Cloak);

          // The surrogate is created hidden specifically so this can
          // anchor the backdrop overlay behind it *before*
          // `ResizeSession::show` reveals it -- the surrogate's `HWND` is
          // already stable at this point even though it isn't visible
          // yet, which is all `overlay_z_anchor` needs. Without this,
          // showing the surrogate first leaves it visible with no
          // correctly-positioned overlay behind
          // it until this window's backdrop overlay upsert eventually
          // catches up, giving DWM a real window to composite an unblurred
          // frame in.
          //
          // Deferred into the pass-wide batches (rather than each window
          // committing its own `SurrogateBatch`) so the whole relayout's
          // overlay repositions land in one `DeferWindowPos` transaction
          // -- see `pending_cloaks`' declaration.
          if let Some(session) =
            state.animation_manager.resize_sessions.get(&window.id())
          {
            if let (Some(params), Some(anchor), Some(rect)) = (
              session.backdrop_overlay_params(),
              session.surrogate_hwnd(),
              session.current_rect(),
            ) {
              upsert_overlay(
                &mut state.backdrop_overlays,
                window.id(),
                params,
                &rect,
                anchor,
                &mut cloak_backdrop_batch,
              );
            }
            // Framed with its stack's tab bar, which goes along with it.
            if let (Some(params), Some(anchor), Some(rect)) = (
              session.border_overlay_params(),
              session.surrogate_hwnd(),
              session.current_rect().map(|rect| {
                crate::tab_bars::animated_with_tab_bar(
                  state,
                  window.id(),
                  &rect,
                )
              }),
            ) {
              upsert_overlay(
                &mut state.border_overlays,
                window.id(),
                params,
                &rect,
                anchor,
                &mut cloak_border_batch,
              );
            }
          }
          if let Some(session) = state
            .animation_manager
            .resize_sessions
            .get_mut(&window.id())
          {
            session.show();
          }

          // Queued rather than cloaked here: the cloak has to be preceded
          // by a `DwmFlush`, and doing that inline blocks the WM thread
          // for a full composition frame *per window*. See
          // `pending_cloaks`.
          pending_cloaks.push(PendingCloak {
            window: (*window).clone(),
            z_order: z_order.clone(),
            target_rect: real_rect.clone(),
          });
        }
      }
      AnimationPositionResult::Apply(ref apply_rect) => {
        let _scope = perf::scope(Stage::RedrawApply);

        // Only omit `SWP_ASYNCWINDOWPOS` when a surrogate is active for
        // this window — adjacent windows must stay in lock-step
        // with the overlay. For pure moves (no surrogate) async is
        // correct and avoids blocking on the target process's
        // message queue each frame. Also treat incoming ws-switch
        // windows as having a surrogate when being uncloaked at
        // animation completion. Without this, the window
        // would be repositioned with `SWP_ASYNCWINDOWPOS` and immediately
        // uncloaked — if its message queue is slow, it appears at its old
        // position for one frame.
        #[cfg(target_os = "windows")]
        // Windows handed back by a scrolling layout's scroll view too: a
        // just-opened one hasn't been moved into place while cloaked.
        let has_surrogate = state
          .animation_manager
          .resize_sessions
          .contains_key(&window.id())
          || state
            .animation_manager
            .scroll_view
            .owns_window(&window.id());
        #[cfg(not(target_os = "windows"))]
        let has_surrogate = false;

        // Skip the `SetWindowPos` on the animation-completion redraw when
        // `pre_commit` already positioned the window at exactly this rect.
        // Repositioning again is not just redundant — `SWP_FRAMECHANGED`
        // forces a frame recalculation and full repaint that lands right
        // as the window is uncloaked below, flashing at the end of
        // every move/resize animation. The uncloak and effects
        // below still run.
        #[cfg(target_os = "windows")]
        let already_positioned = is_visible
          && state
            .animation_manager
            .was_pre_committed_at(&window.id(), apply_rect);
        #[cfg(not(target_os = "windows"))]
        let already_positioned = false;

        // Whether the uncloak below is still owed. `reposition_window`
        // performs it itself under `HideMethod::Cloak`, and repeating the
        // `DwmSetWindowAttribute` call is the single most expensive thing
        // in this arm -- see `CloakState`.
        #[cfg(target_os = "windows")]
        let mut cloak_state = CloakState::Untouched;
        #[cfg(target_os = "windows")]
        let mut did_move = false;

        if !already_positioned {
          {
            // Attribute the reposition to the window's own process. When
            // `has_surrogate` is set the `SetWindowPos` is synchronous and
            // blocks on that application's message pump, so a single slow
            // app can dominate `rd_apply` -- the stage totals alone cannot
            // show which one. Scoped tightly so only the reposition itself
            // is counted.
            // A window whose app is hung, or was recently slow to answer a
            // synchronous move, is moved asynchronously instead: in step
            // with its surrogate is not worth stalling the whole WM for.
            #[cfg(target_os = "windows")]
            let has_surrogate = has_surrogate
              && state
                .animation_manager
                .allows_sync_reposition(&window.native());

            let _apply_scope = perf::apply_scope(
              || window.native_properties().process_name,
              has_surrogate,
            );

            match reposition_window(
              window,
              apply_rect,
              *hide_corner,
              &z_order,
              is_visible,
              has_surrogate,
              config,
            ) {
              // LINT: Its fields are Windows-only.
              #[cfg_attr(
                not(target_os = "windows"),
                allow(unused_variables)
              )]
              Ok(reposition) => {
                #[cfg(target_os = "windows")]
                {
                  cloak_state = reposition.cloak_state;
                  did_move = reposition.moved;

                  if let Some(elapsed) = reposition.sync_elapsed {
                    state.animation_manager.note_sync_reposition(
                      window.native().hwnd().0,
                      elapsed,
                    );
                  }
                }
              }
              Err(err) => {
                tracing::warn!("Failed to set window position: {}", err);
              }
            }
          }

          #[cfg(target_os = "windows")]
          if is_visible && did_move {
            needs_transparency_flush = true;
          }
        }

        // Uncloak after repositioning so the window is revealed at the
        // correct position. This undoes `set_cloaked(true)` from
        // the `Frozen` branch for non-`HideMethod::Cloak`
        // configurations (that method already calls `set_cloaked`
        // internally inside `reposition_window`).
        #[cfg(target_os = "windows")]
        if is_visible {
          let _uncloak_scope = perf::scope(Stage::ApplyUncloak);

          // Skipped when `reposition_window` just applied the same cloak
          // state: an identical `set_cloaked(false)` measured ~4.8ms of
          // pure duplicate work per window.
          if cloak_state == CloakState::Untouched {
            let _ = window.native().set_cloaked(false);
          }

          state.animation_manager.scroll_view.release(&window.id());
        }

        // Apply animated opacity for opacity-style focus animations. The
        // real window is not cloaked in this path, so `set_transparency`
        // updates it directly each frame.
        #[cfg(target_os = "windows")]
        if let Some(ref opacity) = anim_opacity {
          let _ = window.native().set_transparency(opacity);
        }
      }
    }

    // Mark fullscreen windows as fullscreen on every redraw (including
    // during animations) to ensure browser fullscreen APIs work
    // correctly.
    #[cfg(target_os = "windows")]
    let is_transitioning_fullscreen =
      match (window.prev_state(), window.state()) {
        (Some(_), WindowState::Fullscreen(s)) if !s.maximized => true,
        (Some(WindowState::Fullscreen(_)), _) => true,
        _ => false,
      };

    #[cfg(target_os = "windows")]
    let is_currently_fullscreen =
      matches!(window.state(), WindowState::Fullscreen(_));

    #[cfg(target_os = "windows")]
    if is_currently_fullscreen {
      if let Err(err) = window.native().mark_fullscreen(true) {
        warn!("Failed to mark window as fullscreen: {}", err);
      }
    } else if is_transitioning_fullscreen {
      if let Err(err) = window.native().mark_fullscreen(false) {
        warn!("Failed to unmark window as fullscreen: {}", err);
      }
    }

    // Skip setting taskbar visibility if the window is hidden (has no
    // effect). Since cloaked windows are normally always visible in the
    // taskbar, we only need to set visibility if `show_all_in_taskbar` is
    // `false`.
    #[cfg(target_os = "windows")]
    if config.value.general.hide_method == HideMethod::Cloak
      && !config.value.general.show_all_in_taskbar
      && matches!(
        window.display_state(),
        DisplayState::Showing | DisplayState::Hiding
      )
    {
      if let Err(err) = window.native().set_taskbar_visibility(is_visible)
      {
        tracing::warn!("Failed to set taskbar visibility: {}", err);
      }
    }
  }

  drop(loop_scope);

  // Cloak every window that entered its first `Frozen` frame this pass,
  // behind a single shared `DwmFlush` barrier. See `pending_cloaks`.
  #[cfg(target_os = "windows")]
  commit_pending_cloaks(
    state,
    pending_cloaks,
    cloak_backdrop_batch,
    cloak_border_batch,
  );

  // Every surrogate and overlay reposition queued during this pass goes
  // into one `DeferWindowPos` transaction, committed at the end of this
  // block.
  //
  // Splitting it -- surrogates in one transaction, overlays in another --
  // lets the two land in different DWM composition frames, so a window's
  // surrogate can move while its border outline is still drawn around the
  // old position. Queuing everything first and committing once makes the
  // window and its overlays move atomically, and hands DWM one transaction
  // per frame instead of three.
  //
  // The ordering this relies on: `queue_surrogate_updates` updates each
  // session's `current_rect` as it queues (`ResizeSession::defer_update`),
  // not when the batch is committed, so the overlay loop below reads the
  // same live rect it read before -- it just no longer has to wait on a
  // commit to do so.
  #[cfg(target_os = "windows")]
  {
    let mut batch = SurrogateBatch::new();
    state.animation_manager.queue_surrogate_updates(&mut batch);

    // Keep the backdrop and border overlays tracking each
    // move/resize/open session's surrogate at its live rect, instead of
    // leaving them hidden for the whole animation. Close sessions are
    // tracked separately, inside `AnimationManager::update_internal`'s
    // direct-drive loop -- that path never reaches `platform_sync`
    // (closing windows are detached from the layout tree), so skip
    // them here to avoid a duplicate upsert this tick.
    let overlay_scope = perf::scope(Stage::SessionOverlays);

    // Each animating window costs three composited surfaces, not one: its
    // surrogate plus these two overlays, all repositioned every frame --
    // see `OverlayTracking`. Untracked windows are filtered out
    // earlier, where the session's overlay params are decided, so a
    // session simply has no params here and this loop skips it;
    // `sync_overlays` hides the static overlay for the duration,
    // exactly as it does for any window whose effect isn't configured.
    for (id, session) in &state.animation_manager.resize_sessions {
      if state.animation_manager.has_close_animation(id) {
        continue;
      }

      let anchor = session.surrogate_hwnd();
      let rect = session.current_rect();
      // Framed with its stack's tab bar, which goes along with it.
      let border_rect = rect.as_ref().map(|rect| {
        crate::tab_bars::animated_with_tab_bar(state, *id, rect)
      });

      // A session that carries no params for an overlay kind has no
      // tracker for it, and `sync_overlays` will not step in: it
      // skips every window with a live resize session outright
      // (`has_live_resize_tracker`), assuming this loop owns it.
      // Left alone, the overlay stays visible at the window's
      // pre-animation rect for the whole animation -- a border
      // outline, or a filled backdrop rect, stranded at the old position
      // until the session ends. So hide it here, once, guarded on
      // `is_visible` because `hide` issues a `ShowWindow` unconditionally.
      if session.backdrop_overlay_params().is_none() {
        if let Some(overlay) = state.backdrop_overlays.get_mut(id) {
          if overlay.is_visible() {
            overlay.hide();
          }
        }
      }

      if session.border_overlay_params().is_none() {
        if let Some(overlay) = state.border_overlays.get_mut(id) {
          if overlay.is_visible() {
            overlay.hide();
          }
        }
      }

      if let Some(params) = session.backdrop_overlay_params() {
        match (anchor, rect.clone()) {
          (Some(anchor), Some(rect)) => upsert_overlay(
            &mut state.backdrop_overlays,
            *id,
            params,
            &rect,
            anchor,
            &mut batch,
          ),
          _ => {
            if let Some(overlay) = state.backdrop_overlays.get_mut(id) {
              overlay.hide();
            }
          }
        }
      }

      if let Some(params) = session.border_overlay_params() {
        match (anchor, border_rect) {
          (Some(anchor), Some(rect)) => upsert_overlay(
            &mut state.border_overlays,
            *id,
            params,
            &rect,
            anchor,
            &mut batch,
          ),
          _ => {
            if let Some(overlay) = state.border_overlays.get_mut(id) {
              overlay.hide();
            }
          }
        }
      }
    }
    drop(overlay_scope);

    batch.commit();
  }

  // Force one composited frame reflecting every `reassert_transparency`
  // call made above, deterministically closing the race against DWM's
  // own compositor thread instead of leaving it to scheduling luck. One
  // flush covers every window that completed an animation this pass,
  // rather than blocking once per window.
  //
  // Unlike the cleanup path's flush, this one cannot be deferred to a
  // later tick: the alpha belongs to the frame being composed now, and
  // without it a window using the `transparency` effect composites at the
  // wrong alpha for a frame on every move or resize landing.
  #[cfg(target_os = "windows")]
  if needs_transparency_flush {
    wm_platform::dwm_flush();
  }

  Ok(z_order_touched)
}

/// Above this duration, `reposition_window`'s synchronous
/// (non-`ASYNCWINDOWPOS`) `SetWindowPos`/restore/minimize/maximize
/// sequence is logged as a warning. See its `has_surrogate` doc comment
/// for why this path can block on a slow app's message queue. Mirrors
/// `wm_platform::resize_session`'s `SLOW_SYNC_SETWINDOWPOS_THRESHOLD`.
#[cfg(target_os = "windows")]
const SLOW_SYNC_REPOSITION_THRESHOLD: Duration = Duration::from_millis(8);

/// What [`reposition_window`] did, for the caller's follow-up work.
struct Reposition {
  /// Cloak state the window was left in.
  #[cfg(target_os = "windows")]
  cloak_state: CloakState,
  /// Whether a geometry-changing `SetWindowPos` was actually issued.
  /// Only such a call carries `SWP_FRAMECHANGED`, so the caller's
  /// `DwmFlush` -- which exists only to close that flag's alpha race --
  /// is owed only when this is `true`.
  #[cfg(target_os = "windows")]
  moved: bool,
  /// How long the move took, if it was synchronous.
  #[cfg(target_os = "windows")]
  sync_elapsed: Option<Duration>,
}

/// Whether [`reposition_window`] already applied the window's cloak state.
///
/// `redraw_containers`' `Apply` arm uncloaks visible windows itself, but
/// `HideMethod::Cloak` means [`reposition_window`] has already done
/// exactly that. The repeat `DwmSetWindowAttribute(DWMWA_CLOAK)` is not
/// free: it measured ~4.8ms per window, ~27% of the whole `rd_apply` stage
/// on an eight-window relayout.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloakState {
  /// Applied by `reposition_window`; the caller must not repeat it.
  Applied,
  /// Untouched, so a visible window still needs uncloaking by the caller.
  Untouched,
}

#[allow(clippy::too_many_lines)]
fn reposition_window(
  window: &WindowContainer,
  rect: &Rect,
  hide_corner: HideCorner,
  // LINT: `z_order` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  z_order: &WindowZOrder,
  is_visible: bool,
  // When true, `SWP_ASYNCWINDOWPOS` is omitted so that adjacent windows
  // move synchronously with the surrogate overlay (both hit DWM in the
  // same frame), preventing a one-frame gap between the overlay and its
  // neighbours.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  has_surrogate: bool,
  config: &UserConfig,
) -> anyhow::Result<Reposition> {
  // For `HideMethod::PlaceInCorner`, we need to reposition hidden windows
  // to the corner of the monitor.
  if config.value.general.hide_method == HideMethod::PlaceInCorner
    && !is_visible
  {
    const VISIBLE_SLIVER: i32 = 1;

    let monitor_rect = window
      .monitor()
      .context("No monitor.")?
      .native_properties()
      .working_area;

    let frame = window.native_properties().frame;

    let position_y = monitor_rect.bottom - VISIBLE_SLIVER;
    let position_x = match hide_corner {
      HideCorner::BottomLeft => {
        monitor_rect.left + VISIBLE_SLIVER - frame.width()
      }
      HideCorner::BottomRight => monitor_rect.right - VISIBLE_SLIVER,
    };

    // Even though the window size is unchanged, `NativeWindow::set_frame`
    // is used instead of `NativeWindow::reposition` because the latter
    // resulted in occasional incorrect positionings on macOS.
    window.native().set_frame(&Rect::from_xy(
      position_x,
      position_y,
      frame.width(),
      frame.height(),
    ))?;

    return Ok(Reposition {
      #[cfg(target_os = "windows")]
      cloak_state: CloakState::Untouched,
      #[cfg(target_os = "windows")]
      moved: true,
      #[cfg(target_os = "windows")]
      sync_elapsed: None,
    });
  }

  #[cfg(target_os = "windows")]
  let mut cloak_state = CloakState::Untouched;
  #[cfg(target_os = "windows")]
  let mut moved = true;
  #[cfg(target_os = "windows")]
  let mut sync_elapsed = None;

  if window.active_drag().is_some() {
    window.native().resize(rect.width(), rect.height())?;
  } else {
    #[cfg(target_os = "macos")]
    window.native().set_frame(rect)?;

    #[cfg(target_os = "windows")]
    {
      use wm_platform::{
        discard_bits_if_resized, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING,
        WS_MAXIMIZEBOX,
      };

      // Restore window if it's minimized/maximized and shouldn't be. This
      // is needed to be able to move and resize it.
      let query_scope = perf::scope(Stage::RepositionQuery);
      let should_restore = match &window.state() {
        // A stack's hidden tabs are restored once shown, since restoring
        // activates the window.
        _ if is_inactive_stack_child(window) => false,
        // Need to restore window if transitioning from maximized
        // fullscreen to non-maximized fullscreen.
        WindowState::Fullscreen(fullscreen) => {
          !fullscreen.maximized && window.native().is_maximized()?
        }
        // No need to restore window if it'll be minimized. Transitioning
        // from maximized to minimized works without having to
        // restore.
        WindowState::Minimized => false,
        _ => {
          window.native().is_minimized()?
            || window.native().is_maximized()?
        }
      };

      if should_restore {
        // Restoring to position has the same effect as `ShowWindow` with
        // `SW_RESTORE`, but doesn't cause a flicker.
        window.native().restore(Some(rect))?;
      }

      drop(query_scope);

      // Whether the window already sits exactly where this pass wants
      // it, making the geometry call below a no-op move. Compared
      // against `GetWindowRect`: the space `set_window_pos` writes, and
      // the one `rect` is in, since it carries the border delta. That
      // read is kernel-side rather than a message-pump round-trip, so it
      // is far cheaper than the call it guards.
      //
      // A window being cloaked stays where it is, so one already there
      // needs no call at all. That also spares it the `SWP_FRAMECHANGED`
      // below, which can show a semi-transparent window solid for a frame
      // right before it disappears, e.g. as a workspace switch begins.
      //
      // Not one the shell can't cloak, which is hidden outright instead:
      // its app may stop drawing while hidden (WPF does), and without the
      // call a stack's hidden WPF tabs came back partly unpainted. Checked
      // last, as it asks the shell.
      let current_frame = window.native().frame_with_shadows().ok();
      let already_at_rect = !window.has_pending_dpi_adjustment()
        && current_frame.as_ref() == Some(rect)
        && (is_visible
          || (config.value.general.hide_method == HideMethod::Cloak
            && window.native().can_be_cloaked()));

      // During animation frames, omit `SWP_ASYNCWINDOWPOS` so that
      // adjacent windows are repositioned synchronously. This keeps
      // their on-screen position in lock-step with surrogate
      // overlays (which update DWM directly via
      // `UpdateLayeredWindow`), closing the blank gap that
      // appears when async repositioning lags one frame behind the
      // surrogate.
      //
      // `SWP_NOCOPYBITS` is always added for that same surrogate case:
      // without it, DWM can briefly show the window's old bitmap
      // stretched/offset over the new rect for one frame while it
      // catches up to the surrogate. Otherwise only when the size
      // changes (see `discard_bits_if_resized`): a pure move keeps its
      // still-correct pixels and spares the app a repaint.
      let mut swp_flags = SWP_NOACTIVATE
        | SWP_NOSENDCHANGING
        | if has_surrogate {
          SWP_NOCOPYBITS
        } else {
          SWP_ASYNCWINDOWPOS
            | discard_bits_if_resized(current_frame.as_ref(), rect)
        };
      let sync_reposition_start = has_surrogate.then(Instant::now);
      let swp_scope = perf::scope(Stage::RepositionSwp);

      match &window.state() {
        WindowState::Minimized => {
          // A minimized stack only minimizes its active window; the hidden
          // tabs stay as they are, as minimizing moves focus.
          if !is_inactive_stack_child(window)
            && !window.native().is_minimized()?
          {
            window.native().minimize()?;
          }
        }
        WindowState::Fullscreen(fullscreen)
          if fullscreen.maximized
            && window.native().has_window_style(WS_MAXIMIZEBOX) =>
        {
          // As with minimizing, a stack's hidden tabs are left alone,
          // since maximizing activates the window.
          if !is_inactive_stack_child(window)
            && !window.native().is_maximized()?
          {
            window.native().maximize()?;
          }

          window.native().set_window_pos(z_order, rect, swp_flags)?;
        }
        // Already there, so skip the call -- the most expensive thing in
        // this function. Under `has_surrogate` it is synchronous (see
        // `swp_flags` above) and blocks the WM's whole main loop on the
        // target app's message pump: `explorer.exe` measured 15-21ms per
        // window against a 5.7ms frame budget. Every workspace-switch
        // landing hits this, because `commit_pending_cloaks` already
        // pre-positioned these windows at this rect when it cloaked them.
        //
        // Z-order still has to be applied: for windows being redrawn it
        // rides on the geometry call, since `redraw_containers` calls
        // `set_z_order` directly only for windows it is *not* redrawing.
        // That call is async and returns early when the order is already
        // correct, so it does not reintroduce the block. Not for a window
        // being hidden: it carries `SWP_SHOWWINDOW`.
        _ if already_at_rect => {
          moved = false;
          if is_visible {
            window.native().set_z_order(z_order)?;
          }
        }
        _ => {
          swp_flags |= SWP_FRAMECHANGED;

          window.native().set_window_pos(z_order, rect, swp_flags)?;

          // `SWP_FRAMECHANGED` forces a non-client-area recalculation that
          // can make DWM briefly composite the window at full opacity
          // before its existing `LWA_ALPHA` is reasserted -- a one-frame
          // flash to solid on every resize/move landing for windows using
          // the `transparency` effect. Reassert it immediately so DWM
          // recomposites with the correct alpha right away.
          _ = window.native().reassert_transparency();

          // When there's a mismatch between the DPI of the monitor and the
          // window, the window might be sized incorrectly after the first
          // move. Setting the position twice resolves inconsistencies from
          // the first call. The flag is cleared after so this only runs
          // once per DPI-change event, not on every subsequent animation
          // frame.
          if window.has_pending_dpi_adjustment() {
            window.native().set_window_pos(z_order, rect, swp_flags)?;
            _ = window.native().reassert_transparency();
            window.set_has_pending_dpi_adjustment(false);
          }
        }
      }

      drop(swp_scope);

      if let Some(start) = sync_reposition_start {
        let elapsed = start.elapsed();
        sync_elapsed = Some(elapsed);

        if elapsed > SLOW_SYNC_REPOSITION_THRESHOLD {
          tracing::warn!(
            "Synchronous reposition for {window} took {elapsed:?} -- \
             likely blocked on the target process's message queue, \
             stalling the whole WM main loop for that long (see \
             SLOW_SYNC_REPOSITION_THRESHOLD's doc comment)."
          );
        }
      }

      // Set visibility based on the hide method.
      let _visibility_scope = perf::scope(Stage::RepositionVisibility);
      if config.value.general.hide_method == HideMethod::Cloak {
        // Windows that can't be cloaked are shown and hidden outright, the
        // latter only as a stack's hidden tabs: no thumbnail shows those,
        // while elsewhere DWM keeps rendering a window cloaked along with
        // its owner.
        if is_visible || is_hidden_in_layout(window) {
          window.native().set_cloaked_or_hidden(!is_visible)?;
        } else {
          window.native().set_cloaked(true)?;
        }
        cloak_state = CloakState::Applied;
      } else if is_visible {
        window.native().show()?;
      } else {
        window.native().hide()?;
      }
    }
  }

  Ok(Reposition {
    #[cfg(target_os = "windows")]
    cloak_state,
    #[cfg(target_os = "windows")]
    moved,
    #[cfg(target_os = "windows")]
    sync_elapsed,
  })
}

fn jump_cursor(
  focused_container: Container,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let cursor_jump = &config.value.general.cursor_jump;

  let jump_target = match cursor_jump.trigger {
    CursorJumpTrigger::WindowFocus => Some(focused_container),
    CursorJumpTrigger::MonitorFocus => {
      let target_monitor =
        focused_container.monitor().context("No monitor.")?;

      let cursor_monitor = state
        .dispatcher
        .cursor_position()
        .ok()
        .and_then(|pos| state.monitor_at_point(&pos));

      // Jump to the target monitor if the cursor is not already on it.
      cursor_monitor
        .filter(|monitor| monitor.id() != target_monitor.id())
        .map(|_| target_monitor.into())
    }
  };

  if let Some(jump_target) = jump_target {
    let center = jump_target.to_rect()?.center_point();

    if let Err(err) = state.dispatcher.set_cursor_position(&center) {
      tracing::warn!("Failed to set cursor position: {}", err);
    }
  }

  Ok(())
}

fn apply_window_effects(
  // LINT: `window` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  window: &WindowContainer,
  is_focused: bool,
  config: &UserConfig,
) {
  let window_effects = &config.value.window_effects;

  // LINT: `effect_config` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  let effect_config = if is_focused {
    &window_effects.focused_window
  } else {
    &window_effects.other_windows
  };

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.hide_title_bar.enabled
    || window_effects.other_windows.hide_title_bar.enabled
  {
    apply_hide_title_bar_effect(window, effect_config);
  }

  // `corner_radius` counts as opting in on its own, so setting only the
  // new knob still reaches `apply_corner_effect`.
  #[cfg(target_os = "windows")]
  if window_effects.focused_window.corner_style.enabled
    || window_effects.other_windows.corner_style.enabled
    || window_effects.focused_window.corner_radius.is_some()
    || window_effects.other_windows.corner_radius.is_some()
  {
    apply_corner_effect(window, effect_config);
  }

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.transparency.enabled
    || window_effects.other_windows.transparency.enabled
  {
    apply_transparency_effect(window, effect_config);
  }

  // Keyed off both focus classes rather than `effect_config` alone. Keying
  // it per-class would restore the OS border every time a window lost
  // focus with `border` enabled for only one of the two, so the 1px line
  // would blink in and out on every focus change.
  #[cfg(target_os = "windows")]
  apply_native_border_effect(window, window_effects);
}

/// Suppresses the window's own Windows 11 border while `GlazeWM` draws a
/// border overlay around it, and restores it otherwise.
///
/// The OS border sits on the window's outer edge, *inside* the overlay's
/// ring, so with both present they read as two concentric lines -- most
/// obvious while a window is resizing, where the overlay tracks the
/// animated rect and the OS line tracks the window.
#[cfg(target_os = "windows")]
fn apply_native_border_effect(
  window: &WindowContainer,
  window_effects: &wm_common::WindowEffectsConfig,
) {
  let drawing_own_border = window_effects.focused_window.border.enabled
    || window_effects.other_windows.border.enabled;

  // Errors are expected on Windows 10, which has no such border.
  _ = window.native().set_native_border_hidden(drawing_own_border);
}

#[cfg(target_os = "windows")]
fn apply_hide_title_bar_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  _ = window
    .native()
    .set_title_bar_visibility(!effect_config.hide_title_bar.enabled);
}

#[cfg(target_os = "windows")]
fn apply_corner_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  _ = window
    .native()
    .set_corner_style(&effect_config.resolved_corner_style());
}

#[cfg(target_os = "windows")]
fn apply_transparency_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let transparency = if effect_config.transparency.enabled {
    &effect_config.transparency.opacity
  } else {
    // Reset the transparency to default.
    &OpacityValue::from_alpha(u8::MAX)
  };

  debug!(
    "Applying transparency to {}: alpha={}.",
    window.id(),
    transparency.to_alpha()
  );

  _ = window.native().set_transparency(transparency);
}

/// Creates or updates `window_id`'s overlay of kind `O`, applying `params`
/// and moving it to `rect`.
///
/// Used by the trackers in `AnimationManager::update_internal`, which
/// always have a definite up-to-date rect from the surrogate they follow
/// each tick and so need no conditional re-query -- unlike
/// [`sync_overlays`] below, which only re-queries `frame()` (a
/// cross-process call) when the window is actually being redrawn this
/// tick.
///
/// Takes the overlay map directly (rather than `&mut WmState`) so callers
/// already holding an unrelated borrow into other `WmState` fields can
/// pass `&mut state.backdrop_overlays` without a borrow-checker conflict.
#[cfg(target_os = "windows")]
pub(crate) fn upsert_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, O>,
  window_id: uuid::Uuid,
  params: O::Params,
  rect: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  upsert_blended_overlay(
    overlays,
    window_id,
    StyleBlend::settled(params),
    rect,
    anchor,
    batch,
  );
}

/// [`upsert_overlay`] for a window cut off at the edges of `clip`, e.g. a
/// scrolling layout's column reaching onto another monitor.
#[cfg(target_os = "windows")]
pub(crate) fn upsert_clipped_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, O>,
  window_id: uuid::Uuid,
  params: O::Params,
  rect: &Rect,
  clip: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  if let Some(overlay) =
    overlay_entry(overlays, window_id, params, rect, anchor)
  {
    overlay.apply(params);
    overlay.defer_clipped_rect(batch, rect, clip, anchor);
  }
}

/// [`upsert_overlay`] for a style mid-transition.
#[cfg(target_os = "windows")]
pub(crate) fn upsert_blended_overlay<O: SyncableOverlay>(
  overlays: &mut std::collections::HashMap<uuid::Uuid, O>,
  window_id: uuid::Uuid,
  blend: StyleBlend<O::Params>,
  rect: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  if let Some(overlay) =
    overlay_entry(overlays, window_id, blend.from, rect, anchor)
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
#[cfg(target_os = "windows")]
pub(crate) fn follow_window_frame(
  window: &WindowContainer,
  frame: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) {
  let is_focused = state
    .focused_container()
    .is_some_and(|container| container.id() == window.id());

  let mut batch = SurrogateBatch::new();
  follow_frame::<NativeBackdropOverlay>(
    window, frame, is_focused, state, config, &mut batch,
  );
  follow_frame::<NativeBorderOverlay>(
    window, frame, is_focused, state, config, &mut batch,
  );
  batch.commit();
}

#[cfg(target_os = "windows")]
fn follow_frame<O: SyncableOverlay>(
  window: &WindowContainer,
  frame: &Rect,
  is_focused: bool,
  state: &mut WmState,
  config: &UserConfig,
  batch: &mut SurrogateBatch,
) {
  let params = O::params_for(is_focused, config)
    .filter(|_| !O::suppressed_for(window, config));

  if let Some(params) = params {
    upsert_overlay(
      O::overlays(state),
      window.id(),
      params,
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
#[cfg(target_os = "windows")]
pub(crate) fn resync_overlays_and_tab_bars(state: &mut WmState) {
  // Backdrop first, then border, matching `platform_sync`: each goes
  // directly behind the window, so the border ends up between the two.
  resync_overlay_z_order::<NativeBackdropOverlay>(state);
  resync_overlay_z_order::<NativeBorderOverlay>(state);

  for bar in state.tab_bars.values() {
    bar.keep_behind_anchor();
  }
}

/// Puts shown overlays of kind `O` back directly behind their windows.
#[cfg(target_os = "windows")]
fn resync_overlay_z_order<O: SyncableOverlay>(state: &mut WmState) {
  for window in state.windows() {
    // Animated windows' overlays are owned by their animation driver.
    if state.animation_manager.has_active_surrogate(&window.id()) {
      continue;
    }

    // A peek's overlays stay behind the peek.
    let anchor = state
      .animation_manager
      .scroll_view
      .peek_hwnd(&window.id())
      .unwrap_or_else(|| overlay_z_anchor(&window));

    let Some(overlay) = O::overlays(state).get_mut(&window.id()) else {
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
/// scrolling layout's moving column, keeping it pinned to `viewport` and
/// sliding its ring to `rect` (the window's *unclipped* rect for this
/// frame), cut at `panel`, the area the column is drawn in.
///
/// Unlike [`upsert_overlay`] this issues no `SetWindowPos` per frame
/// and takes no [`SurrogateBatch`]: after the initial pin, a frame costs a
/// single composition property write. See
/// `NativeBorderOverlay::pin_or_slide` for why the window is pinned rather
/// than moved.
#[cfg(target_os = "windows")]
pub(crate) fn upsert_pinned_border_overlay(
  overlays: &mut std::collections::HashMap<
    uuid::Uuid,
    NativeBorderOverlay,
  >,
  window_id: uuid::Uuid,
  blend: StyleBlend<BorderOverlayParams>,
  rect: &Rect,
  viewport: &Rect,
  panel: &Rect,
  anchor: HWND,
) {
  let Some(overlay) =
    overlay_entry(overlays, window_id, blend.from, rect, anchor)
  else {
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
#[cfg(target_os = "windows")]
fn overlay_entry<'a, O: SyncableOverlay>(
  overlays: &'a mut std::collections::HashMap<uuid::Uuid, O>,
  window_id: uuid::Uuid,
  params: O::Params,
  rect: &Rect,
  anchor: HWND,
) -> Option<&'a mut O> {
  match overlays.entry(window_id) {
    std::collections::hash_map::Entry::Occupied(e) => Some(e.into_mut()),
    std::collections::hash_map::Entry::Vacant(e) => {
      match O::create(rect, params, anchor) {
        Ok(overlay) => {
          debug!("{} overlay created for {window_id}.", O::LABEL);
          Some(e.insert(overlay))
        }
        Err(err) => {
          debug!(
            "{} overlay creation failed for {window_id}: {err}.",
            O::LABEL
          );
          None
        }
      }
    }
  }
}

/// Resolves the effects a surrogate needs to stand in for a window:
/// `(effect_opacity, corner_style, backdrop_overlay, border_overlay)`.
///
/// Snapshotted onto the `ResizeSession` rather than re-read live, because
/// the close and minimize direct-drive loops run once the window is
/// detached from the container tree (close) or already minimized
/// (minimize), where the per-window effect config can no longer be
/// recomputed.
#[cfg(target_os = "windows")]
pub(crate) fn surrogate_effects_for(
  is_focused: bool,
  config: &UserConfig,
) -> (
  u8,
  CornerStyle,
  Option<BackdropOverlayParams>,
  Option<BorderOverlayParams>,
) {
  let effect_cfg = if is_focused {
    &config.value.window_effects.focused_window
  } else {
    &config.value.window_effects.other_windows
  };

  let effect_opacity = effect_cfg.transparency.alpha();

  let corner_style = effect_cfg.resolved_corner_style();

  (
    effect_opacity,
    corner_style,
    backdrop_overlay_params_for(is_focused, config),
    border_overlay_params_for(is_focused, config),
  )
}

/// Whether `window`'s border ring is suppressed because it is fullscreen
/// and `window_effects.hide_border_on_fullscreen` is set.
///
/// Applies to the static overlay path and to the surrogate that stands in
/// for the window mid-animation, so a fullscreen transition doesn't
/// animate a ring that the window won't have once it lands.
#[cfg(target_os = "windows")]
pub(crate) fn border_hidden_for_fullscreen(
  window: &WindowContainer,
  config: &UserConfig,
) -> bool {
  config.value.window_effects.hide_border_on_fullscreen
    && matches!(window.state(), WindowState::Fullscreen(_))
}

/// Resolves `window_id`'s border overlay params from its
/// focused/other-window border config, or `None` when the border effect
/// isn't enabled for it. Mirrors [`backdrop_overlay_params_for`].
#[cfg(target_os = "windows")]
pub(crate) fn border_overlay_params_for(
  is_focused: bool,
  config: &UserConfig,
) -> Option<BorderOverlayParams> {
  let effect_cfg = if is_focused {
    &config.value.window_effects.focused_window
  } else {
    &config.value.window_effects.other_windows
  };

  let target_color = effect_cfg.border.abgr_color()?;

  // Mirrors `corner_style` (falling back to `CornerStyle::Default` when
  // disabled) so the overlay's outer radius lines up concentrically with
  // the real managed window's own DWM-rendered corners.
  let corner_radius = effect_cfg.window_corner_radius_px();

  Some(
    effect_cfg
      .border
      .to_overlay_params(target_color, corner_radius),
  )
}

/// Resolves `window`'s backdrop overlay params from its
/// focused/other-window backdrop config, or `None` when backdrop isn't
/// configured or enabled for it.
///
/// Shared by [`sync_overlays`] (the per-tick static path) and the
/// interactive-drag tracker in `handle_window_moved_or_resized`, which
/// needs the same params to keep the overlay live while the OS drags the
/// window outside of `GlazeWM`'s own redraw pipeline.
#[cfg(target_os = "windows")]
pub(crate) fn backdrop_overlay_params_for(
  is_focused: bool,
  config: &UserConfig,
) -> Option<BackdropOverlayParams> {
  let effect_cfg = if is_focused {
    &config.value.window_effects.focused_window
  } else {
    &config.value.window_effects.other_windows
  };

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
fn is_shown_on_top(window: &WindowContainer) -> bool {
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
#[cfg(target_os = "windows")]
pub(crate) fn overlay_z_anchor(window: &WindowContainer) -> HWND {
  window.native().hwnd()
}

/// The WM-side half of an [`Overlay`]: where its instances live in
/// `WmState` and how its params are resolved from the user config, so
/// [`sync_overlays`] drives the backdrop and border through one path.
#[cfg(target_os = "windows")]
pub(crate) trait SyncableOverlay: Overlay {
  /// Label used in this overlay kind's debug log messages.
  const LABEL: &'static str;

  /// Profiler stage this overlay kind's [`sync_overlays`] pass reports as.
  const PERF_STAGE: Stage;

  /// Borrows this overlay kind's tracked-overlay map out of `state`.
  fn overlays(
    state: &mut WmState,
  ) -> &mut std::collections::HashMap<uuid::Uuid, Self>;

  /// Resolves the focused/other-window params, or `None` when this effect
  /// isn't enabled for that focus class. Depends only on its arguments,
  /// so [`sync_overlays`] resolves it once per focus class, not per
  /// window.
  fn params_for(
    is_focused: bool,
    config: &UserConfig,
  ) -> Option<Self::Params>;

  /// Whether this effect is suppressed for one specific window despite
  /// being configured for its focus class. A suppressed overlay is hidden
  /// rather than destroyed.
  fn suppressed_for(
    _window: &WindowContainer,
    _config: &UserConfig,
  ) -> bool {
    false
  }

  /// The rect this overlay tracks for `window` shown at `frame`.
  fn overlay_rect(_window: &WindowContainer, frame: &Rect) -> Rect {
    frame.clone()
  }
}

#[cfg(target_os = "windows")]
impl SyncableOverlay for NativeBackdropOverlay {
  const LABEL: &'static str = "Backdrop";
  const PERF_STAGE: Stage = Stage::BackdropSync;

  fn overlays(
    state: &mut WmState,
  ) -> &mut std::collections::HashMap<uuid::Uuid, Self> {
    &mut state.backdrop_overlays
  }

  fn params_for(
    is_focused: bool,
    config: &UserConfig,
  ) -> Option<Self::Params> {
    backdrop_overlay_params_for(is_focused, config)
  }
}

#[cfg(target_os = "windows")]
impl SyncableOverlay for NativeBorderOverlay {
  const LABEL: &'static str = "Border";
  const PERF_STAGE: Stage = Stage::BorderSync;

  fn overlays(
    state: &mut WmState,
  ) -> &mut std::collections::HashMap<uuid::Uuid, Self> {
    &mut state.border_overlays
  }

  fn params_for(
    is_focused: bool,
    config: &UserConfig,
  ) -> Option<Self::Params> {
    border_overlay_params_for(is_focused, config)
  }

  fn suppressed_for(
    window: &WindowContainer,
    config: &UserConfig,
  ) -> bool {
    border_hidden_for_fullscreen(window, config)
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
/// Windows a scene still shows copies of keep their overlays shown
/// beneath it, untouched, since the outgoing workspace's
/// `!is_displayed()` flips true the instant the switch command runs.
///
/// Windows with an active or fading `ResizeSession` (move/resize/open/
/// close) are likewise owned entirely by the trackers in `platform_sync`'s
/// post-flush loop, `AnimationManager::update_internal`'s close
/// direct-drive loop, and its `pending_session_cleanup` fade-tail loop --
/// same reasoning as the workspace-switch case above.
///
/// Otherwise, hides the static overlay while some other kind of
/// surrogate/session is active for this window without a live tracker of
/// its own (e.g. this effect not configured for it). When such a surrogate
/// is running, the real window is cloaked at its target rect; reading
/// `frame()` would return the target position while the surrogate is still
/// mid-animation, causing the overlay to jump ahead. Also hides for
/// windows on inactive workspaces; the entry stays alive so it can be
/// re-shown immediately when the workspace returns.
///
/// `full_z_order_resync` is `true` for one tick after any `place_at_top`
/// surrogate goes up, which can silently displace *any* window's overlay
/// (of either kind) out of its z-order slot -- see the doc comment at the
/// call site in `platform_sync`. Otherwise every other configured window's
/// overlay is resynced only when `z_order_touched` names it (its own real
/// z-order was actually touched this cycle by `redraw_containers` or
/// `sync_focus`) -- an overlay whose anchor didn't move can't have drifted
/// (see `sync_z_order`'s doc comment), and unconditionally resyncing every
/// configured window on every tick would issue a `GetWindow` syscall per
/// window that's a guaranteed no-op in the common case.
#[cfg(target_os = "windows")]
#[allow(clippy::too_many_lines)]
fn sync_overlays<O: SyncableOverlay>(
  state: &mut WmState,
  config: &UserConfig,
  focused_container: &Container,
  full_z_order_resync: bool,
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

  let surrogates_alive = state.animation_manager.has_any_surrogate();

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

  // `params_for` depends only on `is_focused`, not on any particular
  // window -- resolved once per focus class here instead of once per
  // window in the loop below, so a config with a `{ file, key }` color
  // source pays its `fs::metadata` stat once per tick rather than once
  // per tracked window.
  let focused_params = O::params_for(true, config);
  let other_params = O::params_for(false, config);

  // Whether the effect is configured for *either* focus state. When it is
  // configured for neither it is off outright, and every overlay is
  // destroyed below rather than kept hidden forever.
  let effect_configured =
    focused_params.is_some() || other_params.is_some();

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

    // The animation trackers own these overlays outright -- including
    // fading one out whose effect is off on the window's new focus side,
    // which the hide below would otherwise cut short on the first tick.
    // A scene still showing a copy of one leaves it on screen beneath it.
    if state.animation_manager.scenes.holds(&window.id())
      || state
        .animation_manager
        .has_live_resize_tracker(&window.id())
      || state
        .animation_manager
        .scroll_view
        .draws_window(&window.id())
    {
      continue;
    }

    let params = (if is_focused {
      focused_params
    } else {
      other_params
    })
    .filter(|_| !O::suppressed_for(window, config));

    let Some(params) = params else {
      // Configured, but not for this window's focus state, or suppressed
      // for this particular window: hide rather than destroy.
      // Guarded on `is_visible` because `hide` issues a `ShowWindow`
      // unconditionally.
      if let Some(overlay) = O::overlays(state).get_mut(&window.id()) {
        if overlay.is_visible() {
          overlay.hide();
        }
      }
      continue;
    };

    let should_hide =
      state.animation_manager.has_active_surrogate(&window.id())
        || !window.workspace().is_some_and(|ws| ws.is_displayed())
        || is_hidden_in_layout(window);

    if should_hide {
      if let Some(overlay) = O::overlays(state).get_mut(&window.id()) {
        overlay.hide();
      }
      continue;
    }

    let is_redrawing = redrawing_ids.contains(&window.id());
    let had_overlay = O::overlays(state).contains_key(&window.id());
    let anchor = overlay_z_anchor(window);

    match O::overlays(state).entry(window.id()) {
      std::collections::hash_map::Entry::Occupied(e) => {
        let overlay = e.into_mut();
        overlay.apply(params);

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
        // a workspace switch or resize is on screen, and for
        // nobody once they are gone.
        //
        // Neither covers a window whose z-order was changed *without*
        // being repositioned -- `redraw_containers`' bare `set_z_order`
        // and `sync_focus`' `SetForegroundWindow`. Those leave the
        // overlay behind with nothing in the overlay's own state to show
        // for it, which is what `z_order_touched`/`full_z_order_resync`
        // report. They force the re-assert rather than letting
        // `sync_z_order` decide: the window's move is issued with
        // `SWP_ASYNCWINDOWPOS` and may not have landed yet, so the
        // `GW_HWNDPREV` check can still see the stale (correct-looking)
        // ordering and skip. See `sync_z_order`'s doc comment.
        let forced =
          full_z_order_resync || z_order_touched.contains(&window.id());

        if repositioned || surrogates_alive || forced {
          z_order_resyncs.push((window.id(), anchor, forced));
        }
      }
      std::collections::hash_map::Entry::Vacant(e) => {
        debug_assert!(!had_overlay);

        let Ok(rect) = window.native().frame() else {
          continue;
        };

        match O::create(&O::overlay_rect(window, &rect), params, anchor) {
          Ok(overlay) => {
            debug!("{} overlay created for {}.", O::LABEL, window.id());
            e.insert(overlay);
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
  }

  // Destroy overlays for windows that are gone, or for every window once
  // the effect is switched off entirely -- `wanted_ids` is empty in that
  // case. A window that merely lost focus keeps its (hidden) overlay, and
  // so does one whose close a scene plays.
  let held = O::overlays(state)
    .keys()
    .copied()
    .collect::<Vec<_>>()
    .into_iter()
    .filter(|id| state.animation_manager.scenes.holds(id));
  wanted_ids.extend(held.collect::<Vec<_>>());
  O::overlays(state).retain(|id, _| wanted_ids.contains(id));

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
    if let Some(overlay) = O::overlays(state).get_mut(&id) {
      if let Err(err) = overlay.sync_z_order(anchor, forced) {
        debug!(
          "{} overlay z-order sync failed for {id}: {err}.",
          O::LABEL
        );
      }
    }
  }
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
#[cfg(target_os = "windows")]
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

    let Some(window) = state
      .container_by_id(id)
      .and_then(|container| container.as_window_container().ok())
    else {
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
