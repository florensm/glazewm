//! Scrolling layout: the workspace's children are columns on a horizontal
//! strip that can be wider than the workspace, scrolled so the focused
//! column is on screen.
//!
//! A column is a window, a stack, or a split of windows stacked
//! vertically. Its `tiling_size` is its width as a fraction of the
//! strip's view, the workspace less its struts (see `view_rect`); unlike
//! in the other layouts, the columns' sizes don't add up to 1.
//!
//! The view stays put relative to the active column while other
//! columns open, close or resize, and only scrolls when
//! another column is activated or the active one changes width. See
//! `StripView::update`.

use std::ops::RangeInclusive;

use uuid::Uuid;
use wm_common::{
  CenterFocusedColumn, ColumnWidth, Layout, ScrollingLayoutConfig,
};
use wm_platform::{LengthUnit, LengthValue, Rect};

use super::Insertion;
use crate::{
  models::{
    Container, Monitor, TilingContainer, WindowContainer, Workspace,
  },
  traits::{
    CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
  },
  wm_state::WmState,
};

/// Smallest column width, as a fraction of the workspace width.
pub const MIN_COLUMN_SIZE: f32 = 0.1;

/// The scrolling workspace that `container` is a column of, if any.
///
/// Only tiling containers are columns; a floating or fullscreen window of
/// the workspace (e.g. one being dragged) is not on the strip.
pub fn column_workspace(
  container: &impl CommonGetters,
) -> Option<Workspace> {
  container
    .as_tiling_container()
    .ok()?
    .parent()
    .and_then(|parent| parent.as_workspace().cloned())
    .filter(|workspace| workspace.layout() == Layout::Scrolling)
}

/// The column holding `container`, and its scrolling workspace, if
/// `container` is in a scrolling layout.
pub fn column_of(
  container: &Container,
) -> Option<(TilingContainer, Workspace)> {
  container.self_and_ancestors().find_map(|ancestor| {
    let workspace = column_workspace(&ancestor)?;
    Some((ancestor.as_tiling_container().ok()?, workspace))
  })
}

/// The part of `workspace` its strip is fitted into: the workspace less
/// its struts. Columns still show past it, so a column scrolled to its
/// edge leaves the one beside it peeking out over the strut.
pub fn view_rect(workspace: &Workspace) -> anyhow::Result<Rect> {
  let rect = workspace.to_rect()?;
  let struts = workspace.scrolling_struts();
  let scale_factor = workspace.gaps_scale_factor()?;
  let strut =
    |length: &LengthValue| length.to_px(rect.width(), Some(scale_factor));

  let left = rect.left + strut(&struts.left);
  // Struts wider than the workspace leave no room, not a negative width.
  let right = (rect.right - strut(&struts.right)).max(left);

  Ok(Rect::from_ltrb(left, rect.top, right, rect.bottom))
}

/// Width in pixels of a column of `size` on a workspace `workspace_width`
/// wide, `gap` apart from its neighbours.
///
/// Sized so that columns whose sizes add up to 1 fill the workspace
/// exactly, gaps included.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn width_for_size(size: f32, workspace_width: i32, gap: i32) -> i32 {
  ((workspace_width + gap) as f32 * size).round() as i32 - gap
}

/// Width in pixels `column` has at `size` on `workspace`'s strip.
pub fn width_at<T>(
  column: &T,
  workspace: &Workspace,
  size: f32,
) -> anyhow::Result<i32>
where
  T: TilingSizeGetters,
{
  width_on_strip(column, size, view_rect(workspace)?.width())
}

/// Width in pixels `column` has at `size` on a strip seen through a view
/// `view_width` wide.
fn width_on_strip<T>(
  column: &T,
  size: f32,
  view_width: i32,
) -> anyhow::Result<i32>
where
  T: TilingSizeGetters,
{
  let (gap, _) = column.inner_gaps()?;

  Ok(
    width_for_size(size, view_width, gap)
      .max(min_width(column, view_width)),
  )
}

/// Narrowest `column` can be, as its widest window's app allows, but at
/// most `max` (which outer gaps wider than the monitor make negative).
fn min_width(column: &impl CommonGetters, max: i32) -> i32 {
  column
    .self_and_descendants()
    .filter_map(|container| container.as_window_container().ok())
    .map(|window| {
      // The app's minimum is of the window with its shadow borders,
      // which are laid out around the column's width.
      let borders = window.total_border_delta().map_or(0, |delta| {
        delta.left.to_px(0, None) + delta.right.to_px(0, None)
      });

      window.native_properties().min_width - borders
    })
    .max()
    .unwrap_or(0)
    .min(max)
    .max(0)
}

/// Rect of a column on its workspace's strip.
pub(super) fn column_rect<T>(
  column: &T,
  workspace: &Workspace,
) -> anyhow::Result<Rect>
where
  T: TilingSizeGetters,
{
  let view = view_rect(workspace)?;
  let mut x = view.x() - workspace.scroll_offset();

  for sibling in workspace.tiling_children() {
    if sibling.id() == column.id() {
      break;
    }

    let (gap, _) = sibling.inner_gaps()?;
    x +=
      width_on_strip(&sibling, sibling.tiling_size(), view.width())? + gap;
  }

  Ok(Rect::from_xy(
    x,
    view.y(),
    width_on_strip(column, column.tiling_size(), view.width())?,
    view.height(),
  ))
}

/// Size giving `column` a width of `width`; the inverse of `column_rect`.
///
/// A percentage is of the view's width, as for `default_column_width`.
pub fn column_size<T>(
  column: &T,
  workspace: &Workspace,
  width: &LengthValue,
) -> anyhow::Result<f32>
where
  T: TilingSizeGetters,
{
  if width.unit == LengthUnit::Percentage {
    return Ok(width.amount);
  }

  let (gap, _) = column.inner_gaps()?;
  let total = view_rect(workspace)?.width() + gap;

  #[allow(clippy::cast_precision_loss)]
  Ok((width.amount + gap as f32) / total as f32)
}

/// Size of `column` as it joins `workspace`'s strip:
/// `default_column_width`, or with `"window"` the width its windows have
/// now (for a just-opened window, the size its app gave it).
pub fn default_column_size(
  column: &TilingContainer,
  workspace: &Workspace,
) -> f32 {
  let width = match workspace.scrolling_config().default_column_width {
    ColumnWidth::Length(width) => width,
    ColumnWidth::Window => match windows_width(column) {
      Some(width) => LengthValue::from_px(width),
      // Only an empty column has no windows, and it isn't kept.
      None => return 0.5,
    },
  };

  // A detached workspace has no width to measure pixels against.
  column_size(column, workspace, &width)
    .map_or(1., |size| size.clamp(MIN_COLUMN_SIZE, 1.))
}

/// Width of the screen area `column`'s windows cover now.
fn windows_width(column: &TilingContainer) -> Option<i32> {
  column
    .self_and_descendants()
    .filter_map(|container| container.as_window_container().ok())
    .map(|window| window.native_properties().frame)
    .reduce(|covered, frame| covered.union(&frame))
    .map(|covered| covered.width())
}

