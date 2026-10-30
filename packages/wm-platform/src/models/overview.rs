use serde::{Deserialize, Serialize};

use crate::{Color, Rect, TabFrame};

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

  /// Working area of the monitor.
  pub rect: Rect,

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
  /// after the others, and reached with the `s` key.
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

/// A window shown in the overview.
#[derive(Clone, Debug, PartialEq, Eq)]
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
  /// Blur radius of the wallpaper behind the cards.
  pub backdrop_blur: f32,

  /// Tint over the blurred wallpaper.
  pub backdrop_tint: Color,

  /// Selection and highlights.
  pub accent: Color,

  /// Card background.
  pub card: Color,

  /// Hovered card background, and behind a window with no preview.
  pub surface: Color,

  /// Strip under a window's title.
  pub caption: Color,

  pub text: Color,
  pub subtext: Color,

  /// Border of windows matching a search.
  pub search: Color,

  pub font_family: String,

  /// Columns of the grid of every workspace.
  pub grid_columns: usize,

  /// Length of the zoom out when opening; 0 opens instantly.
  pub open_duration_ms: u32,

  /// Length of the zoom into the picked workspace when closing; 0 closes
  /// instantly.
  pub close_duration_ms: u32,

  /// About how long cards and windows take to move to a new selection or
  /// layout; 0 moves them at once.
  pub switch_duration_ms: u32,

  /// How long highlights take to glide to what they mark next and to
  /// fade, as the shortcut sheet does; 0 snaps them.
  pub highlight_duration_ms: u32,

  /// How far what moves overshoots where it heads before settling, from
  /// 0 (not at all) to 0.8.
  pub bounce: f32,

  pub effects: OverviewEffects,
}

/// Extra flourishes of the overview, each of which can be turned off.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewEffects {
  /// Soft glow of the accent color around the selected workspace.
  pub glow: bool,

  /// Whether the glow slowly breathes. Keeps the overview animating on
  /// every frame while it is open.
  pub pulse: bool,

  /// A ring spreading out of a card as it is clicked, or as a window
  /// lands on it.
  pub ripples: bool,

  /// Sparks bursting out of where a dropped window lands; `0` for none,
  /// at most `MAX_SPARKS`.
  pub sparks: u32,
}

impl OverviewEffects {
  /// Most sparks a landing bursts into.
  pub const MAX_SPARKS: u32 = 48;
}

impl Default for OverviewEffects {
  fn default() -> Self {
    Self {
      glow: true,
      pulse: false,
      ripples: true,
      sparks: 14,
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

  /// The shown workspace large and as laid out, the others in a column
  /// at the side.
  Stage,

  /// One row, the selected workspace in the middle and larger than the
  /// rest.
  Carousel,

  /// Every workspace at once.
  Grid,
}

impl OverviewLayoutMode {
  /// Every layout, in the order `Tab` goes through them.
  pub const ALL: [Self; 4] =
    [Self::Activities, Self::Stage, Self::Carousel, Self::Grid];

  /// The layout `Tab` goes on to.
  #[must_use]
  pub fn next(self) -> Self {
    let index = Self::ALL.iter().position(|layout| *layout == self);
    Self::ALL[index.map_or(0, |index| (index + 1) % Self::ALL.len())]
  }

  /// Whether the special workspace's card is docked at the bottom, apart
  /// from the others, rather than coming after them.
  #[must_use]
  pub fn docks_special(self) -> bool {
    matches!(self, Self::Carousel | Self::Grid)
  }

  /// Whether the selected workspace's windows are laid out on their own
  /// rather than on its card.
  #[must_use]
  pub fn has_scene(self) -> bool {
    self == Self::Activities
  }

  /// Whether the selected workspace shows large, its windows big enough
  /// to point at and to tag with their titles.
  #[must_use]
  pub fn shows_selected_large(self) -> bool {
    matches!(self, Self::Activities | Self::Stage)
  }

  /// Name of the layout, as in the config.
  #[must_use]
  pub fn name(self) -> &'static str {
    match self {
      Self::Activities => "activities",
      Self::Stage => "stage",
      Self::Carousel => "carousel",
      Self::Grid => "grid",
    }
  }
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
