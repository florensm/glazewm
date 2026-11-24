//! The scenes of the activities: each workspace's windows laid out on
//! their own rather than on its card.

use std::cmp::Ordering;

use super::{cards::screen_rect, to_f32_size, to_ms, Screen, ScreenFrame};

/// Gap, in logical pixels, a sliding scene keeps from the one it slides
/// past.
const SLIDE_GAP: f32 = 48.0;

use crate::{
  overview::{
    chrome::note_size,
    expose::{arrange, shelf, Arrangement},
    layout::{CardView, RectF, Tile},
    native::cards::SEARCH_MISS_OPACITY,
    scene::{Scene, SceneWindow},
    state::ScreenFocus,
  },
  OverviewExposeMinimized, OverviewExposeSearch, OverviewRowSide,
  OverviewStripView, OverviewSwitchStyle, OverviewWorkspace,
};

/// Where a scene window shows now, before the camera, and how strongly.
///
/// `landing` is the openness while its workspace is zoomed into on close:
/// it then heads on from where it is, and as strongly as it shows, to
/// where and as strongly as it settles, getting there as the zoom ends
/// however far its own springs had to go, e.g. for a workspace picked
/// before it had slid in.
pub(super) fn scene_window_shown(
  window: &SceneWindow,
  landing: Option<f32>,
) -> (RectF, f32) {
  let (rect, alpha) = (window.rect(), window.alpha.value);

  // Landing, it settles where it heads, at its own size.
  let (rect, alpha, lift) = match landing {
    Some(openness) => {
      let closed = 1.0 - openness.clamp(0.0, 1.0);
      (
        rect.lerp(&window.target(), closed),
        alpha + (window.alpha.target - alpha) * closed,
        window.lift.value * (1.0 - closed),
      )
    }
    None => (rect, alpha, window.lift.value),
  };
  let (grow_w, grow_h) = (rect.w * lift, rect.h * lift);
  (
    RectF::new(
      rect.x - grow_w / 2.0,
      rect.y - grow_h / 2.0,
      rect.w + grow_w,
      rect.h + grow_h,
    ),
    alpha,
  )
}

/// What of `workspace` a scene shows, in screen coordinates: its whole
/// strip, or what its monitor shows, as `frame`'s style has it.
pub(super) fn scene_content(
  workspace: &OverviewWorkspace,
  frame: &ScreenFrame,
) -> RectF {
  match frame.style.strip_view {
    OverviewStripView::Visible => RectF::from_rect(&frame.rect),
    OverviewStripView::Whole => RectF::from_rect(&workspace.content_rect),
  }
}

/// How the exposé spreads a scene's windows.
struct Spread {
  area: RectF,
  how: Arrangement,
  /// The shelf windows go on, if any do.
  shelf: ShelfLook,
  /// Whether minimized windows go on the shelf.
  shelves_minimized: bool,
  search: OverviewExposeSearch,
  /// The screen's own rect, which a peeked window is enlarged within.
  view: RectF,
}

/// Sizes of the exposé's shelf, in physical pixels.
#[derive(Clone, Copy)]
struct ShelfLook {
  height: f32,
  label: f32,
}

/// Height of the previews on the exposé's shelf, in logical pixels.
const SHELF_HEIGHT: f32 = 96.0;

/// Labels over the exposé's shelf: of minimized windows, and of a
/// search's misses.
pub(super) const SHELF_LABEL: &str = "Minimized";
const MISSES_LABEL: &str = "Not matching";

/// Part of the screen's width and height a peeked window takes at most.
const PEEK_SHARE: (f32, f32) = (0.84, 0.8);

/// How strongly the other windows show while one is peeked at.
const PEEK_DIM: f32 = 0.22;

/// Where a window settles, and how strongly it shows there, as a part of
/// its usual strength.
type Settled = Option<(RectF, f32)>;

/// What the exposé's spread of a scene needs of the search and the
/// keyboard: the matches, if a search has any, and the window peeked at.
struct Focused<'a> {
  matches: Option<&'a [isize]>,
  peeked: Option<isize>,
}

