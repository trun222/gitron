use crate::git::{error::GitError, repository};

/// Create a commit with staged changes
#[tauri::command]
pub fn create_commit(path: String, message: String) -> Result<String, GitError> {
    let repo = repository::open(&path)?;
    repository::create_commit(&repo, &message)
}
