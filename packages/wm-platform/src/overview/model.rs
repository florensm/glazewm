use serde::{Deserialize, Serialize};

use crate::{
  BorderOverlayParams, Color, Direction, ParticleEffects, ParticleLook,
  Point, Rect, TabFrame,
};

/// What the window overview shows, sent by the WM as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewFrame {
  /// A screen per monitor, in the order the arrows go through them.
  pub screens: Vec<OverviewScreen>,

  /// Workspaces named "1" to "10" without a card that digit keys can
  /// still go to: inactive ones from the config.
  pub digit_workspaces: Vec<String>,

  /// Handle of the focused window, if any.
  pub focused_window: Option<isize>,

  pub style: OverviewStyle,
}

/// The overview of one monitor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverviewScreen {
  /// The WM's ID of the monitor, which actions picked on it carry.
  pub id: u128,

  /// Working area of the monitor: where its workspaces lay out their
  /// windows, which the workspaces' cards show.
  pub rect: Rect,

  /// Bounds of the monitor, all of which the overview covers, the space
  /// reserved by app bars or the taskbar included.
  pub bounds: Rect,

  pub scale_factor: f32,

  /// The focused monitor, where the keyboard starts.
  pub is_focused: bool,

  /// The monitor's workspaces, in order, then the special workspace,
  /// wherever it is.
  pub workspaces: Vec<OverviewWorkspace>,
}

/// A workspace shown as a card in the overview.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverviewWorkspace {
  /// Name to address it by.
  pub name: String,

  /// Name shown on its card.
  pub label: String,

  /// Displayed on its screen's monitor: the overview zooms out of it on
  /// open, and back into it when closed without a pick there.
  pub is_displayed: bool,

  /// Not active yet: shown as a "+" card, and picking it or dropping a
  /// window on it activates (or creates) the workspace.
  pub is_new: bool,

  /// The special workspace: shown on every monitor's overview, set apart
  /// after the others, and reached with `Alt` + `-`.
  pub is_special: bool,

  /// Area its card shows, in physical pixels: the monitor's working area,
  /// widened to the whole strip of a scrolling workspace.
  pub content_rect: Rect,

  /// Tiling windows in layout order, then the rest, which are drawn above
  /// them.
  pub windows: Vec<OverviewWindow>,

  /// Tab bars of its stacks, as the stacks' own bars draw them.
  pub tab_bars: Vec<TabFrame>,
}

impl OverviewWorkspace {
  /// Its window `hwnd`, if it has it.
  #[must_use]
  pub fn window(&self, hwnd: isize) -> Option<&OverviewWindow> {
    self.windows.iter().find(|window| window.hwnd == hwnd)
  }
}

/// A window shown in the overview.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewWindow {
  /// Handle of the window, whose live thumbnail is shown.
  pub hwnd: isize,

  pub title: String,
  pub process_name: String,

  /// Where the layout puts it, in physical pixels; off screen for a
  /// column scrolled out of view.
  pub rect: Rect,

  pub is_minimized: bool,

  /// A tab of a stack other than its shown one: it has no preview of its
  /// own, and shows as a tab in its stack's tab bar.
  pub is_hidden_tab: bool,

  /// In a column scrolled out of view: shown on its card, but faded in
  /// and out by the zoom rather than moved from where it sits.
  pub is_scrolled_off: bool,

  /// Opacity its `transparency` effect gives it, from 0 to 255; DWM's
  /// thumbnails of it don't carry it.
  pub opacity: u8,

  /// Its backdrop and border, bottom to top, which show under its preview
  /// as the WM stacks them under the window.
  pub overlays: Vec<OverviewOverlay>,

  /// The border the WM gives it, if any, which the overview draws itself
  /// where `overlays` has none on screen to show, e.g. on a workspace not
  /// displayed.
  pub border: Option<BorderOverlayParams>,
}

impl OverviewWindow {
  /// Its title, or its process name when it has none.
  #[must_use]
  pub fn display_title(&self) -> String {
    if self.title.is_empty() {
      self.process_name.clone()
    } else {
      self.title.clone()
    }
  }
}