/// Where each window of `scene` settles, given its `tiles` as laid out:
/// there, or spread as `spread` has it. `None` for a window not shown,
/// e.g. a stack's other tab where it isn't spread. Crops each preview as
/// it then shows, and notes where its shelf is.
fn scene_rects(
  scene: &mut Scene,
  tiles: &[Option<Tile>],
  spread_as: Option<&Spread>,
  focused: &Focused,
) -> Vec<Settled> {
  if let Some(how) = spread_as {
    for window in &mut scene.windows {
      window.preview.tile.crop = RectF::new(0.0, 0.0, 1.0, 1.0);
    }
    let (rects, shelf) = spread_windows(scene, how, focused);
    scene.shelf = shelf;
    return rects;
  }
  scene.shelf = None;

  scene
    .windows
    .iter_mut()
    .zip(tiles)
    .map(|(window, tile)| {
      let tile = tile.as_ref()?;
      window.preview.tile.crop = tile.crop;
      // Only the exposé shows a stack's other tabs and minimized windows.
      (!window.is_hidden_tab && !window.is_minimized)
        .then_some((tile.rect, 1.0))
    })
    .collect()
}

/// Where the exposé shows each window of `scene`, every tab of a stack
/// and minimized window a window of its own. During a search, as `how`'s
/// search mode has it: the matches alone, all with the misses dimmed, or
/// the misses on the shelf. Minimized windows go on the shelf too if
/// `how` has them there. A peeked window shows enlarged, the rest dimmed.
/// `None` for a window left out. Also returns the band the shelf takes,
/// and its label.
fn spread_windows(
  scene: &Scene,
  how: &Spread,
  focused: &Focused,
) -> (Vec<Settled>, Option<(RectF, &'static str)>) {
  let is_miss = |hwnd: isize| {
    focused
      .matches
      .is_some_and(|matches| !matches.contains(&hwnd))
  };
  let windows = scene.windows.iter().enumerate().filter(|(_, window)| {
    how.search != OverviewExposeSearch::Spread
      || !is_miss(window.preview.hwnd)
  });
  let is_shelved = |window: &SceneWindow| {
    let misses = how.search == OverviewExposeSearch::Shelf
      && is_miss(window.preview.hwnd);
    misses || (how.shelves_minimized && window.is_minimized)
  };
  let (shelved, shown): (Vec<_>, Vec<_>) =
    windows.partition(|(_, window)| is_shelved(window));
  let screen_rects = |windows: &[(usize, &SceneWindow)]| {
    windows
      .iter()
      .map(|(_, window)| window.preview.screen_rect)
      .collect::<Vec<_>>()
  };

  let mut rects = vec![None; scene.windows.len()];
  let mut room = how.area;
  let mut band = None;
  if !shelved.is_empty() {
    let shelf = shelf(
      &screen_rects(&shelved),
      &how.area,
      how.how.gap,
      how.shelf.height,
      how.shelf.label,
    );
    for ((index, _), rect) in shelved.iter().zip(shelf.rects) {
      rects[*index] = Some((rect, 1.0));
    }
    let has_misses = shelved
      .iter()
      .any(|(_, window)| is_miss(window.preview.hwnd));
    let label = if has_misses {
      MISSES_LABEL
    } else {
      SHELF_LABEL
    };
    room = shelf.room;
    band = Some((shelf.band, label));
  }

  let slots = arrange(&screen_rects(&shown), &room, &how.how);
  for ((index, window), slot) in shown.iter().zip(slots) {
    let dims = how.search == OverviewExposeSearch::Dim
      && is_miss(window.preview.hwnd);
    let strength = if dims { SEARCH_MISS_OPACITY } else { 1.0 };
    rects[*index] = Some((slot, strength));
  }

  let peeked = focused.peeked.and_then(|hwnd| {
    scene.windows.iter().position(|w| w.preview.hwnd == hwnd)
  });
  if let Some(peeked) = peeked {
    for (index, settled) in rects.iter_mut().enumerate() {
      if let Some((rect, strength)) = settled {
        if index == peeked {
          *rect = peek_rect(
            &scene.windows[index].preview.screen_rect,
            &how.view,
          );
          *strength = 1.0;
        } else {
          *strength *= PEEK_DIM;
        }
      }
    }
  }
  (rects, band)
}

/// Where a window at `screen_rect` shows while peeked at: as large as
/// `PEEK_SHARE` of `view` allows, no larger than it is, in its middle.
fn peek_rect(screen_rect: &RectF, view: &RectF) -> RectF {
  let scale = (view.w * PEEK_SHARE.0 / screen_rect.w.max(1.0))
    .min(view.h * PEEK_SHARE.1 / screen_rect.h.max(1.0))
    .min(1.0);
  let (w, h) = (screen_rect.w * scale, screen_rect.h * scale);
  let (cx, cy) = view.center();
  RectF::new(cx - w / 2.0, cy - h / 2.0, w, h)
}

/// The windows of `workspace` a search matches, which the exposé
/// spreads alone: `None` without a query, or with no match, which leaves
/// every window spread.
fn search_matches(
  workspace: &OverviewWorkspace,
  focus: &ScreenFocus,
) -> Option<Vec<isize>> {
  if focus.query.is_empty() {
    return None;
  }
  let matches = workspace
    .windows
    .iter()
    .filter(|window| focus.is_match(window))
    .map(|window| window.hwnd)
    .collect::<Vec<_>>();
  (!matches.is_empty()).then_some(matches)
}

impl Screen {
  /// Builds each workspace's scene, keeping the windows still on it.
  /// Returns whether a window came, went or moved, which calls for
  /// registering the thumbnails anew.
  pub(super) fn rebuild_scenes(&mut self) -> bool {
    let Some(frame) = &self.frame else {
      return false;
    };

    let mut old = std::mem::take(&mut self.scenes);
    let mut is_changed = old.iter().map(|scene| scene.name.as_str()).ne(
      frame
        .workspaces
        .iter()
        .map(|workspace| workspace.name.as_str()),
    );
    let settle_ms = to_ms(frame.style.switch_duration_ms);
    let bounce = frame.style.bounce;
    let origin = (self.area.x, self.area.y);

    self.scenes = frame
      .workspaces
      .iter()
      .map(|workspace| {
        let previous = old
          .iter()
          .position(|scene| scene.name == workspace.name)
          .map_or_else(Scene::default, |position| {
            std::mem::take(&mut old[position])
          });

        let (scene, is_scene_changed) = Scene::rebuild(
          previous,
          workspace,
          settle_ms,
          bounce,
          origin,
          |window| screen_rect(workspace, window),
        );
        is_changed |= is_scene_changed;
        scene
      })
      .collect();

    is_changed
  }

  /// Where card `index` shows each of its windows now, in the activities,
  /// before the camera.
  fn miniature_rects(&self, index: usize) -> Vec<(isize, RectF)> {
    let Some(card) = self.cards.get(index) else {
      return Vec::new();
    };

    let placement = card.placement();
    card
      .previews
      .iter()
      .map(|preview| {
        (
          preview.hwnd,
          placement.map(&card.metrics, &preview.tile.rect),
        )
      })
      .collect()
  }

  /// Puts the selected workspace's scene windows where its card shows
  /// them now, so they fly on from there. With `keep_shown`, those still
  /// showing (e.g. on their way back into the card) carry on from where
  /// they are instead.
  pub(super) fn seed_scene_from_card(&mut self, keep_shown: bool) {
    let selected = self.focus.selected;
    self.scene_shown = Some(selected);
    let rects = self.miniature_rects(selected);
    let Some(scene) = self.scenes.get_mut(selected) else {
      return;
    };

    for window in &mut scene.windows {
      if keep_shown && window.alpha.value > 0.05 {
        continue;
      }
      let rect = rects
        .iter()
        .find(|(hwnd, _)| *hwnd == window.preview.hwnd)
        .map(|(_, rect)| *rect);
      if let Some(rect) = rect {
        window.show_from(&rect);
      }
    }
  }

  /// Brings the selected workspace's scene windows on as `style` has a
  /// newly shown workspace come in, from `previous`, the one shown until
  /// now: sliding in from its side, or fading in in place. Those still
  /// showing (e.g. on their way out) carry on from where they are.
  fn seed_scene(
    &mut self,
    style: OverviewSwitchStyle,
    previous: Option<usize>,
    slide: (f32, f32),
  ) {
    let selected = self.focus.selected;
    if style == OverviewSwitchStyle::Miniature {
      self.seed_scene_from_card(true);
      return;
    }

    self.scene_shown = Some(selected);
    let offset = match (style, previous) {
      (OverviewSwitchStyle::Slide, Some(previous)) => {
        slide_offset(slide, selected.cmp(&previous))
      }
      _ => (0.0, 0.0),
    };

    let Some(scene) = self.scenes.get_mut(selected) else {
      return;
    };
    for window in &mut scene.windows {
      if window.alpha.value <= 0.05 {
        let from = window.target().translate(offset.0, offset.1);
        window.appear_from(&from);
      }
    }
  }

  /// Points each scene's windows where the layout shows them: the
  /// selected workspace's in place, brought on as the style's switch has
  /// it when it was just selected, and the others sent off that way and
  /// faded out.
  pub(super) fn retarget_scenes(&mut self, snap: bool) {
    let Some(style) = self.style().map(|style| style.switch_style) else {
      return;
    };

    let selected = self.focus.selected;
    let previous = self.scene_shown;
    let slide = self.slide(style);

    let is_switched = previous != Some(selected);
    if snap {
      self.scene_shown = Some(selected);
    }

    let miniatures = (0..self.cards.len())
      .map(|index| self.miniature_rects(index))
      .collect::<Vec<_>>();
    let closing = self.closing_card();
    let spreads = self.focus.layout.spreads_windows();
    // Opening, spread windows fly out of where they are on screen.
    let flies_out =
      spreads && snap && self.zoom.is_some() && closing.is_none();
    let Some(frame) = &self.frame else {
      return;
    };
    let scene_area = self.scene_area;
    let spread_as = self.spread_as(frame);

    // Opening, they fly out one after another, out from the focused one.
    let stagger = flies_out.then(|| {
      (
        to_ms(frame.style.effects.stagger_ms),
        to_ms(frame.style.open_duration_ms) / 2.0,
      )
    });

    for (index, (scene, workspace)) in
      self.scenes.iter_mut().zip(&frame.workspaces).enumerate()
    {
      // Out of sight and staying so: where it would settle is worked out
      // once it's selected, and spreading every workspace's windows on
      // each open held up the zoom's start.
      if index != selected && !scene.is_seen() {
        scene.keep_hidden(snap);
        continue;
      }

      let offset = slide_offset(slide, index.cmp(&selected));
      let alpha = if index == selected { 1.0 } else { 0.0 };

      let content = scene_content(workspace, frame);
      let view = CardView::fitted(&content, &scene_area);
      let tiles = scene
        .windows
        .iter()
        .map(|window| view.tile(&window.preview.screen_rect))
        .collect::<Vec<_>>();

      // A scene being zoomed into lands as laid out, where the real
      // windows are.
      let spreading = spreads && closing != Some(index);
      let matches = search_matches(workspace, &self.focus);
      let focused = Focused {
        matches: matches.as_deref(),
        peeked: self.focus.peeked,
      };
      let rects = scene_rects(
        scene,
        &tiles,
        spreading.then_some(&spread_as),
        &focused,
      );

      for ((window, rect), tile) in
        scene.windows.iter_mut().zip(rects).zip(&tiles)
      {
        if flies_out {
          start_flight(window, rect, tile.as_ref(), offset, alpha);
        }
        let miniature = miniatures.get(index).and_then(|rects| {
          rects
            .iter()
            .find(|(hwnd, _)| *hwnd == window.preview.hwnd)
            .map(|(_, rect)| *rect)
        });
        let into_miniature =
          style == OverviewSwitchStyle::Miniature && index != selected;

        let snap = snap && !flies_out;
        match (rect, miniature) {
          (_, Some(miniature)) if into_miniature => {
            window.retarget(&miniature, alpha, snap);
          }
          (Some((rect, strength)), _) => {
            window.retarget(
              &rect.translate(offset.0, offset.1),
              alpha * strength,
              snap,
            );
          }
          (None, _) => {
            let target = window.target();
            window.retarget(&target, 0.0, snap);
          }
        }
      }

      if let (Some(stagger), true) = (stagger, index == selected) {
        stagger_flight(scene, frame.focused_window, stagger);
      }
    }

    // Once its windows head where they settle, so that is where they come
    // in toward.
    if is_switched && !snap {
      self.seed_scene(style, previous, slide);
    }

    self.retarget_workspace_rect(snap);
  }

  /// Where `style` has a later workspace's scene wait beside the shown
  /// one, along the way the row runs: a full slide clears the scene; a
  /// fade stays in place.
  fn slide(&self, style: OverviewSwitchStyle) -> (f32, f32) {
    let is_column = matches!(
      self.scene_chrome().side,
      OverviewRowSide::Left | OverviewRowSide::Right
    );
    let along = |distance: f32| {
      if is_column {
        (0.0, distance)
      } else {
        (distance, 0.0)
      }
    };
    let (area, view) = if is_column {
      (self.scene_area.h, self.metrics.view.1)
    } else {
      (self.scene_area.w, self.metrics.view.0)
    };
    match style {
      OverviewSwitchStyle::Slide => {
        along(area + self.metrics.px(SLIDE_GAP))
      }
      OverviewSwitchStyle::Fade => (0.0, 0.0),
      OverviewSwitchStyle::Miniature => along(view * 0.12),
    }
  }

  /// How the exposé spreads a scene of `frame` over the scene area.
  fn spread_as(&self, frame: &ScreenFrame) -> Spread {
    let expose = &frame.style.expose;
    Spread {
      area: self.scene_area,
      how: Arrangement {
        style: expose.arrange,
        gap: self.metrics.px(expose.gap.max(0.0)),
        largest: expose.clamped_largest(),
        small_window_boost: expose.clamped_small_window_boost(),
        readable_floor: expose.clamped_readable_floor(),
        work: RectF::from_rect(&frame.rect),
      },
      shelf: ShelfLook {
        height: self.metrics.px(SHELF_HEIGHT),
        label: to_f32_size(note_size(
          SHELF_LABEL,
          &frame.style,
          self.metrics.scale_factor,
        ))
        .1,
      },
      shelves_minimized: expose.minimized
        == OverviewExposeMinimized::Shelf,
      search: expose.search_mode,
      view: RectF::new(0.0, 0.0, self.metrics.view.0, self.metrics.view.1),
    }
  }

  /// Points the shown workspace's working area where the scene lays it
  /// out (a scrolling workspace's whole strip reaches past it), holding
  /// still for a zoom.
  fn retarget_workspace_rect(&mut self, snap: bool) {
    let Some(frame) = &self.frame else {
      return;
    };
    let shown = frame
      .workspaces
      .get(self.focus.selected)
      .filter(|_| self.closing_card().is_none());
    if let (true, Some(overlays), Some(shown)) =
      (self.focus.layout.has_scene(), &mut self.overlays, shown)
    {
      let content = scene_content(shown, frame);
      let area = CardView::fitted(&content, &self.scene_area)
        .map(&RectF::from_rect(&frame.rect));
      overlays.workspace_rect.retarget(&area, snap);
    }
  }
}

/// How long, in ms, each window whose middle is at `centers` holds still
/// before flying out: `stagger_ms` after the one before, from window
/// `from` outward (else from their middle), all within `within_ms`.
fn ripple_delays(
  centers: &[(f32, f32)],
  from: Option<usize>,
  stagger_ms: f32,
  within_ms: f32,
) -> Vec<f32> {
  if centers.len() < 2 || stagger_ms <= 0.0 {
    return vec![0.0; centers.len()];
  }
  #[allow(clippy::cast_precision_loss)]
  let count = centers.len() as f32;
  let origin = from.map_or_else(
    || {
      let (x, y) = centers
        .iter()
        .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));
      (x / count, y / count)
    },
    |index| centers[index],
  );
  let distance = |(x, y): (f32, f32)| (x - origin.0).hypot(y - origin.1);

  let mut order = (0..centers.len()).collect::<Vec<_>>();
  order.sort_by(|&a, &b| {
    distance(centers[a])
      .total_cmp(&distance(centers[b]))
      .then(a.cmp(&b))
  });
  let step = stagger_ms.min(within_ms.max(0.0) / (count - 1.0));
  let mut delays = vec![0.0; centers.len()];
  for (rank, index) in order.into_iter().enumerate() {
    #[allow(clippy::cast_precision_loss)]
    let rank = rank as f32;
    delays[index] = rank * step;
  }
  delays
}

