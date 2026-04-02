//! PTY (pseudo-terminal) session management for the integrated terminal.
//!
//! Provides a `PtySession` that spawns a shell process in a pseudo-terminal,
//! allowing bidirectional I/O and terminal resize. Used by both the Tauri
//! desktop app and the Axum web server.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

/// Unique identifier for a PTY session.
pub type PtyId = String;

/// A running pseudo-terminal session.
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
}

impl PtySession {
    /// Spawn a new PTY session.
    ///
    /// * `shell` — shell binary (e.g. "/bin/zsh"). Empty or `None` uses the
    ///   platform default (`$SHELL` on Unix, `cmd.exe` on Windows).
    /// * `cwd` — working directory. `None` uses the process's cwd.
    /// * `cols` / `rows` — initial terminal size.
    pub fn spawn(
        shell: Option<&str>,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<Self, String> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {e}"))?;

        let shell_bin = resolve_shell(shell);
        let mut cmd = CommandBuilder::new(&shell_bin);

        // Pass a login shell flag so the shell sources the user's profile.
        #[cfg(unix)]
        {
            cmd.arg("-l");
        }

        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell `{shell_bin}`: {e}"))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {e}"))?;

        Ok(Self {
            master: pair.master,
            child,
            writer,
        })
    }

    /// Get a reader for the PTY output. The caller owns the reader and should
    /// read from it in a background thread/task.
    pub fn take_reader(&self) -> Result<Box<dyn Read + Send>, String> {
        self.master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {e}"))
    }

    /// Write raw bytes to the PTY (i.e. user keyboard input).
    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(data)
            .map_err(|e| format!("PTY write error: {e}"))?;
        self.writer
            .flush()
            .map_err(|e| format!("PTY flush error: {e}"))?;
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY resize error: {e}"))
    }

    /// Kill the shell process.
    pub fn kill(&mut self) -> Result<(), String> {
        self.child
            .kill()
            .map_err(|e| format!("Failed to kill PTY child: {e}"))
    }

    /// Check if the child process has exited.
    pub fn try_wait(&mut self) -> Result<Option<u32>, String> {
        match self.child.try_wait() {
            Ok(Some(status)) => Ok(Some(status.exit_code())),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("PTY wait error: {e}")),
        }
    }
}

/// Registry of active PTY sessions keyed by a unique ID.
#[derive(Default, Clone)]
pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<PtyId, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new session and return its ID.
    pub fn spawn(
        &self,
        shell: Option<&str>,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<PtyId, String> {
        let session = PtySession::spawn(shell, cwd, cols, rows)?;
        let id = uuid::Uuid::new_v4().to_string();
        self.sessions
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?
            .insert(id.clone(), session);
        Ok(id)
    }

    /// Take the reader for a session (can only be called once per session).
    pub fn take_reader(&self, id: &str) -> Result<Box<dyn Read + Send>, String> {
        self.sessions
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?
            .get(id)
            .ok_or_else(|| format!("PTY session not found: {id}"))?
            .take_reader()
    }

    /// Write data to a session.
    pub fn write(&self, id: &str, data: &[u8]) -> Result<(), String> {
        self.sessions
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?
            .get_mut(id)
            .ok_or_else(|| format!("PTY session not found: {id}"))?
            .write(data)
    }

    /// Resize a session.
    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        self.sessions
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?
            .get(id)
            .ok_or_else(|| format!("PTY session not found: {id}"))?
            .resize(cols, rows)
    }

    /// Kill and remove a session.
    pub fn kill(&self, id: &str) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        if let Some(mut session) = sessions.remove(id) {
            session.kill().ok(); // best-effort
        }
        Ok(())
    }

    /// Check if a session's child has exited, remove if so.
    pub fn try_wait(&self, id: &str) -> Result<Option<u32>, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("PTY session not found: {id}"))?;
        let result = session.try_wait()?;
        if result.is_some() {
            sessions.remove(id);
        }
        Ok(result)
    }

    /// Remove all sessions (kill all shells).
    pub fn kill_all(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for (_, mut session) in sessions.drain() {
                session.kill().ok();
            }
        }
    }
}

/// Resolve the shell binary to use.
fn resolve_shell(shell: Option<&str>) -> String {
    if let Some(s) = shell {
        if !s.is_empty() {
            return s.to_string();
        }
    }

    #[cfg(unix)]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.is_empty() {
                return shell;
            }
        }
        "/bin/sh".to_string()
    }

    #[cfg(windows)]
    {
        if let Ok(shell) = std::env::var("COMSPEC") {
            return shell;
        }
        "cmd.exe".to_string()
    }
}
