//! What the overview shows over and around its cards and windows: the
//! highlights, the picked window's tag and the workspaces' names.

use std::time::Instant;

use windows::Win32::Foundation::HWND;

use super::{
  scenes::scene_window_rect, to_f32_size, PlaceContext, Screen,
};
use crate::{
  dwm_thumbnail::Thumbnail,
  overview_chrome::{
    card_radius, draw_label, draw_note, draw_tag, label_size, note_size,
    tag_size, with_alpha, LabelPicture, SliceLook, TagPicture,
  },
  overview_layout::{
    chrome_opacity, Camera, CardMetrics, RectF, SpringRect, LABEL_GAP,
  },
  overview_overlays::{Highlight, Tag},
  overview_state::Mode,
  overview_thumbnails::Picture,
  OverviewLayoutMode, OverviewStyle,
};

/// Corner radius a workspace's highlight is drawn with, and its gap to
/// what it highlights, in logical pixels.
const RING_RADIUS: f32 = 16.0;
const RING_GAP: f32 = 5.0;

/// Corner radius a window's highlight is drawn with, in logical pixels.
const WINDOW_RING_RADIUS: f32 = 10.0;

/// Corner radius of the ring around the screen with the keyboard, and its
/// gap to the screen's edges, in logical pixels.
const SCREEN_RING_RADIUS: f32 = 14.0;
const SCREEN_RING_GAP: f32 = 6.0;

/// A workspace's name beside its card, for cards without a header.
pub(super) struct Label {
  pub(super) picture: Picture,
  pub(super) drawn: Option<LabelPicture>,
}

/// What the overview shows over and around its cards and windows.
pub(super) struct Overlays {
  /// Around the screen with the keyboard, when there are several.
  pub(super) screen_ring: Highlight<()>,

  /// Around the selected workspace, or the one a window would drop on.
  pub(super) workspace_ring: Highlight<usize>,

  /// Around the workspace under the cursor.
  pub(super) hover_ring: Highlight<usize>,

  /// Around the picked window.
  pub(super) window_ring: Highlight<isize>,

  /// The picked window's title, under it.
  pub(super) tag: Tag<TagPicture>,

  /// Where the shown workspace has no windows.
  pub(super) note: Tag<String>,

  /// The shown workspace's working area as the activities' scene lays it
  /// out, which its note centers on.
  pub(super) workspace_rect: SpringRect,

  /// A name per workspace.
  pub(super) labels: Vec<Label>,
}

impl Overlays {
  /// Overlays whose highlights and tags glide over `highlight_ms`, and
  /// whose shown workspace's area changes shape over `switch_ms`.
  pub(super) fn new(switch_ms: f32, highlight_ms: f32) -> Option<Self> {
    Some(Self {
      screen_ring: Highlight::new(highlight_ms)?,
      workspace_ring: Highlight::new(highlight_ms)?,
      hover_ring: Highlight::new(highlight_ms)?,
      window_ring: Highlight::new(highlight_ms)?,
      tag: Tag::new(highlight_ms)?,
      note: Tag::new(highlight_ms)?,
      workspace_rect: SpringRect::new(RectF::default(), switch_ms),
      labels: Vec::new(),
    })
  }

  /// Glides the highlights and tags over `highlight_ms`, and changes the
  /// shown workspace's area over `switch_ms`, from now on.
  pub(super) fn retune(&mut self, switch_ms: f32, highlight_ms: f32) {
    self.screen_ring.glide.retune(highlight_ms);
    self.workspace_ring.glide.retune(highlight_ms);
    self.hover_ring.glide.retune(highlight_ms);
    self.window_ring.glide.retune(highlight_ms);
    self.tag.glide.retune(highlight_ms);
    self.note.glide.retune(highlight_ms);
    self.workspace_rect.retune(switch_ms);
  }

  /// Registers what `layout` shows of them on `host`, bottom to top.
  pub(super) fn register(
    &mut self,
    host: HWND,
    layout: OverviewLayoutMode,
  ) {
    if matches!(
      layout,
      OverviewLayoutMode::Activities | OverviewLayoutMode::Stage
    ) {
      for label in &mut self.labels {
        label.picture.thumbnail =
          Thumbnail::register(host, label.picture.window);
      }
    }

    self.screen_ring.slices.register(host);
    self.hover_ring.slices.register(host);
    self.workspace_ring.slices.register(host);
    self.window_ring.slices.register(host);
    self.tag.picture.thumbnail =
      Thumbnail::register(host, self.tag.picture.window);

    if layout.has_scene() {
      self.note.picture.thumbnail =
        Thumbnail::register(host, self.note.picture.window);
    }
  }

