//! The overview's cards: one per workspace, showing its windows where
//! they sit, scaled down.

use windows::Win32::Foundation::HWND;

use super::{to_ms, Card, KeptCard, Screen};
use crate::{
  overview::{
    chrome::{fade, with_alpha, CardPicture, TilePicture},
    layout::{CardLook, CardSlot, CardView, RectF, Spring},
    state::Mode,
    tab_bars::CardTabBar,
    thumbnails::{visible_frame_on_screen, Picture, Preview},
  },
  OverviewStyle, OverviewWindow, OverviewWorkspace, TabFrame,
};

/// Width, in logical pixels, a window's own border is drawn at on a card,
/// where at scale it would be too thin to see.
const CARD_BORDER_WIDTH: f32 = 1.5;

/// How strongly a window shows during a search it doesn't match.
pub(super) const SEARCH_MISS_OPACITY: f32 = 0.25;

/// Where window `hwnd` of `workspace` is on screen: measured for the
/// displayed workspace's windows in view, so a zoom starts and ends
/// exactly on them.
pub(super) fn screen_rect(
  workspace: &OverviewWorkspace,
  window: &OverviewWindow,
) -> RectF {
  if window.is_minimized {
    return placeholder_rect(
      &RectF::from_rect(&workspace.content_rect),
      &RectF::from_rect(&window.rect),
    );
  }

  if workspace.is_displayed && !window.is_scrolled_off {
    visible_frame_on_screen(HWND(window.hwnd))
      .unwrap_or_else(|| RectF::from_rect(&window.rect))
  } else {
    RectF::from_rect(&window.rect)
  }
}

/// Part of its workspace's height a minimized window's placeholder takes,
/// when where it's restored to says nothing of its size.
const PLACEHOLDER_SHARE: f32 = 0.35;

/// Width of such a placeholder against its height.
const PLACEHOLDER_ASPECT: f32 = 1.6;

/// Least width and height, in pixels, a restored rect has to be taken as
/// the window's size rather than a parked sliver: a small dialog still
/// shows as small as it is.
const PLACEHOLDER_LEAST_SIZE: f32 = 48.0;

/// Where a minimized window shows as its placeholder, on a workspace
/// showing `area`: as large as `restored`, the rect it's restored to, so
/// the exposé scales it alike with the windows around it, unless that's a
/// sliver (then a set size); centered where `restored` is, kept within
/// `area`.
fn placeholder_rect(area: &RectF, restored: &RectF) -> RectF {
  let (width, height) = if restored.w >= PLACEHOLDER_LEAST_SIZE
    && restored.h >= PLACEHOLDER_LEAST_SIZE
  {
    (restored.w.min(area.w), restored.h.min(area.h))
  } else {
    let height = area.h * PLACEHOLDER_SHARE;
    ((height * PLACEHOLDER_ASPECT).min(area.w), height)
  };
  let (cx, cy) = restored.center();
  let cx = cx.clamp(area.x + width / 2.0, area.right() - width / 2.0);
  let cy = cy.clamp(area.y + height / 2.0, area.bottom() - height / 2.0);
  RectF::new(cx - width / 2.0, cy - height / 2.0, width, height)
}