/// A window's backdrop or border, as the WM draws it around and behind
/// the window, shown with the window's preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverviewOverlay {
  /// Handle of the overlay's window, whose live thumbnail is shown.
  pub hwnd: isize,

  pub kind: OverviewOverlayKind,

  /// How far it reaches past the window's visible frame on each side
  /// (left, top, right, bottom), in physical pixels.
  pub outset: [i32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewOverlayKind {
  Backdrop,
  Border,
}

/// Look of the overview.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewStyle {
  /// What shows behind the cards.
  pub wallpaper: OverviewWallpaper,

  /// Selection and highlights.
  pub accent: Color,

  /// Card background.
  pub card: Color,

  /// Corner radius of a card and of the framed wallpaper's panel, and at
  /// most of the windows drawn on them, in logical pixels.
  pub corner_radius: f32,

  /// Hovered card background, and behind a window with no preview.
  pub surface: Color,

  /// Strip under a window's title.
  pub caption: Color,

  pub text: Color,
  pub subtext: Color,

  /// What matched, in the search box's list.
  pub search: Color,

  pub font_family: String,

  /// How it opens and closes.
  pub zoom_style: OverviewZoomStyle,

  /// Pace of the open, from 0 to 1 over its duration.
  pub open_curve: OverviewCurve,

  /// Pace of the close, from 0 to 1 over its duration.
  pub close_curve: OverviewCurve,

  /// Openness from which names and highlights start to show, from 0 to
  /// 0.95.
  pub chrome_from: f32,

  /// Length of the open; 0 opens instantly.
  pub open_duration_ms: u32,

  /// Length of the close, into the picked workspace; 0 closes instantly.
  pub close_duration_ms: u32,

  /// How the activities change the workspace they show large.
  pub switch_style: OverviewSwitchStyle,

  /// About how long cards and windows take to move to a new selection or
  /// layout; 0 moves them at once.
  pub switch_duration_ms: u32,

  /// How long highlights take to settle onto what they mark next and to
  /// fade, as the shortcut sheet does; 0 snaps them.
  pub highlight_duration_ms: u32,

  /// How far what moves overshoots where it heads before settling, from
  /// 0 (not at all) to 0.8.
  pub bounce: f32,

  pub effects: OverviewEffects,

  /// How the selected or hovered window is marked.
  pub window_highlight: OverviewWindowHighlight,

  /// How much of a scrolling workspace's strip the activities show.
  pub strip_view: OverviewStripView,

  /// How the exposé spreads the windows.
  pub expose: OverviewExpose,

  pub activities: OverviewActivities,

  /// How the stack shows the workspaces.
  pub stack: OverviewStack,

  /// Whether the activities and the exposé show their row of miniatures.
  pub workspace_row: OverviewWorkspaceRow,

  /// Whether the search field shows along the bottom; without it, typing
  /// still searches, and `F1` still shows the shortcuts.
  pub search_hint: bool,

  /// How its particle bursts look, as the WM's do.
  pub particles: ParticleLook,
}

impl OverviewStyle {
  /// The row of miniatures `layout` shows, and the edge it sits along.
  #[must_use]
  pub fn row(
    &self,
    layout: OverviewLayoutMode,
  ) -> (OverviewWorkspaceRow, OverviewRowSide) {
    match layout {
      OverviewLayoutMode::Expose => (
        self.expose.workspace_row.or(self.workspace_row),
        self.expose.workspace_row_position,
      ),
      OverviewLayoutMode::Activities => {
        (self.workspace_row, self.activities.workspace_row_position)
      }
      OverviewLayoutMode::Stack => {
        (self.workspace_row, OverviewRowSide::default())
      }
    }
  }

  /// How `layout` opens and closes, and whether it cascades: the stack's
  /// own `open_style`, unless that inherits `animation.style`.
  #[must_use]
  pub fn zoom_style_of(
    &self,
    layout: OverviewLayoutMode,
  ) -> (OverviewZoomStyle, bool) {
    if layout != OverviewLayoutMode::Stack {
      return (self.zoom_style, false);
    }
    match self.stack.open_style {
      OverviewStackOpen::Inherit => (self.zoom_style, false),
      OverviewStackOpen::Zoom => (OverviewZoomStyle::Zoom, false),
      OverviewStackOpen::Fade => (OverviewZoomStyle::Fade, false),
      OverviewStackOpen::None => (OverviewZoomStyle::None, false),
      OverviewStackOpen::Cascade => (OverviewZoomStyle::Zoom, true),
    }
  }

