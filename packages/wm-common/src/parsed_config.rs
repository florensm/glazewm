use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wm_platform::{
  BackdropOverlayParams, BorderOverlayParams, Color, CornerStyle, Key,
  Keybinding, LengthUnit, LengthValue, OpacityValue, RectDelta,
};

use crate::{app_command::InvokeCommand, Layout};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct ParsedConfig {
  pub animations: AnimationsConfig,
  pub binding_modes: Vec<BindingModeConfig>,
  pub gaps: GapsConfig,
  pub general: GeneralConfig,
  pub keybindings: Vec<KeybindingConfig>,
  pub layouts: LayoutsConfig,
  pub overview: OverviewConfig,
  pub special_workspace: SpecialWorkspaceConfig,
  pub stack: StackConfig,
  pub window_behavior: WindowBehaviorConfig,
  pub window_effects: WindowEffectsConfig,
  pub window_rules: Vec<WindowRuleConfig>,
  pub workspaces: Vec<WorkspaceConfig>,
}

/// Position of the stack tab bar relative to the stack content.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TabBarPosition {
  #[default]
  Top,
  Bottom,
}

/// Where a window added to a stack is placed among its tabs.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum NewTabPosition {
  /// After the last tab.
  #[default]
  End,
  /// Right after the active tab.
  AfterActive,
}

/// Configuration for stacks and their tab bar.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct StackConfig {
  /// Height of the tab bar (0 = disabled).
  pub tab_bar_height: LengthValue,

  /// Whether the tab bar appears above or below the stack content.
  pub tab_bar_position: TabBarPosition,

  /// Background of the tab bar strip. Always drawn opaque.
  pub tab_bar_background: BorderColorSource,

  /// Background of the active tab's highlight.
  pub tab_active_background: BorderColorSource,

  /// Background of a hovered inactive tab.
  pub tab_hover_background: BorderColorSource,

  /// Background of an inactive tab whose window requests attention.
  pub tab_urgent_background: BorderColorSource,

  /// Background of inactive tabs. Transparent by default, so they show
  /// the strip.
  pub tab_inactive_background: BorderColorSource,

  /// Text color of the active tab.
  pub tab_text_color: BorderColorSource,

  /// Text color of inactive tabs.
  pub tab_inactive_text_color: BorderColorSource,

  /// Font family of tab titles; Segoe UI when it isn't installed.
  pub tab_font_family: String,

  /// Font size of tab titles.
  pub tab_font_size: LengthValue,

  /// Corner radius of the tab highlights and the strip's outer corners. A
  /// strip inside a window border follows the border's corners instead.
  pub tab_corner_radius: LengthValue,

  /// Narrowest a tab gets before the bar scrolls. Below about 60px, tabs
  /// only show their icon.
  pub tab_min_width: LengthValue,

  /// Widest a tab gets (0 = tabs share the whole bar).
  pub tab_max_width: LengthValue,

  /// Whether tabs show their window's icon.
  pub show_tab_icons: bool,

  /// Whether tab titles are prefixed with their position ("1. Title").
  pub show_tab_numbers: bool,

  /// When tabs show a close button.
  pub tab_close_button: TabCloseButton,

  /// Regex replacements applied to window titles before they are shown on
  /// tabs, in order.
  pub tab_title_overrides: Vec<TabTitleOverride>,

  /// Rules that put matching windows into a named stack as they open.
  pub auto_stack: Vec<AutoStackRuleConfig>,

  /// How long a window that could still match an `auto_stack` rule is
  /// held back while its title is empty, before it is placed normally.
  pub auto_stack_title_timeout_ms: u64,

  /// Whether a new tiling window opened while a stacked window has focus
  /// joins that stack. Otherwise it is tiled next to the stack.
  pub new_windows_join_focused_stack: bool,

  /// Where windows added to a stack are placed among its tabs.
  pub new_tab_position: NewTabPosition,

  /// Whether popups opened by an app that has windows in a stack (windows
  /// with an owner window from that app) open as floating windows.
  pub float_owned_popups: bool,
}

impl Default for StackConfig {
  fn default() -> Self {
    let color = |hex: &str| BorderColorSource::Value(hex.to_string());

    Self {
      tab_bar_height: LengthValue::from_px(28),
      tab_bar_position: TabBarPosition::Top,
      tab_bar_background: color("#1f1f1f"),
      tab_active_background: color("#3a3a3a"),
      tab_hover_background: color("#2c2c2c"),
      tab_urgent_background: color("#8a5a00"),
      tab_inactive_background: color("#00000000"),
      tab_text_color: color("#ffffff"),
      tab_inactive_text_color: color("#a0a0a0"),
      tab_font_family: "Segoe UI Variable Text".to_string(),
      tab_font_size: LengthValue::from_px(12),
      tab_corner_radius: LengthValue::from_px(8),
      tab_min_width: LengthValue::from_px(48),
      tab_max_width: LengthValue::from_px(0),
      show_tab_icons: true,
      show_tab_numbers: false,
      tab_close_button: TabCloseButton::Hover,
      tab_title_overrides: Vec::new(),
      auto_stack: Vec::new(),
      auto_stack_title_timeout_ms: 1500,
      new_windows_join_focused_stack: false,
      new_tab_position: NewTabPosition::End,
      float_owned_popups: true,
    }
  }
}

/// When tabs show a close button.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub enum TabCloseButton {
  /// On the active tab and the hovered tab.
  #[default]
  Hover,
  /// On every tab.
  Always,
  /// Never; tabs are closed with a middle click or the context menu.
  Never,
}

/// A regex replacement applied to window titles shown on tabs.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TabTitleOverride {
  /// Pattern to replace (Rust `regex` syntax).
  pub regex: String,

  /// Replacement text; `$1` etc. refer to capture groups.
  #[serde(default)]
  pub replace: String,
}

/// A rule that puts matching windows into a named stack when they open.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AutoStackRuleConfig {
  /// Name of the stack. One stack exists per name, across all workspaces.
  pub name: String,

  /// Windows matching any of these join the stack.
  #[serde(rename = "match")]
  pub match_window: Vec<WindowMatchConfig>,

  /// Windows matching any of these never join the stack.
  #[serde(default)]
  pub exclude: Vec<WindowMatchConfig>,

  /// Workspace to create the stack on when it doesn't exist yet. Defaults
  /// to the workspace the window would otherwise open on.
  #[serde(default)]
  pub workspace: Option<String>,

  /// Whether windows that have an owner window may join. Owned windows
  /// are usually dialogs, so they are skipped by default.
  #[serde(default)]
  pub allow_owned: bool,

  /// Key combinations (e.g. `ctrl+p`) pressed in a window once, after it
  /// opens and joins the stack.
  #[serde(
    default,
    deserialize_with = "deserialize_bindings",
    serialize_with = "serialize_bindings"
  )]
  pub send_keys_on_join: Vec<Keybinding>,

  /// What happens to tabs with the same title as a window joining the
  /// stack.
  #[serde(default)]
  pub duplicates: DuplicateTabs,

  /// Lets a new window that matches apart from its still empty title join
  /// the stack right away, rather than wait for its title. It leaves the
  /// stack again if the title it gets doesn't match.
  #[serde(default)]
  pub join_untitled: Option<JoinUntitledConfig>,
}

/// Size an untitled window needs to join its stack before it has a title.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct JoinUntitledConfig {
  pub min_width: LengthValue,
  pub min_height: LengthValue,
}

impl Default for JoinUntitledConfig {
  fn default() -> Self {
    Self {
      min_width: LengthValue::from_px(0),
      min_height: LengthValue::from_px(0),
    }
  }
}

/// What happens to tabs with the same title as a window joining a stack.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateTabs {
  /// Both are kept.
  #[default]
  Keep,

  /// The older windows are closed.
  CloseOlder,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BindingModeConfig {
  /// Name of the binding mode.
  pub name: String,

  /// Display name of the binding mode.
  #[serde(default)]
  pub display_name: Option<String>,

  /// Keybindings that will be active when the binding mode is active.
  #[serde(default)]
  pub keybindings: Vec<KeybindingConfig>,
}

/// Settings of the individual layouts.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct LayoutsConfig {
  pub scrolling: ScrollingLayoutConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct ScrollingLayoutConfig {
  /// Width of a new column.
  pub default_column_width: ColumnWidth,

  /// Widths `cycle-column-width` steps through, as percentages of the
  /// workspace or in pixels.
  pub preset_column_widths: Vec<LengthValue>,

  /// Heights `cycle-window-height` steps through, as percentages of the
  /// workspace or in pixels.
  pub preset_window_heights: Vec<LengthValue>,

  /// When focusing a column puts it in the middle of the workspace.
  pub center_focused_column: CenterFocusedColumn,

  /// Whether the only column of a workspace is kept in its middle.
  pub always_center_single_column: bool,

  /// How a column that windows are added to shows them.
  pub default_column_display: ColumnDisplay,

  /// With `general.focus_follows_cursor`, how far the view may scroll to
  /// focus the window under the cursor, as a percentage of the workspace
  /// width. Windows that would scroll it further aren't focused by
  /// hovering. Unset for no limit.
  pub focus_follows_cursor_max_scroll: Option<LengthValue>,

  /// Room kept free at the workspace's sides: columns are fitted between
  /// the struts, so the columns beside them always peek out there.
  pub struts: StrutsConfig,
}

/// Room a scrolling layout keeps free at the sides of its workspace, each
/// a percentage of the workspace width or pixels. Pixels scale with the
/// monitor's DPI as the gaps do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct StrutsConfig {
  pub left: LengthValue,
  pub right: LengthValue,
}

impl Default for StrutsConfig {
  fn default() -> Self {
    Self {
      left: LengthValue::from_px(0),
      right: LengthValue::from_px(0),
    }
  }
}

