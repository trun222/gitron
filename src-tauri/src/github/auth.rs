use super::credential;
use super::error::{GitHubError, GitHubResult};
use super::types::*;

/// GitHub OAuth App client ID (public — no secret needed for Device Flow)
const CLIENT_ID: &str = "Ov23liwNcg1lTApCIg0j";

/// Scopes requested during authorization
const SCOPE: &str = "repo read:user read:org";

/// Request a device code from GitHub to begin the Device Flow.
pub async fn request_device_code() -> GitHubResult<DeviceCodeResponse> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPE)])
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(GitHubError::ApiError(format!(
            "Device code request failed: {text}"
        )));
    }

    let body: DeviceCodeResponse = resp.json().await?;
    Ok(body)
}

/// Poll GitHub until the user authorizes or the code expires.
/// On success, stores the token in the OS keychain and returns auth info.
pub async fn poll_for_token(
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> GitHubResult<GitHubAuthInfo> {
    let client = reqwest::Client::new();
    let mut poll_interval = std::time::Duration::from_secs(interval.max(5));
    let deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(expires_in);

    loop {
        tokio::time::sleep(poll_interval).await;

        if tokio::time::Instant::now() >= deadline {
            return Err(GitHubError::DeviceCodeExpired);
        }

        let resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        let token_resp: TokenResponse = resp.json().await?;

        if let Some(token) = token_resp.access_token {
            // Store in keychain
            credential::store_token(&token)?;
            // Fetch user profile
            let user = get_authenticated_user(&token).await?;
            return Ok(GitHubAuthInfo {
                status: GitHubAuthStatus::Authenticated { user },
            });
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => {
                // User hasn't entered code yet — keep polling
            }
            Some("slow_down") => {
                // Back off by 5 seconds
                if let Some(new_interval) = token_resp.interval {
                    poll_interval = std::time::Duration::from_secs(new_interval);
                } else {
                    poll_interval += std::time::Duration::from_secs(5);
                }
            }
            Some("expired_token") => {
                return Err(GitHubError::DeviceCodeExpired);
            }
            Some("access_denied") => {
                return Err(GitHubError::AuthFailed(
                    "User denied authorization".to_string(),
                ));
            }
            Some(err) => {
                let desc = token_resp.error_description.unwrap_or_default();
                return Err(GitHubError::AuthFailed(format!("{err}: {desc}")));
            }
            None => {
                return Err(GitHubError::AuthFailed(
                    "Unexpected empty response from GitHub".to_string(),
                ));
            }
        }
    }
}

/// Fetch the authenticated user's profile from the GitHub API.
pub async fn get_authenticated_user(token: &str) -> GitHubResult<GitHubUser> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", "Gitron")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GitHubError::TokenExpired);
    }

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(GitHubError::ApiError(format!("GET /user failed: {text}")));
    }

    let user: GitHubUser = resp.json().await?;
    Ok(user)
}

/// Check current authentication status by validating the stored token.
pub async fn check_auth_status() -> GitHubResult<GitHubAuthInfo> {
    let token = match credential::get_token() {
        Some(t) => t,
        None => {
            return Ok(GitHubAuthInfo {
                status: GitHubAuthStatus::NotAuthenticated,
            });
        }
    };

    match get_authenticated_user(&token).await {
        Ok(user) => Ok(GitHubAuthInfo {
            status: GitHubAuthStatus::Authenticated { user },
        }),
        Err(GitHubError::TokenExpired) => {
            credential::delete_token()?;
            Ok(GitHubAuthInfo {
                status: GitHubAuthStatus::TokenExpired,
            })
        }
        Err(e) => Err(e),
    }
}

/// Log out by deleting the stored token.
pub fn logout() -> GitHubResult<()> {
    credential::delete_token()
}
