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
    #[serde(rename = "excludedAuthors", default)]
    excluded_authors: Vec<String>,
}

pub async fn get_commit_graph(
    Json(req): Json<GraphRequest>,
) -> Result<Json<CommitGraph>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let options = GraphOptions {
        max_commits: req.max_commits.or(Some(500)),
        from_oid: None,
        include_remotes: req.include_remotes.unwrap_or(true),
        excluded_authors: req.excluded_authors,
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

#[derive(Deserialize)]
pub struct SearchRequest {
    path: String,
    query: String,
    #[serde(rename = "maxCommits")]
    max_commits: Option<usize>,
    #[serde(rename = "includeRemotes")]
    include_remotes: Option<bool>,
    #[serde(rename = "searchDiffs")]
    search_diffs: Option<bool>,
}

pub async fn search_commits(
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let options = GraphOptions {
        max_commits: req.max_commits.or(Some(500)),
        from_oid: None,
        include_remotes: req.include_remotes.unwrap_or(true),
        excluded_authors: Vec::new(),
    };
    let results = git_graph::search_commits(&repo, &req.query, &options, req.search_diffs.unwrap_or(false)).map_err(err)?;
    Ok(Json(results))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
