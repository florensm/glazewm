//! The window overview's runtime: a thread of its own running a
//! full-screen window per monitor, with every card, window, highlight and
//! hint on them composited by DWM.
//!
//! Each picture is drawn once into a cloaked layered window, and shown on
//! the overview as a DWM thumbnail of that window; window previews are
//! thumbnails too. An animation frame therefore draws nothing: it only
//! moves and scales thumbnails. Selection and hover are highlights moved
//! over the cards rather than drawn into them, so going through the
//! workspaces redraws nothing either.
//!
//! One `Interaction` spans every monitor's screen, so the keyboard,
//! searches and dragged windows go from one screen to the next; each
//! `Screen` draws its own part of it.

mod cards;
mod ghost;
mod overlays;
mod scenes;
mod screen;

use std::{
  cell::RefCell,
  sync::{mpsc, OnceLock},
  thread,
  time::{Duration, Instant},
};

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
    UI::{
      Controls::WM_MOUSELEAVE,
      Input::KeyboardAndMouse::{
        ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE,
        TRACKMOUSEEVENT, VIRTUAL_KEY, VK_0, VK_9, VK_BACK, VK_DOWN,
        VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_NUMPAD0, VK_NUMPAD9,
        VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
      },
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
        GetCursorPos, LoadCursorW, PeekMessageW, PostMessageW,
        PostQuitMessage, SetCursor, SetForegroundWindow, TranslateMessage,
        WaitMessage, HTCLIENT, HWND_MESSAGE, IDC_ARROW, IDC_HAND,
        IDC_SIZEALL, MSG, PM_REMOVE, SC_KEYMENU, WA_INACTIVE,
        WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATE, WM_APP, WM_CHAR,
        WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP,
        WM_MBUTTONDOWN, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_QUIT,
        WM_SETCURSOR, WM_SYSCOMMAND, WS_EX_NOACTIVATE,
        WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
        WS_POPUP,
      },
    },
  },
};

use self::{ghost::Ghost, overlays::Overlays};
use crate::{
  overview_chrome::{picture_rect, CardPicture, SearchControls},
  overview_layout::{
    Camera, CardMetrics, CardView, GridCells, Placement, RectF, Spring,
    Zoom,
  },
  overview_scene::Scene,
  overview_state::{
    all_workspaces, Hit, Interaction, Key, ScreenFocus, SearchField,
    ShortcutGroup,
  },
  overview_tab_bars::CardTabBar,
  overview_thumbnails::{Picture, Preview, Thumbnail},
  platform_impl::{self, composition::OverviewBackdrop},
  window_class::{self, mouse_position},
  window_icons, DxgiVsyncWaiter, OverviewAction, OverviewFrame,
  OverviewLayoutMode, OverviewScreen, OverviewStyle, OverviewWorkspace,
  Rect,
};

/// Posted with a `Box<Open>` in `WPARAM`.
const WM_OPEN_OVERVIEW: u32 = WM_APP + 1;

/// Posted with a `Box<OverviewFrame>` in `WPARAM` to change what an open
/// overview shows.
const WM_UPDATE_OVERVIEW: u32 = WM_APP + 2;

/// Posted with the requested layout's index in
/// `OverviewLayoutMode::ALL` in `WPARAM`.
const WM_TOGGLE_OVERVIEW: u32 = WM_APP + 3;

/// Posted to close the overview.
const WM_HIDE_OVERVIEW: u32 = WM_APP + 4;

/// Posted by the icon thread once an icon is cached.
const WM_ICON_READY: u32 = WM_APP + 5;

/// How long the zoom into a pick waits for the WM to carry it out before
/// starting regardless, a safety net for a pick it never answers.
///
/// Generous on purpose: switching workspaces moves every window of both,
/// and some apps answer a move slowly. A zoom that sets off sooner heads
/// for where the windows were, then veers off once the WM's answer says
/// where they landed.
const PICK_TIMEOUT: Duration = Duration::from_millis(400);

/// What the WM thread posts to open the overview.
struct Open {
  session: u64,
  frame: OverviewFrame,
  layout: OverviewLayoutMode,
}

type ActionCallback = Box<dyn Fn(u64, OverviewAction) + Send + 'static>;

/// The WM's handle to the overview, which lives on a thread of its own.
///
/// The WM thread only ever posts to it, so neither can stall the other.
pub struct NativeOverview {
  /// The overview thread's message-only window, which the WM posts to.
  hwnd: isize,

  /// Frame last posted, to skip posting identical ones. `None` while
  /// hidden.
  last_frame: Option<OverviewFrame>,

  /// Incremented on each open, so actions from an earlier session can be
  /// told apart.
  session: u64,
}

impl NativeOverview {
  /// Starts the overview's thread, with its windows hidden. `on_action` is
  /// called on that thread for each user action, with the session it
  /// happened in.
  pub fn create(on_action: ActionCallback) -> crate::Result<Self> {
    let (ready_tx, ready_rx) = mpsc::channel();

    thread::Builder::new()
      .name("overview".to_string())
      .spawn(move || run(&ready_tx, on_action))
      .map_err(|err| crate::Error::Platform(err.to_string()))?;

    let hwnd = ready_rx
      .recv_timeout(Duration::from_secs(5))
      .map_err(|err| crate::Error::Platform(err.to_string()))??;

    Ok(Self {
      hwnd,
      last_frame: None,
      session: 0,
    })
  }

  /// Opens the overview on `frame`, laid out as `layout`, and takes
  /// keyboard focus on the focused monitor's screen.
  ///
  /// Must be called on the WM thread, not the event loop thread.
  pub fn open(
    &mut self,
    frame: OverviewFrame,
    layout: OverviewLayoutMode,
  ) {
    self.session += 1;
    self.last_frame = Some(frame.clone());

    let open = Box::new(Open {
      session: self.session,
      frame,
      layout,
    });
    self.post(WM_OPEN_OVERVIEW, Box::into_raw(open) as usize);
  }