  /// Pace of the close if `is_closing`, else of the open.
  #[must_use]
  pub fn zoom_curve(&self, is_closing: bool) -> &OverviewCurve {
    if is_closing {
      &self.close_curve
    } else {
      &self.open_curve
    }
  }
}

/// How much of a scrolling workspace's strip the activities show large.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewStripView {
  /// What its monitor shows; columns scrolled out of view are left out.
  #[default]
  Visible,

  /// The whole strip, scaled to fit.
  Whole,
}

/// Whether the activities' and the exposé's row of miniatures stays.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewWorkspaceRow {
  /// Over the scene, which starts below it.
  Shown,

  /// A slim strip of small miniatures over the scene, as macOS's Spaces
  /// bar and Windows' Task View show their desktops, leaving the scene
  /// most of the room.
  #[default]
  Compact,

  /// Left out, the scene taking its room. Workspaces are still gone
  /// through with the keyboard and the wheel.
  Hidden,
}

/// The exposé's own row of miniatures, or the one `workspace_row` sets.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewExposeRow {
  #[default]
  Inherit,
  Compact,
  Shown,
  Hidden,
}

impl OverviewExposeRow {
  /// The row it shows, given the one `workspace_row` sets.
  #[must_use]
  pub fn or(self, shared: OverviewWorkspaceRow) -> OverviewWorkspaceRow {
    match self {
      Self::Inherit => shared,
      Self::Compact => OverviewWorkspaceRow::Compact,
      Self::Shown => OverviewWorkspaceRow::Shown,
      Self::Hidden => OverviewWorkspaceRow::Hidden,
    }
  }
}

/// Edge of the screen the row of miniatures sits along.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewRowSide {
  #[default]
  Top,
  /// Over the search line.
  Bottom,
  /// A column, the scene taking the rest of the width.
  Left,
  Right,
}

/// How the exposé arranges the shown workspace's windows.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewExposeArrange {
  /// One cluster, each window about where it sits on screen.
  #[default]
  Cluster,
  /// Even cells in reading order, each window fit to its cell.
  Grid,
  /// GNOME's rows.
  Rows,
}

/// Which of the exposé's windows show their titles.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewExposeTitles {
  /// The picked or hovered one.
  #[default]
  Hover,
  Always,
}

/// How the exposé shows a search.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewExposeSearch {
  /// The matches spread out alone.
  #[default]
  Spread,
  /// Nothing moves; the rest fade where they are.
  Dim,
  /// The matches gather in the room, the rest wait on the shelf.
  Shelf,
}

/// Whether holding `Alt` in the exposé shows a letter on each window.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewExposeHints {
  #[default]
  Off,
  Alt,
}

/// Key held in the exposé to enlarge the picked window.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewPeekKey {
  #[default]
  None,
  Ctrl,
}

/// Where the exposé shows minimized windows.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewExposeMinimized {
  /// Arranged with the others.
  #[default]
  Inline,
  /// On a shelf of small previews along the bottom of the room.
  Shelf,
}

/// How the exposé spreads the shown workspace's windows.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewExpose {
  /// Space between windows, in logical pixels.
  pub gap: f32,

  /// Most of the room's width and height the windows take, from 0.6 to
  /// 1.
  pub largest: f32,

  /// Space kept free along the screen's edges, in logical pixels, from 8
  /// to 120.
  pub margin: f32,

  /// Its row of miniatures.
  pub workspace_row: OverviewExposeRow,

  /// Edge its row of miniatures sits along.
  pub workspace_row_position: OverviewRowSide,

  pub arrange: OverviewExposeArrange,

  /// How much the cluster grows small windows relative to large ones,
  /// from 0 to 0.6.
  pub small_window_boost: f32,

  /// Smallest window scale, from 0 (off) to 0.7, under which the windows
  /// are arranged in rows instead.
  pub readable_floor: f32,

  pub titles: OverviewExposeTitles,

  /// Whether each window shows its app's icon large in its middle.
  pub app_icons: bool,

  pub minimized: OverviewExposeMinimized,

  pub search_mode: OverviewExposeSearch,

  pub hints: OverviewExposeHints,

  pub peek_key: OverviewPeekKey,

  /// Whether letting go of the modifier of the keybinding that opened it
  /// picks the selected window, once the selection moved.
  pub release_picks: bool,
}