/// Starts `window` where it flies out of on open, shifted by `offset`:
/// where it is on screen (its `tile`) at `alpha`, or, minimized and so
/// nowhere on screen, unseen where it settles (`rect`), to fade in there.
fn start_flight(
  window: &mut SceneWindow,
  rect: Settled,
  tile: Option<&Tile>,
  offset: (f32, f32),
  alpha: f32,
) {
  let from = if window.is_minimized {
    rect.map(|(rect, _)| (rect, 0.0))
  } else {
    let shown = if window.is_hidden_tab { 0.0 } else { alpha };
    tile.map(|tile| (tile.rect, shown))
  };
  if let Some((rect, alpha)) = from {
    window.retarget(&rect.translate(offset.0, offset.1), alpha, true);
  }
}

/// Holds each of `scene`'s windows back before it flies out, one after
/// another out from `focused`: `stagger_ms` apart, all within
/// `within_ms`.
fn stagger_flight(
  scene: &mut Scene,
  focused: Option<isize>,
  (stagger_ms, within_ms): (f32, f32),
) {
  let centers = scene
    .windows
    .iter()
    .map(|window| window.preview.screen_rect.center())
    .collect::<Vec<_>>();
  let from = focused.and_then(|hwnd| {
    scene
      .windows
      .iter()
      .position(|window| window.preview.hwnd == hwnd)
  });
  let delays = ripple_delays(&centers, from, stagger_ms, within_ms);
  for (window, delay) in scene.windows.iter_mut().zip(delays) {
    window.hold_for(delay / 1000.0);
  }
}

