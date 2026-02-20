pub mod repo;
pub mod graph;
pub mod diff;
pub mod staging;
pub mod branch;
pub mod commit;
pub mod stash;
pub mod remote;
pub mod github;
pub mod clone;

use std::path::PathBuf;
use std::sync::Mutex;

use crate::cache::repo_state::RepoStateCache;
use crate::watcher::manager::WatcherManager;

/// Shared application state
pub struct AppState {
    pub watcher: Mutex<Option<WatcherManager>>,
    pub cache: RepoStateCache,
    pub repo_path: Mutex<Option<PathBuf>>,
    pub poll_interval_ms: Mutex<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            watcher: Mutex::new(None),
            cache: RepoStateCache::new(),
            repo_path: Mutex::new(None),
            poll_interval_ms: Mutex::new(0),
        }
    }
}