impl OverviewExpose {
  /// `small_window_boost`, within 0 to 0.6.
  #[must_use]
  pub fn clamped_small_window_boost(&self) -> f32 {
    clamped(self.small_window_boost, (0.0, 0.6), 0.0)
  }

  /// `readable_floor`, within 0 to 0.7.
  #[must_use]
  pub fn clamped_readable_floor(&self) -> f32 {
    clamped(self.readable_floor, (0.0, 0.7), 0.0)
  }

  /// `largest`, within 0.6 to 1.
  #[must_use]
  pub fn clamped_largest(&self) -> f32 {
    clamped(self.largest, (0.6, 1.0), Self::default().largest)
  }

  /// `margin`, within 8 to 120 px.
  #[must_use]
  pub fn clamped_margin(&self) -> f32 {
    clamped(self.margin, MARGIN_RANGE, Self::default().margin)
  }
}

impl Default for OverviewExpose {
  fn default() -> Self {
    Self {
      gap: 28.0,
      largest: 0.85,
      margin: 48.0,
      workspace_row: OverviewExposeRow::default(),
      workspace_row_position: OverviewRowSide::default(),
      arrange: OverviewExposeArrange::default(),
      small_window_boost: 0.0,
      readable_floor: 0.0,
      titles: OverviewExposeTitles::default(),
      app_icons: false,
      minimized: OverviewExposeMinimized::default(),
      search_mode: OverviewExposeSearch::default(),
      hints: OverviewExposeHints::default(),
      peek_key: OverviewPeekKey::default(),
      release_picks: false,
    }
  }
}

/// How the activities frame the workspace they show large.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewActivities {
  /// Space kept free along the screen's edges, in logical pixels, from 8
  /// to 120.
  pub margin: f32,

  /// Edge its row of miniatures sits along.
  pub workspace_row_position: OverviewRowSide,
}

impl OverviewActivities {
  /// `margin`, within 8 to 120 px.
  #[must_use]
  pub fn clamped_margin(&self) -> f32 {
    clamped(self.margin, MARGIN_RANGE, Self::default().margin)
  }
}

impl Default for OverviewActivities {
  fn default() -> Self {
    Self {
      margin: 48.0,
      workspace_row_position: OverviewRowSide::default(),
    }
  }
}

/// Smallest and largest margin honored, in logical pixels.
const MARGIN_RANGE: (f32, f32) = (8.0, 120.0);

/// `value` within `range`, or `default` if it isn't a number.
fn clamped(value: f32, range: (f32, f32), default: f32) -> f32 {
  if value.is_finite() {
    value.clamp(range.0, range.1)
  } else {
    default
  }
}

/// Way the stack's workspaces run.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewStackDirection {
  /// Top to bottom.
  #[default]
  Vertical,
  /// Side by side, each as wide as its strip.
  Horizontal,
}

/// Where the stack shows each workspace's name.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewStackNames {
  /// Left of it; above it in a horizontal stack, where its neighbour is
  /// beside it.
  #[default]
  Beside,
  Above,
  Below,
  Hidden,
}

/// How the stack opens and closes, apart from the other layouts.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewStackOpen {
  /// As `animation.style` has it.
  #[default]
  Inherit,
  Zoom,
  Fade,
  None,
  /// A zoom, each workspace starting a step after the one nearer the
  /// selected.
  Cascade,
}

/// How the stack shows the workspaces.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewStack {
  /// How large a workspace shows, as a part of the screen's width.
  pub zoom: f32,

  pub direction: OverviewStackDirection,

  /// Space between workspaces, in logical pixels, from 0 to 200.
  pub gap: f32,

  pub names: OverviewStackNames,

  /// Opacity of the workspaces but the selected one, from 0.2 to 1.
  pub neighbour_opacity: f32,

  /// Their scale against the selected one, from 0.7 to 1.
  pub neighbour_scale: f32,

  pub open_style: OverviewStackOpen,
}

impl OverviewStack {
  /// Smallest and largest `zoom` honored.
  const ZOOM_RANGE: (f32, f32) = (0.2, 0.9);

  /// `zoom`, within what the stack can lay out.
  #[must_use]
  pub fn clamped_zoom(&self) -> f32 {
    clamped(self.zoom, Self::ZOOM_RANGE, Self::default().zoom)
  }

