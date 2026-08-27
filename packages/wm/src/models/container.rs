use std::{
  cell::{Ref, RefMut},
  collections::VecDeque,
};

use ambassador::Delegate;
use uuid::Uuid;
use wm_common::{
  ActiveDrag, ContainerDto, DisplayState, GapsConfig, TilingDirection,
  WindowRuleConfig, WindowState,
};
use wm_platform::{Direction, NativeWindow, Rect, RectDelta};

#[allow(clippy::wildcard_imports)]
use crate::{
  models::{
    Monitor, NativeWindowProperties, NonTilingWindow, RootContainer,
    SplitContainer, TilingWindow, Workspace,
  },
  traits::*,
  user_config::UserConfig,
};

/// A container of any type.
///
/// Uses:
///
///  * [`wm_macros::SubEnum`] to define subtypes of containers.
///  * [`wm_macros::EnumFromInner`] to define conversions between the enum
///    and wrapped types.
///  * [`ambassador::Delegate`] to delegate common getters to the contained
///    types. E.g. implements [`CommonGetters`] for [Container] by
///    forwarding the call to the item contained in the enum variant.
///
/// # Example
/// Conversion between the different container types:
/// ```
/// use wm::models::{Container, DirectionContainer, SplitContainer, TilingContainer};
/// use wm::traits::{TilingSizeGetters, TilingDirectionGetters};
///
/// fn example(split: SplitContainer) {
///   // Convert a `SplitContainer` into a `Container`
///   let container: Container = split.into(); // Will be a `Container::Split`
///
///   // Could also have gone straight to a [TilingContainer] from SplitContainer
///   // let tiling: TilingContainer = split.into(); // Will be a `TilingContainer::Split`
///
///   // Try to convert a [Container] into a sub container type ([TilingContainer] in this case).
///   let tiling: TilingContainer = container.try_into().unwrap(); // Will be a `TilingContainer::Split`
///   tiling.tiling_size(); // Can use methods from the `TilingSizeGetters` trait.
///
///   // Try to convert a one sub container type into another. ([TilingContainer] to [DirectionContainer] in this case).
///   let direction: DirectionContainer = tiling.try_into().unwrap(); // Will be a `DirectionContainer::Split`
///   direction.tiling_direction(); // Can use methods from the `TilingDirectionGetters` trait.
///
///   // Convert a sub container back into a [Container]
///   let container: Container = direction.into(); // Will be a `Container::Split`
/// }
/// ```
#[derive(Clone, Debug, wm_macros::EnumFromInner, Delegate, wm_macros::SubEnum)]
#[delegate(CommonGetters)]
#[delegate(PositionGetters)]
#[subenum(defaults, {
  /// Subenum of [Container]
  #[derive(Clone, Debug, Delegate, wm_macros::EnumFromInner)]
  #[delegate(CommonGetters)]
  #[delegate(PositionGetters)]
})]
#[subenum(TilingContainer, {
  /// Subset of containers that implement the following traits:
  /// * `CommonGetters`
  /// * `PositionGetters`
  /// * `TilingSizeGetters`
  #[delegate(TilingSizeGetters)]
})]
#[subenum(WindowContainer, {
  /// Subset of containers that implement the following traits:
  /// * `CommonGetters`
  /// * `PositionGetters`
  /// * `WindowGetters`
  #[delegate(WindowGetters)]
})]
#[subenum(DirectionContainer, {
  /// Subset of containers that implement the following traits:
  /// * `CommonGetters`
  /// * `PositionGetters`
  /// * `DirectionGetters`
  #[delegate(TilingDirectionGetters)]
})]
pub enum Container {
  Root(RootContainer),
  Monitor(Monitor),
  #[subenum(DirectionContainer)]
  Workspace(Workspace),
  #[subenum(TilingContainer, DirectionContainer)]
  Split(SplitContainer),
  #[subenum(TilingContainer, WindowContainer)]
  TilingWindow(TilingWindow),
  #[subenum(WindowContainer)]
  NonTilingWindow(NonTilingWindow),
}

