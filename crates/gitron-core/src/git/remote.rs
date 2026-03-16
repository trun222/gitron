use git2::Repository;

use super::cli;
use super::error::{GitError, GitResult};
use super::repository;
use super::types::*;

/// List all configured remotes
pub fn list_remotes(repo: &Repository) -> GitResult<Vec<Remote>> {
    let remote_names = repo.remotes()?;
    let mut remotes = Vec::new();

    for name in remote_names.iter().flatten() {
        let remote = repo.find_remote(name)?;
        remotes.push(Remote {
            name: name.to_string(),
            url: remote.url().unwrap_or("").to_string(),
            push_url: remote.pushurl().map(|s| s.to_string()),
        });
    }

    Ok(remotes)
}

/// Add a new remote
pub fn add_remote(repo: &Repository, name: &str, url: &str) -> GitResult<Remote> {
    let remote = repo.remote(name, url)?;
    Ok(Remote {
        name: name.to_string(),
        url: remote.url().unwrap_or(url).to_string(),
        push_url: remote.pushurl().map(|s| s.to_string()),
    })
}

/// Remove a remote
pub fn remove_remote(repo: &Repository, name: &str) -> GitResult<()> {
    repo.remote_delete(name)?;
    Ok(())
}

/// Get tracking status (ahead/behind) for a branch
pub fn get_tracking_status(repo: &Repository, branch_name: &str) -> GitResult<TrackingStatus> {
    let branch = repo
        .find_branch(branch_name, git2::BranchType::Local)
        .map_err(|_| GitError::BranchNotFound(branch_name.to_string()))?;

    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(_) => {
            return Ok(TrackingStatus {
                ahead: 0,
                behind: 0,
                upstream: None,
            });
        }
    };

    let upstream_name = upstream
        .name()?
        .unwrap_or("")
        .to_string();

    let local_oid = branch
        .get()
        .target()
        .ok_or_else(|| GitError::BranchNotFound(branch_name.to_string()))?;

    let upstream_oid = upstream
        .get()
        .target()
        .ok_or_else(|| GitError::BranchNotFound(upstream_name.clone()))?;

    let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;

    Ok(TrackingStatus {
        ahead,
        behind,
        upstream: Some(upstream_name),
    })
}

