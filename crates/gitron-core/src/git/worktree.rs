use super::cli::run_git;
use super::error::{GitError, GitResult};
use super::types::*;

/// List all worktrees for the repository at `workdir`.
///
/// Uses `git worktree list --porcelain` which outputs blocks like:
///   worktree /path/to/worktree
///   HEAD abc123...
///   branch refs/heads/feature-a
///   <blank line>
///
/// The main worktree is always listed first.
pub fn list_worktrees(workdir: &str) -> GitResult<Vec<WorktreeInfo>> {
    let output = run_git(workdir, &["worktree", "list", "--porcelain"])?;
    Ok(parse_worktree_list(&output.stdout))
}

/// Create a new worktree.
///
/// - `path`: Where to create the worktree (absolute or relative to workdir)
/// - `branch`: Branch to check out. If None, creates a detached HEAD at HEAD.
/// - `new_branch`: If true, creates a new branch with the given name.
///
/// Maps to: `git worktree add [-b <branch>] <path> [<commit-ish>]`
pub fn add_worktree(
    workdir: &str,
    path: &str,
    branch: Option<&str>,
    new_branch: bool,
) -> GitResult<WorktreeCreateResult> {
    let mut args = vec!["worktree", "add"];

    if let Some(b) = branch {
        if new_branch {
            args.extend(&["-b", b, path]);
        } else {
            args.extend(&[path, b]);
        }
    } else {
        args.extend(&["--detach", path]);
    }

    let output = run_git(workdir, &args)?;

    // Find the newly created worktree by matching path
    let worktrees = list_worktrees(workdir)?;
    let abs_path = std::path::Path::new(workdir).join(path);
    let canonical = abs_path
        .canonicalize()
        .unwrap_or_else(|_| abs_path.to_path_buf());
    let canonical_str = canonical.to_string_lossy().to_string();

    let worktree = worktrees
        .into_iter()
        .find(|w| w.path == canonical_str || w.path == path)
        .ok_or_else(|| GitError::Other("Worktree created but not found in list".into()))?;

    Ok(WorktreeCreateResult {
        worktree,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Remove a worktree.
///
/// - `force`: If true, removes even if there are uncommitted changes.
///
/// Maps to: `git worktree remove [--force] <worktree>`
pub fn remove_worktree(
    workdir: &str,
    worktree_path: &str,
    force: bool,
) -> GitResult<WorktreeRemoveResult> {
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree_path);

    let result = run_git(workdir, &args);
    match result {
        Ok(output) => Ok(WorktreeRemoveResult {
            success: true,
            output: OperationOutput {
                stdout: output.stdout,
                stderr: output.stderr,
            },
        }),
        Err(GitError::CliError {
            stderr,
            exit_code,
            ..
        }) => Ok(WorktreeRemoveResult {
            success: false,
            output: OperationOutput {
                stdout: String::new(),
                stderr: format!("exit code {exit_code}: {stderr}"),
            },
        }),
        Err(e) => Err(e),
    }
}

/// Lock a worktree to prevent pruning.
///
/// Maps to: `git worktree lock [--reason <reason>] <worktree>`
pub fn lock_worktree(
    workdir: &str,
    worktree_path: &str,
    reason: Option<&str>,
) -> GitResult<()> {
    let mut args = vec!["worktree", "lock"];
    if let Some(r) = reason {
        args.extend(&["--reason", r]);
    }
    args.push(worktree_path);
    run_git(workdir, &args)?;
    Ok(())
}

/// Unlock a worktree.
///
/// Maps to: `git worktree unlock <worktree>`
pub fn unlock_worktree(workdir: &str, worktree_path: &str) -> GitResult<()> {
    run_git(workdir, &["worktree", "unlock", worktree_path])?;
    Ok(())
}

/// Prune stale worktree references.
///
/// Maps to: `git worktree prune [--dry-run] --verbose`
pub fn prune_worktrees(workdir: &str, dry_run: bool) -> GitResult<WorktreePruneResult> {
    let mut args = vec!["worktree", "prune", "--verbose"];
    if dry_run {
        args.push("--dry-run");
    }
    let output = run_git(workdir, &args)?;
    let pruned = output
        .stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    Ok(WorktreePruneResult {
        pruned,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Parse the porcelain output of `git worktree list --porcelain`.
///
/// Format (blocks separated by blank lines):
///   worktree /absolute/path
///   HEAD <oid>
///   branch refs/heads/<name>   (or "detached" for detached HEAD)
///   locked [<reason>]          (only if locked)
///   prunable                   (only if prunable)
fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut is_first = true;

    for block in output.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let mut path = String::new();
        let mut head_oid = None;
        let mut branch = None;
        let mut is_locked = false;
        let mut lock_reason = None;

        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = p.to_string();
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                head_oid = Some(h.to_string());
            } else if let Some(b) = line.strip_prefix("branch ") {
                branch = Some(
                    b.strip_prefix("refs/heads/")
                        .unwrap_or(b)
                        .to_string(),
                );
            } else if line == "detached" {
                branch = None;
            } else if line == "locked" {
                is_locked = true;
            } else if let Some(reason) = line.strip_prefix("locked ") {
                is_locked = true;
                lock_reason = Some(reason.to_string());
            }
        }

        if path.is_empty() {
            continue;
        }

        let is_valid = std::path::Path::new(&path).exists();
        let head_short_oid = head_oid
            .as_ref()
            .map(|h| h.chars().take(7).collect::<String>());
        let name = if is_first {
            "main".to_string()
        } else {
            std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone())
        };

        worktrees.push(WorktreeInfo {
            name,
            path,
            branch,
            head_oid,
            head_short_oid,
            is_main: is_first,
            is_locked,
            lock_reason,
            is_valid,
        });

        is_first = false;
    }

    worktrees
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list_single() {
        let output = "worktree /home/user/repo\nHEAD abc1234567890\nbranch refs/heads/main\n\n";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "main");
        assert_eq!(result[0].path, "/home/user/repo");
        assert_eq!(result[0].branch, Some("main".to_string()));
        assert_eq!(
            result[0].head_oid,
            Some("abc1234567890".to_string())
        );
        assert_eq!(
            result[0].head_short_oid,
            Some("abc1234".to_string())
        );
        assert!(result[0].is_main);
        assert!(!result[0].is_locked);
    }

    #[test]
    fn test_parse_worktree_list_multiple() {
        let output = "\
worktree /home/user/repo
HEAD abc1234567890
branch refs/heads/main

worktree /home/user/repo-feature
HEAD def5678901234
branch refs/heads/feat/auth

";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 2);
        assert!(result[0].is_main);
        assert_eq!(result[0].branch, Some("main".to_string()));
        assert!(!result[1].is_main);
        assert_eq!(result[1].name, "repo-feature");
        assert_eq!(result[1].branch, Some("feat/auth".to_string()));
    }

    #[test]
    fn test_parse_worktree_list_detached() {
        let output = "\
worktree /home/user/repo
HEAD abc1234567890
branch refs/heads/main

worktree /home/user/repo-detached
HEAD def5678901234
detached

";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 2);
        assert_eq!(result[1].branch, None);
    }

    #[test]
    fn test_parse_worktree_list_locked() {
        let output = "\
worktree /home/user/repo
HEAD abc1234567890
branch refs/heads/main

worktree /home/user/repo-locked
HEAD def5678901234
branch refs/heads/feat/locked
locked AI agent working

";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 2);
        assert!(result[1].is_locked);
        assert_eq!(
            result[1].lock_reason,
            Some("AI agent working".to_string())
        );
    }

    #[test]
    fn test_parse_worktree_list_empty() {
        let result = parse_worktree_list("");
        assert!(result.is_empty());
    }
}