  /// Hands their thumbnails over to `retired`, still registered.
  pub(super) fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    for label in &mut self.labels {
      retired.extend(label.picture.thumbnail.take());
    }
    for slices in [
      &mut self.screen_ring.slices,
      &mut self.hover_ring.slices,
      &mut self.workspace_ring.slices,
      &mut self.window_ring.slices,
    ] {
      slices.retire(retired);
    }
    retired.extend(self.tag.picture.thumbnail.take());
    retired.extend(self.note.picture.thumbnail.take());
  }

  /// Draws the highlights in `style`, at `metrics`' scale; each only if
  /// its look changed.
  #[allow(clippy::cast_possible_truncation)]
  fn draw_looks(&mut self, style: &OverviewStyle, metrics: &CardMetrics) {
    let px = |length: f32| (length * metrics.scale_factor).round() as i32;
    let accent = style.accent;
    self.screen_ring.slices.draw(SliceLook::ring(
      px(SCREEN_RING_RADIUS),
      px(3.0).max(1),
      with_alpha(accent, 0.7),
    ));
    self.workspace_ring.slices.draw(SliceLook::ring(
      px(RING_RADIUS),
      px(3.0).max(1),
      accent,
    ));
    self.hover_ring.slices.draw(SliceLook::ring(
      px(RING_RADIUS),
      px(2.0).max(1),
      with_alpha(accent, 0.45),
    ));
    self.window_ring.slices.draw(SliceLook::ring(
      px(WINDOW_RING_RADIUS),
      px(3.0).max(1),
      accent,
    ));
  }

  /// Hides every highlight at once, e.g. for a new open.
  pub(super) fn reset(&mut self) {
    self.screen_ring.glide.reset();
    self.workspace_ring.glide.reset();
    self.hover_ring.glide.reset();
    self.window_ring.glide.reset();
    self.tag.glide.reset();
    self.note.glide.reset();
  }

  /// Forgets what the pictures show, so they are all drawn anew.
  pub(super) fn forget_drawn(&mut self) {
    self.tag.drawn = None;
    self.note.drawn = None;
    for label in &mut self.labels {
      label.drawn = None;
    }
  }

  /// Advances their fades by `dt` seconds. Returns whether any still
  /// moves.
  pub(super) fn step(&mut self, dt: f32) -> bool {
    [
      self.screen_ring.glide.step(dt),
      self.workspace_ring.glide.step(dt),
      self.hover_ring.glide.step(dt),
      self.window_ring.glide.step(dt),
      self.tag.glide.step(dt),
      self.note.glide.step(dt),
      self.workspace_rect.step(dt),
    ]
    .contains(&true)
  }
}

impl Screen {
  /// Keeps a label picture per workspace.
  pub(super) fn ensure_labels(&mut self) {
    let count = self.workspaces().len();
    let Some(overlays) = &mut self.overlays else {
      return;
    };

    while overlays.labels.len() < count {
      let Some(picture) = Picture::new() else {
        break;
      };
      overlays.labels.push(Label {
        picture,
        drawn: None,
      });
    }
    overlays.labels.truncate(count);
  }

  /// Redraws the highlights, labels and tags whose content changed, or all
  /// of them when `force`.
  pub(super) fn refresh_overlays(
    &mut self,
    style: &OverviewStyle,
    force: bool,
  ) {
    let layout = self.focus.layout;
    let metrics = self.metrics;
    let scale_factor = metrics.scale_factor;
    let notify = self.notify;

    let labels = matches!(
      layout,
      OverviewLayoutMode::Activities | OverviewLayoutMode::Stage
    )
    .then(|| self.label_pictures())
    .unwrap_or_default();
    let tag = self.tag_picture();
    let note = self.note_text();
    let max_tag = (560.0 * scale_factor).min(metrics.view.0 * 0.6);

    let Some(overlays) = &mut self.overlays else {
      return;
    };
    if force {
      overlays.forget_drawn();
    }

    overlays.draw_looks(style, &metrics);

    for (label, picture) in overlays.labels.iter_mut().zip(labels) {
      if label.drawn.as_ref() != Some(&picture) {
        let size = label_size(&picture, style, scale_factor);
        label.picture.draw(size, |surface| {
          draw_label(surface, &picture, style, scale_factor);
        });
        label.drawn = Some(picture);
      }
    }

    if let Some(tag) = tag {
      if overlays.tag.drawn.as_ref() != Some(&tag) {
        let size = tag_size(&tag, style, scale_factor, max_tag);
        overlays.tag.picture.draw(size, |surface| {
          draw_tag(surface, &tag, style, scale_factor, notify);
        });
        overlays.tag.drawn = Some(tag);
      }
    }

    if let Some(note) = note {
      if overlays.note.drawn.as_ref() != Some(&note) {
        let size = note_size(&note, style, scale_factor);
        overlays.note.picture.draw(size, |surface| {
          draw_note(surface, &note, style, scale_factor);
        });
        overlays.note.drawn = Some(note);
      }
    }
  }

