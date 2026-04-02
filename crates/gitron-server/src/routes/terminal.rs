use std::io::Read;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::ServerState;

/// Query params for the WebSocket upgrade request.
#[derive(Deserialize)]
pub struct TerminalParams {
    pub shell: Option<String>,
    pub cwd: Option<String>,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
}

fn default_cols() -> u16 { 80 }
fn default_rows() -> u16 { 24 }

/// Upgrade an HTTP request to a WebSocket that drives a PTY session.
///
/// Protocol (binary frames):
///  - Client → Server: raw bytes are written to the PTY stdin.
///  - Server → Client: raw bytes are PTY stdout.
///  - Client → Server (text): JSON control messages, e.g. `{"resize":[cols,rows]}`
///  - Server → Client (text): `{"exit": <code>}` when the shell exits.
pub async fn terminal_ws(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<TerminalParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_terminal_socket(state, params, socket))
}

async fn handle_terminal_socket(
    state: Arc<ServerState>,
    params: TerminalParams,
    socket: WebSocket,
) {
    let pty = &state.pty_manager;

    // Spawn the PTY session.
    let id = match pty.spawn(
        params.shell.as_deref(),
        params.cwd.as_deref(),
        params.cols,
        params.rows,
    ) {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to spawn PTY: {e}");
            return;
        }
    };

    // Take the reader for background streaming.
    let reader = match pty.take_reader(&id) {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to take PTY reader: {e}");
            pty.kill(&id).ok();
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    use futures_util::{SinkExt, StreamExt};

    // Channel: PTY reader thread → async sender task
    let (tx, mut rx) = mpsc::channel::<Message>(256);

    // Background thread: read from PTY, send to channel.
    let session_id = id.clone();
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        stream_pty_to_channel(reader, tx_clone, &session_id);
    });

    // Async task: forward channel messages to WebSocket.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Read loop: receive from WebSocket, write to PTY.
    let pty_manager = state.pty_manager.clone();
    let session_id = id.clone();
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Binary(data) => {
                if pty_manager.write(&session_id, &data).is_err() {
                    break;
                }
            }
            Message::Text(ref text) => {
                // Parse control messages.
                if let Ok(ctrl) = serde_json::from_str::<ControlMessage>(text) {
                    match ctrl {
                        ControlMessage::Resize { cols, rows } => {
                            pty_manager.resize(&session_id, cols, rows).ok();
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Clean up.
    pty_manager.kill(&id).ok();
    send_task.abort();
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ControlMessage {
    #[serde(rename = "resize")]
    Resize { cols: u16, rows: u16 },
}

fn stream_pty_to_channel(
    mut reader: Box<dyn Read + Send>,
    tx: mpsc::Sender<Message>,
    session_id: &str,
) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => {
                // EOF — shell exited.
                let exit_msg = serde_json::json!({"exit": 0}).to_string();
                tx.blocking_send(Message::Text(exit_msg.into())).ok();
                break;
            }
            Ok(n) => {
                let data = buf[..n].to_vec();
                if tx.blocking_send(Message::Binary(data.into())).is_err() {
                    break; // WebSocket closed
                }
            }
            Err(e) => {
                log::error!("PTY read error for {session_id}: {e}");
                let exit_msg = serde_json::json!({"exit": 1}).to_string();
                tx.blocking_send(Message::Text(exit_msg.into())).ok();
                break;
            }
        }
    }
}
