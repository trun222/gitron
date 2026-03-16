use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{graph, repository, types::*};
use gitron_core::watcher::manager::WatcherManager;

use crate::ServerState;

#[derive(Deserialize)]
pub struct OpenRepoRequest {
    path: String,
}

pub async fn open_repo(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<OpenRepoRequest>,
) -> Result<Json<RepoInfo>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let info = repository::get_repo_info(&repo).map_err(err)?;

    let status = repository::get_status(&repo).map_err(err)?;
    let graph_data = graph::build_commit_graph(&repo, &GraphOptions::default()).map_err(err)?;
    state.cache.initialize(info.clone(), status, graph_data);

    // Stop existing watcher
    if let Some(old) = state.watcher.lock().unwrap().take() {
        old.stop();
    }

    *state.repo_path.lock().unwrap() = Some(PathBuf::from(&req.path));

    // Start file watcher with SSE broadcaster as event emitter
    let poll_interval = *state.poll_interval_ms.lock().unwrap();
    let repo_path = PathBuf::from(&req.path);
    let emitter: Arc<dyn gitron_core::event::EventEmitter> = state.broadcaster.clone();
    match WatcherManager::start(&repo_path, poll_interval, emitter, state.cache.clone()) {
        Ok(manager) => {
            *state.watcher.lock().unwrap() = Some(manager);
            log::info!("File watcher started for {}", req.path);
        }
        Err(e) => {
            log::warn!("File watcher failed: {e}");
        }
    }

    Ok(Json(info))
}

pub async fn close_repo(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<()>, (StatusCode, String)> {
    if let Some(watcher) = state.watcher.lock().unwrap().take() {
        watcher.stop();
    }
    // Invalidate the repo path cache for fast opens
    if let Some(path) = state.repo_path.lock().unwrap().as_ref() {
        repository::invalidate_cache(&path.to_string_lossy());
    }
    state.cache.clear();
    *state.repo_path.lock().unwrap() = None;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

pub async fn get_status(
    Json(req): Json<PathRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn get_repo_info(
    Json(req): Json<PathRequest>,
) -> Result<Json<RepoInfo>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let info = repository::get_repo_info(&repo).map_err(err)?;
    Ok(Json(info))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
