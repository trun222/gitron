use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{error::GitError, remote, repository, types::*};

#[derive(Deserialize)]
pub struct PathRequest {
    path: String,
}

pub async fn list_remotes(
    Json(req): Json<PathRequest>,
) -> Result<Json<Vec<Remote>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let remotes = remote::list_remotes(&repo).map_err(err)?;
    Ok(Json(remotes))
}

#[derive(Deserialize)]
pub struct AddRemoteRequest {
    path: String,
    name: String,
    url: String,
}

pub async fn add_remote(
    Json(req): Json<AddRemoteRequest>,
) -> Result<Json<Vec<Remote>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    remote::add_remote(&repo, &req.name, &req.url).map_err(err)?;
    let remotes = remote::list_remotes(&repo).map_err(err)?;
    Ok(Json(remotes))
}

#[derive(Deserialize)]
pub struct RemoveRemoteRequest {
    path: String,
    name: String,
}

pub async fn remove_remote(
    Json(req): Json<RemoveRemoteRequest>,
) -> Result<Json<Vec<Remote>>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    remote::remove_remote(&repo, &req.name).map_err(err)?;
    let remotes = remote::list_remotes(&repo).map_err(err)?;
    Ok(Json(remotes))
}

#[derive(Deserialize)]
pub struct TrackingRequest {
    path: String,
    #[serde(rename = "branchName")]
    branch_name: String,
}

pub async fn get_tracking_status(
    Json(req): Json<TrackingRequest>,
) -> Result<Json<TrackingStatus>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    let status = remote::get_tracking_status(&repo, &req.branch_name).map_err(err)?;
    Ok(Json(status))
}

#[derive(Deserialize)]
pub struct FetchRequest {
    path: String,
    #[serde(rename = "remoteName")]
    remote_name: String,
    branch: Option<String>,
}

pub async fn fetch_remote(
    Json(req): Json<FetchRequest>,
) -> Result<Json<FetchResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = remote::fetch(&workdir, &req.remote_name, req.branch.as_deref())
        .await
        .map_err(err)?;
    Ok(Json(result))
}

pub async fn fetch_all_remotes(
    Json(req): Json<PathRequest>,
) -> Result<Json<FetchResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = remote::fetch_all(&workdir).await.map_err(err)?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct PushRequest {
    path: String,
    #[serde(rename = "remoteName")]
    remote_name: String,
    branch: Option<String>,
    force: Option<bool>,
    #[serde(rename = "setUpstream")]
    set_upstream: Option<bool>,
}

pub async fn push_to_remote(
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = remote::push(
        &workdir,
        &req.remote_name,
        req.branch.as_deref(),
        req.force.unwrap_or(false),
        req.set_upstream.unwrap_or(false),
    )
    .await
    .map_err(err)?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct PullRequest {
    path: String,
    #[serde(rename = "remoteName")]
    remote_name: String,
    branch: Option<String>,
}

pub async fn pull_from_remote(
    Json(req): Json<PullRequest>,
) -> Result<Json<PullResult>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    let result = remote::pull(&workdir, &req.remote_name, req.branch.as_deref())
        .await
        .map_err(err)?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct DeleteRemoteBranchRequest {
    path: String,
    #[serde(rename = "remoteName")]
    remote_name: String,
    branch: String,
}

pub async fn delete_remote_branch(
    Json(req): Json<DeleteRemoteBranchRequest>,
) -> Result<Json<Vec<Branch>>, (StatusCode, String)> {
    let workdir = get_workdir(&req.path)?;
    remote::delete_remote_branch(&workdir, &req.remote_name, &req.branch)
        .await
        .map_err(err)?;
    let repo = repository::open(&req.path).map_err(err)?;
    let branches = repository::list_branches(&repo).map_err(err)?;
    Ok(Json(branches))
}

#[derive(Deserialize)]
pub struct CheckoutRemoteRequest {
    path: String,
    #[serde(rename = "remoteBranchName")]
    remote_branch_name: String,
}

pub async fn checkout_remote_branch(
    Json(req): Json<CheckoutRemoteRequest>,
) -> Result<Json<RepoInfo>, (StatusCode, String)> {
    let repo = repository::open(&req.path).map_err(err)?;
    remote::checkout_remote_branch(&repo, &req.remote_branch_name).map_err(err)?;
    let info = repository::get_repo_info(&repo).map_err(err)?;
    Ok(Json(info))
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
