//! Workspace switches: their surrogates' motion, and handing windows over
//! to them and back.

use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_common::{
  EasingFunction, WorkspaceSwitchDirection, WorkspaceSwitchStyle,
};
use wm_platform::{
  perf::{self, Stage},
  NativeWindow, Overlay, Rect, SurrogateBatch, WorkspaceSurrogate, HWND,
};

use super::AnimationManager;
use crate::{
  commands::general::{
    border_hidden_for_fullscreen, overlay_params_for, overlay_z_anchor,
    track_overlays,
  },
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  window_overlays::OverlayPlacement,
  wm_state::WmState,
};

/// Maximum residual slide distance, in pixels, at which a workspace-switch
/// completes early.
///
/// Decelerating easing curves spend a large fraction of their wall-clock
/// duration covering the final sliver of distance, which looks "stuck" at
/// the destination. Completing early avoids that crawl, but the surrogate
/// is then snapped the remaining distance to its target in a single frame
/// before the real windows uncloak. Gating that completion on a fixed
/// *pixel* distance (rather than a fixed progress fraction) keeps the snap
/// below one pixel regardless of slide distance or duration, so it is
/// imperceptible — a fixed 1% fraction would snap ~34px across a 3440px
/// monitor.
pub(super) const WS_COMPLETE_THRESHOLD_PX: f32 = 1.5;

/// How far a drift moves each workspace, as a fraction of the monitor's
/// extent along the slide axis.
const DRIFT_DISTANCE: f32 = 0.05;

/// How much smaller the incoming workspace starts in a soft zoom.
const SOFT_ZOOM_INCOMING: f32 = 0.05;

/// How much larger the outgoing workspace ends in a soft zoom.
const SOFT_ZOOM_OUTGOING: f32 = 0.03;

/// Tracks a single window's participation in the current workspace-switch
/// slide animation.
pub(crate) struct WorkspaceSwitchEntry {
  /// Surrogate overlay that slides across the monitor each frame.
  pub surrogate: Option<WorkspaceSurrogate>,
  /// `true` for windows on the incoming workspace, `false` for outgoing.
  pub is_incoming: bool,
  /// Whether the window wore the focused style when the switch began.
  ///
  /// Its border, backdrop, and transparency blend from that style to the
  /// one for its post-switch focus state over the switch, rather than
  /// snapping on the first frame.
  pub was_focused: bool,
  /// How far its real window is shown again in its place, once the slide
  /// is over (see `AnimationManager::hide_pending_ws_cleanup_surrogate`).
  pub hand_back: HandBack,
}

/// How far a window of a finished workspace switch is handed back from its
/// surrogate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HandBack {
  /// Its real window is still cloaked.
  #[default]
  Pending,
  /// Uncloaked under its surrogate, which covers it until the window's
  /// companion is back over it (see
  /// `WorkspaceSurrogate::awaits_companion`).
  AwaitingCompanion,
  /// Shown again in its place.
  Done,
  /// Drawn by its resting peek again, its real window left cloaked as
  /// the layout hides it (see `AnimationManager::hand_back_to_peek`).
  ToPeek,
}

/// A step of tearing down a finished workspace switch's surrogates.
///
/// Each step changes what is on screen, which DWM has to have composited
/// before the next: the uncloak before the overlays move, those before the
/// surrogates go. Each runs a tick after the last: the tick loop is paced
/// by the animation thread's vblank wait, so the next tick is a later
/// composition, without the WM thread blocking on a `DwmFlush` for each
/// (three in a row at the end of every switch).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum WsCleanupStep {
  /// Waits for the incoming windows' uncloak, issued this tick.
  #[default]
  AwaitUncloak,
  /// Shows their overlays where they end, still behind the surrogates.
  ShowOverlays,
  /// Moves those overlays behind the windows themselves.
  Reanchor,
  /// Drops the surrogates.
  Drop,
}

impl WsCleanupStep {
  fn next(self) -> Self {
    match self {
      Self::AwaitUncloak => Self::ShowOverlays,
      Self::ShowOverlays => Self::Reanchor,
      Self::Reanchor | Self::Drop => Self::Drop,
    }
  }
}

/// Shared state for all windows in a workspace-switch animation.
///
/// A single elapsed-time driver advances all surrogates in lock-step so
/// every window translates by the same pixel offset on every frame,
/// preserving the illusion that both workspaces move as a single connected
/// panel.
/// How a workspace-switch style moves its surrogates (see
/// `WorkspaceSurrogate::slide_axis`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SwitchMotion {
  /// Travel along the slide axis, in pixels.
  pub(super) distance: i32,
  /// `zoom_factor` of the incoming workspace: how much smaller it starts.
  pub(super) zoom_incoming: f32,
  /// `zoom_factor` of the outgoing workspace: how much smaller it ends,
  /// or larger when negative.
  pub(super) zoom_outgoing: f32,
}

