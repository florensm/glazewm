//! The scenes of the activities: each workspace's windows laid out on
//! their own rather than on its card.

use std::cmp::Ordering;

use super::{cards::screen_rect, Screen, SCENE_PADDING};
use crate::{
  overview_layout::{CardView, RectF},
  overview_scene::{Scene, SceneWindow},
  OverviewLayoutMode,
};

/// Where a scene window shows now: on its way between where it is on
/// screen and where the scene puts it while its workspace zooms.
pub(super) fn scene_window_rect(
  window: &SceneWindow,
  is_zooming: bool,
  openness: f32,
) -> RectF {
  if is_zooming {
    window.preview.screen_rect.lerp(&window.rect(), openness)
  } else {
    window.rect()
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

        let (scene, is_scene_changed) =
          Scene::rebuild(previous, workspace, |window| {
            screen_rect(workspace, window)
          });
        is_changed |= is_scene_changed;
        scene
      })
      .collect();

    is_changed
  }

  /// Puts the selected workspace's scene windows where its card shows
  /// them now, so they fly on from there.
  pub(super) fn seed_scene_from_card(&mut self) {
    let context = self.place_context();
    let selected = self.focus.selected;
    let (Some(card), Some(scene)) =
      (self.cards.get(selected), self.scenes.get_mut(selected))
    else {
      return;
    };

    let placement = card.placement();
    for window in &mut scene.windows {
      let preview = card
        .previews
        .iter()
        .find(|preview| preview.hwnd == window.preview.hwnd);
      if let Some(preview) = preview {
        let rect = context
          .camera
          .apply(&placement.map(&card.metrics, &preview.tile.rect));
        window.jump(&rect, 1.0);
      }
    }
  }

  /// Points each scene's windows where the layout shows them: the
  /// selected workspace's in place, the others faded out to its sides.
  pub(super) fn retarget_scenes(&mut self, snap: bool) {
    let Some(frame) = &self.frame else {
      return;
    };

    let layout = self.focus.layout;
    let selected = self.focus.selected;
    let slide = self.metrics.view.0 * 0.12;
    let padding = self.metrics.px(SCENE_PADDING);
    let scene_in = self.scene_area.inset(padding);

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
      let view = CardView::fitted(&content, &scene_in);
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
        if let Some(rect) = rect {
          window.retarget(&rect.translate(offset, 0.0), alpha, snap);
        } else {
          let target = window.target();
          window.retarget(&target, 0.0, snap);
        }
      }
    }

    // The activities' panel takes the shape of what it shows.
    let shown = frame.workspaces.get(selected);
    if let (OverviewLayoutMode::Activities, Some(overlays), Some(shown)) =
      (layout, &mut self.overlays, shown)
    {
      let content = RectF::from_rect(&shown.content_rect);
      let panel = CardView::fitted(&content, &scene_in)
        .map(&content)
        .inset(-padding);
      overlays.scene_panel_rect.retarget(&panel, snap);
    }
  }
}
