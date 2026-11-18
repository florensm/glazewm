//! How the WM draws windows: the effects of their effect set, and the
//! overlays it keeps around them.

use std::collections::HashMap;

use uuid::Uuid;
use wm_common::{
  EffectSet, WindowBorderDto, WindowDto, WindowOverlayDto,
  WindowOverlayKind, WindowState, WindowStyleDto,
};
use wm_platform::{
  NativeBackdropOverlay, NativeBorderOverlay, OverlayParams, Rect,
};

use crate::{
  commands::general::{surrogate_effects_for, SyncableOverlay},
  layout::is_hidden_in_layout,
  models::{is_inactive_stack_child, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  window_overlays::WindowOverlays,
  wm_state::WmState,
};

/// How the WM draws windows, resolved once: its effects on the focused
/// window, on urgent ones and on the others.
pub struct WindowLooks {
  focused_id: Option<Uuid>,
  focused: WindowLook,
  urgent: WindowLook,
  other: WindowLook,
}

/// The effects the WM gives a window.
pub struct WindowLook {
  pub opacity: u8,
  pub overlays: OverlayParams,
}

impl WindowLooks {
  pub fn new(state: &WmState, config: &UserConfig) -> Self {
    let look = |set| {
      let (opacity, _, overlays) = surrogate_effects_for(set, config);
      WindowLook { opacity, overlays }
    };

    Self {
      focused_id: state.focused_container().map(|focused| focused.id()),
      focused: look(EffectSet::Focused),
      urgent: look(EffectSet::Urgent),
      other: look(EffectSet::Other),
    }
  }

  pub fn of(&self, window: &WindowContainer) -> &WindowLook {
    match window.effect_set(self.focused_id == Some(window.id())) {
      EffectSet::Focused => &self.focused,
      EffectSet::Urgent => &self.urgent,
      EffectSet::Other => &self.other,
    }
  }
}

/// Whether `window`'s overlays are on screen, for a preview to show them:
/// it is in view on a shown workspace.
///
/// The WM hides the others, which DWM composes for no thumbnail; cloaked
/// instead, they showed in previews as small, stale boxes.
pub fn shows_overlays(window: &WindowContainer) -> bool {
  window.state() != WindowState::Minimized
    && !is_hidden_in_layout(window)
    && window
      .workspace()
      .is_some_and(|workspace| workspace.is_displayed())
}

/// `window`'s overlay of kind `O`, while the WM draws one with `params`
/// for it: its handle, and its whole window's rect with the window's
/// visible frame at `frame`.
pub fn overlay_bounds<O: SyncableOverlay>(
  overlays: &HashMap<Uuid, WindowOverlays>,
  window: &WindowContainer,
  params: Option<O::Params>,
  frame: &Rect,
  config: &UserConfig,
) -> Option<(isize, Rect)> {
  let params = params.filter(|_| !O::suppressed_for(window, config))?;
  let overlay = overlays.get(&window.id()).and_then(O::get)?;

  Some((
    overlay.hwnd().0,
    O::bounds(&O::overlay_rect(window, frame), &params),
  ))
}

/// Fills in a window DTO's `style`, for windows with a picture of their
/// own: not minimized ones, nor a stack's hidden tabs.
pub fn style_filler<'a>(
  state: &'a WmState,
  config: &'a UserConfig,
) -> impl FnMut(&mut WindowDto) + 'a {
  let looks = WindowLooks::new(state, config);

  move |dto| {
    dto.style = state
      .container_by_id(dto.id)
      .and_then(|container| container.as_window_container().ok())
      .filter(|window| {
        window.state() != WindowState::Minimized
          && !is_inactive_stack_child(window)
      })
      .and_then(|window| window_style(state, &window, &looks, config));
  }
}

fn window_style(
  state: &WmState,
  window: &WindowContainer,
  looks: &WindowLooks,
  config: &UserConfig,
) -> Option<WindowStyleDto> {
  // Where the layout rests it, as `platform_sync` places it.
  let bounds = window
    .to_rect()
    .ok()?
    .apply_delta(&window.total_border_delta().ok()?, None);
  let frame = window.frame_at(&bounds);
  let look = looks.of(window);

  let overlays = if shows_overlays(window) {
    [
      (
        WindowOverlayKind::Backdrop,
        overlay_bounds::<NativeBackdropOverlay>(
          &state.window_overlays,
          window,
          look.overlays.backdrop,
          &frame,
          config,
        ),
      ),
      (
        WindowOverlayKind::Border,
        overlay_bounds::<NativeBorderOverlay>(
          &state.window_overlays,
          window,
          look.overlays.border,
          &frame,
          config,
        ),
      ),
    ]
    .into_iter()
    .filter_map(|(kind, found)| {
      found.map(|(handle, rect)| WindowOverlayDto { handle, kind, rect })
    })
    .collect()
  } else {
    Vec::new()
  };

  let border = look
    .overlays
    .border
    .filter(|params| {
      params.opacity > 0.0
        && !NativeBorderOverlay::suppressed_for(window, config)
    })
    .map(|params| WindowBorderDto {
      color: format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        params.color.r, params.color.g, params.color.b, params.color.a
      ),
      width: params.width,
      corner_radius: params.corner_radius,
      opacity: params.opacity,
    });

  Some(WindowStyleDto {
    frame,
    bounds,
    opacity: look.opacity,
    companion_handle: wm_platform::companion_of(&window.native()),
    overlays,
    border,
  })
}
