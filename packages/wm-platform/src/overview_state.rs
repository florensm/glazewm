//! What the overview is doing, and how keys and the mouse change it, kept
//! free of drawing so it can be tested.
//!
//! The keyboard moves through three steps, one space bar at a time:
//! browsing workspaces, picking a window inside one, and carrying a window
//! to another workspace. The mouse does the same directly: click to go,
//! drag to move.
//!
//! Every typed character goes to the search, so no command is on a
//! letter, digit or symbol: commands are on the keys that type nothing
//! (arrows, `Tab`, `Delete`, ...) and on `Alt` chords, which mirror the
//! sample config's `focus` and `move` keybindings. Keybindings the WM
//! takes before they get here come in as an [`OverviewInput`] instead.
//!
//! Each monitor has a screen of its own, and the keyboard is on one of
//! them at a time. Workspaces are indexed across every screen, screen
//! after screen, so a window can be carried, dragged or searched for
//! anywhere.

use std::{borrow::Cow, ops::Range};

use crate::{
  overview_layout::RectF, Direction, OverviewAction, OverviewInput,
  OverviewLayoutMode, OverviewScreen, OverviewWindow, OverviewWorkspace,
  Point, Rect,
};

/// Where the keyboard is in the overview's flow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
  /// Browsing workspaces.
  #[default]
  Spaces,

  /// Inside the selected workspace, picking a window.
  Windows,

  /// Holding a window, picking a workspace to put it on.
  Carrying,
}

/// A key press the overview acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
  Left,
  Right,
  Up,
  Down,
  Home,
  End,
  Enter,
  Space,
  Backspace,
  Delete,
  Tab,

  /// `Tab` with `Shift` held.
  BackTab,
  Escape,
  F1,

  /// A typed character other than whitespace.
  Text(char),

  /// A letter, digit or `-` pressed with `Alt` held, in lower case.
  Alt(char),

  /// The same with `Shift` held too.
  AltShift(char),
}

impl Key {
  /// How the shortcut sheet shows it.
  fn label(self) -> String {
    match self {
      Self::Left => "\u{2190}".into(),
      Self::Up => "\u{2191}".into(),
      Self::Down => "\u{2193}".into(),
      Self::Right => "\u{2192}".into(),
      Self::Home => "Home".into(),
      Self::End => "End".into(),
      Self::Enter => "\u{23ce}".into(),
      Self::Space => "Space".into(),
      Self::Backspace => "\u{232b}".into(),
      Self::Delete => "Del".into(),
      Self::Tab => "Tab".into(),
      Self::BackTab => "\u{21e7}Tab".into(),
      Self::Escape => "Esc".into(),
      Self::F1 => "F1".into(),
      Self::Text(text) => text.to_string(),
      Self::Alt(key) => format!("Alt+{}", key.to_ascii_uppercase()),
      Self::AltShift(key) => {
        format!("Alt+\u{21e7}{}", key.to_ascii_uppercase())
      }
    }
  }
}

/// What a key does. None is on a key that types a character, which all
/// go to the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
  Cancel,
  ShowShortcuts,
  CloseWindow,

  /// The next match of a search, or else the next layout.
  Next,

  /// The previous match of a search, or else the previous layout.
  Previous,
  Space,
  Back,
  Move(Toward),
  First,
  Last,
  Enter,

  /// Go to the special workspace.
  Special,

  /// Send the picked or carried window to the special workspace.
  SendToSpecial,

  /// Go to the workspace of a digit.
  Workspace(u8),

  /// Send the picked or carried window to the workspace of a digit.
  SendToWorkspace(u8),
}

/// The keys of every command but those of a digit, which `Alt` and
/// `Alt` + `Shift` with it give. The shortcut sheet shows the first key
/// of each.
const BINDINGS: &[(Command, &[Key])] = &[
  (Command::Cancel, &[Key::Escape]),
  (Command::ShowShortcuts, &[Key::F1]),
  (Command::CloseWindow, &[Key::Delete]),
  (Command::Next, &[Key::Tab]),
  (Command::Previous, &[Key::BackTab]),
  (Command::Space, &[Key::Space]),
  (Command::Back, &[Key::Backspace]),
  (Command::Move(Toward::Left), &[Key::Left, Key::Alt('h')]),
  (Command::Move(Toward::Right), &[Key::Right, Key::Alt('l')]),
  (Command::Move(Toward::Up), &[Key::Up, Key::Alt('k')]),
  (Command::Move(Toward::Down), &[Key::Down, Key::Alt('j')]),
  (Command::First, &[Key::Home]),
  (Command::Last, &[Key::End]),
  (Command::Enter, &[Key::Enter]),
  (Command::Special, &[Key::Alt('-')]),
  (Command::SendToSpecial, &[Key::AltShift('-')]),
];

impl Command {
  fn of(key: Key) -> Option<Self> {
    let digit =
      |key: char| key.to_digit(10).and_then(|d| u8::try_from(d).ok());

    match key {
      Key::Alt(key) if key.is_ascii_digit() => {
        return digit(key).map(Self::Workspace);
      }
      Key::AltShift(key) if key.is_ascii_digit() => {
        return digit(key).map(Self::SendToWorkspace);
      }
      _ => {}
    }

    BINDINGS
      .iter()
      .find(|(_, keys)| keys.contains(&key))
      .map(|(command, _)| *command)
  }

  /// The key the shortcut sheet shows for it.
  fn label(self) -> String {
    match self {
      Self::Workspace(_) => "Alt+1\u{2013}9".into(),
      Self::SendToWorkspace(_) => "Alt+\u{21e7}1\u{2013}9".into(),
      _ => BINDINGS
        .iter()
        .find(|(command, _)| *command == self)
        .and_then(|(_, keys)| keys.first())
        .map_or_else(String::new, |key| key.label()),
    }
  }
}

/// How the shortcut sheet shows the `Alt` chords that do as the arrows.
const ALT_ARROWS: &str = "Alt+H J K L";

/// The labels of `commands`, one after another.
fn joined_labels(commands: impl IntoIterator<Item = Command>) -> String {
  commands.into_iter().map(Command::label).collect()
}

/// A way to move the cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Toward {
  Left,
  Right,
  Up,
  Down,
}

impl From<&Direction> for Toward {
  fn from(direction: &Direction) -> Self {
    match direction {
      Direction::Left => Self::Left,
      Direction::Right => Self::Right,
      Direction::Up => Self::Up,
      Direction::Down => Self::Down,
    }
  }
}

impl Toward {
  /// Every way, in the order the shortcut sheet shows their arrows.
  const ALL: [Self; 4] = [Self::Left, Self::Up, Self::Down, Self::Right];

  fn opposite(self) -> Self {
    match self {
      Self::Left => Self::Right,
      Self::Right => Self::Left,
      Self::Up => Self::Down,
      Self::Down => Self::Up,
    }
  }
}

/// What is under the cursor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Hit {
  pub workspace: Option<usize>,
  pub window: Option<isize>,
  pub control: Option<Control>,

  /// Where in `workspace` it is, in the screen coordinates its windows
  /// sit at, displayed or not.
  pub point: Option<(i32, i32)>,
}

/// A control along the bottom of a screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Control {
  /// The search field, which typing goes to wherever the cursor is.
  Search,

  /// The chip beside it, which shows the keyboard shortcuts.
  Shortcuts,
}

/// Keyboard shortcuts of one step, as the shortcut sheet lists them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShortcutGroup {
  pub title: &'static str,
  pub shortcuts: Vec<Shortcut>,
}

/// A key and what it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Shortcut {
  pub key: String,
  pub action: String,
}

impl Shortcut {
  fn new(key: &str, action: &str) -> Self {
    Self {
      key: key.to_string(),
      action: action.to_string(),
    }
  }
}

/// The search field, as every screen shows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SearchField {
  pub query: String,

  pub hits: usize,

  /// Which of its matches is picked, from 0, if one is.
  pub current: Option<usize>,
}

/// A window being dragged with the mouse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drag {
  pub hwnd: isize,

  /// Where the cursor is, in physical screen coordinates.
  pub position: (f32, f32),

  /// Workspace it would be dropped on.
  pub target: Option<usize>,

  /// Where in `target`, as `Hit::point` has it.
  pub point: Option<(i32, i32)>,
}

/// A window matching the search, and the workspace it is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Match {
  workspace: usize,
  hwnd: isize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Press {
  position: (f32, f32),
  hit: Hit,
}

/// One screen's place in the interaction.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScreenCursor {
  /// Its monitor's ID, which actions picked on it carry.
  pub monitor: u128,

  /// Working area of its monitor, which the arrows go between screens
  /// by.
  pub rect: Rect,

  /// Its workspaces among every screen's.
  pub cards: Range<usize>,

  /// Workspace it shows: the selected one while it has the keyboard.
  pub shown: usize,
}

/// How the picked window was picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickedBy {
  /// Pointed at with the mouse, or the strip panned under it: the stack
  /// leaves its strip where it is rather than panning the pick into view
  /// under the cursor.
  Mouse,

  /// Any other way, e.g. with the arrows or a search.
  Other,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Interaction {
  /// A cursor per screen, in the order the arrows go through them.
  pub screens: Vec<ScreenCursor>,

  /// Screen the keyboard is on.
  pub active: usize,

  pub mode: Mode,
  pub layout: OverviewLayoutMode,

  /// Window picked inside the selected workspace.
  pub selected_window: Option<isize>,

  /// How the picked window was picked.
  picked_by: PickedBy,

  /// Window held while picking a workspace for it.
  pub carried: Option<isize>,

  /// Search query, over every screen's windows: a search is on while
  /// there is one.
  pub query: String,

  /// Windows matching `query`, best first. Found as the query or the
  /// workspaces change, rather than for each key and screen that asks.
  matches: Vec<Match>,

  /// Match a search last went to while carrying, which picks no window
  /// to tell by.
  match_index: Option<usize>,

  /// Whether the sheet of keyboard shortcuts is open.
  pub shows_shortcuts: bool,

  /// Workspace under the cursor.
  pub hover: Option<usize>,

  /// Control under the cursor.
  pub hover_control: Option<Control>,

  press: Option<Press>,
  pub drag: Option<Drag>,

  /// Way the last arrow or wheel step found nowhere to go, until the
  /// screen with the keyboard takes it to bump that way.
  nudge: Option<Toward>,

  /// Cursor travel before a press becomes a drag.
  drag_threshold: f32,

  /// Numbered workspaces without a card that digit keys still reach.
  pub digit_workspaces: Vec<String>,

  /// Where the selected workspace's windows show, first to last, for the
  /// arrows to move between them. Empty until laid out, which leaves
  /// them where they are on screen.
  pub nav: Vec<(isize, RectF)>,
}

impl Interaction {
  /// Starts out on the focused screen, and each screen on its displayed
  /// workspace.
  pub fn new(
    screens: &[OverviewScreen],
    layout: OverviewLayoutMode,
    drag_threshold: f32,
  ) -> Self {
    Self {
      screens: cursors(screens),
      active: screens
        .iter()
        .position(|screen| screen.is_focused)
        .unwrap_or(0),
      mode: Mode::Spaces,
      layout,
      selected_window: None,
      picked_by: PickedBy::Other,
      carried: None,
      query: String::new(),
      matches: Vec::new(),
      match_index: None,
      shows_shortcuts: false,
      hover: None,
      hover_control: None,
      press: None,
      drag: None,
      nudge: None,
      drag_threshold,
      digit_workspaces: Vec::new(),
      nav: Vec::new(),
    }
  }

  /// Index of the selected workspace: the one shown on the screen with the
  /// keyboard.
  pub fn selected(&self) -> usize {
    self
      .screens
      .get(self.active)
      .map_or(0, |screen| screen.shown)
  }

  /// Keeps the selection on the same workspaces and windows when what is
  /// shown changes from `old` to `screens`.
  pub fn sync(
    &mut self,
    old: &[OverviewWorkspace],
    screens: &[OverviewScreen],
  ) {
    let new = all_workspaces(screens);
    let mut cursors = cursors(screens);

    // The same workspace on the same monitor: the special workspace's
    // card is on every screen.
    let remap = |index: usize| {
      let monitor = self
        .screens
        .iter()
        .find(|screen| screen.cards.contains(&index))?
        .monitor;
      let name = &old.get(index)?.name;

      cursors
        .iter()
        .find(|screen| screen.monitor == monitor)?
        .cards
        .clone()
        .find(|index| new.get(*index).is_some_and(|w| w.name == *name))
    };

    let kept = cursors
      .iter()
      .map(|cursor| {
        let old = self
          .screens
          .iter()
          .find(|screen| screen.monitor == cursor.monitor)?;
        remap(old.shown)
      })
      .collect::<Vec<_>>();
    let selected = remap(self.selected());
    let hover = self.hover.and_then(remap);
    let drag_target =
      self.drag.and_then(|drag| drag.target).and_then(remap);

    for (cursor, kept) in cursors.iter_mut().zip(kept) {
      if let Some(shown) = kept {
        cursor.shown = shown;
      }
    }

    let monitor = self.screens.get(self.active).map(|s| s.monitor);
    self.active = selected
      .and_then(|index| {
        cursors
          .iter()
          .position(|screen| screen.cards.contains(&index))
      })
      .or_else(|| {
        cursors
          .iter()
          .position(|screen| Some(screen.monitor) == monitor)
      })
      .or_else(|| screens.iter().position(|screen| screen.is_focused))
      .unwrap_or(0);
    self.screens = cursors;
    self.hover = hover;

    self.matches = find_matches(&self.query, &new);

    let exists = |hwnd: isize| find(&new, hwnd).is_some();

    if self.selected_window.is_some_and(|hwnd| !exists(hwnd)) {
      self.selected_window = None;
    }

    if self.carried.is_some_and(|hwnd| !exists(hwnd)) {
      self.carried = None;
      self.mode = Mode::Spaces;
    }

    if let Some(drag) = &mut self.drag {
      if exists(drag.hwnd) {
        drag.target = drag_target;
      } else {
        self.drag = None;
        self.press = None;
      }
    }
  }

