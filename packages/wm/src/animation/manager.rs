use std::{
  cell::Cell,
  collections::{HashMap, HashSet},
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
  },
  time::{Duration, Instant},
};

/// Fraction of the monitor's vblank period to lead the animation clock by.
///
/// After `IDXGIOutput::WaitForVBlank` returns, the surrogate update
/// written this tick is not composited by DWM until the *next* vblank — up
/// to one full frame period later. Advancing the animation clock by a
/// fraction of that period makes the computed position align with where it
/// will be when DWM actually presents it, rather than systematically
/// lagging behind.
///
/// Expressed as a fraction of the per-monitor frame period read live from
/// the installed `DxgiVsyncWaiter`, so the compensation scales correctly
/// across monitors with different refresh rates (e.g. 60 Hz + 175 Hz).
/// `0.5` leads by half a frame: a balance between under-compensating
/// (motion trails the cursor) and over-compensating (motion arrives
/// early). Tune here if the slide feels laggy (increase toward `1.0`) or
/// rushed (decrease toward `0.0`).
const VSYNC_LEAD_FRACTION: f32 = 0.5;

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
const WS_COMPLETE_THRESHOLD_PX: f32 = 1.5;

/// How far a drift moves each workspace, as a fraction of the monitor's
/// extent along the slide axis.
const DRIFT_DISTANCE: f32 = 0.05;

/// How much smaller the incoming workspace starts in a soft zoom.
const SOFT_ZOOM_INCOMING: f32 = 0.05;

/// How much larger the outgoing workspace ends in a soft zoom.
const SOFT_ZOOM_OUTGOING: f32 = 0.03;

/// Grace period held at the start of a window-open animation before the
/// surrogate begins revealing the window.
///
/// A freshly created window has often not painted its first frame when the
/// open animation starts, so its DWM thumbnail is momentarily blank/black
/// — producing a black box that slides in and "pops" to real content at
/// the end. Holding the animation at progress `0.0` for this period (the
/// surrogate stays off-screen for slide/zoom and fully transparent for
/// fade) gives the app time to paint, so the slide reveals real content
/// from the first visible frame. Implemented via
/// `WindowAnimationState::start_delay`, which is measured from the first
/// rendered frame, so the app gets this long *after* the first animation
/// tick to paint. Roughly two frames at 60 Hz — long enough to cover the
/// typical first-paint latency without a perceptible delay in the window
/// appearing.
pub(super) const OPEN_PAINT_GRACE: Duration = Duration::from_millis(30);

/// How long a sampled surrogate backdrop color stays valid per window.
///
/// Sampling costs two GPU→CPU `BitBlt` readbacks; now always done on a
/// background thread via `sample_edge_color_async` rather than blocking
/// the WM's main thread, but a miss still means the *current* motion
/// plays with a transparent backdrop instead of a matched color. App
/// background colors change rarely (theme switches), so a stale hit is a
/// cosmetic-only risk -- kept generous so a window resized only
/// occasionally still benefits from the earlier background sample instead
/// of re-paying for one every time.
const EDGE_COLOR_CACHE_TTL: Duration = Duration::from_secs(300);

/// Cache-size threshold above which stale edge-color entries are pruned.
const EDGE_COLOR_CACHE_PRUNE_LEN: usize = 128;

/// How long a window's overlays keep being re-asserted behind it after its
/// own z-order last changed, or after its surrogate went away. See
/// `AnimationManager::settle_overlay_z_order`.
///
/// Covers the asynchronous move itself, `set_z_order`'s 10ms retry, and
/// apps that briefly re-assert their own z-order in response; measured
/// cases land within a few frames.
const OVERLAY_Z_SETTLE: Duration = Duration::from_millis(200);

/// Upper bound on how long a window's overlays keep settling, however
/// often the settle is extended -- so a window that never reaches its
/// intended band (an app that refuses `HWND_TOPMOST`) cannot keep the
/// animation timer ticking forever.
const OVERLAY_Z_SETTLE_MAX: Duration = Duration::from_secs(2);

/// Deadlines for one window in `AnimationManager::overlay_z_settle`.
#[derive(Clone, Copy, Debug)]
struct SettleWindow {
  /// When re-checking stops, unless extended.
  until: Instant,
  /// When re-checking stops regardless; see [`OVERLAY_Z_SETTLE_MAX`].
  cap: Instant,
}

/// A synchronous move slower than this demotes its window to asynchronous
/// moves. Several frames' worth: the whole WM waits on it.
const SLOW_SYNC_DEMOTE_THRESHOLD: Duration = Duration::from_millis(50);

/// How long a window demoted by [`SLOW_SYNC_DEMOTE_THRESHOLD`] is moved
/// asynchronously.
const SYNC_DEMOTE_DURATION: Duration = Duration::from_secs(30);

/// How long animations may go without a frame before the watchdog runs
/// one itself and replaces the timer thread.
const FRAME_STALL_TIMEOUT: Duration = Duration::from_millis(250);

/// Replaced timer threads that may be left blocked at once. See
/// [`AnimationManager::replace_stalled_timer`].
const MAX_STALLED_TIMERS: usize = 3;

/// The animation timer thread, with the flag that retires it.
struct TimerThread {
  handle: std::thread::JoinHandle<()>,

  /// Makes the thread exit once its current wait returns. Set when the
  /// watchdog replaces a thread stuck in a vsync wait.
  retired: Arc<AtomicBool>,
}

/// Counts a retired timer thread out of `stalled_timers` once it exits.
struct StalledTimerGuard {
  retired: Arc<AtomicBool>,
  stalled_timers: Arc<AtomicUsize>,
}

impl Drop for StalledTimerGuard {
  fn drop(&mut self) {
    if self.retired.load(Ordering::Relaxed) {
      self.stalled_timers.fetch_sub(1, Ordering::Relaxed);
    }
  }
}

/// Stores `next` in `result` if it is the first error; later errors are
/// only logged.
fn keep_first_error(
  result: &mut anyhow::Result<()>,
  next: anyhow::Result<()>,
) {
  if let Err(err) = next {
    if result.is_ok() {
      *result = Err(err);
    } else {
      tracing::error!("{err:?}");
    }
  }
}

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{
  EasingFunction, EffectSet, ResizeStyle, WindowTransitionParams,
  WorkspaceSwitchDirection, WorkspaceSwitchStyle,
};
use wm_platform::{
  lerp_f32,
  perf::{self, Stage},
  sample_edge_color_async, Color, CompanionCover, DxgiVsyncWaiter,
  EdgeColorCache, FrameClock, NativeBorderOverlay, NativeWindow, Overlay,
  Rect, StyleBlend, SurrogateBatch, WorkspaceSurrogate, HWND,
};

use crate::{
  animation::{
    focus_fade::{FocusFades, FocusMix},
    transit::Direction,
  },
  commands::general::{
    advance_focus_fades, apply_focus_effects,
    border_hidden_for_fullscreen, overlay_params_for, overlay_z_anchor,
    platform_sync, resync_settling_overlays, surrogate_effects_for,
    track_overlays, upsert_pinned_border_overlay, SyncableOverlay,
  },
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  window_overlays::OverlayPlacement,
  wm_state::WmState,
};

/// The instants an animation tick's frame is evaluated at, read once per
/// frame.
#[derive(Clone, Copy, Debug)]
struct FrameTimes {
  /// See [`AnimationManager::frame_now`].
  now: Instant,
  /// One frame period before `now`, where clocks started this frame
  /// begin, so its first frame already shows motion.
  clock_origin: Instant,
}

