use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{error::GitError, repository, types::CommitResult};

#[derive(Deserialize)]
pub struct CommitRequest {
    path: String,
    message: String,
}

pub async fn create_commit(
    Json(req): Json<CommitRequest>,
) -> Result<Json<CommitResult>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))
        .map_err(err)?
        .to_string_lossy()
        .to_string();
    let result = repository::create_commit(&workdir, &req.message).map_err(err)?;
    Ok(Json(result))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
