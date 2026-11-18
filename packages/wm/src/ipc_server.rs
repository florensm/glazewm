use std::{iter, net::SocketAddr};

use anyhow::{bail, Context};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::{
  net::{TcpListener, TcpStream},
  sync::{broadcast, mpsc},
  task,
};
use tokio_tungstenite::{
  accept_hdr_async,
  tungstenite::{
    handshake::server::{ErrorResponse, Request, Response},
    http::{header::ORIGIN, StatusCode},
    Message,
  },
};
use tracing::{info, warn};
use uuid::Uuid;
use wm_common::{
  AppCommand, AppMetadataData, BindingModesData, ClientResponseData,
  ClientResponseMessage, CommandData, EventSubscribeData,
  EventSubscriptionMessage, FocusedData, MonitorsData, OverlayCommand,
  OverlayData, QueryCommand, ServerMessage, SubscribableEvent,
  TilingDirectionData, WindowsData, WmEvent, WorkspacesData,
  DEFAULT_IPC_PORT,
};

use crate::{
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm::WindowManager,
};

/// IPC message, with the channels to answer its client on.
pub type IpcMessage = (
  String,
  mpsc::UnboundedSender<Message>,
  broadcast::Sender<()>,
);

pub struct IpcServer {
  abort_handle: task::AbortHandle,
  pub message_rx: mpsc::UnboundedReceiver<IpcMessage>,
  _event_rx: broadcast::Receiver<(SubscribableEvent, WmEvent)>,
  event_tx: broadcast::Sender<(SubscribableEvent, WmEvent)>,
  _unsubscribe_rx: broadcast::Receiver<Uuid>,
  unsubscribe_tx: broadcast::Sender<Uuid>,
  /// Overlay holds by connection, keyed by its response channel.
  overlay_holds: Vec<(mpsc::UnboundedSender<Message>, Uuid)>,
  overlay_release_tx: mpsc::UnboundedSender<Uuid>,
  /// Holds whose connection went without ending them.
  pub overlay_release_rx: mpsc::UnboundedReceiver<Uuid>,
}

impl IpcServer {
  pub async fn start() -> anyhow::Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel();
    let (event_tx, _event_rx) = broadcast::channel(16);
    let (unsubscribe_tx, _unsubscribe_rx) = broadcast::channel(16);
    let (overlay_release_tx, overlay_release_rx) =
      mpsc::unbounded_channel();

    let server_addr = format!("127.0.0.1:{DEFAULT_IPC_PORT}");
    let server = TcpListener::bind(server_addr.clone()).await?;
    info!("IPC server started on: '{}'.", server_addr);

    let task = task::spawn(async move {
      while let Ok((stream, addr)) = server.accept().await {
        let message_tx = message_tx.clone();

        task::spawn(async move {
          if let Err(err) =
            Self::handle_connection(stream, addr, message_tx).await
          {
            warn!("Error handling connection: {}", err);
          }
        });
      }
    });