/// A new tiling window becomes a column right of the focused one.
pub(super) fn insertion_target(workspace: &Workspace) -> Insertion {
  let index = active_column(workspace)
    .map_or(workspace.child_count(), |column| column.index() + 1);

  Insertion::At {
    parent: workspace.clone().into(),
    index,
  }
}

/// The column holding the workspace's most recently focused window, i.e.
/// the column the view follows.
///
/// A focused window off the strip (floating or fullscreen) leaves the
/// last focused column active.
pub fn active_column(workspace: &Workspace) -> Option<TilingContainer> {
  workspace
    .child_focus_order()
    .find_map(|child| child.as_tiling_container().ok())
}

/// A scrolling workspace's columns laid end to end from the first one's
/// left edge, however far the strip is scrolled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strip {
  pub ids: Vec<Uuid>,

  /// Left edge of each column, then where one more column would start.
  starts: Vec<i32>,

  pub widths: Vec<i32>,

  /// Width of the view the strip is fitted into (see `view_rect`).
  pub view_width: i32,
}

impl Strip {
  pub fn new(workspace: &Workspace) -> anyhow::Result<Self> {
    let view_width = view_rect(workspace)?.width();
    let mut strip = Self {
      ids: Vec::new(),
      starts: vec![0],
      widths: Vec::new(),
      view_width,
    };

    for column in workspace.tiling_children() {
      let (gap, _) = column.inner_gaps()?;
      let width =
        width_on_strip(&column, column.tiling_size(), view_width)?;
      strip.push(column.id(), width, gap);
    }

    Ok(strip)
  }

  /// Adds a column of `width` at the end, `gap` from the next one.
  fn push(&mut self, id: Uuid, width: i32, gap: i32) {
    let start = self.starts.last().copied().unwrap_or(0);
    self.ids.push(id);
    self.widths.push(width);
    self.starts.push(start + width + gap);
  }

  pub fn len(&self) -> usize {
    self.ids.len()
  }

  pub fn is_empty(&self) -> bool {
    self.ids.is_empty()
  }

  /// Left edge of column `index`, or past the last one, where a column
  /// after it would start.
  pub fn start(&self, index: usize) -> i32 {
    self.starts[index.min(self.len())]
  }

  pub fn index_of(&self, id: Uuid) -> Option<usize> {
    self.ids.iter().position(|column| *column == id)
  }

  /// The columns fully on screen with the view at `offset`, left to
  /// right, if any.
  pub fn fully_visible(
    &self,
    offset: i32,
  ) -> Option<RangeInclusive<usize>> {
    let mut visible = (0..self.len()).filter(|index| {
      let left = self.start(*index) - offset;
      left >= 0 && left + self.widths[*index] <= self.view_width
    });

    let first = visible.next()?;
    Some(first..=visible.next_back().unwrap_or(first))
  }

  /// Offset that shows column `index` whole, moving the view at `offset`
  /// as little as possible: not at all if it already shows it.
  ///
  /// A column as wide as the workspace is aligned to its left edge.
  pub fn fit(&self, index: usize, offset: i32) -> i32 {
    let (start, width) = (self.start(index), self.widths[index]);
    let left = start - offset;

    if width >= self.view_width || left < 0 {
      start
    } else if left + width > self.view_width {
      start + width - self.view_width
    } else {
      offset
    }
  }

  /// Offset that puts column `index` in the middle of the workspace.
  pub fn center(&self, index: usize, offset: i32) -> i32 {
    let width = self.widths[index];

    if width >= self.view_width {
      return self.fit(index, offset);
    }

    self.start(index) - (self.view_width - width) / 2
  }

  /// Offset that brings column `index` into view from a view at `offset`,
  /// coming from column `previous`, the way `config` wants.
  pub fn offset_for(
    &self,
    index: usize,
    offset: i32,
    previous: Option<usize>,
    config: &ScrollingLayoutConfig,
  ) -> i32 {
    if config.keeps_centered(self.len()) {
      return self.center(index, offset);
    }

    match (config.center_focused_column, previous) {
      (CenterFocusedColumn::OnOverflow, Some(previous))
        if previous != index =>
      {
        // The neighbour on the side it came from stands in for where it
        // came from, so a far jump centers like a step would.
        let source = if previous > index {
          (index + 1).min(self.len() - 1)
        } else {
          index.saturating_sub(1)
        };

        let (left, right) = (source.min(index), source.max(index));
        let span =
          self.start(right) + self.widths[right] - self.start(left);

        if span <= self.view_width {
          self.fit(index, offset)
        } else {
          self.center(index, offset)
        }
      }
      _ => self.fit(index, offset),
    }
  }
}

/// Where a scrolling workspace's view is, kept between syncs so that it
/// moves along with its columns as they open, close and resize.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StripView {
  /// How far the strip is scrolled: its first column starts this far left
  /// of the view's left edge.
  pub offset: i32,

  /// The active column as of the last sync.
  anchor: Option<Anchor>,

  /// Columns on the strip as of the last sync.
  known: Vec<Uuid>,

  /// Column to go back to, where it was, should the active column close
  /// before focus moves: the column it was opened from.
  return_to: Option<Placed>,

  /// What a command asked of the next sync.
  request: Option<ViewRequest>,

  /// Preset widths columns were last set to, by index, with the size each
  /// got, to cycle on from.
  preset_widths: Vec<Preset>,

  /// Preset heights windows were last set to in their columns.
  preset_heights: Vec<Preset>,

  /// Sizes of maximized columns to go back to.
  maximized: Vec<(Uuid, f32)>,
}

/// A preset a container was set to, and the size that gave it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Preset {
  container: Uuid,
  index: usize,
  size: f32,
}

/// Most windows `preset_heights` keeps, dropping the oldest beyond.
const MAX_PRESET_HEIGHTS: usize = 32;

/// The active column, and where it was shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Anchor {
  column: Uuid,
  index: usize,
  /// Left edge, relative to the view's left edge.
  x: i32,
  width: i32,
}

/// A column, and where it was shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Placed {
  column: Uuid,
  /// Left edge, relative to the view's left edge.
  x: i32,
}

/// A change to the view a command asks for, applied at the next sync.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewRequest {
  /// The active column moved along the strip from `from`: the view stays
  /// where it is, then scrolls to show the column.
  KeepCamera { from: usize },

  /// Put the active column in the middle, once.
  CenterActive,

  /// Show the active column with its left edge `x` from the view's left
  /// edge, or as close to that as shows it whole.
  PlaceActive { x: i32 },

  /// Show the active column with its right edge `right` from the view's
  /// left edge, or as close to that as shows it whole.
  PlaceActiveRight { right: i32 },
}

impl StripView {
  /// Column the active one was opened from, if it should get focus back
  /// when the active one closes.
  pub fn return_to(&self) -> Option<Uuid> {
    self.return_to.map(|placed| placed.column)
  }