  /// Window lifted by the mouse or the keyboard, if any.
  pub fn floating(&self) -> Option<isize> {
    self.drag.map(|drag| drag.hwnd).or(self.carried)
  }

  /// Workspace the floating window would land on.
  fn drop_target(&self) -> Option<usize> {
    match &self.drag {
      Some(drag) => drag.target,
      None => (self.mode == Mode::Carrying).then(|| self.selected()),
    }
  }

  /// Moves the keyboard to `screen`, onto the workspace it shows. Returns
  /// whether it moved.
  pub fn activate(&mut self, screen: usize) -> bool {
    let shown = self
      .screens
      .get(screen)
      .filter(|cursor| {
        screen != self.active && cursor.cards.contains(&cursor.shown)
      })
      .map(|cursor| cursor.shown);

    if let Some(shown) = shown {
      self.select(shown);
    }
    shown.is_some()
  }

  /// Shows workspace `index` on its screen, leaving the keyboard where it
  /// is: e.g. the one the overview zooms into there as it closes.
  pub fn show(&mut self, index: usize) {
    match self.screen_of(index) {
      Some(screen) if screen == self.active => {
        self.select(index);
      }
      Some(screen) => self.set_shown(screen, index),
      None => {}
    }
  }

  /// Acts on `key`: a typed character goes to the search, and any other
  /// key does its command.
  pub fn key(
    &mut self,
    key: Key,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    self.picked_by = PickedBy::Other;
    let command = Command::of(key);

    // Any key closes the shortcut sheet; its own and `Escape` do nothing
    // more.
    if std::mem::take(&mut self.shows_shortcuts)
      && matches!(command, Some(Command::ShowShortcuts | Command::Cancel))
    {
      return None;
    }

    match key {
      Key::Text(text) => {
        self.type_text(text, workspaces);
        None
      }
      // A space only ever comes between the words of a query.
      Key::Space if self.is_searching() => {
        self.type_text(' ', workspaces);
        None
      }
      _ => self.run(command?, workspaces),
    }
  }

  /// Acts on `input`, a keybinding of the WM's pressed while the overview
  /// has the keyboard, as on the key of its own that does the same.
  pub fn input(
    &mut self,
    input: &OverviewInput,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    self.picked_by = PickedBy::Other;
    self.shows_shortcuts = false;

    let command = match input {
      OverviewInput::Move(direction) => Command::Move(direction.into()),
      OverviewInput::FocusWorkspace(name) => {
        let monitor = self.monitor_for(name, workspaces);
        return self.go_to(name.clone(), monitor, workspaces);
      }
      OverviewInput::MoveToWorkspace(name) => {
        let monitor = self.monitor_for(name, workspaces);
        return self.send_to(name.clone(), monitor, workspaces);
      }
      OverviewInput::FocusSpecialWorkspace => Command::Special,
      OverviewInput::MoveToSpecialWorkspace => Command::SendToSpecial,
      OverviewInput::NextWorkspace | OverviewInput::PreviousWorkspace => {
        self.wheel(*input == OverviewInput::NextWorkspace, workspaces);
        return None;
      }
      OverviewInput::CloseWindow => Command::CloseWindow,
    };

    self.run(command, workspaces)
  }

  /// Carries out `command`. During a search, the keys that move the
  /// cursor go through its matches instead, and those that step back
  /// take the query back first.
  fn run(
    &mut self,
    command: Command,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    if self.is_searching() && self.run_in_search(command, workspaces) {
      return None;
    }

    match command {
      Command::Cancel => return Some(OverviewAction::Cancel),
      Command::ShowShortcuts => self.shows_shortcuts = true,
      Command::CloseWindow => {
        let hwnd =
          self.selected_window.filter(|_| self.mode == Mode::Windows);
        if let Some(hwnd) = hwnd {
          self.selected_window = None;
          return Some(OverviewAction::CloseWindow(hwnd));
        }
      }
      Command::Next => self.layout = self.layout.next(),
      Command::Previous => self.layout = self.layout.previous(),
      Command::Space => return self.space(workspaces),
      Command::Back => match self.mode {
        Mode::Carrying => {
          self.carried = None;
          self.mode = Mode::Windows;
        }
        Mode::Windows => {
          self.mode = Mode::Spaces;
          self.selected_window = None;
        }
        Mode::Spaces => {}
      },
      Command::Move(toward) => self.arrow(toward, workspaces),
      Command::First => self.edge(false, workspaces),
      Command::Last => self.edge(true, workspaces),
      Command::Enter => return self.enter(workspaces),
      Command::Workspace(digit) => {
        let (name, monitor) = self.digit_workspace(digit, workspaces)?;
        return self.go_to(name, monitor, workspaces);
      }
      Command::SendToWorkspace(digit) => {
        let (name, monitor) = self.digit_workspace(digit, workspaces)?;
        return self.send_to(name, monitor, workspaces);
      }
      Command::Special => {
        let (name, monitor) = self.special_workspace(workspaces)?;
        return self.go_to(name, monitor, workspaces);
      }
      Command::SendToSpecial => {
        let (name, monitor) = self.special_workspace(workspaces)?;
        return self.send_to(name, monitor, workspaces);
      }
    }

    None
  }

  /// Does what `command` does to a search, if that is something else
  /// during one. Returns whether it was.
  fn run_in_search(
    &mut self,
    command: Command,
    workspaces: &[OverviewWorkspace],
  ) -> bool {
    match command {
      // The pick stays on the match gone to.
      Command::Cancel => self.end_search(),
      Command::Back => {
        self.query.pop();
        self.follow_search(workspaces);
      }
      Command::Next | Command::Move(Toward::Down | Toward::Right) => {
        self.goto_match(1);
      }
      Command::Previous | Command::Move(Toward::Up | Toward::Left) => {
        self.goto_match(-1);
      }
      Command::First => {
        self.goto_nth_match(0);
      }
      Command::Last => {
        self.goto_nth_match(self.matches.len().saturating_sub(1));
      }
      // Never the workspace the cursor happens to be on.
      Command::Enter if self.matches.is_empty() => {}
      _ => return false,
    }

    true
  }

  /// Name and monitor of the workspace digit key `digit` stands for.
  fn digit_workspace(
    &self,
    digit: u8,
    workspaces: &[OverviewWorkspace],
  ) -> Option<(String, u128)> {
    let name = digit_target(
      digit,
      workspaces,
      self.active_cards(),
      &self.digit_workspaces,
    )?;
    let monitor = self.monitor_for(&name, workspaces);
    Some((name, monitor))
  }

  /// Name and monitor of the special workspace, as carded on the screen
  /// with the keyboard.
  fn special_workspace(
    &self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<(String, u128)> {
    let special = self
      .active_cards()
      .find(|index| is_special(workspaces, *index))?;
    let name = workspaces.get(special)?.name.clone();
    Some((name, self.monitor_of(Some(special))))
  }

  /// Switches to workspace `name`, picked on `monitor`'s screen, or drops
  /// the carried window there.
  fn go_to(
    &mut self,
    name: String,
    monitor: u128,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    if self.mode == Mode::Carrying {
      return self.send_to(name, monitor, workspaces);
    }
    Some(OverviewAction::FocusWorkspace { name, monitor })
  }

  /// Sends the picked window to workspace `name`, picked on `monitor`'s
  /// screen, or drops the carried one there. There is none to send while
  /// browsing workspaces.
  fn send_to(
    &mut self,
    name: String,
    monitor: u128,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    match self.mode {
      Mode::Spaces => None,
      Mode::Windows => {
        let hwnd = self.selected_window?;
        self.send(hwnd, name, monitor, workspaces)
      }
      Mode::Carrying => {
        let action = self
          .carried
          .and_then(|hwnd| move_action(hwnd, name, monitor, workspaces));
        self.put_down();
        action
      }
    }
  }

  /// Whether a search is on, which typed characters and the keys that
  /// move the cursor then go to.
  pub fn is_searching(&self) -> bool {
    !self.query.is_empty()
  }

  /// Types `text` into the search, which goes to its best match.
  fn type_text(&mut self, text: char, workspaces: &[OverviewWorkspace]) {
    self.query.push(text);
    self.follow_search(workspaces);
  }

  /// Finds the matches of the query as it is now and goes to the best,
  /// staying put without one.
  fn follow_search(&mut self, workspaces: &[OverviewWorkspace]) {
    self.matches = find_matches(&self.query, workspaces);
    self.match_index = None;
    self.goto_nth_match(0);
  }

  /// Ends the search, leaving the cursor where it is.
  fn end_search(&mut self) {
    self.query.clear();
    self.matches.clear();
    self.match_index = None;
  }

  /// Looks inside the selected workspace, lifts the selected window, or
  /// drops the carried one.
  fn space(
    &mut self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    match self.mode {
      Mode::Spaces => {
        self.enter_windows(workspaces);
        None
      }
      Mode::Windows => {
        if self
          .selected_window
          .is_some_and(|hwnd| find(workspaces, hwnd).is_some())
        {
          self.carried = self.selected_window;
          self.mode = Mode::Carrying;
        }
        None
      }
      Mode::Carrying => self.drop_carried(workspaces),
    }
  }

  fn enter(
    &mut self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let selected = self.selected();
    let monitor = self.monitor_of(Some(selected));

    match self.mode {
      Mode::Carrying => self.drop_carried(workspaces),
      Mode::Windows if self.selected_window.is_some() => self
        .selected_window
        .map(|hwnd| OverviewAction::FocusWindow { hwnd, monitor }),
      _ => workspaces.get(selected).map(|workspace| {
        OverviewAction::FocusWorkspace {
          name: workspace.name.clone(),
          monitor,
        }
      }),
    }
  }

  fn drop_carried(
    &mut self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let selected = self.selected();
    let action = self.carried.and_then(|hwnd| {
      let to = workspaces.get(selected)?;
      let monitor = self.monitor_of(Some(selected));
      move_action(hwnd, to.name.clone(), monitor, workspaces)
    });

    self.put_down();
    action
  }

  /// Ends carrying, back to browsing workspaces.
  fn put_down(&mut self) {
    self.mode = Mode::Spaces;
    self.carried = None;
    self.selected_window = None;
    self.end_search();
  }

  /// Sends the picked window `hwnd` to workspace `name`, and moves the
  /// cursor on to its neighbour so the rest can be sorted in a row.
  fn send(
    &mut self,
    hwnd: isize,
    name: String,
    monitor: u128,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let action = move_action(hwnd, name, monitor, workspaces)?;

    let windows = workspaces
      .get(self.selected())
      .map_or_else(Vec::new, ordered);
    let position = windows.iter().position(|window| window.hwnd == hwnd);
    let neighbour = position.and_then(|position| {
      windows
        .get(position + 1)
        .or_else(|| windows.get(position.checked_sub(1)?))
    });

    self.selected_window = neighbour.map(|window| window.hwnd);
    if self.selected_window.is_none() {
      self.mode = Mode::Spaces;
    }
    Some(action)
  }

  /// Selects workspace `index`, moving the keyboard to its screen.
  fn select(&mut self, index: usize) {
    let Some(screen) = self.screen_of(index) else {
      return;
    };

    if index != self.selected() {
      // The window cursor and its windows belong to one workspace.
      self.selected_window = None;
      self.nav.clear();
      if self.mode == Mode::Windows {
        self.mode = Mode::Spaces;
      }
    }

    self.active = screen;
    self.set_shown(screen, index);
  }

  /// Shows workspace `index` on `screen`.
  fn set_shown(&mut self, screen: usize, index: usize) {
    if let Some(cursor) = self.screens.get_mut(screen) {
      cursor.shown = index;
    }
  }

  /// Index of screen `screen`'s first workspace among every screen's.
  pub fn first_card(&self, screen: usize) -> Option<usize> {
    self.screens.get(screen).map(|cursor| cursor.cards.start)
  }

  /// Screen workspace `index` is on.
  fn screen_of(&self, index: usize) -> Option<usize> {
    self
      .screens
      .iter()
      .position(|screen| screen.cards.contains(&index))
  }

  /// Workspaces of the screen with the keyboard.
  fn active_cards(&self) -> Range<usize> {
    self
      .screens
      .get(self.active)
      .map_or(0..0, |screen| screen.cards.clone())
  }

  /// Monitor of the screen workspace `index` is on, or else of the one
  /// with the keyboard.
  fn monitor_of(&self, index: Option<usize>) -> u128 {
    let screen = index
      .and_then(|index| self.screen_of(index))
      .unwrap_or(self.active);
    self.screens.get(screen).map_or(0, |screen| screen.monitor)
  }

  /// Monitor workspace `name` is picked on: the screen with the keyboard
  /// if it has a card for it (as it does for the special workspace), else
  /// the one that has.
  fn monitor_for(
    &self,
    name: &str,
    workspaces: &[OverviewWorkspace],
  ) -> u128 {
    let has_name = |index: &usize| {
      workspaces.get(*index).is_some_and(|w| w.name == name)
    };
    let card = self
      .active_cards()
      .find(has_name)
      .or_else(|| (0..workspaces.len()).find(has_name));

    self.monitor_of(card)
  }

  /// Moves along the workspaces of the screen with the keyboard.
  fn step(&mut self, delta: isize, workspaces: &[OverviewWorkspace]) {
    let cards = self.active_cards();
    let target = self
      .selected()
      .checked_add_signed(delta)
      .filter(|index| cards.contains(index) && *index < workspaces.len());

    if let Some(index) = target {
      self.select(index);
    }
  }

  /// The screen next to the one with the keyboard `toward`: of those that
  /// way and lining up with it, the closest, as `focus --direction` finds
  /// the next monitor.
  fn neighbour(&self, toward: Toward) -> Option<usize> {
    let from = &self.screens.get(self.active)?.rect;

    self
      .screens
      .iter()
      .enumerate()
      .filter(|(_, screen)| {
        let rect = &screen.rect;
        match toward {
          Toward::Left => rect.x() < from.x() && rect.y_overlap(from) > 0,
          Toward::Right => rect.x() > from.x() && rect.y_overlap(from) > 0,
          Toward::Up => rect.y() < from.y() && rect.x_overlap(from) > 0,
          Toward::Down => rect.y() > from.y() && rect.x_overlap(from) > 0,
        }
      })
      .min_by_key(|(_, screen)| match toward {
        Toward::Left => -screen.rect.x(),
        Toward::Right => screen.rect.x(),
        Toward::Up => -screen.rect.y(),
        Toward::Down => screen.rect.y(),
      })
      .map(|(index, _)| index)
  }

  /// Goes on to the screen next to this one `toward`: onto the end of its
  /// workspaces nearest this one when they run that way, else onto the
  /// workspace it shows.
  fn cross(&mut self, toward: Toward, workspaces: &[OverviewWorkspace]) {
    let Some(cursor) = self
      .neighbour(toward)
      .and_then(|screen| self.screens.get(screen))
    else {
      return;
    };

    let row = cursor
      .cards
      .clone()
      .filter(|index| *index < workspaces.len())
      .collect::<Vec<_>>();
    let forward = geometry(self.layout).forward;
    let target = if toward == forward {
      row.first().copied()
    } else if toward == forward.opposite() {
      row.last().copied()
    } else {
      Some(cursor.shown)
        .filter(|shown| cursor.cards.contains(shown))
        .or_else(|| row.first().copied())
    };

    if let Some(index) = target {
      self.select(index);
    }
  }

  /// Moves the cursor to what lies `toward` on screen: a window, a
  /// workspace, the selected workspace's windows where the layout shows
  /// them apart, or else the screen that way. With nothing that way, it
  /// bumps.
  fn arrow(&mut self, toward: Toward, workspaces: &[OverviewWorkspace]) {
    let before = (self.selected(), self.mode, self.selected_window);
    let geometry = geometry(self.layout);

    if self.mode == Mode::Windows {
      self.move_window_cursor(toward, workspaces);
    } else if let Some(delta) = geometry.step(toward) {
      self.step(delta, workspaces);
    } else if geometry.windows == Some(toward) && self.mode == Mode::Spaces
    {
      self.enter_windows(workspaces);
    }

    let is_unmoved = |state: &Self| {
      (state.selected(), state.mode, state.selected_window) == before
    };
    if is_unmoved(self) {
      self.cross(toward, workspaces);
    }
    if is_unmoved(self) {
      self.nudge = Some(toward);
    }
  }

  /// Way the last arrow or wheel step found nowhere to go, if it did,
  /// for the screen with the keyboard to bump that way once.
  pub fn take_nudge(&mut self) -> Option<Toward> {
    self.nudge.take()
  }

  /// Moves the window cursor to the nearest window `toward`. Off the
  /// windows' edge, it goes where the layout has the workspaces: back out
  /// to their row, or on to the next one where the windows show on their
  /// cards.
  fn move_window_cursor(
    &mut self,
    toward: Toward,
    workspaces: &[OverviewWorkspace],
  ) {
    let nav = self.nav(workspaces);
    let from = self.selected_window.and_then(|hwnd| {
      nav
        .iter()
        .find(|(other, _)| *other == hwnd)
        .map(|(_, rect)| *rect)
    });

    let next = match from {
      Some(from) => nearest_toward(&nav, &from, toward),
      // E.g. a hidden tab picked by a search: on to the first window.
      None => nav.first().map(|(hwnd, _)| *hwnd),
    };

    let geometry = geometry(self.layout);
    if let Some(hwnd) = next {
      self.selected_window = Some(hwnd);
    } else if geometry.windows == Some(toward.opposite()) {
      self.mode = Mode::Spaces;
    } else if let Some(delta) =
      geometry.step(toward).filter(|_| geometry.windows.is_none())
    {
      self.step(delta, workspaces);
    }
  }

  /// Looks inside the selected workspace, at the window picked there
  /// last, or else its first.
  fn enter_windows(&mut self, workspaces: &[OverviewWorkspace]) {
    let nav = self.nav(workspaces);
    let hwnd = self
      .selected_window
      .filter(|hwnd| nav.iter().any(|(other, _)| other == hwnd))
      .or_else(|| nav.first().map(|(hwnd, _)| *hwnd));

    let Some(hwnd) = hwnd else {
      return;
    };

    self.mode = Mode::Windows;
    self.selected_window = Some(hwnd);
  }

  /// Where the selected workspace's windows show, first to last: as laid
  /// out, or else where they are on screen.
  fn nav(
    &self,
    workspaces: &[OverviewWorkspace],
  ) -> Cow<'_, [(isize, RectF)]> {
    if !self.nav.is_empty() {
      return Cow::Borrowed(&self.nav);
    }

    workspaces
      .get(self.selected())
      .map_or_else(Vec::new, |workspace| {
        ordered(workspace)
          .into_iter()
          .map(|window| (window.hwnd, RectF::from_rect(&window.rect)))
          .collect()
      })
      .into()
  }

