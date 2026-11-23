//! The window overview: a screen per monitor, with a card per workspace
//! of that monitor showing its windows live where they sit, to switch to
//! a workspace or window, or move windows between workspaces.
//!
//! The overview is drawn and driven by `NativeOverview` on a thread of its
//! own; the WM decides what it shows and carries out what is picked.
//!
//! `frame` builds what it shows from the WM's state, `actions` carries
//! out what is picked in it, and `hot_corner` opens it from a corner of
//! a monitor.

mod actions;
mod frame;
mod hot_corner;

use std::time::Instant;

use anyhow::Context;
use frame::overview_frame;
use hot_corner::CornerVisit;
pub use hot_corner::{check_hot_corner, process_hot_corner};
use tokio::sync::mpsc;
use wm_common::{
  InvokeCommand, InvokeFocusCommand, InvokeMoveCommand, OverviewLayout,
};
use wm_platform::{
  Color, NativeOverview, OverviewAction, OverviewInput, OverviewLayoutMode,
};

use crate::{user_config::UserConfig, wm_state::WmState};

/// The WM's side of the overview.
pub struct Overview {
  /// Created on first open.
  native: Option<NativeOverview>,

  /// What is shown, while open.
  open: Option<OpenOverview>,

  /// What was shown, from its close until the next sync hides it.
  closing: Option<OpenOverview>,

  /// Whether the native overview is on screen: from its open until it
  /// says it hid, which can be a zoom after its close.
  is_shown: bool,

  /// The cursor's visit to the hot corner, which opens the overview once
  /// per visit.
  hot_corner: CornerVisit,

  /// Where the native overview sends user actions, with their session.
  action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
}

/// Colors of an open overview, resolved once per open, since they can be
/// read from a file.
struct OpenOverview {
  accent: Color,

  /// Around the `framed` backdrop's panel.
  frame_color: Color,
}

impl Overview {
  pub fn new(
    action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
  ) -> Self {
    Self {
      native: None,
      open: None,
      closing: None,
      is_shown: false,
      hot_corner: CornerVisit::Away,
      action_tx,
    }
  }

  pub fn is_open(&self) -> bool {
    self.open.is_some()
  }

  /// Whether the overview is on screen, which it still is for a moment
  /// after its close, zooming into the pick.
  pub fn is_shown(&self) -> bool {
    self.is_shown
  }

  /// When the cursor resting in the hot corner opens the overview, while
  /// it does.
  pub fn hot_corner_deadline(&self) -> Option<Instant> {
    match self.hot_corner {
      CornerVisit::Resting(due) => Some(due),
      CornerVisit::Away | CornerVisit::Spent => None,
    }
  }

  /// Takes in that the native overview of `session` hid. Returns whether
  /// that was the one on screen.
  pub fn mark_hidden(&mut self, session: u64) -> bool {
    let is_latest = self
      .native
      .as_ref()
      .is_some_and(|native| native.session() == session);

    is_latest && std::mem::take(&mut self.is_shown)
  }

  /// Session of the open overview, which its actions carry.
  pub fn open_session(&self) -> Option<u64> {
    self
      .open
      .as_ref()
      .and(self.native.as_ref())
      .map(NativeOverview::session)
  }

  /// Hands the open overview `input` to act on. Returns whether it could.
  ///
  /// If the overview's thread is gone, it is marked closed and hidden, so
  /// keybindings go to the WM again rather than nowhere.
  pub fn input(&mut self, input: OverviewInput) -> bool {
    let is_delivered = self.is_open()
      && self
        .native
        .as_ref()
        .is_some_and(|native| native.input(input));

    if !is_delivered {
      tracing::warn!("The overview took no input; marking it closed.");
      self.close();
      self.is_shown = false;
    }
    is_delivered
  }

  /// Marks the overview closed.
  ///
  /// The native overview stays up until the next [`sync_overview`] (or
  /// until its zoom into the picked workspace ends), so it only hides
  /// once focus has moved on; hiding the foreground window first would
  /// let the OS pick the next one.
  pub fn close(&mut self) {
    self.closing = self.open.take();
  }

  /// The native overview, created on first use.
  fn native(&mut self) -> anyhow::Result<&mut NativeOverview> {
    if self.native.is_none() {
      let action_tx = self.action_tx.clone();
      let on_action = Box::new(move |session, action| {
        if let Err(err) = action_tx.send((session, action)) {
          tracing::warn!("Failed to send overview action: {err}");
        }
      });

      self.native = Some(NativeOverview::create(on_action)?);
    }

    self.native.as_mut().context("No native overview.")
  }
}