/// Why a window is taken off the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AwayTransition {
  Close,
  Minimize,
}

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
enum WsCleanupStep {
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
struct SwitchMotion {
  /// Travel along the slide axis, in pixels.
  distance: i32,
  /// `zoom_factor` of the incoming workspace: how much smaller it starts.
  zoom_incoming: f32,
  /// `zoom_factor` of the outgoing workspace: how much smaller it ends,
  /// or larger when negative.
  zoom_outgoing: f32,
}

struct WorkspaceSwitchState {
  /// All participating windows keyed by window ID.
  windows: HashMap<Uuid, WorkspaceSwitchEntry>,
  /// Time of the first rendered frame, lazily set on the first tick.
  ///
  /// Initialized to `None` so the clock starts when `update_internal`
  /// first renders the animation rather than when
  /// `start_workspace_switch` is called mid-`platform_sync`. Without
  /// lazy init, a cold-start gap of 1-3 DWM frames causes surrogates to
  /// jump ahead on their first visible tick.
  start_time: Option<Instant>,
  /// Total animation duration.
  duration: Duration,
  /// Easing function applied to raw elapsed-time progress.
  easing: EasingFunction,
  /// Motion style (slide, fade, or zoom).
  style: WorkspaceSwitchStyle,
  /// Slide axis (horizontal or vertical). Only used when `style` is
  /// `Slide`.
  slide_direction: WorkspaceSwitchDirection,
  /// Workspace ordering direction: `+1` = target workspace is
  /// higher-index (incoming from the far edge, outgoing to the near
  /// edge). `-1` = opposite.
  order_direction: i32,
  /// Left x-coordinate of the animation monitor in screen pixels.
  monitor_x: i32,
  /// Width of the animation monitor in screen pixels.
  monitor_width: i32,
  /// Top y-coordinate of the animation monitor in screen pixels.
  monitor_y: i32,
  /// Height of the animation monitor in screen pixels.
  monitor_height: i32,
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
  border_viewport: Rect,
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
  vsync_anchored: bool,
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
  fn motion(&self) -> SwitchMotion {
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
  let restacked =
    super::peek_view::out_of_order(&spans, |lower, upper| {
      match (bottom_first.get(lower), bottom_first.get(upper)) {
        (Some(lower), Some(upper)) => meets(*lower, *upper),
        _ => false,
      }
    });

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

/// What `platform_sync`'s redraw does with the real app window's position
/// this frame.
pub enum AnimationPositionResult {
  /// Apply this rect to the real window via `reposition_window`.
  ///
  /// The carried `Rect` is the current animated position, available for
  /// callers that bypass the surrogate path.
  Apply(Rect),
  /// The surrogate overlay is handling all visuals; skip repositioning
  /// the real window this frame.
  Frozen,
}

/// Manages animations for all windows.
pub struct AnimationManager {
  /// Sender for animation tick events.
  animation_tick_tx: mpsc::UnboundedSender<()>,
  /// Whether the animation timer thread is currently ticking (vs.
  /// parked).
  ///
  /// Acts as the gate for the persistent timer thread: set `true` to
  /// start a ticking phase, `false` to send it back to parking.
  animation_timer_running: Arc<AtomicBool>,
  /// Handle to the persistent animation timer thread.
  ///
  /// Spawned lazily on the first animation and kept for the process
  /// lifetime, parking between animations rather than being re-spawned
  /// each time. This removes the per-animation cost of thread creation
  /// + priority/MMCSS setup from the input-to-first-frame latency path.
  timer_thread: Mutex<Option<TimerThread>>,
  /// Signals the persistent timer thread to exit. Set on drop.
  timer_shutdown: Arc<AtomicBool>,
  /// When the last frame ran, or ticking last started. Read by the
  /// watchdog; see [`watchdog_deadline`].
  ///
  /// [`watchdog_deadline`]: AnimationManager::watchdog_deadline
  last_frame_at: Cell<Instant>,
  /// Timer threads replaced by the watchdog that are still blocked in
  /// their wait. See [`replace_stalled_timer`].
  ///
  /// [`replace_stalled_timer`]: AnimationManager::replace_stalled_timer
  stalled_timers: Arc<AtomicUsize>,
  /// The timestamp every window is evaluated at in the current redraw
  /// pass; see [`frame_now`]. Cleared by [`begin_redraw_pass`].
  ///
  /// [`frame_now`]: AnimationManager::frame_now
  /// [`begin_redraw_pass`]: AnimationManager::begin_redraw_pass
  pass_now: Option<Instant>,
  /// Windows whose own z-order was changed recently, mapped to when their
  /// overlays stop being re-checked. See [`settle_overlay_z_order`].
  ///
  /// [`settle_overlay_z_order`]: AnimationManager::settle_overlay_z_order
  overlay_z_settle: HashMap<Uuid, SettleWindow>,
  /// DXGI vsync waiter for the animation monitor.
  ///
  /// When `Some`, the timer thread calls `WaitForVBlank` on this output
  /// instead of `DwmFlush`. This gives a full frame period after each
  /// vsync to update surrogates, regardless of which monitor is the
  /// Windows primary. Cleared on workspace-switch completion.
  animation_timer_vsync: Arc<Mutex<Option<DxgiVsyncWaiter>>>,
  /// When the pacing monitor's vblanks happen, as the timer thread wakes
  /// for them. Frames are evaluated at its time plus a fraction of its
  /// period (see [`VSYNC_LEAD_FRACTION`]) rather than at
  /// `Instant::now()`, which varies with how late the thread woke and
  /// how long the WM thread took to get to the tick.
  frame_clock: Arc<Mutex<FrameClock>>,
  /// Recently sampled backdrop fill colors keyed by window handle, kept
  /// for [`EDGE_COLOR_CACHE_TTL`]. Shared
  /// (rather than owned outright) because `sample_edge_color_async`'s
  /// background thread inserts into this same map directly once a
  /// sample completes, instead of routing the result back through the
  /// event loop.
  edge_color_cache: EdgeColorCache,
  /// Windows whose edge color is re-sampled once animations go idle.
  pending_edge_samples: HashSet<isize>,
  /// Active workspace-switch slide animation, or `None` when idle.
  workspace_switch: Option<WorkspaceSwitchState>,
  /// Peeks drawing scrolling layouts' columns.
  pub peeks: super::peek_view::PeekView,
  /// Workspace-switch state that just completed; kept alive until the
  /// incoming real windows are uncloaked and their overlays settled under
  /// its surrogates, over the ticks of `ws_cleanup_step`.
  pending_ws_cleanup: Option<WorkspaceSwitchState>,
  /// What tearing down `pending_ws_cleanup` does on the coming tick.
  ws_cleanup_step: WsCleanupStep,
  /// Windows of a slide cut short by a newer one, to be redrawn on the
  /// next frame so they are uncloaked at their final rect. See
  /// [`start_workspace_switch`].
  ///
  /// [`start_workspace_switch`]: AnimationManager::start_workspace_switch
  interrupted_ws_windows: Vec<Uuid>,
  /// Windows moved asynchronously until the stored time, keyed by
  /// `HWND`, because a synchronous move of them was slow. See
  /// [`allows_sync_reposition`].
  ///
  /// [`allows_sync_reposition`]: AnimationManager::allows_sync_reposition
  sync_demoted_until: HashMap<isize, Instant>,
  /// Windows whose effects fade between focus states. Drawn by
  /// `advance_focus_fades` while at rest, and by the scroll view while it
  /// draws them.
  focus_fades: FocusFades,
  /// Covers of windows uncloaked without an animation, until their
  /// companions are back over them (see `CompanionCover`).
  companion_covers: HashMap<Uuid, CompanionCover>,
}

impl Drop for AnimationManager {
  /// Signals the persistent timer thread to exit.
  ///
  /// Sets the shutdown flag, clears the ticking gate, and unparks the
  /// thread so it observes the shutdown and returns. The handle is
  /// dropped without joining so shutdown never blocks — a thread
  /// mid-`WaitForVBlank` on a sleeping monitor could otherwise stall the
  /// join indefinitely. The thread exits on its own and is reaped by the
  /// OS at process exit.
  fn drop(&mut self) {
    self.timer_shutdown.store(true, Ordering::Relaxed);
    self.animation_timer_running.store(false, Ordering::Relaxed);
    if let Some(timer) = self
      .timer_thread
      .lock()
      .expect("animation mutex poisoned")
      .take()
    {
      timer.handle.thread().unpark();
    }
  }
}

impl AnimationManager {
  /// Creates a new `AnimationManager`.
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      timer_thread: Mutex::new(None),
      timer_shutdown: Arc::new(AtomicBool::new(false)),
      last_frame_at: Cell::new(Instant::now()),
      stalled_timers: Arc::new(AtomicUsize::new(0)),
      pass_now: None,
      overlay_z_settle: HashMap::new(),
      animation_timer_vsync: Arc::new(Mutex::new(None)),
      frame_clock: Arc::new(Mutex::new(FrameClock::new(
        FrameClock::DEFAULT_PERIOD,
      ))),
      edge_color_cache: Arc::new(Mutex::new(HashMap::new())),
      pending_edge_samples: HashSet::new(),
      workspace_switch: None,
      peeks: super::peek_view::PeekView::default(),
      pending_ws_cleanup: None,
      ws_cleanup_step: WsCleanupStep::default(),
      interrupted_ws_windows: Vec::new(),
      sync_demoted_until: HashMap::new(),
      focus_fades: FocusFades::default(),
      companion_covers: HashMap::new(),
    }
  }

  /// How long until `window_id` comes to rest at `now`: once its peek and
  /// a workspace switch it slides in with have both ended. Zero when
  /// neither moves it.
  pub fn time_to_rest(&self, window_id: &Uuid, now: Instant) -> Duration {
    let switch = self
      .workspace_switch
      .as_ref()
      .filter(|_| self.is_workspace_switch_incoming(window_id))
      .map(|switch| match switch.start_time {
        Some(start) => {
          (start + switch.duration).saturating_duration_since(now)
        }
        None => switch.duration,
      });

    [switch, self.peeks.time_to_land(window_id, now)]
      .into_iter()
      .flatten()
      .max()
      .unwrap_or_default()
  }

