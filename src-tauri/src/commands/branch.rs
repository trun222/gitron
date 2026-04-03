use gitron_core::git::{error::GitError, remote, repository, types::*};

/// List all branches
#[tauri::command]
pub fn list_branches(path: String) -> Result<Vec<Branch>, GitError> {
    let repo = repository::open(&path)?;
    repository::list_branches(&repo)
}

/// Create a new branch
#[tauri::command]
pub fn create_branch(
    path: String,
    name: String,
    target: Option<String>,
) -> Result<Branch, GitError> {
    let repo = repository::open(&path)?;
    let target_ref = target.as_deref().unwrap_or("HEAD");
    repository::create_branch(&repo, &name, target_ref)
}

/// Checkout a branch
#[tauri::command]
pub fn checkout_branch(path: String, name: String) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    repository::checkout_branch(&repo, &name)?;
    repository::get_repo_info(&repo)
}

/// Delete a branch
#[tauri::command]
pub fn delete_branch(path: String, name: String) -> Result<Vec<Branch>, GitError> {
    let repo = repository::open(&path)?;
    repository::delete_branch(&repo, &name)?;
    repository::list_branches(&repo)
}

/// Find branches fully merged into HEAD
#[tauri::command]
pub fn find_merged_branches(path: String) -> Result<Vec<MergedBranch>, GitError> {
    let repo = repository::open(&path)?;
    repository::find_merged_branches(&repo)
}

/// Clean up merged branches (delete local and remote)
#[tauri::command]
pub async fn cleanup_merged_branches(
    path: String,
    branches: Vec<MergedBranch>,
) -> Result<Vec<String>, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    };
    remote::cleanup_merged_branches(&workdir, &branches).await
}

/// Reset current branch to a specific commit
#[tauri::command]
pub fn reset_to_commit(path: String, commit_oid: String, reset_type: String) -> Result<RepoInfo, GitError> {
    let repo = repository::open(&path)?;
    repository::reset_to_commit(&repo, &commit_oid, &reset_type)?;
    repository::get_repo_info(&repo)
}

/// Rebase current branch onto a target branch
#[tauri::command]
pub fn rebase_onto(path: String, onto_branch: String) -> Result<RebaseResult, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    }; // repo dropped here before CLI calls
    repository::rebase_onto(&workdir, &onto_branch)
}

/// Merge a branch into the current branch
#[tauri::command]
pub fn merge_into(path: String, branch_name: String) -> Result<MergeResult, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    }; // repo dropped here before CLI calls
    repository::merge_branch(&workdir, &branch_name)
}

/// Continue a rebase in progress
#[tauri::command]
pub fn rebase_continue(path: String) -> Result<RebaseResult, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    };
    repository::rebase_continue(&workdir)
}

/// Abort a rebase in progress
#[tauri::command]
pub fn rebase_abort(path: String) -> Result<RebaseResult, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    };
    repository::rebase_abort(&workdir)
}

/// Abort a merge in progress
#[tauri::command]
pub fn merge_abort(path: String) -> Result<MergeResult, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    };
    repository::merge_abort(&workdir)
}

/// Abort a cherry-pick in progress
#[tauri::command]
pub fn cherry_pick_abort(path: String) -> Result<OperationOutput, GitError> {
    let workdir = {
        let repo = repository::open(&path)?;
        repo.workdir()
            .ok_or_else(|| GitError::Other("Bare repository".into()))?
            .to_string_lossy()
            .to_string()
    };
    repository::cherry_pick_abort(&workdir)
}
