use super::error::{GitHubError, GitHubResult};

const KEY: &str = "github-oauth-token";

/// Store a GitHub OAuth token in the app credential store.
pub fn store_token(token: &str) -> GitHubResult<()> {
    crate::credential::set(KEY, token)
        .map_err(|e| GitHubError::Keychain(e))
}

/// Retrieve the stored GitHub OAuth token, or None if not found.
pub fn get_token() -> Option<String> {
    crate::credential::get(KEY)
}

/// Delete the stored GitHub OAuth token.
pub fn delete_token() -> GitHubResult<()> {
    crate::credential::delete(KEY)
        .map_err(|e| GitHubError::Keychain(e))
}
