use super::credential;
use super::error::{GitHubError, GitHubResult};

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
