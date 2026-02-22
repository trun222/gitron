use gitron_core::git::{error::GitError, repository, types::*};

/// Stage a file
#[tauri::command]
pub fn stage_file(path: String, file_path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::stage_file(&repo, &file_path)?;
    repository::get_status(&repo)
}

/// Unstage a file
#[tauri::command]
pub fn unstage_file(path: String, file_path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::unstage_file(&repo, &file_path)?;
    repository::get_status(&repo)
}

/// Stage a list of files
#[tauri::command]
pub fn stage_files(path: String, file_paths: Vec<String>) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::stage_files(&repo, &file_paths)?;
    repository::get_status(&repo)
}

/// Stage all changes
#[tauri::command]
pub fn stage_all(path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::stage_all(&repo)?;
    repository::get_status(&repo)
}

/// Unstage all changes
#[tauri::command]
pub fn unstage_all(path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::unstage_all(&repo)?;
    repository::get_status(&repo)
}

/// Discard all changes (staged, unstaged, and untracked)
#[tauri::command]
pub fn discard_all_changes(path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::discard_all_changes(&repo)?;
    repository::get_status(&repo)
}