  /// Shows `frame` in the open overview.
  pub fn update(&mut self, frame: OverviewFrame) {
    if self.last_frame.is_none()
      || self.last_frame.as_ref() == Some(&frame)
    {
      return;
    }

    self.last_frame = Some(frame.clone());
    self.post(WM_UPDATE_OVERVIEW, Box::into_raw(Box::new(frame)) as usize);
  }

  /// Switches the open overview to `layout`, or cancels it if it already
  /// shows that.
  pub fn toggle(&self, layout: OverviewLayoutMode) {
    let index = OverviewLayoutMode::ALL
      .iter()
      .position(|other| *other == layout)
      .unwrap_or(0);
    self.post(WM_TOGGLE_OVERVIEW, index);
  }

  /// Closes the overview once its zoom into the picked workspace ends.
  ///
  /// `frame` is shown first if it scrolled a workspace's strip (e.g. to
  /// bring the picked window into view), so the zoom lands on the columns
  /// now on screen.
  pub fn hide_on(&mut self, frame: OverviewFrame) {
    let is_scrolled = self.last_frame.as_ref().is_some_and(|last| {
      is_strip_scrolled(&last.screens, &frame.screens)
    });

    if is_scrolled {
      self.update(frame);
    }
    self.hide();
  }

  /// Closes the overview.
  pub fn hide(&mut self) {
    if self.last_frame.take().is_some() {
      self.post(WM_HIDE_OVERVIEW, 0);
    }
  }

  /// Session of the last open, which the actions of that open carry.
  #[must_use]
  pub fn session(&self) -> u64 {
    self.session
  }

  fn post(&self, msg: u32, wparam: usize) {
    // SAFETY: A boxed payload passes to the window procedure, which frees
    // it. If the post fails the window is gone and it leaks.
    unsafe {
      let _ =
        PostMessageW(HWND(self.hwnd), msg, WPARAM(wparam), LPARAM(0));
    }
  }
}

/// Whether a workspace on a screen in both `old` and `new` shows other
/// content.
fn is_strip_scrolled(
  old: &[OverviewScreen],
  new: &[OverviewScreen],
) -> bool {
  new.iter().any(|screen| {
    let Some(old) = old.iter().find(|old| old.id == screen.id) else {
      return false;
    };

    screen.workspaces.iter().any(|workspace| {
      old.workspaces.iter().any(|other| {
        other.name == workspace.name
          && other.content_rect != workspace.content_rect
      })
    })
  })
}

impl Drop for NativeOverview {
  fn drop(&mut self) {
    self.post(WM_CLOSE, 0);
  }
}

thread_local! {
  /// The overview, owned by its thread's message-only window.
  static OVERVIEW: RefCell<Option<Overview>> = const { RefCell::new(None) };
}

/// Runs the overview's thread: creates its message-only window, then pumps
/// messages, stepping animations once per display refresh while any are
/// running.
fn run(
  ready: &mpsc::Sender<crate::Result<isize>>,
  on_action: ActionCallback,
) {
  // SAFETY: Called once at the start of this thread, which then owns
  // windows and so wants a single-threaded apartment.
  unsafe {
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
  }

  let control = match create_control_window() {
    Ok(control) => control,
    Err(err) => {
      let _ = ready.send(Err(err));
      return;
    }
  };

  OVERVIEW.with(|cell| {
    *cell.borrow_mut() = Some(Overview::new(control, on_action));
  });
  let _ = ready.send(Ok(control.0));

  let mut msg = MSG::default();
  loop {
    // SAFETY: `msg` outlives each call and is only read after a message
    // was retrieved into it.
    unsafe {
      while PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE).as_bool() {
        if msg.message == WM_QUIT {
          return;
        }
        TranslateMessage(&raw const msg);
        DispatchMessageW(&raw const msg);
      }
    }

    let vsync = with_overview(|overview| {
      overview.tick().then(|| overview.vsync.clone())
    })
    .flatten();

    match vsync {
      Some(waiter) => wait_for_frame(waiter.as_ref()),
      // SAFETY: No preconditions.
      None => unsafe {
        let _ = WaitMessage();
      },
    }
  }
}

/// Waits for the next display refresh, so each animation step lands in a
/// frame of its own.
fn wait_for_frame(waiter: Option<&DxgiVsyncWaiter>) {
  let started = Instant::now();

  if !waiter.is_some_and(DxgiVsyncWaiter::wait) {
    crate::dwm_flush();
  }

  // Neither wait is guaranteed to block (e.g. with nothing composited),
  // and a busy loop would starve DWM itself.
  if started.elapsed() < Duration::from_millis(2) {
    thread::sleep(Duration::from_millis(4));
  }
}

/// Runs `f` on the overview, unless it is already borrowed further up the
/// stack.
fn with_overview<T>(f: impl FnOnce(&mut Overview) -> T) -> Option<T> {
  OVERVIEW.with(|cell| {
    let mut overview = cell.try_borrow_mut().ok()?;
    overview.as_mut().map(f)
  })
}

/// The window the WM posts to, which owns the overview.
fn create_control_window() -> crate::Result<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_OverviewControl"),
    control_proc,
  );

  // SAFETY: The class is registered above.
  let hwnd = unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE::default(),
      w!("GlazeWM_OverviewControl"),
      w!(""),
      WINDOW_STYLE::default(),
      0,
      0,
      0,
      0,
      HWND_MESSAGE,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create the overview's control window.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// A screen's full-screen window, hidden.
fn create_screen_window() -> crate::Result<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_Overview"),
    screen_proc,
  );

  // `WS_EX_NOREDIRECTIONBITMAP`: the backdrop is a composition visual tree
  // rooted on the window, with the thumbnails drawn above it.
  // `WS_EX_NOACTIVATE`: clicking a screen leaves the foreground, and so
  // the keyboard, on the one the overview opened with; each still takes
  // the foreground when asked to.
  //
  // SAFETY: The class is registered above.
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_TOPMOST
        | WS_EX_TOOLWINDOW
        | WS_EX_NOREDIRECTIONBITMAP
        | WS_EX_NOACTIVATE,
      w!("GlazeWM_Overview"),
      w!(""),
      WS_POPUP,
      0,
      0,
      0,
      0,
      None,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create overview window.".to_string(),
    ));
  }

  Ok(hwnd)
}

