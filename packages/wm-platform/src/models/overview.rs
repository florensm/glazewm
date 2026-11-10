use serde::{Deserialize, Serialize};

use crate::{
  BorderOverlayParams, Color, Direction, ParticleEffects, ParticleLook,
  ParticleStyle, Point, Rect, TabFrame,
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

  /// How it opens and closes.
  pub zoom_style: OverviewZoomStyle,

  /// Pace of the open and close, from 0 to 1 over their durations.
  pub zoom_curve: OverviewCurve,

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

  /// How its particle bursts look, as the WM's do.
  pub particles: ParticleLook,
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
}

impl OverviewCurve {
  /// The curve through `samples`, taken at even steps from 0 to 1; a
  /// straight line for fewer than two.
  #[must_use]
  pub fn new(samples: Vec<f32>) -> Self {
    Self { samples }
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
  };
}

impl Default for OverviewEffects {
  fn default() -> Self {
    Self {
      glow: true,
      pulse: false,
      ripples: false,
      drop_particles: ParticleStyle::Sparks.into(),
      close_particles: ParticleStyle::Dust.into(),
      open_particles: ParticleEffects::NONE,
      exit_particles: ParticleEffects::NONE,
      jelly: true,
      stagger_ms: 24,
      lean: 8.0,
      dim: 0.0,
      squash: 0.5,
      trail: 0,
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
}

impl OverviewLayoutMode {
  /// Every layout, in the order `Tab` goes through them.
  pub const ALL: [Self; 2] = [Self::Activities, Self::Stack];

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
    self == Self::Activities
  }

  /// Name of the layout, as in the config.
  #[must_use]
  pub fn name(self) -> &'static str {
    match self {
      Self::Activities => "activities",
      Self::Stack => "stack",
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
