use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{worktree, types::*};

#[derive(Deserialize)]
pub struct ListRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct AddRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
    branch: Option<String>,
    #[serde(rename = "newBranch")]
    new_branch: bool,
}

#[derive(Deserialize)]
pub struct RemoveRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
    force: bool,
}

#[derive(Deserialize)]
pub struct LockRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
pub struct UnlockRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
}

#[derive(Deserialize)]
pub struct PruneRequest {
    path: String,
    #[serde(rename = "dryRun")]
    dry_run: bool,
}

pub async fn list_worktrees(
    Json(req): Json<ListRequest>,
) -> Result<Json<Vec<WorktreeInfo>>, (StatusCode, String)> {
    let result = worktree::list_worktrees(&req.path).map_err(err)?;
    Ok(Json(result))
}

pub async fn add_worktree(
    Json(req): Json<AddRequest>,
) -> Result<Json<WorktreeCreateResult>, (StatusCode, String)> {
    let result = worktree::add_worktree(
        &req.path,
        &req.worktree_path,
        req.branch.as_deref(),
        req.new_branch,
    )
    .map_err(err)?;
    Ok(Json(result))
}

pub async fn remove_worktree(
    Json(req): Json<RemoveRequest>,
) -> Result<Json<WorktreeRemoveResult>, (StatusCode, String)> {
    let result =
        worktree::remove_worktree(&req.path, &req.worktree_path, req.force).map_err(err)?;
    Ok(Json(result))
}

pub async fn lock_worktree(
    Json(req): Json<LockRequest>,
) -> Result<Json<Vec<WorktreeInfo>>, (StatusCode, String)> {
    worktree::lock_worktree(&req.path, &req.worktree_path, req.reason.as_deref())
        .map_err(err)?;
    let list = worktree::list_worktrees(&req.path).map_err(err)?;
    Ok(Json(list))
}

pub async fn unlock_worktree(
    Json(req): Json<UnlockRequest>,
) -> Result<Json<Vec<WorktreeInfo>>, (StatusCode, String)> {
    worktree::unlock_worktree(&req.path, &req.worktree_path).map_err(err)?;
    let list = worktree::list_worktrees(&req.path).map_err(err)?;
    Ok(Json(list))
}

pub async fn prune_worktrees(
    Json(req): Json<PruneRequest>,
) -> Result<Json<WorktreePruneResult>, (StatusCode, String)> {
    let result = worktree::prune_worktrees(&req.path, req.dry_run).map_err(err)?;
    Ok(Json(result))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
