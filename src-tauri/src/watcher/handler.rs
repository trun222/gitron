use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify_debouncer_mini::new_debouncer;

/// Start watching a repository directory for changes.
/// Returns a channel receiver that emits repo change events.
pub fn watch_repo(
    repo_path: &Path,
) -> anyhow::Result<(
    notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    mpsc::Receiver<RepoChangeEvent>,
)> {
    let (tx, rx) = mpsc::channel();

    let sender = tx.clone();
    let mut debouncer = new_debouncer(Duration::from_millis(200), move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
        if let Ok(events) = events {
            let mut has_git_change = false;
            let mut has_workdir_change = false;

            for event in &events {
                let path_str = event.path.to_string_lossy();
                if path_str.contains(".git") {
                    has_git_change = true;
                } else {
                    has_workdir_change = true;
                }
            }

            if has_git_change {
                sender.send(RepoChangeEvent::RefsChanged).ok();
            }
            if has_workdir_change {
                sender.send(RepoChangeEvent::WorkdirChanged).ok();
            }
        }
    })?;

    debouncer
        .watcher()
        .watch(repo_path, notify::RecursiveMode::Recursive)?;

    Ok((debouncer, rx))
}

/// Types of repository change events
#[derive(Debug, Clone)]
pub enum RepoChangeEvent {
    WorkdirChanged,
    RefsChanged,
}
