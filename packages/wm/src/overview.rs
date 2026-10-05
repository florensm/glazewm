//! The window overview: live previews of the focused workspace's windows,
//! to pick one to focus.
//!
//! The overview is drawn and driven by `NativeOverview` on the event loop
//! thread; the WM decides what it shows and carries out what is picked.

use anyhow::Context;
use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{
  Color, Dispatcher, NativeOverview, OverviewAction, OverviewFrame,
  OverviewItem, OverviewStyle,
};

use crate::{
  models::{Container, WindowContainer, Workspace},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Container IDs of the windows an overview shows, by window handle.
type ShownWindows = Vec<(isize, Uuid)>;

/// The WM's side of the overview.
pub struct Overview {
  /// Created on first open.
  native: Option<NativeOverview>,

  /// What is shown, while open.
  open: Option<OpenOverview>,

  /// Where the native overview sends user actions, with their session.
  action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
}

struct OpenOverview {
  workspace_id: Uuid,

  windows: ShownWindows,

  /// Resolved once per open, since it can be read from a file.
  focused_border: Color,
}

impl Overview {
  pub fn new(
    action_tx: mpsc::UnboundedSender<(u64, OverviewAction)>,
  ) -> Self {
    Self {
      native: None,
      open: None,
      action_tx,
    }
  }

  pub fn is_open(&self) -> bool {
    self.open.is_some()
  }

  /// Session of the open overview, which its actions carry.
  pub fn open_session(&self) -> Option<u64> {
    self
      .open
      .as_ref()
      .and(self.native.as_ref())
      .map(NativeOverview::session)
  }

  /// The native overview, created on first use.
  fn native(
    &mut self,
    dispatcher: &Dispatcher,
  ) -> anyhow::Result<&mut NativeOverview> {
    if self.native.is_none() {
      let action_tx = self.action_tx.clone();
      let on_action = Box::new(move |session, action| {
        if let Err(err) = action_tx.send((session, action)) {
          tracing::warn!("Failed to send overview action: {err}");
        }
      });

      self.native = Some(NativeOverview::create(dispatcher, on_action)?);
    }

    self.native.as_mut().context("No native overview.")
  }

  /// Marks the overview closed, returning the container ID of the shown
  /// window with handle `picked`, if any.
  ///
  /// The native overview stays up until the next [`sync_overview`], so it
  /// only hides once focus has moved on; hiding the foreground window
  /// first would let the OS pick the next one.
  pub fn close(&mut self, picked: Option<isize>) -> Option<Uuid> {
    let open = self.open.take()?;

    open
      .windows
      .iter()
      .find(|(hwnd, _)| Some(*hwnd) == picked)
      .map(|(_, id)| *id)
  }
}

/// Opens the overview on the focused workspace, or closes it if open,
/// giving focus back to the focused window.
pub fn toggle_overview(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if !state.overview.is_open() {
    return open_overview(state, config);
  }

  state.overview.close(None);
  state.pending_sync.queue_focus_change();
  Ok(())
}

fn open_overview(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace_id = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No focused workspace.")?
    .id();

  let mut open = OpenOverview {
    workspace_id,
    windows: Vec::new(),
    focused_border: config.value.overview.focused_border_color.resolve(),
  };

  let Some((frame, windows)) = current_frame(state, &open, config)? else {
    tracing::info!("No windows to show in the overview.");
    return Ok(());
  };
  open.windows = windows;

  state.overview.native(&state.dispatcher)?.show(frame);
  state.overview.open = Some(open);
  Ok(())
}

/// Brings the native overview in line with the WM's state: hides it once
/// closed, and shows windows as they come, go or change while open.
///
/// Called at the end of a platform sync, after focus has been synced.
pub fn sync_overview(state: &mut WmState, config: &UserConfig) {
  let frame = state
    .overview
    .open
    .as_ref()
    .map(|open| current_frame(state, open, config));

  let overview = &mut state.overview;

  match (frame, &mut overview.open, &mut overview.native) {
    (Some(Ok(Some((frame, windows)))), Some(open), Some(native)) => {
      open.windows = windows;
      native.show(frame);
    }
    (frame, _, native) => {
      if let Some(Err(err)) = frame {
        tracing::warn!("Closing the overview: {err:?}");
      }

      overview.open = None;
      if let Some(native) = native {
        native.hide();
      }
    }
  }
}

/// What the open overview shows now, with the IDs of its windows, or
/// `None` once its workspace lost focus or has no windows left.
fn current_frame(
  state: &WmState,
  open: &OpenOverview,
  config: &UserConfig,
) -> anyhow::Result<Option<(OverviewFrame, ShownWindows)>> {
  let Some(workspace) = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .filter(|workspace| workspace.id() == open.workspace_id)
  else {
    return Ok(None);
  };

  let windows = overview_windows(&workspace);
  if windows.is_empty() {
    return Ok(None);
  }

  let frame = overview_frame(state, &workspace, &windows, open, config)?;
  Ok(Some((frame, window_ids(&windows))))
}

/// Windows the overview shows on `workspace`: tiling windows in layout
/// order, then the others, without minimized ones.
fn overview_windows(workspace: &Workspace) -> Vec<WindowContainer> {
  fn collect(container: &Container, windows: &mut Vec<WindowContainer>) {
    for child in container.children() {
      match child.as_window_container() {
        Ok(window) => windows.push(window),
        Err(_) => collect(&child, windows),
      }
    }
  }

  let mut windows = Vec::new();
  collect(&workspace.as_container(), &mut windows);

  windows.retain(|window| window.state() != WindowState::Minimized);
  windows.sort_by_key(|window| window.state() != WindowState::Tiling);
  windows
}

fn window_ids(windows: &[WindowContainer]) -> ShownWindows {
  windows
    .iter()
    .map(|window| (window.native().id().0, window.id()))
    .collect()
}

/// What the overview shows for `windows` of `workspace`.
fn overview_frame(
  state: &WmState,
  workspace: &Workspace,
  windows: &[WindowContainer],
  open: &OpenOverview,
  config: &UserConfig,
) -> anyhow::Result<OverviewFrame> {
  let monitor = workspace.monitor().context("No monitor.")?;
  let monitor = monitor.native_properties();
  let scale = Some(monitor.scale_factor);
  let overview = &config.value.overview;

  let focused_id = state.focused_container().map(|focused| focused.id());

  let items = windows
    .iter()
    .map(|window| {
      let properties = window.native_properties();
      let title = properties.title.trim();

      OverviewItem {
        hwnd: window.native().id().0,
        title: if title.is_empty() {
          properties.process_name.clone()
        } else {
          title.to_string()
        },
      }
    })
    .collect();

  Ok(OverviewFrame {
    focused_index: windows
      .iter()
      .position(|window| Some(window.id()) == focused_id),
    items,
    style: OverviewStyle {
      background: overview.background_color,
      selection: overview.selection_color,
      focused_border: open.focused_border,
      text: overview.text_color,
      font_family: overview.font_family.clone(),
      font_size: overview.font_size.to_px(0, scale),
      gap: overview.gap.to_px(monitor.working_area.width(), scale),
      scale_factor: monitor.scale_factor,
    },
    rect: monitor.working_area,
  })
}
