use anyhow::Context;
use tracing::debug;
#[cfg(target_os = "windows")]
use tracing::warn;
#[cfg(target_os = "windows")]
use wm_common::WindowEffectConfig;
use wm_common::{
  CursorJumpTrigger, DisplayState, HideCorner, HideMethod, WindowState,
  WmEvent,
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
  StyleBlend, WindowBatch, HWND,
};

#[cfg(target_os = "windows")]
use crate::{
  commands::window::unmanage_if_embedded, models::is_inactive_stack_child,
};
use crate::{
  layout::is_hidden_in_layout,
  models::{Container, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

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
    crate::animation::scroll_view::sync_scroll_view(state, config);
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

  // Focus is synced after `redraw_containers`, once the window to focus
  // is shown.
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

  // Sync backdrop and border overlays every tick so they track window
  // position through moves, resizes, and workspace changes. See
  // `sync_overlays`.
  #[cfg(target_os = "windows")]
  sync_overlays::<NativeBackdropOverlay>(
    state,
    config,
    &focused_container,
    &z_order_touched,
  );
  #[cfg(target_os = "windows")]
  sync_overlays::<NativeBorderOverlay>(
    state,
    config,
    &focused_container,
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

/// Moves, shows and hides the windows queued for redraw, and raises the
/// ones to bring to front. Returns the IDs of windows whose z-order this
/// pass changed *without* repositioning them -- see `z_order_touched`'s
/// declaration below. A window this pass also redraws is not included:
/// its overlay is already resynced on the strength of having moved.
#[allow(clippy::too_many_lines)]
fn redraw_containers(
  focused_container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<std::collections::HashSet<uuid::Uuid>> {
  let _scope = perf::scope(Stage::Redraw);
  let prep_scope = perf::scope(Stage::RedrawPrep);

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
    // index below is an O(total windows) tree walk -- skip it unless there
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

  // Get monitors by their optimal hide corner.
  let monitors_by_hide_corner = state.monitors_by_hide_corner();

  // Set when a visible window was moved with `SWP_FRAMECHANGED` this pass
  // outside a scene. `reassert_transparency` inside `reposition_window` is
  // only a best-effort fix-up for the full-opacity flash that flag causes
  // (it races DWM's own compositor thread), so a single flush after the
  // whole loop deterministically forces one composited frame with the
  // corrected alpha, for every such window in one blocking call.
  #[cfg(target_os = "windows")]
  let mut needs_transparency_flush = false;

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

    // Transition display state depending on whether window will be
    // shown or hidden. Only the active tab of a stack is shown.
    let previous_display_state = window.display_state();
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
    // shows, and for a window kept at its size while its copy shrinks.
    let real_rect = crate::layout::scrolling::real_window_rect(
      &window.as_container(),
      &target_rect,
    );
    #[cfg(target_os = "windows")]
    let real_rect = state
      .animation_manager
      .scenes
      .real_rect(&window.id(), real_rect);

    let is_visible = matches!(
      window.display_state(),
      DisplayState::Showing | DisplayState::Shown
    );

    state
      .window_target_positions
      .insert(window.id(), target_rect.clone());

    debug!("Updating window position: {window}");

    let apply_scope = perf::scope(Stage::RedrawApply);

    // Whether the uncloak below is still owed. `reposition_window`
    // performs it itself under `HideMethod::Cloak`, and repeating the
    // `DwmSetWindowAttribute` call is the single most expensive thing
    // here -- see `CloakState`.
    #[cfg(target_os = "windows")]
    let mut cloak_state = CloakState::Untouched;

    {
      let _apply_scope = perf::apply_scope(
        || window.native_properties().process_name,
        false,
      );

      match reposition_window(
        window,
        &real_rect,
        *hide_corner,
        &z_order,
        is_visible,
        config,
      ) {
        // LINT: Its fields are Windows-only.
        #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
        Ok(reposition) => {
          #[cfg(target_os = "windows")]
          {
            cloak_state = reposition.cloak_state;

            // Beneath a scene, the flash is never seen.
            needs_transparency_flush |= is_visible
              && reposition.moved
              && !state.animation_manager.scenes.covers(monitor.id());
          }
        }
        Err(err) => {
          tracing::warn!("Failed to set window position: {}", err);
        }
      }
    }

    // Uncloak after repositioning so the window is revealed at the
    // correct position, unless `reposition_window` already applied the
    // cloak state.
    #[cfg(target_os = "windows")]
    if is_visible && cloak_state == CloakState::Untouched {
      let _uncloak_scope = perf::scope(Stage::ApplyUncloak);
      let _ = window.native().set_cloaked(false);
    }

    drop(apply_scope);

    // Mark fullscreen windows as fullscreen on every redraw to ensure
    // browser fullscreen APIs work correctly.
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

  #[cfg(target_os = "windows")]
  if needs_transparency_flush {
    wm_platform::dwm_flush();
  }

  Ok(z_order_touched)
}

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
}

/// Whether [`reposition_window`] already applied the window's cloak state.
///
/// `redraw_containers` uncloaks visible windows itself, but
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
    });
  }

  #[cfg(target_os = "windows")]
  let mut cloak_state = CloakState::Untouched;
  #[cfg(target_os = "windows")]
  let mut moved = true;

  if window.active_drag().is_some() {
    window.native().resize(rect.width(), rect.height())?;
  } else {
    #[cfg(target_os = "macos")]
    window.native().set_frame(rect)?;

    #[cfg(target_os = "windows")]
    {
      use wm_platform::{
        discard_bits_if_resized, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOSENDCHANGING, WS_MAXIMIZEBOX,
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

      // Moved asynchronously, so a slow app can't hold up the WM. A move
      // that resizes discards the old bits (see
      // `discard_bits_if_resized`); a pure one keeps its
      // still-correct pixels and spares the app a repaint.
      let mut swp_flags = SWP_NOACTIVATE
        | SWP_NOSENDCHANGING
        | SWP_ASYNCWINDOWPOS
        | discard_bits_if_resized(current_frame.as_ref(), rect);
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
        // this function.
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
/// and moving it to `rect`, for a caller that knows where the window is.
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
  batch: &mut WindowBatch,
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
  batch: &mut WindowBatch,
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
  batch: &mut WindowBatch,
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

  let mut batch = WindowBatch::new();
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
  batch: &mut WindowBatch,
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
    // A peek's overlays stay behind the peek.
    let anchor = state
      .animation_manager
      .scroll_view
      .peek_of(&window.id())
      .map_or_else(|| overlay_z_anchor(&window), |(_, peek)| peek);

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

/// The effects a window wears focused or not, for what draws it in its
/// place: `(effect_opacity, corner_style, backdrop_overlay,
/// border_overlay)`.
#[cfg(target_os = "windows")]
pub(crate) fn effects_for(
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
/// Applies to the overlay and to the window's copies on a scene, so a
/// fullscreen transition doesn't animate a ring that the window won't
/// have once it lands.
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

/// Where `window`'s overlays go: around its visible frame, as DWM reports
/// it so they sit flush with its edge, past its invisible resize borders.
/// Beneath a scene (`is_covered`), around where the layout puts it
/// instead: the scene copies them there, while the window's asynchronous
/// move may not have landed yet.
#[cfg(target_os = "windows")]
fn overlay_frame(
  window: &WindowContainer,
  is_covered: bool,
) -> anyhow::Result<Rect> {
  match crate::animation::scene::layout_frame(window) {
    Some(frame) if is_covered => Ok(frame),
    _ => Ok(window.native().frame()?),
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
/// `!is_displayed()` flips true the instant the switch command runs; so
/// do windows whose close a scene plays.
///
/// Hides the overlays of windows on inactive workspaces; the entry stays
/// alive so it can be re-shown immediately when the workspace returns.
///
/// An overlay that didn't move is resynced only when `z_order_touched`
/// names its window (its own real z-order was actually touched this cycle
/// by `redraw_containers` or `sync_focus`) -- an overlay whose anchor
/// didn't move can't have drifted (see `sync_z_order`'s doc comment), and
/// unconditionally resyncing every configured window on every tick would
/// issue a `GetWindow` syscall per window that's a guaranteed no-op in the
/// common case.
#[cfg(target_os = "windows")]
#[allow(clippy::too_many_lines)]
fn sync_overlays<O: SyncableOverlay>(
  state: &mut WmState,
  config: &UserConfig,
  focused_container: &Container,
  z_order_touched: &std::collections::HashSet<uuid::Uuid>,
) {
  let _scope = perf::scope(O::PERF_STAGE);

  let all_windows = state.windows();
  let mut wanted_ids = std::collections::HashSet::new();
  let mut batch = WindowBatch::new();

  // Overlays whose z-order needs re-asserting, applied *after* the batch
  // below is committed. Issuing a `SetWindowPos` mid-loop would reorder an
  // overlay while it still sits at its previous position -- the batch has
  // not moved it yet -- so DWM composites a frame of it re-stacked but
  // stale. Across a layout change that is every overlay flashing at once,
  // which is exactly the artifact this ordering avoids.
  let mut z_order_resyncs: Vec<(uuid::Uuid, HWND, bool)> = Vec::new();

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

    // A scene still showing a copy of one leaves it on screen beneath it,
    // and the scroll view keeps a peek's on the peek.
    if state.animation_manager.scenes.holds(&window.id())
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
      !window.workspace().is_some_and(|ws| ws.is_displayed())
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
    let is_covered = window.monitor().is_some_and(|monitor| {
      state.animation_manager.scenes.covers(monitor.id())
    });

    match O::overlays(state).entry(window.id()) {
      std::collections::hash_map::Entry::Occupied(e) => {
        let overlay = e.into_mut();
        overlay.apply(params);

        // Always re-query and re-show when the overlay isn't currently
        // visible, even if this window isn't part of this tick's redraw --
        // otherwise an overlay hidden on some earlier tick stays hidden
        // indefinitely once `should_hide` clears, unless this exact window
        // happens to be redrawn again for an unrelated reason (see
        // `is_visible` doc comment).
        let repositioned = is_redrawing || !overlay.is_visible();

        if repositioned {
          match overlay_frame(window, is_covered) {
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

        // Overlays that moved this tick.
        //
        // `defer_rect` batches through `DeferWindowPos` with
        // `SWP_NOZORDER`, so it moves an overlay without restoring where
        // it sits in the stack -- that part still has to be re-asserted.
        // An overlay nobody moved has nothing to re-assert, unless its
        // window's z-order was changed *without* being repositioned --
        // `redraw_containers`' bare `set_z_order` and `sync_focus`'
        // `SetForegroundWindow`. Those leave the overlay behind with
        // nothing in the overlay's own state to show for it, which is what
        // `z_order_touched` reports. They force the re-assert rather than
        // letting `sync_z_order` decide: the window's move is issued with
        // `SWP_ASYNCWINDOWPOS` and may not have landed yet, so the
        // `GW_HWNDPREV` check can still see the stale (correct-looking)
        // ordering and skip. See `sync_z_order`'s doc comment.
        let forced = z_order_touched.contains(&window.id());

        if repositioned || forced {
          z_order_resyncs.push((window.id(), anchor, forced));
        }
      }
      std::collections::hash_map::Entry::Vacant(e) => {
        debug_assert!(!had_overlay);

        let Ok(rect) = overlay_frame(window, is_covered) else {
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
/// The settle is extended rather than left to run out while the window's
/// own band change has not landed. Letting it expire left a floating
/// window's border in the normal band, where the next tiled window to be
/// raised covered it.
#[cfg(target_os = "windows")]
pub(crate) fn resync_settling_overlays(state: &mut WmState) {
  let settling = state.animation_manager.settling_overlay_windows();
  if settling.is_empty() {
    return;
  }

  for id in settling {
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
