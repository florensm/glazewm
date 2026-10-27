#[cfg(target_os = "windows")]
use std::collections::HashSet;
use std::{
  cell::Cell,
  collections::HashMap,
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
#[cfg(target_os = "windows")]
const VSYNC_LEAD_FRACTION: f32 = 0.5;

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
#[cfg(target_os = "windows")]
const OPEN_PAINT_GRACE: Duration = Duration::from_millis(30);

/// Duration of the surrogate fade-out at animation completion.
///
/// After the real window is uncloaked beneath the (pixel-aligned, live)
/// surrogate, the surrogate's opacity ramps to zero over this period so
/// shadow, border, and late-repaint differences blend in rather than
/// switching in a single composition.
#[cfg(target_os = "windows")]
const SESSION_FADE_OUT: Duration = Duration::from_millis(100);

/// Maximum mid-animation handoff lead in milliseconds.
///
/// The actual lead is `duration_ms * 0.35`, clamped to
/// `[50, HANDOFF_LEAD_MAX_MS]`. Scaling by duration keeps the handoff near
/// the end of the visual travel regardless of easing speed; the cap
/// ensures apps always get ≥50 ms to repaint at the new size before
/// uncloaking.
#[cfg(target_os = "windows")]
const HANDOFF_LEAD_MAX_MS: u64 = 100;

/// Maximum number of real-window handoffs (`ResizeSession::maybe_handoff`)
/// performed per redraw pass.
///
/// Each handoff issues a synchronous `SetWindowPos` plus a DWM thumbnail
/// dims update on the WM thread. When a relayout brings many sessions past
/// their handoff lead on the same tick (e.g. a workspace-wide resize),
/// doing all of them in one pass can stack up enough synchronous work to
/// stall that tick. Capping the count spreads the overflow across the next
/// few ticks (each ~6-16 ms) instead of widening any individual window's
/// lead -- deferred windows retry every tick since `remaining_at` stays ≤
/// the lead until they're handed off, so the extra delay is at most a
/// couple of frames, not a fraction of the animation's duration.
#[cfg(target_os = "windows")]
const MAX_HANDOFFS_PER_TICK: usize = 3;

/// How long a sampled surrogate backdrop color stays valid per window.
///
/// Sampling costs two GPU→CPU `BitBlt` readbacks; now always done on a
/// background thread via `sample_edge_color_async` rather than blocking
/// the WM's main thread, but a miss still means the *current* session
/// plays with a transparent backdrop instead of a matched color. App
/// background colors change rarely (theme switches), so a stale hit is a
/// cosmetic-only risk -- kept generous so a window resized only
/// occasionally still benefits from the earlier background sample instead
/// of re-paying for one every time.
#[cfg(target_os = "windows")]
const EDGE_COLOR_CACHE_TTL: Duration = Duration::from_secs(300);

/// Cache-size threshold above which stale edge-color entries are pruned.
#[cfg(target_os = "windows")]
const EDGE_COLOR_CACHE_PRUNE_LEN: usize = 128;

/// How long a window's surrogate stays warm in
/// [`AnimationManager::warm_surrogates`] after its resize/move session
/// ends, available for a follow-up resize of the same window to reuse
/// instead of paying full `CreateWindowExW`/`DwmRegisterThumbnail` cost
/// again.
///
/// Long enough to cover a realistic "resize, look at it, resize again"
/// fine-tuning pause; short enough that a window resized once and never
/// touched again doesn't keep its overlay window and DWM thumbnail
/// registration alive indefinitely.
#[cfg(target_os = "windows")]
const WARM_SURROGATE_TTL: Duration = Duration::from_secs(8);

/// How long a window's overlays keep being re-asserted behind it after its
/// own z-order last changed, or after its surrogate went away. See
/// `AnimationManager::settle_overlay_z_order`.
///
/// Covers the asynchronous move itself, `set_z_order`'s 10ms retry, and
/// apps that briefly re-assert their own z-order in response; measured
/// cases land within a few frames.
#[cfg(target_os = "windows")]
const OVERLAY_Z_SETTLE: Duration = Duration::from_millis(200);

/// Upper bound on how long a window's overlays keep settling, however
/// often the settle is extended -- so a window that never reaches its
/// intended band (an app that refuses `HWND_TOPMOST`) cannot keep the
/// animation timer ticking forever.
#[cfg(target_os = "windows")]
const OVERLAY_Z_SETTLE_MAX: Duration = Duration::from_secs(2);

/// Deadlines for one window in `AnimationManager::overlay_z_settle`.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug)]
struct SettleWindow {
  /// When re-checking stops, unless extended.
  until: Instant,
  /// When re-checking stops regardless; see [`OVERLAY_Z_SETTLE_MAX`].
  cap: Instant,
}

/// Cache-size threshold above which stale warm-surrogate entries are
/// pruned. Kept much smaller than [`EDGE_COLOR_CACHE_PRUNE_LEN`] -- a warm
/// surrogate holds a real OS window and DWM thumbnail registration, not
/// just a color value, so letting the cache grow large before pruning is a
/// heavier idle cost.
#[cfg(target_os = "windows")]
const WARM_SURROGATE_PRUNE_LEN: usize = 16;

/// Inserts `key`/`value` into a TTL-cache `map`, first pruning entries
/// older than `ttl` once the map has grown to `prune_len`.
///
/// Used by [`AnimationManager`]'s `warm_surrogates` map.
/// `edge_color_cache` uses the same size-then-age strategy but applies it
/// inline from `sample_edge_color_async`'s background thread instead,
/// since that map is written to from there rather than from the main
/// thread.
#[cfg(target_os = "windows")]
fn prune_and_insert<K: std::hash::Hash + Eq, V>(
  map: &mut HashMap<K, (V, Instant)>,
  prune_len: usize,
  ttl: Duration,
  key: K,
  value: V,
) {
  if map.len() >= prune_len {
    map.retain(|_, (_, inserted_at)| inserted_at.elapsed() < ttl);
  }
  map.insert(key, (value, Instant::now()));
}

/// A synchronous move slower than this demotes its window to asynchronous
/// moves. Several frames' worth: the whole WM waits on it.
#[cfg(target_os = "windows")]
const SLOW_SYNC_DEMOTE_THRESHOLD: Duration = Duration::from_millis(50);

/// How long a window demoted by [`SLOW_SYNC_DEMOTE_THRESHOLD`] is moved
/// asynchronously.
#[cfg(target_os = "windows")]
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
#[cfg(target_os = "windows")]
use wm_common::{
  ResizeStyle, WindowTransitionParams, WindowTransitionStyle,
};
use wm_platform::{
  perf::{self, Stage},
  NativeWindow, OpacityValue, Rect,
};
#[cfg(target_os = "windows")]
use wm_platform::{
  sample_edge_color_async, BackdropOverlayParams, BorderOverlayParams,
  Color, CornerStyle, DxgiVsyncWaiter, EdgeColorCache, NativeSurrogate,
  NativeWindowWindowsExt, Overlay, ResizeSession, SessionOptions,
  SurrogateBatch, HWND,
};

#[cfg(target_os = "windows")]
use crate::animation::stand_in::{moved_along, StandIn};
#[cfg(target_os = "windows")]
use crate::commands::general::{
  overlay_z_anchor, resync_settling_overlays, upsert_overlay,
};
use crate::{
  animation::state::WindowAnimationState,
  commands::general::platform_sync, traits::CommonGetters,
  user_config::UserConfig, wm_state::WmState,
};

/// A single entry in the surrogate update queue built each redraw pass.
#[cfg(target_os = "windows")]
struct PendingSurrogateUpdate {
  window_id: Uuid,
  rect: Rect,
  opacity: u8,
  /// `true` when `remaining_at` ≤ the proportional handoff lead.
  handoff: bool,
}

/// Result of [`AnimationManager::start_animation_if_needed`], describing
/// what the caller should do with the real app window's position this
/// frame.
pub enum AnimationPositionResult {
  /// Apply this rect to the real window via `reposition_window`.
  ///
  /// The carried `Rect` is the current animated position, available for
  /// callers that bypass the surrogate path (e.g. future macOS support).
  Apply(Rect),
  /// The surrogate overlay is handling all visuals; skip repositioning
  /// the real window this frame.
  // LINT: Surrogates only exist on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  Frozen,
}

/// Manages animations for all windows.
pub struct AnimationManager {
  /// Active animations keyed by window ID.
  animations: HashMap<Uuid, WindowAnimationState>,
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
  #[cfg(target_os = "windows")]
  overlay_z_settle: HashMap<Uuid, SettleWindow>,
  /// DXGI vsync waiter for the animation monitor.
  ///
  /// When `Some`, the timer thread calls `WaitForVBlank` on this output
  /// instead of `DwmFlush`. This gives a full frame period after each
  /// vsync to update surrogates, regardless of which monitor is the
  /// Windows primary. Cleared on workspace-switch completion.
  #[cfg(target_os = "windows")]
  animation_timer_vsync: Arc<Mutex<Option<DxgiVsyncWaiter>>>,
  /// Timestamp of the most recent `IDXGIOutput::WaitForVBlank` wake-up.
  ///
  /// Written by the timer thread immediately after vsync fires. Read by
  /// `update_internal` to compute animation progress at a predictive
  /// timestamp (vsync time + a fraction of the vblank period; see
  /// [`VSYNC_LEAD_FRACTION`]) rather than `Instant::now()`, compensating
  /// for the pipeline delay between vsync wake and DWM composition.
  #[cfg(target_os = "windows")]
  animation_vsync_time: Arc<Mutex<Option<Instant>>>,
  /// Active resize sessions keyed by window ID.
  ///
  /// A session is created when a movement/resize animation starts with
  /// `use_surrogate = true` and is destroyed once the animation completes
  /// and the real window has been moved to its final position.
  #[cfg(target_os = "windows")]
  pub(crate) resize_sessions: HashMap<Uuid, ResizeSession>,
  /// Set whenever a new surrogate window is created with `place_at_top:
  /// true` (workspace-switch, or a resize/move/open session -- see
  /// `SessionOptions::place_at_top`) -- such surrogates are inserted at
  /// `HWND_TOP`, which can silently displace *any* other window's static
  /// backdrop overlay out of its correct just-behind-its-own-window
  /// z-order slot, not just the surrogate's own window. `sync_overlays`
  /// checks this to fall back to a full per-window z-order resync for one
  /// tick instead of its normal narrowly-targeted one, then clears it.
  /// Not cleared on its own -- callers that create such a surrogate are
  /// responsible for setting it, `platform_sync` for resetting it once
  /// both overlay kinds have acted on it.
  #[cfg(target_os = "windows")]
  pub(crate) backdrop_overlay_z_order_dirty: bool,
  /// Recently sampled surrogate backdrop colors keyed by window handle.
  ///
  /// Lets `ResizeSession::begin` skip its two-`BitBlt` screen sample for
  /// windows resized again within [`EDGE_COLOR_CACHE_TTL`]. Shared
  /// (rather than owned outright) because `sample_edge_color_async`'s
  /// background thread inserts into this same map directly once a
  /// sample completes, instead of routing the result back through the
  /// event loop.
  #[cfg(target_os = "windows")]
  edge_color_cache: EdgeColorCache,
  /// Windows whose edge color is re-sampled once animations go idle.
  #[cfg(target_os = "windows")]
  pending_edge_samples: HashSet<isize>,
  /// Surrogates kept alive (hidden) after their resize/move session ends,
  /// keyed by window ID, so a follow-up resize of the same window within
  /// [`WARM_SURROGATE_TTL`] can reuse the existing overlay window and DWM
  /// thumbnail registration via `NativeSurrogate::revive` instead of
  /// paying full `CreateWindowExW`/`DwmRegisterThumbnail` cost again.
  /// See `AnimationManager::reclaim_surrogate`/`stash_warm_surrogate`.
  #[cfg(target_os = "windows")]
  warm_surrogates: HashMap<Uuid, (NativeSurrogate, Instant)>,
  /// Surrogate updates queued during this redraw pass; drained into the
  /// pass's shared `SurrogateBatch` by [`queue_surrogate_updates`] so
  /// adjacent surrogates land in the same DWM composition frame.
  ///
  /// [`queue_surrogate_updates`]: AnimationManager::queue_surrogate_updates
  #[cfg(target_os = "windows")]
  pending_surrogate_updates: Vec<PendingSurrogateUpdate>,
  /// Sessions that have been removed from `resize_sessions` after their
  /// animation completed but must outlive the final `platform_sync` call
  /// that repositions the real window. Keyed by window ID so the final
  /// redraw can detect that `pre_commit` already positioned the window.
  ///
  /// The `Option<Instant>` is the fade-out start time: `None` until the
  /// real window has been uncloaked beneath the surrogate, then set on
  /// the first cleanup tick. Entries are dropped once the fade
  /// completes.
  #[cfg(target_os = "windows")]
  pub(crate) pending_session_cleanup:
    Vec<(Uuid, Option<Instant>, ResizeSession)>,
  /// Monitor rects for active slide-in (window-open) animations, keyed by
  /// window ID. Used to hide the surrogate while it is fully off the
  /// monitor.
  #[cfg(target_os = "windows")]
  slide_in_monitor_rects: HashMap<Uuid, Rect>,
  /// Peeks drawing scrolling layouts' columns.
  #[cfg(target_os = "windows")]
  pub scroll_view: super::scroll_view::ScrollView,
  /// Windows moved asynchronously until the stored time, keyed by
  /// `HWND`, because a synchronous move of them was slow. See
  /// [`allows_sync_reposition`].
  ///
  /// [`allows_sync_reposition`]: AnimationManager::allows_sync_reposition
  #[cfg(target_os = "windows")]
  sync_demoted_until: HashMap<isize, Instant>,
  /// Windows with an active close animation, keyed by window ID.
  ///
  /// The stored value is the raw `HWND` (as `isize`) so `WM_CLOSE` can be
  /// sent after the fade finishes without borrowing the window container.
  #[cfg(target_os = "windows")]
  pending_close_windows: HashMap<Uuid, isize>,
  /// Workspace switches playing on scenes over their monitors.
  #[cfg(target_os = "windows")]
  pub scenes: super::scene_switch::SceneSwitches,
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
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      timer_thread: Mutex::new(None),
      timer_shutdown: Arc::new(AtomicBool::new(false)),
      last_frame_at: Cell::new(Instant::now()),
      stalled_timers: Arc::new(AtomicUsize::new(0)),
      pass_now: None,
      #[cfg(target_os = "windows")]
      overlay_z_settle: HashMap::new(),
      #[cfg(target_os = "windows")]
      animation_timer_vsync: Arc::new(Mutex::new(None)),
      #[cfg(target_os = "windows")]
      animation_vsync_time: Arc::new(Mutex::new(None)),
      #[cfg(target_os = "windows")]
      resize_sessions: HashMap::new(),
      #[cfg(target_os = "windows")]
      backdrop_overlay_z_order_dirty: false,
      #[cfg(target_os = "windows")]
      edge_color_cache: Arc::new(Mutex::new(HashMap::new())),
      #[cfg(target_os = "windows")]
      pending_edge_samples: HashSet::new(),
      #[cfg(target_os = "windows")]
      warm_surrogates: HashMap::new(),
      #[cfg(target_os = "windows")]
      pending_surrogate_updates: Vec::new(),
      #[cfg(target_os = "windows")]
      pending_session_cleanup: Vec::new(),
      #[cfg(target_os = "windows")]
      slide_in_monitor_rects: HashMap::new(),
      #[cfg(target_os = "windows")]
      scroll_view: super::scroll_view::ScrollView::default(),
      #[cfg(target_os = "windows")]
      sync_demoted_until: HashMap::new(),
      #[cfg(target_os = "windows")]
      pending_close_windows: HashMap::new(),
      #[cfg(target_os = "windows")]
      scenes: super::scene_switch::SceneSwitches::default(),
    }
  }

  /// Inserts or replaces the animation state for a window.
  pub fn start_animation(
    &mut self,
    window_id: Uuid,
    animation: WindowAnimationState,
  ) {
    self.animations.insert(window_id, animation);
  }

  /// Returns the current animation state for a window, if any.
  pub fn get_animation(
    &self,
    window_id: &Uuid,
  ) -> Option<&WindowAnimationState> {
    self.animations.get(window_id)
  }

  /// Returns `true` if a close animation is in flight for the given
  /// window.
  #[cfg(target_os = "windows")]
  pub fn has_close_animation(&self, window_id: &Uuid) -> bool {
    self.pending_close_windows.contains_key(window_id)
  }

  /// Returns `true` if a DWM surrogate is currently covering `window_id`.
  ///
  /// Used by backdrop-overlay tracking to skip position updates and hide
  /// the overlay while the real window is cloaked — preventing the
  /// overlay from jumping to the window's pre-positioned target rect while
  /// the surrogate is still mid-animation.
  #[cfg(target_os = "windows")]
  pub fn has_active_surrogate(&self, window_id: &Uuid) -> bool {
    self.resize_sessions.contains_key(window_id)
      || self.scroll_view.owns_window(window_id)
      || self.pending_close_windows.contains_key(window_id)
  }

  /// Returns `true` if `window_id` has a live `ResizeSession` whose
  /// overlays the per-tick driver owns.
  ///
  /// Deliberately does *not* cover `pending_session_cleanup`, the tail
  /// where a finished session's surrogate fades out over the real window.
  /// By then the real window is uncloaked at its final rect, and the
  /// overlay driver in `platform_sync` no longer runs for it (that loop
  /// walks `resize_sessions`) -- so counting the tail here left nobody
  /// driving the backdrop overlay. It kept the z-anchor it had behind a
  /// surrogate that was being destroyed, and the window rendered without
  /// its backdrop until the tail ended: measured as a ~12ms flash at the
  /// end of a move, in two runs out of three.
  #[cfg(target_os = "windows")]
  pub fn has_live_resize_tracker(&self, window_id: &Uuid) -> bool {
    self.resize_sessions.contains_key(window_id)
  }

  /// Returns `true` while an animation still owns `window_id`'s real
  /// position -- either a live `ResizeSession`, or the
  /// `pending_session_cleanup` tail that outlives it.
  ///
  /// Covers the tail because the handoff `SetWindowPos` that parks the
  /// real window at its final rect is asynchronous and cross-process:
  /// the resulting location-change notification routinely arrives
  /// *after* the session has been retired (measured ~344ms for a 250ms
  /// animation, i.e. inside the fade-out tail). Reprocessing it as an
  /// independent move rewrites a floating window's `floating_placement`
  /// to a mid-animation rect and marks it
  /// `has_custom_floating_placement`, which permanently suppresses
  /// `--centered` on every later `toggle-floating`.
  #[cfg(target_os = "windows")]
  pub fn owns_window_position(&self, window_id: &Uuid) -> bool {
    self.resize_sessions.contains_key(window_id)
      || self
        .pending_session_cleanup
        .iter()
        .any(|(id, _, _)| id == window_id)
  }

  /// Removes a window's animation and any associated resize session.
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.animations.remove(window_id);
    #[cfg(target_os = "windows")]
    self.resize_sessions.remove(window_id);
    #[cfg(target_os = "windows")]
    self.slide_in_monitor_rects.remove(window_id);
    #[cfg(target_os = "windows")]
    self.pending_close_windows.remove(window_id);
  }

  /// Removes animations that have both finished their eased progress *and*
  /// (on Windows, for a resize-session-backed animation) had their real
  /// window's final move confirmed by `ResizeSession::pre_commit`. Returns
  /// the window IDs actually removed this call.
  ///
  /// An animation whose eased progress is done but whose `pre_commit` is
  /// still waiting on the real window (see `pre_commit`'s doc comment) is
  /// left in place -- both `self.animations` and `self.resize_sessions`
  /// keep their entries, and this is retried on a later tick. This is what
  /// keeps `has_active_animations` (hence the tick timer) running and the
  /// window correctly frozen/cloaked behind its surrogate for the whole
  /// wait, without blocking this call itself.
  ///
  /// Once actually removed, a session is moved to
  /// `pending_session_cleanup` so it remains visible until after the
  /// final `platform_sync` call has repositioned the real window.
  pub fn remove_completed_animations(&mut self) -> Vec<Uuid> {
    let candidate_ids: Vec<Uuid> = self
      .animations
      .iter()
      .filter(|(_, anim)| anim.is_complete())
      .map(|(id, _)| *id)
      .collect();

    let mut completed_ids = Vec::with_capacity(candidate_ids.len());

    for id in &candidate_ids {
      #[cfg(target_os = "windows")]
      {
        let ready = self
          .resize_sessions
          .get_mut(id)
          .is_none_or(ResizeSession::pre_commit);

        if !ready {
          continue;
        }

        if let Some(session) = self.resize_sessions.remove(id) {
          self.pending_session_cleanup.push((*id, None, session));
        }
        self.slide_in_monitor_rects.remove(id);
      }

      self.animations.remove(id);
      completed_ids.push(*id);
    }

    completed_ids
  }

  /// Whether there are any active animations, scenes included.
  pub fn has_active_animations(&self) -> bool {
    if !self.animations.is_empty() {
      return true;
    }
    #[cfg(target_os = "windows")]
    if self.scenes.is_active() {
      return true;
    }
    #[cfg(target_os = "windows")]
    if self.scroll_view.is_active() {
      return true;
    }
    // Completed sessions fading out still need ticks to advance the fade.
    #[cfg(target_os = "windows")]
    if !self.pending_session_cleanup.is_empty() {
      return true;
    }
    // Anything still holding a surrogate needs ticks to be torn down, even
    // once its animation entry is gone. Without this the timer parks with
    // surrogates still up and the real windows still cloaked, which looks
    // like every window being frozen in place.
    #[cfg(target_os = "windows")]
    if !self.resize_sessions.is_empty() {
      return true;
    }
    // Settling overlays are re-checked from the tick; see
    // `settle_overlay_z_order`.
    #[cfg(target_os = "windows")]
    if !self.overlay_z_settle.is_empty() {
      return true;
    }
    false
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
  #[cfg(target_os = "windows")]
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
  #[cfg(target_os = "windows")]
  pub fn extend_overlay_z_settle(&mut self, window_id: &Uuid) -> bool {
    let Some(settle) = self.overlay_z_settle.get_mut(window_id) else {
      return false;
    };

    let was_capped = settle.until >= settle.cap;
    settle.until = (Instant::now() + OVERLAY_Z_SETTLE).min(settle.cap);
    !was_capped && settle.until >= settle.cap
  }

  /// Windows whose overlays are still settling, dropping expired ones.
  #[cfg(target_os = "windows")]
  pub fn settling_overlay_windows(&mut self) -> Vec<Uuid> {
    let now = Instant::now();
    self.overlay_z_settle.retain(|_, settle| settle.until > now);
    self.overlay_z_settle.keys().copied().collect()
  }

  /// Returns all active animation window IDs.
  pub fn active_window_ids(&self) -> Vec<Uuid> {
    self.animations.keys().copied().collect()
  }

  /// Drains all active and pending resize sessions and returns them.
  ///
  /// Used by `WmState::Drop` to commit sessions during shutdown or crash
  /// so that no window is left at an intermediate animation position.
  #[cfg(target_os = "windows")]
  pub fn drain_all_sessions(&mut self) -> Vec<ResizeSession> {
    let mut sessions: Vec<ResizeSession> =
      self.resize_sessions.drain().map(|(_, s)| s).collect();
    sessions
      .extend(self.pending_session_cleanup.drain(..).map(|(_, _, s)| s));
    self.pending_surrogate_updates.clear();
    *self
      .animation_timer_vsync
      .lock()
      .expect("animation mutex poisoned") = None;
    *self
      .animation_vsync_time
      .lock()
      .expect("animation mutex poisoned") = None;
    // On WM shutdown close-animation windows are left open — only clear
    // the tracking state without sending WM_CLOSE.
    self.pending_close_windows.clear();
    sessions
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
  ///
  /// On non-Windows, `DwmFlush` is a no-op so a fixed 60 fps sleep is
  /// used.
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

    #[cfg(target_os = "windows")]
    {
      *self
        .animation_timer_vsync
        .lock()
        .expect("animation mutex poisoned") = None;
      *self
        .animation_vsync_time
        .lock()
        .expect("animation mutex poisoned") = None;
    }

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
    #[cfg(target_os = "windows")]
    let vsync_waiter = self.animation_timer_vsync.clone();
    #[cfg(target_os = "windows")]
    let vsync_time = self.animation_vsync_time.clone();

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
        #[cfg(target_os = "windows")]
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
            // Per-monitor IDXGIOutput::WaitForVBlank during
            // workspace-switch. Clone under the lock so the
            // wait runs without holding it — cleanup can clear
            // the Arc without blocking an in-progress wait.
            // Record the wake-up time immediately after vsync fires so
            // `update_internal` can compute phase-accurate animation
            // progress.
            #[cfg(target_os = "windows")]
            let dxgi_waited = {
              let waiter = vsync_waiter
                .lock()
                .expect("animation mutex poisoned")
                .clone();
              let did_wait = waiter.is_some_and(|w| w.wait());
              if did_wait {
                *vsync_time.lock().expect("animation mutex poisoned") =
                  Some(Instant::now());
              }
              did_wait
            };
            #[cfg(not(target_os = "windows"))]
            let dxgi_waited = false;

            if !dxgi_waited {
              // DwmFlush for window move/resize animations (no
              // workspace-switch active). On non-Windows this is a no-op,
              // so a fixed 60 fps sleep paces the loop.
              wm_platform::dwm_flush();
              #[cfg(not(target_os = "windows"))]
              std::thread::sleep(std::time::Duration::from_micros(16_667));
            }

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

  /// Internal update, accessed through `WmState` to avoid double-borrow.
  #[allow(clippy::too_many_lines)]
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    state.animation_manager.last_frame_at.set(Instant::now());

    perf::begin_frame();
    let tick_scope = perf::scope(Stage::Tick);

    // A failed sync must not end the frame early: the teardown after it is
    // what removes finished surrogates and reveals the real windows. The
    // first error is returned once the frame is done.
    let mut frame_result = Ok(());

    // Workspaces whose slide has played on a scene go, and so do scenes
    // with nothing left moving.
    #[cfg(target_os = "windows")]
    super::scene_switch::tick(state);

    // Start the clock of every animation created since the last tick (see
    // `WindowAnimationState::start_clock`), one frame period before this
    // frame so the first tick already shows a frame of motion instead of
    // repeating the start position.
    state.animation_manager.begin_redraw_pass();
    let frame_now = state.animation_manager.frame_now();
    let clock_origin = frame_now
      .checked_sub(state.animation_manager.frame_period())
      .unwrap_or(frame_now);
    for animation in state.animation_manager.animations.values() {
      animation.start_clock(clock_origin);
    }

    // Moved columns that end shown get their real window back in the
    // redraw below, and focus deferred while they moved lands then. Ones
    // that end hidden have their real window put where it stays cloaked,
    // and their overlays settled.
    #[cfg(target_os = "windows")]
    let mut scroll_batch = SurrogateBatch::new();
    #[cfg(target_os = "windows")]
    let landed = state.animation_manager.scroll_view.tick(
      frame_now,
      clock_origin,
      &mut scroll_batch,
    );
    #[cfg(target_os = "windows")]
    if !landed.shown.is_empty() {
      state.pending_sync.queue_focus_change();
    }
    #[cfg(target_os = "windows")]
    for window_id in landed.shown.into_iter().chain(landed.hidden) {
      if let Some(window) = state
        .container_by_id(window_id)
        .and_then(|container| container.as_window_container().ok())
      {
        state.pending_sync.queue_container_to_redraw(window);
      }
    }
    #[cfg(target_os = "windows")]
    super::scroll_view::track_overlays(state, config, scroll_batch);

    // Queue in-progress windows for redraw.
    let active_window_ids: Vec<_> = state
      .animation_manager
      .active_window_ids()
      .into_iter()
      .filter(|id| {
        state
          .animation_manager
          .get_animation(id)
          .is_some_and(|anim| !anim.is_complete())
      })
      .collect();

    // Recorded here rather than at the end of the tick: by then completed
    // animations have already been drained, so a burst's real concurrency
    // reads back as zero.
    perf::note_window_count(active_window_ids.len());

    for window_id in &active_window_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Drive close surrogates directly. They don't reach the redraw loop
    // above: a closing window was detached from the layout tree when its
    // animation started, so `platform_sync` has nothing to position for
    // it. We replicate the same per-frame update logic used inside
    // `start_animation_if_needed` for surrogate sessions.
    #[cfg(target_os = "windows")]
    {
      let close_in_progress: Vec<Uuid> = state
        .animation_manager
        .pending_close_windows
        .keys()
        .filter(|id| {
          state
            .animation_manager
            .animations
            .get(id)
            .is_some_and(|a| !a.is_complete())
        })
        .copied()
        .collect();

      // Batches every closing window's backdrop-overlay reposition this
      // tick into one `DeferWindowPos` transaction instead of each issuing
      // its own synchronous `SetWindowPos` -- same reasoning as
      // `platform_sync`'s per-tick backdrop-overlay sync passes.
      let mut backdrop_batch = SurrogateBatch::new();
      let mut border_batch = SurrogateBatch::new();

      for id in &close_in_progress {
        let is_zoom = state
          .animation_manager
          .resize_sessions
          .get(id)
          .is_some_and(|s| s.zoom);

        // Extract values before taking a mutable borrow on
        // resize_sessions.
        let anim_data =
          state.animation_manager.animations.get(id).map(|a| {
            let (rect, opacity) = a.current_state();
            let progress = a.eased_progress();
            (rect, opacity, progress)
          });

        let Some((current_rect, opacity, progress)) = anim_data else {
          continue;
        };
        let opacity_u8 = opacity.map_or(u8::MAX, |o| o.to_alpha());

        if is_zoom {
          if let Some(session) =
            state.animation_manager.resize_sessions.get_mut(id)
          {
            session.update_zoom_fade(1.0 - progress, opacity_u8);
          }
        } else if let Some(session) =
          state.animation_manager.resize_sessions.get_mut(id)
        {
          session.update(&current_rect, opacity_u8);
        }

        // Close animations are detached from the layout tree, so they
        // never reach `platform_sync`'s per-window loop -- track
        // the backdrop and border overlays directly here instead,
        // mirroring `sync_overlays`' steady-state behavior but
        // following the surrogate's live rect.
        if let Some(session) =
          state.animation_manager.resize_sessions.get(id)
        {
          let anchor = session.surrogate_hwnd();
          let rect = session.current_rect();

          if let Some(params) = session.backdrop_overlay_params() {
            match (anchor, rect.clone()) {
              (Some(anchor), Some(rect)) => upsert_overlay(
                &mut state.backdrop_overlays,
                *id,
                params,
                &rect,
                anchor,
                &mut backdrop_batch,
              ),
              _ => {
                if let Some(overlay) = state.backdrop_overlays.get_mut(id)
                {
                  overlay.hide();
                }
              }
            }
          }

          if let Some(params) = session.border_overlay_params() {
            match (anchor, rect) {
              (Some(anchor), Some(rect)) => upsert_overlay(
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
      }

      backdrop_batch.commit();
      border_batch.commit();
    }

    // Finalize completed close animations before
    // `remove_completed_animations` so that their sessions are dropped
    // directly (not moved to `pending_session_cleanup`) and
    // `platform_sync` never attempts to reposition or uncloak these
    // windows.
    //
    // The window was already detached from the layout tree when the close
    // animation started, so `WM_CLOSE` is sent via the stored HWND rather
    // than through the container tree.
    #[cfg(target_os = "windows")]
    {
      let close_done: Vec<Uuid> = state
        .animation_manager
        .pending_close_windows
        .keys()
        .filter(|id| {
          state
            .animation_manager
            .get_animation(id)
            .is_some_and(super::state::WindowAnimationState::is_complete)
        })
        .copied()
        .collect();

      for id in close_done {
        let hwnd = state
          .animation_manager
          .pending_close_windows
          .get(&id)
          .copied();

        // Drop surrogate and animation directly — bypasses
        // `pending_session_cleanup` so `platform_sync` does not attempt to
        // reposition or uncloak the window.
        state.animation_manager.animations.remove(&id);
        state.animation_manager.resize_sessions.remove(&id);
        state.animation_manager.pending_close_windows.remove(&id);

        // The close-drive loop above populates these via
        // `upsert_overlay` since closing windows
        // are detached from the layout tree and never reach
        // `sync_overlays`'s normal retain-based cleanup. Remove them here
        // too, or the overlay window they own is never destroyed and
        // lingers on screen as a ghost.
        state.backdrop_overlays.remove(&id);
        state.border_overlays.remove(&id);

        // Reconstruct a `NativeWindow` from the stored HWND and send
        // `WM_CLOSE` to destroy the OS window. The layout was already
        // updated when the animation started, so no `unmanage_window` call
        // is needed here.
        if let Some(handle) = hwnd {
          let native = NativeWindow::from_handle(handle);
          if let Err(err) = native.close() {
            tracing::warn!(
              "Failed to send WM_CLOSE for window {id}: {err}."
            );
          }
          state.closing_windows.mark_close_sent(id, Instant::now());
        }
      }
    }

    // Remove completed animations. Their sessions are moved to
    // `pending_session_cleanup` and must outlive the `platform_sync` call
    // below so the real window is repositioned before surrogates
    // disappear.
    let completed_ids =
      state.animation_manager.remove_completed_animations();

    // Queue completed animations for a final redraw so `platform_sync`
    // moves the real window to its target position and uncloak it.
    for window_id in &completed_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Re-apply any focus change that was deferred while the now-completed
    // resize surrogates were active. `sync_focus` skips
    // `SetForegroundWindow` while a resize session is live to prevent
    // the OS from asynchronously removing the DWM cloak and triggering
    // a costly re-cloak on the next tick. Queuing here ensures the
    // focus transfer happens in the same `platform_sync` that uncloak
    // the windows.
    #[cfg(target_os = "windows")]
    if !completed_ids.is_empty() {
      state.pending_sync.queue_focus_change();
    }

    if state.pending_sync.has_changes() {
      keep_first_error(&mut frame_result, platform_sync(state, config));
    } else {
      // `platform_sync` syncs tab bars, which go with the peeks of moving
      // columns even when nothing else moved.
      #[cfg(target_os = "windows")]
      if state.animation_manager.scroll_view.is_active() {
        crate::tab_bars::sync_tab_bars(
          state,
          config,
          crate::tab_bars::Restack::None,
        );
      }
    }

    #[cfg(target_os = "windows")]
    resync_settling_overlays(state);

    // Held in an `Option` so the nested `platform_sync` below can close it
    // early: leaving it open there made `Cleanup` enclose a second
    // `PlatformSync`, double-counting that time so the two stages summed
    // to more than the enclosing `Tick`.
    // LINT: Only taken early by the Windows teardown below.
    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut cleanup_scope = Some(perf::scope(Stage::Cleanup));

    // Fade out pending sessions now that `platform_sync` has moved the
    // real windows to their final positions, then drop them. Dropping
    // a session destroys its surrogate overlay.
    #[cfg(target_os = "windows")]
    {
      // Flush before fading new surrogates or dropping workspace-switch
      // cleanup. The uncloak issued inside `platform_sync` is a DWM
      // attribute change that only takes effect at the next composition —
      // fading or destroying the surrogate without waiting for that frame
      // can produce one composition where the surrogate is dimmed/gone but
      // the real window is still cloaked: a visible blank flash at the end
      // of the animation. The flush guarantees DWM renders one frame with
      // the real window visible (including its compositor shadow, which
      // thumbnails do not capture) while the surrogate still fully covers
      // it.
      //
      // For transparent windows (`effect_opacity < 255`), the surrogate
      // opacity and the real window's `LWA_ALPHA` compose
      // multiplicatively: a surrogate at 80 % over a real window at
      // 80 % would appear ~96 % opaque in the flushed frame instead
      // of the intended 80 %. Zero out those surrogates before the
      // flush so the rendered frame already shows the correct
      // real-window + backdrop-overlay composite.
      for (_, fade_start, session) in
        &mut state.animation_manager.pending_session_cleanup
      {
        if fade_start.is_none() && session.effect_opacity < u8::MAX {
          session.fade_overlay(0);
        }
      }

      // Keep the backdrop overlay tracking each fading session's own
      // surrogate through the whole `pending_session_cleanup` tail,
      // exactly as it did through the active animation. Batched into one
      // `DeferWindowPos` transaction, same reasoning as
      // the other backdrop-overlay sync loops.
      let mut fade_tail_batch = SurrogateBatch::new();
      let mut border_fade_tail_batch = SurrogateBatch::new();
      for (id, _, session) in
        &state.animation_manager.pending_session_cleanup
      {
        // The real window, not the surrogate. A session only reaches
        // `pending_session_cleanup` after `platform_sync` has uncloaked
        // it, and it then spends `SESSION_FADE_OUT` sitting
        // beneath a surrogate fading to zero. Anchoring the
        // overlay to the surrogate through that stretch puts it
        // *between* the two, so as the surrogate fades it reveals
        // the overlay rather than the window -- a flash of
        // backdrop on every window an animation touched, clearing only
        // once the session is dropped and `sync_overlays`
        // re-anchors it.
        //
        // The active-animation loop above is a different case and
        // correctly still anchors to the surrogate: the real
        // window is cloaked there, so the surrogate genuinely is
        // what the overlay sits behind.
        let anchor =
          session.window_hwnd().or_else(|| session.surrogate_hwnd());
        let rect = session.current_rect();
        let border_rect = rect
          .as_ref()
          .map(|rect| crate::tab_bars::with_tab_bar_of(state, *id, rect));

        if let Some(params) = session.backdrop_overlay_params() {
          match (anchor, rect.clone()) {
            (Some(anchor), Some(rect)) => upsert_overlay(
              &mut state.backdrop_overlays,
              *id,
              params,
              &rect,
              anchor,
              &mut fade_tail_batch,
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
              &mut border_fade_tail_batch,
            ),
            _ => {
              if let Some(overlay) = state.border_overlays.get_mut(id) {
                overlay.hide();
              }
            }
          }
        }
      }
      fade_tail_batch.commit();
      border_fade_tail_batch.commit();

      // A session entering cleanup needs one composition between
      // `platform_sync`'s uncloak and the first fade step, otherwise the
      // frame can show a dimmed/absent surrogate over a still-cloaked
      // window -- a blank flash at the end of the animation. That used to
      // be bought with a `DwmFlush` right here, blocking the WM's single
      // thread for a whole composition every time any session finished --
      // which, during a burst of rapid resizes, is most ticks.
      //
      // The `retain_mut` below instead defers each session's first fade
      // step to the *next* tick. The tick loop is already paced by the
      // animation thread's own vblank wait, so the next tick is by
      // construction a later composition: the same guarantee, without this
      // thread waiting for it. Deferring costs nothing visually either --
      // the first step's progress is ~0, so it was a no-op fade anyway.

      // After `pre_commit` the surrogate is a pixel-aligned live mirror of
      // the uncloaked window beneath it, so ramping its opacity to zero
      // blends shadow/border/late-repaint differences instead of swapping
      // them in a single composition. Entries are dropped once fully
      // faded.
      let fade_now = Instant::now();

      // Re-anchor to the real window's own `HWND` any session whose
      // surrogate is about to be destroyed this tick: the `retain_mut`
      // below drops (and thereby destroys the
      // surrogate of) any session that's done fading or skips fading
      // outright, and without re-anchoring first the overlay is left
      // pointing at a `HWND` that no longer exists until a later
      // `platform_sync` call happens to fix it -- a one-frame flicker,
      // most visible when moving a window past a sibling whose own
      // relayout-triggered session finishes mid-gesture.
      let finishing_sessions: Vec<(Uuid, Rect)> = state
        .animation_manager
        .pending_session_cleanup
        .iter()
        .filter_map(|(id, fade_start, session)| {
          // `fade_start.is_some()` gates both arms: a session on its very
          // first cleanup tick is deferred (see the `retain_mut` below)
          // and so is not dropped yet, and re-anchoring it here
          // would fire this block's flush a tick before it is
          // needed.
          let is_finishing = fade_start.is_some_and(|start| {
            session.effect_opacity < u8::MAX
              || fade_now.saturating_duration_since(start).as_secs_f32()
                / SESSION_FADE_OUT.as_secs_f32()
                >= 1.0
          });
          is_finishing
            .then(|| session.current_rect().map(|rect| (*id, rect)))
            .flatten()
        })
        .collect();

      if !finishing_sessions.is_empty() {
        let mut finishing_batch = SurrogateBatch::new();
        let mut border_finishing_batch = SurrogateBatch::new();
        for (id, rect) in &finishing_sessions {
          let Some(container) = state.container_by_id(*id) else {
            tracing::debug!("finishing_sessions: no container for {id}.");
            continue;
          };
          let Ok(window) = container.as_window_container() else {
            tracing::debug!(
              "finishing_sessions: {id} not a window container."
            );
            continue;
          };
          let anchor = overlay_z_anchor(&window);
          if let Some(overlay) = state.backdrop_overlays.get_mut(id) {
            tracing::debug!(
              "finishing_sessions: re-anchoring {id} to {:?} at {:?}.",
              anchor,
              rect,
            );
            overlay.defer_rect(&mut finishing_batch, rect, anchor);
          } else {
            tracing::debug!(
              "finishing_sessions: no backdrop overlay for {id}."
            );
          }
          if let Some(overlay) = state.border_overlays.get_mut(id) {
            overlay.defer_rect(&mut border_finishing_batch, rect, anchor);
          }
        }
        finishing_batch.commit();
        border_finishing_batch.commit();
        wm_platform::dwm_flush();
      }

      // Windows whose fading resize-session surrogate is dropped this tick
      // -- `has_active_surrogate` now covers
      // `pending_session_cleanup`, so once dropped here the static
      // overlay needs a follow-up sync to reappear (see the same
      // reasoning below for workspace-switch cleanup).
      let mut dropped_session_ids: Vec<Uuid> = Vec::new();
      // Surrogates reclaimed from sessions dropped below, stashed into
      // `warm_surrogates` after `retain_mut` returns rather than inside
      // its closure (which only holds `&mut
      // pending_session_cleanup`, not `&mut AnimationManager`) --
      // see `stash_warm_surrogate`'s doc comment.
      let mut reclaimed_surrogates: Vec<(Uuid, NativeSurrogate)> =
        Vec::new();
      state.animation_manager.pending_session_cleanup.retain_mut(
        |(id, fade_start, session)| {
          // First tick in cleanup: `platform_sync` uncloaked the real
          // window moments ago and DWM has not composited that
          // yet. Record the start and act from the next tick --
          // the vsync-paced tick loop guarantees it is a later
          // composition -- rather than blocking this thread on a
          // `DwmFlush` to buy the same barrier. Transparent
          // sessions were already zeroed by the loop above, so the frame
          // that lands in between is the correct final composite for them
          // too.
          let Some(start) = *fade_start else {
            // The surrogate shows the window's companion, whose owner
            // puts it back on screen only once it notices the uncloak.
            if session.effect_opacity == u8::MAX
              && session.awaits_companion()
            {
              return true;
            }

            *fade_start = Some(fade_now);
            return true;
          };

          // Transparent windows were zeroed on the tick before this one,
          // so the composite the user is looking at is already
          // correct. Drop.
          if session.effect_opacity < u8::MAX {
            tracing::debug!(
              "pending_session_cleanup: dropping {id} (transparent, \
               effect_opacity={}).",
              session.effect_opacity,
            );
            if let Some(surrogate) = session.take_surrogate() {
              reclaimed_surrogates.push((*id, surrogate));
            }
            dropped_session_ids.push(*id);
            return false;
          }

          let progress =
            fade_now.saturating_duration_since(start).as_secs_f32()
              / SESSION_FADE_OUT.as_secs_f32();
          if progress >= 1.0 {
            tracing::debug!(
              "pending_session_cleanup: dropping {id} (fade complete).",
            );
            if let Some(surrogate) = session.take_surrogate() {
              reclaimed_surrogates.push((*id, surrogate));
            }
            dropped_session_ids.push(*id);
            return false;
          }
          #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
          )]
          let opacity =
            (f32::from(session.effect_opacity) * (1.0 - progress)) as u8;
          session.fade_overlay(opacity);
          true
        },
      );
      for (id, surrogate) in reclaimed_surrogates {
        state.animation_manager.stash_warm_surrogate(id, surrogate);
      }

      // Resize-session surrogates dropped above are redrawn by a follow-up
      // `platform_sync` call so their overlays settle tint/rect through
      // the normal path. Without this second sync, an overlay would
      // stay however it was left above until some unrelated redraw
      // happens to fire -- the animation timer stops right after
      // this block when no other animation is active, so there may
      // be no later tick.
      let cleanup_window_ids = dropped_session_ids;

      if !cleanup_window_ids.is_empty() {
        let mut queued = 0;
        for id in cleanup_window_ids {
          if let Some(container) = state.container_by_id(id) {
            if let Ok(window) = container.as_window_container() {
              state.pending_sync.queue_container_to_redraw(window);
              queued += 1;
            }
          }
        }
        tracing::debug!(
          "Post-cleanup overlay re-sync: queued {queued} window(s) for \
           redraw, running follow-up platform_sync."
        );
        drop(cleanup_scope.take());
        keep_first_error(&mut frame_result, platform_sync(state, config));
      }
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

      #[cfg(target_os = "windows")]
      state.animation_manager.sample_pending_edge_colors();

      state
        .animation_manager
        .animation_timer_running
        .store(false, Ordering::Relaxed);

      // All animations are done: clear the DXGI vsync waiter so a later
      // animation re-selects its own monitor and the timer reverts to
      // DwmFlush when vsync is unavailable. Safe here because no
      // animation is active.
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
      #[cfg(target_os = "windows")]
      {
        *state
          .animation_manager
          .animation_timer_vsync
          .lock()
          .expect("animation mutex poisoned") = None;
        *state
          .animation_manager
          .animation_vsync_time
          .lock()
          .expect("animation mutex poisoned") = None;
      }
    }

    frame_result
  }

  /// Returns the predicted vsync instant: last wake + frame period ×
  /// (1 - `VSYNC_LEAD_FRACTION`).
  ///
  /// Reads the period live from the installed waiter instead of a stale
  /// cached field. Returns `None` when no waiter is installed or no wake
  /// has been recorded yet.
  #[cfg(target_os = "windows")]
  fn predictive_vsync_now(&self) -> Option<Instant> {
    let guard = self.animation_timer_vsync.lock().ok()?;
    let waiter = guard.as_ref()?;
    let period_us = waiter.frame_period_us();
    let last_wake = waiter.last_wake.lock().ok()?.as_ref().copied()?;
    #[allow(
      clippy::cast_precision_loss,
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss
    )]
    let lead = Duration::from_micros(
      (period_us as f64 * f64::from(1.0_f32 - VSYNC_LEAD_FRACTION)) as u64,
    );
    Some(last_wake + lead)
  }

  /// The pacing monitor's frame period, or zero before a vsync waiter is
  /// installed.
  // LINT: The vsync waiter only exists on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(clippy::unused_self))]
  fn frame_period(&self) -> Duration {
    #[cfg(target_os = "windows")]
    if let Some(period_us) =
      self.animation_timer_vsync.lock().ok().and_then(|waiter| {
        waiter
          .as_ref()
          .map(wm_platform::DxgiVsyncWaiter::frame_period_us)
      })
    {
      return Duration::from_micros(period_us);
    }

    Duration::ZERO
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
  /// session setup (surrogate creation, fill, preposition) and the ones
  /// before it, so windows relaid out together started up to a frame
  /// apart: measured 11ms between a shrinking window and the growing
  /// neighbour started after it, which the steep start of the easing
  /// curve turned into a ~190px hole between them on the first frame.
  fn frame_now(&mut self) -> Instant {
    if let Some(now) = self.pass_now {
      return now;
    }

    #[cfg(target_os = "windows")]
    let now = self.predictive_vsync_now().unwrap_or_else(Instant::now);
    #[cfg(not(target_os = "windows"))]
    let now = Instant::now();

    self.pass_now = Some(now);
    now
  }

  /// Installs or switches the vsync waiter to the monitor with handle
  /// `monitor_handle`, so animation ticks are always paced against
  /// whichever monitor the animating window is actually on.
  ///
  /// Skips DXGI enumeration when the window is already on the installed
  /// monitor.
  #[cfg(target_os = "windows")]
  fn ensure_waiter_for(&self, monitor_handle: isize) {
    // Fast path: already pacing from this monitor.
    {
      let guard = self
        .animation_timer_vsync
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
      if guard
        .as_ref()
        .is_some_and(|w| w.monitor_handle() == monitor_handle)
      {
        return;
      }
    }

    // Always switch to the target monitor's waiter here -- the fast path
    // above already ruled out "no-op, same monitor", so by this point the
    // animating window is confirmed to be on a *different* monitor than
    // whatever's currently installed (or nothing is installed yet).
    // Always switch, even to a slower refresh rate: keeping a faster
    // waiter paces a 60Hz monitor's animations against the wrong vblank,
    // which reads as stutter specific to that monitor.
    match DxgiVsyncWaiter::for_monitor(monitor_handle) {
      Ok(new_waiter) => {
        let mut guard = self
          .animation_timer_vsync
          .lock()
          .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Slow frames are counted against the frame period of whichever
        // monitor is actually pacing the animation, so the profiler stays
        // meaningful across a 60 Hz panel and a 175 Hz one alike.
        perf::set_frame_budget(Duration::from_micros(
          new_waiter.frame_period_us(),
        ));
        tracing::debug!(
          monitor = monitor_handle,
          period_us = new_waiter.frame_period_us(),
          "vsync waiter switched"
        );
        *guard = Some(new_waiter);
      }
      Err(err) => {
        tracing::warn!(?err, "failed to create vsync waiter for monitor");
      }
    }
  }

  /// Determines whether a new animation should be started for a window.
  fn should_start_new_animation(
    &self,
    window_id: &Uuid,
    is_resize: bool,
    is_column: bool,
    target_rect: &Rect,
    previous_target: Option<&Rect>,
    config: &UserConfig,
  ) -> bool {
    let existing_animation = self.get_animation(window_id);

    let (enabled, threshold_px) = if is_column {
      let c = &config.value.animations.scroll;
      (c.enabled, c.threshold_px)
    } else if is_resize {
      let c = &config.value.animations.window_resize;
      (c.enabled, c.threshold_px)
    } else {
      let c = &config.value.animations.window_move;
      (c.enabled, c.threshold_px)
    };
    let threshold = i32::try_from(threshold_px).unwrap_or(i32::MAX);

    if enabled {
      if let Some(anim) = existing_animation {
        if anim.is_complete() {
          // Animation already at its target — treat as a static window and
          // apply the threshold check against the completed target so a
          // new animation starts if the window needs to move.
          let distance = (anim.target_rect.x() - target_rect.x()).abs()
            + (anim.target_rect.y() - target_rect.y()).abs()
            + (anim.target_rect.width() - target_rect.width()).abs()
            + (anim.target_rect.height() - target_rect.height()).abs();
          distance > threshold
        } else {
          // Redirect any in-progress animation to the new target whenever
          // the destination changes, regardless of distance.
          // Without this, small target adjustments (< threshold)
          // are silently swallowed and the window snaps after
          // the stale animation finishes.
          anim.target_rect != *target_rect
        }
      } else if let Some(prev_target) = previous_target {
        let distance = (prev_target.x() - target_rect.x()).abs()
          + (prev_target.y() - target_rect.y()).abs()
          + (prev_target.width() - target_rect.width()).abs()
          + (prev_target.height() - target_rect.height()).abs();
        distance > threshold
      } else {
        false
      }
    } else {
      false
    }
  }

  /// Determines the rect and opacity to use for a window this frame.
  ///
  /// Starts a new animation when movement or resize crosses the configured
  /// threshold.
  ///
  /// `cycle_has_resize` is `true` when any window in the same redraw cycle
  /// changes size. Pure translations then adopt the `window_resize` timing
  /// and easing so adjacent edges stay in lock-step throughout the
  /// relayout — with differing `window_move`/`window_resize` durations,
  /// a moved window would otherwise arrive early and detach from its
  /// still-resizing neighbors.
  ///
  /// A column of a scrolling layout (`is_column`) moves and resizes as
  /// `animations.scroll` says instead.
  ///
  /// Returns [`AnimationPositionResult::Frozen`] while a surrogate overlay
  /// is active so the caller does not reposition the real window on
  /// intermediate frames.
  #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
    is_resize: bool,
    cycle_has_resize: bool,
    is_column: bool,
    target_rect: Rect,
    previous_target: Option<Rect>,
    // Only used on Windows to capture the window for the surrogate.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    native_window: &NativeWindow,
    // Opacity from window-effects config; used as surrogate opacity when
    // the animation has no per-frame fade component.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    effect_opacity: u8,
    // Corner style from window-effects config; applied to the surrogate
    // so it matches the real window's rounded corners during the
    // animation.
    #[cfg(target_os = "windows")] corner_style: CornerStyle,
    // Tint/blur-amount/corner-radius/opacity/saturation for the
    // backdrop-overlay tracker, or `None` when no backdrop is configured.
    #[cfg(target_os = "windows")] backdrop_overlay: Option<
      BackdropOverlayParams,
    >,
    // Color/width/corner-radius/opacity for the border-overlay tracker,
    // or `None` when the border effect is not configured.
    #[cfg(target_os = "windows")] border_overlay: Option<
      BorderOverlayParams,
    >,
    config: &UserConfig,
  ) -> (AnimationPositionResult, Option<OpacityValue>) {
    let existing_animation = self.get_animation(&window_id).cloned();

    let should_start = self.should_start_new_animation(
      &window_id,
      is_resize,
      is_column,
      &target_rect,
      previous_target.as_ref(),
      config,
    );

    if should_start {
      perf::note_animation_start(is_resize);

      if let Some(prev_target) = previous_target {
        // Start from the current animated position on cancel-and-replace
        // so the animation does not jump back to the original
        // start.
        let now = Instant::now();
        let start_rect = existing_animation.as_ref().map_or_else(
          || prev_target.clone(),
          |a| a.current_state_at(now).0,
        );

        // Share the resize timing across the whole cycle when any window
        // in it resizes, keeping all edges in lock-step (see doc
        // comment).
        let use_resize_timing = is_resize
          || (cycle_has_resize
            && config.value.animations.window_resize.enabled);

        let (duration_ms, easing) = if is_column {
          let c = &config.value.animations.scroll;
          (c.duration_ms, c.easing.clone())
        } else if use_resize_timing {
          let c = &config.value.animations.window_resize;
          (c.duration_ms, c.easing.clone())
        } else {
          let c = &config.value.animations.window_move;
          (c.duration_ms, c.easing.clone())
        };

        // A column already moving carries on at its speed toward the new
        // target, as the strip's columns do, rather than restarting.
        let animation = match &existing_animation {
          Some(existing) if is_column && !existing.is_complete() => {
            existing.retargeted(
              target_rect.clone(),
              duration_ms,
              easing,
              now,
            )
          }
          _ => WindowAnimationState::new_movement(
            start_rect.clone(),
            target_rect.clone(),
            duration_ms,
            easing,
          ),
        };
        self.start_animation(window_id, animation);

        // Redirect an in-flight surrogate session to the new target, or
        // create a new one. The surrogate overlay is our own window and
        // moves instantly each frame; the real window only needs
        // one async move to its final position. This avoids
        // per-frame cross-process `SWP_ASYNCWINDOWPOS` calls,
        // which lag behind when the target process's message loop
        // is slow.
        #[cfg(target_os = "windows")]
        if let Some(session) = self.resize_sessions.get_mut(&window_id) {
          session.update_target(&start_rect, &target_rect);
        } else {
          let hwnd = native_window.hwnd();
          // Stretch never shows a gap, so it needs no edge color.
          let stretch = config.value.animations.window_resize.style
            == ResizeStyle::Stretch;
          let cached_edge_color = if stretch {
            None
          } else {
            self.cached_edge_color(hwnd.0)
          };
          // Reuses a still-fading or recently-warm surrogate for this same
          // window when one exists (see `reclaim_surrogate`'s doc
          // comment), skipping `CreateWindowExW`/
          // `DwmRegisterThumbnail` entirely in that case --
          // otherwise `begin_reusing_surrogate` falls through to
          // a fresh `NativeSurrogate::create` on its own.
          let begin_scope = perf::scope(Stage::SessionBegin);
          let session_result = match self.reclaim_surrogate(window_id) {
            Some(warm_surrogate) => {
              ResizeSession::begin_reusing_surrogate(
                hwnd,
                &start_rect,
                &target_rect,
                &SessionOptions {
                  effect_opacity,
                  // `false`: the backdrop overlay is positioned behind the
                  // surrogate (in `platform_sync`'s `Frozen` handling)
                  // before the surrogate is revealed via
                  // `ResizeSession::show`, so
                  // the surrogate is never visible without its overlay
                  // correctly in place behind it -- closing the race at
                  // the source instead of chasing it
                  // with a fast follow-up.
                  corner_style,
                  place_at_top: true,
                  edge_color: cached_edge_color,
                  backdrop_overlay,
                  border_overlay,
                  stretch,
                },
                warm_surrogate,
              )
            }
            None => ResizeSession::begin(
              hwnd,
              &start_rect,
              &target_rect,
              &SessionOptions {
                effect_opacity,
                corner_style,
                place_at_top: true,
                edge_color: cached_edge_color,
                backdrop_overlay,
                border_overlay,
                stretch,
              },
            ),
          };
          drop(begin_scope);

          match session_result {
            Ok(session) => {
              // Re-sampled after every session rather than once per TTL,
              // so the cached color is never more than one animation
              // behind the window it stands in for -- a five-minute-old
              // sample of an app that has since changed theme reads as a
              // wrong-coloured flash. This session plays with whatever was
              // already cached (see `cached_edge_color`); the sample is
              // for the next one, taken once animations are
              // idle (see `sample_pending_edge_colors`).
              if !stretch {
                self.pending_edge_samples.insert(hwnd.0);
              }
              // `place_at_top: true` above means the session's surrogate
              // (if any) was inserted at `HWND_TOP` -- see
              // `backdrop_overlay_z_order_dirty`'s doc comment.
              if session.surrogate_hwnd().is_some() {
                self.backdrop_overlay_z_order_dirty = true;
              }
              self.resize_sessions.insert(window_id, session);
            }
            Err(err) => {
              tracing::warn!(
                "Failed to begin resize session for window {window_id}: \
                 {err}."
              );
            }
          }
        }
        // Install or upgrade the vsync waiter to the highest-Hz monitor
        // among active sessions after both the new-session and redirect
        // paths.
        #[cfg(target_os = "windows")]
        self.ensure_waiter_for(DxgiVsyncWaiter::window_monitor(
          native_window.hwnd(),
        ));
      }
    }

    // Evaluate this frame's position at a predictive timestamp so the
    // surrogate aligns with the next DWM composition rather than lagging
    // by one pipeline delay -- the same one for every window this pass.
    let now = self.frame_now();

    // Re-fetch the animation after potentially starting a new one.
    if let Some(animation) = self.get_animation(&window_id) {
      let (current_rect, opacity) = animation.current_state_at(now);

      // Drive the surrogate overlay when one is active. `has_surrogate()`
      // requires a valid DWM thumbnail — if thumbnail registration failed
      // (e.g. elevated/UWP window), the surrogate is transparent and
      // useless: snap the window to target and clean up rather than
      // cloaking it behind an empty overlay.
      // Extract the session status with a shared borrow first, then take a
      // mutable borrow only for the drive path. Avoids a triple-lookup
      // (contains_key → get → get_mut) on the same key.
      #[cfg(target_os = "windows")]
      let session_status = self
        .resize_sessions
        .get(&window_id)
        .map(|s| (s.has_surrogate(), s.effect_opacity, s.zoom));

      #[cfg(target_os = "windows")]
      match session_status {
        Some((true, effect_opacity, zoom)) => {
          let monitor_rect =
            self.slide_in_monitor_rects.get(&window_id).cloned();
          let session = self
            .resize_sessions
            .get_mut(&window_id)
            .expect("resize session must exist after status check");
          let opacity_u8 = opacity
            .as_ref()
            .map_or(effect_opacity, wm_platform::OpacityValue::to_alpha);
          if zoom {
            // Extract progress with a separate borrow before mutably using
            // session.
            let progress = self.animations.get(&window_id).map_or(
              1.0,
              super::state::WindowAnimationState::eased_progress,
            );
            let is_close =
              self.pending_close_windows.contains_key(&window_id);
            let forward_progress =
              if is_close { 1.0 - progress } else { progress };
            let session = self
              .resize_sessions
              .get_mut(&window_id)
              .expect("resize session must exist after status check");
            session.update_zoom_fade(forward_progress, opacity_u8);
          } else if let Some(monitor_rect) = monitor_rect {
            session.update_clipped(
              &current_rect,
              &monitor_rect,
              opacity_u8,
            );
          } else {
            // Queue instead of applying immediately: all surrogate
            // repositions in this redraw pass are drained into the pass's
            // shared batch by `queue_surrogate_updates` and committed
            // atomically, so adjacent windows' edges land in the same DWM
            // composition frame.
            // A stretched surrogate shows whatever size the app has
            // reached, so the real window gets its final size right away
            // (Hyprland-style) instead of late in the animation.
            let handoff = session.is_stretch()
              || self.animations.get(&window_id).is_some_and(|a| {
                // Scale the lead with the animation duration so the
                // handoff stays near the end of the visual
                // travel regardless of easing speed. For
                // expo-out at 150 ms this fires at ~96%
                // visual progress.
                #[allow(
                  clippy::cast_possible_truncation,
                  clippy::cast_sign_loss,
                  clippy::cast_precision_loss
                )]
                let lead_ms = (a.duration.as_millis() as f32 * 0.35)
                  .clamp(50.0, HANDOFF_LEAD_MAX_MS as f32)
                  as u64;
                a.remaining_at(now) <= Duration::from_millis(lead_ms)
              });
            self.pending_surrogate_updates.push(PendingSurrogateUpdate {
              window_id,
              rect: current_rect,
              opacity: opacity_u8,
              handoff,
            });
          }
          return (AnimationPositionResult::Frozen, None);
        }
        Some((false, _, _)) => {
          // Thumbnail failed — drop the transparent surrogate and snap.
          // Goes through `remove_animation` (rather than removing
          // `resize_sessions`/`animations` by hand) so
          // `slide_in_monitor_rects` is cleared too -- otherwise
          // a later unrelated resize of the same window id would
          // read the stale monitor rect and get incorrectly
          // routed through `update_clipped`.
          self.remove_animation(&window_id);
          return (AnimationPositionResult::Apply(target_rect), None);
        }
        None => {}
      }

      (AnimationPositionResult::Apply(current_rect), opacity)
    } else {
      // No animation in the map — either the animation completed and
      // `remove_completed_animations` was already called, or animations
      // are disabled. Apply the final target rect directly.
      (AnimationPositionResult::Apply(target_rect), None)
    }
  }

  /// Returns `true` when `window_id`'s animation just completed and
  /// `pre_commit` synchronously positioned the real window at `rect`.
  ///
  /// Used by `platform_sync` on the completion redraw to skip the
  /// redundant `SetWindowPos` — its `SWP_FRAMECHANGED` would force a
  /// full frame recalculation and repaint of the window right as it is
  /// uncloaked, producing a visible flash at the end of move/resize
  /// animations.
  #[cfg(target_os = "windows")]
  pub fn was_pre_committed_at(
    &self,
    window_id: &Uuid,
    rect: &Rect,
  ) -> bool {
    self.pending_session_cleanup.iter().any(|(id, _, session)| {
      id == window_id && session.target_rect() == rect
    })
  }

  /// Re-samples the edge color of every window that animated since the
  /// last idle point.
  ///
  /// Deferred to here because the screen readback stalls DWM: run at a
  /// session's start, it more than tripled keypress-to-motion latency.
  #[cfg(target_os = "windows")]
  fn sample_pending_edge_colors(&mut self) {
    for hwnd in self.pending_edge_samples.drain() {
      sample_edge_color_async(
        HWND(hwnd),
        self.edge_color_cache.clone(),
        EDGE_COLOR_CACHE_PRUNE_LEN,
        EDGE_COLOR_CACHE_TTL,
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
  /// and undimmed. Callers re-sample on every session (see
  /// [`sample_edge_color_async`]), so this entry is never more than one
  /// animation behind the window.
  ///
  /// `sample_edge_color_async`'s background thread populates this same map
  /// directly (see `edge_color_cache`'s doc comment), so a lock failure
  /// here is treated the same as a miss rather than propagated.
  #[cfg(target_os = "windows")]
  fn cached_edge_color(&self, hwnd: isize) -> Option<Color> {
    let map = self.edge_color_cache.lock().ok()?;
    map.get(&hwnd).map(|(color, _)| *color)
  }

  /// Reclaims a surrogate for `window_id` to reuse instead of building one
  /// from scratch, checking (in order):
  ///
  /// 1. A still-fading `pending_session_cleanup` entry for the same window
  ///    -- the caller is about to start a fresh animation on it anyway, so
  ///    its fade is cancelled and its surrogate handed over directly
  ///    rather than let it finish fading out only to be torn down a moment
  ///    later.
  /// 2. The [`warm_surrogates`] cache (see its doc comment), for a window
  ///    whose previous session already finished fading.
  ///
  /// Returns `None` if neither has one, in which case the caller must
  /// create a surrogate fresh via `NativeSurrogate::create`.
  ///
  /// [`warm_surrogates`]: AnimationManager::warm_surrogates
  #[cfg(target_os = "windows")]
  fn reclaim_surrogate(
    &mut self,
    window_id: Uuid,
  ) -> Option<NativeSurrogate> {
    if let Some(idx) = self
      .pending_session_cleanup
      .iter()
      .position(|(id, _, _)| *id == window_id)
    {
      let (_, _, mut session) = self.pending_session_cleanup.remove(idx);
      if let Some(surrogate) = session.take_surrogate() {
        return Some(surrogate);
      }
    }

    let (surrogate, stashed_at) =
      self.warm_surrogates.remove(&window_id)?;
    (stashed_at.elapsed() < WARM_SURROGATE_TTL).then_some(surrogate)
  }

  /// Stashes `surrogate` as `window_id`'s warm surrogate (see
  /// [`warm_surrogates`]'s doc comment) instead of letting it drop -- and
  /// with it, destroy its overlay window and unregister its DWM thumbnail
  /// -- outright.
  ///
  /// Prunes expired entries once the cache exceeds
  /// [`WARM_SURROGATE_PRUNE_LEN`] so windows that are never resized again
  /// don't accumulate indefinitely.
  ///
  /// [`warm_surrogates`]: AnimationManager::warm_surrogates
  #[cfg(target_os = "windows")]
  fn stash_warm_surrogate(
    &mut self,
    window_id: Uuid,
    mut surrogate: NativeSurrogate,
  ) {
    // Belt-and-suspenders: the caller's fade already zeroed the
    // thumbnail's own opacity, but explicitly hide the overlay window
    // too so it can't show up in z-order queries or alt-tab while
    // sitting idle in the cache.
    surrogate.set_visible(false);

    prune_and_insert(
      &mut self.warm_surrogates,
      WARM_SURROGATE_PRUNE_LEN,
      WARM_SURROGATE_TTL,
      window_id,
      surrogate,
    );
  }

  /// Queues every surrogate update recorded during the current redraw pass
  /// into `batch`, leaving the commit to the caller.
  ///
  /// Called at the end of each redraw pass. The caller commits `batch`
  /// only after it has also queued the backdrop/border overlays that ride
  /// along with these surrogates, so window and overlays land in a single
  /// `DeferWindowPos` transaction: sequential transactions can straddle a
  /// composition boundary and let a surrogate move a frame ahead of its
  /// own border outline, or adjacent windows' edges visibly desync.
  ///
  /// Each session's `current_rect` is updated here, at queue time rather
  /// than at commit time, which is what lets the overlay pass read the
  /// live rect before the batch is committed.
  ///
  /// Real-window handoffs are capped at [`MAX_HANDOFFS_PER_TICK`] per
  /// pass; see its docs for why. Surrogate position/opacity updates are
  /// never throttled -- only the real-window handoff is deferred.
  #[cfg(target_os = "windows")]
  pub fn queue_surrogate_updates(&mut self, batch: &mut SurrogateBatch) {
    if self.pending_surrogate_updates.is_empty() {
      return;
    }

    let _scope = perf::scope(Stage::SurrogateFlush);

    // Locked only when some session is still waiting on its fill color
    // (a cache miss at `begin`), so that color shows up the tick its
    // background sample lands rather than one animation later.
    let mut edge_colors = None;

    let mut handoffs_this_tick = 0usize;
    for update in std::mem::take(&mut self.pending_surrogate_updates) {
      if let Some(session) =
        self.resize_sessions.get_mut(&update.window_id)
      {
        if session.edge_color().is_none() && !session.is_stretch() {
          let color = session.window_hwnd().and_then(|hwnd| {
            edge_colors
              .get_or_insert_with(|| self.edge_color_cache.lock().ok())
              .as_ref()?
              .get(&hwnd.0)
              .map(|(color, _)| *color)
          });
          if let Some(color) = color {
            session.set_edge_color(color);
          }
        }

        if update.handoff && handoffs_this_tick < MAX_HANDOFFS_PER_TICK {
          session.maybe_handoff();
          handoffs_this_tick += 1;
        }
        session.defer_update(batch, &update.rect, update.opacity);
      }
    }
  }

  /// Whether any surrogate exists right now, of either kind.
  ///
  /// Surrogates are created at `HWND_TOP`, which displaces *other*
  /// windows' overlays out of their z-order slot -- so while any are
  /// alive, every overlay has to re-check where it sits rather than
  /// assume it has not been moved.
  #[cfg(target_os = "windows")]
  pub fn has_any_surrogate(&self) -> bool {
    !self.resize_sessions.is_empty()
  }

  /// Whether `window_id` slides or zooms in as it opens: on its own, e.g.
  /// as a new tab of a stack that stays put.
  #[cfg(target_os = "windows")]
  pub fn is_entering(&self, window_id: &Uuid) -> bool {
    self.slide_in_monitor_rects.contains_key(window_id)
      || self
        .resize_sessions
        .get(window_id)
        .is_some_and(|session| session.zoom)
  }

  /// Where `window_id` is drawn mid move or resize animation, with
  /// `rect`, a rect of the layout that moves with the window (e.g. its
  /// stack's), moved along.
  #[cfg(target_os = "windows")]
  pub fn animation_stand_in(
    &self,
    window_id: &Uuid,
    rect: &Rect,
  ) -> Option<StandIn> {
    if self.is_entering(window_id) {
      return None;
    }

    let animation = self.animations.get(window_id)?;
    let session = self.resize_sessions.get(window_id);

    // Evaluated at the instant this pass drew the window at.
    let drawn_at = self.pass_now.unwrap_or_else(Instant::now);
    let (current, _) = animation.current_state_at(drawn_at);

    Some(StandIn {
      rect: moved_along(rect, &animation.target_rect, &current),
      clip: None,
      anchor: session.and_then(ResizeSession::surrogate_hwnd),
    })
  }

  /// Whether `native` may be moved synchronously, to stay in step with its
  /// surrogate.
  ///
  /// A synchronous `SetWindowPos` waits on the window's app, so it is
  /// skipped for an app the OS reports as hung, and for one whose last
  /// synchronous move was slow (see [`note_sync_reposition`]).
  ///
  /// [`note_sync_reposition`]: AnimationManager::note_sync_reposition
  #[cfg(target_os = "windows")]
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
  #[cfg(target_os = "windows")]
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

  /// Starts an open animation for a newly appearing window.
  ///
  /// The surrogate animates from a computed start state (determined by
  /// `window_open.direction`) to the window's final target rect. A
  /// `ResizeSession` handles all visuals; the real window remains cloaked
  /// until the animation completes.
  ///
  /// No-ops when `style` is `None` and `opacity_from` is `1.0` (nothing
  /// would visually change for the duration).
  #[cfg(target_os = "windows")]
  #[allow(clippy::too_many_arguments)]
  pub fn start_open_animation(
    &mut self,
    window_id: Uuid,
    target_rect: &Rect,
    monitor_rect: Rect,
    effect_opacity: u8,
    corner_style: CornerStyle,
    backdrop_overlay: Option<BackdropOverlayParams>,
    border_overlay: Option<BorderOverlayParams>,
    params: &WindowTransitionParams,
    native_window: &NativeWindow,
  ) {
    let is_zoom = params.style == WindowTransitionStyle::Zoom;
    let is_stationary = params.style.is_stationary();

    // Skip `None` style (no slide, no zoom) with no opacity change —
    // nothing would visually change for the duration.
    if is_stationary && !is_zoom && params.away_opacity >= 1.0 {
      return;
    }

    // Pace the open animation on the window's own monitor.
    self.ensure_waiter_for(DxgiVsyncWaiter::window_monitor(
      native_window.hwnd(),
    ));

    // Stationary styles keep the surrogate at target position; slide
    // styles offset the start one full window dimension off-screen.
    let start_rect = if is_stationary {
      target_rect.clone()
    } else {
      Self::compute_transition_start_rect(target_rect, &params.style)
    };

    let mut anim = WindowAnimationState::new_movement(
      start_rect.clone(),
      target_rect.clone(),
      params.duration_ms,
      params.easing.clone(),
    );

    // For `None`/fade style only: hold at progress 0.0 so the app can
    // paint before the surrogate reveals it. At progress 0.0 the
    // surrogate sits at the window's target rect with `start_opacity`,
    // so showing it too early would flash a black (unpainted)
    // rectangle at the window's position.
    //
    // Slide and zoom surrogates are invisible at progress 0.0 (off-screen
    // and zero-size respectively), so the grace period only adds a
    // blank gap for those styles — omit it so the animation starts
    // immediately and the blank between cloak and first visible
    // surrogate pixel is minimised.
    if is_stationary && !is_zoom {
      anim.start_delay = OPEN_PAINT_GRACE;
    }

    // Zoom open does NOT auto-fade — the surrogate is fully opaque so the
    // small thumbnail is immediately visible as it grows. Fade-in while
    // zooming makes the initial frames invisible (opacity=0 + tiny
    // size = nothing to see), which is why it felt unsmooth. Users can
    // still set opacity_from explicitly to combine fade with zoom.
    let effective_opacity_from = params.away_opacity;

    if effective_opacity_from < 1.0 {
      let effect_frac = f32::from(effect_opacity) / 255.0;
      let start_frac =
        effective_opacity_from.clamp(0.0, 1.0) * effect_frac;
      anim.start_opacity = Some(OpacityValue(start_frac));
      anim.target_opacity = Some(OpacityValue(effect_frac));
    }

    // Cloak zoom windows immediately so the real window never appears at
    // full size before the surrogate takes over. Non-zoom styles are
    // cloaked later in the Frozen branch of platform_sync (on the
    // first frame).
    if is_zoom {
      let _ = native_window.set_cloaked(true);
    }

    match ResizeSession::begin(
      native_window.hwnd(),
      &start_rect,
      target_rect,
      &SessionOptions {
        effect_opacity,
        corner_style,
        place_at_top: true,
        // No cache for open animations: the window is new, so no prior
        // sample exists.
        edge_color: None,
        backdrop_overlay,
        border_overlay,
        stretch: false,
      },
    ) {
      Ok(mut session) => {
        session.zoom = is_zoom;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let initial_opacity_u8 = (effective_opacity_from.clamp(0.0, 1.0)
          * f32::from(effect_opacity))
        .round() as u8;
        if effective_opacity_from < 1.0 {
          session.update(&start_rect, initial_opacity_u8);
        }
        // For zoom: the drive loop handles the first frame.
        // update_zoom_fade is NOT called here so the surrogate stays
        // hidden until the first animation tick sets the correct progress.
        // `place_at_top: true` above means the session's surrogate (if
        // any) was inserted at `HWND_TOP` -- see
        // `backdrop_overlay_z_order_dirty`'s doc comment.
        if session.surrogate_hwnd().is_some() {
          self.backdrop_overlay_z_order_dirty = true;
        }
        self.animations.insert(window_id, anim);
        self.resize_sessions.insert(window_id, session);
        if !is_stationary {
          self.slide_in_monitor_rects.insert(window_id, monitor_rect);
        }
      }
      Err(err) => {
        // Undo early cloak so the window doesn't disappear permanently.
        if is_zoom {
          let _ = native_window.set_cloaked(false);
        }
        tracing::warn!(
          "Failed to begin open animation for {window_id}: {err}."
        );
      }
    }
  }

  /// Starts a close animation for a window still on screen, unless one is
  /// already running.
  ///
  /// The surrogate is created and shown at once as a pixel-identical
  /// overlay of the window; the caller cloaks the real window only after
  /// this returns, so the surrogate already covers it and no gap exposes
  /// the desktop. The surrogate stays put and fades or zooms out, or
  /// slides off the screen edge `params.style` names; the real window
  /// never moves.
  ///
  /// On completion, `update_internal` sends the window `WM_CLOSE`.
  #[cfg(target_os = "windows")]
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn start_close_animation(
    &mut self,
    window_id: Uuid,
    current_rect: &Rect,
    effect_opacity: u8,
    corner_style: CornerStyle,
    backdrop_overlay: Option<BackdropOverlayParams>,
    border_overlay: Option<BorderOverlayParams>,
    params: &WindowTransitionParams,
    native_window: &NativeWindow,
  ) {
    if self.pending_close_windows.contains_key(&window_id) {
      return;
    }

    let is_zoom = params.style == WindowTransitionStyle::Zoom;
    let is_stationary = params.style.is_stationary();

    // Skip stationary style (no slide, no zoom) with no opacity change —
    // nothing would visually change for the duration.
    if is_stationary && !is_zoom && params.away_opacity >= 1.0 {
      return;
    }

    // Pace the animation on the window's own monitor.
    self.ensure_waiter_for(DxgiVsyncWaiter::window_monitor(
      native_window.hwnd(),
    ));

    // For slide-out, the surrogate travels from current_rect to an
    // off-screen target.
    let target_rect = if is_stationary {
      current_rect.clone()
    } else {
      Self::compute_transition_start_rect(current_rect, &params.style)
    };

    let mut anim = WindowAnimationState::new_movement(
      current_rect.clone(),
      target_rect.clone(),
      params.duration_ms,
      params.easing.clone(),
    );

    if params.away_opacity < 1.0 {
      let effect_frac = f32::from(effect_opacity) / 255.0;
      let target_frac = params.away_opacity.clamp(0.0, 1.0) * effect_frac;
      anim.start_opacity = Some(OpacityValue(effect_frac));
      anim.target_opacity = Some(OpacityValue(target_frac));
    }

    match ResizeSession::begin(
      native_window.hwnd(),
      current_rect,
      &target_rect,
      &SessionOptions {
        effect_opacity,
        corner_style,
        place_at_top: false,
        // Reuse a cached color when the window was recently animated;
        // otherwise sample — the window is still on screen.
        edge_color: self.cached_edge_color(native_window.hwnd().0),
        backdrop_overlay,
        border_overlay,
        stretch: false,
      },
    ) {
      Ok(mut session) => {
        // Cover the window before the caller cloaks it.
        session.show();
        session.zoom = is_zoom;
        self.animations.insert(window_id, anim);
        self.resize_sessions.insert(window_id, session);
        self
          .pending_close_windows
          .insert(window_id, native_window.hwnd().0);
      }
      Err(err) => {
        tracing::warn!(
          "Failed to begin close animation for {window_id}: {err}."
        );
      }
    }
  }

  /// Computes the off-screen rect for a slide open/close transition.
  ///
  /// For open (`start_open_animation`): returns the start rect positioned
  /// off-screen, one full window dimension outside the target edge. The
  /// surrogate slides from this rect to `base`.
  ///
  /// For close (`start_close_animation`): returns the off-screen target
  /// rect so the surrogate slides from `base` (the window's current
  /// position) to off-screen.
  ///
  /// `SlideRight` → exits/enters from the right edge;
  /// `SlideLeft` → left edge; `SlideTop` → top; `SlideBottom` → bottom.
  #[cfg(target_os = "windows")]
  fn compute_transition_start_rect(
    base: &Rect,
    style: &WindowTransitionStyle,
  ) -> Rect {
    let w = base.width();
    let h = base.height();

    let (x, y) = match style {
      WindowTransitionStyle::SlideRight => (base.x() + w, base.y()),
      WindowTransitionStyle::SlideLeft => (base.x() - w, base.y()),
      WindowTransitionStyle::SlideTop => (base.x(), base.y() - h),
      WindowTransitionStyle::SlideBottom => (base.x(), base.y() + h),
      // Stationary styles never call this function.
      _ => (base.x(), base.y()),
    };

    Rect::from_xy(x, y, w, h)
  }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
  use super::*;

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
}
