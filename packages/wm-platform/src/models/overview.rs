use crate::{Color, Rect};

/// What the window overview shows, sent by the WM as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewFrame {
  /// Working area of the monitor the overview covers.
  pub rect: Rect,

  pub scale_factor: f32,

  /// The monitor's workspaces, in order.
  pub workspaces: Vec<OverviewWorkspace>,

  /// Handle of the focused window, if any.
  pub focused_window: Option<isize>,

  pub style: OverviewStyle,
}

/// A workspace shown as a card in the overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewWorkspace {
  /// Name to address it by.
  pub name: String,

  /// Name shown on its card.
  pub label: String,

  pub is_focused: bool,

  /// Not active yet: shown as a "+" card, and picking it or dropping a
  /// window on it activates (or creates) the workspace.
  pub is_new: bool,

  /// Tiling windows in layout order, then the rest, which are drawn above
  /// them.
  pub windows: Vec<OverviewWindow>,
}

/// A window shown in the overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewWindow {
  /// Handle of the window, whose live thumbnail is shown.
  pub hwnd: isize,

  pub title: String,
  pub process_name: String,

  /// Where it sits on screen, in physical pixels.
  pub rect: Rect,

  pub is_minimized: bool,
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
}

/// How the overview lays out its cards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverviewLayoutMode {
  /// One row, the selected workspace in the middle at full size.
  #[default]
  Carousel,

  /// Every workspace at once.
  Grid,
}

/// What the user did in the overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverviewAction {
  /// Focus the window with this handle, switching to its workspace.
  FocusWindow(isize),

  /// Switch to the named workspace.
  FocusWorkspace(String),

  /// Move the window with this handle to the named workspace. The
  /// overview stays open.
  MoveWindow { hwnd: isize, workspace: String },

  /// Close the window with this handle. The overview stays open.
  CloseWindow(isize),

  /// Close the overview, giving focus back to the window that had it.
  Cancel,

  /// Another window took focus, which closed the overview.
  Deactivated,
}
