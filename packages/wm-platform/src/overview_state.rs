//! What the overview is doing, and how keys and the mouse change it, kept
//! free of drawing so it can be tested.
//!
//! The keyboard moves through three steps, one space bar at a time:
//! browsing workspaces, picking a window inside one, and carrying a window
//! to another workspace. The mouse does the same directly: click to go,
//! drag to move.

use crate::{
  overview_layout::GridCells, OverviewAction, OverviewLayoutMode,
  OverviewWindow, OverviewWorkspace,
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

  /// Where the cursor is, in overview coordinates.
  pub position: (f32, f32),

  /// Workspace it would be dropped on.
  pub target: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Press {
  position: (f32, f32),
  hit: Hit,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Interaction {
  /// Index of the selected workspace.
  pub selected: usize,

  pub mode: Mode,
  pub layout: OverviewLayoutMode,

  /// Window picked inside the selected workspace.
  pub selected_window: Option<isize>,

  /// Window held while picking a workspace for it.
  pub carried: Option<isize>,

  /// Whether typed characters go to the search query.
  pub searching: bool,

  /// Search query; its matches stay highlighted after typing ends.
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

  /// Where the grid puts each card, for moving a row up or down.
  pub grid: GridCells,

  /// Card of the row of workspaces the row is centered on: the selected
  /// one, or while the special workspace's dock is selected, the one it
  /// was entered from, which moving up goes back to.
  pub row_selected: usize,

  /// Card on the shelf's stage: the selected one, except while carrying,
  /// when it stays the one the window was lifted from.
  pub staged: usize,

  /// Title of a window dragged in from outside, which has no card of its
  /// own to find it on.
  pub dragged_title: Option<String>,
}

impl Interaction {
  /// Starts out on the focused workspace.
  pub fn new(
    workspaces: &[OverviewWorkspace],
    layout: OverviewLayoutMode,
    drag_threshold: f32,
  ) -> Self {
    let selected = workspaces
      .iter()
      .position(|workspace| workspace.is_focused)
      .unwrap_or(0);

    Self {
      selected,
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
      grid: GridCells::default(),
      row_selected: row_card(workspaces, selected, None).unwrap_or(0),
      staged: selected,
      dragged_title: None,
    }
  }

  /// Keeps the selection on the same workspace and windows when what is
  /// shown changes from `old` to `new`.
  pub fn sync(
    &mut self,
    old: &[OverviewWorkspace],
    new: &[OverviewWorkspace],
  ) {
    let remap = |index: usize| {
      let name = &old.get(index)?.name;
      new.iter().position(|workspace| &workspace.name == name)
    };

    self.selected = remap(self.selected)
      .or_else(|| new.iter().position(|workspace| workspace.is_focused))
      .unwrap_or(0);
    self.hover = self.hover.and_then(remap);
    self.row_selected =
      row_card(new, self.selected, remap(self.row_selected)).unwrap_or(0);
    self.staged = remap(self.staged).unwrap_or(self.selected);

    let exists = |hwnd: isize| find(new, hwnd).is_some();

    if self.selected_window.is_some_and(|hwnd| !exists(hwnd)) {
      self.selected_window = None;
    }

    if self.carried.is_some_and(|hwnd| !exists(hwnd)) {
      self.carried = None;
      self.mode = Mode::Spaces;
    }

    if let Some(drag) = &mut self.drag {
      if exists(drag.hwnd) {
        drag.target = drag.target.and_then(remap);
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

  /// Workspace the floating window would land on, if any.
  pub fn drop_target(&self) -> Option<usize> {
    match &self.drag {
      Some(drag) => drag.target,
      None => (self.mode == Mode::Carrying).then_some(self.selected),
    }
  }

  /// Whether workspace `index` is where the floating window would land.
  pub fn is_drop_target(&self, index: usize) -> bool {
    self.drop_target() == Some(index)
  }

  /// Switches to `layout`.
  pub fn set_layout(&mut self, layout: OverviewLayoutMode) {
    self.layout = layout;

    match layout {
      // The grid is too small to pick a window in.
      OverviewLayoutMode::Grid if self.mode == Mode::Windows => {
        self.mode = Mode::Spaces;
        self.selected_window = None;
      }
      OverviewLayoutMode::Shelf if self.mode != Mode::Carrying => {
        self.staged = self.selected;
      }
      _ => {}
    }
  }

  /// Whether `window` matches the search query.
  pub fn is_match(&self, window: &OverviewWindow) -> bool {
    match_score(window, &self.query).is_some()
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
      Key::Tab => self.set_layout(match self.layout {
        OverviewLayoutMode::Carousel => OverviewLayoutMode::Grid,
        OverviewLayoutMode::Grid => OverviewLayoutMode::Shelf,
        OverviewLayoutMode::Shelf => OverviewLayoutMode::Carousel,
      }),
      Key::Space => return self.space(workspaces),
      Key::Backspace => match self.mode {
        Mode::Carrying => {
          let carried = self.carried.take();
          self.mode = Mode::Windows;

          // Back to picking on the stage, where the window still is.
          if self.layout == OverviewLayoutMode::Shelf {
            self.selected = self.staged;
            self.selected_window = carried;
            self.row_selected =
              row_card(workspaces, self.staged, Some(self.row_selected))
                .unwrap_or(0);
          }
        }
        Mode::Windows => {
          self.mode = Mode::Spaces;
          self.selected_window = None;
        }
        Mode::Spaces => {}
      },
      Key::Left | Key::Text('h') => self.horizontal(-1, workspaces),
      Key::Right | Key::Text('l') => self.horizontal(1, workspaces),
      Key::Up | Key::Text('k') => self.vertical(-1, workspaces),
      Key::Down | Key::Text('j') => self.vertical(1, workspaces),
      Key::Home => self.edge(false, workspaces),
      Key::End => self.edge(true, workspaces),
      Key::Enter => return self.enter(workspaces),
      Key::Digit(digit) => {
        let name =
          digit_target(digit, workspaces, &self.digit_workspaces)?;
        return self.go_to(name, workspaces);
      }
      Key::Text('s') => {
        let special = workspaces.iter().find(|w| w.is_special)?;
        return self.go_to(special.name.clone(), workspaces);
      }
      Key::Text(_) => {}
    }

    None
  }

  /// Goes straight to workspace `name`: switches to it, sends the picked
  /// window there, or drops the carried one there.
  fn go_to(
    &mut self,
    name: String,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    match self.mode {
      Mode::Spaces => Some(OverviewAction::FocusWorkspace(name)),
      Mode::Windows => match self.selected_window {
        Some(hwnd) => self.send(hwnd, name, workspaces),
        None => Some(OverviewAction::FocusWorkspace(name)),
      },
      Mode::Carrying => {
        let action = self
          .carried
          .and_then(|hwnd| move_action(hwnd, name, workspaces));
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
        let first = workspaces.get(self.selected).and_then(|workspace| {
          ordered(workspace, Axis::Across)
            .first()
            .map(|window| window.hwnd)
        });

        // Picking a window is a zoomed-in thing; the grid is too small to
        // aim at one.
        if let Some(hwnd) = first {
          if self.layout == OverviewLayoutMode::Grid {
            self.layout = OverviewLayoutMode::Carousel;
          }
          self.mode = Mode::Windows;
          self.selected_window = Some(hwnd);
        }
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
    match self.mode {
      Mode::Carrying => self.drop_carried(workspaces),
      Mode::Windows if self.selected_window.is_some() => {
        self.selected_window.map(OverviewAction::FocusWindow)
      }
      _ => workspaces.get(self.selected).map(|workspace| {
        OverviewAction::FocusWorkspace(workspace.name.clone())
      }),
    }
  }

  fn drop_carried(
    &mut self,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let action = self.carried.and_then(|hwnd| {
      let to = workspaces.get(self.selected)?;
      move_action(hwnd, to.name.clone(), workspaces)
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
    self.staged = self.selected;
  }

  /// Sends the picked window `hwnd` to workspace `name`, and moves the
  /// cursor on to its neighbour so the rest can be sorted in a row.
  fn send(
    &mut self,
    hwnd: isize,
    name: String,
    workspaces: &[OverviewWorkspace],
  ) -> Option<OverviewAction> {
    let action = move_action(hwnd, name, workspaces)?;

    let windows = workspaces
      .get(self.selected)
      .map_or_else(Vec::new, |workspace| ordered(workspace, Axis::Across));
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

  fn select(&mut self, index: usize, workspaces: &[OverviewWorkspace]) {
    if index != self.selected {
      self.selected = index;
      // The window cursor belongs to one workspace.
      self.selected_window = None;
    }

    // While carrying, the stage keeps showing where the window came from.
    if self.mode != Mode::Carrying {
      self.staged = index;
    }

    self.row_selected =
      row_card(workspaces, index, Some(self.row_selected)).unwrap_or(0);
  }

  /// The cards moved through with the arrows: on the shelf every card,
  /// elsewhere all but the special workspace's, docked below the rest.
  fn row(&self, workspaces: &[OverviewWorkspace]) -> Vec<usize> {
    if self.layout == OverviewLayoutMode::Shelf {
      (0..workspaces.len()).collect()
    } else {
      row_indices(workspaces)
    }
  }

  /// Moves along the row of workspaces, which the special workspace's
  /// dock below it isn't part of; from the dock, back to the row.
  fn step(&mut self, delta: isize, workspaces: &[OverviewWorkspace]) {
    if self.layout != OverviewLayoutMode::Shelf
      && is_special(workspaces, self.selected)
    {
      if let Some(index) = row_card(workspaces, self.row_selected, None) {
        self.select(index, workspaces);
      }
      return;
    }

    let row = self.row(workspaces);
    let Some(position) = row.iter().position(|i| *i == self.selected)
    else {
      return;
    };
    let position = position
      .saturating_add_signed(delta)
      .min(row.len().saturating_sub(1));
    self.select(row[position], workspaces);
  }

  fn horizontal(
    &mut self,
    delta: isize,
    workspaces: &[OverviewWorkspace],
  ) {
    if self.mode == Mode::Windows {
      self.step_window(Axis::Across, delta, workspaces);
    } else {
      self.step(delta, workspaces);
    }
  }

  fn vertical(&mut self, delta: isize, workspaces: &[OverviewWorkspace]) {
    if self.mode == Mode::Windows {
      self.step_window(Axis::Down, delta, workspaces);
      return;
    }

    // Down from the shelf goes into the windows on the stage below it.
    if self.layout == OverviewLayoutMode::Shelf {
      if delta > 0 && self.mode == Mode::Spaces {
        self.space(workspaces);
      }
      return;
    }

    // Up from the dock goes back to where it was entered from. Down onto
    // the dock is from the carousel, or from the grid's last row, below
    // which `GridCells` puts it.
    let target = if is_special(workspaces, self.selected) {
      (delta < 0)
        .then(|| row_card(workspaces, self.row_selected, None))
        .flatten()
    } else {
      match self.layout {
        OverviewLayoutMode::Grid => {
          self.grid.vertical_neighbour(self.selected, delta)
        }
        OverviewLayoutMode::Carousel | OverviewLayoutMode::Shelf => {
          workspaces
            .iter()
            .position(|workspace| workspace.is_special)
            .filter(|_| delta > 0)
        }
      }
    };

    if let Some(index) = target {
      self.select(index, workspaces);
    }
  }

  /// Selects the first (or last) workspace, or window while picking one.
  fn edge(&mut self, is_last: bool, workspaces: &[OverviewWorkspace]) {
    if self.mode == Mode::Windows {
      let windows = workspaces
        .get(self.selected)
        .map_or_else(Vec::new, |workspace| {
          ordered(workspace, Axis::Across)
        });
      let window = if is_last {
        windows.last()
      } else {
        windows.first()
      };
      self.selected_window = window.map(|window| window.hwnd);
    } else {
      let row = self.row(workspaces);
      let index = if is_last { row.last() } else { row.first() };
      if let Some(index) = index {
        self.select(*index, workspaces);
      }
    }
  }

  /// Moves the window cursor to the next window in on-screen order.
  fn step_window(
    &mut self,
    axis: Axis,
    delta: isize,
    workspaces: &[OverviewWorkspace],
  ) {
    let Some(workspace) = workspaces.get(self.selected) else {
      return;
    };

    let windows = ordered(workspace, axis);
    let Some(last) = windows.len().checked_sub(1) else {
      return;
    };

    let next = match windows
      .iter()
      .position(|window| Some(window.hwnd) == self.selected_window)
    {
      Some(index) => index.saturating_add_signed(delta).min(last),
      None => 0,
    };

    self.selected_window = windows.get(next).map(|window| window.hwnd);
  }

  /// Search matches, best first, then in carousel order.
  pub fn matches<'a>(
    &self,
    workspaces: &'a [OverviewWorkspace],
  ) -> Vec<(usize, &'a OverviewWindow)> {
    let mut matches = workspaces
      .iter()
      .enumerate()
      .flat_map(|(index, workspace)| {
        let unshown = workspace
          .windows
          .iter()
          .filter(|window| window.is_minimized || window.is_hidden_tab);
        ordered(workspace, Axis::Across)
          .into_iter()
          .chain(unshown)
          .filter_map(move |window| {
            let score = match_score(window, &self.query)?;
            Some((score, index, window))
          })
      })
      .collect::<Vec<_>>();

    matches.sort_by_key(|(score, ..)| *score);
    matches
      .into_iter()
      .map(|(_, index, window)| (index, window))
      .collect()
  }

  /// Selects the next (or previous) search match.
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
    let changed = self.hover != hit.workspace;
    self.hover = hit.workspace;

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

    if let Some(drag) = self.drag.take() {
      let from = find(workspaces, drag.hwnd).map(|(index, _)| index);
      let to = drag.target.and_then(|index| workspaces.get(index));

      // Dropped on its own card, or on nothing: no move.
      return to.filter(|_| drag.target != from).map(|to| {
        OverviewAction::MoveWindow {
          hwnd: drag.hwnd,
          workspace: to.name.clone(),
        }
      });
    }

    match press.hit {
      Hit {
        window: Some(hwnd), ..
      } => Some(OverviewAction::FocusWindow(hwnd)),
      // A card on the shelf goes on the stage; the one on the stage is
      // gone to.
      Hit {
        workspace: Some(index),
        ..
      } if self.layout == OverviewLayoutMode::Shelf
        && index != self.staged
        && self.mode != Mode::Carrying =>
      {
        self.mode = Mode::Spaces;
        self.select(index, workspaces);
        None
      }
      Hit {
        workspace: Some(index),
        ..
      } => {
        self.selected = index;
        workspaces.get(index).map(|workspace| {
          OverviewAction::FocusWorkspace(workspace.name.clone())
        })
      }
      Hit { .. } => Some(OverviewAction::Cancel),
    }
  }

  /// Selects the next (or previous) workspace.
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

  /// The keys for the current step, shown under the cards.
  pub fn hint(
    &self,
    workspaces: &[OverviewWorkspace],
  ) -> (String, HintTone) {
    let title = |hwnd: Option<isize>| {
      hwnd
        .and_then(|hwnd| find(workspaces, hwnd))
        .map_or_else(String::new, |(_, window)| window.title.clone())
    };

    if self.searching || !self.query.is_empty() {
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
        text.push_str(&title(self.carried));
        text.push_str("\" here");
      }

      return (text, HintTone::Search);
    }

    if let Some(drag) = &self.drag {
      let title = find(workspaces, drag.hwnd)
        .map(|(_, window)| window.title.clone())
        .or_else(|| self.dragged_title.clone())
        .unwrap_or_default();

      let text = match drag.target.and_then(|index| workspaces.get(index))
      {
        Some(to) => format!("move \"{title}\" to {}", to.label),
        None => format!("drop \"{title}\" on a workspace"),
      };
      return (text, HintTone::Accent);
    }

    match (self.mode, self.layout) {
      (Mode::Carrying, _) => (
        format!(
          "carrying \"{}\"   \u{b7}   \u{2190}\u{2192} or / to pick a workspace   \u{b7}   space to drop   \u{b7}   1\u{2013}9 or s drop on that one   \u{b7}   \u{232b} back",
          title(self.carried)
        ),
        HintTone::Accent,
      ),
      (Mode::Windows, _) => (
        "\u{2190}\u{2192} pick a window   \u{b7}   1\u{2013}9 or s send it there   \u{b7}   space to lift it   \u{b7}   \u{23ce} jump to it   \u{b7}   x close   \u{b7}   \u{232b} back"
          .to_string(),
        HintTone::Quiet,
      ),
      (Mode::Spaces, OverviewLayoutMode::Grid) => (
        "arrows workspaces   \u{b7}   drag windows between them   \u{b7}   s special   \u{b7}   / search   \u{b7}   \u{23ce} go there   \u{b7}   tab shelf   \u{b7}   esc close"
          .to_string(),
        HintTone::Quiet,
      ),
      (Mode::Spaces, OverviewLayoutMode::Shelf) => (
        "\u{2190}\u{2192} workspaces   \u{b7}   space or \u{2193} into its windows   \u{b7}   drag windows onto the shelf   \u{b7}   s special   \u{b7}   / search   \u{b7}   \u{23ce} go there   \u{b7}   tab carousel   \u{b7}   esc close"
          .to_string(),
        HintTone::Quiet,
      ),
      (Mode::Spaces, OverviewLayoutMode::Carousel) => (
        "\u{2190}\u{2192} workspaces   \u{b7}   space to look inside   \u{b7}   s special   \u{b7}   / search   \u{b7}   \u{23ce} go there   \u{b7}   tab all workspaces   \u{b7}   esc close"
          .to_string(),
        HintTone::Quiet,
      ),
    }
  }
}

#[derive(Clone, Copy)]
enum Axis {
  /// Left to right, then top to bottom.
  Across,
  /// Top to bottom, then left to right.
  Down,
}

/// Shown windows of `workspace` in on-screen order: not minimized, and of
/// a stack only its shown tab.
fn ordered(
  workspace: &OverviewWorkspace,
  axis: Axis,
) -> Vec<&OverviewWindow> {
  let mut windows = workspace
    .windows
    .iter()
    .filter(|window| {
      !window.is_minimized
        && !window.is_hidden_tab
        && window.rect.width() > 0
    })
    .collect::<Vec<_>>();

  windows.sort_by_key(|window| match axis {
    Axis::Across => (window.rect.left, window.rect.top),
    Axis::Down => (window.rect.top, window.rect.left),
  });
  windows
}

/// Moves window `hwnd` to workspace `name`, unless it is already there.
fn move_action(
  hwnd: isize,
  name: String,
  workspaces: &[OverviewWorkspace],
) -> Option<OverviewAction> {
  let (from, _) = find(workspaces, hwnd)?;
  let is_there = workspaces
    .get(from)
    .is_some_and(|workspace| workspace.name == name);

  (!is_there).then_some(OverviewAction::MoveWindow {
    hwnd,
    workspace: name,
  })
}

/// Name of the workspace digit key `digit` stands for, 0 being the tenth:
/// the one named after it when workspaces are numbered, whether it has a
/// card or is one of `others`, else the card at its position (not counting
/// the special workspace's, which `s` goes to).
fn digit_target(
  digit: u8,
  workspaces: &[OverviewWorkspace],
  others: &[String],
) -> Option<String> {
  let number = if digit == 0 { 10 } else { usize::from(digit) };
  let is_numbered = !others.is_empty()
    || workspaces
      .iter()
      .any(|workspace| workspace.name.parse::<usize>().is_ok());

  if !is_numbered {
    return workspaces
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

/// The row card to center the row on with card `selected` selected: that
/// card, or for the special workspace's, `previous` (if still a row card)
/// or else the first. `None` when there are no row cards.
fn row_card(
  workspaces: &[OverviewWorkspace],
  selected: usize,
  previous: Option<usize>,
) -> Option<usize> {
  let is_row_card = |index: &usize| {
    *index < workspaces.len() && !is_special(workspaces, *index)
  };

  Some(selected)
    .filter(is_row_card)
    .or_else(|| previous.filter(is_row_card))
    .or_else(|| row_indices(workspaces).first().copied())
}

/// Whether card `index` is the special workspace's, docked below the rest.
fn is_special(workspaces: &[OverviewWorkspace], index: usize) -> bool {
  workspaces
    .get(index)
    .is_some_and(|workspace| workspace.is_special)
}

/// The cards in the row (or grid), i.e. all but the special workspace's.
fn row_indices(workspaces: &[OverviewWorkspace]) -> Vec<usize> {
  workspaces
    .iter()
    .enumerate()
    .filter(|(_, workspace)| !workspace.is_special)
    .map(|(index, _)| index)
    .collect()
}

#[cfg(test)]
mod tests {
  use super::{HintTone, Hit, Interaction, Key, Mode};
  use crate::{
    overview_layout::{CardMetrics, CardSlot, GridCells},
    OverviewAction, OverviewLayoutMode, OverviewWindow, OverviewWorkspace,
    Rect,
  };

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
        is_focused: true,
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
    Interaction::new(&workspaces(), OverviewLayoutMode::Carousel, 6.0)
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
    assert_eq!(interaction().selected, 1);
  }

  #[test]
  fn arrows_step_through_workspaces_and_stop_at_the_ends() {
    let mut state = interaction();

    press(&mut state, &[Key::Left, Key::Left]);
    assert_eq!(state.selected, 0);

    press(&mut state, &[Key::Text('l'), Key::Right, Key::Right]);
    assert_eq!(state.selected, 2);

    press(&mut state, &[Key::Home]);
    assert_eq!(state.selected, 0);
  }

  #[test]
  fn enter_switches_to_the_selected_workspace() {
    let mut state = interaction();

    assert_eq!(
      press(&mut state, &[Key::Left, Key::Enter]),
      Some(OverviewAction::FocusWorkspace("1".to_string()))
    );
  }

  #[test]
  fn digits_switch_straight_to_a_workspace() {
    let mut state = interaction();

    assert_eq!(
      press(&mut state, &[Key::Digit(3)]),
      Some(OverviewAction::FocusWorkspace("3".to_string()))
    );
    assert_eq!(press(&mut state, &[Key::Digit(9)]), None);
  }

  #[test]
  fn digits_go_by_name_when_workspaces_are_numbered() {
    let mut workspaces = workspaces();
    workspaces.remove(1);
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);

    assert_eq!(
      state.key(Key::Digit(3), &workspaces),
      Some(OverviewAction::FocusWorkspace("3".to_string()))
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
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);

    assert_eq!(
      state.key(Key::Digit(1), &workspaces),
      Some(OverviewAction::FocusWorkspace("web".to_string()))
    );
  }

  #[test]
  fn digits_reach_workspaces_without_a_card() {
    let mut state = interaction();
    state.digit_workspaces = vec!["5".to_string()];

    assert_eq!(
      press(&mut state, &[Key::Digit(5)]),
      Some(OverviewAction::FocusWorkspace("5".to_string()))
    );
    assert_eq!(press(&mut state, &[Key::Digit(6)]), None);
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Digit(5)]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "5".to_string(),
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
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);

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
      })
    );
    assert_eq!(
      press(&[Key::End, Key::Enter]),
      Some(OverviewAction::FocusWorkspace("4".to_string()))
    );
  }

  #[test]
  fn hidden_tabs_are_skipped_but_found() {
    let mut workspaces = workspaces();
    workspaces[0].windows.push(OverviewWindow {
      is_hidden_tab: true,
      ..window(13, "Docs", Rect::from_xy(0, 0, 960, 1000))
    });
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);
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
      Some(OverviewAction::FocusWindow(13)),
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

    assert_eq!(
      press(&mut state, &[Key::Enter]),
      Some(OverviewAction::FocusWindow(12))
    );
  }

  #[test]
  fn home_and_end_pick_the_outer_windows_while_picking() {
    let mut state = interaction();

    press(&mut state, &[Key::Left, Key::Space, Key::End]);
    assert_eq!(state.selected, 0);
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

    press(
      &mut state,
      &[Key::Tab, Key::Tab, Key::Space, Key::Space, Key::Tab],
    );
    assert_eq!(state.layout, OverviewLayoutMode::Grid);
    assert_eq!(state.mode, Mode::Carrying, "carrying works in the grid");
  }

  #[test]
  fn tab_cycles_carousel_grid_and_shelf() {
    let mut state = interaction();

    press(&mut state, &[Key::Tab]);
    assert_eq!(state.layout, OverviewLayoutMode::Grid);
    press(&mut state, &[Key::Tab]);
    assert_eq!(state.layout, OverviewLayoutMode::Shelf);
    press(&mut state, &[Key::Tab]);
    assert_eq!(state.layout, OverviewLayoutMode::Carousel);
  }

  fn shelf() -> Interaction {
    Interaction::new(&workspaces(), OverviewLayoutMode::Shelf, 6.0)
  }

  #[test]
  fn the_shelf_stages_the_selected_workspace() {
    let mut state = shelf();
    assert_eq!(state.staged, 1);

    press(&mut state, &[Key::Left]);
    assert_eq!((state.selected, state.staged), (0, 0));

    press(&mut state, &[Key::Down]);
    assert_eq!(state.mode, Mode::Windows, "down goes into the stage");
    assert_eq!(state.layout, OverviewLayoutMode::Shelf);
    assert_eq!(state.selected_window, Some(11));
  }

  #[test]
  fn carrying_on_the_shelf_keeps_the_stage_on_the_source() {
    let mut state = shelf();

    press(&mut state, &[Key::Space, Key::Space, Key::Right]);
    assert_eq!(state.mode, Mode::Carrying);
    assert_eq!(state.selected, 2);
    assert_eq!(state.staged, 1);
    assert_eq!(state.drop_target(), Some(2));

    press(&mut state, &[Key::Backspace]);
    assert_eq!(state.mode, Mode::Windows);
    assert_eq!(state.selected, 1, "back to picking on the stage");
    assert_eq!(state.selected_window, Some(21));

    let action = press(&mut state, &[Key::Space, Key::Left, Key::Space]);
    assert_eq!(
      action,
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "1".to_string(),
      })
    );
    assert_eq!(state.staged, 0, "the stage shows where it went");
  }

  #[test]
  fn the_shelf_reaches_the_special_workspace_with_the_arrows() {
    let workspaces = with_special();
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Shelf, 6.0);

    state.key(Key::End, &workspaces);
    assert!(workspaces[state.selected].is_special);
    assert_eq!(state.staged, state.selected);
  }

  #[test]
  fn clicking_a_shelf_card_stages_it_and_the_stage_goes_there() {
    let workspaces = workspaces();
    let mut state = shelf();
    let card = |index| Hit {
      workspace: Some(index),
      window: None,
    };

    state.mouse_down((10.0, 10.0), card(2));
    assert_eq!(state.mouse_up(&workspaces), None);
    assert_eq!(state.staged, 2);

    state.mouse_down((10.0, 10.0), card(2));
    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::FocusWorkspace("3".to_string()))
    );
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
    assert!(state.is_drop_target(1));

    press(&mut state, &[Key::Left]);
    assert!(state.is_drop_target(0));

    assert_eq!(
      press(&mut state, &[Key::Space]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "1".to_string(),
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
    let metrics = CardMetrics::new(1920.0, 1040.0, 1.0);
    let mut state = interaction();
    state.grid = GridCells::new(
      &metrics,
      &vec![CardSlot::new(&metrics, false); workspaces.len()],
      2,
    );

    press(&mut state, &[Key::Left, Key::Down]);
    assert_eq!(state.selected, 0, "no rows in the carousel");

    press(&mut state, &[Key::Tab, Key::Down]);
    assert_eq!(state.layout, OverviewLayoutMode::Grid);
    assert_eq!(state.selected, 2);

    press(&mut state, &[Key::Text('k')]);
    assert_eq!(state.selected, 0);
  }

  #[test]
  fn search_jumps_to_the_best_match() {
    let mut state = interaction();
    let query = "term".chars().map(Key::Text).collect::<Vec<_>>();

    press(&mut state, &[Key::Text('/')]);
    press(&mut state, &query);
    assert!(state.searching);
    assert_eq!(state.query, "term");
    assert_eq!(state.selected, 1, "typing doesn't move the selection");

    press(&mut state, &[Key::Enter]);
    assert!(!state.searching);
    assert_eq!(state.selected, 0);
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
    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::FocusWindow(11))
    );

    let card = Hit {
      workspace: Some(2),
      window: None,
    };
    state.mouse_down((10.0, 10.0), card);
    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::FocusWorkspace("3".to_string()))
    );

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
    assert!(state.is_drop_target(2));

    assert_eq!(
      state.mouse_up(&workspaces),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "3".to_string(),
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
    state.sync(&old, &new);

    assert_eq!(state.selected, 0, "still on workspace 2");
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
  fn s_goes_to_the_special_workspace_like_a_digit() {
    let workspaces = with_special();
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);
    let press = |state: &mut Interaction, keys: &[Key]| {
      keys
        .iter()
        .fold(None, |_, key| state.key(*key, &workspaces))
    };

    assert_eq!(
      press(&mut state, &[Key::Text('s')]),
      Some(OverviewAction::FocusWorkspace("special".to_string()))
    );
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Text('s')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
      }),
      "sends the picked window"
    );

    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);
    assert_eq!(
      press(&mut state, &[Key::Space, Key::Space, Key::Text('s')]),
      Some(OverviewAction::MoveWindow {
        hwnd: 21,
        workspace: "special".to_string(),
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
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);

    assert_eq!(
      state.key(Key::Digit(1), &workspaces),
      Some(OverviewAction::FocusWorkspace("code".to_string()))
    );
    assert_eq!(state.key(Key::Digit(4), &workspaces), None);
  }

  #[test]
  fn hints_mention_the_special_workspace() {
    let workspaces = with_special();
    let state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);

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
      let mut state = Interaction::new(&workspaces, layout, 6.0);
      if layout == OverviewLayoutMode::Grid {
        state.grid = GridCells::new(
          &CardMetrics::new(1920.0, 1080.0, 1.0),
          &workspaces
            .iter()
            .map(|w| {
              CardSlot::new(
                &CardMetrics::new(1920.0, 1080.0, 1.0),
                w.is_special,
              )
            })
            .collect::<Vec<_>>(),
          5,
        );
      }

      press(&mut state, &[Key::Right, Key::Right, Key::Right]);
      assert_eq!(state.selected, 2, "left/right stay in the row");
      press(&mut state, &[Key::End]);
      assert_eq!(state.selected, 2, "the row's end, not the dock");

      press(&mut state, &[Key::Home, Key::Down]);
      assert_eq!(state.selected, 3, "down goes to the dock");
      assert_eq!(
        state.row_selected, 0,
        "the row stays on the card the dock was entered from"
      );
      press(&mut state, &[Key::Down]);
      assert_eq!(state.selected, 3);
      press(&mut state, &[Key::Up]);
      assert_eq!(state.selected, 0, "up goes back where it came from");

      press(&mut state, &[Key::Down, Key::Right]);
      assert_eq!(state.selected, 0, "sideways leaves the dock too");
    }
  }

  #[test]
  fn opened_on_the_special_workspace_the_row_starts_at_its_first_card() {
    let mut workspaces = with_special();
    for workspace in &mut workspaces {
      workspace.is_focused = workspace.is_special;
    }

    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);
    assert_eq!(state.selected, 3);
    assert_eq!(state.row_selected, 0);

    state.key(Key::Up, &workspaces);
    assert_eq!(state.selected, 0);
  }

  #[test]
  fn a_carried_window_drops_on_the_dock_below() {
    let workspaces = with_special();
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 6.0);
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
      })
    );
  }
}
