//! The window overview's runtime: a thread of its own running a
//! full-screen window, with every card, hint and preview on it composited
//! by DWM.
//!
//! Each card's picture is drawn once into a cloaked layered window, and
//! shown on the overview as a DWM thumbnail of that window; window
//! previews are thumbnails too. An animation frame therefore draws
//! nothing: it only moves and scales thumbnails, all through one channel
//! to DWM, so cards and their previews can't drift apart mid-motion.

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
        PostQuitMessage, RegisterClassW, SetCursor, SetForegroundWindow,
        SetWindowPos, ShowWindow, TranslateMessage, WaitMessage, HTCLIENT,
        HWND_TOPMOST, IDC_ARROW, IDC_HAND, IDC_SIZEALL, MSG, PM_REMOVE,
        SC_KEYMENU, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_HIDE, WA_INACTIVE, WM_ACTIVATE, WM_APP,
        WM_CHAR, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDOWN,
        WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MOUSEMOVE, WM_MOUSEWHEEL,
        WM_QUIT, WM_SETCURSOR, WM_SYSCOMMAND, WNDCLASSW,
        WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
        WS_POPUP,
      },
    },
  },
};

use crate::{
  overview_chrome::{
    card_size, draw_card, draw_ghost, draw_hint, with_alpha, CardPicture,
    TilePicture, GHOST_SIZE, HINT_HEIGHT,
  },
  overview_layout::{
    carousel, cover_crop, ease_out_cubic, grid, tile, Camera, CardMetrics,
    Placement, RectF, Spring, Tile,
  },
  overview_state::{HintTone, Hit, Interaction, Key, Mode},
  overview_thumbnails::{
    visible_frame, visible_frame_on_screen, Picture, Preview, Thumbnail,
    FALLBACK_FRAME,
  },
  platform_impl::{self, composition::OverviewBackdrop},
  window_icons, DxgiVsyncWaiter, OverviewAction, OverviewFrame,
  OverviewLayoutMode, OverviewStyle, OverviewWorkspace,
};

/// Posted with a `Box<Open>` in `WPARAM`.
const WM_OPEN_OVERVIEW: u32 = WM_APP + 1;

/// Posted with a `Box<OverviewFrame>` in `WPARAM` to change what an open
/// overview shows.
const WM_UPDATE_OVERVIEW: u32 = WM_APP + 2;

/// Posted with the requested layout in `WPARAM` (1 for the grid).
const WM_TOGGLE_OVERVIEW: u32 = WM_APP + 3;

/// Posted to close the overview.
const WM_HIDE_OVERVIEW: u32 = WM_APP + 4;

/// Posted by the icon thread once an icon is cached.
const WM_ICON_READY: u32 = WM_APP + 5;

/// How long card moves and zooms take to settle.
const SETTLE_MS: f32 = 260.0;

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
    self.post(
      WM_TOGGLE_OVERVIEW,
      usize::from(layout == OverviewLayoutMode::Grid),
    );
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
  REGISTERED.get_or_init(|| {
    let class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_Overview"),
      lpfnWndProc: Some(wnd_proc),
      // SAFETY: `IDC_ARROW` is a system cursor.
      hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default(),
      ..Default::default()
    };

    // SAFETY: `class` is fully initialized with a static class name.
    unsafe { RegisterClassW(&raw const class) };
  });

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
  previews: Vec<Preview>,
  picture: Picture,

  /// What `picture` shows, to skip redrawing it unchanged.
  drawn: Option<CardPicture>,
}