pub(super) struct WorkspaceSwitchState {
  /// All participating windows keyed by window ID.
  pub(super) windows: HashMap<Uuid, WorkspaceSwitchEntry>,
  /// Time of the first rendered frame, lazily set on the first tick.
  ///
  /// Initialized to `None` so the clock starts when `update_internal`
  /// first renders the animation rather than when
  /// `start_workspace_switch` is called mid-`platform_sync`. Without
  /// lazy init, a cold-start gap of 1-3 DWM frames causes surrogates to
  /// jump ahead on their first visible tick.
  pub(super) start_time: Option<Instant>,
  /// Total animation duration.
  pub(super) duration: Duration,
  /// Easing function applied to raw elapsed-time progress.
  pub(super) easing: EasingFunction,
  /// Motion style (slide, fade, or zoom).
  pub(super) style: WorkspaceSwitchStyle,
  /// Slide axis (horizontal or vertical). Only used when `style` is
  /// `Slide`.
  pub(super) slide_direction: WorkspaceSwitchDirection,
  /// Workspace ordering direction: `+1` = target workspace is
  /// higher-index (incoming from the far edge, outgoing to the near
  /// edge). `-1` = opposite.
  pub(super) order_direction: i32,
  /// Left x-coordinate of the animation monitor in screen pixels.
  pub(super) monitor_x: i32,
  /// Width of the animation monitor in screen pixels.
  pub(super) monitor_width: i32,
  /// Top y-coordinate of the animation monitor in screen pixels.
  pub(super) monitor_y: i32,
  /// Height of the animation monitor in screen pixels.
  pub(super) monitor_height: i32,
  /// Viewport the border overlays are pinned to for the slide, i.e. the
  /// monitor's *working area* rather than its full bounds.
  ///
  /// A pinned overlay's `HWND` covers its whole viewport, and it is
  /// banded with its surrogate (`window_class::match_z_band`) -- so
  /// pinning to the full monitor puts a monitor-sized window above any
  /// ordinary-band always-on-top bar for the length of every switch.
  /// Tiled windows live inside the working area anyway, so clipping to
  /// it costs nothing but keeps the overlays out of the reserved strip
  /// entirely. A floating window overhanging that strip has its ring
  /// clipped there mid-slide.
  pub(super) border_viewport: Rect,
  /// Effective horizontal slide travel distance in screen pixels.
  ///
  /// Less than `monitor_width` by the sum of the outgoing workspace's
  /// trailing gap and the incoming workspace's leading gap (both equal to
  /// `outer_gap` in a standard config). This makes the two workspace
  /// panels start adjacent with no visible seam between them.
  slide_distance_h: i32,
  /// Effective vertical slide travel distance in screen pixels.
  ///
  /// Mirrors `slide_distance_h` on the y-axis.
  slide_distance_v: i32,
  /// Whether `start_time` has been re-anchored to a vsync timestamp.
  ///
  /// The clock is provisionally anchored on the wall-clock cold-start
  /// tick, then re-anchored once to the first real vblank so all
  /// vsync-driven frames share a single origin. Stays `false`
  /// (wall-clock only) if no vblank signal ever arrives.
  pub(super) vsync_anchored: bool,
  /// Scale applied to the whole workspace during slide transitions.
  ///
  /// Derived from `WorkspaceSwitchAnimationConfig::zoom_factor`. `0.0`
  /// means no zoom (plain slide). The outgoing workspace scales from
  /// `1.0` to `1.0 - zoom_factor`; the incoming from `1.0 -
  /// zoom_factor` to `1.0`.
  zoom_factor: f32,
}

impl WorkspaceSwitchState {
  /// How this switch's style moves its surrogates.
  pub(super) fn motion(&self) -> SwitchMotion {
    let travel = match self.slide_direction {
      WorkspaceSwitchDirection::Horizontal => self.slide_distance_h,
      WorkspaceSwitchDirection::Vertical => self.slide_distance_v,
    };

    switch_motion(&self.style, travel, self.zoom_factor)
  }
}

/// How `style` moves a switch's surrogates, its slides travelling `travel`
/// and its zooms by `zoom_factor`.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn switch_motion(
  style: &WorkspaceSwitchStyle,
  travel: i32,
  zoom_factor: f32,
) -> SwitchMotion {
  let (distance, zoom_incoming, zoom_outgoing) = match style {
    WorkspaceSwitchStyle::Slide => (travel, zoom_factor, zoom_factor),
    WorkspaceSwitchStyle::Drift => {
      ((travel as f32 * DRIFT_DISTANCE).round() as i32, 0.0, 0.0)
    }
    WorkspaceSwitchStyle::SoftZoom => {
      (0, SOFT_ZOOM_INCOMING, -SOFT_ZOOM_OUTGOING)
    }
    WorkspaceSwitchStyle::Fade
    | WorkspaceSwitchStyle::Zoom
    | WorkspaceSwitchStyle::Cut => (0, 0.0, 0.0),
  };

  SwitchMotion {
    distance,
    zoom_incoming,
    zoom_outgoing,
  }
}

/// Where a switch's incoming windows may be drawn over its outgoing ones
/// (see [`switch_tenants_meet`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Crossing {
  /// Nowhere: each workspace is drawn inside its own panel, and the
  /// panels never overlap.
  Never,
  /// Where an incoming window, moved by this, overlaps an outgoing one at
  /// rest: both sides move alike, this far apart.
  Offset(i32, i32),
  /// Anywhere.
  Anywhere,
}

/// Where `motion`'s incoming windows may meet its outgoing ones, sliding
/// in from `order_direction` along an axis the monitor spans `extent` of.
///
/// Unzoomed, the two sides are a fixed offset apart. Zoomed in to a
/// smaller scale, panels a monitor apart stay apart: each is the working
/// area scaled about the monitor's center.
fn crossing(
  motion: SwitchMotion,
  order_direction: i32,
  is_vertical: bool,
  extent: i32,
) -> Crossing {
  if motion.zoom_incoming == 0.0 && motion.zoom_outgoing == 0.0 {
    let offset = order_direction * motion.distance;
    if is_vertical {
      Crossing::Offset(0, offset)
    } else {
      Crossing::Offset(offset, 0)
    }
  } else if motion.zoom_incoming >= 0.0
    && motion.zoom_outgoing >= 0.0
    && motion.distance >= extent
  {
    Crossing::Never
  } else {
    Crossing::Anywhere
  }
}

/// Where a starting switch's incoming windows may meet its outgoing ones,
/// for a switch from `order_direction` along `axis` on a monitor of
/// `monitor_width` by `monitor_height`, as `start_workspace_switch` sets
/// it up.
pub(crate) fn switch_crossing(
  config: &UserConfig,
  axis: &WorkspaceSwitchDirection,
  order_direction: i32,
  monitor_width: i32,
  monitor_height: i32,
) -> Crossing {
  let ws_config = &config.value.animations.workspace_switch;
  let is_vertical = *axis == WorkspaceSwitchDirection::Vertical;
  let travel = if is_vertical {
    monitor_height.max(1)
  } else {
    monitor_width.max(1)
  };
  let motion = switch_motion(
    &ws_config.style,
    travel,
    ws_config.zoom_factor.clamp(0.0, 1.0),
  );

  crossing(motion, order_direction, is_vertical, travel)
}