/// A workspace's card.
struct Card {
  cx: Spring,
  cy: Spring,
  scale: Spring,
  alpha: Spring,

  /// Its size, wider than normal for a wide workspace.
  metrics: CardMetrics,

  /// How its workspace maps onto it.
  view: CardView,

  previews: Vec<Preview>,

  /// Its stacks' tab bars, each under its stack's window.
  tab_bars: Vec<CardTabBar>,

  picture: Picture,

  /// What `picture` shows, to skip redrawing it unchanged.
  drawn: Option<CardPicture>,
}

impl Card {
  /// Settles within about `settle_ms` from now on.
  fn retune(&mut self, settle_ms: f32) {
    for spring in
      [&mut self.cx, &mut self.cy, &mut self.scale, &mut self.alpha]
    {
      spring.retune(settle_ms);
    }
  }

  /// Its pictures, unregistered, for reuse.
  fn into_pictures(self) -> impl Iterator<Item = Picture> {
    std::iter::once(self.picture)
      .chain(self.tab_bars.into_iter().map(|bar| bar.picture))
      .map(|mut picture| {
        picture.thumbnail = None;
        picture
      })
  }

  fn placement(&self) -> Placement {
    Placement {
      cx: self.cx.value,
      cy: self.cy.value,
      scale: self.scale.value,
      opacity: self.alpha.value,
    }
  }
}

/// What placing a card depends on besides the card itself.
struct PlaceContext {
  camera: Camera,
  /// How far the overview is open.
  fade: f32,
  /// The screen's own rect.
  view: RectF,
  /// Card the screen zooms out of or into.
  zoom_card: Option<usize>,
}

/// Places card `index`'s thumbnails where its animation has it now.
fn place_card(card: &mut Card, index: usize, context: &PlaceContext) {
  let PlaceContext {
    camera,
    fade,
    view,
    zoom_card,
  } = context;
  let placement = card.placement();
  let metrics = card.metrics;
  let rect = camera.apply(&placement.rect(&metrics));
  let is_visible = rect.intersect(view).is_some();

  // Only the zoomed card has windows on screen to zoom from or into. It
  // shows at full strength while it does, so its windows can't dim in or
  // out of their real ones.
  let (openness, alpha) = if *zoom_card == Some(index) {
    (
      *fade,
      placement.opacity + (1.0 - placement.opacity) * (1.0 - fade),
    )
  } else {
    (1.0, placement.opacity)
  };

  for preview in &mut card.previews {
    let dest = camera.apply(&placement.map(&metrics, &preview.tile.rect));
    if is_visible {
      preview.place(&dest, view, preview.zoomed_opacity(openness) * alpha);
    } else {
      preview.hide();
    }
  }

  for bar in &mut card.tab_bars {
    // Faded with its stack's shown window.
    let opacity = card
      .previews
      .iter()
      .find(|preview| preview.hwnd == bar.frame.anchor)
      .map_or(1.0, |preview| preview.zoomed_opacity(openness));
    let dest = camera.apply(&placement.map(&metrics, &bar.rect));

    if is_visible {
      bar.picture.place(&dest, view, opacity * alpha);
    } else if let Some(thumbnail) = &mut bar.picture.thumbnail {
      thumbnail.hide();
    }
  }

  if is_visible {
    let picture = camera.apply(&picture_rect(&metrics, &placement));
    card.picture.place(&picture, view, fade * alpha);
  } else if let Some(thumbnail) = &mut card.picture.thumbnail {
    thumbnail.hide();
  }
}

/// What a screen shows: its monitor's part of the frame, with what every
/// screen shares.
struct ScreenFrame {
  /// Its monitor's ID.
  id: u128,

  /// Working area of the monitor.
  rect: Rect,

  scale_factor: f32,
  workspaces: Vec<OverviewWorkspace>,
  focused_window: Option<isize>,
  style: OverviewStyle,
}

impl ScreenFrame {
  fn new(screen: &OverviewScreen, frame: &OverviewFrame) -> Self {
    Self {
      id: screen.id,
      rect: screen.rect.clone(),
      scale_factor: screen.scale_factor,
      workspaces: screen.workspaces.clone(),
      focused_window: frame.focused_window,
      style: frame.style.clone(),
    }
  }
}

/// One monitor's overview: a full-screen window of its own, showing the
/// monitor's workspaces.
struct Screen {
  hwnd: HWND,

  /// What is shown, or `None` while hidden.
  frame: Option<ScreenFrame>,

  /// The interaction as this screen shows it.
  focus: ScreenFocus,

  metrics: CardMetrics,

  /// Layout the cards are drawn for.
  drawn_for: OverviewLayoutMode,

  /// Layout whose thumbnails are registered, if any are.
  registered_for: Option<OverviewLayoutMode>,

  /// ID of the last registration of its thumbnails, unique across
  /// screens. Each registration leaves them above anything registered on
  /// it before.
  registration: u64,

  /// Working area covered, in screen coordinates.
  area: RectF,

  backdrop: Option<OverviewBackdrop>,

  cards: Vec<Card>,

  /// Each workspace's windows shown on their own, in the activities.
  scenes: Vec<Scene>,

  /// Where the activities show the selected workspace large.
  scene_area: RectF,

  /// Pictures of cards no longer shown, kept for reuse.
  spare: Vec<Picture>,

  /// `None` if their pictures couldn't be created.
  overlays: Option<Overlays>,

  /// The search line, what it was drawn as (with the shortcuts chip
  /// ringed or not), and where its controls are in it.
  search: Option<Picture>,
  search_drawn: Option<(SearchField, bool)>,
  search_controls: Option<SearchControls>,
  search_y: Spring,

