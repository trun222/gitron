use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvider {
    pub id: String,
    pub name: String,
    pub has_key: bool,
    pub models: Vec<AIModel>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    /// Remembers the last selected model per provider so switching back restores it.
    #[serde(default)]
    pub selected_models: HashMap<String, String>,
    pub custom_base_urls: HashMap<String, String>,
    /// Max output tokens for AI generation. Default: 1500.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    1500
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            selected_provider: None,
            selected_model: None,
            selected_models: HashMap::new(),
            custom_base_urls: HashMap::new(),
            max_tokens: default_max_tokens(),
        }
    }
}