  /// Returns `true` if a close animation is in flight for the given
  /// window.
  pub fn has_close_animation(&self, window_id: &Uuid) -> bool {
    self.has_away_animation(window_id, AwayTransition::Close)
  }

  /// Returns `true` while a minimize animation is playing for `window_id`.
  ///
  /// `platform_sync` checks this to keep its hands off a window whose
  /// peek is mid-minimize: the real window is already minimized.
  pub fn has_minimize_animation(&self, window_id: &Uuid) -> bool {
    self.has_away_animation(window_id, AwayTransition::Minimize)
  }

  fn has_away_animation(
    &self,
    window_id: &Uuid,
    transition: AwayTransition,
  ) -> bool {
    self.peeks.leaves_by(window_id, transition)
  }

  /// Returns `true` if a DWM surrogate is currently covering `window_id`.
  ///
  /// Used by backdrop-overlay tracking to skip position updates and hide
  /// the overlay while the real window is cloaked — preventing the
  /// overlay from jumping to the window's pre-positioned target rect while
  /// the surrogate is still mid-animation.
  pub fn has_active_surrogate(&self, window_id: &Uuid) -> bool {
    self.peeks.owns_window(window_id)
      || self
        .workspace_switch
        .as_ref()
        .is_some_and(|ws| ws.windows.contains_key(window_id))
      || self
        .pending_ws_cleanup
        .as_ref()
        .is_some_and(|ws| ws.windows.contains_key(window_id))
      || self.peeks.is_leaving(window_id)
  }

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

  /// Whether a stand-in other than a `CompanionCover` draws `window_id` or
  /// hands it back: a workspace switch's surrogate or a peek.
  pub fn stands_in_for(&self, window_id: &Uuid) -> bool {
    self.has_active_surrogate(window_id)
      || self.peeks.draws_window(window_id)
  }

  /// Covers `window_id`, about to be uncloaked, with `cover` until its
  /// companion is back over it.
  pub fn add_companion_cover(
    &mut self,
    window_id: Uuid,
    cover: CompanionCover,
  ) {
    self.companion_covers.insert(window_id, cover);
    self.ensure_timer_running();
  }

  pub fn has_companion_cover(&self, window_id: &Uuid) -> bool {
    self.companion_covers.contains_key(window_id)
  }

  pub fn drop_companion_cover(&mut self, window_id: &Uuid) {
    self.companion_covers.remove(window_id);
  }

  /// Keeps each companion cover over its window, dropping those done
  /// waiting (see `CompanionCover::awaits_companion`) and those whose
  /// window another stand-in has taken since.
  fn advance_companion_covers(&mut self) {
    let taken = self
      .companion_covers
      .keys()
      .filter(|id| self.stands_in_for(id))
      .copied()
      .collect::<Vec<_>>();

    for id in taken {
      self.companion_covers.remove(&id);
    }
    self
      .companion_covers
      .retain(|_, cover| cover.awaits_companion());
  }

  /// Ends the peek `window_id` is leaving on, as for a close or minimize.
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.peeks.forget_leaving(window_id);
  }

  /// Whether there are any active animations or a workspace-switch in
  /// flight.
  pub fn has_active_animations(&self) -> bool {
    if self.workspace_switch.is_some() {
      return true;
    }
    if self.peeks.is_active() {
      return true;
    }
    // Anything still holding a surrogate needs ticks to be torn down.
    // `pending_ws_cleanup` is cleared
    // only ticks after it is set, with a fallible `platform_sync` in
    // between: an error there leaves it set while `workspace_switch` is
    // already `None`. Without these two checks the timer then parks with
    // surrogates still up and the real windows still cloaked, which looks
    // like every window being frozen in place -- unmovable, and clicks
    // falling through the click-through surrogates into cloaked windows.
    // It resolves only when some unrelated new animation restarts the
    // timer and the pending teardown finally runs.
    if self.pending_ws_cleanup.is_some()
      || !self.companion_covers.is_empty()
    {
      return true;
    }
    // Settling overlays are re-checked from the tick; see
    // `settle_overlay_z_order`.
    if !self.overlay_z_settle.is_empty() {
      return true;
    }
    !self.focus_fades.is_empty()
  }

  /// Fades `window_id`'s effects from `from`'s style to `to`'s (see
  /// [`FocusFades::start`]).
  pub fn start_focus_fade(
    &mut self,
    window_id: Uuid,
    from: EffectSet,
    to: EffectSet,
    config: &UserConfig,
  ) {
    self.focus_fades.start(
      window_id,
      from,
      to,
      Instant::now(),
      &config.value.animations.focus_change,
    );

    if !self.focus_fades.is_empty() {
      self.ensure_timer_running();
    }
  }

  /// See [`FocusFades::restyle_focus`].
  pub fn restyle_focus(&mut self, focused: Option<Uuid>) -> Option<Uuid> {
    self.focus_fades.restyle_focus(focused)
  }

  /// Where `window_id` is in a fade toward `set`'s style, if it is fading
  /// toward it.
  pub fn focus_mix(
    &self,
    window_id: &Uuid,
    set: EffectSet,
    config: &UserConfig,
  ) -> Option<FocusMix> {
    self
      .focus_fades
      .mix(
        window_id,
        Instant::now(),
        &config.value.animations.focus_change,
      )
      .filter(|mix| mix.to == set)
  }

  /// Ends `window_id`'s focus fade, leaving it to whatever draws it now.
  pub fn cancel_focus_fade(&mut self, window_id: &Uuid) {
    self.focus_fades.cancel(window_id);
  }

  /// Windows with a focus fade, dropping the fades done, whose windows
  /// are still among those returned so they can be settled.
  pub fn focus_fade_windows(&mut self, config: &UserConfig) -> Vec<Uuid> {
    let ids = self.focus_fades.window_ids().collect();
    self.focus_fades.take_finished(
      Instant::now(),
      &config.value.animations.focus_change,
    );
    ids
  }

  /// Keeps re-asserting the overlays of `window_ids` behind their windows
  /// on every tick for [`OVERLAY_Z_SETTLE`], after their own z-order was
  /// changed.
  ///
  /// `set_z_order` moves the real window with `SWP_ASYNCWINDOWPOS` (plus a
  /// delayed retry), so it lands only after the same pass has already put
  /// the overlays behind the window's *old* position. A move that then
  /// takes the window down past them -- `AfterWindow` behind the focused
  /// window, or leaving the topmost band when a `shown_on_top` floating
  /// window is tiled again -- leaves its own opaque backdrop on top of it,
  /// and nothing else looks again. There is no completion signal to wait
  /// on, so the overlays are re-checked until the move has surely landed.
  pub fn settle_overlay_z_order(
    &mut self,
    window_ids: impl IntoIterator<Item = Uuid>,
  ) {
    let now = Instant::now();
    let settle = SettleWindow {
      until: now + OVERLAY_Z_SETTLE,
      cap: now + OVERLAY_Z_SETTLE_MAX,
    };
    self
      .overlay_z_settle
      .extend(window_ids.into_iter().map(|id| (id, settle)));

    if !self.overlay_z_settle.is_empty() {
      self.ensure_timer_running();
    }
  }

  /// Pushes `window_id`'s settle deadline out by another
  /// [`OVERLAY_Z_SETTLE`] from now, within its cap.
  ///
  /// For a window whose overlays cannot be judged settled yet: one still
  /// behind a surrogate (the fade-out tail anchors its overlays itself),
  /// or one whose async band change has not landed.
  ///
  /// Returns `true` only on the call that first clamps the deadline to the
  /// cap, so callers can report a settle that never finished once.
  pub fn extend_overlay_z_settle(&mut self, window_id: &Uuid) -> bool {
    let Some(settle) = self.overlay_z_settle.get_mut(window_id) else {
      return false;
    };

    let was_capped = settle.until >= settle.cap;
    settle.until = (Instant::now() + OVERLAY_Z_SETTLE).min(settle.cap);
    !was_capped && settle.until >= settle.cap
  }

  /// Windows whose overlays are still settling, dropping expired ones.
  pub fn settling_overlay_windows(&mut self) -> Vec<Uuid> {
    let now = Instant::now();
    self.overlay_z_settle.retain(|_, settle| settle.until > now);
    self.overlay_z_settle.keys().copied().collect()
  }

  /// Drops every stand-in's tracking state on shutdown or crash.
  ///
  /// Workspace-switch surrogates are dropped (real windows are already at
  /// their final positions by the time this is called).
  pub fn drop_all_stand_ins(&mut self) {
    self.workspace_switch = None;
    self.pending_ws_cleanup = None;
    self.companion_covers.clear();
    self.clear_waiter();
    // On WM shutdown close-animation windows are left open — only clear
    // the tracking state without sending WM_CLOSE.
    self.peeks.forget_all_leaving();
    self.peeks.drop_unused_surfaces();
  }