/// A picture for workspace `name`'s new card, and what it shows: as the
/// last open left it, taken from `kept`, if it did, or else a blank one
/// from `spare`.
fn card_picture_for(
  kept: &mut Vec<KeptCard>,
  name: &str,
  spare: &mut Vec<Picture>,
) -> Option<(Picture, Option<CardPicture>)> {
  match kept.iter().position(|card| card.name == name) {
    Some(position) => {
      let card = kept.swap_remove(position);
      Some((card.picture, Some(card.drawn)))
    }
    None => spare
      .pop()
      .or_else(Picture::new)
      .map(|picture| (picture, None)),
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
    let settle_ms = to_ms(frame.style.switch_duration_ms);
    let bounce = frame.style.bounce;

    for (index, workspace) in frame.workspaces.iter().enumerate() {
      let content = RectF::from_rect(&workspace.content_rect);
      let metrics = self.metrics.for_content(&self.work, &content);
      let view = CardView::new(&metrics, &self.work, &content);

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
        card.retune(settle_ms, bounce);
        card.metrics = metrics;
        card.view = view;
        card.previews = previews;
        card.tab_bars = tab_bars;
        continue;
      }

      let Some((picture, drawn)) =
        card_picture_for(&mut self.kept, &workspace.name, &mut spare)
      else {
        tracing::warn!("Failed to create an overview card.");
        continue;
      };

      cards.push(Card {
        cx: Spring::new(0.0, settle_ms, 0.25).with_bounce(bounce),
        cy: Spring::new(0.0, settle_ms, 0.25).with_bounce(bounce),
        scale: Spring::new(1.0, settle_ms, 0.001).with_bounce(bounce),
        alpha: Spring::new(1.0, settle_ms, 0.002),
        metrics,
        view,
        previews,
        tab_bars,
        picture,
        // The icons it showed, so a kept picture still matches.
        icons: drawn.as_ref().map_or(0, |drawn| drawn.icons),
        drawn,
      });
    }

    for card in cards.drain(frame.workspaces.len().min(cards.len())..) {
      spare.extend(card.into_pictures());
    }

    self.cards = cards;
    self.spare = spare;
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
      card.metrics = self.metrics.for_content(&self.work, &content);
      card.view = CardView::new(&card.metrics, &self.work, &content);

      for preview in &mut card.previews {
        if let Some(tile) = card.view.tile(&preview.screen_rect) {
          preview.tile = tile;
        }
      }
      for bar in &mut card.tab_bars {
        bar.update(bar.frame.clone(), &card.view);
      }
    }
  }

  /// What the layouts need of each card: where its monitor's view is,
  /// and on the selected card, the window being picked, unless pointed
  /// at.
  pub(super) fn slots(&self) -> Vec<CardSlot> {
    let picked = self.focus.selected_window.filter(|_| {
      self.focus.mode == Mode::Windows && self.focus.keeps_window_in_view
    });

    self
      .cards
      .iter()
      .enumerate()
      .map(|(index, card)| {
        let view = card.view.map(&self.work);
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
          ..CardSlot::new(&card.metrics)
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
      picture.unregister();
      picture
    }));
    tab_bars
  }

  /// How strongly window `hwnd` of workspace `index` shows: faint while
  /// lifted, so it visibly left without vanishing, and while a search
  /// passes it by.
  pub(super) fn window_opacity(&self, index: usize, hwnd: isize) -> f32 {
    if self.focus.floating == Some(hwnd) {
      0.35
    } else if !self.focus.query.is_empty()
      && !self.matches_search(index, hwnd)
    {
      SEARCH_MISS_OPACITY
    } else {
      1.0
    }
  }

  /// Whether window `hwnd` of workspace `index` matches the search.
  pub(super) fn matches_search(&self, index: usize, hwnd: isize) -> bool {
    self
      .workspaces()
      .get(index)
      .and_then(|workspace| workspace.window(hwnd))
      .is_some_and(|window| self.focus.is_match(window))
  }

  /// Whether a search is on that no window of workspace `index` matches.
  pub(super) fn misses_search(&self, index: usize) -> bool {
    !self.focus.query.is_empty()
      && self.workspaces().get(index).is_some_and(|workspace| {
        !workspace
          .windows
          .iter()
          .any(|window| self.focus.is_match(window))
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

    let (accent, caption) = (style.accent, style.caption);
    let focused_window =
      self.frame.as_ref().and_then(|frame| frame.focused_window);

    let tiles = card.map_or_else(Vec::new, |card| {
      card
        .previews
        .iter()
        .map(|preview| {
          let window =
            workspace.and_then(|workspace| workspace.window(preview.hwnd));
          let is_picked = focus.floating == Some(preview.hwnd);
          let is_focused = focused_window == Some(preview.hwnd);

          // A window with a border of its own on screen shows that one,
          // and one whose border the WM hides (e.g. on a workspace not
          // displayed) shows it drawn, but for the overview's marks.
          let drawn_border = window
            .and_then(|window| window.border)
            .filter(|_| !preview.has_border());
          let (border_width, border) = if is_picked {
            (2.0, accent)
          } else if preview.has_border() {
            (0.0, caption)
          } else if let Some(params) = drawn_border {
            (CARD_BORDER_WIDTH, fade(params.color, params.opacity))
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
              .map(OverviewWindow::display_title),
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
      let view = card.view.map(&self.work);
      (view.w < card.metrics.tile_area().w - 1.0).then_some(view)
    });

    CardPicture {
      label: workspace.map_or_else(String::new, |w| w.label.clone()),
      look,
      viewport,
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
      icons: card.map_or(0, |card| card.icons),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::placeholder_rect;
  use crate::overview::layout::RectF;

  fn assert_near(a: (f32, f32), b: (f32, f32)) {
    assert!(
      (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01,
      "{a:?} != {b:?}"
    );
  }

  #[test]
  fn placeholders_are_as_large_as_where_their_windows_are_restored_to() {
    let area = RectF::new(0.0, 0.0, 1920.0, 1000.0);
    let big =
      placeholder_rect(&area, &RectF::new(0.0, 0.0, 1900.0, 980.0));
    let small =
      placeholder_rect(&area, &RectF::new(100.0, 100.0, 600.0, 400.0));
    let dialog =
      placeholder_rect(&area, &RectF::new(800.0, 400.0, 250.0, 150.0));
    let sliver =
      placeholder_rect(&area, &RectF::new(1500.0, 300.0, 160.0, 4.0));

    assert_near((big.w, big.h), (1900.0, 980.0));
    assert_near((small.w, small.h), (600.0, 400.0));
    assert_near((dialog.w, dialog.h), (250.0, 150.0));
    // A sliver says nothing of its size: a set one.
    assert_near((sliver.w, sliver.h), (560.0, 350.0));
    // Centered where restored to, kept within the workspace.
    assert_near(sliver.center(), (1580.0, 302.0));
    assert_near(big.center(), (950.0, 490.0));
  }

  #[test]
  fn a_placeholder_restored_off_the_workspace_stays_on_it() {
    let area = RectF::new(0.0, 0.0, 1920.0, 1000.0);
    let parked = placeholder_rect(
      &area,
      &RectF::new(-32000.0, -32000.0, 160.0, 28.0),
    );

    assert_near((parked.x, parked.y), (0.0, 0.0));
  }
}