  /// `gap`, within 0 to 200 px.
  #[must_use]
  pub fn clamped_gap(&self) -> f32 {
    clamped(self.gap, (0.0, 200.0), Self::default().gap)
  }

  /// `neighbour_opacity`, within 0.2 to 1.
  #[must_use]
  pub fn clamped_neighbour_opacity(&self) -> f32 {
    clamped(self.neighbour_opacity, (0.2, 1.0), 1.0)
  }

  /// `neighbour_scale`, within 0.7 to 1.
  #[must_use]
  pub fn clamped_neighbour_scale(&self) -> f32 {
    clamped(self.neighbour_scale, (0.7, 1.0), 1.0)
  }
}

impl Default for OverviewStack {
  fn default() -> Self {
    Self {
      zoom: 0.5,
      direction: OverviewStackDirection::default(),
      gap: 28.0,
      names: OverviewStackNames::default(),
      neighbour_opacity: 1.0,
      neighbour_scale: 1.0,
      open_style: OverviewStackOpen::default(),
    }
  }
}

/// What the overview shows behind its cards.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewWallpaperStyle {
  /// The wallpaper kept sharp on a rounded panel where the shown
  /// workspace sits, over a plain color: the desktop shrunk as a whole.
  #[default]
  Framed,

  /// The wallpaper over the whole screen, blurred and tinted.
  Blurred,
}

/// The wallpaper behind the overview's cards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverviewWallpaper {
  pub style: OverviewWallpaperStyle,

  /// Around the framed panel.
  pub color: Color,

  /// Wallpaper showing around the shown workspace's windows on the
  /// framed panel, in logical pixels.
  pub padding: f32,

  /// Whether the framed panel casts a soft shadow.
  pub shadow: bool,

  /// Blur radius of the blurred wallpaper.
  pub blur: f32,

  /// Tint over the blurred wallpaper; its alpha is how dark it gets.
  pub tint: Color,
}

impl OverviewWallpaper {
  /// Itself when framed, `None` when blurred.
  #[must_use]
  pub fn framed(&self) -> Option<Self> {
    (self.style == OverviewWallpaperStyle::Framed).then_some(*self)
  }
}

/// How the overview opens and closes.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewZoomStyle {
  /// Everything zooms out of the workspace on screen as one, and back
  /// into the one picked, as hyprexpo and niri do.
  #[default]
  Zoom,

  /// It fades in over the desktop settling from a little larger, and
  /// fades back out growing.
  Fade,

  /// At once.
  None,
}

/// How the activities change the workspace they show large.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewSwitchStyle {
  /// The new one slides in from the side its miniature is on, as a
  /// workspace switch does, and the last one out the other way.
  #[default]
  Slide,

  /// One fades into the other in place.
  Fade,

  /// The new one's windows fly out of its miniature, and the last one's
  /// back into theirs.
  Miniature,
}

/// An easing curve sampled at even steps from progress 0 to 1, so the
/// overview's thread needn't know how the config describes it.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewCurve {
  samples: Vec<f32>,
  is_spring: bool,
}

impl OverviewCurve {
  /// The curve through `samples`, taken at even steps from 0 to 1; a
  /// straight line for fewer than two.
  #[must_use]
  pub fn new(samples: Vec<f32>) -> Self {
    Self {
      samples,
      is_spring: false,
    }
  }

  /// A spring's curve, sampled until it settles: a zoom on it may end
  /// once what moves is close enough (see `Zoom::trimmed`).
  #[must_use]
  pub fn spring(samples: Vec<f32>) -> Self {
    Self {
      samples,
      is_spring: true,
    }
  }

  #[must_use]
  pub fn is_spring(&self) -> bool {
    self.is_spring
  }

  /// Progress from which it stays within `epsilon` of 1 for good.
  #[must_use]
  #[allow(clippy::cast_precision_loss)]
  pub fn settled_from(&self, epsilon: f32) -> f32 {
    let steps = self.samples.len().saturating_sub(1);
    let is_off = |sample: f32| (1.0 - sample).abs() >= epsilon;
    let Some(last_off) = self.samples.iter().rposition(|&s| is_off(s))
    else {
      return 0.0;
    };
    if last_off >= steps {
      return 1.0;
    }

    // Where the line to the next sample crosses into the band.
    let (from, to) = (self.samples[last_off], self.samples[last_off + 1]);
    let edge = if from < 1.0 {
      1.0 - epsilon
    } else {
      1.0 + epsilon
    };
    let part = if (to - from).abs() > f32::EPSILON {
      ((edge - from) / (to - from)).clamp(0.0, 1.0)
    } else {
      0.0
    };
    (last_off as f32 + part) / steps as f32
  }

