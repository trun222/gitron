use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::git::{error::GitError, graph, repository, types::*};
use crate::watcher::manager::WatcherManager;

use super::AppState;

/// Open a repository at the given path.
/// Starts the file watcher and initializes the cache.
#[tauri::command]
pub async fn open_repo(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    let info = repository::get_repo_info(&repo)?;

    // Initialize cache with current state
    let status = repository::get_status(&repo)?;
    let graph_data = graph::build_commit_graph(&repo, &GraphOptions::default())?;
    state.cache.initialize(info.clone(), status, graph_data);

    // Stop any existing watcher
    if let Some(old) = state.watcher.lock().unwrap().take() {
        old.stop();
    }

    // Store repo path
    *state.repo_path.lock().unwrap() = Some(PathBuf::from(&path));

    // Start file watcher (graceful degradation — repo still usable if watcher fails)
    let poll_interval = *state.poll_interval_ms.lock().unwrap();
    let repo_path = PathBuf::from(&path);
    match WatcherManager::start(&repo_path, poll_interval, app, state.cache.clone()) {
        Ok(manager) => {
            *state.watcher.lock().unwrap() = Some(manager);
            log::info!("File watcher started for {path}");
        }
        Err(e) => {
            log::warn!("File watcher failed to start: {e}. Live updates disabled.");
        }
    }

    Ok(info)
}

/// Close the currently opened repository.
/// Stops the file watcher and clears the cache.
#[tauri::command]
pub fn close_repo(state: State<'_, AppState>) -> Result<(), GitError> {
    if let Some(watcher) = state.watcher.lock().unwrap().take() {
        watcher.stop();
    }
    state.cache.clear();
    *state.repo_path.lock().unwrap() = None;
    Ok(())
}

/// Update the watcher poll interval. Restarts the watcher if a repo is open.
#[tauri::command]
pub async fn set_watcher_interval(
    interval_ms: u64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), GitError> {
    *state.poll_interval_ms.lock().unwrap() = interval_ms;

    // Restart watcher if a repo is currently open
    let repo_path = state.repo_path.lock().unwrap().clone();
    if let Some(path) = repo_path {
        // Stop existing watcher
        if let Some(old) = state.watcher.lock().unwrap().take() {
            old.stop();
        }

        // Start new watcher with updated interval
        match WatcherManager::start(&path, interval_ms, app, state.cache.clone()) {
            Ok(manager) => {
                *state.watcher.lock().unwrap() = Some(manager);
                log::info!(
                    "File watcher restarted with interval {interval_ms}ms for {}",
                    path.display()
                );
            }
            Err(e) => {
                log::warn!("File watcher restart failed: {e}");
            }
        }
    }

    Ok(())
}

/// Get the current repository status
#[tauri::command]
pub fn get_status(path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::get_status(&repo)
}

/// Get basic repository information
#[tauri::command]
pub fn get_repo_info(path: String) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    repository::get_repo_info(&repo)
}