  /// Starts a ticking phase of the persistent animation timer thread.
  ///
  /// The timer thread is spawned once (on the first animation) and parked
  /// between animations rather than re-created each time, so the cost of
  /// thread creation and priority/MMCSS setup never lands on the
  /// input-to-first-frame latency path. This call wakes the parked thread
  /// (or spawns it the first time) when there are active animations and
  /// the thread is not already ticking.
  ///
  /// The thread uses a two-tier vsync strategy on Windows:
  ///
  /// 1. **`IDXGIOutput::WaitForVBlank`** — when a `DxgiVsyncWaiter` is
  ///    installed, waits for the animation monitor's specific vblank
  ///    signal. Per-monitor and reliable at any Hz.
  /// 2. **`DwmFlush`** — fallback when no waiter is installed, aligning to
  ///    the primary monitor's composition cycle.
  pub fn ensure_timer_running(&self) {
    if !self.has_active_animations() {
      return;
    }

    // Idempotent: if the thread is already ticking, there is nothing to
    // do. The swap also claims the idle -> ticking transition so only
    // one caller wakes the thread.
    if self.animation_timer_running.swap(true, Ordering::Relaxed) {
      return;
    }

    self.last_frame_at.set(Instant::now());

    let mut guard =
      self.timer_thread.lock().expect("animation mutex poisoned");
    match guard.as_ref() {
      // Wake the parked thread to begin a new ticking phase.
      Some(timer) => timer.handle.thread().unpark(),
      // First animation: spawn the persistent thread.
      None => {
        if let Some(timer) = self.spawn_timer_thread() {
          *guard = Some(timer);
        }
      }
    }
  }

  /// When the watchdog should step in if no frame has run by then, or
  /// `None` while nothing is animating.
  pub fn watchdog_deadline(&self) -> Option<Instant> {
    self
      .has_active_animations()
      .then(|| self.last_frame_at.get() + FRAME_STALL_TIMEOUT)
  }

  /// Replaces a timer thread that stopped ticking with a new one.
  ///
  /// The old thread is most likely blocked in `WaitForVBlank` on an output
  /// that went stale, which can block indefinitely. It is left to exit
  /// whenever that wait returns, and the vsync waiter is dropped so the
  /// new thread paces itself with `DwmFlush` instead.
  ///
  /// At most [`MAX_STALLED_TIMERS`] replaced threads are left blocked at
  /// once: past that, replacements are stalling too (e.g. `DwmFlush` with
  /// the displays off), and more threads would only pile up behind them.
  /// The watchdog keeps running frames itself until one recovers.
  pub fn replace_stalled_timer(&self) {
    if self.stalled_timers.load(Ordering::Relaxed) >= MAX_STALLED_TIMERS {
      tracing::debug!("Animation timer still stalled; not replacing it.");
      return;
    }

    tracing::warn!(
      "Animation timer stalled for over {FRAME_STALL_TIMEOUT:?}; \
       replacing it."
    );

    self.clear_waiter();

    if let Some(timer) = self
      .timer_thread
      .lock()
      .expect("animation mutex poisoned")
      .take()
    {
      self.stalled_timers.fetch_add(1, Ordering::Relaxed);
      timer.retired.store(true, Ordering::Relaxed);
      timer.handle.thread().unpark();
    }

    self.animation_timer_running.store(false, Ordering::Relaxed);
    self.ensure_timer_running();
  }

  /// Spawns the persistent animation timer thread.
  ///
  /// Returns the join handle, or `None` if the OS thread could not be
  /// created. On failure the ticking gate is reset so a later
  /// [`ensure_timer_running`] call retries the spawn.
  ///
  /// [`ensure_timer_running`]: AnimationManager::ensure_timer_running
  fn spawn_timer_thread(&self) -> Option<TimerThread> {
    let tx = self.animation_tick_tx.clone();
    let running = self.animation_timer_running.clone();
    let shutdown = self.timer_shutdown.clone();
    let retired = Arc::new(AtomicBool::new(false));
    let thread_retired = retired.clone();
    let stalled_guard = StalledTimerGuard {
      retired: retired.clone(),
      stalled_timers: self.stalled_timers.clone(),
    };
    let vsync_waiter = self.animation_timer_vsync.clone();
    let frame_clock = self.frame_clock.clone();

    // Spawn a real OS thread (not a Tokio task) so blocking vsync waits do
    // not stall the async runtime.
    let result = std::thread::Builder::new()
      .name("glazewm-anim-tick".into())
      .spawn(move || {
        let _stalled_guard = stalled_guard;

        // Elevate scheduling priority once for the thread's whole
        // lifetime. MMCSS "DisplayPostProcessing" gives
        // near-real-time guarantees beyond
        // THREAD_PRIORITY_HIGHEST, matching the scheduling class used by
        // DWM and video renderers. Falls back gracefully to
        // THREAD_PRIORITY_HIGHEST if avrt.dll is unavailable. The
        // persistent thread keeps the registration across
        // animations rather than re-acquiring it per switch; while
        // parked it consumes no CPU.
        wm_platform::set_thread_priority_highest();
        let _mmcss = wm_platform::try_set_thread_mmcss();

        let should_exit = || {
          shutdown.load(Ordering::Relaxed)
            || thread_retired.load(Ordering::Relaxed)
        };

        loop {
          // Idle: park until a ticking phase begins (or shutdown). A
          // `while` loop re-checks the gate to absorb spurious
          // wake-ups and any buffered unpark token from a
          // just-ended phase.
          while !running.load(Ordering::Relaxed) {
            if should_exit() {
              return;
            }
            std::thread::park();
          }
          if should_exit() {
            return;
          }

          // Send an immediate tick so the first animation frame begins
          // without waiting for the next vblank. Without this the
          // surrogate is frozen at its start position for up to
          // one full frame period (~16 ms at 60 Hz, ~5.7 ms at
          // 175 Hz) before any movement begins.
          if tx.send(()).is_err() {
            return;
          }

          // Ticking phase: drive frames until the gate clears.
          while running.load(Ordering::Relaxed) {
            // Per-monitor `IDXGIOutput::WaitForVBlank`. Cloned under the
            // lock so the wait runs without holding it: cleanup can clear
            // the waiter without blocking on a wait in progress.
            let waiter = vsync_waiter
              .lock()
              .expect("animation mutex poisoned")
              .clone();

            // `DwmFlush` when no waiter is installed, or its wait failed.
            // Its return is as periodic as a vblank, so it feeds the
            // clock the same way.
            if !waiter.is_some_and(|waiter| waiter.wait()) {
              wm_platform::dwm_flush();
            }

            frame_clock
              .lock()
              .unwrap_or_else(std::sync::PoisonError::into_inner)
              .note_vblank(Instant::now());

            // A retired thread's wait can return long after it was
            // replaced; its tick would only duplicate the new thread's.
            if should_exit() || tx.send(()).is_err() {
              return;
            }
          }
          // Gate cleared: loop back to park until the next animation.
        }
        // `_mmcss` is dropped on return, reverting the MMCSS registration.
      });

    match result {
      Ok(handle) => Some(TimerThread { handle, retired }),
      Err(err) => {
        tracing::warn!("Failed to spawn animation tick thread: {err}.");
        // Reset the gate so a later call retries the spawn.
        self.animation_timer_running.store(false, Ordering::Relaxed);
        None
      }
    }
  }

  /// Advances the peeks to `times`, ending the animations of windows done
  /// leaving. Returns the rest of what landed, with their moves queued in
  /// the batch, for [`track_overlays`] to commit along with their
  /// overlays'.
  ///
  /// [`track_overlays`]: super::peek_view::track_overlays
  fn tick_peeks(
    state: &mut WmState,
    times: FrameTimes,
  ) -> (super::peek_view::Landed, SurrogateBatch) {
    let batch = SurrogateBatch::new();
    super::peek_view::note_fill_inputs(state);
    let mut landed = state
      .animation_manager
      .peeks
      .tick(times.now, times.clock_origin);
    super::peek_view::finish_leaving(
      state,
      std::mem::take(&mut landed.left),
    );

    (landed, batch)
  }