/// Whether two windows of a switch, each drawn about `reach` at rest and
/// incoming or not, may overlap at any frame of it.
///
/// Windows of one side move as one, so only overlap where they rest.
fn switch_tenants_meet(
  a: (&Rect, bool),
  b: (&Rect, bool),
  crossing: Crossing,
) -> bool {
  let ((a, a_incoming), (b, b_incoming)) = (a, b);
  if a_incoming == b_incoming {
    return a.intersection(b).is_some();
  }

  let (incoming, outgoing) = if a_incoming { (a, b) } else { (b, a) };
  match crossing {
    Crossing::Never => false,
    Crossing::Anywhere => true,
    Crossing::Offset(dx, dy) => Rect::from_xy(
      incoming.x() + dx,
      incoming.y() + dy,
      incoming.width(),
      incoming.height(),
    )
    .intersection(outgoing)
    .is_some(),
  }
}

/// Registers anew, in z-order, each hosted surrogate of a starting switch
/// drawn under a lower one it may meet (see [`switch_tenants_meet`]), as
/// `PeekView::restack_crossings` does tiles: so each draws whole over the
/// lower ones, as one of its own just below its window does. Their rings
/// were carried after every window's thumbnail.
///
/// Before they are shown, so nothing drawn changes. Free while none may
/// meet, as in most slides.
pub(crate) fn restack_switch_tenants(
  windows: &mut [(Uuid, WorkspaceSwitchEntry)],
  crossing: Crossing,
) {
  let mut tenants = windows
    .iter_mut()
    .filter_map(|(_, entry)| {
      let is_incoming = entry.is_incoming;
      entry
        .surrogate
        .as_mut()
        .filter(|surrogate| surrogate.is_hosted())
        .map(|surrogate| (surrogate.rest_reach(), is_incoming, surrogate))
    })
    .collect::<Vec<_>>();

  let meets = |a: usize, b: usize| match (tenants.get(a), tenants.get(b)) {
    (Some((a, a_in, _)), Some((b, b_in, _))) => {
      switch_tenants_meet((a, *a_in), (b, *b_in), crossing)
    }
    _ => false,
  };
  let count = tenants.len();
  if !(0..count).any(|a| (a + 1..count).any(|b| meets(a, b))) {
    return;
  }

  let _scope = perf::scope(Stage::Restack);
  let ranks = wm_platform::stacking_ranks(
    &tenants
      .iter()
      .map(|(_, _, surrogate)| surrogate.source())
      .collect::<Vec<_>>(),
  );
  let mut bottom_first = (0..count).collect::<Vec<_>>();
  bottom_first.sort_by_key(|index| ranks.get(*index).copied());

  let spans = bottom_first
    .iter()
    .map(|index| tenants.get(*index).and_then(|(_, _, s)| s.stacking()))
    .collect::<Vec<_>>();
  let restacked = crate::animation::peek_view::out_of_order(
    &spans,
    |lower, upper| match (bottom_first.get(lower), bottom_first.get(upper))
    {
      (Some(lower), Some(upper)) => meets(*lower, *upper),
      _ => false,
    },
  );

  for index in restacked {
    let Some((_, _, surrogate)) = bottom_first
      .get(index)
      .and_then(|index| tenants.get_mut(*index))
    else {
      continue;
    };
    if !surrogate.register_anew() {
      tracing::warn!("Failed to stack a switch surrogate over another.");
    }
  }
}

/// Takes a finished workspace switch's teardown one step further (see
/// `WsCleanupStep`), returning its windows once its surrogates are gone.
pub(super) fn advance_ws_cleanup(
  state: &mut WmState,
  config: &UserConfig,
) -> Vec<Uuid> {
  let Some(ws) = state.animation_manager.pending_ws_cleanup.as_mut()
  else {
    return Vec::new();
  };
  let is_awaiting_companions = finish_ws_hand_backs(ws);

  loop {
    let step = state.animation_manager.ws_cleanup_step;
    let changed_screen = match step {
      WsCleanupStep::AwaitUncloak => true,
      WsCleanupStep::ShowOverlays => show_final_overlays(state, config),
      WsCleanupStep::Reanchor => reanchor_final_overlays(state),
      // The surrogates go with their thumbnails, which those waiting for
      // a companion still show.
      WsCleanupStep::Drop if is_awaiting_companions => return Vec::new(),
      WsCleanupStep::Drop => {
        return state
          .animation_manager
          .pending_ws_cleanup
          .take()
          .map_or_default(|ws| ws.windows.into_keys().collect());
      }
    };

    state.animation_manager.ws_cleanup_step = step.next();

    // The next step only has to wait for a change to the screen.
    if changed_screen {
      return Vec::new();
    }
  }
}

/// Hands back the incoming windows of a finished switch whose surrogates
/// waited for their companions, once each is back over its window or the
/// wait timed out (see `WorkspaceSurrogate::awaits_companion`). Returns
/// whether any still waits.
fn finish_ws_hand_backs(ws: &mut WorkspaceSwitchState) -> bool {
  let mut is_awaiting = false;

  for entry in ws
    .windows
    .values_mut()
    .filter(|entry| entry.hand_back == HandBack::AwaitingCompanion)
  {
    let Some(surrogate) = entry.surrogate.as_mut() else {
      entry.hand_back = HandBack::Done;
      continue;
    };

    // Raised since, e.g. by being focused, the window would show over it.
    if surrogate.covers_window() {
      surrogate.cover_window();
    }

    if surrogate.awaits_companion() {
      is_awaiting = true;
    } else {
      surrogate.hide_thumbnail();
      entry.hand_back = HandBack::Done;
    }
  }

  is_awaiting
}

