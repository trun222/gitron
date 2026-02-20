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

/// Stage a list of files by path
pub fn stage_files(repo: &Repository, paths: &[String]) -> GitResult<()> {
    let mut index = repo.index()?;
    let workdir = repo.workdir().ok_or(GitError::Other("Bare repository".into()))?;

    for path in paths {
        let file_path = Path::new(path);
        if workdir.join(file_path).exists() {
            index.add_path(file_path)?;
        } else {
            index.remove_path(file_path)?;
        }
    }

    index.write()?;
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

/// Helper: checkout a tree object and set HEAD to the given ref
pub fn do_checkout(repo: &Repository, refname: &str) -> GitResult<()> {
    let obj = repo
        .revparse_single(refname)
        .map_err(|_| GitError::BranchNotFound(refname.to_string()))?;
    let mut checkout = git2::build::CheckoutBuilder::new();
    checkout.safe();
    repo.checkout_tree(&obj, Some(&mut checkout))?;
    repo.set_head(refname)?;
    Ok(())
}

/// Checkout a branch by name (supports both local and remote branches)
pub fn checkout_branch(repo: &Repository, name: &str) -> GitResult<()> {
    // If it's a full ref, use it directly
    if name.starts_with("refs/") {
        return do_checkout(repo, name);
    }

    // Try local branch first
    if let Ok(branch) = repo.find_branch(name, git2::BranchType::Local) {
        let refname = branch
            .get()
            .name()
            .ok_or_else(|| GitError::BranchNotFound(name.to_string()))?
            .to_string();
        return do_checkout(repo, &refname);
    }

    // Try remote branch — find one matching the name (e.g., "origin/feature" or just "feature")
    let remote_branch = if name.contains('/') {
        // Full remote branch name like "origin/feature"
        repo.find_branch(name, git2::BranchType::Remote).ok()
    } else {
        // Short name like "feature" — look for any remote with this branch
        repo.branches(Some(git2::BranchType::Remote))
            .ok()
            .and_then(|mut branches| {
                branches.find_map(|b| {
                    let (branch, _) = b.ok()?;
                    let branch_name = branch.name().ok()??;
                    // Match "origin/feature" when searching for "feature"
                    let short = branch_name.split('/').last()?;
                    if short == name {
                        Some(branch)
                    } else {
                        None
                    }
                })
            })
    };

    if let Some(remote_branch) = remote_branch {
        let remote_full = remote_branch
            .name()?
            .unwrap_or("")
            .to_string();

        // Extract local name: "origin/feature" -> "feature"
        let local_name = if name.contains('/') {
            name.splitn(2, '/').last().unwrap_or(name)
        } else {
            name
        };

        // If local branch already exists, just checkout it
        if repo.find_branch(local_name, git2::BranchType::Local).is_ok() {
            let local_refname = format!("refs/heads/{}", local_name);
            return do_checkout(repo, &local_refname);
        }

        // Create local tracking branch from the remote commit
        let commit = remote_branch.get().peel_to_commit()?;
        let mut local_branch = repo.branch(local_name, &commit, false)?;

        // Set upstream tracking
        local_branch.set_upstream(Some(&remote_full))?;

        // Checkout the new local branch
        let local_refname = format!("refs/heads/{}", local_name);
        return do_checkout(repo, &local_refname);
    }

    Err(GitError::BranchNotFound(name.to_string()))
}

/// Delete a local branch
pub fn delete_branch(repo: &Repository, name: &str) -> GitResult<()> {
    let mut branch = repo
        .find_branch(name, git2::BranchType::Local)
        .map_err(|_| GitError::BranchNotFound(name.to_string()))?;

    branch.delete()?;
    Ok(())
}

/// Reset current branch to a specific commit
pub fn reset_to_commit(repo: &Repository, commit_oid: &str, reset_type: &str) -> GitResult<()> {
    let oid = git2::Oid::from_str(commit_oid)
        .map_err(|_| GitError::CommitNotFound(commit_oid.to_string()))?;
    let commit = repo
        .find_commit(oid)
        .map_err(|_| GitError::CommitNotFound(commit_oid.to_string()))?;
    let rt = match reset_type {
        "soft" => git2::ResetType::Soft,
        "mixed" => git2::ResetType::Mixed,
        "hard" => git2::ResetType::Hard,
        _ => return Err(GitError::Other(format!("Invalid reset type: {}", reset_type))),
    };
    repo.reset(commit.as_object(), rt, None)?;
    Ok(())
}

/// Apply a stash by index (does not remove it)
pub fn apply_stash(repo: &mut Repository, index: usize) -> GitResult<()> {
    repo.stash_apply(index, None)
        .map_err(|e| GitError::Other(format!("Failed to apply stash: {}", e)))
}

/// Pop a stash by index (applies and removes it)
pub fn pop_stash(repo: &mut Repository, index: usize) -> GitResult<()> {
    repo.stash_pop(index, None)
        .map_err(|e| GitError::Other(format!("Failed to pop stash: {}", e)))
}

/// Drop a stash by index (removes without applying)
pub fn drop_stash(repo: &mut Repository, index: usize) -> GitResult<()> {
    repo.stash_drop(index)
        .map_err(|e| GitError::Other(format!("Failed to drop stash: {}", e)))
}

/// Discard all changes: reset index to HEAD, checkout HEAD (overwrite tracked files),
/// and clean untracked files/dirs via CLI.
pub fn discard_all_changes(repo: &Repository) -> GitResult<()> {
    let head = repo.head()?.peel_to_commit()?;

    // Hard reset: resets index and working directory to HEAD
    repo.reset(
        head.as_object(),
        git2::ResetType::Hard,
        None,
    )?;

    // Clean untracked files and directories
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Bare repository".into()))?
        .to_string_lossy()
        .to_string();

    super::cli::run_git(&workdir, &["clean", "-fd"])?;

    Ok(())
}

/// Clone a repository from a URL to a destination path (uses CLI for network ops)
pub async fn clone_repo(url: &str, dest: &str) -> GitResult<CloneResult> {
    let dest_path = std::path::Path::new(dest);
    let parent = dest_path
        .parent()
        .ok_or_else(|| GitError::Other("Invalid destination path".into()))?;

    let parent_str = parent.to_string_lossy().to_string();

    // Use the dest folder name as the target (git clone creates it)
    let folder_name = dest_path
        .file_name()
        .ok_or_else(|| GitError::Other("Invalid destination path".into()))?
        .to_string_lossy()
        .to_string();

    let output = super::cli::run_git_async_with_github_auth(
        &parent_str,
        &["clone", url, &folder_name],
    )
    .await?;

    // Open the cloned repo and get info
    let repo = open(dest)?;
    let repo_info = get_repo_info(&repo)?;

    Ok(CloneResult {
        path: dest.to_string(),
        repo_info,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Rebase current branch onto the given branch (uses CLI)
pub fn rebase_onto(workdir: &str, onto_branch: &str) -> GitResult<RebaseResult> {
    let output = super::cli::run_git_raw(workdir, &["rebase", onto_branch])?;

    let conflicted = output.exit_code != 0
        && (output.stderr.contains("CONFLICT") || output.stdout.contains("CONFLICT"));

    Ok(RebaseResult {
        success: output.exit_code == 0,
        conflicted,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Merge source_branch into target_branch (checks out target, merges source, uses CLI)
pub fn merge_branch_into(workdir: &str, source_branch: &str, target_branch: &str) -> GitResult<MergeResult> {
    // First checkout the target branch
    let checkout_output = super::cli::run_git_raw(workdir, &["checkout", target_branch])?;
    if checkout_output.exit_code != 0 {
        return Ok(MergeResult {
            success: false,
            conflicted: false,
            output: OperationOutput {
                stdout: checkout_output.stdout,
                stderr: checkout_output.stderr,
            },
        });
    }

    // Then merge the source branch
    let merge_output = super::cli::run_git_raw(workdir, &["merge", source_branch])?;

    let conflicted = merge_output.exit_code != 0
        && (merge_output.stderr.contains("CONFLICT") || merge_output.stdout.contains("CONFLICT"));

    Ok(MergeResult {
        success: merge_output.exit_code == 0,
        conflicted,
        output: OperationOutput {
            stdout: merge_output.stdout,
            stderr: merge_output.stderr,
        },
    })
}

/// Create a commit with the current index using git CLI (runs hooks)
pub fn create_commit(workdir: &str, message: &str) -> GitResult<CommitResult> {
    let output = super::cli::run_git_raw(workdir, &["commit", "-m", message])?;

    if output.exit_code == 0 {
        // Parse OID from stdout, format: "[branch abc1234] message"
        let oid = output
            .stdout
            .lines()
            .next()
            .and_then(|line| {
                let start = line.find(' ')? + 1;
                let end = line.find(']')?;
                Some(line[start..end].to_string())
            })
            .unwrap_or_default();

        Ok(CommitResult {
            oid,
            success: true,
            output: OperationOutput {
                stdout: output.stdout,
                stderr: output.stderr,
            },
        })
    } else {
        Ok(CommitResult {
            oid: String::new(),
            success: false,
            output: OperationOutput {
                stdout: output.stdout,
                stderr: output.stderr,
            },
        })
    }
}