/// Width of a new column in a scrolling layout.
#[derive(Clone, Debug, PartialEq)]
pub enum ColumnWidth {
  /// The width its windows have as they join the strip, e.g. the size an
  /// app opened its window at. Written as `"window"`.
  Window,

  /// A percentage of the workspace, or pixels.
  Length(LengthValue),
}

/// Deserializes `"window"`, or a `LengthValue` as a string or a struct.
impl<'de> Deserialize<'de> for ColumnWidth {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ColumnWidthDe {
      String(String),
      Length(LengthValue),
    }

    match ColumnWidthDe::deserialize(deserializer)? {
      ColumnWidthDe::String(str) if str == "window" => Ok(Self::Window),
      ColumnWidthDe::String(str) => LengthValue::from_str(&str)
        .map(Self::Length)
        .map_err(serde::de::Error::custom),
      ColumnWidthDe::Length(length) => Ok(Self::Length(length)),
    }
  }
}

impl Serialize for ColumnWidth {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Window => serializer.serialize_str("window"),
      Self::Length(length) => length.serialize(serializer),
    }
  }
}

/// When focusing a column of a scrolling layout centers it.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CenterFocusedColumn {
  /// Never: the view scrolls just far enough to show the column whole.
  #[default]
  Never,
  /// The focused column is always in the middle.
  Always,
  /// Only a column that doesn't fit on screen together with the
  /// previously focused one is centered.
  OnOverflow,
}

/// How a column of a scrolling layout shows its windows.
#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  Deserialize,
  PartialEq,
  Eq,
  Serialize,
  clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum ColumnDisplay {
  /// Stacked one above the other.
  #[default]
  Normal,
  /// As the tabs of a stack, one shown at a time.
  Tabbed,
}

impl ScrollingLayoutConfig {
  /// Whether the view keeps the active column of a strip of `columns` in
  /// its middle, rather than only scrolling to show it.
  #[must_use]
  pub fn keeps_centered(&self, columns: usize) -> bool {
    self.center_focused_column == CenterFocusedColumn::Always
      || (self.always_center_single_column && columns <= 1)
  }
}

