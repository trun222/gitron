use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{repository, types::*};

#[derive(Deserialize)]
pub struct SaveStashRequest {
    path: String,
    message: Option<String>,
}

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct StashRequest {
    path: String,
    index: usize,
}

pub async fn save_stash(
    Json(req): Json<SaveStashRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let mut repo = repository::open(&req.path).map_err(err)?;
    repository::save_stash(&mut repo, req.message.as_deref()).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn list_stashes(
    Json(req): Json<PathRequest>,
) -> Result<Json<Vec<StashEntry>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let stashes = repository::list_stashes(&repo).map_err(err)?;
    Ok(Json(stashes))
}

pub async fn apply_stash(
    Json(req): Json<StashRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let mut repo = repository::open(&req.path).map_err(err)?;
    repository::apply_stash(&mut repo, req.index).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn pop_stash(
    Json(req): Json<StashRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let mut repo = repository::open(&req.path).map_err(err)?;
    repository::pop_stash(&mut repo, req.index).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn drop_stash(
    Json(req): Json<StashRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let mut repo = repository::open(&req.path).map_err(err)?;
    repository::drop_stash(&mut repo, req.index).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
