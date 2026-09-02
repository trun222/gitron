use thiserror::Error;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("No API key configured for provider: {0}")]
    NoApiKey(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("No staged files to generate a commit message from")]
    NoStagedFiles,

    #[error("No commits found between {0} and {1}")]
    NoCommitsInRange(String, String),

    #[error("Git error: {0}")]
    Git(#[from] crate::git::error::GitError),
}

impl serde::Serialize for AIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AIResult<T> = Result<T, AIError>;
