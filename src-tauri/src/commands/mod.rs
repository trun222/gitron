pub mod repo;
pub mod graph;
pub mod diff;
pub mod staging;
pub mod branch;
pub mod commit;

use std::path::PathBuf;
use std::sync::Mutex;

use git2::Repository;

/// Shared application state holding the currently opened repository
pub struct AppState {
    pub repo: Mutex<Option<Repository>>,
    pub repo_path: Mutex<Option<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repo: Mutex::new(None),
            repo_path: Mutex::new(None),
        }
    }
}