/// The finished switch's incoming windows, each with where it ends and
/// what its overlays go behind: its surrogate, or the window itself for
/// one its surrogate covers.
///
/// Where it ends is read from the real window's own `frame()`
/// (shadow-*excluded* visual bounds -- the same value steady-state overlay
/// sync reads) rather than the surrogate's own `rect`, which is a
/// *positioning* rect (`window.to_rect() + total_border_delta()`,
/// including the OS's invisible drop-shadow expansion) meant for
/// `SetWindowPos`, not overlay tracking. Using `surrogate.rect` made
/// backdrop/border overlays pop visibly larger right at the end of every
/// workspace switch, then snap down once a later `platform_sync` tick
/// re-synced them from `frame()`. Falls back to `surrogate.rect` only if
/// the window/frame lookup fails.
fn final_overlay_rects(state: &WmState) -> Vec<(Uuid, Rect, HWND)> {
  state
    .animation_manager
    .pending_ws_cleanup
    .as_ref()
    .map_or_default(|ws| {
      ws.windows
        .iter()
        .filter(|(_, entry)| entry.is_incoming)
        // Its peek draws its overlays (see `hand_back_to_peek`).
        .filter(|(_, entry)| entry.hand_back != HandBack::ToPeek)
        .filter_map(|(id, entry)| {
          let surrogate = entry.surrogate.as_ref()?;
          let window = state.window_by_id(*id);
          let rect = window
            .as_ref()
            .and_then(|w| w.native().frame().ok())
            .unwrap_or_else(|| surrogate.rect.clone());
          // Behind one above its window would put them over the window.
          let anchor = match &window {
            Some(window) if surrogate.covers_window() => {
              overlay_z_anchor(window)
            }
            _ => surrogate.hwnd(),
          };
          Some((*id, rect, anchor))
        })
        .collect()
    })
}

/// Shows the overlays of the finished switch's incoming windows where they
/// end, still behind their surrogates. Returns whether any is shown.
///
/// `sync_overlays` (called from `platform_sync`) hides each window's
/// static backdrop overlay while its workspace-switch surrogate is alive,
/// including through this teardown. The backdrop overlay is repositioned
/// onto the surrogate's own footprint throughout, so that's fine while
/// it's alive -- but dropping the surrogate destroys it outright, and
/// nothing shows the static overlay again until a later `platform_sync`
/// call runs. Between those two moments DWM can composite a frame with
/// *no* backdrop behind the (semi-transparent) real window at all -- a
/// one-frame flicker at the end of every switch. Shown first, the two
/// overlap for a composited frame instead of leaving a gap.
///
/// Border overlays likewise: a ring parked for the switch is revealed here
/// under its identical copy, which goes with the surrogate.
fn show_final_overlays(state: &mut WmState, config: &UserConfig) -> bool {
  let focused_id = state.focused_container().map(|c| c.id());
  let mut shown = false;
  let mut batch = SurrogateBatch::new();

  for (id, rect, anchor) in final_overlay_rects(state) {
    let is_focused = Some(id) == focused_id;
    let mut params =
      overlay_params_for(state.effect_set(id, is_focused), config);

    if state
      .window_by_id(id)
      .is_some_and(|window| border_hidden_for_fullscreen(&window, config))
    {
      params.border = None;
    }

    shown |= params.backdrop.is_some() || params.border.is_some();

    track_overlays(
      &mut state.window_overlays,
      id,
      params,
      Some(&OverlayPlacement::new(anchor, rect)),
      &mut batch,
    );

    // One the window doesn't end with goes without being revealed.
    if params.border.is_none() {
      if let Some(overlay) = state
        .window_overlays
        .get_mut(&id)
        .and_then(|overlays| overlays.border.as_mut())
        .filter(|overlay| overlay.is_parked())
      {
        overlay.hide();
      }
    }
  }
  batch.commit();

  shown
}

/// Moves the shown overlays of the finished switch's incoming windows
/// behind the windows themselves. Returns whether it had windows to do so
/// for.
///
/// Done before the surrogates are dropped: `WorkspaceSurrogate`'s `Drop`
/// synchronously unregisters its DWM thumbnail, and without re-anchoring
/// first there's a window where the overlay is left anchored to a `HWND`
/// that no longer exists until the follow-up `platform_sync` call
/// eventually corrects it via the normal `sync_overlays` path. DWM can
/// composite a frame in that gap showing the overlay in whatever stale
/// z-slot that leaves it -- a one-frame flicker at the end of every
/// switch, on every incoming window (masked on the focused one in
/// practice, since `sync_focus`'s `SetForegroundWindow` happens to trigger
/// its own z-order correction as a side effect; unfocused siblings get no
/// such incidental save).
fn reanchor_final_overlays(state: &mut WmState) -> bool {
  let windows = final_overlay_rects(state);
  let mut batch = SurrogateBatch::new();

  for (id, rect, _) in &windows {
    let Some(window) = state.window_by_id(*id) else {
      continue;
    };
    let Some(overlays) = state.window_overlays.get_mut(id) else {
      continue;
    };
    // Only shown ones: placing one would show it, e.g. the ring of a
    // window whose border is off now that it is unfocused, for the frames
    // until the follow-up sync hid it again.
    let anchor = overlay_z_anchor(&window);
    if let Some(overlay) = overlays
      .backdrop
      .as_mut()
      .filter(|overlay| overlay.is_visible())
    {
      overlay.defer_rect(&mut batch, rect, anchor);
    }
    if let Some(overlay) = overlays
      .border
      .as_mut()
      .filter(|overlay| overlay.is_visible())
    {
      overlay.defer_rect(&mut batch, rect, anchor);
    }
  }
  batch.commit();

  !windows.is_empty()
}

