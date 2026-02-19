use super::error::{GitHubError, GitHubResult};

const SERVICE: &str = "com.thomasunderwoodii.gitron";
const ACCOUNT: &str = "github-oauth-token";

/// Store a GitHub OAuth token in the OS keychain.
pub fn store_token(token: &str) -> GitHubResult<()> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| GitHubError::Keychain(e.to_string()))?;
    entry
        .set_password(token)
        .map_err(|e| GitHubError::Keychain(e.to_string()))
}

/// Retrieve the stored GitHub OAuth token, or None if not found.
pub fn get_token() -> Option<String> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT).ok()?;
    entry.get_password().ok()
}

/// Delete the stored GitHub OAuth token.
pub fn delete_token() -> GitHubResult<()> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| GitHubError::Keychain(e.to_string()))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // already gone
        Err(e) => Err(GitHubError::Keychain(e.to_string())),
    }
}
