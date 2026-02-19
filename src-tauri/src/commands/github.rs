use crate::github::auth;
use crate::github::{api, error::GitHubResult};
use crate::github::types::*;

#[tauri::command]
pub async fn github_check_auth() -> GitHubResult<GitHubAuthInfo> {
    auth::check_auth_status().await
}

#[tauri::command]
pub async fn github_start_device_flow() -> GitHubResult<DeviceCodeResponse> {
    auth::request_device_code().await
}

#[tauri::command]
pub async fn github_poll_device_flow(
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> GitHubResult<GitHubAuthInfo> {
    auth::poll_for_token(&device_code, interval, expires_in).await
}

#[tauri::command]
pub async fn github_logout() -> GitHubResult<()> {
    auth::logout()
}

#[tauri::command]
pub async fn github_get_user() -> GitHubResult<Option<GitHubUser>> {
    let token = crate::github::credential::get_token();
    match token {
        Some(t) => {
            let user = auth::get_authenticated_user(&t).await?;
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn github_list_repos() -> GitHubResult<Vec<GitHubRepo>> {
    api::list_user_repos().await
}