impl AnimationManager {
  /// Returns `true` if `window_id` has a workspace-switch surrogate
  /// actively tracked this tick, or in its `pending_ws_cleanup` grace
  /// tail.
  ///
  /// Both the backdrop and border overlay for such a window are owned
  /// entirely by the per-tick driver in `update_internal` for as long as
  /// this returns `true` -- `sync_overlays` must leave these windows alone
  /// rather than hiding/repositioning them, even when only one of the two
  /// effects is actually configured for it.
  pub fn has_live_ws_surrogate(&self, window_id: &Uuid) -> bool {
    let has_surrogate = |ws: &WorkspaceSwitchState| {
      ws.windows
        .get(window_id)
        .is_some_and(|e| e.surrogate.is_some())
    };
    self.workspace_switch.as_ref().is_some_and(has_surrogate)
      || self.pending_ws_cleanup.as_ref().is_some_and(has_surrogate)
  }

  /// Returns `true` while a workspace-switch slide animation is in
  /// progress or its surrogates are still live during post-animation
  /// cleanup.
  ///
  /// Includes `pending_ws_cleanup` so that callers (e.g. focus deferral)
  /// stay in their animation-active state until surrogates are fully
  /// dropped, preventing a one-frame flash between animation completion
  /// and surrogate teardown.
  pub fn is_workspace_switch_active(&self) -> bool {
    self.workspace_switch.is_some() || self.pending_ws_cleanup.is_some()
  }

  /// Windows sliding in a playing workspace switch, whether each is
  /// incoming, and where it is drawn this frame, cut to the monitor.
  pub fn switching_rects(&self) -> Vec<(Uuid, bool, Rect)> {
    let Some(switch) = &self.workspace_switch else {
      return Vec::new();
    };

    let monitor = Rect::from_xy(
      switch.monitor_x,
      switch.monitor_y,
      switch.monitor_width,
      switch.monitor_height,
    );

    switch
      .windows
      .iter()
      .filter_map(|(id, entry)| {
        let rect = entry
          .surrogate
          .as_ref()?
          .unclipped_rect()?
          .intersection(&monitor)?;
        Some((*id, entry.is_incoming, rect))
      })
      .collect()
  }

  /// Whether `window_id` takes part in a workspace-switch slide, playing
  /// or waiting on its surrogates' teardown.
  pub fn is_in_workspace_switch(&self, window_id: &Uuid) -> bool {
    [&self.workspace_switch, &self.pending_ws_cleanup]
      .into_iter()
      .flatten()
      .any(|ws| ws.windows.contains_key(window_id))
  }

  /// Hands a finished or interrupted slide's surrogates to
  /// `pending_ws_cleanup`, merging with any still waiting there so that
  /// none are destroyed before their real window is uncloaked.
  pub(super) fn stash_for_ws_cleanup(
    &mut self,
    mut ws: WorkspaceSwitchState,
  ) {
    // A last look for companions shown since, before the hand-back and
    // the z-order it keeps for one (see `hands_back_to_companion`).
    for surrogate in ws
      .windows
      .values_mut()
      .filter(|entry| {
        entry.is_incoming && entry.hand_back == HandBack::Pending
      })
      .filter_map(|entry| entry.surrogate.as_mut())
    {
      surrogate.adopt_companion();
    }

    match &mut self.pending_ws_cleanup {
      Some(pending) => pending.windows.extend(ws.windows),
      None => self.pending_ws_cleanup = Some(ws),
    }

    // Its windows are uncloaked on this tick or the next, which the
    // teardown then has to wait for from the start.
    self.ws_cleanup_step = WsCleanupStep::default();
  }

  /// Returns `true` while `window_id` is an incoming participant in the
  /// active workspace-switch animation.
  ///
  /// Unlike the `pending_sync` incoming flag (cleared after the first
  /// `platform_sync`), this stays `true` for the full animation duration
  /// so that focus events during the animation do not prematurely
  /// uncloak the real window before the surrogate finishes sliding in.
  pub fn is_workspace_switch_incoming(&self, window_id: &Uuid) -> bool {
    self
      .workspace_switch
      .as_ref()
      .and_then(|ws| ws.windows.get(window_id))
      .is_some_and(|e| e.is_incoming)
  }

  /// Returns `true` when `window_id` is an incoming participant held in
  /// `pending_ws_cleanup` (the cleanup state after the animation
  /// completes) that is yet to be handed back. Used to force synchronous
  /// `SetWindowPos` before uncloaking so the window is already at its
  /// target position when revealed.
  pub fn is_pending_ws_cleanup_incoming(&self, window_id: &Uuid) -> bool {
    self
      .pending_ws_cleanup
      .as_ref()
      .and_then(|ws| ws.windows.get(window_id))
      .is_some_and(|e| e.is_incoming && e.hand_back == HandBack::Pending)
  }

  /// Whether `window_id` is about to be handed back from a surrogate
  /// drawing its companion, which only comes back over it some time after
  /// its uncloak: as an incoming window of a finished switch (see
  /// `prepare_ws_hand_back`), or from a `CompanionCover`.
  pub fn hands_back_to_companion(&self, window_id: &Uuid) -> bool {
    if self.companion_covers.contains_key(window_id) {
      return true;
    }

    self.is_pending_ws_cleanup_incoming(window_id)
      && self
        .pending_ws_cleanup
        .as_ref()
        .and_then(|ws| ws.windows.get(window_id))
        .and_then(|entry| entry.surrogate.as_ref())
        .is_some_and(WorkspaceSurrogate::shows_companion)
  }

  /// Puts `window_id`'s `CompanionCover` back over the window while it
  /// waits for the window's companion.
  pub fn cover_companion_hand_back(&self, window_id: &Uuid) {
    if let Some(cover) = self.companion_covers.get(window_id) {
      cover.cover_window();
    }
  }

