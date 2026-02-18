use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::git::types::*;

/// Cached repository state for fast access from the frontend.
/// Updated incrementally by the file watcher.
pub struct RepoStateCache {
    inner: Arc<RwLock<Option<CachedState>>>,
}

struct CachedState {
    #[allow(dead_code)]
    pub repo_info: RepoInfo,
    pub status: RepoStatus,
    pub graph: CommitGraph,
    pub last_updated: Instant,
}

impl RepoStateCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub fn update_status(&self, status: RepoStatus) {
        if let Ok(mut guard) = self.inner.write() {
            if let Some(ref mut state) = *guard {
                state.status = status;
                state.last_updated = Instant::now();
            }
        }
    }

    pub fn update_graph(&self, graph: CommitGraph) {
        if let Ok(mut guard) = self.inner.write() {
            if let Some(ref mut state) = *guard {
                state.graph = graph;
                state.last_updated = Instant::now();
            }
        }
    }

    pub fn get_status(&self) -> Option<RepoStatus> {
        self.inner
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.status.clone()))
    }

    pub fn get_graph(&self) -> Option<CommitGraph> {
        self.inner
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.graph.clone()))
    }

    pub fn initialize(&self, info: RepoInfo, status: RepoStatus, graph: CommitGraph) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = Some(CachedState {
                repo_info: info,
                status,
                graph,
                last_updated: Instant::now(),
            });
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = None;
        }
    }
}