  /// Selects the first (or last) workspace of the screen with the
  /// keyboard, or window while picking one.
  fn edge(&mut self, is_last: bool, workspaces: &[OverviewWorkspace]) {
    if self.mode == Mode::Windows {
      let nav = self.nav(workspaces);
      let window = if is_last { nav.last() } else { nav.first() };
      if let Some(hwnd) = window.map(|(hwnd, _)| *hwnd) {
        self.selected_window = Some(hwnd);
      }
    } else {
      let cards = self.active_cards();
      let last = cards.end.min(workspaces.len()).checked_sub(1);
      let index = if is_last { last } else { Some(cards.start) };
      if let Some(index) = index.filter(|index| cards.contains(index)) {
        self.select(index);
      }
    }
  }

  /// Which match the cursor is on: the picked window, or while carrying,
  /// which picks none, the match last gone to.
  fn match_position(&self) -> Option<usize> {
    if self.mode == Mode::Carrying {
      return self.match_index.filter(|index| *index < self.matches.len());
    }

    let hwnd = self
      .selected_window
      .filter(|_| self.mode == Mode::Windows)?;
    self.matches.iter().position(|found| found.hwnd == hwnd)
  }

  /// Selects the match `delta` on from the one the cursor is on, around
  /// the ends, or the first (or last) when it is on none. Returns whether
  /// there was a match.
  fn goto_match(&mut self, delta: isize) -> bool {
    let Ok(len) = isize::try_from(self.matches.len()) else {
      return false;
    };
    if len == 0 {
      return false;
    }

    let next = match self.match_position() {
      Some(index) => isize::try_from(index)
        .unwrap_or(0)
        .saturating_add(delta)
        .rem_euclid(len),
      None if delta > 0 => 0,
      None => len - 1,
    };

    self.goto_nth_match(usize::try_from(next).unwrap_or(0))
  }

  /// Selects match `index`, on whichever screen it is, as the arrows
  /// would have: its workspace, and its window in it. Returns whether
  /// there is one.
  fn goto_nth_match(&mut self, index: usize) -> bool {
    let Some(found) = self.matches.get(index).copied() else {
      return false;
    };

    self.match_index = Some(index);
    self.select(found.workspace);

    // While carrying, a search only aims the drop; selecting the match
    // would make the space bar pick it up instead.
    if self.mode != Mode::Carrying {
      self.mode = Mode::Windows;
      self.selected_window = Some(found.hwnd);
    }
    true
  }

  pub fn mouse_down(&mut self, position: (f32, f32), hit: Hit) {
    self.press = Some(Press { position, hit });
  }

  /// Closes the window under the cursor, unless one is being dragged.
  pub fn middle_click(&self, hit: Hit) -> Option<OverviewAction> {
    if self.drag.is_some() {
      return None;
    }
    hit.window.map(OverviewAction::CloseWindow)
  }

  /// Tracks hover, and a press turning into a drag. Returns whether
  /// anything changed.
  pub fn mouse_move(
    &mut self,
    position: (f32, f32),
    hit: Hit,
    is_left_down: bool,
  ) -> bool {
    let mut changed = self.hover != hit.workspace;
    self.hover = hit.workspace;
    self.hover_control = hit.control;

    // Pointing at a window of the shown workspace picks it, as the arrows
    // would.
    let pointed = hit.window.filter(|_| {
      hit.workspace == Some(self.selected())
        && self.mode != Mode::Carrying
        && !is_left_down
    });
    if let Some(hwnd) = pointed {
      if self.mode != Mode::Windows || self.selected_window != Some(hwnd) {
        self.mode = Mode::Windows;
        self.selected_window = Some(hwnd);
        self.picked_by = PickedBy::Mouse;
        changed = true;
      }
    }

    let pressed_window = self
      .press
      .filter(|_| is_left_down)
      .and_then(|press| press.hit.window.map(|hwnd| (press, hwnd)));

    let Some((press, hwnd)) = pressed_window else {
      return changed;
    };

    if self.drag.is_none() {
      let travel = (position.0 - press.position.0).abs()
        + (position.1 - press.position.1).abs();
      if travel < self.drag_threshold {
        return changed;
      }
    }

    self.drag = Some(Drag {
      hwnd,
      position,
      target: hit.workspace,
      point: hit.point,
    });
    true
  }

  /// Ends a click or a drag.
  pub fn mouse_up(
    &mut self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let press = self.press.take()?;

    // Dropped where it is, or on nothing: no move.
    if let Some(drag) = self.drag.take() {
      return drag.target.and_then(|index| {
        let to = workspaces.get(index)?;
        let monitor = self.monitor_of(Some(index));
        match place_point(&drag, index, workspaces) {
          Some((x, y)) => Some(OverviewAction::PlaceWindow {
            hwnd: drag.hwnd,
            workspace: to.name.clone(),
            monitor,
            point: Point { x, y },
          }),
          None => {
            move_action(drag.hwnd, to.name.clone(), monitor, workspaces)
          }
        }
      });
    }

    // A click anywhere closes the shortcut sheet, and does nothing more.
    let showed_shortcuts = std::mem::take(&mut self.shows_shortcuts);
    match press.hit.control {
      Some(Control::Shortcuts) => {
        self.shows_shortcuts = !showed_shortcuts;
        return None;
      }
      // Typing goes to the field as it is.
      Some(Control::Search) => return None,
      _ if showed_shortcuts => return None,
      None => {}
    }

    match press.hit {
      Hit {
        window: Some(hwnd),
        workspace,
        ..
      } => Some(OverviewAction::FocusWindow {
        hwnd,
        monitor: self.monitor_of(workspace),
      }),
      Hit {
        workspace: Some(index),
        ..
      } => {
        self.select(index);
        workspaces.get(index).map(|workspace| {
          OverviewAction::FocusWorkspace {
            name: workspace.name.clone(),
            monitor: self.monitor_of(Some(index)),
          }
        })
      }
      Hit { .. } => Some(OverviewAction::Cancel),
    }
  }

  /// Selects the next (or previous) workspace of the screen with the
  /// keyboard.
  pub fn wheel(
    &mut self,
    is_down: bool,
    workspaces: &[OverviewWorkspace],
  ) {
    let before = self.selected();
    self.step(if is_down { 1 } else { -1 }, workspaces);

    if self.selected() == before {
      let forward = geometry(self.layout).forward;
      self.nudge =
        Some(if is_down { forward } else { forward.opposite() });
    }
  }

  /// Notes that the strip was panned by the mouse, which then keeps it
  /// where it is panned to rather than on the picked window.
  pub fn pan(&mut self) {
    self.picked_by = PickedBy::Mouse;
  }

  /// Workspace held down by the left button, until it is let go of or
  /// dragged from.
  fn pressed(&self) -> Option<usize> {
    self
      .press
      .filter(|press| press.hit.control.is_none() && self.drag.is_none())
      .and_then(|press| press.hit.workspace)
  }

