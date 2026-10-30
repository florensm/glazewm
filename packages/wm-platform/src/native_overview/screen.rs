//! A monitor's screen of the overview: its window, and laying out,
//! drawing and animating what it shows.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::Instant,
};

use windows::Win32::UI::WindowsAndMessaging::{
  SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
  SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
};

use super::{
  create_screen_window, elapsed_ms, overlays::Overlays, place_card,
  scenes::scene_window_rect, to_ms, PlaceContext, Screen, ScreenFrame,
};
use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{
    draw_card, draw_search, draw_sheet, picture_size, sheet_size,
  },
  overview_layout::{
    activities, carousel, chrome_opacity, grid, stage as stage_layout,
    Camera, CardMetrics, GridCells, Placement, RectF, Spring, Zoom,
    SEARCH_HEIGHT,
  },
  overview_scene::SceneWindow,
  overview_state::{Control, Hit, Mode, ScreenFocus, Toward},
  overview_thumbnails::Picture,
  platform_impl::composition::OverviewBackdrop,
  tab_layout::TabHit,
  OverviewLayoutMode, OverviewStyle, OverviewWorkspace,
};

/// ID of the next registration of a screen's thumbnails.
static NEXT_REGISTRATION: AtomicU64 = AtomicU64::new(1);

/// How much a card grows under the cursor, shrinks while held down, and
/// grows as where a window would drop.
const HOVER_LIFT: f32 = 1.03;
const PRESS_SINK: f32 = 0.96;
const DROP_LIFT: f32 = 1.06;

/// How far what the keyboard is on bumps toward where it can't go, in
/// logical pixels.
const NUDGE_DISTANCE: f32 = 18.0;