  pub fn request(&mut self, request: ViewRequest) {
    self.request = Some(request);
  }

  /// Which of `count` preset widths `column` was last set to, if it
  /// still has that size. Presets since reloaded to fewer don't count.
  pub fn preset_width(
    &self,
    column: Uuid,
    size: f32,
    count: usize,
  ) -> Option<usize> {
    find_preset(&self.preset_widths, column, size, count)
  }

  pub fn set_preset_width(
    &mut self,
    column: Uuid,
    index: usize,
    size: f32,
  ) {
    set_preset(&mut self.preset_widths, column, index, size, usize::MAX);
  }

  /// Which of `count` preset heights `row` was last set to, if it still
  /// has that size.
  pub fn preset_height(
    &self,
    row: Uuid,
    size: f32,
    count: usize,
  ) -> Option<usize> {
    find_preset(&self.preset_heights, row, size, count)
  }

  pub fn set_preset_height(&mut self, row: Uuid, index: usize, size: f32) {
    let max = MAX_PRESET_HEIGHTS;
    set_preset(&mut self.preset_heights, row, index, size, max);
  }

  /// Size `column` had before it was maximized, forgetting it.
  pub fn take_size_before_maximize(
    &mut self,
    column: Uuid,
  ) -> Option<f32> {
    let index = self.maximized.iter().position(|(id, _)| *id == column)?;
    Some(self.maximized.remove(index).1)
  }

  pub fn set_size_before_maximize(&mut self, column: Uuid, size: f32) {
    self.maximized.retain(|(id, _)| *id != column);
    self.maximized.push((column, size));
  }

  /// Moves the view for what changed on `strip` since the last update,
  /// with column `focused` holding the focused window, if any.
  ///
  /// While the active column stays active, the view keeps it where it is
  /// on screen, so columns opening, closing or resizing beside it move
  /// the others, not it. When the active column itself closes, the one
  /// taking its place takes over its position too, as far as that shows
  /// it whole. A newly activated column is scrolled into view per
  /// `config`, as is the active column when its width changes. A column
  /// `config` keeps centered is put back in the middle whatever changed.
  pub fn update(
    &mut self,
    strip: &Strip,
    focused: Option<usize>,
    config: &ScrollingLayoutConfig,
  ) {
    let request = self.request.take();

    if strip.is_empty() {
      *self = Self::default();
      return;
    }

    self
      .preset_widths
      .retain(|preset| strip.ids.contains(&preset.container));
    self.maximized.retain(|(id, _)| strip.ids.contains(id));

    let anchor_index = self.anchor.and_then(|a| strip.index_of(a.column));

    // The view before anything is activated: the active column, or what
    // took its place, where it was on screen.
    let base = match (request, self.anchor) {
      (Some(ViewRequest::KeepCamera { .. }), _) | (_, None) => self.offset,
      (_, Some(anchor)) => {
        strip.start(anchor_index.unwrap_or(anchor.index)) - anchor.x
      }
    };

    let Some(active) = focused.or_else(|| {
      self
        .anchor
        .map(|a| anchor_index.unwrap_or(a.index.min(strip.len() - 1)))
    }) else {
      self.offset = base;
      self.known.clone_from(&strip.ids);
      return;
    };

    let is_activation = self
      .anchor
      .is_none_or(|anchor| anchor.column != strip.ids[active]);

    let offset = match (request, self.anchor) {
      (Some(ViewRequest::CenterActive), _) => strip.center(active, base),
      _ if config.keeps_centered(strip.len()) => {
        strip.center(active, base)
      }
      (Some(ViewRequest::PlaceActive { x }), _) => {
        strip.fit(active, strip.start(active) - x)
      }
      (Some(ViewRequest::PlaceActiveRight { right }), _) => strip
        .fit(active, strip.start(active) + strip.widths[active] - right),
      (Some(ViewRequest::KeepCamera { from }), _) => {
        strip.offset_for(active, base, Some(from), config)
      }
      // The active column closed and the one now in its place took over.
      (None, Some(anchor))
        if is_activation
          && anchor_index.is_none()
          && active == anchor.index =>
      {
        strip.fit(active, base)
      }
      // The active column closed before focus moved on from it, so the
      // column it was opened from is back where it was.
      (None, Some(_))
        if is_activation
          && anchor_index.is_none()
          && self
            .return_to
            .is_some_and(|placed| placed.column == strip.ids[active]) =>
      {
        let x = self.return_to.map_or(0, |placed| placed.x);
        strip.offset_for(active, strip.start(active) - x, None, config)
      }
      (None, _) if is_activation => {
        strip.offset_for(active, base, anchor_index, config)
      }
      (None, Some(anchor)) if anchor.width != strip.widths[active] => {
        strip.offset_for(active, base, None, config)
      }
      (None, _) => base,
    };

    self.return_to = if request.is_some() {
      None
    } else if is_activation {
      // Opened right of the column that was active, which it returns to.
      self
        .anchor
        .zip(anchor_index)
        .filter(|(_, index)| index + 1 == active)
        .filter(|_| !self.known.contains(&strip.ids[active]))
        .map(|(anchor, index)| Placed {
          column: anchor.column,
          x: strip.start(index) - base,
        })
    } else {
      self.return_to.filter(|placed| {
        active > 0 && strip.ids[active - 1] == placed.column
      })
    };

    self.anchor = Some(Anchor {
      column: strip.ids[active],
      index: active,
      x: strip.start(active) - offset,
      width: strip.widths[active],
    });
    self.offset = offset;
    self.known.clone_from(&strip.ids);
  }
}

/// The preset among `presets` that `container` was set to, if it still
/// has the size it got and is one of `count`.
fn find_preset(
  presets: &[Preset],
  container: Uuid,
  size: f32,
  count: usize,
) -> Option<usize> {
  presets
    .iter()
    .find(|preset| {
      preset.container == container && (preset.size - size).abs() < 1e-6
    })
    .map(|preset| preset.index)
    .filter(|index| *index < count)
}

/// Records that `container` was set to preset `index`, giving it `size`,
/// keeping at most `max` records.
fn set_preset(
  presets: &mut Vec<Preset>,
  container: Uuid,
  index: usize,
  size: f32,
  max: usize,
) {
  presets.retain(|preset| preset.container != container);
  presets.push(Preset {
    container,
    index,
    size,
  });

  if presets.len() > max {
    presets.remove(0);
  }
}

/// Index on `strip` of the column holding the focused window of
/// `workspace`, unless a window off the strip has focus.
fn focused_index(workspace: &Workspace, strip: &Strip) -> Option<usize> {
  workspace
    .child_focus_order()
    .next()
    .and_then(|child| child.as_tiling_container().ok())
    .and_then(|column| strip.index_of(column.id()))
}

