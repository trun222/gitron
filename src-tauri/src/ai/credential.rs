use super::error::{AIError, AIResult};

fn key_for(provider: &str) -> String {
    format!("ai-{}", provider)
}

/// Store an API key for a provider in the app credential store.
pub fn store_key(provider: &str, key: &str) -> AIResult<()> {
    crate::credential_store::set(&key_for(provider), key)
        .map_err(|e| AIError::Keychain(e))
}

/// Retrieve the stored API key for a provider, or None if not found.
pub fn get_key(provider: &str) -> Option<String> {
    crate::credential_store::get(&key_for(provider))
}

/// Delete the stored API key for a provider.
pub fn delete_key(provider: &str) -> AIResult<()> {
    crate::credential_store::delete(&key_for(provider))
        .map_err(|e| AIError::Keychain(e))
}

/// Check whether an API key exists for a provider.
pub fn has_key(provider: &str) -> bool {
    crate::credential_store::has(&key_for(provider))
}