impl Screen {
  /// A screen with its window hidden, whose pictures post to `notify`
  /// once an icon they show arrives.
  pub(super) fn new(
    notify: (windows::Win32::Foundation::HWND, u32),
  ) -> crate::Result<Self> {
    let layout = OverviewLayoutMode::default();

    // Its springs and glides are timed by the style on each open.
    Ok(Self {
      hwnd: create_screen_window()?,
      frame: None,
      focus: ScreenFocus::default(),
      metrics: CardMetrics::new(1.0, 1.0, 1.0, layout),
      drawn_for: layout,
      registered_for: None,
      registration: 0,
      area: RectF::default(),
      backdrop: None,
      cards: Vec::new(),
      scenes: Vec::new(),
      scene_area: RectF::default(),
      scene_shown: None,
      spare: Vec::new(),
      overlays: Overlays::new(0.0, 0.0),
      search: None,
      search_drawn: None,
      search_controls: None,
      search_y: Spring::new(0.0, 0.0, 0.25),
      sheet: None,
      sheet_drawn: None,
      sheet_alpha: Spring::new(0.0, 0.0, 0.002),
      dock: None,
      retired: Vec::new(),
      grid: GridCells::default(),
      nav: Vec::new(),
      zoom: None,
      is_dirty: false,
      is_gliding: false,
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

  /// Shows `frame` as `focus` has it, zooming out of the workspace it
  /// shows if `animate`.
  pub(super) fn open(
    &mut self,
    frame: ScreenFrame,
    focus: ScreenFocus,
    animate: bool,
  ) {
    self.close();

    self.area = RectF::from_rect(&frame.rect);
    self.drawn_for = focus.layout;
    self.metrics = CardMetrics::new(
      self.area.w,
      self.area.h,
      frame.scale_factor,
      focus.layout,
    );
    self.focus = focus;

    let (open_duration, bounce) =
      (frame.style.open_duration_ms, frame.style.bounce);
    self.sync_backdrop(&frame, if animate { 0.0 } else { 1.0 });

    self.frame = Some(frame);
    self.sync_durations();
    self.rebuild_cards();
    self.rebuild_scenes();
    self.ensure_labels();
    if let Some(overlays) = &mut self.overlays {
      overlays.reset();
    }

    self.zoom = animate.then(|| {
      (
        Zoom::open(self.focus.selected, open_duration, bounce),
        Instant::now(),
      )
    });

    self.retarget(true);
    self.register_thumbnails();
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

    self.is_dirty = true;
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

    if old.rect != frame.rect || old.scale_factor != frame.scale_factor {
      self.area = RectF::from_rect(&frame.rect);
      self.metrics = CardMetrics::new(
        self.area.w,
        self.area.h,
        frame.scale_factor,
        self.drawn_for,
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

    if self.registered_for != Some(self.focus.layout) {
      self.register_thumbnails();
    }
    self.refresh_pictures(is_redrawn);
  }

  /// Hides it, keeping its pictures for the next open.
  pub(super) fn close(&mut self) {
    if self.frame.take().is_none() {
      return;
    }

    // SAFETY: `self.hwnd` is the screen's window, on this thread.
    unsafe {
      ShowWindow(self.hwnd, SW_HIDE);
    }

    self.unregister_thumbnails();
    for card in self.cards.drain(..) {
      self.spare.extend(card.into_pictures());
    }
    self.scenes.clear();
    self.scene_shown = None;
    self.registered_for = None;
    self.nav.clear();
    self.zoom = None;
    self.sheet_drawn = None;
    self.sheet_alpha.snap(0.0);
    self.is_tracking_leave = false;
  }

  fn sync_backdrop(&mut self, frame: &ScreenFrame, progress: f32) {
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
    self.sheet_alpha.retune(highlight_ms);
    if let Some(overlays) = &mut self.overlays {
      overlays.retune(switch_ms, highlight_ms, bounce);
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
      self.seed_scene_from_card(false);
    }

    true
  }

  /// Registers every thumbnail the layout shows again, bottom to top:
  /// the windows on the scenes, each card's tab bars, previews and
  /// picture, the labels, highlights and tags, then the search line and
  /// the shortcut sheet.
  fn register_thumbnails(&mut self) {
    let host = self.hwnd;
    let layout = self.focus.layout;

    // Those shown now stay up until their replacements are placed, so
    // nothing blinks out in between.
    let retired = self.retire_thumbnails();
    self.retired.extend(retired);

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

    // Over everything else, the sheet over the search line.
    for picture in [&mut self.search, &mut self.sheet] {
      if picture.is_none() {
        *picture = Picture::new();
      }
      if let Some(picture) = picture {
        picture.thumbnail = Thumbnail::register(host, picture.window);
      }
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
      retired.extend(card.picture.thumbnail.take());
      for bar in &mut card.tab_bars {
        retired.extend(bar.picture.thumbnail.take());
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
    if let Some(overlays) = &mut self.overlays {
      overlays.retire(&mut retired);
    }
    for picture in
      [&mut self.search, &mut self.sheet].into_iter().flatten()
    {
      retired.extend(picture.thumbnail.take());
    }

    retired
  }

  /// Points every card and scene window at where it belongs now.
  fn retarget(&mut self, snap: bool) {
    let slots = self.slots();
    // The special workspace's dock grows while a window is carried or
    // dragged, to be dropped on.
    let is_receiving = self.focus.floating.is_some();
    // Looking inside the docked special workspace leaves the row as it
    // is.
    let is_zoomed = self.focus.mode == Mode::Windows
      && self.focus.selected == self.focus.row_selected;
    let selected = self.focus.selected;

    let targets = match self.focus.layout {
      OverviewLayoutMode::Carousel => carousel(
        &self.metrics,
        &slots,
        self.focus.row_selected,
        is_zoomed,
        is_receiving,
      ),
      OverviewLayoutMode::Grid => {
        grid(&self.metrics, &slots, &self.grid, is_receiving)
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

    // A closing screen's cards hold still for its zoom.
    if !self.is_closing() {
      self.retarget_cards(&targets.cards, snap);
    }

    if snap {
      self.search_y.snap(targets.search_y);
    } else {
      self.search_y.target = targets.search_y;
    }

    if self.focus.layout.has_scene() {
      self.retarget_scenes(snap);
    }
    self.sync_nav();
    self.is_dirty = true;
  }

  /// Points each card at its target in `targets`; there at once if
  /// `snap`.
  fn retarget_cards(&mut self, targets: &[Placement], snap: bool) {
    let focus = &self.focus;
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

    for (index, (card, target)) in
      self.cards.iter_mut().zip(targets).enumerate()
    {
      for (spring, value) in [
        (&mut card.cx, target.cx),
        (&mut card.cy, target.cy),
        (&mut card.scale, target.scale * lift(index)),
        (&mut card.alpha, target.opacity),
      ] {
        if snap {
          spring.snap(value);
        } else {
          spring.target = value;
        }
      }
    }
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
      let cursor = self.focus.selected_window;
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

    self.refresh_search(&style, force);
    self.refresh_sheet(&style, force);
    self.is_dirty = true;
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

  /// How far it is open, from 0 (a card's windows fill the screen) to 1
  /// (every card in place).
  pub(super) fn openness(&self) -> f32 {
    self
      .zoom
      .map_or(1.0, |(zoom, started)| zoom.openness(elapsed_ms(started)))
  }

  /// How far its zoom has moved things: the openness, or past it for a
  /// moment as an open overshoots.
  pub(super) fn reach(&self) -> f32 {
    self
      .zoom
      .map_or(1.0, |(zoom, started)| zoom.reach(elapsed_ms(started)))
  }

  /// Whether it is zooming into the workspace it closes on.
  pub(super) fn is_closing(&self) -> bool {
    self.zoom.is_some_and(|(zoom, _)| zoom.is_closing)
  }

  /// Whether its zoom into the workspace it closes on has ended.
  pub(super) fn is_zoomed_in(&self) -> bool {
    self.zoom.is_some_and(|(zoom, started)| {
      zoom.is_closing && zoom.is_finished(elapsed_ms(started))
    })
  }

  /// The zoom between the screen and the zoomed card's windows, at the
  /// current openness. Follows the card if it is still moving. The
  /// activities zoom each window on their own instead.
  pub(super) fn camera(&self) -> Camera {
    if self.focus.layout.has_scene() {
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
    Camera::zoom(&screen, &self.view(), self.reach())
  }

  /// Zooms into card `index` until its windows fill the screen, over
  /// `duration_ms`, showing `focus`.
  ///
  /// What it zooms from holds still meanwhile, so each window heads
  /// straight to where it lands rather than veering off with the layout
  /// (e.g. a scrolled strip settling, or the stage centering the pick).
  pub(super) fn start_closing(
    &mut self,
    index: usize,
    focus: ScreenFocus,
    duration_ms: u32,
  ) {
    // The scenes zoom into the picked workspace's own windows, which set
    // off from where its miniature shows them.
    let is_switching =
      focus.layout.has_scene() && index != self.focus.selected;
    self.focus = focus;
    if is_switching {
      self.seed_scene_from_card(false);
    }

    self.hold(index);
    let zoom = Zoom::close(index, self.openness(), duration_ms);
    self.zoom = Some((zoom, Instant::now()));

    // The scene shown until now fades out.
    if is_switching {
      self.retarget(false);
    }
    self.is_dirty = true;
  }

  /// Card it zooms into to close on, which holds still meanwhile.
  pub(super) fn closing_card(&self) -> Option<usize> {
    self
      .zoom
      .filter(|(zoom, _)| zoom.is_closing)
      .map(|(zoom, _)| zoom.card)
  }

  /// Stops the cards, card `index`'s scene and the shown workspace's area
  /// where they are now.
  fn hold(&mut self, index: usize) {
    for card in &mut self.cards {
      for spring in [&mut card.cx, &mut card.cy, &mut card.scale] {
        spring.snap(spring.value);
      }
    }
    if let Some(scene) = self.scenes.get_mut(index) {
      scene.windows.iter_mut().for_each(SceneWindow::hold);
    }
    if let Some(overlays) = &mut self.overlays {
      overlays.workspace_rect.hold();
    }
  }

  pub(super) fn place_context(&self) -> PlaceContext {
    PlaceContext {
      camera: self.camera(),
      fade: self.openness(),
      reach: self.reach(),
      view: self.view(),
      zoom_card: self.zoom.map(|(zoom, _)| zoom.card),
    }
  }

  /// Steps its animations by `dt` seconds and places what moved. Returns
  /// whether anything is still moving.
  pub(super) fn tick(&mut self, dt: f32) -> bool {
    if self.frame.is_none() {
      return false;
    }

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
    is_moving |= self.search_y.step(dt);
    is_moving |= self.sheet_alpha.step(dt);

    if let Some((zoom, started)) = self.zoom {
      is_moving = true;
      let elapsed = elapsed_ms(started);

      if let Some(backdrop) = &self.backdrop {
        let _ = backdrop.set_progress(zoom.openness(elapsed));
      }

      if zoom.is_finished(elapsed) {
        // Zoomed in, it stays as it is until the overview hides.
        if zoom.is_closing {
          return false;
        }
        self.zoom = None;
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
    let (fade, reach) = (context.fade, context.reach);
    let layout = self.focus.layout;
    let shift = self.top_shift(fade);

    match layout {
      OverviewLayoutMode::Activities => {
        let miniatures = PlaceContext {
          camera: Camera::shifted(0.0, shift),
          fade,
          reach,
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
            scene_window_rect(window, zoom_card == Some(index), reach);
          let opacity = window.alpha.value * window.preview.opacity;

          if opacity > 0.002 {
            window.preview.place(&rect, &view, opacity);
          } else {
            window.preview.hide();
          }
        }
      }
    }

    let is_gliding = self.place_overlays(&context, shift);

    let search_height = SEARCH_HEIGHT * scale_factor;
    let has_search = self.search_drawn.is_some();
    if let Some(search) = &mut self.search {
      if has_search {
        search.place(
          &RectF::new(0.0, self.search_y.value, view.w, search_height),
          &view,
          chrome_opacity(fade),
        );
      } else if let Some(thumbnail) = &mut search.thumbnail {
        thumbnail.hide();
      }
    }

    let sheet_rect = self.sheet_rect();
    let sheet_opacity = chrome_opacity(fade) * self.sheet_alpha.value;
    if let Some(sheet) = &mut self.sheet {
      match sheet_rect {
        Some(rect) if sheet_opacity > 0.002 => {
          sheet.place(&rect, &view, sheet_opacity);
        }
        _ => {
          if let Some(thumbnail) = &mut sheet.thumbnail {
            thumbnail.hide();
          }
        }
      }
    }

    // Their replacements are in place now.
    self.retired.clear();

    is_gliding
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
    if self.focus.layout.has_scene() && index == self.focus.selected {
      return Some(self.scene_area);
    }

    let context = self.place_context();
    self
      .workspace_rect(index, &context, self.top_shift(context.fade))
      .map(|(rect, _)| rect)
  }

  /// What is under the point (`x`, `y`) of the screen.
  pub(super) fn hit(&self, x: f32, y: f32) -> Hit {
    // The open shortcut sheet covers what is under it.
    let is_on_sheet = self.sheet_drawn.is_some()
      && self.sheet_rect().is_some_and(|rect| rect.contains(x, y));
    if is_on_sheet {
      return Hit::default();
    }

    if let Some(control) = self.hit_control(x, y) {
      return Hit {
        control: Some(control),
        ..Hit::default()
      };
    }

    match self.focus.layout {
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

  /// The control of the search line under (`x`, `y`), once the opening
  /// zoom ended and until the closing one starts.
  fn hit_control(&self, x: f32, y: f32) -> Option<Control> {
    if self.zoom.is_some() {
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
        control: None,
      };
    }

    Hit::default()
  }

  /// What of the shown workspace's scene is under (`x`, `y`): the
  /// topmost of its windows there, or else `area` itself.
  fn hit_scene(&self, x: f32, y: f32, area: &RectF) -> Hit {
    let selected = self.focus.selected;
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
        control: None,
      }
    } else {
      Hit::default()
    }
  }
}
