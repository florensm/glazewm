//! The window overview's runtime: a thread of its own running a
//! full-screen window, with every card, window, highlight and hint on it
//! composited by DWM.
//!
//! Each picture is drawn once into a cloaked layered window, and shown on
//! the overview as a DWM thumbnail of that window; window previews are
//! thumbnails too. An animation frame therefore draws nothing: it only
//! moves and scales thumbnails. Selection and hover are highlights moved
//! over the cards rather than drawn into them, so going through the
//! workspaces redraws nothing either.

mod cards;
mod overlays;
mod scenes;

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
    Graphics::Gdi::ScreenToClient,
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
        PostQuitMessage, SetCursor, SetForegroundWindow, SetWindowPos,
        ShowWindow, TranslateMessage, WaitMessage, HTCLIENT, HWND_TOPMOST,
        IDC_ARROW, IDC_HAND, IDC_SIZEALL, MSG, PM_REMOVE, SC_KEYMENU,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
        WA_INACTIVE, WM_ACTIVATE, WM_APP, WM_CHAR, WM_CLOSE, WM_DESTROY,
        WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
        WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_QUIT, WM_SETCURSOR, WM_SYSCOMMAND,
        WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
        WS_POPUP,
      },
    },
  },
};

use self::{overlays::Overlays, scenes::scene_window_rect};
use crate::{
  overview_chrome::{
    draw_card, draw_ghost, draw_hint, picture_rect, picture_size,
    with_alpha, CardPicture, GHOST_SIZE,
  },
  overview_layout::{
    activities, carousel, cover_crop, grid, stage as stage_layout, Camera,
    CardMetrics, CardView, Placement, RectF, Spring, Tile, Zoom,
    HINT_HEIGHT,
  },
  overview_scene::Scene,
  overview_state::{HintTone, Hit, Interaction, Key, Mode},
  overview_tab_bars::CardTabBar,
  overview_thumbnails::{
    visible_frame, Picture, Preview, Thumbnail, FALLBACK_FRAME,
  },
  platform_impl::{self, composition::OverviewBackdrop},
  tab_layout::TabHit,
  window_class::{self, mouse_position},
  window_icons, DxgiVsyncWaiter, OverviewAction, OverviewFrame,
  OverviewLayoutMode, OverviewStyle, OverviewWorkspace,
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

/// How long card moves and zooms take to settle.
const SETTLE_MS: f32 = 260.0;

/// Room between the activities' scene panel and what it shows, in
/// logical pixels.
const SCENE_PADDING: f32 = 14.0;

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
  /// The overview's window, owned by its thread.
  hwnd: isize,

  /// Frame last posted, to skip posting identical ones. `None` while
  /// hidden.
  last_frame: Option<OverviewFrame>,

  /// Incremented on each open, so actions from an earlier session can be
  /// told apart.
  session: u64,
}

