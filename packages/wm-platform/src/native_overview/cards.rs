//! The overview's cards: one per workspace, showing its windows where
//! they sit, scaled down.

use windows::Win32::Foundation::HWND;

use super::{desk::Desk, Card, Screen, SETTLE_MS};
use crate::{
  overview_chrome::{with_alpha, CardPicture, TilePicture},
  overview_layout::{
    CardLook, CardSlot, CardView, GridCells, RectF, Spring,
  },
  overview_state::Mode,
  overview_tab_bars::CardTabBar,
  overview_thumbnails::{visible_frame_on_screen, Picture, Preview},
  OverviewStyle, OverviewWindow, OverviewWorkspace, TabFrame,
};

/// Where window `hwnd` of `workspace` is on screen: measured for the
/// displayed workspace's windows in view, so a zoom starts and ends
/// exactly on them.
pub(super) fn screen_rect(
  workspace: &OverviewWorkspace,
  window: &OverviewWindow,
) -> RectF {
  if workspace.is_displayed && !window.is_scrolled_off {
    visible_frame_on_screen(HWND(window.hwnd))
      .unwrap_or_else(|| RectF::from_rect(&window.rect))
  } else {
    RectF::from_rect(&window.rect)
  }
}

impl Screen {
  /// Builds a card per workspace, with a preview per visible window,
  /// keeping the previews of windows still there. Returns whether a
  /// window, its overlays or a tab bar came, went or moved, which calls
  /// for registering the thumbnails anew.
  pub(super) fn rebuild_cards(&mut self) -> bool {
    let Some(frame) = &self.frame else {
      return false;
    };

    let mut cards = std::mem::take(&mut self.cards);
    let mut spare = std::mem::take(&mut self.spare);
    let mut is_changed = cards.len() != frame.workspaces.len();

    for (index, workspace) in frame.workspaces.iter().enumerate() {
      let content = RectF::from_rect(&workspace.content_rect);
      let metrics = self.metrics.for_content(&self.area, &content);
      let view = CardView::new(&metrics, &self.area, &content);

      let mut old = cards
        .get_mut(index)
        .map_or_else(Vec::new, |card| std::mem::take(&mut card.previews));
      let old_order =
        old.iter().map(|preview| preview.hwnd).collect::<Vec<_>>();
      let mut is_restyled = false;

      let previews = workspace
        .windows
        .iter()
        .filter(|window| !window.is_minimized && !window.is_hidden_tab)
        .filter_map(|window| {
          let rect = screen_rect(workspace, window);
          let tile = view.tile(&rect)?;
          let reused = old
            .iter()
            .position(|preview| preview.hwnd == window.hwnd)
            .map(|position| old.swap_remove(position));

          let mut preview = match reused {
            Some(mut preview) => {
              preview.refresh_frame();
              preview
            }
            None => Preview::new(window.hwnd, tile),
          };
          preview.tile = tile;
          preview.screen_rect = rect;
          preview.is_scrolled_off = window.is_scrolled_off;
          is_restyled |= preview.restyle(window);
          Some(preview)
        })
        .collect::<Vec<_>>();
      is_changed |= is_restyled
        || previews.iter().map(|preview| preview.hwnd).ne(old_order);

      let old_tab_bars = cards
        .get_mut(index)
        .map_or_else(Vec::new, |card| std::mem::take(&mut card.tab_bars));
      let old_anchors = old_tab_bars
        .iter()
        .map(|bar| bar.frame.anchor)
        .collect::<Vec<_>>();
      let tab_bars = Self::rebuild_tab_bars(
        &workspace.tab_bars,
        &view,
        old_tab_bars,
        &mut spare,
      );
      is_changed |=
        tab_bars.iter().map(|bar| bar.frame.anchor).ne(old_anchors);

      if let Some(card) = cards.get_mut(index) {
        card.metrics = metrics;
        card.view = view;
        card.previews = previews;
        card.tab_bars = tab_bars;
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
        alpha: Spring::new(1.0, SETTLE_MS, 0.002),
        desk: None,
        metrics,
        view,
        previews,
        tab_bars,
        picture,
        drawn: None,
      });
    }

