use std::path::Path;

use git2::Repository;

use super::error::{GitError, GitResult};
use super::types::*;

/// Opens a git repository at the given path
pub fn open(path: &str) -> GitResult<Repository> {
    Repository::discover(path).map_err(|_| GitError::NotARepository(path.to_string()))
}

/// Check if a path is inside a valid git repository
pub fn is_valid_repo(path: &str) -> bool {
    Repository::discover(path).is_ok()
}

/// Get basic repository information
pub fn get_repo_info(repo: &Repository) -> GitResult<RepoInfo> {
    let path = repo
        .path()
        .to_string_lossy()
        .to_string();

    let workdir = repo
        .workdir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    let head_branch = repo
        .head()
        .ok()
        .and_then(|r| {
            if r.is_branch() {
                r.shorthand().map(|s| s.to_string())
            } else {
                None
            }
        });

    let head_oid = repo
        .head()
        .ok()
        .and_then(|r| r.target().map(|oid| oid.to_string()));

    Ok(RepoInfo {
        path,
        workdir,
        head_branch,
        head_oid,
        is_bare: repo.is_bare(),
        is_empty: repo.is_empty().unwrap_or(true),
    })
}

/// Get repository status (staged, unstaged, untracked files)
pub fn get_status(repo: &Repository) -> GitResult<RepoStatus> {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicted = Vec::new();

    let statuses = repo.statuses(Some(
        git2::StatusOptions::new()
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false),
    ))?;

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        // Conflicted
        if status.is_conflicted() {
            conflicted.push(path.clone());
            continue;
        }

        // Index (staged) changes
        if status.is_index_new() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Added,
            });
        } else if status.is_index_modified() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Modified,
            });
        } else if status.is_index_deleted() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Deleted,
            });
        } else if status.is_index_renamed() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Renamed,
            });
        } else if status.is_index_typechange() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::TypeChanged,
            });
        }

        // Workdir (unstaged) changes
        if status.is_wt_modified() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Modified,
            });
        } else if status.is_wt_deleted() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Deleted,
            });
        } else if status.is_wt_renamed() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Renamed,
            });
        } else if status.is_wt_typechange() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::TypeChanged,
            });
        }

        // Untracked
        if status.is_wt_new() {
            untracked.push(path);
        }
    }

    Ok(RepoStatus {
        staged,
        unstaged,
        untracked,
        conflicted,
    })
}

/// Stage a file by path
pub fn stage_file(repo: &Repository, path: &str) -> GitResult<()> {
    let mut index = repo.index()?;
    let file_path = Path::new(path);

    // Check if file exists in workdir — if not, it was deleted
    let workdir = repo.workdir().ok_or(GitError::Other("Bare repository".into()))?;
    if workdir.join(file_path).exists() {
        index.add_path(file_path)?;
    } else {
        index.remove_path(file_path)?;
    }

    index.write()?;
    Ok(())
}

/// Unstage a file by path
pub fn unstage_file(repo: &Repository, path: &str) -> GitResult<()> {
    let head = repo.head()?.peel_to_commit()?;
    repo.reset_default(Some(&head.into_object()), [path])?;
    Ok(())
}

/// Stage all changes
pub fn stage_all(repo: &Repository) -> GitResult<()> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

/// Unstage all changes
pub fn unstage_all(repo: &Repository) -> GitResult<()> {
    let head = repo.head()?.peel_to_commit()?;
    repo.reset(
        &head.into_object(),
        git2::ResetType::Mixed,
        None,
    )?;
    Ok(())
}

/// List all branches
pub fn list_branches(repo: &Repository) -> GitResult<Vec<Branch>> {
    let mut branches = Vec::new();
    let head_ref = repo.head().ok();

    for branch_result in repo.branches(None)? {
        let (branch, branch_type) = branch_result?;
        let name = branch
            .name()?
            .unwrap_or("")
            .to_string();

        let is_head = head_ref
            .as_ref()
            .and_then(|h| h.shorthand())
            .map(|h| h == name)
            .unwrap_or(false);

        let upstream = branch
            .upstream()
            .ok()
            .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

        let target_oid = branch
            .get()
            .target()
            .map(|oid| oid.to_string());

        branches.push(Branch {
            name,
            is_head,
            is_remote: branch_type == git2::BranchType::Remote,
            upstream,
            target_oid,
        });
    }

    Ok(branches)
}

/// Create a new branch
pub fn create_branch<'repo>(
    repo: &'repo Repository,
    name: &str,
    target: &str,
) -> GitResult<Branch> {
    let commit = if target == "HEAD" {
        repo.head()?.peel_to_commit()?
    } else {
        let oid = git2::Oid::from_str(target)
            .map_err(|_| GitError::CommitNotFound(target.to_string()))?;
        repo.find_commit(oid)?
    };

    let branch = repo.branch(name, &commit, false)?;
    let target_oid = branch.get().target().map(|oid| oid.to_string());

    Ok(Branch {
        name: name.to_string(),
        is_head: false,
        is_remote: false,
        upstream: None,
        target_oid,
    })
}

/// Checkout a branch by name
pub fn checkout_branch(repo: &Repository, name: &str) -> GitResult<()> {
    let refname = if name.starts_with("refs/") {
        name.to_string()
    } else {
        format!("refs/heads/{}", name)
    };

    let obj = repo
        .revparse_single(&refname)
        .map_err(|_| GitError::BranchNotFound(name.to_string()))?;

    repo.checkout_tree(&obj, None)?;
    repo.set_head(&refname)?;

    Ok(())
}

/// Delete a local branch
pub fn delete_branch(repo: &Repository, name: &str) -> GitResult<()> {
    let mut branch = repo
        .find_branch(name, git2::BranchType::Local)
        .map_err(|_| GitError::BranchNotFound(name.to_string()))?;

    branch.delete()?;
    Ok(())
}
