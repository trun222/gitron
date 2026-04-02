use std::io::Read;

use tauri::{AppHandle, Emitter};

use crate::commands::AppState;

/// Spawn a new PTY session and start streaming its output as Tauri events.
/// Returns the session ID.
#[tauri::command]
pub async fn terminal_spawn(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    shell: Option<String>,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let pty = state.pty_manager.clone();
    let id = pty.spawn(
        shell.as_deref(),
        cwd.as_deref(),
        cols,
        rows,
    )?;

    // Take the reader and spawn a background thread to stream PTY output
    // as Tauri events to the frontend.
    let reader = pty.take_reader(&id)?;
    let session_id = id.clone();

    std::thread::spawn(move || {
        stream_pty_output(app, reader, &session_id);
    });

    Ok(id)
}

/// Write raw bytes (user input) to a PTY session.
#[tauri::command]
pub fn terminal_write(
    state: tauri::State<'_, AppState>,
    id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.pty_manager.write(&id, &data)
}

/// Resize a PTY session.
#[tauri::command]
pub fn terminal_resize(
    state: tauri::State<'_, AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state.pty_manager.resize(&id, cols, rows)
}

/// Kill a PTY session.
#[tauri::command]
pub fn terminal_kill(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state.pty_manager.kill(&id)
}

/// Stream PTY output to the frontend as `terminal:data` events.
fn stream_pty_output(app: AppHandle, mut reader: Box<dyn Read + Send>, session_id: &str) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => {
                // EOF — the shell exited.
                app.emit(
                    &format!("terminal:exit:{session_id}"),
                    0u32,
                ).ok();
                break;
            }
            Ok(n) => {
                // Send raw bytes as a Vec<u8> payload.
                let data = buf[..n].to_vec();
                app.emit(
                    &format!("terminal:data:{session_id}"),
                    data,
                ).ok();
            }
            Err(e) => {
                log::error!("PTY read error for {session_id}: {e}");
                app.emit(
                    &format!("terminal:exit:{session_id}"),
                    1u32,
                ).ok();
                break;
            }
        }
    }
}