  /// Window the dragged window would land beside, and on which side, as
  /// a mouse drag decides: the window under the cursor, on the side of it
  /// nearest the cursor.
  fn drop_beside(
    &self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<(isize, Toward)> {
    let drag = self.drag?;
    let (x, y) = place_point(&drag, drag.target?, workspaces)?;
    let point = Point { x, y };

    // Later windows sit above earlier ones, e.g. floating over tiling.
    let window = workspaces
      .get(drag.target?)?
      .windows
      .iter()
      .rev()
      .filter(|window| {
        window.hwnd != drag.hwnd
          && !window.is_minimized
          && !window.is_hidden_tab
      })
      .find(|window| window.rect.contains_point(&point))?;

    let rect = &window.rect;
    let center = rect.center_point();
    let (dx, dy) = (x - center.x, y - center.y);
    let side = if dx.abs() > dy.abs() {
      if dx > 0 {
        Toward::Right
      } else {
        Toward::Left
      }
    } else if dy > 0 {
      Toward::Down
    } else {
      Toward::Up
    };

    Some((window.hwnd, side))
  }

  pub fn mouse_leave(&mut self) {
    self.hover = None;
    self.hover_control = None;
  }

  /// What `screen` shows of the interaction, in its own card indices.
  pub fn focus(
    &self,
    screen: usize,
    workspaces: &[OverviewWorkspace],
  ) -> ScreenFocus {
    let Some(cursor) = self.screens.get(screen) else {
      return ScreenFocus::default();
    };

    let local = |index: usize| {
      cursor
        .cards
        .contains(&index)
        .then(|| index - cursor.cards.start)
    };
    let is_active = screen == self.active;

    ScreenFocus {
      layout: self.layout,
      is_active,
      mode: if is_active { self.mode } else { Mode::Spaces },
      selected: local(cursor.shown).unwrap_or(0),
      selected_window: self.selected_window.filter(|_| is_active),
      hover: self.hover.and_then(local),
      pressed: self.pressed().and_then(local),
      floating: self.floating(),
      drop: self.drop_target().and_then(local),
      drop_beside: self.drop_beside(workspaces),
      is_dragging: self.drag.is_some(),
      keeps_window_in_view: self.picked_by != PickedBy::Mouse,
      marks_keyboard: is_active && self.screens.len() > 1,
      query: self.query.clone(),
      search: self.search_field(),
      shortcuts: (is_active && self.shows_shortcuts)
        .then(|| self.shortcuts()),
    }
  }

  /// The search field every screen shows.
  pub fn search_field(&self) -> SearchField {
    SearchField {
      query: self.query.clone(),
      hits: self.matches.len(),
      current: self.match_position(),
    }
  }

  /// The keyboard shortcuts of every step, with the arrows `layout` goes
  /// by.
  pub fn shortcuts(&self) -> Vec<ShortcutGroup> {
    let geometry = geometry(self.layout);
    let workspace_arrows = joined_labels(
      [geometry.forward.opposite(), geometry.forward].map(Command::Move),
    );
    let window_arrows = joined_labels(Toward::ALL.map(Command::Move));
    // The arrow into the windows too, where one points at them.
    let look_inside = joined_labels(
      std::iter::once(Command::Space)
        .chain(geometry.windows.map(Command::Move)),
    );
    let send = format!(
      "{} {}",
      Command::SendToWorkspace(1).label(),
      Command::SendToSpecial.label()
    );
    let matches =
      format!("{} {}", Command::Next.label(), Command::Previous.label());

    let windows = vec![
      Shortcut::new(&look_inside, "look inside"),
      Shortcut::new(&window_arrows, "pick a window"),
      Shortcut::new(&Command::Enter.label(), "go to it"),
      Shortcut::new(&Command::CloseWindow.label(), "close it"),
      Shortcut::new(&send, "send it there"),
      Shortcut::new(&Command::Back.label(), "back"),
    ];

    vec![
      ShortcutGroup {
        title: "Workspaces",
        shortcuts: vec![
          Shortcut::new(&workspace_arrows, "select"),
          Shortcut::new(ALT_ARROWS, "as the arrows"),
          Shortcut::new(&Command::Workspace(1).label(), "go to one"),
          Shortcut::new(&Command::Special.label(), "special"),
          Shortcut::new(&Command::Enter.label(), "go there"),
          Shortcut::new(
            &Command::Next.label(),
            &format!("{} layout", self.layout.next().name()),
          ),
          Shortcut::new(&Command::Cancel.label(), "close"),
        ],
      },
      ShortcutGroup {
        title: "Windows",
        shortcuts: windows,
      },
      ShortcutGroup {
        title: "Carrying",
        shortcuts: vec![
          Shortcut::new(&Command::Space.label(), "lift / drop"),
          Shortcut::new(&workspace_arrows, "pick a workspace"),
          Shortcut::new(&send, "drop on that one"),
          Shortcut::new(&Command::Back.label(), "put it down"),
        ],
      },
      ShortcutGroup {
        title: "Search",
        shortcuts: vec![
          Shortcut::new("A\u{2013}Z", "type to search"),
          Shortcut::new(&matches, "next / previous"),
          Shortcut::new(&Command::Enter.label(), "go to the match"),
          Shortcut::new(&Command::Back.label(), "erase"),
          Shortcut::new(&Command::Cancel.label(), "clear"),
        ],
      },
    ]
  }
}

/// The interaction as one screen shows it, in that screen's own card
/// indices.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct ScreenFocus {
  pub layout: OverviewLayoutMode,

  /// Whether the keyboard is on this screen.
  pub is_active: bool,

  /// Where the keyboard is in the flow, while it is on this screen; else
  /// browsing.
  pub mode: Mode,

  /// Workspace it shows.
  pub selected: usize,

  /// Window picked on it.
  pub selected_window: Option<isize>,

  /// Workspace under the cursor.
  pub hover: Option<usize>,

  /// Workspace held down by the left button.
  pub pressed: Option<usize>,

  /// Window lifted by the mouse or the keyboard, from whichever screen.
  pub floating: Option<isize>,

  /// Workspace the floating window would land on.
  pub drop: Option<usize>,

  /// Window a dragged window would land beside, and on which side of it.
  pub drop_beside: Option<(isize, Toward)>,

  /// Whether a window is dragged, over whichever screen.
  pub is_dragging: bool,

  /// Whether the stack pans the picked window's strip to keep it in
  /// view: not for one pointed at, which would slide away under the
  /// cursor.
  pub keeps_window_in_view: bool,

  /// Whether it marks that the keyboard is on it, as one of several
  /// screens.
  pub marks_keyboard: bool,

  /// Search query, whose matches show on every screen.
  pub query: String,

  /// The search field.
  pub search: SearchField,

  /// The keyboard shortcuts, while their sheet is open on this screen.
  pub shortcuts: Option<Vec<ShortcutGroup>>,
}

impl ScreenFocus {
  /// Whether `window` matches the search query.
  pub fn is_match(&self, window: &OverviewWindow) -> bool {
    match_score(window, &self.query).is_some()
  }
}

/// A cursor per screen, each on its displayed workspace.
fn cursors(screens: &[OverviewScreen]) -> Vec<ScreenCursor> {
  let mut start = 0;

  screens
    .iter()
    .map(|screen| {
      let cards = start..start + screen.workspaces.len();
      start = cards.end;
      let shown = cards.start
        + screen
          .workspaces
          .iter()
          .position(|workspace| workspace.is_displayed)
          .unwrap_or(0);

      ScreenCursor {
        monitor: screen.id,
        rect: screen.rect.clone(),
        cards,
        shown,
      }
    })
    .collect()
}

/// Every screen's workspaces, screen after screen, as the interaction
/// indexes them.
pub(crate) fn all_workspaces(
  screens: &[OverviewScreen],
) -> Vec<OverviewWorkspace> {
  screens
    .iter()
    .flat_map(|screen| screen.workspaces.iter().cloned())
    .collect()
}

/// The special workspace's card its windows count on: the one displayed
/// on its screen, if any is, else the first.
fn home_special(workspaces: &[OverviewWorkspace]) -> Option<usize> {
  let specials = || {
    workspaces
      .iter()
      .enumerate()
      .filter(|(_, workspace)| workspace.is_special)
  };

  specials()
    .find(|(_, workspace)| workspace.is_displayed)
    .or_else(|| specials().next())
    .map(|(index, _)| index)
}

/// The windows matching `query` on every screen, best first, then in
/// order. The special workspace's windows count once, on the screen
/// showing it if any does, else on the first.
fn find_matches(
  query: &str,
  workspaces: &[OverviewWorkspace],
) -> Vec<Match> {
  if query.is_empty() {
    return Vec::new();
  }

  let special = home_special(workspaces);
  let query = query.to_lowercase();

  let mut matches = workspaces
    .iter()
    .enumerate()
    .filter(|(index, workspace)| {
      !workspace.is_special || Some(*index) == special
    })
    .flat_map(|(index, workspace)| {
      let unshown = workspace
        .windows
        .iter()
        .filter(|window| window.is_minimized || window.is_hidden_tab);
      let query = &query;
      ordered(workspace).into_iter().chain(unshown).filter_map(
        move |window| {
          let score = lowercase_match_score(window, query)?;
          let found = Match {
            workspace: index,
            hwnd: window.hwnd,
          };
          Some((score, found))
        },
      )
    })
    .collect::<Vec<_>>();

  matches.sort_by_key(|(score, _)| *score);
  matches.into_iter().map(|(_, found)| found).collect()
}

/// Where a layout puts things on screen, which the arrows go by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Geometry {
  /// Way its workspaces run, first to last.
  forward: Toward,

  /// Way from the workspaces to the selected one's windows, where those
  /// show apart from them. `None` where they show on its card, which no
  /// arrow points into.
  windows: Option<Toward>,
}

impl Geometry {
  /// How many workspaces on an arrow `toward` goes, if it runs along
  /// them.
  fn step(self, toward: Toward) -> Option<isize> {
    if toward == self.forward {
      Some(1)
    } else if toward == self.forward.opposite() {
      Some(-1)
    } else {
      None
    }
  }
}

fn geometry(layout: OverviewLayoutMode) -> Geometry {
  match layout {
    OverviewLayoutMode::Activities => Geometry {
      forward: Toward::Right,
      windows: Some(Toward::Down),
    },
    OverviewLayoutMode::Stack => Geometry {
      forward: Toward::Down,
      windows: None,
    },
  }
}

/// The window in `nav` nearest `from`, `toward`: of those whose middle
/// lies that way and that line up with `from` or lie within 45 degrees of
/// it, the closest, sideways distance counting more for those not lined
/// up.
fn nearest_toward(
  nav: &[(isize, RectF)],
  from: &RectF,
  toward: Toward,
) -> Option<isize> {
  let (from_x, from_y) = from.center();

  nav
    .iter()
    .filter_map(|(hwnd, rect)| {
      let (x, y) = rect.center();
      let lines_up_across =
        rect.y < from.bottom() && rect.bottom() > from.y;
      let lines_up_down = rect.x < from.right() && rect.right() > from.x;

      let (along, across, lines_up) = match toward {
        Toward::Left => (from_x - x, (y - from_y).abs(), lines_up_across),
        Toward::Right => (x - from_x, (y - from_y).abs(), lines_up_across),
        Toward::Up => (from_y - y, (x - from_x).abs(), lines_up_down),
        Toward::Down => (y - from_y, (x - from_x).abs(), lines_up_down),
      };

      (along > 0.5 && (lines_up || across <= along)).then(|| {
        let weight = if lines_up { 0.25 } else { 2.0 };
        (*hwnd, along + across * weight)
      })
    })
    .min_by(|(_, a), (_, b)| a.total_cmp(b))
    .map(|(hwnd, _)| hwnd)
}

/// Shown windows of `workspace` in on-screen order, left to right: not
/// minimized, and of a stack only its shown tab.
fn ordered(workspace: &OverviewWorkspace) -> Vec<&OverviewWindow> {
  let mut windows = workspace
    .windows
    .iter()
    .filter(|window| {
      !window.is_minimized
        && !window.is_hidden_tab
        && window.rect.width() > 0
    })
    .collect::<Vec<_>>();

  windows.sort_by_key(|window| (window.rect.left, window.rect.top));
  windows
}

/// Moves window `hwnd` to workspace `name`, picked on `monitor`'s screen,
/// unless it is already there.
/// Where in workspace `index` a window dragged as `drag` is placed, as
/// a mouse drag of it to there would: `None` if it only moves there, onto
/// the special workspace (whose card shows it wherever it is summoned),
/// or is dropped onto itself.
fn place_point(
  drag: &Drag,
  index: usize,
  workspaces: &[OverviewWorkspace],
) -> Option<(i32, i32)> {
  let (x, y) = drag.point?;
  let workspace = workspaces
    .get(index)
    .filter(|_| !is_special(workspaces, index))?;
  let is_on_itself = workspace.windows.iter().any(|window| {
    window.hwnd == drag.hwnd && window.rect.contains_point(&Point { x, y })
  });

  (!is_on_itself).then_some((x, y))
}

fn move_action(
  hwnd: isize,
  name: String,
  monitor: u128,
  workspaces: &[OverviewWorkspace],
) -> Option<OverviewAction> {
  let (from, _) = find(workspaces, hwnd)?;
  let is_there = workspaces
    .get(from)
    .is_some_and(|workspace| workspace.name == name);

  (!is_there).then_some(OverviewAction::MoveWindow {
    hwnd,
    workspace: name,
    monitor,
  })
}

/// Name of the workspace digit key `digit` stands for, 0 being the tenth:
/// the one named after it when workspaces are numbered, whether it has a
/// card on any screen or is one of `others`, else the one at its position
/// among `cards` (not counting the special workspace's, which `-` goes
/// to).
fn digit_target(
  digit: u8,
  workspaces: &[OverviewWorkspace],
  cards: Range<usize>,
  others: &[String],
) -> Option<String> {
  let number = if digit == 0 { 10 } else { usize::from(digit) };
  let is_numbered = !others.is_empty()
    || workspaces
      .iter()
      .any(|workspace| workspace.name.parse::<usize>().is_ok());

  if !is_numbered {
    return workspaces
      .get(cards)?
      .iter()
      .filter(|workspace| !workspace.is_special)
      .nth(number - 1)
      .map(|workspace| workspace.name.clone());
  }

  workspaces
    .iter()
    .map(|workspace| &workspace.name)
    .chain(others)
    .find(|name| name.parse::<usize>() == Ok(number))
    .cloned()
}

/// The workspace index and window with handle `hwnd`.
fn find(
  workspaces: &[OverviewWorkspace],
  hwnd: isize,
) -> Option<(usize, &OverviewWindow)> {
  workspaces
    .iter()
    .enumerate()
    .find_map(|(index, workspace)| {
      workspace
        .windows
        .iter()
        .find(|window| window.hwnd == hwnd)
        .map(|window| (index, window))
    })
}

/// Where `query` sits in the window's title or process name, lower being
/// better; `None` when it is in neither.
fn match_score(window: &OverviewWindow, query: &str) -> Option<usize> {
  if query.is_empty() {
    return None;
  }
  lowercase_match_score(window, &query.to_lowercase())
}

/// `match_score` for a `query` already in lower case.
fn lowercase_match_score(
  window: &OverviewWindow,
  query: &str,
) -> Option<usize> {
  [&window.title, &window.process_name]
    .into_iter()
    .filter_map(|text| text.to_lowercase().find(query))
    .min()
}

