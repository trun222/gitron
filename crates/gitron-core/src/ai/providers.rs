use serde::Deserialize;

use super::credential;
use super::error::{AIError, AIResult};
use super::types::{AIModel, AIProvider};

pub struct ProviderDef {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub fallback_models: &'static [(&'static str, &'static str)],
}

const PROVIDERS: &[ProviderDef] = &[
    ProviderDef {
        id: "openai",
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        fallback_models: &[
            ("gpt-4.1-nano", "GPT-4.1 Nano"),
            ("gpt-4.1-mini", "GPT-4.1 Mini"),
            ("gpt-4o-mini", "GPT-4o Mini"),
        ],
    },
    ProviderDef {
        id: "anthropic",
        name: "Anthropic",
        base_url: "https://api.anthropic.com/v1",
        fallback_models: &[
            ("claude-haiku-4-5-20251001", "Claude 4.5 Haiku"),
        ],
    },
    ProviderDef {
        id: "gemini",
        name: "Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        fallback_models: &[
            ("gemini-2.0-flash-lite", "Gemini 2.0 Flash Lite"),
            ("gemini-2.0-flash", "Gemini 2.0 Flash"),
        ],
    },
    ProviderDef {
        id: "openrouter",
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        fallback_models: &[
            ("openrouter/auto", "Auto (best available)"),
        ],
    },
];

/// Get all providers with `has_key` flags and fallback models.
pub fn get_providers() -> Vec<AIProvider> {
    PROVIDERS
        .iter()
        .map(|def| AIProvider {
            id: def.id.to_string(),
            name: def.name.to_string(),
            has_key: credential::has_key(def.id),
            models: def
                .fallback_models
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

/// Fetch available models from a provider's API.
/// Errors propagate to the frontend, which keeps existing models on failure.
pub async fn fetch_models(
    provider_id: &str,
    base_url: Option<&str>,
) -> AIResult<Vec<AIModel>> {
    let api_key = credential::get_key(provider_id)
        .ok_or_else(|| AIError::NoApiKey(provider_id.to_string()))?;

    let default_url = default_base_url(provider_id).unwrap_or("");
    let effective_url = base_url.unwrap_or(default_url);

    match provider_id {
        "openai" => fetch_openai_models(effective_url, &api_key).await,
        "anthropic" => fetch_anthropic_models(effective_url, &api_key).await,
        "gemini" => fetch_gemini_models(effective_url, &api_key).await,
        "openrouter" => fetch_openrouter_models(effective_url).await,
        _ => Err(AIError::ApiError(format!("Unknown provider: {}", provider_id))),
    }
}

// --- OpenAI: GET /v1/models ---

/// Fetch models from OpenAI. Filter to chat-capable models, sorted cheapest-first.
async fn fetch_openai_models(base_url: &str, api_key: &str) -> AIResult<Vec<AIModel>> {
    #[derive(Deserialize)]
    struct Model {
        id: String,
    }
    #[derive(Deserialize)]
    struct Response {
        data: Vec<Model>,
    }

    let client = reqwest::Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("OpenAI {}: {}", status, text)));
    }

    let data: Response = resp.json().await?;

    // Non-chat model families to exclude
    let exclude_keywords = [
        "instruct", "audio", "realtime", "search", "embedding", "tts",
        "whisper", "dall-e", "davinci", "babbage", "moderation", "omni-moderation",
    ];

    let mut models: Vec<AIModel> = data
        .data
        .into_iter()
        .filter(|m| {
            let id = m.id.to_lowercase();
            is_openai_chat_model(&id) && !exclude_keywords.iter().any(|kw| id.contains(kw))
        })
        .map(|m| AIModel {
            name: format_openai_name(&m.id),
            id: m.id,
        })
        .collect();

    // Sort: nano < mini < regular, then alphabetically
    models.sort_by(|a, b| model_cost_tier(&a.id).cmp(&model_cost_tier(&b.id)).then(a.id.cmp(&b.id)));
    Ok(models)
}

