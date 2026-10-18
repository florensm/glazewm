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
  create_screen_window, desk::Desk, elapsed_ms, overlays::Overlays,
  place_card, scenes::scene_window_rect, PlaceContext, Screen,
  ScreenFrame, SETTLE_MS,
};
use crate::{
  overview_chrome::{draw_card, draw_hint, picture_size, with_alpha},
  overview_layout::{
    activities, carousel, chrome_opacity, grid, stage as stage_layout,
    Camera, CardMetrics, GridCells, Placement, RectF, Spring, Zoom,
    HINT_HEIGHT,
  },
  overview_scene::SceneWindow,
  overview_state::{HintTone, Hit, Mode, ScreenFocus},
  overview_thumbnails::{Picture, Thumbnail},
  platform_impl::composition::OverviewBackdrop,
  tab_layout::TabHit,
  OverviewLayoutMode, OverviewStyle, OverviewWorkspace,
};

/// ID of the next registration of a screen's thumbnails.
static NEXT_REGISTRATION: AtomicU64 = AtomicU64::new(1);

impl Screen {
  /// A screen with its window hidden, whose pictures post to `notify`
  /// once an icon they show arrives.
  pub(super) fn new(
    notify: (windows::Win32::Foundation::HWND, u32),
  ) -> crate::Result<Self> {
    let layout = OverviewLayoutMode::default();

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
      desk: None,
      desk_thumbnail: None,
      cards: Vec::new(),
      scenes: Vec::new(),
      scene_area: RectF::default(),
      spare: Vec::new(),
      overlays: Overlays::new(),
      hint: None,
      hint_drawn: None,
      hint_y: Spring::new(0.0, SETTLE_MS, 0.25),
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

    let open_duration = frame.style.open_duration_ms;
    self.sync_backdrop(&frame, if animate { 0.0 } else { 1.0 });

    self.frame = Some(frame);
    self.rebuild_cards();
    self.rebuild_scenes();
    self.ensure_labels();
    if let Some(overlays) = &mut self.overlays {
      overlays.reset();
    }

    self.zoom = animate.then(|| {
      (
        Zoom::open(self.focus.selected, open_duration),
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
    self.registered_for = None;
    self.nav.clear();
    self.zoom = None;
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

    if self.desk.is_none() {
      self.desk = Desk::new();
    }
    if let Some(desk) = &mut self.desk {
      desk.sync(&frame.rect, style);
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
      self.seed_scene_from_card();
    }

    true
  }

  /// Registers every thumbnail the layout shows again, bottom to top:
  /// the activities' desk, the windows on the scenes, each card's
  /// tab bars, previews and picture, the labels, highlights and tags,
  /// then the hints.
  fn register_thumbnails(&mut self) {
    let host = self.hwnd;
    let layout = self.focus.layout;

    // Those shown now stay up until their replacements are placed, so
    // nothing blinks out in between.
    let retired = self.retire_thumbnails();
    self.retired.extend(retired);

    if let (OverviewLayoutMode::Activities, Some(desk)) =
      (layout, &self.desk)
    {
      self.desk_thumbnail = desk.register(host);
    }

    if layout.has_scene() {
      for window in
        self.scenes.iter_mut().flat_map(|scene| &mut scene.windows)
      {
        window.preview.register(host);
      }
    }

    let shows_desk = (0..self.cards.len())
      .map(|index| self.shows_desk(index))
      .collect::<Vec<_>>();
    for (card, shows_desk) in self.cards.iter_mut().zip(shows_desk) {
      card.desk = self
        .desk
        .as_ref()
        .filter(|_| shows_desk)
        .and_then(|desk| desk.register(host));

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
      retired.extend(card.desk.take());
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
    retired.extend(self.desk_thumbnail.take());
    if let Some(overlays) = &mut self.overlays {
      overlays.retire(&mut retired);
    }
    if let Some(hint) = &mut self.hint {
      retired.extend(hint.thumbnail.take());
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
      self.hint_y.snap(targets.hint_y);
    } else {
      self.hint_y.target = targets.hint_y;
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
    for (index, (card, target)) in
      self.cards.iter_mut().zip(targets).enumerate()
    {
      let bump = if self.focus.drop == Some(index) {
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

    let hint = self.focus.hint.clone();
    if force || self.hint_drawn != hint {
      if let Some((text, tone)) = &hint {
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
      }
      self.hint_drawn = hint;
    }

    self.is_dirty = true;
  }

  /// How far it is open, from 0 (a card's windows fill the screen) to 1
  /// (every card in place).
  pub(super) fn openness(&self) -> f32 {
    self
      .zoom
      .map_or(1.0, |(zoom, started)| zoom.openness(elapsed_ms(started)))
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
    Camera::zoom(&screen, &self.view(), self.openness())
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
      self.seed_scene_from_card();
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

  /// Stops the cards, card `index`'s scene and the desk where they are
  /// now.
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
      overlays.desk_rect.hold();
    }
  }

  pub(super) fn place_context(&self) -> PlaceContext {
    PlaceContext {
      camera: self.camera(),
      fade: self.openness(),
      view: self.view(),
      area: self.area,
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
    is_moving |= self.hint_y.step(dt);

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
    let fade = context.fade;
    let layout = self.focus.layout;
    let shift = self.top_shift(fade);

    match layout {
      OverviewLayoutMode::Activities => {
        let miniatures = PlaceContext {
          camera: Camera::shifted(0.0, shift),
          fade,
          view,
          area: self.area,
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
          } else {
            window.preview.hide();
          }
        }
      }
    }

    let is_gliding = self.place_overlays(&context, shift);

    let hint_height = HINT_HEIGHT * scale_factor;
    let has_hint = self.hint_drawn.is_some();
    if let Some(hint) = &mut self.hint {
      if has_hint {
        hint.place(
          &RectF::new(0.0, self.hint_y.value, view.w, hint_height),
          &view,
          chrome_opacity(fade),
        );
      } else if let Some(thumbnail) = &mut hint.thumbnail {
        thumbnail.hide();
      }
    }

    // Their replacements are in place now.
    self.retired.clear();

    is_gliding
  }

  /// What is under the point (`x`, `y`) of the screen.
  pub(super) fn hit(&self, x: f32, y: f32) -> Hit {
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
      }
    } else {
      Hit::default()
    }
  }
}