  /// The sheet of keyboard shortcuts, what it lists while open, and how
  /// far it is faded in.
  sheet: Option<Picture>,
  sheet_drawn: Option<Vec<ShortcutGroup>>,
  sheet_alpha: Spring,

  /// Where the special workspace's card is docked, from the last layout.
  dock: Option<RectF>,

  /// Thumbnails replaced by a new registration, kept up until the next
  /// placement puts their replacements where they are.
  retired: Vec<Thumbnail>,

  /// Where the grid puts each card, for moving a row up or down.
  grid: GridCells,

  /// Where the shown workspace's windows show once settled, first to
  /// last, for the arrows to go between them.
  nav: Vec<(isize, RectF)>,

  /// The zoom out on open or into a card on close, and when it started,
  /// while it runs.
  zoom: Option<(Zoom, Instant)>,

  /// Whether something changed that the next tick has to place.
  is_dirty: bool,

  /// Whether a highlight was still gliding when last placed.
  is_gliding: bool,

  is_tracking_leave: bool,

  /// Bumped as window icons arrive.
  icons: u64,

  /// Where its pictures say an icon arrived.
  notify: (HWND, u32),
}

impl Drop for Screen {
  fn drop(&mut self) {
    // Its thumbnails and backdrop go before the window they are on.
    self.unregister_thumbnails();
    self.backdrop = None;

    // SAFETY: Created in `Screen::new` on this thread and destroyed once.
    unsafe {
      let _ = DestroyWindow(self.hwnd);
    }
  }
}

/// The overview, owned by its thread: a screen per monitor, and what the
/// user does across them.
struct Overview {
  /// The message-only window the WM posts to.
  control: HWND,

  on_action: ActionCallback,
  session: u64,

  /// What is shown, or `None` while hidden.
  frame: Option<OverviewFrame>,

  /// Every screen's workspaces, as the interaction indexes them.
  workspaces: Vec<OverviewWorkspace>,

  interaction: Interaction,

  /// A screen per monitor of the frame, in its order. Kept hidden while
  /// closed, for the next open.
  screens: Vec<Screen>,

  ghost: Option<Ghost>,

  /// Cursor position at open, in screen coordinates. Windows sends a
  /// mouse move when a window appears under a still cursor, which must
  /// not count as hovering.
  open_cursor: (i32, i32),

  last_tick: Instant,

  /// Set once a pick or cancel is sent, after which input is ignored
  /// until the WM closes the overview.
  is_done: bool,

  /// A pick sent to the WM, and when, whose zoom waits for the WM to have
  /// carried it out: until then, where its windows land isn't known, e.g.
  /// while a strip scrolls the picked window into view.
  pending_pick: Option<(OverviewAction, Instant)>,

  /// Paces animation frames to the focused monitor.
  vsync: Option<DxgiVsyncWaiter>,
}

impl Overview {
  fn new(control: HWND, on_action: ActionCallback) -> Self {
    Self {
      control,
      on_action,
      session: 0,
      frame: None,
      workspaces: Vec::new(),
      interaction: Interaction::new(
        &[],
        OverviewLayoutMode::default(),
        0.0,
      ),
      screens: Vec::new(),
      ghost: None,
      open_cursor: (0, 0),
      last_tick: Instant::now(),
      is_done: false,
      pending_pick: None,
      vsync: None,
    }
  }

  /// Shows `frame` as session `session` and takes the foreground.
  fn open(
    &mut self,
    session: u64,
    frame: OverviewFrame,
    layout: OverviewLayoutMode,
  ) {
    self.close();
    self.session = session;
    self.is_done = false;

    let focused = frame
      .screens
      .iter()
      .position(|screen| screen.is_focused)
      .unwrap_or(0);
    let scale_factor = frame
      .screens
      .get(focused)
      .map_or(1.0, |screen| screen.scale_factor);

    self.workspaces = all_workspaces(&frame.screens);
    self.interaction =
      Interaction::new(&frame.screens, layout, 6.0 * scale_factor);
    self
      .interaction
      .digit_workspaces
      .clone_from(&frame.digit_workspaces);

    if let Err(err) = self.ensure_screens(frame.screens.len()) {
      tracing::warn!("Failed to open the overview: {err}");
      self.send(OverviewAction::Cancel);
      (self.on_action)(self.session, OverviewAction::Hidden);
      return;
    }

    window_icons::forget_closed();

    let animate = frame.style.open_duration_ms > 0;
    for (index, screen) in self.screens.iter_mut().enumerate() {
      let focus = self.interaction.focus(index, &self.workspaces);
      screen.open(
        ScreenFrame::new(&frame.screens[index], &frame),
        focus,
        animate,
      );
    }
    self.frame = Some(frame);
    self.after_layout();
    self.last_tick = Instant::now();
    self.open_cursor = cursor_position();

    let Some(foreground) = self.screens.get(focused).map(|s| s.hwnd)
    else {
      return;
    };

    // Asked for right before it is taken: input reaching another window in
    // between, such as releasing the keys that opened the overview, takes
    // the permission away again.
    platform_impl::send_foreground_input();

    // SAFETY: `foreground` is one of the overview's windows, on this
    // thread.
    if !unsafe { SetForegroundWindow(foreground) }.as_bool() {
      // Left open, it would look like it has the keyboard while keys go on
      // to the window behind it.
      tracing::warn!(
        "Overview could not take the foreground; closing it."
      );
      self.send(OverviewAction::Cancel);
      self.close();
      return;
    }

    let monitor = DxgiVsyncWaiter::window_monitor(foreground);
    self.vsync = DxgiVsyncWaiter::for_monitor(monitor).ok();
  }

