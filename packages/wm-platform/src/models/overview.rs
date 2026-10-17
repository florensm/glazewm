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

  /// Whether a workspace picked with the mouse is shown first, and only
  /// gone to once picked again.
  #[must_use]
  pub fn shows_before_going(self) -> bool {
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
}
