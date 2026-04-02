use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{repository, types::*};

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct PurgeRequest {
    path: String,
    refs: Vec<CheckpointRef>,
}

pub async fn find_checkpoint_refs(
    Json(req): Json<PathRequest>,
) -> Result<Json<Vec<CheckpointRef>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| "Bare repository")
        .map_err(err)?
        .to_string_lossy()
        .to_string();
    let refs = repository::find_checkpoint_refs(&workdir).map_err(err)?;
    Ok(Json(refs))
}

pub async fn purge_checkpoint_refs(
    Json(req): Json<PurgeRequest>,
) -> Result<Json<usize>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| "Bare repository")
        .map_err(err)?
        .to_string_lossy()
        .to_string();
    let deleted = repository::purge_checkpoint_refs(&workdir, &req.refs).map_err(err)?;
    Ok(Json(deleted))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