impl Default for ScrollingLayoutConfig {
  fn default() -> Self {
    let thirds = || {
      [1. / 3., 0.5, 2. / 3.]
        .into_iter()
        .map(|amount| LengthValue {
          amount,
          unit: LengthUnit::Percentage,
        })
        .collect()
    };

    Self {
      default_column_width: ColumnWidth::Length(LengthValue {
        amount: 0.5,
        unit: LengthUnit::Percentage,
      }),
      preset_column_widths: thirds(),
      preset_window_heights: thirds(),
      center_focused_column: CenterFocusedColumn::default(),
      always_center_single_column: false,
      default_column_display: ColumnDisplay::default(),
      focus_follows_cursor_max_scroll: None,
      struts: StrutsConfig::default(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct GapsConfig {
  /// Whether to scale the gaps with the DPI of the monitor.
  pub scale_with_dpi: bool,

  /// Gap between adjacent windows.
  pub inner_gap: LengthValue,

  /// Gap between windows and the screen edge.
  pub outer_gap: RectDelta,

  /// Gap between window and the screen edge if there is only one window
  /// in the workspace
  pub single_window_outer_gap: Option<RectDelta>,
}

impl Default for GapsConfig {
  fn default() -> Self {
    GapsConfig {
      scale_with_dpi: true,
      inner_gap: LengthValue::from_px(0),
      outer_gap: RectDelta::new(
        LengthValue::from_px(0),
        LengthValue::from_px(0),
        LengthValue::from_px(0),
        LengthValue::from_px(0),
      ),
      single_window_outer_gap: None,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
#[allow(clippy::struct_excessive_bools)]
pub struct GeneralConfig {
  /// Config for automatically moving the cursor.
  pub cursor_jump: CursorJumpConfig,

  /// Whether to automatically focus windows underneath the cursor.
  pub focus_follows_cursor: bool,

  /// Whether to switch back and forth between the previously focused
  /// workspace when focusing the current workspace.
  pub toggle_workspace_on_refocus: bool,

  /// Whether to keep focus where it is when a window on a hidden
  /// workspace forces itself into the foreground.
  ///
  /// The window is marked as urgent instead of being followed, which is
  /// broadcast via the `window_urgency_changed` event.
  pub ignore_focus_steal: bool,

  /// Whether workspaces can be created on-demand beyond the ones declared
  /// in the `workspaces` config.
  ///
  /// Dynamic workspaces are named with the lowest unused positive integer
  /// and are destroyed once they become empty.
  pub dynamic_workspaces: bool,

  /// Layout of workspaces that don't set their own `layout`.
  pub default_layout: Layout,

  /// Commands to run when the WM has started (e.g. to run a script or
  /// launch another application).
  pub startup_commands: Vec<InvokeCommand>,

  /// Commands to run just before the WM is shutdown.
  pub shutdown_commands: Vec<InvokeCommand>,

  /// Commands to run after the WM config has reloaded.
  pub config_reload_commands: Vec<InvokeCommand>,

  /// How windows should be hidden when switching workspaces.
  #[serde(deserialize_with = "deserialize_hide_method")]
  pub hide_method: HideMethod,

  /// Affects which windows get shown in the native Windows taskbar.
  pub show_all_in_taskbar: bool,
}

impl Default for GeneralConfig {
  fn default() -> Self {
    GeneralConfig {
      cursor_jump: CursorJumpConfig::default(),
      focus_follows_cursor: false,
      toggle_workspace_on_refocus: true,
      dynamic_workspaces: false,
      default_layout: Layout::Split,
      ignore_focus_steal: false,
      startup_commands: vec![],
      shutdown_commands: vec![],
      config_reload_commands: vec![],
      hide_method: {
        #[cfg(target_os = "macos")]
        {
          HideMethod::PlaceInCorner
        }
        #[cfg(not(target_os = "macos"))]
        {
          HideMethod::Cloak
        }
      },
      show_all_in_taskbar: false,
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct CursorJumpConfig {
  /// Whether to automatically move the cursor on the specified trigger.
  pub enabled: bool,

  /// Trigger for cursor jump.
  pub trigger: CursorJumpTrigger,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorJumpTrigger {
  #[default]
  MonitorFocus,
  WindowFocus,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HideMethod {
  Hide,
  #[default]
  Cloak,
  PlaceInCorner,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct KeybindingConfig {
  /// Keyboard shortcut to trigger the keybinding.
  #[serde(
    deserialize_with = "deserialize_bindings",
    serialize_with = "serialize_bindings"
  )]
  pub bindings: Vec<Keybinding>,

  /// WM commands to run when the keybinding is triggered.
  pub commands: Vec<InvokeCommand>,

  /// How long after it triggers it can't trigger again, e.g. to step once
  /// per flick of a mouse wheel.
  #[serde(default)]
  pub cooldown_ms: Option<u32>,
}

/// Look of the window overview opened by `toggle-overview`.
///
/// # Platform-specific
///
/// Only has an effect on Windows.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct OverviewConfig {
  /// Blur radius of the wallpaper behind the cards.
  pub backdrop_blur: f32,

  /// Tint over the blurred wallpaper; its alpha is how dark it gets.
  pub backdrop_tint: Color,

  /// Selection and highlights, given like a border's color (see
  /// [`BorderColorSource`]). The focused window's border color when
  /// `None`.
  pub accent_color: Option<BorderColorSource>,

  /// Background of a workspace card.
  pub card_color: Color,

  /// Background of a hovered card, and of a window without a preview.
  pub surface_color: Color,

  /// Strip a window's title sits on.
  pub caption_color: Color,

  pub text_color: Color,

  /// Secondary text, e.g. the search's hit count.
  pub subtext_color: Color,

  /// Border of windows matching a search.
  pub search_color: Color,

  /// Font of the overview's text; Segoe UI when it isn't installed.
  pub font_family: String,

  /// Columns of the grid of every workspace.
  pub grid_columns: usize,

  /// Layout `toggle-overview` opens on, unless it names one.
  pub layout: OverviewLayout,

  /// Length of the zoom out when the overview opens; `0` opens it
  /// instantly.
  pub open_duration_ms: u32,

  /// Length of the zoom into the picked workspace when the overview
  /// closes; `0` closes it instantly.
  pub close_duration_ms: u32,

  /// About how long cards and windows take to move to a new selection or
  /// layout, e.g. the shown workspace sliding over to the next; `0` moves
  /// them at once.
  pub switch_duration_ms: u32,

  /// How long the selection and hover highlights take to glide to what
  /// they mark next and to fade in or out, as the shortcut sheet does;
  /// `0` snaps them.
  pub highlight_duration_ms: u32,
}

/// How the overview lays out the workspaces.
#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  Deserialize,
  PartialEq,
  Eq,
  Serialize,
  clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum OverviewLayout {
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

impl Default for OverviewConfig {
  fn default() -> Self {
    // `0xRRGGBBAA`, as in the config.
    let color = |hex: u32| {
      let [r, g, b, a] = hex.to_be_bytes();
      Color { r, g, b, a }
    };

    OverviewConfig {
      backdrop_blur: 40.0,
      backdrop_tint: color(0x0000_0066),
      accent_color: None,
      card_color: color(0x221e_24e6),
      surface_color: color(0x2d29_2eff),
      caption_color: color(0x1612_17c7),
      text_color: color(0xe8e0_e8ff),
      subtext_color: color(0xcdc3_ceff),
      search_color: color(0x89b4_faff),
      font_family: "Segoe UI Variable Text".to_string(),
      grid_columns: 5,
      layout: OverviewLayout::default(),
      open_duration_ms: 250,
      close_duration_ms: 200,
      switch_duration_ms: 300,
      highlight_duration_ms: 170,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowBehaviorConfig {
  /// New windows are created in this state whenever possible.
  pub initial_state: InitialWindowState,

  /// Sets the default options for when a new window is created. This also
  /// changes the defaults for when the state change commands, like
  /// `set_floating`, are used without any flags.
  pub state_defaults: WindowStateDefaultsConfig,

  /// Whether a new window on a workspace takes any fullscreen window on
  /// that workspace back out of fullscreen.
  ///
  /// A fullscreen window covers the entire workspace, so without this a
  /// newly spawned window opens hidden behind it. Matches Hyprland, which
  /// drops fullscreen as soon as another window joins the workspace.
  pub exit_fullscreen_on_new_window: bool,

  /// Highlight of where a dragged window will land.
  pub drop_indicator: DropIndicatorConfig,
}

impl Default for WindowBehaviorConfig {
  fn default() -> Self {
    WindowBehaviorConfig {
      initial_state: InitialWindowState::default(),
      state_defaults: WindowStateDefaultsConfig::default(),
      exit_fullscreen_on_new_window: true,
      drop_indicator: DropIndicatorConfig::default(),
    }
  }
}

/// Highlight shown while dragging a window, over where it will land if
/// dropped: the half of the window it goes beside, or the stack it joins.
///
/// # Platform-specific
///
/// Only has an effect on Windows.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct DropIndicatorConfig {
  pub enabled: bool,

  /// Given like a border's color (see [`BorderColorSource`]). The focused
  /// window's border color when `None`.
  pub color: Option<BorderColorSource>,
}

impl Default for DropIndicatorConfig {
  fn default() -> Self {
    DropIndicatorConfig {
      enabled: true,
      color: None,
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialWindowState {
  #[default]
  Tiling,
  Floating,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowStateDefaultsConfig {
  pub floating: FloatingStateConfig,
  pub fullscreen: FullscreenStateConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct FloatingStateConfig {
  /// Whether to center new floating windows.
  pub centered: bool,

  /// Whether to show floating windows as always on top.
  pub shown_on_top: bool,
}

impl Default for FloatingStateConfig {
  fn default() -> Self {
    FloatingStateConfig {
      centered: true,
      // Defaults on because tiling windows fill the workspace: the z-order
      // pass raises whichever state group the focused window belongs to,
      // so focusing a tiling window otherwise buries every floating
      // one behind a full-screen-width tile with no way to see it.
      // Still an option -- set `false` to get the old behaviour
      // back.
      shown_on_top: true,
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct FullscreenStateConfig {
  /// Whether to prefer fullscreen windows to be maximized.
  pub maximized: bool,

  /// Whether to show fullscreen windows as always on top.
  pub shown_on_top: bool,
}

impl Default for FullscreenStateConfig {
  fn default() -> Self {
    FullscreenStateConfig {
      maximized: true,
      shown_on_top: false,
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowEffectsConfig {
  /// Visual effects to apply to the focused window.
  pub focused_window: WindowEffectConfig,

  /// Visual effects to apply to non-focused windows.
  pub other_windows: WindowEffectConfig,

  /// Whether to drop the border effect while a window is fullscreen.
  ///
  /// A fullscreen window has nothing beside it to be delimited from, so
  /// the ring reads as an inset frame around the screen -- and on a
  /// monitor-sized window it is drawn over content rather than beside
  /// it.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub hide_border_on_fullscreen: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowEffectConfig {
  /// Config for optionally applying a colored border.
  pub border: BorderEffectConfig,

  /// Config for optionally hiding the title bar.
  pub hide_title_bar: HideTitleBarEffectConfig,

  /// Config for optionally changing the corner style.
  pub corner_style: CornerEffectConfig,

  /// Config for optionally applying transparency.
  pub transparency: TransparencyEffectConfig,

  /// Config for optionally applying a DWM backdrop material.
  #[serde(alias = "blur_behind")]
  pub backdrop: BackdropEffectConfig,

  /// Single knob for how round this window's corners are.
  ///
  /// Drives the window's own corners *and* every overlay drawn around it,
  /// so the two always agree.
  ///
  /// The overlays follow this value exactly. The window itself can only
  /// take the three presets Windows exposes through
  /// `DWMWA_WINDOW_CORNER_PREFERENCE`, so it snaps to the nearest one:
  /// `0` gives square, up to [`CornerStyle::SmallRounded`]'s radius
  /// gives small rounded, anything above gives rounded. A radius past
  /// roughly 8px therefore rounds the ring further than the window can
  /// follow -- an OS limit, not a config one.
  ///
  /// When unset, falls back to the `corner_style` block. `border.radius`
  /// still overrides the ring on top of either.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub corner_radius: Option<LengthValue>,
}

impl WindowEffectConfig {
  /// The corner preference to hand the real window.
  ///
  /// `corner_radius` wins when set, snapped to the nearest preset;
  /// otherwise the `corner_style` block decides, and `Default` means
  /// "leave whatever the app asked for alone".
  #[must_use]
  pub fn resolved_corner_style(&self) -> CornerStyle {
    match &self.corner_radius {
      Some(radius) => {
        #[allow(clippy::cast_precision_loss)]
        let px = radius.to_px(0, None) as f32;

        if px <= 0.0 {
          CornerStyle::Square
        } else if px <= CornerStyle::SmallRounded.approx_radius_px() {
          CornerStyle::SmallRounded
        } else {
          CornerStyle::Rounded
        }
      }
      None if self.corner_style.enabled => self.corner_style.style,
      None => CornerStyle::Default,
    }
  }

  /// Radius, in px, that overlays should treat the window's corners as
  /// having, so they sit concentric with it.
  ///
  /// Follows `corner_radius` exactly when set -- deliberately *not* the
  /// snapped preset's radius, so a ring can be rounder than the three
  /// presets allow (which is the point of accepting a free-form radius
  /// at all).
  #[must_use]
  pub fn window_corner_radius_px(&self) -> f32 {
    match &self.corner_radius {
      #[allow(clippy::cast_precision_loss)]
      Some(radius) => radius.to_px(0, None) as f32,
      None => self.resolved_corner_style().approx_radius_px(),
    }
  }
}

/// Backdrop rendered behind a window: a crop of the desktop wallpaper,
/// blurred once per monitor into an opaque image.
///
/// In a tiling layout nothing sits behind a tiled window except the
/// wallpaper, so the image never changes and is computed once rather than
/// per frame. Being opaque, it also lets DWM skip compositing what is
/// behind the overlay rather than blending it every frame.
///
/// Drawn by a persistent overlay window behind the managed window, never
/// applied to the window itself: SWCA on a window the `transparency`
/// effect has made `WS_EX_LAYERED` conflicts, and DWM's own materials only
/// paint where an application leaves its surface unpainted, which nearly
/// none do.
///
/// The overlay's own corner radius is deliberately not configurable here:
/// it is derived from `corner_style` (see `CornerStyle::approx_radius_px`)
/// so it always matches the real managed window's rendered corners instead
/// of risking a visual mismatch.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct BackdropEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// RGBA tint blended over the blurred backdrop.
  ///
  /// Accepts `#RRGGBB` (fully opaque) or `#RRGGBBAA`. When `None`, a
  /// near-transparent black (`alpha = 1`) is used.
  pub tint: Option<Color>,

  /// Blur radius/intensity.
  ///
  /// Baked into the per-monitor image rather than evaluated per frame, so
  /// changing it re-renders that image once and costs nothing thereafter.
  pub blur_amount: f32,

  /// Opacity of the backdrop overlay's own composited visual (blur and
  /// tint together, as one unit), from `0.0` to `1.0`.
  ///
  /// Distinct from `tint`'s own alpha channel: `tint`'s alpha only blends
  /// the flat tint layer over the blur layer beneath it, while this
  /// scales the *entire* overlay -- unrelated to the real managed
  /// window's own `transparency` effect, which fades the window itself
  /// via `SetLayeredWindowAttributes`, not the overlay.
  pub opacity: f32,

  /// Saturation of the blurred backdrop, from `0.0` (grayscale) to `2.0`
  /// (oversaturated); `1.0` leaves it unchanged. Values outside that
  /// range aren't clamped -- D2D1's `Saturation` effect accepts them but
  /// the result is undefined/implementation-specific.
  pub saturation: f32,

  /// Exposure adjustment in stops; `0.0` is unchanged, negative darkens.
  pub exposure: f32,

  /// Contrast adjustment from `-1.0` to `1.0`; `0.0` is unchanged.
  pub contrast: f32,

  /// Highlight recovery from `-1.0` to `1.0`; `0.0` is unchanged,
  /// negative pulls bright areas down.
  ///
  /// Tone-selective where `exposure` is not: it touches only the bright
  /// end, so a blown-out sky can be brought down without dragging the
  /// whole image toward black. The right knob for making text readable.
  pub highlights: f32,

  /// Shadow lift from `-1.0` to `1.0`; `0.0` is unchanged, positive opens
  /// dark areas up.
  pub shadows: f32,

  /// Strength of a darkening gradient toward each window's own edges,
  /// from `0.0` (off) to `1.0`.
  ///
  /// The only one of these knobs that is not baked: it is a gradient
  /// visual measured from the window's rect. Baking it would anchor the
  /// falloff to the monitor instead, since one image is shared by every
  /// window on it.
  ///
  /// The right knob for toning a bright backdrop down behind text -- it
  /// leaves the overlay opaque, where `opacity` would make it translucent
  /// again and undo the backdrop's whole reason for being.
  pub vignette: f32,

  /// Opacity of a monochrome noise layer over the blurred image, from
  /// `0.0` (off) to `1.0`.
  ///
  /// A fine dither is what makes a blurred image read as frosted glass
  /// rather than an out-of-focus photo.
  pub grain: f32,

  /// How much the backdrop follows the window across its monitor. `1.0`
  /// pins the image to the desktop; lower values let it drift against the
  /// window as it moves, which reads as depth.
  ///
  /// Not baked -- it selects a different part of an already-rendered
  /// image, so any value costs the same (nothing).
  pub parallax: f32,
}

impl Default for BackdropEffectConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      tint: None,
      blur_amount: 30.0,
      opacity: 1.0,
      saturation: 1.0,
      exposure: 0.0,
      contrast: 0.0,
      highlights: 0.0,
      shadows: 0.0,
      vignette: 0.0,
      grain: 0.0,
      parallax: 1.0,
    }
  }
}

impl BackdropEffectConfig {
  /// Returns the tint for the backdrop overlay.
  ///
  /// Doubles as the "does this window get a `NativeBackdropOverlay`?"
  /// predicate for every overlay call site, so it returns `None` only when
  /// the effect is disabled -- every remaining style is drawn by an
  /// overlay. When no tint is set, falls back to near-transparent black
  /// (`alpha = 1`) to avoid the solid-fill rendering bug present on some
  /// Windows 10 builds.
  #[must_use]
  pub fn overlay_tint(&self) -> Option<Color> {
    if !self.enabled {
      return None;
    }

    Some(self.tint.unwrap_or(Color {
      r: 0,
      g: 0,
      b: 0,
      a: 1,
    }))
  }

  /// Builds a [`BackdropOverlayParams`] from this config, given the
  /// already-resolved `tint` (see `overlay_tint`) and `corner_radius`
  /// (derived from the sibling `corner_style` effect, not stored on this
  /// struct -- see the trailing comment on this struct's definition).
  ///
  /// Consolidates the field list every `BackdropOverlayParams` call site
  /// would otherwise hand-write, so adding a new overlay knob only touches
  /// this method and the config struct itself, not every call site.
  #[must_use]
  pub fn to_overlay_params(
    &self,
    tint: Color,
    corner_radius: f32,
  ) -> BackdropOverlayParams {
    BackdropOverlayParams {
      tint,
      blur_amount: self.blur_amount,
      corner_radius,
      opacity: self.opacity,
      saturation: self.saturation,
      exposure: self.exposure,
      contrast: self.contrast,
      highlights: self.highlights,
      shadows: self.shadows,
      vignette: self.vignette,
      grain: self.grain,
      parallax: self.parallax,
    }
  }
}

/// Default/fallback border color: used both as [`BorderColorSource`]'s
/// default and when a dynamic source (`"accent"` or `file`) fails to
/// resolve.
const DEFAULT_BORDER_COLOR: Color = Color {
  r: 140,
  g: 190,
  b: 255,
  a: 255,
};

/// Where a border's color comes from.
///
/// Re-resolved every time the border's target color is needed (on focus
/// change, or on a config reload), so a dynamic source tracks its live
/// value with no restart needed.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BorderColorSource {
  /// A static hex color (`"#rrggbb"`/`"#rrggbbaa"`), or the literal
  /// `"accent"` for the OS's live accent/colorization color.
  Value(String),

  /// Reads a named CSS custom property (`--key: #hex;` or
  /// `--key: rgb(r, g, b);`) from an external generated palette file --
  /// e.g. matugen, pywal, or YASB's `yasb_colors.css`.
  File {
    /// Path to the palette file.
    file: std::path::PathBuf,
    /// The CSS custom property's name, including its leading `--`.
    key: String,
  },
}

impl Default for BorderColorSource {
  fn default() -> Self {
    Self::Value("#8cbeff".to_string())
  }
}

impl BorderColorSource {
  /// Resolves this source to a concrete [`Color`].
  ///
  /// Falls back to [`DEFAULT_BORDER_COLOR`] (logging a warning) if
  /// resolution fails -- an unreadable file, a missing key, an
  /// unrecognized value, an unparseable static hex string, or an OS
  /// accent-color read failure (e.g. on an OS version predating the
  /// `DwmGetColorizationColor` API).
  #[must_use]
  pub fn resolve(&self) -> Color {
    let result: wm_platform::Result<Color> = match self {
      Self::Value(value) if value.eq_ignore_ascii_case("accent") => {
        wm_platform::system_accent_color()
      }
      Self::Value(value) => value.parse::<Color>().map_err(Into::into),
      Self::File { file, key } => wm_platform::color_from_file(file, key),
    };

    result.unwrap_or_else(|err| {
      tracing::warn!(
        "Failed to resolve border color ({err}), falling back to a \
         default color."
      );
      DEFAULT_BORDER_COLOR
    })
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct BorderEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// Color of the window border: a static hex string, the literal
  /// `"accent"` for the OS's live accent/colorization color, or a
  /// `{ file, key }` mapping to read a named CSS custom property from an
  /// external generated palette file (matugen, pywal, YASB, etc.). See
  /// [`BorderColorSource`].
  pub color: BorderColorSource,

  /// Thickness of the window border.
  pub width: LengthValue,

  /// Corner radius of the border ring's outer edge. When `None` (the
  /// default), it's derived from the sibling `corner_style` effect's
  /// radius plus `width`, so the ring lines up concentrically with the
  /// real window's own corners. Set explicitly to use a different radius
  /// than that approximation -- e.g. a larger, more rounded look than
  /// `corner_style`'s fixed presets (`square`/`small_rounded`/`rounded`)
  /// allow.
  pub radius: Option<LengthValue>,
}

impl Default for BorderEffectConfig {
  fn default() -> Self {
    BorderEffectConfig {
      enabled: false,
      color: BorderColorSource::default(),
      width: LengthValue::from_px(2),
      radius: None,
    }
  }
}

impl BorderEffectConfig {
  /// Border color, or `None` when the border effect is disabled.
  ///
  /// See [`BorderColorSource::resolve`] for how `color` is resolved,
  /// including its fallback behavior on a failed dynamic read.
  #[must_use]
  pub fn abgr_color(&self) -> Option<Color> {
    if !self.enabled {
      return None;
    }

    Some(self.color.resolve())
  }

  /// Builds a [`BorderOverlayParams`] from this config, given the
  /// already-resolved `color` (see [`abgr_color`]) and the tracked
  /// window's *own* corner radius (derived from the sibling
  /// `corner_style` effect, not stored on this type).
  ///
  /// The overlay's own outer corner radius is `self.radius` when set,
  /// otherwise `window_corner_radius + width` so the ring lines up
  /// concentrically with the real window's own corners by default.
  ///
  /// [`abgr_color`]: BorderEffectConfig::abgr_color
  #[must_use]
  pub fn to_overlay_params(
    &self,
    color: Color,
    window_corner_radius: f32,
  ) -> BorderOverlayParams {
    #[allow(clippy::cast_precision_loss)]
    let width = self.width.to_px(0, None) as f32;

    #[allow(clippy::cast_precision_loss)]
    let corner_radius = self
      .radius
      .as_ref()
      .map_or(window_corner_radius + width, |r| r.to_px(0, None) as f32);

    BorderOverlayParams {
      color,
      width,
      corner_radius,
      opacity: 1.0,
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct HideTitleBarEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct CornerEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// Style of the window corners.
  pub style: CornerStyle,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct TransparencyEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// The opacity to apply.
  pub opacity: OpacityValue,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowRuleConfig {
  pub commands: Vec<InvokeCommand>,

  #[serde(rename = "match")]
  pub match_window: Vec<WindowMatchConfig>,

  #[serde(default = "default_window_rule_on")]
  pub on: Vec<WindowRuleEvent>,

  #[serde(default = "default_bool::<true>")]
  pub run_once: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowMatchConfig {
  pub window_process: Option<MatchType>,
  pub window_class: Option<MatchType>,
  pub window_title: Option<MatchType>,
}

/// Due to limitations in `serde_yaml`, we need to use an untagged enum
/// instead of a regular enum for serialization. Using a regular enum
/// causes issues with flow-style objects in YAML.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MatchType {
  Equals { equals: String },
  Includes { includes: String },
  Regex { regex: String },
  NotEquals { not_equals: String },
  NotRegex { not_regex: String },
}

impl MatchType {
  /// Whether the given value is a match for the match type.
  #[must_use]
  pub fn is_match(&self, value: &str) -> bool {
    match self {
      MatchType::Equals { equals } => value == equals,
      MatchType::Includes { includes } => value.contains(includes),
      MatchType::Regex { regex } => {
        regex::Regex::new(regex).is_ok_and(|re| re.is_match(value))
      }
      MatchType::NotEquals { not_equals } => value != not_equals,
      MatchType::NotRegex { not_regex } => {
        regex::Regex::new(not_regex).is_ok_and(|re| !re.is_match(value))
      }
    }
  }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowRuleEvent {
  /// When a window receives native focus.
  Focus,

  /// When a window is initially managed.
  Manage,

  /// When the title of a window changes.
  TitleChange,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WorkspaceConfig {
  pub name: String,

  #[serde(default)]
  pub display_name: Option<String>,

  #[serde(default)]
  pub bind_to_monitor: Option<u32>,

  #[serde(default = "default_bool::<false>")]
  pub keep_alive: bool,

  /// Overrides `general.default_layout` for this workspace.
  #[serde(default)]
  pub layout: Option<Layout>,
}

/// The special workspace: a scratch workspace outside the `workspaces`
/// list, summoned onto the focused monitor.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct SpecialWorkspaceConfig {
  /// Name to address it by. Must not be numeric or a `workspaces` name.
  pub name: String,

  pub display_name: Option<String>,

  pub layout: Layout,
}

impl Default for SpecialWorkspaceConfig {
  fn default() -> Self {
    Self {
      name: "special".to_string(),
      display_name: None,
      layout: Layout::Scrolling,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct AnimationsConfig {
  /// Animation settings for pure window translations (position changes
  /// only).
  pub window_move: AnimationTypeConfig,
  /// Animation settings for how a scrolling layout's columns move and
  /// resize, the strip scrolling included.
  pub scroll: AnimationTypeConfig,
  /// Animation settings for operations that change window size.
  pub window_resize: WindowResizeConfig,
  /// Animation settings for when a new window appears.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub window_open: WindowOpenConfig,
  /// Animation settings for workspace-switch slide transitions.
  pub workspace_switch: WorkspaceSwitchAnimationConfig,
  /// Animation settings for when a window is closed.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub window_close: WindowCloseConfig,
  /// Animation settings for when a window is minimized or restored.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub window_minimize: WindowMinimizeConfig,

  /// Which windows keep their effect overlays tracking during an
  /// animation.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub overlay_tracking: OverlayTracking,
}

/// Which animating windows keep their backdrop and border overlays glued
/// to them for the duration of an animation.
///
/// Every animating window is composited as three windows, not one: the
/// surrogate thumbnail plus a backdrop overlay and a border overlay, each
/// repositioned every frame. Since the animation is bound by how many
/// surfaces DWM has to absorb, this is the largest lever available -- a
/// measured 2.1x between borders on every window and borders on the
/// focused window only.
///
/// Only affects windows *while they animate*; the static overlays are
/// restored by the normal sync pass as soon as the animation ends, so the
/// at-rest appearance is unchanged.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayTracking {
  /// Track every animating window's overlays. Smoothest at rest, most
  /// expensive in motion.
  #[default]
  All,
  /// Track only the focused window's overlays; other windows animate
  /// without theirs.
  FocusedOnly,
  /// Track none of them; all overlays are hidden for the animation.
  None,
}

impl Default for AnimationsConfig {
  fn default() -> Self {
    AnimationsConfig {
      window_move: AnimationTypeConfig::default(),
      scroll: AnimationTypeConfig {
        duration_ms: 220,
        easing: EasingFunction::Spring(SpringParams::default()),
        ..AnimationTypeConfig::default()
      },
      window_resize: WindowResizeConfig::default(),
      window_open: WindowOpenConfig::default(),
      workspace_switch: WorkspaceSwitchAnimationConfig::default(),
      window_close: WindowCloseConfig::default(),
      window_minimize: WindowMinimizeConfig::default(),
      overlay_tracking: OverlayTracking::default(),
    }
  }
}

/// Spatial style for window open/close transitions.
///
/// Used by both `WindowOpenConfig.style` and `WindowCloseConfig.style` so
/// the same values apply symmetrically: a window that opens with
/// `slide_right` (entering from the right) closes with `slide_right`
/// (exiting to the right).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowTransitionStyle {
  /// Slide in/out from/to the right edge (default).
  #[default]
  #[serde(alias = "right")]
  SlideRight,
  /// Slide in/out from/to the left edge.
  #[serde(alias = "left")]
  SlideLeft,
  /// Slide in/out from/to the top edge.
  #[serde(alias = "top")]
  SlideTop,
  /// Slide in/out from/to the bottom edge.
  #[serde(alias = "bottom")]
  SlideBottom,
  /// No positional movement. Combine with `opacity_from`/`opacity_to` for
  /// a pure fade. Accepts `"fade"` as a legacy alias.
  #[serde(alias = "fade")]
  None,
  /// Zoom in/out from the window center. Combine with
  /// `opacity_from`/`opacity_to` to also fade while zooming.
  Zoom,
}

impl WindowTransitionStyle {
  /// Returns `true` when the style has no positional slide component.
  ///
  /// Stationary styles keep the surrogate at the window's final position
  /// for the full animation; the surrogate window itself never moves.
  #[must_use]
  pub fn is_stationary(&self) -> bool {
    matches!(self, Self::None | Self::Zoom)
  }
}

/// Resolved parameters for a one-shot window transition.
///
/// `window_open` and `window_minimize` drive the same entry animation and
/// `window_close`/`window_minimize` the same exit animation; they differ
/// only in which config block supplies the values. Resolving to this
/// struct lets one driver serve all of them instead of branching on the
/// config type.
#[derive(Clone, Debug)]
pub struct WindowTransitionParams {
  pub duration_ms: u32,
  pub easing: EasingFunction,
  pub style: WindowTransitionStyle,
  /// Opacity at the away end of the transition -- the value a window
  /// enters from and exits to.
  pub away_opacity: f32,
}

impl WindowTransitionParams {
  #[must_use]
  pub fn from_open(config: &WindowOpenConfig) -> Self {
    Self {
      duration_ms: config.duration_ms,
      easing: config.easing.clone(),
      style: config.style.clone(),
      away_opacity: config.opacity_from,
    }
  }

  #[must_use]
  pub fn from_close(config: &WindowCloseConfig) -> Self {
    Self {
      duration_ms: config.duration_ms,
      easing: config.easing.clone(),
      style: config.style.clone(),
      away_opacity: config.opacity_to,
    }
  }

  #[must_use]
  pub fn from_minimize(config: &WindowMinimizeConfig) -> Self {
    Self {
      duration_ms: config.duration_ms,
      easing: config.easing.clone(),
      style: config.style.clone(),
      away_opacity: config.opacity_to,
    }
  }
}

/// Animation settings for when a new window appears (Windows only).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowOpenConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Transition type for the open animation.
  ///
  /// - `slide_right` (default): slides in from the right.
  /// - `slide_left` / `slide_top` / `slide_bottom`: slide from that edge.
  /// - `none` / `fade`: no slide; combine with `opacity_from` for a pure
  ///   fade-in.
  /// - `zoom`: zoom in from the window center.
  ///
  /// Accepts `direction` as a legacy alias for this key.
  #[serde(alias = "direction")]
  pub style: WindowTransitionStyle,
  /// Starting opacity (0.0–1.0). At `1.0` no fade is applied; at `0.0`
  /// the window fades in from fully transparent. Can be combined with
  /// any style.
  pub opacity_from: f32,
}

impl Default for WindowOpenConfig {
  fn default() -> Self {
    WindowOpenConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.16, 1.0, 0.3, 1.0),
      style: WindowTransitionStyle::SlideRight,
      opacity_from: 1.0,
    }
  }
}

/// Animation settings for when a window is closed.
///
/// # Platform-specific
///
/// Only has an effect on Windows.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowCloseConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Transition type for the close animation.
  ///
  /// - `none` / `fade` (default): no positional movement; combine with
  ///   `opacity_to` for a pure fade-out.
  /// - `zoom`: zoom out from the window center.
  /// - `slide_right` / `slide_left` / `slide_top` / `slide_bottom`: slide
  ///   off that edge.
  pub style: WindowTransitionStyle,
  /// Final opacity (0.0–1.0). At `0.0` the window fades to fully
  /// transparent; at `1.0` no fade is applied.
  pub opacity_to: f32,
}

impl Default for WindowCloseConfig {
  fn default() -> Self {
    WindowCloseConfig {
      enabled: false,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.32, 0.0, 0.67, 0.0),
      style: WindowTransitionStyle::None,
      opacity_to: 0.0,
    }
  }
}

/// Animation settings for when a window is minimized or restored from
/// minimized.
///
/// One block drives both directions, the same way `WindowTransitionStyle`
/// is shared between open and close: a window that minimizes with
/// `slide_bottom` (exiting toward the bottom edge) restores by entering
/// from that same edge. `opacity_to` is the minimize-out target, and is
/// played in reverse as the restore-in starting opacity.
///
/// # Platform-specific
///
/// Only has an effect on Windows.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowMinimizeConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Transition style for the minimize/restore animation.
  ///
  /// - `zoom` (default): shrink toward the window center on minimize,
  ///   grow back out on restore.
  /// - `slide_bottom` / `slide_top` / `slide_left` / `slide_right`: slide
  ///   off that edge on minimize, and back in from it on restore.
  /// - `none` / `fade`: no movement; combine with `opacity_to` for a pure
  ///   fade.
  pub style: WindowTransitionStyle,
  /// Opacity at the end of the minimize animation (0.0-1.0), and at the
  /// start of the restore animation. At `0.0` the window fades fully
  /// out; at `1.0` no fade is applied.
  pub opacity_to: f32,
}

impl Default for WindowMinimizeConfig {
  fn default() -> Self {
    WindowMinimizeConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.32, 0.0, 0.67, 0.0),
      style: WindowTransitionStyle::Zoom,
      opacity_to: 0.0,
    }
  }
}

/// Motion style of the workspace-switch transition.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchStyle {
  /// Workspaces slide along the axis set by `direction` (default).
  #[default]
  Slide,
  /// Pure crossfade; no positional slide. Both surrogates stay in place
  /// and their opacities are driven by `opacity_outgoing` /
  /// `opacity_incoming`.
  Fade,
  /// Outgoing workspace shrinks to the monitor center; incoming expands
  /// from it. Opacities are also animated via `opacity_outgoing` /
  /// `opacity_incoming`.
  Zoom,
  /// Iris wipe: a frozen snapshot of the outgoing workspace stays on top
  /// while a circular hole grows from `iris_origin`, revealing the live
  /// incoming workspace beneath. Requires Windows; falls back to an
  /// instant switch when the monitor cannot be captured.
  Iris,
}

impl WorkspaceSwitchStyle {
  /// Returns `true` when the transition has no positional slide component.
  #[must_use]
  pub fn is_no_slide(&self) -> bool {
    matches!(self, Self::Fade | Self::Zoom)
  }

  /// Returns `true` for the iris-wipe style, which is driven by a single
  /// snapshot overlay rather than per-window surrogates.
  #[must_use]
  pub fn is_iris(&self) -> bool {
    matches!(self, Self::Iris)
  }
}

/// Origin point from which the iris-wipe circle grows.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchIrisOrigin {
  /// Grow from the center of the monitor (default).
  #[default]
  Center,
  /// Grow from the current mouse-cursor position.
  Cursor,
  /// Grow from the center of the newly focused window on the incoming
  /// workspace. Falls back to the monitor center when the incoming
  /// workspace has no focusable window.
  FocusedWindow,
}

/// What plays the workspace-switch transition.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchEngine {
  /// One overlay per monitor draws live copies of everything on it while
  /// the switch happens at once beneath it, as with animations off.
  #[default]
  Scene,
  /// A stand-in window per window, with the real windows cloaked until
  /// the transition ends.
  Surrogates,
}

/// Slide axis for the `slide` workspace-switch style.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchDirection {
  /// Slide left/right (default).
  #[default]
  Horizontal,
  /// Slide up/down.
  Vertical,
}

/// Animation config for workspace-switch transitions.
///
/// Outgoing workspaces translate off-screen (for the `slide` style) or
/// stay in place (for `fade`/`zoom`) while the incoming workspace slides
/// or crossfades in, all constrained to the monitor on which the switch
/// occurs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WorkspaceSwitchAnimationConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Motion type: `slide` (default), `fade`, `zoom`, or `iris`.
  pub style: WorkspaceSwitchStyle,
  /// What plays `slide`, `fade` and `zoom`: `scene` (default) or
  /// `surrogates`. `scene` falls back to `surrogates` on Windows builds
  /// without the DWM exports it needs.
  pub engine: WorkspaceSwitchEngine,
  /// Slide axis when `style` is `slide`: `horizontal` (default) or
  /// `vertical`.
  pub direction: WorkspaceSwitchDirection,
  /// Origin of the iris circle when `style` is `iris`: `center`
  /// (default), `cursor`, or `focused_window`. Ignored by other styles.
  pub iris_origin: WorkspaceSwitchIrisOrigin,
  /// Opacity at the end of the outgoing workspace's animation (0.0–1.0).
  ///
  /// At `1.0` (default) the outgoing workspace stays fully opaque. At
  /// `0.0` it fades out to transparent. Any value in between produces a
  /// partial fade. Applies to all `style` values.
  pub opacity_outgoing: f32,
  /// Opacity at the start of the incoming workspace's animation
  /// (0.0–1.0).
  ///
  /// At `1.0` (default) the incoming workspace starts fully opaque. At
  /// `0.0` it fades in from transparent. Any value in between produces
  /// a partial fade. Applies to all `style` values.
  pub opacity_incoming: f32,
  /// Amount of workspace-level scale applied during `slide` transitions.
  ///
  /// The outgoing workspace shrinks from `1.0` to `1.0 - zoom_factor` as
  /// it exits; the incoming grows from `1.0 - zoom_factor` to `1.0` as
  /// it enters. Scaling is from the monitor center so all windows move
  /// inward together, preserving the workspace-as-a-panel illusion. Has
  /// no effect on `fade` or `zoom` styles. Valid range: `0.0` (no zoom)
  /// to `1.0` (collapses to a point). Recommended range: `0.05`–`0.15`
  /// for a subtle depth effect.
  pub zoom_factor: f32,
}

impl Default for WorkspaceSwitchAnimationConfig {
  fn default() -> Self {
    WorkspaceSwitchAnimationConfig {
      enabled: true,
      duration_ms: 250,
      easing: EasingFunction::CubicBezier(0.16, 1.0, 0.3, 1.0),
      style: WorkspaceSwitchStyle::default(),
      engine: WorkspaceSwitchEngine::default(),
      direction: WorkspaceSwitchDirection::default(),
      iris_origin: WorkspaceSwitchIrisOrigin::default(),
      opacity_outgoing: 1.0,
      opacity_incoming: 1.0,
      zoom_factor: 0.1,
    }
  }
}

/// Animation settings for window move operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct AnimationTypeConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Minimum pixel distance required to trigger movement animations.
  /// Helps prevent animations from starting on very small position
  /// changes. Increase this value on high-DPI displays to reduce
  /// sensitivity.
  pub threshold_px: u32,
}

impl Default for AnimationTypeConfig {
  fn default() -> Self {
    AnimationTypeConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0),
      threshold_px: 10,
    }
  }
}

/// Animation settings for window resize operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowResizeConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Minimum pixel distance required to trigger resize animations.
  /// Increase this value on high-DPI displays to reduce sensitivity.
  pub threshold_px: u32,
  /// How a window's content is shown while its size animates.
  pub style: ResizeStyle,
}

impl Default for WindowResizeConfig {
  fn default() -> Self {
    WindowResizeConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0),
      threshold_px: 10,
      style: ResizeStyle::default(),
    }
  }
}

/// How a window's content is shown while its size animates.
#[derive(
  Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum ResizeStyle {
  /// Content stays at its real size: a growing window reveals it as the
  /// app catches up, with the uncovered strip filled in the window's
  /// sampled edge color; a shrinking one is clipped. The real window is
  /// resized late in the animation.
  #[default]
  Fill,
  /// Content is scaled to the animated size. The real window is sent its
  /// final size on the first frame, and no edge color is sampled.
  Stretch,
}

/// Easing function for animations.
///
/// Named aliases map to their CSS cubic-bezier equivalents and can be used
/// interchangeably with `cubic_bezier(x1, y1, x2, y2)` notation:
/// `linear`, `ease_in`, `ease_out`, `ease_in_out`,
/// `ease_in_cubic`, `ease_out_cubic`, `ease_in_out_cubic`,
/// `ease_out_spring`. A physical spring is written
/// `spring(damping_ratio, stiffness, epsilon)`, or just `spring`.
#[derive(Clone, Debug, PartialEq)]
pub enum EasingFunction {
  /// CSS cubic bezier curve: `cubic_bezier(x1, y1, x2, y2)`.
  ///
  /// Control points `(x1, y1)` and `(x2, y2)` define the shape between
  /// the implicit anchors `(0, 0)` and `(1, 1)`. `x1` and `x2` must be
  /// in `[0, 1]`; `y1` and `y2` may exceed that range to produce
  /// overshoot.
  CubicBezier(f32, f32, f32, f32),
  /// Exponentially-decaying spring. Overshoots past 1.0 and oscillates
  /// before settling. Runs to full wall-clock duration to preserve the
  /// bounce.
  EaseOutSpring,
  /// A damped spring pulled to the target. Lasts as long as it takes to
  /// settle, whatever the configured duration.
  Spring(SpringParams),
}

/// A damped spring from 0 to 1, of unit mass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringParams {
  /// 1 settles as fast as possible without overshooting; less bounces,
  /// more creeps in.
  pub damping_ratio: f32,

  pub stiffness: f32,

  /// How close to the target, as a fraction of the way, counts as there.
  pub epsilon: f32,

  /// Speed at the start, in progress per second: nonzero for a movement
  /// that takes over from one still under way.
  pub initial_velocity: f32,
}

impl Default for SpringParams {
  fn default() -> Self {
    Self {
      damping_ratio: 1.,
      stiffness: 800.,
      epsilon: 0.0001,
      initial_velocity: 0.,
    }
  }
}

/// Longest a spring is let run, for ones that never settle.
const MAX_SPRING_SECS: f64 = 10.;

impl SpringParams {
  /// Decay rate and natural frequency.
  fn rates(&self) -> (f64, f64) {
    let stiffness = f64::from(self.stiffness.max(0.));
    let omega = stiffness.sqrt();
    (f64::from(self.damping_ratio.max(0.)) * omega, omega)
  }

  /// Progress `secs` in, from 0 toward 1.
  #[must_use]
  #[allow(clippy::cast_possible_truncation)]
  pub fn value_at(&self, secs: f32) -> f32 {
    let t = f64::from(secs.max(0.));
    let (beta, omega) = self.rates();
    let (x0, v0) = (-1., f64::from(self.initial_velocity));
    let envelope = (-beta * t).exp();

    let offset = if (beta - omega).abs() <= 1e-6 * omega.max(1.) {
      envelope * (x0 + (beta * x0 + v0) * t)
    } else if beta < omega {
      let omega1 = (omega * omega - beta * beta).sqrt();
      envelope
        * (x0 * (omega1 * t).cos()
          + (beta * x0 + v0) / omega1 * (omega1 * t).sin())
    } else {
      // As two decaying exponentials, which unlike the hyperbolic form
      // can't overflow.
      let omega2 = (beta * beta - omega * omega).sqrt();
      let (slow, fast) = (omega2 - beta, -omega2 - beta);
      let c1 = (v0 - fast * x0) / (slow - fast);
      c1 * (slow * t).exp() + (x0 - c1) * (fast * t).exp()
    };

    (1. + offset) as f32
  }

  /// How long it takes to come within `epsilon` of 1 for good.
  #[must_use]
  pub fn duration(&self) -> std::time::Duration {
    let (beta, omega) = self.rates();
    let (x0, v0) = (-1_f64, f64::from(self.initial_velocity));
    let epsilon = f64::from(self.epsilon.max(1e-9));

    let secs = if beta <= 0. {
      MAX_SPRING_SECS
    } else if beta <= omega {
      // The offset is at most the decaying envelope times
      // `|x0| + |beta * x0 + v0| * min(t, 1 / omega1)`, whose root is
      // found in a few steps since it grows only logarithmically.
      let drift = (beta * x0 + v0).abs();
      let reach = if beta < omega {
        1. / (omega * omega - beta * beta).sqrt()
      } else {
        f64::INFINITY
      };

      (0..8).fold((1. / epsilon).ln() / beta, |secs, _| {
        ((x0.abs() + drift * secs.clamp(0., reach)) / epsilon).ln() / beta
      })
    } else {
      let omega2 = (beta * beta - omega * omega).sqrt();
      let (slow, fast) = (omega2 - beta, -omega2 - beta);
      let c1 = (v0 - fast * x0) / (slow - fast);
      let amplitude = (c1.abs() + (x0 - c1).abs()).max(1.);
      (amplitude / epsilon).ln() / -slow
    };

    // `Duration` panics on NaN, which a spring's odd parameters could
    // still give.
    std::time::Duration::try_from_secs_f64(secs.clamp(0., MAX_SPRING_SECS))
      .unwrap_or(std::time::Duration::from_secs_f64(MAX_SPRING_SECS))
  }

  /// Whether it goes past 1 before settling.
  #[must_use]
  pub fn can_overshoot(&self) -> bool {
    let (beta, omega) = self.rates();
    let v0 = f64::from(self.initial_velocity);

    // Overdamped and critically damped springs cross over only when they
    // start out faster than their quickest decay.
    beta < omega * (1. - 1e-6)
      || v0 > beta + (beta * beta - omega * omega).max(0.).sqrt()
  }
}

impl Default for EasingFunction {
  fn default() -> Self {
    EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0) // ease_in_out
  }
}

impl Eq for EasingFunction {}

impl EasingFunction {
  /// Returns `true` when this function can produce values outside `[0,
  /// 1]`.
  ///
  /// Non-overshooting functions are cut off at 99% eased progress to avoid
  /// the "stuck at destination" look. Overshooting ones run to full
  /// wall-clock duration to preserve their bounce.
  #[must_use]
  pub fn can_overshoot(&self) -> bool {
    match self {
      EasingFunction::EaseOutSpring => true,
      EasingFunction::CubicBezier(_, y1, _, y2) => {
        *y1 < 0.0 || *y1 > 1.0 || *y2 < 0.0 || *y2 > 1.0
      }
      EasingFunction::Spring(spring) => spring.can_overshoot(),
    }
  }

  /// How long an animation with this easing lasts, given the configured
  /// `duration_ms`: a spring takes as long as it needs to settle, unless
  /// that is 0, which makes any animation instant.
  #[must_use]
  pub fn duration_ms(&self, duration_ms: u32) -> u32 {
    match self {
      EasingFunction::Spring(spring) if duration_ms > 0 => {
        u32::try_from(spring.duration().as_millis()).unwrap_or(u32::MAX)
      }
      _ => duration_ms,
    }
  }
}

impl<'de> Deserialize<'de> for EasingFunction {
  fn deserialize<D: serde::Deserializer<'de>>(
    d: D,
  ) -> Result<Self, D::Error> {
    let s = String::deserialize(d)?;
    // Named aliases expand to their CSS cubic-bezier control points.
    match s.as_str() {
      "linear" => Ok(EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0)),
      "ease_in_out" => {
        Ok(EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0))
      }
      "ease_in" => Ok(EasingFunction::CubicBezier(0.42, 0.0, 1.0, 1.0)),
      "ease_out" => Ok(EasingFunction::CubicBezier(0.0, 0.0, 0.58, 1.0)),
      "ease_in_out_cubic" => {
        Ok(EasingFunction::CubicBezier(0.65, 0.0, 0.35, 1.0))
      }
      "ease_in_cubic" => {
        Ok(EasingFunction::CubicBezier(0.32, 0.0, 0.67, 0.0))
      }
      "ease_out_cubic" => {
        Ok(EasingFunction::CubicBezier(0.33, 1.0, 0.68, 1.0))
      }
      "ease_out_spring" => Ok(EasingFunction::EaseOutSpring),
      "spring" => Ok(EasingFunction::Spring(SpringParams::default())),
      s if s.starts_with("spring(") => parse_spring(s)
        .map(EasingFunction::Spring)
        .map_err(serde::de::Error::custom),
      s => {
        if let Some(inner) = s
          .strip_prefix("cubic_bezier(")
          .and_then(|s| s.strip_suffix(')'))
        {
          let parts: Vec<&str> = inner.split(',').collect();
          if parts.len() != 4 {
            return Err(serde::de::Error::custom(
              "cubic_bezier requires exactly 4 arguments: \
               cubic_bezier(x1, y1, x2, y2)",
            ));
          }
          let mut floats = [0f32; 4];
          for (i, part) in parts.iter().enumerate() {
            floats[i] = part.trim().parse::<f32>().map_err(|_| {
              serde::de::Error::custom(format!(
                "cubic_bezier argument {} is not a valid number: {}",
                i + 1,
                part.trim()
              ))
            })?;
          }
          let [x1, y1, x2, y2] = floats;
          if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
            return Err(serde::de::Error::custom(
              "cubic_bezier x1 and x2 must be in [0, 1]",
            ));
          }
          Ok(EasingFunction::CubicBezier(x1, y1, x2, y2))
        } else {
          Err(serde::de::Error::custom(format!(
            "unknown easing function '{s}'; valid values: linear, \
             ease_in, ease_out, ease_in_out, ease_in_cubic, \
             ease_out_cubic, ease_in_out_cubic, ease_out_spring, \
             cubic_bezier(x1, y1, x2, y2), spring, \
             spring(damping_ratio, stiffness, epsilon)"
          )))
        }
      }
    }
  }
}