  /// Installs a workspace-switch animation for the provided windows.
  ///
  /// Accepts pre-created [`WorkspaceSurrogate`] instances together with
  /// their incoming/outgoing flags. A shared driver advances all
  /// surrogates in lock-step so the entire workspace moves as one panel.
  /// A previous switch still playing is finished, not dropped.
  ///
  /// `monitor_handle` is the `HMONITOR` of the animation monitor, used to
  /// look up the `IDXGIOutput` for per-monitor vsync waiting.
  /// on secondary monitors whose refresh rate differs from the primary.
  #[allow(clippy::too_many_arguments)]
  pub fn start_workspace_switch(
    &mut self,
    windows: Vec<(Uuid, WorkspaceSwitchEntry)>,
    order_direction: i32,
    slide_direction: WorkspaceSwitchDirection,
    monitor_x: i32,
    monitor_width: i32,
    monitor_y: i32,
    monitor_height: i32,
    monitor_handle: isize,
    monitor_work_area: &Rect,
    config: &UserConfig,
  ) {
    // A slide still playing (typically on another monitor) is finished
    // rather than dropped: its incoming windows are only uncloaked when it
    // completes, so dropping it left them invisible until something else
    // redrew them. Windows taking part in the new slide are left to it.
    let is_in_new_slide =
      |id: &Uuid| windows.iter().any(|(new_id, _)| new_id == id);

    if let Some(mut previous) = self.workspace_switch.take() {
      previous.windows.retain(|id, _| !is_in_new_slide(id));
      self.interrupted_ws_windows.extend(previous.windows.keys());
      self.stash_for_ws_cleanup(previous);
    }

    // Likewise for a finished one still being torn down, whose surrogates
    // would go on drawing what is left of them (a ring copy) where the
    // window ended.
    if let Some(pending) = &mut self.pending_ws_cleanup {
      pending.windows.retain(|id, _| !is_in_new_slide(id));
      if pending.windows.is_empty() {
        self.pending_ws_cleanup = None;
      }
    }

    let ws_config = &config.value.animations.workspace_switch;

    let duration_ms = ws_config.effective_duration_ms();

    // Slide each workspace the full monitor dimension. The outgoing
    // workspace exits the screen completely (no residual sliver at the
    // trailing edge), and the incoming workspace starts one full
    // monitor away. The two workspaces keep their normal outer-gap
    // spacing during the slide rather than being pulled together by a
    // seam-gap reduction.
    let slide_distance_h = monitor_width.max(1);
    let slide_distance_v = monitor_height.max(1);

    let ws_windows: HashMap<Uuid, WorkspaceSwitchEntry> =
      windows.into_iter().collect();

    if ws_windows.is_empty() {
      tracing::warn!("Workspace-switch skipped: no windows to animate.");
    } else {
      tracing::info!(
        "Starting workspace-switch animation: style={:?}, direction={:?}, \
         order={}, monitor=({monitor_x},{monitor_y},{monitor_width}x{monitor_height}), \
         slide_distance=({slide_distance_h}x{slide_distance_v}), \
         windows={}",
        ws_config.style,
        slide_direction,
        order_direction,
        ws_windows.len(),
      );
      // So the timer thread wakes right after each of this monitor's
      // vblanks, leaving a full frame period for surrogate updates before
      // the next DWM composition.
      self.install_waiter(monitor_handle);
      self.workspace_switch = Some(WorkspaceSwitchState {
        windows: ws_windows,
        start_time: None,
        duration: Duration::from_millis(u64::from(duration_ms)),
        easing: ws_config.easing.clone(),
        style: ws_config.style.clone(),
        slide_direction,
        order_direction,
        monitor_x,
        monitor_width,
        monitor_y,
        monitor_height,
        border_viewport: monitor_work_area.clone(),
        slide_distance_h,
        slide_distance_v,
        zoom_factor: ws_config.zoom_factor.clamp(0.0, 1.0),
        vsync_anchored: false,
      });
    }
  }

  /// Hides the workspace-switch surrogate thumbnail for a single window in
  /// `pending_ws_cleanup`.
  ///
  /// Called immediately after `set_cloaked(false)` for each incoming
  /// window so the surrogate thumbnail disappears at the same DWM
  /// composition event as the window uncloak, eliminating the
  /// double-blend frame that would occur if thumbnail hide were deferred
  /// until after all windows are processed.
  pub fn hide_pending_ws_cleanup_surrogate(&mut self, window_id: Uuid) {
    let Some(ref mut ws) = self.pending_ws_cleanup else {
      return;
    };
    if let Some(entry) = ws.windows.get_mut(&window_id) {
      entry.hand_back = HandBack::Done;
      if let Some(ref mut s) = entry.surrogate {
        // Its window, still hidden by its alpha, would show uncolored
        // until its companion is back over it, so the hide waits
        // for that (see `finish_ws_hand_backs`).
        if s.awaits_companion() {
          entry.hand_back = HandBack::AwaitingCompanion;
        } else {
          s.hide_thumbnail();
        }
      }
    }
  }

  /// Whether `window_id` slides in with a workspace switch, playing or
  /// finished, whose surrogate has yet to hand it back.
  pub fn awaits_peek_hand_back(&self, window_id: &Uuid) -> bool {
    [&self.workspace_switch, &self.pending_ws_cleanup]
      .into_iter()
      .flatten()
      .filter_map(|ws| ws.windows.get(window_id))
      .any(|entry| {
        entry.is_incoming
          && entry.surrogate.is_some()
          && entry.hand_back == HandBack::Pending
      })
  }

  /// Hands incoming `window_id` of a finished switch, which the layout
  /// keeps hidden, over to its resting peek, drawn again just before:
  /// its surrogate's thumbnail and backdrop copy go, its ring copy stays
  /// over the peek's until the surrogate is dropped (`WsCleanupStep`).
  pub fn hand_back_to_peek(&mut self, window_id: Uuid) {
    let Some(entry) = self
      .pending_ws_cleanup
      .as_mut()
      .and_then(|ws| ws.windows.get_mut(&window_id))
      .filter(|entry| {
        entry.is_incoming && entry.hand_back == HandBack::Pending
      })
    else {
      return;
    };

    entry.hand_back = HandBack::ToPeek;
    if let Some(surrogate) = entry.surrogate.as_mut() {
      surrogate.hide_thumbnail();
      surrogate.hide_backdrop();
    }
  }

