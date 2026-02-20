use crate::git::{error::GitError, remote, repository, types::*};

/// Create a tag at a specific commit
#[tauri::command]
pub fn create_tag(
    path: String,
    name: String,
    target_oid: String,
    message: Option<String>,
) -> Result<Tag, GitError> {
    let repo = repository::open(&path)?;
    repository::create_tag(&repo, &name, &target_oid, message.as_deref())
}

/// Delete a local tag
#[tauri::command]
pub fn delete_tag(path: String, name: String) -> Result<(), GitError> {
    let repo = repository::open(&path)?;
    repository::delete_tag(&repo, &name)
}

/// Push a tag to a remote
#[tauri::command]
pub async fn push_tag(
    path: String,
    remote_name: String,
    tag_name: String,
) -> Result<PushResult, GitError> {
    let workdir = get_workdir(&path)?;
    remote::push_tag(&workdir, &remote_name, &tag_name).await
}

/// Delete a remote tag
#[tauri::command]
pub async fn delete_remote_tag(
    path: String,
    remote_name: String,
    tag_name: String,
) -> Result<(), GitError> {
    let workdir = get_workdir(&path)?;
    remote::delete_remote_tag(&workdir, &remote_name, &tag_name).await
}

/// List tag names that exist on a remote
#[tauri::command]
pub async fn list_remote_tags(
    path: String,
    remote_name: String,
) -> Result<Vec<String>, GitError> {
    let workdir = get_workdir(&path)?;
    remote::list_remote_tags(&workdir, &remote_name).await
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