impl Serialize for EasingFunction {
  // Exact comparisons on purpose: only control points that are exactly a
  // named alias's are written back as that alias.
  #[allow(clippy::float_cmp)]
  fn serialize<S: serde::Serializer>(
    &self,
    s: S,
  ) -> Result<S::Ok, S::Error> {
    match self {
      EasingFunction::EaseOutSpring => s.serialize_str("ease_out_spring"),
      EasingFunction::Spring(spring) => s.serialize_str(&format!(
        "spring({}, {}, {})",
        spring.damping_ratio, spring.stiffness, spring.epsilon
      )),
      EasingFunction::CubicBezier(x1, y1, x2, y2) => {
        // Serialize back to a named alias when the control points match
        // exactly, so round-tripped configs stay human-readable.
        let repr = if *x1 == 0.0 && *y1 == 0.0 && *x2 == 1.0 && *y2 == 1.0
        {
          "linear".to_string()
        } else if *x1 == 0.42 && *y1 == 0.0 && *x2 == 0.58 && *y2 == 1.0 {
          "ease_in_out".to_string()
        } else if *x1 == 0.42 && *y1 == 0.0 && *x2 == 1.0 && *y2 == 1.0 {
          "ease_in".to_string()
        } else if *x1 == 0.0 && *y1 == 0.0 && *x2 == 0.58 && *y2 == 1.0 {
          "ease_out".to_string()
        } else if *x1 == 0.65 && *y1 == 0.0 && *x2 == 0.35 && *y2 == 1.0 {
          "ease_in_out_cubic".to_string()
        } else if *x1 == 0.32 && *y1 == 0.0 && *x2 == 0.67 && *y2 == 0.0 {
          "ease_in_cubic".to_string()
        } else if *x1 == 0.33 && *y1 == 1.0 && *x2 == 0.68 && *y2 == 1.0 {
          "ease_out_cubic".to_string()
        } else {
          format!("cubic_bezier({x1}, {y1}, {x2}, {y2})")
        };
        s.serialize_str(&repr)
      }
    }
  }
}

