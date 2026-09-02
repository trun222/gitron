use gitron_core::git::{error::GitError, graph as git_graph, range, repository, types::*};

/// Get the commit graph for a repository
#[tauri::command]
pub fn get_commit_graph(
    path: String,
    max_commits: Option<usize>,
    include_remotes: Option<bool>,
    excluded_authors: Option<Vec<String>>,
) -> Result<CommitGraph, GitError> {
    let repo = repository::open(&path)?;
    let options = GraphOptions {
        max_commits: max_commits.or(Some(500)),
        from_oid: None,
        include_remotes: include_remotes.unwrap_or(true),
        excluded_authors: excluded_authors.unwrap_or_default(),
    };
    git_graph::build_commit_graph(&repo, &options)
}

/// Search commits by message, author, and optionally file paths + diff content
#[tauri::command]
pub fn search_commits(
    path: String,
    query: String,
    max_commits: Option<usize>,
    include_remotes: Option<bool>,
    search_diffs: Option<bool>,
) -> Result<Vec<String>, GitError> {
    let repo = repository::open(&path)?;
    let options = GraphOptions {
        max_commits: max_commits.or(Some(500)),
        from_oid: None,
        include_remotes: include_remotes.unwrap_or(true),
        excluded_authors: Vec::new(),
    };
    git_graph::search_commits(&repo, &query, &options, search_diffs.unwrap_or(false))
}

/// Get detailed information about a specific commit
#[tauri::command]
pub fn get_commit_detail(path: String, oid: String) -> Result<Commit, GitError> {
    let repo = repository::open(&path)?;
    git_graph::get_commit_detail(&repo, &oid)
}

/// Summarize the commits and diffstat between two revisions (`from` exclusive, `to` inclusive)
#[tauri::command]
pub fn get_commit_range(path: String, from: String, to: String) -> Result<CommitRangeSummary, GitError> {
    let repo = repository::open(&path)?;
    range::summarize_range(&repo, &from, &to)
}
