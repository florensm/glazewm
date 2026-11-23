//! The scenes of the activities: each workspace's windows laid out on
//! their own rather than on its card.

use std::cmp::Ordering;

use super::{cards::screen_rect, to_ms, Screen, ScreenFrame};

/// Gap, in logical pixels, a sliding scene keeps from the one it slides
/// past.
const SLIDE_GAP: f32 = 48.0;

use crate::{
  overview::{
    chrome::STAND_IN_SIZE,
    expose::spread,
    layout::{CardView, RectF, Tile},
    scene::{Scene, SceneWindow},
    state::ScreenFocus,
  },
  OverviewExposeArrangement, OverviewStripView, OverviewSwitchStyle,
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
  max_scale: f32,
  max_per_row: usize,
  arrangement: OverviewExposeArrangement,
  /// Size of a minimized window's stand-in, in physical pixels.
  stand_in: (f32, f32),
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
/// given, every tab of a stack a window of its own, and its minimized
/// windows' stand-ins in a row along the bottom. `None` for a window left
/// out.
fn spread_windows(
  scene: &Scene,
  how: &Spread,
  only: Option<&[isize]>,
) -> Vec<Option<RectF>> {
  let (minimized, shown): (Vec<_>, Vec<_>) = scene
    .windows
    .iter()
    .enumerate()
    .filter(|(_, window)| {
      only.is_none_or(|only| only.contains(&window.preview.hwnd))
    })
    .partition(|(_, window)| window.is_minimized);

  let (row, area) = stand_in_row(
    &how.area,
    minimized.len(),
    how.stand_in,
    how.gap,
    !shown.is_empty(),
  );
  let screen_rects = shown
    .iter()
    .map(|(_, window)| window.preview.screen_rect)
    .collect::<Vec<_>>();
  let slots = spread(
    &screen_rects,
    &area,
    how.gap,
    how.max_scale,
    how.max_per_row,
    how.arrangement,
  );

  let mut rects = vec![None; scene.windows.len()];
  for ((index, _), slot) in shown.iter().zip(slots) {
    rects[*index] = Some(slot);
  }
  for ((index, _), slot) in minimized.iter().zip(row) {
    rects[*index] = Some(slot);
  }
  rects
}

/// Where a row of `count` stand-ins, each `size` big and `gap` apart,
/// shows: centered along the bottom of `area`, shrunk to fit its width and
/// at most a third of its height, or in its middle without `others`.
/// Returns them left to right, and the part of `area` left above them.
fn stand_in_row(
  area: &RectF,
  count: usize,
  size: (f32, f32),
  gap: f32,
  others: bool,
) -> (Vec<RectF>, RectF) {
  if count == 0 || size.0 <= 0.0 || size.1 <= 0.0 {
    return (Vec::new(), *area);
  }

  #[allow(clippy::cast_precision_loss)]
  let count_f = count as f32;
  let gaps = gap * (count_f - 1.0);
  let scale = ((area.w - gaps).max(0.0) / (size.0 * count_f))
    .min(area.h / 3.0 / size.1)
    .min(1.0);
  let (width, height) = (size.0 * scale, size.1 * scale);

  let row_width = width * count_f + gaps;
  let left = area.x + (area.w - row_width) / 2.0;
  let top = if others {
    area.bottom() - height
  } else {
    area.y + (area.h - height) / 2.0
  };

  #[allow(clippy::cast_precision_loss)]
  let row = (0..count)
    .map(|index| {
      RectF::new(left + index as f32 * (width + gap), top, width, height)
    })
    .collect();
  let above =
    RectF::new(area.x, area.y, area.w, (area.h - height - gap).max(0.0));
  (row, above)
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
    slide: f32,
  ) {
    let selected = self.focus.selected;
    if style == OverviewSwitchStyle::Miniature {
      self.seed_scene_from_card(true);
      return;
    }

    self.scene_shown = Some(selected);
    let offset = match (style, previous) {
      (OverviewSwitchStyle::Slide, Some(previous)) => {
        match selected.cmp(&previous) {
          Ordering::Less => -slide,
          Ordering::Equal => 0.0,
          Ordering::Greater => slide,
        }
      }
      _ => 0.0,
    };

    let Some(scene) = self.scenes.get_mut(selected) else {
      return;
    };
    for window in &mut scene.windows {
      if window.alpha.value <= 0.05 {
        let from = window.target().translate(offset, 0.0);
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
    // A full slide clears the scene; a fade stays in place.
    let slide = match style {
      OverviewSwitchStyle::Slide => {
        self.scene_area.w + self.metrics.px(SLIDE_GAP)
      }
      OverviewSwitchStyle::Fade => 0.0,
      OverviewSwitchStyle::Miniature => self.metrics.view.0 * 0.12,
    };

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

    for (index, (scene, workspace)) in
      self.scenes.iter_mut().zip(&frame.workspaces).enumerate()
    {
      let offset = match index.cmp(&selected) {
        Ordering::Less => -slide,
        Ordering::Equal => 0.0,
        Ordering::Greater => slide,
      };
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
            window.retarget(&rect.translate(offset, 0.0), 0.0, true);
          }
        } else if let (true, Some(tile)) = (flies_out, tile) {
          let shown = if window.is_hidden_tab { 0.0 } else { alpha };
          window.retarget(&tile.rect.translate(offset, 0.0), shown, true);
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
            window.retarget(&rect.translate(offset, 0.0), alpha, snap);
          }
          (None, _) => {
            let target = window.target();
            window.retarget(&target, 0.0, snap);
          }
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

  /// How the exposé spreads a scene of `frame` over the scene area.
  fn spread_as(&self, frame: &ScreenFrame) -> Spread {
    let area = self.scene_area;
    let monitor = RectF::from_rect(&frame.rect);
    let expose = frame.style.expose;
    Spread {
      area,
      gap: self.metrics.px(expose.gap.max(0.0)),
      // Never larger than the activities show them.
      max_scale: (area.w / monitor.w).min(area.h / monitor.h).max(0.0),
      max_per_row: usize::try_from(expose.max_per_row)
        .unwrap_or(usize::MAX),
      arrangement: expose.arrangement,
      stand_in: (
        self.metrics.px(STAND_IN_SIZE.0),
        self.metrics.px(STAND_IN_SIZE.1),
      ),
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

#[cfg(test)]
mod tests {
  use super::{scene_window_shown, stand_in_row};
  use crate::overview::{
    layout::{RectF, Tile},
    scene::SceneWindow,
    thumbnails::Preview,
  };

  #[test]
  fn stand_ins_sit_in_a_centered_row_below_the_other_windows() {
    let area = RectF::new(0.0, 0.0, 1000.0, 900.0);
    let (row, above) = stand_in_row(&area, 2, (240.0, 150.0), 20.0, true);

    assert_eq!(
      row,
      [
        RectF::new(250.0, 750.0, 240.0, 150.0),
        RectF::new(510.0, 750.0, 240.0, 150.0),
      ]
    );
    assert_eq!(above, RectF::new(0.0, 0.0, 1000.0, 730.0));
  }

  #[test]
  fn stand_ins_shrink_to_fit_and_center_alone() {
    let area = RectF::new(0.0, 0.0, 500.0, 900.0);
    let (row, above) = stand_in_row(&area, 4, (240.0, 150.0), 20.0, false);

    // 4 x 110 wide + 3 gaps of 20 fill the 500 across.
    assert_eq!(row.len(), 4);
    assert!((row[0].w - 110.0).abs() < 0.01);
    assert!((row[3].right() - 500.0).abs() < 0.01);
    // Without other windows, in the middle.
    assert!(((row[0].y + row[0].h / 2.0) - 450.0).abs() < 0.01);
    assert!(above.h < area.h);
  }

  #[test]
  fn no_stand_ins_leave_the_area_whole() {
    let area = RectF::new(10.0, 20.0, 500.0, 400.0);
    assert_eq!(
      stand_in_row(&area, 0, (240.0, 150.0), 20.0, true),
      (Vec::new(), area)
    );
  }

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
}
