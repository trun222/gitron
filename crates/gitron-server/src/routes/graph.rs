use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{graph as git_graph, repository, types::*};

#[derive(Deserialize)]
pub struct GraphRequest {
    path: String,
    #[serde(rename = "maxCommits")]
    max_commits: Option<usize>,
    #[serde(rename = "includeRemotes")]
    include_remotes: Option<bool>,
}

pub async fn get_commit_graph(
    Json(req): Json<GraphRequest>,
) -> Result<Json<CommitGraph>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let options = GraphOptions {
        max_commits: req.max_commits.or(Some(500)),
        from_oid: None,
        include_remotes: req.include_remotes.unwrap_or(true),
    };
    let graph = git_graph::build_commit_graph(&repo, &options).map_err(err)?;
    Ok(Json(graph))
}

#[derive(Deserialize)]
pub struct CommitDetailRequest {
    path: String,
    oid: String,
}

pub async fn get_commit_detail(
    Json(req): Json<CommitDetailRequest>,
) -> Result<Json<Commit>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let commit = git_graph::get_commit_detail(&repo, &req.oid).map_err(err)?;
    Ok(Json(commit))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
