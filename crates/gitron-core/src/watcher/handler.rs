use std::path::Path;
use std::time::Duration;

use notify::PollWatcher;
use notify_debouncer_mini::{new_debouncer, new_debouncer_opt, Debouncer};
use tokio::sync::mpsc;

/// Types of repository change events
#[derive(Debug, Clone)]
pub enum RepoChangeEvent {
    WorkdirChanged,
    HeadChanged,
    RefsChanged,
}

/// Wraps either a native or poll-based file watcher
pub enum WatcherHandle {
    Native(Debouncer<notify::RecommendedWatcher>),
    Poll(Debouncer<PollWatcher>),
}

/// Classify a changed path into a repo change event.
///
/// Matches on a path component that is exactly `.git` (the git dir, or the
/// `.git` file of a linked worktree) rather than the substring ".git", so that
/// `.gitignore`, `.github/`, or a repo living under `~/foo.github/` are not
/// mistaken for ref changes.
fn classify_event(path: &Path) -> RepoChangeEvent {
    use std::path::Component;

    let mut components = path.components();
    let in_git_dir = components
        .by_ref()
        .any(|c| matches!(c, Component::Normal(name) if name == ".git"));
    if !in_git_dir {
        return RepoChangeEvent::WorkdirChanged;
    }

    let rel: Vec<&str> = components
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    match rel.as_slice() {
        // The .git entry itself (e.g. a worktree's .git file) or .git/HEAD
        [] | ["HEAD"] => RepoChangeEvent::HeadChanged,
        // .git/index -> staging area
        ["index"] => RepoChangeEvent::WorkdirChanged,
        // .git/refs/**, packed-refs, logs, objects, ... -> let the consumer's
        // refs fingerprint decide whether anything actually changed
        _ => RepoChangeEvent::RefsChanged,
    }
}

/// Start watching a repository directory for changes.
/// Tries native watcher first; falls back to PollWatcher if `poll_interval_ms > 0`.
/// Returns a handle and a channel receiver for repo change events.
pub fn watch_repo(
    repo_path: &Path,
    poll_interval_ms: u64,
) -> anyhow::Result<(WatcherHandle, mpsc::Receiver<RepoChangeEvent>)> {
    let (tx, rx) = mpsc::channel(64);

    // Try native watcher first
    match try_native_watcher(repo_path, tx.clone()) {
        Ok(handle) => return Ok((WatcherHandle::Native(handle), rx)),
        Err(e) => {
            log::warn!("Native file watcher failed: {e}");
            if poll_interval_ms == 0 {
                return Err(e);
            }
            log::info!("Falling back to poll watcher (interval: {poll_interval_ms}ms)");
        }
    }

    // Fall back to poll watcher
    let handle = try_poll_watcher(repo_path, tx, poll_interval_ms)?;
    Ok((WatcherHandle::Poll(handle), rx))
}

fn try_native_watcher(
    repo_path: &Path,
    tx: mpsc::Sender<RepoChangeEvent>,
) -> anyhow::Result<Debouncer<notify::RecommendedWatcher>> {
    let mut debouncer = new_debouncer(
        Duration::from_millis(200),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            handle_watcher_result(&tx, events);
        },
    )?;

    debouncer
        .watcher()
        .watch(repo_path, notify::RecursiveMode::Recursive)?;

    Ok(debouncer)
}

fn try_poll_watcher(
    repo_path: &Path,
    tx: mpsc::Sender<RepoChangeEvent>,
    poll_interval_ms: u64,
) -> anyhow::Result<Debouncer<PollWatcher>> {
    let notify_config = notify::Config::default()
        .with_poll_interval(Duration::from_millis(poll_interval_ms));

    let config = notify_debouncer_mini::Config::default()
        .with_timeout(Duration::from_millis(200))
        .with_notify_config(notify_config);

    let mut debouncer = new_debouncer_opt::<_, PollWatcher>(
        config,
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            handle_watcher_result(&tx, events);
        },
    )?;

    debouncer
        .watcher()
        .watch(repo_path, notify::RecursiveMode::Recursive)?;

    Ok(debouncer)
}

/// Forward a batch from the OS watcher. An error (event queue overflow, a
/// watch that could not be established, ...) means events may have been lost,
/// so it is treated as "something changed" to trigger a resync rather than
/// being dropped on the floor.
fn handle_watcher_result(
    tx: &mpsc::Sender<RepoChangeEvent>,
    events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>,
) {
    match events {
        Ok(events) => send_classified_events(tx, &events),
        Err(e) => {
            log::warn!("File watcher error (forcing resync): {e}");
            tx.try_send(RepoChangeEvent::RefsChanged).ok();
        }
    }
}

fn send_classified_events(
    tx: &mpsc::Sender<RepoChangeEvent>,
    events: &[notify_debouncer_mini::DebouncedEvent],
) {
    let mut has_workdir = false;
    let mut has_head = false;
    let mut has_refs = false;

    for event in events {
        match classify_event(&event.path) {
            RepoChangeEvent::WorkdirChanged => has_workdir = true,
            RepoChangeEvent::HeadChanged => has_head = true,
            RepoChangeEvent::RefsChanged => has_refs = true,
        }
    }

    // Send deduplicated events (head/refs imply workdir too, but we handle them separately)
    if has_head {
        tx.try_send(RepoChangeEvent::HeadChanged).ok();
    } else if has_refs {
        tx.try_send(RepoChangeEvent::RefsChanged).ok();
    }
    if has_workdir {
        tx.try_send(RepoChangeEvent::WorkdirChanged).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn classify(p: &str) -> RepoChangeEvent {
        classify_event(&PathBuf::from(p))
    }

    #[test]
    fn classifies_git_dir_entries() {
        assert!(matches!(classify("/r/.git/HEAD"), RepoChangeEvent::HeadChanged));
        assert!(matches!(classify("/r/.git"), RepoChangeEvent::HeadChanged));
        assert!(matches!(classify("/r/.git/index"), RepoChangeEvent::WorkdirChanged));
        assert!(matches!(classify("/r/.git/refs/heads/main"), RepoChangeEvent::RefsChanged));
        assert!(matches!(classify("/r/.git/packed-refs"), RepoChangeEvent::RefsChanged));
        assert!(matches!(classify("/r/.git/worktrees/x/HEAD"), RepoChangeEvent::RefsChanged));
    }

    #[test]
    fn workdir_paths_that_merely_contain_dot_git_are_workdir_changes() {
        assert!(matches!(classify("/r/.gitignore"), RepoChangeEvent::WorkdirChanged));
        assert!(matches!(classify("/r/.github/workflows/ci.yml"), RepoChangeEvent::WorkdirChanged));
        assert!(matches!(classify("/home/me/foo.github/src/a.ts"), RepoChangeEvent::WorkdirChanged));
        assert!(matches!(classify("/r/src/lib.rs"), RepoChangeEvent::WorkdirChanged));
    }
}
