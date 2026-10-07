use anyhow::Context;
use tracing::{info, warn};
use wm_common::WindowState;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;

use crate::{
  commands::container::{
    attach_container, detach_container, flatten_child_split_containers,
    resize_tiling_container, set_focused_descendant,
  },
  models::{
    Container, InsertionTarget, StackContainer, WindowContainer, Workspace,
  },
  traits::{
    CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// Changes the state of every window in `stack`, so that the stack
/// floats, goes fullscreen, minimizes or tiles as a whole, like a single
/// window.
///
/// A minimized state is only applied as-is; minimizing the active window
/// natively is up to the caller.
#[allow(clippy::needless_pass_by_value)]
pub fn update_stack_state(
  stack: &StackContainer,
  target_state: WindowState,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if stack.state() == target_state {
    return Ok(());
  }

  info!("Updating stack state: {:?}.", target_state);

  let workspace = stack.workspace().context("No workspace.")?;
  let had_focus = has_workspace_focus(stack, &workspace);

  for window in stack.windows() {
    state.pending_sync.mark_window_state_change(window.id());
  }

  match (stack.is_tiling(), &target_state) {
    (false, WindowState::Tiling) => {
      tile_stack(stack, &workspace, config)?;
    }
    (true, _) => {
      untile_stack(stack, &workspace, &target_state)?;
    }
    (false, _) => {
      for window in stack.windows() {
        if let WindowContainer::NonTilingWindow(window) = window {
          if !window.state().is_same_state(&target_state)
            && window.active_drag().is_none()
          {
            window.set_prev_state(window.state());
          }

          window.set_state(target_state.clone());
        }
      }
    }
  }

  if had_focus {
    focus_within_workspace(stack);
  }

  if matches!(target_state, WindowState::Fullscreen(_)) {
    for window in stack.windows() {
      if let Err(err) = window.native().mark_fullscreen(true) {
        warn!("Failed to mark window as fullscreen: {}", err);
      }
    }
  }

  state
    .pending_sync
    .queue_containers_to_redraw(stack.windows())
    .queue_containers_to_redraw(workspace.tiling_children())
    .queue_workspace_to_reorder(workspace);

  Ok(())
}

/// Takes a tiling stack out of the layout and gives its windows
/// `target_state`.
fn untile_stack(
  stack: &StackContainer,
  workspace: &Workspace,
  target_state: &WindowState,
) -> anyhow::Result<()> {
  let parent = stack.parent().context("No parent.")?;

  stack.set_insertion_target(Some(InsertionTarget {
    target_parent: parent,
    target_index: stack.index(),
    prev_tiling_size: stack.tiling_size(),
    prev_sibling_count: stack.tiling_siblings().count(),
  }));

  // Floated windows keep the floating size of the active window, all in
  // one place.
  let placement = stack
    .active_child()
    .and_then(|child| child.as_window_container().ok())
    .map(|window| window.floating_placement());

  let ancestors = stack.ancestors().take(3).collect::<Vec<_>>();
  detach_container(stack.clone().into())?;

  // E.g. a split left with a single child.
  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor)?;
  }

  for window in stack.windows() {
    if let WindowContainer::TilingWindow(window) = &window {
      let non_tiling = window.to_non_tiling(target_state.clone(), None);
      replace_tab(stack, &window.clone().into(), non_tiling.into());
    }
  }

  // Attached as a non-tiling container, so the layout is left as is.
  attach_container(
    &stack.clone().into(),
    &workspace.clone().into(),
    None,
  )?;

  if let (Some(placement), Some(window)) =
    (placement, stack.windows().first())
  {
    window.set_floating_placement(keep_tab_bar_on_screen(
      stack, workspace, placement,
    ));
  }

  Ok(())
}

