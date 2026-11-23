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
use wm_common::{EasingFunction, EffectSet, WindowTransitionParams};
use wm_platform::{
  perf::{self, Stage},
  CompanionCover, DxgiVsyncWaiter, EdgeColorCache, FrameClock,
  NativeWindow, Rect, SurrogateBatch,
};
pub use workspace_switch::SwitchedTabBar;
use workspace_switch::{
  advance_ws_cleanup, WorkspaceSwitchState, WsCleanupStep,
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
    resync_settling_overlays,
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
      .unwrap_or_else(std::sync::PoisonError::into_inner)
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

    // Before `platform_sync`, so that when the switch completes, the
    // incoming windows are queued for redraw and uncloaked in the same
    // tick.
    let ws_complete_ids = Self::tick_workspace_switch(state, config);

    if let Some(ids) = ws_complete_ids {
      Self::finish_workspace_switch(state, config, ids);
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
      state.animation_manager.end_burst();
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
