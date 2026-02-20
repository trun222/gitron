use std::sync::OnceLock;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Initialize the credential store with the Tauri app handle.
/// Must be called once during app setup.
pub fn init(handle: AppHandle) {
    APP_HANDLE.set(handle).ok();
}

fn store() -> Result<impl std::ops::Deref<Target = tauri_plugin_store::Store<tauri::Wry>>, String> {
    let handle = APP_HANDLE
        .get()
        .ok_or_else(|| "Credential store not initialized".to_string())?;
    handle
        .store("credentials.json")
        .map_err(|e| format!("Failed to open credential store: {e}"))
}

/// Store a credential value.
pub fn set(key: &str, value: &str) -> Result<(), String> {
    let s = store()?;
    s.set(key, serde_json::json!(value));
    Ok(())
}

/// Retrieve a credential value, or None if not found.
pub fn get(key: &str) -> Option<String> {
    let s = store().ok()?;
    s.get(key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

/// Delete a credential value.
pub fn delete(key: &str) -> Result<(), String> {
    let s = store()?;
    s.delete(key);
    Ok(())
}

/// Check whether a credential key exists.
pub fn has(key: &str) -> bool {
    get(key).is_some()
}