/// `slide` one way or the other, for a workspace that `order` puts
/// before, at or after the shown one.
fn slide_offset(slide: (f32, f32), order: Ordering) -> (f32, f32) {
  match order {
    Ordering::Less => (-slide.0, -slide.1),
    Ordering::Equal => (0.0, 0.0),
    Ordering::Greater => slide,
  }
}

#[cfg(test)]
mod tests {
  use super::{peek_rect, ripple_delays, scene_window_shown, PEEK_SHARE};
  use crate::overview::{
    layout::{RectF, Tile},
    scene::SceneWindow,
    thumbnails::Preview,
  };

  #[test]
  fn a_landing_window_reaches_where_it_settles_as_the_zoom_ends() {
    let preview = Preview::new(
      0,
      Tile {
        rect: RectF::default(),
        crop: RectF::new(0.0, 0.0, 1.0, 1.0),
      },
    );
    let mut window = SceneWindow::new(preview, (0.0, 0.0), 300.0, 0.0);
    let scene = RectF::new(200.0, 200.0, 800.0, 600.0);
    window.retarget(&scene, 1.0, false);
    window.appear_from(&scene.translate(1000.0, 0.0));

    // Where it is as the zoom sets off, faint.
    let (start, alpha) = scene_window_shown(&window, Some(1.0));
    assert_eq!(start, window.rect());
    assert!(alpha < 0.01);

    // Where it settles, at full strength, as the zoom ends, though its
    // springs haven't moved.
    let (end, alpha) = scene_window_shown(&window, Some(0.0));
    assert_eq!(end, scene);
    assert!((alpha - 1.0).abs() < f32::EPSILON);

    // Not landing, it shows where it is.
    assert_eq!(scene_window_shown(&window, None).0, window.rect());
  }

