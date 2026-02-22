use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{repository, types::CloneResult};

#[derive(Deserialize)]
pub struct CloneRequest {
    url: String,
    dest: String,
}

pub async fn clone_repo(
    Json(req): Json<CloneRequest>,
) -> Result<Json<CloneResult>, (StatusCode, String)> {
    let result = repository::clone_repo(&req.url, &req.dest)
        .await
        .map_err(err)?;
    Ok(Json(result))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
