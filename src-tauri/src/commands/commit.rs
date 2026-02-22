use gitron_core::git::{error::GitError, repository, types::CommitResult};

/// Create a commit with staged changes (runs git hooks)
#[tauri::command]
pub fn create_commit(path: String, message: String) -> Result<CommitResult, GitError> {
    let repo = repository::open(&path)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))?
        .to_string_lossy()
        .to_string();
    repository::create_commit(&workdir, &message)
}