/// Parses `spring(damping_ratio, stiffness, epsilon)`, whose arguments may
/// also be named (e.g. `spring(stiffness=600)`), the rest defaulting.
fn parse_spring(value: &str) -> Result<SpringParams, String> {
  let inner = value
    .strip_prefix("spring(")
    .and_then(|rest| rest.strip_suffix(')'))
    .ok_or_else(|| format!("invalid spring '{value}'"))?;

  let mut spring = SpringParams::default();
  let names = ["damping_ratio", "stiffness", "epsilon"];

  for (index, argument) in inner
    .split(',')
    .map(str::trim)
    .filter(|argument| !argument.is_empty())
    .enumerate()
  {
    let (name, number) = match argument.split_once('=') {
      Some((name, number)) => (name.trim(), number.trim()),
      None => (
        *names
          .get(index)
          .ok_or_else(|| "spring takes at most 3 arguments".to_string())?,
        argument,
      ),
    };

    let number = number
      .parse::<f32>()
      .ok()
      .filter(|number| number.is_finite() && *number >= 0.)
      .ok_or_else(|| {
        format!("spring argument '{name}' is not a valid number")
      })?;

    match name {
      "damping_ratio" => spring.damping_ratio = number,
      "stiffness" if number > 0. => spring.stiffness = number,
      "stiffness" => {
        return Err("spring stiffness must be above 0".into())
      }
      // A fraction of the way, not pixels.
      "epsilon" if number > 0. && number < 1. => spring.epsilon = number,
      "epsilon" => {
        return Err("spring epsilon must be between 0 and 1".into());
      }
      _ => return Err(format!("unknown spring argument '{name}'")),
    }
  }

  Ok(spring)
}

