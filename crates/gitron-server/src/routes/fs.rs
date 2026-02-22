use std::path::{Path, PathBuf};

use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ListRequest {
    path: Option<String>,
}

#[derive(Serialize)]
pub struct DirEntry {
    name: String,
    path: String,
    #[serde(rename = "isDir")]
    is_dir: bool,
    #[serde(rename = "isGitRepo")]
    is_git_repo: bool,
}

pub async fn list_directory(
    Json(req): Json<ListRequest>,
) -> Result<Json<Vec<DirEntry>>, (StatusCode, String)> {
    let dir = match req.path {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => home_dir(),
    };

    if !dir.exists() || !dir.is_dir() {
        return Err((StatusCode::BAD_REQUEST, format!("Not a directory: {}", dir.display())));
    }

    let mut entries = Vec::new();

    // Add parent entry (unless we're at root)
    if let Some(parent) = dir.parent() {
        entries.push(DirEntry {
            name: "..".to_string(),
            path: parent.to_string_lossy().to_string(),
            is_dir: true,
            is_git_repo: false,
        });
    }

    let mut dir_entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(err)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            // Skip hidden files except ..
            !name.starts_with('.')
        })
        .collect();

    // Sort: directories first, then by name
    dir_entries.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for entry in dir_entries {
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue; // Only show directories in the browser
        }

        let path = entry.path();
        let is_git_repo = is_git_repository(&path);

        entries.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            is_dir,
            is_git_repo,
        });
    }

    Ok(Json(entries))
}

fn is_git_repository(path: &Path) -> bool {
    path.join(".git").exists()
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
