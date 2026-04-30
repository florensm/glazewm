use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ActiveDrag, ContainerDto, DisplayState, GapsConfig, TilingDirection,
  WindowDto, WindowRuleConfig, WindowState,
};
use wm_platform::{NativeWindow, Rect, RectDelta};

use crate::{
  impl_common_getters, impl_container_debug, impl_tiling_size_getters,
  impl_window_getters,
  models::{
    Container, DirectionContainer, InsertionTarget,
    NativeWindowProperties, NonTilingWindow, TilingContainer,
    WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters, WindowGetters,
  },
};

#[derive(Clone)]
pub struct TilingWindow(Rc<RefCell<TilingWindowInner>>);

struct TilingWindowInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  native: NativeWindow,
  native_properties: NativeWindowProperties,
  state: WindowState,
  prev_state: Option<WindowState>,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
  has_custom_floating_placement: bool,
  gaps_config: GapsConfig,
  done_window_rules: Vec<WindowRuleConfig>,
  active_drag: Option<ActiveDrag>,
}

impl TilingWindow {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    id: Option<Uuid>,
    native: NativeWindow,
    properties: NativeWindowProperties,
    prev_state: Option<WindowState>,
    border_delta: RectDelta,
    floating_placement: Rect,
    has_custom_floating_placement: bool,
    gaps_config: GapsConfig,
    done_window_rules: Vec<WindowRuleConfig>,
    active_drag: Option<ActiveDrag>,
  ) -> Self {
    let window = TilingWindowInner {
      id: id.unwrap_or_else(Uuid::new_v4),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      native,
      native_properties: properties,
      state: WindowState::Tiling,
      prev_state,
      display_state: DisplayState::Shown,
      border_delta,
      has_pending_dpi_adjustment: false,
      floating_placement,
      has_custom_floating_placement,
      gaps_config,
      done_window_rules,
      active_drag,
    };

    Self(Rc::new(RefCell::new(window)))
  }

  pub fn to_non_tiling(
    &self,
    state: WindowState,
    insertion_target: Option<InsertionTarget>,
  ) -> NonTilingWindow {
    NonTilingWindow::new(
      Some(self.id()),
      self.native().clone(),
      self.native_properties().clone(),
      state,
      Some(WindowState::Tiling),
      self.border_delta(),
      insertion_target,
      self.floating_placement(),
      self.has_custom_floating_placement(),
      self.done_window_rules(),
      self.active_drag(),
    )
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;

    Ok(ContainerDto::Window(WindowDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      has_focus: self.has_focus(None),
      tiling_size: Some(self.tiling_size()),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
      state: self.state(),
      prev_state: self.prev_state(),
      display_state: self.display_state(),
      border_delta: self.border_delta(),
      floating_placement: self.floating_placement(),
      #[allow(clippy::cast_possible_wrap, clippy::unnecessary_cast)]
      handle: self.native().id().0 as isize,
      title: self.native_properties().title,
      #[cfg(target_os = "windows")]
      class_name: self.native_properties().class_name,
      process_name: self.native_properties().process_name,
      active_drag: self.active_drag(),
    }))
  }
}

impl_container_debug!(TilingWindow);
impl_common_getters!(TilingWindow);
impl_tiling_size_getters!(TilingWindow);
impl_window_getters!(TilingWindow);

impl PositionGetters for TilingWindow {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    let parent = self.parent().context("No parent container.")?;

    // All children of a stack share the stack rect offset below the tab bar.
    if let Some(stack) = parent.as_stack() {
      let stack_rect = stack.to_rect()?;
      let tab_h = stack.tab_bar_height_px();
      if tab_h > 0 {
        return Ok(Rect::from_ltrb(
          stack_rect.left,
          stack_rect.top + tab_h,
          stack_rect.right,
          stack_rect.bottom,
        ));
      }
      return Ok(stack_rect);
    }

    // Otherwise, use the normal tiling position calculation (same as
    // `impl_position_getters_as_resizable!` macro).
    let parent = parent
      .as_direction_container()
      .context("Parent does not have a tiling direction.")?;

    let parent_rect = parent.to_rect()?;

    let (horizontal_gap, vertical_gap) = self.inner_gaps()?;
    let inner_gap = match parent.tiling_direction() {
      TilingDirection::Vertical => vertical_gap,
      TilingDirection::Horizontal => horizontal_gap,
    };

    #[allow(
      clippy::cast_precision_loss,
      clippy::cast_possible_truncation,
      clippy::cast_possible_wrap
    )]
    let (width, height) = match parent.tiling_direction() {
      TilingDirection::Vertical => {
        let available_height = parent_rect.height()
          - inner_gap * self.tiling_siblings().count() as i32;

        let height =
          (self.tiling_size() * available_height as f32) as i32;

        (parent_rect.width(), height)
      }
      TilingDirection::Horizontal => {
        let available_width = parent_rect.width()
          - inner_gap * self.tiling_siblings().count() as i32;

        let width =
          (available_width as f32 * self.tiling_size()).round() as i32;

        (width, parent_rect.height())
      }
    };

    let (x, y) = {
      let mut prev_siblings = self
        .prev_siblings()
        .filter_map(|sibling| sibling.as_tiling_container().ok());

      match prev_siblings.next() {
        None => (parent_rect.x(), parent_rect.y()),
        Some(sibling) => {
          let sibling_rect = sibling.to_rect()?;

          match parent.tiling_direction() {
            TilingDirection::Vertical => (
              parent_rect.x(),
              sibling_rect.y() + sibling_rect.height() + inner_gap,
            ),
            TilingDirection::Horizontal => (
              sibling_rect.x() + sibling_rect.width() + inner_gap,
              parent_rect.y(),
            ),
          }
        }
      }
    };

    Ok(Rect::from_xy(x, y, width, height))
  }
}
