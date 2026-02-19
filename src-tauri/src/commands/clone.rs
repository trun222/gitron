use crate::git::{error::GitError, repository, types::CloneResult};

/// Clone a repository from a URL to a destination path
#[tauri::command]
pub async fn clone_repo(url: String, dest: String) -> Result<CloneResult, GitError> {
    repository::clone_repo(&url, &dest).await
}
