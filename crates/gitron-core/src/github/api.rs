use super::credential;
use super::error::{GitHubError, GitHubResult};
use super::types::GitHubRepo;

/// Get a reqwest client configured with the stored GitHub token (if available).
pub fn authenticated_client() -> GitHubResult<(reqwest::Client, Option<String>)> {
    let token = credential::get_token();
    let client = reqwest::Client::builder()
        .user_agent("Gitron")
        .build()
        .map_err(GitHubError::Http)?;
    Ok((client, token))
}

/// Generic authenticated GET to the GitHub API.
pub async fn get(path: &str) -> GitHubResult<reqwest::Response> {
    let (client, token) = authenticated_client()?;
    let url = format!("https://api.github.com{path}");

    let mut req = client
        .get(&url)
        .header("Accept", "application/vnd.github+json");

    if let Some(token) = token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp = req.send().await?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GitHubError::TokenExpired);
    }

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(GitHubError::ApiError(format!(
            "{} {path}: {text}",
            status.as_u16()
        )));
    }

    Ok(resp)
}

/// List repositories for the authenticated user, paginating until all repos are fetched.
pub async fn list_user_repos() -> GitHubResult<Vec<GitHubRepo>> {
    let mut all_repos = Vec::new();
    let mut page = 1u32;

    loop {
        let resp = get(&format!(
            "/user/repos?per_page=100&sort=pushed&direction=desc&type=all&page={page}"
        ))
        .await?;

        let repos: Vec<GitHubRepo> = resp.json().await.map_err(GitHubError::Http)?;
        let count = repos.len();
        all_repos.extend(repos);

        if count < 100 {
            break;
        }
        page += 1;
    }

    Ok(all_repos)
}