/// Where a keybinding pressed while the overview is open goes.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyRoute {
  /// To the WM, which runs its commands as ever.
  Wm,

  /// To the overview, which does this to its selection.
  Overview(OverviewInput),

  /// Nowhere: its commands would act on the desktop behind the
  /// overview, which has nothing like them.
  Dropped,
}

/// Routes a keybinding running `commands` while the overview is open.
///
/// The overview has the keyboard then, so a keybinding acts on it rather
/// than on the workspace behind it: `focus`, `move` and `close` do to its
/// selection what they would do to the focused window, and the rest are
/// dropped. Only commands on the WM itself, the overview's own toggle and
/// launching a program still run.
///
/// Enabling a binding mode is dropped too: its keybindings are often on
/// bare keys (`h`, `escape`), which the hook would then take from the
/// overview's search and its own `Escape`.
pub fn route_keybinding(commands: &[InvokeCommand]) -> KeyRoute {
  let is_the_wms = |command: &InvokeCommand| {
    matches!(
      command,
      InvokeCommand::ToggleOverview { .. }
        | InvokeCommand::ShellExec { .. }
        | InvokeCommand::WmDisableBindingMode { .. }
        | InvokeCommand::WmExit
        | InvokeCommand::WmRedraw
        | InvokeCommand::WmReloadConfig
        | InvokeCommand::WmTogglePause
    )
  };

  if commands.iter().any(is_the_wms) {
    return KeyRoute::Wm;
  }

  // The first that means something there, e.g. the `move` of a binding
  // that moves a window to a workspace and follows it.
  commands
    .iter()
    .find_map(overview_input)
    .map_or(KeyRoute::Dropped, KeyRoute::Overview)
}

/// What `command` does in the overview, if anything.
fn overview_input(command: &InvokeCommand) -> Option<OverviewInput> {
  match command {
    InvokeCommand::Close => Some(OverviewInput::CloseWindow),
    InvokeCommand::Focus(InvokeFocusCommand {
      direction: Some(direction),
      ..
    })
    | InvokeCommand::Move(InvokeMoveCommand {
      direction: Some(direction),
      ..
    }) => Some(OverviewInput::Move(direction.clone())),
    InvokeCommand::Focus(focus) => {
      if let Some(name) = &focus.workspace {
        Some(OverviewInput::FocusWorkspace(name.clone()))
      } else if focus.special_workspace {
        Some(OverviewInput::FocusSpecialWorkspace)
      } else if focus.next_workspace
        || focus.next_active_workspace
        || focus.next_active_workspace_on_monitor
      {
        Some(OverviewInput::NextWorkspace)
      } else if focus.prev_workspace
        || focus.prev_active_workspace
        || focus.prev_active_workspace_on_monitor
      {
        Some(OverviewInput::PreviousWorkspace)
      } else {
        None
      }
    }
    InvokeCommand::Move(to) => {
      if let Some(name) = &to.workspace {
        Some(OverviewInput::MoveToWorkspace(name.clone()))
      } else if to.special_workspace {
        Some(OverviewInput::MoveToSpecialWorkspace)
      } else {
        None
      }
    }
    _ => None,
  }
}

/// Opens the overview on every monitor, laid out as `layout`, with the
/// keyboard on the focused one.
///
/// While open, it switches to `layout` instead, or closes when it already
/// shows that, giving focus back to the focused window.
pub fn toggle_overview(
  state: &mut WmState,
  config: &UserConfig,
  layout: OverviewLayoutMode,
) -> anyhow::Result<()> {
  if state.overview.is_open() {
    if let Some(native) = &state.overview.native {
      native.toggle(layout);
    }
    return Ok(());
  }

  let overview = &config.value.overview;
  let open = OpenOverview {
    accent: overview
      .colors
      .accent
      .as_ref()
      .unwrap_or(&config.value.window_effects.focused_window.border.color)
      .resolve(),
    frame_color: overview.backdrop.frame.color.resolve(),
  };
  let frame = overview_frame(state, &open, config);
  if frame.screens.is_empty() {
    anyhow::bail!("No monitors to show the overview on.");
  }

  state.overview.native()?.open(frame, layout);
  state.overview.open = Some(open);
  state.overview.closing = None;
  state.overview.is_shown = true;
  Ok(())
}