/// Puts a non-tiling stack back into the tiling layout, where it was
/// before if that place still exists.
fn tile_stack(
  stack: &StackContainer,
  workspace: &Workspace,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let insertion_target = stack.insertion_target().filter(|target| {
    target.target_parent.as_stack().is_none()
      && target
        .target_parent
        .workspace()
        .is_some_and(|ws| ws.id() == workspace.id())
  });

  detach_container(stack.clone().into())?;

  for window in stack.windows() {
    if let WindowContainer::NonTilingWindow(window) = &window {
      let tiling = window.to_tiling(config.value.gaps.clone());
      replace_tab(stack, &window.clone().into(), tiling.into());
    }
  }

  // Default to beside the last focused tiling window, but never inside
  // another stack.
  let (target_parent, target_index) = insertion_target
    .as_ref()
    .map(|target| (target.target_parent.clone(), target.target_index))
    .or_else(|| {
      let focused = workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window)?;

      let sibling = match focused.parent()?.as_stack() {
        Some(other_stack) => other_stack.clone().into(),
        None => focused,
      };

      Some((sibling.parent()?, sibling.index() + 1))
    })
    .unwrap_or((workspace.clone().into(), workspace.child_count()));

  attach_container(
    &stack.clone().into(),
    &target_parent,
    Some(target_index),
  )?;

  #[allow(clippy::cast_precision_loss)]
  if let (Some(target), Ok(tiling_stack)) =
    (&insertion_target, stack.as_tiling_container())
  {
    let size_scale = (target.prev_sibling_count + 1) as f32
      / (tiling_stack.tiling_siblings().count() + 1) as f32;

    resize_tiling_container(
      &tiling_stack,
      target.prev_tiling_size * size_scale,
    );
  }

  stack.set_insertion_target(None);

  Ok(())
}

/// Puts `replacement` in place of `tab`, a window of the same ID
/// converted to or from tiling.
///
/// Swapped in place rather than detached, which could flatten the stack.
fn replace_tab(
  stack: &StackContainer,
  tab: &Container,
  replacement: Container,
) {
  *tab.borrow_parent_mut() = None;
  *replacement.borrow_parent_mut() = Some(stack.clone().into());

  if let Some(slot) = stack
    .borrow_children_mut()
    .iter_mut()
    .find(|child| child.id() == tab.id())
  {
    *slot = replacement;
  }
}

/// Moves `placement` down or up as needed so that the tab bar above or
/// below it stays within the monitor's working area.
pub(crate) fn keep_tab_bar_on_screen(
  stack: &StackContainer,
  workspace: &Workspace,
  placement: wm_platform::Rect,
) -> wm_platform::Rect {
  let Ok(bounds) = workspace.to_rect() else {
    return placement;
  };

  let outer = stack.outer_rect(&placement);
  let shift = if outer.top < bounds.top {
    bounds.top - outer.top
  } else if outer.bottom > bounds.bottom {
    (bounds.bottom - outer.bottom).max(bounds.top - outer.top)
  } else {
    0
  };

  placement.translate_to_coordinates(placement.x(), placement.y() + shift)
}

/// Whether `stack` holds the most recently focused window of `workspace`.
fn has_workspace_focus(
  stack: &StackContainer,
  workspace: &Workspace,
) -> bool {
  workspace
    .descendant_focus_order()
    .next()
    .is_some_and(|focused| {
      focused
        .parent()
        .is_some_and(|parent| parent.id() == stack.id())
    })
}