/// Helper function for setting a default value for a boolean field.
const fn default_bool<const V: bool>() -> bool {
  V
}

/// Helper function for setting a default value for window rule events.
fn default_window_rule_on() -> Vec<WindowRuleEvent> {
  vec![WindowRuleEvent::Manage, WindowRuleEvent::TitleChange]
}

/// Helper function for serializing a vector of keybindings.
///
/// Returns a vector of strings (e.g. `["cmd+shift+a", "ctrl+shift+b"]`).
fn serialize_bindings<S>(
  bindings: &[Keybinding],
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let binding_strings: Vec<String> = bindings
    .iter()
    .map(|binding| {
      binding
        .keys()
        .iter()
        .map(|key| key.to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join("+")
    })
    .collect();

  binding_strings.serialize(serializer)
}

/// Helper function for deserializing a vector of strings into keybindings.
///
/// Returns a vector of [`Keybinding`].
fn deserialize_bindings<'de, D>(
  deserializer: D,
) -> Result<Vec<Keybinding>, D::Error>
where
  D: serde::de::Deserializer<'de>,
{
  let s: Vec<&str> = serde::de::Deserialize::deserialize(deserializer)?;
  s.iter()
    .map(|keybinding_str| {
      let keys: Vec<Key> = keybinding_str
        .split('+')
        .map(|key| {
          key.trim().parse().or_else(|_| Key::try_from_literal(key))
        })
        .collect::<Result<Vec<Key>, _>>()
        .map_err(serde::de::Error::custom)?;

      Keybinding::new(keys).map_err(serde::de::Error::custom)
    })
    .collect()
}