  /// Hands outgoing `window_id`, which a resting peek drew, over to its
  /// workspace-switch surrogate, before the peek goes. Its real window is
  /// cloaked already, so it needn't be hidden by its alpha.
  pub fn take_over_from_peek(&mut self, window_id: Uuid) {
    if let Some(surrogate) = self
      .workspace_switch
      .as_mut()
      .and_then(|ws| ws.windows.get_mut(&window_id))
      .filter(|entry| !entry.is_incoming)
      .and_then(|entry| entry.surrogate.as_mut())
    {
      surrogate.apply_effect_opacity();
    }
    self.peeks.drop_switched_out(&window_id);
  }

  /// Hands outgoing `window` over to its workspace-switch surrogate, just
  /// before the window is cloaked (see `WorkspaceSurrogate::take_over`).
  pub fn take_over_outgoing(
    &mut self,
    window_id: Uuid,
    window: &NativeWindow,
  ) {
    let surrogate = self
      .workspace_switch
      .as_mut()
      .and_then(|ws| ws.windows.get_mut(&window_id))
      .filter(|entry| !entry.is_incoming)
      .and_then(|entry| entry.surrogate.as_mut());

    if let Some(surrogate) = surrogate {
      surrogate.take_over(window);
    }
  }

  /// Hands the windows of `windows`, a slide about to cut short the one
  /// still playing, over from their surrogates in that one, returning
  /// the ids of those handed over, each with whether it slides in.
  ///
  /// Such a window has no real window on screen to take it over from
  /// (see `WorkspaceSurrogate::take_over`), and its old surrogate goes as
  /// the new slide starts. So, once the new surrogates have been
  /// composited at least once, an outgoing one is brought up and the old
  /// one's thumbnail hidden back to back, and no frame shows neither. One
  /// sliding in starts off-screen and is only shown on the new slide's
  /// first frame, so its old surrogate is just hidden.
  ///
  /// So are the windows of a finished slide still being torn down. One
  /// handed back is on screen again and taken over from as usual, so of
  /// its old surrogate only the ring and backdrop copies need covering
  /// for: the teardown may not have shown the real ones again yet, and
  /// the old surrogate goes before the take-over.
  pub fn hand_over_cut_short_slide(
    &mut self,
    windows: &mut [(Uuid, WorkspaceSwitchEntry)],
  ) -> Vec<(Uuid, bool)> {
    let mut handed_over = Vec::new();
    for (id, entry) in windows {
      let Some(new) = entry.surrogate.as_mut() else {
        continue;
      };
      let Some((old, handed_back)) = [
        self.workspace_switch.as_mut(),
        self.pending_ws_cleanup.as_mut(),
      ]
      .into_iter()
      .flatten()
      .find_map(|previous| previous.windows.get_mut(id))
      .and_then(|previous| {
        let handed_back = previous.hand_back != HandBack::Pending;
        previous.surrogate.as_mut().map(|old| (old, handed_back))
      }) else {
        continue;
      };

      match (entry.is_incoming, handed_back) {
        (true, _) => {}
        (false, true) => {
          new.cover_ring();
          new.cover_backdrop();
        }
        (false, false) => new.apply_effect_opacity(),
      }
      old.hide_thumbnail();
      old.hide_ring();
      old.hide_backdrop();
      handed_over.push((*id, entry.is_incoming));
    }

    handed_over
  }

  /// Readies handing incoming `window` back from its finished
  /// workspace-switch surrogate, just before the window is uncloaked (see
  /// `WorkspaceSurrogate::prepare_hand_back`).
  pub fn prepare_ws_hand_back(
    &mut self,
    window_id: Uuid,
    window: &NativeWindow,
  ) {
    let surrogate = self
      .pending_ws_cleanup
      .as_mut()
      .and_then(|ws| ws.windows.get_mut(&window_id))
      .filter(|entry| entry.is_incoming)
      .and_then(|entry| entry.surrogate.as_mut());

    if let Some(surrogate) = surrogate {
      surrogate.prepare_hand_back(window);
    }
  }