/// Makes `stack` the most recently focused container within its
/// workspace.
fn focus_within_workspace(stack: &StackContainer) {
  let top_ancestor = stack
    .self_and_ancestors()
    .find(|ancestor| ancestor.parent().is_some_and(|p| p.is_workspace()));

  if let Some(top_ancestor) = top_ancestor {
    set_focused_descendant(&stack.clone().into(), Some(&top_ancestor));
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{
    FloatingStateConfig, FullscreenStateConfig, ParsedConfig, WindowState,
  };
  use wm_platform::Rect;

  use super::update_stack_state;
  use crate::{
    commands::{
      container::set_focused_descendant,
      window::{float_out_of_stack, join_stack, update_window_state},
    },
    models::{
      Monitor, NonTilingWindow, StackContainer, TilingWindow, Workspace,
    },
    traits::{CommonGetters, TilingSizeGetters, WindowGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
  }

  fn floating() -> WindowState {
    WindowState::Floating(FloatingStateConfig::default())
  }

  struct Setup {
    /// Never dropped, which would restore its mock windows via Win32
    /// calls.
    state: std::mem::ManuallyDrop<WmState>,
    config: UserConfig,
    workspace: Workspace,
    left: TilingWindow,
    stack: StackContainer,
  }

  /// A workspace with a window at 0.3 and a two-tab stack at 0.7.
  fn setup() -> Setup {
    let left = TilingWindow::mock().call();
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![first.into(), second.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![left.clone().into(), stack.clone().into()])
      .call();
    left.set_tiling_size(0.3);
    stack.set_tiling_size(0.7);

    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let state = WmState::mock(vec![monitor]);
    set_focused_descendant(&second.into(), None);

    Setup {
      state: std::mem::ManuallyDrop::new(state),
      config: UserConfig::from_parsed(ParsedConfig::default()),
      workspace,
      left,
      stack,
    }
  }

  #[test]
  fn a_floated_stack_leaves_the_layout_as_one_window() {
    let mut s = setup();

    update_stack_state(&s.stack, floating(), &mut s.state, &s.config)
      .unwrap();

    assert!(!s.stack.is_tiling());
    assert_eq!(s.stack.parent().unwrap().id(), s.workspace.id());
    assert!(approx_eq(s.left.tiling_size(), 1.0));
    assert_eq!(s.workspace.tiling_children().count(), 1);
    assert!(s.stack.as_tiling_container().is_err());

    let windows = s.stack.windows();
    assert_eq!(windows.len(), 2);
    assert!(windows.iter().all(|w| w.state() == floating()));
    assert_eq!(
      windows[0].floating_placement(),
      windows[1].floating_placement()
    );
  }

  #[test]
  fn a_tiled_again_stack_gets_its_slot_back() {
    let mut s = setup();

    update_stack_state(&s.stack, floating(), &mut s.state, &s.config)
      .unwrap();
    update_stack_state(
      &s.stack,
      WindowState::Tiling,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert!(s.stack.is_tiling());
    assert_eq!(s.stack.index(), 1);
    assert!(approx_eq(s.stack.tiling_size(), 0.7));
    assert!(approx_eq(s.left.tiling_size(), 0.3));
    assert!(s.stack.windows().iter().all(|w| w.is_tiling_window()));
  }

  #[test]
  fn a_window_state_change_applies_to_its_whole_stack() {
    let mut s = setup();
    let fullscreen =
      WindowState::Fullscreen(FullscreenStateConfig::default());
    let tab = s.stack.windows()[0].clone();

    let tab = update_window_state(
      tab,
      fullscreen.clone(),
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert_eq!(tab.parent().unwrap().id(), s.stack.id());
    assert!(s.stack.windows().iter().all(|w| w.state() == fullscreen));
    assert!(!s.stack.shows_tab_bar());
  }

  #[test]
  fn a_floating_window_joins_a_floating_stack_as_is() {
    let mut s = setup();
    update_stack_state(&s.stack, floating(), &mut s.state, &s.config)
      .unwrap();

    let shared = Rect::from_xy(100, 100, 600, 400);
    s.stack.windows()[0].set_floating_placement(shared.clone());

    let loose = NonTilingWindow::mock().call();
    crate::commands::container::attach_container(
      &loose.clone().into(),
      &s.workspace.clone().into(),
      None,
    )
    .unwrap();

    let joined =
      join_stack(loose.into(), &s.stack, 2, &mut s.state, &s.config)
        .unwrap();

    assert_eq!(joined.parent().unwrap().id(), s.stack.id());
    assert_eq!(joined.state(), floating());
    assert_eq!(joined.floating_placement(), shared);
    assert!(approx_eq(s.left.tiling_size(), 1.0));
  }

  #[test]
  fn a_floated_out_tab_tiles_back_into_its_stack() {
    let mut s = setup();
    // Named, so it outlives being left with a single window.
    s.stack.set_name("details".to_string());
    let tab = s.stack.windows()[0].clone();
    let tab_id = tab.id();

    float_out_of_stack(&tab, &mut s.state, &s.config).unwrap();

    let floated = s
      .workspace
      .children()
      .into_iter()
      .find(|c| c.id() == tab_id)
      .and_then(|c| c.as_window_container().ok())
      .unwrap();
    assert!(matches!(floated.state(), WindowState::Floating(_)));
    assert_eq!(s.stack.child_count(), 1);
    assert!(approx_eq(s.left.tiling_size(), 0.3));

    let tiled = update_window_state(
      floated,
      WindowState::Tiling,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert_eq!(tiled.parent().unwrap().id(), s.stack.id());
  }

  #[test]
  fn emptying_a_floating_stack_leaves_the_layout_alone() {
    let mut s = setup();
    s.stack.set_name("details".to_string());
    update_stack_state(&s.stack, floating(), &mut s.state, &s.config)
      .unwrap();

    for window in s.stack.windows() {
      crate::commands::container::detach_container(window.into()).unwrap();
    }

    assert!(s.stack.is_detached());
    assert!(approx_eq(s.left.tiling_size(), 1.0));
    assert_eq!(s.workspace.child_count(), 1);
  }

  #[test]
  fn a_floated_stack_tiles_back_into_its_split() {
    let left = TilingWindow::mock().call();
    let other = TilingWindow::mock().call();
    let tab = TilingWindow::mock().call();
    let stack = StackContainer::mock()
      .tiling_containers(vec![tab.into()])
      .call();
    let split = crate::models::SplitContainer::mock()
      .tiling_direction(wm_common::TilingDirection::Vertical)
      .tiling_containers(vec![stack.clone().into(), other.clone().into()])
      .call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![left.into(), split.clone().into()])
      .call();
    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    let mut state =
      std::mem::ManuallyDrop::new(WmState::mock(vec![monitor]));
    let config = UserConfig::from_parsed(ParsedConfig::default());

    update_stack_state(&stack, floating(), &mut state, &config).unwrap();
    assert_eq!(workspace.tiling_children().count(), 2);

    update_stack_state(&stack, WindowState::Tiling, &mut state, &config)
      .unwrap();

    assert_eq!(stack.parent().unwrap().id(), split.id());
    assert_eq!(stack.index(), 0);
    assert_eq!(other.parent().unwrap().id(), split.id());
  }

  #[test]
  fn a_minimized_stack_comes_back_to_its_slot() {
    let mut s = setup();
    let tab = s.stack.windows()[1].clone();
    tab.update_native_properties(|properties| {
      properties.is_minimized = true;
    });

    let tab = update_window_state(
      tab,
      WindowState::Minimized,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert!(s
      .stack
      .windows()
      .iter()
      .all(|w| w.state() == WindowState::Minimized));
    assert!(!s.stack.shows_tab_bar());
    assert!(approx_eq(s.left.tiling_size(), 1.0));

    tab.update_native_properties(|properties| {
      properties.is_minimized = false;
    });
    update_window_state(tab, WindowState::Tiling, &mut s.state, &s.config)
      .unwrap();

    assert!(s.stack.is_tiling());
    assert!(approx_eq(s.stack.tiling_size(), 0.7));
  }

  #[test]
  fn joining_a_minimized_stack_restores_it() {
    let mut s = setup();
    s.stack.windows()[1].update_native_properties(|properties| {
      properties.is_minimized = true;
    });
    update_stack_state(
      &s.stack,
      WindowState::Minimized,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    let joined = join_stack(
      s.left.clone().into(),
      &s.stack,
      0,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert!(s.stack.is_tiling());
    assert_eq!(joined.parent().unwrap().id(), s.stack.id());
    assert_eq!(s.stack.child_count(), 3);
  }

  #[test]
  fn a_tab_of_a_stack_can_start_a_new_named_stack() {
    let mut s = setup();
    update_stack_state(&s.stack, floating(), &mut s.state, &s.config)
      .unwrap();
    let tab = s.stack.windows()[0].clone();

    let moved = crate::commands::window::move_to_stack(
      tab,
      "notes",
      &mut s.state,
      &s.config,
    )
    .unwrap();

    let notes = moved.parent().unwrap();
    assert_eq!(notes.as_stack().unwrap().name().as_deref(), Some("notes"));
    assert_eq!(notes.parent().unwrap().id(), s.workspace.id());
    assert!(matches!(moved.state(), WindowState::Floating(_)));
  }

  #[test]
  fn a_tab_floated_out_of_a_removed_stack_tiles_back_beside_it() {
    let mut s = setup();
    let tab = s.stack.windows()[0].clone();
    let tab_id = tab.id();
    let other = s.stack.windows()[1].clone();

    float_out_of_stack(&tab, &mut s.state, &s.config).unwrap();
    assert!(s.stack.is_detached());

    let floated = s
      .workspace
      .children()
      .into_iter()
      .find(|c| c.id() == tab_id)
      .and_then(|c| c.as_window_container().ok())
      .unwrap();
    let tiled = update_window_state(
      floated,
      WindowState::Tiling,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert_eq!(tiled.parent().unwrap().id(), s.workspace.id());
    assert_eq!(tiled.index(), other.index() + 1);
    // Half of the stack it left.
    let tiled = tiled.as_tiling_container().unwrap();
    assert!(approx_eq(tiled.tiling_size(), 0.35));
  }

  #[test]
  fn absorbing_a_stack_keeps_its_tab_order() {
    let mut s = setup();
    s.config.value.stack.new_tab_position =
      wm_common::NewTabPosition::AfterActive;
    let left = s.left.clone();
    let order =
      s.stack.windows().iter().map(|w| w.id()).collect::<Vec<_>>();

    crate::commands::window::stack_absorb_neighbor(
      &left,
      &wm_platform::Direction::Right,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    let stack = left.parent().unwrap().as_stack().cloned().unwrap();
    let ids = stack.windows().iter().map(|w| w.id()).collect::<Vec<_>>();
    assert_eq!(ids, vec![left.id(), order[0], order[1]]);
  }

  #[test]
  fn the_only_tab_of_a_named_stack_tiles_back_into_its_slot() {
    let mut s = setup();
    s.stack.set_name("details".to_string());
    let first = s.stack.windows()[0].clone();
    crate::commands::window::toggle_stack(&first, &mut s.state, &s.config)
      .unwrap();
    let tab = s.stack.windows()[0].clone();
    let tab_id = tab.id();
    let stack_size = s.stack.tiling_size();
    let stack_index = s.stack.index();

    float_out_of_stack(&tab, &mut s.state, &s.config).unwrap();
    assert!(s.stack.is_detached());

    let floated = s
      .workspace
      .children()
      .into_iter()
      .find(|c| c.id() == tab_id)
      .and_then(|c| c.as_window_container().ok())
      .unwrap();
    let tiled = update_window_state(
      floated,
      WindowState::Tiling,
      &mut s.state,
      &s.config,
    )
    .unwrap();

    assert_eq!(tiled.index(), stack_index);
    let tiled = tiled.as_tiling_container().unwrap();
    assert!(approx_eq(tiled.tiling_size(), stack_size));
  }
}