impl PartialEq for Container {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for Container {}

impl Container {
  /// Returns `true` if this is a `Container::TilingWindow`.
  #[must_use]
  pub fn is_tiling_window(&self) -> bool {
    matches!(self, Self::TilingWindow(_))
  }

  /// Returns `true` if this is a `Container::Split`.
  #[must_use]
  pub fn is_split(&self) -> bool {
    matches!(self, Self::Split(_))
  }

  /// Returns `true` if this is a `Container::Workspace`.
  #[must_use]
  pub fn is_workspace(&self) -> bool {
    matches!(self, Self::Workspace(_))
  }

  /// Returns the inner `SplitContainer` if this is a `Container::Split`.
  #[must_use]
  pub fn as_split(&self) -> Option<&SplitContainer> {
    match self {
      Self::Split(split) => Some(split),
      _ => None,
    }
  }

  /// Returns the inner `Workspace` if this is a `Container::Workspace`.
  #[must_use]
  pub fn as_workspace(&self) -> Option<&Workspace> {
    match self {
      Self::Workspace(workspace) => Some(workspace),
      _ => None,
    }
  }

  /// Returns the inner `Monitor` if this is a `Container::Monitor`.
  #[must_use]
  pub fn as_monitor(&self) -> Option<&Monitor> {
    match self {
      Self::Monitor(monitor) => Some(monitor),
      _ => None,
    }
  }

  /// Returns the inner `NonTilingWindow` if this is a
  /// `Container::NonTilingWindow`.
  #[must_use]
  pub fn as_non_tiling_window(&self) -> Option<&NonTilingWindow> {
    match self {
      Self::NonTilingWindow(window) => Some(window),
      _ => None,
    }
  }
}

impl PartialEq for TilingContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for TilingContainer {}

impl TilingContainer {
  /// Returns `true` if this is a `TilingContainer::TilingWindow`.
  #[must_use]
  pub fn is_tiling_window(&self) -> bool {
    matches!(self, Self::TilingWindow(_))
  }
}

impl PartialEq for WindowContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for WindowContainer {}

impl WindowContainer {
  /// Returns `true` if this is a `WindowContainer::TilingWindow`.
  #[must_use]
  pub fn is_tiling_window(&self) -> bool {
    matches!(self, Self::TilingWindow(_))
  }

  /// Returns the inner `NonTilingWindow` if this is a
  /// `WindowContainer::NonTilingWindow`.
  #[must_use]
  pub fn as_non_tiling_window(&self) -> Option<&NonTilingWindow> {
    match self {
      Self::NonTilingWindow(window) => Some(window),
      Self::TilingWindow(_) => None,
    }
  }
}

impl std::fmt::Display for WindowContainer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // Truncate title if longer than 20 chars. Need to use `chars()`
    // instead of byte slices to handle invalid byte indices.
    let title = {
      let title = self.native_properties().title;
      if title.len() > 20 {
        format!("{}...", title.chars().take(17).collect::<String>())
      } else {
        title
      }
    };

    let class = {
      #[cfg(target_os = "windows")]
      {
        self.native_properties().class_name
      }
      #[cfg(not(target_os = "windows"))]
      {
        String::new()
      }
    };

    let process = self.native_properties().process_name;

    write!(
      f,
      "Window(id={:?}, process={}, class={}, title={})",
      self.native().id(),
      process,
      class,
      title,
    )?;

    Ok(())
  }
}

impl PartialEq for DirectionContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for DirectionContainer {}

impl DirectionContainer {
  /// Returns `true` if this is a `DirectionContainer::Workspace`.
  #[must_use]
  pub fn is_workspace(&self) -> bool {
    matches!(self, Self::Workspace(_))
  }
}

/// Implements the `Debug` trait for a given container struct.
///
/// Expects that the struct has a `to_dto()` method.
#[macro_export]
macro_rules! impl_container_debug {
  ($type:ty) => {
    impl std::fmt::Debug for $type {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(
          &self.to_dto().map_err(|_| std::fmt::Error),
          f,
        )
      }
    }
  };
}