    for card in cards.drain(frame.workspaces.len().min(cards.len())..) {
      spare.extend(card.into_pictures());
    }

    self.cards = cards;
    self.spare = spare;
    self.grid = GridCells::fitting(
      &self.metrics,
      &self.slots(),
      frame.style.grid_columns,
    );
    is_changed
  }

  /// Maps each card's windows anew for the current metrics, keeping its
  /// previews and their thumbnails.
  pub(super) fn remap_cards(&mut self) {
    let Some(frame) = &self.frame else {
      return;
    };

    for (card, workspace) in self.cards.iter_mut().zip(&frame.workspaces) {
      let content = RectF::from_rect(&workspace.content_rect);
      card.metrics = self.metrics.for_content(&self.area, &content);
      card.view = CardView::new(&card.metrics, &self.area, &content);

      for preview in &mut card.previews {
        if let Some(tile) = card.view.tile(&preview.screen_rect) {
          preview.tile = tile;
        }
      }
      for bar in &mut card.tab_bars {
        bar.update(bar.frame.clone(), &card.view);
      }
    }

    let columns = frame.style.grid_columns;
    self.grid = GridCells::fitting(&self.metrics, &self.slots(), columns);
  }

  /// What the layouts need of each card: where its monitor's view is,
  /// and on the selected card, the window being picked.
  pub(super) fn slots(&self) -> Vec<CardSlot> {
    let picked = self
      .focus
      .selected_window
      .filter(|_| self.focus.mode == Mode::Windows);

    self
      .cards
      .iter()
      .zip(self.workspaces())
      .enumerate()
      .map(|(index, (card, workspace))| {
        let view = card.view.map(&self.area);
        let keep = picked
          .filter(|_| index == self.focus.selected)
          .and_then(|hwnd| {
            // A hidden tab shows on its stack's tab bar.
            let preview = card
              .previews
              .iter()
              .find(|preview| preview.hwnd == hwnd)
              .map(|preview| preview.tile.rect);
            let bar = || {
              card
                .tab_bars
                .iter()
                .find(|bar| bar.tab_index(hwnd).is_some())
                .map(|bar| bar.rect)
            };
            preview.or_else(bar)
          })
          .map(|rect| (rect.x, rect.right()));

        CardSlot {
          view_center: view.x + view.w / 2.0,
          keep,
          ..CardSlot::new(&card.metrics, workspace.is_special)
        }
      })
      .collect()
  }

  /// A card's tab bars for `frames`, reusing the bars in `old` that
  /// belong to the same stack (by its shown window), and pictures from
  /// `spare` for new ones.
  pub(super) fn rebuild_tab_bars(
    frames: &[TabFrame],
    view: &CardView,
    mut old: Vec<CardTabBar>,
    spare: &mut Vec<Picture>,
  ) -> Vec<CardTabBar> {
    let tab_bars = frames
      .iter()
      .filter_map(|frame| {
        let reused = old
          .iter()
          .position(|bar| bar.frame.anchor == frame.anchor)
          .map(|index| old.swap_remove(index));

        if let Some(mut bar) = reused {
          bar.update(frame.clone(), view);
          return Some(bar);
        }

        let picture = spare.pop().or_else(Picture::new)?;
        Some(CardTabBar::new(frame.clone(), view, picture))
      })
      .collect();

    spare.extend(old.into_iter().map(|bar| {
      let mut picture = bar.picture;
      picture.thumbnail = None;
      picture
    }));
    tab_bars
  }

  /// How strongly window `hwnd` of workspace `index` shows: faint while
  /// lifted, so it visibly left without vanishing, and while a search
  /// passes it by.
  pub(super) fn window_opacity(&self, index: usize, hwnd: isize) -> f32 {
    let focus = &self.focus;
    if focus.floating == Some(hwnd) {
      return 0.35;
    }

    let is_miss = !focus.query.is_empty()
      && !self
        .workspaces()
        .get(index)
        .and_then(|workspace| {
          workspace.windows.iter().find(|window| window.hwnd == hwnd)
        })
        .is_some_and(|window| focus.is_match(window));

    if is_miss {
      0.3
    } else {
      1.0
    }
  }

  /// Whether card `index` shows its windows on the desk: a workspace of
  /// the monitor, not the "+" card or the special workspace.
  pub(super) fn shows_desk(&self, index: usize) -> bool {
    self.desk.as_ref().is_some_and(Desk::is_drawn)
      && self.workspaces().get(index).is_some_and(|workspace| {
        !workspace.is_new && !workspace.is_special
      })
  }

  /// Everything card `index` shows, but for its selection and hover,
  /// which show as highlights over it.
  pub(super) fn card_picture(
    &self,
    index: usize,
    style: &OverviewStyle,
  ) -> CardPicture {
    let workspace = self.workspaces().get(index);
    let card = self.cards.get(index);
    let focus = &self.focus;
    let look = self.metrics.look;

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
          let is_match = window.is_some_and(|w| focus.is_match(w));
          let is_picked = focus.floating == Some(preview.hwnd);
          let is_focused = focused_window == Some(preview.hwnd);

          // A window with a border of its own shows that one, but for
          // the overview's marks.
          let (border_width, border) = if is_match {
            (3.0, search)
          } else if is_picked {
            (2.0, accent)
          } else if preview.has_border() {
            (0.0, caption)
          } else if is_focused {
            (2.0, accent)
          } else if look == CardLook::Clean {
            (0.0, caption)
          } else {
            (1.0, with_alpha(caption, 0.6))
          };

          // A stack's window is outlined with its tab bar, which shows
          // its title.
          let tab_bar = card
            .tab_bars
            .iter()
            .find(|bar| bar.frame.anchor == preview.hwnd);

          TilePicture {
            hwnd: preview.hwnd,
            rect: preview.tile.rect,
            hole: preview.bounds(&preview.tile.rect),
            outline: tab_bar.map_or(preview.tile.rect, |bar| {
              preview.tile.rect.union(&bar.bar_rect())
            }),
            title: window
              .filter(|_| tab_bar.is_none() && look == CardLook::Classic)
              .map(|w| {
                if w.title.is_empty() {
                  w.process_name.clone()
                } else {
                  w.title.clone()
                }
              }),
            has_preview: preview.thumbnail.is_some(),
            opacity: self.window_opacity(index, preview.hwnd),
            border_width,
            border,
          }
        })
        .collect()
    });

    // A strip's card shows where its monitor is on it.
    let viewport = card.and_then(|card| {
      let view = card.view.map(&self.area);
      (view.w < card.metrics.tile_area().w - 1.0).then_some(view)
    });

    CardPicture {
      label: workspace.map_or_else(String::new, |w| w.label.clone()),
      look,
      viewport,
      desk: card
        .filter(|card| card.desk.is_some())
        .map(|card| card.view.map(&self.area)),
      is_displayed: workspace.is_some_and(|w| w.is_displayed),
      is_new: workspace.is_some_and(|w| w.is_new),
      is_special: workspace.is_some_and(|w| w.is_special),
      is_receiving: focus.floating.is_some(),
      minimized: workspace.map_or_else(Vec::new, |w| {
        w.windows
          .iter()
          .filter(|window| window.is_minimized)
          .map(|window| window.hwnd)
          .collect()
      }),
      tiles,
      tab_bars: card.map_or_else(Vec::new, |card| {
        card.tab_bars.iter().map(CardTabBar::hole).collect()
      }),
      icons: self.icons,
    }
  }
}