/// Whether card `index` is the special workspace's.
fn is_special(workspaces: &[OverviewWorkspace], index: usize) -> bool {
  workspaces
    .get(index)
    .is_some_and(|workspace| workspace.is_special)
}

#[cfg(test)]
mod tests {
  use super::{
    all_workspaces, Control, Hit, Interaction, Key, Mode, SearchField,
    Toward, BINDINGS,
  };
  use crate::{
    overview_layout::RectF, Direction, OverviewAction, OverviewInput,
    OverviewLayoutMode, OverviewScreen, OverviewWindow, OverviewWorkspace,
    Point, Rect,
  };

  /// ID of the monitor of the screen most tests have.
  const MONITOR: u128 = 1;

  /// A screen of monitor `id` showing `workspaces`.
  fn screen(
    id: u128,
    workspaces: Vec<OverviewWorkspace>,
  ) -> OverviewScreen {
    OverviewScreen {
      id,
      workspaces,
      ..OverviewScreen::default()
    }
  }

  /// The interaction of a single screen showing `workspaces`.
  fn single(
    workspaces: &[OverviewWorkspace],
    layout: OverviewLayoutMode,
  ) -> Interaction {
    Interaction::new(&[screen(MONITOR, workspaces.to_vec())], layout, 6.0)
  }

  fn focus_workspace(name: &str) -> OverviewAction {
    OverviewAction::FocusWorkspace {
      name: name.to_string(),
      monitor: MONITOR,
    }
  }

  fn focus_window(hwnd: isize) -> OverviewAction {
    OverviewAction::FocusWindow {
      hwnd,
      monitor: MONITOR,
    }
  }

  fn window(hwnd: isize, title: &str, rect: Rect) -> OverviewWindow {
    OverviewWindow {
      hwnd,
      title: title.to_string(),
      process_name: format!("{title}.exe"),
      rect,
      is_minimized: false,
      is_hidden_tab: false,
      is_scrolled_off: false,
      opacity: u8::MAX,
      overlays: Vec::new(),
      border: None,
    }
  }

  /// Workspace "1" with two side-by-side windows, "2" focused with one,
  /// "3" with none.
  fn workspaces() -> Vec<OverviewWorkspace> {
    vec![
      OverviewWorkspace {
        windows: vec![
          window(12, "Editor", Rect::from_xy(960, 0, 960, 1000)),
          window(11, "Terminal", Rect::from_xy(0, 0, 960, 1000)),
        ],
        ..card("1")
      },
      OverviewWorkspace {
        is_displayed: true,
        windows: vec![window(
          21,
          "Browser",
          Rect::from_xy(0, 0, 1920, 1000),
        )],
        ..card("2")
      },
      card("3"),
    ]
  }

  /// An empty card for workspace `name`.
  fn card(name: &str) -> OverviewWorkspace {
    OverviewWorkspace {
      name: name.to_string(),
      label: name.to_string(),
      content_rect: Rect::from_xy(0, 0, 1920, 1000),
      ..OverviewWorkspace::default()
    }
  }

  fn interaction() -> Interaction {
    single(&workspaces(), OverviewLayoutMode::Activities)
  }

  fn press(
    state: &mut Interaction,
    keys: &[Key],
  ) -> Option<OverviewAction> {
    let workspaces = workspaces();
    keys
      .iter()
      .fold(None, |_, key| state.key(*key, &workspaces))
  }

  /// The keys that type `text`.
  fn typed(text: &str) -> Vec<Key> {
    text
      .chars()
      .map(|text| {
        if text == ' ' {
          Key::Space
        } else {
          Key::Text(text)
        }
      })
      .collect()
  }

  #[test]
  fn starts_on_the_focused_workspace() {
    assert_eq!(interaction().selected(), 1);
  }

  #[test]
  fn no_command_is_on_a_key_that_types() {
    for (command, keys) in BINDINGS {
      assert!(
        keys.iter().all(|key| !matches!(key, Key::Text(_))),
        "{command:?} is on a typed character"
      );
    }
  }

  #[test]
  fn no_key_is_bound_twice() {
    for (index, (_, keys)) in BINDINGS.iter().enumerate() {
      for key in *keys {
        assert!(
          BINDINGS[index + 1..]
            .iter()
            .all(|(_, later)| !later.contains(key)),
          "{key:?} is bound twice"
        );
      }
    }
  }

  #[test]
  fn the_shortcut_sheet_shows_the_bound_keys() {
    let keys = interaction()
      .shortcuts()
      .into_iter()
      .map(|group| {
        group
          .shortcuts
          .into_iter()
          .map(|shortcut| shortcut.key)
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    assert_eq!(
      keys,
      [
        vec![
          "\u{2190}\u{2192}",
          "Alt+H J K L",
          "Alt+1\u{2013}9",
          "Alt+-",
          "\u{23ce}",
          "Tab",
          "Esc"
        ],
        vec![
          "Space\u{2193}",
          "\u{2190}\u{2191}\u{2193}\u{2192}",
          "\u{23ce}",
          "Del",
          "Alt+\u{21e7}1\u{2013}9 Alt+\u{21e7}-",
          "\u{232b}",
        ],
        vec![
          "Space",
          "\u{2190}\u{2192}",
          "Alt+\u{21e7}1\u{2013}9 Alt+\u{21e7}-",
          "\u{232b}"
        ],
        vec![
          "A\u{2013}Z",
          "Tab \u{21e7}Tab",
          "\u{23ce}",
          "\u{232b}",
          "Esc"
        ],
      ]
    );
  }

  #[test]
  fn arrows_step_through_workspaces_and_stop_at_the_ends() {
    let mut state = interaction();

    press(&mut state, &[Key::Left, Key::Left]);
    assert_eq!(state.selected(), 0);

    press(&mut state, &[Key::Alt('l'), Key::Right, Key::Right]);
    assert_eq!(state.selected(), 2);

    press(&mut state, &[Key::Home]);
    assert_eq!(state.selected(), 0);
  }

  #[test]
  fn enter_switches_to_the_selected_workspace() {
    let mut state = interaction();

    assert_eq!(
      press(&mut state, &[Key::Left, Key::Enter]),
      Some(focus_workspace("1"))
    );
  }

  #[test]
  fn digits_switch_straight_to_a_workspace() {
    let mut state = interaction();

    assert_eq!(
      press(&mut state, &[Key::Alt('3')]),
      Some(focus_workspace("3"))
    );
    assert_eq!(press(&mut state, &[Key::Alt('9')]), None);
  }

  #[test]
  fn digits_go_by_name_when_workspaces_are_numbered() {
    let mut workspaces = workspaces();
    workspaces.remove(1);
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);

    assert_eq!(
      state.key(Key::Alt('3'), &workspaces),
      Some(focus_workspace("3"))
    );
    assert_eq!(state.key(Key::Alt('2'), &workspaces), None, "not shown");
  }

  #[test]
  fn digits_go_by_position_when_workspaces_are_named() {
    let mut workspaces = workspaces();
    for (workspace, name) in
      workspaces.iter_mut().zip(["web", "code", "chat"])
    {
      workspace.name = name.to_string();
    }
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);

    assert_eq!(
      state.key(Key::Alt('1'), &workspaces),
      Some(focus_workspace("web"))
    );
  }

  #[test]
  fn digits_reach_workspaces_without_a_card() {
    let mut state = interaction();
    state.digit_workspaces = vec!["5".to_string()];

    assert_eq!(
      press(&mut state, &[Key::Alt('5')]),
      Some(focus_workspace("5"))
    );
    assert_eq!(press(&mut state, &[Key::Alt('6')]), None);
    assert_eq!(
      press(&mut state, &[Key::Space, Key::AltShift('5')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "5".to_string(),
        monitor: MONITOR,
      })
    );
  }

  #[test]
  fn shifted_digits_send_the_picked_window_and_move_on() {
    let mut state = interaction();
    assert_eq!(
      press(&mut state, &[Key::AltShift('3')]),
      None,
      "no window to send while browsing"
    );
    press(&mut state, &[Key::Left, Key::Space]);

    assert_eq!(
      press(&mut state, &[Key::AltShift('1')]),
      None,
      "already on workspace 1"
    );
    assert_eq!(
      press(&mut state, &[Key::AltShift('3')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 11,
        workspace: "3".to_string(),
        monitor: MONITOR,
      })
    );
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected_window, Some(12), "next window picked");

