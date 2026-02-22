use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use gitron_core::credential::CredentialStore;

/// File-based credential storage for the web server.
/// Stores credentials in `~/.config/gitron/credentials.json`.
pub struct FileCredentialStore {
    path: PathBuf,
    cache: RwLock<HashMap<String, String>>,
}

impl FileCredentialStore {
    pub fn new() -> Self {
        let config_dir = dirs_path();
        let path = config_dir.join("credentials.json");

        // Load existing credentials
        let cache = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => {
                    serde_json::from_str(&contents).unwrap_or_default()
                }
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        Self {
            path,
            cache: RwLock::new(cache),
        }
    }

    fn persist(&self) -> Result<(), String> {
        let cache = self.cache.read().map_err(|e| format!("Lock error: {e}"))?;
        let dir = self.path.parent().unwrap();
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
        let json = serde_json::to_string_pretty(&*cache)
            .map_err(|e| format!("Serialize error: {e}"))?;
        fs::write(&self.path, json).map_err(|e| format!("Write error: {e}"))?;
        Ok(())
    }
}

impl CredentialStore for FileCredentialStore {
    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        {
            let mut cache = self.cache.write().map_err(|e| format!("Lock error: {e}"))?;
            cache.insert(key.to_string(), value.to_string());
        }
        self.persist()
    }

    fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().ok()?;
        cache.get(key).cloned()
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        {
            let mut cache = self.cache.write().map_err(|e| format!("Lock error: {e}"))?;
            cache.remove(key);
        }
        self.persist()
    }
}

pub fn config_dir() -> PathBuf {
    dirs_path()
}

fn dirs_path() -> PathBuf {
    if let Some(config) = dirs_config_dir() {
        config.join("gitron")
    } else {
        // Fallback to home dir
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("gitron")
    }
}

fn dirs_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}