  /// The focused window flies first, the others by how far they are from
  /// it, and many of them still all start within the time given.
  #[test]
  fn windows_ripple_out_from_the_focused_one() {
    let centers = [(1500.0, 500.0), (100.0, 500.0), (900.0, 500.0)];
    let delays = ripple_delays(&centers, Some(2), 20.0, 140.0);
    assert_eq!(delays, vec![20.0, 40.0, 0.0]);

    let many = (0..15)
      .map(|index| {
        #[allow(clippy::cast_precision_loss)]
        let x = index as f32 * 100.0;
        (x, 0.0)
      })
      .collect::<Vec<_>>();
    let delays = ripple_delays(&many, Some(0), 20.0, 140.0);
    assert!((delays[14] - 140.0).abs() < 1e-3, "{delays:?}");

    assert_eq!(ripple_delays(&centers, None, 0.0, 140.0), vec![0.0; 3]);
  }

  /// A peeked window grows to most of the screen, centered, but no larger
  /// than it is.
  #[test]
  fn a_peeked_window_takes_most_of_the_screen() {
    let view = RectF::new(0.0, 0.0, 1920.0, 1080.0);
    let large = peek_rect(&RectF::new(0.0, 0.0, 2560.0, 1440.0), &view);
    assert!((large.h - 1080.0 * PEEK_SHARE.1).abs() < 0.01, "{large:?}");
    assert!(large.w <= 1920.0 * PEEK_SHARE.0 + 0.01);
    assert_eq!(large.center(), view.center());

    let small = peek_rect(&RectF::new(50.0, 50.0, 400.0, 300.0), &view);
    assert!((small.w - 400.0).abs() < 0.01, "{small:?}");
    assert_eq!(small.center(), view.center());
  }
}
