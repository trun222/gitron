use crate::git::{error::GitError, remote, repository, types::*};

/// List all configured remotes
#[tauri::command]
pub fn list_remotes(path: String) -> Result<Vec<Remote>, GitError> {
    let repo = repository::open(&path)?;
    remote::list_remotes(&repo)
}

/// Add a new remote
#[tauri::command]
pub fn add_remote(path: String, name: String, url: String) -> Result<Vec<Remote>, GitError> {
    let repo = repository::open(&path)?;
    remote::add_remote(&repo, &name, &url)?;
    remote::list_remotes(&repo)
}

/// Remove a remote
#[tauri::command]
pub fn remove_remote(path: String, name: String) -> Result<Vec<Remote>, GitError> {
    let repo = repository::open(&path)?;
    remote::remove_remote(&repo, &name)?;
    remote::list_remotes(&repo)
}

/// Get tracking status for a branch
#[tauri::command]
pub fn get_tracking_status(
    path: String,
    branch_name: String,
) -> Result<TrackingStatus, GitError> {
    let repo = repository::open(&path)?;
    remote::get_tracking_status(&repo, &branch_name)
}

/// Fetch from a specific remote
#[tauri::command]
pub async fn fetch_remote(
    path: String,
    remote_name: String,
    branch: Option<String>,
) -> Result<FetchResult, GitError> {
    let workdir = get_workdir(&path)?;
    remote::fetch(&workdir, &remote_name, branch.as_deref()).await
}

/// Fetch from all remotes
#[tauri::command]
pub async fn fetch_all_remotes(path: String) -> Result<FetchResult, GitError> {
    let workdir = get_workdir(&path)?;
    remote::fetch_all(&workdir).await
}

/// Push to a remote
#[tauri::command]
pub async fn push_to_remote(
    path: String,
    remote_name: String,
    branch: Option<String>,
    force: Option<bool>,
    set_upstream: Option<bool>,
) -> Result<PushResult, GitError> {
    let workdir = get_workdir(&path)?;
    remote::push(
        &workdir,
        &remote_name,
        branch.as_deref(),
        force.unwrap_or(false),
        set_upstream.unwrap_or(false),
    )
    .await
}

/// Pull from a remote
#[tauri::command]
pub async fn pull_from_remote(
    path: String,
    remote_name: String,
    branch: Option<String>,
) -> Result<PullResult, GitError> {
    let workdir = get_workdir(&path)?;
    remote::pull(&workdir, &remote_name, branch.as_deref()).await
}

/// Delete a remote branch
#[tauri::command]
pub async fn delete_remote_branch(
    path: String,
    remote_name: String,
    branch: String,
) -> Result<Vec<Branch>, GitError> {
    let workdir = get_workdir(&path)?;
    remote::delete_remote_branch(&workdir, &remote_name, &branch).await?;
    let repo = repository::open(&path)?;
    repository::list_branches(&repo)
}

/// Checkout a remote branch, resetting the local branch to match the remote
#[tauri::command]
pub fn checkout_remote_branch(
    path: String,
    remote_branch_name: String,
) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    remote::checkout_remote_branch(&repo, &remote_branch_name)?;
    repository::get_repo_info(&repo)
}

/// Helper to get the workdir path from a repo path
fn get_workdir(path: &str) -> Result<String, GitError> {
    let repo = repository::open(path)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))?
        .to_string_lossy()
        .to_string();
    Ok(workdir)
}