/// Check if an OpenAI model ID is a chat-capable model.
fn is_openai_chat_model(id: &str) -> bool {
    // Fine-tuned models
    if id.starts_with("ft:") {
        return false;
    }
    // GPT family
    if id.starts_with("gpt-") {
        return true;
    }
    // ChatGPT models
    if id.starts_with("chatgpt-") {
        return true;
    }
    // o-series reasoning models: o1, o1-mini, o1-pro, o3, o3-mini, o4-mini, etc.
    if id.starts_with('o') {
        let rest = &id[1..];
        // Must start with a digit after 'o' (o1, o3, o4, ...)
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

fn model_cost_tier(id: &str) -> u8 {
    if id.contains("nano") {
        0
    } else if id.contains("mini") {
        1
    } else {
        2
    }
}

fn format_openai_name(id: &str) -> String {
    // gpt-4.1-nano -> GPT-4.1 Nano, o3-mini -> O3 Mini
    id.split('-')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_uppercase()
            } else {
                let mut c = part.chars();
                match c.next() {
                    None => String::new(),
                    Some(first) => {
                        if part.chars().all(|c| c.is_ascii_digit() || c == '.') {
                            part.to_string()
                        } else {
                            format!("{}{}", first.to_uppercase(), c.as_str())
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// --- Anthropic: GET /v1/models ---

async fn fetch_anthropic_models(base_url: &str, api_key: &str) -> AIResult<Vec<AIModel>> {
    #[derive(Deserialize)]
    struct Model {
        id: String,
        display_name: String,
    }
    #[derive(Deserialize)]
    struct Response {
        data: Vec<Model>,
    }

    let client = reqwest::Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("Anthropic {}: {}", status, text)));
    }

    let resp: Response = resp.json().await?;

    let mut models: Vec<AIModel> = resp
        .data
        .into_iter()
        .map(|m| AIModel {
            name: m.display_name,
            id: m.id,
        })
        .collect();

    // Sort: haiku < sonnet < opus
    models.sort_by(|a, b| anthropic_tier(&a.id).cmp(&anthropic_tier(&b.id)));
    Ok(models)
}

fn anthropic_tier(id: &str) -> u8 {
    if id.contains("haiku") {
        0
    } else if id.contains("sonnet") {
        1
    } else if id.contains("opus") {
        2
    } else {
        3
    }
}

// --- Gemini: GET /v1beta/models ---

async fn fetch_gemini_models(base_url: &str, api_key: &str) -> AIResult<Vec<AIModel>> {
    #[derive(Deserialize)]
    struct Model {
        name: String,
        #[serde(rename = "displayName")]
        display_name: String,
        #[serde(rename = "supportedGenerationMethods", default)]
        supported_methods: Vec<String>,
    }
    #[derive(Deserialize)]
    struct Response {
        models: Vec<Model>,
        #[serde(rename = "nextPageToken")]
        next_page_token: Option<String>,
    }

    let client = reqwest::Client::new();
    let base = base_url.trim_end_matches('/');
    let mut all_models: Vec<Model> = Vec::new();
    let mut page_token: Option<String> = None;

    // Paginate through all model pages
    loop {
        let mut url = format!("{}/models?key={}&pageSize=100", base, api_key);
        if let Some(token) = &page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!("Gemini {}: {}", status, text)));
        }

        let page: Response = resp.json().await?;
        all_models.extend(page.models);

        match page.next_page_token {
            Some(token) if !token.is_empty() => page_token = Some(token),
            _ => break,
        }
    }

    let mut models: Vec<AIModel> = all_models
        .into_iter()
        .filter(|m| {
            // Only models that support generateContent (text generation)
            m.supported_methods.iter().any(|s| s == "generateContent")
                // Skip embedding, AQA, and image/video-only models
                && !m.name.contains("embedding")
                && !m.name.contains("aqa")
                && !m.name.contains("imagen")
                && !m.name.contains("veo")
        })
        .map(|m| {
            // "models/gemini-2.0-flash" -> "gemini-2.0-flash"
            let id = m.name.strip_prefix("models/").unwrap_or(&m.name).to_string();
            AIModel {
                name: m.display_name,
                id,
            }
        })
        .collect();

    // Sort: lite < flash < pro < ultra
    models.sort_by(|a, b| gemini_tier(&a.id).cmp(&gemini_tier(&b.id)));
    Ok(models)
}

fn gemini_tier(id: &str) -> u8 {
    if id.contains("lite") {
        0
    } else if id.contains("flash") {
        1
    } else if id.contains("pro") {
        2
    } else if id.contains("ultra") {
        3
    } else {
        4
    }
}

// --- OpenRouter: GET /api/v1/models (no auth needed) ---

/// Max input price in $/token. $2/M tokens = 0.000002/token.
const OPENROUTER_MAX_PROMPT_PRICE: f64 = 0.000002;

async fn fetch_openrouter_models(base_url: &str) -> AIResult<Vec<AIModel>> {
    #[derive(Deserialize)]
    struct Pricing {
        prompt: String,
    }
    #[derive(Deserialize)]
    struct Model {
        id: String,
        name: String,
        pricing: Option<Pricing>,
    }
    #[derive(Deserialize)]
    struct Response {
        data: Vec<Model>,
    }

    let client = reqwest::Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AIError::ApiError(format!("OpenRouter {}: {}", status, text)));
    }

    let resp: Response = resp.json().await?;

    let mut models: Vec<AIModel> = resp
        .data
        .into_iter()
        .filter(|m| {
            let price = m
                .pricing
                .as_ref()
                .and_then(|p| p.prompt.parse::<f64>().ok())
                .unwrap_or(f64::MAX);
            price <= OPENROUTER_MAX_PROMPT_PRICE && price >= 0.0
        })
        .map(|m| AIModel {
            name: m.name,
            id: m.id,
        })
        .collect();

    // Sort alphabetically by name
    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models)
}
