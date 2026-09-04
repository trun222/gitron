use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::cache::repo_state::RepoStateCache;
use crate::event::EventEmitter;
use crate::git::{graph, repository, types::*};

use super::handler::{self, RepoChangeEvent, WatcherHandle};

/// How long the consumer waits for a watcher event before checking on its own
/// whether refs changed.
///
/// Native watchers drop or coalesce events under heavy churn (an AI agent
/// rewriting hundreds of files, `git gc`, the bounded event channel filling up,
/// a `with_watcher_paused` window). A missed HEAD or ref update would otherwise
/// leave the UI on a stale branch until the next unrelated event — the user
/// sees Gitron "lose track" of the repo. The check is a cheap ref walk.
const RESYNC_INTERVAL: Duration = Duration::from_secs(5);

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
            event_consumer(rx, path_for_task, emitter, cache, RESYNC_INTERVAL).await;
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
///
/// Every wake-up — whether caused by a watcher event or by `resync_interval`
/// elapsing with no events — compares the refs fingerprint against the cache, so
/// a ref change is picked up even when the event that should have announced it
/// was dropped, coalesced away, or swallowed by a paused window.
async fn event_consumer(
    mut rx: mpsc::Receiver<RepoChangeEvent>,
    repo_path: PathBuf,
    emitter: Arc<dyn EventEmitter>,
    cache: RepoStateCache,
    resync_interval: Duration,
) {
    let path_str = repo_path.to_string_lossy().to_string();

    loop {
        let (has_workdir, has_refs) = match tokio::time::timeout(resync_interval, rx.recv()).await {
            Ok(Some(event)) => {
                // Coalesce rapid events: drain any queued events within a short window
                let mut has_workdir = matches!(event, RepoChangeEvent::WorkdirChanged);
                let mut has_refs =
                    matches!(event, RepoChangeEvent::HeadChanged | RepoChangeEvent::RefsChanged);

                // Brief pause to coalesce rapid-fire events
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Drain any additional queued events
                while let Ok(extra) = rx.try_recv() {
                    match extra {
                        RepoChangeEvent::WorkdirChanged => has_workdir = true,
                        RepoChangeEvent::HeadChanged | RepoChangeEvent::RefsChanged => {
                            has_refs = true
                        }
                    }
                }
                (has_workdir, has_refs)
            }
            // Watcher handle dropped — nothing more will ever arrive.
            Ok(None) => break,
            // Idle: no events, but still verify refs below.
            Err(_) => (false, false),
        };

        // Skip processing if watcher is paused (git CLI operation in progress).
        // Anything that changes meanwhile is caught by the next wake-up.
        if WATCHER_PAUSED.load(Ordering::SeqCst) {
            continue;
        }

        // Compare ref targets against the cache on every wake-up. This is what
        // makes the consumer self-healing when events go missing.
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
        } else if has_workdir || has_refs {
            // Refs didn't actually change — just refresh status
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::Mutex;
    use std::time::Instant;

    /// Records the branch list (name, is_head) of every refs-changed payload.
    struct RecordingEmitter {
        refs: Mutex<Vec<Vec<(String, bool)>>>,
    }

    impl EventEmitter for RecordingEmitter {
        fn emit_status_changed(&self, _payload: &StatusChangedPayload) {}
        fn emit_refs_changed(&self, payload: &RefsChangedPayload) {
            let branches = payload
                .graph
                .branches
                .iter()
                .map(|b| (b.name.clone(), b.is_head))
                .collect();
            self.refs.lock().expect("lock").push(branches);
        }
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git should run");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// An AI agent (or any external tool) switching branches while the OS
    /// watcher drops the event must still be reflected in the UI. No event is
    /// ever sent on the channel here; only the periodic resync can notice.
    #[tokio::test]
    async fn resyncs_refs_without_any_watcher_event() {
        let dir = std::env::temp_dir().join(format!("gitron-watcher-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        git(&dir, &["init", "--initial-branch=main"]);
        git(&dir, &["config", "user.email", "test@example.com"]);
        git(&dir, &["config", "user.name", "Test"]);
        std::fs::write(dir.join("a.txt"), "hello").expect("write");
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-m", "init"]);
        let path = dir.to_string_lossy().to_string();

        let cache = RepoStateCache::new();
        {
            let repo = repository::open(&path).expect("open");
            let info = repository::get_repo_info(&repo).expect("info");
            let status = repository::get_status(&repo).expect("status");
            let graph = graph::build_commit_graph(&repo, &GraphOptions::default()).expect("graph");
            cache.initialize(info, status, graph);
        }
        cache.update_refs_fingerprint(compute_refs_fingerprint(&path));

        let emitter = Arc::new(RecordingEmitter { refs: Mutex::new(Vec::new()) });
        // Keep the sender alive but never use it: simulates lost events.
        let (_tx, rx) = mpsc::channel::<RepoChangeEvent>(8);
        let consumer = tokio::spawn(event_consumer(
            rx,
            dir.clone(),
            emitter.clone(),
            cache,
            Duration::from_millis(100),
        ));

        // Idle checks must not emit spurious refs-changed events.
        tokio::time::sleep(Duration::from_millis(350)).await;
        assert!(
            emitter.refs.lock().expect("lock").is_empty(),
            "refs-changed emitted while nothing changed"
        );

        // External branch switch — Gitron is never told about it.
        git(&dir, &["checkout", "-b", "agent/work"]);

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let seen = emitter
                .refs
                .lock()
                .expect("lock")
                .iter()
                .any(|branches| branches.iter().any(|(n, head)| n == "agent/work" && *head));
            if seen {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "consumer never resynced after an external branch switch"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        consumer.abort();
        repository::invalidate_cache(&path);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