/// Moves `workspace`'s view for what changed since the last sync (see
/// `StripView::update`).
///
/// Returns whether the strip scrolled.
pub fn sync_view(workspace: &Workspace) -> anyhow::Result<bool> {
  let strip = Strip::new(workspace)?;
  let mut view = workspace.strip_view();
  let previous = view.offset;

  view.update(
    &strip,
    focused_index(workspace, &strip),
    &workspace.scrolling_config(),
  );

  let is_scrolled = view.offset != previous;
  workspace.set_strip_view(view);

  Ok(is_scrolled)
}

/// Whether the cursor resting on `window` may focus it: unless focusing
/// it would scroll its scrolling layout's view further than
/// `focus_follows_cursor_max_scroll` allows.
pub fn is_focusable_by_hover(
  window: &WindowContainer,
) -> anyhow::Result<bool> {
  let Some((column, workspace)) = column_of(&window.as_container()) else {
    return Ok(true);
  };

  let config = workspace.scrolling_config();
  let Some(max_scroll) = &config.focus_follows_cursor_max_scroll else {
    return Ok(true);
  };

  let strip = Strip::new(&workspace)?;
  let Some(index) = strip.index_of(column.id()) else {
    return Ok(true);
  };

  // The column the view follows, also while a window off the strip has
  // focus. Focus moving within it leaves the view where it is.
  let active = active_column(&workspace)
    .and_then(|active| strip.index_of(active.id()));
  if active == Some(index) {
    return Ok(true);
  }

  let offset = workspace.scroll_offset();
  let scroll =
    (strip.offset_for(index, offset, active, &config) - offset).abs();

  Ok(scroll <= max_scroll.to_px(workspace.to_rect()?.width(), None))
}

/// Keeps the right edge of `container`'s column where it is on screen
/// through the next sync, e.g. as its left edge is dragged.
///
/// Does nothing unless it is the active column, which the view follows.
pub fn keep_right_edge(container: &Container) -> anyhow::Result<()> {
  let Some((column, workspace)) = column_of(container) else {
    return Ok(());
  };

  if active_column(&workspace)
    .is_some_and(|active| active.id() == column.id())
  {
    let rect = column.to_rect()?;
    workspace.request_view(ViewRequest::PlaceActiveRight {
      right: rect.x() + rect.width() - view_rect(&workspace)?.x(),
    });
  }

  Ok(())
}

/// The window to focus when the focused tiling `window` leaves its
/// scrolling workspace, or `None` when there is nothing to pick on the
/// strip.
///
/// Within a column, the window after it takes over, or the one before it
/// if it was the last. When it leaves its column empty, focus goes back
/// to the column it was opened from if focus hasn't moved since, else to
/// the column right of it, or, past the strip's end, the one left of it.
pub fn focus_target_after_removal(
  window: &WindowContainer,
) -> Option<Container> {
  let WindowContainer::TilingWindow(window) = window else {
    return None;
  };
  let window: Container = window.clone().into();
  let (column, workspace) = column_of(&window)?;

  // Its place within the column: among its stack's tabs or the column's
  // rows, whichever has others.
  let row = window.self_and_ancestors().find(|container| {
    container.id() != column.id()
      && container.tiling_siblings().next().is_some()
  });

  if let Some(row) = row.filter(|row| row.id() != column.id()) {
    let siblings = row
      .parent()?
      .tiling_children()
      .collect::<Vec<TilingContainer>>();
    let index = siblings
      .iter()
      .position(|sibling| sibling.id() == row.id())?;
    let next = siblings
      .get(index + 1)
      .or_else(|| index.checked_sub(1).and_then(|i| siblings.get(i)))?;

    return Some(focused_window_in(next));
  }

  column_focus_target(&column, &workspace)
}

/// The window to focus when the focused `column` leaves `workspace`'s
/// strip: in the column it was opened from if focus hasn't moved since,
/// else in the column right of it, or past the strip's end the one left
/// of it.
pub fn column_focus_target(
  column: &TilingContainer,
  workspace: &Workspace,
) -> Option<Container> {
  let columns = workspace.tiling_children().collect::<Vec<_>>();
  let index =
    columns.iter().position(|other| other.id() == column.id())?;
  let return_to = workspace.strip_view().return_to();

  let target = index
    .checked_sub(1)
    .filter(|previous| return_to == Some(columns[*previous].id()))
    .or_else(|| (index + 1 < columns.len()).then_some(index + 1))
    .or_else(|| index.checked_sub(1))?;

  Some(focused_window_in(&columns[target]))
}

/// The window in `container` that last had focus, or `container` itself.
pub fn focused_window_in(container: &TilingContainer) -> Container {
  container
    .descendant_focus_order()
    .find(Container::is_tiling_window)
    .unwrap_or_else(|| container.clone().into())
}

/// Whether `container`'s column lies within its workspace horizontally.
///
/// Containers outside a scrolling workspace are always in view.
pub fn is_in_view(container: &Container) -> bool {
  column_rects(container).is_none_or(|(rect, workspace_rect, _)| {
    rect.x() >= workspace_rect.x()
      && rect.x() + rect.width()
        <= workspace_rect.x() + workspace_rect.width()
  })
}

/// Whether `container`'s column is shown by its real window.
///
/// That is unless the part of it past the workspace would reach onto
/// another monitor. Past an edge with no monitor behind it, the window
/// simply runs off screen, even entirely, so it stays live and is drawn
/// like any other window when it scrolls back into view.
///
/// Containers outside a scrolling workspace are always shown.
pub fn shows_real_window(container: &Container) -> bool {
  is_in_view(container)
    || column_rects(container).is_none_or(|(rect, _, workspace)| {
      workspace.monitor().is_none_or(|monitor| {
        !overflows_onto_other_monitor(&rect, &monitor)
      })
    })
}

/// Where the real window of `container` goes when the layout puts it at
/// `rect`: there, unless a peek shows its column (see
/// `shows_real_window`).
///
/// The cloaked window is then kept on its own monitor at the same size,
/// since the OS gives a window the DPI of the monitor holding most of it:
/// on a monitor of another DPI, the app would rescale it, which the peek
/// shows, and again once it scrolls back.
pub fn real_window_rect(container: &Container, rect: &Rect) -> Rect {
  if shows_real_window(container) {
    return rect.clone();
  }

  container.monitor().map_or_else(
    || rect.clone(),
    |monitor| {
      let area = monitor.native_properties().working_area;
      let x = rect.x().min(area.right - rect.width()).max(area.left);
      rect.translate_to_coordinates(x, rect.y())
    },
  )
}

/// Rect of the column holding `container`, its workspace's rect, and the
/// workspace, when `container` is in a scrolling layout.
fn column_rects(container: &Container) -> Option<(Rect, Rect, Workspace)> {
  let (column, workspace) = column_of(container)?;

  Some((column.to_rect().ok()?, workspace.to_rect().ok()?, workspace))
}

