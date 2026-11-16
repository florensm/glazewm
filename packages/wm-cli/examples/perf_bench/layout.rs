//! Which workspace each window is on, read from the WM and restored to it.
//!
//! The bench scripts restart the WM, and a restart manages every
//! window onto its monitor's displayed workspace. Windows are matched by
//! handle, the one ID a restart keeps.

use std::{fmt::Write as _, path::Path};

use anyhow::Context;
use uuid::Uuid;
use wm_ipc_client::IpcClient;

use crate::ipc::{query_workspaces, send_command, windows_in};

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
