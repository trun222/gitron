use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::cache::repo_state::RepoStateCache;
use crate::event::EventEmitter;
use crate::git::{graph, repository, types::*};

use super::handler::{self, RepoChangeEvent, WatcherHandle};

/// Global flag to suppress watcher processing during git CLI operations.
/// Set this before running destructive git CLI commands (rebase, merge, etc.)
/// to prevent the watcher from racing with ref updates.
static WATCHER_PAUSED: AtomicBool = AtomicBool::new(false);

/// Pause watcher event processing. Call before git CLI operations.
pub fn pause_watcher() {
    WATCHER_PAUSED.store(true, Ordering::SeqCst);
}

/// Resume watcher event processing. Call after git CLI operations complete.
pub fn resume_watcher() {
    WATCHER_PAUSED.store(false, Ordering::SeqCst);
}

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
        emitter: Arc<dyn EventEmitter>,
        cache: RepoStateCache,
    ) -> anyhow::Result<Self> {
        let (watcher_handle, rx) = handler::watch_repo(repo_path, poll_interval_ms)?;
        let path = repo_path.to_path_buf();
        let path_for_task = path.clone();

        let task = tokio::spawn(async move {
            event_consumer(rx, path_for_task, emitter, cache).await;
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

/// Background task that consumes repo change events and emits events via the emitter.
async fn event_consumer(
    mut rx: mpsc::Receiver<RepoChangeEvent>,
    repo_path: PathBuf,
    emitter: Arc<dyn EventEmitter>,
    cache: RepoStateCache,
) {
    let path_str = repo_path.to_string_lossy().to_string();

    while let Some(event) = rx.recv().await {
        // Coalesce rapid events: drain any queued events within a short window
        let mut has_workdir = matches!(event, RepoChangeEvent::WorkdirChanged);
        let mut has_refs = matches!(event, RepoChangeEvent::HeadChanged | RepoChangeEvent::RefsChanged);

        // Brief pause to coalesce rapid-fire events
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Drain any additional queued events
        while let Ok(extra) = rx.try_recv() {
            match extra {
                RepoChangeEvent::WorkdirChanged => has_workdir = true,
                RepoChangeEvent::HeadChanged | RepoChangeEvent::RefsChanged => has_refs = true,
            }
        }

        // Skip processing if watcher is paused (git CLI operation in progress)
        if WATCHER_PAUSED.load(Ordering::SeqCst) {
            continue;
        }

        if has_refs {
            // Compute a fingerprint of all ref targets to skip redundant graph rebuilds
            let new_fingerprint = compute_refs_fingerprint(&path_str);
            let old_fingerprint = cache.get_refs_fingerprint();
            let refs_changed = new_fingerprint != old_fingerprint.unwrap_or(0);

            if refs_changed {
                if let Some((status, graph)) = refresh_status_and_graph(&path_str) {
                    cache.update_status(status.clone());
                    cache.update_graph(graph.clone());
                    cache.update_refs_fingerprint(new_fingerprint);
                    let status_payload = StatusChangedPayload {
                        status: status.clone(),
                    };
                    let refs_payload = RefsChangedPayload { graph, status };
                    emitter.emit_status_changed(&status_payload);
                    emitter.emit_refs_changed(&refs_payload);
                }
            } else {
                // Refs didn't actually change — just refresh status
                if let Some(status) = refresh_status(&path_str) {
                    cache.update_status(status.clone());
                    let payload = StatusChangedPayload { status };
                    emitter.emit_status_changed(&payload);
                }
            }
        } else if has_workdir {
            if let Some(status) = refresh_status(&path_str) {
                cache.update_status(status.clone());
                let payload = StatusChangedPayload { status };
                emitter.emit_status_changed(&payload);
            }
        }
    }
}

/// Compute a hash of all ref targets (HEAD + branch tips + tags).
/// Used to detect whether refs have actually changed.
fn compute_refs_fingerprint(path: &str) -> u64 {
    let repo = match repository::open(path) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let mut hasher = DefaultHasher::new();

    // Hash HEAD
    if let Ok(head) = repo.head() {
        if let Some(oid) = head.target() {
            oid.as_bytes().hash(&mut hasher);
        }
        if let Some(name) = head.shorthand() {
            name.hash(&mut hasher);
        }
    }

    // Hash all references
    if let Ok(refs) = repo.references() {
        let mut ref_pairs: Vec<(String, String)> = refs
            .flatten()
            .filter_map(|r| {
                let name = r.name()?.to_string();
                let oid = r.target().map(|o| o.to_string())?;
                Some((name, oid))
            })
            .collect();
        // Sort for deterministic hashing
        ref_pairs.sort();
        for (name, oid) in &ref_pairs {
            name.hash(&mut hasher);
            oid.hash(&mut hasher);
        }
    }

    hasher.finish()
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