    Ok(Self {
      abort_handle: task.abort_handle(),
      #[allow(clippy::used_underscore_binding)]
      _event_rx,
      event_tx,
      message_rx,
      unsubscribe_tx,
      #[allow(clippy::used_underscore_binding)]
      _unsubscribe_rx,
      overlay_holds: Vec::new(),
      overlay_release_tx,
      overlay_release_rx,
    })
  }

  async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    message_tx: mpsc::UnboundedSender<IpcMessage>,
  ) -> anyhow::Result<()> {
    info!("Incoming IPC connection from: {}.", addr);

    let ws_stream = accept_hdr_async(stream, reject_browsers)
      .await
      .context("Error during websocket handshake.")?;

    let (mut outgoing, mut incoming) = ws_stream.split();
    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    let (disconnection_tx, _) = broadcast::channel(16);

    let res = async {
      loop {
        tokio::select! {
          Some(response) = response_rx.recv() => {
            outgoing.send(response).await?;
          }
          message = incoming.next() => {
            match message {
              Some(Ok(message)) => {
                if message.is_text() || message.is_binary() {
                  message_tx.send((
                    message.to_text()?.to_string(),
                    response_tx.clone(),
                    disconnection_tx.clone(),
                  ))?;
                }
              }
              Some(Err(err)) => bail!("WebSocket error: {}", err),
              None => {
                // WebSocket connection closed.
                break Ok(());
              },
            }
          }
        }
      }
    }
    .await;

    info!("IPC disconnection from: {}.", addr);

    // Fails only when no event subscription of this connection is
    // listening, which leaves nothing to notify.
    let _ = disconnection_tx.send(());

    res
  }

  pub fn process_message(
    &mut self,
    message: String,
    response_tx: &mpsc::UnboundedSender<Message>,
    disconnection_tx: &broadcast::Sender<()>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let app_command = AppCommand::try_parse_from(
      iter::once("").chain(message.split_whitespace()),
    );

    let response_data = app_command
      .map_err(anyhow::Error::msg)
      .and_then(|app_command| {
        self.handle_app_command(
          app_command,
          response_tx,
          disconnection_tx,
          wm,
          config,
        )
      })
      .map(|mut data| {
        // How the WM shows each window, which its DTO can't tell.
        data.for_each_window_mut(&mut crate::window_styles::style_filler(
          &wm.state, config,
        ));
        data
      });

    // Respond to the client with the result of the command.
    response_tx
      .send(Self::to_client_response_msg(message, response_data)?)
      .map_err(|err| {
        anyhow::anyhow!("Failed to send response: {}", err)
      })?;

    Ok(())
  }

  #[allow(clippy::too_many_lines)]
  fn handle_app_command(
    &mut self,
    app_command: AppCommand,
    response_tx: &mpsc::UnboundedSender<Message>,
    disconnection_tx: &broadcast::Sender<()>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<ClientResponseData> {
    let response_data = match app_command {
      AppCommand::Query { command } => match command {
        QueryCommand::Windows => {
          ClientResponseData::Windows(WindowsData {
            windows: wm
              .state
              .windows()
              .into_iter()
              .map(|window| window.to_dto())
              .try_collect()?,
          })
        }
        QueryCommand::Workspaces => {
          ClientResponseData::Workspaces(WorkspacesData {
            workspaces: wm
              .state
              .workspaces()
              .into_iter()
              .map(|workspace| workspace.to_dto())
              .try_collect()?,
          })
        }
        QueryCommand::Monitors => {
          ClientResponseData::Monitors(MonitorsData {
            monitors: wm
              .state
              .monitors()
              .into_iter()
              .map(|monitor| monitor.to_dto())
              .try_collect()?,
          })
        }
        QueryCommand::BindingModes => {
          ClientResponseData::BindingModes(BindingModesData {
            binding_modes: wm.state.binding_modes.clone(),
          })
        }
        QueryCommand::Focused => {
          let focused_container = wm
            .state
            .focused_container()
            .context("No focused container.")?;

          ClientResponseData::Focused(FocusedData {
            focused: focused_container.to_dto()?,
          })
        }
        QueryCommand::AppMetadata => {
          ClientResponseData::AppMetadata(AppMetadataData {
            version: env!("VERSION_NUMBER").to_string(),
          })
        }
        QueryCommand::TilingDirection => {
          let direction_container = wm
            .state
            .focused_container()
            .and_then(|focused| focused.direction_container())
            .context("No direction container.")?;

          ClientResponseData::TilingDirection(TilingDirectionData {
            direction_container: direction_container.to_dto()?,
            tiling_direction: direction_container.tiling_direction(),
          })
        }
        QueryCommand::Paused => {
          ClientResponseData::Paused(wm.state.is_paused)
        }
      },
      AppCommand::Command {
        subject_container_id,
        command,
      } => {
        let subject_container_id = wm.process_commands(
          &vec![command],
          subject_container_id,
          config,
        )?;

        ClientResponseData::Command(CommandData {
          subject_container_id,
        })
      }
      AppCommand::Sub { events } => {
        let subscription_id = Uuid::new_v4();
        info!("New event subscription {}: {:?}", subscription_id, events);

        let response_tx = response_tx.clone();
        let mut event_rx = self.event_tx.subscribe();
        let mut unsubscribe_rx = self.unsubscribe_tx.subscribe();
        let mut disconnection_rx = disconnection_tx.subscribe();

        task::spawn(async move {
          loop {
            tokio::select! {
              Ok(()) = disconnection_rx.recv() => {
                break;
              }
              Ok(id) = unsubscribe_rx.recv() => {
                if id == subscription_id {
                  break;
                }
              }
              Ok((event_type, event)) = event_rx.recv() => {
                // Check whether the event is one of the subscribed events.
                if events.contains(&event_type)
                  || events.contains(&SubscribableEvent::All)
                {
                  let send_result = Self::to_event_subscription_msg(
                    subscription_id,
                    event,
                  )
                  .and_then(|event_msg| {
                    response_tx
                      .send(event_msg)
                      .map_err(anyhow::Error::from)
                  });

                  if let Err(err) = send_result {
                    warn!("Error emitting WM event: {}", err);
                    break;
                  }
                }
              }
            }
          }
        });

        ClientResponseData::EventSubscribe(EventSubscribeData {
          subscription_id,
        })
      }
      AppCommand::Unsub { subscription_id } => {
        self
          .unsubscribe_tx
          .send(subscription_id)
          .context("Failed to unsubscribe from event.")?;

        ClientResponseData::EventUnsubscribe
      }
      AppCommand::Overlay { command } => {
        match command {
          OverlayCommand::Begin => {
            self.begin_overlay(response_tx, wm, config)?;
          }
          OverlayCommand::End => {
            let held = self
              .overlay_holds
              .iter()
              .position(|(tx, _)| tx.same_channel(response_tx));

            if let Some(index) = held {
              let (_, id) = self.overlay_holds.remove(index);
              wm.release_overlay(id, config)?;
            }
          }
        }

        ClientResponseData::Overlay(OverlayData {
          is_held: wm.state.overlay_holds.is_held(),
        })
      }
      AppCommand::Start { .. } => bail!("Unsupported IPC command."),
    };

    Ok(response_data)
  }

  /// Starts an overlay hold for the connection of `response_tx`, unless
  /// it holds one already. The hold ends with the connection.
  fn begin_overlay(
    &mut self,
    response_tx: &mpsc::UnboundedSender<Message>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    if self
      .overlay_holds
      .iter()
      .any(|(tx, _)| tx.same_channel(response_tx))
    {
      return Ok(());
    }

    let id = Uuid::new_v4();
    self.overlay_holds.push((response_tx.clone(), id));

    // The connection's task owns the receiving end, so this resolves once
    // it is gone, even if it went before this message was processed.
    let response_tx = response_tx.clone();
    let release_tx = self.overlay_release_tx.clone();
    task::spawn(async move {
      response_tx.closed().await;
      let _ = release_tx.send(id);
    });

    wm.hold_overlay(id, config)
  }

  /// Whether any connection is subscribed to events.
  pub fn has_subscribers(&self) -> bool {
    // Less the receiver the server keeps.
    self.event_tx.receiver_count() > 1
  }

  /// Forgets overlay hold `id`, whose connection went.
  pub fn forget_overlay_hold(&mut self, id: Uuid) {
    self.overlay_holds.retain(|(_, held)| *held != id);
  }

  fn to_client_response_msg(
    client_message: String,
    response_data: anyhow::Result<ClientResponseData>,
  ) -> anyhow::Result<Message> {
    let error = response_data.as_ref().err().map(ToString::to_string);
    let success = response_data.as_ref().is_ok();

    let message = ServerMessage::ClientResponse(ClientResponseMessage {
      client_message,
      data: response_data.ok(),
      error,
      success,
    });

    let message_json = serde_json::to_string(&message)?;
    Ok(Message::Text(message_json.into()))
  }

  fn to_event_subscription_msg(
    subscription_id: Uuid,
    event: WmEvent,
  ) -> anyhow::Result<Message> {
    let message =
      ServerMessage::EventSubscription(EventSubscriptionMessage {
        data: Some(event),
        error: None,
        subscription_id,
        success: true,
      });

    let message_json = serde_json::to_string(&message)?;
    Ok(Message::Text(message_json.into()))
  }

  pub fn process_event(&mut self, event: WmEvent) -> anyhow::Result<()> {
    let event_type = match event {
      WmEvent::ApplicationExiting => SubscribableEvent::ApplicationExiting,
      WmEvent::BindingModesChanged { .. } => {
        SubscribableEvent::BindingModesChanged
      }
      WmEvent::FocusChanged { .. } => SubscribableEvent::FocusChanged,
      WmEvent::FocusedContainerMoved { .. } => {
        SubscribableEvent::FocusedContainerMoved
      }
      WmEvent::MonitorAdded { .. } => SubscribableEvent::MonitorAdded,
      WmEvent::MonitorUpdated { .. } => SubscribableEvent::MonitorUpdated,
      WmEvent::MonitorRemoved { .. } => SubscribableEvent::MonitorRemoved,
      WmEvent::TilingDirectionChanged { .. } => {
        SubscribableEvent::TilingDirectionChanged
      }
      WmEvent::LayoutChanged { .. } => SubscribableEvent::LayoutChanged,
      WmEvent::UserConfigChanged { .. } => {
        SubscribableEvent::UserConfigChanged
      }
      WmEvent::WindowManaged { .. } => SubscribableEvent::WindowManaged,
      WmEvent::WindowUnmanaged { .. } => {
        SubscribableEvent::WindowUnmanaged
      }
      WmEvent::WindowUrgencyChanged { .. } => {
        SubscribableEvent::WindowUrgencyChanged
      }
      WmEvent::WorkspaceActivated { .. } => {
        SubscribableEvent::WorkspaceActivated
      }
      WmEvent::WorkspaceDeactivated { .. } => {
        SubscribableEvent::WorkspaceDeactivated
      }
      WmEvent::WorkspaceUpdated { .. } => {
        SubscribableEvent::WorkspaceUpdated
      }
      WmEvent::PauseChanged { .. } => SubscribableEvent::PauseChanged,
      WmEvent::OverlayInput { .. } => SubscribableEvent::OverlayInput,
    };

    self
      .event_tx
      .send((event_type, event))
      .map_err(|err| anyhow::anyhow!("Failed to send event: {}", err))?;

    Ok(())
  }

  pub fn stop(&self) {
    info!("Shutting down IPC server.");
    self.abort_handle.abort();
  }
}

