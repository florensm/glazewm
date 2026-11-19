//! `AnimationManager`: what every animation needs, ticked by one timer.
//!
//! `frame_timer` runs the timer thread and paces frames by the monitor's
//! vblank, `workspace_switch` plays switches and tears them down, and
//! `edge_colors` samples the colors filled windows are drawn with.

mod edge_colors;
mod frame_timer;
mod workspace_switch;

use std::{
  cell::Cell,
  collections::{HashMap, HashSet},
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
  },
  time::{Duration, Instant},
};

use frame_timer::{FrameTimes, TimerThread};
use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{
  EasingFunction, EffectSet, WindowTransitionParams,
  WorkspaceSwitchDirection, WorkspaceSwitchStyle,
};
use wm_platform::{
  lerp_f32,
  perf::{self, Stage},
  CompanionCover, DxgiVsyncWaiter, EdgeColorCache, FrameClock,
  NativeBorderOverlay, NativeWindow, Overlay, Rect, StyleBlend,
  SurrogateBatch,
};
pub use workspace_switch::SwitchedTabBar;
use workspace_switch::{
  advance_ws_cleanup, WorkspaceSwitchState, WsCleanupStep,
  WS_COMPLETE_THRESHOLD_PX,
};
pub(crate) use workspace_switch::{
  restack_switch_tenants, switch_crossing, HandBack, WorkspaceSwitchEntry,
};

use crate::{
  animation::{
    focus_fade::{FocusFades, FocusMix},
    transit::Direction,
  },
  commands::general::{
    advance_focus_fades, apply_focus_effects, platform_sync,
    resync_settling_overlays, surrogate_effects_for,
    upsert_pinned_border_overlay, SyncableOverlay,
  },
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

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

/// Why a window is taken off the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AwayTransition {
  Close,
  Minimize,
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
  /// period (see
  /// `VSYNC_LEAD_FRACTION`) rather
  /// than at `Instant::now()`, which varies with how late the thread
  /// woke and how long the WM thread took to get to the tick.
  frame_clock: Arc<Mutex<FrameClock>>,
  /// Recently sampled backdrop fill colors keyed by window handle, kept
  /// for `EDGE_COLOR_CACHE_TTL`.
  /// Shared (rather than owned outright) because
  /// `sample_edge_color_async`'s background thread inserts into this
  /// same map directly once a sample completes, instead of routing the
  /// result back through the event loop.
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

  /// Whether a stand-in other than a `CompanionCover` draws `window_id` or
  /// hands it back: a workspace switch's surrogate or a peek.
  pub fn stands_in_for(&self, window_id: &Uuid) -> bool {
    self.has_active_surrogate(window_id)
      || self.peeks.draws_window(window_id)
  }

  /// Covers `window_id`, about to be uncloaked, moved or resized, with
  /// `cover` until its companion is back over it.
  pub fn add_companion_cover(
    &mut self,
    window_id: Uuid,
    cover: CompanionCover,
  ) {
    self.companion_covers.insert(window_id, cover);
    self.ensure_timer_running();
  }

  /// The cover of `window_id`, e.g. to move it with the window.
  pub fn companion_cover_mut(
    &mut self,
    window_id: &Uuid,
  ) -> Option<&mut CompanionCover> {
    self.companion_covers.get_mut(window_id)
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
}
