//! Windows of the workspaces shown on their own rather than on a card, as
//! laid out in the activities.
//! Each has a preview and springs of its own, so it can fly between where
//! it is on screen and where the layout puts it.

use crate::{
  overview_layout::{RectF, Spring, SpringRect, Tile},
  overview_thumbnails::Preview,
  OverviewWindow, OverviewWorkspace,
};

/// A window on a scene.
pub(crate) struct SceneWindow {
  pub preview: Preview,
  rect: SpringRect,
  pub alpha: Spring,
}

impl SceneWindow {
  /// Starts out invisible where it is on screen, settling within about
  /// `settle_ms` once retargeted.
  fn new(preview: Preview, settle_ms: f32) -> Self {
    let rect = SpringRect::new(preview.screen_rect, settle_ms);
    Self {
      preview,
      rect,
      alpha: Spring::new(0.0, settle_ms, 0.002),
    }
  }

  /// Settles within about `settle_ms` from now on.
  fn retune(&mut self, settle_ms: f32) {
    self.rect.retune(settle_ms);
    self.alpha.retune(settle_ms);
  }

  /// Where it is now.
  pub fn rect(&self) -> RectF {
    self.rect.value()
  }

  /// Where it heads.
  pub fn target(&self) -> RectF {
    self.rect.target()
  }

  /// Points it at `rect` and `alpha`; there at once if `snap`.
  pub fn retarget(&mut self, rect: &RectF, alpha: f32, snap: bool) {
    self.rect.retarget(rect, snap);
    if snap {
      self.alpha.snap(alpha);
    } else {
      self.alpha.target = alpha;
    }
  }

  /// Puts it at `rect` and `alpha` at once, keeping where it heads.
  pub fn jump(&mut self, rect: &RectF, alpha: f32) {
    let target = self.target();
    let alpha_target = self.alpha.target;
    self.retarget(rect, alpha, true);
    self.retarget(&target, alpha_target, false);
  }

  /// Stops it where it is now, still fading toward its target.
  pub fn hold(&mut self) {
    self.rect.hold();
  }

  /// Advances by `dt` seconds. Returns whether it is still moving.
  pub fn step(&mut self, dt: f32) -> bool {
    self.rect.step(dt) | self.alpha.step(dt)
  }
}

/// A workspace's windows laid out on their own, bottom to top.
#[derive(Default)]
pub(crate) struct Scene {
  /// Name of its workspace, to find it again when workspaces change.
  pub name: String,
  pub windows: Vec<SceneWindow>,
}

impl Scene {
  /// The scene of `workspace`'s shown windows, keeping the windows of
  /// `old` that are still there, each settling within about `settle_ms`.
  /// `screen_rect` says where a window is on screen.
  ///
  /// Returns it, and whether its windows or their overlays changed, which
  /// calls for registering their thumbnails anew.
  pub fn rebuild(
    mut old: Scene,
    workspace: &OverviewWorkspace,
    settle_ms: f32,
    screen_rect: impl Fn(&OverviewWindow) -> RectF,
  ) -> (Self, bool) {
    let old_order = old
      .windows
      .iter()
      .map(|window| window.preview.hwnd)
      .collect::<Vec<_>>();
    let mut is_restyled = false;

    let windows = workspace
      .windows
      .iter()
      .filter(|window| !window.is_minimized && !window.is_hidden_tab)
      .map(|window| {
        let rect = screen_rect(window);
        let reused = old
          .windows
          .iter()
          .position(|old| old.preview.hwnd == window.hwnd)
          .map(|index| old.windows.swap_remove(index));

        let mut scene_window = if let Some(mut reused) = reused {
          reused.preview.refresh_frame();
          reused.retune(settle_ms);
          reused
        } else {
          let mut preview = Preview::new(
            window.hwnd,
            Tile {
              rect: RectF::default(),
              crop: RectF::new(0.0, 0.0, 1.0, 1.0),
            },
          );
          preview.screen_rect = rect;
          SceneWindow::new(preview, settle_ms)
        };

        scene_window.preview.screen_rect = rect;
        scene_window.preview.is_scrolled_off = window.is_scrolled_off;
        is_restyled |= scene_window.preview.restyle(window);
        scene_window
      })
      .collect::<Vec<_>>();

    let is_changed = is_restyled
      || windows
        .iter()
        .map(|window| window.preview.hwnd)
        .ne(old_order);

    (
      Self {
        name: workspace.name.clone(),
        windows,
      },
      is_changed,
    )
  }

  /// Indices of its windows left to right, then top to bottom, by where
  /// they are on screen: the order the arrows step through them in.
  pub fn reading_order(&self) -> Vec<usize> {
    let mut order = (0..self.windows.len()).collect::<Vec<_>>();
    order.sort_by(|a, b| {
      let (a, b) = (
        &self.windows[*a].preview.screen_rect,
        &self.windows[*b].preview.screen_rect,
      );
      a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y))
    });
    order
  }

  /// Advances every window by `dt` seconds. Returns whether any is still
  /// moving.
  pub fn step(&mut self, dt: f32) -> bool {
    self
      .windows
      .iter_mut()
      .fold(false, |is_moving, window| window.step(dt) | is_moving)
  }
}