  /// Shows a new `frame` in the open overview, keeping the selection.
  ///
  /// Screens follow their monitors: one that came gets a screen of its
  /// own, shown at once, and one that went takes its screen with it.
  fn update(&mut self, frame: OverviewFrame) {
    if self.frame.is_none() {
      return;
    }

    let old = std::mem::replace(
      &mut self.workspaces,
      all_workspaces(&frame.screens),
    );
    self.interaction.sync(&old, &frame.screens);
    self
      .interaction
      .digit_workspaces
      .clone_from(&frame.digit_workspaces);

    let mut old_screens = std::mem::take(&mut self.screens);
    for screen in &frame.screens {
      let position = old_screens
        .iter()
        .position(|old| old.id() == Some(screen.id));

      match position.map(|position| old_screens.swap_remove(position)) {
        Some(screen) => self.screens.push(screen),
        None => match Screen::new((self.control, WM_ICON_READY)) {
          Ok(screen) => self.screens.push(screen),
          Err(err) => {
            tracing::warn!("Failed to add an overview screen: {err}");
            self.close();
            self.send(OverviewAction::Cancel);
            return;
          }
        },
      }
    }

    for (index, screen) in self.screens.iter_mut().enumerate() {
      let focus = self.interaction.focus(index, &self.workspaces);
      let screen_frame = ScreenFrame::new(&frame.screens[index], &frame);

      if screen.frame.is_some() {
        screen.update(screen_frame, focus);
      } else {
        screen.open(screen_frame, focus, false);
      }
    }

    if let (Some(old), Some(ghost)) = (&self.frame, &mut self.ghost) {
      if old.style != frame.style {
        ghost.forget_drawn();
      }
    }
    self.frame = Some(frame);
    self.after_layout();
  }

  /// Hides the overview, keeping its screens for the next open, and tells
  /// the WM it did.
  fn close(&mut self) {
    if self.frame.take().is_none() {
      return;
    }

    for screen in &mut self.screens {
      screen.close();
    }

    // SAFETY: No preconditions.
    unsafe {
      let _ = ReleaseCapture();
    }

    self.ghost = None;
    self.pending_pick = None;
    self.vsync = None;
    (self.on_action)(self.session, OverviewAction::Hidden);
  }

  /// Keeps a screen per monitor: as many as `count`.
  fn ensure_screens(&mut self, count: usize) -> crate::Result<()> {
    self.screens.truncate(count);
    while self.screens.len() < count {
      self
        .screens
        .push(Screen::new((self.control, WM_ICON_READY))?);
    }
    Ok(())
  }

  /// Tells the interaction what the screens laid out, and moves the
  /// floating window along.
  fn after_layout(&mut self) {
    for (cursor, screen) in
      self.interaction.screens.iter_mut().zip(&self.screens)
    {
      cursor.grid.clone_from(&screen.grid);
    }

    if let Some(screen) = self.screens.get(self.interaction.active) {
      self.interaction.nav.clone_from(&screen.nav);
    }

    self.sync_ghost();
  }

  /// Shows on each screen what the interaction has it show now.
  fn refocus(&mut self) {
    for (index, screen) in self.screens.iter_mut().enumerate() {
      screen.focus = self.interaction.focus(index, &self.workspaces);
    }
  }

  /// Creates, moves or drops the floating window's picture.
  fn sync_ghost(&mut self) {
    let floating = self.interaction.floating();
    let host = self.ghost_host();

    if self.ghost.as_ref().map(|ghost| ghost.hwnd) != floating {
      self.ghost = floating
        .zip(host)
        .and_then(|(hwnd, host)| Ghost::new(hwnd, &self.screens[host]));
    }

    let (Some(ghost), Some(host)) = (&mut self.ghost, host) else {
      return;
    };
    let screen = &self.screens[host];
    ghost.host_on(screen);

    match self.interaction.drag {
      Some(drag) => ghost.follow(drag.position, screen),
      None => ghost.park(screen),
    }

    let title = self.workspaces.iter().find_map(|workspace| {
      workspace
        .windows
        .iter()
        .find(|window| window.hwnd == ghost.hwnd)
        .map(|window| window.title.clone())
    });
    if let (Some(title), Some(style)) = (title, screen.style()) {
      ghost.draw(&title, screen.metrics.scale_factor, style);
    }
  }

  /// Screen the floating window shows on: the one it is dragged over, or
  /// else the one with the keyboard.
  fn ghost_host(&self) -> Option<usize> {
    let dragged_over = self
      .interaction
      .drag
      .and_then(|drag| self.locate(drag.position))
      .map(|(screen, _)| screen);

    dragged_over
      .or(Some(self.interaction.active))
      .filter(|screen| *screen < self.screens.len())
  }

  /// Whether a screen is zooming into the picked workspace.
  fn is_closing(&self) -> bool {
    self.screens.iter().any(Screen::is_closing)
  }

  /// Zooms each screen into what it closes on for `action`: the picked
  /// workspace on the screen it was picked on, and the displayed one on
  /// the others. Once the zooms end, the overview hides, by when the WM
  /// has those windows in place behind it.
  ///
  /// Without a close duration, or a card to zoom into on the screen of the
  /// pick, the overview stays up until the WM hides it once focus has
  /// moved on.
  fn start_closing(&mut self, action: &OverviewAction) {
    let duration = self
      .frame
      .as_ref()
      .map_or(0, |frame| frame.style.close_duration_ms);
    if duration == 0 {
      return;
    }

    let cards = (0..self.screens.len())
      .map(|index| self.closing_card(index, action))
      .collect::<Vec<_>>();

    // The scenes zoom into the picked workspace's own windows, so they
    // show it first.
    for (index, card) in cards.iter().enumerate() {
      let start =
        self.interaction.screens.get(index).map(|s| s.cards.start);
      if let (Some(card), Some(start)) = (card, start) {
        self.interaction.show(start + card, &self.workspaces);
      }
    }

    for (index, (screen, card)) in
      self.screens.iter_mut().zip(cards).enumerate()
    {
      let focus = self.interaction.focus(index, &self.workspaces);
      match card {
        Some(card) => screen.start_closing(card, focus, duration),
        None => screen.focus = focus,
      }
    }
  }

