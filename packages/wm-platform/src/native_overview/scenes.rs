//! The scenes of the activities: each workspace's windows laid out on
//! their own rather than on its card.

use std::cmp::Ordering;

use super::{cards::screen_rect, to_ms, Screen};

/// Gap, in logical pixels, a sliding scene keeps from the one it slides
/// past.
const SLIDE_GAP: f32 = 48.0;
use crate::{
  overview_layout::{CardView, RectF},
  overview_scene::{Scene, SceneWindow},
  OverviewLayoutMode, OverviewSwitchStyle,
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

  match landing {
    Some(openness) => {
      let closed = 1.0 - openness.clamp(0.0, 1.0);
      (
        rect.lerp(&window.target(), closed),
        alpha + (window.alpha.target - alpha) * closed,
      )
    }
    None => (rect, alpha),
  }
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
    let Some(frame) = &self.frame else {
      return;
    };
    let scene_area = self.scene_area;

    for (index, (scene, workspace)) in
      self.scenes.iter_mut().zip(&frame.workspaces).enumerate()
    {
      let offset = match index.cmp(&selected) {
        Ordering::Less => -slide,
        Ordering::Equal => 0.0,
        Ordering::Greater => slide,
      };
      let alpha = if index == selected { 1.0 } else { 0.0 };

      let content = RectF::from_rect(&workspace.content_rect);
      let view = CardView::fitted(&content, &scene_area);
      let rects = scene
        .windows
        .iter_mut()
        .map(|window| {
          let tile = view.tile(&window.preview.screen_rect)?;
          window.preview.tile.crop = tile.crop;
          Some(tile.rect)
        })
        .collect::<Vec<_>>();

      for (window, rect) in scene.windows.iter_mut().zip(rects) {
        let miniature = miniatures.get(index).and_then(|rects| {
          rects
            .iter()
            .find(|(hwnd, _)| *hwnd == window.preview.hwnd)
            .map(|(_, rect)| *rect)
        });
        let into_miniature =
          style == OverviewSwitchStyle::Miniature && index != selected;

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

    // The shown workspace's working area, laid out as its windows are (a
    // scrolling workspace's strip reaches past it), holding still for a
    // zoom.
    let Some(frame) = &self.frame else {
      return;
    };
    let shown = frame
      .workspaces
      .get(selected)
      .filter(|_| self.closing_card().is_none());
    if let (OverviewLayoutMode::Activities, Some(overlays), Some(shown)) =
      (self.focus.layout, &mut self.overlays, shown)
    {
      let content = RectF::from_rect(&shown.content_rect);
      let area = CardView::fitted(&content, &scene_area)
        .map(&RectF::from_rect(&frame.rect));
      overlays.workspace_rect.retarget(&area, snap);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::scene_window_shown;
  use crate::{
    overview_layout::{RectF, Tile},
    overview_scene::SceneWindow,
    overview_thumbnails::Preview,
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
}
