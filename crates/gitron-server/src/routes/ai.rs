use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::ai::{credential, generate, providers};
use gitron_core::ai::types::{AIModel, AIProvider, AISettings, GenerateResult};

pub async fn get_providers() -> Result<Json<Vec<AIProvider>>, (StatusCode, String)> {
    Ok(Json(providers::get_providers()))
}

#[derive(Deserialize)]
pub struct SaveKeyRequest {
    provider: String,
    key: String,
}

pub async fn save_key(
    Json(req): Json<SaveKeyRequest>,
) -> Result<Json<()>, (StatusCode, String)> {
    credential::store_key(&req.provider, &req.key).map_err(err)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct DeleteKeyRequest {
    provider: String,
}

pub async fn delete_key(
    Json(req): Json<DeleteKeyRequest>,
) -> Result<Json<()>, (StatusCode, String)> {
    credential::delete_key(&req.provider).map_err(err)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct FetchModelsRequest {
    provider: String,
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
}

pub async fn fetch_models(
    Json(req): Json<FetchModelsRequest>,
) -> Result<Json<Vec<AIModel>>, (StatusCode, String)> {
    let models = providers::fetch_models(&req.provider, req.base_url.as_deref())
        .await
        .map_err(err)?;
    Ok(Json(models))
}

#[derive(Deserialize)]
pub struct GenerateRequest {
    path: String,
    provider: String,
    model: String,
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    #[serde(rename = "maxTokens")]
    max_tokens: Option<u32>,
}

pub async fn generate_commit_message(
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResult>, (StatusCode, String)> {
    let result = generate::generate_commit_message(
        &req.path,
        &req.provider,
        &req.model,
        req.base_url.as_deref(),
        req.max_tokens.unwrap_or(1500),
    )
    .await
    .map_err(err)?;
    Ok(Json(result))
}

pub async fn get_settings() -> Result<Json<AISettings>, (StatusCode, String)> {
    // Server-side AI settings stored in config file
    let config_path = crate::file_store::config_dir().join("ai_settings.json");
    let settings: AISettings = if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path).map_err(err)?;
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        AISettings::default()
    };
    Ok(Json(settings))
}

#[derive(Deserialize)]
pub struct SaveSettingsRequest {
    settings: AISettings,
}

pub async fn save_settings(
    Json(req): Json<SaveSettingsRequest>,
) -> Result<Json<()>, (StatusCode, String)> {
    let settings = req.settings;
    let config_dir = crate::file_store::config_dir();
    std::fs::create_dir_all(&config_dir).map_err(err)?;
    let config_path = config_dir.join("ai_settings.json");
    let json = serde_json::to_string_pretty(&settings).map_err(err)?;
    std::fs::write(&config_path, json).map_err(err)?;
    Ok(Json(()))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