/// Helper function for deserializing [`HideMethod`].
///
/// On macOS, [`HideMethod::Hide`] and [`HideMethod::Cloak`] are not valid
/// and are automatically converted to [`HideMethod::PlaceInCorner`].
fn deserialize_hide_method<'de, D>(
  deserializer: D,
) -> Result<HideMethod, D::Error>
where
  D: serde::de::Deserializer<'de>,
{
  // LINT: The deserialized value is ignored on macOS, but we still want
  // to produce an error for invalid values.
  #[allow(unused_variables)]
  let method = HideMethod::deserialize(deserializer)?;

  #[cfg(target_os = "macos")]
  {
    Ok(HideMethod::PlaceInCorner)
  }

  #[cfg(not(target_os = "macos"))]
  {
    Ok(method)
  }
}

#[cfg(test)]
mod tests {
  use super::{
    BackdropEffectConfig, Color, EasingFunction, OverviewConfig,
    OverviewLayout, ScrollingLayoutConfig, SpringParams,
  };

  fn easing(value: &str) -> Result<EasingFunction, serde_json::Error> {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
  }

  #[test]
  fn springs_parse_by_position_name_or_default() {
    let EasingFunction::Spring(spring) =
      easing("spring(0.8, 600, 0.001)").unwrap()
    else {
      panic!("not a spring");
    };
    assert_eq!(
      (spring.damping_ratio, spring.stiffness, spring.epsilon),
      (0.8, 600., 0.001)
    );

    let EasingFunction::Spring(spring) =
      easing("spring(stiffness=500)").unwrap()
    else {
      panic!("not a spring");
    };
    assert_eq!(spring.stiffness, 500.);
    assert_eq!(spring.damping_ratio, 1.);

    assert_eq!(
      easing("spring").unwrap(),
      EasingFunction::Spring(SpringParams::default())
    );
    assert!(easing("spring(1, 0)").is_err());
    assert!(easing("spring(1, 2, 3, 4)").is_err());
    assert!(easing("spring(bounce=1)").is_err());
    assert!(easing("spring(1, 800, 1.06)").is_err());
    assert!(easing("spring(epsilon=0)").is_err());
  }

