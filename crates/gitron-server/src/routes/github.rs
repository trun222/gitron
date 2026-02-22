use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::github::{auth, api};
use gitron_core::github::types::*;

pub async fn check_auth() -> Result<Json<GitHubAuthInfo>, (StatusCode, String)> {
    let info = auth::check_auth_status().await.map_err(err)?;
    Ok(Json(info))
}

pub async fn start_device_flow() -> Result<Json<DeviceCodeResponse>, (StatusCode, String)> {
    let code = auth::request_device_code().await.map_err(err)?;
    Ok(Json(code))
}

#[derive(Deserialize)]
pub struct PollFlowRequest {
    #[serde(rename = "deviceCode")]
    device_code: String,
    interval: u64,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
}

pub async fn poll_device_flow(
    Json(req): Json<PollFlowRequest>,
) -> Result<Json<GitHubAuthInfo>, (StatusCode, String)> {
    let info = auth::poll_for_token(&req.device_code, req.interval, req.expires_in)
        .await
        .map_err(err)?;
    Ok(Json(info))
}

pub async fn logout() -> Result<Json<()>, (StatusCode, String)> {
    auth::logout().map_err(err)?;
    Ok(Json(()))
}

pub async fn get_user() -> Result<Json<Option<GitHubUser>>, (StatusCode, String)> {
    let token = gitron_core::github::credential::get_token();
    match token {
        Some(t) => {
            let user = auth::get_authenticated_user(&t).await.map_err(err)?;
            Ok(Json(Some(user)))
        }
        None => Ok(Json(None)),
    }
}

pub async fn list_repos() -> Result<Json<Vec<GitHubRepo>>, (StatusCode, String)> {
    let repos = api::list_user_repos().await.map_err(err)?;
    Ok(Json(repos))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