  /// Each workspace's name beside its card.
  pub(super) fn label_pictures(&self) -> Vec<LabelPicture> {
    self
      .workspaces()
      .iter()
      .map(|workspace| LabelPicture {
        text: workspace.label.clone(),
        is_displayed: workspace.is_displayed,
        is_special: workspace.is_special,
      })
      .collect()
  }

  /// The tag under the picked window, where windows aren't captioned on
  /// cards.
  pub(super) fn tag_picture(&self) -> Option<TagPicture> {
    let focus = &self.focus;
    if !focus.layout.shows_before_going() || focus.mode != Mode::Windows {
      return None;
    }

    let hwnd = focus.selected_window?;
    let window = self
      .workspaces()
      .get(focus.selected)?
      .windows
      .iter()
      .find(|window| window.hwnd == hwnd)?;

    Some(TagPicture {
      hwnd,
      text: if window.title.is_empty() {
        window.process_name.clone()
      } else {
        window.title.clone()
      },
      icons: self.icons,
    })
  }

  /// What shows where the shown workspace has no windows.
  pub(super) fn note_text(&self) -> Option<String> {
    if !self.focus.layout.has_scene() {
      return None;
    }

    let index = self.focus.selected;
    let workspace = self.workspaces().get(index)?;
    let is_empty = self
      .scenes
      .get(index)
      .is_none_or(|scene| scene.windows.is_empty());

    is_empty.then(|| {
      if workspace.is_new {
        "New workspace"
      } else if workspace.is_special {
        "Drop windows here"
      } else {
        "No windows"
      }
      .to_string()
    })
  }

  /// How far up the activities' miniatures are while the overview opens
  /// or closes: out of sight when it is shut.
  pub(super) fn top_shift(&self, openness: f32) -> f32 {
    let extent = match self.focus.layout {
      OverviewLayoutMode::Activities => self.scene_area.y,
      _ => 0.0,
    };

    -(1.0 - openness) * (extent + self.metrics.px(12.0))
  }

  /// Where workspace `index` shows now in the layout: its card or its
  /// miniature, with how much its corners are scaled from a highlight's.
  pub(super) fn workspace_rect(
    &self,
    index: usize,
    context: &PlaceContext,
    shift: f32,
  ) -> Option<(RectF, f32)> {
    let gap = self.metrics.px(RING_GAP);

    let card = self.cards.get(index)?;
    let camera = if self.focus.layout == OverviewLayoutMode::Activities {
      Camera::shifted(0.0, shift)
    } else {
      context.camera
    };
    let rect = camera.apply(&card.placement().rect(&card.metrics));
    let scale = rect.w / card.metrics.width.max(1.0);
    let radius = card_radius(&card.metrics) * scale + gap;

    Some((rect.inset(-gap), radius / self.metrics.px(RING_RADIUS)))
  }

  /// Where window `hwnd` of the selected workspace shows now.
  pub(super) fn window_rect(
    &self,
    hwnd: isize,
    context: &PlaceContext,
  ) -> Option<RectF> {
    let selected = self.focus.selected;

    if self.focus.layout.has_scene() {
      let window = self
        .scenes
        .get(selected)?
        .windows
        .iter()
        .find(|window| window.preview.hwnd == hwnd)?;
      return Some(scene_window_rect(
        window,
        context.zoom_card == Some(selected),
        context.fade,
      ));
    }

    let card = self.cards.get(selected)?;
    let preview =
      card.previews.iter().find(|preview| preview.hwnd == hwnd)?;
    Some(
      context
        .camera
        .apply(&card.placement().map(&card.metrics, &preview.tile.rect)),
    )
  }

