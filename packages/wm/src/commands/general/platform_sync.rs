use anyhow::Context;
use itertools::Itertools;
#[cfg(target_os = "windows")]
use wm_common::{WindowEffectConfig, WorkspaceSwitchStyle};
use tracing::{debug, warn};
use wm_common::{
  CursorJumpTrigger, DisplayState, HideCorner, HideMethod, WindowState,
  WmEvent,
};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use wm_platform::{
  BackdropStyle, BlurOverlayParams, BorderOverlayParams, CornerStyle,
  NativeBlurOverlay, NativeBorderOverlay, NativeIrisOverlay, OpacityValue,
  SurrogateBatch, WorkspaceSurrogate, HWND, HWND_TOPMOST,
};
use wm_platform::{
  perf::{self, Stage},
  Rect, WindowZOrder,
};

#[cfg(target_os = "windows")]
use crate::pending_sync::IrisSwitchRequest;
use crate::{
  animation::AnimationPositionResult,
  models::{Container, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Returns the smallest iris radius that fully covers the monitor from the
/// origin — the distance to the farthest monitor corner.
#[cfg(target_os = "windows")]
fn iris_max_radius(req: &IrisSwitchRequest) -> i32 {
  let corners = [
    (req.monitor_x, req.monitor_y),
    (req.monitor_x + req.monitor_width, req.monitor_y),
    (req.monitor_x, req.monitor_y + req.monitor_height),
    (
      req.monitor_x + req.monitor_width,
      req.monitor_y + req.monitor_height,
    ),
  ];
  corners
    .iter()
    .map(|&(x, y)| {
      let dx = f64::from(x - req.origin_x);
      let dy = f64::from(y - req.origin_y);
      dx.hypot(dy)
    })
    .fold(0.0_f64, f64::max)
    .ceil() as i32
}

pub fn platform_sync(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let _scope = perf::scope(Stage::PlatformSync);

  let focused_container =
    state.focused_container().context("No focused container.")?;

  // Windows whose real (non-overlay) z-order was actually touched this
  // cycle, either by `redraw_containers`'s `set_z_order` or by
  // `sync_focus`'s `SetForegroundWindow` below. Passed to
  // `sync_blur_overlays` so it only pays for a `sync_z_order` resync on the
  // handful of windows that could plausibly have drifted, instead of every
  // blur-configured window on every tick -- most commands (e.g. a plain
  // focus change between tiled windows) touch at most one window's z-order.
  let mut z_order_touched = std::collections::HashSet::new();

  if !state.pending_sync.containers_to_redraw().is_empty()
    || !state.pending_sync.workspaces_to_reorder().is_empty()
  {
    redraw_containers(&focused_container, state, config, &mut z_order_touched)?;
  }

  // Focus is synced after `redraw_containers` so that the workspace-switch
  // animation is already set up when `sync_focus` runs. This lets the
  // deferral check in `sync_focus` correctly suppress `SetForegroundWindow`
  // during the slide (the animation manager re-queues focus after it
  // completes), preventing the OS from asynchronously uncloaking the
  // incoming focused window mid-animation.
  if state.pending_sync.needs_focus_update() {
    if let Some(id) = sync_focus(&focused_container, state)? {
      z_order_touched.insert(id);
    }
  }

  if state.pending_sync.needs_cursor_jump()
    && config.value.general.cursor_jump.enabled
  {
    jump_cursor(focused_container.clone(), state, config)?;
  }

  if state.pending_sync.needs_focused_effect_update()
    || state.pending_sync.needs_all_effects_update()
  {
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
    let unfocused_windows =
      if state.pending_sync.needs_all_effects_update() {
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
      if let Some(anim) = state.animation_manager.get_animation(&window.id()) {
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

  // A surrogate created this cycle with `place_at_top: true` may have
  // displaced any window's overlay (of either kind), not just its own --
  // fall back to resyncing every non-redrawing window's z-order this one
  // tick instead of only `z_order_touched`'s narrower set (see the field's
  // doc comment on `AnimationManager`). Read and cleared once here so both
  // overlay kinds observe the same value regardless of call order --
  // previously each of `sync_blur_overlays`/`sync_border_overlays` read and
  // cleared this flag independently, so whichever ran first silently
  // consumed it even when its own overlay kind wasn't configured for any
  // window, leaving the other permanently starved of the full resync.
  #[cfg(target_os = "windows")]
  let full_z_order_resync = state.animation_manager.blur_overlay_z_order_dirty;
  #[cfg(target_os = "windows")]
  {
    state.animation_manager.blur_overlay_z_order_dirty = false;
  }

  // Sync acrylic blur and border overlays every tick so they track window
  // position through moves, resizes, and workspace changes. See
  // `sync_overlays`.
  #[cfg(target_os = "windows")]
  sync_overlays::<NativeBlurOverlay>(
    state,
    config,
    &focused_container,
    &z_order_touched,
    full_z_order_resync,
  );
  #[cfg(target_os = "windows")]
  sync_overlays::<NativeBorderOverlay>(
    state,
    config,
    &focused_container,
    &z_order_touched,
    full_z_order_resync,
  );

  state.pending_sync.clear();

  Ok(())
}

/// Syncs OS input focus to `focused_container`.
///
/// Returns the ID of the window `SetForegroundWindow` was actually called
/// on, if any -- this raises the window's real z-order independently of
/// [`redraw_containers`]'s own `set_z_order` calls, so callers use it to
/// know which windows' acrylic blur overlays may need a z-order resync (see
/// `sync_blur_overlays`'s `z_order_touched` parameter).
fn sync_focus(
  focused_container: &Container,
  state: &mut WmState,
) -> anyhow::Result<Option<uuid::Uuid>> {
  let native_window = focused_container.as_window_container().ok();

  // Defer `SetForegroundWindow` while the focused window is covered by an
  // active surrogate (workspace-switch or resize). The OS may asynchronously
  // remove the DWM cloak when a window becomes the foreground window,
  // causing the slow `IApplicationView::SetCloak` path to fire on the next
  // animation tick and blocking the frame loop. `AnimationManager::update_internal`
  // re-queues the focus change once the animation completes and the window is
  // uncloaked.
  #[cfg(target_os = "windows")]
  if let Some(window) = &native_window {
    let is_ws_incoming = state.animation_manager.is_workspace_switch_active()
      && state
        .animation_manager
        .is_workspace_switch_incoming(&window.id());
    let has_resize_session = state
      .animation_manager
      .resize_sessions
      .contains_key(&window.id());
    if is_ws_incoming || has_resize_session {
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
    .unique_by(|workspace| workspace.id());

  // Bring forward windows that match the focused state. Only do this for
  // tiling/floating windows.
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
/// Carries everything the cloak-and-preposition step needs, so the step can
/// run after the redraw loop has released its borrow on `windows_to_update`.
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
/// `blur_batch`/`border_batch` hold the overlay repositions that must be
/// visible in the flushed frame; they are committed first so the flush
/// covers them too. See `pending_cloaks`' declaration in
/// [`redraw_containers`] for why the flush is hoisted out of the per-window
/// loop.
#[cfg(target_os = "windows")]
fn commit_pending_cloaks(
  state: &mut WmState,
  pending: Vec<PendingCloak>,
  blur_batch: SurrogateBatch,
  border_batch: SurrogateBatch,
) {
  use wm_platform::{
    SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOSENDCHANGING,
    SWP_NOZORDER,
  };

  if pending.is_empty() {
    return;
  }

  let _scope = perf::scope(Stage::Cloak);

  blur_batch.commit();
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
    // captures the correctly-sized content during the curtain-reveal. Mixed
    // and shrinking sessions use the clip/wipe approach (thumbnail at
    // source), and stretch sessions sample source-sized content for the
    // whole animation — both leave the window at source until `pre_commit`.
    //
    // The thumbnail stays registered at source dims until
    // `sync_registration` confirms the resize landed, so a slow-to-respond
    // app costs at most a few frames of backdrop fill in the newly revealed
    // area — never a mis-sized capture. `pre_commit` issues a final
    // synchronous move at animation end as a correctness guarantee.
    let session_flags = state
      .animation_manager
      .resize_sessions
      .get(&window.id())
      .map(|session| (session.needs_preposition(), session.is_move_only()));

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
        let mut flags =
          SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_ASYNCWINDOWPOS;
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
      let _ =
        window.native().set_window_pos(&z_order, &target_rect, swp_flags);
    }

    // Mark the session cloaked so subsequent Frozen ticks skip the per-tick
    // `DwmGetWindowAttribute` query.
    if let Some(session) =
      state.animation_manager.resize_sessions.get_mut(&window.id())
    {
      session.mark_session_cloaked();
    }
  }
}

#[allow(clippy::too_many_lines)]
fn redraw_containers(
  focused_container: &Container,
  state: &mut WmState,
  config: &UserConfig,
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  z_order_touched: &mut std::collections::HashSet<uuid::Uuid>,
) -> anyhow::Result<()> {
  let _scope = perf::scope(Stage::Redraw);
  let prep_scope = perf::scope(Stage::RedrawPrep);

  let windows_to_redraw = state.windows_to_redraw();
  let windows_to_bring_to_front =
    windows_to_bring_to_front(focused_container, state)?;

  let windows_to_update = {
    let mut windows = windows_to_redraw
      .iter()
      .chain(&windows_to_bring_to_front)
      .unique_by(|window| window.id())
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

      // Sort the windows to update by their focus order. The most recently
      // focused window will be updated first.
      // TODO: To reduce flicker, redraw windows that will be shown first,
      // then redraw the ones to be hidden last.
      windows.sort_by_key(|window| {
        focus_order_index.get(&window.id()).copied()
      });
    }

    windows
  };

  // Whether animations are skipped for this sync cycle (e.g. display
  // setting changes). In-flight animations of redrawn windows are cancelled
  // below so their windows snap to their target rect.
  let suppress_animations = state.pending_sync.animations_suppressed();

  // Workspace-switch pre-pass: create slide surrogates for all
  // incoming/outgoing windows before any real window is repositioned.
  // Outgoing surrogates are shown immediately (before the real window is
  // cloaked) to eliminate the blank-frame flicker.
  #[cfg(target_os = "windows")]
  {
    let ws_config = &config.value.animations.workspace_switch;
    if ws_config.enabled && !suppress_animations {
      // Iris-wipe pre-pass: snapshot the monitor (still showing the outgoing
      // workspace) and show the overlay before the real windows are switched in
      // the redraw loop below. The hole is then driven by the animation manager.
      if let Some(req) = state.pending_sync.take_iris_switch() {
        let monitor = Rect::from_xy(
          req.monitor_x,
          req.monitor_y,
          req.monitor_width,
          req.monitor_height,
        );
        // Drop any in-flight overlay first so the new snapshot captures the
        // real current workspace, not the previous overlay mid-wipe. This makes
        // rapid switches play as clean successive wipes rather than nested ones.
        state.animation_manager.clear_iris_switch();
        match NativeIrisOverlay::create(&monitor) {
          Ok(overlay) => {
            state.animation_manager.start_iris_switch(
              overlay,
              req.origin_x,
              req.origin_y,
              iris_max_radius(&req),
              req.monitor_handle,
              ws_config.duration_ms,
              ws_config.easing.clone(),
            );
            // Composite the overlay (covering the outgoing workspace) before the
            // redraw loop below switches the real windows underneath, so the
            // switch never shows through for a frame. Without this the cover and
            // the switch race within one frame, causing an occasional flicker.
            wm_platform::dwm_flush();
          }
          Err(err) => {
            tracing::warn!(
              "Iris overlay failed; instant workspace switch: {err}."
            );
          }
        }
      }

      let direction = state.pending_sync.workspace_switch_direction();
      // Only start a new workspace-switch animation when there are actually
      // incoming/outgoing windows in this sync (i.e., this is the initial
      // platform_sync for the switch, not a follow-up focus event).
      let has_ws_windows = windows_to_update.iter().any(|w| {
        let id = w.id();
        state.pending_sync.is_workspace_switch_incoming(&id)
          || state.pending_sync.is_workspace_switch_outgoing(&id)
      });

      if has_ws_windows {
        let is_no_slide = ws_config.style.is_no_slide();
        let mut ws_windows: Vec<(uuid::Uuid, Option<WorkspaceSurrogate>, bool)> =
          Vec::new();
        let mut monitor_x = 0i32;
        let mut monitor_width = 0i32;
        let mut monitor_y = 0i32;
        let mut monitor_height = 0i32;
        let mut monitor_handle = 0isize;

        for window in windows_to_update.iter() {
          let id = window.id();
          let is_incoming =
            state.pending_sync.is_workspace_switch_incoming(&id);
          let is_outgoing =
            state.pending_sync.is_workspace_switch_outgoing(&id);

          if !is_incoming && !is_outgoing {
            continue;
          }

          if monitor_width == 0 {
            if let Some(m) = window.monitor() {
              let props = m.native_properties();
              let b = &props.bounds;
              monitor_x = b.x();
              monitor_width = b.width();
              monitor_y = b.y();
              monitor_height = b.height();
              monitor_handle = props.handle;
            }
          }

          let hwnd = window.native().hwnd();

          let effect_cfg =
            if window.id() == focused_container.id() {
              &config.value.window_effects.focused_window
            } else {
              &config.value.window_effects.other_windows
            };
          let opacity = if effect_cfg.transparency.enabled {
            effect_cfg.transparency.opacity.to_alpha()
          } else {
            u8::MAX
          };
          // Carry the acrylic blur onto the surrogate so the frosted-glass
          // effect stays visible while the workspace slides. The static
          // blur overlay is hidden for the duration of the animation.
          let acrylic_tint =
            effect_cfg.backdrop.acrylic_tint().map(|c| c.to_abgr());
          let corner_style = if effect_cfg.corner_style.enabled {
            effect_cfg.corner_style.style.clone()
          } else {
            CornerStyle::Default
          };

          if is_incoming {
            let surrogate = window
              .to_rect()
              .and_then(|r| {
                window.total_border_delta().map(|d| r.apply_delta(&d, None))
              })
              .ok()
              .and_then(|rect| {
                let viewport =
                  Rect::from_xy(monitor_x, monitor_y, monitor_width, monitor_height);
                WorkspaceSurrogate::new(
                  hwnd,
                  &rect,
                  &viewport,
                  opacity,
                  ws_config.opacity_incoming,
                  &corner_style,
                  acrylic_tint,
                )
                .map_err(|e| {
                  tracing::warn!(
                    "Failed to create incoming surrogate: {e}."
                  );
                  e
                })
                .ok()
              });
            // New surrogates are always inserted at `HWND_TOP`, which can
            // displace any other window's blur overlay out of its correct
            // z-order slot -- flag a full resync (see the field doc).
            if surrogate.is_some() {
              state.animation_manager.blur_overlay_z_order_dirty = true;
            }
            // Always register incoming windows even without a surrogate so
            // `is_frozen_by_ws_animation` is true for all of them — this
            // prevents the real window from being uncloaked before the
            // animation ends.
            ws_windows.push((id, surrogate, true));
          } else {
            let current = state
              .window_target_positions
              .get(&id)
              .cloned()
              .or_else(|| window.native().frame().ok())
              .unwrap_or_else(|| Rect::from_xy(0, 0, 0, 0));
            let viewport =
              Rect::from_xy(monitor_x, monitor_y, monitor_width, monitor_height);
            let surrogate = WorkspaceSurrogate::new(
              hwnd,
              &current,
              &viewport,
              opacity,
              ws_config.opacity_outgoing,
              &corner_style,
              acrylic_tint,
            )
            .map_err(|e| {
              tracing::warn!("Failed to create outgoing surrogate: {e}.");
              e
            })
            .ok();
            // See the matching comment in the incoming-surrogate branch
            // above.
            if surrogate.is_some() {
              state.animation_manager.blur_overlay_z_order_dirty = true;
            }
            ws_windows.push((id, surrogate, false));
          }
        }

        let has_outgoing =
          ws_windows.iter().any(|(_, _, is_incoming)| !*is_incoming);
        let has_incoming =
          ws_windows.iter().any(|(_, _, is_incoming)| *is_incoming);

        // For slide styles, skip when direction == 0: workspace names were not
        // found in the config so the slide offset would be 0, placing
        // surrogates at their target and causing an instant flash. Non-slide
        // styles (fade/zoom) have no slide offset so direction == 0 is fine.
        if (has_outgoing || has_incoming) && (direction != 0 || is_no_slide) {
          // Show outgoing surrogates before flushing: real windows are still
          // active so their DWM thumbnails are immediately warm.
          // For stationary (non-slide) styles, also show incoming surrogates at
          // their start opacity so DWM warms their thumbnails before the loop.
          for (_, ref mut surrogate, is_incoming) in &mut ws_windows {
            if let Some(s) = surrogate {
              if !*is_incoming {
                s.show_initial();
              } else if ws_config.style != WorkspaceSwitchStyle::Slide {
                s.show_incoming();
              }
            }
          }

          // Single flush: DWM renders one frame with outgoing surrogates
          // at full opacity, ensuring surrogate content is composited
          // before the real windows are cloaked below. Incoming windows
          // start off-screen, so DWM warms their thumbnails over the
          // first few frames of the slide without a visible gap.
          wm_platform::dwm_flush();

          state.animation_manager.start_workspace_switch(
            ws_windows,
            direction, // order_direction: +1/-1
            monitor_x,
            monitor_width,
            monitor_y,
            monitor_height,
            monitor_handle,
            config,
          );
        }
        // If the incoming workspace is empty, or direction == 0 (workspace not
        // in config), skip the animation.
      }
    }
  }

  // Get monitors by their optimal hide corner.
  let monitors_by_hide_corner = state.monitors_by_hide_corner();

  // Whether any window in this redraw cycle changes size. Pure translations
  // in the same cycle then share the `window_resize` timing so all edges
  // stay in lock-step during the relayout (see
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
  // the loop below so the whole relayout pays one `DwmFlush` instead of one
  // per window.
  //
  // Each window's surrogate must be visible and its overlays anchored for
  // one composited frame *before* the real window is cloaked, otherwise the
  // cloak can land on a frame where the surrogate's thumbnail is not yet
  // populated -- a blank flash at animation start. That barrier is a
  // `DwmFlush`, which blocks until DWM's next composition (~16.7 ms at
  // 60 Hz, ~5.7 ms at 175 Hz). Issued inline it multiplied by the number of
  // windows in the relayout, so a five-window resize stalled the WM's
  // single thread -- which also serves keybindings, mouse events and IPC --
  // for five whole frames before the animation's first frame even rendered.
  // The barrier is per-composition, not per-window, so one flush after the
  // loop satisfies every queued window at once.
  #[cfg(target_os = "windows")]
  let mut pending_cloaks: Vec<PendingCloak> = Vec::new();
  #[cfg(target_os = "windows")]
  let mut cloak_blur_batch = SurrogateBatch::new();
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

    // Whether the window should be shown above all other windows.
    let z_order = match window.state() {
      WindowState::Floating(config) if config.shown_on_top => {
        WindowZOrder::TopMost
      }
      WindowState::Fullscreen(config) if config.shown_on_top => {
        WindowZOrder::TopMost
      }
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
      z_order_touched.insert(window.id());
      if let Err(err) = window.native().set_z_order(&z_order) {
        tracing::warn!("Failed to set window z-order: {}", err);
      }
    }

    // Skip updating the window's position if it only required a z-order
    // change.
    if !windows_to_redraw.contains(window) {
      continue;
    }

    // Capture display state before transition to detect opening windows
    let previous_display_state = window.display_state();

    // Transition display state depending on whether window will be
    // shown or hidden.
    let new_display_state =
      match (previous_display_state.clone(), workspace.is_displayed()) {
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
    // `window_move` animation when the window just crossed the tiling/floating
    // boundary so the transition is smooth rather than a teleport.
    let is_floating = matches!(window.state(), WindowState::Floating(_));

    // Fullscreen windows are never animated: cloaking the real window (or
    // covering it with a surrogate) kicks exclusive-fullscreen games out of
    // fullscreen, reverting their resolution mode-set and re-triggering a
    // display-settings-changed relayout in a loop.
    let is_fullscreen =
      matches!(window.state(), WindowState::Fullscreen(_));
    let is_state_change =
      state.pending_sync.is_window_state_change(&window.id());

    let is_outgoing_switch =
      state.pending_sync.is_workspace_switch_outgoing(&window.id());

    // True while this window is an incoming participant in the active
    // workspace-switch animation. Unlike `is_workspace_switch_incoming` on
    // `pending_sync` (cleared after the first `platform_sync`), this stays
    // `true` for the full animation so that focus events during the slide do
    // not prematurely uncloak the real window.
    #[cfg(target_os = "windows")]
    let is_frozen_by_ws_animation = state
      .animation_manager
      .is_workspace_switch_incoming(&window.id());
    #[cfg(not(target_os = "windows"))]
    let is_frozen_by_ws_animation = false;

    // A window is resizing when its dimensions change (vs. a pure translation).
    let is_resize = previous_target
      .as_ref()
      .map(|prev| {
        prev.width() != target_rect.width()
          || prev.height() != target_rect.height()
      })
      .unwrap_or(false);

    let anim_enabled = if is_resize {
      config.value.animations.window_resize.enabled
    } else {
      config.value.animations.window_move.enabled
    };

    // Compute effect opacity and corner style unconditionally — needed for
    // both the movement surrogate path and the fade-in path.
    #[cfg(target_os = "windows")]
    let (effect_opacity, corner_style, blur_overlay, border_overlay) = {
      let effect_cfg = if window.id() == focused_container.id() {
        &config.value.window_effects.focused_window
      } else {
        &config.value.window_effects.other_windows
      };
      let opacity = if effect_cfg.transparency.enabled {
        effect_cfg.transparency.opacity.to_alpha()
      } else {
        u8::MAX
      };
      let style = if effect_cfg.corner_style.enabled {
        effect_cfg.corner_style.style.clone()
      } else {
        CornerStyle::Default
      };
      // Snapshotted onto the `ResizeSession` (rather than re-read live from
      // config at tracking time) since the close animation's direct-drive
      // loop runs after the window is detached from the container tree,
      // where `effect_cfg` can no longer be recomputed.
      let corner_radius = if effect_cfg.corner_style.enabled {
        effect_cfg.corner_style.style.approx_radius_px()
      } else {
        CornerStyle::Default.approx_radius_px()
      };
      let blur_overlay = effect_cfg
        .backdrop
        .acrylic_tint()
        .map(|tint| effect_cfg.backdrop.to_overlay_params(tint, corner_radius));
      let border_overlay = effect_cfg
        .border
        .abgr_color()
        .map(|color| effect_cfg.border.to_overlay_params(color, corner_radius));
      (opacity, style, blur_overlay, border_overlay)
    };

    // Start a slide-in animation for newly appearing tiling windows.
    // `previous_target.is_none()` is true only on the first `platform_sync`
    // call for this window, so the slide-in starts exactly once.
    #[cfg(target_os = "windows")]
    if previous_target.is_none()
      && is_visible
      && !is_floating
      && !is_fullscreen
      && !is_outgoing_switch
      && !is_frozen_by_ws_animation
      && !suppress_animations
      && config.value.animations.window_open.enabled
    {
      let monitor_rect = monitor.to_rect()?;
      let native_ref = window.native();
      state.animation_manager.start_open_animation(
        window.id(),
        target_rect.clone(),
        monitor_rect,
        effect_opacity,
        corner_style,
        blur_overlay,
        border_overlay,
        config,
        &*native_ref,
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
        .map_or(false, |a| !a.is_complete());
    #[cfg(not(target_os = "windows"))]
    let has_slide_in = false;

    // Windows frozen by an in-flight workspace-switch animation stay on the
    // animation path regardless of suppression — the switch's surrogate
    // teardown uncloaks them, so dropping them here would break its
    // invariants. Fullscreen windows and suppressed cycles otherwise always
    // take the non-animated path, which also cancels any in-flight
    // animation (and its surrogate) via `remove_animation` below.
    let should_use_animations = !is_outgoing_switch
      && (is_frozen_by_ws_animation
        || (!is_fullscreen
          && !suppress_animations
          && ((!is_floating && anim_enabled)
            || (is_state_change && anim_enabled)
            || has_slide_in)));

    // Determine the rect to use for this frame.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    let anim_scope = perf::scope(Stage::AnimStep);
    let (position_result, anim_opacity) = if should_use_animations {
      // Incoming workspace-switch windows: the surrogate handles all visuals
      // for the full animation duration — freeze the real window.
      #[cfg(target_os = "windows")]
      let native_ref = window.native();
      #[cfg(target_os = "windows")]
      if is_frozen_by_ws_animation {
        (AnimationPositionResult::Frozen, None)
      } else {
        state.animation_manager.start_animation_if_needed(
          window.id(),
          is_resize,
          cycle_has_resize,
          target_rect.clone(),
          previous_target,
          &*native_ref,
          effect_opacity,
          corner_style,
          blur_overlay,
          border_overlay,
          config,
        )
      }
      #[cfg(not(target_os = "windows"))]
      state.animation_manager.start_animation_if_needed(
        window.id(),
        is_resize,
        cycle_has_resize,
        target_rect.clone(),
        previous_target,
        u8::MAX,
        config,
      )
    } else {
      // Animations are skipped for this window. Cancel any in-progress
      // animation and its surrogate so subsequent ticks don't re-cloak it.
      state.animation_manager.remove_animation(&window.id());
      (AnimationPositionResult::Apply(target_rect.clone()), None)
    };

    drop(anim_scope);

    debug!("Updating window position: {window}");

    match position_result {
      AnimationPositionResult::Frozen => {
        let _scope = perf::scope(Stage::RedrawFrozen);

        // A surrogate overlay is covering this window. On the first frame,
        // cloak the real window (so only the surrogate is visible) and
        // synchronously pre-position it at its target rect. Both operations
        // are skipped on subsequent frames: they are idempotent, and repeating
        // a blocking `SetWindowPos` cross-process every tick stalls the
        // animation loop on slow apps and delays keybinding processing.
        //
        // `handle_window_hidden` is guarded against unmanaging cloaked windows
        // so cloaking is safe. If something unclocks the window mid-animation
        // the next tick will re-cloak and re-position it.
        //
        // For `ResizeSession`-backed animations, `pre_commit` also calls
        // `SetWindowPos` synchronously just before the surrogate drops,
        // guaranteeing the window is at `target_rect` when uncloaked.
        // Skip the per-tick `DwmGetWindowAttribute(DWMWA_CLOAKED)` round-trip
        // for resize-session windows whose cloak state is already known — the
        // check only fires on the first `Frozen` frame and after session
        // teardown. Workspace-switch frozen windows (no resize session) retain
        // the full per-tick guard as a safety net.
        #[cfg(target_os = "windows")]
        let already_cloaked_by_session = state
          .animation_manager
          .resize_sessions
          .get(&window.id())
          .map_or(false, |s| s.is_session_cloaked());
        #[cfg(target_os = "windows")]
        if !already_cloaked_by_session
          && !window.native().is_cloaked().unwrap_or(false)
        {
          let _scope = perf::scope(Stage::Cloak);

          // The surrogate is created hidden (`initially_visible: false`,
          // see `SessionOptions`) specifically so this can anchor the blur
          // overlay behind it *before* `ResizeSession::show` reveals it --
          // the surrogate's `HWND` is already stable at this point even
          // though it isn't visible yet, which is all `overlay_z_anchor`
          // needs. Without this, showing the surrogate first (as a plain
          // `initially_visible: true` session used to) leaves it visible
          // with no correctly-positioned overlay behind it until this
          // window's blur overlay upsert eventually catches up, giving DWM
          // a real window to composite an unblurred frame in.
          //
          // Deferred into the pass-wide batches (rather than each window
          // committing its own `SurrogateBatch`) so the whole relayout's
          // overlay repositions land in one `DeferWindowPos` transaction --
          // see `pending_cloaks`' declaration.
          if let Some(session) =
            state.animation_manager.resize_sessions.get(&window.id())
          {
            if let (Some(params), Some(anchor), Some(rect)) = (
              session.blur_overlay_params(),
              session.surrogate_hwnd(),
              session.current_rect(),
            ) {
              upsert_blur_overlay(
                &mut state.blur_overlays,
                window.id(),
                params,
                &rect,
                anchor,
                &mut cloak_blur_batch,
              );
            }
            if let (Some(params), Some(anchor), Some(rect)) = (
              session.border_overlay_params(),
              session.surrogate_hwnd(),
              session.current_rect(),
            ) {
              upsert_border_overlay(
                &mut state.border_overlays,
                window.id(),
                params,
                &rect,
                anchor,
                &mut cloak_border_batch,
              );
            }
          }
          if let Some(session) =
            state.animation_manager.resize_sessions.get_mut(&window.id())
          {
            session.show();
          }

          // Queued rather than cloaked here: the cloak has to be preceded
          // by a `DwmFlush`, and doing that inline blocks the WM thread for
          // a full composition frame *per window*. See `pending_cloaks`.
          pending_cloaks.push(PendingCloak {
            window: (*window).clone(),
            z_order: z_order.clone(),
            target_rect: target_rect.clone(),
          });
        }
      }
      AnimationPositionResult::Apply(ref apply_rect) => {
        let _scope = perf::scope(Stage::RedrawApply);

        // Only omit `SWP_ASYNCWINDOWPOS` when a surrogate is active for this
        // window — adjacent windows must stay in lock-step with the overlay.
        // For pure moves (no surrogate) async is correct and avoids blocking
        // on the target process's message queue each frame.
        // Also treat incoming ws-switch windows as having a surrogate when
        // being uncloaked at animation completion. Without this, the window
        // would be repositioned with `SWP_ASYNCWINDOWPOS` and immediately
        // uncloaked — if its message queue is slow, it appears at its old
        // position for one frame.
        #[cfg(target_os = "windows")]
        let has_surrogate = state
          .animation_manager
          .resize_sessions
          .contains_key(&window.id())
          || (is_visible
            && state
              .animation_manager
              .is_pending_ws_cleanup_incoming(&window.id()));
        #[cfg(not(target_os = "windows"))]
        let has_surrogate = false;

        // Skip the `SetWindowPos` on the animation-completion redraw when
        // `pre_commit` already positioned the window at exactly this rect.
        // Repositioning again is not just redundant — `SWP_FRAMECHANGED`
        // forces a frame recalculation and full repaint that lands right as
        // the window is uncloaked below, flashing at the end of every
        // move/resize animation. The uncloak and effects below still run.
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
        let mut cloak_state = CloakState::Untouched;

        if !already_positioned {
          {
            // Attribute the reposition to the window's own process. When
            // `has_surrogate` is set the `SetWindowPos` is synchronous and
            // blocks on that application's message pump, so a single slow
            // app can dominate `rd_apply` -- the stage totals alone cannot
            // show which one. Scoped tightly so only the reposition itself
            // is counted.
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
              Ok(state) => cloak_state = state,
              Err(err) => {
                tracing::warn!("Failed to set window position: {}", err);
              }
            }
          }

          #[cfg(target_os = "windows")]
          if is_visible {
            needs_transparency_flush = true;
          }
        }

        // Uncloak after repositioning so the window is revealed at the correct
        // position. This undoes `set_cloaked(true)` from the `Frozen` branch
        // for non-`HideMethod::Cloak` configurations (that method already
        // calls `set_cloaked` internally inside `reposition_window`).
        #[cfg(target_os = "windows")]
        if is_visible {
          let _uncloak_scope = perf::scope(Stage::ApplyUncloak);

          // Skipped when `reposition_window` just applied the same cloak
          // state: an identical `set_cloaked(false)` measured ~4.8ms of
          // pure duplicate work per window.
          if cloak_state == CloakState::Untouched {
            let _ = window.native().set_cloaked(false);
          }

          // Hide the workspace-switch surrogate thumbnail immediately after
          // uncloaking so both changes land in the same DWM composition frame.
          // Deferring the hide until after the full main loop would leave the
          // thumbnail visible during the remaining window processing time,
          // producing a multi-frame double-blend when transparency is enabled.
          state
            .animation_manager
            .hide_pending_ws_cleanup_surrogate(window.id());
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

    // Mark fullscreen windows as fullscreen on every redraw (including during animations)
    // to ensure browser fullscreen APIs work correctly.
    let is_transitioning_fullscreen =
      match (window.prev_state(), window.state()) {
        (Some(_), WindowState::Fullscreen(s)) if !s.maximized => true,
        (Some(WindowState::Fullscreen(_)), _) => true,
        _ => false,
      };

    let is_currently_fullscreen =
      matches!(window.state(), WindowState::Fullscreen(_));

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
    cloak_blur_batch,
    cloak_border_batch,
  );

  // Commit all surrogate repositions queued during this pass in a single
  // `DeferWindowPos` transaction so adjacent windows' edges land in the
  // same DWM composition frame.
  #[cfg(target_os = "windows")]
  state.animation_manager.flush_surrogate_updates();

  // Keep the acrylic blur overlay tracking each move/resize/open session's
  // surrogate at its just-flushed live rect, instead of leaving it hidden
  // for the whole animation. Close sessions are tracked separately, inside
  // `AnimationManager::update_internal`'s direct-drive loop -- that path
  // never reaches `platform_sync` (closing windows are detached from the
  // layout tree), so skip them here to avoid a duplicate upsert this tick.
  //
  // Repositions are batched into one `DeferWindowPos` transaction (separate
  // from the surrogate batch `flush_surrogate_updates` just committed above)
  // rather than each overlay issuing its own synchronous `SetWindowPos` --
  // that per-window cost scales with tick rate, which is most visible on
  // high-refresh-rate displays where the animation manager ticks in
  // lockstep with vsync.
  #[cfg(target_os = "windows")]
  {
    let _scope = perf::scope(Stage::SessionOverlays);

    let mut blur_batch = SurrogateBatch::new();
    let mut border_batch = SurrogateBatch::new();
    for (id, session) in &state.animation_manager.resize_sessions {
      if state.animation_manager.has_close_animation(id) {
        continue;
      }
      let anchor = session.surrogate_hwnd();
      let rect = session.current_rect();

      if let Some(params) = session.blur_overlay_params() {
        match (anchor, rect.clone()) {
          (Some(anchor), Some(rect)) => upsert_blur_overlay(
            &mut state.blur_overlays,
            *id,
            params,
            &rect,
            anchor,
            &mut blur_batch,
          ),
          _ => {
            if let Some(overlay) = state.blur_overlays.get_mut(id) {
              overlay.hide();
            }
          }
        }
      }

      if let Some(params) = session.border_overlay_params() {
        match (anchor, rect) {
          (Some(anchor), Some(rect)) => upsert_border_overlay(
            &mut state.border_overlays,
            *id,
            params,
            &rect,
            anchor,
            &mut border_batch,
          ),
          _ => {
            if let Some(overlay) = state.border_overlays.get_mut(id) {
              overlay.hide();
            }
          }
        }
      }
    }
    blur_batch.commit();
    border_batch.commit();
  }

  // Apply effect opacity to outgoing surrogates now that the real windows
  // have been cloaked. This removes the double-blend that would occur if
  // the surrogate's configured opacity were set before cloaking.
  #[cfg(target_os = "windows")]
  state.animation_manager.apply_outgoing_surrogate_opacities();

  // Force one composited frame reflecting every `reassert_transparency` call
  // made above, deterministically closing the race against DWM's own
  // compositor thread instead of leaving it to scheduling luck. One flush
  // covers every window that completed an animation this pass, rather than
  // blocking once per window.
  #[cfg(target_os = "windows")]
  if needs_transparency_flush {
    wm_platform::dwm_flush();
  }

  Ok(())
}

/// Above this duration, `reposition_window`'s synchronous (non-`ASYNCWINDOWPOS`)
/// `SetWindowPos`/restore/minimize/maximize sequence is logged as a warning.
/// See its `has_surrogate` doc comment for why this path can block on a slow
/// app's message queue. Mirrors `wm_platform::resize_session`'s
/// `SLOW_SYNC_SETWINDOWPOS_THRESHOLD`.
#[cfg(target_os = "windows")]
const SLOW_SYNC_REPOSITION_THRESHOLD: Duration = Duration::from_millis(8);

/// Whether [`reposition_window`] already applied the window's cloak state.
///
/// `redraw_containers`' `Apply` arm uncloaks visible windows itself, but
/// `HideMethod::Cloak` means [`reposition_window`] has already done exactly
/// that. The repeat `DwmSetWindowAttribute(DWMWA_CLOAK)` is not free: it
/// measured ~4.8ms per window, ~27% of the whole `rd_apply` stage on an
/// eight-window relayout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloakState {
  /// Applied by `reposition_window`; the caller must not repeat it.
  Applied,
  /// Untouched, so a visible window still needs uncloaking by the caller.
  Untouched,
}

fn reposition_window(
  window: &WindowContainer,
  rect: &Rect,
  hide_corner: HideCorner,
  // LINT: `z_order` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  z_order: &WindowZOrder,
  is_visible: bool,
  // When true, `SWP_ASYNCWINDOWPOS` is omitted so that adjacent windows move
  // synchronously with the surrogate overlay (both hit DWM in the same frame),
  // preventing a one-frame gap between the overlay and its neighbours.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  has_surrogate: bool,
  config: &UserConfig,
) -> anyhow::Result<CloakState> {
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

    return Ok(CloakState::Untouched);
  }

  let mut cloak_state = CloakState::Untouched;

  if window.active_drag().is_some() {
    window.native().resize(rect.width(), rect.height())?;
  } else {
    #[cfg(target_os = "macos")]
    window.native().set_frame(rect)?;

    #[cfg(target_os = "windows")]
    {
      use wm_platform::{
        SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOCOPYBITS,
        SWP_NOSENDCHANGING, WS_MAXIMIZEBOX,
      };

      // Restore window if it's minimized/maximized and shouldn't be. This
      // is needed to be able to move and resize it.
      let query_scope = perf::scope(Stage::RepositionQuery);
      let should_restore = match &window.state() {
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

      // During animation frames, omit `SWP_ASYNCWINDOWPOS` so that adjacent
      // windows are repositioned synchronously. This keeps their on-screen
      // position in lock-step with surrogate overlays (which update DWM
      // directly via `UpdateLayeredWindow`), closing the blank gap that
      // appears when async repositioning lags one frame behind the surrogate.
      //
      // `SWP_NOCOPYBITS` is only added for that same surrogate case: without
      // it, DWM can briefly show the window's old bitmap stretched/offset
      // over the new rect for one frame while it catches up to the
      // surrogate. Applying it unconditionally would force a full repaint
      // on every reposition -- including live interactive drag-resizing and
      // plain sibling-window reflows that never had a surrogate or a gap to
      // close -- which is needless repaint cost on the target app's own
      // thread and a real source of resize/move lag.
      let mut swp_flags = SWP_NOACTIVATE
        | SWP_NOSENDCHANGING
        | if has_surrogate {
          SWP_NOCOPYBITS
        } else {
          SWP_ASYNCWINDOWPOS
        };
      let sync_reposition_start = has_surrogate.then(Instant::now);
      let swp_scope = perf::scope(Stage::RepositionSwp);

      match &window.state() {
        WindowState::Minimized => {
          if !window.native().is_minimized()? {
            window.native().minimize()?;
          }
        }
        WindowState::Fullscreen(fullscreen)
          if fullscreen.maximized
            && window.native().has_window_style(WS_MAXIMIZEBOX) =>
        {
          if !window.native().is_maximized()? {
            window.native().maximize()?;
          }

          window.native().set_window_pos(z_order, rect, swp_flags)?;
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
        window.native().set_cloaked(!is_visible)?;
        cloak_state = CloakState::Applied;
      } else if is_visible {
        window.native().show()?;
      } else {
        window.native().hide()?;
      }
    }
  }

  Ok(cloak_state)
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

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.corner_style.enabled
    || window_effects.other_windows.corner_style.enabled
  {
    apply_corner_effect(window, effect_config);
  }

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.transparency.enabled
    || window_effects.other_windows.transparency.enabled
  {
    apply_transparency_effect(window, effect_config);
  }

  #[cfg(target_os = "windows")]
  if window_effects.focused_window.backdrop.enabled
    || window_effects.other_windows.backdrop.enabled
  {
    apply_backdrop_effect(window, effect_config);
  }
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
  let corner_style = if effect_config.corner_style.enabled {
    &effect_config.corner_style.style
  } else {
    &CornerStyle::Default
  };

  _ = window.native().set_corner_style(corner_style);
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

#[cfg(target_os = "windows")]
fn apply_backdrop_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  // `Acrylic` is handled via a persistent `NativeBlurOverlay` placed at
  // `HWND_BOTTOM` behind the managed window — SWCA is not applied to the
  // managed window itself to avoid the `WS_EX_LAYERED`/SWCA compositing
  // conflict. `Mica`/`MicaAlt` use `DWMWA_SYSTEMBACKDROP_TYPE` directly
  // on the managed window (Win11 22H2+).
  let style = match (
    effect_config.backdrop.enabled,
    &effect_config.backdrop.style,
  ) {
    (true, BackdropStyle::Mica | BackdropStyle::MicaAlt) => {
      Some(&effect_config.backdrop.style)
    }
    _ => None,
  };

  if let Err(e) = window.native().set_blur_behind(style) {
    warn!("Failed to set blur-behind on window: {e}.");
  }
}

/// Creates, repositions, and removes acrylic blur overlay windows so that
/// every managed window with `backdrop: acrylic` has a matching overlay
/// positioned at `HWND_BOTTOM` flush with the window's DWM frame rect.
///
/// Called on every `platform_sync` tick so overlays track window position
/// through moves, resizes, and focus changes. The overlay is hidden (but
/// kept alive) for windows on inactive workspaces so it can be re-shown
/// instantly when the workspace becomes active again.
///
/// During animations, surrogates carry the acrylic effect directly via SWCA
/// so blur is visible throughout the animation. The static overlay is hidden
/// while a surrogate is active and restored once the animation completes.
///
/// The tint is re-evaluated for every window on every tick (cheap: it
/// no-ops internally unless the resolved value changed, e.g. on focus
/// change). The window's on-screen rect, however, is only re-queried via
/// `frame()` -- a cross-process `DwmGetWindowAttribute` call -- for windows
/// actually queued for redraw this tick (or on first creation). GlazeWM owns
/// all positioning of managed windows through that same redraw queue, so a
/// window that isn't in it this tick cannot have moved; skipping the query
/// avoids a `DwmGetWindowAttribute` round-trip per acrylic window per tick
/// for windows unrelated to whatever triggered this sync (e.g. other
/// monitors/workspaces during a workspace-switch animation elsewhere).
/// Creates or updates a tracked acrylic blur overlay for `window_id`,
/// applying `tint`/`blur_amount`/`corner_radius` and moving it to `rect`.
///
/// Used by the workspace-switch live-tracking driver in
/// `AnimationManager::update_internal`, which always has a definite
/// up-to-date rect from the surrogate it's following each tick and so needs
/// no conditional re-query -- unlike `sync_blur_overlays` below, which only
/// re-queries `frame()` (a cross-process call) when the window is actually
/// being redrawn this tick.
///
/// Takes the overlay map directly (rather than `&mut WmState`) so callers
/// already holding an unrelated borrow into other `WmState` fields -- e.g.
/// the workspace-switch driver, which calls this while still borrowing
/// `state.animation_manager.workspace_switch` for `rect` itself -- can pass
/// `&mut state.blur_overlays` without a borrow-checker conflict.
#[cfg(target_os = "windows")]
pub(crate) fn upsert_blur_overlay(
  overlays: &mut std::collections::HashMap<uuid::Uuid, NativeBlurOverlay>,
  window_id: uuid::Uuid,
  params: BlurOverlayParams,
  rect: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  match overlays.entry(window_id) {
    std::collections::hash_map::Entry::Occupied(e) => {
      let overlay = e.into_mut();
      overlay.apply(params);
      overlay.defer_rect(batch, rect, anchor);
    }
    std::collections::hash_map::Entry::Vacant(e) => {
      match NativeBlurOverlay::create(rect, params, anchor) {
        Ok(overlay) => {
          debug!("Blur overlay created for {window_id}.");
          e.insert(overlay);
        }
        Err(err) => {
          debug!("Blur overlay creation failed for {window_id}: {err}.");
        }
      }
    }
  }
}

/// Creates or updates a tracked border overlay for `window_id`, applying
/// `color`/`width`/`corner_radius` and moving it to `rect`. Mirrors
/// [`upsert_blur_overlay`] exactly -- see its doc comment for the shared
/// rationale (used by both the workspace-switch live-tracking driver and
/// the static per-tick path, and takes the overlay map directly so callers
/// already holding an unrelated borrow into other `WmState` fields can pass
/// `&mut state.border_overlays` without a borrow-checker conflict).
#[cfg(target_os = "windows")]
pub(crate) fn upsert_border_overlay(
  overlays: &mut std::collections::HashMap<uuid::Uuid, NativeBorderOverlay>,
  window_id: uuid::Uuid,
  params: BorderOverlayParams,
  rect: &Rect,
  anchor: HWND,
  batch: &mut SurrogateBatch,
) {
  match overlays.entry(window_id) {
    std::collections::hash_map::Entry::Occupied(e) => {
      let overlay = e.into_mut();
      overlay.apply(params);
      overlay.defer_rect(batch, rect, anchor);
    }
    std::collections::hash_map::Entry::Vacant(e) => {
      match NativeBorderOverlay::create(rect, params, anchor) {
        Ok(overlay) => {
          debug!("Border overlay created for {window_id}.");
          e.insert(overlay);
        }
        Err(err) => {
          debug!("Border overlay creation failed for {window_id}: {err}.");
        }
      }
    }
  }
}

/// Resolves `window_id`'s border overlay params from its focused/other-window
/// border config, or `None` when the border effect isn't enabled for it.
/// Mirrors [`blur_overlay_params_for`].
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
  let corner_radius = if effect_cfg.corner_style.enabled {
    effect_cfg.corner_style.style.approx_radius_px()
  } else {
    CornerStyle::Default.approx_radius_px()
  };

  Some(effect_cfg.border.to_overlay_params(target_color, corner_radius))
}

/// Resolves `window`'s acrylic overlay params from its focused/other-window
/// backdrop config, or `None` when backdrop isn't configured/enabled or
/// isn't set to `BackdropStyle::Acrylic` for it.
///
/// Shared by [`sync_blur_overlays`] (the per-tick static path) and the
/// interactive-drag tracker in `handle_window_moved_or_resized`, which needs
/// the same params to keep the overlay live while the OS drags the window
/// outside of `GlazeWM`'s own redraw pipeline.
#[cfg(target_os = "windows")]
pub(crate) fn blur_overlay_params_for(
  is_focused: bool,
  config: &UserConfig,
) -> Option<BlurOverlayParams> {
  let effect_cfg = if is_focused {
    &config.value.window_effects.focused_window
  } else {
    &config.value.window_effects.other_windows
  };

  let tint = effect_cfg.backdrop.acrylic_tint()?;

  // Mirrors `corner_style` (falling back to `CornerStyle::Default` when
  // disabled, same as `apply_corner_effect`) so the overlay's rounded clip
  // lines up with the real managed window's own DWM-rendered corners
  // sitting on top of it, rather than being an independently configured
  // radius that can mismatch what's actually on screen.
  let corner_radius = if effect_cfg.corner_style.enabled {
    effect_cfg.corner_style.style.approx_radius_px()
  } else {
    CornerStyle::Default.approx_radius_px()
  };

  Some(effect_cfg.backdrop.to_overlay_params(tint, corner_radius))
}

/// Resolves the z-order anchor to keep `window`'s acrylic overlay pinned
/// directly behind it (see [`NativeBlurOverlay`]'s doc comment).
///
/// Windows keeps "always on top" windows in a separate band above every
/// other window, and a *non*-topmost overlay wedged in via `window`'s own
/// `HWND` doesn't reliably stay adjacent to a topmost `window` -- the OS can
/// silently displace it out of that exact slot, which then shows the
/// overlay itself (flat tint+blur, no live window content behind it)
/// instead of it staying invisible behind the window. Returning
/// `HWND_TOPMOST` here instead makes the overlay topmost too, so both stay
/// pinned together in the same band -- mirroring `window.native().set_z_order`'s
/// own `WindowZOrder::TopMost` case just above in this file.
#[cfg(target_os = "windows")]
pub(crate) fn overlay_z_anchor(window: &WindowContainer) -> HWND {
  let is_topmost = matches!(
    window.state(),
    WindowState::Floating(config) if config.shown_on_top
  ) || matches!(
    window.state(),
    WindowState::Fullscreen(config) if config.shown_on_top
  );

  if is_topmost {
    HWND_TOPMOST
  } else {
    window.native().hwnd()
  }
}

/// A per-window overlay effect (acrylic blur or border) kept in sync with
/// its window's rect and z-order every `platform_sync` tick.
///
/// Implemented for [`NativeBlurOverlay`] and [`NativeBorderOverlay`] so
/// [`sync_overlays`] can drive both through one shared implementation
/// instead of two ~160-line copies that had drifted enough to hide a real
/// bug -- see `full_z_order_resync`'s doc comment at the `sync_overlays`
/// call site.
#[cfg(target_os = "windows")]
trait SyncableOverlay: Sized {
  /// Effect-specific config resolved from `WindowEffectConfig` (tint/blur
  /// amount for blur, color/width for border). `Copy` so [`sync_overlays`]
  /// can resolve it once per focus class (focused/other) instead of once
  /// per window -- see the call site.
  type Params: Copy;

  /// Label used in this overlay kind's debug log messages (e.g. `"Blur"`).
  const LABEL: &'static str;

  /// Profiler stage this overlay kind's [`sync_overlays`] pass reports as.
  const PERF_STAGE: Stage;

  /// Borrows this overlay kind's tracked-overlay map out of `state`.
  fn overlays(
    state: &mut WmState,
  ) -> &mut std::collections::HashMap<uuid::Uuid, Self>;

  /// Resolves the focused/other-window config's params, or `None` when this
  /// effect isn't enabled for that focus class.
  ///
  /// Depends only on `is_focused` and `config`, not on any particular
  /// window -- [`sync_overlays`] calls this twice per tick (once per focus
  /// class) rather than once per window.
  fn params_for(
    is_focused: bool,
    config: &UserConfig,
  ) -> Option<Self::Params>;

  fn create(
    rect: &Rect,
    params: Self::Params,
    anchor: HWND,
  ) -> wm_platform::Result<Self>;
  fn apply(&mut self, params: Self::Params);
  fn defer_rect(&mut self, batch: &mut SurrogateBatch, rect: &Rect, anchor: HWND);
  fn sync_z_order(&mut self, anchor: HWND) -> wm_platform::Result<()>;
  fn is_visible(&self) -> bool;
  fn hide(&mut self);
}

#[cfg(target_os = "windows")]
impl SyncableOverlay for NativeBlurOverlay {
  type Params = BlurOverlayParams;
  const LABEL: &'static str = "Blur";
  const PERF_STAGE: Stage = Stage::BlurSync;

  fn overlays(
    state: &mut WmState,
  ) -> &mut std::collections::HashMap<uuid::Uuid, Self> {
    &mut state.blur_overlays
  }

  fn params_for(
    is_focused: bool,
    config: &UserConfig,
  ) -> Option<Self::Params> {
    blur_overlay_params_for(is_focused, config)
  }

  fn create(
    rect: &Rect,
    params: Self::Params,
    anchor: HWND,
  ) -> wm_platform::Result<Self> {
    Self::create(rect, params, anchor)
  }

  fn apply(&mut self, params: Self::Params) {
    Self::apply(self, params);
  }

  fn defer_rect(&mut self, batch: &mut SurrogateBatch, rect: &Rect, anchor: HWND) {
    Self::defer_rect(self, batch, rect, anchor);
  }

  fn sync_z_order(&mut self, anchor: HWND) -> wm_platform::Result<()> {
    Self::sync_z_order(self, anchor)
  }

  fn is_visible(&self) -> bool {
    Self::is_visible(self)
  }

  fn hide(&mut self) {
    Self::hide(self);
  }
}

#[cfg(target_os = "windows")]
impl SyncableOverlay for NativeBorderOverlay {
  type Params = BorderOverlayParams;
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

  fn create(
    rect: &Rect,
    params: Self::Params,
    anchor: HWND,
  ) -> wm_platform::Result<Self> {
    Self::create(rect, params, anchor)
  }

  fn apply(&mut self, params: Self::Params) {
    Self::apply(self, params);
  }

  fn defer_rect(&mut self, batch: &mut SurrogateBatch, rect: &Rect, anchor: HWND) {
    Self::defer_rect(self, batch, rect, anchor);
  }

  fn sync_z_order(&mut self, anchor: HWND) -> wm_platform::Result<()> {
    Self::sync_z_order(self, anchor)
  }

  fn is_visible(&self) -> bool {
    Self::is_visible(self)
  }

  fn hide(&mut self) {
    Self::hide(self);
  }
}

/// Creates, repositions, and removes overlay windows of kind `O` so every
/// managed window with the matching effect enabled has an overlay tracking
/// its rect and z-order. Shared by acrylic blur and border overlays -- see
/// [`SyncableOverlay`].
///
/// Windows with a `Live`-mode (acrylic-tinted) workspace-switch surrogate
/// are owned entirely by the per-tick tracker in
/// `AnimationManager::update_internal` for the whole slide plus the
/// `pending_ws_cleanup` grace tick -- it updates `tint`/`blur_amount`/
/// `corner_radius`/`rect` every tick using the surrogate's own live position,
/// which this function cannot do since it isn't invoked on most mid-slide
/// ticks (only when `platform_sync` itself runs). Leave these overlays
/// untouched here entirely: falling through to the hide logic below would
/// hide them out from under the tracker, since the outgoing workspace's
/// `!is_displayed()` flips true the instant the switch command runs, well
/// before the animation completes.
///
/// Windows with an active or fading `ResizeSession` (move/resize/open/
/// close) are likewise owned entirely by the trackers in `platform_sync`'s
/// post-flush loop, `AnimationManager::update_internal`'s close direct-drive
/// loop, and its `pending_session_cleanup` fade-tail loop -- same reasoning
/// as the workspace-switch case above.
///
/// Otherwise, hides the static overlay while some other kind of
/// surrogate/session is active for this window without a live tracker of
/// its own (e.g. this effect not configured for it). When such a surrogate
/// is running, the real window is cloaked at its target rect; reading
/// `frame()` would return the target position while the surrogate is still
/// mid-animation, causing the overlay to jump ahead. Also hides for windows
/// on inactive workspaces; the entry stays alive so it can be re-shown
/// immediately when the workspace returns.
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
fn sync_overlays<O: SyncableOverlay>(
  state: &mut WmState,
  config: &UserConfig,
  focused_container: &Container,
  z_order_touched: &std::collections::HashSet<uuid::Uuid>,
  full_z_order_resync: bool,
) {
  let _scope = perf::scope(O::PERF_STAGE);

  let all_windows = state.windows();
  let mut wanted_ids = std::collections::HashSet::new();
  let mut batch = SurrogateBatch::new();

  // `containers_to_redraw()` may hold an ancestor (e.g. a whole workspace on
  // a workspace switch) rather than each window individually -- mirror the
  // same descendant expansion `redraw_containers` uses via `windows_to_redraw`
  // so a window whose ancestor was queued still counts as redrawing.
  let redrawing_ids: std::collections::HashSet<_> =
    state.windows_to_redraw().iter().map(CommonGetters::id).collect();

  // `params_for` depends only on `is_focused`, not on any particular
  // window -- resolved once per focus class here instead of once per
  // window in the loop below, so a config with a `{ file, key }` color
  // source pays its `fs::metadata` stat once per tick rather than once
  // per tracked window.
  let focused_params = O::params_for(true, config);
  let other_params = O::params_for(false, config);

  for window in &all_windows {
    let is_focused = window.id() == focused_container.id();

    let Some(params) = (if is_focused { focused_params } else { other_params })
    else {
      continue;
    };

    wanted_ids.insert(window.id());

    if state.animation_manager.has_live_ws_surrogate(&window.id())
      || state.animation_manager.has_live_resize_tracker(&window.id())
    {
      continue;
    }

    let should_hide = state.animation_manager.has_active_surrogate(&window.id())
      || !window.workspace().is_some_and(|ws| ws.is_displayed());

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
        // surrogate was active) stays hidden indefinitely once `should_hide`
        // clears, unless this exact window happens to be redrawn again for
        // an unrelated reason (see `is_visible` doc comment).
        if is_redrawing || !overlay.is_visible() {
          // Use the DWM extended frame bounds so the overlay sits flush
          // with the window's visible edge, clipping the invisible resize
          // border.
          match window.native().frame() {
            Ok(rect) => overlay.defer_rect(&mut batch, &rect, anchor),
            Err(err) => debug!(
              "{} overlay frame() query failed for {}: {err}.",
              O::LABEL,
              window.id()
            ),
          }
        } else if full_z_order_resync
          || z_order_touched.contains(&window.id())
        {
          if let Err(err) = overlay.sync_z_order(anchor) {
            debug!(
              "{} overlay z-order sync failed for {}: {err}.",
              O::LABEL,
              window.id()
            );
          }
        }
      }
      std::collections::hash_map::Entry::Vacant(e) => {
        debug_assert!(!had_overlay);

        let Ok(rect) = window.native().frame() else {
          continue;
        };

        match O::create(&rect, params, anchor) {
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

  // Destroy overlays for windows that no longer need them.
  O::overlays(state).retain(|id, _| wanted_ids.contains(id));

  batch.commit();
}
