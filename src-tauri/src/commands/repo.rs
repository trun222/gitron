use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, State};

use gitron_core::git::{error::GitError, graph, remote, repository, types::*};
use gitron_core::watcher::manager::WatcherManager;

use crate::tauri_impls::TauriEventEmitter;

use super::AppState;

/// Open a repository at the given path.
/// Returns all data needed by the frontend in a single round-trip.
/// Starts the file watcher and initializes the cache.
#[tauri::command]
pub async fn open_repo(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<OpenRepoResult, GitError> {
    let repo = repository::open(&path)?;
    let info = repository::get_repo_info(&repo)?;
    let status = repository::get_status(&repo)?;
    let graph_data = graph::build_commit_graph(&repo, &GraphOptions::default())?;
    let remotes = remote::list_remotes(&repo)?;
    let tracking = info
        .head_branch
        .as_deref()
        .map(|branch| remote::get_tracking_status(&repo, branch))
        .transpose()?;

    // Initialize cache
    state.cache.initialize(info.clone(), status.clone(), graph_data.clone());

    // Stop any existing watcher
    if let Some(old) = state.watcher.lock().unwrap().take() {
        old.stop();
    }

    // Store repo path
    *state.repo_path.lock().unwrap() = Some(PathBuf::from(&path));

    // Start file watcher (graceful degradation — repo still usable if watcher fails)
    let poll_interval = *state.poll_interval_ms.lock().unwrap();
    let repo_path = PathBuf::from(&path);
    let emitter: Arc<dyn gitron_core::event::EventEmitter> = Arc::new(TauriEventEmitter::new(app));
    match WatcherManager::start(&repo_path, poll_interval, emitter, state.cache.clone()) {
        Ok(manager) => {
            *state.watcher.lock().unwrap() = Some(manager);
            log::info!("File watcher started for {path}");
        }
        Err(e) => {
            log::warn!("File watcher failed to start: {e}. Live updates disabled.");
        }
    }

    Ok(OpenRepoResult {
        info,
        status,
        graph: graph_data,
        remotes,
        tracking,
    })
}

/// Close the currently opened repository.
/// Stops the file watcher and clears the cache.
#[tauri::command]
pub fn close_repo(state: State<'_, AppState>) -> Result<(), GitError> {
    if let Some(watcher) = state.watcher.lock().unwrap().take() {
        watcher.stop();
    }
    // Invalidate the repo path cache for fast opens
    if let Some(path) = state.repo_path.lock().unwrap().as_ref() {
        repository::invalidate_cache(&path.to_string_lossy());
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
        let emitter: Arc<dyn gitron_core::event::EventEmitter> = Arc::new(TauriEventEmitter::new(app));
        match WatcherManager::start(&path, interval_ms, emitter, state.cache.clone()) {
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