  /// Glides the highlights to what they highlight now and places them,
  /// with the labels and tags. Returns whether any is still gliding.
  #[allow(clippy::too_many_lines)]
  pub(super) fn place_overlays(
    &mut self,
    context: &PlaceContext,
    shift: f32,
  ) -> bool {
    let now = Instant::now();
    let focus = &self.focus;
    let layout = focus.layout;
    let selected = focus.selected;
    let view = context.view;
    let fade = context.fade;
    let chrome = chrome_opacity(fade);
    let px = |length: f32| length * self.metrics.scale_factor;

    // Where a window would drop: on what it is dragged over, or where it
    // is carried to. The selection shows on the screen with the keyboard,
    // and in the stage as the workspace shown large.
    let drop = focus.drop;
    let ringed = if layout == OverviewLayoutMode::Stage
      || focus.is_dragging
      || !focus.is_active
    {
      drop
    } else {
      drop.or(Some(selected))
    };
    let ring_target = ringed.and_then(|index| {
      self
        .workspace_rect(index, context, shift)
        .map(|(rect, scale)| (index, rect, scale))
    });

    let hover_target = focus
      .hover
      .filter(|hovered| {
        *hovered != selected
          && Some(*hovered) != ringed
          && !focus.is_dragging
      })
      .and_then(|index| {
        self
          .workspace_rect(index, context, shift)
          .map(|(rect, scale)| (index, rect, scale))
      });

    let picked = focus
      .selected_window
      .filter(|_| focus.mode == Mode::Windows);
    let picked_rect = picked.and_then(|hwnd| {
      self.window_rect(hwnd, context).map(|rect| (hwnd, rect))
    });
    let window_target = picked_rect
      .map(|(hwnd, rect)| (hwnd, rect.inset(-px(RING_GAP)), 1.0));

    let tag_target = picked_rect
      .filter(|_| layout.shows_before_going())
      .zip(self.overlays.as_ref())
      .map(|((hwnd, rect), overlays)| {
        let (width, height) = to_f32_size(overlays.tag.picture.size());
        let x = (rect.x + rect.w / 2.0 - width / 2.0)
          .clamp(px(8.0), (view.w - width - px(8.0)).max(px(8.0)));
        // Under the window, or else over its bottom edge.
        let below = rect.bottom() + px(8.0);
        let y = if below + height <= self.search_y.target - px(4.0) {
          below
        } else if rect.h > height * 2.0 {
          rect.bottom() - height - px(10.0)
        } else {
          rect.y - px(8.0) - height
        };
        (hwnd, RectF::new(x, y, width, height), 1.0)
      });

    let note_target =
      self
        .note_text()
        .zip(self.overlays.as_ref())
        .map(|(_, overlays)| {
          let (width, height) = to_f32_size(overlays.note.picture.size());
          let (cx, cy) = overlays.workspace_rect.value().center();
          #[allow(clippy::cast_possible_wrap)]
          let key = selected as isize;
          (
            key,
            RectF::new(cx - width / 2.0, cy - height / 2.0, width, height),
            1.0,
          )
        });

    let screen_target = focus
      .marks_keyboard
      .then(|| ((), view.inset(px(SCREEN_RING_GAP)), 1.0));

    let label_rects = self.label_rects(context, shift);

    let Some(overlays) = &mut self.overlays else {
      return false;
    };
    let mut is_gliding = false;

    is_gliding |= overlays.screen_ring.glide.follow(screen_target, now);
    overlays.screen_ring.place(&view, chrome);
    is_gliding |= overlays.workspace_ring.glide.follow(ring_target, now);
    overlays.workspace_ring.place(&view, chrome);
    is_gliding |= overlays.hover_ring.glide.follow(hover_target, now);
    overlays.hover_ring.place(&view, chrome);
    is_gliding |= overlays.window_ring.glide.follow(window_target, now);
    overlays.window_ring.place(&view, chrome);

    for (label, rect) in overlays.labels.iter_mut().zip(&label_rects) {
      if let Some((rect, opacity)) = rect {
        label.picture.place(rect, &view, *opacity);
      } else if let Some(thumbnail) = &mut label.picture.thumbnail {
        thumbnail.hide();
      }
    }

    is_gliding |= overlays.tag.glide.follow(tag_target, now);
    overlays.tag.place(&view, chrome);
    is_gliding |= overlays.note.glide.follow(note_target, now);
    overlays.note.place(&view, chrome);

    is_gliding
  }

  /// Where each workspace's name shows now, and how strongly: under its
  /// miniature, or in the stage's column, under its card or over the
  /// shown one.
  pub(super) fn label_rects(
    &self,
    context: &PlaceContext,
    shift: f32,
  ) -> Vec<Option<(RectF, f32)>> {
    let layout = self.focus.layout;
    let gap = self.metrics.px(LABEL_GAP);
    let Some(overlays) = &self.overlays else {
      return Vec::new();
    };

    self
      .cards
      .iter()
      .zip(&overlays.labels)
      .enumerate()
      .map(|(index, (card, label))| {
        let camera = match layout {
          OverviewLayoutMode::Activities => Camera::shifted(0.0, shift),
          OverviewLayoutMode::Stage => context.camera,
          _ => return None,
        };
        let rect = camera.apply(&card.placement().rect(&card.metrics));
        let (width, height) = to_f32_size(label.picture.size());

        let is_over = layout == OverviewLayoutMode::Stage
          && index == self.focus.selected;
        let (x, y) = if is_over {
          (rect.x, rect.y - gap - height)
        } else {
          (rect.x + rect.w / 2.0 - width / 2.0, rect.bottom() + gap)
        };

        Some((
          RectF::new(x, y, width, height),
          chrome_opacity(context.fade) * card.alpha.value,
        ))
      })
      .collect()
  }
}
