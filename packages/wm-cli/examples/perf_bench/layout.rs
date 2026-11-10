//! Which workspace each window is on, read from the WM and restored to it.
//!
//! `switch_modes.ps1` restarts the WM twice, and a restart manages every
//! window onto its monitor's displayed workspace. Windows are matched by
//! handle, the one ID a restart keeps.

use std::{fmt::Write as _, path::Path};

use anyhow::{bail, Context};
use uuid::Uuid;
use wm_common::{
  ClientResponseData, ContainerDto, WindowDto, WindowState, WorkspaceDto,
};
use wm_ipc_client::IpcClient;

/// Every workspace the WM has active.
pub async fn query_workspaces(
  client: &mut IpcClient,
) -> anyhow::Result<Vec<WorkspaceDto>> {
  let message = "query workspaces";
  client.send(message).await?;
  let response = client
    .client_response(message)
    .await
    .context("No response to workspace query.")?;

  let Some(ClientResponseData::Workspaces(data)) = response.data else {
    bail!("Unexpected response to workspace query.");
  };

  Ok(
    data
      .workspaces
      .into_iter()
      .filter_map(|container| match container {
        ContainerDto::Workspace(workspace) => Some(workspace),
        _ => None,
      })
      .collect(),
  )
}

/// The windows in `children`, through splits and stacks.
pub fn windows_in(children: &[ContainerDto]) -> Vec<&WindowDto> {
  children
    .iter()
    .flat_map(|child| match child {
      ContainerDto::Window(window) => vec![window],
      ContainerDto::Split(split) => windows_in(&split.children),
      ContainerDto::Stack(stack) => windows_in(&stack.children),
      _ => Vec::new(),
    })
    .collect()
}

/// The tiled windows of `workspace`.
pub fn tiled_windows(workspace: &WorkspaceDto) -> Vec<&WindowDto> {
  windows_in(&workspace.children)
    .into_iter()
    .filter(|window| window.state == WindowState::Tiling)
    .collect()
}

/// Writes each window's workspace to `path`, as `handle<TAB>workspace`
/// lines after a `focused<TAB>workspace` one.
pub async fn save(
  client: &mut IpcClient,
  path: &Path,
) -> anyhow::Result<()> {
  let workspaces = query_workspaces(client).await?;
  let mut text = String::new();

  if let Some(focused) = workspaces.iter().find(|ws| ws.has_focus) {
    let _ = writeln!(text, "focused\t{}", focused.name);
  }

  for workspace in &workspaces {
    for window in windows_in(&workspace.children) {
      let _ = writeln!(text, "{}\t{}", window.handle, workspace.name);
    }
  }

  std::fs::write(path, text)
    .with_context(|| format!("Failed to write {}.", path.display()))
}

/// Moves each window saved in `path` back to its workspace, then focuses
/// the workspace that had focus. A window that has since closed is
/// skipped; one that can't be moved is reported and left.
pub async fn restore(
  client: &mut IpcClient,
  path: &Path,
) -> anyhow::Result<()> {
  let text = std::fs::read_to_string(path)
    .with_context(|| format!("Failed to read {}.", path.display()))?;

  let current: Vec<(isize, Uuid, String)> = query_workspaces(client)
    .await?
    .iter()
    .flat_map(|workspace| {
      windows_in(&workspace.children)
        .into_iter()
        .map(|window| (window.handle, window.id, workspace.name.clone()))
        .collect::<Vec<_>>()
    })
    .collect();

  let mut focused = None;

  for line in text.lines() {
    let Some((key, workspace)) = line.split_once('\t') else {
      continue;
    };

    if key == "focused" {
      focused = Some(workspace.to_string());
      continue;
    }

    let Ok(handle) = key.parse::<isize>() else {
      continue;
    };

    let Some((_, id, now_on)) =
      current.iter().find(|(current, ..)| *current == handle)
    else {
      continue;
    };

    if now_on != workspace {
      let message =
        format!("command --id {id} move --workspace {workspace}");
      if let Err(err) = send_command(client, &message).await {
        println!("Couldn't move window {handle:#x} back: {err}");
      }
    }
  }

  if let Some(workspace) = focused {
    send_command(
      client,
      &format!("command focus --workspace {workspace}"),
    )
    .await?;
  }

  Ok(())
}

/// Sends `message` and fails unless the WM reports success.
pub async fn send_command(
  client: &mut IpcClient,
  message: &str,
) -> anyhow::Result<()> {
  client.send(message).await?;
  let response = client
    .client_response(message)
    .await
    .context("No response to command.")?;

  if !response.success {
    bail!(
      "Command '{message}' failed: {}",
      response.error.unwrap_or_default()
    );
  }

  Ok(())
}
