use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{error::GitError, remote, repository, types::*};

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

pub async fn list_branches(
    Json(req): Json<PathRequest>,
) -> Result<Json<Vec<Branch>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let branches = repository::list_branches(&repo).map_err(err)?;
    Ok(Json(branches))
}

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    path: String,
    name: String,
    target: Option<String>,
}

pub async fn create_branch(
    Json(req): Json<CreateBranchRequest>,
) -> Result<Json<Branch>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let target_ref = req.target.as_deref().unwrap_or("HEAD");
    let branch = repository::create_branch(&repo, &req.name, target_ref).map_err(err)?;
    Ok(Json(branch))
}

#[derive(Deserialize)]
pub struct BranchNameRequest {
    path: String,
    name: String,
}

pub async fn checkout_branch(
    Json(req): Json<BranchNameRequest>,
) -> Result<Json<RepoInfo>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::checkout_branch(&repo, &req.name).map_err(err)?;
    let info = repository::get_repo_info(&repo).map_err(err)?;
    Ok(Json(info))
}

pub async fn delete_branch(
    Json(req): Json<BranchNameRequest>,
) -> Result<Json<Vec<Branch>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::delete_branch(&repo, &req.name).map_err(err)?;
    let branches = repository::list_branches(&repo).map_err(err)?;
    Ok(Json(branches))
}

#[derive(Deserialize)]
pub struct ResetRequest {
    path: String,
    #[serde(rename = "commitOid")]
    commit_oid: String,
    #[serde(rename = "resetType")]
    reset_type: String,
}

pub async fn reset_to_commit(
    Json(req): Json<ResetRequest>,
) -> Result<Json<RepoInfo>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    repository::reset_to_commit(&repo, &req.commit_oid, &req.reset_type).map_err(err)?;
    let info = repository::get_repo_info(&repo).map_err(err)?;
    Ok(Json(info))
}

#[derive(Deserialize)]
pub struct RebaseRequest {
    path: String,
    #[serde(rename = "ontoBranch")]
    onto_branch: String,
}

pub async fn rebase_onto(
    Json(req): Json<RebaseRequest>,
) -> Result<Json<RebaseResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::rebase_onto(&workdir, &req.onto_branch).map_err(err)?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct MergeRequest {
    path: String,
    #[serde(rename = "branchName")]
    branch_name: String,
}

pub async fn merge_into(
    Json(req): Json<MergeRequest>,
) -> Result<Json<MergeResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::merge_branch(&workdir, &req.branch_name).map_err(err)?;
    Ok(Json(result))
}

pub async fn find_merged_branches(
    Json(req): Json<PathRequest>,
) -> Result<Json<Vec<MergedBranch>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let merged = repository::find_merged_branches(&repo).map_err(err)?;
    Ok(Json(merged))
}

#[derive(Deserialize)]
pub struct CleanupBranchesRequest {
    path: String,
    branches: Vec<MergedBranch>,
}

pub async fn cleanup_merged_branches(
    Json(req): Json<CleanupBranchesRequest>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let deleted = remote::cleanup_merged_branches(&workdir, &req.branches)
        .await
        .map_err(err)?;
    Ok(Json(deleted))
}

pub async fn rebase_continue(
    Json(req): Json<PathRequest>,
) -> Result<Json<RebaseResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::rebase_continue(&workdir).map_err(err)?;
    Ok(Json(result))
}

pub async fn rebase_abort(
    Json(req): Json<PathRequest>,
) -> Result<Json<RebaseResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::rebase_abort(&workdir).map_err(err)?;
    Ok(Json(result))
}

pub async fn merge_continue(
    Json(req): Json<PathRequest>,
) -> Result<Json<MergeResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::merge_continue(&workdir).map_err(err)?;
    Ok(Json(result))
}

pub async fn merge_abort(
    Json(req): Json<PathRequest>,
) -> Result<Json<MergeResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::merge_abort(&workdir).map_err(err)?;
    Ok(Json(result))
}

pub async fn cherry_pick_abort(
    Json(req): Json<PathRequest>,
) -> Result<Json<OperationOutput>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = repository::cherry_pick_abort(&workdir).map_err(err)?;
    Ok(Json(result))
}

fn get_workdir(path: &str) -> Result<String, (StatusCode, String)> {
    let repo = repository::open(path).map_err(err)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))
        .map_err(err)?
        .to_string_lossy()
        .to_string();
    Ok(workdir)
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
