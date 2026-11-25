//! A monitor's screen of the overview: its window, and laying out,
//! drawing and animating what it shows.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::Instant,
};

use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
  },
};

use super::{
  create_screen_window,
  effects::Effects,
  elapsed_ms,
  overlays::Overlays,
  pacing::{draw_order, DrawBudget, FrameTime},
  place_card,
  scene_badges::{BadgeContent, SceneBadge, SceneBadges},
  scene_borders::{SceneBorder, SceneBorders},
  scenes::{scene_content, scene_window_shown},
  to_ms, Card, KeptCard, PlaceContext, Screen, ScreenFrame,
  MAX_KEPT_CARDS,
};
use crate::{
  dwm_thumbnail::Thumbnail,
  overview::{
    chrome::{
      draw_card, draw_plain_card, draw_search, draw_sheet, draw_spotlight,
      picture_size, plain_picture_size, sheet_size, spotlight_margin,
      spotlight_size, TagPicture,
    },
    effects::{lean_toward, Entrance},
    layout::{
      activities, cascade_reach, cascaded, chrome_opacity, stack,
      stack_shift, Camera, CardMetrics, CardView, Placement, RectF,
      SceneChrome, Spring, Zoom, SEARCH_HEIGHT,
    },
    scene::SceneWindow,
    state::{Control, Hit, ScreenFocus, Toward},
    thumbnails::Picture,
  },
  platform_impl::composition::OverviewBackdrop,
  tab_bar::layout::TabHit,
  BorderOverlayParams, OverviewEffects, OverviewExposeHints,
  OverviewExposeSearchScope, OverviewExposeTitles, OverviewLayoutMode,
  OverviewStack, OverviewStackDirection, OverviewStyle,
  OverviewWallpaperStyle, OverviewWindow, OverviewWorkspace,
  OverviewZoomStyle,
};

/// Registers `card`'s thumbnails on `host`: its tab bars under its
/// windows, which cover the strip reaching under them, and its picture
/// over both.
fn register_card(card: &mut Card, host: HWND) {
  for bar in &mut card.tab_bars {
    bar.picture.register(host);
  }
  for preview in &mut card.previews {
    preview.register(host);
  }
  card.picture.register(host);
}

/// The border to draw around scene `window` of `workspace`: the one the
/// WM gives it, unless it shows its own.
fn drawn_border(
  workspace: &OverviewWorkspace,
  window: &SceneWindow,
) -> Option<BorderOverlayParams> {
  if window.preview.has_border() {
    return None;
  }
  workspace
    .window(window.preview.hwnd)
    .and_then(|candidate| candidate.border)
}

/// ID of the next registration of a screen's thumbnails.
static NEXT_REGISTRATION: AtomicU64 = AtomicU64::new(1);

/// How long the wheel rests before the stack settles on the selected
/// workspace, in milliseconds.
const SCROLL_SETTLE_MS: f32 = 140.0;

/// How long the search, highlights and names take to go once a pick is
/// sent, in milliseconds: quick, as an answer to the key or click.
const DISMISS_MS: f32 = 140.0;

/// Furthest the stack scrolls past its first or last workspace, in
/// workspaces, before springing back.
const SCROLL_OVERSHOOT: f32 = 0.2;

/// How much a card grows under the cursor, shrinks while held down, and
/// grows as where a window would drop.
const HOVER_LIFT: f32 = 1.03;
const PRESS_SINK: f32 = 0.96;
const DROP_LIFT: f32 = 1.06;

/// How strongly a card shows during a search none of its windows match.
const SEARCH_EMPTY_CARD: f32 = 0.4;

/// How much a window of the activities' scene grows during a search it
/// matches.
const SEARCH_LIFT: f32 = 0.02;

/// How far what the keyboard is on bumps toward where it can't go, in
/// logical pixels.
const NUDGE_DISTANCE: f32 = 18.0;

/// Most the picked window grows by in the large view.
const MAX_WINDOW_LIFT: f32 = 0.15;

/// How much larger everything is when the fade style starts opening or
/// ends closing.
const FADE_GROWTH: f32 = 0.05;

impl Screen {
  /// A screen with its window hidden, whose pictures post to `notify`
  /// once an icon they show arrives.
  pub(super) fn new(
    notify: (windows::Win32::Foundation::HWND, u32),
  ) -> crate::Result<Self> {
    let layout = OverviewLayoutMode::default();

    // Its springs and marks are timed by the style on each open.
    Ok(Self {
      hwnd: create_screen_window()?,
      frame: None,
      focus: ScreenFocus::default(),
      metrics: CardMetrics::new(
        1.0,
        1.0,
        1.0,
        layout,
        crate::OverviewStack::default().clamped_zoom(),
      ),
      drawn_for: layout,
      registered_for: None,
      pending_registration: None,
      raised: None,
      registration: 0,
      area: RectF::default(),
      work: RectF::default(),
      backdrop: None,
      cards: Vec::new(),
      scenes: Vec::new(),
      scene_area: RectF::default(),
      scene_shown: None,
      gather_home: None,
      is_picked: false,
      scene_borders: SceneBorders::default(),
      scene_badges: SceneBadges::default(),
      card_targets: Vec::new(),
      pointer: None,
      entrance: None,
      spare: Vec::new(),
      kept: Vec::new(),
      overlays: Overlays::new(0.0, 0.0),
      effects: Effects::new(0.0),
      search: None,
      search_drawn: None,
      search_controls: None,
      search_y: Spring::new(0.0, 0.0, 0.25),
      spotlight: None,
      spotlight_drawn: None,
      spotlight_rows: Vec::new(),
      spotlight_y: Spring::new(0.0, 0.0, 0.25),
      spotlight_alpha: Spring::new(0.0, 0.0, 0.002),
      overlays_alpha: Spring::new(1.0, 0.0, 0.002),
      dismiss: Spring::new(1.0, DISMISS_MS, 0.002),
      picked_card: None,
      sheet: None,
      sheet_drawn: None,
      sheet_alpha: Spring::new(0.0, 0.0, 0.002),
      retired: Vec::new(),
      pan: None,
      scroll: 0.0,
      scrolled_at: None,
      nav: Vec::new(),
      zoom: None,
      frame_time: FrameTime::new(Instant::now()),
      is_dirty: false,
      is_gliding: false,
      plain_cards: false,
      cards_behind: false,
      is_tracking_leave: false,
      icons: 0,
      notify,
    })
  }

  /// Its monitor's ID, while shown.
  pub(super) fn id(&self) -> Option<u128> {
    self.frame.as_ref().map(|frame| frame.id)
  }

  pub(super) fn workspaces(&self) -> &[OverviewWorkspace] {
    self
      .frame
      .as_ref()
      .map_or(&[], |frame| frame.workspaces.as_slice())
  }

  pub(super) fn style(&self) -> Option<&OverviewStyle> {
    self.frame.as_ref().map(|frame| &frame.style)
  }

  /// Its own rect, in its client coordinates.
  pub(super) fn view(&self) -> RectF {
    RectF::new(0.0, 0.0, self.metrics.view.0, self.metrics.view.1)
  }

  /// The monitor's working area, in its client coordinates: where the
  /// windows a card shows really are.
  fn work_in_view(&self) -> RectF {
    self.work.translate(-self.area.x, -self.area.y)
  }

