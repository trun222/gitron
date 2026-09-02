use git2::{DiffOptions, Repository};

use super::error::GitResult;
use super::types::{CommitRangeEntry, CommitRangeSummary};

/// Summarize the commits and file changes between two revisions.
///
/// `from` is exclusive and `to` is inclusive, matching `git log from..to`.
/// Both accept anything `git rev-parse` understands (tag, branch, SHA, `HEAD~3`).
pub fn summarize_range(repo: &Repository, from: &str, to: &str) -> GitResult<CommitRangeSummary> {
    let from_commit = repo.revparse_single(from)?.peel_to_commit()?;
    let to_commit = repo.revparse_single(to)?.peel_to_commit()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
    revwalk.push(to_commit.id())?;
    revwalk.hide(from_commit.id())?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        let oid = commit.id().to_string();
        commits.push(CommitRangeEntry {
            short_oid: oid[..7.min(oid.len())].to_string(),
            oid,
            summary: commit.summary().unwrap_or("").to_string(),
            body: commit.body().unwrap_or("").trim().to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            is_merge: commit.parent_count() > 1,
        });
    }

    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(
        Some(&from_commit.tree()?),
        Some(&to_commit.tree()?),
        Some(&mut opts),
    )?;
    let stats = diff.stats()?;

    let mut files = Vec::new();
    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string());
        if let Some(path) = path {
            files.push(path);
        }
    }

    Ok(CommitRangeSummary {
        from_oid: from_commit.id().to_string(),
        to_oid: to_commit.id().to_string(),
        commits,
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
        files,
    })
}
