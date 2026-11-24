//! The scenes of the activities: each workspace's windows laid out on
//! their own rather than on its card.

use std::cmp::Ordering;

use super::{cards::screen_rect, to_ms, Screen, ScreenFrame};

/// Gap, in logical pixels, a sliding scene keeps from the one it slides
/// past.
const SLIDE_GAP: f32 = 48.0;

use crate::{
  overview::{
    expose::spread,
    layout::{CardView, RectF, Tile},
    scene::{Scene, SceneWindow},
    state::ScreenFocus,
  },
  OverviewRowSide, OverviewStripView, OverviewSwitchStyle,
  OverviewWorkspace,
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
  /// Space between windows, in physical pixels.
  gap: f32,
  /// Most of `area` either way the windows take.
  largest: f32,
}

/// Where each window of `scene` settles, given its `tiles` as laid out:
/// there, or spread as `spread` has it. `None` for a window not shown,
/// e.g. a stack's other tab where it isn't spread. Crops each preview as
/// it then shows.
fn scene_rects(
  scene: &mut Scene,
  tiles: &[Option<Tile>],
  spread_as: Option<&Spread>,
  only: Option<&[isize]>,
) -> Vec<Option<RectF>> {
  if let Some(how) = spread_as {
    for window in &mut scene.windows {
      window.preview.tile.crop = RectF::new(0.0, 0.0, 1.0, 1.0);
    }
    return spread_windows(scene, how, only);
  }

  scene
    .windows
    .iter_mut()
    .zip(tiles)
    .map(|(window, tile)| {
      let tile = tile.as_ref()?;
      window.preview.tile.crop = tile.crop;
      // Only the exposé shows a stack's other tabs and minimized windows.
      (!window.is_hidden_tab && !window.is_minimized).then_some(tile.rect)
    })
    .collect()
}

/// Where the exposé shows each window of `scene`, of those in `only` if
/// given, every tab of a stack and minimized window a window of its own.
/// `None` for a window left out.
fn spread_windows(
  scene: &Scene,
  how: &Spread,
  only: Option<&[isize]>,
) -> Vec<Option<RectF>> {
  let shown = scene
    .windows
    .iter()
    .enumerate()
    .filter(|(_, window)| {
      only.is_none_or(|only| only.contains(&window.preview.hwnd))
    })
    .map(|(index, window)| (index, window.preview.screen_rect))
    .collect::<Vec<_>>();
  let screen_rects =
    shown.iter().map(|(_, rect)| *rect).collect::<Vec<_>>();
  let slots = spread(&screen_rects, &how.area, how.gap, how.largest);

  let mut rects = vec![None; scene.windows.len()];
  for ((index, _), slot) in shown.iter().zip(slots) {
    rects[*index] = Some(slot);
  }
  rects
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
      let rects = scene_rects(
        scene,
        &tiles,
        spreading.then_some(&spread_as),
        matches.as_deref(),
      );

      for ((window, rect), tile) in
        scene.windows.iter_mut().zip(rects).zip(&tiles)
      {
        if window.is_minimized {
          // Nowhere on screen to fly out of: it fades in where it
          // settles.
          if let (true, Some(rect)) = (flies_out, rect) {
            window.retarget(
              &rect.translate(offset.0, offset.1),
              0.0,
              true,
            );
          }
        } else if let (true, Some(tile)) = (flies_out, tile) {
          let shown = if window.is_hidden_tab { 0.0 } else { alpha };
          window.retarget(
            &tile.rect.translate(offset.0, offset.1),
            shown,
            true,
          );
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
          (Some(rect), _) => {
            window.retarget(
              &rect.translate(offset.0, offset.1),
              alpha,
              snap,
            );
          }
          (None, _) => {
            let target = window.target();
            window.retarget(&target, 0.0, snap);
          }
        }
      }

      if let (Some((stagger_ms, within_ms)), true) =
        (stagger, index == selected)
      {
        let centers = scene
          .windows
          .iter()
          .map(|window| window.preview.screen_rect.center())
          .collect::<Vec<_>>();
        let from = frame.focused_window.and_then(|hwnd| {
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
    Spread {
      area: self.scene_area,
      gap: self.metrics.px(frame.style.expose.gap.max(0.0)),
      largest: frame.style.expose.clamped_largest(),
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
  use super::{ripple_delays, scene_window_shown};
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
}