  /// Shows `frame` as `focus` has it, zooming out of the workspace it
  /// shows if `animate`.
  pub(super) fn open(
    &mut self,
    frame: ScreenFrame,
    focus: ScreenFocus,
    animate: bool,
  ) {
    self.close();

    self.area = RectF::from_rect(&frame.bounds);
    self.work = RectF::from_rect(&frame.rect);
    self.drawn_for = focus.layout;
    self.metrics = CardMetrics::new(
      self.area.w,
      self.area.h,
      frame.scale_factor,
      focus.layout,
      frame.style.stack.clamped_zoom(),
    );
    self.focus = focus;

    let (open_duration, bounce, stagger_ms) = (
      frame.style.open_duration_ms,
      frame.style.bounce,
      frame.style.effects.stagger_ms,
    );
    self.sync_backdrop(&frame, if animate { 0.0 } else { 1.0 });

    self.frame = Some(frame);
    self.sync_durations();
    self.rebuild_cards();
    self.rebuild_scenes();
    self.ensure_labels();
    if let Some(overlays) = &mut self.overlays {
      overlays.reset();
    }
    if let Some(effects) = &mut self.effects {
      effects.reset();
    }

    // Restarted by `start_clocks` once every screen is set up.
    let started = Instant::now();
    self.frame_time = FrameTime::new(started);
    self.zoom = animate
      .then(|| (Zoom::open(self.focus.selected, open_duration), started));
    self.entrance = (animate && stagger_ms > 0).then(|| {
      let entrance = Entrance::new(
        self.focus.selected,
        to_ms(stagger_ms),
        to_ms(open_duration),
        bounce,
      );
      (entrance, started)
    });

    self.show_backdrop();
    self.retarget(true);
    // Once the cards are in place, which is where the zoom heads.
    self.zoom = self
      .zoom
      .map(|(zoom, started)| (self.trimmed(zoom), started));
    self.register_shown_first();
    self.plain_cards = self.zoom.is_some();
    self.refresh_pictures(true);
    self.is_gliding = self.place();

    let rect = self.area.to_rect();
    // SAFETY: `self.hwnd` is the screen's window, on this thread.
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
    }

    // Open at once, so settled already.
    if self.zoom.is_none() {
      self.celebrate_open();
    }