/// The native overview's layout for the config's `layout`.
pub fn to_layout_mode(layout: OverviewLayout) -> OverviewLayoutMode {
  match layout {
    OverviewLayout::Activities => OverviewLayoutMode::Activities,
    OverviewLayout::Stack => OverviewLayoutMode::Stack,
    OverviewLayout::Expose => OverviewLayoutMode::Expose,
  }
}

/// Brings the native overview in line with the WM's state: hides it once
/// closed, and shows workspaces and windows as they change while open.
///
/// Called at the end of a platform sync, after focus has been synced.
pub fn sync_overview(state: &mut WmState, config: &UserConfig) {
  let is_open = state.overview.open.is_some();
  let shown = state
    .overview
    .open
    .as_ref()
    .or(state.overview.closing.as_ref());

  // `None` once hidden, or once every monitor is gone.
  let frame = shown
    .map(|shown| overview_frame(state, shown, config))
    .filter(|frame| !frame.screens.is_empty());

  let overview = &mut state.overview;
  overview.closing = None;

  let Some(native) = &mut overview.native else {
    overview.open = None;
    return;
  };

  match frame {
    Some(frame) if is_open => native.update(frame),
    // Its zoom into the picked workspace lands on what the pick brought
    // on screen.
    Some(frame) => {
      tracing::debug!("Hiding the overview on the pick's frame.");
      native.hide_on(frame);
    }
    None => {
      overview.open = None;
      native.hide();
    }
  }
}

#[cfg(test)]
mod tests {
  use clap::Parser;
  use wm_common::InvokeCommand;
  use wm_platform::{Direction, OverviewInput};

  use super::{route_keybinding, KeyRoute};

  /// Where a keybinding running `commands`, as written in a config, goes
  /// while the overview is open.
  fn route(commands: &[&str]) -> KeyRoute {
    let commands = commands
      .iter()
      .map(|command| {
        let args = std::iter::once("").chain(command.split_whitespace());
        InvokeCommand::try_parse_from(args).expect("a command")
      })
      .collect::<Vec<_>>();

    route_keybinding(&commands)
  }

  #[test]
  fn direction_keybindings_move_the_overview_selection() {
    for (command, direction) in [
      ("focus --direction left", Direction::Left),
      ("focus --direction down", Direction::Down),
      ("move --direction up", Direction::Up),
      ("move --direction right", Direction::Right),
    ] {
      assert_eq!(
        route(&[command]),
        KeyRoute::Overview(OverviewInput::Move(direction)),
        "{command}"
      );
    }
  }

  #[test]
  fn workspace_keybindings_act_on_the_overview_selection() {
    let named = |name: &str| name.to_string();

    for (commands, input) in [
      (
        vec!["focus --workspace 3"],
        OverviewInput::FocusWorkspace(named("3")),
      ),
      // Moving a window and following it only moves it there.
      (
        vec!["move --workspace 3", "focus --workspace 3"],
        OverviewInput::MoveToWorkspace(named("3")),
      ),
      (
        vec!["focus --special-workspace"],
        OverviewInput::FocusSpecialWorkspace,
      ),
      (
        vec!["move --special-workspace"],
        OverviewInput::MoveToSpecialWorkspace,
      ),
      (
        vec!["focus --next-active-workspace"],
        OverviewInput::NextWorkspace,
      ),
      (
        vec!["focus --prev-workspace"],
        OverviewInput::PreviousWorkspace,
      ),
      (vec!["close"], OverviewInput::CloseWindow),
    ] {
      assert_eq!(
        route(&commands),
        KeyRoute::Overview(input),
        "{commands:?}"
      );
    }
  }

  #[test]
  fn keybindings_on_the_desktop_behind_the_overview_are_dropped() {
    for command in [
      "resize --width +2%",
      "toggle-floating",
      "toggle-tiling-direction",
      "focus --recent-workspace",
      "move --next-active-workspace",
      "move-workspace --direction left",
      "center-column",
      "wm-enable-binding-mode --name resize",
    ] {
      assert_eq!(route(&[command]), KeyRoute::Dropped, "{command}");
    }
  }

  #[test]
  fn keybindings_on_the_wm_itself_still_run() {
    for command in [
      "toggle-overview",
      "toggle-overview --layout stack",
      "wm-exit",
      "wm-reload-config",
      "wm-toggle-pause",
      "wm-disable-binding-mode --name resize",
      "shell-exec cmd",
    ] {
      assert_eq!(route(&[command]), KeyRoute::Wm, "{command}");
    }

    // Whatever else they run.
    assert_eq!(
      route(&["focus --direction left", "wm-redraw"]),
      KeyRoute::Wm
    );
  }
}