impl Card {
  fn placement(&self) -> Placement {
    Placement {
      cx: self.cx.value,
      cy: self.cy.value,
      scale: self.scale.value,
    }
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
struct Overview {
  hwnd: HWND,
  on_action: ActionCallback,
  session: u64,

  /// What is shown, or `None` while hidden.
  frame: Option<OverviewFrame>,

  interaction: Interaction,
  metrics: CardMetrics,

  /// Working area covered, in screen coordinates.
  area: RectF,

  backdrop: Option<OverviewBackdrop>,
  cards: Vec<Card>,

  /// Pictures of cards no longer shown, kept for reuse.
  spare: Vec<Picture>,

  hint: Option<Picture>,
  hint_drawn: Option<(String, HintTone)>,
  hint_y: Spring,
  ghost: Option<Ghost>,

  /// When the zoom out on open started, while it runs.
  opening: Option<Instant>,

  /// The focused workspace's tiles, which the zoom out starts from.
  zoom_from: RectF,

  last_tick: Instant,

  /// Whether something changed that the next tick has to place.
  is_dirty: bool,

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
    Self {
      hwnd,
      on_action,
      session: 0,
      frame: None,
      interaction: Interaction::new(
        &[],
        OverviewLayoutMode::Carousel,
        1,
        0.0,
      ),
      metrics: CardMetrics::new(1.0, 1.0, 1.0),
      area: RectF::default(),
      backdrop: None,
      cards: Vec::new(),
      spare: Vec::new(),
      hint: None,
      hint_drawn: None,
      hint_y: Spring::new(0.0, SETTLE_MS, 0.25),
      ghost: None,
      opening: None,
      zoom_from: RectF::default(),
      last_tick: Instant::now(),
      is_dirty: false,
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
    self.metrics =
      CardMetrics::new(self.area.w, self.area.h, scale_factor);
    self.interaction = Interaction::new(
      &frame.workspaces,
      layout,
      frame.style.grid_columns,
      6.0 * scale_factor,
    );
    self.session = session;
    self.is_done = false;

    let animate = frame.style.open_duration_ms > 0;
    self.sync_backdrop(&frame, if animate { 0.0 } else { 1.0 });

    window_icons::retain(|hwnd| {
      frame.workspaces.iter().any(|workspace| {
        workspace.windows.iter().any(|window| window.hwnd == hwnd)
      })
    });

    self.frame = Some(frame);
    self.rebuild_cards();
    self.retarget(true);

    self.opening = animate.then(Instant::now);
    self.zoom_from = self
      .cards
      .get(self.interaction.selected)
      .map_or_else(RectF::default, |card| {
        card
          .placement()
          .map(&self.metrics, &self.metrics.tile_area())
      });

    self.register_thumbnails();
    self.refresh_pictures(true);
    self.last_tick = Instant::now();
    self.place();

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
  #[allow(clippy::float_cmp)]
  fn update(&mut self, frame: OverviewFrame) {
    let Some(old) = self.frame.take() else {
      return;
    };

    if old.rect != frame.rect || old.scale_factor != frame.scale_factor {
      self.area = RectF::from_rect(&frame.rect);
      self.metrics =
        CardMetrics::new(self.area.w, self.area.h, frame.scale_factor);
    }

    self.interaction.sync(&old.workspaces, &frame.workspaces);
    self.sync_backdrop(&frame, self.open_progress());

    let is_restyled = old.style != frame.style;
    self.frame = Some(frame);
    self.rebuild_cards();
    self.register_thumbnails();
    self.retarget(false);
    self.refresh_pictures(is_restyled);
    self.is_dirty = true;

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

    for mut card in self.cards.drain(..) {
      card.picture.thumbnail = None;
      self.spare.push(card.picture);
    }
    if let Some(hint) = &mut self.hint {
      hint.thumbnail = None;
    }
    self.ghost = None;
    self.opening = None;
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

  /// Builds a card per workspace, with a preview per visible window.
  fn rebuild_cards(&mut self) {
    let Some(frame) = &self.frame else {
      return;
    };

    let mut cards = std::mem::take(&mut self.cards);
    let mut spare = std::mem::take(&mut self.spare);

    for (index, workspace) in frame.workspaces.iter().enumerate() {
      let previews = workspace
        .windows
        .iter()
        .filter(|window| !window.is_minimized)
        .filter_map(|window| {
          let rect = if workspace.is_focused {
            // Measured, so the zoom out starts exactly where the window
            // is on screen.
            visible_frame_on_screen(HWND(window.hwnd))
              .unwrap_or_else(|| RectF::from_rect(&window.rect))
          } else {
            RectF::from_rect(&window.rect)
          };

          let tile = tile(&self.metrics, &self.area, &rect)?;
          Some(Preview::new(window.hwnd, tile))
        })
        .collect::<Vec<_>>();

      if let Some(card) = cards.get_mut(index) {
        card.previews = previews;
        continue;
      }

      let Some(picture) = spare.pop().or_else(Picture::new) else {
        tracing::warn!("Failed to create an overview card.");
        continue;
      };

      cards.push(Card {
        cx: Spring::new(0.0, SETTLE_MS, 0.25),
        cy: Spring::new(0.0, SETTLE_MS, 0.25),
        scale: Spring::new(1.0, SETTLE_MS, 0.001),
        previews,
        picture,
        drawn: None,
      });
    }

    for mut card in cards.drain(frame.workspaces.len().min(cards.len())..)
    {
      card.picture.thumbnail = None;
      spare.push(card.picture);
    }

    self.cards = cards;
    self.spare = spare;
  }

  /// Registers every thumbnail again, in stacking order: each card's
  /// previews then its picture, then the hints, then the floating window.
  fn register_thumbnails(&mut self) {
    let host = self.hwnd;

    for card in &mut self.cards {
      card.picture.thumbnail = None;
      for preview in &mut card.previews {
        preview.thumbnail = None;
      }
    }
    if let Some(hint) = &mut self.hint {
      hint.thumbnail = None;
    }

    for card in &mut self.cards {
      for preview in &mut card.previews {
        preview.register(host);
      }
      card.picture.thumbnail =
        Thumbnail::register(host, card.picture.window);
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
  }

  /// Points every card at where it belongs now.
  fn retarget(&mut self, snap: bool) {
    let count = self.cards.len();
    let targets = match self.interaction.layout {
      OverviewLayoutMode::Carousel => carousel(
        &self.metrics,
        count,
        self.interaction.selected,
        self.interaction.mode == Mode::Windows,
      ),
      OverviewLayoutMode::Grid => grid(
        &self.metrics,
        count,
        self.style().map_or(5, |style| style.grid_columns),
      ),
    };

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

    self.sync_ghost();
    self.is_dirty = true;
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
        // Parked at the bottom while carrying.
        ghost.x.target = (view_width - width) / 2.0;
        ghost.y.target = view_height - height - px(80.0);
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

    for index in 0..self.cards.len() {
      let picture = self.card_picture(index, &style);
      let Some(card) = self.cards.get_mut(index) else {
        continue;
      };

      for (preview, tile) in card.previews.iter_mut().zip(&picture.tiles) {
        preview.opacity = tile.opacity;
      }

      if !force && card.drawn.as_ref() == Some(&picture) {
        continue;
      }

      let size = card_size(&self.metrics, picture.scale);
      let metrics = self.metrics;
      card.picture.draw(size, |surface| {
        draw_card(surface, &picture, &metrics, &style, notify);
      });
      card.drawn = Some(picture);
    }

    let hint = self.interaction.hint(self.workspaces());
    if force || self.hint_drawn.as_ref() != Some(&hint) {
      let (text, tone) = &hint;
      let color = match tone {
        HintTone::Quiet => with_alpha(style.subtext, 0.6),
        HintTone::Accent => with_alpha(style.accent, 0.9),
        HintTone::Search => with_alpha(style.search, 0.6),
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

  /// Everything card `index` shows.
  fn card_picture(
    &self,
    index: usize,
    style: &OverviewStyle,
  ) -> CardPicture {
    let workspace = self.workspaces().get(index);
    let card = self.cards.get(index);
    let interaction = &self.interaction;
    let is_selected = index == interaction.selected;

    let (accent, search, caption) =
      (style.accent, style.search, style.caption);
    let focused_window =
      self.frame.as_ref().and_then(|frame| frame.focused_window);

    let tiles = card.map_or_else(Vec::new, |card| {
      card
        .previews
        .iter()
        .map(|preview| {
          let window = workspace.and_then(|workspace| {
            workspace.windows.iter().find(|w| w.hwnd == preview.hwnd)
          });
          let is_match = window.is_some_and(|w| interaction.is_match(w));
          let is_picked = interaction.floating() == Some(preview.hwnd);
          let is_cursor = is_selected
            && interaction.selected_window == Some(preview.hwnd);
          let is_marked = is_cursor && interaction.mode == Mode::Windows;
          let is_focused = focused_window == Some(preview.hwnd);

          // Dragged away it goes translucent, so it visibly left without
          // vanishing; during a search the misses step back.
          let opacity = if is_picked {
            0.35
          } else if !interaction.query.is_empty() && !is_match {
            0.3
          } else {
            1.0
          };

          let (border_width, border) = if is_match {
            (3.0, search)
          } else if is_marked {
            (3.0, accent)
          } else if is_picked {
            (2.0, with_alpha(accent, 0.9))
          } else if is_cursor || is_focused {
            (2.0, with_alpha(accent, 0.6))
          } else {
            (1.0, with_alpha(caption, 0.6))
          };

          TilePicture {
            hwnd: preview.hwnd,
            rect: preview.tile.rect,
            title: window.map_or_else(String::new, |w| {
              if w.title.is_empty() {
                w.process_name.clone()
              } else {
                w.title.clone()
              }
            }),
            has_preview: preview.thumbnail.is_some(),
            opacity,
            border_width,
            border,
          }
        })
        .collect()
    });

    CardPicture {
      // Drawn at the size it's heading for, so it's scaled down rather
      // than up while it animates.
      scale: card.map_or(1.0, |card| card.scale.target.max(0.05)),
      label: workspace.map_or_else(String::new, |w| w.label.clone()),
      is_focused: workspace.is_some_and(|w| w.is_focused),
      is_hovered: interaction.hover == Some(index),
      is_drop_target: interaction.is_drop_target(index),
      is_selected,
      window_count: workspace.map_or(0, |w| w.windows.len()),
      minimized: workspace.map_or_else(Vec::new, |w| {
        w.windows
          .iter()
          .filter(|window| window.is_minimized)
          .map(|window| window.hwnd)
          .collect()
      }),
      tiles,
      icons: self.icons,
    }
  }

  fn open_progress(&self) -> f32 {
    let Some(started) = self.opening else {
      return 1.0;
    };
    let duration = self
      .style()
      .map_or(0, |style| style.open_duration_ms)
      .max(1);

    #[allow(clippy::cast_precision_loss)]
    let progress =
      started.elapsed().as_secs_f32() * 1000.0 / duration as f32;
    progress.min(1.0)
  }

  /// The zoom out on open, at the current progress.
  fn camera(&self) -> Camera {
    if self.opening.is_none() {
      return Camera::IDENTITY;
    }

    let view =
      RectF::new(0.0, 0.0, self.metrics.view.0, self.metrics.view.1);
    Camera::zoom(
      &self.zoom_from,
      &view,
      ease_out_cubic(self.open_progress()),
    )
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
    }
    is_moving |= self.hint_y.step(dt);
    if let Some(ghost) = &mut self.ghost {
      is_moving |= ghost.x.step(dt);
      is_moving |= ghost.y.step(dt);
    }

    if self.opening.is_some() {
      is_moving = true;
      let progress = self.open_progress();
      if let Some(backdrop) = &self.backdrop {
        let _ = backdrop.set_progress(ease_out_cubic(progress));
      }
      if progress >= 1.0 {
        self.opening = None;
      }
    }

    if is_moving || self.is_dirty {
      self.place();
      self.is_dirty = false;
    }

    is_moving
  }

  /// Places every thumbnail where its animation has it now.
  fn place(&mut self) {
    let camera = self.camera();
    let fade = ease_out_cubic(self.open_progress());
    let view =
      RectF::new(0.0, 0.0, self.metrics.view.0, self.metrics.view.1);
    let metrics = self.metrics;

    for card in &mut self.cards {
      let placement = card.placement();
      let rect = camera.apply(&placement.rect(&metrics));
      let is_visible = rect.intersect(&view).is_some();

      for preview in &mut card.previews {
        let dest =
          camera.apply(&placement.map(&metrics, &preview.tile.rect));
        if is_visible {
          preview.place(&dest, preview.opacity);
        } else if let Some(thumbnail) = &mut preview.thumbnail {
          thumbnail.hide();
        }
      }

      if is_visible {
        card.picture.place(&rect, fade);
      } else if let Some(thumbnail) = &mut card.picture.thumbnail {
        thumbnail.hide();
      }
    }

    let hint_height = HINT_HEIGHT * metrics.scale_factor;
    if let Some(hint) = &mut self.hint {
      hint.place(
        &RectF::new(0.0, self.hint_y.value, view.w, hint_height),
        fade,
      );
    }

    if let Some(ghost) = &mut self.ghost {
      let rect = RectF::new(
        ghost.x.value,
        ghost.y.value,
        GHOST_SIZE.0 * metrics.scale_factor,
        GHOST_SIZE.1 * metrics.scale_factor,
      );
      ghost
        .preview
        .place(&rect.inset(metrics.scale_factor * 2.0), 0.9);
      ghost.picture.place(&rect, 1.0);
    }
  }

  /// What is under the overview point (`x`, `y`).
  fn hit(&self, x: f32, y: f32) -> Hit {
    // Mid-zoom, the cards aren't where they will settle.
    let (x, y) = self.camera().unapply(x, y);

    // Later cards stack above earlier ones (e.g. a grown drop target over
    // its neighbour), so they are hit first.
    for (index, card) in self.cards.iter().enumerate().rev() {
      let Some((local_x, local_y)) =
        card.placement().unmap(&self.metrics, x, y)
      else {
        continue;
      };

      let window = card
        .previews
        .iter()
        .rev()
        .find(|preview| preview.tile.rect.contains(local_x, local_y))
        .map(|preview| preview.hwnd);

      return Hit {
        workspace: Some(index),
        window,
      };
    }

    Hit::default()
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
      self.send(action);
      self.is_done |= is_final;
    }

    self.retarget(false);
    self.refresh_pictures(false);
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
      self.interaction.layout = layout;
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
      WM_TOGGLE_OVERVIEW if is_open => self.on_toggle(if wparam.0 == 1 {
        OverviewLayoutMode::Grid
      } else {
        OverviewLayoutMode::Carousel
      }),
      WM_HIDE_OVERVIEW => self.close(),
      WM_ICON_READY => {
        self.icons += 1;
        self.refresh_pictures(false);
      }
      WM_ACTIVATE => {
        // The low word of `wparam` is the activation state.
        if (wparam.0 & 0xffff) == WA_INACTIVE as usize
          && self.frame.is_some()
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

/// Signed client coordinates packed into a mouse message's `LPARAM`.
fn mouse_position(lparam: LPARAM) -> (i32, i32) {
  #[allow(clippy::cast_possible_truncation)]
  let (x, y) = (lparam.0 as i16, (lparam.0 >> 16) as i16);
  (i32::from(x), i32::from(y))
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
