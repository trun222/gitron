use gitron_core::git::{error::GitError, repository, types::*};

/// Find all checkpoint refs created by AI coding tools
#[tauri::command]
pub fn find_checkpoint_refs(path: String) -> Result<Vec<CheckpointRef>, GitError> {
    let repo = repository::open(&path)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))?
        .to_string_lossy()
        .to_string();
    repository::find_checkpoint_refs(&workdir)
}

/// Purge checkpoint refs and garbage collect
#[tauri::command]
pub fn purge_checkpoint_refs(path: String, refs: Vec<CheckpointRef>) -> Result<usize, GitError> {
    let repo = repository::open(&path)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))?
        .to_string_lossy()
        .to_string();
    repository::purge_checkpoint_refs(&workdir, &refs)
}
