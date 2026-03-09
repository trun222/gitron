use std::process::Command;
use gitron_core::git::{error::GitError, worktree, types::*};

/// List all worktrees for the repository
#[tauri::command]
pub fn list_worktrees(path: String) -> Result<Vec<WorktreeInfo>, GitError> {
    worktree::list_worktrees(&path)
}

/// Create a new worktree
#[tauri::command]
pub fn add_worktree(
    path: String,
    worktree_path: String,
    branch: Option<String>,
    new_branch: bool,
) -> Result<WorktreeCreateResult, GitError> {
    worktree::add_worktree(&path, &worktree_path, branch.as_deref(), new_branch)
}

/// Remove a worktree
#[tauri::command]
pub fn remove_worktree(
    path: String,
    worktree_path: String,
    force: bool,
) -> Result<WorktreeRemoveResult, GitError> {
    worktree::remove_worktree(&path, &worktree_path, force)
}

/// Lock a worktree (returns updated list)
#[tauri::command]
pub fn lock_worktree(
    path: String,
    worktree_path: String,
    reason: Option<String>,
) -> Result<Vec<WorktreeInfo>, GitError> {
    worktree::lock_worktree(&path, &worktree_path, reason.as_deref())?;
    worktree::list_worktrees(&path)
}

/// Unlock a worktree (returns updated list)
#[tauri::command]
pub fn unlock_worktree(
    path: String,
    worktree_path: String,
) -> Result<Vec<WorktreeInfo>, GitError> {
    worktree::unlock_worktree(&path, &worktree_path)?;
    worktree::list_worktrees(&path)
}

/// Prune stale worktree references
#[tauri::command]
pub fn prune_worktrees(path: String, dry_run: bool) -> Result<WorktreePruneResult, GitError> {
    worktree::prune_worktrees(&path, dry_run)
}

/// Open a directory in the system terminal.
/// `terminal_app` is the user's configured terminal preference (e.g. "iTerm", "Warp", "Alacritty").
/// Empty or absent means use the OS default.
#[tauri::command]
pub fn open_in_terminal(path: String, terminal_app: Option<String>) -> Result<(), String> {
    let app = terminal_app
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    open_terminal_at(&path, app)
}

fn open_terminal_at(dir: &str, app: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return open_terminal_macos(dir, app);

    #[cfg(target_os = "linux")]
    return open_terminal_linux(dir, app);

    #[cfg(target_os = "windows")]
    return open_terminal_windows(dir, app);

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    Err("Unsupported platform".into())
}

#[cfg(target_os = "macos")]
fn open_terminal_macos(dir: &str, app: &str) -> Result<(), String> {
    match app.to_lowercase().as_str() {
        // Default: macOS Terminal.app — understands directory arguments
        "" | "terminal" | "terminal.app" => {
            Command::new("open")
                .args(["-a", "Terminal", dir])
                .spawn()
                .map_err(|e| format!("Failed to open Terminal: {}", e))?;
        }
        // Alacritty — CLI binary with --working-directory flag
        "alacritty" => {
            Command::new("alacritty")
                .args(["--working-directory", dir])
                .spawn()
                .map_err(|e| format!("Failed to open Alacritty: {}", e))?;
        }
        // Kitty — CLI binary with --directory flag
        "kitty" => {
            Command::new("kitty")
                .args(["--directory", dir])
                .spawn()
                .map_err(|e| format!("Failed to open Kitty: {}", e))?;
        }
        // iTerm2, Warp, Ghostty, Hyper, and other .app terminals:
        // Use `open -a <name>` which opens the app. Most modern macOS terminals
        // accept a directory argument and will open a new window/tab at that path.
        _ => {
            let output = Command::new("open")
                .args(["-a", app, dir])
                .output()
                .map_err(|e| format!("Failed to launch '{}': {}", app, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "Could not open '{}': {}. Check the app name in Settings > Git > Terminal.",
                    app,
                    stderr.trim()
                ));
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_terminal_linux(dir: &str, app: &str) -> Result<(), String> {
    if !app.is_empty() {
        // Known terminals with --working-directory or --directory flags
        match app.to_lowercase().as_str() {
            "alacritty" => {
                Command::new("alacritty")
                    .args(["--working-directory", dir])
                    .spawn()
                    .map_err(|e| format!("Failed to open Alacritty: {}", e))?;
            }
            "kitty" => {
                Command::new("kitty")
                    .args(["--directory", dir])
                    .spawn()
                    .map_err(|e| format!("Failed to open Kitty: {}", e))?;
            }
            _ => {
                // Try with --working-directory (works for gnome-terminal, konsole, etc.)
                Command::new(app)
                    .args(["--working-directory", dir])
                    .spawn()
                    .map_err(|e| format!("Failed to open '{}': {}", app, e))?;
            }
        }
    } else {
        let terminals = [
            ("x-terminal-emulator", vec!["--working-directory", dir]),
            ("gnome-terminal", vec!["--working-directory", dir]),
            ("konsole", vec!["--workdir", dir]),
            ("xfce4-terminal", vec!["--working-directory", dir]),
            ("xterm", vec!["-e", &format!("cd '{}' && exec $SHELL", dir)]),
        ];
        let mut launched = false;
        for (term, args) in &terminals {
            if Command::new(term).args(args).spawn().is_ok() {
                launched = true;
                break;
            }
        }
        if !launched {
            return Err(
                "No terminal emulator found. Set one in Settings > Git > Terminal.".into(),
            );
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_terminal_windows(dir: &str, app: &str) -> Result<(), String> {
    if !app.is_empty() {
        Command::new(app)
            .current_dir(dir)
            .spawn()
            .map_err(|e| format!("Failed to open '{}': {}", app, e))?;
    } else {
        // Default: Windows Terminal if available, else cmd
        if Command::new("wt")
            .args(["-d", dir])
            .spawn()
            .is_err()
        {
            Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", &format!("cd /d \"{}\"", dir)])
                .spawn()
                .map_err(|e| format!("Failed to open terminal: {}", e))?;
        }
    }
    Ok(())
}
