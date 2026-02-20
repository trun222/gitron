use super::error::{AIError, AIResult};

const SERVICE: &str = "com.thomasunderwoodii.gitron";

fn account_for(provider: &str) -> String {
    format!("ai-{}", provider)
}

/// Store an API key for a provider in the OS keychain.
pub fn store_key(provider: &str, key: &str) -> AIResult<()> {
    let account = account_for(provider);
    let entry = keyring::Entry::new(SERVICE, &account)
        .map_err(|e| AIError::Keychain(e.to_string()))?;
    entry
        .set_password(key)
        .map_err(|e| AIError::Keychain(e.to_string()))
}

/// Retrieve the stored API key for a provider, or None if not found.
pub fn get_key(provider: &str) -> Option<String> {
    let account = account_for(provider);
    let entry = keyring::Entry::new(SERVICE, &account).ok()?;
    entry.get_password().ok()
}

/// Delete the stored API key for a provider.
pub fn delete_key(provider: &str) -> AIResult<()> {
    let account = account_for(provider);
    let entry = keyring::Entry::new(SERVICE, &account)
        .map_err(|e| AIError::Keychain(e.to_string()))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // already gone
        Err(e) => Err(AIError::Keychain(e.to_string())),
    }
}

/// Check whether an API key exists for a provider.
pub fn has_key(provider: &str) -> bool {
    get_key(provider).is_some()
}
