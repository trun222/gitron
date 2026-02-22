use gitron_core::git::{error::GitError, repository, types::*};

/// Apply a stash (does not remove it)
#[tauri::command]
pub fn apply_stash(path: String, index: usize) -> Result<RepoStatus, GitError> {
    let mut repo = repository::open(&path)?;
    repository::apply_stash(&mut repo, index)?;
    repository::get_status(&repo)
}

/// Pop a stash (applies and removes it)
#[tauri::command]
pub fn pop_stash(path: String, index: usize) -> Result<RepoStatus, GitError> {
    let mut repo = repository::open(&path)?;
    repository::pop_stash(&mut repo, index)?;
    repository::get_status(&repo)
}

/// Drop a stash (removes without applying)
#[tauri::command]
pub fn drop_stash(path: String, index: usize) -> Result<RepoStatus, GitError> {
    let mut repo = repository::open(&path)?;
    repository::drop_stash(&mut repo, index)?;
    repository::get_status(&repo)
}
