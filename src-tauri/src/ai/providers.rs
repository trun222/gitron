use super::credential;
use super::types::{AIModel, AIProvider};

pub struct ProviderDef {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub models: &'static [(&'static str, &'static str)],
}

const PROVIDERS: &[ProviderDef] = &[
    ProviderDef {
        id: "openai",
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        models: &[
            ("gpt-4.1-nano", "GPT-4.1 Nano"),
            ("gpt-4.1-mini", "GPT-4.1 Mini"),
            ("gpt-4o-mini", "GPT-4o Mini"),
        ],
    },
    ProviderDef {
        id: "anthropic",
        name: "Anthropic",
        base_url: "https://api.anthropic.com/v1",
        models: &[
            ("claude-haiku-4-5-20251001", "Claude 4.5 Haiku"),
        ],
    },
    ProviderDef {
        id: "gemini",
        name: "Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        models: &[
            ("gemini-2.0-flash-lite", "Gemini 2.0 Flash Lite"),
            ("gemini-2.0-flash", "Gemini 2.0 Flash"),
        ],
    },
    ProviderDef {
        id: "openrouter",
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        models: &[
            ("openrouter/auto", "Auto (best available)"),
        ],
    },
];

/// Get all providers with `has_key` flags from the keychain.
pub fn get_providers() -> Vec<AIProvider> {
    PROVIDERS
        .iter()
        .map(|def| AIProvider {
            id: def.id.to_string(),
            name: def.name.to_string(),
            has_key: credential::has_key(def.id),
            models: def
                .models
                .iter()
                .map(|(id, name)| AIModel {
                    id: id.to_string(),
                    name: name.to_string(),
                })
                .collect(),
            base_url: Some(def.base_url.to_string()),
        })
        .collect()
}

/// Get the default base URL for a provider.
pub fn default_base_url(provider_id: &str) -> Option<&'static str> {
    PROVIDERS
        .iter()
        .find(|p| p.id == provider_id)
        .map(|p| p.base_url)
}
