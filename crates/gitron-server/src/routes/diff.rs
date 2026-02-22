use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{diff as git_diff, repository, types::*};

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

pub async fn get_workdir_diff(
    Json(req): Json<PathRequest>,
) -> Result<Json<Vec<FileDiff>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let diffs = git_diff::diff_workdir(&repo).map_err(err)?;
    Ok(Json(diffs))
}

#[derive(Deserialize)]
pub struct FileDiffRequest {
    path: String,
    #[serde(rename = "filePath")]
    file_path: String,
}

pub async fn get_file_diff(
    Json(req): Json<FileDiffRequest>,
) -> Result<Json<FileDiff>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let diff = git_diff::diff_file(&repo, &req.file_path).map_err(err)?;
    Ok(Json(diff))
}

pub async fn get_staged_file_diff(
    Json(req): Json<FileDiffRequest>,
) -> Result<Json<FileDiff>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let diff = git_diff::diff_file_staged(&repo, &req.file_path).map_err(err)?;
    Ok(Json(diff))
}

#[derive(Deserialize)]
pub struct CommitDiffRequest {
    path: String,
    oid: String,
}

pub async fn get_commit_diff(
    Json(req): Json<CommitDiffRequest>,
) -> Result<Json<Vec<FileDiff>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let diffs = git_diff::diff_commit(&repo, &req.oid).map_err(err)?;
    Ok(Json(diffs))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
