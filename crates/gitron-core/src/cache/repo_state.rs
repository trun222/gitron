use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::git::types::*;

/// Cached repository state for fast access from the frontend.
/// Updated incrementally by the file watcher.
#[derive(Clone)]
pub struct RepoStateCache {
    inner: Arc<RwLock<Option<CachedState>>>,
}

struct CachedState {
    #[allow(dead_code)]
    pub repo_info: RepoInfo,
    pub status: RepoStatus,
    pub graph: CommitGraph,
    pub last_updated: Instant,
    /// Fingerprint of ref targets — used to skip redundant graph rebuilds
    pub refs_fingerprint: u64,
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

    /// Get the current refs fingerprint for change detection
    pub fn get_refs_fingerprint(&self) -> Option<u64> {
        self.inner
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.refs_fingerprint))
    }

    /// Update the refs fingerprint
    pub fn update_refs_fingerprint(&self, fingerprint: u64) {
        if let Ok(mut guard) = self.inner.write() {
            if let Some(ref mut state) = *guard {
                state.refs_fingerprint = fingerprint;
            }
        }
    }

    pub fn initialize(&self, info: RepoInfo, status: RepoStatus, graph: CommitGraph) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = Some(CachedState {
                repo_info: info,
                status,
                graph,
                last_updated: Instant::now(),
                refs_fingerprint: 0,
            });
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = None;
        }
    }
}