  /// Applies the focus effects (transparency among them) while the
  /// focused window, landing in `shown`, is still cloaked, if the OS
  /// hasn't confirmed its focus yet.
  ///
  /// `sync_focus` defers `SetForegroundWindow` while a peek draws the
  /// focused window, so a focus change that scrolls only gets its effects
  /// from the focus event after the hand-back, while peeks, rings and
  /// backdrops wore the new styles all along: each window was handed back
  /// at its old alpha, the one losing focus still see-through over its
  /// faded-out backdrop for a few frames. As a workspace switch's end
  /// does (see `update_internal`).
  fn apply_landing_focus_effects(
    state: &mut WmState,
    shown: &[Uuid],
    config: &UserConfig,
  ) {
    let Some(focused) = state.focused_container() else {
      return;
    };
    let effects_on =
      state.prev_effects_window.as_ref().map(CommonGetters::id);
    if !lands_unconfirmed_focus(focused.id(), effects_on, shown) {
      return;
    }

    tracing::debug!(
      "Focus effects applied to {} as it lands, ahead of its focus event.",
      focused.id()
    );
    apply_focus_effects(state, &focused, false, config);
  }

  /// Moved columns that end shown get their real window back in the
  /// redraw below, and focus deferred while they moved lands then. Ones
  /// that end hidden have their real window put where it stays cloaked,
  /// and their overlays settled. Released ones get their overlays back on
  /// the real window. Ones no longer in the tree are skipped.
  fn queue_landed(state: &mut WmState, landed: super::peek_view::Landed) {
    if !landed.shown.is_empty() {
      state.pending_sync.queue_focus_change();
    }
    // Sampled at the next idle point, once handed back.
    for window_id in &landed.filled {
      if let Some(window) = state.window_by_id(*window_id) {
        state
          .animation_manager
          .pending_edge_samples
          .insert(window.native().hwnd().0);
      }
    }
    for window_id in landed
      .shown
      .into_iter()
      .chain(landed.hidden)
      .chain(landed.released)
    {
      if let Some(window) = state.window_by_id(window_id) {
        state.pending_sync.queue_container_to_redraw(window);
      }
    }
  }

  /// Internal update, accessed through `WmState` to avoid double-borrow.
  #[allow(clippy::too_many_lines)]
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    let woke_at =
      perf::is_enabled().then(|| state.animation_manager.tick_woke_at());
    state.animation_manager.last_frame_at.set(Instant::now());

    perf::begin_frame();
    let tick_scope = perf::scope(Stage::Tick);

    // A failed sync must not end the frame early: the teardown after it is
    // what removes finished surrogates and reveals the real windows. The
    // first error is returned once the frame is done.
    let mut frame_result = Ok(());

    // Finish what a newer slide cut short, the same way a completed slide
    // is finished: its surrogates already wait in `pending_ws_cleanup`.
    for id in
      std::mem::take(&mut state.animation_manager.interrupted_ws_windows)
    {
      if let Some(window) = state.window_by_id(id) {
        state.pending_sync.queue_container_to_redraw(window);
      }
    }

    // Clocks started this frame begin one frame period before it, so the
    // first tick already shows a frame of motion instead of repeating the
    // start position.
    let times = state.animation_manager.begin_frame_times();

    let peek_scope = woke_at
      .filter(|_| state.animation_manager.peeks.is_active())
      .map(|at| perf::scope_since(Stage::PeekFrame, Some(at)));
    let (landed, batch) = Self::tick_peeks(state, times);
    Self::apply_landing_focus_effects(state, &landed.shown, config);
    Self::queue_landed(state, landed);
    // Under `peek_frame` only, which a frame with no peek moving lacks.
    let track_scope =
      peek_scope.as_ref().map(|_| perf::scope(Stage::PeekTrack));
    super::peek_view::track_overlays(state, config, batch);
    drop(track_scope);
    drop(peek_scope);

    // Drive workspace-switch slide surrogates. All windows share a single
    // elapsed-time driver so every surrogate translates by the same pixel
    // offset each frame, making both workspaces move as one connected
    // panel.
    //
    // This runs before `platform_sync` so that when the animation
    // completes, the incoming windows are queued for redraw and
    // uncloaked in the same tick.
    let ws_complete_ids: Option<Vec<Uuid>> = {
      use crate::animation::engine::{
        animation_progress_at, apply_easing,
      };

      // Compute the predictive vsync timestamp before taking a mutable
      // borrow on `workspace_switch`. `predictive_vsync_now` reads
      // from the installed waiter, so the period is always current —
      // no stale cached field.
      let ws_vsync_now = state.animation_manager.predictive_vsync_now();

      // Likewise precomputed before the mutable borrow below -- needed by
      // the per-tick overlay tracker to pick each surrogate's effect
      // set.
      let focused_id_for_overlay =
        state.focused_container().map(|c| c.id());
      let urgent_ids: std::collections::HashSet<Uuid> = state
        .windows()
        .iter()
        .filter(|window| window.is_urgent())
        .map(CommonGetters::id)
        .collect();
      let effect_set = |id: Uuid, is_focused: bool| {
        EffectSet::of(is_focused, urgent_ids.contains(&id))
      };

      if let Some(ws) = &mut state.animation_manager.workspace_switch {
        // Anchor the animation clock to the first vsync-aligned tick so
        // every inter-frame step is measured on the same clock.
        // Falls back to the wall clock so progress always advances
        // if no vblank signal arrives.
        let now = match ws_vsync_now {
          Some(vsync) => {
            if !ws.vsync_anchored {
              ws.start_time = Some(vsync);
              ws.vsync_anchored = true;
            }
            vsync
          }
          None => Instant::now(),
        };
        let start = *ws.start_time.get_or_insert(now);
        let raw_progress = animation_progress_at(start, ws.duration, now);
        let eased = apply_easing(raw_progress, &ws.easing);

        // Complete early once the surrogate is within
        // `WS_COMPLETE_THRESHOLD_PX` of its target for non-overshooting
        // curves — decelerating easing spends a large fraction of wall
        // time covering the final sliver of distance, which looks
        // "stuck" at the destination. Gating on residual *pixels*
        // rather than a fixed progress fraction keeps the
        // completion-frame snap sub-pixel for any slide distance
        // or duration. Slide styles use their axis travel
        // distance; fade/zoom have no positional travel, so fall back to a
        // 99% fraction (a 1% opacity/scale snap is invisible).
        // Overshooting curves always run to full wall-clock
        // duration to preserve bounce.
        let ws_done = if ws.easing.can_overshoot() {
          raw_progress >= 1.0
        } else if raw_progress >= 1.0 {
          true
        } else {
          let slide_px = ws.motion().distance;
          if slide_px > 0 {
            #[allow(clippy::cast_precision_loss)]
            let remaining_px = (1.0 - eased) * slide_px as f32;
            remaining_px <= WS_COMPLETE_THRESHOLD_PX
          } else {
            eased >= 0.99
          }
        };

        // When completing early (eased < 1.0), snap surrogates to 1.0 so
        // they sit exactly at the final window position. Without this, a
        // ~1% gap between surrogate and the just-uncloaked real window
        // exposes the desktop for one frame.
        let eased_final = if ws_done { 1.0 } else { eased };

        // Incoming first, which a cut relies on: a composition landing
        // between the two halves of its one tick then shows the new
        // workspace drawn over the old one, never holes where neither is.
        let motion = ws.motion();
        let mut entries: Vec<_> = ws.windows.iter_mut().collect();
        entries.sort_by_key(|(_, entry)| !entry.is_incoming);

        for (&window_id, entry) in entries {
          if let Some(ref mut s) = entry.surrogate {
            // An outgoing window was hidden by its alpha as it was
            // cloaked, a tick ago at least, so the cloak is on
            // screen by now.
            if !entry.is_incoming {
              s.restore_alpha();

              // Likewise its copy of the backdrop covered the real one,
              // which can go, staying at rest while the window leaves.
              if let Some(overlay) = s
                .carries_backdrop()
                .then(|| state.window_overlays.get_mut(&window_id))
                .flatten()
                .and_then(|overlays| overlays.backdrop.as_mut())
                .filter(|overlay| overlay.is_visible())
              {
                overlay.hide();
              }
            }

            // At completion, hide outgoing surrogates immediately. They
            // have already slid fully off-screen, but hiding
            // the thumbnail outright guarantees nothing
            // lingers for the final composition frame
            // before the real windows are uncloaked.
            if ws_done && !entry.is_incoming {
              s.hide_thumbnail();
              s.hide_ring();
              s.hide_backdrop();

              // The ring is parked, or pinned to the viewport, rather
              // than tracking its window, so nothing else would take it
              // off-screen with the thumbnail it was following.
              if let Some(overlay) = state
                .window_overlays
                .get_mut(&window_id)
                .and_then(|overlays| overlays.border.as_mut())
              {
                overlay.hide();
              }

              continue;
            }

            // Before the motion update below, which then draws it.
            s.refresh_companion();

            let is_focused = Some(window_id) == focused_id_for_overlay;
            let (from_alpha, _, from_overlays) = surrogate_effects_for(
              effect_set(window_id, entry.was_focused),
              config,
            );
            let (to_alpha, _, to_overlays) = surrogate_effects_for(
              effect_set(window_id, is_focused),
              config,
            );

            // Set before the motion update below, which applies it.
            #[allow(
              clippy::cast_possible_truncation,
              clippy::cast_sign_loss
            )]
            s.set_effect_opacity(
              lerp_f32(
                f32::from(from_alpha),
                f32::from(to_alpha),
                eased_final,
              )
              .round() as u8,
            );

            let border_blend = StyleBlend::between(
              from_overlays.border,
              to_overlays.border,
              eased_final,
              NativeBorderOverlay::transparent,
            );

            // A carried ring is styled on its parked overlay here, and
            // moves with the window in the motion update below. Its
            // opacity follows the window's fade there too.
            if let (Some(blend), Some(overlay)) = (
              border_blend.filter(|_| s.carries_ring()),
              state
                .window_overlays
                .get_mut(&window_id)
                .and_then(|overlays| overlays.border.as_mut()),
            ) {
              overlay.apply_blend(blend);
              // A width change resized the overlay.
              s.carry_ring(overlay.hwnd(), &overlay.outer_rect());
            }

            match ws.style {
              WorkspaceSwitchStyle::Slide
              | WorkspaceSwitchStyle::Drift
              | WorkspaceSwitchStyle::SoftZoom => {
                let monitor_rect = Rect::from_xy(
                  ws.monitor_x,
                  ws.monitor_y,
                  ws.monitor_width,
                  ws.monitor_height,
                );
                let is_vertical =
                  ws.slide_direction == WorkspaceSwitchDirection::Vertical;
                s.slide_axis(
                  eased_final,
                  entry.is_incoming,
                  ws.order_direction,
                  &monitor_rect,
                  motion.distance,
                  if entry.is_incoming {
                    motion.zoom_incoming
                  } else {
                    motion.zoom_outgoing
                  },
                  is_vertical,
                );
              }
              // A cut only ever runs this on its one tick, where it shows
              // the incoming surrogates at full.
              WorkspaceSwitchStyle::Fade | WorkspaceSwitchStyle::Cut => {
                s.update_fade(eased_final, entry.is_incoming);
              }
              WorkspaceSwitchStyle::Zoom => {
                s.update_zoom(eased_final, entry.is_incoming);
              }
            }

            // A surrogate that doesn't carry a copy of the ring (see
            // `WorkspaceSurrogate::carry_ring`) has the overlay pinned to
            // the monitor viewport instead, its ring slid within it by a
            // composition offset. It follows the unclipped window rect,
            // since the ring sits *outside* that rect and so is not
            // described by the clipped visible strip.
            if let Some(mut blend) =
              border_blend.filter(|_| !s.carries_ring())
            {
              match (s.unclipped_rect(), s.panel()) {
                (Some(rect), Some(panel)) => {
                  // Fade the ring in step with the window it outlines, so
                  // a configured `opacity_incoming`/`opacity_outgoing`
                  // does not leave a full-strength border floating over a
                  // half-faded window.
                  let frac =
                    s.opacity_frac(eased_final, entry.is_incoming);
                  blend.from.opacity *= frac;
                  blend.to.opacity *= frac;

                  upsert_pinned_border_overlay(
                    &mut state.window_overlays,
                    window_id,
                    blend,
                    rect,
                    &ws.border_viewport,
                    panel,
                    s.hwnd(),
                  );
                }
                _ => {
                  if let Some(overlay) = state
                    .window_overlays
                    .get_mut(&window_id)
                    .and_then(|overlays| overlays.border.as_mut())
                  {
                    overlay.hide();
                  }
                }
              }
            }
          }
        }