  /// Card screen `index` zooms into when the overview closes for
  /// `action`.
  fn closing_card(
    &self,
    index: usize,
    action: &OverviewAction,
  ) -> Option<usize> {
    let screen = self.screens.get(index)?;
    let id = screen.id()?;
    let workspaces = screen.workspaces();

    match action {
      OverviewAction::FocusWindow { hwnd, monitor } if *monitor == id => {
        workspaces.iter().position(|workspace| {
          workspace.windows.iter().any(|window| window.hwnd == *hwnd)
        })
      }
      OverviewAction::FocusWorkspace { name, monitor }
        if *monitor == id =>
      {
        workspaces
          .iter()
          .position(|workspace| workspace.name == *name)
      }
      OverviewAction::FocusWindow { .. }
      | OverviewAction::FocusWorkspace { .. }
      | OverviewAction::Cancel => workspaces
        .iter()
        .position(|workspace| workspace.is_displayed),
      _ => None,
    }
  }

  /// Steps animations and places everything. Returns whether anything is
  /// still moving.
  fn tick(&mut self) -> bool {
    if self.frame.is_none() {
      return false;
    }

    let now = Instant::now();
    let dt = now
      .duration_since(self.last_tick)
      .as_secs_f32()
      .min(1.0 / 30.0);
    self.last_tick = now;

    // The WM hasn't carried out the pick in time: zoom anyway, so the
    // overview still closes.
    let overdue = self
      .pending_pick
      .take_if(|(_, picked)| picked.elapsed() >= PICK_TIMEOUT);
    if let Some((action, _)) = overdue {
      self.start_closing(&action);
      self.after_input(None);
    }

    let was_closing = self.is_closing();
    let mut is_moving = self.pending_pick.is_some();
    for screen in &mut self.screens {
      is_moving |= screen.tick(dt);
    }

    if was_closing
      && self
        .screens
        .iter()
        .all(|screen| !screen.is_closing() || screen.is_zoomed_in())
    {
      self.close();
      return false;
    }

    let host = self.ghost_host();
    if let (Some(ghost), Some(host)) = (&mut self.ghost, host) {
      is_moving |= ghost.step(dt);
      ghost.place(&self.screens[host]);
    }

    is_moving
  }

  /// Screen whose window is `hwnd`.
  fn screen_of(&self, hwnd: HWND) -> Option<usize> {
    self.screens.iter().position(|screen| screen.hwnd == hwnd)
  }

  /// The screen at screen point `point`, and where on it.
  fn locate(&self, (x, y): (f32, f32)) -> Option<(usize, (f32, f32))> {
    self
      .screens
      .iter()
      .enumerate()
      .find(|(_, screen)| {
        screen.frame.is_some() && screen.area.contains(x, y)
      })
      .map(|(index, screen)| {
        (index, (x - screen.area.x, y - screen.area.y))
      })
  }

  /// What is under screen point `point`, as the interaction indexes it,
  /// and on which screen.
  fn hit(&self, point: (f32, f32)) -> (Option<usize>, Hit) {
    let Some((index, (x, y))) = self.locate(point) else {
      return (None, Hit::default());
    };

    let hit = self.screens[index].hit(x, y);
    let start = self
      .interaction
      .screens
      .get(index)
      .map_or(0, |screen| screen.cards.start);

    (
      Some(index),
      Hit {
        workspace: hit.workspace.map(|workspace| start + workspace),
        ..hit
      },
    )
  }

  /// Screen point of `position` in window `hwnd`'s client area.
  #[allow(clippy::cast_precision_loss)]
  fn to_screen(
    &self,
    hwnd: HWND,
    (x, y): (i32, i32),
  ) -> Option<(f32, f32)> {
    let screen = &self.screens[self.screen_of(hwnd)?];
    Some((screen.area.x + x as f32, screen.area.y + y as f32))
  }

  /// Acts on what the user did, after the interaction state took it in.
  fn after_input(&mut self, action: Option<OverviewAction>) {
    if let Some(action) = action {
      let is_final = matches!(
        action,
        OverviewAction::FocusWindow { .. }
          | OverviewAction::FocusWorkspace { .. }
          | OverviewAction::Cancel
      );

      if is_final && !self.is_done {
        match action {
          // Nothing moves for a cancel.
          OverviewAction::Cancel => self.start_closing(&action),
          _ => self.pending_pick = Some((action.clone(), Instant::now())),
        }
      }
      self.send(action);
      self.is_done |= is_final;
    }

    self.refocus();
    for screen in &mut self.screens {
      screen.after_input();
    }
    self.after_layout();
  }

  fn send(&self, action: OverviewAction) {
    if !self.is_done {
      (self.on_action)(self.session, action);
    }
  }

  fn on_key(&mut self, key: Key) {
    let action = self.interaction.key(key, &self.workspaces);
    self.after_input(action);
  }

  fn on_toggle(&mut self, layout: OverviewLayoutMode) {
    if self.interaction.layout == layout {
      self.after_input(Some(OverviewAction::Cancel));
    } else {
      self.interaction.set_layout(layout, &self.workspaces);
      self.after_input(None);
    }
  }

  fn on_mouse_move(
    &mut self,
    hwnd: HWND,
    position: (i32, i32),
    is_left_down: bool,
  ) {
    let Some(point) = self.to_screen(hwnd, position) else {
      return;
    };

    #[allow(clippy::cast_possible_truncation)]
    let rounded = (point.0.round() as i32, point.1.round() as i32);
    if rounded == self.open_cursor {
      return;
    }
    self.open_cursor = (i32::MIN, i32::MIN);

    if let Some(index) = self.screen_of(hwnd) {
      self.track_leave(index);
    }

    let (screen, hit) = self.hit(point);
    // The keyboard goes where the mouse is.
    let is_activated = screen.is_some_and(|screen| {
      self.interaction.activate(screen, &self.workspaces)
    });
    let was_dragging = self.interaction.drag.is_some();

    if self.interaction.mouse_move(point, hit, is_left_down)
      || is_activated
    {
      self.after_input(None);
    } else if was_dragging {
      self.sync_ghost();
    }
  }

