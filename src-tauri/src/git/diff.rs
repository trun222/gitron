use git2::{DiffOptions, Repository};

use super::error::GitResult;
use super::types::*;

/// Get diff of working directory changes (unstaged)
pub fn diff_workdir(repo: &Repository) -> GitResult<Vec<FileDiff>> {
    let mut diff_opts = DiffOptions::new();
    diff_opts.include_untracked(true);
    diff_opts.show_untracked_content(true);

    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;
    parse_diff(&diff)
}

/// Get diff of staged changes (index vs HEAD)
pub fn diff_staged(repo: &Repository) -> GitResult<Vec<FileDiff>> {
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let diff = repo.diff_tree_to_index(
        head_tree.as_ref(),
        None,
        None,
    )?;
    parse_diff(&diff)
}

/// Get diff for a specific file (workdir changes)
pub fn diff_file(repo: &Repository, path: &str) -> GitResult<FileDiff> {
    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(path);
    diff_opts.include_untracked(true);
    diff_opts.show_untracked_content(true);

    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;
    let files = parse_diff(&diff)?;

    files
        .into_iter()
        .next()
        .ok_or_else(|| super::error::GitError::Other(format!("No diff for file: {}", path)))
}

/// Get diff for a specific staged file (index vs HEAD)
pub fn diff_file_staged(repo: &Repository, path: &str) -> GitResult<FileDiff> {
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(path);

    let diff = repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_opts))?;
    let files = parse_diff(&diff)?;

    files
        .into_iter()
        .next()
        .ok_or_else(|| super::error::GitError::Other(format!("No staged diff for file: {}", path)))
}

/// Parse a git2 Diff into our FileDiff types
fn parse_diff(diff: &git2::Diff) -> GitResult<Vec<FileDiff>> {
    let mut file_diffs = Vec::new();

    let mut current_file: Option<FileDiff> = None;
    let mut current_hunks: Vec<DiffHunk> = Vec::new();
    let mut current_lines: Vec<DiffLine> = Vec::new();
    let mut current_hunk_header = String::new();
    let mut current_old_start = 0u32;
    let mut current_old_lines = 0u32;
    let mut current_new_start = 0u32;
    let mut current_new_lines = 0u32;

    diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        let file_path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let old_path = delta
            .old_file()
            .path()
            .map(|p| p.to_string_lossy().to_string());

        // Detect file boundary
        let need_new_file = current_file
            .as_ref()
            .map(|f| f.path != file_path)
            .unwrap_or(true);

        if need_new_file {
            // Save previous hunk
            if !current_lines.is_empty() {
                current_hunks.push(DiffHunk {
                    header: current_hunk_header.clone(),
                    old_start: current_old_start,
                    old_lines: current_old_lines,
                    new_start: current_new_start,
                    new_lines: current_new_lines,
                    lines: std::mem::take(&mut current_lines),
                });
            }

            // Save previous file
            if let Some(mut file) = current_file.take() {
                file.hunks = std::mem::take(&mut current_hunks);
                file_diffs.push(file);
            }

            let status = match delta.status() {
                git2::Delta::Added => FileStatusType::Added,
                git2::Delta::Deleted => FileStatusType::Deleted,
                git2::Delta::Modified => FileStatusType::Modified,
                git2::Delta::Renamed => FileStatusType::Renamed,
                git2::Delta::Copied => FileStatusType::Copied,
                git2::Delta::Typechange => FileStatusType::TypeChanged,
                git2::Delta::Untracked => FileStatusType::Added,
                _ => FileStatusType::Modified,
            };

            current_file = Some(FileDiff {
                path: file_path.clone(),
                old_path,
                hunks: Vec::new(),
                is_binary: delta.flags().is_binary(),
                status,
            });
        }

        // Process hunk header — git2 sends the hunk ref with every line,
        // so only start a new hunk when the header actually changes.
        if let Some(h) = hunk {
            let header = String::from_utf8_lossy(h.header()).to_string();
            if header != current_hunk_header {
                // Save previous hunk
                if !current_lines.is_empty() {
                    current_hunks.push(DiffHunk {
                        header: current_hunk_header.clone(),
                        old_start: current_old_start,
                        old_lines: current_old_lines,
                        new_start: current_new_start,
                        new_lines: current_new_lines,
                        lines: std::mem::take(&mut current_lines),
                    });
                }

                current_hunk_header = header;
                current_old_start = h.old_start();
                current_old_lines = h.old_lines();
                current_new_start = h.new_start();
                current_new_lines = h.new_lines();
            }
        }

        // Process line — skip file header lines (---, +++, etc.)
        let origin = match line.origin() {
            '+' => DiffLineType::Addition,
            '-' => DiffLineType::Deletion,
            ' ' => DiffLineType::Context,
            _ => return true,
        };

        let content = String::from_utf8_lossy(line.content()).to_string();

        current_lines.push(DiffLine {
            origin,
            content,
            old_lineno: line.old_lineno(),
            new_lineno: line.new_lineno(),
        });

        true
    })?;

    // Flush remaining hunk and file
    if !current_lines.is_empty() {
        current_hunks.push(DiffHunk {
            header: current_hunk_header,
            old_start: current_old_start,
            old_lines: current_old_lines,
            new_start: current_new_start,
            new_lines: current_new_lines,
            lines: current_lines,
        });
    }

    if let Some(mut file) = current_file.take() {
        file.hunks = current_hunks;
        file_diffs.push(file);
    }

    Ok(file_diffs)
}