    // A plain digit chord still goes there, whatever is picked.
    assert_eq!(
      press(&mut state, &[Key::Alt('3')]),
      Some(focus_workspace("3"))
    );
  }

  #[test]
  fn digits_drop_a_carried_window() {
    for digit in [Key::Alt('1'), Key::AltShift('1')] {
      let mut state = interaction();

      assert_eq!(
        press(&mut state, &[Key::Space, Key::Space, digit]),
        Some(OverviewAction::MoveWindow {
          hwnd: 21,
          workspace: "1".to_string(),
          monitor: MONITOR,
        })
      );
      assert_eq!(state.mode, Mode::Spaces);
      assert_eq!(state.carried, None);
    }
  }

  #[test]
  fn a_new_workspace_is_picked_like_any_other() {
    let mut workspaces = workspaces();
    workspaces.push(OverviewWorkspace {
      is_new: true,
      ..card("4")
    });
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);

    let mut press = |keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    assert_eq!(
      press(&[Key::Space, Key::Space, Key::End, Key::Space]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "4".to_string(),
        monitor: MONITOR,
      })
    );
    assert_eq!(press(&[Key::End, Key::Enter]), Some(focus_workspace("4")));
  }

  #[test]
  fn hidden_tabs_are_skipped_but_found() {
    let mut workspaces = workspaces();
    workspaces[0].windows.push(OverviewWindow {
      is_hidden_tab: true,
      ..window(13, "Docs", Rect::from_xy(0, 0, 960, 1000))
    });
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);
    let press = |state: &mut Interaction, keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    press(&mut state, &[Key::Left, Key::Space, Key::Right, Key::Right]);
    assert_eq!(
      state.selected_window,
      Some(12),
      "steps over the hidden tab"
    );

    press(&mut state, &[Key::Backspace]);
    press(&mut state, &typed("docs"));
    assert_eq!(
      press(&mut state, &[Key::Enter]),
      Some(focus_window(13)),
      "search finds it"
    );
  }

  #[test]
  fn windows_are_picked_in_on_screen_order() {
    let mut state = interaction();

    press(&mut state, &[Key::Left, Key::Space]);
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected_window, Some(11), "left window first");

    press(&mut state, &[Key::Right, Key::Right]);
    assert_eq!(state.selected_window, Some(12));

    assert_eq!(press(&mut state, &[Key::Enter]), Some(focus_window(12)));
  }

  #[test]
  fn home_and_end_pick_the_outer_windows_while_picking() {
    let mut state = interaction();

    press(&mut state, &[Key::Left, Key::Space, Key::End]);
    assert_eq!(state.selected(), 0);
    assert_eq!(state.selected_window, Some(12));

    press(&mut state, &[Key::Home]);
    assert_eq!(state.selected_window, Some(11));
  }

  #[test]
  fn tab_goes_through_every_layout() {
    let mut state = interaction();
    let mut seen = vec![state.layout];
    for key in [Key::Tab, Key::BackTab] {
      press(&mut state, &[key]);
      seen.push(state.layout);
    }

    assert_eq!(
      seen,
      vec![
        OverviewLayoutMode::Activities,
        OverviewLayoutMode::Stack,
        OverviewLayoutMode::Activities,
      ]
    );
  }

  #[test]
  fn looking_inside_keeps_the_layout() {
    for layout in OverviewLayoutMode::ALL {
      let mut state = single(&workspaces(), layout);
      assert_eq!(state.mode, Mode::Spaces);
      press(&mut state, &[Key::Space]);
      assert_eq!(state.mode, Mode::Windows);
      assert_eq!(state.layout, layout);
    }
  }

  #[test]
  fn arrows_go_to_the_nearest_window_where_windows_show() {
    let mut state = single(&workspaces(), OverviewLayoutMode::Activities);
    state.mode = Mode::Windows;

    // Laid out as two rows: 1 2 3 over 4 5.
    state.nav = vec![
      (1, RectF::new(0.0, 0.0, 100.0, 100.0)),
      (2, RectF::new(120.0, 10.0, 100.0, 80.0)),
      (3, RectF::new(240.0, 0.0, 100.0, 100.0)),
      (4, RectF::new(60.0, 140.0, 100.0, 100.0)),
      (5, RectF::new(180.0, 140.0, 100.0, 100.0)),
    ];
    state.selected_window = Some(1);
    let mut walk = |key: Key| {
      state.key(key, &workspaces());
      state.selected_window
    };

    assert_eq!(walk(Key::Right), Some(2));
    assert_eq!(walk(Key::Right), Some(3));
    assert_eq!(walk(Key::Right), Some(3), "nothing further right");
    assert_eq!(walk(Key::Down), Some(5));
    assert_eq!(walk(Key::Left), Some(4));
    assert_eq!(walk(Key::Up), Some(1));
    assert_eq!(walk(Key::End), Some(5));
    assert_eq!(walk(Key::Home), Some(1));
  }

  #[test]
  fn the_activities_go_down_into_windows_and_back_up() {
    let mut state = single(&workspaces(), OverviewLayoutMode::Activities);

    // Down into the shown workspace's windows, up to the workspaces, along
    // them, and down again.
    press(&mut state, &[Key::Down]);
    assert_eq!(state.mode, Mode::Windows);
    press(&mut state, &[Key::Up]);
    assert_eq!(state.mode, Mode::Spaces);
    press(&mut state, &[Key::Left, Key::Down]);
    assert_eq!(state.selected(), 0);
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected_window, Some(11), "its first window");

    assert_eq!(
      press(&mut state, &[Key::Right, Key::Enter]),
      Some(focus_window(12))
    );
  }

  #[test]
  fn the_stack_steps_down_its_column_and_only_space_goes_inside() {
    let workspaces = workspaces();
    let mut state = single(&workspaces, OverviewLayoutMode::Stack);
    let mut press = |keys: &[Key]| {
      for key in keys {
        state.key(*key, &workspaces);
      }
      (state.selected(), state.mode, state.selected_window)
    };

    assert_eq!(press(&[Key::Down]), (2, Mode::Spaces, None));
    assert_eq!(press(&[Key::Up, Key::Up]), (0, Mode::Spaces, None));

    // No arrow points into a card's windows: sideways only bumps.
    assert_eq!(press(&[Key::Right]), (0, Mode::Spaces, None));
    assert_eq!(press(&[Key::Left]), (0, Mode::Spaces, None));

    assert_eq!(press(&[Key::Space]), (0, Mode::Windows, Some(11)));
    assert_eq!(press(&[Key::Right]), (0, Mode::Windows, Some(12)));
    assert_eq!(
      press(&[Key::Right]),
      (0, Mode::Windows, Some(12)),
      "nothing further right"
    );
    assert_eq!(press(&[Key::Left]), (0, Mode::Windows, Some(11)));
    assert_eq!(press(&[Key::Left]), (0, Mode::Windows, Some(11)));
  }

  #[test]
  fn the_stack_leaves_a_workspace_windows_up_and_down() {
    let workspaces = workspaces();
    let mut state = single(&workspaces, OverviewLayoutMode::Stack);
    let mut press = |keys: &[Key]| {
      for key in keys {
        state.key(*key, &workspaces);
      }
      (state.selected(), state.mode)
    };

    // Off the windows' bottom edge is the workspace below, and off their
    // top the one above.
    assert_eq!(press(&[Key::Space]), (1, Mode::Windows));
    assert_eq!(press(&[Key::Down]), (2, Mode::Spaces));
    assert_eq!(press(&[Key::Up, Key::Space, Key::Up]), (0, Mode::Spaces));

    // Off the first workspace's, there is nowhere to go.
    assert_eq!(press(&[Key::Space, Key::Up]), (0, Mode::Windows));
    assert_eq!(state.take_nudge(), Some(Toward::Up));
  }

  #[test]
  fn arrows_bump_alike_both_ways_in_every_layout() {
    for layout in OverviewLayoutMode::ALL {
      let workspaces = workspaces();
      let mut state = single(&workspaces, layout);
      let across = match layout {
        OverviewLayoutMode::Activities => [Key::Up, Key::Up],
        OverviewLayoutMode::Stack => [Key::Left, Key::Right],
      };

      for (key, toward) in across.into_iter().zip(
        [[Toward::Up, Toward::Up], [Toward::Left, Toward::Right]]
          [usize::from(layout == OverviewLayoutMode::Stack)],
      ) {
        state.key(key, &workspaces);
        assert_eq!((state.selected(), state.mode), (1, Mode::Spaces));
        assert_eq!(state.take_nudge(), Some(toward), "{layout:?}");
      }
    }
  }

  #[test]
  fn clicking_another_workspace_goes_to_it_in_every_layout() {
    let workspaces = workspaces();

    for layout in OverviewLayoutMode::ALL {
      let mut state = single(&workspaces, layout);
      state.mouse_down(
        (0.0, 0.0),
        Hit {
          workspace: Some(0),
          window: None,
          control: None,
          point: None,
        },
      );
      assert_eq!(
        state.mouse_up(&workspaces),
        Some(focus_workspace("1")),
        "{layout:?}"
      );
      assert_eq!(state.selected(), 0, "{layout:?}");
    }
  }

  /// A window is a precise target, e.g. a search match outlined on a
  /// workspace not shown yet.
  #[test]
  fn clicking_a_window_of_another_workspace_goes_to_it() {
    let workspaces = workspaces();

    for layout in
      [OverviewLayoutMode::Activities, OverviewLayoutMode::Stack]
    {
      let mut state = single(&workspaces, layout);
      state.mouse_down(
        (0.0, 0.0),
        Hit {
          workspace: Some(0),
          window: Some(11),
          control: None,
          point: None,
        },
      );
      assert_eq!(
        state.mouse_up(&workspaces),
        Some(focus_window(11)),
        "{layout:?}"
      );
    }
  }

  #[test]
  fn only_a_window_picked_by_the_mouse_stays_where_it_is() {
    // On the workspace with two windows to step between.
    let mut workspaces = workspaces();
    workspaces[0].is_displayed = true;
    workspaces[1].is_displayed = false;
    let mut state = single(&workspaces, OverviewLayoutMode::Stack);
    let keeps = |state: &Interaction| {
      state.focus(0, &workspaces).keeps_window_in_view
    };

    state.key(Key::Space, &workspaces);
    assert_eq!(state.selected_window, Some(11));
    assert!(keeps(&state), "stepped to");

    let pointed = Hit {
      workspace: Some(0),
      window: Some(12),
      ..Hit::default()
    };
    state.mouse_move((10.0, 10.0), pointed, false);
    assert_eq!(state.selected_window, Some(12));
    assert!(!keeps(&state), "pointed at");

    state.key(Key::Left, &workspaces);
    assert!(keeps(&state), "stepped to again");
    state.pan();
    assert!(!keeps(&state), "panned away from");
  }

  #[test]
  fn pointing_at_a_shown_window_picks_it() {
    let workspaces = workspaces();
    let mut state = single(&workspaces, OverviewLayoutMode::Stack);

    let changed = state.mouse_move(
      (10.0, 10.0),
      Hit {
        workspace: Some(1),
        window: Some(21),
        control: None,
        point: None,
      },
      false,
    );
    assert!(changed);
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected_window, Some(21));

    // Not in another workspace's miniature.
    state.mouse_move(
      (10.0, 10.0),
      Hit {
        workspace: Some(0),
        window: Some(11),
        control: None,
        point: None,
      },
      false,
    );
    assert_eq!(state.selected_window, Some(21));
  }

  #[test]
  fn space_on_an_empty_workspace_stays_put() {
    let mut state = interaction();

    press(&mut state, &[Key::End, Key::Space]);
    assert_eq!(state.mode, Mode::Spaces);
  }

  #[test]
  fn a_carried_window_drops_on_the_picked_workspace() {
    let mut state = interaction();

    press(&mut state, &[Key::Space, Key::Space]);
    assert_eq!(state.mode, Mode::Carrying);
    assert_eq!(state.floating(), Some(21));
    assert_eq!(state.drop_target(), Some(1));

    press(&mut state, &[Key::Left]);
    assert_eq!(state.drop_target(), Some(0));

    assert_eq!(
      press(&mut state, &[Key::Space]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "1".to_string(),
        monitor: MONITOR,
      })
    );
    assert_eq!(state.mode, Mode::Spaces);
    assert_eq!(state.carried, None);
  }

  #[test]
  fn dropping_where_it_came_from_moves_nothing() {
    let mut state = interaction();
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Space, Key::Enter]),
      None
    );
  }

  #[test]
  fn backspace_steps_back_one_step() {
    let mut state = interaction();

    press(&mut state, &[Key::Space, Key::Space, Key::Backspace]);
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.carried, None);

    press(&mut state, &[Key::Backspace]);
    assert_eq!(state.mode, Mode::Spaces);
    assert_eq!(state.selected_window, None);
  }

  #[test]
  fn escape_cancels_and_delete_closes_the_picked_window() {
    let mut state = interaction();
    assert_eq!(
      press(&mut state, &[Key::Escape]),
      Some(OverviewAction::Cancel)
    );

    assert_eq!(press(&mut state, &[Key::Delete]), None, "nothing picked");
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Delete]),
      Some(OverviewAction::CloseWindow(21))
    );
  }

  #[test]
  fn the_wm_keybindings_do_as_the_overview_keys() {
    let workspaces = workspaces();
    let named = |name: &str| name.to_string();

    // `focus --direction` and `move --direction` are the arrows.
    let mut state = interaction();
    state.input(&OverviewInput::Move(Direction::Left), &workspaces);
    assert_eq!(state.selected(), 0);
    state.input(&OverviewInput::Move(Direction::Down), &workspaces);
    assert_eq!(
      (state.mode, state.selected_window),
      (Mode::Windows, Some(11))
    );
    state.input(&OverviewInput::Move(Direction::Right), &workspaces);
    assert_eq!(state.selected_window, Some(12));

    // `move --workspace` sends what is picked, `close` closes it, and
    // `focus --workspace` goes there.
    assert_eq!(
      state
        .input(&OverviewInput::MoveToWorkspace(named("3")), &workspaces),
      Some(OverviewAction::MoveWindow {
        hwnd: 12,
        workspace: named("3"),
        monitor: MONITOR,
      })
    );
    assert_eq!(
      state.input(&OverviewInput::CloseWindow, &workspaces),
      Some(OverviewAction::CloseWindow(11))
    );
    assert_eq!(
      state.input(&OverviewInput::FocusWorkspace(named("3")), &workspaces),
      Some(focus_workspace("3"))
    );

    // The next and previous workspace are stepped to, not switched to.
    let mut state = interaction();
    assert_eq!(
      state.input(&OverviewInput::NextWorkspace, &workspaces),
      None
    );
    assert_eq!(state.selected(), 2);
    state.input(&OverviewInput::PreviousWorkspace, &workspaces);
    assert_eq!(state.selected(), 1);

    // During a search, a direction steps through its matches.
    press(&mut state, &typed("e"));
    let first = state.selected_window;
    state.input(&OverviewInput::Move(Direction::Down), &workspaces);
    assert_ne!(state.selected_window, first);
    assert_eq!(state.query, "e");
  }

  #[test]
  fn the_wm_keybindings_reach_the_special_workspace() {
    let workspaces = with_special();
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);

    assert_eq!(
      state.input(&OverviewInput::MoveToSpecialWorkspace, &workspaces),
      None,
      "no window to send while browsing"
    );
    assert_eq!(
      state.input(&OverviewInput::FocusSpecialWorkspace, &workspaces),
      Some(focus_workspace("special"))
    );
  }

  #[test]
  fn typing_starts_a_search_that_follows_its_best_match() {
    let mut state = interaction();
    assert!(!state.is_searching());

    // The first character already searches, and picks.
    press(&mut state, &[Key::Text('t')]);
    assert!(state.is_searching());
    assert_eq!(state.query, "t");
    assert_eq!(state.selected_window, Some(11), "Terminal, at its start");

    // "Editor" has it later than "Terminal", which a longer query drops.
    press(&mut state, &typed("o"));
    assert_eq!(state.query, "to");
    assert_eq!((state.selected(), state.mode), (0, Mode::Windows));
    assert_eq!(state.selected_window, Some(12));

    // Taking a character back follows the best match again.
    press(&mut state, &[Key::Backspace]);
    assert_eq!(state.query, "t");
    assert_eq!(state.selected_window, Some(11));
  }

  #[test]
  fn every_typed_character_is_searched_for() {
    // What used to be commands, and the digits.
    for text in ['/', '?', 'x', 's', 'n', 'N', 'h', 'j', 'k', 'l', '1'] {
      let mut state = interaction();
      assert_eq!(press(&mut state, &[Key::Text(text)]), None);
      assert_eq!(state.query, text.to_string());
      assert!(!state.shows_shortcuts);
      assert_eq!(state.layout, OverviewLayoutMode::Activities);
    }
  }

  #[test]
  fn a_space_is_typed_only_into_a_query() {
    let mut state = interaction();

    press(&mut state, &[Key::Space]);
    assert_eq!(state.query, "");
    assert_eq!(state.mode, Mode::Windows, "looks inside");

    press(&mut state, &typed("my file"));
    assert_eq!(state.query, "my file");
    assert_eq!(state.carried, None, "lifts nothing");
  }

  #[test]
  fn a_query_without_a_match_leaves_the_cursor_where_it_is() {
    let mut state = interaction();

    press(&mut state, &typed("zzz"));
    assert_eq!((state.selected(), state.mode), (1, Mode::Spaces));
    assert_eq!(press(&mut state, &[Key::Enter]), None);
    assert_eq!(press(&mut state, &[Key::Down, Key::Tab]), None);
    assert_eq!((state.selected(), state.mode), (1, Mode::Spaces));
    assert_eq!(state.query, "zzz", "still typing");
  }

  #[test]
  fn enter_in_a_search_goes_to_the_picked_match() {
    let mut state = interaction();

    press(&mut state, &typed("term"));
    assert_eq!(press(&mut state, &[Key::Enter]), Some(focus_window(11)));

    let mut state = interaction();
    press(&mut state, &typed(".exe"));
    assert_eq!(
      press(&mut state, &[Key::Tab, Key::Enter]),
      Some(focus_window(21)),
      "the one stepped to"
    );
  }

  #[test]
  fn matches_are_stepped_through_while_typing_goes_on() {
    let mut state = interaction();
    let at = |state: &Interaction| {
      (state.selected_window, state.search_field().current)
    };

    // All three match ".exe" alike, so in order: 11, 12, 21.
    press(&mut state, &typed("e"));
    assert_eq!(state.search_field().hits, 3);
    assert_eq!(at(&state), (Some(12), Some(0)), "Editor, at its start");

    for (keys, expected) in [
      ([Key::Tab], (Some(11), Some(1))),
      ([Key::Down], (Some(21), Some(2))),
      ([Key::Right], (Some(12), Some(0))),
      ([Key::Alt('k')], (Some(21), Some(2))),
      ([Key::BackTab], (Some(11), Some(1))),
      ([Key::Left], (Some(12), Some(0))),
      ([Key::Up], (Some(21), Some(2))),
      ([Key::Home], (Some(12), Some(0))),
      ([Key::End], (Some(21), Some(2))),
    ] {
      press(&mut state, &keys);
      assert_eq!(at(&state), expected, "{keys:?}");
      assert_eq!(state.query, "e", "the search stays on");
    }

    // Typing on narrows it down from wherever the cursor was.
    press(&mut state, &typed("d"));
    assert_eq!(state.query, "ed");
    assert_eq!(at(&state), (Some(12), Some(0)));
  }

  #[test]
  fn escape_clears_a_query_before_it_closes() {
    let mut state = interaction();

    assert_eq!(press(&mut state, &typed("term")), None);
    assert_eq!(press(&mut state, &[Key::Escape]), None);
    assert_eq!(state.query, "");
    assert_eq!(state.selected_window, Some(11), "the pick stays");

    assert_eq!(
      press(&mut state, &[Key::Escape]),
      Some(OverviewAction::Cancel)
    );
  }

  #[test]
  fn backspace_empties_a_query_before_it_steps_back() {
    let mut state = interaction();

    press(&mut state, &typed("te"));
    press(&mut state, &[Key::Backspace, Key::Backspace]);
    assert_eq!(state.query, "");
    assert_eq!(state.mode, Mode::Windows, "the pick stays");

    // With none left, it steps back as ever, and the arrows move again.
    press(&mut state, &[Key::Backspace]);
    assert_eq!(state.mode, Mode::Spaces);
    press(&mut state, &[Key::Right]);
    assert_eq!(state.selected(), 1);
  }

  #[test]
  fn a_search_while_carrying_aims_the_drop() {
    let mut state = interaction();

    press(&mut state, &[Key::Space, Key::Space]);
    press(&mut state, &typed("e"));
    assert_eq!((state.selected(), state.mode), (0, Mode::Carrying));
    press(&mut state, &[Key::End]);
    assert_eq!(state.selected(), 1, "its last match");
    assert_eq!(state.carried, Some(21));

    press(&mut state, &[Key::Home]);
    assert_eq!(
      press(&mut state, &[Key::Enter]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "1".to_string(),
        monitor: MONITOR,
      })
    );
    assert_eq!(state.query, "", "dropping ends the search");
  }

  #[test]
  fn clicks_go_to_windows_and_workspaces_or_cancel() {
    let workspaces = workspaces();
    let mut state = interaction();

    let window = Hit {
      workspace: Some(0),
      window: Some(11),
      control: None,
      point: None,
    };
    state.mouse_down((10.0, 10.0), window);
    assert_eq!(state.mouse_up(&workspaces), Some(focus_window(11)));

    let card = Hit {
      workspace: Some(2),
      window: None,
      control: None,
      point: None,
    };
    state.mouse_down((10.0, 10.0), card);
    assert_eq!(state.mouse_up(&workspaces), Some(focus_workspace("3")));

    state.mouse_down((10.0, 10.0), Hit::default());
    assert_eq!(state.mouse_up(&workspaces), Some(OverviewAction::Cancel));
  }

  #[test]
  fn arrows_and_the_wheel_nudge_where_they_cant_go() {
    let workspaces = workspaces();
    let mut state = interaction();

    press(&mut state, &[Key::Right]);
    assert_eq!(state.take_nudge(), None);
    press(&mut state, &[Key::Right]);
    assert_eq!(state.selected(), 2);
    assert_eq!(state.take_nudge(), Some(Toward::Right));
    assert_eq!(state.take_nudge(), None, "bumps once");

    state.wheel(false, &workspaces);
    state.wheel(false, &workspaces);
    assert_eq!(state.take_nudge(), None);
    state.wheel(false, &workspaces);
    assert_eq!(state.selected(), 0);
    assert_eq!(state.take_nudge(), Some(Toward::Left));
  }

  #[test]
  fn a_held_card_shows_pressed_until_let_go_or_dragged_from() {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(1),
      window: Some(21),
      control: None,
      point: None,
    };
    let pressed =
      |state: &Interaction| state.focus(0, &workspaces).pressed;

    state.mouse_down((100.0, 100.0), grab);
    assert_eq!(pressed(&state), Some(1));
    state.mouse_up(&workspaces);
    assert_eq!(pressed(&state), None);

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move((400.0, 100.0), grab, true);
    assert_eq!(pressed(&state), None, "dragged from");

    let search = Hit {
      control: Some(Control::Search),
      ..Hit::default()
    };
    state.mouse_up(&workspaces);
    state.mouse_down((10.0, 10.0), search);
    assert_eq!(pressed(&state), None, "not a card");
  }

  #[test]
  fn dragging_a_window_onto_another_card_moves_it() {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(1),
      window: Some(21),
      control: None,
      point: None,
    };

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move((102.0, 101.0), grab, true);
    assert_eq!(state.drag, None, "a little slop stays a click");

    let over = Hit {
      workspace: Some(2),
      window: None,
      control: None,
      point: None,
    };
    assert!(state.mouse_move((400.0, 100.0), over, true));
    assert_eq!(state.floating(), Some(21));
    assert_eq!(state.drop_target(), Some(2));

    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "3".to_string(),
        monitor: MONITOR,
      })
    );
    assert_eq!(state.drag, None);
  }

  #[test]
  fn dropping_a_drag_over_a_window_places_it_beside_that_window() {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(1),
      window: Some(21),
      ..Hit::default()
    };
    let over = |point| Hit {
      workspace: Some(0),
      window: Some(12),
      point: Some(point),
      ..Hit::default()
    };

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move((400.0, 100.0), over((1800, 500)), true);
    assert_eq!(
      state.focus(0, &workspaces).drop_beside,
      Some((12, Toward::Right))
    );
    state.mouse_move((400.0, 100.0), over((1500, 50)), true);
    assert_eq!(
      state.focus(0, &workspaces).drop_beside,
      Some((12, Toward::Up))
    );

    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::PlaceWindow {
        hwnd: 21,
        workspace: "1".to_string(),
        monitor: MONITOR,
        point: Point { x: 1500, y: 50 },
      })
    );
  }

  #[test]
  fn a_window_dragged_within_its_workspace_is_placed_but_not_onto_itself()
  {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(0),
      window: Some(12),
      ..Hit::default()
    };
    let drop_at = |state: &mut Interaction, point| {
      state.mouse_down((100.0, 100.0), grab);
      state.mouse_move(
        (400.0, 100.0),
        Hit {
          point: Some(point),
          ..grab
        },
        true,
      );
      state.mouse_up(&workspaces)
    };

    assert_eq!(
      drop_at(&mut state, (100, 500)),
      Some(OverviewAction::PlaceWindow {
        hwnd: 12,
        workspace: "1".to_string(),
        monitor: MONITOR,
        point: Point { x: 100, y: 500 },
      })
    );
    assert_eq!(drop_at(&mut state, (1500, 500)), None, "onto itself");
  }

  #[test]
  fn dropping_a_drag_on_its_own_card_moves_nothing() {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(1),
      window: Some(21),
      control: None,
      point: None,
    };

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move((300.0, 100.0), grab, true);
    assert_eq!(state.mouse_up(&workspaces), None);
  }

  #[test]
  fn middle_click_closes_a_window() {
    let state = interaction();
    let hit = Hit {
      workspace: Some(0),
      window: Some(12),
      control: None,
      point: None,
    };

    assert_eq!(
      state.middle_click(hit),
      Some(OverviewAction::CloseWindow(12))
    );
    assert_eq!(state.middle_click(Hit::default()), None);
  }

  #[test]
  fn sync_finds_the_matches_among_the_windows_there_now() {
    let old = workspaces();
    let mut state = interaction();
    press(&mut state, &typed("e"));
    assert_eq!(state.search_field().hits, 3);

    // Workspace 1 goes away, along with two of them.
    state.sync(&old, &[screen(MONITOR, old[1..].to_vec())]);
    assert_eq!(state.search_field().hits, 1);
    assert_eq!(press(&mut state, &[Key::Tab]), None);
    assert_eq!((state.selected(), state.selected_window), (0, Some(21)));
  }

  #[test]
  fn sync_keeps_the_selection_by_name() {
    let old = workspaces();
    let mut state = interaction();

    // Carry a window from workspace 1 over to workspace 2, then have
    // workspace 1 go away along with the window.
    press(&mut state, &[Key::Left, Key::Space, Key::Space, Key::Right]);
    let new = old[1..].to_vec();
    state.sync(&old, &[screen(MONITOR, new)]);

    assert_eq!(state.selected(), 0, "still on workspace 2");
    assert_eq!(state.carried, None);
    assert_eq!(state.mode, Mode::Spaces);
  }

  /// `workspaces` with the special workspace, empty, after them.
  fn with_special() -> Vec<OverviewWorkspace> {
    let mut workspaces = workspaces();
    workspaces.push(OverviewWorkspace {
      is_special: true,
      ..card("special")
    });
    workspaces
  }

  #[test]
  fn the_special_workspace_comes_last() {
    let workspaces = with_special();
    let special = workspaces.len() - 1;

    for (layout, back, forward) in [
      (OverviewLayoutMode::Activities, Key::Left, Key::Right),
      (OverviewLayoutMode::Stack, Key::Up, Key::Down),
    ] {
      let mut state = single(&workspaces, layout);
      let mut press = |keys: &[Key]| {
        for key in keys {
          state.key(*key, &workspaces);
        }
        state.selected()
      };

      assert_eq!(press(&[forward, forward]), special);
      assert_eq!(press(&[forward]), special, "the last one");
      assert_eq!(press(&[back]), special - 1);
      assert_eq!(press(&[Key::Home, Key::End]), special);
    }
  }

  #[test]
  fn minus_goes_to_the_special_workspace_like_a_digit() {
    let workspaces = with_special();
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);
    let press = |state: &mut Interaction, keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    assert_eq!(
      press(&mut state, &[Key::Alt('-')]),
      Some(focus_workspace("special"))
    );
    assert_eq!(
      press(&mut state, &[Key::Space, Key::AltShift('-')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
        monitor: MONITOR,
      }),
      "sends the picked window"
    );

    let mut state = single(&workspaces, OverviewLayoutMode::Activities);
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Space, Key::Alt('-')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
        monitor: MONITOR,
      }),
      "drops the carried window"
    );
    assert_eq!(state.mode, Mode::Spaces);

    // During a search too, as it types nothing.
    press(&mut state, &typed("zzz"));
    assert_eq!(
      press(&mut state, &[Key::Alt('-')]),
      Some(focus_workspace("special"))
    );
  }

  #[test]
  fn minus_does_nothing_without_a_special_card() {
    let mut state = interaction();
    assert_eq!(press(&mut state, &[Key::Alt('-')]), None);
  }

  #[test]
  fn digits_by_position_skip_the_special_workspace() {
    let mut workspaces = with_special();
    for (workspace, name) in
      workspaces.iter_mut().zip(["web", "code", "chat"])
    {
      workspace.name = name.to_string();
    }
    workspaces.swap(0, 3);
    let mut state = single(&workspaces, OverviewLayoutMode::Activities);

    assert_eq!(
      state.key(Key::Alt('1'), &workspaces),
      Some(focus_workspace("code"))
    );
    assert_eq!(state.key(Key::Alt('4'), &workspaces), None);
  }

  /// Working area of the first screen of `two_screens`.
  const FIRST: (i32, i32, i32, i32) = (0, 0, 1920, 1040);

  /// `two_screens_at`, side by side.
  fn two_screens() -> Vec<OverviewScreen> {
    two_screens_at(Rect::from_xy(1920, 0, 1920, 1040))
  }

  /// A screen of monitor 1, focused, showing "1" (with "Terminal") and
  /// "2" (with "Editor"), and one of monitor 2 at `second` showing "3"
  /// (with "Browser") and "4"; each with the special workspace's card,
  /// last.
  fn two_screens_at(second: Rect) -> Vec<OverviewScreen> {
    let special = || OverviewWorkspace {
      is_special: true,
      ..card("special")
    };
    let full = Rect::from_xy(0, 0, 1920, 1000);
    let (x, y, width, height) = FIRST;

    vec![
      OverviewScreen {
        is_focused: true,
        rect: Rect::from_xy(x, y, width, height),
        ..screen(
          1,
          vec![
            OverviewWorkspace {
              is_displayed: true,
              windows: vec![window(11, "Terminal", full.clone())],
              ..card("1")
            },
            OverviewWorkspace {
              windows: vec![window(12, "Editor", full.clone())],
              ..card("2")
            },
            special(),
          ],
        )
      },
      OverviewScreen {
        rect: second,
        ..screen(
          2,
          vec![
            OverviewWorkspace {
              is_displayed: true,
              windows: vec![window(31, "Browser", full)],
              ..card("3")
            },
            card("4"),
            special(),
          ],
        )
      },
    ]
  }

  fn press_on(
    state: &mut Interaction,
    workspaces: &[OverviewWorkspace],
    keys: &[Key],
  ) -> Option<OverviewAction> {
    keys.iter().fold(None, |_, key| state.key(*key, workspaces))
  }

  #[test]
  fn each_screen_starts_on_its_displayed_workspace() {
    let state = Interaction::new(
      &two_screens(),
      OverviewLayoutMode::Activities,
      6.0,
    );

    assert_eq!(state.active, 0, "the focused screen has the keyboard");
    assert_eq!(state.selected(), 0);
    assert_eq!(state.screens[1].cards, 3..6);
    assert_eq!(state.screens[1].shown, 3);
  }

  #[test]
  fn arrows_go_on_to_the_next_screen_past_the_end_of_the_row() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(&mut state, &workspaces, &[Key::Right, Key::Right]);
    assert_eq!((state.active, state.selected()), (0, 2), "its special");
    press_on(&mut state, &workspaces, &[Key::Right]);
    assert_eq!((state.active, state.selected()), (1, 3), "past its end");
    assert_eq!(state.screens[0].shown, 2, "the left one stays put");

    press_on(
      &mut state,
      &workspaces,
      &[Key::Right, Key::Right, Key::Right],
    );
    assert_eq!(state.selected(), 5, "nothing further right");

    press_on(&mut state, &workspaces, &[Key::Home, Key::Left]);
    assert_eq!((state.active, state.selected()), (0, 2));

    press_on(&mut state, &workspaces, &[Key::Home, Key::End]);
    assert_eq!(state.selected(), 2, "the end of its own row");
  }

  #[test]
  fn arrows_only_cross_toward_a_screen_that_way() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(&mut state, &workspaces, &[Key::Left, Key::Up]);
    assert_eq!((state.active, state.selected()), (0, 0));

    press_on(&mut state, &workspaces, &[Key::Down, Key::Down]);
    assert_eq!((state.active, state.selected()), (0, 0), "none below");
  }

  #[test]
  fn screens_out_of_line_are_not_crossed_to() {
    let screens = two_screens_at(Rect::from_xy(1920, 1040, 1920, 1040));
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(&mut state, &workspaces, &[Key::End, Key::Right]);
    assert_eq!((state.active, state.selected()), (0, 2));
  }

  #[test]
  fn stacked_screens_are_crossed_up_and_down() {
    let screens = two_screens_at(Rect::from_xy(0, 1040, 1920, 1040));
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(&mut state, &workspaces, &[Key::End, Key::Right]);
    assert_eq!((state.active, state.selected()), (0, 2), "nothing right");

    // Down into the windows, then off their bottom edge onto the screen
    // below, onto what that screen shows.
    press_on(&mut state, &workspaces, &[Key::Home, Key::Down]);
    assert_eq!(state.mode, Mode::Windows);
    press_on(&mut state, &workspaces, &[Key::Down]);
    assert_eq!((state.active, state.selected()), (1, 3));
    assert_eq!(state.mode, Mode::Spaces);

    press_on(&mut state, &workspaces, &[Key::Up]);
    assert_eq!((state.active, state.selected()), (0, 0));
  }

  #[test]
  fn the_stack_crosses_sideways_onto_the_shown_workspace() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Stack, 6.0);

    // Sideways is the next screen, whether browsing or among windows.
    press_on(&mut state, &workspaces, &[Key::Down, Key::Up, Key::Right]);
    assert_eq!((state.active, state.selected()), (1, 3));
    assert_eq!(state.mode, Mode::Spaces);

    press_on(&mut state, &workspaces, &[Key::Down, Key::Left]);
    assert_eq!((state.active, state.selected()), (0, 0), "its shown one");

    press_on(&mut state, &workspaces, &[Key::Space, Key::Right]);
    assert_eq!((state.active, state.selected()), (1, 4));
    assert_eq!(state.mode, Mode::Spaces);

    press_on(&mut state, &workspaces, &[Key::Right]);
    assert_eq!((state.active, state.selected()), (1, 4), "none further");
    assert_eq!(state.take_nudge(), Some(Toward::Right));
  }

  #[test]
  fn stacked_screens_are_crossed_past_the_ends_of_the_stack() {
    let screens = two_screens_at(Rect::from_xy(0, 1040, 1920, 1040));
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Stack, 6.0);

    // Off the windows of the last workspace with any, down its stack and
    // past its end onto the start of the one below.
    press_on(&mut state, &workspaces, &[Key::Down, Key::Space, Key::Down]);
    assert_eq!((state.active, state.selected()), (0, 2));
    press_on(&mut state, &workspaces, &[Key::Down]);
    assert_eq!((state.active, state.selected()), (1, 3));

    press_on(&mut state, &workspaces, &[Key::Space, Key::Up]);
    assert_eq!((state.active, state.selected()), (0, 2), "its last");
  }

  #[test]
  fn only_the_screen_with_the_keyboard_marks_it() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    assert!(state.focus(0, &workspaces).marks_keyboard);
    assert!(!state.focus(1, &workspaces).marks_keyboard);

    state.activate(1);
    assert!(!state.focus(0, &workspaces).marks_keyboard);
    assert!(state.focus(1, &workspaces).marks_keyboard);

    let alone = single(&workspaces[..3], OverviewLayoutMode::Activities);
    assert!(!alone.focus(0, &workspaces).marks_keyboard, "one screen");
  }

  #[test]
  fn the_wheel_stays_on_its_screen() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    for _ in 0..4 {
      state.wheel(true, &workspaces);
    }
    assert_eq!((state.active, state.selected()), (0, 2));
  }

  #[test]
  fn a_carried_window_crosses_to_the_other_screen() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    assert_eq!(
      press_on(
        &mut state,
        &workspaces,
        &[
          Key::Space,
          Key::Space,
          Key::Right,
          Key::Right,
          Key::Right,
          Key::Space,
        ],
      ),
      Some(OverviewAction::MoveWindow {
        hwnd: 11,
        workspace: "3".to_string(),
        monitor: 2,
      })
    );
  }

  #[test]
  fn search_finds_windows_on_every_screen_and_follows_them() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(&mut state, &workspaces, &typed("brow"));

    assert_eq!(
      state.focus(1, &workspaces).search.hits,
      1,
      "on every screen"
    );

    // Typing alone took the keyboard to its screen.
    assert_eq!((state.active, state.selected()), (1, 3));
    assert_eq!(state.selected_window, Some(31));
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Enter]),
      Some(OverviewAction::FocusWindow {
        hwnd: 31,
        monitor: 2,
      })
    );
  }

  #[test]
  fn special_windows_count_once_in_a_search() {
    let mut screens = two_screens();
    for screen in &mut screens {
      let special = screen.workspaces.last_mut().expect("special");
      special.windows =
        vec![window(51, "Notes", Rect::from_xy(0, 0, 1920, 1000))];
    }
    screens[1].workspaces[0].is_displayed = false;
    screens[1].workspaces[2].is_displayed = true;
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);
    press_on(&mut state, &workspaces, &typed("notes"));

    assert_eq!(state.search_field().hits, 1);
    assert_eq!(
      (state.active, state.selected(), state.selected_window),
      (1, 5, Some(51)),
      "on the screen showing it"
    );
  }

  #[test]
  fn the_search_field_shows_what_is_typed() {
    let mut state = interaction();
    let typing_e = SearchField {
      query: "e".to_string(),
      hits: 3,
      current: Some(0),
    };
    assert_eq!(state.search_field(), SearchField::default());

    press(&mut state, &[Key::Text('e')]);
    assert_eq!(state.search_field(), typing_e);

    // Which match is picked shows as they are stepped through.
    press(&mut state, &[Key::Down]);
    assert_eq!(
      state.search_field(),
      SearchField {
        current: Some(1),
        ..typing_e.clone()
      }
    );

    // With the cursor on a window that is no match, e.g. one pointed
    // at, none of them is picked.
    press(&mut state, &typed("d"));
    assert_eq!(state.selected_window, Some(12), "Editor alone");
    let terminal = Hit {
      workspace: Some(0),
      window: Some(11),
      ..Hit::default()
    };
    state.mouse_move((10.0, 10.0), terminal, false);
    assert_eq!(
      state.search_field(),
      SearchField {
        query: "ed".to_string(),
        hits: 1,
        current: None,
      }
    );

    press(&mut state, &[Key::Escape]);
    assert_eq!(state.search_field(), SearchField::default());
  }

  #[test]
  fn f1_toggles_the_shortcut_sheet() {
    let workspaces = workspaces();
    let mut state = interaction();

    press(&mut state, &[Key::F1]);
    assert!(state.focus(0, &workspaces).shortcuts.is_some());

    // Escape closes the sheet, not the overview.
    assert_eq!(press(&mut state, &[Key::Escape]), None);
    assert_eq!(state.focus(0, &workspaces).shortcuts, None);

    press(&mut state, &[Key::F1, Key::F1]);
    assert!(!state.shows_shortcuts);

    // Any other key closes it and does what it does.
    press(&mut state, &[Key::F1, Key::Right]);
    assert!(!state.shows_shortcuts);
    assert_eq!(state.selected(), 2);
  }

  #[test]
  fn clicking_the_search_line_searches_or_shows_the_shortcuts() {
    let workspaces = workspaces();
    let mut state = interaction();
    let click = |state: &mut Interaction, hit: Hit| {
      state.mouse_down((0.0, 0.0), hit);
      state.mouse_up(&workspaces)
    };
    let control = |control| Hit {
      control: Some(control),
      ..Hit::default()
    };

    assert_eq!(click(&mut state, control(Control::Shortcuts)), None);
    assert!(state.shows_shortcuts);
    assert_eq!(click(&mut state, control(Control::Shortcuts)), None);
    assert!(!state.shows_shortcuts);

    // A click on the field changes nothing: typing goes to it anyway.
    press(&mut state, &[Key::Text('e')]);
    let before = state.search_field();
    assert_eq!(click(&mut state, control(Control::Search)), None);
    assert_eq!(state.search_field(), before);

    // While the sheet is open, a click anywhere only closes it.
    press(&mut state, &[Key::Escape, Key::F1]);
    assert_eq!(click(&mut state, Hit::default()), None);
    assert!(!state.shows_shortcuts);
    assert_eq!(
      click(&mut state, Hit::default()),
      Some(OverviewAction::Cancel)
    );
  }

  #[test]
  fn picks_carry_the_monitor_they_were_picked_on() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);
    let focus = |name: &str, monitor| OverviewAction::FocusWorkspace {
      name: name.to_string(),
      monitor,
    };

    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Alt('-')]),
      Some(focus("special", 1))
    );
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Alt('4')]),
      Some(focus("4", 2)),
      "where its card is"
    );

    state.activate(1);
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Alt('-')]),
      Some(focus("special", 2))
    );

    state.digit_workspaces = vec!["7".to_string()];
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Alt('7')]),
      Some(focus("7", 2)),
      "without a card, where the keyboard is"
    );
  }

  #[test]
  fn digits_by_position_count_on_the_screen_with_the_keyboard() {
    let mut screens = two_screens();
    for (workspace, name) in screens
      .iter_mut()
      .flat_map(|screen| &mut screen.workspaces)
      .filter(|workspace| !workspace.is_special)
      .zip(["web", "code", "chat", "mail"])
    {
      workspace.name = name.to_string();
    }
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    state.activate(1);
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Alt('2')]),
      Some(OverviewAction::FocusWorkspace {
        name: "mail".to_string(),
        monitor: 2,
      })
    );
  }

  #[test]
  fn a_window_dragged_onto_another_screen_moves_there() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);
    let grab = Hit {
      workspace: Some(0),
      window: Some(11),
      control: None,
      point: None,
    };

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move(
      (2400.0, 100.0),
      Hit {
        workspace: Some(4),
        window: None,
        control: None,
        point: None,
      },
      true,
    );
    assert_eq!(state.drop_target(), Some(4));
    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::MoveWindow {
        hwnd: 11,
        workspace: "4".to_string(),
        monitor: 2,
      })
    );
  }

  #[test]
  fn a_special_window_dropped_on_another_special_card_moves_nothing() {
    let mut screens = two_screens();
    for screen in &mut screens {
      let special = screen.workspaces.last_mut().expect("special");
      special.windows =
        vec![window(51, "Notes", Rect::from_xy(0, 0, 1920, 1000))];
    }
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    state.mouse_down(
      (100.0, 100.0),
      Hit {
        workspace: Some(2),
        window: Some(51),
        control: None,
        point: None,
      },
    );
    state.mouse_move(
      (2400.0, 900.0),
      Hit {
        workspace: Some(5),
        window: None,
        control: None,
        point: None,
      },
      true,
    );
    assert_eq!(state.mouse_up(&workspaces), None);
  }

  #[test]
  fn activating_a_screen_leaves_picking_a_window() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(&mut state, &workspaces, &[Key::Down]);
    assert_eq!(state.mode, Mode::Windows);

    assert!(state.activate(1));
    assert_eq!((state.active, state.selected()), (1, 3));
    assert_eq!(state.mode, Mode::Spaces);
    assert_eq!(state.selected_window, None);
    assert!(!state.activate(1), "already there");
  }

  #[test]
  fn each_screen_sees_the_interaction_in_its_own_indices() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Activities, 6.0);

    press_on(
      &mut state,
      &workspaces,
      &[
        Key::Space,
        Key::Space,
        Key::Right,
        Key::Right,
        Key::Right,
        Key::Right,
      ],
    );
    let (left, right) =
      (state.focus(0, &workspaces), state.focus(1, &workspaces));

    assert!(!left.is_active && right.is_active);
    assert_eq!((left.selected, left.mode), (2, Mode::Spaces));
    assert_eq!((right.selected, right.mode), (1, Mode::Carrying));
    assert_eq!((left.drop, right.drop), (None, Some(1)));
    assert_eq!(left.floating, Some(11), "lifted on every screen");
    assert_eq!(left.search, SearchField::default());
  }

  #[test]
  fn sync_keeps_each_screen_on_its_workspace() {
    let old_screens = two_screens();
    let old = all_workspaces(&old_screens);
    let mut state =
      Interaction::new(&old_screens, OverviewLayoutMode::Activities, 6.0);
    press_on(
      &mut state,
      &old,
      &[Key::Right, Key::Right, Key::Right, Key::Right],
    );
    assert_eq!(state.selected(), 4);

    // A workspace shows up on the left, moving the right one's cards.
    let mut screens = two_screens();
    screens[0].workspaces.insert(0, card("0"));
    state.sync(&old, &screens);

    assert_eq!((state.active, state.selected()), (1, 5), "still on 4");
    assert_eq!(state.screens[0].shown, 3, "still on the special one");

    // The right monitor goes away.
    let old = all_workspaces(&screens);
    screens.truncate(1);
    state.sync(&old, &screens);
    assert_eq!((state.active, state.selected()), (0, 3));
  }
}