  /// Asks for `WM_MOUSELEAVE` once the cursor leaves screen `index`.
  fn track_leave(&mut self, index: usize) {
    let screen = &mut self.screens[index];
    if screen.is_tracking_leave {
      return;
    }

    let mut track = TRACKMOUSEEVENT {
      cbSize: u32::try_from(std::mem::size_of::<TRACKMOUSEEVENT>())
        .unwrap_or_default(),
      dwFlags: TME_LEAVE,
      hwndTrack: screen.hwnd,
      dwHoverTime: 0,
    };
    // SAFETY: `track` outlives the call.
    screen.is_tracking_leave =
      unsafe { TrackMouseEvent(&raw mut track) }.is_ok();
  }

  fn on_mouse_leave(&mut self, hwnd: HWND) {
    let Some(index) = self.screen_of(hwnd) else {
      return;
    };
    self.screens[index].is_tracking_leave = false;

    // The cursor may already be over another screen.
    let cards =
      self.interaction.screens.get(index).map(|s| s.cards.clone());
    let is_hovering_here = self
      .interaction
      .hover
      .zip(cards)
      .is_some_and(|(hover, cards)| cards.contains(&hover));

    if is_hovering_here {
      self.interaction.mouse_leave();
      self.after_input(None);
    }
  }

  fn on_left_down(&mut self, hwnd: HWND, position: (i32, i32)) {
    let Some(point) = self.to_screen(hwnd, position) else {
      return;
    };
    let (_, hit) = self.hit(point);
    self.interaction.mouse_down(point, hit);

    // SAFETY: `hwnd` is one of the overview's windows, on this thread.
    unsafe {
      SetCapture(hwnd);
    }
  }

  fn on_left_up(&mut self) {
    // SAFETY: No preconditions.
    unsafe {
      let _ = ReleaseCapture();
    }

    let action = self.interaction.mouse_up(&self.workspaces);
    self.after_input(action);
  }

  fn on_middle_down(&mut self, hwnd: HWND, position: (i32, i32)) {
    let Some(point) = self.to_screen(hwnd, position) else {
      return;
    };
    let (_, hit) = self.hit(point);
    let action = self.interaction.middle_click(hit);
    self.after_input(action);
  }

  /// Steps through the workspaces of the screen at screen point `point`.
  #[allow(clippy::cast_precision_loss)]
  fn on_wheel(&mut self, (x, y): (i32, i32), is_down: bool) {
    if let Some((screen, _)) = self.locate((x as f32, y as f32)) {
      self.interaction.activate(screen, &self.workspaces);
    }

    self.interaction.wheel(is_down, &self.workspaces);
    self.after_input(None);
  }

  fn cursor(&self) -> windows::core::PCWSTR {
    if self.interaction.drag.is_some() {
      IDC_SIZEALL
    } else if self.interaction.hover.is_some()
      || self.interaction.hover_control.is_some()
    {
      IDC_HAND
    } else {
      IDC_ARROW
    }
  }

