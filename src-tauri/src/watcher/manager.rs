use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::cache::repo_state::RepoStateCache;
use crate::git::{graph, repository, types::*};

use super::handler::{self, RepoChangeEvent, WatcherHandle};

/// Manages the file watcher lifecycle: start, stop, restart.
pub struct WatcherManager {
    _handle: WatcherHandle,
    task: JoinHandle<()>,
    pub repo_path: PathBuf,
}

impl WatcherManager {
    /// Start watching a repo. Spawns a background task that processes change events.
    pub fn start(
        repo_path: &Path,
        poll_interval_ms: u64,
        app_handle: AppHandle,
        cache: RepoStateCache,
    ) -> anyhow::Result<Self> {
        let (watcher_handle, rx) = handler::watch_repo(repo_path, poll_interval_ms)?;
        let path = repo_path.to_path_buf();
        let path_for_task = path.clone();

        let task = tokio::spawn(async move {
            event_consumer(rx, path_for_task, app_handle, cache).await;
        });

        Ok(Self {
            _handle: watcher_handle,
            task,
            repo_path: path,
        })
    }

    /// Stop the watcher and abort the background task.
    pub fn stop(self) {
        self.task.abort();
        // WatcherHandle is dropped here, which stops the watcher
    }
}

/// Background task that consumes repo change events and emits Tauri events.
async fn event_consumer(
    mut rx: mpsc::Receiver<RepoChangeEvent>,
    repo_path: PathBuf,
    app_handle: AppHandle,
    cache: RepoStateCache,
) {
    let path_str = repo_path.to_string_lossy().to_string();

    while let Some(event) = rx.recv().await {
        match event {
            RepoChangeEvent::WorkdirChanged => {
                if let Some(status) = refresh_status(&path_str) {
                    cache.update_status(status.clone());
                    let payload = StatusChangedPayload { status };
                    app_handle.emit("repo:status-changed", &payload).ok();
                }
            }
            RepoChangeEvent::HeadChanged | RepoChangeEvent::RefsChanged => {
                if let Some((status, graph)) = refresh_status_and_graph(&path_str) {
                    cache.update_status(status.clone());
                    cache.update_graph(graph.clone());
                    let status_payload = StatusChangedPayload {
                        status: status.clone(),
                    };
                    let refs_payload = RefsChangedPayload { graph, status };
                    app_handle.emit("repo:status-changed", &status_payload).ok();
                    app_handle.emit("repo:refs-changed", &refs_payload).ok();
                }
            }
        }

        // 50ms cooldown between processing batches
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

fn refresh_status(path: &str) -> Option<RepoStatus> {
    let repo = repository::open(path).ok()?;
    repository::get_status(&repo).ok()
}

fn refresh_status_and_graph(path: &str) -> Option<(RepoStatus, CommitGraph)> {
    let repo = repository::open(path).ok()?;
    let status = repository::get_status(&repo).ok()?;
    let graph = graph::build_commit_graph(&repo, &GraphOptions::default()).ok()?;
    Some((status, graph))
}
