use crate::git::{error::GitError, repository, types::*};

/// Open a repository at the given path
#[tauri::command]
pub fn open_repo(path: String) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    let info = repository::get_repo_info(&repo)?;
    Ok(info)
}

/// Get the current repository status
#[tauri::command]
pub fn get_status(path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::get_status(&repo)
}

/// Get basic repository information
#[tauri::command]
pub fn get_repo_info(path: String) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    repository::get_repo_info(&repo)
}
