use anyhow::Context;

use super::{
  activate_workspace, focus_workspace, move_workspace_to_monitor,
};
use crate::{
  commands::window::move_window_to_workspace,
  models::{Monitor, WindowContainer, Workspace, WorkspaceTarget},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Summons the special workspace to the focused monitor and focuses it,
/// or goes back to the previous workspace when it is already focused.
pub fn focus_special_workspace(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let target_name = toggle_target(&focused_workspace, state, config)?;

  if !focused_workspace.is_special() {
    let monitor = focused_workspace.monitor().context("No monitor.")?;
    summon_special_workspace(&monitor, state, config)?;
  }

  focus_workspace(WorkspaceTarget::Name(target_name), state, config)
}

/// Moves `window` to the special workspace, or back out of it to the
/// previous workspace. Focus stays where it is.
pub fn move_window_to_special_workspace(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace = window.workspace().context("No workspace.")?;
  let target_name = toggle_target(&workspace, state, config)?;

  move_window_to_workspace(
    window,
    WorkspaceTarget::Name(target_name),
    state,
    config,
  )
}

/// Name of the workspace a toggle from `workspace` goes to: out of the
/// special workspace to the workspace before it, else into it.
fn toggle_target(
  workspace: &Workspace,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<String> {
  if !workspace.is_special() {
    return Ok(config.value.special_workspace.name.clone());
  }

  let monitor = workspace.monitor().context("No monitor.")?;
  state
    .workspace_before_special(&monitor)
    .context("No workspace to go back to from the special workspace.")
}

/// Activates the special workspace on `monitor`, or moves it there from
/// another monitor.
fn summon_special_workspace(
  monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let name = &config.value.special_workspace.name;

  match state.workspace_by_name(name) {
    None => {
      activate_workspace(Some(name), Some(monitor.clone()), state, config)
    }
    Some(workspace)
      if workspace
        .monitor()
        .is_some_and(|other| other.id() != monitor.id()) =>
    {
      move_workspace_to_monitor(&workspace, monitor, state, config)
    }
    Some(_) => Ok(()),
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{Layout, ParsedConfig};
  use wm_platform::Rect;

  use super::{focus_special_workspace, move_window_to_special_workspace};
  use crate::{
    commands::{
      container::set_focused_descendant, workspace::focus_workspace,
    },
    models::{
      Container, Monitor, TilingWindow, Workspace, WorkspaceTarget,
    },
    test_utils::{
      mock_workspace_configs, MOCK_MONITOR_HEIGHT, MOCK_MONITOR_WIDTH,
    },
    traits::CommonGetters,
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// Config declaring workspaces "1" and "2".
  fn config() -> UserConfig {
    UserConfig::from_parsed(ParsedConfig {
      workspaces: mock_workspace_configs(&["1", "2"]),
      ..ParsedConfig::default()
    })
  }

  /// Two side-by-side monitors showing "1" (holding `window`, if any) and
  /// "2", with "1" focused.
  fn two_monitors(
    window: Option<&TilingWindow>,
  ) -> (WmState, Monitor, Monitor) {
    let first = Workspace::mock()
      .name("1".to_string())
      .tiling_containers(
        window
          .map(|window| window.clone().into())
          .into_iter()
          .collect(),
      )
      .call();
    let left = Monitor::mock().workspaces(vec![first.clone()]).call();

    let bounds = Rect::from_xy(
      MOCK_MONITOR_WIDTH,
      0,
      MOCK_MONITOR_WIDTH,
      MOCK_MONITOR_HEIGHT,
    );
    let right = Monitor::mock()
      .bounds(bounds.clone())
      .working_area(bounds)
      .workspaces(vec![Workspace::mock().name("2".to_string()).call()])
      .call();

    let state = WmState::mock(vec![left.clone(), right.clone()]);
    let focus_target: Container =
      window.map_or_else(|| first.into(), |window| window.clone().into());
    set_focused_descendant(&focus_target, None);

    (state, left, right)
  }

  fn focused_workspace(state: &WmState) -> Workspace {
    state
      .focused_container()
      .and_then(|focused| focused.workspace())
      .expect("focused workspace")
  }

  #[test]
  fn focus_toggles_into_the_special_workspace_and_back() {
    let config = config();
    let (mut state, left, _) = two_monitors(None);

    focus_special_workspace(&mut state, &config).unwrap();
    let special = focused_workspace(&state);
    assert!(special.is_special());
    assert_eq!(special.config().name, "special");
    assert_eq!(special.layout(), Layout::Scrolling);
    assert_eq!(special.monitor().map(|m| m.id()), Some(left.id()));
    assert_eq!(state.recent_workspace_name.as_deref(), Some("1"));

    focus_special_workspace(&mut state, &config).unwrap();
    assert_eq!(focused_workspace(&state).config().name, "1");
    assert_eq!(
      state.recent_workspace_name.as_deref(),
      Some("1"),
      "the special workspace is never recent"
    );
  }

  #[test]
  fn focus_summons_the_special_workspace_to_the_focused_monitor() {
    let config = config();
    let (mut state, left, right) = two_monitors(None);

    focus_special_workspace(&mut state, &config).unwrap();
    focus_workspace(
      WorkspaceTarget::Name("2".to_string()),
      &mut state,
      &config,
    )
    .unwrap();
    assert_eq!(focused_workspace(&state).config().name, "2");

    focus_special_workspace(&mut state, &config).unwrap();
    let special = focused_workspace(&state);
    assert!(special.is_special());
    assert_eq!(special.monitor().map(|m| m.id()), Some(right.id()));
    assert_eq!(
      left
        .displayed_workspace()
        .map(|workspace| workspace.config().name)
        .as_deref(),
      Some("1")
    );

    // Back to what this monitor showed before.
    focus_special_workspace(&mut state, &config).unwrap();
    assert_eq!(focused_workspace(&state).config().name, "2");
  }

  #[test]
  fn move_sends_a_window_to_the_special_workspace_and_back() {
    let config = config();
    let window = TilingWindow::mock().call();
    let (mut state, _, _) = two_monitors(Some(&window));

    move_window_to_special_workspace(
      window.clone().into(),
      &mut state,
      &config,
    )
    .unwrap();
    let special = window.workspace().expect("workspace");
    assert!(special.is_special());
    assert_eq!(
      focused_workspace(&state).config().name,
      "1",
      "focus stays"
    );

    move_window_to_special_workspace(
      window.clone().into(),
      &mut state,
      &config,
    )
    .unwrap();
    assert_eq!(
      window.workspace().map(|w| w.config().name).as_deref(),
      Some("1")
    );
  }
}