  /// Its value at `progress`, between the samples either side of it.
  #[must_use]
  #[allow(clippy::cast_precision_loss)]
  pub fn at(&self, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let steps = self.samples.len().saturating_sub(1);
    if steps == 0 {
      return progress;
    }

    let position = progress * steps as f32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = (position.floor() as usize).min(steps - 1);
    let (from, to) = (self.samples[index], self.samples[index + 1]);
    from + (to - from) * (position - index as f32)
  }
}

impl Default for OverviewCurve {
  /// Fast at first, gently settling: a quartic ease-out.
  fn default() -> Self {
    Self::new(
      (0..=32_u8)
        .map(|step| 1.0 - (1.0 - f32::from(step) / 32.0).powi(4))
        .collect(),
    )
  }
}

/// How the overview marks the selected or hovered window.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewWindowHighlight {
  pub style: OverviewWindowHighlightStyle,

  /// The ring's thickness, or how far the glow reaches, in logical
  /// pixels.
  pub width: f32,

  /// Space between the window and its ring or glow, in logical pixels.
  pub gap: f32,
}

impl Default for OverviewWindowHighlight {
  fn default() -> Self {
    Self {
      style: OverviewWindowHighlightStyle::default(),
      width: 2.0,
      gap: 0.0,
    }
  }
}

/// What marks the selected or hovered window in the overview.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewWindowHighlightStyle {
  /// A ring in the accent color.
  #[default]
  Ring,

  /// A soft glow in the accent color, fading out from the window's edge.
  Glow,

  /// Nothing; the title tag under it still shows which it is.
  None,
}

/// Extra flourishes of the overview, each of which can be turned off.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewEffects {
  /// Soft glow of the accent color around the selected workspace.
  pub glow: bool,

  /// Whether the glow slowly breathes. Keeps the overview animating on
  /// every frame while it is open.
  pub pulse: bool,

  /// A ring spreading out of a card as it is clicked, or as a window
  /// lands on it, and drawing in on a window closed from the overview.
  pub ripples: bool,

  /// Particles bursting out of where a dropped window lands, over the
  /// overview.
  pub drop_particles: ParticleEffects,

  /// Particles thrown off the edges of a window closed from the
  /// overview, once it is gone.
  pub close_particles: ParticleEffects,

  /// Particles thrown off the shown workspace as the overview settles
  /// open.
  pub open_particles: ParticleEffects,

  /// Particles thrown off the windows of each workspace the overview
  /// zooms into as it closes, as the zoom lands.
  pub exit_particles: ParticleEffects,

  /// Whether a card wobbles as a window lands on it.
  pub jelly: bool,

  /// Delay between one card and the next as they pop in on open, in
  /// milliseconds; `0` brings them in together.
  pub stagger_ms: u32,

  /// How far a hovered card leans toward the cursor, in logical pixels;
  /// `0` keeps it still.
  pub lean: f32,

  /// How much the cards dim while another is hovered, from `0` (not at
  /// all) to `1`.
  pub dim: f32,

  /// How much a dragged window stretches with its speed, and squashes as
  /// it stops, from `0` (not at all) to `1`.
  pub squash: f32,

  /// Fading copies of a dragged window's outline trailing it; `0` for
  /// none, at most `MAX_TRAIL`.
  pub trail: u32,

  /// How much the picked window grows in the large view of the
  /// activities and the exposé, as a fraction of its size.
  pub window_lift: f32,
}

impl OverviewEffects {
  /// Most copies trailing a dragged window.
  pub const MAX_TRAIL: u32 = 8;

  /// Every effect off.
  pub const NONE: Self = Self {
    glow: false,
    pulse: false,
    ripples: false,
    drop_particles: ParticleEffects::NONE,
    close_particles: ParticleEffects::NONE,
    open_particles: ParticleEffects::NONE,
    exit_particles: ParticleEffects::NONE,
    jelly: false,
    stagger_ms: 0,
    lean: 0.0,
    dim: 0.0,
    squash: 0.0,
    trail: 0,
    window_lift: 0.0,
  };
}