    self.is_dirty = true;
  }

  /// Starts the open's zoom and entrance at `at`, the overview's first
  /// frame, so setting up doesn't eat into them.
  pub(super) fn start_clocks(&mut self, at: Instant) {
    self.frame_time = FrameTime::new(at);
    if let Some((_, started)) = &mut self.zoom {
      *started = at;
    }
    if let Some((_, started)) = &mut self.entrance {
      *started = at;
    }
  }

  /// Milliseconds from `started` to the frame being drawn.
  fn since(&self, started: Instant) -> f32 {
    self.frame_time.since(started)
  }

  /// Shows a new `frame` as `focus` has it.
  ///
  /// Thumbnails are only registered anew when windows came, went or
  /// moved between workspaces; a title changing redraws its pictures and
  /// nothing else.
  #[allow(clippy::float_cmp)]
  pub(super) fn update(&mut self, frame: ScreenFrame, focus: ScreenFocus) {
    let Some(old) = self.frame.take() else {
      return;
    };

    self.work = RectF::from_rect(&frame.rect);
    if old.bounds != frame.bounds
      || old.scale_factor != frame.scale_factor
      || old.style.stack != frame.style.stack
    {
      self.area = RectF::from_rect(&frame.bounds);
      self.metrics = CardMetrics::new(
        self.area.w,
        self.area.h,
        frame.scale_factor,
        self.drawn_for,
        frame.style.stack.clamped_zoom(),
      );
    }

    self.focus = focus;
    self.sync_backdrop(&frame, self.openness());

    let is_restyled = old.style != frame.style;
    self.frame = Some(frame);
    self.sync_durations();
    let is_changed = self.rebuild_cards() | self.rebuild_scenes();
    self.ensure_labels();
    self.retarget(false);

    if is_changed || self.registered_for != Some(self.focus.layout) {
      self.register_thumbnails();
    }
    self.refresh_pictures(is_restyled);
    self.is_gliding = self.place();

    // Windows moved by the WM can be raised over the overview.
    // SAFETY: `self.hwnd` is the screen's window, on this thread.
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

  /// Lays it out anew for what the user did, as its focus has it now.
  pub(super) fn after_input(&mut self) {
    if self.frame.is_none() {
      return;
    }
    let is_redrawn = self.draw_for_layout();
    self.ensure_labels();
    self.retarget(false);
    // A peeked window grows over the others.
    if let Some(peeked) =
      self.focus.peeked.filter(|h| Some(*h) != self.raised)
    {
      self.raise(Some(peeked));
    }

    if self.registered_for != Some(self.focus.layout) {
      self.register_thumbnails();
    }
    self.refresh_pictures(is_redrawn);
  }

  /// Hides it, keeping its pictures' windows for the next open.
  pub(super) fn close(&mut self) {
    let Some(frame) = self.frame.take() else {
      return;
    };

    // SAFETY: `self.hwnd` is the screen's window, on this thread.
    unsafe {
      ShowWindow(self.hwnd, SW_HIDE);
    }

    self.unregister_thumbnails();
    // Drawn cards are kept for the next open, which then draws only those
    // that changed; the rest go spare.
    let unused = self.kept.drain(..).map(|kept| kept.picture);
    self.spare.extend(unused);
    for (mut card, workspace) in
      self.cards.drain(..).zip(&frame.workspaces)
    {
      let drawn = card.drawn.take();
      let tab_bars = std::mem::take(&mut card.tab_bars);
      self
        .spare
        .extend(tab_bars.into_iter().map(|bar| bar.picture));
      match drawn.filter(|_| self.kept.len() < MAX_KEPT_CARDS) {
        Some(drawn) => {
          card.picture.unregister();
          self.kept.push(KeptCard {
            name: workspace.name.clone(),
            picture: card.picture,
            drawn,
          });
        }
        None => self.spare.extend(card.into_pictures()),
      }
    }
    // Spare pictures would only sit in memory until the next open, to be
    // paged back in when it does: up to a strip wide each, twice over
    // with DWM's copy.
    for picture in &mut self.spare {
      picture.release();
    }
    self.scenes.clear();
    self.scene_shown = None;
    self.gather_home = None;
    self.dismiss.snap(1.0);
    self.picked_card = None;
    self.is_picked = false;
    self.card_targets.clear();
    self.pointer = None;
    self.entrance = None;
    self.registered_for = None;
    self.pending_registration = None;
    self.raised = None;
    self.nav.clear();
    self.pan = None;
    self.scroll = 0.0;
    self.scrolled_at = None;
    self.zoom = None;
    self.sheet_drawn = None;
    self.sheet_alpha.snap(0.0);
    self.is_tracking_leave = false;
  }

  /// Blurs and shows the backdrop as far as it is open now.
  fn show_backdrop(&self) {
    if let Some(backdrop) = &self.backdrop {
      let _ = backdrop.set_progress(self.openness());
      let _ = backdrop.set_opacity(self.presence());
    }
  }

  fn sync_backdrop(&mut self, frame: &ScreenFrame, progress: f32) {
    let result = match &mut self.backdrop {
      Some(backdrop) => {
        backdrop.update(&frame.bounds, &frame.rect, &frame.style)
      }
      None => OverviewBackdrop::create(
        self.hwnd,
        &frame.bounds,
        &frame.rect,
        &frame.style,
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

  /// Times the search line, the shortcut sheet and the overlays by the
  /// style's durations; cards and scenes take theirs as they are rebuilt.
  fn sync_durations(&mut self) {
    let Some(style) = self.style() else {
      return;
    };
    let switch_ms = to_ms(style.switch_duration_ms);
    let highlight_ms = to_ms(style.highlight_duration_ms);
    let bounce = style.bounce;

    self.search_y.retune(switch_ms);
    self.spotlight_y.retune(switch_ms);
    self.spotlight_alpha.retune(highlight_ms);
    self.overlays_alpha.retune(highlight_ms);
    self.sheet_alpha.retune(highlight_ms);
    if let Some(overlays) = &mut self.overlays {
      overlays.retune(switch_ms, highlight_ms, bounce);
    }
    if let Some(effects) = &mut self.effects {
      effects.retune(highlight_ms);
    }
  }

  /// Draws the cards for the layout now shown, at the size it shows them,
  /// if they were drawn for another, and starts the windows it shows on
  /// their own from where their cards showed them. Returns whether it
  /// did.
  fn draw_for_layout(&mut self) -> bool {
    let layout = self.focus.layout;
    let previous = self.drawn_for;
    if layout == previous {
      return false;
    }

    let metrics = CardMetrics::new(
      self.area.w,
      self.area.h,
      self.metrics.scale_factor,
      layout,
      self.style().map_or_else(
        || crate::OverviewStack::default().clamped_zoom(),
        |style| style.stack.clamped_zoom(),
      ),
    );
    let is_redrawn = metrics != self.metrics;
    self.drawn_for = layout;

    // E.g. the activities and the exposé draw their cards alike.
    if is_redrawn {
      let widths = self
        .cards
        .iter()
        .map(|card| card.metrics.width)
        .collect::<Vec<_>>();
      self.metrics = metrics;
      self.remap_cards();

      // Each card stays the size it is on screen, in its new units.
      for (card, width) in self.cards.iter_mut().zip(widths) {
        card.scale.rescale(width / card.metrics.width.max(1.0));
      }
    }

    if layout.has_scene() && !previous.has_scene() {
      self.seed_scene_from_card(false);
    }

    is_redrawn
  }

  /// Registers every thumbnail the layout shows again, bottom to top:
  /// the windows on the scenes, each card's tab bars, previews and
  /// picture, the labels, highlights and tags, then the search line and
  /// the shortcut sheet.
  fn register_thumbnails(&mut self) {
    // Those shown now stay up until their replacements are placed, so
    // nothing blinks out in between.
    let retired = self.retire_thumbnails();
    self.retired.extend(retired);
    self.pending_registration = None;
    self.register_rest(None);
  }

  /// Registers, on open, only what zooms before the first frame: the
  /// shown workspace's previews (its scene's windows, or its card in the
  /// stack). The rest follows on the next frame (see `tick`), so the
  /// first frame comes a round trip to DWM per thumbnail sooner. Its
  /// thumbnails then sit under the rest, other scenes' windows included,
  /// which only meet it sliding in.
  fn register_shown_first(&mut self) {
    let retired = self.retire_thumbnails();
    self.retired.extend(retired);

    let (host, shown) = (self.hwnd, self.focus.selected);
    if self.focus.layout.has_scene() {
      if let Some(scene) = self.scenes.get_mut(shown) {
        for window in &mut scene.windows {
          window.preview.register(host);
        }
      }
    } else if let Some(card) = self.cards.get_mut(shown) {
      register_card(card, host);
    }
    self.pending_registration = Some(shown);
    self.registered_for = Some(self.focus.layout);
    self.registration = NEXT_REGISTRATION.fetch_add(1, Ordering::Relaxed);
  }

  /// Registers every thumbnail, but those of workspace `registered`
  /// registered already, above everything registered so far.
  fn register_rest(&mut self, registered: Option<usize>) {
    let host = self.hwnd;
    let layout = self.focus.layout;
    let raised = self.raised;
    if layout.has_scene() {
      for window in self
        .scenes
        .iter_mut()
        .enumerate()
        .filter(|(index, _)| Some(*index) != registered)
        .flat_map(|(_, scene)| &mut scene.windows)
        .filter(|window| Some(window.preview.hwnd) != raised)
      {
        window.preview.register(host);
      }
      let bordered = self
        .workspaces()
        .iter()
        .zip(&self.scenes)
        .flat_map(|(workspace, scene)| {
          scene
            .windows
            .iter()
            .filter(|window| drawn_border(workspace, window).is_some())
        })
        .count();
      self.scene_borders.register(host, bordered);

      let windows = self
        .scenes
        .iter()
        .map(|scene| scene.windows.len())
        .sum::<usize>();
      let badges = self.badges_per_window() * windows;
      self.scene_badges.register(host, badges);
    }

    let is_card_registered = !layout.has_scene();
    for (index, card) in self.cards.iter_mut().enumerate() {
      if !(is_card_registered && Some(index) == registered) {
        register_card(card, host);
      }
    }

    // Over the cards, under the highlights.
    self.register_effects();
    if let Some(overlays) = &mut self.overlays {
      overlays.register(host, layout);
    }

    // Over everything else, the search box over the search line, and the
    // sheet over both.
    for picture in [&mut self.search, &mut self.spotlight, &mut self.sheet]
    {
      if picture.is_none() {
        *picture = Picture::new();
      }
      if let Some(picture) = picture {
        picture.register(host);
      }
    }

    // The window zoomed into on close, over everything else.
    if let Some(window) = self.raised_window() {
      window.preview.register(host);
    }

    self.registered_for = Some(layout);
    self.registration = NEXT_REGISTRATION.fetch_add(1, Ordering::Relaxed);
  }

  pub(super) fn unregister_thumbnails(&mut self) {
    self.retired.clear();
    drop(self.retire_thumbnails());
  }

  /// Takes every thumbnail out of what shows it, still registered.
  fn retire_thumbnails(&mut self) -> Vec<Thumbnail> {
    let mut retired = Vec::new();

    for card in &mut self.cards {
      card.picture.retire(&mut retired);
      for bar in &mut card.tab_bars {
        bar.picture.retire(&mut retired);
      }
      for preview in &mut card.previews {
        preview.retire(&mut retired);
      }
    }
    for window in
      self.scenes.iter_mut().flat_map(|scene| &mut scene.windows)
    {
      window.preview.retire(&mut retired);
    }
    self.scene_borders.retire(&mut retired);
    self.scene_badges.retire(&mut retired);
    if let Some(effects) = &mut self.effects {
      effects.retire(&mut retired);
    }
    if let Some(overlays) = &mut self.overlays {
      overlays.retire(&mut retired);
    }
    for picture in [&mut self.search, &mut self.spotlight, &mut self.sheet]
      .into_iter()
      .flatten()
    {
      picture.retire(&mut retired);
    }

    retired
  }

  /// Points every card and scene window at where it belongs now.
  fn retarget(&mut self, snap: bool) {
    let slots = self.slots();
    let selected = self.focus.selected;

    let targets = match self.focus.layout {
      OverviewLayoutMode::Stack => {
        // A pan is the selected workspace's own: a newly selected one
        // starts out on what its monitor shows.
        let pan = self
          .pan
          .filter(|(index, _)| *index == selected)
          .map_or(0.0, |(_, pan)| pan);
        let look = self
          .style()
          .map_or_else(OverviewStack::default, |style| style.stack);
        let (mut targets, pan) =
          stack(&self.metrics, &slots, selected, pan, &look);
        self.pan = Some((selected, pan));

        // Scrolled part of the way to the next or previous workspace.
        let shift = stack_shift(
          &self.metrics,
          &targets,
          selected,
          self.scroll,
          &look,
        );
        let is_horizontal =
          look.direction == OverviewStackDirection::Horizontal;
        for card in &mut targets.cards {
          if is_horizontal {
            card.cx -= shift;
          } else {
            card.cy -= shift;
          }
        }
        targets
      }
      OverviewLayoutMode::Activities | OverviewLayoutMode::Expose => {
        let (targets, area) =
          activities(&self.metrics, &slots, self.scene_chrome());
        self.scene_area = area.inset(self.frame_padding());
        targets
      }
    };

    self.retarget_cards(&targets.cards, snap);

    self.search_y.aim(targets.search_y, snap);
    self.spotlight_y.aim(targets.spotlight_y, snap);

    if self.focus.layout.has_scene() {
      self.retarget_scenes(snap);
    }
    self.sync_nav();
    self.is_dirty = true;
  }

  /// Points each card at its target in `targets`; there at once if
  /// `snap`.
  fn retarget_cards(&mut self, targets: &[Placement], snap: bool) {
    self.card_targets = targets.to_vec();
    self.aim_cards(snap);
  }

  /// Points each card at where the layout puts it, lifted under the
  /// cursor, sunk while held down or grown as a drop target, leaning
  /// toward the cursor and dimmed while another is hovered, as the style
  /// has it; there at once if `snap`.
  fn aim_cards(&mut self, snap: bool) {
    // A closing screen's cards hold still for its zoom.
    if self.is_closing() {
      return;
    }

    let effects = self
      .style()
      .map_or(OverviewEffects::NONE, |style| style.effects);
    let lean = self.metrics.px(effects.lean.max(0.0));
    let dim = 1.0 - effects.dim.clamp(0.0, 1.0);
    let focus = &self.focus;
    let pointer = self.pointer;

    let lift = |index: usize| {
      // The activities show the selected workspace in the scene, which
      // its miniature doesn't answer to the mouse for.
      let is_still = focus.is_dragging
        || (focus.layout.has_scene() && index == focus.selected);

      if focus.drop == Some(index) {
        DROP_LIFT
      } else if is_still {
        1.0
      } else if focus.pressed == Some(index) {
        PRESS_SINK
      } else if focus.hover == Some(index) && index != focus.selected {
        HOVER_LIFT
      } else {
        1.0
      }
    };
    let dims = |index: usize| {
      focus.hover.is_some_and(|hovered| {
        hovered != index && index != focus.selected && !focus.is_dragging
      })
    };
    let misses = (0..self.cards.len())
      .map(|index| self.misses_search(index))
      .collect::<Vec<_>>();

    for (index, (card, target)) in
      self.cards.iter_mut().zip(&self.card_targets).enumerate()
    {
      let lift = lift(index);
      let scale = target.scale * lift;
      let (dx, dy) = pointer
        .filter(|_| lift == HOVER_LIFT && lean > 0.0)
        .map_or((0.0, 0.0), |pointer| {
          let size =
            (card.metrics.width * scale, card.metrics.height * scale);
          lean_toward((target.cx, target.cy), size, pointer, lean)
        });
      let mut opacity = if dims(index) {
        target.opacity * dim
      } else {
        target.opacity
      };
      if misses[index] {
        opacity *= SEARCH_EMPTY_CARD;
      }

      for (spring, value) in [
        (&mut card.cx, target.cx + dx),
        (&mut card.cy, target.cy + dy),
        (&mut card.scale, scale),
        (&mut card.alpha, opacity),
      ] {
        spring.aim(value, snap);
      }
    }
    self.is_dirty = true;
  }

  /// Notes where the cursor is over it, if it is, and leans the hovered
  /// card toward it.
  pub(super) fn point_at(&mut self, pointer: Option<(f32, f32)>) {
    if self.pointer == pointer {
      return;
    }
    self.pointer = pointer;

    let leans = self.style().is_some_and(|style| style.effects.lean > 0.0);
    if leans && self.focus.hover.is_some() && self.frame.is_some() {
      self.aim_cards(false);
    }
  }

  /// What the activities and the exposé show around their scene, as the
  /// style has it.
  pub(super) fn scene_chrome(&self) -> SceneChrome {
    self.style().map_or(SceneChrome::FULL, |style| {
      let (row, side) = style.row(self.focus.layout);
      SceneChrome {
        row,
        side,
        has_search_line: style.search_hint,
        margin: match self.focus.layout {
          OverviewLayoutMode::Activities => {
            style.activities.clamped_margin()
          }
          OverviewLayoutMode::Expose | OverviewLayoutMode::Stack => {
            style.expose.clamped_margin()
          }
        },
      }
    })
  }

  /// Bumps what the keyboard is on `toward` and lets it spring back, for
  /// an arrow or the wheel finding nowhere to go that way.
  pub(super) fn nudge(&mut self, toward: Toward) {
    if self.frame.is_none() || self.zoom.is_some() {
      return;
    }

    let distance = self.metrics.px(NUDGE_DISTANCE);
    let (dx, dy) = match toward {
      Toward::Left => (-distance, 0.0),
      Toward::Right => (distance, 0.0),
      Toward::Up => (0.0, -distance),
      Toward::Down => (0.0, distance),
    };

    let selected = self.focus.selected;
    if let Some(card) = self.cards.get_mut(selected) {
      card.cx.kick(dx);
      card.cy.kick(dy);
    }
    if self.focus.layout.has_scene() {
      if let Some(scene) = self.scenes.get_mut(selected) {
        for window in &mut scene.windows {
          window.kick(dx, dy);
        }
      }
    }
    self.is_dirty = true;
  }

  /// Whether the wheel scrolls it smoothly, as the stack does, rather than
  /// stepping a workspace a notch.
  pub(super) fn scrolls_smoothly(&self) -> bool {
    self.focus.layout == OverviewLayoutMode::Stack
      && self.zoom.is_none()
      && self.frame.is_some()
  }

  /// Scrolls the stack `workspaces` along, toward the next for a positive
  /// one. Returns the way it has scrolled past halfway to a neighbour, if
  /// it has, for that one to be selected (see `scrolled_onto`).
  pub(super) fn scroll_by(&mut self, workspaces: f32) -> Option<bool> {
    self.scroll += workspaces;
    self.scrolled_at = Some(Instant::now());
    self.retarget(false);
    self.scroll_step()
  }

  /// The way the stack has scrolled past halfway to a neighbour, if it
  /// has.
  pub(super) fn scroll_step(&self) -> Option<bool> {
    if self.scroll >= 0.5 {
      Some(true)
    } else if self.scroll <= -0.5 {
      Some(false)
    } else {
      None
    }
  }

  /// Takes a step scrolled toward the next workspace when `is_down` off
  /// the scroll if that one is now selected, or else holds the scroll
  /// short of it, at an end of the stack.
  pub(super) fn scrolled_onto(
    &mut self,
    is_down: bool,
    is_selected: bool,
  ) {
    let way = if is_down { 1.0 } else { -1.0 };
    self.scroll = if is_selected {
      self.scroll - way
    } else {
      way * self.scroll.abs().min(SCROLL_OVERSHOOT)
    };
    self.retarget(false);
  }

  /// Pans the stack's selected strip `distance` monitor widths along,
  /// e.g. for the wheel turned sideways.
  pub(super) fn pan_by(&mut self, distance: f32) {
    let is_still = self.zoom.is_some() || self.frame.is_none();
    if self.focus.layout != OverviewLayoutMode::Stack || is_still {
      return;
    }

    let selected = self.focus.selected;
    let pan = self
      .pan
      .filter(|(index, _)| *index == selected)
      .map_or(0.0, |(_, pan)| pan);
    let monitor = self.metrics.tile_area().w;
    self.pan = Some((selected, pan + distance * monitor));
    self.retarget(false);
  }

  /// Notes where the shown workspace's windows show once settled, first
  /// to last, for the arrows to go between them.
  fn sync_nav(&mut self) {
    let selected = self.focus.selected;

    let nav = if self.focus.layout.has_scene() {
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

    self.nav = nav.unwrap_or_default();
  }

  /// Redraws every picture whose content changed, or all of them when
  /// `force`.
  pub(super) fn refresh_pictures(&mut self, force: bool) {
    let Some(frame) = &self.frame else {
      return;
    };
    let style = frame.style.clone();
    let notify = self.notify;
    let context = self.place_context();
    let layout = self.focus.layout;

    let count = self.cards.len().min(self.workspaces().len());
    let mut budget = DrawBudget::new(force);

    for index in draw_order(count, self.focus.selected) {
      let picture = self.card_picture(index, &style);
      let Some(card) = self.cards.get_mut(index) else {
        continue;
      };

      for (preview, tile) in card.previews.iter_mut().zip(&picture.tiles) {
        preview.opacity = tile.opacity;
      }

      // A hidden tab picked with the keyboard (e.g. found by a search)
      // shows on its stack's bar.
      let cursor = self.focus.selected_window;
      for bar in &mut card.tab_bars {
        let hover = cursor
          .and_then(|hwnd| bar.tab_index(hwnd))
          .filter(|index| *index != bar.frame.active_index)
          .map_or(TabHit::Empty, TabHit::Tab);
        bar.draw(hover, card.icons, notify);
      }

      // Kept from the last open as it is still, it shows as it is.
      if self.plain_cards && card.drawn.as_ref() != Some(&picture) {
        let metrics = card.metrics;
        let size = plain_picture_size(&metrics);
        if card.picture.size() != size {
          card.picture.draw(size, |surface| {
            draw_plain_card(surface, &metrics, &style);
          });
          card.drawn = None;
          place_card(card, index, &context);
        }
        continue;
      }
      if self.plain_cards {
        continue;
      }
      if !force && card.drawn.as_ref() == Some(&picture) {
        continue;
      }
      if !budget.take() {
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
      let picked =
        Some(self.focus.selected).zip(self.focus.selected_window);
      let window_lift =
        style.effects.window_lift.clamp(0.0, MAX_WINDOW_LIFT);
      for index in 0..self.scenes.len() {
        let looks = self.scenes[index]
          .windows
          .iter()
          .map(|window| {
            let hwnd = window.preview.hwnd;
            let search_lift = if self.matches_search(index, hwnd) {
              SEARCH_LIFT
            } else {
              0.0
            };
            let pick_lift = if picked == Some((index, hwnd)) {
              window_lift
            } else {
              0.0
            };
            let lift = search_lift.max(pick_lift);
            (self.window_opacity(index, hwnd), lift)
          })
          .collect::<Vec<_>>();
        for (window, (opacity, lift)) in
          self.scenes[index].windows.iter_mut().zip(looks)
        {
          window.preview.opacity = opacity;
          window.lift.aim(lift, false);
        }
      }

      self.draw_placeholders(&style, force);
    }

    self.refresh_overlays(&style, force);
    if let Some(effects) = &mut self.effects {
      effects.draw_looks(&style, &self.metrics);
    }

    self.refresh_search(&style, force);
    self.refresh_spotlight(&style, force);
    self.refresh_sheet(&style, force);
    self.cards_behind = budget.is_behind();
    self.is_dirty = true;
  }

  /// Redraws what shows `window`'s icon, which just arrived: the cards
  /// it's on, and the tag, search and placeholders, which show any
  /// window's.
  pub(super) fn icon_ready(&mut self, window: isize) {
    self.icons += 1;
    let showing = self
      .workspaces()
      .iter()
      .map(|workspace| workspace.window(window).is_some())
      .collect::<Vec<_>>();
    for (card, is_showing) in self.cards.iter_mut().zip(showing) {
      if is_showing {
        card.icons += 1;
      }
    }
    self.refresh_pictures(false);
  }

  /// Draws the placeholders of the scenes' minimized windows, unless they
  /// show that already, or all of them when `force`.
  fn draw_placeholders(&mut self, style: &OverviewStyle, force: bool) {
    let titles = self
      .workspaces()
      .iter()
      .map(|workspace| {
        workspace
          .windows
          .iter()
          .filter(|window| window.is_minimized)
          .map(|window| (window.hwnd, window.display_title()))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    let (scale_factor, notify) = (self.metrics.scale_factor, self.notify);
    let spreads = self.focus.layout.spreads_windows();

    for (scene, titles) in self.scenes.iter_mut().zip(&titles) {
      for window in scene.windows.iter_mut().filter(|w| w.is_minimized) {
        let hwnd = window.preview.hwnd;
        let Some((_, title)) = titles.iter().find(|(h, _)| *h == hwnd)
        else {
          continue;
        };
        if force {
          window.preview.forget_placeholder_drawn();
        }
        let target = window.target();
        let shown = spreads.then_some((target.w, target.h));
        window.preview.draw_placeholder(
          title,
          scale_factor,
          shown,
          style,
          notify,
        );
      }
    }
  }

  /// Redraws the search line if it changed, or when `force`.
  fn refresh_search(&mut self, style: &OverviewStyle, force: bool) {
    let drawn =
      (self.focus.search.clone(), self.focus.shortcuts.is_some());
    if !force && self.search_drawn.as_ref() == Some(&drawn) {
      return;
    }

    let scale_factor = self.metrics.scale_factor;
    #[allow(clippy::cast_possible_truncation)]
    let size = (
      self.metrics.view.0.ceil() as i32,
      (SEARCH_HEIGHT * scale_factor).ceil() as i32,
    );

    if let Some(picture) = &mut self.search {
      let (search, shows_shortcuts) = &drawn;
      picture.draw(size, |surface| {
        self.search_controls = Some(draw_search(
          surface,
          search,
          *shows_shortcuts,
          scale_factor,
          style,
        ));
      });
    }
    self.search_drawn = Some(drawn);
  }

  /// Redraws the search box if the search or the icons changed, or when
  /// `force`, and fades it in or out as a search starts or ends.
  fn refresh_spotlight(&mut self, style: &OverviewStyle, force: bool) {
    let search = &self.focus.search;
    let is_searching = !search.query.is_empty();
    let drawn = (search.clone(), self.icons);
    if !force && self.spotlight_drawn.as_ref() == Some(&drawn) {
      return;
    }

    // Ending keeps what it showed, to fade out.
    if let (true, Some(picture)) = (is_searching, &mut self.spotlight) {
      let scale_factor = self.metrics.scale_factor;
      let size =
        spotlight_size(&drawn.0, scale_factor, self.metrics.view.0);
      let notify = self.notify;
      picture.draw(size, |surface| {
        self.spotlight_rows =
          draw_spotlight(surface, &drawn.0, scale_factor, style, notify);
      });
    }
    self.spotlight_alpha.target = if is_searching { 1.0 } else { 0.0 };
    self.spotlight_drawn = Some(drawn);
  }

  /// Redraws the shortcut sheet if what it lists changed, or when
  /// `force`, and fades it in or out as it opens or closes.
  fn refresh_sheet(&mut self, style: &OverviewStyle, force: bool) {
    let groups = self.focus.shortcuts.clone();
    if !force && self.sheet_drawn == groups {
      return;
    }

    // Closing keeps what it showed, to fade out.
    if let (Some(groups), Some(picture)) = (&groups, &mut self.sheet) {
      let scale_factor = self.metrics.scale_factor;
      let size = sheet_size(groups, scale_factor, style);
      picture.draw(size, |surface| {
        draw_sheet(surface, groups, scale_factor, style);
      });
    }
    self.sheet_alpha.target = if groups.is_some() { 1.0 } else { 0.0 };
    self.sheet_drawn = groups;
  }

  /// How far it is open, from 0 (what a card shows fills the screen as
  /// the desktop shows it) to 1 (every card in place). Runs a little past
  /// either end for a moment on a curve that overshoots.
  pub(super) fn openness(&self) -> f32 {
    let Some((zoom, started)) = self.zoom else {
      return 1.0;
    };
    self.style().map_or(1.0, |style| {
      zoom.openness(self.since(started), style.zoom_curve(zoom.is_closing))
    })
  }

  /// Whether it is zooming into the workspace it closes on.
  pub(super) fn is_closing(&self) -> bool {
    self.zoom.is_some_and(|(zoom, _)| zoom.is_closing)
  }

  /// Whether its zoom into the workspace it closes on has ended.
  pub(super) fn is_zoomed_in(&self) -> bool {
    self.zoom.is_some_and(|(zoom, started)| {
      zoom.is_closing && zoom.is_finished(self.since(started))
    })
  }

  /// What everything on it is seen through at the current openness: for
  /// the zoom, from where the zoomed workspace shows its monitor to the
  /// whole screen, so all of it scales as one; for the fade, a little
  /// larger the less open.
  pub(super) fn camera(&self) -> Camera {
    self.zoom.map_or(Camera::IDENTITY, |(zoom, _)| {
      self.zoom_camera(zoom.card, self.openness())
    })
  }

  /// The camera zooming out of or into card `index` at `openness`.
  fn zoom_camera(&self, index: usize, openness: f32) -> Camera {
    let layout = self.focus.layout;
    match self.style().map(|style| style.zoom_style_of(layout).0) {
      Some(OverviewZoomStyle::Zoom) => {
        self.zoom_rect(index).map_or(Camera::IDENTITY, |rect| {
          Camera::zoom(&rect, &self.work_in_view(), openness)
        })
      }
      Some(OverviewZoomStyle::Fade) => {
        Camera::grown(&self.view(), FADE_GROWTH * (1.0 - openness))
      }
      Some(OverviewZoomStyle::None) | None => Camera::IDENTITY,
    }
  }

  /// The camera each card is seen through on a cascading zoom: those
  /// further from the zoomed card a step behind it, so they come in after
  /// it on open and leave before it on close. `None` without one.
  fn cascade_cameras(&self) -> Option<Vec<Camera>> {
    let (zoom, _) = self.zoom?;
    let cascades = self
      .style()
      .is_some_and(|style| style.zoom_style_of(self.focus.layout).1);
    if !cascades {
      return None;
    }
    let openness = self.openness();
    Some(
      (0..self.cards.len())
        .map(|index| {
          let steps = index.abs_diff(zoom.card);
          self.zoom_camera(zoom.card, cascaded(openness, steps))
        })
        .collect(),
    )
  }

  /// `zoom`, ending once nothing on screen is more than half a pixel from
  /// where it lands, on a spring.
  fn trimmed(&self, zoom: Zoom) -> Zoom {
    self.style().map_or(zoom, |style| {
      let mut travel =
        self.zoom_camera(zoom.card, 0.0).travel(&self.view());
      if style.zoom_style_of(self.focus.layout).1 {
        travel = cascade_reach(travel);
      }
      zoom.trimmed(style.zoom_curve(zoom.is_closing), travel)
    })
  }

  /// Where workspace `index` shows its monitor's working area, on its
  /// card or in the activities' scene: a scrolling workspace's strip
  /// reaches past it. Follows its card if that is still moving.
  fn zoom_rect(&self, index: usize) -> Option<RectF> {
    match self.focus.layout {
      OverviewLayoutMode::Activities | OverviewLayoutMode::Expose => {
        let workspace = self.workspaces().get(index)?;
        let content = scene_content(workspace, self.frame.as_ref()?);
        Some(CardView::fitted(&content, &self.scene_area).map(&self.work))
      }
      OverviewLayoutMode::Stack => {
        let card = self.cards.get(index)?;
        Some(
          card
            .placement()
            .map(&card.metrics, &card.view.map(&self.work)),
        )
      }
    }
  }

  /// How strongly it shows over the desktop as a whole.
  fn presence(&self) -> f32 {
    let fades = self.style().is_some_and(|style| {
      style.zoom_style_of(self.focus.layout).0 == OverviewZoomStyle::Fade
    });
    if fades && self.zoom.is_some() {
      self.openness().clamp(0.0, 1.0)
    } else {
      1.0
    }
  }

  /// Zooms into card `index` until what it shows fills the screen, over
  /// `duration_ms`, showing `focus`.
  ///
  /// The cards hold still meanwhile, so the zoom heads straight for the
  /// picked one rather than veering off with the layout (e.g. the stack
  /// panning to the pick). The activities' scene of a workspace not shown
  /// yet changes over as it would have, its windows landing where they
  /// settle as the zoom ends (see `scene_window_rect`).
  pub(super) fn start_closing(
    &mut self,
    index: usize,
    focus: ScreenFocus,
    duration_ms: u32,
  ) {
    let is_switching =
      focus.layout.has_scene() && index != self.focus.selected;
    self.focus = focus;
    self.picked_card = None;
    self.entrance = None;
    let zoom =
      self.trimmed(Zoom::close(index, self.openness(), duration_ms));
    self.zoom = Some((zoom, self.frame_time.start(Instant::now())));

    // Once zooming, so the workspace closed on comes in where it lands.
    if is_switching {
      self.retarget(false);
    }
    self.hold();
    // Spread windows head back to where they really are.
    if self.focus.layout.spreads_windows() {
      self.retarget_scenes(false);
    }
    self.raise_picked(index);
    self.celebrate_exit(index, zoom.duration());

    let shown_windows = self.scenes.get(index).map(|scene| {
      scene
        .windows
        .iter()
        .filter(|window| window.alpha.value > 0.01)
        .count()
    });
    tracing::debug!(
      "Overview screen zooming into card {index} ({:?}, switching: \
       {is_switching}) from openness {:.2}; card on screen at {:?}, \
       scene windows shown: {shown_windows:?}.",
      self.focus.layout,
      self.openness(),
      self
        .cards
        .get(index)
        .map(|card| card.placement().rect(&card.metrics)),
    );
    self.is_dirty = true;
  }

  /// Registers the window picked on scene `index` anew, above everything
  /// else, so it flies home over the windows around it as the one coming
  /// to the front. Its old thumbnails stay up until the new ones are
  /// placed, so it doesn't blink.
  fn raise_picked(&mut self, index: usize) {
    let picked = self
      .picked_window()
      .filter(|(card, _)| *card == index && self.focus.layout.has_scene())
      .map(|(_, hwnd)| hwnd);
    // Raised already on the pick, while the WM carried it out.
    if picked.is_none() || picked != self.raised {
      self.raise(picked);
    }
  }

  /// Takes in that a pick landing on card `card` was sent: its own
  /// drawing goes at once, the picked window rises, and what won't land
  /// fades where it is, while the WM carries the pick out.
  pub(super) fn commit_pick(&mut self, card: Option<usize>) {
    self.is_picked = true;
    self.picked_card = card;
    self.dismiss.aim(0.0, false);
    if let Some(card) = card {
      self.raise_picked(card);
    }
    if self.focus.layout.has_scene() {
      self.retarget_scenes(false);
    }
    self.is_dirty = true;
  }

  /// Raises scene window `hwnd` over everything else, if any.
  fn raise(&mut self, hwnd: Option<isize>) {
    self.raised = hwnd;
    let host = self.hwnd;
    let mut retired = Vec::new();
    if let Some(window) = self.raised_window() {
      window.preview.retire(&mut retired);
      window.preview.register(host);
    }
    self.retired.extend(retired);
    self.is_dirty = true;
  }

  /// The scene window raised over everything else, if any.
  fn raised_window(&mut self) -> Option<&mut SceneWindow> {
    let hwnd = self.raised?;
    self
      .scenes
      .iter_mut()
      .flat_map(|scene| &mut scene.windows)
      .find(|window| window.preview.hwnd == hwnd)
  }

  /// Card it zooms into to close on, which holds still meanwhile.
  pub(super) fn closing_card(&self) -> Option<usize> {
    self
      .zoom
      .filter(|(zoom, _)| zoom.is_closing)
      .map(|(zoom, _)| zoom.card)
  }

  /// Stops the cards and the shown workspace's area where they are now.
  fn hold(&mut self) {
    for card in &mut self.cards {
      for spring in [&mut card.cx, &mut card.cy, &mut card.scale] {
        spring.snap(spring.value);
      }
    }
    if let Some(overlays) = &mut self.overlays {
      overlays.workspace_rect.hold();
    }
  }

  pub(super) fn place_context(&self) -> PlaceContext {
    PlaceContext {
      camera: self.camera(),
      fade: self.openness().clamp(0.0, 1.0),
      chrome: chrome_opacity(
        self.openness(),
        self.style().map_or(0.0, |style| style.chrome_from),
      ) * self.dismiss.value,
      overlays: if self.zoom.is_some() {
        0.0
      } else {
        self.overlays_alpha.value
      },
      active: self.frame.as_ref().and_then(|frame| frame.focused_window),
      presence: self.presence(),
      entrance: self
        .entrance
        .map(|(entrance, started)| (entrance, self.since(started))),
      view: self.view(),
      zoom_card: self.zoom.map(|(zoom, _)| zoom.card),
    }
  }

  /// Steps its animations by `dt` seconds to the frame shown at `now`,
  /// and places what moved. Returns whether anything is still moving.
  pub(super) fn tick(&mut self, dt: f32, now: Instant) -> bool {
    if self.frame.is_none() {
      return false;
    }
    self.frame_time.advance(now);

    // The thumbnails an open left for its second frame.
    let mut is_moving = false;
    if let Some(shown) = self.pending_registration.take() {
      self.register_rest(Some(shown));
      self.is_dirty = true;
      is_moving = true;
    }
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
    if let Some(effects) = &mut self.effects {
      is_moving |= effects.step(dt);
    }
    is_moving |= self.search_y.step(dt);

    if let Some((entrance, started)) = self.entrance {
      if self.since(started) < entrance.length_ms(self.cards.len()) {
        is_moving = true;
      } else {
        self.entrance = None;
        self.is_dirty = true;
      }
    }
    // Once the wheel rests, the stack settles on the selected workspace.
    if self.scroll != 0.0 {
      is_moving = true;
      if self
        .scrolled_at
        .is_none_or(|at| elapsed_ms(at) >= SCROLL_SETTLE_MS)
      {
        self.scroll = 0.0;
        self.scrolled_at = None;
        self.retarget(false);
      }
    }

    is_moving |= self.sheet_alpha.step(dt);
    is_moving |= self.spotlight_y.step(dt);
    is_moving |= self.spotlight_alpha.step(dt);
    // Hidden while a zoom moves everything, in once it rests.
    if self.zoom.is_some() {
      self.overlays_alpha.snap(0.0);
    } else {
      self.overlays_alpha.target = 1.0;
    }
    is_moving |= self.overlays_alpha.step(dt);
    is_moving |= self.dismiss.step(dt);

    if let Some((zoom, started)) = self.zoom {
      is_moving = true;
      if zoom.is_finished(self.since(started)) {
        // Zoomed in, it stays as it is until the overview hides.
        if zoom.is_closing {
          return false;
        }
        self.zoom = None;
        self.celebrate_open();
        if self.plain_cards {
          self.plain_cards = false;
          self.refresh_pictures(false);
        }
      }
      self.show_backdrop();
    } else if self.cards_behind {
      is_moving = true;
      self.refresh_pictures(false);
    }

    if is_moving || self.is_dirty || self.is_gliding {
      self.is_gliding = self.place();
      self.is_dirty = false;
    }

    is_moving || self.is_gliding
  }

  /// Places every thumbnail where its animation has it now. Returns
  /// whether a highlight is still settling.
  fn place(&mut self) -> bool {
    let context = self.place_context();
    let (view, scale_factor) = (context.view, self.metrics.scale_factor);
    let layout = self.focus.layout;
    self.place_backdrop(&context);

    // The activities' miniatures are too small to zoom out of: what the
    // zoom sets off from is the scene.
    let cards = PlaceContext {
      zoom_card: context.zoom_card.filter(|_| !layout.has_scene()),
      ..context
    };
    let cameras = self.cascade_cameras();
    for (index, card) in self.cards.iter_mut().enumerate() {
      let camera = cameras
        .as_ref()
        .and_then(|cameras| cameras.get(index))
        .copied()
        .unwrap_or(cards.camera);
      place_card(card, index, &PlaceContext { camera, ..cards });
    }

    if layout.has_scene() {
      self.place_scenes(&context);
    }

    let is_gliding =
      self.place_overlays(&context) | self.place_effects(&context);

    let search_height = SEARCH_HEIGHT * scale_factor;
    let has_search =
      self.search_drawn.is_some() && self.scene_chrome().has_search_line;
    if let Some(search) = &mut self.search {
      if has_search {
        search.place(
          &RectF::new(0.0, self.search_y.value, view.w, search_height),
          &view,
          context.chrome,
        );
      } else {
        search.hide();
      }
    }

    let spotlight_rect = self.spotlight_rect();
    let spotlight_opacity = context.chrome * self.spotlight_alpha.value;
    if let Some(spotlight) = &mut self.spotlight {
      match spotlight_rect {
        Some(rect) if spotlight_opacity > 0.002 => {
          spotlight.place(&rect, &view, spotlight_opacity);
        }
        _ => spotlight.hide(),
      }
    }

    let sheet_rect = self.sheet_rect();
    let sheet_opacity = context.chrome * self.sheet_alpha.value;
    if let Some(sheet) = &mut self.sheet {
      match sheet_rect {
        Some(rect) if sheet_opacity > 0.002 => {
          sheet.place(&rect, &view, sheet_opacity);
        }
        _ => sheet.hide(),
      }
    }

    // Their replacements are in place now.
    self.retired.clear();

    is_gliding
  }

  /// Puts the framed backdrop's panel where `backdrop_panel` has it, its
  /// corners rounding off as it shrinks out of the full screen.
  fn place_backdrop(&self, context: &PlaceContext) {
    let (Some(backdrop), Some(style)) = (&self.backdrop, self.style())
    else {
      return;
    };
    if !backdrop.is_framed() {
      return;
    }
    let Some(panel) = self.backdrop_panel(&context.camera) else {
      return;
    };

    let scale_factor = self.metrics.scale_factor;
    let fade = context.fade.clamp(0.0, 1.0);
    let radius = style.corner_radius * scale_factor * fade;
    let padding = self.frame_padding() * fade;
    if let Err(err) =
      backdrop.place_panel(&panel, padding, radius, scale_factor)
    {
      tracing::warn!("Failed to place the overview backdrop: {err}");
    }
  }

  /// Wallpaper the framed panel shows around the scene's windows, in
  /// physical pixels; none without a scene or a framed backdrop.
  fn frame_padding(&self) -> f32 {
    self
      .style()
      .filter(|style| {
        style.wallpaper.style == OverviewWallpaperStyle::Framed
          && self.focus.layout.has_scene()
      })
      .map_or(0.0, |style| self.metrics.px(style.wallpaper.padding))
  }

  /// Where the framed backdrop's panel shows now: where the workspace
  /// zoomed out of or into shows its monitor, else the shown one (in the
  /// activities, the large one; in the stack, the selected card).
  fn backdrop_panel(&self, camera: &Camera) -> Option<RectF> {
    let rect = match (self.zoom, self.focus.layout) {
      (Some((zoom, _)), _) => self.zoom_rect(zoom.card),
      (
        None,
        OverviewLayoutMode::Activities | OverviewLayoutMode::Expose,
      ) => self
        .overlays
        .as_ref()
        .map(|overlays| overlays.workspace_rect.value())
        .or_else(|| self.zoom_rect(self.focus.selected)),
      (None, OverviewLayoutMode::Stack) => {
        self.zoom_rect(self.focus.selected)
      }
    }?;
    Some(camera.apply(&rect))
  }

  /// Places the scenes' windows, with their borders and badges, where
  /// their animations have them now.
  fn place_scenes(&mut self, context: &PlaceContext) {
    let (view, scale_factor, fade) =
      (context.view, self.metrics.scale_factor, context.fade);
    let landing = self.closing_card();
    let (has_titles, has_icons) = self.badges_shown();
    let picked = self.picked_window().map(|(_, hwnd)| hwnd);
    let hints = self.focus.hints.clone();
    let gathered_in = self.gathering_in();
    let icons = self.icons;
    let workspaces = self
      .frame
      .as_ref()
      .map_or(&[][..], |frame| &frame.workspaces);
    let mut borders = Vec::new();
    let mut badges = Vec::new();

    for (index, scene) in self.scenes.iter_mut().enumerate() {
      for window in &mut scene.windows {
        let (rect, alpha) = scene_window_shown(
          window,
          (landing == Some(index)).then_some(fade),
        );
        let rect = context.camera.apply(&rect);
        let opacity = alpha * window.preview.opacity * context.presence;

        if opacity > 0.002 {
          window.preview.place_faded(
            &rect,
            &view,
            opacity,
            context.overlays_of(window.preview.hwnd),
          );
        } else {
          window.preview.hide();
          continue;
        }

        let hwnd = window.preview.hwnd;
        let badge = |content| SceneBadge {
          content,
          rect,
          opacity: opacity * context.chrome,
        };
        if has_titles && picked != Some(hwnd) {
          let title = workspaces
            .get(index)
            .and_then(|workspace| workspace.window(hwnd))
            .map(OverviewWindow::display_title);
          if let Some(text) = title {
            badges.push(badge(BadgeContent::Title(TagPicture {
              hwnd,
              text,
              icons,
            })));
          }
        }
        // A minimized window's placeholder shows its icon already.
        if has_icons && !window.is_minimized {
          badges.push(badge(BadgeContent::Icon(hwnd, icons)));
        }
        if let Some((_, letter)) = hints.iter().find(|(h, _)| *h == hwnd) {
          badges.push(badge(BadgeContent::Hint(*letter)));
        }
        let from_elsewhere = gathered_in.is_some_and(|home| home != index);
        if let Some(workspace) =
          workspaces.get(index).filter(|_| from_elsewhere)
        {
          badges
            .push(badge(BadgeContent::Workspace(workspace.label.clone())));
        }

        let params = workspaces
          .get(index)
          .and_then(|workspace| drawn_border(workspace, window));
        let width = window.preview.screen_rect.w;
        if let (Some(params), true) = (params, width > 0.0) {
          borders.push(SceneBorder {
            rect,
            params,
            scale: rect.w / width,
            opacity,
          });
        }
      }
    }

    self.scene_borders.place(&borders, &view);
    if let Some(frame) = &self.frame {
      self.scene_badges.place(
        &badges,
        &view,
        &frame.style,
        scale_factor,
        self.notify,
      );
    }
  }

  /// Whether the exposé shows a title on every window, and an app icon
  /// in each.
  fn badges_shown(&self) -> (bool, bool) {
    self
      .style()
      .filter(|_| self.focus.layout.spreads_windows())
      .map_or((false, false), |style| {
        (
          style.expose.titles == OverviewExposeTitles::Always,
          style.expose.app_icons,
        )
      })
  }

  /// How many badges each scene window may show: a title, an icon, a
  /// hint's letter, and its workspace's name when a search gathers it
  /// from there.
  fn badges_per_window(&self) -> usize {
    let (has_titles, has_icons) = self.badges_shown();
    let expose = self
      .style()
      .filter(|_| self.focus.layout.spreads_windows())
      .map(|style| style.expose);
    let has_hints = expose
      .is_some_and(|expose| expose.hints == OverviewExposeHints::Alt);
    let gathers = expose.is_some_and(|expose| {
      expose.search_scope == OverviewExposeSearchScope::All
    });
    usize::from(has_titles)
      + usize::from(has_icons)
      + usize::from(has_hints)
      + usize::from(gathers)
  }

  /// Where the search box's picture shows, its shadow included.
  #[allow(clippy::cast_precision_loss)]
  fn spotlight_rect(&self) -> Option<RectF> {
    let (width, height) = self.spotlight.as_ref()?.size();
    // Over the scene, which a row in a column pushes aside.
    let middle = if self.focus.layout.has_scene() {
      self.scene_area.center().0
    } else {
      self.view().w / 2.0
    };
    let margin = spotlight_margin(self.metrics.scale_factor);
    (width > 0).then(|| {
      RectF::new(
        (middle - width as f32 / 2.0).round(),
        (self.spotlight_y.value - margin).round(),
        width as f32,
        height as f32,
      )
    })
  }

  /// The control of the search box under (`x`, `y`) while it shows: a
  /// row's match, or else the box itself.
  fn hit_spotlight(&self, x: f32, y: f32) -> Option<Control> {
    if self.focus.query.is_empty() || self.zoom.is_some() {
      return None;
    }

    let rect = self.spotlight_rect()?;
    let margin = spotlight_margin(self.metrics.scale_factor);
    if !rect.inset(margin).contains(x, y) {
      return None;
    }

    let (x, y) = (x - rect.x, y - rect.y);
    let row = self
      .spotlight_rows
      .iter()
      .find(|(row, _)| RectF::from_rect(row).contains(x, y));
    Some(row.map_or(Control::Search, |(_, hwnd)| Control::Result(*hwnd)))
  }

  /// Where the shortcut sheet shows, centered on the screen, once drawn.
  #[allow(clippy::cast_precision_loss)]
  fn sheet_rect(&self) -> Option<RectF> {
    let (width, height) = self.sheet.as_ref()?.size();
    let view = self.view();
    (width > 0).then(|| {
      RectF::new(
        ((view.w - width as f32) / 2.0).round(),
        ((view.h - height as f32) / 2.0).round(),
        width as f32,
        height as f32,
      )
    })
  }

  /// Where a window let go of over card `index` lands: the scene, for
  /// the activities' shown workspace, or else the card.
  pub(super) fn landing_rect(&self, index: usize) -> Option<RectF> {
    let context = self.place_context();
    if self.focus.layout.has_scene() && index == self.focus.selected {
      return Some(context.camera.apply(&self.scene_area));
    }

    self.workspace_rect(index, &context).map(|(rect, _)| rect)
  }

  /// What is under the point (`x`, `y`) of the screen.
  pub(super) fn hit(&self, x: f32, y: f32) -> Hit {
    // The open shortcut sheet covers what is under it.
    let is_on_sheet = self.sheet_drawn.is_some()
      && self.sheet_rect().is_some_and(|rect| rect.contains(x, y));
    if is_on_sheet {
      return Hit::default();
    }

    if let Some(control) = self
      .hit_spotlight(x, y)
      .or_else(|| self.hit_control(x, y))
      .or_else(|| self.hit_close(x, y).map(Control::Close))
    {
      return Hit {
        control: Some(control),
        ..Hit::default()
      };
    }

    // Mid-zoom, nothing is where it will settle.
    let (x, y) = self.camera().unapply(x, y);

    match self.focus.layout {
      OverviewLayoutMode::Activities | OverviewLayoutMode::Expose => {
        let hit = self.hit_cards(x, y);
        if hit.workspace.is_some() {
          // Too small to aim at: a window dropped on a miniature only
          // moves there, keeping its layout.
          Hit { point: None, ..hit }
        } else {
          // The framed panel's padding is the scene's too.
          self.hit_scene(
            x,
            y,
            &self.scene_area.inset(-self.frame_padding()),
          )
        }
      }
      OverviewLayoutMode::Stack => self.hit_cards(x, y),
    }
  }

  /// The control of the search line under (`x`, `y`), once the opening
  /// zoom ended and until the closing one starts.
  fn hit_control(&self, x: f32, y: f32) -> Option<Control> {
    if self.zoom.is_some() || !self.scene_chrome().has_search_line {
      return None;
    }

    let controls = self.search_controls.as_ref()?;
    let y = y - self.search_y.value;
    [
      (&controls.shortcuts, Control::Shortcuts),
      (&controls.field, Control::Search),
    ]
    .into_iter()
    .find(|(rect, _)| RectF::from_rect(rect).contains(x, y))
    .map(|(_, control)| control)
  }

  /// What card is under (`x`, `y`), before the camera, and what window
  /// on it.
  fn hit_cards(&self, x: f32, y: f32) -> Hit {
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
        control: None,
        point: card.view.unmap(local_x, local_y),
      };
    }

    Hit::default()
  }

  /// What of the shown workspace's scene is under (`x`, `y`): the
  /// topmost of its windows there (or of those a search gathered from
  /// other workspaces, of theirs), or else `area` itself.
  fn hit_scene(&self, x: f32, y: f32, area: &RectF) -> Hit {
    let selected = self.focus.selected;
    let scenes = if self.gather_home.is_some() {
      0..self.scenes.len()
    } else {
      selected..selected + 1
    };
    // Later scenes' windows are registered over earlier ones'.
    let hovered = scenes.rev().find_map(|index| {
      let window =
        self
          .scenes
          .get(index)?
          .windows
          .iter()
          .rev()
          .find(|window| {
            window.alpha.target > 0.0 && window.rect().contains(x, y)
          })?;
      Some((index, window))
    });
    // The room itself is the shown workspace's, a gathering search's home.
    let shown = self.gathering_in().unwrap_or(selected);
    let index = hovered.map_or(shown, |(index, _)| index);
    let hovered = hovered.map(|(_, window)| window);

    if hovered.is_some() || area.contains(x, y) {
      let workspace = self.workspaces().get(index);
      let point = if self.focus.layout.spreads_windows() {
        // The same spot on the window as it really is, so a drop lands
        // beside it as one there would; nowhere off the windows.
        hovered.zip(workspace).and_then(|(hovered, workspace)| {
          let real = workspace.window(hovered.preview.hwnd)?;
          spread_point(
            &hovered.rect(),
            &RectF::from_rect(&real.rect),
            x,
            y,
          )
        })
      } else {
        // Where the scene shows the workspace's working area, as laid
        // out.
        workspace.zip(self.frame.as_ref()).and_then(
          |(workspace, frame)| {
            let content = scene_content(workspace, frame);
            CardView::fitted(&content, &self.scene_area).unmap(x, y)
          },
        )
      };

      Hit {
        workspace: Some(index),
        window: hovered.map(|window| window.preview.hwnd),
        control: None,
        point,
      }
    } else {
      Hit::default()
    }
  }
}

/// The point of `real` (screen coordinates) at the spot (`x`, `y`) marks
/// on `shown`, where the same window shows elsewhere.
#[allow(clippy::cast_possible_truncation)]
fn spread_point(
  shown: &RectF,
  real: &RectF,
  x: f32,
  y: f32,
) -> Option<(i32, i32)> {
  if shown.w <= 0.0 || shown.h <= 0.0 {
    return None;
  }
  let (along, down) = ((x - shown.x) / shown.w, (y - shown.y) / shown.h);
  Some((
    // Short of the far edges, which the real rect doesn't contain.
    (real.x + (real.w - 1.0).max(0.0) * along.clamp(0.0, 1.0)).round()
      as i32,
    (real.y + (real.h - 1.0).max(0.0) * down.clamp(0.0, 1.0)).round()
      as i32,
  ))
}
