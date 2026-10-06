//! What the overview is doing, and how keys and the mouse change it, kept
//! free of drawing so it can be tested.
//!
//! The keyboard moves through three steps, one space bar at a time:
//! browsing workspaces, picking a window inside one, and carrying a window
//! to another workspace. The mouse does the same directly: click to go,
//! drag to move.

use crate::{
  OverviewAction, OverviewLayoutMode, OverviewWindow, OverviewWorkspace,
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

  grid_columns: usize,
}

impl Interaction {
  /// Starts out on the focused workspace.
  pub fn new(
    workspaces: &[OverviewWorkspace],
    layout: OverviewLayoutMode,
    grid_columns: usize,
    drag_threshold: f32,
  ) -> Self {
    Self {
      selected: workspaces
        .iter()
        .position(|workspace| workspace.is_focused)
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
      grid_columns: grid_columns.max(1),
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

  /// Whether workspace `index` is where the floating window would land.
  pub fn is_drop_target(&self, index: usize) -> bool {
    match &self.drag {
      Some(drag) => drag.target == Some(index),
      None => self.mode == Mode::Carrying && self.selected == index,
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
      Key::Tab => match self.layout {
        OverviewLayoutMode::Carousel => {
          self.layout = OverviewLayoutMode::Grid;
          // The grid is too small to pick a window in.
          if self.mode == Mode::Windows {
            self.mode = Mode::Spaces;
            self.selected_window = None;
          }
        }
        OverviewLayoutMode::Grid => {
          self.layout = OverviewLayoutMode::Carousel;
        }
      },
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
      Key::Left | Key::Text('h') => self.horizontal(-1, workspaces),
      Key::Right | Key::Text('l') => self.horizontal(1, workspaces),
      Key::Up | Key::Text('k') => self.vertical(-1, workspaces),
      Key::Down | Key::Text('j') => self.vertical(1, workspaces),
      Key::Home => self.edge(false, workspaces),
      Key::End => self.edge(true, workspaces),
      Key::Enter => return self.enter(workspaces),
      Key::Digit(digit) => {
        let name = digit_target(digit, workspaces)?.name.clone();

        return match self.mode {
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
        };
      }
      Key::Text(_) => {}
    }

    None
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
          self.layout = OverviewLayoutMode::Carousel;
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

  fn select(&mut self, index: usize) {
    if index != self.selected {
      self.selected = index;
      // The window cursor belongs to one workspace.
      self.selected_window = None;
    }
  }

  fn step(&mut self, delta: isize, workspaces: &[OverviewWorkspace]) {
    let last = workspaces.len().saturating_sub(1);
    let index = self.selected.saturating_add_signed(delta).min(last);
    self.select(index);
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
    } else if self.layout == OverviewLayoutMode::Grid {
      let columns = self.grid_columns.min(workspaces.len()).max(1);
      let columns = isize::try_from(columns).unwrap_or(1);
      self.step(delta * columns, workspaces);
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
      let last = workspaces.len().saturating_sub(1);
      self.select(if is_last { last } else { 0 });
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
        let minimized =
          workspace.windows.iter().filter(|w| w.is_minimized);
        ordered(workspace, Axis::Across)
          .into_iter()
          .chain(minimized)
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
    self.select(*index);

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
      return (
        format!("drop \"{}\" on a workspace", title(Some(drag.hwnd))),
        HintTone::Accent,
      );
    }

    match (self.mode, self.layout) {
      (Mode::Carrying, _) => (
        format!(
          "carrying \"{}\"   \u{b7}   \u{2190}\u{2192} or / to pick a workspace   \u{b7}   space to drop   \u{b7}   1\u{2013}9 drop on that one   \u{b7}   \u{232b} back",
          title(self.carried)
        ),
        HintTone::Accent,
      ),
      (Mode::Windows, _) => (
        "\u{2190}\u{2192} pick a window   \u{b7}   1\u{2013}9 send it there   \u{b7}   space to lift it   \u{b7}   \u{23ce} jump to it   \u{b7}   x close   \u{b7}   \u{232b} back"
          .to_string(),
        HintTone::Quiet,
      ),
      (Mode::Spaces, OverviewLayoutMode::Grid) => (
        "arrows workspaces   \u{b7}   drag windows between them   \u{b7}   / search   \u{b7}   \u{23ce} go there   \u{b7}   tab carousel   \u{b7}   esc close"
          .to_string(),
        HintTone::Quiet,
      ),
      (Mode::Spaces, OverviewLayoutMode::Carousel) => (
        "\u{2190}\u{2192} workspaces   \u{b7}   space to look inside   \u{b7}   / search   \u{b7}   \u{23ce} go there   \u{b7}   tab all workspaces   \u{b7}   esc close"
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

/// Visible windows of `workspace` in on-screen order.
fn ordered(
  workspace: &OverviewWorkspace,
  axis: Axis,
) -> Vec<&OverviewWindow> {
  let mut windows = workspace
    .windows
    .iter()
    .filter(|window| !window.is_minimized && window.rect.width() > 0)
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

/// Workspace digit key `digit` stands for, 0 being the tenth: the one
/// named after it when workspaces are numbered, else the one at its
/// position.
fn digit_target(
  digit: u8,
  workspaces: &[OverviewWorkspace],
) -> Option<&OverviewWorkspace> {
  let number = if digit == 0 { 10 } else { usize::from(digit) };
  let is_numbered = workspaces
    .iter()
    .any(|workspace| workspace.name.parse::<usize>().is_ok());

  if is_numbered {
    workspaces
      .iter()
      .find(|workspace| workspace.name.parse::<usize>() == Ok(number))
  } else {
    workspaces.get(number - 1)
  }
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

#[cfg(test)]
mod tests {
  use super::{HintTone, Hit, Interaction, Key, Mode};
  use crate::{
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
    }
  }

  /// Workspace "1" with two side-by-side windows, "2" focused with one,
  /// "3" with none.
  fn workspaces() -> Vec<OverviewWorkspace> {
    vec![
      OverviewWorkspace {
        name: "1".to_string(),
        label: "1".to_string(),
        is_focused: false,
        is_new: false,
        windows: vec![
          window(12, "Editor", Rect::from_xy(960, 0, 960, 1000)),
          window(11, "Terminal", Rect::from_xy(0, 0, 960, 1000)),
        ],
      },
      OverviewWorkspace {
        name: "2".to_string(),
        label: "2".to_string(),
        is_focused: true,
        is_new: false,
        windows: vec![window(
          21,
          "Browser",
          Rect::from_xy(0, 0, 1920, 1000),
        )],
      },
      OverviewWorkspace {
        name: "3".to_string(),
        label: "3".to_string(),
        is_focused: false,
        is_new: false,
        windows: vec![],
      },
    ]
  }

  fn interaction() -> Interaction {
    Interaction::new(&workspaces(), OverviewLayoutMode::Carousel, 5, 6.0)
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
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 5, 6.0);

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
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 5, 6.0);

    assert_eq!(
      state.key(Key::Digit(1), &workspaces),
      Some(OverviewAction::FocusWorkspace("web".to_string()))
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
      name: "4".to_string(),
      label: "4".to_string(),
      is_focused: false,
      is_new: true,
      windows: vec![],
    });
    let mut state =
      Interaction::new(&workspaces, OverviewLayoutMode::Carousel, 5, 6.0);

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

    press(&mut state, &[Key::Tab, Key::Space, Key::Space, Key::Tab]);
    assert_eq!(state.mode, Mode::Carrying, "carrying works in the grid");
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
    let mut state = Interaction::new(
      &workspaces(),
      OverviewLayoutMode::Carousel,
      2,
      6.0,
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
}