impl NativeOverview {
  /// Starts the overview's thread, with its window hidden. `on_action` is
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
  /// keyboard focus.
  ///
  /// Must be called on the WM thread, not the event loop thread.
  pub fn open(
    &mut self,
    frame: OverviewFrame,
    layout: OverviewLayoutMode,
  ) {
    self.session += 1;
    self.last_frame = Some(frame.clone());

    // Lets the overview take the foreground once it is shown.
    platform_impl::send_foreground_input();

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
      is_strip_scrolled(&last.workspaces, &frame.workspaces)
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

/// Whether a workspace in both `old` and `new` shows other content.
fn is_strip_scrolled(
  old: &[OverviewWorkspace],
  new: &[OverviewWorkspace],
) -> bool {
  new.iter().any(|workspace| {
    old.iter().any(|other| {
      other.name == workspace.name
        && other.content_rect != workspace.content_rect
    })
  })
}

impl Drop for NativeOverview {
  fn drop(&mut self) {
    self.post(WM_CLOSE, 0);
  }
}

thread_local! {
  /// The overview, owned by its thread's window.
  static OVERVIEW: RefCell<Option<Overview>> = const { RefCell::new(None) };
}

/// Runs the overview's thread: creates its window, then pumps messages,
/// stepping animations once per display refresh while any are running.
fn run(
  ready: &mpsc::Sender<crate::Result<isize>>,
  on_action: ActionCallback,
) {
  // SAFETY: Called once at the start of this thread, which then owns
  // windows and so wants a single-threaded apartment.
  unsafe {
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
  }

  let hwnd = match create_window() {
    Ok(hwnd) => hwnd,
    Err(err) => {
      let _ = ready.send(Err(err));
      return;
    }
  };

  OVERVIEW.with(|cell| {
    *cell.borrow_mut() = Some(Overview::new(hwnd, on_action));
  });
  let _ = ready.send(Ok(hwnd.0));

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

fn create_window() -> crate::Result<HWND> {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  window_class::ensure_class_registered(
    &REGISTERED,
    w!("GlazeWM_Overview"),
    wnd_proc,
  );

  // `WS_EX_NOREDIRECTIONBITMAP`: the backdrop is a composition visual tree
  // rooted on the window, with the thumbnails drawn above it.
  //
  // SAFETY: The class is registered above.
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOREDIRECTIONBITMAP,
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
  /// The overview's own rect.
  view: RectF,
  /// Card the overview zooms out of or into.
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
    } else if let Some(thumbnail) = &mut preview.thumbnail {
      thumbnail.hide();
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

/// The window being dragged or carried.
struct Ghost {
  hwnd: isize,
  x: Spring,
  y: Spring,
  preview: Preview,
  picture: Picture,
}

/// The overview, owned by its thread.
#[allow(clippy::struct_excessive_bools)]
struct Overview {
  hwnd: HWND,
  on_action: ActionCallback,
  session: u64,

  /// What is shown, or `None` while hidden.
  frame: Option<OverviewFrame>,

  interaction: Interaction,
  metrics: CardMetrics,

  /// Layout the cards are drawn for.
  drawn_for: OverviewLayoutMode,

  /// Layout whose thumbnails are registered, if any are.
  registered_for: Option<OverviewLayoutMode>,

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

  hint: Option<Picture>,
  hint_drawn: Option<(String, HintTone)>,
  hint_y: Spring,

  /// Where the special workspace's card is docked, from the last layout.
  dock: Option<RectF>,
  ghost: Option<Ghost>,

  /// The zoom out on open or into a card on close, and when it started,
  /// while it runs.
  zoom: Option<(Zoom, Instant)>,

  last_tick: Instant,

  /// Whether something changed that the next tick has to place.
  is_dirty: bool,

  /// Whether a highlight was still gliding when last placed.
  is_gliding: bool,

  /// Cursor position at open, in client coordinates. Windows sends a
  /// mouse move when a window appears under a still cursor, which must
  /// not count as hovering.
  open_cursor: (i32, i32),
  is_tracking_leave: bool,

  /// Set once a pick or cancel is sent, after which input is ignored
  /// until the WM closes the overview.
  is_done: bool,

  /// Bumped as window icons arrive.
  icons: u64,

  /// Paces animation frames to the overview's monitor.
  vsync: Option<DxgiVsyncWaiter>,
}

impl Overview {
  fn new(hwnd: HWND, on_action: ActionCallback) -> Self {
    let layout = OverviewLayoutMode::default();

    Self {
      hwnd,
      on_action,
      session: 0,
      frame: None,
      interaction: Interaction::new(&[], layout, 0.0),
      metrics: CardMetrics::new(1.0, 1.0, 1.0, layout),
      drawn_for: layout,
      registered_for: None,
      area: RectF::default(),
      backdrop: None,
      cards: Vec::new(),
      scenes: Vec::new(),
      scene_area: RectF::default(),
      spare: Vec::new(),
      overlays: Overlays::new(),
      hint: None,
      hint_drawn: None,
      hint_y: Spring::new(0.0, SETTLE_MS, 0.25),
      dock: None,
      ghost: None,
      zoom: None,
      last_tick: Instant::now(),
      is_dirty: false,
      is_gliding: false,
      open_cursor: (0, 0),
      is_tracking_leave: false,
      is_done: false,
      icons: 0,
      vsync: None,
    }
  }

  fn workspaces(&self) -> &[OverviewWorkspace] {
    self
      .frame
      .as_ref()
      .map_or(&[], |frame| frame.workspaces.as_slice())
  }

  fn style(&self) -> Option<&OverviewStyle> {
    self.frame.as_ref().map(|frame| &frame.style)
  }

  /// Shows `frame` as session `session` and takes the foreground.
  fn open(
    &mut self,
    session: u64,
    frame: OverviewFrame,
    layout: OverviewLayoutMode,
  ) {
    self.close();

    let scale_factor = frame.scale_factor;
    self.area = RectF::from_rect(&frame.rect);
    self.drawn_for = layout;
    self.metrics =
      CardMetrics::new(self.area.w, self.area.h, scale_factor, layout);
    self.interaction =
      Interaction::new(&frame.workspaces, layout, 6.0 * scale_factor);
    self
      .interaction
      .digit_workspaces
      .clone_from(&frame.digit_workspaces);
    self.session = session;
    self.is_done = false;

    let open_duration = frame.style.open_duration_ms;
    let animate = open_duration > 0;
    self.sync_backdrop(&frame, if animate { 0.0 } else { 1.0 });

    window_icons::forget_closed();

    self.frame = Some(frame);
    self.rebuild_cards();
    self.rebuild_scenes();
    self.ensure_labels();
    if let Some(overlays) = &mut self.overlays {
      overlays.reset();
    }

    self.zoom = animate.then(|| {
      (
        Zoom::open(self.interaction.selected, open_duration),
        Instant::now(),
      )
    });

    self.retarget(true);
    self.register_thumbnails();
    self.refresh_pictures(true);
    self.last_tick = Instant::now();
    self.is_gliding = self.place();

    let rect = self.area.to_rect();
    // SAFETY: `self.hwnd` is the overview's window, on this thread.
    unsafe {
      if let Err(err) = SetWindowPos(
        self.hwnd,
        HWND_TOPMOST,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
      ) {
        tracing::warn!("Failed to show the overview: {err}");
      }

      self.open_cursor = cursor_in_client(self.hwnd);

      if !SetForegroundWindow(self.hwnd).as_bool() {
        tracing::warn!("Overview could not take the foreground.");
      }
    }

    let monitor = DxgiVsyncWaiter::window_monitor(self.hwnd);
    self.vsync = DxgiVsyncWaiter::for_monitor(monitor).ok();
    self.is_dirty = true;
  }

  /// Shows a new `frame` in the open overview, keeping the selection.
  ///
  /// Thumbnails are only registered anew when windows came, went or
  /// moved between workspaces; a title changing redraws its pictures and
  /// nothing else.
  #[allow(clippy::float_cmp)]
  fn update(&mut self, frame: OverviewFrame) {
    let Some(old) = self.frame.take() else {
      return;
    };

    if old.rect != frame.rect || old.scale_factor != frame.scale_factor {
      self.area = RectF::from_rect(&frame.rect);
      self.metrics = CardMetrics::new(
        self.area.w,
        self.area.h,
        frame.scale_factor,
        self.drawn_for,
      );
    }

    self.interaction.sync(&old.workspaces, &frame.workspaces);
    self
      .interaction
      .digit_workspaces
      .clone_from(&frame.digit_workspaces);
    self.sync_backdrop(&frame, self.openness());

    let is_restyled = old.style != frame.style;
    self.frame = Some(frame);
    let is_changed = self.rebuild_cards() | self.rebuild_scenes();
    self.ensure_labels();
    self.retarget(false);

    if is_changed || self.registered_for != Some(self.interaction.layout) {
      self.register_thumbnails();
    }
    self.refresh_pictures(is_restyled);
    self.is_gliding = self.place();

    // Windows moved by the WM can be raised over the overview.
    // SAFETY: `self.hwnd` is the overview's window, on this thread.
    unsafe {
      let _ = SetWindowPos(
        self.hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
      );
    }
  }

  /// Hides the overview, keeping its pictures for the next open.
  fn close(&mut self) {
    if self.frame.take().is_none() {
      return;
    }

    // SAFETY: `self.hwnd` is the overview's window, on this thread.
    unsafe {
      ShowWindow(self.hwnd, SW_HIDE);
      let _ = ReleaseCapture();
    }

    for card in self.cards.drain(..) {
      self.spare.extend(card.into_pictures());
    }
    self.scenes.clear();
    if let Some(overlays) = &mut self.overlays {
      overlays.unregister();
    }
    if let Some(hint) = &mut self.hint {
      hint.thumbnail = None;
    }
    self.registered_for = None;
    self.ghost = None;
    self.zoom = None;
    self.vsync = None;
    self.is_tracking_leave = false;
  }

  fn sync_backdrop(&mut self, frame: &OverviewFrame, progress: f32) {
    let style = &frame.style;
    let result = match &mut self.backdrop {
      Some(backdrop) => backdrop.update(
        &frame.rect,
        style.backdrop_blur,
        style.backdrop_tint,
      ),
      None => OverviewBackdrop::create(
        self.hwnd,
        &frame.rect,
        style.backdrop_blur,
        style.backdrop_tint,
      )
      .map(|backdrop| {
        self.backdrop = Some(backdrop);
      }),
    };

    if let Err(err) = result {
      tracing::warn!("Overview backdrop unavailable: {err}");
    }

    if let Some(backdrop) = &self.backdrop {
      let _ = backdrop.set_progress(progress);
    }
  }

  /// Draws the cards for the layout now shown, at the size it shows them,
  /// if they were drawn for another, and starts the windows it shows on
  /// their own from where their cards showed them. Returns whether it
  /// did.
  fn draw_for_layout(&mut self) -> bool {
    let layout = self.interaction.layout;
    let previous = self.drawn_for;
    if layout == previous {
      return false;
    }

    let widths = self
      .cards
      .iter()
      .map(|card| card.metrics.width)
      .collect::<Vec<_>>();

    self.drawn_for = layout;
    self.metrics = CardMetrics::new(
      self.area.w,
      self.area.h,
      self.metrics.scale_factor,
      layout,
    );
    self.remap_cards();

    // Each card stays the size it is on screen, in its new units.
    for (card, width) in self.cards.iter_mut().zip(widths) {
      card.scale.rescale(width / card.metrics.width.max(1.0));
    }

    if layout.has_scene() && !previous.has_scene() {
      self.seed_scene_from_card();
    }

    true
  }

  /// Registers every thumbnail the layout shows again, bottom to top:
  /// the activities' scene panel, the windows on the scenes, each card's
  /// tab bars, previews and picture, the labels, highlights and tags, the
  /// hints, then the floating window.
  fn register_thumbnails(&mut self) {
    let host = self.hwnd;
    let layout = self.interaction.layout;
    self.unregister_thumbnails();

    if let (OverviewLayoutMode::Activities, Some(overlays)) =
      (layout, &mut self.overlays)
    {
      overlays.scene_panel.register(host);
    }

    if layout.has_scene() {
      for window in
        self.scenes.iter_mut().flat_map(|scene| &mut scene.windows)
      {
        window.preview.register(host);
      }
    }

    for card in &mut self.cards {
      // Under the windows, which cover the strip reaching under them.
      for bar in &mut card.tab_bars {
        bar.picture.thumbnail =
          Thumbnail::register(host, bar.picture.window);
      }
      for preview in &mut card.previews {
        preview.register(host);
      }
      card.picture.thumbnail =
        Thumbnail::register(host, card.picture.window);
    }

    if let Some(overlays) = &mut self.overlays {
      overlays.register(host, layout);
    }

    if self.hint.is_none() {
      self.hint = Picture::new();
    }
    if let Some(hint) = &mut self.hint {
      hint.thumbnail = Thumbnail::register(host, hint.window);
    }

    if let Some(ghost) = &mut self.ghost {
      ghost.preview.register(host);
      ghost.picture.thumbnail =
        Thumbnail::register(host, ghost.picture.window);
    }

    self.registered_for = Some(layout);
  }

  fn unregister_thumbnails(&mut self) {
    for card in &mut self.cards {
      card.picture.thumbnail = None;
      for bar in &mut card.tab_bars {
        bar.picture.thumbnail = None;
      }
      for preview in &mut card.previews {
        preview.thumbnail = None;
      }
    }
    for window in
      self.scenes.iter_mut().flat_map(|scene| &mut scene.windows)
    {
      window.preview.thumbnail = None;
    }
    if let Some(overlays) = &mut self.overlays {
      overlays.unregister();
    }
    if let Some(hint) = &mut self.hint {
      hint.thumbnail = None;
    }
    if let Some(ghost) = &mut self.ghost {
      ghost.preview.thumbnail = None;
      ghost.picture.thumbnail = None;
    }
  }

  /// Points every card and scene window at where it belongs now.
  fn retarget(&mut self, snap: bool) {
    let slots = self.slots();
    // The special workspace's dock grows while a window is carried or
    // dragged, to be dropped on.
    let is_receiving = self.interaction.floating().is_some();
    // Looking inside the docked special workspace leaves the row as it
    // is.
    let is_zoomed = self.interaction.mode == Mode::Windows
      && self.interaction.selected == self.interaction.row_selected;
    let selected = self.interaction.selected;

    let targets = match self.interaction.layout {
      OverviewLayoutMode::Carousel => carousel(
        &self.metrics,
        &slots,
        self.interaction.row_selected,
        is_zoomed,
        is_receiving,
      ),
      OverviewLayoutMode::Grid => {
        grid(&self.metrics, &slots, &self.interaction.grid, is_receiving)
      }
      OverviewLayoutMode::Stage => {
        stage_layout(&self.metrics, &slots, selected)
      }
      OverviewLayoutMode::Activities => {
        let (targets, area) = activities(&self.metrics, &slots);
        self.scene_area = area;
        targets
      }
    };
    self.dock = targets.dock;

    for (index, (card, target)) in
      self.cards.iter_mut().zip(&targets.cards).enumerate()
    {
      let bump = if self.interaction.is_drop_target(index) {
        1.06
      } else {
        1.0
      };

      for (spring, value) in [
        (&mut card.cx, target.cx),
        (&mut card.cy, target.cy),
        (&mut card.scale, target.scale * bump),
        (&mut card.alpha, target.opacity),
      ] {
        if snap {
          spring.snap(value);
        } else {
          spring.target = value;
        }
      }
    }

    if snap {
      self.hint_y.snap(targets.hint_y);
    } else {
      self.hint_y.target = targets.hint_y;
    }

    if self.interaction.layout.has_scene() {
      self.retarget_scenes(snap);
    }
    self.sync_nav();
    self.sync_ghost();
    self.is_dirty = true;
  }

  /// Tells the interaction where the selected workspace's windows show
  /// once settled, first to last, for the arrows to go between them.
  fn sync_nav(&mut self) {
    let selected = self.interaction.selected;

    let nav = if self.interaction.layout.has_scene() {
      self.scenes.get(selected).map(|scene| {
        scene
          .reading_order()
          .into_iter()
          .map(|index| &scene.windows[index])
          .filter(|window| window.alpha.target > 0.0)
          .map(|window| (window.preview.hwnd, window.target()))
          .collect()
      })
    } else {
      self.cards.get(selected).map(|card| {
        let placement = Placement::new(
          card.cx.target,
          card.cy.target,
          card.scale.target,
        );
        let mut previews = card.previews.iter().collect::<Vec<_>>();
        previews.sort_by(|a, b| {
          let (a, b) = (&a.screen_rect, &b.screen_rect);
          a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y))
        });
        previews
          .into_iter()
          .map(|preview| {
            (
              preview.hwnd,
              placement.map(&card.metrics, &preview.tile.rect),
            )
          })
          .collect()
      })
    };

    self.interaction.nav = nav.unwrap_or_default();
  }

  /// Creates, moves or drops the floating window's picture.
  fn sync_ghost(&mut self) {
    let floating = self.interaction.floating();
    if self.ghost.as_ref().map(|ghost| ghost.hwnd) != floating {
      self.ghost = floating.and_then(|hwnd| self.create_ghost(hwnd));
    }

    let px = |length: f32| length * self.metrics.scale_factor;
    let (width, height) = (px(GHOST_SIZE.0), px(GHOST_SIZE.1));
    let (view_width, view_height) = self.metrics.view;
    let drag = self.interaction.drag;

    if let Some(ghost) = &mut self.ghost {
      if let Some(drag) = drag {
        ghost.x.snap(drag.position.0 - width / 2.0);
        ghost.y.snap(drag.position.1 - height / 2.0);
      } else {
        // Parked while carrying: beside the dock it can be dropped on, or
        // else in the corner above the hints.
        let (x, y) = match self.dock {
          Some(dock) => {
            (dock.x - width - px(32.0), dock.y + (dock.h - height) / 2.0)
          }
          None => (
            view_width - width - px(40.0),
            self.hint_y.target - height - px(16.0),
          ),
        };
        ghost.x.target = x;
        ghost.y.target = y.min(view_height - height);
      }
    }
  }

  fn create_ghost(&self, hwnd: isize) -> Option<Ghost> {
    let px = |length: f32| length * self.metrics.scale_factor;
    let (width, height) = (px(GHOST_SIZE.0), px(GHOST_SIZE.1));
    let (view_width, view_height) = self.metrics.view;

    let frame = visible_frame(HWND(hwnd)).unwrap_or(FALLBACK_FRAME);
    let inset = px(2.0) * 2.0;

    #[allow(clippy::cast_precision_loss)]
    let crop = cover_crop(
      (
        (frame.right - frame.left) as f32,
        (frame.bottom - frame.top) as f32,
      ),
      (width - inset, height - inset),
    );
    let mut preview = Preview::new(
      hwnd,
      Tile {
        rect: RectF::default(),
        crop,
      },
    );
    preview.frame = frame;
    preview.register(self.hwnd);

    let mut picture = Picture::new()?;
    picture.thumbnail = Thumbnail::register(self.hwnd, picture.window);

    // Starts out in the row, so a lifted window visibly drops to the
    // bottom.
    let mut x = Spring::new(0.0, 280.0, 0.25);
    let mut y = Spring::new(0.0, 340.0, 0.25);
    x.snap((view_width - width) / 2.0);
    y.snap((view_height - height) / 2.0);

    Some(Ghost {
      hwnd,
      x,
      y,
      preview,
      picture,
    })
  }

  /// Redraws every picture whose content changed, or all of them when
  /// `force`.
  fn refresh_pictures(&mut self, force: bool) {
    let Some(frame) = &self.frame else {
      return;
    };
    let style = frame.style.clone();
    let notify = (self.hwnd, WM_ICON_READY);
    let context = self.place_context();
    let layout = self.interaction.layout;

    for index in 0..self.cards.len().min(self.workspaces().len()) {
      let picture = self.card_picture(index, &style);
      let Some(card) = self.cards.get_mut(index) else {
        continue;
      };

      for (preview, tile) in card.previews.iter_mut().zip(&picture.tiles) {
        preview.opacity = tile.opacity;
      }

      // A hidden tab picked with the keyboard (e.g. found by a search)
      // shows on its stack's bar.
      let cursor = self.interaction.selected_window;
      for bar in &mut card.tab_bars {
        let hover = cursor
          .and_then(|hwnd| bar.tab_index(hwnd))
          .filter(|index| *index != bar.frame.active_index)
          .map_or(TabHit::Empty, TabHit::Tab);
        bar.draw(hover, self.icons, notify);
      }

      if !force && card.drawn.as_ref() == Some(&picture) {
        continue;
      }

      let metrics = card.metrics;
      let old_size = card.picture.size();
      card.picture.draw(picture_size(&metrics), |surface| {
        draw_card(surface, &picture, &metrics, &style, notify);
      });
      card.drawn = Some(picture);

      // Placed at once when it changed size: until it is, its thumbnail
      // shows the new drawing through the old size's source rect.
      if card.picture.size() != old_size {
        place_card(card, index, &context);
      }
    }

    if layout.has_scene() {
      for index in 0..self.scenes.len() {
        let opacities = self.scenes[index]
          .windows
          .iter()
          .map(|window| self.window_opacity(index, window.preview.hwnd))
          .collect::<Vec<_>>();
        for (window, opacity) in
          self.scenes[index].windows.iter_mut().zip(opacities)
        {
          window.preview.opacity = opacity;
        }
      }
    }

    self.refresh_overlays(&style, force);

    let hint = self.interaction.hint(self.workspaces());
    if force || self.hint_drawn.as_ref() != Some(&hint) {
      let (text, tone) = &hint;
      let color = match tone {
        HintTone::Quiet => with_alpha(style.subtext, 0.85),
        HintTone::Accent => with_alpha(style.accent, 0.9),
        HintTone::Search => with_alpha(style.search, 0.85),
      };
      let scale_factor = self.metrics.scale_factor;

      #[allow(clippy::cast_possible_truncation)]
      let size = (
        self.metrics.view.0.ceil() as i32,
        (HINT_HEIGHT * scale_factor).ceil() as i32,
      );

      if let Some(picture) = &mut self.hint {
        picture.draw(size, |surface| {
          draw_hint(surface, text, color, scale_factor, &style);
        });
      }
      self.hint_drawn = Some(hint);
    }

    let title = self.ghost.as_ref().and_then(|ghost| {
      self.workspaces().iter().find_map(|workspace| {
        workspace
          .windows
          .iter()
          .find(|window| window.hwnd == ghost.hwnd)
          .map(|window| window.title.clone())
      })
    });
    let scale_factor = self.metrics.scale_factor;

    if let (Some(ghost), Some(title)) = (&mut self.ghost, title) {
      #[allow(clippy::cast_possible_truncation)]
      let size = (
        (GHOST_SIZE.0 * scale_factor).ceil() as i32,
        (GHOST_SIZE.1 * scale_factor).ceil() as i32,
      );
      if ghost.picture.size() != size || force {
        ghost.picture.draw(size, |surface| {
          draw_ghost(surface, &title, scale_factor, &style);
        });
      }
    }

    self.is_dirty = true;
  }

  /// How far the overview is open, from 0 (a card's windows fill the
  /// screen) to 1 (every card in place).
  fn openness(&self) -> f32 {
    self
      .zoom
      .map_or(1.0, |(zoom, started)| zoom.openness(elapsed_ms(started)))
  }

  fn is_closing(&self) -> bool {
    self.zoom.is_some_and(|(zoom, _)| zoom.is_closing)
  }

  /// The zoom between the screen and the zoomed card's windows, at the
  /// current openness. Follows the card if it is still moving. The
  /// activities zoom each window on its own instead.
  fn camera(&self) -> Camera {
    if self.interaction.layout.has_scene() {
      return Camera::IDENTITY;
    }

    let card = self.zoom.and_then(|(zoom, _)| self.cards.get(zoom.card));
    let Some(card) = card else {
      return Camera::IDENTITY;
    };

    // Where the card shows the monitor, which a scrolling workspace's
    // card shows only part of.
    let screen = card
      .placement()
      .map(&card.metrics, &card.view.map(&self.area));
    let view =
      RectF::new(0.0, 0.0, self.metrics.view.0, self.metrics.view.1);
    Camera::zoom(&screen, &view, self.openness())
  }

  /// Zooms into card `index` until its windows fill the screen, then
  /// hides, by when the WM has those windows in place behind it.
  ///
  /// Without a card or a close duration, the overview stays up until the
  /// WM hides it once focus has moved on.
  fn start_closing(&mut self, index: Option<usize>) {
    let duration = self.style().map_or(0, |style| style.close_duration_ms);
    let Some(index) = index.filter(|_| duration > 0) else {
      return;
    };

    // The scenes zoom into the picked workspace's own windows, so it
    // shows them first.
    if self.interaction.layout.has_scene()
      && index != self.interaction.selected
    {
      self.interaction.selected = index;
      self.retarget(true);
    }

    let zoom = Zoom::close(index, self.openness(), duration);
    self.zoom = Some((zoom, Instant::now()));
    self.is_dirty = true;
  }

  /// Card the overview zooms into when it closes for `action`.
  fn closing_card(&self, action: &OverviewAction) -> Option<usize> {
    let workspaces = self.workspaces();

    match action {
      OverviewAction::FocusWindow(hwnd) => {
        workspaces.iter().position(|workspace| {
          workspace.windows.iter().any(|window| window.hwnd == *hwnd)
        })
      }
      OverviewAction::FocusWorkspace(name) => workspaces
        .iter()
        .position(|workspace| workspace.name == *name),
      OverviewAction::Cancel => {
        workspaces.iter().position(|workspace| workspace.is_focused)
      }
      _ => None,
    }
  }

  fn place_context(&self) -> PlaceContext {
    PlaceContext {
      camera: self.camera(),
      fade: self.openness(),
      view: RectF::new(0.0, 0.0, self.metrics.view.0, self.metrics.view.1),
      zoom_card: self.zoom.map(|(zoom, _)| zoom.card),
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

    let mut is_moving = false;
    for card in &mut self.cards {
      is_moving |= card.cx.step(dt);
      is_moving |= card.cy.step(dt);
      is_moving |= card.scale.step(dt);
      is_moving |= card.alpha.step(dt);
    }
    for scene in &mut self.scenes {
      is_moving |= scene.step(dt);
    }
    if let Some(overlays) = &mut self.overlays {
      is_moving |= overlays.step(dt);
    }
    is_moving |= self.hint_y.step(dt);
    if let Some(ghost) = &mut self.ghost {
      is_moving |= ghost.x.step(dt);
      is_moving |= ghost.y.step(dt);
    }

    if let Some((zoom, started)) = self.zoom {
      is_moving = true;
      let elapsed = elapsed_ms(started);

      if let Some(backdrop) = &self.backdrop {
        let _ = backdrop.set_progress(zoom.openness(elapsed));
      }

      if zoom.is_finished(elapsed) {
        self.zoom = None;
        if zoom.is_closing {
          self.close();
          return false;
        }
      }
    }

    if is_moving || self.is_dirty || self.is_gliding {
      self.is_gliding = self.place();
      self.is_dirty = false;
    }

    is_moving || self.is_gliding
  }

  /// Places every thumbnail where its animation has it now. Returns
  /// whether a highlight is still gliding.
  fn place(&mut self) -> bool {
    let context = self.place_context();
    let (view, scale_factor) = (context.view, self.metrics.scale_factor);
    let fade = context.fade;
    let layout = self.interaction.layout;
    let shift = self.top_shift(fade);

    match layout {
      OverviewLayoutMode::Activities => {
        let miniatures = PlaceContext {
          camera: Camera::shifted(0.0, shift),
          fade,
          view,
          zoom_card: None,
        };
        for (index, card) in self.cards.iter_mut().enumerate() {
          place_card(card, index, &miniatures);
        }
      }
      _ => {
        for (index, card) in self.cards.iter_mut().enumerate() {
          place_card(card, index, &context);
        }
      }
    }

    if layout.has_scene() {
      let zoom_card = context.zoom_card;
      for (index, scene) in self.scenes.iter_mut().enumerate() {
        for window in &mut scene.windows {
          let rect =
            scene_window_rect(window, zoom_card == Some(index), fade);
          let opacity = window.alpha.value * window.preview.opacity;

          if opacity > 0.002 {
            window.preview.place(&rect, &view, opacity);
          } else if let Some(thumbnail) = &mut window.preview.thumbnail {
            thumbnail.hide();
          }
        }
      }
    }

    let is_gliding = self.place_overlays(&context, shift);

    let hint_height = HINT_HEIGHT * scale_factor;
    if let Some(hint) = &mut self.hint {
      hint.place(
        &RectF::new(0.0, self.hint_y.value, view.w, hint_height),
        &view,
        fade,
      );
    }

    if let Some(ghost) = &mut self.ghost {
      let rect = RectF::new(
        ghost.x.value,
        ghost.y.value,
        GHOST_SIZE.0 * scale_factor,
        GHOST_SIZE.1 * scale_factor,
      );
      ghost
        .preview
        .place(&rect.inset(scale_factor * 2.0), &view, 0.9);
      ghost.picture.place(&rect, &view, 1.0);
    }

    is_gliding
  }

  /// What is under the overview point (`x`, `y`).
  fn hit(&self, x: f32, y: f32) -> Hit {
    match self.interaction.layout {
      OverviewLayoutMode::Activities => {
        let hit = self.hit_cards(x, y, &Camera::IDENTITY);
        if hit.workspace.is_some() {
          hit
        } else {
          self.hit_scene(x, y, &self.scene_area)
        }
      }
      _ => self.hit_cards(x, y, &self.camera()),
    }
  }

  /// What card is under (`x`, `y`) with `camera` applied, and what window
  /// on it.
  fn hit_cards(&self, x: f32, y: f32, camera: &Camera) -> Hit {
    // Mid-zoom, the cards aren't where they will settle.
    let (x, y) = camera.unapply(x, y);

    // Later cards stack above earlier ones (e.g. a grown drop target over
    // its neighbour), so they are hit first.
    for (index, card) in self.cards.iter().enumerate().rev() {
      let Some((local_x, local_y)) =
        card.placement().unmap(&card.metrics, x, y)
      else {
        continue;
      };

      // Windows cover the strip of their tab bar reaching under them.
      let window = card
        .previews
        .iter()
        .rev()
        .find(|preview| preview.tile.rect.contains(local_x, local_y))
        .map(|preview| preview.hwnd)
        .or_else(|| {
          card
            .tab_bars
            .iter()
            .rev()
            .find_map(|bar| bar.hit(local_x, local_y))
        });

      return Hit {
        workspace: Some(index),
        window,
      };
    }

    Hit::default()
  }

  /// What of the selected workspace's scene is under (`x`, `y`): the
  /// topmost of its windows there, or else `area` itself.
  fn hit_scene(&self, x: f32, y: f32, area: &RectF) -> Hit {
    let selected = self.interaction.selected;
    let window = self.scenes.get(selected).and_then(|scene| {
      scene
        .windows
        .iter()
        .rev()
        .find(|window| {
          window.alpha.target > 0.0 && window.rect().contains(x, y)
        })
        .map(|window| window.preview.hwnd)
    });

    if window.is_some() || area.contains(x, y) {
      Hit {
        workspace: Some(selected),
        window,
      }
    } else {
      Hit::default()
    }
  }

  /// Acts on what the user did, after the interaction state took it in.
  fn after_input(&mut self, action: Option<OverviewAction>) {
    if let Some(action) = action {
      let is_final = matches!(
        action,
        OverviewAction::FocusWindow(_)
          | OverviewAction::FocusWorkspace(_)
          | OverviewAction::Cancel
      );
      let card = self.closing_card(&action);
      self.send(action);

      if is_final {
        self.is_done = true;
        self.start_closing(card);
      }
    }

    let is_redrawn = self.draw_for_layout();
    self.ensure_labels();
    self.retarget(false);

    if self.registered_for != Some(self.interaction.layout) {
      self.register_thumbnails();
    }
    self.refresh_pictures(is_redrawn);
  }

  fn send(&self, action: OverviewAction) {
    if !self.is_done {
      (self.on_action)(self.session, action);
    }
  }

  fn on_key(&mut self, key: Key) {
    let workspaces = self.workspaces().to_vec();
    let action = self.interaction.key(key, &workspaces);
    self.after_input(action);
  }

  fn on_toggle(&mut self, layout: OverviewLayoutMode) {
    if self.interaction.layout == layout {
      self.after_input(Some(OverviewAction::Cancel));
    } else {
      let workspaces = self.workspaces().to_vec();
      self.interaction.set_layout(layout, &workspaces);
      self.after_input(None);
    }
  }

  fn on_mouse_move(&mut self, (x, y): (i32, i32), is_left_down: bool) {
    if (x, y) == self.open_cursor {
      return;
    }
    self.open_cursor = (i32::MIN, i32::MIN);

    if !self.is_tracking_leave {
      let mut track = TRACKMOUSEEVENT {
        cbSize: u32::try_from(std::mem::size_of::<TRACKMOUSEEVENT>())
          .unwrap_or_default(),
        dwFlags: TME_LEAVE,
        hwndTrack: self.hwnd,
        dwHoverTime: 0,
      };
      // SAFETY: `track` outlives the call.
      self.is_tracking_leave =
        unsafe { TrackMouseEvent(&raw mut track) }.is_ok();
    }

    #[allow(clippy::cast_precision_loss)]
    let position = (x as f32, y as f32);
    let hit = self.hit(position.0, position.1);
    let was_dragging = self.interaction.drag.is_some();

    if self.interaction.mouse_move(position, hit, is_left_down) {
      self.after_input(None);
    } else if was_dragging {
      self.sync_ghost();
      self.is_dirty = true;
    }
  }

  fn on_left_down(&mut self, (x, y): (i32, i32)) {
    #[allow(clippy::cast_precision_loss)]
    let position = (x as f32, y as f32);
    let hit = self.hit(position.0, position.1);
    self.interaction.mouse_down(position, hit);

    // SAFETY: `self.hwnd` is the overview's window, on this thread.
    unsafe {
      SetCapture(self.hwnd);
    }
  }

  fn on_left_up(&mut self) {
    // SAFETY: No preconditions.
    unsafe {
      let _ = ReleaseCapture();
    }

    let workspaces = self.workspaces().to_vec();
    let action = self.interaction.mouse_up(&workspaces);
    self.after_input(action);
  }

  fn on_middle_down(&mut self, (x, y): (i32, i32)) {
    #[allow(clippy::cast_precision_loss)]
    let hit = self.hit(x as f32, y as f32);
    let action = self.interaction.middle_click(hit);
    self.after_input(action);
  }

  fn on_wheel(&mut self, is_down: bool) {
    let workspaces = self.workspaces().to_vec();
    self.interaction.wheel(is_down, &workspaces);
    self.after_input(None);
  }

  fn cursor(&self) -> windows::core::PCWSTR {
    if self.interaction.drag.is_some() {
      IDC_SIZEALL
    } else if self.interaction.hover.is_some() {
      IDC_HAND
    } else {
      IDC_ARROW
    }
  }

  /// Handles an overview message. Returns `None` for the default handling.
  fn handle(
    &mut self,
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
        // A closing overview hides itself once its zoom ends.
        if !self.is_closing() {
          self.close();
        }
      }
      WM_ICON_READY => {
        self.icons += 1;
        self.refresh_pictures(false);
      }
      WM_ACTIVATE => {
        // The low word of `wparam` is the activation state.
        // While closing, the window it zooms into has just been focused.
        if (wparam.0 & 0xffff) == WA_INACTIVE as usize
          && self.frame.is_some()
          && !self.is_closing()
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
        self.on_mouse_move(mouse_position(lparam), wparam.0 & 1 != 0);
      }
      WM_MOUSELEAVE => {
        self.is_tracking_leave = false;
        if is_open {
          self.interaction.mouse_leave();
          self.after_input(None);
        }
      }
      WM_LBUTTONDOWN if is_open => {
        self.on_left_down(mouse_position(lparam));
      }
      WM_LBUTTONUP if is_open => self.on_left_up(),
      WM_MBUTTONDOWN if is_open => {
        self.on_middle_down(mouse_position(lparam));
      }
      WM_MOUSEWHEEL if is_open => {
        // The high word of `wparam` is the signed wheel delta.
        #[allow(clippy::cast_possible_truncation)]
        let delta = ((wparam.0 >> 16) as u16).cast_signed();
        self.on_wheel(delta < 0);
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

#[allow(clippy::cast_precision_loss)]
fn to_f32_size((width, height): (i32, i32)) -> (f32, f32) {
  (width as f32, height as f32)
}

/// Cursor position in `hwnd`'s client coordinates.
fn cursor_in_client(hwnd: HWND) -> (i32, i32) {
  let mut point = POINT::default();

  // SAFETY: `point` outlives both calls.
  unsafe {
    if GetCursorPos(&raw mut point).is_ok() {
      let _ = ScreenToClient(hwnd, &raw mut point);
    }
  }

  (point.x, point.y)
}

/// Window procedure of the overview.
unsafe extern "system" fn wnd_proc(
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
      // Dropped while no longer borrowed: the pictures' windows and
      // thumbnails go with it.
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

  // Showing, hiding and activating the window send it messages while the
  // overview is borrowed; those get the default handling.
  with_overview(|overview| overview.handle(msg, wparam, lparam))
    .flatten()
    .unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}

#[cfg(test)]
mod tests {
  use super::is_strip_scrolled;
  use crate::{OverviewWorkspace, Rect};

  fn workspace(name: &str, content_rect: Rect) -> OverviewWorkspace {
    OverviewWorkspace {
      name: name.to_string(),
      label: name.to_string(),
      content_rect,
      ..OverviewWorkspace::default()
    }
  }

  #[test]
  fn only_a_moved_strip_counts_as_scrolled() {
    let area = Rect::from_xy(0, 0, 1920, 1040);
    let strip = |x| Rect::from_xy(x, 0, 3840, 1040);
    let old = [workspace("1", area.clone()), workspace("2", strip(0))];

    let scrolled =
      [workspace("1", area.clone()), workspace("2", strip(-960))];
    assert!(is_strip_scrolled(&old, &scrolled));

    let added = [
      workspace("1", area.clone()),
      workspace("2", strip(0)),
      workspace("3", area),
    ];
    assert!(!is_strip_scrolled(&old, &added));
  }
}
