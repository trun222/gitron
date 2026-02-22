use std::sync::OnceLock;

use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_store::StoreExt;

use gitron_core::credential::CredentialStore;
use gitron_core::event::EventEmitter;
use gitron_core::git::types::{StatusChangedPayload, RefsChangedPayload};

// --- TauriCredentialStore ---

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub struct TauriCredentialStore;

impl TauriCredentialStore {
    /// Initialize with a Tauri AppHandle. Must be called once during app setup.
    pub fn init(handle: AppHandle) {
        APP_HANDLE.set(handle).ok();
    }
}

impl CredentialStore for TauriCredentialStore {
    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let handle = APP_HANDLE
            .get()
            .ok_or_else(|| "Credential store not initialized".to_string())?;
        let store = handle
            .store("credentials.json")
            .map_err(|e| format!("Failed to open credential store: {e}"))?;
        store.set(key, serde_json::json!(value));
        Ok(())
    }

    fn get(&self, key: &str) -> Option<String> {
        let handle = APP_HANDLE.get()?;
        let store = handle.store("credentials.json").ok()?;
        store.get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        let handle = APP_HANDLE
            .get()
            .ok_or_else(|| "Credential store not initialized".to_string())?;
        let store = handle
            .store("credentials.json")
            .map_err(|e| format!("Failed to open credential store: {e}"))?;
        store.delete(key);
        Ok(())
    }
}

// --- TauriEventEmitter ---

pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl EventEmitter for TauriEventEmitter {
    fn emit_status_changed(&self, payload: &StatusChangedPayload) {
        self.app_handle.emit("repo:status-changed", payload).ok();
    }

    fn emit_refs_changed(&self, payload: &RefsChangedPayload) {
        self.app_handle.emit("repo:status-changed", &gitron_core::git::types::StatusChangedPayload {
            status: payload.status.clone(),
        }).ok();
        self.app_handle.emit("repo:refs-changed", payload).ok();
    }
}
