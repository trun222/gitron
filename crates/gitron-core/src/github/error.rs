use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Device code expired — please restart the login flow")]
    DeviceCodeExpired,

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("GitHub API error: {0}")]
    ApiError(String),
}

impl serde::Serialize for GitHubError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type GitHubResult<T> = Result<T, GitHubError>;
