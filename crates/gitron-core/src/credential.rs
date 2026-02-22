use std::sync::OnceLock;
use std::sync::Arc;

/// Trait for key-value credential storage.
/// Implementations can use OS keychain, file-based storage, etc.
pub trait CredentialStore: Send + Sync + 'static {
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
    fn get(&self, key: &str) -> Option<String>;
    fn delete(&self, key: &str) -> Result<(), String>;
    fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

static CREDENTIAL_STORE: OnceLock<Arc<dyn CredentialStore>> = OnceLock::new();

/// Initialize the global credential store. Must be called once at startup.
pub fn init(store: Arc<dyn CredentialStore>) {
    CREDENTIAL_STORE.set(store).ok();
}

/// Store a credential value.
pub fn set(key: &str, value: &str) -> Result<(), String> {
    let store = CREDENTIAL_STORE
        .get()
        .ok_or_else(|| "Credential store not initialized".to_string())?;
    store.set(key, value)
}

/// Retrieve a credential value, or None if not found.
pub fn get(key: &str) -> Option<String> {
    let store = CREDENTIAL_STORE.get()?;
    store.get(key)
}

/// Delete a credential value.
pub fn delete(key: &str) -> Result<(), String> {
    let store = CREDENTIAL_STORE
        .get()
        .ok_or_else(|| "Credential store not initialized".to_string())?;
    store.delete(key)
}

/// Check whether a credential key exists.
pub fn has(key: &str) -> bool {
    get(key).is_some()
}