  /// Handles a message to the control window or a screen's window.
  /// Returns `None` for the default handling.
  fn handle(
    &mut self,
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> Option<LRESULT> {
    let is_open = self.frame.is_some() && !self.is_done;

    match msg {
      WM_OPEN_OVERVIEW => {
        // SAFETY: Posted by `NativeOverview::open` with an owned payload.
        let open = unsafe { Box::from_raw(wparam.0 as *mut Open) };
        self.open(open.session, open.frame, open.layout);
      }
      WM_UPDATE_OVERVIEW => {
        // SAFETY: Posted by `NativeOverview::update` with an owned frame.
        let frame =
          unsafe { Box::from_raw(wparam.0 as *mut OverviewFrame) };
        self.update(*frame);
      }
      WM_TOGGLE_OVERVIEW if is_open => self.on_toggle(
        OverviewLayoutMode::ALL
          .get(wparam.0)
          .copied()
          .unwrap_or_default(),
      ),
      WM_HIDE_OVERVIEW => {
        // The WM carried out the pick, and the frame before this shows
        // where its windows landed, so the zoom goes there.
        if let Some((action, _)) = self.pending_pick.take() {
          self.start_closing(&action);
          self.after_input(None);
        }

        // A closing overview hides itself once its zoom ends.
        if !self.is_closing() {
          self.close();
        }
      }
      WM_ICON_READY => {
        for screen in &mut self.screens {
          screen.icons += 1;
          screen.refresh_pictures(false);
        }
      }
      WM_ACTIVATE => {
        // The low word of `wparam` is the activation state, and `lparam`
        // the window being activated. After a pick, the window it zooms
        // into has just been focused.
        let is_to_own = self.screen_of(HWND(lparam.0)).is_some();

        if (wparam.0 & 0xffff) == WA_INACTIVE as usize
          && !is_to_own
          && self.frame.is_some()
          && !self.is_closing()
          && self.pending_pick.is_none()
        {
          let was_done = self.is_done;
          self.close();
          if !was_done {
            self.send(OverviewAction::Deactivated);
          }
        }

        // Gives the window keyboard focus on activation.
        return None;
      }
      // Releasing the Alt key of the binding that opened the overview
      // would otherwise enter menu mode, swallowing the next key
      // press.
      WM_SYSCOMMAND if (wparam.0 & 0xfff0) == SC_KEYMENU as usize => {}
      WM_KEYDOWN if is_open => {
        #[allow(clippy::cast_possible_truncation)]
        if let Some(key) = to_key(VIRTUAL_KEY(wparam.0 as u16)) {
          self.on_key(key);
        }
      }
      WM_CHAR if is_open => {
        let text = u32::try_from(wparam.0).ok().and_then(char::from_u32);
        if let Some(text) = text.filter(|c| !c.is_control() && *c != ' ') {
          self.on_key(Key::Text(text));
        }
      }
      WM_MOUSEMOVE if is_open => {
        // `MK_LBUTTON` in `wparam`.
        self.on_mouse_move(
          hwnd,
          mouse_position(lparam),
          wparam.0 & 1 != 0,
        );
      }
      WM_MOUSELEAVE => {
        if is_open {
          self.on_mouse_leave(hwnd);
        } else if let Some(index) = self.screen_of(hwnd) {
          self.screens[index].is_tracking_leave = false;
        }
      }
      WM_LBUTTONDOWN if is_open => {
        self.on_left_down(hwnd, mouse_position(lparam));
      }
      WM_LBUTTONUP if is_open => self.on_left_up(),
      WM_MBUTTONDOWN if is_open => {
        self.on_middle_down(hwnd, mouse_position(lparam));
      }
      WM_MOUSEWHEEL if is_open => {
        // The high word of `wparam` is the signed wheel delta, and
        // `lparam` the cursor in screen coordinates.
        #[allow(clippy::cast_possible_truncation)]
        let delta = ((wparam.0 >> 16) as u16).cast_signed();
        self.on_wheel(mouse_position(lparam), delta < 0);
      }
      WM_SETCURSOR if u32::try_from(lparam.0 & 0xffff) == Ok(HTCLIENT) => {
        // SAFETY: The cursors are system cursors.
        unsafe {
          if let Ok(cursor) = LoadCursorW(None, self.cursor()) {
            SetCursor(cursor);
          }
        }
        return Some(LRESULT(1));
      }
      _ => return None,
    }

    Some(LRESULT(0))
  }
}

/// The overview's key for a key press, if it acts on it.
fn to_key(key: VIRTUAL_KEY) -> Option<Key> {
  let digit =
    |base: VIRTUAL_KEY| u8::try_from(key.0 - base.0).ok().map(Key::Digit);

  match key {
    VK_LEFT => Some(Key::Left),
    VK_RIGHT => Some(Key::Right),
    VK_UP => Some(Key::Up),
    VK_DOWN => Some(Key::Down),
    VK_HOME => Some(Key::Home),
    VK_END => Some(Key::End),
    VK_RETURN => Some(Key::Enter),
    VK_SPACE => Some(Key::Space),
    VK_BACK => Some(Key::Backspace),
    VK_TAB => Some(Key::Tab),
    VK_ESCAPE => Some(Key::Escape),
    _ if (VK_0.0..=VK_9.0).contains(&key.0) => digit(VK_0),
    _ if (VK_NUMPAD0.0..=VK_NUMPAD9.0).contains(&key.0) => {
      digit(VK_NUMPAD0)
    }
    _ => None,
  }
}

/// Milliseconds since `started`.
fn elapsed_ms(started: Instant) -> f32 {
  started.elapsed().as_secs_f32() * 1000.0
}

/// A duration of the style, as springs and glides take it.
#[allow(clippy::cast_precision_loss)]
fn to_ms(duration_ms: u32) -> f32 {
  duration_ms as f32
}

#[allow(clippy::cast_precision_loss)]
fn to_f32_size((width, height): (i32, i32)) -> (f32, f32) {
  (width as f32, height as f32)
}

/// Cursor position in screen coordinates.
fn cursor_position() -> (i32, i32) {
  let mut point = POINT::default();

  // SAFETY: `point` outlives the call.
  unsafe {
    let _ = GetCursorPos(&raw mut point);
  }

  (point.x, point.y)
}

/// Window procedure of the control window.
unsafe extern "system" fn control_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_CLOSE => {
      let _ = DestroyWindow(hwnd);
      return LRESULT(0);
    }
    WM_DESTROY => {
      // Dropped while no longer borrowed: the screens' windows, pictures
      // and thumbnails go with it.
      OVERVIEW.with(|cell| {
        if let Ok(mut overview) = cell.try_borrow_mut() {
          overview.take();
        }
      });
      PostQuitMessage(0);
      return LRESULT(0);
    }
    _ => {}
  }

  with_overview(|overview| overview.handle(hwnd, msg, wparam, lparam))
    .flatten()
    .unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}

/// Window procedure of a screen's window.
unsafe extern "system" fn screen_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if window_class::is_close_request(msg, wparam) {
    return LRESULT(0);
  }

  // Showing, hiding and activating the window send it messages while the
  // overview is borrowed; those get the default handling.
  with_overview(|overview| overview.handle(hwnd, msg, wparam, lparam))
    .flatten()
    .unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}

#[cfg(test)]
mod tests {
  use super::is_strip_scrolled;
  use crate::{OverviewScreen, OverviewWorkspace, Rect};

  fn workspace(name: &str, content_rect: Rect) -> OverviewWorkspace {
    OverviewWorkspace {
      name: name.to_string(),
      label: name.to_string(),
      content_rect,
      ..OverviewWorkspace::default()
    }
  }

  fn screen(
    id: u128,
    workspaces: Vec<OverviewWorkspace>,
  ) -> OverviewScreen {
    OverviewScreen {
      id,
      workspaces,
      ..OverviewScreen::default()
    }
  }

  #[test]
  fn only_a_moved_strip_counts_as_scrolled() {
    let area = Rect::from_xy(0, 0, 1920, 1040);
    let strip = |x| Rect::from_xy(x, 0, 3840, 1040);
    let old = [screen(
      1,
      vec![workspace("1", area.clone()), workspace("2", strip(0))],
    )];

    let scrolled = [screen(
      1,
      vec![workspace("1", area.clone()), workspace("2", strip(-960))],
    )];
    assert!(is_strip_scrolled(&old, &scrolled));

    let added = [screen(
      1,
      vec![
        workspace("1", area.clone()),
        workspace("2", strip(0)),
        workspace("3", area.clone()),
      ],
    )];
    assert!(!is_strip_scrolled(&old, &added));

    // The special workspace's card shows on each screen, offset to it.
    let elsewhere =
      [old[0].clone(), screen(2, vec![workspace("2", strip(1920))])];
    assert!(!is_strip_scrolled(&old, &elsewhere));
  }
}