  #[test]
  fn a_spring_with_an_odd_epsilon_still_has_a_duration() {
    for (damping_ratio, epsilon, initial_velocity) in [
      (1., 1.06, 0.),
      (0.8, 1.5, -10.),
      (0.8, 2., 8.),
      (0.8, 2., 35.),
    ] {
      let spring = SpringParams {
        damping_ratio,
        epsilon,
        initial_velocity,
        ..SpringParams::default()
      };
      assert!(spring.duration().as_secs_f64() <= 10.);
    }
  }

  #[test]
  fn a_spring_runs_from_zero_to_one_and_settles() {
    let spring = SpringParams::default();
    let duration = spring.duration().as_secs_f32();

    assert!(spring.value_at(0.).abs() < 1e-6);
    assert!((spring.value_at(duration) - 1.).abs() < 1e-3);
    assert!(!spring.can_overshoot());

    // Critically damped from rest, it never goes past the target.
    let mut previous = 0.;
    for step in 1..=100 {
      #[allow(clippy::cast_precision_loss)]
      let value = spring.value_at(duration * step as f32 / 100.);
      assert!(value >= previous && value <= 1.);
      previous = value;
    }
  }

  #[test]
  fn bouncy_and_fast_starting_springs_overshoot() {
    let bouncy = SpringParams {
      damping_ratio: 0.5,
      ..SpringParams::default()
    };
    assert!(bouncy.can_overshoot());
    let peak = (1..200)
      .map(|ms| {
        #[allow(clippy::cast_precision_loss)]
        bouncy.value_at(ms as f32 / 1000.)
      })
      .fold(0_f32, f32::max);
    assert!(peak > 1.);

    let fast = SpringParams {
      initial_velocity: 100.,
      ..SpringParams::default()
    };
    assert!(fast.can_overshoot());

    let overdamped = SpringParams {
      damping_ratio: 3.,
      ..SpringParams::default()
    };
    assert!(!overdamped.can_overshoot());
    let duration = overdamped.duration().as_secs_f32();
    assert!((overdamped.value_at(duration) - 1.).abs() < 1e-3);
  }

  #[test]
  fn springs_last_until_settled_whatever_the_duration() {
    let spring = EasingFunction::Spring(SpringParams::default());
    let duration = spring.duration_ms(150);
    assert!((300..500).contains(&duration), "{duration}");
    assert_eq!(spring.duration_ms(0), 0, "still instant");

    let bezier = EasingFunction::CubicBezier(0.4, 0., 0.2, 1.);
    assert_eq!(bezier.duration_ms(150), 150);
  }

  #[test]
  fn scrolling_struts_take_either_side_alone() {
    let config: ScrollingLayoutConfig = serde_json::from_value(
      serde_json::json!({ "struts": { "right": "5%" } }),
    )
    .unwrap();

    assert_eq!(config.struts.left.to_px(1000, None), 0);
    assert_eq!(config.struts.right.to_px(1000, None), 50);
    assert_eq!(ScrollingLayoutConfig::default().struts.right.amount, 0.);
  }

  #[test]
  fn overview_layouts_read_their_names() {
    let layout = |name: &str| {
      serde_json::from_value::<OverviewConfig>(
        serde_json::json!({ "layout": name }),
      )
      .map(|config| config.layout)
      .ok()
    };

    assert_eq!(layout("activities"), Some(OverviewLayout::Activities));
    assert_eq!(layout("stage"), Some(OverviewLayout::Stage));
    assert_eq!(layout("nope"), None);
    assert_eq!(
      OverviewConfig::default().layout,
      OverviewLayout::Activities
    );
  }

  /// `to_overlay_params` must map every field through to the resulting
  /// `BackdropOverlayParams` unchanged (plus the two parameters passed in
  /// separately, since neither is stored on `BackdropEffectConfig`
  /// itself), since every `BackdropOverlayParams` call site now depends on
  /// this method rather than hand-writing the struct literal.
  #[test]
  fn to_overlay_params_maps_all_fields() {
    let config = BackdropEffectConfig {
      blur_amount: 42.0,
      opacity: 0.5,
      saturation: 1.5,
      exposure: -0.5,
      contrast: 0.25,
      highlights: -0.6,
      shadows: 0.3,
      vignette: 0.4,
      grain: 0.1,
      parallax: 0.8,
      ..BackdropEffectConfig::default()
    };

    let tint = Color::from_abgr(0xAABB_CCDD);
    let params = config.to_overlay_params(tint, 12.0);

    assert_eq!(params.tint, tint);
    assert_eq!(params.blur_amount, 42.0);
    assert_eq!(params.corner_radius, 12.0);
    assert_eq!(params.opacity, 0.5);
    assert_eq!(params.saturation, 1.5);
    assert_eq!(params.exposure, -0.5);
    assert_eq!(params.contrast, 0.25);
    assert_eq!(params.highlights, -0.6);
    assert_eq!(params.shadows, 0.3);
    assert_eq!(params.vignette, 0.4);
    assert_eq!(params.grain, 0.1);
    assert_eq!(params.parallax, 0.8);
  }

  /// `overlay_tint` doubles as the "does this window get a
  /// `NativeBackdropOverlay`?" predicate, so it must resolve a tint
  /// whenever the effect is enabled and none at all while it is disabled.
  #[test]
  fn overlay_tint_tracks_enabled() {
    let enabled = BackdropEffectConfig {
      enabled: true,
      ..BackdropEffectConfig::default()
    };
    assert!(enabled.overlay_tint().is_some());

    let disabled = BackdropEffectConfig {
      enabled: false,
      ..BackdropEffectConfig::default()
    };
    assert!(disabled.overlay_tint().is_none());
  }

  /// With no explicit tint, the fallback is near-transparent black.
  #[test]
  fn overlay_tint_defaults_to_near_transparent_black() {
    let config = BackdropEffectConfig {
      enabled: true,
      tint: None,
      ..BackdropEffectConfig::default()
    };

    assert_eq!(
      config.overlay_tint(),
      Some(Color {
        r: 0,
        g: 0,
        b: 0,
        a: 1
      })
    );
  }
}

#[cfg(test)]
mod corner_radius_tests {
  use wm_platform::{CornerStyle, LengthValue};

  use super::{CornerEffectConfig, WindowEffectConfig};

  fn with_radius(px: i32) -> WindowEffectConfig {
    WindowEffectConfig {
      corner_radius: Some(LengthValue::from_px(px)),
      ..WindowEffectConfig::default()
    }
  }

  /// `corner_radius` snaps the *window* to the nearest preset Windows
  /// exposes, since `DWMWA_WINDOW_CORNER_PREFERENCE` takes no free radius.
  #[test]
  fn radius_snaps_window_to_nearest_preset() {
    assert_eq!(
      with_radius(0).resolved_corner_style(),
      CornerStyle::Square
    );
    assert_eq!(
      with_radius(2).resolved_corner_style(),
      CornerStyle::SmallRounded
    );
    assert_eq!(
      with_radius(40).resolved_corner_style(),
      CornerStyle::Rounded
    );
  }

  /// Overlays follow the configured radius exactly rather than the snapped
  /// preset's, so a ring can be rounder than the presets allow.
  #[test]
  fn overlays_follow_exact_radius() {
    assert!(
      (with_radius(40).window_corner_radius_px() - 40.0).abs() < 1e-3
    );
    assert!((with_radius(0).window_corner_radius_px() - 0.0).abs() < 1e-3);
  }

  /// With `corner_radius` unset the older `corner_style` block still
  /// decides, so existing configs keep working unchanged.
  #[test]
  fn falls_back_to_corner_style_when_unset() {
    let mut config = WindowEffectConfig::default();
    assert_eq!(config.resolved_corner_style(), CornerStyle::Default);

    config.corner_style = CornerEffectConfig {
      enabled: true,
      style: CornerStyle::Square,
    };
    assert_eq!(config.resolved_corner_style(), CornerStyle::Square);
  }
}
