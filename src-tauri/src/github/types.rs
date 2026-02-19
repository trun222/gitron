use serde::{Deserialize, Serialize};

/// GitHub user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}

/// Authentication status discriminated union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GitHubAuthStatus {
    NotAuthenticated,
    AwaitingUserCode,
    Authenticated { user: GitHubUser },
    TokenExpired,
    Failed { message: String },
}

/// Wrapper returned by auth check commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAuthInfo {
    pub status: GitHubAuthStatus,
}

/// Response from GitHub's device code endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Response from GitHub's token endpoint (internal use)
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub interval: Option<u64>,
}

/// A GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
    pub clone_url: String,
    pub updated_at: String,
    pub owner: GitHubRepoOwner,
}

/// Owner of a GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepoOwner {
    pub login: String,
    pub avatar_url: String,
}