/// Fetch from a specific remote (optionally a specific branch)
pub async fn fetch(workdir: &str, remote: &str, branch: Option<&str>) -> GitResult<FetchResult> {
    let mut args = vec!["fetch", remote];
    if let Some(b) = branch {
        args.push(b);
    }
    args.push("--prune");

    let output = cli::run_git_async_with_github_auth(workdir, &args).await?;

    let updated_refs: Vec<String> = output
        .stderr
        .lines()
        .filter(|line| line.contains("->"))
        .map(|line| line.trim().to_string())
        .collect();

    let summary = if updated_refs.is_empty() {
        "Already up to date".to_string()
    } else {
        format!("Updated {} ref(s)", updated_refs.len())
    };

    Ok(FetchResult {
        remote: remote.to_string(),
        updated_refs,
        summary,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Fetch from all remotes
pub async fn fetch_all(workdir: &str) -> GitResult<FetchResult> {
    let output = cli::run_git_async_with_github_auth(workdir, &["fetch", "--all", "--prune"]).await?;

    let updated_refs: Vec<String> = output
        .stderr
        .lines()
        .filter(|line| line.contains("->"))
        .map(|line| line.trim().to_string())
        .collect();

    let summary = if updated_refs.is_empty() {
        "Already up to date".to_string()
    } else {
        format!("Updated {} ref(s)", updated_refs.len())
    };

    Ok(FetchResult {
        remote: "--all".to_string(),
        updated_refs,
        summary,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Push to a remote
pub async fn push(
    workdir: &str,
    remote: &str,
    branch: Option<&str>,
    force: bool,
    set_upstream: bool,
) -> GitResult<PushResult> {
    let mut args = vec!["push", remote];

    let branch_name = branch.unwrap_or("").to_string();
    if !branch_name.is_empty() {
        args.push(&branch_name);
    }

    if force {
        args.push("--force-with-lease");
    }
    if set_upstream {
        args.push("--set-upstream");
    }

    // Add --verbose for richer diagnostics on success and failure
    args.push("--verbose");

    match cli::run_git_async_with_github_auth(workdir, &args).await {
        Ok(output) => {
            let summary = if output.stderr.contains("Everything up-to-date") {
                "Everything up-to-date".to_string()
            } else {
                output
                    .stderr
                    .lines()
                    .find(|line| line.contains("->"))
                    .unwrap_or("Push completed")
                    .trim()
                    .to_string()
            };

            Ok(PushResult {
                remote: remote.to_string(),
                branch: branch.unwrap_or("HEAD").to_string(),
                summary,
                output: OperationOutput {
                    stdout: output.stdout,
                    stderr: output.stderr,
                },
            })
        }
        Err(GitError::CliError {
            command,
            stderr,
            exit_code,
        }) => {
            // Build a richer error message combining all available info
            let mut details = String::new();
            for line in stderr.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if !details.is_empty() {
                        details.push('\n');
                    }
                    details.push_str(trimmed);
                }
            }
            Err(GitError::CliError {
                command,
                stderr: details,
                exit_code,
            })
        }
        Err(e) => Err(e),
    }
}

/// Pull from a remote
///
/// First fetches all remotes so every branch is up to date, then merges
/// the current branch's upstream (via `git pull`).
pub async fn pull(
    workdir: &str,
    remote: &str,
    branch: Option<&str>,
) -> GitResult<PullResult> {
    // Fetch all remotes first so the graph reflects every branch's latest state
    let fetch_output = cli::run_git_async_with_github_auth(workdir, &["fetch", "--all", "--prune"]).await;
    // Collect fetch stderr/stdout to include in the result; don't fail the pull if fetch fails
    let (fetch_stdout, fetch_stderr) = match &fetch_output {
        Ok(o) => (o.stdout.clone(), o.stderr.clone()),
        Err(_) => (String::new(), String::new()),
    };

    let mut args = vec!["pull", remote];

    let branch_name = branch.unwrap_or("").to_string();
    if !branch_name.is_empty() {
        args.push(&branch_name);
    }

    let result = cli::run_git_async_with_github_auth(workdir, &args).await;

    match result {
        Ok(output) => {
            let merge_conflicts = output.stdout.contains("CONFLICT")
                || output.stderr.contains("CONFLICT");

            let summary = if output.stdout.contains("Already up to date") {
                "Already up to date".to_string()
            } else {
                output
                    .stdout
                    .lines()
                    .last()
                    .unwrap_or("Pull completed")
                    .trim()
                    .to_string()
            };

            // Combine fetch + pull output so the user sees the full picture
            let combined_stdout = if fetch_stdout.is_empty() {
                output.stdout
            } else {
                format!("{}\n{}", fetch_stdout, output.stdout)
            };
            let combined_stderr = if fetch_stderr.is_empty() {
                output.stderr
            } else {
                format!("{}\n{}", fetch_stderr, output.stderr)
            };

            Ok(PullResult {
                remote: remote.to_string(),
                branch: branch.unwrap_or("HEAD").to_string(),
                summary,
                merge_conflicts,
                output: OperationOutput {
                    stdout: combined_stdout,
                    stderr: combined_stderr,
                },
            })
        }
        Err(GitError::CliError { stderr, .. }) if stderr.contains("CONFLICT") => {
            let combined_stderr = if fetch_stderr.is_empty() {
                stderr
            } else {
                format!("{}\n{}", fetch_stderr, stderr)
            };
            Ok(PullResult {
                remote: remote.to_string(),
                branch: branch.unwrap_or("HEAD").to_string(),
                summary: "Pull completed with merge conflicts".to_string(),
                merge_conflicts: true,
                output: OperationOutput {
                    stdout: fetch_stdout,
                    stderr: combined_stderr,
                },
            })
        }
        Err(e) => Err(e),
    }
}

/// Push a tag to a remote
pub async fn push_tag(workdir: &str, remote: &str, tag_name: &str, force: bool) -> GitResult<PushResult> {
    let refspec = format!("refs/tags/{}", tag_name);
    let mut args = vec!["push"];
    if force {
        args.push("--force");
    }
    args.push(remote);
    args.push(&refspec);
    let output = cli::run_git_async_with_github_auth(workdir, &args).await?;

    let summary = if output.stderr.contains("Everything up-to-date") {
        "Tag already up-to-date".to_string()
    } else {
        output
            .stderr
            .lines()
            .find(|line| line.contains("->"))
            .unwrap_or("Tag pushed")
            .trim()
            .to_string()
    };

    Ok(PushResult {
        remote: remote.to_string(),
        branch: refspec,
        summary,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Delete a remote tag via `git push --delete`
pub async fn delete_remote_tag(workdir: &str, remote: &str, tag_name: &str) -> GitResult<()> {
    let refspec = format!("refs/tags/{}", tag_name);
    cli::run_git_async_with_github_auth(workdir, &["push", remote, "--delete", &refspec]).await?;
    Ok(())
}

/// List tags that exist on a remote via `git ls-remote --tags` (returns name + OID)
pub async fn list_remote_tags(workdir: &str, remote: &str) -> GitResult<Vec<RemoteTagInfo>> {
    let output =
        cli::run_git_async_with_github_auth(workdir, &["ls-remote", "--tags", remote]).await?;

    let tags: Vec<RemoteTagInfo> = output
        .stdout
        .lines()
        .filter_map(|line| {
            // Format: "<sha>\trefs/tags/<name>"
            let mut parts = line.split('\t');
            let oid = parts.next()?.trim().to_string();
            let refname = parts.next()?;
            // Skip dereferenced entries (e.g. refs/tags/v1.0^{})
            if refname.ends_with("^{}") {
                return None;
            }
            let name = refname.strip_prefix("refs/tags/")?.to_string();
            Some(RemoteTagInfo { name, oid })
        })
        .collect();

    Ok(tags)
}

/// Delete a remote branch via `git push --delete`
pub async fn delete_remote_branch(workdir: &str, remote: &str, branch: &str) -> GitResult<()> {
    cli::run_git_async_with_github_auth(workdir, &["push", remote, "--delete", branch]).await?;
    Ok(())
}

/// Reset a local branch to match a remote branch, then checkout it.
/// If the local branch doesn't exist, creates it as a tracking branch.
pub fn checkout_remote_branch(repo: &Repository, remote_branch_name: &str) -> GitResult<()> {
    let remote_branch = repo
        .find_branch(remote_branch_name, git2::BranchType::Remote)
        .map_err(|_| GitError::BranchNotFound(remote_branch_name.to_string()))?;

    let remote_full = remote_branch
        .name()?
        .unwrap_or("")
        .to_string();

    // Extract local name: "origin/feature" -> "feature"
    let local_name = remote_branch_name
        .splitn(2, '/')
        .last()
        .unwrap_or(remote_branch_name);

    let remote_commit = remote_branch.get().peel_to_commit()?;

    // Delete existing local branch if it exists (so we can recreate it)
    if let Ok(mut existing) = repo.find_branch(local_name, git2::BranchType::Local) {
        // If it's the current HEAD, detach first
        if existing.is_head() {
            repo.set_head_detached(remote_commit.id())?;
        }
        existing.delete()?;
    }

    // Create fresh local tracking branch at the remote's commit
    let mut local_branch = repo.branch(local_name, &remote_commit, false)?;
    local_branch.set_upstream(Some(&remote_full))?;

    // Checkout
    repository::do_checkout(repo, &format!("refs/heads/{}", local_name))
}
