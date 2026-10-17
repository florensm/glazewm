//! What the overview is doing, and how keys and the mouse change it, kept
//! free of drawing so it can be tested.
//!
//! The keyboard moves through three steps, one space bar at a time:
//! browsing workspaces, picking a window inside one, and carrying a window
//! to another workspace. The mouse does the same directly: click to go,
//! drag to move.
//!
//! Each monitor has a screen of its own, and the keyboard is on one of
//! them at a time. Workspaces are indexed across every screen, screen
//! after screen, so a window can be carried, dragged or searched for
//! anywhere.

use std::ops::Range;

use crate::{
  overview_layout::{GridCells, RectF},
  OverviewAction, OverviewLayoutMode, OverviewScreen, OverviewWindow,
  OverviewWorkspace, Rect,
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
  Tab,
  Escape,

  /// A digit key, 0 to 9.
  Digit(u8),

  /// A typed character other than whitespace.
  Text(char),
}

/// A way to move the cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Toward {
  Left,
  Right,
  Up,
  Down,
}

impl Toward {
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
}

/// How the key hints line is colored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HintTone {
  Quiet,
  Accent,
  Search,
}

/// A window being dragged with the mouse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drag {
  pub hwnd: isize,

  /// Where the cursor is, in physical screen coordinates.
  pub position: (f32, f32),

  /// Workspace it would be dropped on.
  pub target: Option<usize>,
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

  /// Card of its row of workspaces the row is centered on: the shown
  /// one, or while the special workspace's dock is shown, the one it was
  /// entered from, which moving up goes back to.
  pub row_selected: usize,

  /// Where its grid puts each of its cards, counted from its first, for
  /// moving a row up or down.
  pub grid: GridCells,
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

  /// Window held while picking a workspace for it.
  pub carried: Option<isize>,

  /// Whether typed characters go to the search query.
  pub searching: bool,

  /// Search query, over every screen's windows; its matches stay
  /// highlighted after typing ends.
  pub query: String,

  match_index: Option<usize>,

  /// Workspace under the cursor.
  pub hover: Option<usize>,

  press: Option<Press>,
  pub drag: Option<Drag>,

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
    let mut interaction = Self {
      screens: cursors(screens),
      active: screens
        .iter()
        .position(|screen| screen.is_focused)
        .unwrap_or(0),
      mode: Mode::Spaces,
      layout,
      selected_window: None,
      carried: None,
      searching: false,
      query: String::new(),
      match_index: None,
      hover: None,
      press: None,
      drag: None,
      drag_threshold,
      digit_workspaces: Vec::new(),
      nav: Vec::new(),
    };

    interaction.center_rows(&all_workspaces(screens));
    interaction
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
        Some((remap(old.shown), remap(old.row_selected), old.grid.clone()))
      })
      .collect::<Vec<_>>();
    let selected = remap(self.selected());
    let hover = self.hover.and_then(remap);
    let drag_target =
      self.drag.and_then(|drag| drag.target).and_then(remap);

    for (cursor, kept) in cursors.iter_mut().zip(kept) {
      let Some((shown, row_selected, grid)) = kept else {
        continue;
      };
      cursor.shown = shown.unwrap_or(cursor.shown);
      cursor.row_selected = row_selected.unwrap_or(cursor.shown);
      cursor.grid = grid;
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
    self.center_rows(&new);

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
  pub fn activate(
    &mut self,
    screen: usize,
    workspaces: &[OverviewWorkspace],
  ) -> bool {
    let shown = self
      .screens
      .get(screen)
      .filter(|cursor| {
        screen != self.active && cursor.cards.contains(&cursor.shown)
      })
      .map(|cursor| cursor.shown);

    if let Some(shown) = shown {
      self.select(shown, workspaces);
    }
    shown.is_some()
  }

  /// Shows workspace `index` on its screen, leaving the keyboard where it
  /// is: e.g. the one the overview zooms into there as it closes.
  pub fn show(&mut self, index: usize, workspaces: &[OverviewWorkspace]) {
    match self.screen_of(index) {
      Some(screen) if screen == self.active => {
        self.select(index, workspaces);
      }
      Some(screen) => self.set_shown(screen, index, workspaces),
      None => {}
    }
  }

  /// Acts on `key`.
  pub fn key(
    &mut self,
    key: Key,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    if self.searching {
      self.search_key(key, workspaces);
      return None;
    }

    match key {
      Key::Escape => return Some(OverviewAction::Cancel),
      Key::Text('/') => {
        self.searching = true;
        self.query.clear();
        self.match_index = None;
      }
      Key::Text('n') => self.goto_match(1, workspaces),
      Key::Text('N') => self.goto_match(-1, workspaces),
      Key::Text('x' | 'X') => {
        let hwnd =
          self.selected_window.filter(|_| self.mode == Mode::Windows);
        if let Some(hwnd) = hwnd {
          self.selected_window = None;
          return Some(OverviewAction::CloseWindow(hwnd));
        }
      }
      Key::Tab => self.set_layout(self.layout.next(), workspaces),
      Key::Space => return self.space(workspaces),
      Key::Backspace => match self.mode {
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
      Key::Left | Key::Text('h') => self.arrow(Toward::Left, workspaces),
      Key::Right | Key::Text('l') => self.arrow(Toward::Right, workspaces),
      Key::Up | Key::Text('k') => self.arrow(Toward::Up, workspaces),
      Key::Down | Key::Text('j') => self.arrow(Toward::Down, workspaces),
      Key::Home => self.edge(false, workspaces),
      Key::End => self.edge(true, workspaces),
      Key::Enter => return self.enter(workspaces),
      Key::Digit(digit) => {
        let name = digit_target(
          digit,
          workspaces,
          self.active_cards(),
          &self.digit_workspaces,
        )?;
        let monitor = self.monitor_for(&name, workspaces);
        return self.go_to(name, monitor, workspaces);
      }
      Key::Text('s') => {
        let special = self
          .active_cards()
          .find(|index| is_special(workspaces, *index))?;
        let name = workspaces.get(special)?.name.clone();
        return self.go_to(
          name,
          self.monitor_of(Some(special)),
          workspaces,
        );
      }
      Key::Text(_) => {}
    }

    None
  }

  /// Lays the cards out as `layout`.
  pub fn set_layout(
    &mut self,
    layout: OverviewLayoutMode,
    workspaces: &[OverviewWorkspace],
  ) {
    self.layout = layout;
    self.center_rows(workspaces);

    // The grid is too small to pick a window in.
    if layout == OverviewLayoutMode::Grid && self.mode == Mode::Windows {
      self.mode = Mode::Spaces;
      self.selected_window = None;
    }
  }

  /// Goes straight to workspace `name`, picked on `monitor`'s screen:
  /// switches to it, sends the picked window there, or drops the carried
  /// one there.
  fn go_to(
    &mut self,
    name: String,
    monitor: u128,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    match self.mode {
      Mode::Spaces => {
        Some(OverviewAction::FocusWorkspace { name, monitor })
      }
      Mode::Windows => match self.selected_window {
        Some(hwnd) => self.send(hwnd, name, monitor, workspaces),
        None => Some(OverviewAction::FocusWorkspace { name, monitor }),
      },
      Mode::Carrying => {
        let action = self
          .carried
          .and_then(|hwnd| move_action(hwnd, name, monitor, workspaces));
        self.put_down();
        action
      }
    }
  }

  fn search_key(&mut self, key: Key, workspaces: &[OverviewWorkspace]) {
    match key {
      Key::Escape => {
        self.searching = false;
        self.query.clear();
      }
      Key::Backspace => {
        if self.query.pop().is_none() {
          self.searching = false;
        }
      }
      Key::Enter => {
        self.searching = false;
        self.match_index = None;
        self.goto_match(1, workspaces);
      }
      Key::Space => self.query.push(' '),
      Key::Text(text) => self.query.push(text),
      // Digits arrive as text too.
      _ => {}
    }
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
    self.searching = false;
    self.query.clear();
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
  fn select(&mut self, index: usize, workspaces: &[OverviewWorkspace]) {
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
    self.set_shown(screen, index, workspaces);
  }

  /// Shows workspace `index` on `screen`, and centers its row on it.
  fn set_shown(
    &mut self,
    screen: usize,
    index: usize,
    workspaces: &[OverviewWorkspace],
  ) {
    let layout = self.layout;
    if let Some(cursor) = self.screens.get_mut(screen) {
      cursor.shown = index;
      cursor.row_selected = row_card(
        workspaces,
        &cursor.cards,
        index,
        Some(cursor.row_selected),
        layout,
      )
      .unwrap_or(cursor.cards.start);
    }
  }

  /// Centers each screen's row anew, e.g. for another layout.
  fn center_rows(&mut self, workspaces: &[OverviewWorkspace]) {
    for screen in 0..self.screens.len() {
      let shown = self.screens[screen].shown;
      self.set_shown(screen, shown, workspaces);
    }
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

  /// Moves along the row of workspaces of the screen with the keyboard,
  /// which the special workspace's dock below it isn't part of; from the
  /// dock, back to the row.
  fn step(&mut self, delta: isize, workspaces: &[OverviewWorkspace]) {
    let cards = self.active_cards();
    let selected = self.selected();

    if is_docked(workspaces, selected, self.layout) {
      let row_selected = self
        .screens
        .get(self.active)
        .map_or(selected, |screen| screen.row_selected);
      let back =
        row_card(workspaces, &cards, row_selected, None, self.layout);
      if let Some(index) = back {
        self.select(index, workspaces);
      }
      return;
    }

    let row = row_indices(workspaces, &cards, self.layout);
    let Some(position) = row.iter().position(|i| *i == selected) else {
      return;
    };

    if let Some(index) =
      position.checked_add_signed(delta).and_then(|i| row.get(i))
    {
      self.select(*index, workspaces);
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
  /// row nearest this one when the row runs that way, else onto the
  /// workspace it shows.
  fn cross(&mut self, toward: Toward, workspaces: &[OverviewWorkspace]) {
    let Some(cursor) = self
      .neighbour(toward)
      .and_then(|screen| self.screens.get(screen))
    else {
      return;
    };

    let row = row_indices(workspaces, &cursor.cards, self.layout);
    let (back, forward, _) = arrows(self.layout);
    let target = if toward == forward {
      row.first().copied()
    } else if toward == back {
      row.last().copied()
    } else {
      Some(cursor.shown)
        .filter(|shown| cursor.cards.contains(shown))
        .or_else(|| row.first().copied())
    };

    if let Some(index) = target {
      self.select(index, workspaces);
    }
  }

  /// Moves the window cursor, or else the workspace cursor, `toward`.
  /// With nowhere to go that way on its screen, on to the screen that
  /// way.
  fn arrow(&mut self, toward: Toward, workspaces: &[OverviewWorkspace]) {
    if self.mode == Mode::Windows {
      self.move_window_cursor(toward, workspaces);
      return;
    }

    let before = (self.selected(), self.mode);
    let (back, forward, inside) = arrows(self.layout);
    if toward == back {
      self.step(-1, workspaces);
    } else if toward == forward {
      self.step(1, workspaces);
    } else if Some(toward) == inside {
      if self.mode == Mode::Spaces {
        self.enter_windows(workspaces);
      }
    } else if toward == Toward::Up {
      self.vertical(-1, workspaces);
    } else if toward == Toward::Down {
      self.vertical(1, workspaces);
    }

    if (self.selected(), self.mode) == before {
      self.cross(toward, workspaces);
    }
  }

  /// Moves between the carousel's row and the special workspace's dock,
  /// or between the grid's rows, on the screen with the keyboard.
  fn vertical(&mut self, delta: isize, workspaces: &[OverviewWorkspace]) {
    let selected = self.selected();
    let cards = self.active_cards();
    let Some(screen) = self.screens.get(self.active) else {
      return;
    };

    // Up from the dock goes back to where it was entered from. Down onto
    // the dock is from the carousel, or from the grid's last row, below
    // which `GridCells` puts it.
    let target = if is_special(workspaces, selected) {
      (delta < 0)
        .then(|| {
          row_card(
            workspaces,
            &cards,
            screen.row_selected,
            None,
            self.layout,
          )
        })
        .flatten()
    } else if self.layout == OverviewLayoutMode::Grid {
      selected
        .checked_sub(cards.start)
        .and_then(|card| screen.grid.vertical_neighbour(card, delta))
        .map(|card| cards.start + card)
    } else {
      cards
        .clone()
        .find(|index| is_special(workspaces, *index))
        .filter(|_| delta > 0 && self.layout.docks_special())
    };

    if let Some(index) = target {
      self.select(index, workspaces);
    }
  }

  /// Moves the window cursor to the nearest window `toward`; off the edge
  /// the windows were entered from, back to the workspaces, and off any
  /// other, on to the screen that way.
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

    if let Some(hwnd) = next {
      self.selected_window = Some(hwnd);
    } else if Some(toward) == arrows(self.layout).2.map(Toward::opposite) {
      self.mode = Mode::Spaces;
    } else {
      self.cross(toward, workspaces);
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

    // Picking a window is a zoomed-in thing; the grid is too small to aim
    // at one.
    if self.layout == OverviewLayoutMode::Grid {
      self.set_layout(OverviewLayoutMode::Carousel, workspaces);
    }
    self.mode = Mode::Windows;
    self.selected_window = Some(hwnd);
  }

  /// Where the selected workspace's windows show, first to last: as laid
  /// out, or else where they are on screen.
  fn nav(&self, workspaces: &[OverviewWorkspace]) -> Vec<(isize, RectF)> {
    if !self.nav.is_empty() {
      return self.nav.clone();
    }

    workspaces
      .get(self.selected())
      .map_or_else(Vec::new, |workspace| {
        ordered(workspace)
          .into_iter()
          .map(|window| (window.hwnd, RectF::from_rect(&window.rect)))
          .collect()
      })
  }

  /// Selects the first (or last) workspace of the screen with the
  /// keyboard, or window while picking one.
  fn edge(&mut self, is_last: bool, workspaces: &[OverviewWorkspace]) {
    if self.mode == Mode::Windows {
      let nav = self.nav(workspaces);
      let window = if is_last { nav.last() } else { nav.first() };
      if let Some((hwnd, _)) = window {
        self.selected_window = Some(*hwnd);
      }
    } else {
      let row = row_indices(workspaces, &self.active_cards(), self.layout);
      let index = if is_last { row.last() } else { row.first() };
      if let Some(index) = index {
        self.select(*index, workspaces);
      }
    }
  }

  /// Search matches on every screen, best first, then in order. The
  /// special workspace's windows count once, on the screen showing it if
  /// any does, else on the first.
  pub fn matches<'a>(
    &self,
    workspaces: &'a [OverviewWorkspace],
  ) -> Vec<(usize, &'a OverviewWindow)> {
    let special = home_special(workspaces);

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
        ordered(workspace).into_iter().chain(unshown).filter_map(
          move |window| {
            let score = match_score(window, &self.query)?;
            Some((score, index, window))
          },
        )
      })
      .collect::<Vec<_>>();

    matches.sort_by_key(|(score, ..)| *score);
    matches
      .into_iter()
      .map(|(_, index, window)| (index, window))
      .collect()
  }

  /// Selects the next (or previous) search match, on whichever screen it
  /// is.
  fn goto_match(
    &mut self,
    delta: isize,
    workspaces: &[OverviewWorkspace],
  ) {
    let matches = self.matches(workspaces);
    if matches.is_empty() {
      return;
    }

    let len = isize::try_from(matches.len()).unwrap_or(isize::MAX);
    let next = match self.match_index {
      Some(index) => isize::try_from(index)
        .unwrap_or(0)
        .saturating_add(delta)
        .rem_euclid(len),
      None => {
        if delta > 0 {
          0
        } else {
          len - 1
        }
      }
    };
    let next = usize::try_from(next).unwrap_or(0);
    self.match_index = Some(next);

    let Some((index, window)) = matches.get(next) else {
      return;
    };
    self.select(*index, workspaces);

    // While carrying, a search only aims the drop; selecting the match
    // would make the space bar pick it up instead.
    if self.mode != Mode::Carrying {
      self.mode = Mode::Windows;
      self.selected_window = Some(window.hwnd);
    }
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

    // Pointing at a window of the shown workspace picks it, as the arrows
    // would.
    let pointed = hit.window.filter(|_| {
      self.layout.shows_before_going()
        && hit.workspace == Some(self.selected())
        && self.mode != Mode::Carrying
        && !is_left_down
    });
    if let Some(hwnd) = pointed {
      if self.mode != Mode::Windows || self.selected_window != Some(hwnd) {
        self.mode = Mode::Windows;
        self.selected_window = Some(hwnd);
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
    });
    true
  }

  /// Ends a click or a drag.
  pub fn mouse_up(
    &mut self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let press = self.press.take()?;

    // Dropped on a card of the workspace it is on, or on nothing: no
    // move.
    if let Some(drag) = self.drag.take() {
      return drag.target.and_then(|index| {
        let to = workspaces.get(index)?;
        let monitor = self.monitor_of(Some(index));
        move_action(drag.hwnd, to.name.clone(), monitor, workspaces)
      });
    }

    // A workspace not shown yet is shown first.
    let other = press.hit.workspace.filter(|index| {
      self.layout.shows_before_going() && *index != self.selected()
    });
    if let Some(index) = other {
      self.select(index, workspaces);
      return None;
    }

    match press.hit {
      Hit {
        window: Some(hwnd),
        workspace,
      } => Some(OverviewAction::FocusWindow {
        hwnd,
        monitor: self.monitor_of(workspace),
      }),
      Hit {
        workspace: Some(index),
        ..
      } => {
        self.select(index, workspaces);
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
    self.step(if is_down { 1 } else { -1 }, workspaces);
  }

  pub fn mouse_leave(&mut self) {
    self.hover = None;
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
      row_selected: local(cursor.row_selected).unwrap_or(0),
      selected_window: self.selected_window.filter(|_| is_active),
      hover: self.hover.and_then(local),
      floating: self.floating(),
      drop: self.drop_target().and_then(local),
      is_dragging: self.drag.is_some(),
      marks_keyboard: is_active && self.screens.len() > 1,
      query: self.query.clone(),
      hint: self.screen_hint(screen, workspaces),
    }
  }

  /// The hints line of `screen`: the keys for the current step on the
  /// screen with the keyboard, and the search on every screen.
  pub fn screen_hint(
    &self,
    screen: usize,
    workspaces: &[OverviewWorkspace],
  ) -> Option<(String, HintTone)> {
    if screen == self.active {
      Some(self.hint(workspaces))
    } else {
      (self.searching || !self.query.is_empty())
        .then(|| self.search_hint(workspaces))
    }
  }

  /// The keys for the current step, shown under the cards.
  pub fn hint(
    &self,
    workspaces: &[OverviewWorkspace],
  ) -> (String, HintTone) {
    if self.searching || !self.query.is_empty() {
      return self.search_hint(workspaces);
    }

    let title = |hwnd: Option<isize>| window_title(workspaces, hwnd);

    if let Some(drag) = &self.drag {
      return (
        format!("drop \"{}\" on a workspace", title(Some(drag.hwnd))),
        HintTone::Accent,
      );
    }

    let (back, forward, inside) = arrows(self.layout);
    let workspace_arrows = match (back, forward) {
      _ if self.layout == OverviewLayoutMode::Grid => "arrows",
      (Toward::Up, Toward::Down) => "\u{2191}\u{2193}",
      _ => "\u{2190}\u{2192}",
    };

    match self.mode {
      Mode::Carrying => (
        format!(
          "carrying \"{}\"   \u{b7}   {workspace_arrows} or / to pick a workspace   \u{b7}   space to drop   \u{b7}   1\u{2013}9 or s drop on that one   \u{b7}   \u{232b} back",
          title(self.carried)
        ),
        HintTone::Accent,
      ),
      Mode::Windows => {
        let back = inside.map_or_else(
          || "\u{232b} back".to_string(),
          |inside| format!("{} workspaces", arrow_glyph(inside.opposite())),
        );

        (
          format!(
            "arrows pick a window   \u{b7}   \u{23ce} go to it   \u{b7}   space to lift it   \u{b7}   1\u{2013}9 or s send it there   \u{b7}   x close   \u{b7}   {back}"
          ),
          HintTone::Quiet,
        )
      }
      Mode::Spaces => {
        let inside = match inside {
          Some(inside) => format!("{} its windows", arrow_glyph(inside)),
          None if self.layout == OverviewLayoutMode::Grid => {
            "drag windows between them".to_string()
          }
          None => "space to look inside".to_string(),
        };

        (
          format!(
            "{workspace_arrows} workspaces   \u{b7}   {inside}   \u{b7}   s special   \u{b7}   / search   \u{b7}   \u{23ce} go there   \u{b7}   tab {}   \u{b7}   esc close",
            self.layout.next().name()
          ),
          HintTone::Quiet,
        )
      }
    }
  }

  /// The search query and its hits, on every screen's hints line.
  fn search_hint(
    &self,
    workspaces: &[OverviewWorkspace],
  ) -> (String, HintTone) {
    let count = self.matches(workspaces).len();
    let mut text = format!(
      "/{}{}   {count} {}",
      self.query,
      if self.searching { "\u{258f}" } else { "" },
      if count == 1 { "hit" } else { "hits" },
    );

    if count > 0 {
      text.push_str(
        "   \u{b7}   \u{23ce} best match   \u{b7}   n / N to walk them",
      );
    }
    if self.carried.is_some() {
      text.push_str("   \u{b7}   space drops \"");
      text.push_str(&window_title(workspaces, self.carried));
      text.push_str("\" here");
    }

    (text, HintTone::Search)
  }
}

/// The interaction as one screen shows it, in that screen's own card
/// indices.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScreenFocus {
  pub layout: OverviewLayoutMode,

  /// Whether the keyboard is on this screen.
  pub is_active: bool,

  /// Where the keyboard is in the flow, while it is on this screen; else
  /// browsing.
  pub mode: Mode,

  /// Workspace it shows.
  pub selected: usize,

  /// Card its row is centered on.
  pub row_selected: usize,

  /// Window picked on it.
  pub selected_window: Option<isize>,

  /// Workspace under the cursor.
  pub hover: Option<usize>,

  /// Window lifted by the mouse or the keyboard, from whichever screen.
  pub floating: Option<isize>,

  /// Workspace the floating window would land on.
  pub drop: Option<usize>,

  /// Whether a window is dragged, over whichever screen.
  pub is_dragging: bool,

  /// Whether it marks that the keyboard is on it, as one of several
  /// screens.
  pub marks_keyboard: bool,

  /// Search query, whose matches show on every screen.
  pub query: String,

  /// Its hints line, if it shows one.
  pub hint: Option<(String, HintTone)>,
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
        row_selected: shown,
        grid: GridCells::default(),
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

/// Title of window `hwnd`, or nothing.
fn window_title(
  workspaces: &[OverviewWorkspace],
  hwnd: Option<isize>,
) -> String {
  hwnd
    .and_then(|hwnd| find(workspaces, hwnd))
    .map_or_else(String::new, |(_, window)| window.title.clone())
}
/// The arrows that go to the previous and the next workspace in
/// `layout`, and the one that goes in among the selected workspace's
/// windows, if any does.
fn arrows(layout: OverviewLayoutMode) -> (Toward, Toward, Option<Toward>) {
  match layout {
    OverviewLayoutMode::Stage => {
      (Toward::Up, Toward::Down, Some(Toward::Right))
    }
    OverviewLayoutMode::Activities => {
      (Toward::Left, Toward::Right, Some(Toward::Down))
    }
    OverviewLayoutMode::Carousel | OverviewLayoutMode::Grid => {
      (Toward::Left, Toward::Right, None)
    }
  }
}

fn arrow_glyph(toward: Toward) -> &'static str {
  match toward {
    Toward::Left => "\u{2190}",
    Toward::Right => "\u{2192}",
    Toward::Up => "\u{2191}",
    Toward::Down => "\u{2193}",
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
/// among `cards` (not counting the special workspace's, which `s` goes
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

  let query = query.to_lowercase();
  [&window.title, &window.process_name]
    .into_iter()
    .filter_map(|text| text.to_lowercase().find(&query))
    .min()
}

/// The row card of a screen's `cards` to center its row on with card
/// `selected` selected in `layout`: that card, or for the special
/// workspace's docked card, `previous` (if still a row card) or else the
/// first. `None` when there are no row cards.
fn row_card(
  workspaces: &[OverviewWorkspace],
  cards: &Range<usize>,
  selected: usize,
  previous: Option<usize>,
  layout: OverviewLayoutMode,
) -> Option<usize> {
  let is_row_card = |index: &usize| {
    cards.contains(index)
      && *index < workspaces.len()
      && !is_docked(workspaces, *index, layout)
  };

  Some(selected)
    .filter(is_row_card)
    .or_else(|| previous.filter(is_row_card))
    .or_else(|| row_indices(workspaces, cards, layout).first().copied())
}

/// Whether card `index` is the special workspace's.
fn is_special(workspaces: &[OverviewWorkspace], index: usize) -> bool {
  workspaces
    .get(index)
    .is_some_and(|workspace| workspace.is_special)
}

/// Whether card `index` is the special workspace's, docked below the rest
/// in `layout`.
fn is_docked(
  workspaces: &[OverviewWorkspace],
  index: usize,
  layout: OverviewLayoutMode,
) -> bool {
  layout.docks_special() && is_special(workspaces, index)
}

/// A screen's `cards` of its row (or grid, or stack of rows) in `layout`:
/// all but the special workspace's if it is docked.
fn row_indices(
  workspaces: &[OverviewWorkspace],
  cards: &Range<usize>,
  layout: OverviewLayoutMode,
) -> Vec<usize> {
  cards
    .clone()
    .filter(|index| {
      *index < workspaces.len() && !is_docked(workspaces, *index, layout)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::{all_workspaces, HintTone, Hit, Interaction, Key, Mode};
  use crate::{
    overview_layout::{CardMetrics, CardSlot, GridCells, RectF},
    OverviewAction, OverviewLayoutMode, OverviewScreen, OverviewWindow,
    OverviewWorkspace, Rect,
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
    single(&workspaces(), OverviewLayoutMode::Carousel)
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

  #[test]
  fn starts_on_the_focused_workspace() {
    assert_eq!(interaction().selected(), 1);
  }

  #[test]
  fn arrows_step_through_workspaces_and_stop_at_the_ends() {
    let mut state = interaction();

    press(&mut state, &[Key::Left, Key::Left]);
    assert_eq!(state.selected(), 0);

    press(&mut state, &[Key::Text('l'), Key::Right, Key::Right]);
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
      press(&mut state, &[Key::Digit(3)]),
      Some(focus_workspace("3"))
    );
    assert_eq!(press(&mut state, &[Key::Digit(9)]), None);
  }

  #[test]
  fn digits_go_by_name_when_workspaces_are_numbered() {
    let mut workspaces = workspaces();
    workspaces.remove(1);
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);

    assert_eq!(
      state.key(Key::Digit(3), &workspaces),
      Some(focus_workspace("3"))
    );
    assert_eq!(state.key(Key::Digit(2), &workspaces), None, "not shown");
  }

  #[test]
  fn digits_go_by_position_when_workspaces_are_named() {
    let mut workspaces = workspaces();
    for (workspace, name) in
      workspaces.iter_mut().zip(["web", "code", "chat"])
    {
      workspace.name = name.to_string();
    }
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);

    assert_eq!(
      state.key(Key::Digit(1), &workspaces),
      Some(focus_workspace("web"))
    );
  }

  #[test]
  fn digits_reach_workspaces_without_a_card() {
    let mut state = interaction();
    state.digit_workspaces = vec!["5".to_string()];

    assert_eq!(
      press(&mut state, &[Key::Digit(5)]),
      Some(focus_workspace("5"))
    );
    assert_eq!(press(&mut state, &[Key::Digit(6)]), None);
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Digit(5)]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "5".to_string(),
        monitor: MONITOR,
      })
    );
  }

  #[test]
  fn digits_send_the_picked_window_and_move_on() {
    let mut state = interaction();
    press(&mut state, &[Key::Left, Key::Space]);

    assert_eq!(
      press(&mut state, &[Key::Digit(1)]),
      None,
      "already on workspace 1"
    );
    assert_eq!(
      press(&mut state, &[Key::Digit(3)]),
      Some(OverviewAction::MoveWindow {
        hwnd: 11,
        workspace: "3".to_string(),
        monitor: MONITOR,
      })
    );
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected_window, Some(12), "next window picked");
  }

  #[test]
  fn digits_drop_a_carried_window() {
    let mut state = interaction();

    assert_eq!(
      press(&mut state, &[Key::Space, Key::Space, Key::Digit(1)]),
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
  fn a_new_workspace_is_picked_like_any_other() {
    let mut workspaces = workspaces();
    workspaces.push(OverviewWorkspace {
      is_new: true,
      ..card("4")
    });
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);

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
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);
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

    press(&mut state, &[Key::Backspace, Key::Text('/')]);
    press(
      &mut state,
      &"docs".chars().map(Key::Text).collect::<Vec<_>>(),
    );
    assert_eq!(
      press(&mut state, &[Key::Enter, Key::Enter]),
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
  fn tab_to_the_grid_stops_picking_a_window() {
    let mut state = interaction();

    press(&mut state, &[Key::Space, Key::Tab]);
    assert_eq!(state.layout, OverviewLayoutMode::Grid);
    assert_eq!(state.mode, Mode::Spaces);
    assert_eq!(state.selected_window, None);

    // Over to the stage to pick one up, then around to the grid again.
    press(&mut state, &[Key::Tab, Key::Tab]);
    assert_eq!(state.layout, OverviewLayoutMode::Stage);
    press(&mut state, &[Key::Space, Key::Space]);
    press(&mut state, &[Key::Tab, Key::Tab]);
    assert_eq!(state.layout, OverviewLayoutMode::Grid);
    assert_eq!(state.mode, Mode::Carrying, "carrying works in the grid");
  }

  #[test]
  fn tab_goes_through_every_layout() {
    let mut state = interaction();
    let mut seen = vec![state.layout];
    for _ in 0..4 {
      press(&mut state, &[Key::Tab]);
      seen.push(state.layout);
    }

    assert_eq!(
      seen,
      vec![
        OverviewLayoutMode::Carousel,
        OverviewLayoutMode::Grid,
        OverviewLayoutMode::Activities,
        OverviewLayoutMode::Stage,
        OverviewLayoutMode::Carousel,
      ]
    );
  }

  #[test]
  fn looking_inside_keeps_the_activities_and_the_stage() {
    for layout in
      [OverviewLayoutMode::Activities, OverviewLayoutMode::Stage]
    {
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
  fn the_stage_steps_down_its_column_and_goes_right_into_windows() {
    let workspaces = workspaces();
    let mut state = single(&workspaces, OverviewLayoutMode::Stage);
    let mut press = |keys: &[Key]| {
      for key in keys {
        state.key(*key, &workspaces);
      }
      (state.selected(), state.mode, state.selected_window)
    };

    assert_eq!(press(&[Key::Down]), (2, Mode::Spaces, None));
    assert_eq!(press(&[Key::Up, Key::Up]), (0, Mode::Spaces, None));
    assert_eq!(press(&[Key::Right]), (0, Mode::Windows, Some(11)));
    assert_eq!(press(&[Key::Right]), (0, Mode::Windows, Some(12)));
    assert_eq!(
      press(&[Key::Left, Key::Left]),
      (0, Mode::Spaces, Some(11))
    );
  }

  #[test]
  fn clicking_another_workspace_shows_it_before_going() {
    let workspaces = workspaces();

    for layout in
      [OverviewLayoutMode::Activities, OverviewLayoutMode::Stage]
    {
      let mut state = single(&workspaces, layout);
      let click = |state: &mut Interaction, hit: Hit| {
        state.mouse_down((0.0, 0.0), hit);
        state.mouse_up(&workspaces)
      };

      let other = Hit {
        workspace: Some(0),
        window: Some(11),
      };
      assert_eq!(click(&mut state, other), None);
      assert_eq!(state.selected(), 0);
      assert_eq!(
        click(&mut state, other),
        Some(focus_window(11)),
        "{layout:?}"
      );
    }

    // The carousel goes right away.
    let mut state = interaction();
    state.mouse_down(
      (0.0, 0.0),
      Hit {
        workspace: Some(0),
        window: None,
      },
    );
    assert_eq!(state.mouse_up(&workspaces), Some(focus_workspace("1")));
  }

  #[test]
  fn pointing_at_a_shown_window_picks_it() {
    let workspaces = workspaces();
    let mut state = single(&workspaces, OverviewLayoutMode::Stage);

    let changed = state.mouse_move(
      (10.0, 10.0),
      Hit {
        workspace: Some(1),
        window: Some(21),
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
    assert!(state.drop_target() == Some(1));

    press(&mut state, &[Key::Left]);
    assert!(state.drop_target() == Some(0));

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
  fn escape_cancels_and_x_closes_the_picked_window() {
    let mut state = interaction();
    assert_eq!(
      press(&mut state, &[Key::Escape]),
      Some(OverviewAction::Cancel)
    );

    assert_eq!(
      press(&mut state, &[Key::Text('x')]),
      None,
      "nothing picked"
    );
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Text('x')]),
      Some(OverviewAction::CloseWindow(21))
    );
  }

  #[test]
  fn up_and_down_move_a_row_in_the_grid_only() {
    let workspaces = workspaces();
    let metrics =
      CardMetrics::new(1920.0, 1040.0, 1.0, OverviewLayoutMode::Grid);
    let mut state = interaction();
    state.screens[0].grid = GridCells::new(
      &metrics,
      &vec![CardSlot::new(&metrics, false); workspaces.len()],
      2,
    );

    press(&mut state, &[Key::Left, Key::Down]);
    assert_eq!(state.selected(), 0, "no rows in the carousel");

    press(&mut state, &[Key::Tab, Key::Down]);
    assert_eq!(state.layout, OverviewLayoutMode::Grid);
    assert_eq!(state.selected(), 2);

    press(&mut state, &[Key::Text('k')]);
    assert_eq!(state.selected(), 0);
  }

  #[test]
  fn search_jumps_to_the_best_match() {
    let mut state = interaction();
    let query = "term".chars().map(Key::Text).collect::<Vec<_>>();

    press(&mut state, &[Key::Text('/')]);
    press(&mut state, &query);
    assert!(state.searching);
    assert_eq!(state.query, "term");
    assert_eq!(state.selected(), 1, "typing doesn't move the selection");

    press(&mut state, &[Key::Enter]);
    assert!(!state.searching);
    assert_eq!(state.selected(), 0);
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected_window, Some(11));
  }

  #[test]
  fn search_matches_process_names_and_n_walks_them() {
    let mut state = interaction();
    let query = ".exe".chars().map(Key::Text).collect::<Vec<_>>();

    press(&mut state, &[Key::Text('/')]);
    press(&mut state, &query);
    press(&mut state, &[Key::Enter]);
    let first = state.selected_window;

    press(&mut state, &[Key::Text('n')]);
    assert_ne!(state.selected_window, first);

    press(&mut state, &[Key::Text('N')]);
    assert_eq!(state.selected_window, first);
  }

  #[test]
  fn escape_leaves_search_without_closing() {
    let mut state = interaction();

    assert_eq!(
      press(&mut state, &[Key::Text('/'), Key::Text('q'), Key::Escape]),
      None
    );
    assert!(!state.searching);
    assert_eq!(state.query, "");
  }

  #[test]
  fn clicks_go_to_windows_and_workspaces_or_cancel() {
    let workspaces = workspaces();
    let mut state = interaction();

    let window = Hit {
      workspace: Some(0),
      window: Some(11),
    };
    state.mouse_down((10.0, 10.0), window);
    assert_eq!(state.mouse_up(&workspaces), Some(focus_window(11)));

    let card = Hit {
      workspace: Some(2),
      window: None,
    };
    state.mouse_down((10.0, 10.0), card);
    assert_eq!(state.mouse_up(&workspaces), Some(focus_workspace("3")));

    state.mouse_down((10.0, 10.0), Hit::default());
    assert_eq!(state.mouse_up(&workspaces), Some(OverviewAction::Cancel));
  }

  #[test]
  fn dragging_a_window_onto_another_card_moves_it() {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(1),
      window: Some(21),
    };

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move((102.0, 101.0), grab, true);
    assert_eq!(state.drag, None, "a little slop stays a click");

    let over = Hit {
      workspace: Some(2),
      window: None,
    };
    assert!(state.mouse_move((400.0, 100.0), over, true));
    assert_eq!(state.floating(), Some(21));
    assert!(state.drop_target() == Some(2));

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
  fn dropping_a_drag_on_its_own_card_moves_nothing() {
    let workspaces = workspaces();
    let mut state = interaction();
    let grab = Hit {
      workspace: Some(1),
      window: Some(21),
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
    };

    assert_eq!(
      state.middle_click(hit),
      Some(OverviewAction::CloseWindow(12))
    );
    assert_eq!(state.middle_click(Hit::default()), None);
  }

  #[test]
  fn hints_follow_the_step() {
    let workspaces = workspaces();
    let mut state = interaction();
    assert_eq!(state.hint(&workspaces).1, HintTone::Quiet);

    press(&mut state, &[Key::Space, Key::Space]);
    let (text, tone) = state.hint(&workspaces);
    assert_eq!(tone, HintTone::Accent);
    assert!(text.contains("\"Browser\""));

    press(&mut state, &[Key::Text('/'), Key::Text('e')]);
    let (text, tone) = state.hint(&workspaces);
    assert_eq!(tone, HintTone::Search);
    assert!(text.starts_with("/e"));
    assert!(text.contains("3 hits"));
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
  fn the_special_workspace_comes_last_where_it_isnt_docked() {
    let workspaces = with_special();
    let special = workspaces.len() - 1;

    for (layout, back, forward) in [
      (OverviewLayoutMode::Activities, Key::Left, Key::Right),
      (OverviewLayoutMode::Stage, Key::Up, Key::Down),
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
  fn s_goes_to_the_special_workspace_like_a_digit() {
    let workspaces = with_special();
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);
    let press = |state: &mut Interaction, keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    assert_eq!(
      press(&mut state, &[Key::Text('s')]),
      Some(focus_workspace("special"))
    );
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Text('s')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
        monitor: MONITOR,
      }),
      "sends the picked window"
    );

    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Space, Key::Text('s')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
        monitor: MONITOR,
      }),
      "drops the carried window"
    );
    assert_eq!(state.mode, Mode::Spaces);

    // While searching, it is typed.
    assert_eq!(press(&mut state, &[Key::Text('/'), Key::Text('s')]), None);
    assert_eq!(state.query, "s");
  }

  #[test]
  fn s_does_nothing_without_a_special_card() {
    let mut state = interaction();
    assert_eq!(press(&mut state, &[Key::Text('s')]), None);
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
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);

    assert_eq!(
      state.key(Key::Digit(1), &workspaces),
      Some(focus_workspace("code"))
    );
    assert_eq!(state.key(Key::Digit(4), &workspaces), None);
  }

  #[test]
  fn hints_mention_the_special_workspace() {
    let workspaces = with_special();
    let state = single(&workspaces, OverviewLayoutMode::Carousel);

    assert!(state.hint(&workspaces).0.contains("s special"));
  }

  #[test]
  fn the_dock_is_below_the_row_not_at_its_end() {
    let workspaces = with_special();
    let press = |state: &mut Interaction, keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    for layout in [OverviewLayoutMode::Carousel, OverviewLayoutMode::Grid]
    {
      let mut state = single(&workspaces, layout);
      if layout == OverviewLayoutMode::Grid {
        state.screens[0].grid = GridCells::new(
          &CardMetrics::new(1920.0, 1080.0, 1.0, OverviewLayoutMode::Grid),
          &workspaces
            .iter()
            .map(|w| {
              CardSlot::new(
                &CardMetrics::new(
                  1920.0,
                  1080.0,
                  1.0,
                  OverviewLayoutMode::Grid,
                ),
                w.is_special,
              )
            })
            .collect::<Vec<_>>(),
          5,
        );
      }

      press(&mut state, &[Key::Right, Key::Right, Key::Right]);
      assert_eq!(state.selected(), 2, "left/right stay in the row");
      press(&mut state, &[Key::End]);
      assert_eq!(state.selected(), 2, "the row's end, not the dock");

      press(&mut state, &[Key::Home, Key::Down]);
      assert_eq!(state.selected(), 3, "down goes to the dock");
      assert_eq!(
        state.screens[0].row_selected, 0,
        "the row stays on the card the dock was entered from"
      );
      press(&mut state, &[Key::Down]);
      assert_eq!(state.selected(), 3);
      press(&mut state, &[Key::Up]);
      assert_eq!(state.selected(), 0, "up goes back where it came from");

      press(&mut state, &[Key::Down, Key::Right]);
      assert_eq!(state.selected(), 0, "sideways leaves the dock too");
    }
  }

  #[test]
  fn opened_on_the_special_workspace_the_row_starts_at_its_first_card() {
    let mut workspaces = with_special();
    for workspace in &mut workspaces {
      workspace.is_displayed = workspace.is_special;
    }

    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);
    assert_eq!(state.selected(), 3);
    assert_eq!(state.screens[0].row_selected, 0);

    state.key(Key::Up, &workspaces);
    assert_eq!(state.selected(), 0);
  }

  #[test]
  fn a_carried_window_drops_on_the_dock_below() {
    let workspaces = with_special();
    let mut state = single(&workspaces, OverviewLayoutMode::Carousel);
    let press = |state: &mut Interaction, keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    assert_eq!(
      press(&mut state, &[Key::Space, Key::Space, Key::Down, Key::Space]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
        monitor: MONITOR,
      })
    );
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
  /// docked in the carousel, last.
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
    let state =
      Interaction::new(&two_screens(), OverviewLayoutMode::Carousel, 6.0);

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
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    press_on(&mut state, &workspaces, &[Key::Right, Key::Right]);
    assert_eq!((state.active, state.selected()), (1, 3), "past the dock");
    assert_eq!(state.screens[0].shown, 1, "the left one stays put");

    press_on(&mut state, &workspaces, &[Key::Right, Key::Right]);
    assert_eq!(state.selected(), 4, "nothing further right");

    press_on(&mut state, &workspaces, &[Key::Home, Key::Left]);
    assert_eq!((state.active, state.selected()), (0, 1));

    press_on(&mut state, &workspaces, &[Key::End]);
    assert_eq!(state.selected(), 1, "the end of its own row");
  }

  #[test]
  fn arrows_only_cross_toward_a_screen_that_way() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    press_on(&mut state, &workspaces, &[Key::Left, Key::Up]);
    assert_eq!((state.active, state.selected()), (0, 0));

    press_on(&mut state, &workspaces, &[Key::Down, Key::Down]);
    assert_eq!((state.active, state.selected()), (0, 2), "on the dock");
  }

  #[test]
  fn screens_out_of_line_are_not_crossed_to() {
    let screens = two_screens_at(Rect::from_xy(1920, 1040, 1920, 1040));
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    press_on(&mut state, &workspaces, &[Key::End, Key::Right]);
    assert_eq!((state.active, state.selected()), (0, 1));
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
    // below, where it shows what that screen shows.
    press_on(&mut state, &workspaces, &[Key::Home, Key::Down]);
    assert_eq!(state.mode, Mode::Windows);
    press_on(&mut state, &workspaces, &[Key::Down]);
    assert_eq!((state.active, state.selected()), (1, 3));
    assert_eq!(state.mode, Mode::Spaces);

    press_on(&mut state, &workspaces, &[Key::Up]);
    assert_eq!((state.active, state.selected()), (0, 0));
  }

  #[test]
  fn the_stage_crosses_sideways_onto_the_shown_workspace() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Stage, 6.0);

    // Right goes in among the windows, and past them to the next screen.
    press_on(&mut state, &workspaces, &[Key::Down, Key::Up, Key::Right]);
    assert_eq!(state.mode, Mode::Windows);
    press_on(&mut state, &workspaces, &[Key::Right]);
    assert_eq!((state.active, state.selected()), (1, 3));

    press_on(&mut state, &workspaces, &[Key::Down, Key::Left]);
    assert_eq!((state.active, state.selected()), (0, 0), "its shown one");
  }

  #[test]
  fn only_the_screen_with_the_keyboard_marks_it() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    assert!(state.focus(0, &workspaces).marks_keyboard);
    assert!(!state.focus(1, &workspaces).marks_keyboard);

    state.activate(1, &workspaces);
    assert!(!state.focus(0, &workspaces).marks_keyboard);
    assert!(state.focus(1, &workspaces).marks_keyboard);

    let alone = single(&workspaces[..3], OverviewLayoutMode::Carousel);
    assert!(!alone.focus(0, &workspaces).marks_keyboard, "one screen");
  }

  #[test]
  fn the_wheel_stays_on_its_screen() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    state.wheel(true, &workspaces);
    state.wheel(true, &workspaces);
    assert_eq!((state.active, state.selected()), (0, 1));
  }

  #[test]
  fn a_carried_window_crosses_to_the_other_screen() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    assert_eq!(
      press_on(
        &mut state,
        &workspaces,
        &[Key::Space, Key::Space, Key::Right, Key::Right, Key::Space],
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

    press_on(&mut state, &workspaces, &[Key::Text('/')]);
    press_on(
      &mut state,
      &workspaces,
      &"brow".chars().map(Key::Text).collect::<Vec<_>>(),
    );

    let (text, tone) = state
      .screen_hint(1, &workspaces)
      .expect("search on every screen");
    assert_eq!(tone, HintTone::Search);
    assert!(text.contains("1 hit"));

    press_on(&mut state, &workspaces, &[Key::Enter]);
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
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);
    state.query = "notes".to_string();

    let matches = state
      .matches(&workspaces)
      .into_iter()
      .map(|(index, window)| (index, window.hwnd))
      .collect::<Vec<_>>();
    assert_eq!(matches, [(5, 51)], "on the screen showing it");
  }

  #[test]
  fn hints_of_other_screens_only_show_a_search() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    assert!(state.screen_hint(0, &workspaces).is_some());
    assert_eq!(state.screen_hint(1, &workspaces), None);
  }

  #[test]
  fn picks_carry_the_monitor_they_were_picked_on() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);
    let focus = |name: &str, monitor| OverviewAction::FocusWorkspace {
      name: name.to_string(),
      monitor,
    };

    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Text('s')]),
      Some(focus("special", 1))
    );
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Digit(4)]),
      Some(focus("4", 2)),
      "where its card is"
    );

    state.activate(1, &workspaces);
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Text('s')]),
      Some(focus("special", 2))
    );

    state.digit_workspaces = vec!["7".to_string()];
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Digit(7)]),
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
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    state.activate(1, &workspaces);
    assert_eq!(
      press_on(&mut state, &workspaces, &[Key::Digit(2)]),
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
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);
    let grab = Hit {
      workspace: Some(0),
      window: Some(11),
    };

    state.mouse_down((100.0, 100.0), grab);
    state.mouse_move(
      (2400.0, 100.0),
      Hit {
        workspace: Some(4),
        window: None,
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
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    state.mouse_down(
      (100.0, 100.0),
      Hit {
        workspace: Some(2),
        window: Some(51),
      },
    );
    state.mouse_move(
      (2400.0, 900.0),
      Hit {
        workspace: Some(5),
        window: None,
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

    assert!(state.activate(1, &workspaces));
    assert_eq!((state.active, state.selected()), (1, 3));
    assert_eq!(state.mode, Mode::Spaces);
    assert_eq!(state.selected_window, None);
    assert!(!state.activate(1, &workspaces), "already there");
  }

  #[test]
  fn each_screen_sees_the_interaction_in_its_own_indices() {
    let screens = two_screens();
    let workspaces = all_workspaces(&screens);
    let mut state =
      Interaction::new(&screens, OverviewLayoutMode::Carousel, 6.0);

    press_on(
      &mut state,
      &workspaces,
      &[Key::Space, Key::Space, Key::Right, Key::Right, Key::Right],
    );
    let (left, right) =
      (state.focus(0, &workspaces), state.focus(1, &workspaces));

    assert!(!left.is_active && right.is_active);
    assert_eq!((left.selected, left.mode), (1, Mode::Spaces));
    assert_eq!((right.selected, right.mode), (1, Mode::Carrying));
    assert_eq!((left.drop, right.drop), (None, Some(1)));
    assert_eq!(left.floating, Some(11), "lifted on every screen");
    assert_eq!(left.hint, None);
  }

  #[test]
  fn sync_keeps_each_screen_on_its_workspace() {
    let old_screens = two_screens();
    let old = all_workspaces(&old_screens);
    let mut state =
      Interaction::new(&old_screens, OverviewLayoutMode::Carousel, 6.0);
    press_on(&mut state, &old, &[Key::Right, Key::Right, Key::Right]);
    assert_eq!(state.selected(), 4);

    // A workspace shows up on the left, moving the right one's cards.
    let mut screens = two_screens();
    screens[0].workspaces.insert(0, card("0"));
    state.sync(&old, &screens);

    assert_eq!((state.active, state.selected()), (1, 5), "still on 4");
    assert_eq!(state.screens[0].shown, 2, "still on 2");

    // The right monitor goes away.
    let old = all_workspaces(&screens);
    screens.truncate(1);
    state.sync(&old, &screens);
    assert_eq!((state.active, state.selected()), (0, 2));
  }
}
