use chrono::{TimeZone, Utc};
use git2::Repository;

use super::error::GitResult;
use super::types::*;

/// Build the commit graph for visualization
pub fn build_commit_graph(repo: &Repository, options: &GraphOptions) -> GitResult<CommitGraph> {
    let max_commits = options.max_commits.unwrap_or(500);
    let mut commits = Vec::new();

    // Set up the revwalk
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    // Start from all references to get the full graph
    if let Some(ref from_oid) = options.from_oid {
        let oid = git2::Oid::from_str(from_oid)?;
        revwalk.push(oid)?;
    } else {
        revwalk.push_head().ok(); // May fail on empty repo

        if options.include_remotes {
            // Include all branch tips for a complete graph
            if let Ok(references) = repo.references() {
                for reference in references.flatten() {
                    if let Some(oid) = reference.target() {
                        revwalk.push(oid).ok();
                    }
                }
            }
        }
    }

    // Walk commits
    for oid_result in revwalk {
        if commits.len() >= max_commits {
            break;
        }

        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit_to_type(&commit));
    }

    // Collect branches
    let branches = super::repository::list_branches(repo)?;

    // Collect tags
    let tags = collect_tags(repo)?;

    // HEAD info
    let head_oid = repo
        .head()
        .ok()
        .and_then(|r| r.target().map(|o| o.to_string()));

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

    Ok(CommitGraph {
        commits,
        branches,
        tags,
        head_oid,
        head_branch,
    })
}

/// Get detailed information about a single commit
pub fn get_commit_detail(repo: &Repository, oid_str: &str) -> GitResult<Commit> {
    let oid = git2::Oid::from_str(oid_str)?;
    let commit = repo.find_commit(oid)?;
    Ok(commit_to_type(&commit))
}

/// Convert a git2 commit to our Commit type
fn commit_to_type(commit: &git2::Commit) -> Commit {
    let oid = commit.id().to_string();
    let short_oid = oid[..7.min(oid.len())].to_string();

    let message = commit
        .message()
        .unwrap_or("")
        .to_string();

    let summary = commit
        .summary()
        .unwrap_or("")
        .to_string();

    let author = Signature {
        name: commit.author().name().unwrap_or("").to_string(),
        email: commit.author().email().unwrap_or("").to_string(),
    };

    let committer = Signature {
        name: commit.committer().name().unwrap_or("").to_string(),
        email: commit.committer().email().unwrap_or("").to_string(),
    };

    let parents = commit
        .parent_ids()
        .map(|oid| oid.to_string())
        .collect();

    let time = commit.time();
    let timestamp = Utc
        .timestamp_opt(time.seconds(), 0)
        .single()
        .unwrap_or_else(Utc::now);

    Commit {
        oid,
        short_oid,
        message,
        summary,
        author,
        committer,
        parents,
        timestamp,
    }
}

/// Collect all tags in the repository
fn collect_tags(repo: &Repository) -> GitResult<Vec<Tag>> {
    let mut tags = Vec::new();

    repo.tag_foreach(|oid, name| {
        let name = String::from_utf8_lossy(name)
            .trim_start_matches("refs/tags/")
            .to_string();

        // Try to peel to find the target commit
        let (target_oid, is_annotated, message) = if let Ok(obj) = repo.find_object(oid, None) {
            if let Ok(tag) = obj.peel_to_tag() {
                let msg = tag.message().map(|m| m.to_string());
                let target = tag.target_id().to_string();
                (target, true, msg)
            } else {
                (oid.to_string(), false, None)
            }
        } else {
            (oid.to_string(), false, None)
        };

        tags.push(Tag {
            name,
            target_oid,
            is_annotated,
            message,
        });

        true // continue iteration
    })?;

    Ok(tags)
}