/// Whether the parts of `rect` left or right of `monitor` cover any other
/// monitor.
fn overflows_onto_other_monitor(rect: &Rect, monitor: &Monitor) -> bool {
  let bounds = monitor.native_properties().bounds;

  let overflows = [
    Rect::from_ltrb(rect.left, rect.top, bounds.left, rect.bottom),
    Rect::from_ltrb(bounds.right, rect.top, rect.right, rect.bottom),
  ];

  let others = monitor
    .parent()
    .map_or_else(Default::default, |root| root.children())
    .into_iter()
    .filter_map(|child| child.as_monitor().cloned())
    .filter(|other| other.id() != monitor.id());

  others
    .map(|other| other.native_properties().bounds)
    .any(|other| {
      overflows
        .iter()
        .any(|overflow| overflow.intersection_area(&other) > 0)
    })
}

/// Queues every scrolling workspace whose view has to move for a redraw.
pub fn sync_scroll_offsets(state: &mut WmState) -> anyhow::Result<()> {
  for workspace in state.workspaces() {
    if workspace.layout() == Layout::Scrolling && sync_view(&workspace)? {
      state.pending_sync.queue_container_to_redraw(workspace);
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use uuid::Uuid;
  use wm_common::{
    CenterFocusedColumn, ColumnWidth, Layout, ScrollingLayoutConfig,
    StrutsConfig, TilingDirection, WindowState,
  };
  use wm_platform::{LengthUnit, LengthValue, Rect};

  use super::{
    focus_target_after_removal, is_focusable_by_hover, is_in_view,
    min_width, real_window_rect, shows_real_window, sync_view, view_rect,
    Strip, StripView, ViewRequest,
  };
  use crate::{
    commands::container::{
      attach_container, detach_container, flatten_child_split_containers,
      resize_tiling_container, set_focused_descendant,
    },
    layout::{attach, insertion_target},
    models::{
      Monitor, NonTilingWindow, SplitContainer, TilingWindow, Workspace,
    },
    test_utils::{mock_config as config, MOCK_MONITOR_WIDTH},
    traits::{
      CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
    },
    wm_state::WmState,
  };

  /// A scrolling workspace with three half-width columns, the first one
  /// focused.
  fn setup() -> (WmState, Workspace, [TilingWindow; 3]) {
    let windows = [
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    ];
    let workspace = Workspace::mock().layout(Layout::Scrolling).call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);

    for window in &windows {
      let insertion =
        insertion_target(&WindowState::Tiling, false, &state).unwrap();
      attach(&window.clone().into(), &insertion, &config()).unwrap();
      set_focused_descendant(&window.clone().into(), None);
    }
    set_focused_descendant(&windows[0].clone().into(), None);
    sync_view(&workspace).unwrap();

    (state, workspace, windows)
  }

  #[test]
  fn columns_continue_past_the_workspace() {
    let (_state, workspace, [first, second, third]) = setup();
    let workspace_rect = workspace.to_rect().unwrap();

    assert!((first.tiling_size() - 0.5).abs() < 1e-4);
    assert!((third.tiling_size() - 0.5).abs() < 1e-4);

    let (first_rect, second_rect, third_rect) = (
      first.to_rect().unwrap(),
      second.to_rect().unwrap(),
      third.to_rect().unwrap(),
    );
    assert_eq!(first_rect.x(), workspace_rect.x());
    assert_eq!(
      second_rect.x() + second_rect.width(),
      workspace_rect.x() + workspace_rect.width()
    );
    assert!(third_rect.x() >= workspace_rect.x() + workspace_rect.width());

    assert!(is_in_view(&first.clone().into()));
    assert!(!is_in_view(&third.clone().into()));
    // No monitor lies past the edge, so it stays a live window off screen.
    assert!(shows_real_window(&third.clone().into()));
  }

  #[test]
  fn hovering_focuses_only_columns_within_the_max_scroll() {
    let (_state, workspace, [_, second, third]) = setup();
    let is_focusable = |window: &TilingWindow| {
      is_focusable_by_hover(&window.clone().into()).unwrap()
    };
    let limit_to = |amount| {
      workspace.set_scrolling_config(ScrollingLayoutConfig {
        focus_follows_cursor_max_scroll: Some(LengthValue {
          amount,
          unit: LengthUnit::Percentage,
        }),
        ..workspace.scrolling_config()
      });
    };

    assert!(is_focusable(&third), "no limit by default");

    limit_to(0.);
    assert!(is_focusable(&second), "already on screen");
    assert!(!is_focusable(&third), "half a workspace away");

    limit_to(0.5);
    assert!(is_focusable(&third));
  }

  #[test]
  fn hovering_keeps_to_the_max_scroll_while_a_floating_window_has_focus() {
    let (_state, workspace, [_, second, third]) = setup();
    workspace.set_scrolling_config(ScrollingLayoutConfig {
      focus_follows_cursor_max_scroll: Some(LengthValue {
        amount: 0.,
        unit: LengthUnit::Percentage,
      }),
      ..workspace.scrolling_config()
    });

    let floating = NonTilingWindow::mock().call();
    attach_container(
      &floating.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();
    set_focused_descendant(&floating.into(), None);

    assert!(is_focusable_by_hover(&second.clone().into()).unwrap());
    assert!(!is_focusable_by_hover(&third.clone().into()).unwrap());
  }

  #[test]
  fn a_column_is_never_narrower_than_its_app_allows() {
    let (_state, workspace, [first, second, _]) = setup();
    let delta = first.total_border_delta().unwrap();
    let borders = delta.left.to_px(0, None) + delta.right.to_px(0, None);

    first.update_native_properties(|properties| {
      properties.min_width = 1000 + borders;
    });
    sync_view(&workspace).unwrap();

    let (first_rect, second_rect) =
      (first.to_rect().unwrap(), second.to_rect().unwrap());
    assert_eq!(first_rect.width(), 1000);
    assert!(second_rect.x() >= first_rect.x() + first_rect.width());
    assert!((first.tiling_size() - 0.5).abs() < 1e-4, "keeps its size");

    // At most as wide as the workspace.
    first.update_native_properties(|properties| {
      properties.min_width = 5000;
    });
    assert_eq!(
      first.to_rect().unwrap().width(),
      workspace.to_rect().unwrap().width()
    );

    // Outer gaps wider than the monitor leave no width at all.
    assert_eq!(min_width(&first, -100), 0);
  }

  #[test]
  fn new_windows_open_right_of_the_focused_column() {
    let (state, workspace, [first, second, _]) = setup();

    let window = TilingWindow::mock().call();
    let insertion =
      insertion_target(&WindowState::Tiling, false, &state).unwrap();
    attach(&window.clone().into(), &insertion, &config()).unwrap();

    assert_eq!(window.parent().unwrap().id(), workspace.id());
    assert_eq!(window.index(), first.index() + 1);
    assert_eq!(second.index(), window.index() + 1);
  }

  #[test]
  fn scrolls_just_enough_to_show_the_focused_column() {
    let (_state, workspace, [first, second, third]) = setup();
    let workspace_rect = workspace.to_rect().unwrap();

    assert!(!sync_view(&workspace).unwrap());
    assert_eq!(workspace.scroll_offset(), 0);

    set_focused_descendant(&third.clone().into(), None);
    assert!(sync_view(&workspace).unwrap());

    let third_rect = third.to_rect().unwrap();
    assert_eq!(
      third_rect.x() + third_rect.width(),
      workspace_rect.x() + workspace_rect.width()
    );
    assert!(!is_in_view(&first.clone().into()));
    assert!(is_in_view(&second.clone().into()));

    // Back to the first column, which is scrolled to the left edge.
    set_focused_descendant(&first.clone().into(), None);
    assert!(sync_view(&workspace).unwrap());
    assert_eq!(workspace.scroll_offset(), 0);
  }

  /// Gives `workspace` struts of `left` and `right`.
  fn set_struts(
    workspace: &Workspace,
    left: LengthValue,
    right: LengthValue,
  ) {
    workspace.set_scrolling_config(ScrollingLayoutConfig {
      struts: StrutsConfig { left, right },
      ..workspace.scrolling_config()
    });
  }

  #[test]
  fn struts_leave_the_neighbouring_columns_peeking_out() {
    let (_state, workspace, [first, second, third]) = setup();
    set_struts(
      &workspace,
      LengthValue::from_px(100),
      LengthValue::from_px(100),
    );
    sync_view(&workspace).unwrap();
    let workspace_rect = workspace.to_rect().unwrap();
    let (left, right) = (workspace_rect.left, workspace_rect.right);

    // Like niri's, the first column sits a strut in from the edge.
    assert_eq!(first.to_rect().unwrap().x(), left + 100);
    let third_rect = third.to_rect().unwrap();
    assert!(third_rect.x() < right && third_rect.x() >= right - 100);

    set_focused_descendant(&third.clone().into(), None);
    sync_view(&workspace).unwrap();

    let third_rect = third.to_rect().unwrap();
    assert_eq!(third_rect.x() + third_rect.width(), right - 100);
    assert!(is_in_view(&second.clone().into()));
    let first_rect = first.to_rect().unwrap();
    let first_right = first_rect.x() + first_rect.width();
    assert!(first_right > left && first_right <= left + 100);
  }

  #[test]
  fn columns_are_sized_against_the_room_between_the_struts() {
    let (_state, workspace, [first, ..]) = setup();
    set_struts(
      &workspace,
      LengthValue {
        amount: 0.05,
        unit: LengthUnit::Percentage,
      },
      LengthValue::from_px(0),
    );

    let workspace_width = workspace.to_rect().unwrap().width();
    let view = view_rect(&workspace).unwrap();
    assert_eq!(view.width(), workspace_width - workspace_width / 20);

    resize_tiling_container(&first.clone().into(), 1.);
    assert_eq!(first.to_rect().unwrap().width(), view.width());

    // Struts wider than the workspace leave no room rather than less.
    set_struts(
      &workspace,
      LengthValue::from_px(5000),
      LengthValue::from_px(5000),
    );
    assert_eq!(view_rect(&workspace).unwrap().width(), 0);
  }

  #[test]
  fn closing_and_resizing_columns_leave_the_others_alone() {
    let (_state, _workspace, [first, second, third]) = setup();

    resize_tiling_container(&first.clone().into(), 0.8);
    assert!((first.tiling_size() - 0.8).abs() < 1e-4);
    assert!((second.tiling_size() - 0.5).abs() < 1e-4);

    resize_tiling_container(&first.clone().into(), 5.);
    assert!((first.tiling_size() - 1.).abs() < 1e-4);

    detach_container(second.clone().into()).unwrap();
    assert!((first.tiling_size() - 1.).abs() < 1e-4);
    assert!((third.tiling_size() - 0.5).abs() < 1e-4);
  }

  #[test]
  fn the_only_column_stays_a_column() {
    let (top, bottom) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![top.clone().into(), bottom.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.clone().into()])
      .call();
    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    flatten_child_split_containers(&workspace.clone().into()).unwrap();

    assert_eq!(top.parent().unwrap().id(), column.id());
    assert_eq!(top.to_rect().unwrap().x(), bottom.to_rect().unwrap().x());
  }

  /// A scrolling workspace whose second column runs past the right edge
  /// of its monitor, optionally with another monitor to the right.
  fn edge_setup(has_right_neighbor: bool) -> (WmState, TilingWindow) {
    let (first, second) =
      (TilingWindow::mock().call(), TilingWindow::mock().call());
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![first.clone().into(), second.clone().into()])
      .call();
    second.set_tiling_size(0.7);

    let mut monitors =
      vec![Monitor::mock().workspaces(vec![workspace]).call()];
    if has_right_neighbor {
      let bounds = Rect::from_xy(MOCK_MONITOR_WIDTH, 0, 1000, 1000);
      monitors.push(
        Monitor::mock()
          .bounds(bounds.clone())
          .working_area(bounds)
          .call(),
      );
    }

    (WmState::mock(monitors), second)
  }

  #[test]
  fn a_column_past_a_free_edge_keeps_its_real_window() {
    let (_state, second) = edge_setup(false);
    assert!(shows_real_window(&second.into()));
  }

  #[test]
  fn a_column_reaching_onto_another_monitor_is_hidden() {
    let (_state, second) = edge_setup(true);
    assert!(!shows_real_window(&second.into()));
  }

  #[test]
  fn a_hidden_column_keeps_its_window_on_its_own_monitor() {
    let (_state, second) = edge_setup(true);
    let rect = second.to_rect().unwrap();
    let area = second.monitor().unwrap().native_properties().working_area;

    let parked = real_window_rect(&second.into(), &rect);

    assert!(rect.x() + rect.width() > area.right);
    assert_eq!(parked.x() + parked.width(), area.right);
    assert_eq!(
      (parked.y(), parked.width(), parked.height()),
      (rect.y(), rect.width(), rect.height())
    );
  }

  #[test]
  fn a_shown_column_keeps_its_window_where_it_is() {
    let (_state, second) = edge_setup(false);
    let rect = second.to_rect().unwrap();

    assert_eq!(real_window_rect(&second.into(), &rect), rect);
  }

  #[test]
  fn center_keeps_the_focused_column_in_the_middle() {
    let (_state, workspace, [first, second, _]) = setup();
    workspace.set_scrolling_config(ScrollingLayoutConfig {
      center_focused_column: CenterFocusedColumn::Always,
      ..ScrollingLayoutConfig::default()
    });
    let center = workspace.to_rect().unwrap().center_point().x;

    set_focused_descendant(&second.clone().into(), None);
    sync_view(&workspace).unwrap();
    assert_eq!(second.to_rect().unwrap().center_point().x, center);

    // Also at the strip's start, leaving room before it.
    set_focused_descendant(&first.clone().into(), None);
    sync_view(&workspace).unwrap();
    assert_eq!(first.to_rect().unwrap().center_point().x, center);
    assert!(workspace.scroll_offset() < 0);
  }

  #[test]
  fn closing_the_focused_column_focuses_the_one_right_of_it() {
    let (_state, workspace, [first, second, third]) = setup();
    set_focused_descendant(&second.clone().into(), None);
    sync_view(&workspace).unwrap();

    let target = focus_target_after_removal(&second.clone().into());
    assert_eq!(target.map(|t| t.id()), Some(third.id()));

    // Past the end, the one left of it.
    set_focused_descendant(&third.clone().into(), None);
    sync_view(&workspace).unwrap();
    let target = focus_target_after_removal(&third.clone().into());
    assert_eq!(target.map(|t| t.id()), Some(second.id()));
    let _ = first;
  }

  #[test]
  fn closing_a_just_opened_column_returns_to_where_it_was_opened() {
    let (state, workspace, [first, _, _]) = setup();

    let window = TilingWindow::mock().call();
    let insertion =
      insertion_target(&WindowState::Tiling, false, &state).unwrap();
    attach(&window.clone().into(), &insertion, &config()).unwrap();
    set_focused_descendant(&window.clone().into(), None);
    sync_view(&workspace).unwrap();

    let target = focus_target_after_removal(&window.clone().into());
    assert_eq!(target.map(|t| t.id()), Some(first.id()));
  }

  #[test]
  fn a_window_in_a_column_hands_focus_to_the_one_below_it() {
    let (top, middle, bottom) = (
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
      TilingWindow::mock().call(),
    );
    let column = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![
        top.clone().into(),
        middle.clone().into(),
        bottom.clone().into(),
      ])
      .call();
    let workspace = Workspace::mock()
      .layout(Layout::Scrolling)
      .tiling_containers(vec![column.into()])
      .call();
    let _monitor = Monitor::mock().workspaces(vec![workspace]).call();

    let below = focus_target_after_removal(&middle.clone().into());
    assert_eq!(below.map(|t| t.id()), Some(bottom.id()));

    let above = focus_target_after_removal(&bottom.clone().into());
    assert_eq!(above.map(|t| t.id()), Some(middle.id()));
  }

  /// Opens a window `width` wide as a new column of a scrolling workspace
  /// set to `default_column_width: "window"`.
  fn open_at_window_width(width: i32) -> (Workspace, TilingWindow) {
    let workspace = Workspace::mock().layout(Layout::Scrolling).call();
    workspace.set_scrolling_config(ScrollingLayoutConfig {
      default_column_width: ColumnWidth::Window,
      ..ScrollingLayoutConfig::default()
    });
    let state = WmState::mock(vec![Monitor::mock()
      .workspaces(vec![workspace.clone()])
      .call()]);

    let window = TilingWindow::mock()
      .floating_placement(Rect::from_xy(0, 0, width, 400))
      .call();
    let insertion =
      insertion_target(&WindowState::Tiling, false, &state).unwrap();
    attach(&window.clone().into(), &insertion, &config()).unwrap();

    (workspace, window)
  }

  #[test]
  fn a_window_width_column_keeps_the_size_its_app_gave_it() {
    let (_workspace, window) = open_at_window_width(600);
    assert_eq!(window.to_rect().unwrap().width(), 600);
  }

  #[test]
  fn a_window_width_column_fits_the_workspace_at_most() {
    let (workspace, window) = open_at_window_width(MOCK_MONITOR_WIDTH * 3);
    assert!((window.tiling_size() - 1.).abs() < 1e-4);
    assert_eq!(
      window.to_rect().unwrap().width(),
      workspace.to_rect().unwrap().width()
    );
  }

  /// A strip of columns of `widths`, 10px apart, seen through a view
  /// 1000px wide.
  fn strip(widths: &[i32]) -> Strip {
    let mut strip = Strip {
      ids: Vec::new(),
      starts: vec![0],
      widths: Vec::new(),
      view_width: 1000,
    };
    for width in widths {
      strip.push(Uuid::new_v4(), *width, 10);
    }
    strip
  }

  /// `strip` without column `index`.
  fn without(strip: &Strip, index: usize) -> Strip {
    let mut rest = Strip {
      starts: vec![0],
      ids: Vec::new(),
      widths: Vec::new(),
      ..strip.clone()
    };
    for i in (0..strip.len()).filter(|i| *i != index) {
      rest.push(strip.ids[i], strip.widths[i], 10);
    }
    rest
  }

  /// `strip` with a column of `width` inserted at `index`.
  fn with_inserted(strip: &Strip, index: usize, width: i32) -> Strip {
    let mut ids = strip.ids.clone();
    let mut widths = strip.widths.clone();
    ids.insert(index, Uuid::new_v4());
    widths.insert(index, width);

    let mut new = Strip {
      starts: vec![0],
      ids: Vec::new(),
      widths: Vec::new(),
      ..strip.clone()
    };
    for (id, width) in ids.into_iter().zip(widths) {
      new.push(id, width, 10);
    }
    new
  }

  fn fit() -> ScrollingLayoutConfig {
    ScrollingLayoutConfig::default()
  }

  fn with_centering(mode: CenterFocusedColumn) -> ScrollingLayoutConfig {
    ScrollingLayoutConfig {
      center_focused_column: mode,
      ..ScrollingLayoutConfig::default()
    }
  }

  /// Screen x of column `index` relative to the view's left edge.
  fn screen_x(strip: &Strip, view: &StripView, index: usize) -> i32 {
    strip.start(index) - view.offset
  }

  #[test]
  fn view_keeps_the_active_column_in_place_when_one_left_of_it_closes() {
    let strip = strip(&[495, 495, 495, 495]);
    let mut view = StripView::default();
    view.update(&strip, Some(3), &fit());
    let before = screen_x(&strip, &view, 3);

    let rest = without(&strip, 0);
    view.update(&rest, Some(2), &fit());

    assert_eq!(screen_x(&rest, &view, 2), before);
  }

  #[test]
  fn view_slides_the_next_column_into_a_closed_active_ones_place() {
    let strip = strip(&[495, 495, 495, 495]);
    let mut view = StripView::default();
    view.update(&strip, Some(2), &fit());
    let before = screen_x(&strip, &view, 2);

    // The column right of it takes over, where the closed one was.
    let rest = without(&strip, 2);
    view.update(&rest, Some(2), &fit());

    assert_eq!(screen_x(&rest, &view, 2), before);
  }

  #[test]
  fn view_leaves_room_past_the_strip_end_after_closing_the_last_column() {
    let strip = strip(&[495, 495, 495]);
    let mut view = StripView::default();
    view.update(&strip, Some(2), &fit());
    assert_eq!(view.offset, 505);

    // The new last column is already in view, so nothing moves.
    let rest = without(&strip, 2);
    let offset = view.offset;
    view.update(&rest, Some(1), &fit());
    assert_eq!(view.offset, offset);
  }

  #[test]
  fn view_returns_to_where_a_column_was_opened_from() {
    let strip = strip(&[600, 600]);
    let mut view = StripView::default();
    view.update(&strip, Some(0), &fit());
    view.update(&strip, Some(1), &fit());
    view.update(&strip, Some(0), &fit());
    let before = (view.offset, screen_x(&strip, &view, 0));

    // A window opens right of the first column, scrolling to it.
    let opened = with_inserted(&strip, 1, 600);
    view.update(&opened, Some(1), &fit());
    assert_ne!(view.offset, before.0);

    // It closes again: focus and view go back.
    let closed = without(&opened, 1);
    view.update(&closed, Some(0), &fit());
    assert_eq!(screen_x(&closed, &view, 0), before.1);
  }

  #[test]
  fn view_forgets_where_to_return_once_focus_moves() {
    let strip = strip(&[600, 600]);
    let mut view = StripView::default();
    view.update(&strip, Some(0), &fit());

    let opened = with_inserted(&strip, 1, 600);
    view.update(&opened, Some(1), &fit());
    assert!(view.return_to().is_some());

    view.update(&opened, Some(2), &fit());
    assert!(view.return_to().is_none());
  }

  #[test]
  fn on_overflow_centers_only_columns_that_dont_fit_with_the_last() {
    let strip = strip(&[400, 400, 700]);
    let config = with_centering(CenterFocusedColumn::OnOverflow);
    let mut view = StripView::default();
    view.update(&strip, Some(0), &config);

    // 400 + 10 + 400 fits: scrolled just enough (here, not at all).
    view.update(&strip, Some(1), &config);
    assert_eq!(view.offset, 0);

    // 400 + 10 + 700 doesn't: centered.
    view.update(&strip, Some(2), &config);
    assert_eq!(screen_x(&strip, &view, 2), (1000 - 700) / 2);
  }

  #[test]
  fn a_single_column_can_be_kept_centered() {
    let strip = strip(&[500]);
    let config = ScrollingLayoutConfig {
      always_center_single_column: true,
      ..ScrollingLayoutConfig::default()
    };
    let mut view = StripView::default();
    view.update(&strip, Some(0), &config);
    assert_eq!(screen_x(&strip, &view, 0), 250);

    // Two columns are fitted again.
    let two = with_inserted(&strip, 1, 500);
    view.update(&two, Some(1), &config);
    assert!(screen_x(&two, &view, 1) + 500 <= 1000);
  }

  #[test]
  fn a_column_left_alone_is_centered_without_being_focused_anew() {
    let strip = strip(&[495, 495]);
    let config = ScrollingLayoutConfig {
      always_center_single_column: true,
      ..ScrollingLayoutConfig::default()
    };
    let mut view = StripView::default();
    view.update(&strip, Some(1), &config);
    assert_eq!(screen_x(&strip, &view, 1), 505);

    // The other column's window closes in the background.
    let rest = without(&strip, 0);
    view.update(&rest, Some(0), &config);
    assert_eq!(screen_x(&rest, &view, 0), (1000 - 495) / 2);
  }

  #[test]
  fn a_centered_column_stays_centered_whatever_is_asked() {
    let strip = strip(&[300, 300, 300]);
    let config = with_centering(CenterFocusedColumn::Always);
    let mut view = StripView::default();
    view.update(&strip, Some(1), &config);
    assert_eq!(screen_x(&strip, &view, 1), 350);

    view.request(ViewRequest::PlaceActive { x: 100 });
    view.update(&strip, Some(1), &config);
    assert_eq!(screen_x(&strip, &view, 1), 350);

    // Its left edge is dragged out by 200px, asking to keep its right
    // edge where it was.
    let wide = Strip {
      ids: strip.ids.clone(),
      ..self::strip(&[300, 500, 300])
    };
    view.request(ViewRequest::PlaceActiveRight { right: 650 });
    view.update(&wide, Some(1), &config);
    assert_eq!(screen_x(&wide, &view, 1), 250);
  }

  #[test]
  fn a_wider_column_taking_a_closed_ones_place_is_shown_whole() {
    let strip = strip(&[495, 300, 700]);
    let mut view = StripView::default();
    view.update(&strip, Some(1), &fit());
    assert_eq!(view.offset, 0);

    // It slides into the closed column's place, then left as far as it
    // has to, whether focus went to it or off the strip.
    let rest = without(&strip, 1);
    for focused in [Some(1), None] {
      let mut view = view.clone();
      view.update(&rest, focused, &fit());
      assert_eq!(screen_x(&rest, &view, 1) + 700, 1000);
    }
  }

  #[test]
  fn resizing_the_active_column_brings_it_back_into_view() {
    let narrow = strip(&[300, 300]);
    let mut view = StripView::default();
    view.update(&narrow, Some(1), &fit());
    assert_eq!(view.offset, 0);

    let wide = {
      let mut wide = narrow.clone();
      wide.widths[1] = 800;
      wide.starts[2] = 310 + 800 + 10;
      wide
    };
    view.update(&wide, Some(1), &fit());
    assert_eq!(screen_x(&wide, &view, 1) + 800, 1000);
  }

  #[test]
  fn keep_camera_lets_a_moved_column_slide_then_scrolls_to_it() {
    let strip = strip(&[495, 495, 495]);
    let mut view = StripView::default();
    view.update(&strip, Some(0), &fit());

    // The first column swaps with the second: the view stays put, and the
    // column is still in view at its new place.
    let moved = Strip {
      ids: vec![strip.ids[1], strip.ids[0], strip.ids[2]],
      ..strip.clone()
    };
    view.request(ViewRequest::KeepCamera { from: 0 });
    view.update(&moved, Some(1), &fit());
    assert_eq!(view.offset, 0);
  }

  #[test]
  fn a_focused_window_off_the_strip_leaves_the_view_alone() {
    let strip = strip(&[495, 495, 495, 495]);
    let mut view = StripView::default();
    view.update(&strip, Some(3), &fit());
    let offset = view.offset;

    // The active column is dragged off the strip.
    let rest = without(&strip, 3);
    view.update(&rest, None, &fit());
    assert_eq!(view.offset, offset);
  }

  #[test]
  fn view_requests_center_and_place_the_active_column() {
    let strip = strip(&[300, 300, 300]);
    let mut view = StripView::default();
    view.update(&strip, Some(1), &fit());

    view.request(ViewRequest::CenterActive);
    view.update(&strip, Some(1), &fit());
    assert_eq!(screen_x(&strip, &view, 1), 350);

    view.request(ViewRequest::PlaceActive { x: 100 });
    view.update(&strip, Some(1), &fit());
    assert_eq!(screen_x(&strip, &view, 1), 100);

    // Not past where it would be cut off.
    view.request(ViewRequest::PlaceActive { x: 900 });
    view.update(&strip, Some(1), &fit());
    assert_eq!(screen_x(&strip, &view, 1), 700);
  }

  #[test]
  fn an_empty_strip_resets_the_view() {
    let strip = strip(&[495, 495, 495]);
    let mut view = StripView::default();
    view.update(&strip, Some(2), &fit());
    assert!(view.offset > 0);

    view.update(&self::strip(&[]), None, &fit());
    assert_eq!(view, StripView::default());
  }
}
