//! The WM's IPC answers, read as JSON with only the fields the bench
//! needs.
//!
//! `compare_branches.ps1` drives builds of other branches with this
//! build's `perf_bench`, and their DTOs differ from `wm-common`'s (e.g.
//! `origin/main`'s workspaces have no `layout`), which its typed messages
//! would fail to parse.

use anyhow::{bail, Context};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;
use wm_ipc_client::IpcClient;

/// A container of a workspace or window query.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
  Workspace(Workspace),
  Split(Branch),
  Stack(Branch),
  Window(Window),
  #[serde(other)]
  Other,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
  pub id: Uuid,
  pub name: String,
  pub has_focus: bool,
  pub is_displayed: bool,
  pub parent_id: Option<Uuid>,
  pub children: Vec<Node>,
}

/// A monitor, its rect in physical pixels.
#[derive(Deserialize)]
pub struct Monitor {
  pub x: i32,
  pub y: i32,
  pub width: i32,
  pub height: i32,
  pub children: Vec<Node>,
}

impl Monitor {
  /// Its workspaces.
  pub fn workspaces(&self) -> impl Iterator<Item = &Workspace> {
    self.children.iter().filter_map(|node| match node {
      Node::Workspace(workspace) => Some(workspace),
      _ => None,
    })
  }
}

/// A split or stack container.
#[derive(Deserialize)]
pub struct Branch {
  pub children: Vec<Node>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
  pub id: Uuid,
  pub handle: isize,
  pub process_name: String,
  state: WindowState,
}

impl Window {
  pub fn is_tiling(&self) -> bool {
    self.state.kind == "tiling"
  }
}

#[derive(Deserialize)]
struct WindowState {
  #[serde(rename = "type")]
  kind: String,
}

/// Sends `message` and returns its answer's `data`, failing unless the WM
/// reports success.
pub async fn request(
  client: &mut IpcClient,
  message: &str,
) -> anyhow::Result<Value> {
  client.send(message).await?;
  response(client, message).await
}

/// Waits for the answer to `message`, sent earlier, and returns its
/// `data`, failing unless the WM reports success.
pub async fn response(
  client: &mut IpcClient,
  message: &str,
) -> anyhow::Result<Value> {
  loop {
    let mut reply: Value =
      serde_json::from_str(&client.next_text().await?)
        .context("Invalid IPC message.")?;

    let is_answer = reply["messageType"] == "client_response"
      && reply["clientMessage"] == message;
    if !is_answer {
      continue;
    }

    if reply["success"] != true {
      bail!(
        "'{message}' failed: {}",
        reply["error"].as_str().unwrap_or_default()
      );
    }

    return Ok(reply["data"].take());
  }
}

/// Sends `message` and fails unless the WM reports success.
pub async fn send_command(
  client: &mut IpcClient,
  message: &str,
) -> anyhow::Result<()> {
  request(client, message).await.map(|_| ())
}

/// Every workspace the WM has active.
pub async fn query_workspaces(
  client: &mut IpcClient,
) -> anyhow::Result<Vec<Workspace>> {
  Ok(
    containers(client, "query workspaces", "workspaces")
      .await?
      .into_iter()
      .filter_map(|node| match node {
        Node::Workspace(workspace) => Some(workspace),
        _ => None,
      })
      .collect(),
  )
}

/// Every monitor, with its workspaces.
pub async fn query_monitors(
  client: &mut IpcClient,
) -> anyhow::Result<Vec<Monitor>> {
  let mut data = request(client, "query monitors").await?;
  serde_json::from_value(data["monitors"].take())
    .context("Unexpected answer to 'query monitors'.")
}

/// Every window the WM manages.
pub async fn query_windows(
  client: &mut IpcClient,
) -> anyhow::Result<Vec<Window>> {
  Ok(
    containers(client, "query windows", "windows")
      .await?
      .into_iter()
      .filter_map(|node| match node {
        Node::Window(window) => Some(window),
        _ => None,
      })
      .collect(),
  )
}

/// The containers listed under `field` in the answer to `message`.
async fn containers(
  client: &mut IpcClient,
  message: &str,
  field: &str,
) -> anyhow::Result<Vec<Node>> {
  let mut data = request(client, message).await?;
  serde_json::from_value(data[field].take())
    .with_context(|| format!("Unexpected answer to '{message}'."))
}

/// The windows in `children`, through splits and stacks.
pub fn windows_in(children: &[Node]) -> Vec<&Window> {
  children
    .iter()
    .flat_map(|child| match child {
      Node::Window(window) => vec![window],
      Node::Split(branch) | Node::Stack(branch) => {
        windows_in(&branch.children)
      }
      _ => Vec::new(),
    })
    .collect()
}

/// The tiled windows of `workspace`.
pub fn tiled_windows(workspace: &Workspace) -> Vec<&Window> {
  windows_in(&workspace.children)
    .into_iter()
    .filter(|window| window.is_tiling())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A workspace with fields this bench doesn't know, as another branch's
  /// WM sends it, and without ones it doesn't read, still parses.
  #[test]
  fn reads_workspaces_of_another_version() {
    let json = r#"[{
      "type": "workspace", "id": "6f9619ff-8b86-d011-b42d-00c04fc964ff",
      "name": "1", "hasFocus": true, "isDisplayed": true,
      "parentId": null, "futureField": 3,
      "children": [
        {"type": "split", "children": [
          {"type": "window", "id": "6f9619ff-8b86-d011-b42d-00c04fc964fe",
           "handle": 42, "processName": "notepad",
           "state": {"type": "tiling"}}
        ]},
        {"type": "window", "id": "6f9619ff-8b86-d011-b42d-00c04fc964fd",
         "handle": 43, "processName": "chrome",
         "state": {"type": "floating", "centered": true}},
        {"type": "some_new_container", "children": []}
      ]
    }]"#;

    let nodes: Vec<Node> = serde_json::from_str(json).unwrap();
    let Some(Node::Workspace(workspace)) = nodes.first() else {
      panic!("not a workspace");
    };

    assert_eq!(windows_in(&workspace.children).len(), 2);
    let tiled = tiled_windows(workspace);
    assert_eq!(tiled.len(), 1);
    assert_eq!(tiled[0].handle, 42);
    assert_eq!(tiled[0].process_name, "notepad");
  }

  #[test]
  fn reads_monitors() {
    let json = r#"[{
      "type": "monitor", "id": "6f9619ff-8b86-d011-b42d-00c04fc964fc",
      "x": -2560, "y": 0, "width": 2560, "height": 1440, "dpi": 96,
      "children": [{
        "type": "workspace", "id": "6f9619ff-8b86-d011-b42d-00c04fc964ff",
        "name": "1", "hasFocus": false, "isDisplayed": true,
        "parentId": "6f9619ff-8b86-d011-b42d-00c04fc964fc", "children": []
      }]
    }]"#;

    let monitors: Vec<Monitor> = serde_json::from_str(json).unwrap();

    assert_eq!(monitors[0].x, -2560);
    assert_eq!(monitors[0].width, 2560);
    assert_eq!(monitors[0].workspaces().next().unwrap().name, "1");
  }
}
