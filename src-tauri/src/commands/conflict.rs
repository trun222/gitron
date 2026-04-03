use gitron_core::git::{error::GitError, repository, types::*};

/// Get the content of a conflicted file with parsed conflict markers
#[tauri::command]
pub fn get_conflicted_file(
    path: String,
    file_path: String,
) -> Result<ConflictedFileContent, GitError> {
    let repo = repository::open(&path)?;
    repository::get_conflicted_file(&repo, &file_path)
}

/// Write resolved content back to a file and stage it
#[tauri::command]
pub fn write_resolved_file(
    path: String,
    file_path: String,
    content: String,
) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::write_file_content(&repo, &file_path, &content)?;
    repository::stage_file(&repo, &file_path)?;
    repository::get_status(&repo)
}