  /// Applies the configured effect opacity to all outgoing
  /// workspace-switch surrogates, for any whose window wasn't handed over
  /// as it was cloaked (see `take_over_outgoing`).
  pub fn apply_outgoing_surrogate_opacities(&mut self) {
    let Some(ref mut ws) = self.workspace_switch else {
      return;
    };
    for entry in ws.windows.values_mut() {
      if !entry.is_incoming {
        if let Some(ref mut s) = entry.surrogate {
          s.apply_effect_opacity();
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use tokio::sync::mpsc;
  use wm_common::ParsedConfig;

  use super::*;

  fn motion(
    distance: i32,
    zoom_incoming: f32,
    zoom_outgoing: f32,
  ) -> SwitchMotion {
    SwitchMotion {
      distance,
      zoom_incoming,
      zoom_outgoing,
    }
  }

  /// Unzoomed sides are a fixed offset apart; zoomed down a monitor
  /// apart, they never meet; zoomed up, anywhere.
  #[test]
  fn switch_sides_cross_as_their_motion_moves_them() {
    assert_eq!(
      crossing(motion(1920, 0.0, 0.0), -1, false, 1920),
      Crossing::Offset(-1920, 0)
    );
    assert_eq!(
      crossing(motion(96, 0.0, 0.0), 1, true, 1080),
      Crossing::Offset(0, 96)
    );
    assert_eq!(
      crossing(motion(1920, 0.1, 0.1), 1, false, 1920),
      Crossing::Never
    );
    assert_eq!(
      crossing(motion(0, 0.05, -0.05), 1, false, 1920),
      Crossing::Anywhere
    );
  }

  /// One side's windows meet only where they rest; the other side's
  /// where the crossing puts them.
  #[test]
  fn switch_windows_meet_where_their_sides_overlap() {
    let left = Rect::from_xy(0, 0, 900, 1000);
    let right = Rect::from_xy(1000, 0, 900, 1000);
    let offset = Crossing::Offset(1000, 0);

    assert!(!switch_tenants_meet((&left, true), (&right, true), offset));
    assert!(switch_tenants_meet((&left, true), (&left, true), offset));
    assert!(switch_tenants_meet((&left, true), (&right, false), offset));
    assert!(!switch_tenants_meet((&left, false), (&right, true), offset));
    assert!(!switch_tenants_meet(
      (&left, true),
      (&left, false),
      Crossing::Never
    ));
    assert!(switch_tenants_meet(
      (&left, true),
      (&right, false),
      Crossing::Anywhere
    ));
  }

  fn start_slide(
    manager: &mut AnimationManager,
    ids: &[Uuid],
    config: &UserConfig,
  ) {
    let windows = ids
      .iter()
      .map(|id| {
        let entry = WorkspaceSwitchEntry {
          surrogate: None,
          is_incoming: true,
          was_focused: false,
          hand_back: HandBack::Pending,
        };
        (*id, entry)
      })
      .collect();

    manager.start_workspace_switch(
      windows,
      1,
      WorkspaceSwitchDirection::default(),
      0,
      1920,
      0,
      1080,
      0,
      &Rect::from_xy(0, 0, 1920, 1080),
      config,
    );
  }

  /// A drift moves a short way along the slide axis without scaling, and
  /// a soft zoom scales without moving, the outgoing side growing.
  #[test]
  fn drift_and_soft_zoom_motion() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let mut manager = AnimationManager::new(tick_tx);
    let id = Uuid::new_v4();

    for (style, expected) in [
      (
        WorkspaceSwitchStyle::Drift,
        SwitchMotion {
          distance: 96,
          zoom_incoming: 0.0,
          zoom_outgoing: 0.0,
        },
      ),
      (
        WorkspaceSwitchStyle::SoftZoom,
        SwitchMotion {
          distance: 0,
          zoom_incoming: SOFT_ZOOM_INCOMING,
          zoom_outgoing: -SOFT_ZOOM_OUTGOING,
        },
      ),
    ] {
      let mut parsed = ParsedConfig::default();
      parsed.animations.workspace_switch.style = style;
      start_slide(&mut manager, &[id], &UserConfig::from_parsed(parsed));

      let motion = manager
        .workspace_switch
        .as_ref()
        .expect("switch should start")
        .motion();
      assert_eq!(motion, expected);
    }
  }

  #[test]
  fn new_slide_finishes_interrupted_one() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let mut manager = AnimationManager::new(tick_tx);
    let config = UserConfig::from_parsed(ParsedConfig::default());
    let [a, b, c] = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    start_slide(&mut manager, &[a, b], &config);
    start_slide(&mut manager, &[b, c], &config);

    // `a` only took part in the first slide, so it is finished; `b` moved
    // on to the new one.
    assert_eq!(manager.interrupted_ws_windows, vec![a]);

    let cleanup = manager
      .pending_ws_cleanup
      .as_ref()
      .expect("interrupted slide should await cleanup");
    assert!(cleanup.windows.contains_key(&a));
    assert!(!cleanup.windows.contains_key(&b));

    let active = manager
      .workspace_switch
      .as_ref()
      .expect("new slide should be active");
    assert_eq!(active.windows.len(), 2);
    assert!(active.windows.contains_key(&b));
    assert!(active.windows.contains_key(&c));
  }

  #[test]
  fn interrupted_slides_merge_into_cleanup() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let mut manager = AnimationManager::new(tick_tx);
    let config = UserConfig::from_parsed(ParsedConfig::default());
    let [a, b, c] = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    start_slide(&mut manager, &[a], &config);
    start_slide(&mut manager, &[b], &config);
    start_slide(&mut manager, &[c], &config);

    let cleanup = manager
      .pending_ws_cleanup
      .as_ref()
      .expect("interrupted slides should await cleanup");
    assert!(cleanup.windows.contains_key(&a));
    assert!(cleanup.windows.contains_key(&b));
  }

  #[test]
  fn windows_added_to_cleanup_restart_its_teardown() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let mut manager = AnimationManager::new(tick_tx);
    let config = UserConfig::from_parsed(ParsedConfig::default());
    let [a, b] = [Uuid::new_v4(), Uuid::new_v4()];

    start_slide(&mut manager, &[a], &config);
    manager.ws_cleanup_step = WsCleanupStep::Drop;
    start_slide(&mut manager, &[b], &config);

    // `a`'s window is only uncloaked on the next tick.
    assert_eq!(manager.ws_cleanup_step, WsCleanupStep::AwaitUncloak);
  }

  #[test]
  fn new_slide_takes_over_windows_awaiting_cleanup() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let mut manager = AnimationManager::new(tick_tx);
    let config = UserConfig::from_parsed(ParsedConfig::default());
    let [a, b, c] = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    start_slide(&mut manager, &[a, b], &config);
    start_slide(&mut manager, &[c], &config);
    start_slide(&mut manager, &[a], &config);

    let cleanup = manager
      .pending_ws_cleanup
      .as_ref()
      .expect("windows left out of the new slide should await cleanup");
    assert!(!cleanup.windows.contains_key(&a));
    assert!(cleanup.windows.contains_key(&b));
    assert!(cleanup.windows.contains_key(&c));
  }

  #[test]
  fn cleanup_ends_once_a_new_slide_takes_all_its_windows() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let mut manager = AnimationManager::new(tick_tx);
    let config = UserConfig::from_parsed(ParsedConfig::default());
    let [a, b] = [Uuid::new_v4(), Uuid::new_v4()];

    start_slide(&mut manager, &[a], &config);
    start_slide(&mut manager, &[b], &config);
    start_slide(&mut manager, &[a, b], &config);

    assert!(manager.pending_ws_cleanup.is_none());
  }
}
