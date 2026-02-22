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

/// Classify a changed path into a repo change event
fn classify_event(path: &Path) -> RepoChangeEvent {
    let path_str = path.to_string_lossy();

    if let Some(git_rel) = path_str.find(".git").map(|i| &path_str[i + 4..]) {
        // .git/HEAD -> HeadChanged
        if git_rel.is_empty() || git_rel == "/HEAD" || git_rel == "\\HEAD" {
            return RepoChangeEvent::HeadChanged;
        }
        // .git/refs/ -> RefsChanged
        if git_rel.starts_with("/refs") || git_rel.starts_with("\\refs") {
            return RepoChangeEvent::RefsChanged;
        }
        // .git/index -> WorkdirChanged (staging area)
        if git_rel == "/index" || git_rel == "\\index" {
            return RepoChangeEvent::WorkdirChanged;
        }
        // Other .git/ changes -> RefsChanged
        return RepoChangeEvent::RefsChanged;
    }

    // Non-.git paths -> WorkdirChanged
    RepoChangeEvent::WorkdirChanged
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
            if let Ok(events) = events {
                send_classified_events(&tx, &events);
            }
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
            if let Ok(events) = events {
                send_classified_events(&tx, &events);
            }
        },
    )?;

    debouncer
        .watcher()
        .watch(repo_path, notify::RecursiveMode::Recursive)?;

    Ok(debouncer)
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
