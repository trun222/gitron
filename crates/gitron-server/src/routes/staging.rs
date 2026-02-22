use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{repository, types::*};

#[derive(Deserialize)]
pub struct StageFileRequest {
    path: String,
    #[serde(rename = "filePath")]
    file_path: String,
}

pub async fn stage_file(
    Json(req): Json<StageFileRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::stage_file(&repo, &req.file_path).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn unstage_file(
    Json(req): Json<StageFileRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::unstage_file(&repo, &req.file_path).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

#[derive(Deserialize)]
pub struct StageFilesRequest {
    path: String,
    #[serde(rename = "filePaths")]
    file_paths: Vec<String>,
}

pub async fn stage_files(
    Json(req): Json<StageFilesRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::stage_files(&repo, &req.file_paths).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

pub async fn stage_all(
    Json(req): Json<PathRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::stage_all(&repo).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn unstage_all(
    Json(req): Json<PathRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::unstage_all(&repo).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

pub async fn discard_all(
    Json(req): Json<PathRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::discard_all_changes(&repo).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