impl Drop for IpcServer {
  fn drop(&mut self) {
    self.stop();
  }
}

/// Refuses WebSocket handshakes from web pages.
///
/// Any page a browser opens may connect to a localhost socket, and the IPC
/// runs commands such as `shell-exec`. Browsers always send `Origin` on a
/// WebSocket handshake and pages can't remove it, while native clients
/// (the CLI, `wm-ipc-client`, Qt's `QWebSocket` with no origin set) send
/// none.
#[allow(clippy::result_large_err)]
fn reject_browsers(
  request: &Request,
  response: Response,
) -> Result<Response, ErrorResponse> {
  if is_from_browser(request) {
    warn!("Refused IPC connection from a web page.");
    let mut refusal = ErrorResponse::new(Some(
      "Connections from web pages are refused.".into(),
    ));
    *refusal.status_mut() = StatusCode::FORBIDDEN;
    return Err(refusal);
  }

  Ok(response)
}

fn is_from_browser(request: &Request) -> bool {
  request.headers().contains_key(ORIGIN)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn request(origin: Option<&str>) -> Request {
    let mut builder = Request::builder().uri("ws://127.0.0.1:6123");
    if let Some(origin) = origin {
      builder = builder.header(ORIGIN, origin);
    }
    builder.body(()).expect("Valid request.")
  }

  #[test]
  fn refuses_any_origin() {
    assert!(is_from_browser(&request(Some("https://example.com"))));
    assert!(is_from_browser(&request(Some("null"))));
    assert!(is_from_browser(&request(Some("http://127.0.0.1:6123"))));
  }

  #[test]
  fn accepts_native_clients() {
    assert!(!is_from_browser(&request(None)));
  }
}
