use gitron_core::git::{diff as git_diff, error::GitError, repository, types::*};

/// Get working directory diff (unstaged changes)
#[tauri::command]
pub fn get_workdir_diff(path: String) -> Result<Vec<FileDiff>, GitError> {
    let repo = repository::open(&path)?;
    git_diff::diff_workdir(&repo)
}

/// Get diff for a specific file
#[tauri::command]
pub fn get_file_diff(path: String, file_path: String) -> Result<FileDiff, GitError> {
    let repo = repository::open(&path)?;
    git_diff::diff_file(&repo, &file_path)
}

/// Get diff for a specific staged file
#[tauri::command]
pub fn get_staged_file_diff(path: String, file_path: String) -> Result<FileDiff, GitError> {
    let repo = repository::open(&path)?;
    git_diff::diff_file_staged(&repo, &file_path)
}

/// Get diff for a commit (vs its first parent)
#[tauri::command]
pub fn get_commit_diff(path: String, oid: String) -> Result<Vec<FileDiff>, GitError> {
    let repo = repository::open(&path)?;
    git_diff::diff_commit(&repo, &oid)
}