impl Default for OverviewEffects {
  fn default() -> Self {
    // Calm, as an OS's own overview: nothing but the picked window
    // growing a little. The rest is there to turn on.
    Self {
      window_lift: 0.025,
      ..Self::NONE
    }
  }
}

/// How the overview lays out the workspaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverviewLayoutMode {
  /// The shown workspace large and as laid out, under a row of every
  /// workspace in miniature.
  #[default]
  Activities,

  /// Every workspace at the same zoom, stacked top to bottom, each strip
  /// at its full length and panned to keep what is picked in view.
  Stack,

  /// As the activities, but the shown workspace's windows, each tab of
  /// its stacks included, spread out side by side at one scale.
  Expose,
}

impl OverviewLayoutMode {
  /// Every layout, in the order `Tab` goes through them.
  pub const ALL: [Self; 3] = [Self::Activities, Self::Stack, Self::Expose];

  /// The layout `Tab` goes on to.
  #[must_use]
  pub fn next(self) -> Self {
    let index = Self::ALL.iter().position(|layout| *layout == self);
    Self::ALL[index.map_or(0, |index| (index + 1) % Self::ALL.len())]
  }

  /// The layout `Shift` + `Tab` goes back to.
  #[must_use]
  pub fn previous(self) -> Self {
    let count = Self::ALL.len();
    let index = Self::ALL.iter().position(|layout| *layout == self);
    Self::ALL[index.map_or(0, |index| (index + count - 1) % count)]
  }

  /// Whether the selected workspace's windows are laid out on their own
  /// rather than on its card.
  #[must_use]
  pub fn has_scene(self) -> bool {
    matches!(self, Self::Activities | Self::Expose)
  }

  /// Whether its scene spreads the windows out rather than showing them
  /// as laid out.
  #[must_use]
  pub fn spreads_windows(self) -> bool {
    self == Self::Expose
  }

  /// Name of the layout, as in the config.
  #[must_use]
  pub fn name(self) -> &'static str {
    match self {
      Self::Activities => "activities",
      Self::Stack => "stack",
      Self::Expose => "expose",
    }
  }
}

/// What one of the WM's keybindings does in the open overview, which has
/// the keyboard: what its command would do on the desktop, done to the
/// overview's selection instead.
#[derive(Clone, Debug, PartialEq)]
pub enum OverviewInput {
  /// Move the selection, as an arrow key does.
  Move(Direction),

  /// Switch to the named workspace and close, or drop the carried window
  /// there.
  FocusWorkspace(String),

  /// Send the picked or carried window to the named workspace.
  MoveToWorkspace(String),

  /// The same for the special workspace.
  FocusSpecialWorkspace,
  MoveToSpecialWorkspace,

  /// Select the next or previous workspace, as the wheel does.
  NextWorkspace,
  PreviousWorkspace,

  /// Close the picked window.
  CloseWindow,
}

/// What the user did in the overview.
///
/// `monitor` is the ID of the screen it was picked on, which an inactive
/// workspace is activated on and the special workspace summoned to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverviewAction {
  /// Focus the window with this handle, switching to its workspace.
  FocusWindow { hwnd: isize, monitor: u128 },

  /// Switch to the named workspace.
  FocusWorkspace { name: String, monitor: u128 },

  /// Move the window with this handle to the named workspace. The
  /// overview stays open.
  MoveWindow {
    hwnd: isize,
    workspace: String,
    monitor: u128,
  },

  /// Move the window with this handle to where dragging it to `point` on
  /// the named workspace and letting go would put it, in the screen
  /// coordinates its windows sit at, displayed or not: e.g. beside the
  /// window there. A window that isn't tiling only moves there. The
  /// overview stays open.
  PlaceWindow {
    hwnd: isize,
    workspace: String,
    monitor: u128,
    point: Point,
  },

  /// Close the window with this handle. The overview stays open.
  CloseWindow(isize),

  /// Close the overview, giving focus back to the window that had it.
  Cancel,

  /// Another window took focus, which closed the overview.
  Deactivated,

  /// The overview's windows hid, once it closed (after its zoom into the
  /// pick), so what the WM held off while it was on screen resumes.
  Hidden,
}