        if ws_done {
          Some(ws.windows.keys().copied().collect())
        } else {
          None
        }
      } else {
        None
      }
    };

    // On completion, move surrogates to pending cleanup so they outlive
    // the final `platform_sync` call that unclocks the incoming real
    // windows.
    if let Some(ids) = ws_complete_ids {
      if let Some(ws) = state.animation_manager.workspace_switch.take() {
        state.animation_manager.stash_for_ws_cleanup(ws);
      }

      for id in ids {
        if let Some(window) = state.window_by_id(id) {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }

      // Re-queue focus: `sync_focus` suppressed `SetForegroundWindow`
      // while the animation was running to prevent the OS from
      // asynchronously uncloaking the incoming focused window
      // mid-slide. Now that the surrogates are done and incoming
      // windows are about to be uncloaked, it is safe to transfer OS
      // focus.
      state.pending_sync.queue_focus_change();

      // The surrogates ended on the post-switch focus styles. Apply them
      // to the real windows now, while they are still cloaked: queued for
      // the uncloaking sync instead, they would be applied a step after
      // the uncloak and show their old style (e.g. transparency) for a
      // frame.
      if let Some(focused) = state.focused_container() {
        apply_focus_effects(state, &focused, false, config);
      }
    }

    if state.pending_sync.has_changes() {
      keep_first_error(&mut frame_result, platform_sync(state, config));
    } else {
      // `platform_sync` syncs tab bars, which go with the peeks of moving
      // columns even when nothing else moved.
      if state.animation_manager.peeks.is_active() {
        crate::tab_bars::sync_tab_bars(
          state,
          config,
          crate::tab_bars::Restack::None,
        );
      }
    }

    advance_focus_fades(state, config);
    resync_settling_overlays(state);
    state.animation_manager.advance_companion_covers();

    // Held in an `Option` so the nested `platform_sync` below can close it
    // early: leaving it open there made `Cleanup` enclose a second
    // `PlatformSync`, double-counting that time so the two stages summed
    // to more than the enclosing `Tick`.
    let mut cleanup_scope = Some(perf::scope(Stage::Cleanup));

    // Redrawn by a follow-up `platform_sync` call: the windows whose
    // workspace-switch surrogates were dropped this tick, so that call can
    // settle their overlays' tint/rect through the normal path. Without
    // this second sync, an overlay would stay however it was left until
    // some unrelated redraw happens to fire -- the animation timer stops
    // right after this block when no other animation is active, so there
    // may be no later tick.
    let handed_back = advance_ws_cleanup(state, config);
    if !handed_back.is_empty() {
      let mut queued = 0;
      for id in handed_back {
        if let Some(window) = state.window_by_id(id) {
          state.pending_sync.queue_container_to_redraw(window);
          queued += 1;
        }
      }
      tracing::debug!(
        "Post-cleanup overlay re-sync: queued {queued} window(s) for \
         redraw, running follow-up platform_sync."
      );
      drop(cleanup_scope.take());
      keep_first_error(&mut frame_result, platform_sync(state, config));
    }

    drop(cleanup_scope);

    // Close the frame before deciding whether to report: `report` only
    // emits once at least one full frame has been rolled up.
    drop(tick_scope);
    perf::end_frame();

    // Keep the timer running while animations are active; stop it
    // otherwise so the background thread exits cleanly.
    if state.animation_manager.has_active_animations() {
      state.animation_manager.ensure_timer_running();
    } else {
      // Last frame of this animation burst: emit the accumulated timings.
      perf::report("animations idle");

      state
        .animation_manager
        .animation_timer_running
        .store(false, Ordering::Relaxed);

      // After the flag is cleared, which a sample is checked against.
      state.animation_manager.sample_pending_edge_colors();

      // All animations are done: clear the move/resize DXGI vsync waiter
      // so a later animation re-selects its own monitor and the
      // timer reverts to DwmFlush when vsync is unavailable.
      // The workspace switch clears its own waiter above; this
      // also covers move/resize, which has no dedicated completion
      // hook. Safe here because no animation is active.
      //
      // Tried caching the waiter across idle gaps instead (to skip the
      // DXGI re-enumeration cost on every new gesture) -- reverted: a
      // `DxgiVsyncWaiter` reused after sitting idle can go stale (e.g.
      // once display topology settles differently than when it was
      // created) and `WaitForVBlank` on a stale output can block far
      // longer than a frame instead of erroring cleanly, stalling
      // the tick thread and starving animations almost entirely. Not
      // worth the risk -- eagerly clearing it is what guarantees
      // `ensure_waiter_for` always creates a fresh, known-good
      // waiter for the monitor actually being animated on.
      state.animation_manager.clear_waiter();
    }

    frame_result
  }

  /// The instant this frame is evaluated at: the latest vblank plus a
  /// fraction of the frame period (see [`VSYNC_LEAD_FRACTION`]), or `None`
  /// while the timer thread hasn't woken for one lately.
  fn predictive_vsync_now(&self) -> Option<Instant> {
    let clock = self.clock();
    let vblank = clock.vblank_at(Instant::now())?;

    Some(vblank + clock.period().mul_f32(1.0 - VSYNC_LEAD_FRACTION))
  }

  /// The pacing monitor's frame period, or zero before a vsync waiter is
  /// installed.
  fn frame_period(&self) -> Duration {
    let has_waiter = self
      .animation_timer_vsync
      .lock()
      .is_ok_and(|waiter| waiter.is_some());

    if has_waiter {
      self.clock().period()
    } else {
      Duration::ZERO
    }
  }

  fn clock(&self) -> std::sync::MutexGuard<'_, FrameClock> {
    self
      .frame_clock
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
  }

  /// Drops the vsync waiter, so the timer thread paces itself with
  /// `DwmFlush` until an animation installs one.
  fn clear_waiter(&self) {
    *self
      .animation_timer_vsync
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    self.clock().reset(FrameClock::DEFAULT_PERIOD);
  }

  /// The time a frame drawn now is evaluated at, read afresh: for
  /// sampling a movement outside a redraw pass where it was last drawn.
  pub(crate) fn frame_time(&self) -> Instant {
    self.predictive_vsync_now().unwrap_or_else(Instant::now)
  }

  /// Starts a new redraw pass, so the next [`frame_now`] takes a fresh
  /// reading.
  ///
  /// [`frame_now`]: AnimationManager::frame_now
  pub fn begin_redraw_pass(&mut self) {
    self.pass_now = None;
  }

  /// The timestamp this redraw pass evaluates every window at: the
  /// predictive vsync instant on Windows when one is available, else
  /// wall-clock now -- read once per pass and shared.
  ///
  /// Shared because a window's animation clock starts at its first
  /// evaluation. Read per window, the reading lands after that window's
  /// stand-in setup (peek creation, preposition) and the ones before it,
  /// so windows relaid out together started up to a frame
  /// apart: measured 11ms between a shrinking window and the growing
  /// neighbour started after it, which the steep start of the easing
  /// curve turned into a ~190px hole between them on the first frame.
  pub(crate) fn frame_now(&mut self) -> Instant {
    if let Some(now) = self.pass_now {
      return now;
    }

    let now = self.frame_time();

    self.pass_now = Some(now);
    now
  }

  /// Starts an animation tick's redraw pass and reads the instants its
  /// frame is evaluated at.
  fn begin_frame_times(&mut self) -> FrameTimes {
    self.begin_redraw_pass();
    let now = self.frame_now();
    let clock_origin = now.checked_sub(self.frame_period()).unwrap_or(now);

    FrameTimes { now, clock_origin }
  }

  /// When the timer thread woke for the tick about to be drawn, for the
  /// profiler: its latest wake, if that came since the last frame began,
  /// else now, as a ticking phase's first tick is sent without a wait.
  fn tick_woke_at(&self) -> Instant {
    tick_wake(
      self.clock().last_wake(),
      self.last_frame_at.get(),
      Instant::now(),
    )
  }

  /// Paces animation ticks against the monitor with `monitor_handle`,
  /// i.e. the one the animating window is on. Leaves a workspace switch's
  /// monitor in place while it plays.
  pub(crate) fn ensure_waiter_for(&self, monitor_handle: isize) {
    if self.workspace_switch.is_none() {
      self.install_waiter(monitor_handle);
    }
  }

  /// Installs the vsync waiter of the monitor with `monitor_handle`,
  /// unless it is the one installed.
  ///
  /// Always switches, even to a slower refresh rate: keeping a faster
  /// waiter paces a 60 Hz monitor's animations against the wrong vblank,
  /// which reads as stutter specific to that monitor.
  fn install_waiter(&self, monitor_handle: isize) {
    let mut installed = self
      .animation_timer_vsync
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);

    if installed
      .as_ref()
      .is_some_and(|waiter| waiter.monitor_handle() == monitor_handle)
    {
      return;
    }

    match DxgiVsyncWaiter::for_monitor(monitor_handle) {
      Ok(waiter) => {
        let period = Duration::from_micros(waiter.frame_period_us());

        // Slow frames are counted against the frame period of whichever
        // monitor is actually pacing the animation, so the profiler stays
        // meaningful across a 60 Hz panel and a 175 Hz one alike.
        perf::set_frame_budget(period);
        tracing::debug!(
          monitor = monitor_handle,
          period_us = waiter.frame_period_us(),
          "vsync waiter switched"
        );

        *installed = Some(waiter);
        self.clock().reset(period);
      }
      Err(err) => {
        tracing::warn!(?err, "failed to create vsync waiter for monitor");
      }
    }
  }

  /// Queues a first edge-color sample for a window just shown that has
  /// none, so its first `fill` resize, or grab resize on peeks, has a
  /// fill.
  ///
  /// Taken at the next idle point like a resize's, by then after any open
  /// animation has landed; never at a resize's start (see
  /// `sample_pending_edge_colors`).
  pub fn queue_first_edge_sample(
    &mut self,
    hwnd: HWND,
    config: &UserConfig,
  ) {
    let resize = &config.value.animations.window_resize;
    let grab = &config.value.general.grab_and_move;
    let grab_fills = grab.enabled
      && grab.resize
      && grab.draw == wm_common::GrabDraw::Peeks;
    let fills = (resize.enabled && resize.style != ResizeStyle::Stretch)
      || grab_fills;
    if fills && self.cached_edge_color(hwnd.0).is_none() {
      self.pending_edge_samples.insert(hwnd.0);
    }
  }

  /// Samples the edge color of every window that animated, or was shown
  /// with none yet, since the last idle point.
  ///
  /// Deferred to here because the screen readback stalls DWM: run at a
  /// motion's start, it more than tripled keypress-to-motion latency.
  fn sample_pending_edge_colors(&mut self) {
    for hwnd in self.pending_edge_samples.drain() {
      sample_edge_color_async(
        HWND(hwnd),
        self.edge_color_cache.clone(),
        EDGE_COLOR_CACHE_PRUNE_LEN,
        EDGE_COLOR_CACHE_TTL,
        self.animation_timer_running.clone(),
      );
    }
  }

  /// Returns the last sampled surrogate backdrop color for `hwnd`,
  /// however old it is.
  ///
  /// Age is not consulted: the color only ever stands in for window
  /// content in a strip the thumbnail has not reached yet, and an app's
  /// background changes about as often as its theme -- so the previous
  /// sample beats none, which leaves that strip showing the backdrop raw
  /// and undimmed. Callers re-sample after every motion (see
  /// [`sample_edge_color_async`]), so this entry is never more than one
  /// animation behind the window.
  ///
  /// `sample_edge_color_async`'s background thread populates this same map
  /// directly (see `edge_color_cache`'s doc comment), so a lock failure
  /// here is treated the same as a miss rather than propagated.
  pub(super) fn cached_edge_color(&self, hwnd: isize) -> Option<Color> {
    let map = self.edge_color_cache.lock().ok()?;
    map.get(&hwnd).map(|(color, _)| *color)
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

  /// Whether `native` may be moved synchronously, to stay in step with its
  /// surrogate.
  ///
  /// A synchronous `SetWindowPos` waits on the window's app, so it is
  /// skipped for an app the OS reports as hung, and for one whose last
  /// synchronous move was slow (see [`note_sync_reposition`]).
  ///
  /// [`note_sync_reposition`]: AnimationManager::note_sync_reposition
  pub fn allows_sync_reposition(&mut self, native: &NativeWindow) -> bool {
    let handle = native.hwnd().0;

    if let Some(until) = self.sync_demoted_until.get(&handle) {
      if Instant::now() < *until {
        return false;
      }
      self.sync_demoted_until.remove(&handle);
    }

    !native.is_hung()
  }

  /// Records how long a synchronous move of the window `handle` took, and
  /// moves it asynchronously for a while if that was slow.
  pub fn note_sync_reposition(
    &mut self,
    handle: isize,
    elapsed: Duration,
  ) {
    if elapsed < SLOW_SYNC_DEMOTE_THRESHOLD {
      return;
    }

    tracing::warn!(
      "Synchronous move of window {handle:#x} took {elapsed:?}; moving it \
       asynchronously for {SYNC_DEMOTE_DURATION:?}."
    );

    let now = Instant::now();
    self.sync_demoted_until.retain(|_, until| *until > now);
    self
      .sync_demoted_until
      .insert(handle, now + SYNC_DEMOTE_DURATION);
  }

  /// Hands a finished or interrupted slide's surrogates to
  /// `pending_ws_cleanup`, merging with any still waiting there so that
  /// none are destroyed before their real window is uncloaked.
  fn stash_for_ws_cleanup(&mut self, mut ws: WorkspaceSwitchState) {
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

/// Takes a finished workspace switch's teardown one step further (see
/// `WsCleanupStep`), returning its windows once its surrogates are gone.
fn advance_ws_cleanup(
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

/// What a window's move or resize animation is for, which picks the
/// settings it runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
  /// A pure translation (`window_move`).
  Move,
  /// A change of size (`window_resize`).
  Resize,
  /// A scrolling layout's column moving or resizing (`scroll`).
  Column,
  /// A window going in or out of fullscreen (`window_fullscreen`).
  Fullscreen,
}

/// A window entering or leaving that a relayout's tiles make way for,
/// whose timing they move on (see [`relayout_timing`]).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MakingWay {
  direction: Direction,
  duration_ms: u32,
  easing: EasingFunction,
}

impl MakingWay {
  /// For a window opening or restoring by `params`.
  pub fn entering(params: &WindowTransitionParams) -> Self {
    Self::new(Direction::In, params)
  }

  /// For a window closing or minimizing by `params`.
  pub fn leaving(params: &WindowTransitionParams) -> Self {
    Self::new(Direction::Out, params)
  }

  fn new(direction: Direction, params: &WindowTransitionParams) -> Self {
    Self {
      direction,
      duration_ms: params.duration_ms,
      easing: params.easing.clone(),
    }
  }
}

/// The duration and easing a tile's `motion` runs on in a relayout that
/// makes way for a window entering or leaving, if any: the same as that
/// window's, so the edges they share meet throughout rather than drifting
/// apart and closing up again at the end.
///
/// For a leaving window, only its duration: its easing is usually an
/// ease-in, which would bring the tiles growing over it to a dead stop.
/// Columns and fullscreen toggles keep their own (see
/// [`movement_timing`]).
pub(crate) fn relayout_timing(
  motion: Motion,
  cycle_has_resize: bool,
  making_way: Option<&MakingWay>,
  config: &UserConfig,
) -> (u32, EasingFunction) {
  let own = movement_timing(motion, cycle_has_resize, config);

  match (motion, making_way) {
    (Motion::Move | Motion::Resize, Some(way)) => match way.direction {
      Direction::In => (way.duration_ms, way.easing.clone()),
      Direction::Out => (way.duration_ms, own.1),
    },
    _ => own,
  }
}

/// The duration and easing a window's `motion` runs on.
///
/// When any window in the cycle resizes, pure translations share the
/// resize timing too, keeping all edges in lock-step.
pub(crate) fn movement_timing(
  motion: Motion,
  cycle_has_resize: bool,
  config: &UserConfig,
) -> (u32, EasingFunction) {
  let animations = &config.value.animations;
  let use_resize_timing = motion == Motion::Resize
    || (motion == Motion::Move
      && cycle_has_resize
      && animations.window_resize.enabled);

  match motion {
    Motion::Column => (
      animations.scroll.duration_ms,
      animations.scroll.easing.clone(),
    ),
    Motion::Fullscreen => (
      animations.window_fullscreen.duration_ms,
      animations.window_fullscreen.easing.clone(),
    ),
    _ if use_resize_timing => (
      animations.window_resize.duration_ms,
      animations.window_resize.easing.clone(),
    ),
    _ => (
      animations.window_move.duration_ms,
      animations.window_move.easing.clone(),
    ),
  }
}

/// The wake a tick being drawn at `now` came from: the timer thread's
/// `last_wake`, if it came since the frame before began at
/// `last_frame_at`. An older one was a tick drawn already, or one before
/// the ticking phase began (`ensure_timer_running` resets
/// `last_frame_at`), whose first tick is sent at once: then `now`.
fn tick_wake(
  last_wake: Option<Instant>,
  last_frame_at: Instant,
  now: Instant,
) -> Instant {
  last_wake
    .filter(|wake| *wake > last_frame_at && *wake <= now)
    .unwrap_or(now)
}

/// Whether `focused` lands among `shown` with the focus effects still on
/// `effects_on`, the window the OS last confirmed focus for (see
/// `AnimationManager::apply_landing_focus_effects`).
fn lands_unconfirmed_focus(
  focused: Uuid,
  effects_on: Option<Uuid>,
  shown: &[Uuid],
) -> bool {
  effects_on != Some(focused) && shown.contains(&focused)
}

#[cfg(test)]
mod tests {
  use wm_common::ParsedConfig;

  use super::*;

  #[test]
  fn landing_focus_takes_its_effects_only_if_unconfirmed() {
    let (focused, other) = (Uuid::new_v4(), Uuid::new_v4());

    assert!(lands_unconfirmed_focus(focused, Some(other), &[focused]));
    assert!(lands_unconfirmed_focus(focused, None, &[other, focused]));
    assert!(!lands_unconfirmed_focus(focused, Some(focused), &[focused]));
    assert!(!lands_unconfirmed_focus(focused, Some(other), &[other]));
  }

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

  /// A window just shown is queued for a first fill color only while a
  /// `fill` resize could use one and it has none yet.
  #[test]
  fn a_shown_window_is_queued_for_a_fill_color_it_lacks() {
    let queues = |edit: fn(&mut ParsedConfig), cached: bool| {
      let (tx, _rx) = mpsc::unbounded_channel();
      let mut manager = AnimationManager::new(tx);
      if cached {
        if let Ok(mut map) = manager.edge_color_cache.lock() {
          let color = Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
          };
          map.insert(7, (color, Instant::now()));
        }
      }
      let mut parsed = ParsedConfig::default();
      edit(&mut parsed);
      let config = UserConfig::from_parsed(parsed);

      manager.queue_first_edge_sample(HWND(7), &config);
      manager.pending_edge_samples.contains(&7)
    };

    assert!(queues(|_| {}, false));
    assert!(!queues(|_| {}, true));
    assert!(!queues(
      |parsed| parsed.animations.window_resize.enabled = false,
      false
    ));
    assert!(!queues(
      |parsed| parsed.animations.window_resize.style =
        ResizeStyle::Stretch,
      false
    ));
  }

  /// Tiles making way for an entering window take its timing, for a
  /// leaving one its duration only; columns and fullscreen toggles keep
  /// their own.
  #[test]
  fn tiles_making_way_move_in_step() {
    let mut parsed = ParsedConfig::default();
    parsed.animations.window_open.duration_ms = 400;
    parsed.animations.window_open.easing =
      EasingFunction::CubicBezier(0.05, 0.9, 0.1, 1.05);
    parsed.animations.window_close.duration_ms = 300;
    let config = UserConfig::from_parsed(parsed);
    let animations = &config.value.animations;
    let open = MakingWay::entering(&WindowTransitionParams::from_open(
      &animations.window_open,
    ));
    let close = MakingWay::leaving(&WindowTransitionParams::from_close(
      &animations.window_close,
    ));
    let resize = (
      animations.window_resize.duration_ms,
      animations.window_resize.easing.clone(),
    );

    for motion in [Motion::Move, Motion::Resize] {
      assert_eq!(
        relayout_timing(motion, true, Some(&open), &config),
        (400, animations.window_open.easing.clone())
      );
      assert_eq!(
        relayout_timing(motion, true, Some(&close), &config),
        (300, resize.1.clone())
      );
      assert_eq!(relayout_timing(motion, true, None, &config), resize);
    }
    for motion in [Motion::Column, Motion::Fullscreen] {
      assert_eq!(
        relayout_timing(motion, true, Some(&open), &config),
        movement_timing(motion, true, &config)
      );
    }
  }

  /// The profiler times a peek frame from the wake its tick came from,
  /// not from one already drawn or from before the ticking phase.
  #[test]
  fn a_tick_is_timed_from_its_own_wake() {
    let t0 = Instant::now();
    let at = |ms| t0 + Duration::from_millis(ms);

    assert_eq!(tick_wake(Some(at(10)), at(5), at(12)), at(10));
    assert_eq!(tick_wake(Some(at(4)), at(5), at(12)), at(12));
    assert_eq!(tick_wake(None, at(5), at(12)), at(12));
    assert_eq!(tick_wake(Some(at(13)), at(5), at(12)), at(12));
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
  fn stalled_guard_counts_only_retired_threads() {
    let stalled_timers = Arc::new(AtomicUsize::new(1));

    for (is_retired, expected) in [(false, 1), (true, 0)] {
      drop(StalledTimerGuard {
        retired: Arc::new(AtomicBool::new(is_retired)),
        stalled_timers: stalled_timers.clone(),
      });
      assert_eq!(stalled_timers.load(Ordering::Relaxed), expected);
    }
  }

  #[test]
  fn no_replacement_while_a_replaced_timer_is_stalled() {
    let (tick_tx, _tick_rx) = mpsc::unbounded_channel();
    let manager = AnimationManager::new(tick_tx);
    manager
      .stalled_timers
      .store(MAX_STALLED_TIMERS, Ordering::Relaxed);
    manager
      .animation_timer_running
      .store(true, Ordering::Relaxed);

    manager.replace_stalled_timer();

    // The running gate is left as it was, so no new thread was started.
    assert!(manager.animation_timer_running.load(Ordering::Relaxed));
    assert_eq!(
      manager.stalled_timers.load(Ordering::Relaxed),
      MAX_STALLED_TIMERS
    );
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
