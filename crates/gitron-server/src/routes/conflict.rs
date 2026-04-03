use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{repository, types::*};

#[derive(Deserialize)]
pub struct ConflictFileRequest {
    path: String,
    #[serde(rename = "filePath")]
    file_path: String,
}

pub async fn get_conflicted_file(
    Json(req): Json<ConflictFileRequest>,
) -> Result<Json<ConflictedFileContent>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let content = repository::get_conflicted_file(&repo, &req.file_path).map_err(err)?;
    Ok(Json(content))
}

#[derive(Deserialize)]
pub struct ResolveFileRequest {
    path: String,
    #[serde(rename = "filePath")]
    file_path: String,
    content: String,
}

pub async fn write_resolved_file(
    Json(req): Json<ResolveFileRequest>,
) -> Result<Json<RepoStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::write_file_content(&repo, &req.file_path, &req.content).map_err(err)?;
    repository::stage_file(&repo, &req.file_path).map_err(err)?;
    let status = repository::get_status(&repo).map_err(err)?;
    Ok(Json(status))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
